---
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-16
---

# Editor authoring transaction protected-plan routing evidence (2026-08-16)

## Coordinator decision

The current-source review is complete in
`2026-08-16-editor-authoring-transaction-current-architecture-review.md`: 29/29 Rust files, 4,742
physical lines, 8 inline tests and manifest
`fbb8b6e6b5f558641de1b878abfe648605739c706d3da9a955c147671e329cd1`.

Performance01 owns `docs/plans/performance/**`, but the main plan, `pending.md`, `review.md` and
numbered editor/runtime plans contain concurrent foreign changes. This file requests exact owner
merges and does not overwrite them. Non-validation module review continues while the managed editor
build entry is repaired.

## Required owner merges

### Performance main plan

Add proposed `PERF-MVP-641` after PERF-MVP-640 and link it to PERF-MVP-063/549/600/632/640, Plan02
M1/M2/M4, Editor03/05/14, EditorUI08 and Runtime07/11:

| ID | Priority | Current root cause | Required hard cut | Acceptance summary |
|---|---|---|---|---|
| PERF-MVP-641 | P0 | authoring edit commands can perform multiple fallible mutations then report every failure `Unchanged`, so the transaction layer can skip rollback of a partially applied command; retained event dispatch holds the workbench shell lock across synchronous transaction/world work; capture and every boxed command repeatedly enter the single authoring world; scope commit/cancel/Drop wait forever on one global operation condvar; exclusive transition failure restores selection only; history is entry-bounded but not byte/resident-bounded | Hard-cut one chain `AuthoringWorldGeneration -> PreparedEditBatch -> AuthoringCommitGeneration -> HistoryGeneration -> InspectionRenderDelta`. Capture typed intent under shell and prepare field-specific forward/inverse deltas outside locks; perform one generation-checked all-or-none authoring batch and publish exact deltas; replay one batch; replace arbitrary transition closure with move-owned prepared swap; return immediate busy/ticket outcomes and prohibit waits in Drop; retain history by entries+bytes+resident resources+age; project inspection/render from immutable committed generations. Delete partial-effect inference, callback mutation, per-command world replay, no-deadline waits and selection-only rollback | frozen 29/29 manifest remains pending. Fault at every stage leaves identical world/selection/history/render digest; shell-held slow work 0, authoring mutation leases 1/batch, stable-frame leases 0, no-deadline/Drop waits 0; transform stable-name clone bytes 0 and one final record; history obeys bytes/resident cap; journal serialization outside state lock; current Cargo/fault/F0/F4 plus 31-sample WPR/xperf CPU/wait/CSwitch/RSS/power and first-frame RenderDoc correlation pass |

Do not implement PERF-MVP-641 as an effect-enum-only patch, more mutexes, a worker synchronously
awaited under shell lock, a second authoring cache, per-document compatibility path, or entry-count
only history cap. Atomicity, owner generation, inverse-delta lifetime and deletion land before
parallel execution.

### Performance pending/review indexes

Replace the stale aggregate clause with one concise module record; do not add 29 per-file rows:

- `core/editing/**`: current 29/29 statically reviewed, 4,742 physical lines, 8 inline tests,
  manifest `fbb8b6e6b5f558641de1b878abfe648605739c706d3da9a955c147671e329cd1`;
  PERF-MVP-641 owns atomic batch/generation/nonblocking lifecycle, while PERF-MVP-063 retains
  field-specific transform payload and PERF-MVP-600/632 retain authoring projection/extract work;
  managed Cargo, fault matrix, scale counters and F4 product profiling remain pending.

Keep the module out of `review.md` until A1-A7 in the review are GREEN at the accepted fingerprint.
Move the single module record atomically with measured before/after evidence.

### Plan02 hard-cut milestones

- M1 supplies typed bounded tickets, cancellation/deadline receipts and nonblocking owner admission.
- M2 supplies immutable authoring generation, move-owned prepared batch/replacement and exact changed
  ranges shared by editor inspection and renderer extraction.
