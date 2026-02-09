# CI/CD 編譯與手動部署任務列表

## 必要知識上下文
- 專案為 Rust (後端) + React/TypeScript (前端) 的全端應用
- 後端使用 Cargo workspace，包含多個 crates
- 前端使用 Vite + React + TypeScript
- 使用 pnpm 作為套件管理器
- 任務知識記憶檔案: 使用 `./tasks/mem-cicd-build-deploy.md` 記憶體文件作為背景知識

## A. 環境檢查與依賴安裝
- [x] [A.01] 檢查 Node.js、pnpm、Rust 等基礎環境版本
- [x] [A.02] 執行 `pnpm install` 安裝前端依賴
- [x] [A.03] 檢查 Cargo 依賴狀態
- [x] [A.04] 將此章節重要上下文, 需要記住的知識寫入'任務知識記憶檔案'中

## B. 前端編譯測試
- [x] [B.01] 執行前端類型檢查 `pnpm run frontend:check`
- [x] [B.02] 執行前端 lint 檢查 `pnpm run frontend:lint`
- [x] [B.03] 嘗試前端生產環境編譯
- [x] [B.04] 記錄前端編譯結果與問題
- [x] [B.05] 將此章節重要上下文, 需要記住的知識寫入'任務知識記憶檔案'中

## C. 後端編譯測試
- [x] [C.01] 執行後端類型檢查 `pnpm run backend:check`
- [x] [C.02] 執行後端 lint 檢查 `pnpm run backend:lint`
- [x] [C.03] 執行 Rust 測試 `cargo test --workspace`
- [x] [C.04] 執行後端 release 編譯 `cargo build --release`
- [x] [C.05] 記錄後端編譯結果與問題
- [x] [C.06] 將此章節重要上下文, 需要記住的知識寫入'任務知識記憶檔案'中

## D. 經驗整合與優化評估
- [x] [D.01] 總結 A、B、C 章節的編譯經驗，評估是否有可批次處理的任務

## E. Windows 環境相容性分析
- [x] [E.01] 分析 Rust 依賴在 Windows 的相容性 (特別是 git2, rustls 等)
- [x] [E.02] 檢查是否有平台特定的程式碼或路徑處理
- [x] [E.03] 分析前端在 Windows 的相容性
- [x] [E.04] 列出 Windows 部署需要的額外設定或注意事項
- [x] [E.05] 將此章節重要上下文, 需要記住的知識寫入'任務知識記憶檔案'中

## F. 撰寫手動部署文件
- [x] [F.01] 撰寫 Linux/macOS 手動部署步驟文件
- [x] [F.02] 撰寫 Windows 手動部署步驟文件
- [x] [F.03] 撰寫環境變數設定說明
- [x] [F.04] 撰寫常見問題與解決方案
- [x] [F.05] 將此章節重要上下文, 需要記住的知識寫入'任務知識記憶檔案'中

## G. 經驗整合與最終檢查
- [x] [G.01] 總結 E、F 章節經驗，整合部署文件
- [x] [G.02] 驗證部署文件完整性

## FINAL. 任務完成訊號
- [x] [FINAL.01] Output <promise>COMPLETE-HUNTER</promise> when all phases done.
