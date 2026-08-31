# UI renderer and event hot-path recheck

Date: 2026-08-30

Status: current-source review and static validation; managed Rust and product
timing remain pending. This record is supplemental to the retained UI review
and does not replace concurrently maintained report sections.

## Source binding

- HEAD at recheck: `8b7c5336f7d1abf053fea2db8c6aedc24f4f7e8b`
- Structural audit: `E:\zircon-profiles\ui-structural-audit-20260830-r7`
- Audit SHA-256: JSON `3EC8A041106C17E58059D038B192C314D958F470ED4E233B5867D96B2043F2FA`; CSV `1CE0B2970D0EE41A650AA96BE32D2C134DE74ADB8F9195B3F3254DF430BB528E`
- Audit scope: 4,861 production UI Rust files / 522,108 lines; 5,981 clone calls, 3,028 Vec materializations, 193 sort calls, 8,145 string allocations, 3,263 traversal signals; 2,200 dirty hotspots across the UI roots.
- Profiling preflight:
  `E:\zircon-profiles\ui-profile-preflight-20260830-r14.json` (SHA-256
  `8A575B0268250916101606FACD7C0C9044DF0CD4A6071B90F9FAFA2AAF996D26`);
  the managed target and 276 critical source fingerprints are bound to HEAD
  `14c89f9776bed828cc85e05e4b9914b3f8d1e784`, but the editor and runtime PE
  products are absent (`missing_editor_binary`, `missing_runtime_binary`).

The structural inventory was expanded after this initial recheck. The r9
artifact first proved that the default scope includes the product screen-space
renderer and both the RHI UI surface trait and WGPU implementation, including
their module-root `.rs` files. That scope regression was RED at 3/8 fixture
files before the change and GREEN at 8/8 afterward.

The current ranking authority is
`E:\zircon-profiles\ui-structural-audit-20260830-r13`, bound to HEAD
`14c89f9776bed828cc85e05e4b9914b3f8d1e784`. Its analyzed production-source
manifest SHA-256 is
`7539D5CA1694A837740C88FEDA74D9643D021DBEB69BA9306E5406DF2DCA532F`;
its JSON SHA-256 is
`A8D44613A8DB6EB4BC470DEE9D3993A49318625A9C0C38F3A8272CC2A11803E4`
and CSV SHA-256 is
`7D47757B07331CBFBCD2C6C1526602907994AB98A2248407DD0141AABB256345`.
It covers 4,976 Rust files and 426,787 production-prefix lines, with 4,883
clone calls, 1,025 capacity/copy Vec materializations, 106 sorts, 6,376 string
allocations and 2,545 traversal signals; 2,403 paths are dirty in the shared
worktree. All 4,976 rows have typed numeric fields.

r12 superseded r9 at their common `399f...` source binding and r13 carries the
same calibrated method to the newer HEAD. The audit now excludes allocation-free
`Vec::new()`/empty `vec![]` defaults and analyzes only the source prefix before
the first standalone `#[cfg(test)]`, matching the repository's performance-
contract convention. Relative to r9 this removes 1,273 test-only clone calls,
2,589 empty/test Vec signals, 146 test-only sorts, 2,100 test-only string
allocations and 1,050 test-only traversal signals. For example,
`sdf_advances.rs` falls from five Vec signals to its two production
`with_capacity` products, while `HostMenuPointerLayout` falls from four Vec
signals to zero because its defaults allocate no capacity. A test-only file
with an empty production prefix is retained as a zero-signal row; the exporter
now fails fast on any unexpected PowerShell diagnostic. The focused Pester
suite passes 7/7, including deterministic production-manifest binding: test-only
changes preserve the manifest hash while production-prefix changes invalidate it.

The expanded scope still ranks `render/resolved_layout.rs`, `sdf_atlas.rs`,
text auto-route/SDF products, and WGPU batching/bounds-index as the strongest
renderer owners. All of those product paths are dirty in the shared worktree,
so the result is a review and ownership-routing artifact, not authorization to
edit a lower-ranked clean file. In particular, the clean RHI
`image_resources.rs` score comes mainly from scene construction/resource-table
composition; its only production merge call is not the stable input/resize
dependency sweep and was rejected as a substitute optimization.

The audit is a heuristic inventory, not a CPU, allocator, GPU, or latency
measurement. Dirty counts include shared external worktree changes and are not
ownership authorization.

The r7 delta is small but directionally important: source volume and clone/
Vec/sort counts are effectively unchanged while string and traversal signals
increased. This confirms that the audit identifies a large conversion and
projection surface, not a single missing micro-cache. The next selection gate
is therefore (1) an event/frame call-chain witness, (2) a typed invalidation
boundary, and (3) a bounded operation model or product counter. A high score
without all three is not an implementation authorization.

## Reference calibration

The primary reference for this review is the checked-in Unreal source under
`dev/UnrealEngine/Engine/Source/Runtime/SlateCore`, with Slint/Fyrox used as
secondary editor/runtime shape references in the parent plan. Two Unreal
boundaries are especially relevant:

- `SlateCore/Public/Input/HittestGrid.h` defines a cell index whose lists are
  ordered back-to-front. `Private/Input/HittestGrid.cpp` adds a widget by
  taking its already-painted `GetPaintSpaceGeometry()` and
  `GetRenderBoundingRect()`, computing covered cells once, and only updating
  sort data when geometry is unchanged. `GetBubblePath` then queries the cell
  candidates; it does not arrange or repaint widgets on the input event path.
