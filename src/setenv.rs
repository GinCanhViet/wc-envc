//! Set environment variables permanently from .env file

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Select};

use crate::scanner;

/// Parse .env file content and return list of (key, value) pairs
fn parse_env_file(content: &str) -> Vec<(String, String)> {
    let mut vars = Vec::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        // Parse KEY=VALUE
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..].trim().to_string();
            
            // Remove surrounding quotes if present
            let value = value
                .trim_start_matches('"')
                .trim_end_matches('"')
                .trim_start_matches('\'')
                .trim_end_matches('\'')
                .to_string();
            
            if !key.is_empty() {
                vars.push((key, value));
            }
        }
    }
    
    vars
}

/// Set environment variable permanently (Windows)
#[cfg(target_os = "windows")]
fn set_env_permanent(key: &str, value: &str) -> Result<()> {
    use std::process::Command;
    
    // Use setx command to set user environment variable
    let output = Command::new("setx")
        .arg(key)
        .arg(value)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to set {}: {}", key, stderr);
    }
    
    Ok(())
}

/// Set environment variable permanently (Unix - append to shell config)
#[cfg(not(target_os = "windows"))]
fn set_env_permanent(key: &str, value: &str) -> Result<()> {
    // Determine shell config file
    let home = env::var("HOME")?;
    let shell = env::var("SHELL").unwrap_or_default();
    
    let config_file = if shell.contains("zsh") {
        PathBuf::from(&home).join(".zshrc")
    } else {
        PathBuf::from(&home).join(".bashrc")
    };
    
    // Append export statement
    let export_line = format!("export {}=\"{}\"\n", key, value);
    
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_file)?;
    
    file.write_all(export_line.as_bytes())?;
    
    Ok(())
}

/// Select .env file interactively
fn select_env_file() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    
    // Find all .env files (both encrypted and plain)
    let all_files: Vec<PathBuf> = fs::read_dir(&current_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            if let Some(name) = p.file_name() {
                let name = name.to_string_lossy();
                name.starts_with(".env") && !name.ends_with(".enc") && !name.ends_with(".encrypted")
            } else {
                false
            }
        })
        .collect();
    
    if all_files.is_empty() {
        anyhow::bail!("No .env files found in current directory");
    }
    
    // Show files with var count
    println!("{} Found .env file(s) in current directory:", style("📂").cyan());
    for file in &all_files {
        let name = file.file_name().unwrap_or_default().to_string_lossy();
        let vars = scanner::count_variables(file);
        println!("  • {} ({} vars)", style(&name).cyan(), vars);
    }
    println!();
    
    // Build options
    let options: Vec<String> = all_files
        .iter()
        .map(|p| {
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            let vars = scanner::count_variables(p);
            format!("{} ({} vars)", name, vars)
        })
        .chain(std::iter::once("❌ Quit".to_string()))
        .collect();
    
    let selection = Select::new()
        .with_prompt("Select file to export")
        .items(&options)
        .default(0)
        .interact()?;
    
    // Check if Quit selected
    if selection >= all_files.len() {
        anyhow::bail!("Operation cancelled");
    }
    
    Ok(all_files[selection].clone())
}

/// Handle setenv command
pub fn handle_setenv(file: Option<PathBuf>, skip_confirm: bool) -> Result<()> {
    println!();
    
    // Step 1: Select or validate file
    let file_path = match file {
        Some(path) => {
            if !path.exists() {
                anyhow::bail!("File not found: {}", path.display());
            }
            path
        }
        None => select_env_file()?,
    };
    
    // Step 2: Read and parse file
    let content = fs::read_to_string(&file_path)?;
    let vars = parse_env_file(&content);
    
    if vars.is_empty() {
        anyhow::bail!("No environment variables found in file");
    }
    
    // Step 3: Show variables to be set
    println!("{} Will set {} environment variable(s):", style("📝").cyan(), vars.len());
    for (key, value) in &vars {
        // Mask sensitive values
        let display_value = if value.len() > 20 {
            format!("{}...", &value[..20])
        } else {
            value.clone()
        };
        println!("  • {} = {}", style(key).yellow(), style(&display_value).dim());
    }
    println!();
    
    // Step 4: Confirm
    if !skip_confirm {
        let platform_note = if cfg!(target_os = "windows") {
            "User Environment Variables (requires restart to take effect)"
        } else {
            "shell config file (~/.bashrc or ~/.zshrc)"
        };
        
        println!("{} Variables will be added to: {}", style("ℹ️").blue(), platform_note);
        
        let confirmed = Confirm::new()
            .with_prompt("Proceed?")
            .default(true)
            .interact()?;
        
        if !confirmed {
            anyhow::bail!("Operation cancelled");
        }
    }
    
    // Step 5: Set variables
    println!();
    println!("{} Setting environment variables...", style("⏳").cyan());
    
    let mut success_count = 0;
    let mut failed = Vec::new();
    
    for (key, value) in &vars {
        match set_env_permanent(key, value) {
            Ok(_) => {
                println!("  {} {}", style("✓").green(), key);
                success_count += 1;
            }
            Err(e) => {
                println!("  {} {} - {}", style("✗").red(), key, e);
                failed.push(key.clone());
            }
        }
    }
    
    // Step 6: Summary
    println!();
    if failed.is_empty() {
        println!("{} Done! Set {} variable(s) permanently", style("✅").green(), success_count);
        
        if cfg!(target_os = "windows") {
            println!();
            println!("{} Note: Restart your terminal or log out/in for changes to take effect.", 
                style("💡").yellow());
        } else {
            println!();
            println!("{} Note: Run 'source ~/.bashrc' or 'source ~/.zshrc' to apply changes.", 
                style("💡").yellow());
        }
    } else {
        println!("{} Set {} of {} variable(s). {} failed.", 
            style("⚠️").yellow(),
            success_count,
            vars.len(),
            failed.len()
        );
    }
    
    Ok(())
}
