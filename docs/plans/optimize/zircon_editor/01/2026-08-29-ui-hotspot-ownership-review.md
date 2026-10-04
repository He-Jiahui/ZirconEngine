---
related_code:
  - zircon_editor/src/ui
  - zircon_runtime/src/ui
  - zircon_runtime_interface/src/ui
source_artifact:
  - E:/zircon-profiles/ui-structural-hotspots-20260901-runtime-diagnostics/ui-structural-hotspots.json
  - E:/zircon-profiles/ui-structural-hotspots-20260901-runtime-diagnostics/ui-structural-hotspots.csv
  - E:/zircon-profiles/editor-shell-pointer-resize-pressure-20260901-r7.json
  - E:/zircon-profiles/runtime-diagnostics-surface-pressure-20260901/runtime-diagnostics-surface-pressure.json
  - E:/zircon-profiles/performance-timeline-dead-projection-20260902-r2/performance-timeline-dead-projection.json
  - E:/zircon-profiles/runtime-diagnostics-scope-pressure-20260902-r2/runtime-diagnostics-scope-pressure.json
  - E:/zircon-profiles/editor-pane-presentation-transaction-pressure-20260902-r10/editor-pane-presentation-transaction-pressure.json
  - E:/zircon-profiles/editor-hierarchy-sparse-publication-pressure-20260902-r2.json
  - E:/zircon-profiles/editor-overlay-presentation-pressure-20260902-r1/editor-overlay-presentation-pressure.json
source_binding:
  head: 9963f8eb72e2d725d2536eb50b393b30387a1ffa
status: current_source_static_candidate
---

# UI source hotspot ownership review

## Finding

The current source-bound inventory is a prioritization signal, not a runtime
profile. It scans 5,009 UI source files (429,594 lines) and records 4,760
`clone` calls, 1,012 vector materializations, 109 sorts, 6,251 string
allocation signals, and 2,486 traversal signals. Of those files, 194 are
currently dirty hotspots. The audit deliberately does not claim CPU, allocator,
latency, or GPU cost.

The highest-ranked paths are currently dirty in the shared worktree:

| path | score | ownership decision |
| --- | ---: | --- |
| `zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs` | 572 | Do not edit here. Projection cache and pane conversion ownership are already under another change; review pure resize bypass at the caller. |
| `zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs` | 338 | Do not edit here. Runtime template compiler is outside this editor-side slice and is part of the current-source validation surface. |
| `zircon_editor/src/ui/workbench/debug_reflector/model.rs` | 336 | Do not edit here. Debug reflector model is externally modified and not proven to be a frame hot path. |
| `zircon_editor/src/ui/material_editor/renderer_data_projection.rs` | 324 | Do not edit here. Requires renderer-data publication authority and parity coverage. |
| `zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs` | 321 | Do not edit here. The payload mutation contract is shared with binding/runtime work. |
| `zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs` | 309 | Do not edit here. Most signals are static binding string construction; first prove publication frequency. |
| `zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_display_surface_classes.rs` | 309 | Do not edit here. This is template compilation rather than pointer/resize dispatch. |
| `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs` | 289 | Do not edit here. The pane generation/WindowMetrics cutover is shared and incomplete. |

The clean paths inspected in the same ranking pass do not justify a local
production edit:

- `showcase_demo_state/defaults.rs` constructs demo state during initialization;
  its `BTreeMap` and string construction are cold-path fixture work.
- `template_runtime/harness.rs` is compatibility/test snapshot support, not the
  measured event or paint path.
- `paint_template_nodes/template_node_surface/commands.rs` and
  `template_node_text.rs` copy small `FrameRect` values into owned paint
  commands. Replacing those copies requires changing the owned command contract;
  a local borrowed helper would still have to materialize the command and would
  not establish a reusable frame cache.
- `editable_text_composition.rs` performs work proportional to the small IME
  clause count. The real duplication is that input and render consumers each
  request a fresh validated `Vec`; the correct future owner is a text-state
  generation product, not a leaf-local cache that can go stale.
