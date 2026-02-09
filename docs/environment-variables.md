# Vibe Kanban - 環境變數設定說明

本文件詳細說明 Vibe Kanban 所有支援的環境變數及其用途。

## 目錄

1. [伺服器設定](#伺服器設定)
2. [資料庫設定](#資料庫設定)
3. [日誌設定](#日誌設定)
4. [安全性設定](#安全性設定)
5. [編譯時環境變數](#編譯時環境變數)
6. [範例設定檔](#範例設定檔)

---

## 伺服器設定

| 變數名稱 | 預設值 | 說明 |
|---------|--------|------|
| `HOST` | `127.0.0.1` | 伺服器綁定的 IP 位址。設為 `0.0.0.0` 可接受所有來源連線。 |
| `BACKEND_PORT` | `3001` | 後端 API 伺服器監聽的端口 |
| `FRONTEND_PORT` | `3000` | 前端開發伺服器監聽的端口 (僅開發模式) |

### 範例

```bash
# 生產環境 - 允許外部連線
HOST=0.0.0.0
BACKEND_PORT=3001

# 本地開發 - 僅本機連線
HOST=127.0.0.1
BACKEND_PORT=3001
```

---

## 資料庫設定

| 變數名稱 | 預設值 | 說明 |
|---------|--------|------|
| `DATABASE_URL` | `sqlite:data/vibe-kanban.db` | SQLite 資料庫檔案路徑 |

### 範例

```bash
# Linux/macOS
DATABASE_URL=sqlite:/opt/vibe-kanban/data/vibe-kanban.db

# Windows
DATABASE_URL=sqlite:C:\vibe-kanban\data\vibe-kanban.db

# 相對路徑 (以執行目錄為基準)
DATABASE_URL=sqlite:./data/vibe-kanban.db
```

---

## 日誌設定

| 變數名稱 | 預設值 | 說明 |
|---------|--------|------|
| `RUST_LOG` | `info` | 日誌等級設定。支援模組級別的精細控制。 |

### 日誌等級

| 等級 | 說明 |
|------|------|
| `error` | 僅顯示錯誤 |
| `warn` | 顯示警告和錯誤 |
| `info` | 顯示一般資訊 (推薦生產環境) |
| `debug` | 顯示除錯資訊 |
| `trace` | 顯示最詳細的追蹤資訊 |

### 範例

```bash
# 生產環境 - 僅顯示重要資訊
RUST_LOG=info

# 開發環境 - 顯示除錯資訊
RUST_LOG=debug

# 針對特定模組設定
RUST_LOG=info,server=debug,sqlx=warn

# 最詳細的追蹤
RUST_LOG=trace
```

---

## 安全性設定

| 變數名稱 | 預設值 | 說明 |
|---------|--------|------|
| `VK_ALLOWED_ORIGINS` | `""` | CORS 允許的來源網址 (逗號分隔) |
| `SENTRY_AUTH_TOKEN` | `""` | Sentry 錯誤追蹤服務的認證令牌 |

### VK_ALLOWED_ORIGINS

控制跨來源資源共享 (CORS) 的允許來源。

```bash
# 單一來源
VK_ALLOWED_ORIGINS=http://localhost:3000

# 多個來源 (逗號分隔)
VK_ALLOWED_ORIGINS=http://localhost:3000,https://app.example.com

# 開發環境 (允許多個本地端口)
VK_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:5173,http://127.0.0.1:3000
```

### SENTRY_AUTH_TOKEN

用於 Sentry 錯誤追蹤服務的整合。

```bash
# 從 Sentry 後台取得
SENTRY_AUTH_TOKEN=sntrys_eyJpYXQiOjE2...
```

---

## 編譯時環境變數

這些變數僅在編譯專案時需要設定。

### Linux 專用

| 變數名稱 | 說明 |
|---------|------|
| `CC` | C 編譯器路徑 (建議 `gcc-10`) |
| `CXX` | C++ 編譯器路徑 (建議 `g++-10`) |
| `BINDGEN_EXTRA_CLANG_ARGS` | bindgen 的額外 Clang 參數 |

```bash
# Linux 編譯時必須設定
export CC=gcc-10
export CXX=g++-10
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/10/include"
```

### 通用編譯選項

| 變數名稱 | 說明 |
|---------|------|
| `CARGO_BUILD_JOBS` | 並行編譯的工作數量 |
| `RUSTFLAGS` | Rust 編譯器標誌 |

```bash
# 限制並行編譯數量 (記憶體不足時)
CARGO_BUILD_JOBS=2

# 啟用原生 CPU 優化
RUSTFLAGS="-C target-cpu=native"
```

---

## 範例設定檔

### 開發環境 (.env.development)

```bash
# 伺服器
HOST=127.0.0.1
BACKEND_PORT=3001
FRONTEND_PORT=3000

# 資料庫
DATABASE_URL=sqlite:./data/dev.db

# 日誌
RUST_LOG=debug,sqlx=info

# CORS
VK_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:5173
```

### 生產環境 (.env.production)

```bash
# 伺服器
HOST=0.0.0.0
BACKEND_PORT=3001

# 資料庫
DATABASE_URL=sqlite:/opt/vibe-kanban/data/vibe-kanban.db

# 日誌
RUST_LOG=info

# CORS
VK_ALLOWED_ORIGINS=https://your-domain.com

# Sentry (可選)
SENTRY_AUTH_TOKEN=your-token-here
```

### Windows 生產環境 (.env)

```ini
# 伺服器
HOST=0.0.0.0
BACKEND_PORT=3001

# 資料庫
DATABASE_URL=sqlite:C:\vibe-kanban\data\vibe-kanban.db

# 日誌
RUST_LOG=info

# CORS
VK_ALLOWED_ORIGINS=http://localhost:3000
```

---

## 設定優先順序

環境變數的讀取優先順序 (由高到低):

1. 系統環境變數
2. `.env` 檔案
3. 程式內建預設值

---

## 安全性建議

1. **不要將 `.env` 檔案提交到版本控制**
   ```gitignore
   # .gitignore
   .env
   .env.local
   .env.production
   ```

2. **使用環境專屬的設定檔**
   - `.env.development` - 開發環境
   - `.env.production` - 生產環境

3. **敏感資訊**
   - `SENTRY_AUTH_TOKEN` 等敏感令牌應使用安全的方式管理
   - 考慮使用 Vault 或其他密鑰管理服務

---

*文件版本: 1.0*
*最後更新: 2026-01-28*
