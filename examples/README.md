# wc-envc Examples

Example `.env` files for testing wc-envc.

## Files

| File              | Description                          |
| ----------------- | ------------------------------------ |
| `.env.example`    | Full web app (DB, Redis, OAuth, JWT) |
| `.env.production` | Production (Stripe, AWS, Sentry)     |
| `.env.simple`     | Minimal test file (4 vars)           |

## Quick Start

```bash
cd examples

# Interactive - encrypt all files
wc-envc encrypt
# → Select "All files" → Enter password → Done!

# Interactive - decrypt
wc-envc decrypt
```

## One-liner Mode

```bash
# Encrypt
wc-envc encrypt -p "password" -i .env.simple -o .env.simple.enc -y

# Decrypt
wc-envc decrypt -p "password" -i .env.simple.enc -o .env.simple -y
```

## Using Environment Variable

```powershell
# PowerShell
$env:WC_ENVC_PASSWORD = "password"
wc-envc encrypt -i .env.production -o .env.production.enc -y
```

```bash
# Bash
export WC_ENVC_PASSWORD="password"
wc-envc encrypt -i .env.production -o .env.production.enc -y
```

## Features Demo

1. **Multi-file selection**: Choose "All files" or use Space to select individual files
2. **Auto gitignore**: After encrypt, tool asks to add source files to `.gitignore`
3. **Secure password**: Uses `SecretString` (memory zeroized after use)
