---
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/ScopedTransaction.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/TransBuffer.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorTransaction.cpp
  - dev/Fyrox/editor/src/command/mod.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/coverage.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: static_complete_dynamic_pending
created_at: 2026-08-16
---

# Editor authoring transaction current-architecture review (2026-08-16)

## Status and decision

- Result: `static_complete / dynamic_pending`.
- Scope: `zircon_editor/src/core/editing/**`, 29/29 Rust files, 4,742 physical lines,
  8 inline tests, ordered path-and-raw-content SHA256
  `fbb8b6e6b5f558641de1b878abfe648605739c706d3da9a955c147671e329cd1`.
- Every scoped Rust file was read in full. Production reachability was followed through editor event
  dispatch, workbench intent application, scene capture/apply, project clear, history/journal,
  inspection publication, snapshot building and render extraction.
- This review supersedes the 24-file/4,611-line manifest in
  `2026-07-30-editor-core-editing-current-review.md`, while retaining its accepted corrections and
  open PERF-MVP-063/549/600 ownership.
- Code disposition: no Rust source was changed. Five scoped files are modified and five transaction
  implementation files are untracked by other work; the dominant defects require a coordinated
  contract cut, not a local branch or effect-enum patch.
- Accounting: keep `zircon_editor/src/core/editing/**` in `pending.md`. It cannot enter `review.md`
  until A1-A7 have current-source managed Windows and product evidence.

No latency, throughput, power or algorithm-optimality conclusion is authorized by this static pass.
WPR/xperf and RenderDoc are still blocked by the recorded approved-root editor build failure.

## Per-file review

