# Pane projection generation-cache design

Date: 2026-08-25

Status: current-source architecture review and implementation plan. The production paths described
below contain external in-progress changes, so this slice does not edit them and does not claim a
validated product improvement.

One clean preparatory slice is implemented in
`ui/retained_host/ui/pane_data_conversion/performance_timeline.rs`: frame, span and hotspot visual
nodes are now materialized only for rows intersecting the current list clip. Its 10,000-row lower
regression bounds the projected node count below 100 for a 240 by 160 list. This removes one complete
visual-node traversal, but it neither avoids the earlier full logical-row DTO mapping nor supplies a
scroll generation, retained row identity or visible-start authority. It is therefore evidence for
the collection boundary below, not completion of this design.

## Decision

Pane rendering must no longer use `PaneBodyPresentation -> TOML attributes -> new projection -> new
UiSurface -> full layout -> new host model` as the ordinary publication path. The editor needs four
independent retained authorities:

1. a compiled document skeleton, keyed by an explicit document generation;
2. a pane instance, keyed by pane identity plus document generation;
3. generation-owned semantic/control/collection data, updated by deltas; and
4. a geometry generation, updated independently for content-size, DPI and viewport changes.

Large collections such as Hierarchy, Console, Performance Timeline, Module Plugins and Build Export
must not be serialized into root TOML attributes. They are item sources with stable row identities,
visible ranges and bounded row materialization.

## Current-source proof

The complete reconstruction chain is explicit:

- `ui/template_runtime/runtime/pane_payload_projection.rs:15` instantiates or projects a document,
  then injects pane attributes into the owned projection.
- `pane_payload_projection.rs:90` converts every payload variant into a `BTreeMap<String, Value>`.
  Hierarchy, timeline, plugin and export arms allocate a TOML table per row and clone their strings.
- `pane_payload_projection.rs:567` recursively searches the projection once per component patch;
  `P` patches over `N` nodes can therefore cost `O(P*N)` before surface mutation.
- `pane_payload_projection.rs:678` clones every string in each array payload.
- `ui/retained_host/ui/pane_data_conversion/template_runtime_projection.rs:54-84` calls
  `project_pane_body`, builds a new shared surface, computes layout, builds a host model and collects
  a new host-node vector for one pane projection.
- `hierarchy_projection.rs:55-80` then walks the original hierarchy payload again to build a second
  row model. The same logical hierarchy has therefore crossed owned payload, TOML and host-row
  representations in one publication.
- `runtime_host/dynamic_control_state.rs` builds control attribute maps by cloning node maps, scans
  surface nodes per control ID and clones action attributes while rebinding.

`PaneBodyPresentation` and `PanePayload` derive deep `PartialEq` but carry no authoritative payload
generation. Using equality as a cache key would merely replace reconstruction with an `O(S)` scan
over the semantic payload, where `S` is all pane content.

Console demonstrates a partial better path. `console_projection.rs:90-99` keys a cache by compiled
document pointer identity and exact content-size bits, and reuses or patches retained console slots.
However, non-V2 documents have no identity from `retained_document_identity`, and exact width/height
in the semantic key still forces a miss during continuous resize. The reusable idea is retained
generation plus row deltas, not the current key shape.

The source inventory at
`E:\zircon-profiles\ui-structural-audit-20260825-133934\ui-structural-hotspots.json` is bound to
HEAD `1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8` plus 1,216 dirty UI paths.
`pane_payload_projection.rs` is its highest-scoring production file at 616 heuristic points with 71
syntactic clone calls. This is source-priority evidence, not CPU or allocation measurement.

## Unreal reference contract

The checked-in Unreal Slate source separates item data, generated rows and layout refresh:

- `SListView.h:244-250` describes an item source whose row widget is generated only when needed.
- `SListView.h:976-1070` maintains bidirectional item/widget maps in `FWidgetGenerator` across a
  generation pass.