- `component/state_reducer/world.rs` updates a fixed number of scalar/vector
  fields and has no node or command traversal.
- `pane_component_projection/button_style.rs` already returns borrowed
  attributes when no compatibility alias is required. Its clone occurs only
  when it must synthesize alias keys; eliminating that copy requires changing
  the upstream typed style publication, not another converter cache.
- `platform_input/keyboard_map.rs` allocates the two owned names required by
  `UiKeyboardInputEvent`. Borrowing them locally cannot outlive the Winit event,
  while changing the interface crosses currently modified input owners.
  `typeahead_timer.rs` only builds diagnostic strings on timer delivery and is
  not a pointer or resize hot path.
- `host_contract/window/event_wake.rs` already coalesces channel wakes on one
  atomic pending edge. Repeated callbacks before the event loop consumes the
  edge do not issue repeated native wakes.
- the native-pointer damage helpers clone `FrameRect`, which is four `f32`
  scalars. Those calls do not allocate and are not evidence of retained payload
  reconstruction.
- `module_plugin_projection/cache.rs` avoids rebuilding the projection but
  returns an owned `ModulePluginsPaneViewData`, so a stable read still clones
  the pane. Removing that copy requires shared pane payload ownership through
  the currently modified recompute/pane-composition boundary; changing only
  the clean leaf cache would preserve the deep copy at its caller.

The call-chain review did identify a higher-priority structural owner outside
those ranked UI roots. `RuntimeUiSession::dispatch_input` still reverse-scans
all surfaces for an uncaptured pointer, clones the event for each candidate,
and calls the surface dirty-rebuild boundary during event routing; the
uncaptured fallback repeats the same pattern. Pointer capture already proves
that direct surface routing is possible. For 64 surfaces and 100,000 events in
each of the pointer/focused/unrouted classes, the deterministic owner model
permits 19.2 million surface dispatches, 18.9 million event clones, 19.2 million
event-time rebuild probes, and 38.4 million text-owner sync operations. The
target publication authority reduces those classes to 300,000 direct surface
dispatches, 100,000 owned event transfers, zero event-time rebuild probes, and
300,000 text-owner sync operations. These are operation counts from
`E:/zircon-profiles/runtime-ui-surface-input-publication-20260828.json`, not
timings. The production file and its split module directory are both externally
modified, so this review keeps them read-only and routes the work through
`2026-08-28-runtime-ui-surface-input-publication-authority.md` instead of
adding a second index or leaf cache.

The renderer review found a separate current-source P1 issue in
`zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs`. Even when every
`PlannedScreenSpaceUi` segment and the resource-management generation are
stable, `prepare` iterates every render segment, calls
`refresh_segment_dependencies`, visits every texture dependency, resolves the
GPU texture reference, looks up its bind group, and then scans the binding map
for epoch retention. The stable generation only skips the resource-id
resolution cache clear; it does not skip the dependency walk. The text segment
cache has the correct stable-frame shape and returns its retained frame product
before segment traversal, although a changed text segment still causes
frame-wide dependency/run-index composition.

The image fix must be source-owned, not an Editor cache. A stable frame key
must cover ordered segment identity, viewport, resource generation and backend
epoch and return before prepare-epoch allocation or binding retention. For a
partial change, each segment must retain a shared dependency/binding product so
unchanged segments do not need epoch touches; only changed segments resolve
textures and rebuild geometry. Resource-generation change, backend recovery,
viewport change and forced upload remain typed full fallbacks with counters.
The file currently has overlapping external changes, so this review records the
algorithm and acceptance contract without editing it.

The Runtime Diagnostics conversion contains a separate duplicate authority. In
`ui/apply_presentation/pane_conversion.rs`, every Runtime Diagnostics pane first
passes through `rebuild_pane_template_hit_artifacts`, which publishes the pane's
`body_surface_frame`. The following refresh then calls
`runtime_diagnostics_debug_surface_frame` and creates a second `UiSurface`, a
new root and one new tree node per displayed row before calling `surface.rebuild()`.
The resulting frame is immediately consumed only by `debug_surface_frame`; it is
not retained as a separate input or paint authority. This makes each eligible
presentation conversion perform an avoidable full tree/layout/rebuild pass, and
the existing live-body test already demonstrates that the published body frame
is the intended source for the reflector snapshot.