| file | current-source result |
|---|---|
| `authoring_world.rs` | The stable editor gateway boundary prevents UI ownership of Runtime scene types. Each callback still enters the one authoring `World` owner for its full duration; callers perform repeated capture/apply entries and wide snapshot/render/inspection projections inside that lease. Retain the boundary, replace callback-shaped access with immutable generations and one batch commit. |
| `command.rs` | P0: `apply_node_state` performs parent, name and transform as separate fallible mutations, then maps every failure to `CommandEffect::Unchanged`. A late failure can leave an earlier mutation applied while lifecycle rollback skips the failed command. It also retains full `NodeEditState` before/after for field-specific edits, clones stable names, clones reflected values and retains complete deleted subtrees. |
| `context.rs` | Selection snapshots share typed `Arc` payloads, and scene callbacks execute outside the transaction state mutex. The context still exposes callback-shaped mutable world access, so transaction atomicity is delegated to command code rather than enforced by the owner. |
| `engine/command.rs` | The command/effect/merge contracts are compact. `Applied` versus `Unchanged` is caller-reported after a fallible callback, which cannot safely represent partially applied multi-step commands. A prepared mutation receipt must make the effect exact. |
| `engine/events.rs` | Lifecycle events are compact and publish outside the state lock. Preserve this behavior; generation and exact changed-range receipts should replace broad presentation invalidation at consumers. |
| `engine/history.rs` | History uses bounded `VecDeque`, paged detail and outside-lock finalization. Capacity is entry-count only; one retained subtree or reflected payload can dominate RSS, and replay remains per boxed command. Add a byte/resident budget and batch-owned inverse deltas. |
| `engine/journal.rs` | Explicit journal rows avoid eager serialization on the normal edit path. Projection owns selection vectors and command JSON; current `journal_transaction` constructs it under the engine state lock. Move controlled serialization from immutable record handles outside the lock and budget bytes/deadline. |
| `engine/mod.rs` | Re-exports and the 128-entry default only. The entry limit is not a sufficient history memory limit. |
| `engine/routing.rs` | Small typed routing enum with constant-time matching; no independent bottleneck. |
| `engine/transaction.rs` | Central state/condvar/event owner. It correctly releases the state mutex around command callbacks, but one process-wide operation slot serializes unrelated history contexts and feeds unbounded scope waits. |
| `engine/transaction/dirty_batch.rs` | The 4,096-entry generation journal and cursor/reset contract bound retained mutations. Delta construction uses a `BTreeSet`; acceptable only for explicit dirty consumption, not frame polling. Read APIs currently flush operation groups first, so they may mutate/commit. |
| `engine/transaction/engine_state.rs` | One state mutex owns context, histories, active scopes, operation group and the global operation marker. The ownership is understandable but too broad for document-parallel preparation and generation reads. |
| `engine/transaction/exclusive_transition.rs` | P0: a fallible arbitrary update receives mutable typed context, but failure restores only selection. It cannot prove rollback of world or other context mutations. Project clear currently uses this shape. Replace it with prepare-then-generation-swap and an owner-issued commit receipt. |
| `engine/transaction/lifecycle.rs` | Command callbacks/finalization/events run outside the state mutex and nested cancellation is reverse-order. Push only reverts the failed command when its self-reported effect is `Applied`; therefore the `command.rs` partial-effect error is a transaction correctness failure. Commit builds boxed-command records and performs no retained-byte admission. |
| `engine/transaction/operation_gate.rs` | Busy admission itself is immediate, but `wait_for_operation` has no deadline and waits on a condvar until the global operation clears. This is unsafe for the retained UI caller and must not be used by scope completion or Drop. |
| `engine/transaction/operation_group.rs` | A live group merges repeated gestures and preserves one record intent. Flush/cleanup still serializes through the global operation and contains wait-based cleanup; command application remains one world entry per command rather than one mutation batch. |
| `engine/transaction/replay.rs` | Undo/redo remove the store and execute callbacks outside the state mutex, which is positive. Dirty/status/details flush the open group before reading; journal projection runs inside the state lock; replay applies every boxed command rather than one validated authoring batch. |
| `engine/transaction/save_token.rs` | Save tokens share lineage and compact ids/generation. Preserve compare-and-mark semantics and carry authoring generation through the new commit receipt. |
| `engine/transaction/scope.rs` | P0: `cancel`, `commit`, `commit_after_apply`, and `Drop` loop forever on `EngineBusy` and call the no-deadline condvar wait. Scope is deliberately `!Send`, so this can block the UI/main-affinity owner. `set_merge_mode` and `add_participant` also silently do nothing while busy/faulted. |
| `intent.rs` | Small typed declaration only. Intent capture should remain the maximum work performed under the shell lock. |
| `mod.rs` | Module mounting only; no independent work. |
| `operation/command.rs` | Bridges a registered operation to boxed edit commands. The bridge inherits wide payload retention and per-command commit behavior; it should submit a typed mutation batch/ticket. |
| `operation/error.rs` | Error conversion only; no independent bottleneck. |
| `operation/factory.rs` | Factory trait only; no independent hot path. |
| `operation/mod.rs` | Exports only. |
| `operation/pending_edit_retention.rs` | Typed lossless/latest/bounded policy is a sound declaration. Enforcement remains owned by the Play queue; transaction history still lacks its own byte/age budget. |
| `operation/registration.rs` | Registration freezes metadata/factory/retention once. Construction-time string ownership is not an edit hot path. |
| `paths.rs` | Streaming path validation with no retained queue or frame work; no independent bottleneck. |
| `selection.rs` | Selection state and snapshots share `Arc` payloads and avoid prior JSON/full-copy behavior. Preserve this model; selection restoration alone is not a general transaction rollback. |

## Production lock and work chain

The retained event dispatcher acquires `controller.shell().lock()` before matching any event
(`editor_event_execution/dispatch.rs:14-28`). Hierarchy and common scene handlers then call
`shell.state.apply_intent(...)` inside that guard (`hierarchy_event.rs:7-18`, `common.rs:10-18`).
`EditorState::execute_prepared_scene_commands` starts a transaction, pushes each command
synchronously and commits (`editor_state_apply_intent.rs:231-257`). Command capture first enters
`try_with_world` (`:259-267`), while each pushed command enters the world again to apply.