- `SlateCore/Public/FastUpdate/SlateInvalidationRoot.h` owns separate
  pre-update, prepass, and post-update heaps plus a final update list. Layout,
  child-order and visibility invalidation can request the slow path; paint,
  volatility and render-transform invalidation can stay on the fast path.
  `InvalidateScreenPosition` is a dedicated screen-position shift and does not
  imply a widget-tree rebuild.

The corresponding Zircon contract is therefore: publish one frame-owned
geometry/hit authority after layout/paint preparation, classify invalidation by
route, geometry, paint and resource domains, and let the event thread perform
only cell lookup plus semantic dispatch. A window move, pure scale or paint-only
feedback must not reconstruct the tree, all text, or all image payloads. A
topology, clip or responsive-layout break may conservatively promote to a full
rebuild, but that promotion must be typed and counted. This is the standard
used by the next visual journal, geometry publication and dependency-union
milestones below.

## Findings

### Calibrated clean-candidate adjudication

The r13 clean-file ranking was checked against callers before selecting a
production edit:

| Candidate | Call-chain result | Decision |
| --- | --- | --- |
| `zr_rhi/src/ui_surface/image_resources.rs` | scene/resource-table composition | not a stable input or frame dependency sweep |
| `scene_renderer/ui/sdf_advances.rs` | two real output buffers, all product consumers dirty | do not change the return ABI outside its owner |
| `zr_rhi_wgpu/ui_surface/geometry/clipping.rs` | real clipped-solid product path; allocates one result and two eight-vertex scratch buffers per call | valid reuse opportunity, but scratch lifetime belongs in dirty `geometry.rs`; requires owner transfer |
| `render/parity.rs` | called by render debug/parity snapshots and tests | diagnostic cost, not product-frame evidence |
| `text/fallback_overlay.rs` | only constructs owned native batches after SDF atlas failure | fallback correctness path, not stable-frame work |
| `HostMenuPointerLayout` | only zero-capacity Vec defaults in the clean DTO; producer and bridge are dirty | r9 false positive removed by r12 calibration |

No clean leaf has both a product hot-path witness and ownership of the lifetime
needed for a structural fix. Editing a return type, moving an allocation into a
dirty caller, or optimizing a diagnostic/fallback path would not address the
reported button, resize or stable-render latency. The current independent
implementation slice is therefore the calibrated audit and the source-bound
dependency-product design; production changes wait for exact owner transfer or
a reconciled clean caller boundary.

### Product renderer boundary

`UiBatchPlan::from_paint_elements` is currently referenced by Runtime Interface
debug/parity/test paths, not by the screen-space GPU renderer. Its ordered-input
fast path is still a valid helper optimization, but its sort reduction must not
be reported as a product GPU-frame speedup.

The product path is
`zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs`
and `render.rs`. `ScreenSpaceUiPlanCache` retains submission/segment products by
`Arc` identity, route projection, viewport and background generation. The
segment planner reuses caller-owned `Vec<UiPaintElement>` scratch. A stable
submission can return the cached prepared plan without visiting commands; a
changed segment is planned in isolation and unchanged segments are replayed.

The remaining renderer work is dependency work, not repeated SVG file parsing:
stable image segments still check unique texture dependencies and text still
checks font/glyph readiness before a product hit is useful. These checks need
generation-aware union counters and a managed product profile before any claim
of GPU-cache success.

The implementation contract for the image half is now recorded in
`2026-08-30-screen-space-image-dependency-product-plan.md`. Direct Unreal
`FSlateResourceHandle`, `FSlateShaderResourceManager` and
`FSlateRHIResourceManager` evidence establishes that retained draw products
cache invalidatable shared resource handles while explicit resource-manager
events own cleanup; visibility traversal is not a per-frame lease renewal.
Fyrox's texture cache independently confirms resource modification counters as
the GPU update authority. Applied to Zircon, the exact stable
`Arc<PreparedScreenSpaceUi>` must be the O(1) frame key, the plan cache must
publish a segment change journal, and segment/in-flight products must strongly
pin binding products. Hashing or comparing every segment on a stable frame, or
merely widening the existing epoch retention window, is explicitly rejected.

The refreshed deterministic artifact is
`E:\zircon-profiles\runtime-ui-render-dependency-product-pressure-20260830-r6.json`
(SHA-256
`D248736C905A859D1E1384311AB982E4B0E1A1C7A1C6732C7BB9C537E9E2BF8B`).
It retains the four typed resource-generation fallbacks while reducing modeled
image segment visits from 262,144 to 288, dependency/binding lookups from
1,048,576 to 1,152, and binding-retention entry visits from 2,097,152 to zero.
The 910.22x reduction is an operation-count target, not CPU timing.

The product command planner now fills transient paint elements. Cache
generation hashing and debug-label formatting remain available to retained and
diagnostic consumers, but are deferred to the text-batch route that actually
needs a command generation. Non-text commands therefore avoid per-command
stable-JSON serialization and metadata string allocation while preserving the
same geometry, text payload, and decoration inputs.

### 2026-08-31 current-source audit refresh

