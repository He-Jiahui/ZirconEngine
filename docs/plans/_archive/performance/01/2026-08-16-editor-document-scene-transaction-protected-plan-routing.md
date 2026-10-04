---
related_code:
  - zircon_editor/src/core/document
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_scene_document_submission.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-16
---

# Editor document and scene transaction protected-plan routing evidence (2026-08-16)

## Coordinator decision

The current-source review is complete in
`2026-08-16-editor-document-scene-transaction-current-architecture-review.md`: 6/6 Rust files,
1,690 physical lines, 19 inline tests and manifest
`92f21be754fc1d14b8880ba4d14e03326f8ae09af3e0ac44a65f2ba1f8c7df40`.

Performance01 is authorized for `docs/plans/performance/**`, but the main performance plan,
`pending.md`, `review.md` and numbered editor/runtime plans contain foreign concurrent changes.
This record requests exact owner merges without overwriting them. It is not a terminal blocker:
non-validation module review continues while the managed product entry is repaired.

## Required owner merges

### Performance main plan

Add `PERF-MVP-640` as P0 after PERF-MVP-639 and link it to PERF-MVP-593/632/637/638, Plan02 M1/M2/M4,
Frameworks01, Editor01/03/05/09/10/14, EditorUI08 and Runtime04/11:

| ID | Priority | Current root cause | Required hard cut | Acceptance summary |
|---|---|---|---|---|
| PERF-MVP-640 | P0 | scene open/create holds the retained workbench shell lock and document route gate across file load/decode, staging publish/rollback, asset import/full editor catalog refresh, authoring-world construction and state replacement; rollback can call project-wide `reimport_all`; route results retain full scene documents; document ID insertion scans both capped maps and scene identity formats/clones owned keys | Hard-cut one chain `ProjectGeneration -> ScenePreparationTicket -> PreparedAuthoringSceneGeneration -> DocumentRegistryCommit -> RetainedSurfaceDelta`. Reuse Frameworks01 durable project generation and Runtime04 typed asset delta; Runtime11/Editor14 perform bounded keyed prepare outside locks; Editor03/05 perform one short generation-checked move commit; Editor01/10 own one canonical typed document registry with direct key/reverse ID indexes and explicit O(1) retention order; EditorUI08 only submits intent and consumes compact receipts. Delete route-wide slow gate, full-reimport rollback, complete Scene clones/results and duplicate staging/catalog authorities | frozen 6/6 manifest retained pending. ID probe/eviction/key allocation O(1); UI/main lock excludes slow work; same key single-flight; stale apply/full reimport/complete Scene clone bytes 0; durable fault/restart matrix passes; current Cargo plus 1/100/10K/100K complexity gates, F4 WPR/xperf, RSS/energy and relevant first-frame RenderDoc correlation pass |

Do not implement PERF-MVP-640 as a `HashMap`-only patch, a larger mutex, an editor-private worker,
another project generation or a background job that is synchronously awaited while the shell lock is
held. Owner, generation, lifetime and deletion contracts land before concurrency.

### Performance pending/review indexes

Update the existing aggregate `zircon_editor/src/core/**` row with one concise module clause:

- `core/document/**`: current 6/6 static reviewed, 1,690 physical lines, 19 inline tests, manifest
  `92f21be754fc1d14b8880ba4d14e03326f8ae09af3e0ac44a65f2ba1f8c7df40`; PERF-MVP-640 owns the
  scene-transaction hard cut and PERF-MVP-593 retains the registry scale regression; Cargo,
  complexity counters, fault injection and F4 product tracing remain pending.

Keep the module out of `review.md` until A1-A6 from the current review pass. Move it atomically only
with the accepted fingerprint and measured before/after evidence. Do not add six per-file rows.

### Plan02 hard-cut milestones

- M1 supplies one shared task ticket/admission/cancellation/deadline/completion contract and the
  Runtime durable project generation. No editor-private pool or second WAL is permitted.
- M2 supplies the authoring/runtime scene generation and move-owned prepared scene boundary; the
  editor must not clone a runtime `World` or build a second scene truth.
- M4 fixes the editor composition graph: UI intent, project preparation, authoring commit, document
  fact publication and retained invalidation are explicit phases with one generation lineage.