The bounded repair is to make the reflector read the already-published
`body_surface_frame` and fail closed when that frame is absent. This removes the
second `UiSurface::new`, node/path/attribute materialization, and `surface.rebuild()`
from the refresh path without changing reflector text, overlay semantics, or the
post-refresh rebuild required to publish newly generated debug rows. Acceptance
requires a source guard proving that the production refresh no longer constructs
`UiSurface`, a frame-identity regression proving the snapshot reads the pane's
published frame, and a deterministic pressure model that reports operation
counts only (`timing_claim=false`).

Runtime Diagnostics labels are not dispatchable by default, so this authority
also requires the common pane frame builder to include non-dispatchable nodes
for this pane kind. Those nodes are published with `UiInputPolicy::Ignore` and
click/hover/focus disabled. Other pane kinds retain the existing dispatchable-only
frame behavior, and the hit-test authority does not admit diagnostic labels as
pointer targets.

The render-submission call chain proves that this is a live path: a successful
viewport submission schedules a shell-content or full-presentation refresh
whenever Runtime Diagnostics is visible. The live converter also flattens every
reflector section and node label into temporary `Vec<String>` products and then
copies those strings again into final template-node `SharedString` values. The
local follow-up is a single-pass final-node writer for the live frame path. It
must retain the owned payload/export format used by active external snapshots,
preserve row ordering and selected-node markers, and avoid introducing a cache
inside the converter.

Performance Timeline shares the same render-submission refresh target. Its host
contract declares four complete row-model products plus three summary strings,
but current product consumers only read `PerformanceTimelinePaneData.nodes` for
paint, geometry profiling, and hit testing. No current source reads the copied
frame/span/hotspot/control DTOs or the copied summary/session/output fields.
Nevertheless, every conversion maps all logical rows before the already-present
visible-row projection builds `nodes`. The bounded hard cut removes those dead
host DTOs and mappings; it retains source-side timeline data, visible-row
virtualization, capture-control nodes/actions, and all rendered text.

The retained-host menu pointer bridge now follows the adjacent viewport-toolbar
authority. It derives the exact expected menu-button/popup node identities and
routes, classifies each synchronization as `NoChange`, `Geometry`, or `Topology`,
and calls `publish_authored_geometry` with only changed node IDs. Geometry retains
the existing `UiSurface`, dispatcher, and route-intent map. A visible-node or
route mismatch is a typed full reconstruction. Hover-only state is rejected by
an O(1) surface-state gate before popup-item comparison or node validation.

WindowMetrics publication also uses a geometry-only layout builder. Menu trees,
preset names, and intrinsic measured popup widths are shared immutable products;
resize translates button frames and resolves popup viewport widths without text
measurement or semantic reconstruction.

`E:/zircon-profiles/editor-shell-pointer-resize-pressure-20260901-r7.json`
(SHA-256 `92B6B8BFC12B6FCFD042B695D36E1CDF9BFE66201108169238BE3B69F81D41DA`,
source-manifest SHA-256
`2A536C9BD827FD9F97E8D60C5FE92A252B12E31477E6B49FAF77F3FEB2DA6369`)
binds the current seventeen-source implementation to Unreal `FSlateInvalidationRoot`.
For 200 resize steps and three changed geometry nodes, the corrected five-domain
model is 12,000 legacy versus 3,000 retained visit units (4.0x) and avoids 200
Surface builds plus 2,200 dispatcher/route/path reconstructions. For 10,000
hover-only state changes it avoids 10,000 Surface builds and 110,000 dispatcher
registrations. The geometry-only layout product additionally avoids 200 semantic
layout builds, 8,000 popup text-row measurements and 8,000 popup-item projections.
These are deterministic operation counts, not CPU or latency measurements.

