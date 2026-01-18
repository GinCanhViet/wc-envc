# Đặc tả dự án: wc-envc (WorkCloud Env Crypt)

## 1. Tổng quan

**wc-envc** là CLI tool viết bằng **Rust**, nhẹ và chạy cross-platform.

**Mục đích:** Mã hóa/giải mã các giá trị trong file `.env`.  
Dev có thể commit cấu trúc biến môi trường (keys) lên git  
trong khi giữ các giá trị nhạy cảm (passwords, tokens) được mã hóa.

## 2. Tech Stack

- **Ngôn ngữ:** Rust (Latest Stable)
- **Argument Parser:** `clap` (derive feature)
- **Encryption:** `magic-crypt` (AES-256, output Base64)
- **Error Handling:** `anyhow`

## 3. Logic xử lý

Tool xử lý file theo từng dòng:

1. **Comments/Empty lines:** Giữ nguyên
2. **Variables:** Nhận diện dòng `KEY=VALUE`
   - **Encrypt:** `KEY=RealValue` → `KEY=EncryptedValue(Base64)`
   - **Decrypt:** `KEY=EncryptedValue(Base64)` → `KEY=RealValue`
3. **Chỉ mã hóa VALUE**, KEY vẫn đọc được

## 4. CLI Interface

### Help & Error Handling

```bash
wc-envc -h          # Hiển thị help
wc-envc --help      # Tương tự
wc-envc encrypt -h  # Help cho lệnh encrypt
```

**Output mẫu `wc-envc -h`:**

```
wc-envc - Mã hóa/giải mã file .env

USAGE:
    wc-envc <COMMAND> [OPTIONS]

COMMANDS:
    encrypt    Mã hóa file .env
    decrypt    Giải mã file .env.enc

OPTIONS:
    -h, --help       Hiển thị help
    -V, --version    Hiển thị version

Chạy 'wc-envc <COMMAND> -h' để xem chi tiết từng lệnh.
```

**Khi user gõ lệnh không hợp lệ:**

```
$ wc-envc xyz

❌ Lệnh không hợp lệ: 'xyz'

💡 Gõ 'wc-envc -h' để xem danh sách lệnh.
```

---

### A. Cách hoạt động chính: Interactive Mode

Tool hoạt động theo kiểu **hỏi đáp từng bước**, thân thiện với user:

#### Encrypt Flow

```bash
cd /path/to/project   # Nơi chứa file .env
wc-envc encrypt       # Không cần tham số
```

**Bước 1:** Tool tự động scan thư mục hiện tại, tìm các file `.env*`:

```
📂 Tìm thấy các file .env trong thư mục hiện tại:
  [1] .env
  [2] .env.local
  [3] .env.production

👉 Chọn file cần mã hóa (1-3, hoặc 'a' cho tất cả):
```

**Bước 2:** User chọn file, tool xác nhận:

```
✅ Đã chọn: .env (12 biến môi trường)
📝 File output mặc định: .env.enc

👉 Nhập tên file output (Enter để dùng mặc định):
```

**Bước 3:** Xác nhận ghi đè (nếu file output đã tồn tại):

```
⚠️  File .env.enc đã tồn tại!
👉 Ghi đè file này? (y/N): y
```

**Bước 4:** Nhập password:

```
🔐 Nhập password để mã hóa: ********
🔐 Xác nhận password: ********
```

**Bước 5:** Thực thi và hiển thị kết quả:

```
⏳ Đang mã hóa...
  ✓ DB_HOST
  ✓ DB_PASSWORD
  ✓ API_KEY
  ...

✅ Hoàn tất! File đã được lưu: .env.enc

💡 Tip: Để không phải nhập password lần sau:
   export WC_ENVC_PASSWORD="your_password"
```

#### Decrypt Flow

```bash
wc-envc decrypt
```

Tương tự encrypt, nhưng:

- Tìm các file `.env.enc`, `.env.encrypted`, v.v.
- Output mặc định là `.env`
- Chỉ yêu cầu nhập password 1 lần

#### Quick Mode (chỉ định file cụ thể)

```bash
wc-envc encrypt .env.local        # Chỉ mã hóa file này
wc-envc decrypt .env.production.enc
```

### B. One-liner Mode (cho CI/CD, scripts)

User vẫn có thể dùng full command:

```bash
# Cách 1: Dùng flag -p (không khuyến khích - lưu trong bash history)
wc-envc encrypt -p "mypassword" -i .env -o .env.enc

# Cách 2: Dùng biến môi trường (khuyến khích cho CI/CD)
export WC_ENVC_PASSWORD="mypassword"
wc-envc encrypt -i .env -o .env.enc

# Cách 3: Pipe password (cho scripts)
echo "mypassword" | wc-envc encrypt -i .env -o .env.enc
```

**Thứ tự ưu tiên password:**

1. Flag `-p` (cao nhất)
2. Biến môi trường `WC_ENVC_PASSWORD`
3. Interactive prompt (thấp nhất)

**Options:**
| Flag | Mô tả | Mặc định |
|------|-------|----------|
| `-p, --password <TEXT>` | Password mã hóa/giải mã | Xem thứ tự ưu tiên |
| `-i, --input <PATH>` | File input | `.env` / `.env.enc` |
| `-o, --output <PATH>` | File output | `.env.enc` / `.env` |
| `-y, --yes` | Skip confirmation (ghi đè file) | `false` |

## 5. Cấu trúc code

### Dependencies (`Cargo.toml`)

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
magic-crypt = "3.1"
anyhow = "1.0"
dialoguer = "0.11"  # Cho interactive prompts
console = "0.15"    # Cho colored output
```

### File Structure

```
src/
├── main.rs          # Entry point, CLI parsing (clap)
├── engine.rs        # Logic mã hóa/giải mã từng dòng
├── interactive.rs   # Xử lý interactive mode
└── scanner.rs       # Scan và liệt kê file .env
```

## 6. Edge Cases

- **Trimming:** Trim whitespace quanh value trước khi mã hóa
- **Missing '=':** Dòng không có `=` → giữ nguyên như comment
- **File Not Found:** Thông báo lỗi thân thiện
- **Empty password:** Không cho phép, yêu cầu nhập lại
- **Password mismatch (encrypt):** Yêu cầu xác nhận lại
- **File output đã tồn tại:** Hỏi xác nhận ghi đè (trừ khi có `-y`)
- **Decrypt validation:** Kiểm tra file trước khi giải mã:
  - Check format Base64 của các value
  - Thử giải mã, nếu lỗi → báo "❌ Sai password hoặc file không hợp lệ"
  - Detect file chưa được mã hóa → báo "⚠️ File này có vẻ chưa được mã hóa"

## 7. Ví dụ

### Input (`.env`)

```env
# Database Config
DB_HOST=localhost
DB_PASSWORD=secret_123
```

### Interactive Session

```
$ wc-envc encrypt

📂 Tìm thấy các file .env:
  [1] .env

👉 Chọn file (1): 1
✅ Đã chọn: .env
👉 File output (Enter = .env.enc):
🔐 Nhập password: ********
🔐 Xác nhận: ********

⏳ Đang mã hóa...
  ✓ DB_HOST
  ✓ DB_PASSWORD

✅ Hoàn tất! Đã lưu: .env.enc
```

### Output (`.env.enc`)

```env
# Database Config
DB_HOST=M7d/s...
DB_PASSWORD=k9A...
```
