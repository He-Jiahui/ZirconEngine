# Editor Template Viewport Scene Static Review

- Date: 2026-07-17
- Scope: `paint_template_nodes/{template_viewport_scene*.rs,template_viewport_scene/**,template_viewport_scene_architecture/**,template_viewport_scene_floor/**,template_viewport_scene_gizmos/**,template_viewport_scene_light/**,template_viewport_scene_props/**,template_viewport_scene_surfaces/**,template_viewport_scene_tests/**}`
- Rust files read: 105/105
- Lines read: 3,100
- Acceptance state: `static_complete_dynamic_pending`
- Plan item: `PERF-MVP-217`
- Fixing plan: `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

## Files reviewed

| module | files | result |
|---|---:|---|
| entry, dispatch, identity/classification | 22 | every candidate runs a chain of lighting/gizmo/surface/floor/prop/architecture substring classifiers; unknown prefixed nodes fall back to a styled scene layer |
| surfaces and floor | 26 | many fixed decorative quads; floor grate emits two commands per eight pixels of width |
| architecture and props | 32 | many fixed facets plus cargo/rack loops whose command count grows with frame width/height |
| lighting and gizmos | 17 | bounded per node, but all layers rebuild with repeated style color/radius resolution |
| tests | 8 | representative pixels and container exclusions exist; no live-frame skip, stable-generation, command, theme, or scale budgets |

The matching Workbench ZUI declares 89 `WorkbenchViewport*` controls. Seven are chrome, two are layout containers, and the remainder form a dense decorative fallback scene beneath the viewport surface.

## Bottleneck evidence

Every candidate control runs `viewport_scene_kind`, which executes up to seven classifier functions with multiple `contains` probes. Most leaf painters then resolve style surface/border/radius independently and allocate a new host-command segment. The back wall, floor, ceiling, props, lighting, and gizmos add many layered quads. Floor grate, rack, and cargo details loop by pixel spacing, so host-command count grows with viewport dimensions.

This work is static for a stable layout/theme. More importantly, the actual editor viewport publishes an image separately. If that live image fully covers the surface, rebuilding the 80-ish decorative scene nodes underneath is entirely redundant. Existing source does not express a live-frame/fallback mode at this painter boundary, so current-source product counters must prove whether both are emitted.

## Reference-engine direction

Unreal `SViewport::OnPaint` submits one viewport draw element when a render-target texture exists and a simple box when it does not. Godot's 3D editor uses a `SubViewportContainer` plus `SubViewport` and redraws overlays on explicit state changes. Neither reconstructs a many-node decorative UI scene underneath every live viewport frame.

EditorUI08 should publish a typed live/fallback mode. A live frame skips the fallback subtree and submits the viewport texture handle plus typed overlays. A no-frame fallback is compiled or rasterized once per layout/theme generation and reused through damage tracking.

## Current-source direct mitigation

The componentized renderer's missing-layout fallback previously painted the entire Workbench node model even when a valid live viewport image was present. It now installs a transform that filters typed decorative viewport scene kinds before command construction while retaining toolbar, selection edge, axis, gizmo, and unrelated nodes. Missing or invalid images keep the complete fallback. Focused tests lock those mode and overlay rules. This removes decorative command/style work from that exceptional live path, but it still visits and classifies candidate nodes and therefore does not replace EditorUI08's required generation-owned typed mode.

## Dynamic acceptance still required

- Current-source counters for all 89 controls: nodes visited, classifiers/probes, style/theme reads, host/compiled/RHI commands, allocations, and CPU p50/p95/p99.
- Compare startup/no-frame, first live frame, 300 live frames, stable fallback, resize, stale/error, and device-loss recovery.
- Verify the landed missing-layout live filter has zero decorative builds/commands and preserves overlays; then prove the final typed live mode also reduces fallback visits/classification to zero and stable fallback generation performs zero rebuild work.
- Run current-source `zircon_editor --lib performance_tests`, viewport pixel/hit suites, Softbuffer, and RenderDoc; preserve toolbar, selection/gizmo, hit exclusions, z-order, clip, and recovery pixels before moving the folder to `review.md`.