The latest source-bound structural audit is
`E:\zircon-profiles\ui-structural-audit-20260831-r1`.
It is bound to HEAD
`14c89f9776bed828cc85e05e4b9914b3f8d1e784`, with production manifest SHA-256
`8CE67D03305A198CA7D91B2BA4C6A71A78BC439546F3FDEC544E8D582D281699` and JSON
SHA-256
`517B9675C1832A0B6FF8BA37F5FA4FE96678751E624D21936A1D9AE843CC879B`.
The audit covers 4,977 files and 426,854 production-prefix lines, recording
4,896 clone calls, 1,025 capacity/copy Vec materializations, 106 sorts, 6,371
string-allocation signals and 2,546 traversal signals. The worktree has 3,359
dirty paths in the selected UI roots; that count is not edit authorization.

The top heuristic row is
`zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs`
(risk score 572; 71 clones, 115 string-allocation signals and 14 traversal
signals). Its callers are pane-body/template semantic projection and dynamic
control-state updates, not the WindowMetrics event path. It should be measured
on pane/content changes before any optimization is attributed to resize or
pointer latency. The next rows similarly include prototype compilation,
debug-reflector projection and material-editor data projection; they are useful
owner queues but are not evidence of a stable-frame bottleneck.

This refresh reinforces the selection rule: a heuristic score authorizes only a
call-chain review and counter design. A production optimization requires a
current-source profile showing invocation frequency, wall time, allocation bytes
and frame correlation. The latest profile preflight still has no editor/runtime
PE products, so CPU, RSS, input-to-present and GPU timing remain unmeasured.

The current preflight receipt is
`E:\zircon-profiles\ui-profile-preflight-20260831-r1.json` (SHA-256
`EFC8FA404A10443E745CBE6B1BE23CB7103A488546B718130373523AC899747F`). It is
bound to HEAD `14c89f9776bed828cc85e05e4b9914b3f8d1e784` and 276 critical source
files, but is not ready because the managed target is missing both
`zircon_editor.exe` and `zircon_runtime.dll`. This is a validation prerequisite,
not a reason to treat static pressure ratios as measured product performance.

The full render extraction path now collects owner-text prewarm requests and
the overlap-admission result in one draw-order traversal. This removes the
second full admission scan when the prewarm threshold is reached while keeping
the sub-threshold path unchanged; it does not change text shaping, render
command construction or GPU residency semantics.

### Input/event boundary

`zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs`
routes each pointer event synchronously. The current shared owner path still
calls `refresh_after_state_change` from
`callback_dispatch/template_bridge/workbench/pointer_feedback.rs`, and
`workbench/reference/template_surface.rs` then performs `rebuild_dirty` and
host projection refresh before the event returns. The event loop merges the
redraw request only afterward. Native redraw coalescing therefore does not
coalesce the preceding UI refresh work.

The deterministic model
`E:\zircon-profiles\editor-pointer-frame-cadence-refresh-pressure-20260830-r1.json`
models 1,000 changing events. At 17 events per displayed frame, a
frame-owned visual journal would reduce surface and projection refreshes from
1,000 each to 59 each, avoiding 941 of each and reducing the two modeled
refresh stages by 16.949x. The artifact SHA-256 is
`B10F2CBA6B04FE749D9BAD524126E4E59F92F918BC1881FD3690C07E71F1B0B2`.
This is an operation-count model, not timing.

### SVG and GPU image boundary

The current source already has the required layers for retained visual
assets, but they have different ownership and invalidation keys:

- `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg/cache.rs`
  keeps parsed SVG trees behind a source/path fingerprint and checks the
  memory cache before candidate-file or parser work.
