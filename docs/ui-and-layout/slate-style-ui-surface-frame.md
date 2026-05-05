---
related_code:
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/ui/tree/node/visibility.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/tree/node/render_order.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/template/build/layout_contract.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/redraw.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
  - zircon_editor/src/ui/slint_host/app/invalidation.rs
  - zircon_editor/src/ui/slint_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/route_for_control.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/cycle_display_mode_route.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/cycle_grid_mode_route.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/snap_routes.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/toggle_routes.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/frame_selection_route.rs
implementation_files:
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/ui/tree/node/visibility.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/tree/node/render_order.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/template/build/layout_contract.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/redraw.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/invalidation.rs
  - zircon_editor/src/ui/slint_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/cycle_display_mode_route.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/cycle_grid_mode_route.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/snap_routes.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/toggle_routes.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/frame_selection_route.rs
plan_sources:
  - Shared Slate-Style UI Layout, Render, And Hit Framework
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedWidget.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedChildren.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Visibility.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Events.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/SViewport.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Components/WidgetComponent.cpp
  - dev/bevy/examples/ui/render_ui_to_texture.rs
  - dev/bevy/examples/ui/widgets/viewport_node.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - .codex/plans/Editor 绘制与鼠标事件优化计划.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - user: 2026-05-06 Zircon UI 与 Unreal Slate 差异审计及后续里程碑
  - user: 2026-05-06 完善命中测试，参照 dev 下虚幻源码
tests:
  - cargo test --manifest-path E:\Git\ZirconEngine\Cargo.toml -p zircon_runtime_interface ui_surface_frame_contract_carries_arranged_render_and_hit_state --locked --target-dir E:\zircon-build\targets --jobs 1
  - cargo test -p zircon_runtime_interface --lib ui_visibility_contract_separates_layout_render_and_hit_policy --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime_interface --lib ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test --manifest-path E:\Git\ZirconEngine\Cargo.toml -p zircon_runtime surface_rebuild_derives_render_and_hit_from_same_arranged_geometry --locked --target-dir E:\zircon-build\targets --jobs 1
  - cargo test -p zircon_runtime --lib legacy_visible_false_is_normalized_into_hidden_visibility_for_surface_outputs --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib focus_navigation_and_scroll_candidates_use_effective_visibility --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib hit_grid_respects_slate_visibility_and_clip_semantics --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib surface_rebuild_derives_render_and_hit_from_same_arranged_geometry --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib hit_grid_omits_disabled_nodes_and_debug_dump_reports_why --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib scrollable_virtualized_children_enter_hit_grid_only_when_arranged_visible --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib hit_grid_uses_cursor_radius_as_slate_style_nearby_hit_fallback --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib exact_hit_wins_over_nearby_cursor_radius_candidates --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib virtual_pointer_query_maps_custom_3d_hits_into_surface_local_hit_path --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib pointer_route_can_use_virtual_pointer_hit_from_custom_surface_mapper --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib pointer_dispatch_uses_virtual_pointer_query_for_component_events --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib surface_dirty_rebuild_keeps_render_only_changes_out_of_layout --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test --manifest-path E:\Git\ZirconEngine\Cargo.toml -p zircon_editor native_host_viewport_toolbar_only_dispatches_primary_press --locked --target-dir E:\zircon-build\targets --jobs 1
  - cargo test -p zircon_editor --lib apply_presentation_resolves_splitters_from_shared_visible_drawer_projection --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_editor --lib viewport_toolbar_surface_frame_includes_projected_route_controls_without_action_list --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_viewport_toolbar_buttons_before_viewport_body --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/projection_fallback.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - zircon_editor/src/tests/host/slint_window/native_viewport_image.rs
  - zircon_editor/src/tests/host/slint_window/shell_window.rs
  - cargo test -p zircon_editor --lib native_host_painter_draws_template_svg_image_pixels --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib rust_owned_host_painter_resolves_runtime_svg_image_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib host_contract::redraw::tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib host_invalidation --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib builtin_template_compile_cache_is_reused_across_runtime_instances --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib tests::host::slint_window::native_host_contract --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - tests/acceptance/shared-slate-ui-surface-frame.md
