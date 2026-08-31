---
related_code:
  - zircon_editor/src/core/editing/engine/mod.rs
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/journal/
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
  - zircon_editor/src/core/editing/engine/journal/
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

The engine owns a narrow `EditContext` trait instead of importing UI state. Selection capture,
restoration, and world-route retirement stay explicit transaction operations. Runtime-backed
extension commands request the bounded `runtime_operations` capability instead of obtaining a raw,
replaceable gateway handle. `EditorRuntimeOperationRoute` retains one immutable gateway origin, so
submit, poll, and harvest cannot cross runtime generations. Concrete downcasts through `Any` remain
limited to commands owned by the same context implementation.

## History invariants

`HistoryContextId` is partitioned into persistent authoring contexts (`Global` and
`Document(DocumentId)`) and the instance-qualified volatile context
`PlaySession(PlayInstanceId)`. `HistoryContextId::world_domain` is the canonical mapping to
`WorldDomain`; history routing requires that domain explicitly and returns
`EditCommandError::CrossWorldHistory` instead of allowing an edit-world history to contain a play
entity, or the inverse. The former empty numeric identifier API was removed from the message
subsystem, and transaction messages use the editing-owned identifier directly.

Play history has no saved baseline. Its public dirty state is always false, it is excluded from
dirty batches, and save-token plus transaction-journal persistence APIs reject it with typed
errors. `EditContext` exposes `capture_world_route`, `activate_world_route`, and
`retire_world_route`; the deleted
single-gateway `runtime_gateway` method has no compatibility path. `CoreEditContext` owns stable
authoring and play gateway handles plus one selection slot per `WorldDomain`. The same play handle
is injected into the transaction context and `PlayDomainLink`, while the link remains the sole
attach/detach authority.

A root transaction captures the exact `WorldDomain + GatewaySessionIdentity`, nested scopes inherit
that route, and every retained `TransactionRecord` keeps it. Undo/redo activates the target record's
route instead of recapturing the current gateway generation. Replacing a gateway for the same play
instance therefore yields `WorldRouteStale`; an old command cannot be redirected into a replacement
world. `discard_play_history(instance)` is the per-instance whole-stack cleanup owner: while the
gateway is still reachable it finalizes records, removes the store and branch-generation entry,
reactivates the authoring route, and retires the play selection slot even when the play history is
empty. The host's terminal detach path reserves the exact instance and gateway identity under the
`PlaySessionController` transition gate, releases the gate while extension finalizers run, and then
reacquires it for identity-qualified detach. Reentrant detach and premature backend retirement are
typed rejections, avoiding both callback deadlock and session destruction during cleanup. Normal
stop, crash, project close, and host shutdown share this order.

`HistoryStore` uses a `VecDeque<TransactionRecord>` plus a cursor. Committing after undo drains and finalizes the redo segment. Capacity pressure evicts and finalizes the oldest record. `saved_top` and its reachability flag are the sole dirty-state authority. Evicting the saved first record shifts its reachable baseline to the position before the retained first record; truncation or deeper eviction marks the baseline unreachable only when no retained undo path can return to it.

Every record retains its exact world route, participants, before/after selection snapshots,
significance, and engine frame number. No wall-clock timestamp participates in history ordering.

## Scope and merge behavior

Scopes are RAII guards. `push` applies a command immediately; explicit cancellation and uncommitted drop revert commands in reverse order and finalize them. Same-history nesting folds child commands and participants into the parent record. A cross-history nested begin returns `EditCommandError::CrossContextNested`.

The engine uses one state mutex, but removes the edit context and the active history object before invoking selection hooks, command methods, merge hooks, finalizers, or public inspection closures. Public `with_context` inspection is allowed only when no transaction scope is active, so an inspection closure cannot capture and synchronously terminate the scope whose operation lane it owns. Reentrant or ordinary concurrent operations receive `EngineBusy` instead of waiting on a reverse lock order. Scope termination is stronger: `commit`, `cancel`, and `Drop` wait on an operation condition variable and retry after a concurrent callback lane publishes completion, preventing an active scope from being orphaned by transient contention. If command or selection recovery itself fails, the engine retains the affected active/history state, enters an explicit faulted state, and rejects further mutation with `EngineFaulted`; retained commands are not finalized as though recovery had succeeded.

