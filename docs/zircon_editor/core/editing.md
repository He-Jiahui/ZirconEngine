---
related_code:
  - zircon_editor/src/core/editing/engine/mod.rs
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/engine/routing.rs
  - zircon_editor/src/core/editing/engine/events.rs
  - zircon_editor/src/core/editing/context.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/editor_message/message/transaction.rs
implementation_files:
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editing/engine/mod.rs
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/engine/routing.rs
  - zircon_editor/src/core/editing/engine/events.rs
  - zircon_editor/src/core/editing/context.rs
  - zircon_editor/src/core/context/editor_context.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/core/editing/command.rs::tests::editor_commands_avoid_recollecting_batches_and_reapplying_unchanged_node_fields
  - zircon_editor/src/core/editing/engine/transaction.rs::tests::nested_cancel_does_not_remove_from_the_front_of_a_vec
  - zircon_editor/src/tests/editing/transaction_engine/history.rs
  - zircon_editor/src/tests/editing/transaction_engine/scope.rs
  - zircon_editor/src/tests/editing/transaction_engine/events.rs
  - zircon_editor/src/tests/editing/transaction_engine/locking.rs
  - zircon_editor/src/tests/editing/transaction_engine/recovery.rs
  - zircon_editor/src/tests/editing/transaction_engine/routing.rs
  - zircon_editor/src/tests/editor_message/bus/fixture.rs
  - zircon_editor/src/tests/editing/context_transactions.rs
  - zircon_editor/src/tests/ui/boundary/editor_event_cutover.rs
doc_type: module-detail
---

# Editing transaction engine

`core::editing::engine` is the headless transaction owner for editor mutations. M1.1 establishes the command, history, transaction, routing, and internal-event contracts. M1.2 attaches one engine instance to every `EditorContext`; `EditorContextBuilder` supplies a core-owned `CoreEditContext`, so this wiring introduces no UI dependency.

## Command boundary

`EditCommand` applies immediately when pushed into a `TransactionScope`, reverts for undo or cancellation, and receives a deterministic `finalize` callback whenever a record is discarded. `apply` and `revert` failures use `CommandExecutionError`, which preserves an `EditCommandError` source and explicitly reports whether the failed call left the command effect unchanged or applied. The engine invokes a reverse recovery only for an applied effect; it does not guess from private command state. Errors remain typed, including source-preserving reflection and external-effect variants, and the engine exposes no string error channel.

The engine owns a narrow `EditContext` trait instead of importing UI state. A command can obtain its concrete fixture or future service adapter through `Any`, while selection capture and restoration stay explicit transaction operations. `SelectionSnapshot` is a journal-ready JSON value wrapper until the Selection Model plan supplies the final typed selection DTO.

## History invariants

Each `HistoryContextId` is either `Global` or `Document(DocumentId)`. The former empty numeric identifier API was removed from the message subsystem, and transaction messages now use the editing-owned identifier directly.

`HistoryStore` uses a `VecDeque<TransactionRecord>` plus a cursor. Committing after undo drains and finalizes the redo segment. Capacity pressure evicts and finalizes the oldest record. `saved_top` and its reachability flag are the sole dirty-state authority. Evicting the saved first record shifts its reachable baseline to the position before the retained first record; truncation or deeper eviction marks the baseline unreachable only when no retained undo path can return to it.

Every record retains its participants, before/after selection snapshots, significance, and engine frame number. No wall-clock timestamp participates in history ordering.

## Scope and merge behavior

Scopes are RAII guards. `push` applies a command immediately; explicit cancellation and uncommitted drop revert commands in reverse order and finalize them. Same-history nesting folds child commands and participants into the parent record. A cross-history nested begin returns `EditCommandError::CrossContextNested`.

The engine uses one state mutex, but removes the edit context and the active history object before invoking selection hooks, command methods, merge hooks, finalizers, or public inspection closures. Public `with_context` inspection is allowed only when no transaction scope is active, so an inspection closure cannot capture and synchronously terminate the scope whose operation lane it owns. Reentrant or ordinary concurrent operations receive `EngineBusy` instead of waiting on a reverse lock order. Scope termination is stronger: `commit`, `cancel`, and `Drop` wait on an operation condition variable and retry after a concurrent callback lane publishes completion, preventing an active scope from being orphaned by transient contention. If command or selection recovery itself fails, the engine retains the affected active/history state, enters an explicit faulted state, and rejects further mutation with `EngineFaulted`; retained commands are not finalized as though recovery had succeeded.

`MergeMode::Disable` keeps every command, `Ends` offers only the latest command as a merge target, and `All` searches backward through all commands in the active scope. An absorbed command is finalized immediately.

## Events and validation status

The engine records the five internal states `Started`, `Canceled`, `Committed`, `UndoApplied`, and `RedoApplied`. It does not publish them to the editor message bus in M1.1; that projection is reserved for M4.

`UndoableEditorOperation` remains descriptor metadata for discovery, but event dispatch no longer converts that label into synthetic history. The former `EditorOperationStack`, its entry type, `operation_state.stack`, and the runtime accessor were deleted without aliases or re-exports. Undo and redo events therefore cannot move metadata between fake stacks. Until M3 installs the edit-command factory, `QueryOperationHistory` returns the typed `OperationHistoryPendingFactory` response instead of presenting journal labels as executable history.

The dedicated source tests cover cursor truncation, capacity eviction, dirty state, RAII cancellation, nesting, merge modes, failure rollback, record metadata, routing, event order, context engine identity, empty transaction history, and the old-operation-stack source boundary. The boundary guard now treats `operation_state.rs` as a deleted legacy aggregate, requires the folder-backed `editing/context.rs` and `editing/engine/{history,transaction}.rs` owners, and verifies the current `HistoryStore`/`EditorTransactionEngine` exports without restoring a facade.

On 2026-07-14, a standalone `rustc --test` harness directly compiled the current `editor_event_cutover.rs` source together with its support module. Its three editing/event boundary tests passed; the combined Editor03/Render01 guard run was 6 passed / 0 failed in 9.72 seconds (`.codex/tmp/editor03-render01-guard-standalone-20260714.log`). This is focused source-contract evidence only: the shared full Cargo library gate remains blocked by separately owned failures and is not reported as green here.

## Performance-sensitive mutation paths

The 2026-07-17 pass removed a redundant batch collect. `UpdateNodeCommand` now captures the node
once, derives before/after snapshots, and applies only fields whose directional values differ;
single-field edits no longer recapture or rewrite parent, name, and transform together. Nested
cancel collects frames in normal order and restores them by `pop`, preserving reverse rollback
without repeated front removal and shifting.

The two source guards completed RED-to-GREEN and scoped formatting/diff checks pass. Current-source
transaction behavior tests and an interaction trace are still pending, so this is not dynamic
performance acceptance.
