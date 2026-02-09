# Copilot 執行流程研究 任務列表

## 必要知識上下文(將需求指定重要遵守的規範...)
- 文件目錄: `docs/copilot/`
- 使用繁體中文配合專業英文名詞
- 參考 `docs/claude_code` 的機制
- 使用 mermaid 或 graphviz 圖表
- '任務知識記憶檔案': 使用 `./tasks/mem-doc-002-copilot-execution-flow.md` 記憶體文件作為背景知識

## A. 研究階段 (Research Phase)
- [x] [A.01] 閱讀 `crates/executors/src/executors/copilot.rs` 原始碼
- [x] [A.02] 閱讀 `crates/executors/src/executors/mod.rs` 了解 Executor trait
- [x] [A.03] 閱讀 `crates/local-deployment/src/container.rs` 了解 spawn_exit_monitor
- [x] [A.04] 閱讀 `crates/services/src/services/container.rs` 了解 try_start_next_action
- [x] [A.05] 閱讀 `docs/claude_code` 目錄下的 Loop 機制文件
- [x] [A.06] 比較 Copilot 與 Claude Code 的執行差異

## B. 文件撰寫階段 (Documentation Phase)
- [x] [B.01] 撰寫 `docs/copilot/01-overview.md` - Copilot 執行流程概述
- [x] [B.02] 撰寫 `docs/copilot/02-spawn-mechanism.md` - Copilot Spawn 機制
- [x] [B.03] 撰寫 `docs/copilot/03-exit-monitoring.md` - 退出監控機制
- [x] [B.04] 將重要上下文知識寫入記憶檔案

## C. 比較分析階段 (Comparison Phase)
- [x] [C.01] 撰寫 `docs/copilot/04-comparison-with-claude.md` - 與 Claude Code 差異比較
- [x] [C.02] 將比較分析重要知識寫入記憶檔案

## D. 解決方案階段 (Solution Phase)
- [x] [D.01] 撰寫 `docs/copilot/05-loop-solution-proposal.md` - Loop 循環解決方案
- [x] [D.02] 將解決方案重要知識寫入記憶檔案

## FINAL. 任務完成訊號
- [x] [FINAL.01] Output <promise>COMPLETE-HUNTER</promise> when all phases done.
