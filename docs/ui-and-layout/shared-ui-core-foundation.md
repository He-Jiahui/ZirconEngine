---
related_code:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/layout/mod.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/layout/virtualization.rs
  - zircon_ui/src/dispatch/mod.rs
  - zircon_ui/src/tree/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_ui/src/template/document.rs
  - zircon_ui/src/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_ui/src/tests/asset.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/scene/viewport/controller/mod.rs
  - zircon_editor/src/scene/viewport/pointer/mod.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/slint_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/slint_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - zircon_editor/src/ui/slint_host/app/assets.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/route_mapping.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/snap_cycle.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/projection_support.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/source_frames.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/surface.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/host_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/action_control.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/resolution.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/tests.rs
  - zircon_editor/src/ui/slint_host/detail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/scroll_surface_host.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_popup_state.rs
- zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/activity_rail_pointer/build_workbench_activity_rail_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/host_page_pointer/mod.rs
- zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs
- zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/floating_windows.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/src/core/editing/ui_asset/session.rs
  - zircon_editor/src/core/editing/ui_asset/inspector_semantics.rs
  - zircon_editor/src/core/editing/ui_asset/tree_editing.rs
  - zircon_editor/src/tests/editing/ui_asset/
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/mod.rs
  - zircon_editor/src/tests/host/slint_detail_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/mod.rs
- zircon_editor/src/tests/host/slint_list_pointer/
  - zircon_editor/src/tests/host/slint_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/mod.rs
- zircon_editor/src/tests/host/slint_tab_drag/
  - zircon_editor/tests/workbench_autolayout.rs
  - zircon_editor/tests/workbench_drag_targets.rs
  - zircon_editor/tests/workbench_slint_shell.rs
implementation_files:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/layout/mod.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/layout/virtualization.rs
  - zircon_ui/src/dispatch/mod.rs
  - zircon_ui/src/tree/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_ui/src/template/document.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/scene/viewport/controller/mod.rs
  - zircon_editor/src/scene/viewport/pointer/mod.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/slint_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/slint_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - zircon_editor/src/ui/slint_host/app/assets.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/route_mapping.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/snap_cycle.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/projection_support.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/host_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/action_control.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/resolution.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/tests.rs
  - zircon_editor/src/ui/slint_host/detail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/scroll_surface_host.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_popup_state.rs
- zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/activity_rail_pointer/build_workbench_activity_rail_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/host_page_pointer/mod.rs
- zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs
- zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/floating_windows.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/src/core/editing/ui_asset/session.rs
  - zircon_editor/src/core/editing/ui_asset/inspector_semantics.rs
  - zircon_editor/src/core/editing/ui_asset/tree_editing.rs
plan_sources:
  - user: 2026-04-14 实现运行时/编辑器共享 UI 布局与事件系统架构计划
  - user: 2026-04-15 继续实现 ScrollableBox、scroll state、visible range invalidation 和 pointer dispatcher
  - user: 2026-04-15 继续完善 shared core 线性容器
  - user: 2026-04-15 继续把 Container / Overlay / Space 落到 retained layout core，并把 editor host pointer/scroll 输入直接适配到 UiSurface + UiPointerDispatcher
  - user: 2026-04-15 继续把更完整的 editor shell pointer hit-test / dock target route 往 shared core 迁移
  - user: 2026-04-15 继续完善 shared navigation dispatcher，并把后续 keyboard/gamepad 导航入口固定到 shared core
  - user: 2026-04-16 把非-menu 的 popup/dialog/tree/list scroll 输入继续迁到同一套 shared pointer dispatcher
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - .codex/plans/Zircon UI Editor UI Binding & Reflection Architecture.md
  - .codex/plans/Slint Workbench 响应式 AutoLayout 与约束求解计划.md
  - .codex/plans/全系统重构方案.md
  - user: 2026-04-17 Source/Hierarchy/Canvas 的更强选中同步和 source roundtrip 体验
  - user: 2026-04-17 parent-specific slot/layout inspector，补 Overlay/Grid/Flow/ScrollableBox 语义
  - user: 2026-04-17 designer canvas 的可视化 authoring：插入、重排、reparent、wrap/unwrap
  - user: 2026-04-17 Palette 到真实节点/引用节点创建的落地
  - user: 2026-04-17 结构化 undo/redo，从当前 source-text 级别继续往 tree-command 演进
  - user: 2026-04-18 继续下一步，推进 Runtime visual contract
tests:
  - zircon_ui/src/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_ui/src/tests/asset.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/mod.rs
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_editor/src/tests/host/slint_detail_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/mod.rs
- zircon_editor/src/tests/host/slint_list_pointer/
  - zircon_editor/src/tests/host/slint_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/slint_activity_rail_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_host_page_pointer/mod.rs
- zircon_editor/src/tests/host/slint_document_tab_pointer/
  - zircon_editor/src/tests/host/slint_drawer_header_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/mod.rs
- zircon_editor/src/tests/host/slint_tab_drag/
  - zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/tests/editing/ui_asset/
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/tests/workbench_autolayout.rs
  - zircon_editor/tests/workbench_drag_targets.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - cargo test -p zircon_ui shared_core -- --nocapture
  - cargo test -p zircon_runtime --lib render_extract_uses_label_when_schema_text_default_is_empty --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --nocapture
  - cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --nocapture
  - cargo test -p zircon_ui --lib --locked render_extract_carries_visual_contract_fields_for_visible_nodes -- --nocapture
  - cargo test -p zircon_ui --lib --locked ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets -- --nocapture
  - cargo test -p zircon_ui --offline --verbose
  - cargo test -p zircon_editor slint_callback_dispatch -- --nocapture
  - cargo test -p zircon_editor --lib shared_viewport_surface_replaces_legacy_direct_pointer_callback_abi --locked -- --nocapture
  - cargo test -p zircon_ui --locked
  - cargo test -p zircon_editor --locked
  - rustc --edition=2024 --test zircon_editor/tests/workbench_slint_shell.rs -o <temp> && <temp> --nocapture
  - rustc --edition=2024 --test <temp-workbench-drag-target-prefix-test.rs> --extern zircon_ui=<target/debug/deps/libzircon_ui-*.rlib> -L dependency=target/debug/deps -o <temp> && <temp> --nocapture
  - cargo test -p zircon_editor --test workbench_autolayout -- --nocapture
  - cargo test -p zircon_editor slint_tab_drag -- --nocapture
  - cargo test -p zircon_editor --test workbench_drag_targets -- --nocapture
  - cargo test -p zircon_editor --test workbench_slint_shell -- --nocapture
  - cargo test --workspace --locked --verbose
  - cargo check -p zircon_editor --lib --offline
  - cargo test -p zircon_editor --lib slint_detail_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_activity_rail_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_host_page_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_document_tab_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_drawer_header_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_ --offline -- --nocapture
  - cargo test -p zircon_editor --test workbench_slint_shell --offline -- --nocapture
  - cargo test -p zircon_editor --lib uses_region_frame_fallback_in_real_host --locked -- --nocapture
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_
  - cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_
  - cargo test -p zircon_editor --lib --locked editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo
  - cargo test -p zircon_editor --lib --locked tests::host::slint_window::child_window_callback_wiring_tracks_source_window_for_pane_interactions
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_
  - cargo test -p zircon_editor --lib asset_browser_projection_maps_bootstrap_asset_into_mount_nodes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never
doc_type: module-detail
---

# Shared UI Core Foundation

## Purpose

这一轮不是“给 editor workbench 再补一层更通用的包装”，而是正式把运行时/编辑器都要依赖的 UI 基础下沉到 `zircon_ui`。目标是先收敛权威边界，再继续向 measure/arrange、focus/navigation、capture、scroll 和 ECS bridge 扩展。

这次模块边界重构之后，`layout/pass/*` 和 `tree/node/*` 都已经变成 folder-backed subtree；入口 `mod.rs` 只保留导出职责，算法和 retained tree 行为不再继续堆回单文件。

## Latest Shared Pointer Promotion

最新一刀把 editor viewport 外层 raw pointer seam 也正式提升成 shared-core authority：

- `SharedViewportPointerBridge` 已经进入真实宿主路径，不再只是测试桥
- `dispatch_viewport_pointer_event(...)`、其 dispatcher 和 pointer-route 到 `EditorViewportEvent` 的映射已经成为生产代码
- Rust-owned host callback wiring now uploads `viewport_pointer_event(kind, button, x, y, delta)`，而不是分散的 move/down/up/scroll 宿主回调

这意味着 shared pointer dispatcher 的覆盖面已经从 menu/list/tree/toolbar/scroll surface，继续扩展到了 scene/game viewport 的最外层输入入口。

## Latest Real-Host Pane Size Recovery

这一轮又把真实宿主里几条仍然可能上传 `width == 0 && height == 0` 的 pane-local scroll / pointer callback 收口到同一个 shared host-frame fallback：