`MergeMode::Disable` keeps every command, `Ends` offers only the latest command as a merge target, and `All` searches backward through all commands in the active scope. An absorbed command is finalized immediately.

## Scene command integration

`EditorCommand` is the scene mutation family consumed by the shared engine. Its four variants are create, delete, update, and reflected-field write. Constructors only capture intent or before/after values from `&Scene`; they never mutate the world. `TransactionScope::push` is the first mutation point. The deleted `Batch` variant is replaced by multiple pushes in one scope, and command-local selection fields are replaced by the transaction record's before/after selection snapshots.

`CoreEditContext` binds the authoring `LevelSystem` facade and keeps the replaceable Play facade
separate. A transaction selects the facade only through its captured world route; entity identifiers
never choose a gateway implicitly. `DocumentLifecycleAuthority` remains the only authoring scene
identity authority; after it commits an activation, `EditorState` holds only that committed
`DocumentId` binding. Authoring scene commands, gizmo release, undo/redo, save marking, close
prompts, snapshots, and event tracing use `HistoryContextId::Document(document)` with no `Global`
fallback. An unbound scene refuses editing. Replacing or closing a project clears and finalizes the
bound document history before the bound scene is replaced.

Project replacement, project close, and play enter/exit hold an exclusive transaction-engine transition from gizmo cancellation through world mutation. Project transitions finalize the bound document history and clear `CoreEditContext` inside that same operation lane, so another command cannot bind the old world or append old-world history between cleanup steps. Ordinary scene actions cancel a gizmo preview before command capture; the shared executor rejects a leaked active capture, while gizmo release alone uses the private already-applied commit path.

Gizmo interaction previews transforms directly while the gesture retains only the initial and latest `Transform`. Release constructs one already-applied transform command and commits it through one transaction scope; a 100-frame drag therefore produces one transaction and one retained command without per-frame command or node-name allocation. This staging is gesture-local transform data, not a second undo stack.

The transaction engine supports instance-qualified Play mutation and replay without exposing a
persistent dirty state. The workbench resolves the active selection domain before exposing history:
Play Inspector batches and Ctrl+Z/redo use
`HistoryContextId::PlaySession(attached_instance)`, while an Edit selection does not fall back to the
retained authoring history during Play. Inspector command capture occurs inside the transaction's
already-pinned Play context, so partial transform batches preserve the other runtime axes and never
read authoring drafts as their before value. Authoring edit history remains intact across Play and is
visible again after exit. Play gizmos remain disabled until the secondary session exposes an
identity-qualified renderer picking and editor-overlay contract; authoring viewport hits must not be
reused as Play entity identity.

Undoable operation registrations require an explicit `EditOperationTarget` owned by the generic
editing layer; there is no implicit workspace default or post-construction target setter.
During Play, the host defers and routes an invocation through `PlaySessionController::route_edit`
before calling the operation factory. `PlayDomain` operations can capture immediately, running
document edits are rejected, and other document/workspace operations enter the bounded pending-edit
queue. A queued operation publishes `EditorOperationEvent::EditQueued` without creating a command or
touching history. The former Play-owned target type is deleted rather than retained as an alias.

`runtime.play_mode.keep_changes` is the only explicit Play-to-authoring property bridge. The command
is available from the Play menu/command palette and is injected into scene-row context menus only
while the controller is Playing; every surface dispatches the same typed `MenuAction`. It requires
one selected entity in the active `WorldDomain::Play(instance)`, pins the current play gateway
identity, and requests a fresh typed Inspector projection without a generation hint. The command
copies only fields marked both writable and serializable, excludes `Hierarchy.parent`, and rejects
runtime-spawned entities that have no authoring counterpart. It captures every reflected command
before opening one `HistoryContextId::Document(document)` transaction, so schema/counterpart failure
cannot leave a partial authoring transaction. The Play selection and Play history remain untouched;
the resulting authoring transaction becomes undoable after exiting Play. An Edit selection while
Play is still active does not bypass the existing authoring-history lock.