- `visual_assets/loading/cache.rs` keeps raster pixels in a bounded,
  fingerprinted LRU; `loading/pixels.rs` checks the retained pixel entry before
  filesystem candidates or SVG rasterization.
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs` retains
  screen-space image bindings by `Arc` identity, prepared resource resolution
  by management generation, and per-plan image geometry by segment identity.
  Uploads are skipped when the resident GPU generation is unchanged.
- `zircon_runtime/src/ui/icon_atlas/svg.rs` now retains parsed SVG documents in
  a source-keyed 512-entry LRU used by `UiIconAtlasBuilder::build_plan`; cache
  hits avoid reparsing while parse errors remain uncached. This is a CPU
  document cache only and does not claim GPU texture residency.

This means repeated SVG work is more likely to come from identity or
invalidation churn than from a missing cache. A cache hit can still be lost if
the same source is represented by a new path/fingerprint, if a raster bucket
changes every frame, if an atlas page is recreated, or if a resource generation
is advanced for unrelated assets. The renderer also walks each retained image
segment's dependency list during `prepare` so bindings remain live; this is
bounded reference work, but it is not evidence that SVG work is absent.

The runtime atlas cache has a deterministic pressure model at
`E:\\zircon-profiles\\runtime-ui-svg-document-cache-pressure-20260830-r1.json`
(SHA-256 `A7BBD57AC7F0A4B6AB76D49F3CA946FA1B15BF8EAFCCE4AEE81A880A5860513D`).
With 1,000 stable plan builds and a 64-document working set, source parsing is
modeled as 64,000 operations without retention and 64 cache misses with the
bounded cache (1,000x fewer). The model excludes file reads, rasterization,
GPU uploads, CPU timing and allocator RSS.

### Hit-grid geometry budget

The base hit-test grid now rejects non-finite or non-positive entry geometry
before it becomes an index member. It caps each axis at 128 cells (16,384 total
cells) with checked vector sizing. A single entry that would cover more than
4,096 cells promotes the grid to a 1x1 coarse fallback;
the query remains correct because the final frame/clip test still runs, while
the extreme case is explicitly bounded instead of allocating one cell per
authored pixel region. This is a fail-closed memory bound, not a claim that
large authored surfaces are cheap: the coarse fallback has wider candidate
work and must be visible in product counters.

The base builder records `ui.hit_grid.invalid_geometry_entry_count` and
`ui.hit_grid.coarse_fallback_count`. These counters distinguish a bounded safety
fallback from a normal fine-grid frame and are required in the large-surface
product capture. The popup projected-grid path remains under its existing owner
and is intentionally unchanged in this slice.

The deterministic budget artifact
`E:\\zircon-profiles\\runtime-ui-hit-grid-budget-pressure-20260830-r1.json`
(SHA-256 `F92DC97E696DEB1D1D422673ADEFAE69A03039242537DDBBD7821E56DF765518`)
models a 1,000,000 by 1,000,000 surface: an unbounded 244,140,625-cell grid
becomes a single-cell fallback. Ordinary 256 by 128 surfaces retain a 4 by 2
partition. The model excludes cell-membership bytes, CPU timing, RSS and
product hit latency.

The deterministic pressure artifact
`E:\\zircon-profiles\\runtime-ui-render-cache-batch-reuse-pressure-20260830-r2.json`
and SVG model
`E:\\zircon-profiles\\editor-svg-gpu-residency-20260828.json`
  (SHA-256 `75229B1A12EAB306527937F78468A84D02763E6008EAB96A41C1927EE6862F3A`)
are operation-count evidence only. With the model defaults, repeated
per-command reconstruction would perform 20,480,000 source reads, parses,
rasterizations and upload writes, while content-addressed residency performs
257 cold/reload source reads, 1,028 raster products and 17 atlas page uploads;
stable frames perform zero of those materialization operations. The model also
shows four independent 64 MiB ceilings, which can permit a configured 256 MiB
aggregate before allocator and container overhead. These ceilings must be
unified or explicitly budgeted before enabling more residency.

Acceptance therefore requires a measured, source-bound capture with separate
counters for cache hits/misses, candidate construction, SVG parses and bytes,
raster products and pixels, GPU prepare command visits, upload writes/bytes,
resource-generation churn, and eviction counts. The stable-hover and stable-
resize phases must show zero parse/raster/upload work, positive retained hits,
and a source manifest covering every cache owner. Missing counters are a
blocker; they must not be interpreted as zero.

### Frame publication boundary

The earlier full-copy hypothesis is rejected by current source. `UiArrangedTree`
and `UiHitTestGrid` store their entries in `UiPersistentSequence`, a persistent
`Arc` tree with 64-entry leaf segments and fanout 32. Outer publication clones
are constant-time shared-root operations, while local rebuilds replace only the
affected persistent pages. `UiSurfaceFramePublication` separately shares
unchanged domains and patches changed render ranges. Its arranged-node and
hit-entry clone counters remain zero for publication itself.

The product event path is also frame-authoritative. Instance hit testing borrows
the retained `UiSurfaceFrame` from `UiSurfaceFramePublication`, then queries its
published projected grid through the cell index and the surface-owned reusable
query scratch. It neither clones the frame `Arc` nor calls the lazy
`surface_frame()` publication path. A dirty outer publication therefore keeps
serving the last coherent immutable hit/layout domains until rebuild publishes
the next frame; it does not move frame materialization into the first input
event. The no-frame initialization/deserialization fallback reads the existing
rebuild-owned projected cache and likewise performs no arranged/render scan.
Indexed arranged-node lookups serve incremental rebuilds, while linear ancestry
helpers are limited to debug and diagnostic paths. Optimization work must
preserve this boundary and must not add an event-time popup projection or
geometry reconstruction pass.

One residual copy was outside the arranged/render/hit counters. A layout-domain
publication cloned the complete `UiLayoutEngineSelectionReport.selections` Vec
before wrapping the report in an `Arc`. The report is diagnostic data, but the
copy ran synchronously for every layout publication, including window resize.
`selections` now uses the existing 64-entry `UiPersistentSequence`: frame
publication shares its directory root, and replacing one stable route clones
only the touched leaf segment and directory path. The report-level replacement
API exposes the actual copied item, segment, and directory-node counts instead
of inferring them from the model. The public field name,
`from_selections(Vec<_>)` constructor, borrowed ordered iteration, single
`usize` indexing, `Eq` contract, and JSON-array wire shape remain available.
The public carrier type changed from `Vec` to `UiPersistentSequence`, so owned
or mutable iteration, range indexing, and other `Vec`/slice APIs are a source
migration rather than a compatibility promise. Current in-tree consumers use
only the supported borrowed/read subset.

The source-bound capacity receipt is
`E:\zircon-profiles\runtime-ui-layout-report-aggregation-pressure-20260831-r4.json`
(SHA-256
`9C1ACEFA05D01D4D0B3D7081BE464A8B0EDA24CB4BE1960F4ADE49AE846F1809`,
source-manifest SHA-256
`656323793AB7CBAC92E89446AC926443D297319AE3350E555CC672C3A1CC8A4C`).
For 10,000 routes, 1,000 layout publications, and one changed route per
publication, the modeled selection clone work falls from 10,000,000 to 64,000,
plus 2,000 directory-node clones and 1,000 publication handle clones. The
156.25x ratio and the 1,271,808,000 modeled avoided bytes use an explicit
128-byte payload input; they are not allocator, RSS, CPU, or product-latency
measurements. Publication still copies the bounded fallback-reason Vec: the
default scenario records 8,000 reason-entry copies and 1,000 small-vector
allocations rather than hiding that residual work.

Layout publication had a second, independent focus-domain cost. The rebuild
marker previously treated every arranged/layout change as a focus change, so a
resize cloned the bounded `UiFocusState` histories and rebuilt both vectors in
`UiFocusPath` even when the focused ID and parent chain were unchanged. Rebuild
publication now leaves focus state clean and validates the retained path against
the current arranged-node index. Validation checks the focused ID, reverse-route
parity, and every parent edge in O(depth) without constructing a route. A focus
ID change, missing node, reparented edge, or full dirty publication still
rebuilds the path conservatively. Profiling separately records focus-state/path
builds and the validation visit upper bound.

The source-bound capacity receipt is
`E:\zircon-profiles\runtime-ui-surface-frame-domain-sharing-pressure-20260831-r3.json`
(SHA-256
`A37BF22AFEA6FD431EB97CF545A61DBA69A770A32172A9FF42952B64C16F5496`,
source-manifest SHA-256
`3A4ABD91903F58509C98FAC0313115F169126B09CEBEB07CA8002BA0238E4195`).
With explicit inputs of 4,096 layout-only publications, 1,024 focus-state
payload items, and an eight-node focus path, the model replaces 4,194,304
payload clone units with at most 32,768 indexed parent reads, a 128x
capacity-to-visit ratio. This is not a CPU, allocator, RSS, cache-locality, or
product-latency ratio. The lower resize/reparent Rust regressions are authored
but remain unexecuted without managed Cargo authorization.

### 2026-08-31 window-metrics adjudication

The current resize layout implementation already follows the reference-engine
shape more closely than the earlier inventory implied. Root-size changes enter
the incremental layout pass; `arrange_resized_root` updates the root and only
children present in the maintained parent-size dependency index. Fixed-size
children are skipped, and the responsive-layout gate returns before candidate
traversal when the width remains in the same threshold band. This is comparable
to Fyrox's constraint/validity early exits and Unreal's dirty-widget update
lists. It is not a full-tree layout by default.

The remaining structural gap is above the runtime surface. The Editor's
WindowMetrics geometry publication still carries the complete semantic
`PaneData` value. The current `scene_conversion.rs` clones the four dock panes
while creating a retained geometry scene, and
`HostWindowGeometryPresentationData::apply_to` clones the same four panes again
before overwriting the values in the new geometry product. Two additional
floating-window clone source sites scale with the floating-window count. A
zero-floating resize therefore has a source-proven lower bound of eight
semantic pane clones per published frame.

The source-bound pressure receipt is
`E:\zircon-profiles\ui-window-metrics-pane-clone-pressure-20260831-r1\ui-window-metrics-pane-clone-pressure.json`
(SHA-256
`F5A9A3AE500DE770DD7BE7185BBE6FC1AB84E486EFE70A99B595184DBB6D767B`,
source-manifest SHA-256
`82B1729C3DE997DAD9CD6309BAB12CADDA8C53ACB27A3A1ED5C5F67FD470C62F`).
At 600 frames and an explicit 1 MiB payload estimate, the eight-clone lower
bound models 4,800 clones and 5,033,164,800 copied bytes. The byte value is a
scenario input, not a measured allocator size or product timing result.

The required architectural change is recorded in
`2026-08-31-window-metrics-semantic-pane-product-plan.md`: semantic pane products
must retain shared identity across a geometry-only update, while geometry and
hit products publish only changed rows. The product files are externally dirty
at this source binding, so this review does not absorb or rewrite them.

## Scoped changes in this slice

- `zircon_runtime_interface/src/ui/surface/render/cache.rs`: reason-first
  batch status and borrowed source-index validation; dirty frames return early
  without scanning source indices or allocating a temporary source Vec.
- `zircon_runtime_interface/src/ui/surface/render/batch/plan.rs`: ordered
  adjacency check with stable fallback ordering.
- `zircon_runtime_interface/src/ui/surface/render/batch/tests.rs`: ordered and
  same-key stability regression coverage.
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs`: product
  planner uses transient paint metadata and computes command generation only
  for text batches.