Stable snapshots are also wide callbacks: `snapshot_with_inspector_customizations` keeps the world
lease while filtering hierarchy rows, projecting scene entries and invoking plugin component/schema/
reflection queries (`editor_state_snapshot_build.rs:29-88`). Inspection publication acquires its
publication mutex and shell lock before `try_with_world` (`scene_inspection_publication.rs:193-232`),
and render extraction is likewise callback-shaped (`editor_state_render.rs:18-56`). Consequently a
single ordinary edit can hold the shell lock across transaction admission and multiple authoring
world entries, then trigger broad presentation/reflection/render work.

This is the structural bottleneck to measure. Adding a worker while the shell synchronously waits,
adding a second world cache, or shortening one mutex without changing the generation chain does not
solve it.

## Correctness-first bottlenecks

### P0 - partially applied scene commands are reported unchanged

`UpdateNodeCommand` stores a full before/after state (`command.rs:368-430`). `apply_node_state`
changes parent, then name, then transform through separate fallible calls (`:490-516`), while
`unchanged` always returns `CommandEffect::Unchanged` (`:717-721`). On push failure, lifecycle only
reverts the failed command when the effect equals `Applied` (`lifecycle.rs:50-64`).

Example fault: parent mutation succeeds, rename fails. The engine cancels earlier retained commands
but skips reverting the failed command. The hierarchy can therefore differ from history, selection,
inspection and render generations. Simply changing every error to `Applied` is also incorrect: an
early validation failure has changed nothing, and an attempted revert can itself mutate/fail. The
fix must prepare validation and inverse data before one owner commit, with an exact commit receipt.

### P0 - transaction scope cleanup can block the main-affinity caller forever

The operation gate waits without a timeout (`operation_gate.rs:30-38`). Public scope completion loops
on busy and waits (`scope.rs:153-190`); `Drop` repeats the same unbounded loop (`:198-214`). Because
the scope is `!Send`, a foreign callback or long world mutation can stall the retained UI thread and
make shutdown/non-local error cleanup nondeterministic. Destructors must never wait. Public APIs
should return immediate typed busy/stale outcomes or an explicit bounded completion ticket.

### P0 - exclusive transition rollback does not cover its mutation authority

`clear_history_and_context` gives a closure mutable access to an arbitrary typed context
(`exclusive_transition.rs:13-45`). On error it restores only the selection snapshot (`:45-54`). That
is not an atomic project/world transition. The replacement must be prepared outside the authority,
then moved into one generation-checked swap; failure leaves the old owner untouched.

### P1 - payload and batch complexity is not bounded by changed data

- Rename, reparent and transform all clone/retain `{String, parent, Transform}`; normalization trims
  and allocates the stable name even when the changed field is not name (`command.rs:368-526`).
- Reflected edits retain and clone before/after `ReflectedValue`. Deleted nodes retain full subtree
  records. A 128-entry count cap does not bound bytes or resident resources.
- Every transaction owns `Box<dyn EditCommand>`. A multi-node edit captures once but re-enters the
  world per command on apply/undo/redo.
- Explicit journal serialization traverses commands and builds owned JSON while holding the engine
  state mutex (`replay.rs:103-121`).
- Read-looking history/dirty calls flush the current operation group first (`replay.rs:20-70`). They
  must remain explicit control-plane calls and must not become stable-frame polling.

## Reference-engine evidence and adaptation boundary

### Unreal Engine primary reference

- `FScopedTransaction` defines a lexical Begin/End scope and explicit reentrant Cancel
  (`ScopedTransaction.h:9-48`). Zircon should preserve explicit scope ownership, but not copy a
  destructor that can hide blocking work.
