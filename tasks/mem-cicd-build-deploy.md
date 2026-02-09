# CI/CD 編譯與部署 - 知識記憶檔案

## 專案基本資訊
- **專案名稱**: vibe-kanban
- **版本**: 0.0.159
- **後端技術**: Rust (Cargo workspace)
- **前端技術**: React + TypeScript + Vite
- **套件管理**: pnpm@10.13.1
- **Node.js 需求**: >=18

## Cargo Workspace 結構
- `crates/server` - API 伺服器
- `crates/db` - SQLx 資料庫模型與遷移
- `crates/executors` - 執行器
- `crates/services` - 服務層
- `crates/utils` - 工具函式
- `crates/local-deployment` - 本地部署
- `crates/deployment` - 部署相關
- `crates/remote` - 遠端部署
- `crates/review` - 審核功能

## 重要依賴
- tokio (async runtime)
- axum (web framework)
- git2 (Git 操作)
- reqwest + rustls (HTTP 客戶端)
- SQLx (資料庫)

---

## A 章節: 環境檢查與依賴安裝 (已完成)

### 環境版本
- Node.js: v22.17.0 (滿足 >=18 要求)
- pnpm: 10.13.1 (滿足 >=8 要求)
- Rust: 1.93.0-nightly
- Cargo: 1.93.0-nightly
- GCC: 9.4.0

### 重要發現: libsqlite3-sys 編譯問題
**問題**: bindgen 找不到 `stdarg.h`，導致 libsqlite3-sys 編譯失敗

**解決方案** (必須設定的環境變數):
```bash
export LIBSQLITE3_SYS_USE_PKG_CONFIG=1
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/9/include"
```

### Linux 系統必要套件
- libsqlite3-dev (SQLite 開發檔案)
- libclang (用於 bindgen)
- GCC (提供 stdarg.h)

---

## B 章節: 前端編譯測試 (已完成)

### 測試結果
- ✅ TypeScript 類型檢查：通過
- ✅ ESLint 檢查：通過 (無警告)
- ✅ 生產環境編譯：成功

### 編譯產出
```
dist/index.html       - 0.72 kB (gzip: 0.38 kB)
dist/assets/*.css     - 222.32 kB (gzip: 35.67 kB)
dist/assets/*.js      - 6,027.04 kB (gzip: 1,884.31 kB)
```

### 前端編譯命令
```bash
cd frontend && pnpm run build
```

### 警告與建議 (非致命)
1. **Tailwind CSS content 配置警告** - 建議檢查 tailwind.config.js 的 content 設定
2. **Sentry auth token 警告** - 生產環境需設定 SENTRY_AUTH_TOKEN
3. **Chunk 大小警告** - JS bundle 超過 500KB，建議實施 code splitting

### 前端技術棧
- React + TypeScript
- Vite 5.4.19
- Tailwind CSS
- Sentry (錯誤追蹤)

---

## C 章節: 後端編譯測試 (已完成)

### 測試結果
- ✅ Cargo check：通過
- ✅ Clippy lint：通過 (6 個警告，不影響功能)
- ✅ Rust 測試：全部通過
- ✅ Release 編譯：成功

### 編譯產出
```
target/release/server - 76MB (ELF 64-bit, stripped)
```

### 關鍵編譯環境變數 (Linux)
```bash
# 必須設定 (解決 GCC bug 與 bindgen 問題)
export CC=gcc-10
export CXX=g++-10
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/10/include"

# 編譯命令
cargo build --release
```

### 重要發現與解決方案

