---
title: Editor retained viewport toolbar pointer command generation performance review
date: 2026-08-23
module: zircon_editor retained-host viewport_toolbar_pointer and click dispatch
priority: MVP-P0 scene viewport toolbar input correctness and cost
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine SEditorViewport command list and Slate FHittestGrid
---

# Goal

Make the MVP scene viewport toolbar consume one stable hit artifact and one typed command identity.
A toolbar click must not rebuild a second retained surface, classify the same string repeatedly, or
build the complete editor chrome snapshot merely to read viewport settings.

## Reviewed source

- pre-M0 owner Rust files: 31/31
- pre-M0 physical lines: 892
- pre-M0 bytes: 31,225
- pre-M0 LF path-tab-file-SHA manifest SHA256:
  `f723115e2eb47145c18bff46d4593212586a340db66fa1fd885b8ebc4fb9b149`
- post-M0 owner Rust files: 21/21
- post-M0 physical lines: 899
- post-M0 bytes: 33,064
- post-M0 LF path-tab-file-SHA manifest SHA256:
  `c1a531532d5c0a40f7e829949e052d99419097da9c09e4ff9f77b162c6035e1a`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

All owner files under `zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/**` were read in
full. The review also traced the native pane hit route, host callback wiring and click entry, shared
pointer callback, template binding bridge, surface-frame cache and producer, viewport route mapping,
`EditorHostEventController::chrome_snapshot`, `EditorChromeSnapshot` construction, and runtime
`UiSurfaceFrame` publication. The July 17 and July 30 viewport-toolbar reviews were reread; their
cache work is present, but their pointer-generation acceptance is not complete.

## Correct foundations to retain

1. The template projection cache now returns the same `Arc<UiSurfaceFrame>` for an unchanged
   surface/signature and reprojects route-only changes without relayout.
2. Native pane routing already hit-tests the published `UiSurfaceFrame::hit_grid`.
3. The pointer bridge preserves previously measured controls and skips a same-control/same-frame
   rebuild.
4. Routes are typed before dispatch; runtime bindings remain the command execution boundary.

## Structural findings

### P0 correctness: local frame generations collide across newly built surfaces

`ViewportToolbarSurfaceFrameCache::build_surface_frame` creates a new `UiSurface` for every cache
miss/reprojection. `UiSurfaceFramePublication::default` starts at generation 0 and the first
`surface_frame()` publication increments it to 1. Different route projections for the same
`surface_key` therefore commonly have the same `tree_id`, generation 1 and origin.

`ViewportToolbarPointerBridge::sync_surface_frame` currently treats only `(generation, origin)` as
the applied cursor. A route-only projection change can consequently be rejected as already applied,
leaving stale control ids and stale routes. This is a source-proven generation-domain collision, not
an elapsed-time inference.

M0 accepts the producer's immutable `Arc<UiSurfaceFrame>` and keys reuse by allocation identity plus
origin. A stored `Weak` preserves identity without retaining a complete old frame. M1 replaces this
bridge-local receipt with a typed producer-owned surface generation/handle.

### P0: one native click traverses two retained hit representations

Native pane routing first calls `hit_test_surface_frame` on the published toolbar hit grid. The
callback then drops the matched control id, sends only point/size, and the pointer bridge scans the
frame's entire arranged tree, copies controls, builds another `UiTree`, dispatcher and route map,
calls full `UiSurface::rebuild`, and hit-tests the point again.

M0 at least consumes the authoritative `hit_grid.entries` rather than all arranged nodes. M1 carries
the native hit's stable node/control handle through the callback. M2 deletes the editor-only mirror
surface and binds typed commands to the producer-owned surface topology.

### P0: validation materializes and discards owned routes

`route_for_control` eagerly constructs an array containing all ten classifier calls. Every known
control is checked against all ten domains. Validation in `handle_click` and `sync_surface_frame`
then allocates owned `surface_key`/payload Strings for the matching route and discards the route;
surface rebuild classifies and allocates it again.

M0 replaces the ten tiny classifier modules with one allocation-free borrowed command descriptor
match. Validation uses the descriptor; an owned route is materialized only when binding a retained
route node. This changes known-control source classification from ten domain calls to one match and
validation route allocations from one owned route to zero.

### P0: every routed click builds a complete editor chrome snapshot

`dispatch_viewport_toolbar_pointer_route` unconditionally calls `runtime.chrome_snapshot()`. That
path locks the editor shell, clones descriptors, projects the complete workbench, scene entries,
inspector, console, asset workspaces, project state and other chrome even for `FrameSelection`, play
mode, mode, projection, alignment and transform-space commands. Only eight of fifteen route variants
need the small `SceneViewportChromeSettings` value.

M0 adds a direct settings accessor and calls it only in the eight cycle/toggle arms. Full chrome
snapshot builds on every toolbar route become zero; seven route variants take no settings snapshot.

### P1: stable clicks allocate a one-surface layout before equality rejection

The host click entry builds a `Vec<ViewportToolbarPointerSurface>` and clones `surface_key` before
`sync` can reject the same layout. M0 adds a single-surface sync entry that compares borrowed key and
frame first, allocating only on change. Switching among panes still replaces the bridge's complete
surface set and triggers a rebuild; M1 uses a generation-owned per-surface index instead.

### P1: full mirror topology rebuild remains the dominant changed-frame cost

