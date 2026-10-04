---
status: review_complete_measurement_required
created_at: 2026-08-15
owner_boundary:
  runtime_generation: Runtime07 and Runtime08
  editor_projection: Editor05
  current_session: read_only_for_runtime_and_editor_sources
related_code:
  - zircon_runtime/src/scene/inspection/artifact/cache.rs
  - zircon_runtime/src/scene/inspection/artifact/data.rs
  - zircon_runtime/src/scene/inspection/artifact/fields.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/workbench/snapshot/data/scene_entry/projection_cache.rs
  - zircon_editor/src/ui/workbench/snapshot/data/scene_entry/entries.rs
references:
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner
  - dev/Fyrox/editor/src/scene/selector.rs
  - dev/Fyrox/editor/src/plugins/inspector/mod.rs
---

# F4 World Inspection Generation And Editor Projection Review

## Decision

Do not optimize this path by adding another snapshot cache or by moving editor
selection into Runtime. The required architecture is a single runtime-owned,
generation-scoped inspection source plus an editor-owned, stateful outliner
projection. Runtime publishes stable entity identities and bounded hierarchy or
field deltas. Editor owns selection, expansion, filtering, visible tree nodes,
and UI refresh scheduling.

No implementation is approved before the measurement matrix below produces a
source-bound Windows baseline. This record is a structural decision and a
measurement plan, not a performance claim.

## Current Source Facts

The current inspection artifact implementation is an untracked shared source
candidate in this checkout. It must not be overwritten by this session. No
Cargo, Pester, product, screenshot, or profiler run was performed while the
current validation window is held.

### Runtime Artifact Path

`World::inspection_artifact()` reuses an `Arc<WorldInspectionArtifact>` at an
unchanged world generation. A generation with no hierarchy dirty facts creates a
thin wrapper that reuses the preceding hierarchy indexes and rows. Focused
fields are cached only for the current primary entity, which bounds cache growth
by selection history.

The full hierarchy path remains expensive:

1. `World::node_records()` allocates owned `SceneNode` records for every stable
   entity and then sorts them by id.
2. `WorldInspectionArtifact::from_world()` builds hierarchy rows, an entity-to-
   row index, a parent-to-children map, child positions, and child-hash
   aggregates.
3. A previous generation causes `hierarchy_delta_between()` to scan current rows
   for additions and changes, then prior rows for removals.

Thus a topology fallback is not merely a UI refresh. It projects all nodes,
sorts, builds multiple indexes, and compares both complete generations. The
existing `HierarchyMutationIndex` already maintains parent-to-children stable
ordering for mutation and derived-state traversal, so recreating an unrelated
hierarchy fact set at publication is the wrong long-term direction.

Name-only changes use sparse `HierarchyRows` overrides and update ancestry hash
anchors. That producer path is bounded by changed names and ancestry, but any
consumer calling `hierarchy_rows()` or `hierarchy_rows_arc()` materializes a new
complete `Arc<[WorldInspectionHierarchyRow]>` on first read. Existing runtime
diagnostics correctly count that materialization, but they do not identify the
editor caller or its lock scope.

`WorldInspectionFieldsArtifact::delta_from()` constructs two `BTreeMap`s and
clones changed fields. This is bounded by one focused entity, which is suitable
for MVP, but it must remain on the selected-field path rather than become a
whole-world reflected-field cache.

### Editor Projection Path

`SceneInspectionPublication` correctly retains the prior runtime artifact and
uses a published adjacent-generation delta when available. It nevertheless
holds publication, shell, and world access while it obtains an artifact, builds
a selected-entity `BTreeSet` on selection revision change, and derives the
publication message. The hierarchy fragment and reflow paths likewise hold the
shell/world access while they may construct a complete hierarchy view.

`SceneEntryProjectionCache` is currently a zero-sized marker. Its `project()`
method always calls `SceneEntries::from_artifact()`. That allocates a new
selection `BTreeSet` for each snapshot and asks the artifact for a complete row
`Arc`; a sparse runtime name generation therefore loses its benefit as soon as
this projection is requested. The viewport edit projection has the same full
row request.

The issue is not that an outliner may ever need a full resync. Filtering,
missed generations, startup, and major structural replacement legitimately do.
The issue is that ordinary delta publication and selection overlay updates do
not currently have a retained projection that can avoid it.

## Reference Engine Alignment

Unreal's `ISceneOutlinerHierarchy` exposes hierarchy construction and a changed
event, while `SSceneOutliner` keeps the tree item map, selection restoration,
and refresh policy in the Editor. Normal Added, Moved, and Removed events are
queued as pending operations. `Populate()` consumes those operations with a
per-frame time budget; `FullRefresh()` is an explicit slow path for filter,
undo, and true resync cases. Zircon should adopt this ownership and lifecycle
shape, not Unreal's concrete Slate objects.

Fyrox provides a secondary Rust/editor boundary check. Its `NodeSelector`
retains selection in the editor control and synchronizes the tree when that
selection changes; the Inspector rebuilds its context from the editor scene
selection. Zircon's `SelectionModel` and `SceneModeStack` should retain that
authority. Fyrox's recursive UI traversal is not a Zircon performance model
for 100,000 entities.

## Target Algorithm

### Runtime: Neutral Inspection Generation

Keep Runtime authoritative for scene facts only. A future
`WorldInspectionGeneration` publication must contain:

- a generation id and immutable full-resync hierarchy representation;
- stable entity id, parent id, depth, label revision, activity, and subtree
  identity for hierarchy rows;