`EditorPlaySession` no longer stores an authoring-world clone or replaces the authoring world on
exit. With separate authoring and Play gateways, that legacy restore was redundant and would erase
changes accepted by Keep Changes. Play exit restores only editor selection, gizmo settings, and the
pre-Play session mode.

Production state construction requires an explicit `Arc<EditorContext>`. Test-only convenience constructors may build a fixture Context under `#[cfg(test)]`; production retained-host and CLI operation paths inject the owning `EditorManager` Context so commands, jobs, gateway, and transactions stay single-owner.

## Durable journal boundary

`engine/journal/` is folder-backed: command payload declarations, transaction serialization, codec registration/replay, and durable file storage have separate owners. A `JournalDocumentKey` is derived only from a validated project-relative UTF-8 source path; session-local `DocumentId` never names a journal directory. The durable owner writes `<project>/.zircon/journal/<document-key>/transactions.zjr` with the `ZRJNL001` container and format version 2, followed by monotonically sequenced length-delimited BLAKE3-checked records. Each record payload independently uses the shared `$zircon` schema `zircon.editor.editing.transaction-journal` v1. The retired raw JSON DTO and its private `schema_version` field have no reader. Command-level `schema_version` remains a separate business contract used by `EditCommandCodecRegistry`; it is not the transaction document or container version. Each accepted append calls `sync_data`; the reader enforces container, file, record, checksum, transaction schema, sequence, and valid-prefix boundaries. A corrupt or truncated tail remains visible as typed recovery state and is rejected for further append rather than being silently extended. `JournalTailFault::InvalidTransaction` owns the original `TransactionJournalReadError`, so the source chain remains `DurableJournalError -> JournalTailFault -> TransactionJournalReadError -> LoadError`; startup recovery can distinguish a future schema from malformed payload or engine-invariant failure without parsing display text.

`EditCommandCodecRegistry` is the sole command payload decode authority. `core/editing/journal_codecs/` owns the explicit v1 registrations for scene create, delete, update, and reflected-field commands; it decodes owned DTOs and rejects incompatible create intent/record kinds, noncanonical or no-op node updates, and incomplete or no-op reflected writes before a scope is opened. A delete journal remains a forward descriptor and never serializes its move-only detached inverse batch. `TransactionJournalReplayer` validates and decodes every command before it opens a target scope, then replays through the normal transaction engine using the caller's live history context. It does not reuse the persisted session-local history identifier. The durable owner can explicitly compact a snapshot-covered prefix and read-only discover journal directories into valid entries plus typed isolated issues; linking an autosave checkpoint to compaction and turning discovery into a startup recovery selection remain separate Recovery Coordinator work. This layer intentionally exposes typed artifacts rather than creating a second document or autosave authority.

`core/recovery/document_journal/DocumentJournalCoordinator` is the project-scoped bridge for that work. It holds `DocumentId -> JournalDocumentKey` only in process memory, derives the key from its own project root plus the physical source path, and gives each document an independent writer slot. Project admission creates the one coordinator; startup activation and picker routes reserve a `DocumentId`, bind it before authoring-world installation/lifecycle publication, release it on failed publication or a later scene close, and never infer identity from a `res://` URI. Production append is deliberately absent: only `#[cfg(test)] append_for_test` currently exercises writer ownership, because the transaction engine does not yet publish immutable materialized bytes at commit linearization. Journal persistence never consumes the lossy editor message bus or a `Global` history record. Reads and prefix compaction serialize within the document append gate, and compaction drops the old writer before replacing the file. Immutable commit capture, save/close drain, autosave checkpoint wiring, and startup candidate selection remain product work.

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

Persistent-history commit, successful undo/redo and exclusive history clear advance branch
generation and dirty generation together. Volatile play-history mutations advance only their
per-history branch generation and never enter the dirty journal. A successful
`mark_saved_if_unchanged` that moves `saved_top` advances only the
dirty generation: the clean transition becomes observable without invalidating the same save token's
idempotent `AlreadyMarked` completion. The journal never stores a dirty boolean; each returned state
derives it from the current `HistoryStore::is_dirty` under the engine lock.