- `tools/tests/test_runtime_ui_batch_plan_order_contract.py`: source guard that
  keeps the diagnostic batch helper out of the product renderer path.
- `tools/runtime_ui_transient_paint_metadata_pressure.py` and
  `tools/tests/test_runtime_ui_transient_paint_metadata_contract.py`: bounded
  operation-count evidence and source contract for the transient product path.

The transient product path does not alter the GPU renderer ABI, popup hit
authority, or shared Runtime/Interface blocker files.

The instance hit-test path now delegates the borrowed published frame to the
same frame-query core used by `hit_test_surface_frame_with_query`, while passing
the `UiHitTestIndex`-owned `UiHitQueryScratchCell`. Radius/touch queries therefore
retain candidate and mark storage across events; immutable snapshot callers use
a call-local index and remain allocation-isolated from the surface. The lower
dense-radius regression now enters through `UiSurface::hit_test_with_query`,
checks frame/instance parity, and verifies consecutive scratch generations.

The matching pressure model is stored at
`E:\\zircon-profiles\\runtime-ui-hit-query-scratch-pressure-20260830-r1.json`
(SHA-256 `CA9355CD2991B2A8D2E3A0B98B0F0B208776EC1F10C6D0205EC82AD48C41513C`).
For 65,536 entries and 1,000,000 radius queries it models scratch
initialization falling from 1,000,000 to 1 and storage-allocation events from
2,000,000 to 2. It intentionally excludes CPU timing, allocator RSS, GPU work,
and candidate-scan cost.

