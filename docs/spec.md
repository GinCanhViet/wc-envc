# Project Specification: wc-envc (WorkCloud Env Crypt)

## 1. Overview

**wc-envc** is a lightweight, cross-platform CLI tool written in **Rust**.

**Purpose:** Encrypt/decrypt values within `.env` files.  
Developers can commit environment variable structure (keys) to git  
while keeping sensitive values (passwords, tokens) encrypted.

## 2. Tech Stack

- **Language:** Rust (Latest Stable)
- **Argument Parser:** `clap` (derive feature)
- **Encryption:** `magic-crypt` (AES-256, output Base64)
- **Error Handling:** `anyhow`

## 3. Core Logic

The tool processes files line-by-line:

1. **Comments/Empty lines:** Preserve as-is
2. **Variables:** Detect lines matching `KEY=VALUE`
   - **Encrypt:** `KEY=RealValue` → `KEY=EncryptedValue(Base64)`
   - **Decrypt:** `KEY=EncryptedValue(Base64)` → `KEY=RealValue`
3. **Only VALUE is encrypted**, KEY remains readable

## 4. CLI Interface

### Help & Error Handling

```bash
wc-envc -h          # Show help
wc-envc --help      # Same as above
wc-envc encrypt -h  # Help for encrypt command
```

**Sample output `wc-envc -h`:**

```
wc-envc - Encrypt/decrypt .env files

USAGE:
    wc-envc <COMMAND> [OPTIONS]

COMMANDS:
    encrypt    Encrypt .env file
    decrypt    Decrypt .env.enc file

OPTIONS:
    -h, --help       Show help
    -V, --version    Show version

Run 'wc-envc <COMMAND> -h' for more information on a command.
```

**When user enters invalid command:**

```
$ wc-envc xyz

❌ Invalid command: 'xyz'

💡 Run 'wc-envc -h' to see available commands.
```

---

### A. Primary Mode: Interactive Mode

The tool works in a **step-by-step Q&A** style, user-friendly:

#### Encrypt Flow

```bash
cd /path/to/project   # Directory containing .env file
wc-envc encrypt       # No parameters needed
```

**Step 1:** Tool auto-scans current directory for `.env*` files:

```
📂 Found .env files in current directory:
  [1] .env
  [2] .env.local
  [3] .env.production

👉 Select file to encrypt (1-3, or 'a' for all):
```

**Step 2:** User selects file, tool confirms:

```
✅ Selected: .env (12 environment variables)
📝 Default output file: .env.enc

👉 Enter output filename (Enter for default):
```

**Step 3:** Confirm overwrite (if output file exists):

```
⚠️  File .env.enc already exists!
👉 Overwrite this file? (y/N): y
```

**Step 4:** Enter password:

```
🔐 Enter encryption password: ********
🔐 Confirm password: ********
```

**Step 5:** Execute and display results:

```
⏳ Encrypting...
  ✓ DB_HOST
  ✓ DB_PASSWORD
  ✓ API_KEY
  ...

✅ Done! File saved: .env.enc

💡 Tip: To skip password prompt next time:
   export WC_ENVC_PASSWORD="your_password"
```

#### Decrypt Flow

```bash
wc-envc decrypt
```

Similar to encrypt, but:

- Scans for `.env.enc`, `.env.encrypted`, etc.
- Default output is `.env`
- Only requires password once (no confirmation)

#### Quick Mode (specify file directly)

```bash
wc-envc encrypt .env.local        # Encrypt this specific file
wc-envc decrypt .env.production.enc
```

### B. One-liner Mode (for CI/CD, scripts)

Users can still use full command:

```bash
# Method 1: Using -p flag (not recommended - saved in bash history)
wc-envc encrypt -p "mypassword" -i .env -o .env.enc

# Method 2: Using environment variable (recommended for CI/CD)
export WC_ENVC_PASSWORD="mypassword"
wc-envc encrypt -i .env -o .env.enc

# Method 3: Pipe password (for scripts)
echo "mypassword" | wc-envc encrypt -i .env -o .env.enc
```

**Password priority order:**

1. Flag `-p` (highest)
2. Environment variable `WC_ENVC_PASSWORD`
3. Interactive prompt (lowest)

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `-p, --password <TEXT>` | Encryption/decryption password | See priority order |
| `-i, --input <PATH>` | Input file | `.env` / `.env.enc` |
| `-o, --output <PATH>` | Output file | `.env.enc` / `.env` |
| `-y, --yes` | Skip confirmation (overwrite file) | `false` |

## 5. Code Structure

### Dependencies (`Cargo.toml`)

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
magic-crypt = "3.1"
anyhow = "1.0"
dialoguer = "0.11"  # For interactive prompts
console = "0.15"    # For colored output
```

### File Structure

```
src/
├── main.rs          # Entry point, CLI parsing (clap)
├── engine.rs        # Encryption/decryption logic per line
├── interactive.rs   # Interactive mode handling
└── scanner.rs       # Scan and list .env files
```

## 6. Edge Cases

- **Trimming:** Trim whitespace around value before encrypting
- **Missing '=':** Lines without `=` → preserve as comment
- **File Not Found:** Show user-friendly error message
- **Empty password:** Not allowed, prompt to re-enter
- **Password mismatch (encrypt):** Prompt to confirm again
- **Output file exists:** Ask for overwrite confirmation (unless `-y`)
- **Decrypt validation:** Validate file before decrypting:
  - Check Base64 format of values
  - Try decrypting, if error → show "❌ Wrong password or invalid file"
  - Detect unencrypted file → show "⚠️ This file appears to be unencrypted"

## 7. Examples

### Input (`.env`)

```env
# Database Config
DB_HOST=localhost
DB_PASSWORD=secret_123
```

### Interactive Session

```
$ wc-envc encrypt

📂 Found .env files:
  [1] .env

👉 Select file (1): 1
✅ Selected: .env
👉 Output file (Enter = .env.enc):
🔐 Enter password: ********
🔐 Confirm: ********

⏳ Encrypting...
  ✓ DB_HOST
  ✓ DB_PASSWORD

✅ Done! Saved: .env.enc
```

### Output (`.env.enc`)

```env
# Database Config
DB_HOST=M7d/s...
DB_PASSWORD=k9A...
```