doc_type: module-detail
---

# Slate-Style UI Surface Frame

`UiSurfaceFrame` is the shared surface snapshot for editor and runtime UI. It carries one arranged tree, one render extract, one hit grid, and the current focus/capture/hover state. The arranged tree is the only spatial authority: render extraction and hit testing both consume `UiArrangedNode.frame` and `UiArrangedNode.clip_frame`.

The design follows Unreal Slate's `FArrangedWidget`/`FArrangedChildren` and `FHittestGrid` split. Layout produces arranged widgets; painting and hit grid insertion consume that arranged geometry; hit paths are reported as leaf-to-root bubble routes plus root-to-leaf paths.

## Visibility

`UiVisibility` replaces boolean-only visibility decisions for render and hit behavior:

- `Visible`: occupies layout, renders, self can be hit, children can be hit.
- `Hidden`: occupies layout, does not render, and blocks hit testing for self and children.
- `Collapsed`: does not occupy layout, does not render, and blocks hit testing.
- `HitTestInvisible`: renders but blocks hit testing for self and children.
- `SelfHitTestInvisible`: renders, skips self hit testing, and preserves child hit testing.

Legacy `state_flags.visible == false` is treated as effective `Hidden` before a node reaches arranged output unless the authored visibility is explicitly `Collapsed`, which keeps its layout-collapse semantics. New code should set `UiVisibility` explicitly, but the transitional bool is still normalized by `UiVisibility::effective(...)` and `UiTreeNode::effective_visibility()` so layout, render, focus, scroll, pointer, and hit-grid code cannot reinterpret the same node differently.

The shared helpers define the only allowed predicates for node participation:

- `UiTreeNode::is_render_visible()` and `UiArrangedNode::is_render_visible()` decide render extract inclusion.
- `UiTreeNode::is_focus_candidate()` keeps focus/navigation on enabled render-visible nodes, so `HitTestInvisible` controls can still take keyboard focus when authored as focusable.
- `UiTreeNode::supports_pointer()` and `UiTreeNode::allows_child_hit_test()` split self hit-test from descendant hit-test instead of overloading `state_flags.visible`.
- `UiStateFlags::visible_enabled()` remains only a legacy convenience for places that need the historical bool pair, not a policy source for layout/render/hit.

## Runtime Flow

`UiSurface::compute_layout()` runs layout, then `UiSurface::rebuild()` derives:

1. `UiArrangedTree` from `UiTree` layout cache, clip chain, z index, paint order, input policy, and control metadata.
2. `UiRenderExtract` from the arranged tree, not from a separate coordinate walk.
3. `UiHitTestGrid` from the arranged tree, filtered by visibility, enabled state, input policy, clip frame, and z/paint order.

Linear layout treats `Collapsed` as non-participating, matching Slate's collapsed semantics: collapsed children do not consume main-axis extent and do not create gaps between neighboring visible children. Template-authored `stretch = "Stretch"` axes are recorded on `UiTreeNode` so a node that explicitly asks to fill remaining linear space is not mistaken for an implicit content-sized leaf.

The hit grid stores spatial cells and entries sorted by paint priority. Querying a point through `hit_test_surface_frame(...)` returns `UiHitTestResult` with the top node, front-to-back stack, and `UiHitPath`. Editor adapters should use this runtime helper for submitted frames instead of rebuilding a local hit index around each host control family.

`UiHitTestQuery` extends the plain point query for the Slate custom-hit path slice. `cursor_radius` mirrors Unreal `FHittestGrid::GetBubblePath(... CursorRadius ...)`: exact point hits are considered first, and radius-only candidates are a fallback ordered by distance while still respecting z/paint order inside each class. `virtual_pointer` mirrors Unreal `FVirtualPointerPosition` and UMG `FWidget3DHitTester`: a host-side mapper or future 3D raycast backend converts a screen/world hit into surface-local current/previous coordinates, then the shared hit grid resolves the normal `UiHitPath`. This keeps custom and world-space UI from inventing a separate dispatch path; they supply a mapped query and still consume `UiSurfaceFrame.arranged_tree + hit_grid`.

