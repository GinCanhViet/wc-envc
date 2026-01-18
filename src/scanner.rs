use std::path::{Path, PathBuf};
use std::fs;

use crate::engine::ProcessMode;

/// Patterns to match for decryption (encrypted files)
const DECRYPT_EXTENSIONS: &[&str] = &[".enc", ".encrypted"];

/// Find .env files in directory based on mode
pub fn find_env_files(dir: &Path, mode: ProcessMode) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return files,
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        
        match mode {
            ProcessMode::Encrypt => {
                // Find plain .env files (not already encrypted)
                if is_plain_env_file(filename) {
                    files.push(path);
                }
            }
            ProcessMode::Decrypt => {
                // Find encrypted .env files
                if is_encrypted_env_file(filename) {
                    files.push(path);
                }
            }
        }
    }
    
    // Sort for consistent ordering
    files.sort();
    files
}

/// Check if filename is a plain .env file (not encrypted)
fn is_plain_env_file(filename: &str) -> bool {
    // Must start with .env
    if !filename.starts_with(".env") {
        return false;
    }
    
    // Must not end with encrypted extension
    for ext in DECRYPT_EXTENSIONS {
        if filename.ends_with(ext) {
            return false;
        }
    }
    
    true
}

/// Check if filename is an encrypted .env file
fn is_encrypted_env_file(filename: &str) -> bool {
    // Check for .env*.enc or .env*.encrypted patterns
    if !filename.contains(".env") {
        return false;
    }
    
    for ext in DECRYPT_EXTENSIONS {
        if filename.ends_with(ext) {
            return true;
        }
    }
    
    false
}

/// Count environment variables in a file
pub fn count_variables(path: &Path) -> usize {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    
    content.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.contains('=')
        })
        .count()
}

/// Generate default output filename based on input and mode
pub fn default_output_name(input: &Path, mode: ProcessMode) -> PathBuf {
    let input_str = input.to_string_lossy();
    
    match mode {
        ProcessMode::Encrypt => {
            // .env -> .env.enc
            PathBuf::from(format!("{}.enc", input_str))
        }
        ProcessMode::Decrypt => {
            // .env.enc -> .env
            // .env.local.enc -> .env.local
            let name = input_str.trim_end_matches(".enc").trim_end_matches(".encrypted");
            PathBuf::from(name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_plain_env_file() {
        assert!(is_plain_env_file(".env"));
        assert!(is_plain_env_file(".env.local"));
        assert!(is_plain_env_file(".env.production"));
        assert!(!is_plain_env_file(".env.enc"));
        assert!(!is_plain_env_file(".env.local.enc"));
        assert!(!is_plain_env_file("readme.md"));
    }
    
    #[test]
    fn test_is_encrypted_env_file() {
        assert!(is_encrypted_env_file(".env.enc"));
        assert!(is_encrypted_env_file(".env.local.enc"));
        assert!(is_encrypted_env_file(".env.encrypted"));
        assert!(!is_encrypted_env_file(".env"));
        assert!(!is_encrypted_env_file(".env.local"));
    }
    
    #[test]
    fn test_default_output_name() {
        let encrypt = default_output_name(Path::new(".env"), ProcessMode::Encrypt);
        assert_eq!(encrypt, PathBuf::from(".env.enc"));
        
        let decrypt = default_output_name(Path::new(".env.enc"), ProcessMode::Decrypt);
        assert_eq!(decrypt, PathBuf::from(".env"));
    }
}