The same source-bound fixture now covers the activity rail pointer bridge. For
200 geometry-only resize steps with two visible strips and four buttons, its
retained path performs zero Surface/dispatcher/route rebuilds, visits 5,000
changed-node domain units instead of 7,000 full-rebuild units (1.4x), and avoids
1,200 dispatcher registrations, route bindings, and route-path constructions.
The geometry-only builder also avoids 200 semantic product builds and 800 tab
projections/identifier clones by retaining the two tab `Arc` products. Changing
strip visibility or button cardinality remains a typed topology fallback;
semantic target replacement at equal cardinality retains the hit authority.

Host-page, document-tab, and drawer-header pointer receipts contain no geometry.
WindowMetrics now retains those published semantic DTOs instead of rebuilding
them from the workbench model. The default 200-step fixture avoids 600 receipt
product builds and 2,600 page/tab item projections and identifier clones. Other
recompute targets still execute all three semantic projections.

Welcome/recent pointer state likewise retains its semantic project-path list on
WindowMetrics and calls the existing viewport-only sync. The default 200-step
fixture avoids 200 recent-path clones and 1,600 recent-item projections; regular
semantic recompute still refreshes the list from the chrome snapshot.

The Runtime Diagnostics retained-frame fixture binds the duplicate-surface repair
to the current source. For 200 eligible presentation conversions with 256
displayed rows, the legacy path performs 200 duplicate `UiSurface` builds,
51,400 tree-node/path materializations, 51,200 attribute-map materializations,
and 200 extra layout rebuilds. The current path performs 200 reads of the
already-published `body_surface_frame` and zero duplicate tree, attribute, or
layout work. These are deterministic operation counts, not timing or memory
measurements.

For the same fixture, the live single-pass writer removes 400 intermediate
section/node `Vec` products and 62,600 intermediate owned flattened-string
products while keeping the final template-node count unchanged. It does not
change the active-snapshot payload/export representation. These are product
construction counts; they are not allocator-byte or latency measurements.

`E:/zircon-profiles/runtime-diagnostics-surface-pressure-20260901/runtime-diagnostics-surface-pressure.json`
(schema `zircon.editor.runtime_diagnostics_surface_pressure.v2`, SHA-256
`281034C6945CA64CD74B7AD183A831BA0C6572ABD280B08CE01C75C2F4A83901`,
source-manifest SHA-256
`5D597417E06C831DCDA9BB3ADEBE1BABA63B8BFC30AF20EC8D44EB05442A5A20`)
is the current artifact. Its authority is the host contract's published
`body_surface_frame`; a missing frame fails closed instead of reconstructing a
surface during conversion.

`E:/zircon-profiles/performance-timeline-dead-projection-20260902-r2/performance-timeline-dead-projection.json`
(SHA-256 `0905CA761DFBF67B8312EEA60D7A673864368DBE34FD88829D3CBA2189F19F2E`,
source-manifest SHA-256
`A5057684488C9ADD96C55E787E6192B58034F8925D6030A64ECE07F5C9629B40`)
binds the nodes-only Performance Timeline contract. For 200 refreshes with 256
logical rows in each of the frame/span/hotspot streams, it avoids 153,600 unused
row mappings, 800 unused capture-control mappings, and 600 unused summary/session/
output string clones. Visible virtualized template nodes and capture-control
action nodes remain. These are deterministic operation counts, not timings.