The first accepted custom/3D slice is intentionally a boundary contract, not a full raycaster. Zircon owns the surface-local DTO and route behavior; runtime rendering, physics, or editor viewport systems own the ray/UV/world-to-local mapping that produces `UiVirtualPointerPosition`. This matches the evidence split in Unreal, where `SViewport` registers an `ICustomHitTestPath`, `FWidget3DHitTester` maps scene hits into widget-local virtual cursor positions, and `UWidgetComponent::GetHitWidgetPath(...)` still delegates final widget path construction to the widget component's `FHittestGrid`. Bevy's render-to-texture UI example uses the same architectural split by raycasting into a texture, converting UV to 2D UI coordinates, and emitting virtual pointer input to the normal UI picking path.

`UiSurface::rebuild_dirty(root_size)` is the shared invalidation entry point for retained surfaces. `layout`, `style`, `text`, and `visible_range` dirtiness recompute layout and then regenerate arranged/render/hit outputs together. `hit_test` and `input` dirtiness rebuild only the arranged tree and hit grid. `render` dirtiness regenerates only render extract from the existing arranged tree. The legacy `UiStateFlags::dirty` bool is normalized as render, hit-test, and input dirtiness, then cleared with the structured dirty flags after the rebuild.

Pointer routing carries the same hit result forward as `UiPointerRoute.hit_path`. `bubbled` remains the direct leaf-to-root dispatch route, while `hit_path.root_to_leaf` is available for Slate-style enter/leave, focus-path, and capture diagnostics without reconstructing ancestry from the tree after the hit query.

Milestone 1 currently accepts the effective visibility slice rather than the full editor-native cutover. The runtime arranged-tree builder writes the effective visibility into `UiArrangedNode`, render extract and hit grid consume that arranged output, and the retained-tree focus, scroll, pointer, and render-order helpers call the same shared predicates. This specifically prevents a drawer, toolbar, or pane control from being render-visible in one pass and hit-visible in another because a local path read `state_flags.visible` directly.

The focused regressions for this slice cover three boundaries: the interface contract separates `Hidden`, `Collapsed`, `HitTestInvisible`, and `SelfHitTestInvisible`; runtime surface outputs normalize legacy `visible=false` into `Hidden`; focus/navigation and scroll candidates use the same effective visibility helpers as render and hit testing.

## Surface Diagnostics And Reflector Baseline

`debug_surface_frame(...)` is the shared debug entry point for the Widget Reflector-style milestone. It consumes only `UiSurfaceFrame`, so editor and runtime debug tooling cannot drift into separate coordinate systems. The snapshot contains reflected arranged nodes, render counters, material batch groups, hit-grid occupancy, overdraw samples, and the focus/capture/hover state that was current when the frame was submitted.

`UiWidgetReflectorNode` mirrors the arranged geometry rather than the authoring tree alone. Each reflected node carries frame, clip frame, parent/children, z/paint order, visibility, input policy, state flags, control id, render command count, hit entry count, and hit cell count. This is the minimum data needed to build a live/snapshot tree like Unreal's Widget Reflector while still preserving Zircon's `.ui.toml` retained-tree source model.

Render diagnostics are intentionally named as estimates until the renderer exposes backend-confirmed counters. `UiRenderDebugStats` groups commands by a stable material signature and reports estimated draw calls from geometry and text-producing commands. The material batch list is already useful for finding batch breaks caused by style, image, font, text render mode, opacity, and clip-heavy command groups; the runtime WGPU pass can later replace the estimate with real submitted pass counters without changing the snapshot boundary.

Overdraw diagnostics use a configurable sample grid over the visible render-command union. They report covered cells, overdrawn cells, max layers, and total layer samples. This is not a replacement for a GPU overdraw pass, but it gives the editor debug UI a deterministic CPU-side overlay source while the material batching and runtime renderer instrumentation mature.

## Editor Host Route

