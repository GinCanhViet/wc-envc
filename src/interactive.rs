use std::path::{Path, PathBuf};
use std::env;
use std::io::Write;
use std::fs::{self, OpenOptions};

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, MultiSelect, Password, Select};
use secrecy::SecretString;

use crate::engine::{self, ProcessMode};
use crate::scanner;

/// Environment variable name for password
const PASSWORD_ENV_VAR: &str = "WC_ENVC_PASSWORD";

/// Run interactive encrypt flow
pub fn run_interactive_encrypt(input_file: Option<PathBuf>) -> Result<()> {
    println!();
    
    // Step 1: Select file(s)
    let input_paths = match input_file {
        Some(path) => {
            if !path.exists() {
                anyhow::bail!("File not found: {}", path.display());
            }
            vec![path]
        }
        None => select_files(ProcessMode::Encrypt)?,
    };
    
    // Show selected files
    println!("{} Selected {} file(s):", style("✅").green(), input_paths.len());
    for path in &input_paths {
        let var_count = scanner::count_variables(path);
        println!("  • {} ({} vars)", 
            style(path.file_name().unwrap_or_default().to_string_lossy()).cyan(),
            var_count
        );
    }
    
    // Step 2: Confirm output files
    let output_paths: Vec<PathBuf> = input_paths
        .iter()
        .map(|p| scanner::default_output_name(p, ProcessMode::Encrypt))
        .collect();
    
    println!();
    println!("{} Output files:", style("📝").cyan());
    for output in &output_paths {
        println!("  • {}", style(output.file_name().unwrap_or_default().to_string_lossy()).yellow());
    }
    
    let confirmed = Confirm::new()
        .with_prompt("Proceed with these output files?")
        .default(true)
        .interact()?;
    
    if !confirmed {
        anyhow::bail!("Operation cancelled");
    }
    
    // Step 3: Check for existing files
    let existing: Vec<&PathBuf> = output_paths.iter().filter(|p| p.exists()).collect();
    if !existing.is_empty() {
        println!();
        println!("{} The following files already exist:", style("⚠️").yellow());
        for path in &existing {
            println!("  • {}", style(path.file_name().unwrap_or_default().to_string_lossy()).red());
        }
        
        let confirmed = Confirm::new()
            .with_prompt("Overwrite these files?")
            .default(false)
            .interact()?;
        
        if !confirmed {
            anyhow::bail!("Operation cancelled");
        }
    }
    
    // Step 4: Get password
    let password = get_password_with_confirm()?;
    
    // Step 5: Process all files
    println!();
    println!("{} Encrypting {} file(s)...", style("⏳").cyan(), input_paths.len());
    
    for (input, output) in input_paths.iter().zip(output_paths.iter()) {
        process_and_save_quiet(input, output, &password, ProcessMode::Encrypt)?;
    }
    
    println!();
    println!("{} Done! Encrypted {} file(s)", style("✅").green(), input_paths.len());
    
    // Step 6: Offer to add original files to .gitignore
    offer_gitignore(&input_paths)?;
    
    // Show tip
    println!();
    println!("{} Tip: To skip password prompt next time:", style("💡").yellow());
    println!("   export {}=\"your_password\"", PASSWORD_ENV_VAR);
    
    Ok(())
}

/// Run interactive decrypt flow
pub fn run_interactive_decrypt(input_file: Option<PathBuf>) -> Result<()> {
    println!();
    
    // Step 1: Select file(s)
    let input_paths = match input_file {
        Some(path) => {
            if !path.exists() {
                anyhow::bail!("File not found: {}", path.display());
            }
            vec![path]
        }
        None => select_files(ProcessMode::Decrypt)?,
    };
    
    // Validate all files
    for path in &input_paths {
        let content = std::fs::read_to_string(path)?;
        engine::validate_encrypted_file(&content)?;
    }
    
    // Show selected files
    println!("{} Selected {} file(s):", style("✅").green(), input_paths.len());
    for path in &input_paths {
        let var_count = scanner::count_variables(path);
        println!("  • {} ({} vars)", 
            style(path.file_name().unwrap_or_default().to_string_lossy()).cyan(),
            var_count
        );
    }
    
    // Step 2: Confirm output files
    let output_paths: Vec<PathBuf> = input_paths
        .iter()
        .map(|p| scanner::default_output_name(p, ProcessMode::Decrypt))
        .collect();
    
    println!();
    println!("{} Output files:", style("📝").cyan());
    for output in &output_paths {
        println!("  • {}", style(output.file_name().unwrap_or_default().to_string_lossy()).yellow());
    }
    
    let confirmed = Confirm::new()
        .with_prompt("Proceed with these output files?")
        .default(true)
        .interact()?;
    
    if !confirmed {
        anyhow::bail!("Operation cancelled");
    }
    
    // Step 3: Check for existing files
    let existing: Vec<&PathBuf> = output_paths.iter().filter(|p| p.exists()).collect();
    if !existing.is_empty() {
        println!();
        println!("{} The following files already exist:", style("⚠️").yellow());
        for path in &existing {
            println!("  • {}", style(path.file_name().unwrap_or_default().to_string_lossy()).red());
        }
        
        let confirmed = Confirm::new()
            .with_prompt("Overwrite these files?")
            .default(false)
            .interact()?;
        
        if !confirmed {
            anyhow::bail!("Operation cancelled");
        }
    }
    
    // Step 4: Get password
    let password = get_password()?;
    
    // Step 5: Process all files
    println!();
    println!("{} Decrypting {} file(s)...", style("⏳").cyan(), input_paths.len());
    
    for (input, output) in input_paths.iter().zip(output_paths.iter()) {
        process_and_save_quiet(input, output, &password, ProcessMode::Decrypt)?;
    }
    
    println!();
    println!("{} Done! Decrypted {} file(s)", style("✅").green(), input_paths.len());
    
    Ok(())
}