The published-frame repair does not yet cover active diagnostic document tabs or
floating windows. Their render-submission target remains `FullPresentation`,
whereas a unique visible drawer uses the existing `ShellContent` path. The
existing scoped presentation helper proves that row-level floating storage
replacement is viable, but it is currently owned by the UI Asset Editor path.
The source-bound scope-pressure artifact
`E:/zircon-profiles/runtime-diagnostics-scope-pressure-20260902-r2/runtime-diagnostics-scope-pressure.json`
(SHA-256 `0B15C05728E8274979CBF5A2DC80711E4A6C97472F56F950391B295B25893C9A`,
source-manifest SHA-256 `32E3B3CC21E11DB19DE3E3C6431CAF16C7C81F9FE396D3C13F7B9297ECE1B483`,
tool SHA-256 `2F742C2D5845AAA46631C9712DE09D6D93BC1F89AF7934A86478D06229F0BE76`,
test SHA-256 `7E223F8B2C13465B1DE4EE62BABC4AAC7ED0DBFC3123B5847F782F53AB497EFB`)
models 200 successful render refreshes with eight unrelated panes as 200 full
presentation refreshes, 1,600 unrelated-pane visits, and 200 global hit-index
rebind boundaries. A pane-scoped target would retain one diagnostic payload
build and one damage region per refresh, with zero unrelated-pane visits. This
is an operation model only (`timing_claim=false`); missing identity, layout or
topology changes, and hit-index mismatch must remain typed full fallbacks.

The same path has a deeper aggregate-level cost. `HostContractState` previously
retained `host_structure` and `host_presentation` as two `Arc` handles to the
coherent presentation, so ordinary `Arc::make_mut` always cloned the aggregate.
The first production candidate removes the bookkeeping Arc and uses the existing
`structure_generation` scalar as semantic identity. The state now owns one
presentation Arc and allocates no identity object per update; an outstanding
external generation continues to preserve its immutable snapshot through
required COW. The generic index check still visits every pane model and rebuilds
every pane hit product, but UI Asset scoped refresh no longer enters that path.
It prepares one validated pane transaction and rebinds only exact old/new paint
models before atomically publishing the pane payload and generations. The
source-bound artifact
`E:/zircon-profiles/editor-pane-presentation-transaction-pressure-20260902-r10/editor-pane-presentation-transaction-pressure.json`
(SHA-256 `48B358AC43552BDF41C99CE6BB510949065A474C33B7F149CEA969A55C938ABC`,
source-set SHA-256
`1A3A50074D45F411665212F0B4A3C463F333F7490D04688FF804C416971962AE`)
binds twenty-one current owners and six Unreal/Fyrox/Slint reference anchors. Its
64-pane, 2,048-node, 1,000 one-pane-update fixture records 64,000 aggregate pane
clones in the rejected dual-Arc baseline and zero internally forced clones in
the current no-outstanding-snapshot candidate. The generic fallback shape still
performs 64,000 paint-model identity visits and 131,072,000 hit-node rebuild
visits. The implemented scoped UI Asset transaction visits 2,048,000
changed-pane nodes and zero unchanged-pane nodes, achieving the modeled 64x
reduction for that path. Generic callers remain to migrate. The separate 600
geometry updates still expose 4,800 semantic pane clones. All values are
operation counts with `timing_claim=false`.

The viewport-toolbar projection now reuses the pane transaction. A stable
toolbar publication probes cached SurfaceFrame identities and returns without
advancing presentation generations or scanning paint models; a route change
clones only the affected pane. In the paired 1,000 stable/1,000 route-update
fixture, the rejected generic updater performs 128,000 paint-model identity
visits and 32,000 floating-row clones, while the current stable path performs
zero pane clones, generation increments, and paint-model scans. This is a
source-bound operation model, not a button/resize latency measurement.

Scene-hierarchy sparse refresh exposed the same ownership failure at a larger
scale. The bridge already produced exact changed control IDs and logical row
patches, but the publisher cloned `HostWindowPresentationData` and called
`set_host_presentation`; changing the workbench-node model identity then forced
the global paint-model check and complete workbench hit-index construction. The
new `HostPresentationPatch` combines a hierarchy pane patch with exact workbench
rows. State validates pane paint-model replacement and workbench index membership
before `Arc::make_mut`, then commits both and advances generations once. A failed
validation mutates neither side and retains the existing authoritative-reflow
fallback. The source-bound artifact
`E:/zircon-profiles/editor-hierarchy-sparse-publication-pressure-20260902-r2.json`
(SHA-256 `B68A46211848E1511BDE2A87A6063102E5C88BB3178D94AAB0565706F21F7934`,
source-set SHA-256
`F35698FB115E3F3710B5FF95A68CE50BCA962E26AD5395CA5EB9DE26156ED466`)
models 65,536 one-row updates over 32,768 workbench nodes and 64 paint models.
It removes 65,536 presentation snapshots/full hit-index builds and
2,147,483,648 modeled workbench-node visits; the conservative operation ratio is
456x. It excludes paint-model row indexing, redraw damage, CPU/GPU time,
allocation bytes, RSS, and input-to-present latency.