- a bounded ordered delta derived from mutation facts: added, removed, moved,
  relabeled, and field-invalidated entities;
- on-demand `WorldInspectionFieldsArtifact` for the current primary entity;
- explicit resync metadata only when incremental ordering cannot be proven.

The hierarchy mutation transaction and `HierarchyMutationIndex` are the input
to this delta. Do not infer ordinary add, remove, move, or rename facts by
comparing two complete row arrays. Full rebuild remains an explicit recovery
path for restore, externally invalidated index state, or a generation gap.

For a stable world generation, publication must return the same `Arc` and do no
node projection, row scan, selection work, or allocation. For a transform-only
change, hierarchy rows must remain shared and only the selected entity's field
artifact may rebuild. For a rename, work is bounded by the changed row and its
ancestry; it must not require a contiguous hierarchy materialization merely to
serve a delta consumer.

### Editor: Retained Outliner Projection

Replace the marker `SceneEntryProjectionCache` with an editor-owned
`SceneOutlinerProjection` state machine. It holds the last accepted runtime
generation, entity-to-retained-node map, expansion/filter state, and an
editor-owned selection overlay. It consumes contiguous deltas in order and
applies bounded Add, Remove, Move, Relabel, and selected-state operations.

The full `SceneEntries` allocation is permitted only for startup, explicit
filter reflow, missed generation recovery, or a declared full resync. Selection
changes must update only the editor overlay; they must not request runtime
hierarchy rows. A retained UI may schedule a large full resync over bounded
frame work, but it must record that scheduling as a full-resync cost.

Publication must use two short lock phases:

1. capture immutable runtime artifact handle, selection revision, and required
   editor projection identity;
2. release shell/world locks, compute or apply bounded projection work, then
   reacquire briefly to publish only if generation and selection revision still
   match.

This prevents a full hierarchy materialization, sort, or UI tree operation from
extending the shell/world critical section. It does not move Runtime world
authority or SelectionModel authority across the existing boundary.

### Complexity Contract

| Operation | Required work after convergence |
|---|---|
| Stable world and stable selection | O(1), zero full row scan/build/materialization |
| Focused field change | O(F) for that entity's reflected fields only |
| Selection add/remove | O(log S) or bounded overlay work, no hierarchy read |
| Rename | O(H + R), where H is ancestor height and R is changed retained rows |
| Add/remove/reparent | O(A + H) plus explicit retained tree operations for affected subtree A |
| Filter change or generation gap | Explicit full resync, budgeted and counted |

`N` is total hierarchy rows. A full resync may cost O(N), but it must not be the
implicit result of a normal rename, field mutation, or selection change.

## Measurement Before Implementation

All future artifacts must be written under an approved `D:`, `E:`, or `F:`
root such as `D:\ZirconBuilds\world-inspection-<run-id>`, never `C:`. Run only
after the current validation release permits Windows profiling.

- `Stable`: 1, 1k, and 100k rows at 60/120/240 frames. Record artifact
  reuse, node projections, row builds, full materializations, and editor
  projection allocations.
- `Field edit`: leaf, root, and unrelated entity. Record hierarchy reuse,
  focused field builds and bytes, field delta size, and lock hold/wait.
- `Rename`: leaf, mid-depth, and root in wide and 100k-deep trees. Record
  dirty anchors, hash updates, materialized rows and bytes, and UI patches.
- `Structural`: add, remove, and reparent with affected subtree sizes 1, 1k,
  and 50k. Record mutation facts, delta size, resync decision, and retained
  node operations.
- `Selection`: 1, 1k, and multi-select churn. Record overlay allocations and
  bytes, selection delta size, hierarchy reads, and materializations.
- `Resync/filter`: filter on/off, missed generation, and restore. Record full
  rows, frames to completion, per-frame budget, RSS, and UI responsiveness.

Runtime counters must include generation cache hit/miss, node projection count,
sort count, hierarchy index passes, direct mutation-fact count, full-diff scans,
row clone bytes, focused-field build bytes, and full-row materialization caller.
Editor counters must include projection cache hit/miss, delta operation counts,
selection overlay bytes, complete reflows, retained-node creates/removes, and
shell/world/publication lock wait and hold time. Existing materialization and
focused-field counters remain useful but are insufficient on their own.

Use WPR/WPA or equivalent Windows ETW CPU, allocation, context-switch, and
working-set evidence alongside in-engine counters. Record CPU p50/p95/p99,
RSS, allocation rate, lock hold/wait, and energy only as measured data. GPU
timestamps are not evidence for this CPU/editor path. Do not claim parity with
another engine or a power result before this matrix is captured on comparable
hardware and workload.

## Implementation Order And Gates

1. Runtime07/Runtime08 adds the missing counters and captures the baseline.
2. Runtime derives hierarchy deltas from mutation facts and preserves a complete
   snapshot only for explicit resync.
3. Editor05 implements the retained `SceneOutlinerProjection`, consuming only
   contiguous generation deltas and keeping selection/editor view state local.
4. Editor05 moves expensive projection and reflow work outside shell/world
   critical sections, with generation-checked publication.
5. Add 1/1k/100k structural counter tests before running the Windows profile
   matrix. Stable, field, rename, and selection paths must meet the complexity
   contract above before tuning allocation details.
6. Run the approved Windows measurements, compare before/after bottlenecks, and
   then perform the requested independent second review before acceptance.

The current session owns none of the Runtime/Editor implementation files in
this design. It must not add aliases, a parallel hierarchy cache, or a
test-only projection to bypass the owner boundaries.