/// Run one-liner mode (non-interactive)
pub fn run_one_liner(
    input: PathBuf,
    output: PathBuf,
    password: Option<String>,
    skip_confirm: bool,
    mode: ProcessMode,
) -> Result<()> {
    // Validate input exists
    if !input.exists() {
        anyhow::bail!("File not found: {}", input.display());
    }
    
    // For decrypt, validate file
    if mode == ProcessMode::Decrypt {
        let content = std::fs::read_to_string(&input)?;
        engine::validate_encrypted_file(&content)?;
    }
    
    // Check overwrite
    if output.exists() && !skip_confirm {
        confirm_overwrite(&output)?;
    }
    
    // Get password from: arg > env > prompt
    let password = match password {
        Some(p) => SecretString::new(p),
        None => get_password_from_env_or_prompt(mode == ProcessMode::Encrypt)?,
    };
    
    process_and_save(&input, &output, &password, mode)?;
    
    Ok(())
}

/// Select multiple files from list with "All files" option
fn select_files(mode: ProcessMode) -> Result<Vec<PathBuf>> {
    let current_dir = env::current_dir()?;
    let files = scanner::find_env_files(&current_dir, mode);
    
    if files.is_empty() {
        let file_type = match mode {
            ProcessMode::Encrypt => ".env",
            ProcessMode::Decrypt => ".env.enc",
        };
        anyhow::bail!("No {} files found in current directory", file_type);
    }
    
    // Show found files
    println!("{} Found {} .env file(s) in current directory:", style("📂").cyan(), files.len());
    for file in &files {
        let name = file.file_name().unwrap_or_default().to_string_lossy();
        let vars = scanner::count_variables(file);
        println!("  • {} ({} vars)", style(&name).cyan(), vars);
    }
    println!();
    
    // First: Ask selection mode
    let mode_options = vec![
        format!("📦 All files ({})", files.len()),
        "📋 Select individual files".to_string(),
        "❌ Quit".to_string(),
    ];
    
    let mode_selection = Select::new()
        .with_prompt("Choose an option")
        .items(&mode_options)
        .default(0)
        .interact()?;
    
    match mode_selection {
        0 => {
            // All files
            println!("{} Selected all {} file(s)", style("✅").green(), files.len());
            Ok(files)
        }
        1 => {
            // Individual selection
            let file_options: Vec<String> = files.iter().map(|p| {
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                let vars = scanner::count_variables(p);
                format!("{} ({} vars)", name, vars)
            }).collect();
            
            let selections = MultiSelect::new()
                .with_prompt("Select files (Space to select, Enter to confirm)")
                .items(&file_options)
                .interact()?;
            
            if selections.is_empty() {
                anyhow::bail!("No files selected");
            }
            
            let selected: Vec<PathBuf> = selections
                .iter()
                .map(|&i| files[i].clone())
                .collect();
            
            Ok(selected)
        }
        _ => {
            // Quit
            anyhow::bail!("Operation cancelled");
        }
    }
}

/// Confirm file overwrite
fn confirm_overwrite(path: &Path) -> Result<()> {
    println!("{} File {} already exists!", 
        style("⚠️").yellow(),
        style(path.display()).cyan()
    );
    
    let confirmed = Confirm::new()
        .with_prompt("Overwrite this file?")
        .default(false)
        .interact()?;
    
    if !confirmed {
        anyhow::bail!("Operation cancelled");
    }
    
    Ok(())
}

/// Get password with confirmation (for encrypt)
fn get_password_with_confirm() -> Result<SecretString> {
    // Check env var first
    if let Ok(pwd) = env::var(PASSWORD_ENV_VAR) {
        if !pwd.is_empty() {
            println!("{} Using password from {}", style("🔐").cyan(), PASSWORD_ENV_VAR);
            return Ok(SecretString::new(pwd));
        }
    }
    
    loop {
        let password = Password::new()
            .with_prompt(format!("{} Enter encryption password", style("🔐").cyan()))
            .interact()?;
        
        if password.is_empty() {
            println!("{} Password cannot be empty", style("❌").red());
            continue;
        }
        
        let confirm = Password::new()
            .with_prompt(format!("{} Confirm password", style("🔐").cyan()))
            .interact()?;
        
        if password != confirm {
            println!("{} Passwords do not match, please try again", style("❌").red());
            continue;
        }
        
        return Ok(SecretString::new(password));
    }
}

