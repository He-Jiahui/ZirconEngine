Plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
Milestone: M4
Status: validation_external_failure
Files: ["docs/plans/zircon_editor/editor/03/2026-07-29-transaction-lifecycle-bus-bridge-r2.md", "zircon_editor/src/core/context/builder.rs", "zircon_editor/src/core/editing/engine/events.rs", "zircon_editor/src/core/editing/engine/mod.rs", "zircon_editor/src/core/editing/engine/transaction.rs", "zircon_editor/src/core/editor_message/message/delivery.rs", "zircon_editor/src/core/editor_message/message/transaction.rs", "zircon_editor/src/tests/editing/transaction_engine/events.rs"]

# Editor03 M4 transaction lifecycle canonical bus bridge r2

## Scope Delivered

This is the current-source successor to
[r1](2026-07-29-transaction-lifecycle-bus-bridge.md). It retains the exact lifecycle bridge
business scope and freezes the source after the builder's panic-free built-in transaction topic
construction was completed.

- `EditorTransactionEngine` observes transactions only through `TransactionEventSink`; engine state
  contains no event queue and exposes no `drain_events()` compatibility API.
- The concrete `EditorContextBuilder` bridge maps Started, Committed, Canceled, UndoApplied, and
  RedoApplied to typed `TransactionMessage` payloads on the canonical `TOPIC_TRANSACTION` bus.
- All lifecycle dispatch occurs after transaction state/context restoration and the engine mutex is
  released. Backpressure and rejection are observable, without reconstructing private retention.
- The built-in transaction topic now uses a crate-private typed constructor, so builder construction
  does not depend on a production `expect` path.

Editor02's lossless multi-subscriber admission remains owned by its own failure repair. The M4
journal contract remains open in
[transaction journal contract unimplemented](failure-2026-07-29-transaction-journal-contract-unimplemented.md);
this record does not claim journal roundtrip completion.

## Fresh Testing Evidence

- `rustfmt --check` passed for the lifecycle bridge, Editor02 bus/inbox integration, and affected
  transaction test files.
- Scoped `git diff --check` passed without whitespace errors.
- Static hard-cut scan found no `drain_events()`, no `events: Vec<TransactionEvent>`, and no
  builder `expect` for the built-in transaction topic.
- The r2 managed validation created immutable copy `1e0676bf09ed45f09a8968ea21b64d6f` at
  `2026-07-29 15:42:05 +08:00`. It remained `running` during materialization and was marked
  `removed` at `15:46:10 +08:00`, without an input manifest hash, typed materialization failure,
  Cargo reservation, Cargo job, run output, or test result. This is not an Editor03 source failure
  and cannot be used as behavior evidence.
- The terminal-observability defect is already owned by Coordinator01's open
  [validation-copy Cargo materialization nonterminal failure](../../../zircon_tooling/session_coordinator/01/failure-2026-07-27-validation-copy-cargo-materialization-nonterminal.md).
  Do not create a duplicate handoff, run unmanaged Cargo, or reuse this removed copy. A new
  current-source successor is required only after the Coordinator01 failure returns fixed.

## Review

Independent review is pending current-source managed validation. The review must verify the
five-state mapping, lock-free sink dispatch, lossless backpressure result, no engine-private
retention, and the absence of production panic-based topic construction.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-29 15:39 +08:00 | `current-source validation pending` | 创建 immutable-manifest successor，覆盖 lifecycle bridge exact8；r1 在 builder topic constructor 修正前的 manifest 已明确 superseded。 | r2 session `editor03-transaction-lifecycle-bridge-r2-20260729` 已领取 8/8 路径；Cargo、独立 review 与 milestone commit 尚未执行。 |
| 2026-07-29 15:46 +08:00 | `validation_external_failure` | r2 validation action 成功创建 immutable copy，但 copy 未 materialize 为可运行的 Cargo 输入便被受管移除。 | copy `1e0676bf09ed45f09a8968ea21b64d6f`：`running -> removed`，无 input manifest hash、typed terminal、reservation、Cargo job 或测试输出；归入 Coordinator01 既有 nonterminal-copy failure，不宣称行为验收。 |
