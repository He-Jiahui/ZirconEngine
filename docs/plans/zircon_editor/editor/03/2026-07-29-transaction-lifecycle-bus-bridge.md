Plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
Milestone: M4
Status: superseded_current_source
Files: ["docs/plans/zircon_editor/editor/03/2026-07-29-transaction-lifecycle-bus-bridge.md", "zircon_editor/src/core/context/builder.rs", "zircon_editor/src/core/editing/engine/events.rs", "zircon_editor/src/core/editing/engine/mod.rs", "zircon_editor/src/core/editing/engine/transaction.rs", "zircon_editor/src/core/editor_message/message/delivery.rs", "zircon_editor/src/core/editor_message/message/transaction.rs", "zircon_editor/src/tests/editing/transaction_engine/events.rs"]

# Editor03 M4 事务生命周期 canonical bus bridge

## 范围与状态

本切片把 transaction engine 的五态观测从私有、无界的 `Vec<TransactionEvent>` 硬切到 Editor02 的
canonical bounded bus。当前状态为 `validation_pending`：源码和 scoped static gate 已完成，尚未获得
current-source managed Cargo、独立复审或 milestone commit，不能标记 fixed 或 M4 complete。

本记录只覆盖生命周期 bridge 的精确 8 文件 manifest。Editor02 的 lossless fanout admission 仍由其 own
failure record 验收；journal 合同未实现已独立登记为
[transaction journal contract unimplemented](failure-2026-07-29-transaction-journal-contract-unimplemented.md)，
不以本切片的 event delivery 伪装关闭 M4.2。

## 已实现合同

- `EditorTransactionEngine` 仅依赖 injectable `TransactionEventSink`；engine state 不再保存 event queue，
 也不再发布 `drain_events()` API。
- transaction start, commit, cancel, undo, and redo 均在 state transition/recovery 完成且 engine mutex
 释放后发送事件。sink delivery 的 backpressure/rejection 会被可观测记录，不能重建 engine-private backlog。
- `EditorContextBuilder` 是唯一 concrete bridge owner：它把五态 `TransactionEvent` 映射为 typed
  `TransactionMessage::{Started, Committed, Canceled, Undone, Redone}` 并发布到 `TOPIC_TRANSACTION`。
- `TransactionMessage` 的 lifecycle variants 包含 `transaction`、`history`、`label` 与 `timestamp_frame`；
  delivery byte accounting 覆盖所有五态。
- 事务事件测试改为 inject recording sink，覆盖 start/commit/undo/redo/cancel sequence；builder test 覆盖
  full typed mapping 和 canonical bus backpressure result。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-29 15:03 +08:00 | `实现完成-静态门通过-行为门待办` | 删除 `EngineState.events` 与 `drain_events()`，建立 `TransactionEventSink`、builder-to-bus concrete bridge、五态 typed `TransactionMessage` mapping 与 sink/builder sequence tests。 | `rustfmt --check` 13 个受影响 Rust 文件通过；`git diff --check` scoped paths 无错误；全仓 Rust `drain_events()` 和 `events: Vec<TransactionEvent>` 为零命中。 |
| 2026-07-29 15:03 +08:00 | `journal debt recorded` | 完成 M4.1 journal implementation audit，并确认 `EditCommand::serialize_journal` 的唯一实现仍为默认 `None`。 | [open failure](failure-2026-07-29-transaction-journal-contract-unimplemented.md)；不得把该状态升级为 M4.2 journal roundtrip accepted。 |
| 2026-07-29 15:37 +08:00 | `superseded_current_source` | 第一次 immutable M4 manifest 在 builder panic-free topic constructor 修正前绑定，故不得拿旧指纹启动或接收 Cargo 验证。 | successor `editor03-transaction-lifecycle-bridge-r2-20260729` 将以同一业务路径和独立 r2 子记录重新冻结当前源码；本记录仅保留审计历史。 |

## 后续验收

1. 基于本 exact8 manifest 建立 immutable current-source managed validation，运行 transaction event sequence、
   builder canonical topic/backpressure 与 `zircon_editor` 相关行为门。
2. 独立复审检查 engine 不重新持有 event retention，builder 映射完整且 backpressure 语义不静默丢失。
3. event bridge 行为门通过后，仅可回传 lifecycle bridge；M4 的 journal debt 保持 open，直至其 typed
   envelope 和 roundtrip evidence 独立完成。
