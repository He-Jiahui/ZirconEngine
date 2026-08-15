# F4 Inspection Incremental Publication Audit

Status: static implementation candidate; managed measurement pending; no measured performance conclusion  
Created: 2026-08-10  
Scope: MVP F4 scene inspection from runtime world artifact to retained editor hierarchy  
Artifact root: `E:\\ZirconBuilds\\mvp-perf\\f4-inspection-<source-fingerprint>-<run>`

## Decision

The Layout09 failure handoff and independent source reviews established correctness
and asymptotic defects in the pre-repair protocol, so the current tree contains a
static forward-repair candidate. That candidate is not an accepted performance
result. A fresh, managed Windows build and repeatable profile data are still
required before claiming the bottleneck is removed or comparing Zircon with another
engine. Existing project-relative path resolution remains the only path policy. The
profiling artifact root above is a physical Windows path, not a new virtual path or
URI scheme.

## Ownership Boundary

- `zircon_runtime::scene` owns the world-derived inspection artifact and its
  generation semantics.
- `zircon_editor::scene` owns editor selection and retained hierarchy state.
- `zircon_app` hosts the process and must not become a second owner of world or
  selection state.

This matches the runtime/editor boundary plan. Unreal Scene Outliner is the primary
reference for explicit full refresh plus budgeted pending operations; Fyrox
`editor::world::selection` is the secondary reference for keeping authoring
selection outside the runtime world model.

## Sources Inspected

Current source:

- `zircon_runtime/src/scene/inspection/artifact/cache.rs`
- `zircon_runtime/src/scene/inspection/artifact/data.rs`
- `zircon_runtime/src/scene/inspection/artifact/overrides.rs`
- `zircon_editor/src/ui/workbench/snapshot/data/scene_entry/fragment.rs`
- `zircon_editor/src/ui/host/scene_inspection_publication.rs`
- `zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs`
- `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_fragment.rs`
- `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs`
- `zircon_editor/src/tests/editing/history.rs`
- `zircon_editor/src/tests/editor_message/refresh.rs`

Reference source:

- `dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SSceneOutliner.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/ActorBrowsingMode.cpp`
- `dev/Fyrox/editor/src/world/selection.rs`

## Pre-repair Static Baseline

The following were source facts in the pre-repair snapshot. They remain the
measurement baseline, not claims about the current candidate's runtime impact:

1. A name-only world change reaches `SceneEntries::from_artifact` for the retained-host
   fragment. That calls `WorldInspectionArtifact::hierarchy_rows_arc`; for a sparse
   generation this reaches `HierarchyRows::as_arc` and materializes a complete row array
   before the editor applies its small delta. This is a static call-path fact, not evidence
   that the allocation dominates an interaction.
2. Repeated name changes can copy the override map in
   `HierarchyRows::from_name_changes`.
3. Retained-host fragment processing scans hierarchy entries for anchors and
   selection changes, which may make small deltas scale with total hierarchy size.

The forward repair addresses all three source paths, but current tests and static
review do not establish the product cost of long edit sequences or retained-host
delta handling at large scale. The measurement procedure below remains mandatory.

The runtime cache records complete-view materialization count and rows. Those
totals are cache-local, including when a `World` is cloned, so an independent
world's diagnostics cannot inflate the source world's report. Cache cloning reads
the materialization totals and the current hierarchy `OnceLock` state under one
metrics read-lock snapshot, so a concurrently completed materialization cannot be
copied with stale totals or counted twice. Focused regressions cover cloning both
before and after row materialization; the final static lock-order review reported
`Critical=0 / Important=0`. The profile capture
must correlate those counters with retained-host update counts before choosing a
representation change.

## Pre-repair Protocol Review

Before the forward repair, the producer and consumer did not form an end-to-end
incremental protocol. This is the historical source path used to define the
measurement comparison:

