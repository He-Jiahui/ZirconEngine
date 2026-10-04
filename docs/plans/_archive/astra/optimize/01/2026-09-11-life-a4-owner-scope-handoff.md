---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-11
plan: docs/plans/astra/optimize/01-review-and-repair.md
source_plan: docs/plans/astra/features/runtime/07-lifecycle-deadline-and-census.md
milestone: LIFE-A4 dynamic-session owner retention and teardown retry
session: astra-life-a4-owner-scope-record-20260911
primary_session: astra-life-a4-owner-audit-20260911
reviewer_session: astra-life-a4-abi-handoff-20260911
---

# LIFE-A4 dynamic-session owner-scope handoff

## Finding status

This handoff is `blocked_owner_scope`. It records an ownership and ABI boundary,
not an implementation or acceptance claim. No lifecycle source was edited by
this record session. The active implementation owner
`astra-life-a4-owner-audit-20260911` owns
`session_owner.rs`, `session_owner/runtime.rs`, and `session_owner/tests.rs`;
the reviewer `astra-life-a4-abi-handoff-20260911` has no write scope. A separate
exact lease was used for this child record and is released after the write.

## Current evidence and the dummy code owner

`SessionSlot` currently supplies a placeholder owner at every construction
boundary: the test constructor uses `Arc::new(())` at
`zircon_runtime/src/dynamic_api/session/registry/session_slot.rs:39-47`, normal
creation repeats it at `:59-72`, and linked-plugin creation repeats it at
`:94-108`. `RuntimeSessionOwner::create` accepts an
`Arc<dyn Any + Send + Sync>` and retains it through `SessionOwner` at
`zircon_runtime/src/dynamic_api/session/registry/session_owner/runtime.rs:42-70`,
but the slot has no real dynamic-library lease to pass.

The existing owner regression is the narrow contract that is already present:
`runtime.rs:197-266` blocks one event callback, observes `Pending` on two
shutdown attempts, asserts invocation count remains one, retains the code-owner
Arc until the owner is dropped, and then observes `Joined`. It does not prove
that an application-loaded runtime DLL is retained across a real ABI session
retry or unload.

TaskGraph itself already freezes and reuses its scope set: its shutdown path
uses one absolute `Instant` deadline and `begin_shutdown` retains the closing
scope set (`zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs:151-184,240-262`),
and the focused regression preserves the same census after handles are dropped
(`zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph/tests.rs:142-197`).
The remaining lifecycle gap is propagation/retention across the dynamic owner,
not a new TaskGraph census implementation.

The outer destroy path does establish one deadline and passes it into slot
shutdown (`zircon_runtime/src/dynamic_api/session/registry/session_store.rs:260-324`).
However, `RuntimeDynamicSession::shutdown_before_library_unload_until` returns a
boolean and exits on the first failed phase (`zircon_runtime/src/dynamic_api/session/state.rs:184-230`),
so an incomplete receipt/report is not yet a durable per-owner retry contract.

## ABI and version boundary

The runtime API table is explicitly frozen at V8; adding a field requires a new
table version and coordinated hard cutover
(`zircon_runtime_interface/src/runtime_api/abi/api_table.rs:94-129`). The current
host table V1 carries only ABI/size, diagnostics sink, and resource fetch
(`api_table.rs:74-92`), while `ZrRuntimeSessionConfigV3` carries profile, project
root, play-scene/report fields, and wake sink but no code-owner handle
(`zircon_runtime_interface/src/runtime_api/session/session.rs:41-69`). The
runtime export validates the host pointer/table but has no host-library owner
retention (`zircon_runtime/src/dynamic_api/exports.rs:24-79`).

Consequently, a real application `LoadedRuntime`/DLL owner cannot be wired by
changing `session_slot.rs` alone. The App currently stores the private
`libloading::Library` inside `LoadedRuntime`
(`zircon_app/src/entry/runtime_library/loaded_runtime.rs:28-35,147-182`), and
`RuntimeSession::try_destroy` retains that runtime when destroy is incomplete
(`zircon_app/src/entry/runtime_library/runtime_session.rs:180-207`). A new
versioned owner/lease contract must cross the ABI boundary and preserve the
V8/V3 shape for the coordinated hard cutover. The private
`NativePluginStableLibrary` used by the native-plugin loader is a plugin
generation lease, not a substitute for the App's runtime DLL owner.

## Lowest-owner order

1. **Interface/ABI owner:** define the versioned code-owner lease/retention
   contract and its size/version/skew rules. Keep V8/V3 immutable until every
   dynamic host and consumer is migrated in one hard cutover.
2. **App/Runtime10 owner:** adapt `LoadedRuntime`/`RuntimeSession` to provide the
   real DLL lease and quarantine it until all owner callbacks, diagnostic
   workers, TLS, and runtime workers have joined. The open owner failure is
   `docs/plans/zircon_runtime/runtime/10/failure-2026-08-10-dynamic-runtime-dll-unload-worker-lifetime.md`.
3. **LIFE-A4 runtime owner:** after (1) and (2), wire the lease through
   `SessionSlot` and creation, then carry a durable pending receipt through
   action/wake, event mirror, watcher, scope, module, TaskGraph, and process-log
   teardown. Retries wait the original noncooperative operation and never invoke
   it twice. This is the active owner scope and must receive a fresh lease before
   edits.
4. **Scene/plugin owners as needed:** settle main-thread World/FFI callback
   affinity and intentional event-mirror compensation/rollback semantics before
   adding an exact-once assertion to those paths.

## Required managed evidence

The following evidence is required before changing the status to
`implemented_pending_validation` or `accepted`:

