---
related_code:
  - zircon_editor/src/core/editing/engine/mod.rs
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/engine/transaction/dirty_batch.rs
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
  - zircon_editor/src/core/editing/engine/transaction/dirty_batch.rs
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
  - zircon_editor/src/core/editing/engine/transaction.rs::tests::nested_cancel_does_not_remove_from_the_front_of_a_vec
  - zircon_editor/src/tests/editing/history.rs
  - zircon_editor/src/tests/editing/node_ops.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
  - zircon_editor/src/tests/editing/inspector.rs
  - zircon_editor/src/tests/editing/transaction_engine/history.rs
  - zircon_editor/src/tests/editing/transaction_engine/dirty_batch.rs
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

## Scene command integration

`EditorCommand` is the scene mutation family consumed by the shared engine. Its four variants are create, delete, update, and reflected-field write. Constructors only capture intent or before/after values from `&Scene`; they never mutate the world. `TransactionScope::push` is the first mutation point. The deleted `Batch` variant is replaced by multiple pushes in one scope, and command-local selection fields are replaced by the transaction record's before/after selection snapshots.

`CoreEditContext` binds the active `LevelSystem` handle plus a typed `SceneSelectionSnapshot` immediately before a scene transaction. Workbench create/delete/rename/reparent/transform/import/inspector actions, undo, and redo all use `EditorContext::transactions()` with `HistoryContextId::Global`; `EditorState` has no private history owner. Replacing or closing a project clears and finalizes the shared Global history before the bound scene is replaced.

Project replacement, project close, and play enter/exit hold an exclusive transaction-engine transition from gizmo cancellation through world mutation. Project transitions finalize Global history and clear `CoreEditContext` inside that same operation lane, so another command cannot bind the old world or append old-world history between cleanup steps. Ordinary scene actions cancel a gizmo preview before command capture; the shared executor rejects a leaked active capture, while gizmo release alone uses the private already-applied commit path.

Gizmo interaction previews transforms directly while the gesture retains only the initial and latest `Transform`. Release constructs one already-applied transform command and commits it through one transaction scope; a 100-frame drag therefore produces one transaction and one retained command without per-frame command or node-name allocation. This staging is gesture-local transform data, not a second undo stack.

Play mode retains the edit history but rejects scene mutation, undo, and redo intents while the play snapshot is active. Gizmos are disabled for the play session and the prior setting is restored on exit, preventing viewport handle mutation from bypassing the transaction guard. Snapshot `can_undo/can_redo` is false while playing and reflects the retained edit history again after exit.

Production state construction requires an explicit `Arc<EditorContext>`. Test-only convenience constructors may build a fixture Context under `#[cfg(test)]`; production retained-host and CLI operation paths inject the owning `EditorManager` Context so commands, jobs, gateway, and transactions stay single-owner.

## Events and validation status

The engine records the five internal states `Started`, `Canceled`, `Committed`, `UndoApplied`, and `RedoApplied`. It does not publish them to the editor message bus in M1.1; that projection is reserved for M4.

`UndoableEditorOperation` remains descriptor metadata for discovery, but event dispatch no longer converts that label into synthetic history. The former `EditorOperationStack`, its entry type, `operation_state.stack`, and the runtime accessor were deleted without aliases or re-exports. Undo and redo events therefore cannot move metadata between fake stacks. Until M3 installs the edit-command factory, `QueryOperationHistory` returns the typed `OperationHistoryPendingFactory` response instead of presenting journal labels as executable history.

The dedicated source tests cover cursor truncation, capacity eviction, dirty state, RAII cancellation, nesting, merge modes, failure rollback, record metadata, routing, event order, context engine identity, empty transaction history, and the old-operation-stack source boundary. The boundary guard now treats `operation_state.rs` as a deleted legacy aggregate, requires the folder-backed `editing/context.rs` and `editing/engine/{history,transaction}.rs` owners, and verifies the current `HistoryStore`/`EditorTransactionEngine` exports without restoring a facade.