- `UTransBuffer` owns an undo queue, active record counts and a byte `MaxMemory`
  (`TransBuffer.h:15-37`). Before admitting a new top-level transaction it computes current undo
  bytes once, removes oldest records until below budget and explicitly notes the avoided O(N^2)
  pattern (`:68-112`). This is direct evidence for a byte budget in addition to entry count.
- `FTransaction::SaveObject` stores one record per object and `SaveArray` records an exact range
  (`EditorTransaction.cpp:701-742`). Undo/redo saves the current state to a flip record, loads the
  retained state and swaps the two (`:277-350`). `Apply` chooses forward/reverse record order, saves
  all flip states before loading them, and applies the record set as one transaction operation
  (`:814-900`). Zircon should adapt prepare/inverse/apply ordering and exact changed-range records,
  not UObject serialization or global transaction state.
- Unreal refuses undo while a transaction is active rather than waiting (`:1465-1474`) and wraps
  Undo/Redo in CPU profiler scopes (`:1615-1653`, `:1682-1717`). This supports immediate busy results
  and named phase instrumentation.
- Unreal explicitly says partial transaction cancellation is unsupported and cancels the whole
  transaction record (`:1411-1459`). It is not evidence for arbitrary closure rollback or async
  editing. Zircon's fallible Rust commands need a stronger prepared-commit contract.

### Fyrox corroborating reference

Fyrox defines execute/revert/finalize command ownership (`editor/src/command/mod.rs:83-112`), executes
groups forward and reverts them in reverse (`:177-210`), and finalizes truncated/evicted commands
with a bounded entry stack (`:232-280`). This corroborates deterministic order and cleanup. Its
command callbacks return no failure result, the capacity is entry-based, and group execution is a
simple per-command loop, so it is not evidence that Zircon's partial-failure, byte-budget or world
lease problems are solved by copying that stack.

## Required hard-cut architecture

The target chain is:

`AuthoringWorldGeneration -> PreparedEditBatch -> AuthoringCommitGeneration -> HistoryGeneration -> InspectionRenderDelta`

1. Under the shell lock, capture only a typed intent plus expected project/document/authoring
   generation. Submit it and release the lock. No world callback, plugin query, condvar wait, JSON,
   history replay or broad invalidation is permitted there.
2. Prepare against one immutable authoring generation. Resolve targets, validate hierarchy/schema/
   permissions, normalize only the changed field and construct field-specific forward/inverse
   deltas outside the short commit authority.
3. Commit one batch with one authoring owner lease. Revalidate expected generation, apply all deltas
   or none, and publish one new generation with exact hierarchy/transform/reflection/selection/render
   changed ranges. A failed/stale batch leaves the previous generation byte-identical.
4. Store one compact batch record and inverse handle per transaction. Transform drag retains
   before/current transform only; rename alone owns a name; large subtree deletion uses an immutable
   tombstone/artifact owner governed by entries, bytes, resident resources and age.
5. Undo/redo apply one inverse/forward batch through the same generation commit path. No per-command
   world lock, partial effect guess or second editor scene truth remains.
6. Replace arbitrary exclusive-transition closures with move-owned prepared replacement plus a
   generation-checked swap/receipt. Reuse PERF-MVP-640's prepared scene/project generation rather
   than create a second staging or rollback authority.
7. Scope completion returns immediate outcome or an explicit bounded ticket. `Drop` only releases
   local reservation state and never waits, calls plugins, enters the world or performs I/O.
8. Build inspection/render/history facts from the immutable committed generation after owner locks
   are released. Stable frames reuse shared handles and publish zero deltas; UI never reconstructs
   the scene by holding shell and world locks together.

Old callback mutation, partial-effect inference, no-deadline wait/Drop, selection-only transition
rollback, per-command world replay and count-only history admission are deleted in the same cutover.
No compatibility facade, fallback or dual write survives.

## Measurement and acceptance gates

### A1 - atomic fault matrix

