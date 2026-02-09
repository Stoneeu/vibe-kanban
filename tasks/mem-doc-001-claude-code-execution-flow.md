# DOC-001 Claude Code 執行流程 - 任務知識記憶檔案

## 研究背景
- **目標**：研究 Vibe Kanban 專案中 Claude Code 執行流程
- **重點**：了解 task 執行 Claude Code 時的運作方式，特別是 loop 循環機制

## 核心程式碼位置
- **Claude Executor**: `crates/executors/src/executors/claude.rs`
- **Protocol 處理**: `crates/executors/src/executors/claude/protocol.rs`
- **Client 處理**: `crates/executors/src/executors/claude/client.rs`
- **任務執行流程**: `crates/actions/coding_agent_initial.rs`, `crates/actions/coding_agent_follow_up.rs`
- **Container 服務**: `crates/services/src/services/container.rs`, `crates/local-deployment/src/container.rs`

## 關鍵發現摘要

### 1. Claude Code 基礎命令
- 預設: `npx -y @anthropic-ai/claude-code@2.1.7`
- Router 模式: `npx -y @musistudio/claude-code-router@1.0.66 code`

### 2. ClaudeCode Struct 核心欄位
- `append_prompt`: 附加提示內容
- `claude_code_router`: 啟用 router 模式
- `plan`: 計畫模式
- `approvals`: 啟用審批服務
- `model`: 可選的模型覆寫
- `dangerously_skip_permissions`: 跳過權限檢查
- `cmd`: 命令覆寫設定

### 3. Loop 循環機制確認
**是的，系統確實有 loop 機制**，透過以下方式實現：
1. **Protocol read_loop()**: 持續讀取 Claude Code 輸出直到收到 Result 或 EOF
2. **ExecutorAction Chain**: 每個 executor action 包含 next_action 指標
3. **try_start_next_action()**: 遞迴呼叫 start_execution()
4. **Task Finalization**: 只有當 next_action 為 None 且 should_finalize() 為 true 時才結束

---

## A 章節完成知識 (2026-01-16)

### 文件架構建立
- 文件輸出目錄: `docs/claude_code/`
- 已建立 `00-overview.md` - 執行流程總覽

### 三層 Loop 機制詳解
1. **Protocol Read Loop** (第一層)
   - 位置: `protocol.rs:46-101`
   - 使用 `tokio::select!` 同時監聽 stdout 和中斷訊號
   - 持續解析 JSON 訊息直到收到 `Result` 或 EOF

2. **ExecutorAction Chain** (第二層)
   - 位置: `container.rs:1165-1198`
   - 每個 ExecutorAction 包含可選的 `next_action` 指標
   - `try_start_next_action()` 遞迴呼叫 `start_execution()`

3. **Exit Monitor Loop** (第三層)
   - 位置: `local-deployment/container.rs:344-563`
   - `spawn_os_exit_watcher()` 每 250ms 輪詢程序狀態
   - 程序結束後觸發 commit、next_action 或 finalization

### 訊息類型 (CLIMessage)
- `ControlRequest`: 工具使用權限請求
- `ControlResponse`: 權限回應
- `Result`: 最終執行結果
- 其他: 日誌訊息

---

## B 章節完成知識 (2026-01-16)

### ClaudeCode Struct 完整欄位
- `append_prompt`: AppendPrompt - 附加提示內容
- `claude_code_router`: Option<bool> - Router 模式
- `plan`: Option<bool> - 計畫模式
- `approvals`: Option<bool> - 審批服務
- `model`: Option<String> - 模型覆寫
- `dangerously_skip_permissions`: Option<bool> - 跳過權限
- `disable_api_key`: Option<bool> - 停用 API Key
- `cmd`: CmdOverrides - 命令覆寫
- `approvals_service`: Option<Arc<dyn ExecutorApprovalService>> - 審批服務實例

