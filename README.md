# wc-envc (WorkCloud Env Crypt)

[![Release](https://github.com/GinCanhViet/wc-envc/actions/workflows/release.yml/badge.svg)](https://github.com/GinCanhViet/wc-envc/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-cross--platform-lightgrey.svg)]()

> **Securely encrypt your `.env` files.**  
> **Commit them fearlessly. Share them safely.**

## ❓ Why wc-envc?

Managing environment variables in a team is often a headache:

- ❌ Sending `.env` via Slack/DM is insecure
- ❌ Machine dies = local configs gone forever
- ❌ `.gitignore` keeps secrets safe, but nobody knows what keys are required

**wc-envc solves this.** Encrypt the _values_, keep the _keys_ readable. Commit `.env.enc` to Git. Team pulls, decrypts with shared password, done.

## 📸 Screenshot

![Screenshot](/images/encrypt.jpg)

## ✨ Features

- 🔒 **AES-256 Encryption** - Industry-standard security
- 👁️ **Partial Encryption** - Only values encrypted, keys readable
- 🚀 **Interactive Mode** - Auto-scans for `.env` files, multi-select support
- 📝 **Auto .gitignore** - Prompts to add source files after encryption
- 🔐 **Secure Password** - Uses `SecretString` (memory zeroized after use)
- 🌐 **Permanent System Env** - `setenv` command to export variables to system permanently
- 🤖 **CI/CD Friendly** - Non-interactive mode + env var support
- 🦀 **Written in Rust** - Fast, lightweight, cross-platform

## 📦 Installation

### Option 1: Download Binary (Recommended)

Download the latest release from [GitHub Releases](https://github.com/gincanhviet/wc-envc/releases).

**Add to PATH:**

```powershell
# Windows (PowerShell - run as Admin)
# Move wc-envc.exe to a folder, e.g., C:\Tools\
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Tools", "Machine")
```

```bash
# Linux/macOS
chmod +x wc-envc
sudo mv wc-envc /usr/local/bin/
```

### Option 2: Build from Source

Requires [Rust](https://rustup.rs/) installed.

```bash
git clone https://github.com/gincanhviet/wc-envc.git
cd wc-envc
cargo install --path .
# Binary auto-added to ~/.cargo/bin (already in PATH)
```

## 🚀 Usage

### Interactive Mode (Recommended)

```bash
wc-envc encrypt    # Encrypt .env files
wc-envc decrypt    # Decrypt .env.enc files
```

### Permanent System Environment

Export variables from a `.env` file to your system permanently:

```bash
wc-envc setenv                # Interactive mode
wc-envc setenv .env.staging   # Direct file mode
```

- **Windows**: Adds to User Environment Variables (via Registry/setx).
- **Unix**: Appends to `~/.bashrc` or `~/.zshrc`.

**What happens:**

1. Scans directory for `.env` files
2. Select files (All / Individual / Quit)
3. Enter password (with confirmation for encrypt)
4. Files encrypted → prompts to add to `.gitignore`
5. Done!

### CLI Mode (For Scripts/CI)

```bash
# Encrypt
wc-envc encrypt -p "password" -i .env -o .env.enc -y

# Decrypt
wc-envc decrypt -p "password" -i .env.enc -o .env -y
```

### Using Environment Variable

```bash
export WC_ENVC_PASSWORD="password"
wc-envc encrypt -i .env -o .env.enc -y
```

**Password priority:** `-p` flag > `WC_ENVC_PASSWORD` > interactive prompt

## 👥 Team Workflow

1. **Alice** updates `.env` → runs `wc-envc encrypt` → commits `.env.enc`
2. **Bob** pulls → runs `wc-envc decrypt` (shared password) → local `.env` updated

## 🛠 Tech Stack

- **Language:** Rust 🦀
- **CLI:** `clap`
- **Encryption:** `magic-crypt` (AES-256)
- **Security:** `secrecy` (zeroize memory)
- **UI:** `dialoguer` & `console`

## 📄 License

[MIT License](LICENSE)

---

Made with ❤️ by [WorkCloud.vn](https://workcloud.vn)
