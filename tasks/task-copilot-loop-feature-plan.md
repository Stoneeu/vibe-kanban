# Copilot Loop 功能執行計劃 - 任務列表

> **建立日期**: 2026-01-16
> **狀態**: 規劃完成
> **文件位置**: `docs/copilot/06-copilot-loop-implementation-plan.md`

## 必要知識上下文

- 參考 ralph-wiggum 的 `--max-iterations` 和 `--completion-promise` 參數控制方法
- 現有架構: `CopilotLoopTracker` 追蹤狀態，`spawn_exit_monitor` 處理退出
- 使用 `MsgStore.get_history()` 檢查輸出內容
- 任務知識記憶檔案: `./tasks/mem-copilot-loop-feature-plan.md`

## A. Backend 核心實作

- [ ] [A.01] 新增 Copilot struct 欄位: `loop_enabled`, `max_iterations`, `completion_promise`
  - 檔案: `crates/executors/src/executors/copilot.rs:38-55`
- [ ] [A.02] 更新 Copilot JSON Schema
  - 檔案: `shared/schemas/copilot.json`
- [ ] [A.03] 執行 `pnpm run generate-types` 生成 TypeScript types
- [ ] [A.04] 建立 `CopilotLoopTracker` 模組追蹤 Loop 狀態
  - 檔案: `crates/local-deployment/src/loop_tracker.rs` (新建)
- [ ] [A.05] 實作 `check_completion_promise()` 完成字串偵測
  - 檔案: `crates/local-deployment/src/container.rs`
- [ ] [A.06] 實作 `handle_copilot_loop()` 主要 Loop 邏輯
  - 檔案: `crates/local-deployment/src/container.rs`
- [ ] [A.07] 修改 `spawn_exit_monitor` 整合 Loop 處理
  - 檔案: `crates/local-deployment/src/container.rs:344-563`
- [ ] [A.08] 將重要上下文知識寫入記憶檔案

## B. Frontend UI 實作

- [ ] [B.01] 建立 `CopilotLoopSettings` 元件
  - 檔案: `frontend/src/components/settings/CopilotLoopSettings.tsx` (新建)
- [ ] [B.02] 整合到 Agent Settings 頁面
  - 檔案: `frontend/src/pages/settings/AgentSettings.tsx`
- [ ] [B.03] 更新 i18n 翻譯檔案
  - 檔案: `frontend/src/i18n/locales/*/settings.json`
- [ ] [B.04] 將重要上下文知識寫入記憶檔案

## C. 測試與驗證

- [ ] [C.01] 撰寫 Rust 單元測試 (completion detection, iteration limit)
- [ ] [C.02] 執行整合測試 (完整 Loop 流程)
- [ ] [C.03] 使用 `pnpm run dev:qa` 進行 QA 測試
- [ ] [C.04] 更新使用文件
- [ ] [C.05] 將重要上下文知識寫入記憶檔案

## D. 總結與經驗整合

- [ ] [D.01] 檢視 A、B 階段是否能整合批次處理
- [ ] [D.02] 撰寫實作經驗總結

## FINAL. 任務完成訊號

- [ ] [FINAL.01] Output <promise>COMPLETE-HUNTER</promise> when all phases done.