### 命令建構流程
1. 選擇基礎命令 (標準或 Router)
2. 添加 `-p` 權限標誌
3. 條件添加 permission-prompt-tool 和 permission-mode
4. 條件添加 dangerously-skip-permissions
5. 條件添加 model 覆寫
6. 添加必要輸出格式參數
7. 應用命令覆寫 (CmdOverrides)

### 程序生成關鍵配置
- `kill_on_drop(true)`: 自動終止
- `group_spawn()`: 程序組管理
- `Stdio::piped()`: stdin/stdout/stderr 管道化
- `interrupt channel`: oneshot 中斷通道

### StandardCodingAgentExecutor Trait 方法
- `spawn()`: 初始任務
- `spawn_follow_up()`: 後續任務 (使用 --fork-session --resume)
- `normalize_logs()`: 日誌正規化
- `use_approvals()`: 注入審批服務

---

## C 章節完成知識 (2026-01-16)

### ProtocolPeer 雙向通訊
- **結構**: `Arc<Mutex<ChildStdin>>` 包裝
- **spawn()**: 建立協議通訊，啟動 read_loop 任務
- **initialize()**: 發送 Initialize 控制請求
- **set_permission_mode()**: 設定權限模式
- **send_user_message()**: 發送使用者訊息

### CLIMessage 訊息類型
```rust
pub enum CLIMessage {
    ControlRequest { request_id, request },  // 工具權限請求
    ControlResponse { ... },                  // 權限回應
    Result(serde_json::Value),               // 最終結果 (觸發 Loop 退出)
}
```

### ControlRequestType 請求類型
- `CanUseTool`: 工具使用權限請求
- `HookCallback`: Hook 回調請求

### SDKControlRequestType 發送類型
- `Initialize`: 初始化
- `SetPermissionMode`: 設定權限模式
- `Interrupt`: 中斷執行

### Loop 終止條件 (read_loop)
| 條件 | 說明 |
|------|------|
| `Ok(0)` | EOF - 程序結束 |
| `CLIMessage::Result` | 收到結果訊息 |
| `Err(e)` | 讀取錯誤 |

### should_finalize() 判斷規則
1. DevServer 永不結束
2. 平行模式 SetupScript 不結束
3. Failed 或 Killed 狀態總是結束
4. 沒有 next_action 時結束

### 三層 Loop 整合關係
```
Protocol Read Loop (內層)
    ↓ EOF/Result
Exit Monitor (中層)
    ↓ try_start_next_action
ExecutorAction Chain (外層)
    ↓ 遞迴 start_execution
```

---

## D 章節完成知識 (2026-01-16)

### ExecutorApprovalService Trait
- 抽象審批後端接口
- 方法: `request_tool_approval(tool_name, tool_input, tool_call_id)`
- 返回: `ApprovalStatus` (Approved/Denied/TimedOut/Pending)

### ClaudeAgentClient 審批流程
1. 檢查是否 auto_approve 模式
2. 有 tool_use_id 時呼叫 handle_approval()
3. ExitPlanMode 特殊處理：設定 BypassPermissions 模式
4. Hook Callback 使用 "ask" 決定轉發給 can_use_tool

### ExecutorApprovalBridge
- 位置: `services/approvals/executor_approvals.rs`
- 連接 Executor 層和 Service 層
- 整合 Approvals、DB、NotificationService

### Executable Trait
```rust
trait Executable {
    async fn spawn(current_dir, approvals, env) -> SpawnedChild;
}
```

### CodingAgentInitialRequest vs CodingAgentFollowUpRequest
| 特性 | Initial | Follow-up |
|------|---------|-----------|
| Session | 新建 | 恢復 (--fork-session --resume) |
| 欄位 | prompt, executor_profile_id | + session_id |

### ExecutorAction 鏈式結構
```rust
struct ExecutorAction {
    action_type: ExecutorActionType,
    next_action: Option<Box<ExecutorAction>>,
}
```