- `SListView.h:1524-1660` starts at the visible scroll index and generates until the viewport is
  filled rather than projecting the complete item source.
- `SListView.h:1668-1685` reuses the existing widget for an item and calls `OnRefreshRow`; it creates
  a new widget only when no retained widget exists.
- `STableViewBase.h:160-166` distinguishes list refresh, which generates/releases only required
  deltas, from explicit full `RebuildList`.
- `STableViewBase.h:384-413` separates layout refresh from item regeneration.

Zircon should transfer those ownership boundaries, not copy the C++ containers. A pane document is
the widget skeleton, a collection is an item source, a materialized row is a retained item/widget
association, and content-size changes are layout refreshes rather than semantic reconstruction.

## Inspector property-row finding

The current Inspector path is not virtualized despite its `virtual_rows` name:

- `workbench_inspector_panel.zui:303-309` gives `WorkbenchInspectorMesh` a fixed 96-pixel height,
  authors four physical property rows and declares one clone prototype.
- `component_property_rows.rs:35-44` reconciles physical row capacity to the complete
  `component.properties.len()`.
- `virtual_rows.rs:114-249` prunes and ensures that complete capacity by scanning the surface,
  cloning the prototype and adding one tree node and slot per overflow property.
- `data_sync.rs:138-180` then enumerates every physical control and publishes visibility, label,
  value text and five field metadata values per row. There is no visible-start, visible-count or
  scroll-window authority in this chain.

This is both a CPU/allocation problem and an interaction contract gap: a fixed-height section owns
an unbounded physical child set without a retained viewport model. Consolidating its repeated tree
scans would reduce a constant factor but would preserve the wrong `O(P)` topology and property-write
cost for `P` plugin fields, so it is not the target repair.

Unreal Details uses `SDetailTree = STreeView<TSharedRef<FDetailTreeNode>>` in
`PropertyEditor/Private/SDetailsViewBase.h:47`. `SDetailsView.cpp:522-533` binds a logical root item
source plus `OnGenerateRow` and `OnRowReleased`; the shared Slate list implementation then retains
only generated viewport rows. Zircon's target is the same ownership split: stable field IDs and
logical section expansion state remain semantic, while a bounded physical row pool is rebound to
`visible_start..visible_end + overscan` and released/reused as that window changes.

The migration must be test-first and atomic. First introduce an Inspector collection generation and
field-ID index without changing rendering. Next publish a scroll/clip-derived window and bind the
existing four authored rows plus bounded overscan to logical indices. Only then remove overflow
prototype cloning and full-row synchronization. A value edit patches one field generation and one
materialized row; selection or component replacement may replace the logical source but must not
grow physical nodes with total property count.

## Required retained authorities

```rust
struct PaneDocumentGeneration {
    document_id: Arc<str>,
    generation: u64,
    compiled: Arc<UiV2CompiledDocument>,
    projection_skeleton: Arc<RetainedUiProjection>,
    control_index: Arc<PaneControlIndex>,
}

struct PaneInstanceGeneration {
    pane_id: Arc<str>,
    document_generation: u64,
    structure_generation: u64,
    surface: UiSurface,
    host_nodes: ModelRc<TemplatePaneNodeData>,
    action_epoch: u64,
    geometry_generation: u64,
}

struct PanePayloadGeneration {
    semantic_generation: u64,
    control_state_generation: u64,
    collections: BTreeMap<PaneCollectionId, PaneCollectionGeneration>,
}

struct PaneCollectionGeneration {
    generation: u64,
    logical_count: usize,
    delta: PaneCollectionDelta,
    visible_range: Range<usize>,
    rows: ModelRc<PaneRowData>,
}
```

The public key must use monotonic generations, not pointer addresses or a deep hash/equality scan.
Both legacy and V2 documents need a document generation. Theme, font/text-metrics and compiled
resource generations belong in the document/skeleton dependency set. DPI and content size belong
to geometry/raster generations, not semantic payload identity.

## Publication algorithm