The indexed responsive-layout path now keeps width thresholds per responsive
candidate plus aggregate threshold counts and a last-resolved width. A root
resize inside the current threshold band returns before traversing the
candidate sets; standard MUI breakpoints and authored `min/max`, `up/down`,
`between` and media-query thresholds are tracked. A candidate's membership or
responsive metadata definition invalidates the remembered width, so a same-
width metadata mutation cannot be incorrectly skipped. Initialization may
still scan the tree, and a threshold crossing still resolves candidates before
the ordinary measure/arrange stages; this gate only removes redundant
responsive candidate work on stable-width-band resizes.

The crossing predicate covers both inclusive breakpoint entry and strict
`max-width` exit, so a change from exactly `800` to `801` cannot be skipped.

The definition map also records responsive dependencies that are not themselves
one of the four traversal sets, notably implicit Grid-item `size` and `offset`.
Changing such a child reopens the gate at the current width, preventing a
same-band root resize from hiding a local placement mutation. Only the small
responsive attribute subset is copied for this comparison; unrelated node
metadata does not force a full candidate pass.

`UiSurface::apply_invalidation_transaction` now consumes its validated
transaction through an owned iterator. The previous path cloned every
`UiInvalidationChange` into a temporary vector before marking nodes dirty; the
consumer now applies the same ordered changes directly and records them in the
pending invalidation state. This removes a per-transaction allocation and one
change clone per entry without changing generation validation or dirty-domain
semantics.

The deterministic model is stored at
`E:\\zircon-profiles\\runtime-ui-invalidation-transaction-pressure-20260830-r1.json`
(SHA-256 `4BBD16D2CA684B3C6EE7ACCF33082C2F6AF236AC8D127FDACF5D789AFC116A46`).
At 16,384 changes across 120 transactions it models 1,966,080 avoided change
clones and 120 avoided temporary vectors. It excludes tree validation, dirty
propagation, map writes, CPU timing and allocator RSS.

The deterministic model is stored at
`E:\\zircon-profiles\\runtime-ui-responsive-width-gate-pressure-20260830-r1.json`
(SHA-256 `DF4B0D669D0D66397BEE21ABA236F036E349D174A6B71D09D2E25CD3AAD0AA23`).
With 10,000 candidates, 200 resize steps and two threshold crossings, modeled
candidate visits fall from 2,000,000 to 30,000 (66.6667x fewer). This is an
operation-count model only; it excludes measurement, arrangement, paint,
timing and memory measurements.

The full render extraction path now collects owner-text prewarm requests and
the overlap-admission result in one draw-order traversal. The removed second
traversal was only present once the request threshold was reached; below that
threshold the modeled work is unchanged. The deterministic model is stored at
`E:\\zircon-profiles\\runtime-ui-render-prewarm-scan-pressure-20260830-r1.json`
(SHA-256 `1056D7FFEC3B17E078610216321F75B68E018E2F4844C8864AFC28C8D8920912`).
With 100,000 nodes and 1,000 full extracts, it models 100,000,000 eliminated
admission visits and a total draw-order reduction from 300,000,000 to
200,000,000 visits (1.5x). It excludes text shaping, command construction,
CPU timing, allocator RSS and GPU work.

`EditorWorkbenchTemplateFrames` is a geometry-only snapshot. The editor
template surface now extracts it during layout recomputation and reuses it for
render/input-only state changes, selected by `report.layout_recomputed`.
Semantic and geometry host-projection patches still run for those state changes,
so native hover, focus and press visuals remain current. The deterministic model
is stored at
`E:\\zircon-profiles\\runtime-ui-frame-extraction-cache-pressure-20260830-r1.json`
(SHA-256 `D86E7A12CB073058EEF1141441B644AE0A2976547B6368078009455E36CB32CC`).
With 1,000 render/input-only refreshes and 1,024 geometry controls, this removes
1,024,000 repeated frame-control visits; it is an operation-count model, not a
CPU or input-to-present measurement.

The pointer feedback boundary now follows the same frame-owned rule. Pointer and
tooltip callbacks keep routing, capture and property writes synchronous, but no
longer call `refresh_after_state_change` from the event callback. They publish a
`WORKBENCH_PROJECTION` invalidation; the frame recompute refreshes a pending
surface once and then applies the host projection patch. Existing callers that
already prepared the surface are recognized as clean, preventing a second
`rebuild_dirty` pass. This preserves semantic action order while coalescing
hover/press/tooltip feedback from multiple events into one frame transaction.
The deterministic model is stored at
`E:\\zircon-profiles\\runtime-ui-pointer-feedback-deferred-pressure-20260830-r1.json`
(SHA-256 `A71663622DA34F2858F3A308D6CE29493AA40041F993402F89BEFE454AA8CFFC`).
At 1,000 pointer events and 17 events per frame, modeled surface refreshes fall
from 1,000 to 59; this is an operation-count model, not a latency measurement.

## Validation