Native floating-window configuration had a second high-frequency publication
shape: it retained a `HostPresentationGeneration` snapshot while calling the
generic updater, forcing aggregate COW whenever bounds changed, then enumerated
all paint models to prove the unchanged workbench hit index. The typed metadata
transaction now performs its equality gate before `Arc::make_mut`, updates only
the native shell/surface identifiers, title, and bounds, advances structure and
geometry generations according to the changed domain, and preserves the existing
hit index. The 1,000-update
fixture removes 1,000 forced aggregate COW clones and 64,000 paint-model
identity visits; repeating equal metadata performs no commit. This remains an
operation model, not native-window resize latency evidence. The bounds-only
fixture also reduces structure-generation increments from 1,000 to zero while
retaining 1,000 geometry increments, so resize no longer advertises a semantic
structure change.

The remaining generic-publisher audit also found two overlay-only updates. The
close prompt and asset-deletion blocker are painted and routed directly from the
presentation before workbench hit-index dispatch, but their setters still
materialized and validated the complete paint-model directory. Typed overlay
publishers now retain the existing index while preserving structure/geometry
generations and regional redraw. The source-bound artifact
`E:/zircon-profiles/editor-overlay-presentation-pressure-20260902-r1/editor-overlay-presentation-pressure.json`
(SHA-256 `DE0EC96C8622835FAF34F59AC584ECBC967AD4B1669259F68F0E61BB68C10CCD`,
source-set SHA-256
`870BA921C73A6964BA9560B5D2806FD18E178DC4C04CA4E5D67D7519EEBEFC19`)
models 10,000 publications over 64 paint models: 10,000 temporary model
directories and 640,000 identity visits are removed. This is a low-frequency
P2 cleanup and not evidence that modal overlays caused the current pointer or
resize latency.

UI Asset scoped refresh had a separate event-path regression: it published new
template nodes and `body_surface_frame` without publishing a matching
`body_template_hit_index`. Pointer routing rejected the stale identity and
scanned all pane nodes for popup hits on every event. The current candidate uses
the common frame/index publication helper inside the same pane transaction and
does not re-enter the global hit-index builder. For 2,048 nodes, 1,000 refreshes,
100,000 pointer events, and four popup candidates, the rejected path performs
204,800,000 event-side visits. The current model performs 2,048,000
publication-side index visits and 400,000 event-side candidate visits, 2,448,000
total and an 83.66x operation-count reduction. Product latency remains open.

## Decision

The first pane-presentation production slice changes generation ownership:
semantic structure uses the existing `structure_generation` scalar instead of a
second full presentation Arc or a per-update identity allocation. This removes
guaranteed internal COW without creating a leaf cache or weakening external
immutable snapshots. A second scoped slice makes UI Asset publish pane, frame,
popup hit index, exact paint-model rebind, damage, and generations in one
transaction. Stable location plus pane id/kind provide the current identity;
floating rows retain persistent storage. Missing identity, topology/layout
change, empty/nonempty model cardinality change, ambiguous shared-model
merge/split, or rebind mismatch keeps a typed full fallback. The same transaction
is now the target for remaining generic pane publishers. This boundary matches Unreal's
invalidation-root/paint-space hit-grid ownership, Fyrox's retained validity
flags, and Slint's generation-owned per-item cache.

The layout-report aggregation slice is likewise bounded to the report owner: it
reuses the sorted fallback-reason vector instead of constructing a temporary
`BTreeMap` on every recompute. Its source-bound allocation model is recorded in
`E:\zircon-profiles\runtime-ui-layout-report-aggregation-pressure-20260829-r3.json`;
it changes allocation behavior without changing layout selection, fallback
ordering, or aggregate semantics.

