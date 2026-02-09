# Claude Code 執行流程文件

> **文件編號**: DOC-001
> **建立日期**: 2026-01-16
> **原始碼位置**: `crates/executors/src/executors/claude/`

## 概述

本文件集詳細描述了 Vibe Kanban 專案中 Claude Code 的執行流程，包括 Executor 架構、命令建構、程序生成、協議處理、Loop 循環機制、權限審批、任務執行、Session 管理等核心機制。

## 核心問題解答

### ❓ 系統是否有 Loop 循環機制?

**✅ 是的，系統確實有三層 Loop 機制確保任務持續執行直到完成：**

1. **Protocol Read Loop** - 協議層持續讀取 Claude Code 輸出
2. **ExecutorAction Chain** - 動作鏈式執行
3. **Exit Monitor Loop** - 程序退出監控

詳見: [05-loop-mechanism.md](./05-loop-mechanism.md)

## 文件索引

### 基礎架構 (A-B 章節)

| 序號 | 文件 | 主題 | 原始碼 |
|------|------|------|--------|
| 00 | [00-overview.md](./00-overview.md) | 執行流程總覽 | - |
| 01 | [01-executor-architecture.md](./01-executor-architecture.md) | Executor 架構與核心結構 | `claude.rs` |
| 02 | [02-command-building.md](./02-command-building.md) | 命令建構邏輯 | `claude.rs:78-119` |
| 03 | [03-process-spawning.md](./03-process-spawning.md) | 程序生成機制 | `claude.rs:233-316` |

### 通訊與循環 (C 章節)

| 序號 | 文件 | 主題 | 原始碼 |
|------|------|------|--------|
| 04 | [04-protocol-handling.md](./04-protocol-handling.md) | 協議處理與雙向通訊 | `protocol.rs` |
| 05 | [05-loop-mechanism.md](./05-loop-mechanism.md) | Loop 循環機制詳解 | 多個檔案 |

### 任務執行 (D 章節)

| 序號 | 文件 | 主題 | 原始碼 |
|------|------|------|--------|
| 06 | [06-approval-service.md](./06-approval-service.md) | 權限審批服務 | `client.rs`, `approvals.rs` |
| 07 | [07-task-execution-flow.md](./07-task-execution-flow.md) | 任務執行流程 | `coding_agent_*.rs` |
| 08 | [08-next-action-chain.md](./08-next-action-chain.md) | NextAction 鏈式執行機制 | `container.rs` |

### 進階主題 (E 章節)

| 序號 | 文件 | 主題 | 原始碼 |
|------|------|------|--------|
| 09 | [09-input-parameters.md](./09-input-parameters.md) | 輸入參數詳解 | `claude.rs:52-76` |
| 10 | [10-session-management.md](./10-session-management.md) | Session 管理與 Follow-up | `claude.rs:139-172` |

## 快速導覽流程圖

```mermaid
flowchart TB
    subgraph "使用者層"
        USER[使用者]
    end

    subgraph "服務層"
        CS[ContainerService]
        AS[ApprovalService]
    end

    subgraph "執行器層"
        EX[ClaudeCode Executor]
        PP[ProtocolPeer]
        CAC[ClaudeAgentClient]
    end

    subgraph "程序層"
        CC[Claude Code CLI]
    end

    USER -->|1. 建立任務| CS
    CS -->|2. 生成程序| EX
    EX -->|3. spawn()| CC
    CC <-->|4. JSON 通訊| PP
    PP <-->|5. 控制請求| CAC
    CAC <-->|6. 審批請求| AS
    AS <-->|7. 使用者審批| USER
```

## 關鍵程式碼位置

| 模組 | 路徑 | 說明 |
|------|------|------|
| Claude Executor | `crates/executors/src/executors/claude.rs` | 主執行器 |
| Protocol | `crates/executors/src/executors/claude/protocol.rs` | 協議處理 |
| Client | `crates/executors/src/executors/claude/client.rs` | 客戶端處理 |
| Approvals | `crates/executors/src/approvals.rs` | 審批 Trait |
| Container | `crates/services/src/services/container.rs` | 容器服務 |
| Actions | `crates/executors/src/actions/` | 動作定義 |

## 閱讀建議

### 快速了解

1. 先閱讀 [00-overview.md](./00-overview.md) 取得全貌
2. 查看 [05-loop-mechanism.md](./05-loop-mechanism.md) 了解核心循環機制

### 深入研究

按順序閱讀 01 到 10 的文件，每個文件都有完整的程式碼引用和流程圖。

### 問題排查

- 命令問題 → [02-command-building.md](./02-command-building.md)
- 程序啟動問題 → [03-process-spawning.md](./03-process-spawning.md)
- 權限問題 → [06-approval-service.md](./06-approval-service.md)
- Session 問題 → [10-session-management.md](./10-session-management.md)

## 技術棧

- **語言**: Rust
- **異步運行時**: Tokio
- **序列化**: Serde / JSON
- **程序管理**: command_group
- **圖表格式**: Mermaid

---

*此文件為 Vibe Kanban 專案的技術文件，使用繁體中文撰寫並搭配專業英文術語。*