The native Slint host stores a toolbar `UiSurfaceFrame` in `SceneViewportChromeData`. That frame is built by iterating route-bearing projected controls from the `.ui.toml` host projection in `BuiltinViewportToolbarTemplateBridge`, so button hit rectangles match the component layout. Adding another toolbar button with a projected control id and binding makes it enter the surface frame without adding Rust coordinate rows or a toolbar action list. Root docks and floating-window active panes receive these frames before native pointer routing runs. `host_contract/surface_hit_test` calls the shared `hit_test_surface_frame(...)` helper and maps hit node `control_id` values to the existing toolbar and pane dispatch callbacks.

Toolbar hit control ids are separated from projected control ids only where the current editor state supplies a parameterized action or an existing semantic alias must be preserved. For example, the projected `SetTool` button maps to the current `tool.move`/`tool.rotate` action key, `FrameSelection` keeps the existing `frame.selection` semantic id, and direct no-argument buttons such as play-mode controls can use their projected control id. This keeps hit geometry owned by layout while preserving the existing viewport command semantics.

Projection control ids now fall back through template bindings instead of re-entering the old toolbar alias table. The legacy pointer-route helpers recognize hit-grid action ids such as `display.cycle`, `snap.translate`, and `frame.selection`; TOML projection ids such as `SetDisplayMode` and `FrameSelection` are resolved by `dispatch_builtin_viewport_toolbar_control(...)` when no legacy route exists. This is the hard-cutover boundary that let the old toolbar alias list be deleted without keeping a compatibility shim.

Some real-host callback paths can still report a zero control rectangle while carrying an already-resolved hit-grid action id. In that case `dispatch_shared_viewport_toolbar_pointer_click(...)` first tries the projected template frame for TOML control ids, then uses the actual click point as a one-pixel active frame for legacy action ids that have no projected control frame. That fallback is not a coordinate table: it exists only so the existing shared `ViewportToolbarPointerBridge` can route the already-known action id through its retained `UiSurface + UiPointerDispatcher` path instead of failing before dispatch. Projection ids with no legacy route still go through template bindings.

Native template-node hit testing also routes through `PaneData.body_surface_frame`, which is built during host presentation conversion from projected template node frames. This keeps pane controls on the same arranged/render/hit model as toolbar controls: native pointer dispatch queries a submitted frame rather than rebuilding a local coordinate model at click time.

Pane component projection keeps a separate fallback label heuristic for bound or button-like controls that have no authored text. That helper only derives visible text such as `Apply Draft`; it does not define a toolbar action-control alias or a dispatch compatibility table.

The native pointer dispatch still gates toolbar activation to primary press only. Release, secondary, and middle button events do not dispatch toolbar commands.

## Editor Native Fast Path

The native host now has the first Slate-style fast path for editor repaint pressure. `SoftbufferHostPresenter` retains a `HostRgbaFrame` backbuffer. A full paint is still required for the first frame, resize, and full invalidation; damage redraws repaint only the clipped region into that retained frame and copy only the damaged pixel rows into softbuffer before calling `present_with_damage`. The painter enforces this through an active `HostRgbaFrame` paint clip, so root skeleton, template nodes, text, viewport images, and overlay primitives all consume the same damage rectangle instead of relying on each draw call to remember a local clip.

Pointer move routing also avoids repaint when the native pixels did not change. Viewport mouse moves still dispatch to the runtime pointer bridge, but they return idle to the native presenter; the host repaints when a new viewport image arrives. Hierarchy hover compares the previous and current pointer-only pane state and damages only the affected row union. Repeating the same hierarchy or asset-tree hover target returns idle, which prevents high-frequency mouse motion from turning into repeated host presentation rebuilds or full-frame paints.

Viewport image updates now use the same paint-only channel as pointer damage. `SlintEditorHost::poll_viewport_image_for_native_host()` accepts a fresh viewport image into `HostViewportImageData`, then queues an external `HostRedrawRequest::Region` for the current viewport content frame. The native event loop drains and coalesces those external redraw requests in `about_to_wait`, so multiple image/damage requests collapse before softbuffer presentation. This path does not set `presentation_dirty`, `layout_dirty`, or `render_dirty`, and a drained region redraw does not invoke `request_frame_update()`.