### 動作轉換規則
- Script → Script = SetupScript
- CodingAgent → Script = CleanupScript
- Any → CodingAgent = CodingAgent

### should_finalize() 規則
1. DevServer 永不結束
2. 平行 SetupScript 不結束
3. Failed/Killed 總是結束
4. 無 next_action 則結束

---

## E 章節完成知識 (2026-01-16)

### ClaudeCode 輸入參數
| 參數 | 類型 | 用途 |
|------|------|------|
| append_prompt | AppendPrompt | 前後附加提示 |
| claude_code_router | Option<bool> | Router 模式 |
| plan | Option<bool> | 計畫模式 |
| approvals | Option<bool> | 審批服務 |
| model | Option<String> | 模型覆寫 |
| dangerously_skip_permissions | Option<bool> | 跳過權限 |
| disable_api_key | Option<bool> | 停用 API Key |
| cmd | CmdOverrides | 命令覆寫 |

### CmdOverrides 結構
- base_command_override: 覆寫基礎命令
- additional_params: 額外命令參數
- env_vars: 額外環境變數

### Session 管理
- Initial: `spawn()` 建立新 Session
- Follow-up: `spawn_follow_up()` 使用 `--fork-session --resume <session_id>`
- Session ID 由 Claude Code 產生並透過 JSON 回傳

### StandardCodingAgentExecutor Trait
```rust
trait StandardCodingAgentExecutor {
    fn spawn(...) -> SpawnedChild;           // 新 Session
    fn spawn_follow_up(session_id) -> SpawnedChild;  // 恢復 Session
    fn normalize_logs(...);
    fn use_approvals(...);
}
```

---

## F 章節完成知識 (2026-01-16)

### 文件集完成清單

| 序號 | 文件名 | 主題 | 狀態 |
|------|--------|------|------|
| 00 | 00-overview.md | 執行流程總覽 | ✅ |
| 01 | 01-executor-architecture.md | Executor 架構與核心結構 | ✅ |
| 02 | 02-command-building.md | 命令建構邏輯 | ✅ |
| 03 | 03-process-spawning.md | 程序生成機制 | ✅ |
| 04 | 04-protocol-handling.md | 協議處理與雙向通訊 | ✅ |
| 05 | 05-loop-mechanism.md | Loop 循環機制詳解 | ✅ |
| 06 | 06-approval-service.md | 權限審批服務 | ✅ |
| 07 | 07-task-execution-flow.md | 任務執行流程 | ✅ |
| 08 | 08-next-action-chain.md | NextAction 鏈式執行機制 | ✅ |
| 09 | 09-input-parameters.md | 輸入參數詳解 | ✅ |
| 10 | 10-session-management.md | Session 管理與 Follow-up | ✅ |
| README | README.md | 文件索引與導覽 | ✅ |

### 核心研究結論

**主要問題回答：系統是否有 Loop 循環機制？**

✅ **是的，系統確實有三層 Loop 機制確保任務持續執行直到完成：**

1. **Protocol Read Loop** (`protocol.rs:46-101`)
   - 持續讀取 Claude Code stdout
   - 使用 `tokio::select!` 監聽多事件
   - 遇到 Result/EOF 才跳出

2. **ExecutorAction Chain** (`container.rs:1165-1198`)
   - 每個動作包含 `next_action` 指標
   - 遞迴呼叫 `start_execution()`
   - 形成 Setup → CodingAgent → Cleanup 鏈

3. **Exit Monitor Loop** (`local-deployment/container.rs:344-563`)
   - 250ms 輪詢程序狀態
   - 觸發 commit、next_action 或 finalization
   - `should_finalize()` 判斷最終結束

### 技術棧

- **語言**: Rust
- **異步運行時**: Tokio
- **序列化**: Serde / JSON
- **程序管理**: command_group
- **圖表格式**: Mermaid

---
*DOC-001 文件任務完成於 2026-01-16*
