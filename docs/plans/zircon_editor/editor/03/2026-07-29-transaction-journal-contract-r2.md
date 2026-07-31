Plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
Milestone: M4
Status: validation_external_failure
Files: ["docs/plans/zircon_editor/editor/03/2026-07-29-transaction-journal-contract-r2.md", "zircon_editor/src/core/editing/selection.rs", "zircon_editor/src/core/editing/engine/command.rs", "zircon_editor/src/core/editing/engine/history.rs", "zircon_editor/src/core/editing/engine/journal.rs", "zircon_editor/src/core/editing/engine/mod.rs", "zircon_editor/src/core/editing/engine/transaction.rs", "zircon_editor/src/tests/editing/transaction_engine/mod.rs", "zircon_editor/src/tests/editing/transaction_engine/journal.rs"]

# Editor03 M4 typed transaction journal contract r2

## Current Source Scope

This is the immutable-manifest successor to
[r1](2026-07-29-transaction-journal-contract-r1.md). It keeps the same typed journal business
scope after adding `TransactionJournal::decode`, which makes JSON deserialization and envelope-schema
validation one reader boundary.

- `TransactionJournal` retains typed transaction/history identity, label, frame, participants,
  selection projections, significance, and typed command payloads.
- `EditCommand::journal_payload` returns a typed payload or a typed unsupported result; it no longer
  silently writes an empty successful journal record.
- History and engine journal queries expose completed transaction envelopes without adding a second
  history owner. Plan11 owns storage; Plan17 owns validated replay and recovery.
- Journal tests specify roundtrip, reader schema rejection, unsupported command context, selection
  projection preservation, and identity preservation over undo/redo.

## Validation State

- Exact Rust paths pass `rustfmt --check` and scoped `git diff --check`.
- r1 copy `3265199b4cd04fe8afae40dee3fb2970` failed externally in closure planning before an input
  manifest or Cargo work existed. It is audit history only and cannot validate this r2 source.
- r2 copy `d48460eee6014bd294d81c470679ccfd` repeated the coordinator external-source failure in
  `closure_planning`, before an input manifest or Cargo work existed. A later current-source retry
  is required after Coordinator01 returns its materialization failure fixed; this is not a source
  compile verdict.

## Remaining Debt

Concrete production commands still require their own versioned `journal_payload` migrations. The
unsupported result remains correct until each command contract exists; no empty-payload compatibility
path is permitted.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-29 16:31 +08:00 | `current-source validation pending` | 以 `TransactionJournal::decode` 收紧 journal reader：反序列化后必须在同一 API 内通过 schema validation；测试改为直接覆盖 reader 的 unknown-schema rejection。 | r2 session `editor03-transaction-journal-contract-r2-20260729` 领取 9/9 exact paths；将重新创建受管 immutable copy，未复用 r1 external-failure copy。 |
| 2026-07-29 16:34 +08:00 | `validation_external_failure` | r2 immutable copy `d48460eee6014bd294d81c470679ccfd` 被受管创建，但在 external-source closure planning 终止。 | `validation_copy_external_source_missing`；无 input manifest、Cargo reservation/job/run/test。关联 Coordinator01 既有 materialization failure，不能宣称 Editor03 行为验证。 |