### Document change

Compile/project the document skeleton once, build its control index once, publish a new document
generation, and invalidate only pane instances referencing the retired generation. A plugin owner
generation must be part of action-token ownership exactly as it is today.

### Pane open or structural fallback

Clone/instantiate the skeleton into one retained pane instance, build one `UiSurface` and host-node
authority, bind actions once, and publish the pane structure generation. This is the only normal
path allowed to execute the complete projection/surface/host-model chain.

### Scalar/control update

Resolve the control through `PaneControlIndex` and apply a typed property batch directly to the
retained surface and host-node row. One batch validates all targets first, commits once and returns
exact dirty node IDs. It must not recursively search the projection per patch and must not clone all
node attributes into the action registry.

### Collection update

Apply inserts, removes, moves and row-field patches by stable row ID. Reconcile the visible range
plus bounded overscan, reuse existing materialized rows and release rows that leave the range. A
selection change patches the previous and next selected rows, not the complete list.

### Resize

Publish a geometry generation against the same pane structure. Recompute affected layout frames and
visible range, materialize only rows entering that range, patch hit cells and submit old/new damage.
Do not rebuild the document projection, payload TOML, action registry or semantic host nodes. This
must compose with the separate window geometry transaction described in
`2026-08-25-window-metrics-geometry-publication-design.md`.

## Payload-specific migrations

| Payload | Current duplication | Target delta authority |
| --- | --- | --- |
| Hierarchy | Full nodes -> TOML tables -> host scene rows | Stable node ID, parent/depth/order generation, expanded/filter generation, visible DFS window; selection patches at most old/new rows |
| Console | Existing retained slots and logical generation | Keep current delta model; remove exact content size from semantic key and make geometry/visible slots independent |
| Inspector | All strings and plugin properties cloned into payload/projection | Stable section/field IDs; per-field value/validation generation; materialize visible expanded sections |
| Performance Timeline | Four complete row arrays copied into TOML | Separate frame/span/hotspot/control item sources with capture generation and visible range |
| Module Plugins | Every plugin and 12+ strings copied into a TOML table | Plugin ID keyed rows; patch load/enabled/diagnostic fields; sort/filter generation owns ordering |
| Build Export | Every target copied into TOML | Profile ID keyed target rows; patch status/progress/diagnostics fields |
| Animation | Whole item string arrays copied | Stable track/node/state/transition IDs plus per-collection deltas |
| Template V2 patches | Recursive projection search and surface scan per control | One generation-owned control index used by projection, surface, action registry and host model |

The root template may receive bounded scalar summaries such as count/filter/status. It must not
receive the collection rows themselves.

## Typed fallback

```rust
enum PaneProjectionFallback {
    MissingPaneInstance,
    DocumentGenerationMismatch,
    StructureGenerationMismatch,
    MissingControlId,
    DuplicateControlId,
    UnsupportedCollectionDelta,
    StableRowIdentityConflict,
    GeometryGenerationMismatch,
    ResponsiveStructureChanged,
    ActionOwnerGenerationMismatch,
}
```

Fallback is prepared before mutation and counted by reason. An ordinary selection, scroll, output
append or resize sequence must not silently rebuild the pane instance.

## Complexity budget

Let `N` be template nodes, `S` total semantic payload size, `V` visible rows plus overscan, `K`
changed scalar controls, `R_delta` changed collection rows and `L_aff` affected layout nodes.

| Operation | Required time | Required transient ownership |
| --- | --- | --- |
| Unchanged pane publication | `O(1)` generation checks | `O(1)` |
| Scalar/control patch | `O(K log N)` or expected `O(K)` | `O(K)` typed patches |
| Scroll | `O(V_enter + V_leave)` plus bounded layout | `O(V_enter)` rows |
| Row field/selection update | `O(R_delta log V)` | `O(R_delta)` |
| Resize | `O(L_aff + V_enter + V_leave)` | independent of `S` |
| Document replacement | `O(N + V)` for each active affected pane | explicit structural path |