/// Get password without confirmation (for decrypt)
fn get_password() -> Result<SecretString> {
    // Check env var first
    if let Ok(pwd) = env::var(PASSWORD_ENV_VAR) {
        if !pwd.is_empty() {
            println!("{} Using password from {}", style("🔐").cyan(), PASSWORD_ENV_VAR);
            return Ok(SecretString::new(pwd));
        }
    }
    
    let password = Password::new()
        .with_prompt(format!("{} Enter decryption password", style("🔐").cyan()))
        .interact()?;
    
    if password.is_empty() {
        anyhow::bail!("Password cannot be empty");
    }
    
    Ok(SecretString::new(password))
}

/// Get password from env var or prompt
fn get_password_from_env_or_prompt(with_confirm: bool) -> Result<SecretString> {
    if let Ok(pwd) = env::var(PASSWORD_ENV_VAR) {
        if !pwd.is_empty() {
            return Ok(SecretString::new(pwd));
        }
    }
    
    if with_confirm {
        get_password_with_confirm()
    } else {
        get_password()
    }
}

/// Process file and save result (verbose, for single file)
fn process_and_save(
    input: &Path,
    output: &Path,
    password: &SecretString,
    mode: ProcessMode,
) -> Result<()> {
    let content = std::fs::read_to_string(input)?;
    
    let action = match mode {
        ProcessMode::Encrypt => "Encrypting",
        ProcessMode::Decrypt => "Decrypting",
    };
    
    println!();
    println!("{} {}...", style("⏳").cyan(), action);
    
    let (result, keys) = engine::process_file(&content, password, mode)?;
    
    // Show processed keys
    for key in &keys {
        println!("  {} {}", style("✓").green(), key);
    }
    
    // Write output file
    let mut file = std::fs::File::create(output)?;
    file.write_all(result.as_bytes())?;
    
    println!();
    println!("{} Done! Saved: {}", 
        style("✅").green(),
        style(output.display()).cyan()
    );
    
    Ok(())
}

/// Process file and save result (quiet, for batch processing)
fn process_and_save_quiet(
    input: &Path,
    output: &Path,
    password: &SecretString,
    mode: ProcessMode,
) -> Result<()> {
    let content = std::fs::read_to_string(input)?;
    let (result, keys) = engine::process_file(&content, password, mode)?;
    
    // Write output file
    let mut file = std::fs::File::create(output)?;
    file.write_all(result.as_bytes())?;
    
    // Show summary for this file
    let input_name = input.file_name().unwrap_or_default().to_string_lossy();
    let output_name = output.file_name().unwrap_or_default().to_string_lossy();
    println!("  {} {} → {} ({} vars)", 
        style("✓").green(),
        style(&input_name).cyan(),
        style(&output_name).yellow(),
        keys.len()
    );
    
    Ok(())
}

/// Offer to add encrypted source files to .gitignore
fn offer_gitignore(input_files: &[PathBuf]) -> Result<()> {
    // Get filenames to potentially add to gitignore
    let filenames: Vec<String> = input_files
        .iter()
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .collect();
    
    if filenames.is_empty() {
        return Ok(());
    }
    
    // Check if .gitignore exists in current directory
    let gitignore_path = env::current_dir()?.join(".gitignore");
    
    // Read existing gitignore content
    let existing_content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path).unwrap_or_default()
    } else {
        String::new()
    };
    
    // Find which files are NOT already in gitignore
    let missing: Vec<&String> = filenames
        .iter()
        .filter(|f| !existing_content.lines().any(|line| line.trim() == *f))
        .collect();
    
    if missing.is_empty() {
        return Ok(());
    }
    
    // Ask user
    println!();
    println!("{} The following source files are not in .gitignore:", style("📝").cyan());
    for f in &missing {
        println!("  • {}", style(f).yellow());
    }
    
    let confirmed = Confirm::new()
        .with_prompt("Add them to .gitignore?")
        .default(true)
        .interact()?;
    
    if !confirmed {
        return Ok(());
    }
    
    // Append to .gitignore
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gitignore_path)?;
    
    // Add newline if file doesn't end with one
    if !existing_content.is_empty() && !existing_content.ends_with('\n') {
        writeln!(file)?;
    }
    
    // Add comment and files
    writeln!(file, "\n# Plain .env files (secrets - do not commit)")?;
    for f in &missing {
        writeln!(file, "{}", f)?;
    }
    
    println!("{} Added {} file(s) to .gitignore", style("✅").green(), missing.len());
    
    Ok(())
}

