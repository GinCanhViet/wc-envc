use anyhow::Result;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use secrecy::{ExposeSecret, SecretString};

/// Modes for processing .env files
#[derive(Clone, Copy, PartialEq)]
pub enum ProcessMode {
    Encrypt,
    Decrypt,
}

/// Encrypts a single value using AES-256
pub fn encrypt_value(value: &str, password: &SecretString) -> String {
    let mc = new_magic_crypt!(password.expose_secret(), 256);
    mc.encrypt_str_to_base64(value.trim())
}

/// Decrypts a Base64 encrypted value
/// Returns Err if password is wrong or value is not valid encrypted data
pub fn decrypt_value(encrypted: &str, password: &SecretString) -> Result<String> {
    let mc = new_magic_crypt!(password.expose_secret(), 256);
    mc.decrypt_base64_to_string(encrypted.trim())
        .map_err(|_| anyhow::anyhow!("Wrong password or invalid encrypted data"))
}

/// Checks if a string looks like Base64 encoded data
pub fn is_likely_encrypted(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    
    // Check if it's valid Base64 and has reasonable length for encrypted data
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, trimmed).is_ok()
        && trimmed.len() >= 8 // Encrypted values are typically longer
}

/// Process a single line from .env file
/// Returns the processed line (encrypted/decrypted)
fn process_line(line: &str, password: &SecretString, mode: ProcessMode) -> Result<String> {
    let trimmed = line.trim();
    
    // Preserve empty lines and comments
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(line.to_string());
    }
    
    // Check for KEY=VALUE pattern
    if let Some(eq_pos) = line.find('=') {
        let key = &line[..eq_pos];
        let value = &line[eq_pos + 1..];
        
        match mode {
            ProcessMode::Encrypt => {
                let encrypted = encrypt_value(value, password);
                Ok(format!("{}={}", key, encrypted))
            }
            ProcessMode::Decrypt => {
                let decrypted = decrypt_value(value, password)?;
                Ok(format!("{}={}", key, decrypted))
            }
        }
    } else {
        // No '=' found, preserve the line as-is
        Ok(line.to_string())
    }
}

/// Process entire file content line by line
/// Returns tuple: (processed_content, list of processed keys)
pub fn process_file(content: &str, password: &SecretString, mode: ProcessMode) -> Result<(String, Vec<String>)> {
    let mut output_lines = Vec::new();
    let mut processed_keys = Vec::new();
    
    for line in content.lines() {
        let processed = process_line(line, password, mode)?;
        
        // Track which keys were processed
        if let Some(eq_pos) = line.find('=') {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                let key = line[..eq_pos].trim().to_string();
                processed_keys.push(key);
            }
        }
        
        output_lines.push(processed);
    }
    
    Ok((output_lines.join("\n"), processed_keys))
}

/// Validate that file content appears to be encrypted
/// Checks if values look like Base64
pub fn validate_encrypted_file(content: &str) -> Result<()> {
    let mut has_variables = false;
    let mut encrypted_count = 0;
    let mut plain_count = 0;
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        if let Some(eq_pos) = line.find('=') {
            has_variables = true;
            let value = &line[eq_pos + 1..];
            
            if is_likely_encrypted(value) {
                encrypted_count += 1;
            } else {
                plain_count += 1;
            }
        }
    }
    
    if !has_variables {
        anyhow::bail!("File contains no environment variables");
    }
    
    if encrypted_count == 0 && plain_count > 0 {
        anyhow::bail!("This file appears to be unencrypted");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "secret_value_123";
        let password = SecretString::new("test_password".to_string());
        
        let encrypted = encrypt_value(original, &password);
        let decrypted = decrypt_value(&encrypted, &password).unwrap();
        
        assert_eq!(original, decrypted);
    }
    
    #[test]
    fn test_wrong_password() {
        let correct_pwd = SecretString::new("correct_password".to_string());
        let wrong_pwd = SecretString::new("wrong_password".to_string());
        
        let encrypted = encrypt_value("secret", &correct_pwd);
        let result = decrypt_value(&encrypted, &wrong_pwd);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_process_file_encrypt() {
        let content = "# Comment\nDB_HOST=localhost\nDB_PASS=secret\n";
        let password = SecretString::new("test".to_string());
        
        let (result, keys) = process_file(content, &password, ProcessMode::Encrypt).unwrap();
        
        assert!(result.contains("# Comment"));
        assert!(result.contains("DB_HOST="));
        assert!(!result.contains("localhost")); // Should be encrypted
        assert_eq!(keys, vec!["DB_HOST", "DB_PASS"]);
    }
}
