# Vibe Kanban - 部署指南總覽

本文件為 Vibe Kanban 專案的部署文件索引，彙整所有平台的部署步驟與注意事項。

## 📚 文件目錄

| 文件 | 說明 |
|------|------|
| [Linux/macOS 部署指南](deployment-linux-macos.md) | Ubuntu/Debian/macOS 完整部署步驟 |
| [Windows 部署指南](deployment-windows.md) | Windows 10/11 完整部署步驟 |
| [環境變數參考](environment-variables.md) | 所有可用環境變數的詳細說明 |
| [常見問題解答](troubleshooting.md) | 編譯與部署問題的解決方案 |

---

## 🚀 快速開始

### 系統需求

| 元件 | 最低版本 | 建議版本 |
|------|---------|---------|
| Node.js | 18.x | 22.x LTS |
| pnpm | 8.x | 10.x |
| Rust | stable | latest stable |
| GCC (Linux) | 10 | 10+ |
| Visual Studio Build Tools (Windows) | 2019 | 2022 |

### 平台選擇

```
選擇您的部署平台：

┌─────────────────────────────────────────────────────────┐
│                                                          │
│  🐧 Linux / 🍎 macOS                                     │
│  → 請參閱 deployment-linux-macos.md                      │
│                                                          │
│  🪟 Windows                                              │
│  → 請參閱 deployment-windows.md                          │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 📦 專案架構

```
vibe-kanban/
├── crates/                 # Rust 後端 (Cargo workspace)
│   ├── server/            # API 伺服器主程式
│   ├── db/                # 資料庫模型 (SQLx + SQLite)
│   ├── executors/         # 程式碼執行器
│   ├── services/          # 服務層
│   └── ...                # 其他 crates
├── frontend/              # React + TypeScript 前端 (Vite)
├── shared/                # 共享型別定義
├── docs/                  # 文件
└── tasks/                 # 任務追蹤
```

### 編譯產出

| 平台 | 後端 | 前端 |
|------|------|------|
| Linux/macOS | `target/release/server` | `frontend/dist/` |
| Windows | `target/release/server.exe` | `frontend/dist/` |

---

## ⚙️ 編譯步驟總結

### 1. 環境準備

**Linux (Ubuntu/Debian):**
```bash
# 安裝依賴
sudo apt update
sudo apt install -y build-essential gcc-10 g++-10 cmake nasm libclang-dev

# 設定編譯器
export CC=gcc-10
export CXX=g++-10
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/10/include"
```

**Windows:**
```powershell
# 安裝必要工具
winget install Microsoft.VisualStudio.2022.BuildTools
winget install Kitware.CMake
winget install NASM.NASM
winget install Schniz.fnm  # Node.js 版本管理
winget install OpenJS.NodeJS.LTS

# 在 Developer PowerShell for VS 中執行後續步驟
```

### 2. 安裝專案依賴

```bash
# 前端依賴
pnpm install

# Rust 依賴 (自動)
# cargo build 會自動下載
```

### 3. 編譯前端

```bash
cd frontend
pnpm run build
```

### 4. 編譯後端

```bash
cargo build --release
```

### 5. 部署

將 `target/release/server` (或 `server.exe`) 和 `frontend/dist/` 複製到部署目錄，設定環境變數後執行。

---

## 🔧 關鍵環境變數

| 變數 | 預設值 | 說明 |
|------|-------|------|
| `HOST` | `127.0.0.1` | 伺服器監聽位址 |
| `PORT` | `3001` | 伺服器監聽埠號 |
| `DATABASE_URL` | `sqlite:./data/vibe-kanban.db` | 資料庫連線 |
| `VK_ALLOWED_ORIGINS` | `http://localhost:3000` | CORS 允許來源 |
| `RUST_LOG` | `info` | 日誌等級 |

完整環境變數列表請參閱 [environment-variables.md](environment-variables.md)。

---

## ⚠️ 常見問題

### 編譯問題

| 問題 | 解決方案 |
|------|---------|
| `stdarg.h` not found | 設定 `BINDGEN_EXTRA_CLANG_ARGS` |
| GCC compiler bug | 使用 GCC 10+，設定 `CC=gcc-10` |
| SQLite serialize 缺失 | 使用 bundled SQLite (不設定 `LIBSQLITE3_SYS_USE_PKG_CONFIG`) |
| Windows LINK error | 在 Developer PowerShell for VS 執行 |

更多問題請參閱 [troubleshooting.md](troubleshooting.md)。

---

## 📊 經驗總結

### E 章節 (Windows 相容性分析) 重點

1. **程式碼已具備跨平台支援**
   - `#[cfg(unix)]` / `#[cfg(windows)]` 條件編譯
   - `dunce` crate 處理 Windows UNC 路徑
   - `winsplit` 處理 Windows 命令分割

2. **Windows 編譯關鍵**
   - 必須使用 MSVC 工具鏈
   - 需要安裝 CMake + NASM (aws-lc-rs 依賴)
   - 建議在 Developer PowerShell for VS 中編譯

3. **前端在 Windows 的注意事項**
   - npm scripts 使用 Unix shell 語法
   - 建議使用 Git Bash 或 PowerShell 執行

### F 章節 (文件撰寫) 重點

1. **文件涵蓋範圍**
   - 兩大平台 (Linux/macOS, Windows) 完整部署指南
   - 所有環境變數的詳細說明
   - 17 個常見問題與解決方案

2. **部署驗證方法**
   ```bash
   curl http://localhost:3001/api/health
   # 預期: {"status":"ok"}
   ```

3. **建議的生產環境設定**
   - 使用 systemd (Linux) 或 NSSM (Windows) 管理服務
   - 設定反向代理 (Nginx/Caddy)
   - 啟用 HTTPS

---

## 📝 版本資訊

- **專案版本**: 0.0.159
- **文件版本**: 1.0
- **最後更新**: 2026-01-28

---

*如有問題，請參閱 [troubleshooting.md](troubleshooting.md) 或提交 GitHub Issue。*