Inject failure before/after every parent, name, transform, reflected-field and N-command batch stage
for batch sizes `1/128/10K`. Record generation, world digest, selection, history cursor, dirty state,
inspection/render delta and cleanup receipt. Any rejected/faulted/stale batch must leave the complete
pre-commit digest unchanged. Partial-effect inference count must be zero.

### A2 - lock, wait and main-thread budget

For one edit, 128-node edit and 100K-node hierarchy operation, record shell lock hold/wait, authoring
lease count/hold, transaction state lock, condvar waits, task queue wait, CSwitch and UI dispatch
p50/p95/p99. Require shell-held slow work 0, authoring mutation leases 1/batch, stable-frame authoring
leases 0, no-deadline waits 0 and Drop waits 0.

### A3 - payload and retention complexity

Measure name/reflected/subtree bytes `16 B/4 KiB/1 MiB/256 MiB`, batch sizes `1/1K/100K`, history
entries `1/128` and drag updates `1/1K/100K`. Record allocations, stable-name clone bytes, boxed
commands, tombstone bytes, eviction visits, RSS and cleanup wall time. Transform updates require
stable-name clone bytes 0 and one final record; history must enforce both entry and byte/resident
budgets with O(evicted records) cleanup outside the owner lock.

### A4 - generation and delta behavior

Across 10K stable frames and one changed frame, record world/scene reads, hierarchy visits, plugin
schema calls, projection allocations, changed rows and render extract work. Stable frames require
world locks/visits/plugin reflection calls/delta rows 0. One edit publishes one generation and only
the affected hierarchy/inspection/render ranges.

### A5 - history, replay and journal

Undo/redo batches `1/128/10K` must use one authoring commit each, preserve deterministic reverse/
forward order, and pass the A1 fault matrix. Journal page sizes `1/128`, payloads through 256 MiB:
engine lock excludes serialization, byte/deadline/remaining counters are explicit, and normal edit/
stable-frame journal serialization count is 0.

### A6 - managed Windows product profiling

After the approved-root editor build is repaired, use D/E/F artifact roots only. Capture at least 31
comparable cold/warm F4 samples with WPR/xperf CPU sampling, waits/locks, File I/O, CSwitch, thread
time, working set/RSS and power/energy. Report before/after median, p95, confidence interval, effect
size and environment. Compare phase shape and scaling against the referenced engine contracts; do not
claim equal energy or latency without comparable workloads and hardware.

### A7 - rendering correlation and regression

RenderDoc is relevant only after an authoring commit reaches the first rendered frame. Record CPU
commit generation, render-extract generation, submission/frame ids, pass/draw counts and GPU time;
prove one edit causes no duplicate extract/submission. RenderDoc is not acceptance evidence for
transaction locks, history memory or filesystem work. Current managed Cargo, unit/integration/fault
tests, F0/F4 product flows, rustfmt and scoped diff checks must all pass at the accepted fingerprint.

## Current validation receipt

- Static manifest/read: GREEN, 29/29 files, 4,742 lines, 8 inline tests.
- Production lock/caller trace: GREEN for current source; event dispatch holds shell across sync edit.
- Reference source: GREEN for the quoted Unreal/Fyrox paths and bounded claims above.
- Formatting: GREEN, independent `rustfmt --check --edition 2021` on 29/29 current Rust files.
- Documentation metadata: GREEN for this review and routing record, 35/35 referenced paths exist and
  `own_violations=0`. The repository-wide docs gate remains RED at 652 violations across 2,492
  documents; those foreign current-source path drifts are not rewritten by this module.
- Plan control: GREEN, plan audit and session heartbeat completed successfully.
- Rust implementation: unchanged because scoped files contain concurrent foreign work and the fix is
  a cross-owner hard cut.
- Managed Cargo/product/WPR/xperf/RenderDoc/energy: RED/pending behind
  `failure-2026-08-15-build-editor-approved-root-separator.md`.
- Index status: protected-owner routing pending; this module remains out of `review.md`.