Any layout/control change reconstructs root/surface/control paths, dispatcher closures, typed route
payloads, route bindings, arranged geometry, hit grid and render extraction. Runtime already exposes
incremental dirty rebuilds, but this bridge throws away the entire surface and cannot update one
toolbar suffix atomically. Moving the rebuild to a worker would only queue obsolete input topology;
the correct fix is one producer-owned retained surface and changed-node topology transactions.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/SEditorViewport.cpp`
  - `Construct` creates one `FUICommandList`, calls `BindCommands` once, and builds the viewport and
    toolbar widgets once.
  - `BindCommands` maps stable `FEditorViewportCommands` identities to viewport-client actions;
    input processes those bindings rather than parsing and rebuilding owned route strings.
- `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/SCommonEditorViewportToolbarBase.cpp`
  - toolbar construction receives `ViewportRef->GetCommandList()` and appends the same command list
    to tool-menu contexts.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h` and
  `Private/Input/HittestGrid.cpp`
  - `FHittestGrid` owns one widget identity map and explicit add/remove/update operations.
  - `AddWidget` updates sort metadata in place when cell coverage is unchanged; it does not rebuild a
    second toolbar-specific hit tree for each click.

The transferable architecture is stable widget identity + stable command identity + one retained
hit owner. Zircon's typed Rust routes can differ in representation, but the ownership model should
not.

## Target architecture

1. The template/runtime surface owns arranged, hit, route and binding topology under one generation.
2. Native routing returns a stable surface/node/command receipt; the host callback does not discard
   it and repeat hit testing.
3. Command descriptors use typed compact ids and static/borrowed payloads; owned Strings appear only
   at external binding boundaries.
4. Surface add/remove/resize/route changes update only affected nodes and reject stale receipts.
5. Viewport settings are read through a narrow snapshot; unrelated workbench projection never runs
   on the input hot path.

## Instrumentation and acceptance

Matrix: surfaces `1/4/16/64`; controls `16/64/256`; click rate `10/125/500 Hz`; state
`stable/route-only/resize/add/remove`; command `no-settings/settings/custom/unknown`; frame producer
`same Arc/new Arc same generation/new generation`; floating windows `0/4/16`.

Acceptance requires:

- same-Arc stable sync scans 0 nodes and rebuilds 0 surfaces;
- different-Arc/same-local-generation frames are never rejected as stale duplicates;
- validation owned route allocations `1 -> 0`, and one control is classified once;
- full `EditorChromeSnapshot` builds per toolbar route `1 -> 0`;
- stable single-surface layout String/Vec allocations `1 each -> 0`;
- after M2, one click performs one hit test and changed topology visits only changed nodes;
- p95 toolbar input routing below 0.05 ms at 64 controls and below 0.10 ms at 256 controls on the
  recorded host, with zero sustained allocator growth;
- WPR shows no repeated chrome projection/rebuild wakeups; behavior and pixel output match.

RenderDoc is reserved for current-source GPU/draw/pixel parity after a launchable executable exists;
it cannot prove CPU route/allocation cost. WPR, allocator, build and capture artifacts stay on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Correct frame identity, use hit entries, centralize borrowed command parsing, narrow settings access, skip stable layout allocation. | applied; focused RED 0/5 to GREEN 5/5 |
| M1 | Carry native stable hit/command receipts and retain per-surface generations. | one hit query, zero mirror scan on click |
| M2 | Delete the mirror pointer surface; atomically update producer-owned node/handler/route topology. | changed-node-only topology reports |
| M3 | Run scale/storm/WPR/power plus RenderDoc behavior/pixel parity. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 31/31 current Rust files.
- Producer, cache, native route, host callback, binding, command dispatch and runtime publication
  chain: read and mapped.
- Unreal viewport command-list and Slate hit-grid implementations: read and mapped.
- M0 implementation: applied. The frame cursor now holds `Weak<UiSurfaceFrame>` identity plus
  origin, frame projection reads `hit_grid.entries`, and a different Arc with the same local
  generation is accepted. The callback passes the immutable Arc through rather than erasing
  producer identity.
- Ten classifier modules were deleted after their behavior was consolidated into one borrowed
  command descriptor. All 34 legacy control ids/aliases plus custom and invalid cases have Rust
  unit contracts. Owned routes are created only for retained route binding.
- The stable product click compares borrowed surface key/frame before allocating a layout. Route
  dispatch now reads `SceneViewportChromeSettings` directly and only in the eight settings-dependent
  arms; all fifteen route variants avoid complete chrome projection.
- Exact static path delta: classifier files `10 -> 0`, eager classifier arrays `1 -> 0`, validation
  owned-route materializations `1 per checked control -> 0`, arranged-tree scans in frame sync
  `1 -> 0`, authoritative hit-entry scans `0 -> 1`, full chrome snapshot calls per routed click
  `1 -> 0`, and stable single-surface layout String/Vec allocations `1 each -> 0`.
- Focused static contract:
  `tools/tests/test_editor_retained_viewport_toolbar_pointer_generation_performance_contract.py`,
  95 lines, 3,975 bytes, SHA256
  `a7dfb8482f23658fd357a63bf692979f2f9c91c96977bda688220c6305526014`; RED 0/5 to GREEN 5/5.
- All retained-host performance contracts: GREEN 13/13. Broad performance-contract discovery:
  176/181 passed. The same five external test-fixture/`available_slots`/asset root-clone failures
  remain; no viewport-toolbar contract regressed.
- Rustfmt parsing and scoped `git diff --check`: passed. Owner file count decreased `31 -> 21`;
  physical lines increased `892 -> 899` and bytes `31,225 -> 33,064` because the consolidated
  parser now embeds explicit behavior coverage rather than ten untested classifier files.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- Rust unit execution, M1-M3 and dynamic evidence remain pending; this owner must stay in
  `pending.md`.