- Focused Rust owner contract: a noncooperative callback receives one invocation,
  returns a pending receipt on timeout, is not reinvoked on retry, retains the
  real code-owner lease, and reaches a terminal joined receipt only after the
  callback releases.
- Registry integration: one destroy attempt shares one absolute deadline across
  action drain, wake callbacks, event mirrors, watchers, session scope, module
  shutdown, TaskGraph workers, and process log; every unfinished owner remains
  available for explicit retry.
- Retry census: after the blocked work completes and external handles are
  dropped, the retry and every later successful shutdown report the same frozen
  final TaskGraph scope census.
- ABI contract/skew tests: owner handle/table size and version checks reject
  unsupported callers without borrowing a transient DLL pointer; all consumers
  retain the owner through the final callback and unload boundary.
- Managed Windows real-DLL evidence: repeated create/destroy, forced timeout,
  retry, worker/TLS/diagnostic-log joins, and unload prove no code executes after
  `Library` release. Record the immutable source manifest, dependency snapshot,
  command receipt, and actual test counts under the managed validation policy.

No Cargo, native, DLL, or product command was run for this handoff. Validation is
therefore pending; the current status remains `blocked_owner_scope`.

## Overlap and admission boundary

Do not edit the active `session_owner.rs`, `session_owner/runtime.rs`, or
`session_owner/tests.rs` lease. The worktree also contains foreign dirty changes
in `session_slot.rs`, `session_store.rs`, `runtime_session.rs`, and registry
tests; those changes are not evidence of this handoff. Existing event-mirror
rollback compensation can intentionally call a callback during rollback, so a
generic exact-once test must not be added until its owner specifies the callback
contract. Keep this record open for the Interface/App owner transfer and a fresh
LIFE-A4 source lease; do not mark the finding fixed merely because the placeholder
Arc exists or a unit test passes.

## Lower-layer LIFE-A1/A2/A3 follow-up audit (2026-09-11)

This follow-up searched for a narrowly unowned fix below the active
`session_owner` owner. No safe source slice was admitted. The coordinator
ownership matrices show the relevant candidate files as dirty but unowned, with
stale or archived attribution and no live lease:

- `zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs`,
  `task_graph/shutdown.rs`, and the related task-graph files are attributed to
  archived `astra-full-domain-20260905` (matrix receipt
  `ebd777f36bd444bda1e8d07c9eef97ad`); the active source hash and baseline are
  explicitly stale.
- `zircon_runtime/src/dynamic_api/session/state.rs`, `registry/session_slot.rs`,
  `registry/session_store.rs`, and registry tests are likewise dirty with
  archived/stale attribution and no live lease (matrix receipt
  `757c83ba05cb4557b7a96363d9913157`).
- Even the lower `core/runtime/tasks/pool.rs` candidate is dirty and attributed
  to archived `text09-owner-prewarm-repair-20260809`, with no lease (matrix
  receipt `9cbed951c2524b97a779c135bab45111`); it is not an unmodified safe
  owner boundary.

The active executable LIFE-A4 session still owns only
`registry/session_owner.rs`, `registry/session_owner/runtime.rs`, and
`registry/session_owner/tests.rs`. The clean task-graph support files
`scope_model.rs`, `scope_registration.rs`, and `lease.rs` contain no deadline or
census transition and therefore offer no justified lower-layer fix.

### What is already present

The current candidate implementation does preserve TaskGraph census state:
`engine_task_graph.rs:33-39` stores `shutdown_scopes` and
`stopped_scope_census`; `:240-262` freezes the closing scope set and saves the
final census; `:268-299` serves that saved census after `Stopped`. The focused
regression at `engine_task_graph/tests.rs:143-197` drops external handles,
retries, and compares the repeated final census. `TaskPool::shutdown_until` at
`core/runtime/tasks/pool.rs:215-221` consumes the caller's remaining absolute
deadline, while `close_and_join` at `:223-263` retains join authority when the
deadline or shutdown gate is not available. No isolated defect in these lower
primitives was found that could be fixed without reopening the stale owner.

Absolute-deadline plumbing is also present through the candidate chain:
`session_store.rs:270-306` creates the destroy deadline and passes it to the
slot; `state.rs:184-230` passes it through event mirrors, watchers, scope,
modules, TaskGraph, and process log; `core/runtime/runtime.rs:207-213,345-363`
and `handle/activation.rs:190-200,290-301` consume it for TaskGraph and module
teardown; watcher, event-mirror, and process-log owners retain pending work on
failure. Existing registry tests cover individual bounded owners and a scope →
process-log shared budget (`registry/tests.rs:420-506`), but not the full
cross-owner receipt contract.

### Remaining lowest-owner defect and required handoff

The remaining defect is at the dynamic owner/report boundary, not in the clean
TaskGraph primitives: `RuntimeDynamicSession::shutdown_before_library_unload_until`
returns only `bool` and exits on the first failed phase
(`session/state.rs:184-230`); `SessionSlot`/`RuntimeSessionOwner` collapse the
lower report into `OwnerShutdownReceipt`, and `session_store.rs:304-312` keeps
only the `Joined` decision. Thus a caller cannot receive or retain a durable
per-phase pending receipt/final census across a failed dynamic destroy attempt.
Repair requires the active owner plus the ABI/App owner boundary, not an
isolated edit to TaskGraph, TaskPool, or a clean helper file. The existing
Runtime10 DLL-unload failure owner and the ABI handoff in this record remain
the correct next owners.

Required follow-up evidence remains: one managed destroy attempt covering all
phases against one absolute deadline; a typed receipt retaining every unfinished
owner; retry/repeated success returning the frozen TaskGraph census; and a real
Windows DLL unload/retry run proving owner/code retention. No Cargo, native, or
DLL command was run in this audit.
