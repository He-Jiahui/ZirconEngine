# WindowMetrics geometry-only publication design

Date: 2026-08-25

Status: implementation-ready design; production edits are intentionally deferred while the
current owners hold the affected retained-host files. This document is a current-source review,
not evidence that the product resize path is already fast.

## Preparatory command boundary implemented

A narrow current-source slice now classifies `ResizeSplit`, `SetDrawerExtent` and the new atomic
`SetDrawerRegionExtent` as geometry-only workbench commands. These commands no longer run global
drawer normalization or `EditorUiHost::recompute_session_metadata`; a region resize is journaled as
one event and validates all target drawers before committing. This removes duplicated command-side
structure work, but the event effects still request broad layout/presentation invalidation. The
geometry-generation publication architecture below remains required and unvalidated.

## Decision

`WindowMetrics` must stop publishing a new monolithic `HostWindowPresentationData` for ordinary
window movement and resize. Zircon needs two independently versioned authorities:

1. immutable semantic/structural presentation data; and
2. a geometry generation containing local layout frames, scene geometry, hit-test geometry and
   damage information.

An ordinary size change may recompute affected layout geometry, but it must not rebuild pane
payloads, strings, semantic template nodes, menu data, scene projections or host presentation
structure. A desktop-position-only move must not rerun local layout at all.

The geometry generation and its hit-test patch must publish atomically. A frame must never expose
new visual geometry with an old hit grid, or vice versa.

## Current-source proof

The present path has a narrow invalidation classification but a broad publication path:

- `app/host_lifecycle/recompute.rs:72-75` selects `WindowMetrics` and calls
  `build_window_metrics_shell_snapshot`.
- `app/host_lifecycle/recompute/shell/builder.rs:15-46` recomputes shell template bridge layout
  frames and returns `reuse_shell_layout: false`.
- `app/host_lifecycle/recompute.rs:89-109` continues through floating projection, viewport and
  pointer-layout synchronization, pane payload collection and presentation application.
- `app/host_lifecycle/recompute/presentation.rs:285-334` still prepares hierarchy/showcase data
  and calls `apply_presentation_with_template_v2_data`.
- `ui/apply_presentation.rs:94-252` converts a complete host scene and calls
  `set_host_presentation`; the scene conversion is not a geometry-only publication.
- `host_contract/window/presentation.rs:26-49` records a `PresentationRebuildCount` and replaces
  the presentation.
- `host_contract/globals/state.rs:158-176` replaces the `Arc` and advances
  `presentation_structure_generation` even though ordinary resize has not changed semantic
  structure.

The monolithic data contract is the underlying cause. `HostWindowPresentationData`
(`host_contract/data/host_root.rs:11`) owns semantic content and geometry in one value.
`HostWindowLayoutData` (`host_contract/data/host_components/window.rs:36`) carries the primary
frame rectangles, while `HostWindowSceneData` (`host_contract/data/host_scene.rs:9`) carries a
second layout copy plus metrics, orchestration and all pane projections. Updating geometry by
replacing the enclosing value necessarily visits or clones unrelated semantic state.

The workbench template bridge also exposes many geometry-bearing frames. They are consumed by
painting, pointer routing, viewport placement, splitters, drag/drop and floating-window
projection. Updating only one consumer would create a split authority and is therefore rejected.

Historical profile evidence identifies the priority but is not a current acceptance result. In
the archived two-frame resize capture, recompute consumed about 3056.67 ms, presentation about
1673.89 ms, host-scene construction about 1634.88 ms and pointer bridge work about 1221.80 ms;
the frame also repainted roughly 1,573,352 pixels and uploaded 131,072 bytes. These values must be
replaced by source-bound, repeated measurements after the managed validation lane is available.

A fresh structural inventory is bound to HEAD
`1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8` plus 1,216 dirty UI paths and is stored at
`E:\zircon-profiles\ui-structural-audit-20260825-133934`. It scanned 4,740 production Rust files.
Within this chain, `recompute/presentation.rs` scored 42 heuristic risk points,
`apply_presentation.rs` 78 and `apply_presentation/scene_conversion.rs` 114; the latter contains
38 syntactic clone calls. This is a source prioritization signal only. The receipt explicitly says
that it is not CPU, allocation or latency evidence, so it cannot satisfy the product acceptance
gate below.

## Reference-engine mapping