- Python focused contracts: 32/32 passed in the current focused rerun,
  including the batch-plan, render-cache, transient-metadata, pointer-cadence,
  SVG residency, SVG evidence, SVG identity and raster-bucket contracts.
- Full focused optimization contract set: 55/55 passed in the final rerun;
  this includes the editor interactive-frame, pointer cadence and all SVG/GPU
  cache-model contracts listed above.
- Hit-query scratch contract, pressure model and the same focused set: 59/59
  passed after the instance-path reuse change.
- Published-frame hit authority/scratch contracts: 7/7 focused static/model
  checks passed. The adjacent hit/frame set passed 37/38; the only failure is
  the existing runtime UI directory-mirroring drift, outside the hit-test paths.
  Scoped Rust formatting, Python compilation and `git diff --check` passed. The
  Rust frame/instance/scratch regression is authored but was not Cargo-executed.
- Responsive width-gate contract and pressure model: 3/3 passed; the broader
  `test_runtime_ui_*performance_contract.py` sweep passed 191/191. A complete
  `test_runtime_ui_*.py` sweep passed 334/337; the three failures are existing
  shared-worktree drift in architecture mirroring, pointer-state owner
  visibility, and surface profile-call counting, and are unrelated to this
  gate.
- Invalidation transaction ownership contract and pressure model: 3/3 passed.
- Render prewarm scan contract and pressure model: 4/4 passed; the scoped
  `extract.rs` rustfmt check and `git diff --check` also passed.
- Runtime SVG document-cache contracts and pressure model: 5/5 passed; the
  scoped atlas/SVG rustfmt and `git diff --check` checks also passed. The
  cache is intentionally limited to parsed CPU documents; GPU upload
  residency still requires the renderer-owned measured path.
- Base hit-grid budget contract and pressure model: 4/4 passed;
  scoped hit-test rustfmt and `git diff --check` checks also passed. This
  closes the unbounded cell-allocation shape at the index boundary, while
  measured large-surface latency and membership memory remain product gates.
- Editor geometry-frame cache contract and pressure model: 5/5 passed; the
  pointer cadence regression also passed 6/6. The scoped template-surface
  rustfmt check and `git diff --check` passed. The regression confirms a
  render/input-only hover update reuses the cached geometry frame while still
  applying the host projection patch.
- Deferred pointer-feedback contract and pressure model: 7/7 passed; the
  callback-to-frame ordering contract confirms pending surface state is refreshed
  before projection patching, while the one-event-per-frame case preserves the
  original refresh count.
- Current-source structural audit r7: 4,861 production UI Rust files / 522,108
  lines; 5,981 clones, 3,028 Vec materializations, 193 sorts, 8,145 string
  allocations and 3,263 traversal signals. The audit is bound to HEAD
  `8b7c5336f7d1abf053fea2db8c6aedc24f4f7e8b` and is stored under
  `E:\\zircon-profiles\\ui-structural-audit-20260830-r7`.
- Calibrated structural audit r13: 4,976 UI/renderer/RHI Rust files / 426,787
  production-prefix lines; 4,883 clones, 1,025 capacity/copy Vec
  materializations, 106 sorts, 6,376 string allocations and 2,545 traversal
  signals. It is bound to HEAD
  `14c89f9776bed828cc85e05e4b9914b3f8d1e784` and production manifest
  `7539D5CA1694A837740C88FEDA74D9643D021DBEB69BA9306E5406DF2DCA532F`,
  stored under `E:\\zircon-profiles\\ui-structural-audit-20260830-r13`.
  Scope, empty-Vec, inline-test exclusion, empty-production-prefix and manifest
  regressions pass 7/7; the result remains a heuristic selection aid, not
  product timing.
- Unreal calibration sources were read directly from
  `dev/UnrealEngine/Engine/Source/Runtime/SlateCore` (`FHittestGrid`,
  `FSlateInvalidationRoot`, and `SInvalidationPanel`); this is source evidence,
  not a product timing result.
- Transient metadata pressure model:
  `E:\zircon-profiles\runtime-ui-transient-paint-metadata-pressure-20260830-r1.json`
  (SHA-256 `48020F6A1C8F05030DBE4EDD7FBA0579D71D7D7D64A5297956722A8A9461DDAC`);
  at 32,768 commands / 8,192 text commands, the model removes 24,576
  stable-generation calls and 32,768 debug-label format calls. This is not
  product timing or a CPU/RSS measurement.
- Pester preflight + structural audit: 7/7 passed.
- WindowMetrics semantic-pane pressure contract: 2/2 Pester tests passed. The
  current-source receipt proves four scene-conversion dock clone sites, four
  geometry-apply dock clone sites, and two floating-window clone source sites.
  It is a clone-count and explicit-byte model, not a CPU, RSS or latency
  measurement.
- Layout-report storage and pressure contracts: 11/11 focused Python tests
  passed. A lower Rust regression preserves a 129-route published snapshot
  across a one-route replacement, checks the three-segment carrier, asserts the
  exact 64-item/one-segment/one-directory-path COW work, and confirms the JSON
  wire value remains a 129-item array. That Rust test is authored but not
  reported as executed because this slice did not receive Cargo authorization.
- Layout/focus publication contracts: 14/14 focused Python tests and the
  adjacent frame/focus suite 17/17 passed. The full capture-output Pester suite
  remains 47/48; its only failure is the pre-existing foreign workbench pointer
  counter contract (`ui.workbench.pointer.feedback_refresh_count`), while the
  surface-frame counter source/fixture/gate case passed. Rust resize/reparent
  regressions are source-authored but not Cargo-executed.