```text
WorldInspectionArtifact sparse generation
  -> WorldInspectionDelta has changed/added row payloads
  -> SceneInspectionMessage reduces rows to identity/parent/depth/hash anchors
  -> SceneEntries::from_artifact calls hierarchy_rows_arc
  -> sparse HierarchyRows materializes a complete Arc<[Row]>
  -> retained tick and bridge inspect that complete sequence
```

`WorldInspectionArtifact::hierarchy_row(entity)` could already read one row by
stable entity identity without materializing the full sequence. The pre-repair
message and bridge did not preserve enough data or indexes to use that path for
every operation: changed rows lost display payload at the fragment boundary,
entry/control lookup was linear, and selection reconciliation scanned the whole
sequence. Structural changes deliberately requested reflow, which updated the
affected suffix.

Consequently, the pre-repair asymptotic candidates were:

| Path | Pre-repair source behavior | Candidate scale |
| --- | --- | --- |
| Name-only delta publication | Copies accumulated sparse overrides, then recomputes dirty ancestry | proportional to retained overrides plus dirty ancestry |
| Fragment construction | Materializes all rows whenever sparse overrides exist | O(N) rows and allocation |
| Anchor validation/lookup | Scans entries or controls per anchor | O(N * changed anchors) |
| Selection reconciliation | Scans all retained rows | O(N) per fragment |
| Structural reflow | Rewrites from first affected row to the end | O(N) suffix work |

The table defines the before-side instrumentation and review targets. It is not
post-change measurement evidence.

Unreal's primary contrast is architectural, not a license to copy its containers:
`ActorHierarchy` emits typed Added/Removed/Moved or explicit FullRefresh events;
`SSceneOutliner::Populate` retains a keyed tree-item map, drains pending operations,
and checks its frame budget every 100 operations. Pre-repair Zircon had explicit
generation gaps and full resync, but materialized the complete view before applying
the fragment. The current candidate preserves the separate full-resync path while
giving the non-structural path exact changed rows and retained indexes.

## Current Static Implementation Candidate

The current tree implements the following representation and protocol. These are
source-complexity claims verified by static review and regression contracts, not
measured wall-time or power conclusions.

| Path | Current candidate | Static scale target |
| --- | --- | --- |
| Name-only publication | Base rows plus a persistent Patricia map of row replacements; ordered child-hash contributions are maintained as persistent parent aggregates. | O(A * `usize::BITS`) path-copy work for A affected ancestry anchors; independent of sibling count and historical rename count. |
| Fragment construction | Exact changed rows are resolved by entity identity; a complete row Arc is requested only for explicit Reflow/resync. | O(delta) row payload and lookup work. |
| Patch validation/application | Entity-indexed `HashMap`/`HashSet` validation and retained control indexes. | O(delta) expected work for non-structural patches. |
| Selection reconciliation | Revisioned added/removed identities compose across Latest replacement; a full selection snapshot is requested only on revision mismatch. | O(K log K) Latest composition and expected O(K) bridge application for K changed identities; mismatch repair remains authoritative. |
| Structural change | Explicit Reflow with generation checks and last-known-good pointer routing. | O(N) by design and separately counted. |

`WorldInspectionArtifactDiagnostics::hierarchy_child_hash_updates` counts incremental
parent-aggregate changes. The 5,000-sibling regression requires one child-hash
update, two changed rows, and zero complete-row materializations for one child
rename. The 100,000-row and 10,000-selection regressions remain semantic/scale
contracts; only the managed measurement procedure below may establish p50/p95,
allocation, working-set, or energy improvements.

## Measurement Procedure

Use a clean, source-bound Windows executable. Do not reuse a binary built from an
unknown checkout. Record the source revision, executable hash, machine model,
Windows build, display configuration, power mode, GPU driver, and exact command in
the result directory.

Run each workload after five warm-up iterations and capture at least thirty measured
iterations under the same machine and display configuration:

