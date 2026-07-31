Plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
Milestone: M4
Status: superseded_current_source
Files: ["docs/plans/zircon_editor/editor/03/2026-07-29-transaction-journal-contract-r1.md", "docs/plans/zircon_editor/editor/03/failure-2026-07-29-transaction-journal-contract-unimplemented.md", "zircon_editor/src/core/editing/selection.rs", "zircon_editor/src/core/editing/engine/command.rs", "zircon_editor/src/core/editing/engine/history.rs", "zircon_editor/src/core/editing/engine/journal.rs", "zircon_editor/src/core/editing/engine/mod.rs", "zircon_editor/src/core/editing/engine/transaction.rs", "zircon_editor/src/tests/editing/transaction_engine/mod.rs", "zircon_editor/src/tests/editing/transaction_engine/journal.rs"]

# Editor03 M4 typed transaction journal contract r1

## Scope Delivered

This slice replaces the engine's silent `Option<serde_json::Value>` journal hook with a typed,
versioned transaction envelope. `serde_json::Value` remains the command-defined wire payload inside
`CommandJournalPayload`; it is not transaction-engine state or a replacement for typed history,
selection, identity, or routing state.

- `TransactionJournal` carries schema version, transaction and history identity, label, frame,
  participants, typed selection projections, significance, and typed command payloads.
- `EditCommand::journal_payload` succeeds with a `CommandJournalPayload` or returns
  `CommandJournalUnavailable`; unsupported commands cannot become empty successful journal rows.
- `HistoryStore` and `EditorTransactionEngine` expose an immutable query for a completed
  transaction. `TransactionJournal::decode` couples deserialization to schema validation before
  Plan11 persistence or Plan17 recovery consumes it.
- The engine deliberately does not deserialize or replay journal payloads. Storage belongs to
  Plan11 and recovery ownership belongs to Plan17.

## Validation State

- TDD tests cover typed metadata/payload JSON roundtrip, unknown schema rejection, unsupported
  command context, selection projection preservation, and journal identity stability across
  undo/redo.
- `rustfmt --check` passes for the exact edited Rust files.
- Scoped `git diff --check` passed. The first managed validation copy failed during external-source
  closure planning before it produced a source manifest or Cargo job. A subsequent reader-boundary
  correction changed the exact source again, so this r1 record is superseded rather than reused.

## Remaining Debt

- Existing production `EditCommand` implementations still need their concrete, versioned payload
  migrations. Until each implements `journal_payload`, the typed unsupported result is intentional
  and the failure handoff remains open.
- Plan11 must define persistence around this stable envelope; Plan17 must own validated replay and
  crash recovery. Neither may recreate a private history stack or use an old journal compatibility
  path.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-29 16:10 +08:00 | `实现完成-静态格式门通过-受管行为门待办` | 实现版本化 `TransactionJournal`、typed command payload/unsupported result、typed selection projection、history/engine journal query 和 journal 专属测试。 | `rustfmt --check` 覆盖 exact Rust scope；测试定义覆盖 roundtrip、schema rejection、unsupported command 与 undo/redo identity。未运行非受管 Cargo，未关闭 failure。 |
| 2026-07-29 16:27 +08:00 | `superseded_current_source` | 首次 managed copy `3265199b4cd04fe8afae40dee3fb2970` 在 `closure_planning` 外部源物化失败；随后补齐 `TransactionJournal::decode`，使 r1 immutable source 不再当前。 | copy 终态 `validation_copy_external_source_missing`，没有 input manifest、Cargo reservation/job/run/test；r2 将重新绑定当前源码，不能复用 r1 copy。 |