The implemented slices with a measured authority boundary now also include the
retained menu-pointer Surface delta, alongside Asset Browser generation/index
lookup, pointer snapshot projection, native pointer transfer, incremental layout,
and image/SVG cache contracts.

## Next implementation gates

1. Validate the atomic sparse-hierarchy transaction, then continue migrating
   Runtime Diagnostics, Inspector, and remaining generic pane publishers through
   typed local transactions. Keep dock/document/floating lookup, exact old/new
   paint rebind, damage/generation atomicity, and typed full fallback as
   acceptance conditions.
2. For each dirty high-ranked path, identify the publication owner and prove
   call frequency with a source-bound product counter before editing.
3. Keep pane, renderer-data, inspector, menu, and table caches generation-owned;
   do not put a lazy cache inside a leaf converter or event handler. Product
   validation must prove the menu semantic Arc identities remain stable during
   WindowMetrics resize and topology fallback remains typed.
4. For paint commands, only pursue a change when a retained command/segment
   identity can be reused across frames. Small scalar copies are not sufficient
   evidence for an allocation or latency win.
5. Add a focused source contract and deterministic pressure model before each
   production change. Required fields are source HEAD, dirty paths, operation
   count, allocation/materialization model, and an explicit `timing_claim=false`
   until managed/product profiling is available.
6. Resume managed lower-layer and product validation only when a current-source
   Editor/Runtime product exists in a managed D/E/F target. The latest preflight
   found no such binary; this record does not authorize Cargo.
7. Renderer acceptance must distinguish a stable frame from a one-segment
   delta. Stable image frames require zero segment visits, texture dependency
   checks, binding lookups and binding-map retention scans. A one-segment delta
   requires unchanged-segment reuse conservation and dependency work bounded by
   the changed segment; no counter may silently report a frame-wide sweep as a
   cache hit.
8. Menu-pointer acceptance must report topology, geometry, patch-node, and
   fallback counts. A 200-step resize with stable menus requires zero topology
   rebuilds/registrations after initial publication; a stable recompute requires
   zero geometry publication; a menu/preset topology change must rebuild fail
   closed and preserve pointer route parity.

## Validation

The audit command completed with exit code 0 and wrote the JSON/CSV artifacts
listed in the front matter. The current artifact hashes are JSON
`A6D4D51F0FF0C18D0C3F50645E1873BD029CE5DFEB4D1EBF7B8744FA63730E33` and CSV
`7ACC280F467A55ACAB202049410DBC572D19E0F4055A4CD1EA4379EB01E21836`. The
artifact is intentionally outside the repository on `E:`. Static focused
contracts and rustfmt/diff checks for the previous implementation slices remain
the applicable evidence. The new render dependency-delta gate passes 9/9
focused Python contracts; tool SHA-256 is
`ADD9714B05E6E3C9C338C438DCE79F5B1A676D91DC48512F7447735F9955978F`
and test SHA-256 is
`F1E416426E77937D253CB8720799B494B546AD3C783C2460B6C5CDBC7EDC9DFB`.
The menu resize model contracts pass 6/6. Its artifact SHA-256 is
`26DFCE1CB5BBC9FDF9565B3A92E515FA8A38013B9F140DD657855121BEE61F0B`,
tool SHA-256 is
`708A4ED8297803301472E97686A2CA8ECCA224553E22BA2DDD587ED655198443`,
and test SHA-256 is
`1158C31AF7D7ADD4EECADA9702A01C83B864795814909FE8424E1F913DFB2F93`.
No Cargo, `rustc`, or managed validation was run for this review.

The Runtime Diagnostics and Performance Timeline source contracts pass alongside
the existing retained-host static suite (28/28 Python tests), scoped Rust formatting, and
`git diff --check`. The Rust frame-authority and fail-closed regressions are
present but remain Cargo-pending; no Cargo, `rustc`, or managed validation was
run because the shared index lock was still present.
