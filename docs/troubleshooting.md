# Vibe Kanban - 常見問題與解決方案 (FAQ)

本文件收集了編譯和部署過程中常見的問題及其解決方案。

## 目錄

1. [編譯問題](#編譯問題)
2. [執行時問題](#執行時問題)
3. [網路與連線問題](#網路與連線問題)
4. [資料庫問題](#資料庫問題)
5. [平台特定問題](#平台特定問題)

---

## 編譯問題

### 1. `stdarg.h` file not found

**錯誤訊息:**
```
error: 'stdarg.h' file not found
```

**原因:** bindgen 找不到 C 標準函式庫標頭檔。

**解決方案 (Linux):**
```bash
# 設定 clang 的 include 路徑
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/10/include"

# 然後重新編譯
cargo clean
cargo build --release
```

---

### 2. COMPILER BUG DETECTED (aws-lc-sys)

**錯誤訊息:**
```
COMPILER BUG DETECTED! memcmp related bug (gcc#95189)
```

**原因:** GCC 9 有已知的編譯器 bug。

**解決方案 (Linux):**
```bash
# 安裝 GCC 10
sudo apt install gcc-10 g++-10

# 設定編譯器
export CC=gcc-10
export CXX=g++-10

# 清除並重新編譯
cargo clean
cargo build --release
```

---

### 3. undefined symbol: sqlite3_serialize

**錯誤訊息:**
```
undefined symbol: sqlite3_serialize
undefined symbol: sqlite3_deserialize
```

**原因:** 系統 SQLite 版本過舊 (需要 3.36+)。

**解決方案:**
```bash
# 不要使用系統 SQLite，改用 bundled 版本
# 確保不設定這個環境變數：
unset LIBSQLITE3_SYS_USE_PKG_CONFIG

# 清除並重新編譯
cargo clean
cargo build --release
```

---

### 4. LINK : fatal error (Windows)

**錯誤訊息:**
```
LINK : fatal error LNK1181: cannot open input file
```

**原因:** 缺少 Visual Studio Build Tools 或未在正確的終端機執行。

**解決方案:**
1. 安裝完整的 Visual Studio Build Tools 2019+
2. 在 **Developer PowerShell for VS** 中執行編譯
3. 或執行 `vcvarsall.bat x64` 設定環境

---

### 5. CMake not found (Windows)

**錯誤訊息:**
```
error: failed to run custom build command for `aws-lc-sys`
CMake Error: CMake was unable to find a build program
```

**解決方案:**
```powershell
# 安裝 CMake
winget install Kitware.CMake

# 重新開啟終端機
# 確認 CMake 可用
cmake --version
```

---

### 6. NASM not found (Windows)

**錯誤訊息:**
```
NASM (Netwide Assembler) not found
```

**解決方案:**
```powershell
# 安裝 NASM
winget install NASM.NASM

# 將 NASM 加入 PATH (如果安裝時沒有自動加入)
# 預設路徑: C:\Program Files\NASM

# 重新開啟終端機
nasm -v
```

---

### 7. 編譯時記憶體不足

**錯誤訊息:**
```
error: could not compile `xxx` (lib)
Caused by: process didn't exit successfully
```

**解決方案:**
```bash
# 限制並行編譯數量
export CARGO_BUILD_JOBS=2

# 或使用 cargo 參數
cargo build --release -j 2
```

---

## 執行時問題

### 8. Address already in use

**錯誤訊息:**
```
error binding to address: Address already in use
```

**原因:** 端口已被其他程式佔用。

**解決方案:**

**Linux/macOS:**
```bash
# 找出佔用端口的程式
lsof -i :3001
# 或
netstat -tulpn | grep 3001

# 結束該程式或更換端口
kill -9 <PID>
```

**Windows:**
```powershell
# 找出佔用端口的程式
netstat -ano | findstr :3001

# 結束該程式
taskkill /PID <PID> /F
```

---

### 9. Permission denied

**錯誤訊息:**
```
Permission denied (os error 13)
```

**解決方案:**

**Linux/macOS:**
```bash
# 檢查檔案權限
ls -la /opt/vibe-kanban/

# 修正權限
sudo chown -R $USER:$USER /opt/vibe-kanban/
chmod 755 /opt/vibe-kanban/server
```

**Windows:**
以管理員身份執行 PowerShell，或檢查檔案的安全性設定。

---

### 10. 找不到設定檔

**錯誤訊息:**
```
Error loading .env file
```

**解決方案:**
確保在正確的工作目錄執行程式，或使用絕對路徑設定環境變數。

```bash
# 方法 1: 切換到正確目錄
cd /opt/vibe-kanban
./server

# 方法 2: 使用環境變數
DATABASE_URL=sqlite:/opt/vibe-kanban/data/vibe-kanban.db ./server
```

---

## 網路與連線問題

### 11. CORS 錯誤

**錯誤訊息 (瀏覽器 Console):**
```
Access to fetch has been blocked by CORS policy
```

**解決方案:**
設定正確的 `VK_ALLOWED_ORIGINS` 環境變數：

```bash
# 開發環境
VK_ALLOWED_ORIGINS=http://localhost:3000

# 生產環境
VK_ALLOWED_ORIGINS=https://your-domain.com

# 多個來源
VK_ALLOWED_ORIGINS=http://localhost:3000,https://app.example.com
```

---

### 12. 無法連線到後端

**症狀:** 前端載入但 API 請求失敗。

**檢查清單:**
1. 確認後端服務正在執行
2. 檢查端口是否正確
3. 檢查防火牆設定

```bash
# 測試 API 是否可達
curl http://localhost:3001/api/health
```

**Windows 防火牆:**
```powershell
# 允許端口
New-NetFirewallRule -DisplayName "Vibe Kanban" -Direction Inbound -Port 3001 -Protocol TCP -Action Allow
```

---

## 資料庫問題

### 13. 資料庫鎖定

**錯誤訊息:**
```
database is locked
```

**原因:** 另一個程序正在存取資料庫。

**解決方案:**
1. 確保只有一個伺服器實例在執行
2. 檢查是否有其他程式 (如 DB 瀏覽器) 開啟了資料庫
3. 重啟伺服器

---

### 14. 資料庫損毀

**錯誤訊息:**
```
database disk image is malformed
```

**解決方案:**
```bash
# 嘗試修復
sqlite3 /path/to/vibe-kanban.db "PRAGMA integrity_check;"

# 如果無法修復，從備份還原
# 或刪除資料庫重新開始 (會遺失資料)
rm /path/to/vibe-kanban.db
```

---

## 平台特定問題

### 15. npm scripts 在 Windows CMD 失敗

**症狀:** npm scripts 中的 Unix 語法無法執行。

**解決方案:**
使用 Git Bash 或 PowerShell 而非 CMD：

```bash
# 在 Git Bash 中執行
cd frontend
pnpm run build
```

---

### 16. 路徑過長 (Windows)

**錯誤訊息:**
```
The specified path, file name, or both are too long
```

**解決方案:**
1. 使用較短的專案路徑 (如 `C:\vk\`)
2. 啟用 Windows 長路徑支援：

```powershell
# 以管理員身份執行
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
  -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

---

### 17. macOS 安全性警告

**症狀:** 執行編譯的程式時出現「無法驗證開發者」警告。

**解決方案:**
```bash
# 移除隔離屬性
xattr -d com.apple.quarantine /path/to/server

# 或在系統偏好設定 > 安全性與隱私 中允許執行
```

---

## 取得更多幫助

如果上述解決方案無法解決您的問題：

1. **檢查日誌**: 使用 `RUST_LOG=debug` 取得更詳細的日誌
2. **搜尋 Issues**: 在 GitHub Issues 中搜尋類似問題
3. **提交 Issue**: 附上完整的錯誤訊息、環境資訊和重現步驟

---

*文件版本: 1.0*
*最後更新: 2026-01-28*
