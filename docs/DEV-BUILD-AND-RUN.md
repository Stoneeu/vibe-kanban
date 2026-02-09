# Vibe Kanban - 從原始碼編譯與執行指南

本文件說明如何從原始碼編譯 Vibe Kanban 並在指定的 IP 與 PORT 上執行。

---

## 🚀 快速開始 (TL;DR)

```bash
# 設定編譯環境
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"
sudo apt install gcc-10 g++-10
export CC=gcc-10
export CXX=g++-10

# 編譯 Release 版本
cargo build --bin server --release

# 進入前端目錄並編譯
cd frontend && pnpm install && pnpm run build && cd ..

# 啟動伺服器
HOST=0.0.0.0 PORT=9998 RUST_LOG=debug ./target/release/server
```

---


## ⚠️ 重要概念：Debug vs Release 資料目錄差異

### 為什麼 Debug 版本看不到原有設定？

Vibe Kanban 使用不同的資料目錄來區分開發環境和生產環境：

| 編譯模式 | 資料目錄 | 說明 |
|---------|---------|------|
| **Debug** | `<專案目錄>/dev_assets/` | 開發測試用，避免影響正式資料 |
| **Release** | `~/.local/share/vibe-kanban/` | 正式使用，與 npm 版本共用 |

### 原始碼解析 (`crates/utils/src/assets.rs`)

```rust
pub fn asset_dir() -> std::path::PathBuf {
    let path = if cfg!(debug_assertions) {
        // Debug 模式：使用專案內的 dev_assets/
        std::path::PathBuf::from(PROJECT_ROOT).join("../../dev_assets")
    } else {
        // Release 模式：使用 ~/.local/share/vibe-kanban/
        ProjectDirs::from("ai", "bloop", "vibe-kanban")
            .expect("OS didn't give us a home directory")
            .data_dir()
            .to_path_buf()
    };
    // ...
}
```

`cfg!(debug_assertions)` 是 Rust 的編譯時檢查：
- 當使用 `cargo build`（預設 debug）時，此值為 `true`
- 當使用 `cargo build --release` 時，此值為 `false`

### 各平台 Release 資料目錄

| 平台 | 路徑 |
|------|------|
| **Linux** | `~/.local/share/vibe-kanban/` |
| **macOS** | `~/Library/Application Support/ai.bloop.vibe-kanban/` |
| **Windows** | `%APPDATA%\bloop\vibe-kanban\` |

---

## 🔧 解決方案：讓 Debug 版本使用正式設定

### 方案一：使用 Symlink（推薦）

將 `dev_assets` 指向正式資料目錄：

```bash
cd /var/tmp/vibe-kanban/worktrees/3701-dev-002-copilot/vibe-kanban

# 備份現有 dev_assets（如果有）
mv dev_assets dev_assets.bak 2>/dev/null || true

# 建立 symlink 指向正式資料目錄
ln -sf ~/.local/share/vibe-kanban dev_assets

# 驗證
ls -la dev_assets/
```

### 方案二：編譯 Release 版本

直接使用 release 模式編譯：

```bash
cargo build --bin server --release
./target/release/server --host 0.0.0.0 --port 9998
```

⚠️ **注意**：GCC 9.x 系列有已知的 memcmp bug ([GCC Bug 95189](https://gcc.gnu.org/bugzilla/show_bug.cgi?id=95189))，會導致 release 編譯失敗。解決方案見下方。

### 方案二-A：解決 GCC 9.x Release 編譯問題

如果遇到以下錯誤：
```
COMPILER BUG DETECTED
Your compiler (cc) is not supported due to a memcmp related bug
```

**解決方案：升級 GCC 到 10+ 或使用 Clang**

```bash
# 方法 1：安裝並使用 GCC 10
sudo apt install gcc-10 g++-10
export CC=gcc-10
export CXX=g++-10
cargo build --bin server --release