Unreal Slate provides the closest relevant contract in the checked-in reference source:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h:91-120`
  distinguishes child-order/layout invalidation from `InvalidateScreenPosition`, and owns cached
  hit-test and draw data at the invalidation root.
- `SlateInvalidationRoot.cpp:272-291` promotes child-order/layout changes to the slow path.
- `SlateInvalidationRoot.cpp:342-344` represents a screen-position change with the dedicated
  `bNeedScreenPositionShift` bit.
- `SlateInvalidationRoot.cpp:356-424` shifts cached desktop geometry, retains cached elements, and
  selects `PaintFastPath` unless a real slow-path condition is present.

The transferable principle is not Unreal's exact containers. It is the separation of semantic
widget topology, local geometry, desktop transform and cached paint/hit products. Zircon's
existing generation checks and retained surface indexes are suitable building blocks, but their
ownership must be split before a resize fast path can be honest.

## Required authority split

Introduce the following conceptual state. Exact module names may follow the owner branch, but the
ownership and generations are mandatory.

```rust
struct HostPresentationStructure {
    shell: HostShellSemanticData,
    pane_models: HostPaneModelSet,
    template_nodes: HostTemplateSemanticNodes,
    native_surface_membership: NativeSurfaceMembership,
    structural_signature: HostStructureSignature,
}

struct HostPresentationGeometryGeneration {
    generation: u64,
    window_local_size: UiSize,
    desktop_transform: UiAffine2,
    host_layout: HostWindowLayoutData,
    scene_geometry: HostSceneGeometryData,
    template_frames: BuiltinWorkbenchWindowLayoutFrames,
    hit_index_generation: u64,
    damage: UiDamageRegionSet,
}