- [`resolve_callback_surface_size_for_kind(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs) 现在继续作为真实宿主的统一兜底入口，优先顺序固定为 `callback size -> cached size -> shared host frame`
- `Hierarchy`、`Console`、`Inspector` 以及 `AssetBrowser details` 的 root-shell callback 已经都复用这条入口，不再各自保留“宽高为 0 就直接跳过 retained layout / scroll surface”的分支
- `browser_asset_details_pointer_scrolled(...)` 已经从“只相信 Slint 上传的局部尺寸”切到 `ViewContentKind::AssetBrowser` 的 shared frame fallback，所以 exclusive asset browser page 在真实宿主里也不会再因为零尺寸回调失去 shared scroll authority
- `resolve_callback_surface_size_for_asset_surface(...)` 现在把 `surface_mode -> ViewContentKind::{Assets, AssetBrowser}` 也固定成共享宿主规则；`asset_tree_pointer_*`、`asset_content_pointer_*` 和 `asset_reference_pointer_*` 三组 root-shell callback 都不再直接把 `UiSize::new(width, height)` 当成唯一尺寸事实
- `activity` drawer 和 exclusive `browser` page 这两条 asset pane 输入链现在都走同一条 `callback size -> cached size -> shared host frame` 顺序，因此 tree/content/reference 三类 retained pointer surface 在真实宿主里也不会再因为 `0x0` callback 失去有效 viewport size
- 新增的 real-host regressions 已经把这条 authority 锁到 10 个用例一起跑：原有 `Hierarchy` / `Console` / `Inspector` / `AssetBrowser details`，再加上 `activity/browser` 两套 `asset tree/content/reference` 零尺寸 callback fallback
- `welcome_recent_pointer_*` 这轮也切到了同一条 helper；当 root welcome page 先前已经缓存了有效 viewport size，而 host callback 后续继续上传 `0x0` 时，host 现在会先保留 cached size，再回退 shared projection frame，而不会把 cached size 直接跳成 `PaneSurfaceRoot`
- `zircon_ui/src/template/document.rs` 里的 `UiBindingRef` 也同步去掉了不成立的 `Eq` 派生，避免 shared template binding payload 的 `UiActionRef/toml::Value` 语义继续阻塞 editor-host 验证链

这条收口说明 shared pointer/scroll authority 不只是逻辑上从 legacy host callback glue 移出，连真实宿主里“尺寸事实从哪里来”也开始系统性回到 shared geometry / shared template frame 上。

同一轮继续收掉了两个仍然会重新打开 legacy geometry/getter 的 root-host seam：

- [`build_host_menu_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs) 不再接收或依赖 legacy `get_*_menu_button_frame()` 结果；当 `WorkbenchMenuBarRoot` 暂时缺席时，会直接从 shared `shell_frame` 继续推导 top-level menu button row
- [`app/pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 已删除 `legacy_menu_button_frames(...)` 调用，真实宿主里 menu hit-test/popup presentation 现在只剩 shared root-frame authority
- [`BuiltinWorkbenchRootShellFrames`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs) 现在额外导出 `Left/Right/BottomDrawerContentRoot` frame；[`resolve_host_frame_backed_size_for_kind(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs) 会先消费这些 shared drawer content frame，再回退到 `geometry.region_frame(...)`
- 新增 real-host regressions 已经把这条规则锁到 drawer-backed surface：
  - `root_hierarchy_pointer_move_prefers_shared_drawer_content_projection_over_stale_left_region_geometry`
  - `root_console_pointer_scroll_prefers_shared_drawer_content_projection_over_stale_bottom_region_geometry`

这意味着 root callback-size seam 现在已经不只会在 document pane 上优先使用 `PaneSurfaceRoot`；连 left/right/bottom drawer 里的 hierarchy/console/tree/list scroll surface 也开始首先服从 shared content frame，而不是整个 region shell 的 legacy 宽高。

同一轮里，dynamic floating-window 壳层的 frame authority 也开始从“每个 consumer 各自打开 `geometry.floating_window_frame(...)`”收口到统一 helper：

- [`floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 现在固定导出 floating-window `outer/tab/content` frame，并统一内建“优先 non-zero native host bounds，缺席时回退 geometry”语义
- [`build_workbench_document_tab_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs) 现在通过 host-bounds-aware `resolve_floating_window_tab_strip_frame_with_host_frame(...)` 构建 floating tab strip，而不是在 builder 内手写 header height 或忽略 native child host bounds
- [`ui/floating_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/floating_windows.rs)、[`app/native_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/native_windows.rs) 与 [`shell_pointer/drag_surface.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs) 现在统一读取同一份 host-bounds-aware outer frame
- [`app/helpers.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs) 与 [`app/viewport.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/viewport.rs) 现在转吃 host-bounds-aware `resolve_floating_window_content_frame_with_host_frame(...)`，child-window pane callback/toolbar width recovery 不再把整个 floating shell 高度误当成 pane content 高度，也不再忽略 native child host 尺寸
- [`app/callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 现在也把 `hierarchy/asset tree/content/reference` 的 move 回调接回 `with_callback_source_window(...)`，所以 child-window hover/move 零尺寸 callback 也能落回同一份 projected content frame
- 新增 red/green 回归 `child_window_hierarchy_pointer_move_prefers_projected_floating_window_content_frame_over_outer_window_frame` 已经锁住这条 seam
- 这轮又补上 `floating_window_projection_prefers_host_bounds_when_present`、`shared_document_tab_pointer_layout_prefers_native_window_host_bounds_for_floating_strip` 和 `shared_shell_pointer_route_prefers_native_window_host_bounds_for_floating_attach_surface`，让 helper、floating tab strip 和 shell drag attach surface 在 native child host bounds 存在时收口到同一份 shared projection 规则

同一轮里，root `host_page` strip 也不再只拿 shared shell 宽度配合 metrics 估算：

- [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) 现在直接建模 `HostPageStripRoot`
- [`BuiltinWorkbenchTemplateBridge`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs) 会在 builtin root surface 上把 `HostPageStripRoot` 固定到 `top_bar_height + separator_thickness` 的真实 strip frame，并通过 `root_shell_frames()` 导出 `host_page_strip_frame`
- [`build_workbench_host_page_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_page_pointer/build_workbench_host_page_pointer_layout.rs) 现在优先直接消费 shared `host_page_strip_frame`，只有该 frame 缺席时才回退到先前的 shell+metric 估算
- 新增 focused regressions 已经锁住：
  - `builtin_workbench_template_bridge_recomputes_surface_backed_frames_with_shell_size`
  - `shared_host_page_pointer_layout_prefers_shared_host_strip_frame_over_shell_metric_estimate`

这让 root-shell strip authority 又缩掉一层：host-page 不再是“shared shell 宽度 + legacy host-bar metrics”的混合结果，而是明确来自 builtin template/runtime 导出的 strip frame。

## Root Presentation Projection Consumption

这一轮 shared template/layout authority 不再只给 callback fallback 提供局部读取的 frame，root `slint_host` 的 presentation 主链也开始真正消费这份 shared frame authority：

- [`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在把 `WorkbenchMenuBarRoot`、`WorkbenchBody`、`DocumentHostRoot`、`PaneSurfaceRoot`、`StatusBarRoot` 这组 builtin root-shell control frame 固定打包成宿主结构
- [`apply_presentation(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/apply_presentation.rs) 新增 `shared_root_frames` 输入，root shell 的 `center_band_frame` / `status_bar_frame` 已经直接对齐到 shared projection，而不是继续只相信 `WorkbenchShellGeometry`
- `document_region_frame` 现在采用分层 mixed authority：当 `Left/Right/Bottom` drawer region 全部折叠时，root shell 直接消费 `DocumentHostRoot` 的 shared frame；只要有可见 drawer region，就改为用 shared `WorkbenchBody` 提供 `x/y` 与总可用跨度，再由 legacy drawer extents 扣出 document zone，避免 stale `geometry.region_frame(Document)` 继续充当真源
- visible drawer shell/header 这一层现在已经真正进入 builtin template/shared root projection：
  - [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) 现在直接建模 `Left/Right/BottomDrawerShellRoot` 与对应 `*DrawerHeaderRoot`
  - [`BuiltinWorkbenchTemplateBridge`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 会在 root recompute 时根据 `EditorChromeSnapshot.workbench.drawers[*].extent` 与 shared shell body frame 运行时重建 visible drawer shell/header frame，并通过 `root_shell_frames()` 导出
  - [`resolve_root_left_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs)、[`resolve_root_right_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs)、[`resolve_root_bottom_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 与 visible-drawer `document_region_frame` 现在优先消费这些 shared drawer shell frame，而不是继续从 legacy `geometry.region_frame(...).width/height` 拿主轴 extent
  - [`build_workbench_drawer_header_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs) 也会在 shared `*DrawerHeaderRoot` 存在时直接复用 header frame，因此 visible drawer header retained pointer surface 和 root shell presentation 现在共享同一份 header authority
  - focused regressions 已经扩展到 `builtin_workbench_template_bridge_exports_visible_drawer_shell_and_header_frames_from_chrome`、`apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_extents`、`shared_drawer_header_pointer_layout_prefers_shared_root_projection_for_visible_drawer_regions` 与 real-host `root_host_recomputes_builtin_template_bridge_with_visible_drawer_shell_and_header_frames`
- `viewport_content_frame` 现在也进入同一套 mixed authority：当 drawer region 全部折叠时，root shell 会从 `PaneSurfaceRoot` 推导 viewport frame；`Scene` 额外叠加 toolbar 高度，`Game` 直接使用 pane surface frame；只要有可见 drawer region，viewport frame 就复用同一份 resolved document frame，而不再回退到 legacy viewport geometry
  - [`host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 的 root recompute 现在会把 `template_bridge.root_shell_frames()` 接进 `apply_presentation(...)`；child native window presenter 路径则显式传 `None`，因此 secondary native window 仍继续走 `WorkbenchShellGeometry + configure_native_floating_window_presentation(...)` 的现有边界
- root real-host viewport sizing 也不再和 presentation 分裂：[`host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 现在会复用同一份 resolved viewport frame 更新 `viewport_size` 与 `SharedViewportPointerBridge`，让 WGPU viewport 尺寸、shared pointer bounds 和 root shell presentation 看到的是同一份 shared/template-backed frame
- 同一条 authority 这轮又继续推进到 root document 的 viewport toolbar 自身：[`app/viewport.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/viewport.rs) 现在会在 `ViewHost::Document` 上优先用 `resolve_root_viewport_content_frame(..., Some(&root_shell_frames), true)` 的宽度重算 [`BuiltinViewportToolbarTemplateBridge`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs)，不再让 stale `region_frame(Document).width` 把右侧 toolbar control group 缩回旧几何
  - [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 新增 `root_viewport_toolbar_pointer_click_prefers_shared_projection_surface_width_over_stale_document_geometry`；这条 real-host regression 直接证明，即使 legacy document geometry 被压窄，只要 drawer region 已折叠、shared root projection 仍然给出正确 pane width，toolbar 的 projection-backed hit-test 也必须继续命中并派发 `AlignView`
- root main `document tab` pointer surface 现在也开始跟这条 shared root-frame seam 对齐：
  - [`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在把 `DocumentTabsRoot` 一起纳入 builtin root-shell frame bundle
  - [`resolve_root_document_tabs_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 和 [`build_workbench_document_tab_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs) 现在遵循与 `document_region_frame` / `viewport_content_frame` 相同的 mixed-authority 规则：drawer 折叠时优先 shared projection，drawer 可见时回退 legacy geometry
  - 这意味着 root-shell `document tab` hit-test 根框不再天然依赖旧 `document_region.width`；当 shared `DocumentTabsRoot` 仍然是正确宽度时，tab pointer surface 也应该继续服从 shared projection
- root main `activity rail` pointer surface 现在也开始跟这条 shared root-frame seam 对齐：
  - [`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在把 `ActivityRailRoot` 一起纳入 builtin root-shell frame bundle
  - [`resolve_root_activity_rail_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs)、[`build_workbench_activity_rail_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/activity_rail_pointer/build_workbench_activity_rail_pointer_layout.rs) 和 [`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 现在让 root 左侧 rail 遵循与 `document tab` 相同的 mixed-authority 规则：drawer 可见时继续服从 legacy left-region geometry，drawer 全折叠时优先 shared `ActivityRailRoot`
  - [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 的 `root_activity_rail_pointer_click_prefers_shared_projection_surface_when_left_region_geometry_is_stale` 与 [`layout_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_activity_rail_pointer/layout_projection.rs) 的 root-frame regression 共同锁住这条 seam：即使 legacy `ShellRegionId::Left` 被压成 `0x0`，root host 仍然必须继续用 shared `ActivityRailRoot` 命中左侧 rail toggle
- root `host page` strip 这轮也不再继续靠 `TAB_MIN_WIDTH` 估出来的 legacy metric 根框撑住命中链：
  - [`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在额外导出 `WorkbenchShellRoot`，让 builtin root-frame bundle 终于覆盖整张 root shell，而不只是一组局部子控件
  - [`build_workbench_host_page_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_page_pointer/build_workbench_host_page_pointer_layout.rs) 和 [`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 现在会在 root host path 上优先吃 shared shell width，再叠加既有 `top_bar_height/host_bar_height` 契约；这修掉了“实际 tab 很宽或落在 strip 右侧时，shared `UiSurface` 根框仍然只有 min-tab 估宽”的 mixed-authority seam
- [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 的 `root_host_page_pointer_click_prefers_shared_projection_shell_width_over_metric_strip_estimate` 已经 focused green，直接证明 far-right host-page tab 点击现在继续命中 shared route；[`layout_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_host_page_pointer/layout_projection.rs) 与 [`template_bridge/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs) 则把 `WorkbenchShellRoot` frame 导出和 builder 宽度契约一起锁住
  - Historical pre-fence reruns of `cargo test -p zircon_editor --lib shared_host_page_pointer_layout_prefers_shared_shell_width_over_metric_strip_estimate --locked` / `builtin_workbench_template_bridge_recomputes_surface_backed_frames_with_shell_size --locked` could be blocked by old Slint build-script drift; current validation targets `.ui.toml` and Rust-owned `host_contract` seams instead of deleted Slint copies
- [`ui/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/tests.rs) 新增两条 root presentation regression：`drawers collapsed -> root presentation consumes shared projection frame`，以及 `drawers visible -> document region keeps geometry while center/status already consume shared projection`
- [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 现在又补了一条 real-host alignment regression：`root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed`
- 这条 regression 在生产改动前给出的真实 red 不是抽象语义，而是具体的 frame 漂移：`host.viewport_size = 1600x876`，但 root shell 的 `viewport_content_frame = 1544x884`
- 这一步当前已经有直接 focused green 证据，而不只是 compile green：`apply_presentation_uses_shared_root_projection_frames_when_drawers_are_collapsed`、`apply_presentation_keeps_geometry_document_region_when_drawers_are_visible`、`builtin_workbench_template_bridge_recomputes_surface_backed_frames_with_shell_size`、`root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed` 与 `cargo check -p zircon_editor --lib --locked` 都已通过
- [`template_bridge/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs) 也同步锁住 `root_shell_frames()` 的 control 映射，防止 callback fallback 和真实 presentation 后续对同一组 builtin control frame 漂移
- 同一条 root-shell projection seam 这一刀又继续推进到了 tab-drop 精确 strip 命中：[`resolve_workbench_tab_drop_route_with_root_frames(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/route_resolution.rs)、[`resolve_tab_drop_with_root_frames(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/drop_resolution.rs) 和 [`strip_hitbox.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/strip_hitbox.rs) 现在会把 `BuiltinWorkbenchRootShellFrames` 继续带到 tool/document tab strip 的 `x/y` 计算，让 shared drag surface 命中、presentation frame 和 precise attach anchor 维持同一份 root-shell authority
- 这修掉了一个典型 mixed-authority 回退：shared shell pointer route 已经因为 `WorkbenchBody` 平移而命中了正确的 `Right` tool stack，但 precise drop target 过去仍会按旧 `geometry.center_band_frame.y` 算 tab strip，最终退化成 `anchor: None`。真实宿主的 [`workspace_docking.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/workspace_docking.rs) 现在也会把 `template_bridge.root_shell_frames()` 传进 drop route 解析，所以 attach anchor 不再落回旧 geometry 真源
- 同一轮还把 pure helper/parity 入口补到了相同 authority：[`resolve_workbench_drag_target_group_with_root_frames(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/bridge.rs) 现在允许 `resolve_workbench_drag_target_group(...)` 这类纯函数路径显式接入 builtin root-shell frame。新的 focused regression `resolve_workbench_drag_target_group_with_root_frames_uses_shared_root_projection_document_bounds_when_drawers_are_collapsed` 已经 green，所以 shared root projection 不再只在真实宿主 `WorkbenchShellPointerBridge` 路径上成立

这一步说明 shared authority 已经从“真实宿主输入恢复尺寸时的兜底读模型”推进到“真实宿主 presentation 的主动输入”。虽然 root shell 目前对 drawer/document 仍是混合状态，但 frame authority 已经开始脱离手写 Slint 几何常量，后续 clip/z-order 继续进入真实宿主时也有了明确接线点。

## Implemented Authority Boundary

当前已经进入 `zircon_ui` 的共享基础分成七组：

### 1. Layout Primitives

- `StretchMode`
- `AxisConstraint`
- `ResolvedAxisConstraint`
- `BoxConstraints`
- `DesiredSize`
- `LayoutBoundary`
- `UiAxis`
- `UiContainerKind`
- `UiLinearBoxConfig`
- `UiScrollState`
- `UiScrollableBoxConfig`
- `UiVirtualListConfig`
- `UiPoint`
- `UiSize`
- `UiFrame`
- `solve_axis_constraints(...)`

这套类型现在同时服务 runtime widget 和 editor shell。`max == -1`、优先级/权重拉伸、低优先级先收缩、滚动主轴和容器声明这些语义不再允许 editor 维护一份平行实现。

### 2. Measure, Arrange, And Shared Container Pass

- `compute_layout_tree(...)`
- `UiTreeNode::{constraints, anchor, pivot, position, container, scroll_state}`

当前不再只是 overlay/container 风格的基础 pass，而是已经包含第一批真正的共享容器：

- 反向 measure 先写入每个节点的 `desired_size`
- `ContentDriven` 节点会吸收子节点最大的 `desired_size`
- 正向 arrange 从 surface 根尺寸下发 `frame`
- `Stretch` 轴先按可用空间扩展，`Fixed` 轴保留 measured size
- `Anchor + Pivot + Position` 用统一公式把子节点放进父节点 frame
- `UiContainerKind::Container` 和 `UiContainerKind::Overlay` 已经作为显式共享容器名落地，V1 先复用统一 anchored/free retained layout 语义
- `UiContainerKind::Space` 已经作为共享 spacer 节点落地：measure 时忽略子内容，arrange 时隐藏整棵子树 frame
- `UiContainerKind::HorizontalBox` 和 `UiContainerKind::VerticalBox` 已经能按共享主轴模型排列子节点
- 线性容器会用 `solve_axis_constraints(...)` 在主轴上分配可用空间，并保留 `gap` 语义
- 线性容器的交叉轴行为已经进入 shared core：子节点 `Stretch` 约束会填满父级交叉轴，`Fixed` 约束保留 measured size
- `UiContainerKind::ScrollableBox` 已经能按横向或纵向线性容器排列子节点
- scrollable viewport 会自动 clip 到本地 frame，并维护 `viewport_extent` / `content_extent` / `offset`
- 当前 virtualization 先支持线性列表窗口，而不是完整 grid/flow 求解

这已经把你文档里的“两段式：反向 measure，正向 arrange”以及第一批线性容器真正接进了 shared core。

### 3. Retained Tree And Dirty State

- `UiTree`
- `UiTreeNode`
- `zircon_ui::tree::UiDirtyFlags`
- `zircon_ui::tree::UiLayoutCache`
- `UiInputPolicy`

当前实现先把 retained tree 的基础骨架定下来：

- 节点拥有 parent/children、可见性和 pointer 能力状态
- dirty 传播沿父链向上冒泡
- 当父节点 `LayoutBoundary` 不是 `ContentDriven` 时停止布局失效传播
- `zircon_ui::tree::UiLayoutCache` 现在承载 `desired_size`、`frame`、`clip_frame`、`content_size` 和 `virtual_window`
- `set_scroll_offset(...)` 只会把 scrollable 节点自身标成 `layout/hit_test/render/input` dirty
- 当 visible window 跨项变化时，还会额外标记 `visible_range`

这正对应计划里“子变更向上冒泡，遇到固定边界或外部尺寸边界停止”的第一版要求。

### 4. Hit Test Semantics

- `zircon_ui::tree::UiHitTestIndex`
- `zircon_ui::tree::UiHitTestResult`

当前命中语义固定为：

- 反向绘制顺序查找 top-most 命中节点
- 整条可见性继承链必须可见
- `UiInputPolicy::Ignore` 节点及其显式忽略语义不会成为命中目标
- 节点必须具备 pointer 能力：`clickable`、`hoverable` 或 `focusable`
- `clip_to_bounds` 会沿祖先链检查 `clip_frame` 或 `frame`

这里的索引仍然只是 draw-order cache，不是语义真源。后续要接四叉树或别的空间索引，也必须保持这层语义不变。

### 5. Pointer, Focus, Navigation, Capture Route Foundations

- `UiSurface`
- `UiFocusState`
- `UiNavigationState`
- `UiPointerButton`
- `UiPointerEventKind`
- `UiPointerRoute`
- `UiNavigationEventKind`
- `UiNavigationRoute`
- `UiNavigationDispatcher`
- `UiNavigationDispatchEffect`
- `UiNavigationDispatchResult`
- `UiPointerDispatcher`
- `UiPointerDispatchEffect`
- `UiPointerDispatchResult`

当前 shared core 不再只有 route/state，而是同时拥有 pointer 和 navigation 两套 canonical dispatcher：

- pointer 事件会基于命中或当前 capture 派生 bubble route
- `UiPointerEvent` 现在显式携带 button payload，shared route/dispatcher 不再丢失 primary/secondary/middle 区分
- `Down` 会把焦点移动到 bubble route 上第一个 focusable 节点
- `capture_pointer(...)` / `release_pointer_capture(...)` 让宿主或未来 dispatcher 显式管理 capture
- navigation 先从当前 focus 或 navigation root 派生 route；没有目标时回退 roots
- `UiSurface::dispatch_navigation_event(...)` 现在会在 focused route 或 root fallback 上执行 `UiNavigationDispatcher`
- navigation handler 可以返回 `Handled` 或 `Focus(UiNodeId)`；后者会通过 `UiSurface::focus_node(...)` 收口回共享 focus 状态
- `focus_node(...)` / `clear_focus()` 统一维护 `focused`、`navigation_root` 和 `focus_visible`，避免宿主自己散落地改 focus bookkeeping
- 当没有 focused 节点时，root handler 就是 canonical fallback；这给 editor keyboard、gamepad 和 headless 导航留下了同一条 dispatcher path
- 如果没有 handler 消费导航事件，shared fallback 现在固定为：
  `Next` / `Previous` 沿 retained tree 的共享 focus 顺序循环；
  `Right` / `Down` 在无焦点时从第一个 focusable 节点开始；
  `Left` / `Up` 在无焦点时从最后一个 focusable 节点开始；
  已有焦点时按共享几何最近邻规则做方向选择；
  `Activate` / `Cancel` 不做隐式焦点跳转
- dispatcher 现在显式实现 `Handled`、`Blocked`、`Passthrough`、`Captured`
- `Blocked` 会停止当前 bubble route 并尝试下一个同位置 stacked target
- `Passthrough` 会允许当前 route 之后继续把事件派给后续 stacked target
- `Captured` 会把 pointer capture 收口回 `UiSurface::focus.captured`
- 当 pointer 已经 capture 后，即使光标移出当前 hit bounds，dispatcher 也会继续把 `Move` / `Up` 派回 captured target，而不是因为 `stacked` 为空丢失事件

这保证后续 editor host、headless 控制和 runtime widget 都可以消费同一套 target/bubble/capture/fallback/stacked-target 语义，同时把 keyboard/gamepad 的 navigation bubbling 和 focus handoff 也钉在 shared core，而不是各自再推一遍。

### 6. Scroll Dispatch And Virtualization Helpers

- `UiPointerEvent`
- `UiVirtualListWindow`
- `compute_virtual_list_window(...)`

当前 scroll/virtualization 相关的共享行为已经具备第一个闭环：

- `UiPointerEventKind::Scroll` 现在可以带 `scroll_delta`
- 如果显式 handler 没有消费 scroll 事件，surface 会把滚轮默认路由到最近的 scrollable 节点
- `ScrollableBox` 的 `offset` 更新会复用 shared visible-window 计算
- 超出 visible window 的子树会被清空 frame，避免继续参与命中和绘制
- `UiVirtualListWindow` 仍然保持为共享纯函数工具，后续 `GridBox` 可以继续复用

这还不是完整容器族，也还没接入 editor/runtime 宿主事件桥，但“scroll state、visible range invalidation、pointer dispatcher 都属于 shared core”这个边界已经定下来。

### 7. Surface And Render Extract

- `UiRenderCommand`
- `UiRenderList`
- `UiRenderExtract`

surface 仍然是共享汇总点：

- 一棵 `UiTree`
- 一个命中索引
- focus/navigation/capture/hover 状态
- 从 tree 抽出的 visual draw list

这一轮 `UiRenderExtract` 已经不再只是 `node_id/frame/clip/z_index` 的几何枚举，而是升级成 shared visual contract：

- `UiRenderCommand` 继续保留 `node_id`、`frame`、`clip_frame`、`z_index`
- 同时新增 `kind: UiRenderCommandKind`
  - `Group`
  - `Quad`
  - `Text`
  - `Image`
- 新增 `style: UiResolvedStyle`
  - `background_color`
  - `foreground_color`
  - `border_color`
  - `border_width`
  - `corner_radius`
- 新增 `text`
- 新增 `image: UiVisualAssetRef`
  - `Icon`
  - `Image`
- 新增 `opacity`

当前抽取仍然保持“一条可见 retained node 对应一条 render command”的形状，原因是 editor preview 和后续 renderer 都还需要稳定的 node identity。区别在于没有直接视觉载荷的节点现在会变成 `Group` 命令，而不是继续被当成“只有 frame 的旧式几何条目”。

shared visual payload 当前直接从 `zircon_ui::tree::UiTemplateNodeMetadata.attributes` 解析：

- `background` / `foreground` / `border`
- `text` 或 `label`
- `icon` 或 `image`
- `opacity`

这让 `.ui.toml` 里已经经过样式解析的属性，能够直接沉到 shared surface 的视觉抽取出口；editor preview、后续 runtime renderer 和 screenshot/golden harness 都不需要再各自重读模板 source 才知道节点要画什么。

2026-04-29 的 Runtime UI regression 又把文本解析顺序收紧成“非空 `text` 优先，否则回退非空 `label`”。这避免 legacy/generic visual assets 在 schema defaults 注入 `text = ""` 时丢失作者写在 `label` 上的可见文案；对应 focused test 是 `render_extract_uses_label_when_schema_text_default_is_empty`，随后 broad `zircon_runtime --lib ui::tests` 也覆盖了这条 shared extract contract。当前 Asset Browser bootstrap asset 不再把这条 fallback 当作 `Button` authored contract：runtime `Button` descriptor owns `text`, and `asset_browser.ui.toml` now authors `LocateSelectedAsset` and the other action buttons with `text`, covered by `asset_browser_projection_maps_bootstrap_asset_into_mount_nodes` in the Milestone 0 editor closeout target.

与之对应，`zircon_ui::tree::UiDirtyFlags` 现在也显式保留 `text` 位，开始把 layout/style/text/render 脏域拆开，而不是继续把所有视觉变化都挤回一个粗粒度 render flag。

## Editor Integration Boundary

`zircon_editor::workbench::autolayout` 现在只保留 editor shell 特有的语义：

- `ShellRegionId`
- `PaneConstraintOverride`
- `AxisConstraintOverride`
- `WorkbenchChromeMetrics`
- `WorkbenchShellGeometry`
- `WorkbenchLayout` 到共享约束的映射规则

它不再定义以下基础类型：

- `StretchMode`
- `AxisConstraint`
- `ResolvedAxisConstraint`
- `PaneConstraints`
- `ShellFrame`
- `ShellSizePx`
- `solve_axis_constraints(...)`

对应关系现在是：

- `PaneConstraints` = `zircon_ui::BoxConstraints`
- `ShellFrame` = `zircon_ui::UiFrame`
- `ShellSizePx` = `zircon_ui::UiSize`

这让 editor workbench 和 runtime UI 至少先在“如何表达尺寸约束、如何表达 frame、如何做主轴空间分配”上完全对齐。

## Viewport Host Bridges

shared core 现在已经不只是“供 editor 将来接入”的离线骨架，viewport 已经形成两层真实接线：

- `zircon_editor::host::slint_host::callback_dispatch::SharedViewportPointerBridge` 持有一个最小 `UiSurface`
- bridge 只建立 root + viewport 两个 retained 节点，并在 viewport 节点上注册 `UiPointerDispatcher`
- `Down` 在 shared dispatcher 内触发 capture，`Move` / `Up` / `Scroll` 继续复用同一条 route
- `zircon_editor::host::slint_host::app` 的 viewport pointer/scroll callback 现在先生成 shared `UiPointerEvent`
- shared route 命中 viewport 后，才被语义化成 `EditorViewportEvent::{PointerMoved, Left/Right/MiddlePressed, Left/Right/MiddleReleased, Scrolled}`
- `InputManager` 的原始桌面输入提交仍然保留，shared bridge 只负责 UI route/capture 语义和 editor runtime dispatch 收口

这一刀的意义是把“viewport pointer/scroll 的目标、capture 和事件映射”从 legacy host callback direct dispatch，改成依赖 shared `UiSurface + UiPointerDispatcher` 的第一条真路径。

### Viewport Toolbar Pointer Bridge

Scene viewport toolbar 现在也不再属于 “host 自己按 callback id 猜命令” 的例外面：

- [`zircon_editor::host::slint_host::viewport_toolbar_pointer::ViewportToolbarPointerBridge`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs) 会为每个 scene toolbar surface 建一棵最小 retained `UiSurface`
- bridge surface 里显式放置 toolbar surface 节点和当前激活 control 节点，再由 shared `UiPointerDispatcher` 输出 `ViewportToolbarPointerRoute`
- route 结果统一落成 `SetTool / SetTransformSpace / SetProjectionMode / AlignView / CycleDisplayMode / CycleGridMode / FrameSelection` 等 editor-facing route，而不是再让 host callback 名称直接充当业务协议
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 的 `dispatch_shared_viewport_toolbar_pointer_click(...)` 再把 shared route 对接到 `BuiltinViewportToolbarTemplateBridge`
- Rust-owned host callback wiring 现在只保留 `viewport_toolbar_pointer_clicked(...)` 这条 pointer-fact 边界；旧的 `viewport_set_*` / `viewport_frame_selection` 直连 callback ABI 已经移除

这意味着 Scene toolbar 现在和 menu/list/dock 一样，真实命中先经过 shared surface，再进入 template binding/runtime dispatcher，而不是继续保留一套宿主侧特判协议。

### Viewport Overlay Pointer Bridge

在 viewport 外框 capture 之后，editor runtime 内部的 gizmo / handle / renderable overlay 命中也已经继续并入 shared core：

- [`zircon_editor::scene::viewport::pointer::ViewportOverlayPointerRouter`](/E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/pointer/mod.rs) 持有第二棵专门用于 overlay picking 的 `UiSurface`
- `sync_scene(...)` 会把 handle overlay、scene gizmo pick shape、renderable 候选统一投成 coarse retained 节点
- viewport 节点上的 `UiPointerDispatcher` 不直接相信 coarse frame，而是基于 `context.route.stacked` 做精确排序并收口成 `ViewportPointerRoute`
- 当前 route 优先级固定为 `HandleAxis > SceneGizmo > Renderable`；同优先级再按屏幕距离分数和 depth 决定
- [`SceneViewportController`](/E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/controller/mod.rs) 现在只消费 `ViewportPointerRoute`，不再维护本地 overlay picking cache 或额外 `picking.rs` 语义

因此 viewport 现在不是只有“外框 pointer 进入 shared route”，而是连 Scene 内部 overlay hit-test 也开始遵守和 menu / dock / asset list 相同的 shared `UiSurface + UiPointerDispatcher` 契约。

## Workbench Menu Pointer Bridge

在 viewport 之后，editor shell 的 menu / popup / scroll 命中也已经进入 shared core：

- `zircon_editor::host::slint_host::menu_pointer::HostMenuPointerBridge` 持有一棵独立的 menu `UiSurface`
- retained tree 里会显式放置：
  - top menu button
  - popup dismiss overlay
  - popup surface
  - popup item
- `Window` 菜单 popup surface 会声明成 shared `ScrollableBox`，并用 `UiScrollState` 持有 `offset / viewport_extent / content_extent`
- `Scroll` 事件现在先更新 shared scroll state，再把结果回灌到 Slint 的 `window_menu_scroll_px`
- popup dismiss route 现在只关闭 open/hover/item 状态，不再顺手把 `popup_scroll_offset` 归零；shared `UiScrollState` 会一直保留到下一次显式 `open_popup(...)`，这样真实宿主的 dismiss/reopen、journal/replay 和 parity shell 都不会丢掉 canonical window-menu scroll authority
- [`build_host_menu_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs) 现在只从 `BuiltinWorkbenchRootShellFrames::{shell_frame, menu_bar_frame}` 派生六个顶层 menu button frame；[`app/helpers.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs) 已不再保留 `legacy_menu_button_frames(...)` getter bridge
- [`app/pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 现在会把同一组 projection-backed button frame 回灌给 Rust-owned menu chrome host data，因此 popup presentation anchor 和 shared hit-test 终于共用同一份 root authority，而不是“命中来自 shared surface，但 popup 位置仍由 `top_bar.*_local_frame` 决定”的分裂状态
- `zircon_editor::host::slint_host::callback_dispatch::dispatch_shared_menu_pointer_click(...)` 把：
  Slint 坐标
  -> menu `UiSurface` 命中
  -> `UiPointerDispatcher`
  -> template-aware menu dispatch fallback
  -> `EditorEventRuntime`
  收成一条统一入口
- 新增 focused regressions 已经把这条收口锁住：
  - [`slint_menu_pointer/layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_menu_pointer/layout.rs): `shared_menu_pointer_layout_prefers_shared_root_menu_bar_projection_over_stale_legacy_frames`
  - [`slint_menu_pointer/surface_contract.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs): `shared_menu_popup_presentation_drops_host_menu_button_frame_setters`
  - [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs): `root_menu_popup_scroll_and_dismiss_flow_through_shared_pointer_bridge_in_real_host`

因此 menu/popup 不再只是“命中在 shared core，action 仍由宿主本地猜”的半迁移状态。现在静态 menu button 和动态 preset popup item 都会先经过 shared pointer route，再进入 runtime dispatcher。

## Workbench Structural Tab Pointer Bridges

menu 之外，editor shell 里最频繁的结构性标签输入也已经开始用同样的 shared-core 形态建模，而不是继续让 host callback 名称或 `(slot, id)` 字符串组合作为语义真源。

当前新增了四类 focused host bridge：

- `WorkbenchActivityRailPointerBridge`
- `WorkbenchHostPagePointerBridge`
- `WorkbenchDocumentTabPointerBridge`
- `WorkbenchDrawerHeaderPointerBridge`

这些 bridge 的共同点是：

- 每类 bridge 只建一棵足够小的 shared `UiSurface`
- retained tree 里只放当前 strip 真正需要命中的节点
- route 结果会先收口到 editor-specific public route enum
- 再由 [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 解释成 builtin template binding 或 typed fallback

这四类 bridge 还刻意分成两种 pointer 触发语义：

- activity rail 与 host page strip 使用按下即命中的 `pointer_pressed`
- document/floating tabs 与 drawer header tabs 使用 click-level `pointer_clicked`

差异的原因是 shared core 现在不仅要负责命中，还要尊重宿主已经存在的 drag suppression：

- document/floating tabs 需要和 tab drag/drop 共存
- drawer header tabs 需要和 drawer tab drag 共存
- 因此只有 click-level hook 才能保证 shared route 不会在拖拽起手时误发激活/关闭/折叠

同时，host 并没有把文本测量或视觉布局重新搬回 shared core；对动态宽度 strip，它只把 Slint 当前实例几何当作结构输入上传：

- tab 的 strip-local `x`
- tab 的当前 `width`
- 局部点击点坐标

真正的业务 target 解析仍然发生在 shared surface 侧，而不是由 Slint 自己决定：

坐标与局部几何
-> shared `UiSurface`
-> `UiPointerDispatcher`
-> `Workbench*PointerRoute`
-> builtin template binding / typed fallback
-> `EditorEventRuntime`

这让 shared core 在 editor shell 中第一次同时覆盖了：

- `ActivityRail/*`
- `WorkbenchShell/ActivateMainPage`
- `DocumentTabs/ActivateTab`
- `DocumentTabs/CloseTab`
- drawer header 的 collapse/reopen toggle route

因此当前 shell 的结构性标签输入已经和 menu/popup、list/tree/scroll、dock target/resize 一样，进入了同一套 shared pointer authority。

### Floating Transient Overlay Focus Route

在结构性 tab 之外，当前 shell 里真正还活着的 non-menu transient overlay hit surface 也已经收口到 shared core，但 inventory 比之前推测的小得多：

- menu popup 仍然是唯一带 dismiss overlay 的 transient surface
- menu 之外，当前 workbench shell 没有第二个 standalone popup/dialog/modal dismiss surface
- 唯一需要单独命中的 non-menu transient overlay，是 floating-window header；它属于 persistent workspace host 的 chrome，而不是 dismissible popup

这一块现在的主链是：

- Rust-owned host callback wiring 只上传 `floating_window_header_pointer_clicked(...)`
- [`callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 把 pointer fact 接回宿主
- [`shell_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/shell_pointer.rs) 先通过 shared drag surface 解析 `WorkbenchShellPointerRoute::FloatingWindow(..)` / `FloatingWindowEdge { .. }`
- [`callback_dispatch/layout/floating_window/dispatch.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs) 再把 route 收口成 `LayoutCommand::FocusView`
- [`FloatingWindowModel`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/model/mod.rs) 现在提供 canonical `focus_target_instance()` / `focus_target_tab()` helper，projection 和 dispatch 都复用它
- focus target fallback 现在固定为“存在于 tab 集中的 `focused_view -> active tab -> first tab`”；stale `focused_view` 会在 shared model 层被过滤掉，而不是把 overlay active pane 和 runtime focus target 撕开

这说明 shared pointer dispatcher 现在不只是覆盖可关闭 popup/menu，也开始覆盖 editor shell 里剩余的 transient overlay chrome 命中；同时它也给了当前 inventory 一个明确边界，避免后续再错误地为并不存在的 dialog/dismiss surface 扩张宿主 ABI。

## Projection-backed real-host pane size fallback

shared pointer authority 这一轮又补了一条真实宿主兜底：当 host callback 继续上传 pointer 坐标，但 `width/height` 因宿主时序暂时还是 `0` 时，host 不再把对应 pane 直接判成“没有可用布局”。

- [`BuiltinWorkbenchTemplateBridge::control_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在暴露 builtin workbench control 的 shared frame 查询
- [`callback_dispatch::PANE_SURFACE_CONTROL_ID`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/constants.rs) 固定指向 `PaneSurfaceRoot`，让真实宿主可以从 shared template projection 取到当前 pane surface 的 canonical frame
- [`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 的 welcome recent retained pointer sync 现在会在本地 `welcome_recent_pointer_size` 仍然无效时，回退到 `PaneSurfaceRoot` 的 shared frame；只有两边都拿不到有效尺寸时才跳过布局同步
- 这条 fallback 会把 shared projection 解出的宽高写回宿主 `welcome_recent_pointer_size`，因此后续 hover/scroll/click 继续复用同一份 retained pointer state，而不是把 zero-size callback 当成永久 no-op
- [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 的 `root_welcome_recent_pointer_click_uses_projection_fallback_in_real_host` 现在直接锁住这条真实宿主回归；[`template_bridge/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs) 也同步锁住 `control_frame("PaneSurfaceRoot")`

这一步的意义不只在 welcome recent：它先把“真实宿主 callback 几何可能暂时缺席，但 shared template/layout 已经有权威 frame”这条边界钉死。后续 hierarchy、detail scroll 和更多 pane-local shared pointer surface 都应该沿这条 seam 扩展，而不是继续把 host callback 宽高当成唯一尺寸真源。

## ScrollSurfacePointerBridge

在 menu 之后，shared core 又补上了一类更轻量的宿主接线：只需要 canonical wheel/scroll state、但不需要 editor-only row route 的 scroll-only pane surface。

- [`zircon_editor::host::slint_host::detail_pointer::ScrollSurfacePointerBridge`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/detail_pointer/mod.rs) 会建立一棵最小 retained surface：
  - root 节点
  - 一个声明为 `ScrollableBox` 的 viewport 节点
- bridge 用 shared `UiScrollState` 持有：
  - `offset`
  - `viewport_extent`
  - `content_extent`
- 宿主只需要同步 `pane_size + content_extent + scroll_state`
- `Scroll` 事件统一先进入 `UiSurface::dispatch_pointer_event(...)` 和 `UiPointerDispatcher`
- bridge 再把最新 `scroll_offset` 回灌给 Slint 的 `scroll_px`
- [`scroll_surface_host.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/scroll_surface_host.rs) 现在把 scroll-only 宿主状态封成 `ScrollSurfaceHostState`，避免 `app.rs` 为每个 pane 重复持有 `bridge + state + size`

当前已经复用这条模式的 editor 宿主 surface 包括：

- `ConsolePane`
- asset browser `SelectionDetailsRail`
- `InspectorPane`

现在这三块 pane 都已经不再保留 Slint `ScrollView.viewport-y` 作为第二份滚动真源：

- `InspectorPane` 先一步切成 clipped content stack + host-driven `scroll_px`
- `ConsolePane` 现在也改成 retained text stack + clip + shared `scroll_px`
- `SelectionDetailsRail` 同样改成 retained detail stack + clip + shared `scroll_px`

这让 shared core 对 scroll-only pane 的 authority 变成真正闭环：

- `UiSurface::dispatch_pointer_event(...)` + `UiPointerDispatcher` 先更新 canonical `UiScrollState`
- `ScrollSurfaceHostState` 只保存 shared scroll offset 和 pane size
- Slint 只消费 `scroll_px` 做平移，不再单独维护 viewport state

同一轮里，[`callback_dispatch/viewport/snap_cycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/snap_cycle.rs) 的 helper 可见性也已经补到 `viewport` 子树内，使 [`route_mapping.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/route_mapping.rs) 可以稳定复用同一套 shared viewport toolbar route 映射，而不会因为模块拆分再次卡住验证链路。

## Template-backed pane surface actions

shared core 这一轮虽然没有为 pane empty-state 按钮单独新增一套 pointer bridge，但又补掉了一个真实宿主逃逸口：条件性 pane surface action 不再通过 root shell 的 handwritten `menu_action(action_id)` callback 直接越过 runtime。

- [`pane_surface_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml) 现在定义 builtin `PaneSurface/TriggerAction`
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 用 `BuiltinPaneSurfaceTemplateBridge` 把 `control_id + action_id` 收口回 canonical `MenuAction`
- Rust-owned host callback wiring only uploads `pane_surface_control_clicked(...)` / `surface_control_clicked(...)` generic control facts
- [`pane_surface_actions.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs) 成为真实宿主的唯一入口

这条链路当前覆盖：

- Scene/Game empty-state 的 `Open Scene / Create Scene`
- Project pane overview 的 `Open Assets`

因此 editor shell 里的 transient pane surface 现在也不再保留一条 Slint-local 菜单 ABI。即使 payload 最终仍然属于 `MenuAction` 家族，真正的 action normalization 也重新回到了 template/runtime authority，而不是散落在 Slint 壳层。

## Asset Tree, Content, And Reference Pointer Bridges

在 menu 之后，asset workspace 的列表输入也继续往 shared authority 收口，而不是把 tree/content 做完就停下：

- [`zircon_editor::host::slint_host::asset_pointer`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/asset_pointer.rs) 现在同时承载三类 shared list bridge：
  - `AssetFolderTreePointerBridge`
  - `AssetContentListPointerBridge`
  - `AssetReferenceListPointerBridge`
- `AssetReferenceListPointerBridge` 会显式建立：
  - reference list viewport 节点
  - reference row 节点
  - shared `ScrollableBox + UiScrollState`
- `known_project_asset == false` 的 reference row 不再注册成可点击 target，因此 hover/click 语义会自然回退到 list surface，而不是由 Slint disabled `TouchArea` 单独决定
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 新增 `dispatch_shared_asset_reference_pointer_click(...)`，把：
  坐标
  -> asset reference `UiSurface`
  -> `UiPointerDispatcher`
  -> `AssetPointerReferenceRoute`
  -> stable `AssetSurface/ActivateReference`
  -> `EditorEventRuntime`
  收成一条测试可覆盖的统一入口

宿主侧 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 现在也不再只维护 asset `tree/content` 两块 shared pointer state，而是为 `activity` 与 `browser` 两套 surface 都追加：

- `references` shared pointer state
- `used_by` shared pointer state

对应的 Rust-owned asset reference list host surface 现在只负责：

- 上传 viewport-local pointer/scroll 输入
- 消费 host 回灌的 `hovered_index` / `scroll_px`

它不再自己拥有 row `TouchArea` 命中权。

这一轮又把 shared authority 从“路径可用”收紧到了“ABI 唯一”：

- Rust-owned host callback wiring has removed hierarchy/asset/welcome list-surface direct selection/action callback declarations and forwarding
- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 也移除了对应 `ui.on_*` 注册和 helper，shared `UiPointerDispatcher` 成为这些 list surface 唯一允许的交互 authority
- 这让 list surface 和 menu popup、scroll-only pane 一样，都只剩“上传 pointer/scroll 事实 + 消费 host 投影状态”的宿主边界

这层约束由 [`surface_contract.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_list_pointer/surface_contract.rs) 明确锁定，防止后续 cutover 又把旧 callback ABI 从 Slint 壳层接回来。

同一轮里，asset surface 的非 pointer control 入口也被收成了稳定 control-id seam，而不是继续让 host-specific business callback names 承载语义：

- Activity/browser leaf host surfaces now expose only `control_changed(control_id, value)` and `control_clicked(control_id)`
- Top-level Rust host wiring only keeps `asset_control_changed(source, control_id, value)` and `asset_control_clicked(source, control_id)`
- [`app/callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 只负责把这两个 generic callback 交给宿主
- [`app/assets.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/assets.rs) 再把稳定 `control_id` 翻译成 builtin `AssetSurface/*` template 所需的 `UiEventKind + arguments`
- `SearchEdited` / `SetKindFilter` / `SetViewMode` / `SetUtilityTab` / `OpenAssetBrowser` / `LocateSelectedAsset` / `ImportModel` 现在都通过同一套 generic control route 进入 `BuiltinAssetSurfaceTemplateBridge`
- `mesh_import_path_edited(...)` 仍然保留在 draft binding 主链，因为它还不是 `asset_surface_controls.ui.toml` 当前覆盖的 builtin template control

这说明 shared core 在 editor host 里的 authority 不只是“pointer route first”，还开始要求宿主把非 pointer control 的语义边界也压缩成稳定 route/control id，再由 template/runtime 去决定 typed payload。

## Workbench Shell Pointer Bridge

editor shell 的 dock target route 和 splitter route 现在不再各自维护一棵独立 surface，而是开始收口到同一个 host-owned shell pointer bridge：

- `zircon_editor::host::slint_host::shell_pointer::WorkbenchShellPointerBridge` 持有一棵统一的 `UiSurface`
- 同一棵 retained tree 里同时放置 `document / left / right / bottom` 四个 drag target 节点，以及 `left / right / bottom` 三个 resize target 节点
- drag route 和 resize route 共享同一套 hit-test、clip 链和 capture 状态，但通过两个 mode-specific `UiPointerDispatcher` 分别解释语义
- drag dispatcher 继续在 `left/right` 与 `bottom` 重叠区里根据 shared route point 做 `Handled / Passthrough` 分流，`document` 作为 fallback target
- resize dispatcher 在 `Down` 时对 splitter target 触发 shared `Captured`，后续 `Move / Up` 即使移出 splitter hit bounds 也继续回到同一个 target

这保留了你原文档里“重叠区域按更合理边界归属”和“捕获后继续由同一对象处理”的两层语义，但判断与 capture 都已经收口到 shared route/dispatcher，而不是 Slint 本地字符串条件链。

宿主层新增的 ownership 现在是：

- `SlintEditorHost::recompute_if_dirty(...)` 重算 geometry 后，只同步更新一次 `WorkbenchShellPointerBridge`
- Slint shell 只通过 `update_drag_target(x, y)` 和 `begin/update/finish_drawer_resize(x, y)` 回传 pointer 坐标
- host 用同一棵 shared surface 求 drag target route、splitter route 和 resize capture，再把 editor-only 结果落成 `active_drag_target_group` 或 `LayoutCommand::SetDrawerExtent`
- drag overlay 的高亮、badge 和 drop 执行，及 splitter resize 的 pointer capture，都消费这条 host-owned shell bridge

因此这一刀真正完成的是：editor shell 最关键的两类高频 pointer route 已经不再只是“各自走 shared core”，而是开始在同一张 shared surface 上协同。

## UI Asset Editor Parent-Owned Slot Contracts

共享 layout/slot 语义现在已经被 `UI Asset Editor` 直接消费，而不是在 editor 侧再维护一套平行字段：

- [`UiChildMount.slot`](/E:/Git/ZirconEngine/zircon_ui/src/template/document.rs) 继续是父拥有的 placement 数据；editor inspector 现在按 parent container kind 切换字段，而不是按 child widget type 猜测
- `Overlay` slot 暴露 `layout.anchor / layout.pivot / layout.position / layout.z_index`
- `GridBox` slot 暴露 `row / column / row_span / column_span`
- `FlowBox` slot 暴露 `break_before / alignment`
- `ScrollableBox` 的 authoring 重点落在父容器 `layout.container`：`axis`、`gap`、`scrollbar_visibility` 和 `virtualization` 属于父配置；子项 slot 继续只保留通用 padding/size 等 shared fields
- 因为 `slot` 和 `mount` 本质上属于父容器协议，跨父级 reparent 时编辑器现在会显式清空旧 `mount` 与 `slot`，避免把 overlay/grid/flow 的陈旧 placement 语义带入新的 parent contract
- 稳定 `node_id` 现在同时承担 canonical source block 对位、selection restore 和 tree-command undo/redo 的主键；shared AST roundtrip 不再依赖文本偏移或运行时 child index

## What Still Remains Editor-Only

这次没有把 docking/workbench 语义错误地下沉到 runtime core。以下能力仍然属于 editor-only layer：

- `WorkbenchLayout` 持久化 schema
- drawer/document/floating window 拓扑
- region override 与活动 tab 相关的 content 默认约束选择
- splitter 交互的宿主手势状态
- `ActivityView` / `ActivityWindow` / exclusive page 等 editor pane 语义

换句话说，共享的是布局和命中基础，不共享 editor 页面模型。

## Why This Slice Matters

这一步把后续所有更重的工作提前钉死在正确边界上：

- Slint 不再拥有 `StretchMode`/`frame`/solver 的真源地位
- Slint 也不再拥有 editor shell drag target hit-test / dock target route 的真源地位
- editor workbench 不再持有一份和 runtime 平行演进的约束系统
- 事件系统以后可以直接依赖共享 tree/surface/hit-test/route/dispatcher，而不是先做 editor 壳专用路径
- 容器和滚动系统以后可以直接复用 shared measure/arrange、线性主轴求解、scroll state 与 virtual window 计算
- ECS bridge 以后接入 runtime UI 时，不需要再重新发明一套 layout frame 和命中缓存类型

它对应计划中的里程碑 1 和里程碑 2 的第一段闭环：先冻结共享边界和权威模型，再让 editor 现有 solver 开始消费 shared core。

## Current Validation

这轮继续重复确认过的验证包括：

- `cargo test -p zircon_ui shared_core -- --nocapture`
- `cargo test -p zircon_editor --lib slint_menu_pointer --locked -- --nocapture`
- `cargo test -p zircon_editor --lib resolve_floating_window_focus_instance_ --locked -- --nocapture`
- `cargo test -p zircon_ui --locked`
- `cargo test -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --lib viewport_ --offline -- --nocapture`
- `cargo check -p zircon_editor --lib --offline`

后续在同一工作区继续推进后，又额外通过了：

- `cargo test -p zircon_editor --locked`
- `cargo test --workspace --locked`
- `cargo test -p zircon_editor --lib builtin_workbench_template_bridge_recomputes_surface_backed_frames_with_shell_size --locked -- --nocapture`
- `cargo test -p zircon_editor --lib apply_presentation_uses_shared_root_projection_frames_when_drawers_are_collapsed --locked -- --nocapture`
- `cargo test -p zircon_editor --lib apply_presentation_keeps_geometry_document_region_when_drawers_are_visible --locked -- --nocapture`
- `cargo test -p zircon_editor --lib apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_positions --locked -- --nocapture`
- `cargo test -p zircon_editor --lib shared_drawer_header_pointer_layout_prefers_shared_root_projection_for_visible_drawer_regions --locked -- --nocapture`
- `cargo test -p zircon_editor --lib root_welcome_recent_pointer_click_uses_projection_fallback_in_real_host --locked -- --nocapture`
- `cargo test -p zircon_editor --lib shared_welcome_recent_pointer_bridge_scrolls_and_dispatches_remove_action --locked -- --nocapture`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_`
- `cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_`
- `cargo test -p zircon_editor --lib --locked editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo`
- `cargo test -p zircon_editor --lib --locked tests::host::slint_window::child_window_callback_wiring_tracks_source_window_for_pane_interactions`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_`
- `cargo check -p zircon_editor --lib --locked`

同一轮里，`root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed` 已经先给出真实 red 证据：在 cutover 前，real host 仍然把 viewport size 维持在 `1600x876`，而 root shell 已经展示 `1544x884` 的 shared viewport frame。production cutover 之后，`cargo check -p zircon_editor --lib --locked` 仍保持通过；新增的 pure-helper focused regression `resolve_workbench_drag_target_group_with_root_frames_uses_shared_root_projection_document_bounds_when_drawers_are_collapsed` 也已经 green。当前真正阻断更宽 `--lib` 单测复跑的，转为邻接 `ui_asset` lib-test 漂移，而不是这条 shared root projection cutover 本身。

当前已确认或由现有测试树持续覆盖的关键点包括：

- 共享主轴约束求解的优先级/权重行为
- 反向 measure / 正向 arrange 对 `desired_size`、stretch、anchor/pivot/position 的处理
- `Container` / `Overlay` / `Space` 作为共享 retained 容器名和 spacer 语义
- retained tree 的 layout dirty 冒泡停止条件
- 命中测试对 z-order、`UiInputPolicy` 和 clip 链的处理
- `HorizontalBox` / `VerticalBox` 的 gap、主轴分配和交叉轴 stretch 语义
- pointer capture、focus route 与 navigation root/root fallback 语义
- `UiNavigationDispatcher` 在 focused route 与 root fallback 上的冒泡顺序
- navigation handler 通过 `Focus(UiNodeId)` 触发的 shared focus handoff
- unhandled `Next` / `Previous` / `Up` / `Down` / `Left` / `Right` / `Activate` / `Cancel` 的 canonical fallback 语义
- capture 后移出命中范围时，dispatcher 仍会把 `Move` / `Up` 保持在 captured target
- `ScrollableBox` 的 content extent、viewport extent、visible window 与局部 scroll dirty 语义
- pointer dispatcher 对 handled / blocked / passthrough / captured 的分发结果
- shared pointer button payload 会进入 route 和 handler
- 未显式处理的 scroll 事件会命中最近的 shared scrollable container
- 虚拟化窗口对 offset / viewport / overscan 的范围计算
- viewport pointer/scroll host callback 现在会先经过 shared `UiSurface + UiPointerDispatcher` 再映射成 editor runtime viewport event
- viewport toolbar click 现在也会先经过 `ViewportToolbarPointerBridge` 的 shared `UiSurface + UiPointerDispatcher`，再进入 `BuiltinViewportToolbarTemplateBridge`
- `SceneViewportController` 的 gizmo / handle / renderable overlay 命中现在会先经过 `ViewportOverlayPointerRouter` 的 shared `UiSurface + UiPointerDispatcher`，再落成 `ViewportPointerRoute`
- menu/popup click 现在会先经过 `HostMenuPointerBridge` 的 shared `UiSurface + UiPointerDispatcher`，再进入 template-aware menu dispatcher
- asset references / used-by list click 和滚轮现在也会先经过 `AssetReferenceListPointerBridge` 的 shared `UiSurface + UiPointerDispatcher`，再进入 `AssetSurface/ActivateReference`
- real-host tab drop 现在不只共享 coarse route；`workspace_docking.rs` 会把 `root_shell_frames()` 继续传给 `resolve_workbench_tab_drop_route_with_root_frames(...)`，所以 tool/document tab strip 的 precise attach anchor 也服从 shared root projection，而不是退回旧 `WorkbenchShellGeometry`
- `Window` 菜单 popup 的滚轮输入现在由 shared `ScrollableBox + UiScrollState` 维护，而不是由 host-local scroll 语义持有
- `ConsolePane`、asset details rail 和 `InspectorPane` 现在都会先经过 `ScrollSurfacePointerBridge` 的 shared `UiSurface + UiPointerDispatcher`，再把 `scroll_offset` 回灌到宿主 `scroll_px`
- Rust-owned host callback wiring has removed viewport toolbar direct business callback forwarding; shared pointer route 成为这块 chrome 唯一允许的命中 authority
- drag overlay 与 splitter/full-screen resize capture 现在都只上传 pointer fact：
  - `workbench_drag_pointer_event(kind, x, y)`
  - `workbench_resize_pointer_event(kind, x, y)`
- `workspace_docking.rs` 现在统一把这些 pointer fact 接到 `WorkbenchShellPointerBridge`，再分别落成 normalized drop route 和 drawer resize dispatch；shared shell surface 已经成为这条 capture 链路的唯一命中 authority
- editor workbench 默认约束现在就是共享 `BoxConstraints`
- editor shell drag target route 现在由 host-owned `WorkbenchShellPointerBridge` 通过 shared `UiSurface + UiPointerDispatcher` 求解
- editor shell splitter route 现在也由同一个 `WorkbenchShellPointerBridge` 通过 shared `UiSurface + UiPointerDispatcher` 求解
- `UI Asset Editor` 现在直接消费 shared parent-owned slot contract：`Overlay`、`GridBox`、`FlowBox`、`ScrollableBox` 的 authoring 语义不再由 editor 私有表单决定
- tree-command reparent 现在会在 parent contract 改变时清空旧 `slot/mount`，保持 shared AST 对 panel-slot ownership 的约束
- stable `node_id` 现在支撑 source roundtrip block mapping、selection restore 和 tree-command undo/redo
- dock target normalization parity 现在又多了一层 shared-core-first 护栏：
  - `EditorUiCompatibilityHarness::capture_resolved_tab_drop_route_snapshot(...)` 会把 normalized route 压成稳定字符串摘要
- [`document_routes.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_tab_drag/document_routes.rs) 和 [`floating_routes.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_tab_drag/floating_routes.rs) 已覆盖“shared pointer route vs fallback group key”在 document edge split 和 floating-window attach 两种情况下产出等价 route snapshot
- floating-window focus parity 也已经进入 shared harness：
  - `EditorUiCompatibilityHarness::capture_event_journal_delta_snapshot(...)` 会把 runtime journal delta 压成稳定摘要
  - `slint_callback_dispatch/layout.rs` 现在直接比较 `dispatch_builtin_floating_window_focus(...)` 与 `LayoutCommand::FocusView` 的 journal delta，防止 shared route cutover 造成 focus event/effect 漂移
- bottom/right overlap 会按 shared route point 的边缘距离在 dispatcher 内做 `Handled` / `Passthrough` 分流
- Rust-owned host projection no longer holds `drag_target_group` formulas or exposes `drop_tab(...)` / `update_drag_target(...)` / `begin_drawer_resize(...)` / `update_drawer_resize(...)` / `finish_drawer_resize(...)` direct host callback ABI

这一轮新增并重新确认过的 focused 验证现在包括：

- `cargo test -p zircon_editor --lib asset_surface_controls_use_generic_template_callbacks_instead_of_legacy_business_abi --offline -- --nocapture`
- `cargo test -p zircon_editor --lib slint_asset_pointer --offline -- --nocapture`
- `cargo test -p zircon_editor --lib slint_callback_dispatch --offline -- --nocapture`
- `cargo check -p zircon_editor --lib --offline`
- `cargo test -p zircon_editor --lib shared_drag_capture_surface_replaces_legacy_direct_drop_callback_abi --locked -- --nocapture`
- `cargo test -p zircon_editor --lib slint_tab_drag --locked -- --nocapture`
- `cargo test -p zircon_editor --lib shared_resize_surface_replaces_legacy_direct_resize_callback_abi --locked -- --nocapture`
- `cargo test -p zircon_editor --lib slint_drawer_resize --locked -- --nocapture`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked -- --nocapture`
- `cargo check -p zircon_editor --lib --locked`
- direct run of the already-built `target/debug/deps/zircon_editor-*.exe` test binary for:
  - `tests::host::slint_callback_dispatch::layout::builtin_floating_window_focus_matches_legacy_layout_focus_dispatch_event_log`
  - `tests::host::slint_tab_drag::resolved_workbench_tab_drop_route_snapshot_matches_shared_pointer_and_group_key_for_document_edge`
  - `tests::host::slint_tab_drag::resolved_workbench_tab_drop_route_snapshot_matches_shared_pointer_and_group_key_for_floating_window`

当前本地仍然沿用 `--offline` 作为 editor host 切片的主要验证形态，因为工作区存在预先的 `Cargo.lock` 脏改动；但这次 shared-host asset control cutover 自身已经不再被先前那类中间态编译错误阻断。
这一轮另一个新的验证 caveat 是：后续 `cargo test -p zircon_editor --lib ...` 重跑被另一条活跃会话引入的 `zircon_graphics` 编译错误抢先阻断，因此新增的 floating-focus parity 用例只能复用刚刚编译成功的 `zircon_editor` 单测二进制直接执行；当前没有接管那条 graphics 模块拆分任务。

## Visible Drawer Document Region Cutover

当前 shared root projection 又收掉了一层 visible-drawer mixed authority：

- [`resolve_root_document_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 在 `Left/Right/Bottom` drawer 任一可见时，不再直接返回 `geometry.region_frame(Document)`；现在改为以 shared `WorkbenchBody` 为位置真源，只继续复用 legacy drawer extents
- 这条规则具体表现为：
  - `x` 由 shared body origin 加上 visible left drawer width 与水平 separator 计算
  - `width` 由 shared body width 扣掉 visible left/right drawer widths 与 separator 计算
  - `height` 由 shared body height 扣掉 visible bottom drawer height 与垂直 separator 计算
- [`resolve_root_document_tabs_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 在 visible drawer 下也开始复用同一份 resolved document frame，因此 root document tab pointer strip 不会继续回退到 stale document geometry `x/y/width`
- [`resolve_root_viewport_content_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 现在也会在 visible drawer 下从 resolved document frame 推导 viewport content，而不是继续信任 legacy viewport geometry
- [`apply_presentation_prefers_shared_root_projection_for_visible_drawer_document_region`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/tests.rs) 现在同时锁住 document shell 和 viewport frame：即使 legacy document / viewport frame 故意给出错误 `x/y/width/height`，root shell presentation 仍然必须服从 shared root/body + visible drawer extents 的组合结果

这一步之后，visible drawer 相关 root-shell seam 里仍刻意保留的只剩：

- drawer shell/header 主链已经不再保留 “shared origin + legacy main-axis extent” 过渡桥；visible left/right `width` 与 bottom `height` 都已经通过 builtin template/runtime 下发到 shared root projection
- 当前仍有意保留的 boundary 主要收敛到 shared frame 缺席时的 generic geometry fallback，以及 dynamic floating-window outer frame 仍由 editor-only layout/geometry 生成，再经 `floating_window_projection` helper 投给各个 consumer；visible drawer extent 和 root menu getter 已经不再是主边界

最新这一刀继续把这条 floating-window shared authority 从“统一 helper”推进到“统一 recompute-time bundle”：

- [`FloatingWindowProjectionBundle`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 现在在 host 重算期一次性缓存每个 floating window 的 `outer/tab/content` frame、解析后的 `host_frame`，以及 `native_host_present` 标记
- [`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 会在 `recompute_if_dirty()` 期间调用 `build_floating_window_projection_bundle(...)`，并把结果同时喂给 document-tab pointer、shell drag surface、pane callback-size fallback、floating window presentation 和 native presenter target collection
- 这让 shared core 在 editor host 里的 authority 又前进了一步：consumer 不再需要各自查询 `native_window_hosts()` 或各自决定是否信任 geometry；shared projection bundle 成为真实宿主里的单一 read-model
- focused validation 已重新确认这条 contract：
  - `cargo check -p zircon_editor --lib --locked`
  - `cargo test -p zircon_editor --lib floating_window --locked -- --nocapture`
  - `cargo test -p zircon_editor --lib shared_document_tab_pointer_layout_prefers_native_window_host_bounds_for_floating_strip --locked -- --nocapture`
  - `cargo test -p zircon_editor --lib shared_shell_pointer_route_prefers_native_window_host_bounds_for_floating_attach_surface --locked -- --nocapture`
- 当前仍保留的 boundary 只剩 bundle 底层 outer frame 的 producer 还不是 template/runtime 导出的 shared floating-window shell，而是 editor-owned `WorkbenchLayout/WorkbenchShellGeometry + WindowHostManager`

## Latest Floating-Window Dedicated Source

这一刀继续减少这条 boundary，但不是再借 root shell 节点，而是把 floating-window outer frame 的 base producer 改成 dedicated builtin template/runtime source。

- [`floating_window_source.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml) 新增一份只服务浮窗 placement 的 builtin 模板文档，直接导出 `FloatingWindowCenterBandRoot` 与 `FloatingWindowDocumentRoot`
- [`callback_dispatch/template_bridge/floating_window_source/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/mod.rs) 现在只保留 structural wiring；[`bridge.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs) 与 [`source_frames.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/source_frames.rs) 共同收口 `BuiltinFloatingWindowSourceTemplateBridge` / `BuiltinFloatingWindowSourceFrames` owner，shared source 不再解析 `DocumentHostRoot + WorkbenchBody`
- [`floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 继续只保留一份 `FloatingWindowProjectionSharedSource` 数学入口，但输入现在来自 `BuiltinFloatingWindowSourceFrames`
- [`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 会在 root recompute 时同步刷新 dedicated source bridge，再把这份 source 同时喂给 native-window projection bounds 和 `FloatingWindowProjectionBundle`

对 shared core 边界的实际含义现在是：

- `requested_frame` 仍属于 editor topology 输入
- `FloatingWindowDocumentRoot + FloatingWindowCenterBandRoot` 属于 dedicated shared layout/projection 输入
- `FloatingWindowProjectionBundle` 属于宿主消费的 read model
- root-shell builtin projection 不再是 floating-window source 的必经层
- 只有当 dedicated source 缺席时，`geometry.floating_window_frame(...)` 才继续作为 fallback

当前 widened validation 仍被无关工作区漂移阻断；不过真实阻塞点已经前移到 [`editing/ui_asset`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/mod.rs) 这轮未收敛的模块迁移，而不是本 slice 的 floating-window producer：

- `cargo check -p zircon_asset --lib --offline` 已经单独通过
- `cargo test -p zircon_editor --lib ...floating_window... --offline` 现在先失败在 `binding_inspector` wrapper include、`session.rs` / `session/mod.rs` 双入口、以及一整串 `ui_asset` wrapper/re-export/type-inference 错误