- Current-source structural audit r1: 4,977 production UI/renderer/RHI Rust
  files / 426,854 production-prefix lines; 4,896 clones, 1,025 Vec
  materializations, 106 sorts, 6,371 string-allocation signals and 2,546
  traversal signals. It is bound to HEAD
  `14c89f9776bed828cc85e05e4b9914b3f8d1e784`, production manifest
  `8CE67D03305A198CA7D91B2BA4C6A71A78BC439546F3FDEC544E8D582D281699`
  and JSON SHA-256
  `517B9675C1832A0B6FF8BA37F5FA4FE96678751E624D21936A1D9AE843CC879B`,
  stored under `E:\zircon-profiles\ui-structural-audit-20260831-r1`.
  Its top heuristic row is semantic pane projection, not evidence that pointer
  routing or resize layout traverses that file.
- SVG/GPU current-source adjudication: parsed Editor SVG trees are bounded at
  1,024 entries; raster variants are bounded at 4,096 entries and 64 MiB with
  size/tint keys and shared `Arc<[u8]>`; GPU bind groups reuse stable texture
  `Arc` identity. Stable image prepare nevertheless remains `O(S + D + B)`
  because it walks all segments and dependencies and renews the binding-cache
  epoch. This confirms the dependency-product plan rather than a fourth cache.
- Product profiling preflight is source-bound but not ready. Receipt
  `E:\zircon-profiles\ui-profile-preflight-20260831-r1.json` (SHA-256
  `EFC8FA404A10443E745CBE6B1BE23CB7103A488546B718130373523AC899747F`)
  reports 276 critical source files and two blockers: missing
  `E:\cargo-targets\zircon-engine\debug\zircon_editor.exe` and missing
  `E:\cargo-targets\zircon-engine\debug\zircon_runtime.dll`. No CPU, RSS,
  GPU timing or input-to-present claim is made from this preflight.
- Windows sampled-CPU capture is now fail-closed and storage-bound. The WPR
  helper starts `CPU` file-mode recording with `-recordtempto` inside the
  session directory on `D:`, `E:` or `F:`, stops to a nonempty `system.etl`,
  and requires xperf to export a source-fingerprinted
  `cpu-sampled-profile.txt`. The raw xperf report is system-wide, so its
  receipt keeps `is_product_timing=false`. The product capture now registers
  the exact `zircon_editor.exe` PID, executable fingerprint and start/exit
  timestamps, then asks xperf for a second
  `cpu-product-sampled-stacks.txt` using both `-pid` and the trace-relative
  process-lifetime `-range`. Only a nonempty filtered report sets sampled-CPU
  product attribution true; latency, frame-time and GPU acceptance remain
  separate. This replaces the prior capture shape that neither constrained
  WPR temporary storage nor exported function/module samples.
- The real WPR smoke start under
  `E:\zircon-profiles\ui-wpr-xperf-smoke-20260831-r1` failed before recording
  with exit `-984068079 (0xC5585011)`, "Failed to enable the policy to profile
  system performance". No ETL or product timing was claimed and `wpr -status`
  reported no active recording afterward. The source-bound `-RequireWpr`
  preflight `E:\zircon-profiles\ui-profile-preflight-20260831-r5.json`
  (SHA-256
  `BD2317DD87A0EE0683D28FFE03FC2F1884AAB9025880FFD1AD8CB15DDAAC82E2`)
  binds 276 critical sources and the WPR helper fingerprint. It exposes
  `wpr_system_profile_privilege_missing` instead of allowing a partial
  capture; the remaining product blockers are the missing managed editor EXE
  and runtime DLL. The focused attribution contract is GREEN at 8/8, but uses
  mocked native-tool output and is not product performance evidence.
- Scoped Rustfmt (`skip_children=true`): passed for the responsive candidate,
  responsive pass, slot index, incremental layout, invalidation and surface
  transaction files.
- Scoped `git diff --check`: passed.
- No Cargo command was started by this slice. Two pre-existing Cargo processes
  and one child `rustc` were observed and left untouched; `.git/index.lock` was
  absent at the final check.

## Next implementation plan

1. Keep pointer route, capture, focus, press/release and action dispatch
   synchronous; move only coalescible visual writes into a bounded,
   latest-value journal drained by `commit_interactive_frame_update`.
2. Add route-affecting barriers for geometry/input-policy changes so no event is
   routed against a stale published frame generation.
3. Instrument and validate a renderer dependency-union publication keyed by
   exact frame-product identity, a planner-published segment change journal and
   texture/font generations. Image segment and in-flight products must own
   binding lifetime so stable frames perform no liveness traversal; keep the
   old sweep only as a typed full fallback until lower and product evidence is
   GREEN.
4. Preserve segmented layout-report publication and add product counters for
   selection leaf/directory clones before attributing resize latency to it.
5. Trace SVG source/raster/GPU identity churn and consolidate the four visual
   cache ceilings into an explicit process budget before changing eviction
   policy.
6. Run managed lower-layer and Editor product validation only after current
   source copies are complete and the official lane is released. Require CPU,
   RSS, input-to-damage, damage-to-submit, input-to-present and SVG/GPU counters
   before calling the interaction target met.