Repeated status messages are guarded before presentation invalidation. `EditorEventRuntime::status_line()` exposes the current status without constructing a full chrome snapshot; `SlintEditorHost::set_status_line()` now returns when the message is unchanged. This keeps recurring background errors from causing one presentation rebuild per timer tick while preserving normal status-line refresh when the text actually changes.

Builtin template loading is cached at the compiled-document and parsed-document level. The cache key includes the canonical path, modification timestamp, and file length, so multiple bridge/runtime instances can reuse the same builtin `.ui.toml` documents during one editor process without hiding file changes across process restarts. Diagnostic logs now distinguish cache hits from actual template loads, and host presentation logs include a rebuild count to make accidental full projection loops visible.

`apply_presentation` now projects only the active pane payload for each pane kind before building that pane's `body_surface_frame`. Scene panes do not build hierarchy/inspector/console/module/export payloads, and non-project panes do not project project overview data. This keeps presentation conversion aligned with the current visible pane role instead of rebuilding every possible pane body variant on each refresh.

`SlintEditorHost` also has a host-level invalidation root modeled after Unreal Slate's `FSlateInvalidationRoot`. The root records structured reasons instead of treating every change as the same dirty flag: layout, tree structure, presentation data, paint-only, pointer hover, viewport image, hit-test, window metrics, and render. Layout/window-metrics reasons still drive the current compatibility `layout_dirty`/`presentation_dirty` slow path, render reasons stay separate from presentation rebuilds, and viewport-image updates are counted as paint-only damage before queuing a regional redraw. The diagnostic channel `editor_host_invalidation` logs slow-path and render-path counts with the merged reasons, so repeated full refreshes can be traced to the source that requested them.

This invalidation root is intentionally a cutover layer, not a second layout system. Existing legacy dirty assignments can still force a slow path; when that happens the recompute log falls back to the observed legacy dirty flags. New editor host code should call `invalidate_host(...)` or `record_paint_only_invalidation(...)` so the reason survives coalescing and can later map directly onto retained arranged trees, hit grids, and cached paint output.

Preview image loading for template-projected nodes now lives in `ui/layouts/views/preview_images.rs` instead of a private Slint host adapter. This keeps layout-level `ViewTemplateNodeData` projection independent of native host conversion and lets both material-style template projection and host conversion share the same icon/media lookup behavior.

The retained painter fast path now also covers visual assets. Template-projected `Image`, `Icon`, and `SvgIcon` nodes carry loaded `preview_image` pixels into `TemplatePaneNodeData`; the native painter converts those pixels once per paint command and draws them through the same clipped `HostRgbaFrame` primitive as viewport images. Runtime-style `UiRenderCommandKind::Image` resolves `UiVisualAssetRef::Image` / `Icon` to cached decoded pixels before falling back to deterministic placeholders. Because these draw calls all use `HostRgbaFrame`'s active paint clip, local damage still limits the work to the dirty region and does not force a presentation rebuild just because an image command is present.

The top-right debug readout is likewise part of the native shell paint path, not a Slint UI replacement. `HostWindowShellData.debug_refresh_rate` is projected into the host contract and painted inside the top-bar clip in `workbench.rs`. The startup fallback remains `FPS 0.0 | present 0 | full 0 | region 0`, but after the first native present the text is produced from `HostRefreshDiagnostics` plus `HostInvalidationRoot::diagnostics_snapshot()`.

That live overlay makes the retained-damage contract visible in the shell itself. Presenter counters report full paints, region paints, total presents, and total painted pixels, while invalidation counters report slow-path rebuilds, render-path rebuilds, and paint-only requests. Paint-only invalidations still do not set presentation or layout dirty flags; they update the invalidation snapshot and continue through region redraw where a caller provides damage. The overlay therefore observes the existing damage behavior without introducing a new coordinate table or a screen-specific Asset Browser branch.
