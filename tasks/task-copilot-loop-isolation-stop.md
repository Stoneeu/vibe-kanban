# Copilot Loop Session 隔離性驗證 + 停止機制支援

## 必要知識上下文
- 遵循 `docs/copilot/SPEC-copilot-loop.md` 規格文件
- 任務知識記憶檔案: `./tasks/mem-copilot-loop-isolation-stop.md`

## A. Session 隔離性驗證
- [x] [A.01] 分析 CopilotLoopTracker 資料結構的 Session 隔離設計
- [x] [A.02] 驗證 HashMap<Uuid, CopilotLoopState> 的 workspace_id 隔離機制
- [x] [A.03] 檢查是否有任何共用狀態可能造成多 session 干擾
- [x] [A.04] 撰寫多 session 隔離性的驗證報告

## B. 原生停止機制研究與支援
- [x] [B.01] 研究 Copilot CLI 原生停止機制的運作方式
- [x] [B.02] 分析 container.rs 中的程序生命週期管理
- [x] [B.03] 設計 Copilot Loop 的停止機制方案
- [x] [B.04] 實作停止機制支援
- [x] [B.05] 更新規格文件

## FINAL. 任務完成訊號
- [x] [FINAL.01] Output <promise>COMPLETE-HUNTER</promise> when all phases done.