On 2026-07-14, a standalone `rustc --test` harness directly compiled the current `editor_event_cutover.rs` source together with its support module. Its three editing/event boundary tests passed; the combined Editor03/Render01 guard run was 6 passed / 0 failed in 9.72 seconds (`.codex/tmp/editor03-render01-guard-standalone-20260714.log`). This is focused source-contract evidence only: the shared full Cargo library gate remains blocked by separately owned failures and is not reported as green here.

## Performance-sensitive mutation paths

The 2026-07-17 pass removed a redundant batch collect. `UpdateNodeCommand` captures the node once,
derives before/after snapshots, and applies only fields whose directional values differ;
single-field edits no longer recapture or rewrite parent, name, and transform together. Nested
cancel collects frames in normal order and restores them by `pop`, preserving reverse rollback
without repeated front removal and shifting. The 2026-07-18 hard cut removes the scene-only
`EditorHistory` and `Batch` command entirely, so scene history no longer performs front removal or
duplicates transaction storage.

The Editor03 M2 source contract is 13/13. Non-selection edits
preserve multi-selection, the shared scene executor rejects play-mode mutation, and viewport input
is the sole fallible gizmo transaction owner with transform rollback on transaction failure. The
old host `GizmoDragState` and gizmo lifecycle intents are physically removed. Gesture staging keeps
only initial/latest transforms; ordinary actions cancel preview before capture, and every rollback
path clears lifecycle state even when transform restoration fails. Project replacement, project
clear, and play entry/exit use one exclusive engine transition across preview cleanup and world
switching; project history/context cleanup occurs inside that barrier. Viewport rollback failures
invalidate render as well as presentation. Scoped formatting and diff checks remain pending after
the latest independent-review fixes.
Independent incremental source review is 0/0/0. Current-source Cargo behavior and product
interaction remain pending behind the Coordinator01 validation-copy terminal-evidence failure, so
this is not dynamic performance acceptance.

## Atomic saved-top completion

Saving a document uses `EditorTransactionEngine::capture_save_token(history)` before I/O and
`mark_saved_if_unchanged(history, token)` only after the write succeeds. The token binds the history
context, originating engine lineage, current transaction identity (or the empty root), and a
per-history branch generation. Committed records, successful undo/redo, redo replacement, capacity
eviction, and history clear invalidate older tokens. Capture and completion reject an active
transaction scope; completion updates `saved_top` under the same operation lane and state lock and
returns `HistorySaveMarkOutcome::{Marked, AlreadyMarked}` or a typed engine/history/change error.

Operation groups publish an identity-bearing `Initializing` reservation before creating their root
transaction, bind that reservation to the new transaction before the first push, and transition
through `Open` and `Flushing`. `begin_transaction` validates reservation ownership in the same state
lock that creates the active frame, so a caller that passed an earlier flush cannot cross a newly
published group. Failed flushes restore only the matching group; delayed continuation cleanup cannot
delete a successor group, and an unrecoverable first-push rollback preserves the original command and
rollback errors while removing the failed reservation.

The former unconditional transaction `mark_saved(history)` API is deleted. Callers must not compare
`HistorySnapshot.top` outside the engine, cache a dirty boolean, or assume the UI thread excludes a
concurrent commit. External-effect revisions remain Editor09-owned and are cleared independently.

## Dirty batch generation

`EditorTransactionEngine::dirty_states_since` publishes an engine-lineage-bound
`HistoryDirtyCursor` and `HistoryDirtyBatchKind::{Unchanged, Delta, Reset}`. A 4,096-entry journal
records only changed `HistoryContextId` values. Stable cursors return an empty state slice before any
history-set construction; live cursors deduplicate only journal delta; cursors older than retained
history receive Reset. Reset includes histories known by the engine, including cleared histories
whose branch generation is retained.

Commit, successful undo/redo and exclusive history clear advance branch generation and dirty
generation together. A successful `mark_saved_if_unchanged` that moves `saved_top` advances only the
dirty generation: the clean transition becomes observable without invalidating the same save token's
idempotent `AlreadyMarked` completion. The journal never stores a dirty boolean; each returned state
derives it from the current `HistoryStore::is_dirty` under the engine lock.