### Frameworks01 and Runtime04 project/asset owners

Extend the existing `PreparedFullProjectGeneration`/durable transaction surface or add a typed
targeted operation within the same owner for scene source creation. It must include source,
sidecar/registry/catalog effects, commit-point outcome and restart recovery. Runtime04 returns an
immutable exact add/remove/change delta. Failure compensation must never require Editor to call
`reimport_all` or rescan the project.

### Runtime11 and Editor14 scheduling owners

Provide a scene preparation job keyed by `{project_generation, scene_asset_identity}` with
single-flight, count/source-byte/decoded-byte/resident-byte/age/deadline budgets. Stages record queue
wait, file I/O, decode, asset preparation, authoring preparation, cancellation and stale completion.
Only explicitly main-affine finalization enters the editor commit lane. Shutdown and project close
have bounded terminal receipts; Drop performs no wait or filesystem work.

### Editor10 project/document identity owner

Own the canonical typed `DocumentKey` and project-generation scene request. Exact key lookup and
occupied/reverse ID lookup are direct; closed-document retention uses an explicit bounded order and
does not scan either 1,024-row map. Hash typed root/asset components without allocating a formatted
identity string. Keep stable identity/collision semantics and reject root escape, stale tickets and
source conflicts.

### Editor01 lifecycle owner

Replace `scene_route_gate` with a short commit authority that validates the project/session
generation at commit. It updates active document/scene identity and produces facts in one bounded
critical section. The gate must never cover filesystem, decode, catalog, plugin, callback or
authoring build work. Fact fanout occurs after the state lock is released.

### Editor03 and Editor05 authoring commit owners

Editor05 accepts a move-owned `PreparedAuthoringSceneGeneration`; Editor03's fallible exclusive
transaction defines world/context/selection/history atomicity. On failure or stale generation the
old authoring state remains intact. Successful commit consumes the prepared owner once and returns a
compact receipt, not `ProjectSceneDocument` or `Scene`.

### Editor09 catalog owner

Consume Runtime04's immutable scene asset delta and update only affected catalog/reference/folder/
details rows. The one-scene success/failure path performs zero full project refresh and zero full
catalog rebuild. Preview and shader artifacts remain in their existing generation DAG; scene commit
does not open a second refresh path.

### EditorUI08 retained shell owner

Scene picker submit captures a small request under the shell lock, enqueues it and returns. Progress,
success, failure and cancellation arrive as typed generation facts. Workbench state applies only the
short final receipt/invalidation after authority locks are released. No shell guard crosses I/O,
decode, import, world build, plugin callback, message fanout or wait.

## Dependency and deletion order

1. Freeze project generation, canonical scene/document identity, receipt and fault semantics.
2. Reuse the Frameworks01 durable targeted transaction and Runtime04 exact delta.
3. Add shared bounded preparation tickets and measurements without changing the current commit path.
4. Introduce move-owned prepared authoring generation and short Editor03/05 commit.
5. Cut UI submission to intent/ticket and document publication to compact receipts.
6. Cut registry identity/reverse index and explicit retention order with scale counters.
7. Delete `scene_route_gate` slow closure, `PreparedSceneCreation::finish` clone, full-scene route
   result, editor `reimport_all` rollback, synchronous shell-held route and superseded staging/catalog
   code. No alias, fallback, dual write or compatibility shim survives.

## Required verification return

The implementing owners must return:

- exact current manifests and deleted legacy symbols/paths;
- deterministic registry probe/eviction/allocation and scene clone-byte counters;
- delayed-stage concurrency and stale-generation results;
- durable read/decode/stage/commit/catalog/authoring/message/cleanup fault and restart matrix;
- managed Windows Cargo/build evidence with artifacts on D/E/F only;
- at least 31 comparable F4 WPR/xperf samples with CPU/File I/O/waits/locks/CSwitch/RSS/power;
- RenderDoc only for first-frame GPU correlation after a committed scene, not as evidence for the
  filesystem/lock bottleneck;
- independent review, scoped milestone commit and quantified WeCom only after all gates pass.

Until those returns exist, the module remains `static_complete / dynamic_pending`; no throughput,
latency, power or algorithm-optimality claim is authorized.