struct HostPresentationSnapshot {
    structure_generation: u64,
    structure: Arc<HostPresentationStructure>,
    geometry: Arc<HostPresentationGeometryGeneration>,
}
```

`HostSceneGeometryData` is the geometry-only subset currently mixed into
`HostWindowSceneData`: layout, surface metrics, orchestration, resize/drag frames and dock/floating
frames that truly depend on window metrics. Menu strings, pane content, hierarchy rows, console
lines, inspector fields and other semantic payloads remain in `HostPresentationStructure`.

Do not implement the fast path with `Arc::make_mut` on the current monolithic presentation. If a
renderer or input reader retains the previous snapshot, that approach clones the entire semantic
graph and preserves the original asymptotic problem.

## Event classification

Window changes must enter one of three explicit paths:

| Change | Required work | Forbidden work |
| --- | --- | --- |
| Desktop move only | Publish a new desktop transform or root origin; shift cached desktop-space diagnostics | Local layout, semantic projection, pane payload rebuild, local hit-cell reindex |
| Client size change | Incrementally recompute affected local frames, patch changed hit cells and submit old/new damage union | Semantic presentation rebuild, pane conversion, string reconstruction, structure generation advance |
| DPI/raster-scale change | Recompute local geometry; invalidate scale-keyed text/raster products and affected damage | Semantic/pane structure rebuild unless a responsive structural breakpoint also changes |

A responsive breakpoint may legitimately change mounted controls or topology. That is a typed
fallback to the structural path, not a hidden branch inside the geometry publisher.

## Geometry transaction

The size-change path is a single prepare/validate/commit transaction:

1. Capture the expected structure generation, prior geometry generation and old affected frames.
2. Recompute `WorkbenchShellGeometry` and template layout for the dirty layout roots only.
3. Build a structural signature from mounted control identities/counts, popup topology, native
   surface membership, pane model identities, visibility/mode and component identity.
4. Compare it with the committed signature before publishing. Any mismatch returns a typed
   structural fallback.
5. Produce exact geometry deltas for changed frame/clip rows. Derive the union of old and new
   bounds for damage.
6. Patch/reindex only affected hit-test cells. A desktop move changes the hit-grid root transform,
   not every local entry.
7. Under one mutable state borrow, verify the expected structure and geometry generations.
8. Atomically install geometry, hit-index generation and damage; increment geometry/hit
   generations only.
9. Queue paint for the bounded damage set. Readers receive either the old complete generation or
   the new complete generation.

All downstream geometry consumers must read from the published geometry generation. Keeping an
independent mutable copy in pointer bridges, scene data or viewport projection would reintroduce
split authority. Bridges may retain a generation-tagged derived cache, but a cache miss must be
reconstructed from this authority and must never publish independently.

## Typed fallback contract

The geometry prepare step returns a specific reason rather than silently invoking full rebuild:

```rust
enum HostGeometryPatchFallback {
    MissingCommittedState,
    StructureGenerationMismatch,
    GeometryGenerationMismatch,
    BridgeLayoutFailure,
    MountedControlSetChanged,
    PopupTopologyChanged,
    NativeSurfaceMembershipChanged,
    ResponsiveBreakpointChanged,
    HitIndexPatchFailed,
    GeometryNonFinite,
}
```

Each reason increments a separate bounded counter. Repeated fallback on ordinary continuous
resize is a correctness/performance failure, not an acceptable adaptive behavior.

## Complexity budget

Let:

- `S` be total semantic presentation size, including pane payloads and strings;
- `L_aff` be layout nodes affected by the metric change;
- `H_aff` be hit entries/cells affected by changed geometry; and
- `D` be the number of merged damage regions.

Required ordinary resize cost is `O(L_aff + H_aff + D)` time and `O(L_aff + D)` transient memory.
It must be independent of `S`. A responsive root resize can still make `L_aff = O(N_layout)`, but
it must not add `O(S)` semantic projection and allocation. A desktop move is `O(1)` publication
plus any platform-required damage bookkeeping.

The hit patch must not scan all hit entries to validate global properties. Maintain maximum/order
metadata in the index or validate only affected entries. Damage merging must be bounded; exceeding
the region budget collapses to one union rectangle instead of creating an unbounded vector.

## Required counters

The implementation is not accepted without source-bound counters for:

- window metric events by move/size/DPI class;
- geometry prepare, commit and no-op counts;
- layout nodes visited and frame rows changed;
- hit entries/cells patched and full hit-index rebuilds;
- damage input regions, merged regions and damaged pixels;
- geometry transaction fallback count by reason;
- structure and geometry generation advances;
- presentation rebuild, host-scene build, pane projection and template semantic materialization;
- retained draw-command reuse, raster upload bytes and text/SVG cache misses;
- CPU duration, allocation count/bytes and resident-set samples for the resize interval.

For an ordinary resize sequence the following are hard invariants:

- `presentation_rebuild_count == 0`;
- `host_scene_semantic_build_count == 0`;
- `pane_projection_count == 0`;
- `structure_generation_delta == 0`;
- `hit_index_full_rebuild_count == 0`;
- `geometry_fallback_count == 0`.

## Test-first implementation plan

1. Add a lower-layer transaction test that starts with a retained semantic snapshot, applies one
   size change and proves structure `Arc` identity and structure generation are unchanged while
   all geometry readers observe the same new generation.
2. Add hit-test parity tests at old/new splitter, pane edge, toolbar button and viewport boundary
   coordinates. The old-only region must reject; the new-only region must hit.
3. Add a desktop-move test proving zero local layout visits, zero local hit-cell patches and a
   preserved physical virtual pointer in route diagnostics.
4. Add a DPI test proving scale-keyed text/SVG/raster products invalidate without semantic
   presentation reconstruction.
5. Add one test per typed fallback, especially responsive breakpoint, popup topology and stale
   generation.
6. Add a product-path resize test that uses the real editor host and asserts the hard counters,
   final visual geometry and input parity.
7. Only after the lower and product tests are red for the current full path, implement the state
   split and geometry transaction.

## Product stress and acceptance

Run through the official managed Windows validation lane; raw Cargo is not an accepted substitute.
Artifacts must be written under `E:\zircon-profiles`, never to the system drive.

Required scenarios:

- 200-step interactive resize with the default editor workspace;
- 2,000-step stress resize with at least 10,000 visible/retained UI nodes and populated hierarchy,
  inspector and console panes;
- desktop move without resize;
- repeated DPI changes across two scale factors;
- resize across and away from a responsive breakpoint;
- identical runs for the GPU and software presentation backends where supported.

Use at least three measured runs after warm-up. Record p50/p95/p99 input-to-visible latency,
resize-handler CPU, whole-process CPU, allocations/bytes, peak RSS, damaged pixels, uploads and
cache hit ratios. Capture a final screenshot and probe the same boundary coordinates through the
published frame hit authority.

Initial acceptance targets, subject to replacing the historical baseline with current-source
measurements:

- p95 resize input-to-visible latency at or below one 60 Hz frame budget on the reference machine;
- no individual ordinary resize step above 50 ms;
- peak RSS growth after the 2,000-step run below 5% after quiescence;
- zero semantic presentation rebuilds and zero full hit-grid rebuilds during ordinary resize;
- damage pixels proportional to old/new changed geometry, not unconditional full-window paint;
- no repeated SVG decode/upload for unchanged scale/content keys;
- final geometry, rendering and hit results equal to a forced full rebuild oracle.

These are acceptance gates, not claimed measurements.

## Implementation sequence and ownership

1. Split semantic structure from geometry generation in the host contract.
2. Make paint, hit testing, pointer bridges and viewport projection consume the geometry
   generation.
3. Add the atomic geometry transaction and typed fallback.
4. Route desktop move, client resize and DPI events separately.
5. Add bounded damage publication and remove unconditional structural generation advances.
6. Run lower-layer tests, then the editor product regression and stress matrix.
7. Compare counters and CPU/RSS/allocator evidence against the forced full-rebuild oracle.

At the time of this review, the principal implementation paths (`committed_shell_state.rs`,
`recompute.rs`, `recompute/presentation.rs`, `ui/apply_presentation.rs`,
`host_contract/globals/state.rs` and the host hit-index paths) contain external in-progress changes.
This report therefore does not edit them. Implementation must resume only under their active owner
or after an explicit ownership handoff; the report itself is independent and can be reviewed now.