#### 問題 1: aws-lc-sys 編譯失敗 (GCC bug)
- **錯誤**: `COMPILER BUG DETECTED` - memcmp related bug
- **原因**: GCC 9 有已知 bug (gcc#95189)
- **解決**: 使用 `CC=gcc-10 CXX=g++-10`

#### 問題 2: bindgen 找不到 stdarg.h
- **錯誤**: `'stdarg.h' file not found`
- **解決**: `BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/10/include"`

#### 問題 3: SQLite 版本過舊
- **錯誤**: `undefined symbol: sqlite3_serialize`
- **原因**: 系統 SQLite 3.31 缺少 serialize 功能 (需要 3.36+)
- **解決**: 不設定 `LIBSQLITE3_SYS_USE_PKG_CONFIG`，使用 bundled SQLite

### Clippy 警告 (建議修復)
1. `clamp-like pattern without using clamp function`
2. 3 個可合併的 if 語句
3. `.as_ref().map(|s| s.as_str())` 可簡化
4. 函數參數過多 (8/7) - `crates/local-deployment/src/loop_tracker.rs:65`

---

## D 章節: 經驗整合與優化評估 (已完成)

### 批次處理優化建議

#### 可並行執行的任務
| 組合 | 任務 1 | 任務 2 | 說明 |
|------|--------|--------|------|
| 類型檢查 | `pnpm run frontend:check` | `cargo check --workspace` | 無依賴關係 |
| Lint 檢查 | `pnpm run frontend:lint` | `pnpm run backend:lint` | 無依賴關係 |
| 編譯階段 | `cargo build --release` | `cd frontend && pnpm run build` | 可並行 |

#### 已整合的便捷命令
- `pnpm run lint` - 同時執行前後端 lint
- `pnpm run check` - 同時執行前後端類型檢查

#### 建議的完整編譯腳本
```bash
#!/bin/bash
set -e

# 環境變數設定 (Linux)
export CC=gcc-10
export CXX=g++-10
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/10/include"

# 1. 並行檢查
pnpm run check &
pnpm run lint &
wait

# 2. 執行測試
cargo test --workspace

# 3. 並行編譯
cargo build --release &
(cd frontend && pnpm run build) &
wait

echo "✅ 編譯完成"
```

### 編譯時間參考 (Linux)
- 前端編譯: ~30 秒
- 後端 release 編譯 (首次): ~10-15 分鐘
- 後端 release 編譯 (增量): ~1-3 分鐘

---

## E 章節: Windows 環境相容性分析 (已完成)

### Windows 部署必要工具

| 工具 | 版本要求 | 用途 |
|------|---------|------|
| Visual Studio Build Tools | 2019+ | MSVC 編譯器 (Rust 原生套件) |
| CMake | 3.20+ | aws-lc-sys 編譯 |
| NASM | 最新版 | aws-lc-rs 組譯 |
| Git for Windows | 最新版 | Git 操作 + Git Bash |
| Node.js | 18+ | 前端編譯 |
| pnpm | 8+ | 套件管理 |
| Rust (MSVC target) | stable | 後端編譯 |

### 依賴相容性分析

| 依賴 | Windows 支援 | 備註 |
|------|-------------|------|
| git2 | ✅ 支援 | 需 VS Build Tools |
| aws-lc-rs (rustls) | ⚠️ 需設定 | MSVC + CMake + NASM |
| SQLite (bundled) | ✅ 支援 | MSVC 編譯器 |
| portable-pty | ✅ 支援 | Windows ConPTY |
| nix | ❌ Unix only | 已被 `#[cfg(unix)]` 保護 |

### 程式碼跨平台支援狀態

**已有條件編譯的檔案:**
- `executors/src/stdout_dup.rs` - `#[cfg(windows)]` 分支
- `executors/src/command.rs` - Windows 命令分割
- `local-deployment/src/container.rs` - Windows 處理
- `server/src/main.rs` - 信號處理
- `services/src/services/analytics.rs` - 作業系統偵測

**路徑處理:**
- 使用 `dunce` crate 處理 Windows UNC 路徑
- 標準 `std::path` API 跨平台相容

### 前端 Windows 注意事項

**npm scripts 問題:**
```json
"dev": "VITE_OPEN=${VITE_OPEN:-false} vite"
```
- 此為 Unix shell 語法，Windows cmd 不支援
- **建議**: 使用 PowerShell 或 Git Bash 執行

### Windows 編譯命令

```powershell
# PowerShell 中執行
rustup default stable-x86_64-pc-windows-msvc

# 前端編譯
cd frontend
pnpm install
pnpm run build

# 後端編譯
cargo build --release
```

### 注意事項
1. 必須使用 MSVC 工具鏈 (`x86_64-pc-windows-msvc`)
2. 防火牆可能需要允許 `server.exe` 網路存取
3. Windows 路徑長度限制 (260 字元) 可能影響深層目錄

---

## F 章節: 手動部署文件撰寫 (已完成)

### 已建立的文件

| 文件 | 路徑 | 內容摘要 |
|------|------|---------|
| Linux/macOS 部署指南 | `docs/deployment-linux-macos.md` | 完整的 Unix 系統部署步驟 |
| Windows 部署指南 | `docs/deployment-windows.md` | Windows 專屬部署指南 |
| 環境變數參考 | `docs/environment-variables.md` | 所有環境變數說明 |
| 常見問題解答 | `docs/troubleshooting.md` | 17 個常見問題與解決方案 |

### Linux/macOS 部署要點

**系統需求:**
- Node.js 18+, pnpm 8+
- Rust stable (最新版)
- GCC 10+ (Ubuntu/Debian)
- CMake 3.20+, NASM, libclang-dev

**關鍵環境變數:**
```bash
export CC=gcc-10
export CXX=g++-10
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/10/include"
```

**部署目錄結構:**
```
/opt/vibe-kanban/
├── server          # 後端執行檔
├── frontend/dist/  # 前端靜態檔案
├── data/           # 資料庫與資料
└── .env            # 環境設定
```

### Windows 部署要點

**必要工具:**
- Visual Studio Build Tools 2019+ (含 Desktop C++ 工作負載)
- CMake 3.20+
- NASM (Netwide Assembler)
- Git for Windows
- Node.js 18+ LTS
- Rust (MSVC 工具鏈)

**編譯環境:**
- 必須在 **Developer PowerShell for VS** 中執行
- 使用 `x86_64-pc-windows-msvc` target
- 前端 npm scripts 需在 Git Bash 或 PowerShell 中執行

**部署目錄結構:**
```
C:\vibe-kanban\
├── server.exe      # 後端執行檔
├── frontend\dist\  # 前端靜態檔案
├── data\           # 資料庫與資料
└── .env            # 環境設定
```

### 環境變數分類

| 類別 | 變數範例 | 說明 |
|------|---------|------|
| 伺服器 | `HOST`, `PORT`, `VK_ALLOWED_ORIGINS` | 網路設定 |
| 資料庫 | `DATABASE_URL` | SQLite 連線路徑 |
| 日誌 | `RUST_LOG` | 日誌等級控制 |
| 安全性 | `SESSION_SECRET` | Session 加密金鑰 |
| 編譯 | `CC`, `CXX`, `BINDGEN_EXTRA_CLANG_ARGS` | 編譯器設定 |

### 常見問題快速索引

| # | 問題 | 解決方向 |
|---|-----|---------|
| 1 | stdarg.h not found | 設定 `BINDGEN_EXTRA_CLANG_ARGS` |
| 2 | GCC compiler bug | 升級 GCC 10 並設定 `CC`/`CXX` |
| 3 | SQLite serialize 缺失 | 使用 bundled SQLite |
| 4 | Windows LINK error | 在 VS Developer PowerShell 執行 |
| 5 | CMake not found | 安裝 CMake 並加入 PATH |
| 6 | NASM not found | 安裝 NASM 並加入 PATH |
| 7 | 記憶體不足 | 使用 `CARGO_BUILD_JOBS=2` |
| 8 | Address in use | 檢查並結束佔用端口的程式 |
| 9-17 | 其他問題 | 詳見 `docs/troubleshooting.md` |

### 驗證部署成功

```bash
# 測試 API
curl http://localhost:3001/api/health

# 預期回應
{"status":"ok"}
```

---
*此文件將隨任務進行動態更新*
