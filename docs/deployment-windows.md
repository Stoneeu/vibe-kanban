# Vibe Kanban - Windows 手動部署指南

本文件提供在 Windows 系統上從原始碼編譯並部署 Vibe Kanban 的完整步驟。

## 目錄

1. [系統需求](#系統需求)
2. [前置準備](#前置準備)
3. [編譯步驟](#編譯步驟)
4. [部署步驟](#部署步驟)
5. [驗證部署](#驗證部署)
6. [Windows 特定注意事項](#windows-特定注意事項)

---

## 系統需求

### 硬體需求
- CPU: 2 核心以上
- RAM: 8GB 以上 (編譯時需要較多記憶體)
- 硬碟: 15GB 可用空間 (包含 Visual Studio Build Tools)

### 軟體需求

| 工具 | 版本 | 用途 |
|------|------|------|
| Windows | 10/11 (64-bit) | 作業系統 |
| Visual Studio Build Tools | 2019+ | MSVC 編譯器 |
| CMake | 3.20+ | aws-lc-sys 編譯 |
| NASM | 最新版 | aws-lc-rs 組譯 |
| Node.js | 18+ | 前端編譯 |
| pnpm | 8+ | 套件管理 |
| Rust (MSVC) | stable | 後端編譯 |
| Git for Windows | 最新版 | 版本控制 |

---

## 前置準備

### 步驟 1: 安裝 Visual Studio Build Tools

1. 下載 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)

2. 執行安裝程式，選擇以下工作負載：
   - **「使用 C++ 的桌面開發」**

3. 在「個別元件」中確認已選取：
   - MSVC v143 - VS 2022 C++ x64/x86 建置工具
   - Windows 10/11 SDK
   - C++ CMake 工具

### 步驟 2: 安裝 CMake

```powershell
# 使用 winget 安裝
winget install Kitware.CMake

# 或從官網下載: https://cmake.org/download/
```

### 步驟 3: 安裝 NASM

```powershell
# 使用 winget 安裝
winget install NASM.NASM

# 或從官網下載: https://www.nasm.us/
```

> **重要**: 確保 NASM 已加入系統 PATH

### 步驟 4: 安裝 Node.js 和 pnpm

```powershell
# 使用 winget 安裝 Node.js
winget install OpenJS.NodeJS.LTS

# 安裝 pnpm
npm install -g pnpm
```

### 步驟 5: 安裝 Git for Windows

```powershell
winget install Git.Git
```

> **建議**: 安裝時選擇「使用 Git Bash」作為預設終端

### 步驟 6: 安裝 Rust (MSVC 工具鏈)

```powershell
# 下載並執行 rustup-init.exe
# 從 https://rustup.rs 下載

# 安裝完成後，確認使用 MSVC 工具鏈
rustup default stable-x86_64-pc-windows-msvc
```

### 驗證環境

開啟 **PowerShell** 或 **Developer Command Prompt**：

```powershell
node --version     # 應顯示 v18+ 或 v22+
pnpm --version     # 應顯示 8+ 或 10+
rustc --version    # 應顯示 1.75+ 或更新
cargo --version
cmake --version
nasm -v
git --version
```

---

## 編譯步驟

> **重要**: 建議在 **Developer PowerShell for VS 2022** 或 **Git Bash** 中執行以下步驟

### 步驟 1: 取得原始碼

```powershell
git clone https://github.com/your-org/vibe-kanban.git
cd vibe-kanban
```

### 步驟 2: 安裝前端依賴

```powershell
pnpm install
```

### 步驟 3: 編譯前端

**使用 PowerShell:**
```powershell
cd frontend
pnpm run build
cd ..
```

**使用 Git Bash:**
```bash
cd frontend
pnpm run build
cd ..
```

成功後，編譯產出會在 `frontend\dist\` 目錄。

### 步驟 4: 編譯後端 (Release 模式)

```powershell
cargo build --release
```

> **首次編譯時間**: 約 15-25 分鐘 (Windows 通常較慢)
> **增量編譯時間**: 約 2-5 分鐘

成功後，執行檔位於 `target\release\server.exe`。

### 步驟 5: 執行測試 (可選)

```powershell
cargo test --workspace
```

---

## 部署步驟

### 步驟 1: 建立部署目錄

```powershell
# 建立部署目錄
New-Item -ItemType Directory -Force -Path "C:\vibe-kanban"
New-Item -ItemType Directory -Force -Path "C:\vibe-kanban\data"
New-Item -ItemType Directory -Force -Path "C:\vibe-kanban\static"
```

### 步驟 2: 複製檔案

```powershell
# 複製後端執行檔
Copy-Item "target\release\server.exe" -Destination "C:\vibe-kanban\"

# 複製前端靜態檔案
Copy-Item -Recurse "frontend\dist\*" -Destination "C:\vibe-kanban\static\"
```

### 步驟 3: 建立環境設定檔

建立 `C:\vibe-kanban\.env` 檔案：

```ini
# 伺服器設定
HOST=0.0.0.0
BACKEND_PORT=3001
FRONTEND_PORT=3000

# 資料庫位置
DATABASE_URL=sqlite:C:\vibe-kanban\data\vibe-kanban.db

# 日誌等級
RUST_LOG=info

# 允許的來源 (CORS)
VK_ALLOWED_ORIGINS=http://localhost:3000
```

### 步驟 4: 啟動服務

```powershell
cd C:\vibe-kanban
.\server.exe
```

### 步驟 5: 設定為 Windows 服務 (可選)

使用 [NSSM](https://nssm.cc/) 將應用程式安裝為 Windows 服務：

```powershell
# 下載 NSSM
# https://nssm.cc/download

# 安裝服務
nssm install VibeKanban "C:\vibe-kanban\server.exe"
nssm set VibeKanban AppDirectory "C:\vibe-kanban"
nssm set VibeKanban AppEnvironmentExtra "RUST_LOG=info"

# 啟動服務
nssm start VibeKanban
```

### 步驟 6: 設定 Windows 防火牆

```powershell
# 允許後端 API 端口
New-NetFirewallRule -DisplayName "Vibe Kanban API" -Direction Inbound -Port 3001 -Protocol TCP -Action Allow

# 允許前端端口 (如果分開部署)
New-NetFirewallRule -DisplayName "Vibe Kanban Frontend" -Direction Inbound -Port 3000 -Protocol TCP -Action Allow
```

---

## 驗證部署

### 檢查服務狀態

```powershell
# 如果使用 NSSM
nssm status VibeKanban
```

### 測試 API

```powershell
# 使用 PowerShell
Invoke-RestMethod -Uri "http://localhost:3001/api/health"

# 或使用 curl (如果已安裝)
curl http://localhost:3001/api/health
```

### 存取前端

開啟瀏覽器，訪問 `http://localhost:3000`

---

## Windows 特定注意事項

### 1. 路徑長度限制

Windows 預設有 260 字元的路徑長度限制。如果專案路徑較深，可能會導致編譯失敗。

**解決方案:**
- 使用較短的路徑 (如 `C:\vk\`)
- 或啟用長路徑支援：
  ```powershell
  # 以管理員身份執行
  New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
    -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
  ```

### 2. npm scripts 相容性

專案中的某些 npm scripts 使用 Unix shell 語法，在 Windows CMD 中可能無法執行。

**解決方案:**
- 使用 **Git Bash** 執行 npm scripts
- 或使用 **PowerShell** (部分腳本相容)

### 3. 終端機選擇

| 終端機 | npm scripts | Cargo | 建議用途 |
|--------|-------------|-------|---------|
| CMD | ❌ 有限 | ✅ | 不推薦 |
| PowerShell | ⚠️ 部分 | ✅ | Cargo 編譯 |
| Git Bash | ✅ | ✅ | 推薦使用 |
| Developer PS | ✅ | ✅ | 最佳選擇 |

### 4. 防毒軟體

某些防毒軟體可能會減慢編譯速度或誤報 Rust 編譯的執行檔。

**建議:**
- 將專案目錄加入防毒軟體的排除清單
- 或在編譯時暫時停用即時掃描

---

## 常見問題

### Q: 編譯時出現 LINK error

**A**: 確保已安裝完整的 Visual Studio Build Tools，並在 Developer Command Prompt 中執行編譯。

### Q: CMake 找不到

**A**: 確保 CMake 已加入系統 PATH，或重新開啟終端機。

### Q: NASM 相關錯誤

**A**:
1. 確認 NASM 已正確安裝
2. 確認 `nasm.exe` 所在目錄已加入 PATH
3. 重新開啟終端機

### Q: 前端編譯時出現權限錯誤

**A**: 以管理員身份執行 PowerShell，或確保對專案目錄有完整的讀寫權限。

---

*文件版本: 1.0*
*最後更新: 2026-01-28*
