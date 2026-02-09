# Copilot 執行流程 - 知識記憶檔案

> 此檔案記錄 Copilot 執行流程研究過程中的重要背景知識

## 關鍵發現

### 1. Copilot 執行器結構 (copilot.rs)
- **版本**: `@github/copilot@0.0.375`
- **執行方式**: 透過 `npx` 執行
- **核心參數**:
  - `--no-color`: 停用顏色輸出
  - `--log-level debug`: 開啟 debug 日誌
  - `--log-dir <dir>`: 指定日誌目錄
  - `--model <model>`: 指定模型
  - `--allow-all-tools`: 允許所有工具
  - `--resume <session_id>`: 恢復之前的 session

### 2. Copilot 與 Claude Code 的關鍵差異

| 特性 | Claude Code | Copilot |
|------|-------------|---------|
| ProtocolPeer | ✅ 有 | ❌ 無 |
| 雙向通訊 | ✅ read_loop | ❌ stdin 寫入後關閉 |
| exit_signal | ✅ 有 | ❌ 無 |
| interrupt_sender | ✅ 有 | ❌ 無 |
| SessionFork | ✅ 有 | ❌ 無 |
| capabilities | SessionFork | vec![] (空) |

### 3. 三層 Loop 機制 (Claude Code)
1. **Protocol Read Loop** (`protocol.rs:46-101`)
   - 持續讀取 stdout
   - 處理 ControlRequest
   - 等待 Result 或 EOF

2. **ExecutorAction Chain** (`container.rs:1165-1198`)
   - `next_action` 鏈式結構
   - `try_start_next_action()` 遞迴執行

3. **Exit Monitor Loop** (`local-deployment/container.rs`)
   - `spawn_exit_monitor()` 監控程序退出
   - `spawn_os_exit_watcher()` 每 250ms 輪詢

### 4. Copilot 問題根因
- Copilot 缺少 ProtocolPeer 層
- stdin 寫入後立即關閉，無法持續通訊
- 沒有機制讓 Copilot 回報任務完成狀態
- ~15 分鐘後停止是因為 Copilot CLI 內部行為

### 5. SpawnedChild 結構
```rust
pub struct SpawnedChild {
    pub child: AsyncGroupChild,
    pub exit_signal: Option<ExecutorExitSignal>,      // Copilot: None
    pub interrupt_sender: Option<InterruptSender>,     // Copilot: None
}
```

## 解決方案

### 推薦方案: ExecutorAction Chain (方案 A)

利用現有的 `ExecutorAction.next_action` 機制，在任務建立時預先串接多個 follow-up 動作。

**優點**:
- 最小修改，利用現有機制
- 無需修改 Copilot Executor
- 可配置，向後相容

**實作位置**: `crates/services/src/services/workspace.rs` 或 `crates/services/src/actions/mod.rs`

### 其他方案

| 方案 | 複雜度 | 推薦度 |
|------|--------|--------|
| A. ExecutorAction Chain | 低 | ⭐⭐⭐⭐⭐ |
| B. Exit + Auto Follow-up | 中 | ⭐⭐⭐⭐ |
| C. ProtocolPeer 實作 | 高 | ⭐⭐ |

## 重要程式碼位置
- `crates/executors/src/executors/copilot.rs` - Copilot 執行器
- `crates/executors/src/executors/claude.rs` - Claude Code 執行器 (參考)
- `crates/executors/src/executors/mod.rs` - Executor trait 定義
- `crates/local-deployment/src/container.rs` - spawn_exit_monitor
- `crates/services/src/services/container.rs` - try_start_next_action

## 產出文件
- `docs/copilot/01-overview.md` - 執行流程概述
- `docs/copilot/02-spawn-mechanism.md` - Spawn 機制
- `docs/copilot/03-exit-monitoring.md` - 退出監控機制
- `docs/copilot/04-comparison-with-claude.md` - 與 Claude Code 差異比較
- `docs/copilot/05-loop-solution-proposal.md` - Loop 循環解決方案
- `docs/copilot/06-copilot-loop-implementation-plan.md` - Loop 功能實作計劃

## Copilot Loop 功能實作計劃 (方案 B)

### 參考來源
- Ralph-Wiggum Plugin: https://github.com/anthropics/claude-code/blob/main/plugins/ralph-wiggum/README.md

### 新增參數
| 參數 | 類型 | 預設值 | 說明 |
|------|------|--------|------|
| `loop_enabled` | `boolean` | `false` | 啟用 Loop 功能 |
| `max_iterations` | `u32` | `5` | 最大循環次數 (安全限制) |
| `completion_promise` | `String` | `<promise>COMPLETE</promise>` | 完成偵測字串 |

### 核心元件
1. **CopilotLoopTracker**: 追蹤每個 workspace 的 Loop 狀態
2. **check_completion_promise()**: 掃描 MsgStore 檢查完成字串
3. **handle_copilot_loop()**: 主要 Loop 邏輯，決定是否啟動 Follow-up

### 實作位置
- Backend: `crates/local-deployment/src/container.rs` (spawn_exit_monitor 修改)
- Frontend: `frontend/src/components/settings/CopilotLoopSettings.tsx` (新增)
- Schema: `shared/schemas/copilot.json` (新增屬性)