- M4 cuts the composition graph so shell intent, preparation, authoring commit, history commit and
  retained/render publication are explicit phases. There is no callback-shaped world mutation edge.

### Editor03 command, transaction and undo owner

Own `PreparedEditBatch`, field-specific forward/inverse deltas, exact commit effects, batch history and
fault semantics. A batch is completely validated before owner commit; any failure/stale generation
leaves world/context/selection/history byte-identical. Undo/redo use the same one-batch commit path.
Journal projection receives an immutable record handle and serializes outside the engine lock with
page/byte/deadline limits. History admission governs entries, bytes, resident resources and age.

Replace scope busy loops with immediate typed outcomes or explicit bounded completion tickets.
`Drop` performs no wait, callback, world entry, I/O or serialization. `set_merge_mode` and participant
registration return observable results rather than silently losing changes while busy/faulted.

### Editor05 scene editing and authoring owner

Own `AuthoringWorldGeneration` and one short main-affinity `AuthoringCommitGeneration` swap. Prepare
rename/parent/transform/reflection/subtree changes against an immutable generation, acquire one world
lease per batch, revalidate generation and publish exact hierarchy/transform/reflection/selection
deltas. Transform drag owns before/current transform only; large delete retention is an immutable
budgeted tombstone, not repeated scene copies. Reuse PERF-MVP-640's prepared scene replacement.

### Editor14 and Runtime11 scheduling owners

Provide bounded edit/scene preparation tickets keyed by project/document/authoring generation.
Record queue wait, prepare CPU, bytes, cancellation, deadline and stale completion. Only the short
authoring commit is main-affine. The UI does not block waiting for preparation or an active
transaction, and shutdown has explicit bounded terminal receipts.

### EditorUI08 retained shell owner

Event dispatch captures a compact typed intent and expected generation under the shell lock, submits
it and returns. Completion facts update retained state after owner locks are released. Snapshot,
inspection and render consumers read immutable generation handles and exact deltas; no shell guard
crosses transaction wait, plugin callback, hierarchy traversal, world callback or render extract.

### Runtime07 engine loop owner

Reserve explicit main-affinity authoring commit and retained publication phases with budgets and
telemetry. High-frequency transform updates coalesce by typed owner/generation before commit; input
edges and transaction ordering remain lossless. Stable frames perform no authoring projection work.

## Dependency and deletion order

1. Freeze current world/context/selection/history semantics and full failure digest tests.
2. Introduce immutable authoring generation, typed field deltas, batch receipt and byte accounting.
3. Prepare existing commands into batches while the current commit path remains the only authority.
4. Cut authoring apply/undo/redo to one generation-checked batch commit and exact delta publication.
5. Cut retained event dispatch to intent/ticket and inspection/render to immutable generations.
6. Cut project transition to move-owned prepared replacement from PERF-MVP-640.
7. Delete callback mutation, full `NodeEditState`, partial-effect inference, per-command world replay,
   global no-deadline completion waits, wait-in-Drop and selection-only transition rollback. No alias,
   fallback, dual write or compatibility shim remains.

## Required verification return

The implementing owners must return:

- exact current manifest plus deleted legacy symbols/paths;
- before/after atomic fault digests and stale-generation receipts;
- deterministic lock/lease/wait/allocation/clone/history-byte/journal counters at required scales;
- current managed Windows Cargo and F0/F4 product evidence with artifacts on D/E/F only;
- at least 31 comparable WPR/xperf samples for CPU, locks/waits, CSwitch, RSS and power/energy;
- RenderDoc only for authoring-generation to first-rendered-frame correlation;
- independent review, scoped milestone commit and quantified WeCom after every gate passes.

Until these returns exist, the module remains `static_complete / dynamic_pending`; no performance or
power claim and no `review.md` acceptance is authorized.