# 方法 2：安裝並使用 Clang
sudo apt install clang
export CC=clang
export CXX=clang++
cargo build --bin server --release
```

如果無法升級編譯器，使用**方案一（Symlink）** 是最可靠的替代方案

### 方案三：複製資料

將正式資料複製到 dev_assets：

```bash
mkdir -p dev_assets
cp ~/.local/share/vibe-kanban/*.json dev_assets/
cp -r ~/.local/share/vibe-kanban/project_* dev_assets/
```

---

## 前置需求

- **Rust** (建議 1.75+)
- **Node.js** (建議 v18+)
- **pnpm** (建議 v8+)
- **GCC** (用於編譯 SQLite 綁定)

## 環境變數設定

在某些 Linux 系統上，編譯時可能會遇到 `stdarg.h` 找不到的問題。請先設定以下環境變數：

```bash
# 找到 stdarg.h 的路徑
find /usr -name "stdarg.h" 2>/dev/null

# 設定環境變數（根據你的系統調整路徑）
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"
```

## 快速開始

### 方法一：使用開發模式（推薦用於開發測試）

```bash
# 1. 進入專案目錄
cd /var/tmp/vibe-kanban/worktrees/3701-dev-002-copilot/vibe-kanban

# 2. 安裝依賴
pnpm install

# 3. 設定環境變數（解決編譯問題）
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"

# 4. 啟動開發模式（前後端同時啟動）
pnpm run dev
```

開發模式會自動分配端口，查看 `.dev-env` 檔案獲取實際使用的端口。

### 方法二：分別啟動前後端（可指定 IP 與 PORT）

#### 步驟 1：編譯後端

```bash
# 進入專案目錄
cd /var/tmp/vibe-kanban/worktrees/3701-dev-002-copilot/vibe-kanban

# 設定編譯環境變數
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"

# 編譯後端（debug 模式，編譯較快）
cargo build --bin server

# 或編譯 release 模式（優化後的版本）
cargo build --bin server --release
```

#### 步驟 2：啟動後端伺服器

```bash
# Debug 模式執行
HOST=0.0.0.0 PORT=9999 ./target/debug/server

# 或 Release 模式執行
HOST=0.0.0.0 PORT=9999 ./target/release/server

# 使用命令列參數指定端口（-p 或 --port）
./target/debug/server -p 9999

# 同時指定 host
./target/debug/server --host 0.0.0.0 --port 9999
```

#### 步驟 3：編譯並啟動前端

```bash
# 在另一個終端視窗

# 進入前端目錄
cd /var/tmp/vibe-kanban/worktrees/3701-dev-002-copilot/vibe-kanban/frontend

# 安裝依賴（如果還沒安裝）
pnpm install

# 開發模式啟動（指定端口）
VITE_BACKEND_URL=http://localhost:9999 pnpm run dev -- --port 3000 --host 0.0.0.0

# 或者建置生產版本
pnpm run build

# 前端建置後，後端會自動提供靜態檔案
```

## 完整編譯與執行腳本

建立一個 `run-dev.sh` 腳本：

```bash
#!/bin/bash
set -e

# 設定變數
export HOST="${HOST:-0.0.0.0}"
export PORT="${PORT:-9999}"
export FRONTEND_PORT="${FRONTEND_PORT:-3000}"
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"

PROJECT_DIR="/var/tmp/vibe-kanban/worktrees/3701-dev-002-copilot/vibe-kanban"

cd "$PROJECT_DIR"

echo "🔨 編譯後端..."
cargo build --bin server

echo "🔨 編譯前端..."
cd frontend
pnpm install
pnpm run build
cd ..

echo "🚀 啟動伺服器在 http://${HOST}:${PORT}"
./target/debug/server --host "$HOST" --port "$PORT"
```

執行方式：

```bash
# 使用預設值
./run-dev.sh

# 或指定自訂值
HOST=0.0.0.0 PORT=8888 ./run-dev.sh
```

## 驗證 Copilot Loop 新功能

1. 啟動伺服器後，在瀏覽器開啟：`http://localhost:9999`

2. 進入 **Settings** → **Agents**

3. 在 **Agent** 下拉選單選擇 **COPILOT**

4. 向下滾動，應該能看到以下新欄位：
   - **Loop Enabled** - 啟用自動循環直到任務完成
   - **Max Iterations** - 最大循環次數（預設 5，最大 100）
   - **Completion Promise** - 完成訊號字串（例如 `<promise>COMPLETE</promise>`）

5. 修改設定值並點擊 **Save Configuration** 儲存

## 生成 TypeScript 類型

如果修改了 Rust 結構體，需要重新生成 TypeScript 類型：

```bash
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"
pnpm run generate-types
```

## 執行測試

```bash
# 執行所有測試
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"
cargo test --workspace

# 只執行 loop_tracker 相關測試
cargo test --package local-deployment loop_tracker
```

## 常見問題

### 1. `stdarg.h` 找不到

```
sqlite3/sqlite3.h:35:10: fatal error: 'stdarg.h' file not found
```

**解決方案：**
```bash
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"
```

### 2. `pnpm` 找不到

```bash
npm install -g pnpm
```

### 3. 後端編譯成功但前端顯示 "Please build the frontend"

前端還沒編譯，執行：
```bash
cd frontend && pnpm run build
```

### 4. 端口被佔用

```bash
# 查看佔用端口的程序
lsof -i :9999

# 或使用其他端口
PORT=8888 ./target/debug/server
```

## 相關檔案

| 檔案 | 說明 |
|------|------|
| `crates/executors/src/executors/copilot.rs` | Copilot 執行器定義（含新 Loop 欄位）|
| `crates/local-deployment/src/loop_tracker.rs` | Loop 狀態追蹤器 |
| `crates/local-deployment/src/container.rs` | 容器服務（含 Loop 處理邏輯）|
| `crates/utils/src/assets.rs` | 資料目錄路徑解析（Debug vs Release）|
| `shared/schemas/copilot.json` | 自動生成的 JSON Schema |
| `frontend/src/pages/settings/AgentSettings.tsx` | 前端設定頁面 |

---

## 📋 完整一步一步執行流程

### 場景：測試新編譯的版本，同時使用現有的 npm vibe-kanban 設定

#### Step 1：進入專案目錄

```bash
cd /var/tmp/vibe-kanban/worktrees/3701-dev-002-copilot/vibe-kanban
```

#### Step 2：設定編譯環境變數

```bash
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"
```

#### Step 3：建立 Symlink 讓 Debug 版本使用正式設定

```bash
# 備份現有 dev_assets（如果有）
mv dev_assets dev_assets.bak 2>/dev/null || true

# 建立 symlink
ln -sf ~/.local/share/vibe-kanban dev_assets

# 驗證 symlink 正確
ls -la dev_assets/
# 應該看到: dev_assets -> /home/<user>/.local/share/vibe-kanban
```

#### Step 4：編譯後端

```bash
cargo build --bin server

# 編譯成功後，執行檔位於：
# ./target/debug/server
```

#### Step 5：啟動伺服器

```bash
# 基本啟動（使用不同端口避免與 npm 版本衝突）
HOST=0.0.0.0 PORT=9998 ./target/debug/server

# 啟用 debug 日誌
HOST=0.0.0.0 PORT=9998 RUST_LOG=debug ./target/debug/server

# 啟用特定模組的 debug 日誌
HOST=0.0.0.0 PORT=9998 RUST_LOG=local_deployment=debug ./target/debug/server
```

#### Step 6：驗證伺服器狀態

成功啟動後，應該看到類似訊息：

```
[INFO] executors::profile: Loaded user profile overrides from profiles.json
[INFO] server: Server running on http://0.0.0.0:9998
Found 4 projects
```

#### Step 7：在瀏覽器開啟

```
http://localhost:9998
```

進入 **Settings** → **Agents** → 選擇 **COPILOT**，驗證新的 Loop 設定欄位。

---

## 🔍 Debug 日誌級別

使用 `RUST_LOG` 環境變數控制日誌級別：

```bash
# 全局 debug
RUST_LOG=debug ./target/debug/server

# 全局 trace（最詳細）
RUST_LOG=trace ./target/debug/server

# 特定模組 debug
RUST_LOG=local_deployment=debug,executors=debug ./target/debug/server

# 混合級別
RUST_LOG=info,local_deployment::loop_tracker=debug ./target/debug/server
```

日誌級別（從少到多）：`error` < `warn` < `info` < `debug` < `trace`

---

## 🔄 快速重編譯腳本

建立 `~/bin/vk-dev.sh`：

```bash
#!/bin/bash
set -e

PROJECT_DIR="/var/tmp/vibe-kanban/worktrees/3701-dev-002-copilot/vibe-kanban"
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"

cd "$PROJECT_DIR"

# 確保 symlink 存在
if [ ! -L "dev_assets" ]; then
    echo "Creating symlink to production data..."
    mv dev_assets dev_assets.bak 2>/dev/null || true
    ln -sf ~/.local/share/vibe-kanban dev_assets
fi

echo "🔨 Building..."
cargo build --bin server

echo "🚀 Starting server on http://0.0.0.0:${PORT:-9998}"
HOST=0.0.0.0 PORT=${PORT:-9998} RUST_LOG=${RUST_LOG:-info} ./target/debug/server
```

使用方式：

```bash
chmod +x ~/bin/vk-dev.sh

# 預設啟動
~/bin/vk-dev.sh

# 指定端口和日誌級別
PORT=8888 RUST_LOG=debug ~/bin/vk-dev.sh
```