| Workload | Scale | Operation |
| --- | ---: | --- |
| Initial publication | 1,000 and 100,000 rows | Open the hierarchy and wait for the first retained snapshot. |
| Idle host processing | 1,000 and 100,000 rows | Run 300 host ticks without world or selection changes. |
| Selection delta | 1,000 and 100,000 rows | Switch one selected entity, then apply a multi-selection update. |
| Name sequence | 1,000 and 100,000 rows | Apply 1,000 distinct name-only updates while the hierarchy is visible. |
| Structural delta | 1,000 and 100,000 rows | Apply bounded add, move, and remove operations, then verify a full resync path. |

Use the existing `tools/ui-profile-capture.ps1` capture route with an explicit
`-OutputRoot E:\\ZirconBuilds\\mvp-perf\\...`. Enable its tracing switches only
when the managed executable and required Windows tooling are available. Preserve
the raw report, screenshots, trace files, command transcript, and a compact summary
in that same physical artifact root.

Collect:

- p50, p95, and maximum wall time for each workload and interaction;
- runtime profile span/counter output for artifact build and publication paths;
- editor `scene_inspection` spans (`hierarchy_fragment_projection`,
  `hierarchy_fragment_apply`, and `hierarchy_full_resync`) plus the fragment entries, added/changed/removed rows,
  updated rows, reflow count, resync-required count, and full-resync row counters;
- materialization counters paired with each fragment's message kind, changed-anchor count,
  selection-change count, and whether `SceneEntries::from_artifact` received sparse rows;
- CPU sample attribution and allocation/working-set evidence from WPR/WPA when
  available;
- count of full hierarchy materializations, full resyncs, and delta messages;
- retained hierarchy size, changed-row count, and selected-row count per run;
- screenshot evidence that the hierarchy stays coherent after the sequence.

Power usage must be reported only when a repeatable machine telemetry or external
meter is available. WPR CPU samples by themselves are not power measurements; if
telemetry is unavailable, record power as unavailable rather than inferring it.

## Conditional Implementation Direction

Choose one change only after the dominant measured cost is known:

- If row-array materialization dominates, keep the runtime's stable base rows and
  publish a bounded row delta without forcing editor-side full-array materialization.
- If accumulated name overrides dominate, replace the measured copy path with a
  generation-scoped representation that preserves runtime artifact identity and
  snapshot semantics.
- If retained-host scans dominate, let editor-owned retained state maintain the
  entity-to-row index needed to apply an incremental fragment directly.

Before selecting any of those branches, compare the following protocol options
against the capture data and preserve a full-resync fallback:

- For non-structural row edits, retain editor-owned entity-to-control state and
  read only changed rows by runtime entity identity, or carry only the bounded
  changed-row payload already produced by `WorldInspectionDelta`.
- For selection, publish or retain the bounded changed selection identities at
  the editor boundary instead of rediscovering them through every visible row.
- For add/remove/move, either extend the delta with deterministic ordering
  metadata and apply queued operations, or classify the change as an explicit
  full resync. A flat `Vec` plus index map still has suffix reindex cost, so it
  is not automatically an asymptotic improvement for large structural edits.
- If structural operations are measured as the limiting path, use an explicit
  bounded pending-operation budget analogous to the Unreal outliner rather than
  hiding a long full reflow inside one retained-host tick.

Any selected change must preserve explicit full refresh for resync, filtering, and
structural invalidation. It must not add a virtual path layer, move editor selection
into runtime, or give the application host ownership of scene state.

## Post-change Checks

Run focused semantic tests for empty, single-row, 1,000-row, and 100,000-row
hierarchies; single and multi-selection; name-only updates; structural changes; and
forced full resync. Repeat the exact pre-change profile procedure on the same
hardware, then compare p50/p95, CPU attribution, allocations, memory, hierarchy
coherence screenshots, and power data when available.

The implementation is accepted only when the comparison identifies the measured
dominant cost, preserves the runtime/editor ownership boundary, and records the
post-change product evidence alongside the raw trace artifacts.
