# DEV-003 Debug Copilot Loop 執行流程

## 必要知識上下文
- 參考文件: `docs/copilot/` 目錄下的所有文件
- 任務知識記憶檔案: `./tasks/mem-dev-003-debug-copilot-loop.md`
- 使用 `pnpm run dev:qa` 進行 QA 測試

## A. 研究 Copilot Loop 核心架構
- [x] [A.01] 閱讀 `docs/copilot/06-copilot-loop-implementation-plan.md`
- [x] [A.02] 研究 `crates/local-deployment/src/loop_tracker.rs`
- [x] [A.03] 研究 `crates/local-deployment/src/container.rs` 的 `handle_copilot_loop` 方法
- [x] [A.04] 研究 `crates/executors/src/executors/copilot.rs` 的 loop 相關欄位

## B. 整理 Copilot Loop 關鍵 Log 觀察清單
- [x] [B.01] 整理 loop 開始、迭代、完成的所有 log 訊息
- [x] [B.02] 整理可在終端機觀察的 tracing log 格式

### 關鍵 Log 訊息清單
| Log 訊息 | 觸發時機 | 來源檔案 |
|---------|---------|---------|
| `Registered Copilot loop state for workspace {}: max_iterations={}` | Loop 註冊時 | loop_tracker.rs:85-89 |
| `Copilot loop iteration {} of {} for workspace {} (can_continue={})` | 每次迭代 | loop_tracker.rs:109-115 |
| `Completion promise '{}' detected in output` | 偵測到完成標記 | loop_tracker.rs:151-154 |
| `Copilot loop complete: completion promise detected for workspace {}` | Promise 觸發完成 | container.rs:961-964 |
| `Copilot loop complete: max iterations reached for workspace {}` | 達到最大迭代次數 | container.rs:972-975 |
| `Starting Copilot loop follow-up for workspace {} (iteration {})` | 開始下一次迭代 | container.rs:1019-1023 |
| `Removed Copilot loop state for workspace {}` | Loop 移除時 | loop_tracker.rs:125 |

## C. 實際驗證 Copilot Loop 運作
- [x] [C.01] 連接開發環境 (http://127.0.0.1:9998/)
- [x] [C.02] 透過 Web UI 觀察任務與建立嘗試對話框
- [x] [C.03] 確認 UI 設定: `loop_enabled=true`, `max_iterations=5`, `completion_promise=<promise>COMPLETE-HUNTER</promise>`
- [x] [C.04] 識別 backend log 觀察方式 (需執行 `RUST_LOG=debug cargo watch` 或查看終端輸出)

### UI 設定位置
Settings > Agents > COPILOT > DEFAULT Configuration:
- **Loop Enabled**: Enable automatic loop until task completion (checkbox)
- **Max Iterations**: Maximum number of loop iterations (default: 5, max: 100)
- **Completion Promise**: Exact string that signals task completion (e.g., `<promise>COMPLETE</promise>`)

## D. 總結與文件
- [x] [D.01] 將重要知識寫入 `mem-dev-003-debug-copilot-loop.md`

## FINAL. 任務完成訊號
- [x] [FINAL.01] Output <promise>COMPLETE-HUNTER</promise> when all phases done.

