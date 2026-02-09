# Vibe Kanban - Linux/macOS 手動部署指南

本文件提供在 Linux 和 macOS 系統上從原始碼編譯並部署 Vibe Kanban 的完整步驟。

## 目錄

1. [系統需求](#系統需求)
2. [前置準備](#前置準備)
3. [編譯步驟](#編譯步驟)
4. [部署步驟](#部署步驟)
5. [驗證部署](#驗證部署)

---

## 系統需求

### 硬體需求
- CPU: 2 核心以上
- RAM: 4GB 以上 (編譯時建議 8GB)
- 硬碟: 10GB 可用空間

### 軟體需求

| 工具 | 最低版本 | 用途 |
|------|---------|------|
| Node.js | 18.0+ | 前端編譯 |
| pnpm | 8.0+ | 套件管理 |
| Rust | 1.75+ (stable) | 後端編譯 |
| GCC | 10+ | C 編譯器 (Linux) |
| Git | 2.0+ | 版本控制 |

---

## 前置準備

### Linux (Ubuntu/Debian)

```bash
# 1. 安裝系統套件
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev libsqlite3-dev \
    libclang-dev gcc-10 g++-10 cmake git curl

# 2. 安裝 Node.js (使用 nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 22
nvm use 22

# 3. 安裝 pnpm
npm install -g pnpm

# 4. 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
```

### macOS

```bash
# 1. 安裝 Homebrew (如果尚未安裝)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. 安裝必要工具
brew install node pnpm git cmake

# 3. 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
```

### 驗證環境

```bash
# 檢查所有工具版本
node --version    # 應顯示 v18+ 或 v22+
pnpm --version    # 應顯示 8+ 或 10+
rustc --version   # 應顯示 1.75+ 或更新
cargo --version
git --version
```

---

## 編譯步驟

### 步驟 1: 取得原始碼

```bash
git clone https://github.com/your-org/vibe-kanban.git
cd vibe-kanban
```

### 步驟 2: 設定環境變數 (Linux)

**重要**: Linux 需要設定以下環境變數以避免編譯錯誤：

```bash
# 解決 GCC 9 的已知 bug 與 bindgen 問題
export CC=gcc-10
export CXX=g++-10
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/10/include"
```

> **注意**: macOS 通常不需要這些設定

### 步驟 3: 安裝前端依賴

```bash
pnpm install
```

### 步驟 4: 編譯前端

```bash
cd frontend
pnpm run build
cd ..
```

成功後，編譯產出會在 `frontend/dist/` 目錄。

### 步驟 5: 編譯後端 (Release 模式)

```bash
cargo build --release
```

> **首次編譯時間**: 約 10-15 分鐘
> **增量編譯時間**: 約 1-3 分鐘

成功後，執行檔位於 `target/release/server`。

### 步驟 6: 執行測試 (可選)

```bash
cargo test --workspace
```

---

## 部署步驟

### 步驟 1: 建立部署目錄

```bash
# 建立部署目錄
sudo mkdir -p /opt/vibe-kanban
sudo mkdir -p /opt/vibe-kanban/data
sudo mkdir -p /opt/vibe-kanban/static

# 設定權限 (以您的使用者為例)
sudo chown -R $USER:$USER /opt/vibe-kanban
```

### 步驟 2: 複製檔案

```bash
# 複製後端執行檔
cp target/release/server /opt/vibe-kanban/

# 複製前端靜態檔案
cp -r frontend/dist/* /opt/vibe-kanban/static/
```

### 步驟 3: 建立環境設定檔

```bash
cat > /opt/vibe-kanban/.env << 'EOF'
# 伺服器設定
HOST=0.0.0.0
BACKEND_PORT=3001
FRONTEND_PORT=3000

# 資料庫位置
DATABASE_URL=sqlite:///opt/vibe-kanban/data/vibe-kanban.db

# 日誌等級
RUST_LOG=info

# 允許的來源 (CORS)
VK_ALLOWED_ORIGINS=http://localhost:3000
EOF
```

### 步驟 4: 啟動服務

```bash
cd /opt/vibe-kanban
./server
```

### 步驟 5: 設定為系統服務 (可選)

建立 systemd 服務檔案：

```bash
sudo cat > /etc/systemd/system/vibe-kanban.service << 'EOF'
[Unit]
Description=Vibe Kanban Server
After=network.target

[Service]
Type=simple
User=vibe-kanban
WorkingDirectory=/opt/vibe-kanban
EnvironmentFile=/opt/vibe-kanban/.env
ExecStart=/opt/vibe-kanban/server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# 啟用並啟動服務
sudo systemctl daemon-reload
sudo systemctl enable vibe-kanban
sudo systemctl start vibe-kanban
```

---

## 驗證部署

### 檢查服務狀態

```bash
# 如果使用 systemd
sudo systemctl status vibe-kanban

# 檢查日誌
journalctl -u vibe-kanban -f
```

### 測試 API

```bash
# 健康檢查
curl http://localhost:3001/api/health

# 應返回 {"status": "ok"} 或類似響應
```

### 存取前端

開啟瀏覽器，訪問 `http://localhost:3000` 或 `http://your-server-ip:3000`

---

## 常見問題

### Q: 編譯時出現 `stdarg.h not found`

**A**: 設定環境變數：
```bash
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/10/include"
```

### Q: 編譯時出現 `COMPILER BUG DETECTED`

**A**: 使用 GCC 10 或更新版本：
```bash
export CC=gcc-10
export CXX=g++-10
```

### Q: SQLite 符號錯誤 (`sqlite3_serialize`)

**A**: 使用 bundled SQLite（不設定 `LIBSQLITE3_SYS_USE_PKG_CONFIG`）

---

*文件版本: 1.0*
*最後更新: 2026-01-28*
