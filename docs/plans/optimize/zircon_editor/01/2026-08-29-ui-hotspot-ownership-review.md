---
related_code:
  - zircon_editor/src/ui
  - zircon_runtime/src/ui
  - zircon_runtime_interface/src/ui
source_artifact:
  - E:/zircon-profiles/ui-structural-hotspots-20260831-current-r7/ui-structural-hotspots.json
  - E:/zircon-profiles/ui-structural-hotspots-20260831-current-r7/ui-structural-hotspots.csv
  - E:/zircon-profiles/editor-menu-pointer-resize-pressure-20260831-r1.json
source_binding:
  head: ca3ac3cc6ad218d04a5cd469447cea2452441321
  dirty_path_count: 3440
status: static_review
---

# UI source hotspot ownership review

## Finding

The current source-bound inventory is a prioritization signal, not a runtime
profile. It scans 4,992 UI source files (430,228 lines) and records 4,910
`clone` calls, 1,028 vector materializations, 107 sorts, 6,336 string
allocation signals, and 2,553 traversal signals. Of those files, 2,485 are
currently dirty. The audit deliberately does not claim CPU, allocator, latency,
or GPU cost.

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

The retained-host menu pointer path exposes the same architectural error on the
resize side. `sync_recompute_pointer_surfaces` rebuilds a complete
`HostMenuPointerLayout`; any geometry difference reaches
`HostMenuPointerBridge::sync_shared`, which calls `rebuild_surface`. That method
constructs a new `UiSurface`, dispatcher, and route-intent map, recreates every
menu/popup path string, calls `surface.rebuild()`, and replaces all three
authorities. This is not on every ordinary move, but WindowMetrics recompute
makes it a repeated full-topology path while the window is resized.

The adjacent viewport-toolbar bridge already demonstrates the required local
contract: distinguish topology from geometry, mutate retained node frames, and
call `publish_authored_geometry` for the exact changed-node set. The equivalent
menu repair must retain `UiSurface`, dispatcher, route intents, and popup item
semantics when only frames change. Menu/model/preset changes remain topology or
semantic fallbacks. A stable layout performs no publication. The menu sources
are externally modified, so this review does not implement a competing patch.

`E:/zircon-profiles/editor-menu-pointer-resize-pressure-20260831-r1.json`
binds that finding to the current menu/toolbar sources and Unreal
`FSlateInvalidationRoot`. For 200 resize steps, seven buttons, one root popup,
two submenu levels, 40 popup items, and three changed geometry nodes, the
current shape models 200 surface builds, 2,200 dispatcher registrations, 2,200
route bindings/path-string builds, 8,000 popup item projections, and 9,600
node-domain visit units. Retained geometry publication performs none of the
topology/semantic work and uses 1,800 node-domain visit units, a 5.33x operation
ratio. These are deterministic units, not CPU or latency measurements.

## Decision

No production source was changed solely because of this inventory. This avoids
introducing a second cache or crossing active owner boundaries based only on
static counters. A separate low-risk implementation slice reserves exact-size
route output where the cardinality is already owned by the caller; it does not
alter the ranked ownership decisions below.

The layout-report aggregation slice is likewise bounded to the report owner: it
reuses the sorted fallback-reason vector instead of constructing a temporary
`BTreeMap` on every recompute. Its source-bound allocation model is recorded in
`E:\zircon-profiles\runtime-ui-layout-report-aggregation-pressure-20260829-r3.json`;
it changes allocation behavior without changing layout selection, fallback
ordering, or aggregate semantics.

The implemented slices remain the ones with a measured authority boundary:
Asset Browser generation/index lookup, pointer snapshot projection, native
pointer move transfer, incremental layout/source-frame work, and the existing
image/SVG cache contracts.

## Next implementation gates

1. For each dirty high-ranked path, identify the publication owner and prove
   call frequency with a source-bound product counter before editing.
2. Keep pane, renderer-data, inspector, menu, and table caches generation-owned;
   do not put a lazy cache inside a leaf converter or event handler. For the
   menu pointer surface, separate stable topology/semantic generations from the
   WindowMetrics geometry generation and publish exact changed frames.
3. For paint commands, only pursue a change when a retained command/segment
   identity can be reused across frames. Small scalar copies are not sufficient
   evidence for an allocation or latency win.
4. Add a focused source contract and deterministic pressure model before each
   production change. Required fields are source HEAD, dirty paths, operation
   count, allocation/materialization model, and an explicit `timing_claim=false`
   until managed/product profiling is available.
5. Resume managed lower-layer and product validation only when a current-source
   Editor/Runtime product exists in a managed D/E/F target. The latest preflight
   found no such binary; this record does not authorize Cargo.
6. Renderer acceptance must distinguish a stable frame from a one-segment
   delta. Stable image frames require zero segment visits, texture dependency
   checks, binding lookups and binding-map retention scans. A one-segment delta
   requires unchanged-segment reuse conservation and dependency work bounded by
   the changed segment; no counter may silently report a frame-wide sweep as a
   cache hit.
7. Menu-pointer acceptance must report topology, geometry, patch-node, and
   fallback counts. A 200-step resize with stable menus requires zero topology
   rebuilds/registrations after initial publication; a stable recompute requires
   zero geometry publication; a menu/preset topology change must rebuild fail
   closed and preserve pointer route parity.

## Validation

The audit command completed with exit code 0 and wrote the JSON/CSV artifacts
listed in the front matter. The current artifact hashes are JSON
`62D18ECB8769F900022EBE98DB0DB32A8DFB33767B5AF69A24F6C89A4C74607B` and CSV
`0E14E8E9972C61BC5FB21EAE995C425CCD3D6BD9107659D587A7590EE0D61757`. The
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