No ordinary operation may perform `O(S)` TOML construction, `O(P*N)` control search, or full
host-model reconstruction. Memory must be `O(N_skeleton + active panes * (N_instance + V))`, not
`O(active panes * complete logical collections)`.

## Required counters

- document skeleton build/reuse and retired generation counts;
- pane instance build/reuse/fallback counts;
- projection, surface build, full layout and host-model build counts;
- payload TOML scalar/row/string materialization counts;
- control-index lookup count/candidates and recursive fallback visits;
- logical, visible, materialized, entered, released, rebound and patched rows per collection;
- structure/semantic/control/collection/geometry generation advances;
- action registry full rebinds versus control-state patches;
- allocation count/bytes, CPU duration and peak/quiescent RSS per scenario.

Hard invariants for unchanged publication, scroll and ordinary resize:

- `pane_instance_build_count == 0`;
- `projection_build_count == 0`;
- `surface_build_count == 0`;
- `host_model_full_build_count == 0`;
- `payload_toml_row_materialization_count == 0`;
- `recursive_control_search_visit_count == 0`;
- `materialized_row_count <= visible_row_count + overscan_budget`.

## Test-first implementation sequence

1. Add a lower regression proving two identical pane publications preserve document skeleton,
   pane surface, host-node model and action epoch identities.
2. Add a resize regression proving semantic/structure identities stay stable while geometry and
   visible range advance.
3. Add 10K-row Hierarchy, Inspector, Timeline, Plugins and Export tests that assert bounded
   materialized rows and zero TOML row materialization. Inspector must keep physical row count
   independent of plugin field count and preserve stable field-ID edit routing after row rebinding.
4. Add old/new selection, append/expire, insert/remove/move and filter delta tests with exact cloned
   row counts.
5. Add component patch tests proving one indexed lookup per target and atomic rejection of missing or
   duplicate controls.
6. Generalize the Console cache into document skeleton, pane instance and collection caches; retain
   its working logical-generation/slot-delta behavior.
7. Remove collection serialization from `inject_payload_attributes` only after each consumer uses
   the retained item source.
8. Run the product stress matrix through the official managed Windows lane.

## Product acceptance

Artifacts must remain below `E:\zircon-profiles`. For each scenario run at least three measured
passes after warm-up:

- 10K-node Hierarchy scroll, selection and filter;
- 10K-property Inspector scroll, field edit and component replacement;
- 100K-line Console append/expire and scroll;
- 10K-row Performance Timeline and Module Plugins scroll;
- 2,000-step pane/window resize with all four panes populated;
- plugin document hot replacement followed by action dispatch.

Record p50/p95/p99 input-to-visible latency, CPU, allocations/bytes, peak and quiescent RSS, row
materialization, full projection/surface/layout/host-model counts and damage pixels. Compare the
final visual tree, hit results, actions and accessibility rows against a forced full-rebuild oracle.

Initial gates are p95 within one 60 Hz frame on the reference machine, no ordinary step above 50 ms,
bounded materialization, zero full pane reconstruction for unchanged/scroll/resize paths and less
than 5% quiescent RSS growth after the stress sequence. These are target gates, not measured claims.

## Ownership note

`pane_payload_projection.rs`, `runtime_host.rs`, `template_runtime_projection.rs`,
`console_projection.rs`, `apply_presentation/pane_conversion.rs` and the host projection caches are
currently externally modified. This report is deliberately isolated. Implementation must occur
under those owners or after an explicit handoff; none of their current edits should be reverted or
absorbed into the present static candidate.

The Inspector migration additionally owns `workbench/component_property_rows.rs`, which is
externally modified in the current worktree. `workbench/data_sync.rs` and the shared
`virtual_rows.rs` helper are clean and were reviewed but not changed: truncating data sync without a
visible-window contract would lose fields, while reducing only the helper's scan constant would
preserve unbounded physical row ownership.
