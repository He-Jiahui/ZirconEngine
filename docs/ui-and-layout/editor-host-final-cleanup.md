---
related_code:
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/node_ids.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/resize_surface.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/tab_drag/bridge.rs
  - zircon_editor/src/ui/slint_host/tab_drag/drop_resolution.rs
  - zircon_editor/src/ui/slint_host/tab_drag/route_resolution.rs
  - zircon_editor/src/ui/slint_host/tab_drag/strip_hitbox.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/shell_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/workbench_tabs.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - tests/acceptance/ui-m8-final-cleanup-acceptance.md
  - zircon_editor/src/tests/host/slint_tab_drag/root_projection.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/pointer_bridge.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/resize_target.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/surface_contract.rs
  - zircon_editor/src/tests/host/slint_document_tab_pointer/dispatch.rs
  - zircon_editor/src/tests/host/slint_document_tab_pointer/floating_strip_bounds.rs
  - zircon_editor/src/tests/host/slint_tab_drag/document_routes.rs
  - zircon_editor/src/tests/host/slint_tab_drag/drag_target_groups.rs
  - zircon_editor/src/tests/host/slint_tab_drag/drop_resolution.rs
  - zircon_editor/src/tests/host/slint_tab_drag/floating_pointer.rs
  - zircon_editor/src/tests/host/slint_tab_drag/floating_routes.rs
  - zircon_editor/src/tests/host/slint_tab_drag/support.rs
  - zircon_editor/src/tests/host/slint_window/native_window_targets.rs
  - zircon_editor/src/tests/host/slint_window/ui_asset_editor.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs
  - zircon_editor/tests/integration_contracts/workbench_slint_shell.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
implementation_files:
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/node_ids.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/resize_surface.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/tab_drag/bridge.rs
  - zircon_editor/src/ui/slint_host/tab_drag/drop_resolution.rs
  - zircon_editor/src/ui/slint_host/tab_drag/route_resolution.rs
  - zircon_editor/src/ui/slint_host/tab_drag/strip_hitbox.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/shell_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/workbench_tabs.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - zircon_editor/src/tests/host/slint_tab_drag/root_projection.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/pointer_bridge.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/resize_target.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/surface_contract.rs
  - zircon_editor/src/tests/host/slint_document_tab_pointer/dispatch.rs
  - zircon_editor/src/tests/host/slint_document_tab_pointer/floating_strip_bounds.rs
  - zircon_editor/src/tests/host/slint_tab_drag/document_routes.rs
  - zircon_editor/src/tests/host/slint_tab_drag/drag_target_groups.rs
  - zircon_editor/src/tests/host/slint_tab_drag/drop_resolution.rs
  - zircon_editor/src/tests/host/slint_tab_drag/floating_pointer.rs
  - zircon_editor/src/tests/host/slint_tab_drag/floating_routes.rs
  - zircon_editor/src/tests/host/slint_tab_drag/support.rs
  - zircon_editor/src/tests/host/slint_window/native_window_targets.rs
  - zircon_editor/tests/integration_contracts/workbench_slint_shell.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
plan_sources:
  - user: 2026-04-18 下一步可以直接进入 Final cleanup
  - user: 2026-04-18 control-specific Slint callback/property glue 的继续 generic 化
  - user: 2026-04-18 pane 内部更细的 property/callback schema 继续 generic 化，以及更大面的 generic host boundary
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
tests:
  - zircon_editor/src/tests/host/slint_window/ui_asset_editor.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - zircon_editor/src/tests/host/slint_tab_drag/root_projection.rs
  - zircon_editor/src/tests/host/slint_tab_drag/document_routes.rs
  - zircon_editor/src/tests/host/slint_tab_drag/drag_target_groups.rs
  - zircon_editor/src/tests/host/slint_tab_drag/drop_resolution.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/pointer_bridge.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/resize_target.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/surface_contract.rs
  - tests/acceptance/ui-m8-final-cleanup-acceptance.md
  - zircon_editor/tests/workbench_slint_shell.rs
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_groups_string_selection_properties -- --exact
  - cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib slint_tab_drag::root_projection --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture
  - rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/shell_pointer/resize_surface.rs zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs zircon_editor/src/ui/slint_host/drawer_resize.rs zircon_editor/src/tests/host/slint_drawer_resize/pointer_bridge.rs zircon_editor/src/tests/host/slint_drawer_resize/resize_target.rs zircon_editor/src/tests/host/slint_drawer_resize/surface_contract.rs
  - cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture
  - rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/floating_window_projection.rs zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs zircon_editor/src/tests/host/slint_tab_drag/support.rs zircon_editor/src/tests/host/slint_document_tab_pointer/dispatch.rs zircon_editor/src/tests/host/slint_document_tab_pointer/floating_strip_bounds.rs zircon_editor/src/tests/host/slint_window/native_window_targets.rs zircon_editor/src/tests/host/slint_menu_pointer/visual_screenshot.rs zircon_editor/src/ui/slint_host/ui/tests.rs zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - cargo test -p zircon_editor --lib floating_window_projection --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib slint_tab_drag::floating_pointer --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib slint_window::native_window_targets --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib slint_document_tab_pointer --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture
  - rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs zircon_editor/src/ui/slint_host/tab_drag.rs zircon_editor/src/ui/slint_host/tab_drag/drop_resolution.rs zircon_editor/src/ui/slint_host/tab_drag/strip_hitbox.rs zircon_editor/src/tests/host/slint_tab_drag/support.rs zircon_editor/src/tests/host/slint_tab_drag/drag_target_groups.rs zircon_editor/src/tests/host/slint_tab_drag/document_routes.rs zircon_editor/src/tests/host/slint_tab_drag/drop_resolution.rs zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib runtime_ui_golden --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --ignored --nocapture --test-threads=1
  - cargo test -p zircon_editor tests::host::slint_window::ui_asset_editor_host_genericizes_collection_event_dispatch -- --exact
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_binding_inspector_editing_controls -- --exact
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_preview_activation_callback_and_double_click_binding -- --exact
  - cargo build --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo test --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never
  - cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-lib-fresh-m8 --message-format short --color never
  - cargo test --workspace --locked --jobs 1 --target-dir D:\cargo-targets\zircon-root-fresh-m8 --message-format short --color never
  - cargo test --manifest-path zircon_plugins\Cargo.toml --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_runtime --lib carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib material_icon_button_without_visual_icon_keeps_label_accessibility_only --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_fixture_assets_live_under_crate_assets --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never
  - cargo build --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never
  - cargo test --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never
  - cargo build --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-plugins --message-format short --color never
  - cargo test --workspace --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-plugins --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never
  - cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never -- --nocapture
  - rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/root_shell_projection.rs zircon_editor/src/ui/slint_host/shell_pointer/resize_surface.rs zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs zircon_editor/src/ui/slint_host/shell_pointer/node_ids.rs zircon_editor/src/ui/slint_host/drawer_resize.rs zircon_editor/src/ui/slint_host/app/host_lifecycle.rs zircon_editor/src/ui/slint_host/app/workspace_docking.rs zircon_editor/src/ui/slint_host/tab_drag/bridge.rs zircon_editor/src/ui/slint_host/tab_drag/drop_resolution.rs zircon_editor/src/ui/slint_host/tab_drag/route_resolution.rs zircon_editor/src/tests/host/slint_tab_drag/support.rs zircon_editor/src/tests/host/slint_tab_drag/drag_target_groups.rs zircon_editor/src/tests/host/slint_tab_drag/drop_resolution.rs zircon_editor/src/tests/host/slint_tab_drag/root_projection.rs zircon_editor/src/tests/host/slint_tab_drag/document_routes.rs zircon_editor/src/tests/host/slint_tab_drag/floating_routes.rs zircon_editor/src/tests/host/slint_tab_drag/floating_pointer.rs zircon_editor/src/tests/host/slint_drawer_resize/surface_contract.rs
doc_type: module-detail
---

# Editor Host Final Cleanup

## Purpose

这一轮只处理 `Final cleanup` 里已经能直接删除、且不会重新引入 product-level 设计分支的 editor host seam。目标不是再做一轮新架构，而是把已经完成 cutover 的 authority 真正从生产路径里落干净：

- 删掉只服务旧手写壳的 drawer extent bridge
- 删掉 menu button frame 的 root-host property glue
- 删掉 shell drag/document-tab 生产路径里对 floating-window geometry outer-frame fallback 的依赖
- 删掉 drawer resize hit surface 对 `WorkbenchShellGeometry.splitter_frames` 的重复坐标 truth source
- 把 UiAssetEditor root/host 层 control-specific selected/activated callback glue 收口成 generic collection event boundary
- 把 UiAssetEditor pane 内部重复的 collection callback 与 `items + selected_index` property 对收口成更 generic 的局部 schema

## What Changed

### Drawer extent bridge removed from presentation

`ShellPresentation`、`apply_presentation(...)` 和 Rust-owned host contract root 不再维护 `left/right/bottom_drawer_extent` 这组三元属性。

这条桥之前已经不再提供真正的布局 authority，只是继续把 drawer extent 从 snapshot/presentation 侧透传到 former host root。现在：

- `ShellPresentation` 不再计算 drawer extent
- `apply_presentation(...)` 不再调用 `set_left/right/bottom_drawer_extent(...)`
- `UiHostWindow` / `WorkbenchHostScaffold` root binding 里也不再暴露这组三元 property
- `workbench_tabs.rs` 里的 `drawer_extent(...)` helper 已删除

也就是说，drawer 可见性与主轴尺寸现在只剩 shared template/runtime source 和实际 resolved root frames 两条事实来源，不再保留一个 presentation-only extent alias。

### Menu popup host setters removed

`app/pointer_layout.rs` 已删除对：

- `set_file_menu_button_frame(...)`
- `set_edit_menu_button_frame(...)`
- `set_selection_menu_button_frame(...)`
- `set_view_menu_button_frame(...)`
- `set_window_menu_button_frame(...)`
- `set_help_menu_button_frame(...)`

这批 setter 的调用。

同时 Rust-owned root `UiHostWindow` / `host_contract` orchestrator 不再通过 `*_menu_button_frame` 暴露 control-specific frame 宿主 ABI。

当前 popup 视觉锚点已经收进 Rust-owned `HostMenuChromeData` / template projection 输出，因此 UI 外观不需要再依赖 root/scaffold 代理 frame；menu hit-test / popup open state 继续由 shared `HostMenuPointerLayout` 和 pointer bridge 决定。

### Floating-window geometry fallback removed from production consumers

生产路径里最关键的两条 floating-window 消费链已经不再在 bundle 缺席时自行退回 `geometry.floating_window_frame(...)`：

- `shell_pointer/drag_surface.rs`
- `document_tab_pointer/build_workbench_document_tab_pointer_layout.rs`

现在这两条链都只消费 `FloatingWindowProjectionBundle`：

- bundle 有 frame 就使用 shared projected outer/tab-strip frame
- bundle 没 frame 就当作没有可交互浮窗 surface，而不是重新从 geometry 拼一层兼容结果

`floating_window_projection.rs` 的 test-only helper 也已经收口：它现在接收显式 `FloatingWindowProjectionSharedSource`，不再从 `WorkbenchShellGeometry.region_frame(...)`、`center_band_frame` 或 `floating_window_frame(...)` 反推 source/frame。`HostShellPointerBridge` 的 test update helpers 会为 floating drag surface 构造 projection bundle，但只从 requested frame、native host bounds 或显式 shared source 取值，不会再复活旧 geometry fallback。

这意味着 floating-window projection 的生产路径和模块内测试入口现在都落在同一套 shared-source / requested-frame / native-host-frame 规则上；旧几何对象只允许作为 stale fixture 传入其它宿主测试，用来证明 shared projection 会覆盖它。

### Shell pointer and tab-strip geometry fallback removed

这一轮继续把 root-shell shared frame cutover 往剩余 pointer 路径推进。`HostShellPointerBridge::update_layout_with_root_shell_frames(...)` 不再在 `shared_root_frames` 缺席时从 `WorkbenchShellGeometry` 临时重建 `BuiltinHostRootShellFrames`，因此 bridge 不再把旧 geometry 重新包装成新的 shared source。

`shell_pointer/drag_surface.rs` 现在不接收 geometry 参数。document、left、right、bottom、center band、status bar 和 floating-window hit frame 都只来自：

- `BuiltinHostRootShellFrames`
- `FloatingWindowProjectionBundle`
- root size clamp

如果 shared root frames 缺席，drag surface 不会再静默读旧 `center_band_frame`、`status_bar_frame`、`region_frame(...)` 或 `floating_window_frame(...)`。

`tab_drag/strip_hitbox.rs` 同步收口：精确 tab-strip 命中只用 root-shell resolver 取得 document/left/right/bottom/center frame。`drop_resolution.rs` 和 `route_resolution.rs` 的当前入口直接接收 `BuiltinHostRootShellFrames`，不再保留旧 geometry 参数；没有 shared frames 时只会走 layout/model 层的粗粒度 fallback host resolution，而不会恢复旧坐标表。

### Root-shell helper fallback removed from projection and callback sizing

`root_shell_projection.rs` 现在已经不再把 shared root frame 和 `WorkbenchShellGeometry` 混读拼接。

当前 root shell 相关 helper 的 authority 只剩 shared projection frame：

- `center_band/status_bar/document/left/right/bottom` 全部直接取 `BuiltinWorkbenchRootShellFrames`
- activity rail 只从 shared activity rail frame 或 shared left drawer shell frame 推导
- document tabs / viewport content 只从 shared `document_host_frame` / `pane_surface_frame` 推导
- 不再存在 `geometry.region_frame(...)`、`geometry.viewport_content_frame`、`legacy_root_activity_rail_frame(...)` 这类 legacy fallback helper

与之对应，几个仍会为 pointer callback / viewport toolbar / drawer resize 推导尺寸的宿主 helper 也同步去掉了 legacy geometry 兜底：

- `app/helpers.rs` 的 callback surface size 只看 floating projection content frame、drawer content frame、pane surface/document host shared frame
- `app/viewport.rs` 的 toolbar width 只看 shared pane/drawer frame，不再读 document/drawer geometry 宽度
- `app/workspace_docking.rs` 的 drawer resize capture 不再保留 document-region geometry fallback
- `shell_pointer/resize_surface.rs` 的 drawer resize hit targets 只通过 root-shell projection splitter resolver 读取 `BuiltinHostRootShellFrames`，不再直接读 `geometry.splitter_frame(...)`

这一步的意义不是“换个地方再算一遍旧几何”，而是让 root shell frame 消费点真正收口到 shared projection 输出；如果 shared frame 缺席，行为会退成 empty/default，而不是静默回到旧手写壳布局。

### UiAssetEditor collection callback glue genericized at the root/host boundary

Rust-owned host contract、`callback_wiring.rs` 和 `ui_asset_editor.rs` 这条链现在不再把 UiAssetEditor 的 collection selection/activation 语义拆成一组 root callback 名字。

之前 root/host ABI 仍然暴露并逐层转发一批 control-specific callback，例如：

- `ui_asset_matched_style_rule_selected`
- `ui_asset_palette_selected`
- `ui_asset_palette_target_candidate_selected`
- `ui_asset_hierarchy_selected` / `ui_asset_hierarchy_activated`
- `ui_asset_preview_selected` / `ui_asset_preview_activated`
- `ui_asset_source_outline_selected`
- `ui_asset_preview_mock_selected`
- `ui_asset_binding_selected`
- `ui_asset_binding_event_selected`
- `ui_asset_binding_action_kind_selected`
- `ui_asset_binding_payload_selected`
- `ui_asset_slot_semantic_selected`
- `ui_asset_layout_semantic_selected`

现在这些 root/host 专用 callback 已收口为单一 ABI：

- `ui_asset_collection_event(instance_id, collection_id, event_kind, item_index)`

Host-contract side changes：

- `UiHostWindow` / Rust-owned scaffold / native floating forwarding block 都只暴露并转发 `ui_asset_collection_event(...)`
- `UiAssetEditorPane` 内部仍保留 `matched_style_rule_selected(...)`、`binding_selected(...)` 这类 pane-local callback，但它们不再把语义编码进 host callback 名字，而是统一映射成 `collection_id + event_kind`

Rust 侧的变化：

- `callback_wiring.rs` 只监听 `ui.on_ui_asset_collection_event(...)`
- `ui_asset_editor.rs` 删除了那批 `dispatch_ui_asset_*selected/activated(...)` 方法，改成单一 `dispatch_ui_asset_collection_event(...)`
- host dispatch 通过 `(collection_id, event_kind)` match 继续调用原有 `editor_manager` 选择/激活入口，因此业务行为没有被重新发明，只是 callback ABI 不再按 control 名字膨胀

这一步让 host ABI 不再因为某个 list/view 新增就继续膨胀，但还没有减少 `UiAssetEditorPane { ... }` 绑定块里那一大串重复的 `items + selected_index` 映射，也没有统一 pane-local callback 面。

### UiAssetEditor pane-local collection schema further genericized

这一轮继续把 generic 化往 pane 里面推进，目标是削掉 `UiAssetEditorPane` 内部仍残留的 control-specific callback/property glue，而不是只停在 root/host ABI。

Host-contract side originally added one reusable struct:

- `UiAssetStringSelectionData { items, selected_index }`

目前已切到这套 grouped property 的 collection 包括：

- `palette_collection`
- `hierarchy_collection`
- `preview_collection`
- `source_outline_collection`
- `preview_mock_collection`
- `theme_source_collection`
- `style_matched_rule_collection`
- `inspector_slot_semantic_collection`
- `inspector_layout_semantic_collection`
- `inspector_binding_collection`
- `inspector_binding_event_collection`
- `inspector_binding_action_kind_collection`
- `inspector_binding_payload_collection`

对应变化：

- Rust-owned `UiAssetEditorPane` projection no longer passes each `xxx_items + xxx_selected_index` pair individually; it builds grouped selection objects before forwarding them to the pane surface
- The pane-local `UiAssetEditorPane` contract no longer declares `palette_selected(...)`、`binding_selected(...)`、`layout_semantic_selected(...)` callback fan-out; it routes them through `collection_event(collection_id, event_kind, item_index)`
- `UiAssetSelectableSection`、preview canvas、sticky palette target chooser、matched-rule/semantic/binding/mock-preview surface 都改为在 pane 内直接发 `root.collection_event(...)`
- `palette_has_selection`、binding action-kind 路由文本、payload delete enable、semantic clear enable 这类依赖 selected index 的局部逻辑，全部改成读取 grouped selection object，而不是继续依赖散落的 `*_selected_index` property

这一步的价值不是“少几行 host glue”，而是把 `UiAssetEditorPane` 的内部契约从“每个控件一种 callback/property 名字”收口成“复用 selection data + generic collection event”两条更稳定的局部边界。后续 2026-04-30 fence 已把 deleted Slint shell copies 降为 non-authoritative；这块当前只允许通过 `.ui.toml` assets 与 Rust-owned `host_contract` projection 继续收敛。

## M8 Acceptance Evidence Index

2026-05-07 M8.2 继续推进时新增 `tests/acceptance/ui-m8-final-cleanup-acceptance.md`，作为 final cleanup 的证据索引，而不是最终完成声明。

该索引把四类现有证据放到同一个入口：

- M8.1 stale coordinate cleanup：root-shell projection、drawer resize splitter、floating-window projection、shell pointer / tab-strip root-frame fallback removal。
- M4.3/M4.T editor/runtime `.ui.toml` semantic golden：`runtime_ui_golden.rs` 覆盖 HUD、pause、settings、inventory、quest log 五组 editor/runtime fixture pair。
- M7 Debug Reflector / runtime diagnostics snapshot：`ui-debug-reflector-full-closure.md` 与 `ui-m7-invalidation-performance.md` 中记录的 reflector、diagnostics、hit-grid、presenter/top-right marker gates。
- M3 GUI screenshot artifacts：`target/visual-layout/editor-window-m3-*.png` 中的 Welcome、Workbench、Asset Browser、drawer、menu popup/SVG、drag-release、small/large SVG scale artifact。
- 2026-05-07 M8.2 fresh focused gates：`runtime_ui_golden` 2 / 0，editor `ui_debug_reflector` 14 / 0，runtime `diagnostics` 17 / 0，runtime `hit_grid` 12 / 0，GUI screenshot gate 1 / 0；八张 `editor-window-m3-*.png` artifact 刷新到 2026-05-07 12:08-12:09 +08:00。
- 2026-05-07 M8.3 broad gates：root workspace build 0、root workspace test rerun 0、plugin workspace check/build/test 0/0/0，原始宽门槛跑在 `D:\cargo-targets\zircon-m8-current`；最终隔离目标复跑为 `zircon_editor --lib` 1115 / 0 / 3 ignored、root workspace test 0、plugin workspace test 0。

当前边界：这些证据现在可以支撑 M8 final cleanup 完成结论。宽门槛中暴露过两个问题：`zircon_runtime_interface` dependency boundary allowlist 滞后，已在最低共享层修复并通过 focused test 与完整 root workspace test rerun；旧 target 目录还一度让 editor lib-test 看到两个 `zircon_runtime_interface` 实例，已用全新 target 复跑 1115 个 editor lib 测试证明不是源码合同分裂。

## Current Boundary After Cleanup

做完这一轮以后，editor host 在这块的剩余边界变成：

- root shell / menu / pane / callback ABI cleanup now targets Rust-owned `host_contract` projection and `.ui.toml` assets; deleted Slint shell copies are not current authority
- floating-window projection helper 不再允许从 geometry 合成 shared source；需要浮窗 source 的测试必须显式传入 `FloatingWindowProjectionSharedSource` 或通过 `BuiltinFloatingWindowSourceTemplateBridge` 取得 source frames
- UiAssetEditor pane 内部的 collection callback 与主要 string-selection property 已经 generic 化，但仍有 detail scalar、action-specific callback 和 business scaffold cleanup to finish in Rust-owned projection / `.ui.toml` assets

但本轮已经把三条最直接的 legacy seam 从生产路径里撤掉：

- no drawer extent root binding
- no menu button frame host setter/binding
- no floating drag/document-tab geometry outer-frame fallback
- no root-shell helper geometry fallback for presentation / callback sizing / toolbar sizing / resize capture
- no shell pointer bridge geometry-to-root-frame rewrap
- no tab-drag strip hitbox geometry fallback
- no root/host `UiAssetEditor` selected/activated callback fan-out
- no pane-local `UiAssetEditor` collection callback fan-out
- no repeated pane-local `items + selected_index` glue for the main string-collection surfaces

## Validation

已完成的直接验证：

- 源码检索确认生产代码已不再出现 `set_*_menu_button_frame(...)`、`left/right/bottom_drawer_extent` root binding，以及 `drag_surface.rs` 中的 `resolve_floating_window_outer_frame(geometry, ...)` fallback
- 源码检索确认 `root_shell_projection.rs`、`app/helpers.rs`、`app/viewport.rs`、`app/workspace_docking.rs` 已不再出现 root-shell geometry fallback 字符串
- 源码检索确认 `shell_pointer/resize_surface.rs` 已不再出现 `.splitter_frame(`，drawer resize hit targets 改由 root-shell shared splitter resolver 输出
- 源码检索确认 `floating_window_projection.rs` 已不再出现 `WorkbenchShellGeometry`、`geometry.region_frame`、`geometry.center_band_frame`、`.floating_window_frame(` 或 geometry-backed projection helper
- 源码检索确认生产 `slint_host` 不再出现 root-frame 旧回退模式：`geometry.region_frame`、`geometry.splitter_frame`、`geometry.center_band_frame`、`geometry.status_bar_frame`、`geometry.viewport_content_frame`、`geometry.floating_window_frame`、`WorkbenchShellGeometry {`、`root_frames_from_geometry`、`shared_or_geometry_frame`、`shared_or_fallback_frame`
- 2026-05-07 11:26 +08:00 fresh audit 确认 production 非测试 `zircon_editor/src/ui/slint_host/**/*.rs` 排除 `tests.rs` 后，对旧 geometry/root-frame fallback 扩展模式仍为 0 命中；production `slint_host` 与 `assets/ui/editor/host` 对 drawer extent、menu button frame setter、旧 UiAssetEditor control-specific root/pane callback 模式也为 0 命中。`app/tests.rs` / `ui/tests.rs` 中保留 21 个旧 geometry fixture 命中，只作为 stale geometry regression 输入。
- 源码检索确认 Rust-owned host contract root callback 已只剩 `ui_asset_collection_event(...)` 这条 UiAssetEditor collection ABI，不再出现 root-level `ui_asset_*selected/activated` callback declaration
- 源码检索确认 `UiAssetEditorPane` 已只保留 `collection_event(...)` 这条 pane-local collection callback，不再声明 `palette_selected(...)` / `binding_selected(...)` / `layout_semantic_selected(...)` 这类 callback
- 源码检索确认 `UiAssetEditorPane` 的主要 string collection 已切到 `UiAssetStringSelectionData`，Rust-owned projection 不再逐个绑定这些 collection 的 `items + selected_index`
- 新增/更新了对应 source guard：
  - `ui_asset_editor_pane_genericizes_collection_event_boundary`
  - `ui_asset_editor_pane_groups_string_selection_properties`
  - `ui_asset_editor_host_genericizes_collection_event_dispatch`
  - `ui_asset_editor_pane_declares_binding_inspector_editing_controls`
  - `ui_asset_editor_pane_declares_preview_activation_callback_and_double_click_binding`
  - `workbench_root_shell_projection_uses_shared_frames_without_geometry_fallback`
  - `floating_window_projection_uses_shared_source_without_geometry_fallback`
  - `shared_resize_surface_uses_rust_owned_pointer_event_contract`
  - `shell_pointer_bridge_does_not_recreate_root_frames_from_geometry`
  - `shell_pointer_drag_surface_uses_shared_root_frames_without_geometry_fallback`
  - `tab_drag_strip_hitbox_uses_shared_root_frames_without_geometry_fallback`
- 已通过的 Cargo 级验证：
  - `cargo check -p zircon_editor --lib --locked`
  - `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_groups_string_selection_properties -- --exact`
  - `cargo test -p zircon_editor tests::host::slint_window::ui_asset_editor_host_genericizes_collection_event_dispatch -- --exact`
  - `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_binding_inspector_editing_controls -- --exact`
  - `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_preview_activation_callback_and_double_click_binding -- --exact`
  - `rustfmt --edition 2021 --check zircon_editor/src/tests/host/slint_tab_drag/root_projection.rs zircon_editor/src/ui/slint_host/root_shell_projection.rs zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs`
  - `cargo test -p zircon_editor --lib workbench_root_shell_projection_uses_shared_frames_without_geometry_fallback --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture`
  - `cargo test -p zircon_editor --lib slint_tab_drag::root_projection --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture`
  - `cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture`
  - `rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/shell_pointer/resize_surface.rs zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs zircon_editor/src/ui/slint_host/drawer_resize.rs zircon_editor/src/tests/host/slint_drawer_resize/pointer_bridge.rs zircon_editor/src/tests/host/slint_drawer_resize/resize_target.rs zircon_editor/src/tests/host/slint_drawer_resize/surface_contract.rs`
  - `cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture`
  - `rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/floating_window_projection.rs zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs zircon_editor/src/tests/host/slint_tab_drag/support.rs zircon_editor/src/tests/host/slint_document_tab_pointer/dispatch.rs zircon_editor/src/tests/host/slint_document_tab_pointer/floating_strip_bounds.rs zircon_editor/src/tests/host/slint_window/native_window_targets.rs zircon_editor/src/tests/host/slint_menu_pointer/visual_screenshot.rs zircon_editor/src/ui/slint_host/ui/tests.rs zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs`
  - `cargo test -p zircon_editor --lib floating_window_projection --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture`
  - `cargo test -p zircon_editor --lib slint_tab_drag::floating_pointer --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture`
  - `cargo test -p zircon_editor --lib slint_window::native_window_targets --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture --test-threads=1`
  - `cargo test -p zircon_editor --lib slint_document_tab_pointer --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture --test-threads=1`
  - `rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs zircon_editor/src/ui/slint_host/tab_drag.rs zircon_editor/src/ui/slint_host/tab_drag/drop_resolution.rs zircon_editor/src/ui/slint_host/tab_drag/strip_hitbox.rs zircon_editor/src/tests/host/slint_tab_drag/support.rs zircon_editor/src/tests/host/slint_tab_drag/drag_target_groups.rs zircon_editor/src/tests/host/slint_tab_drag/document_routes.rs zircon_editor/src/tests/host/slint_tab_drag/drop_resolution.rs zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs`
  - `cargo test -p zircon_editor --lib workbench_projection_cutover --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture`
  - `cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture --test-threads=1`
  - `cargo test -p zircon_editor --lib slint_drawer_resize --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8 --message-format short --color never -- --nocapture`
  - `cargo clean --target-dir E:\zircon-build\targets-ui-m8-current` after E: free space dropped below 50GB; removed 23.6GiB and restored E: free space to 57.9GB
  - `cargo test -p zircon_editor --lib runtime_ui_golden --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture`
  - `cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture`
  - `cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture`
  - `cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --nocapture`
  - `cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m8-current --message-format short --color never -- --ignored --nocapture --test-threads=1`
  - `cargo build --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`，退出码 0，用时 18m33s
  - 初次 `cargo test --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never` 暴露 `zircon_runtime_interface` boundary allowlist 未包含 `unicode-segmentation`
  - `rustfmt --edition 2021 --check zircon_runtime_interface/src/tests/boundary.rs`
  - `cargo test -p zircon_runtime_interface --lib manifest_dependencies_stay_contract_only --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never -- --nocapture`，1 passed / 0 failed
  - rerun `cargo test --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`，退出码 0
  - `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`，退出码 0，用时 9m42s
  - `cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`，退出码 0，用时 14m37s
  - `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1 --target-dir D:\cargo-targets\zircon-m8-current --message-format short --color never`，退出码 0，用时 11m38s

2026-05-07 M8.1a 更新：`root_shell_projection.rs` 已删除 `WorkbenchShellGeometry` 派生 layout frame、region frame、center/status frame 和 splitter frame fallback helper。root shell presentation、drag/drop、document tab strip、viewport content 和 splitter frame resolver 现在只消费 `BuiltinHostRootShellFrames`；shared frame 缺席时返回 empty/default frame，不再静默回到旧手写壳 geometry。对应行为用例里的 root projection fixture 也改为显式传入 `document_host_frame`、`right_drawer_shell_frame` 或 `bottom_drawer_shell_frame`，避免测试继续依赖旧几何真源。

2026-05-07 M8.1a.2 更新：`shell_pointer/resize_surface.rs` 不再从 `WorkbenchShellGeometry.splitter_frames` 建立 drawer resize 命中面，而是通过 `resolve_root_left/right/bottom_splitter_frame(...)` 共享 root-shell projection 结果。`HostShellPointerBridge::update_layout_with_root_shell_frames(...)` 将 `BuiltinHostRootShellFrames` 同步传给 drag surface 和 resize surface；旧的 test-only `update_layout(...)` / `resolve_host_resize_target_group(...)` 无 shared frame fallback 已删除。对应 drawer resize fixture 现在清空 `splitter_frames`，显式传入 shared root frames，保证 hit-test/capture 用例不会继续依赖旧 splitter 坐标表。

2026-05-07 M8.1a.3 更新：`floating_window_projection.rs` 删除了 `floating_window_projection_shared_source_from_geometry(...)`、geometry-backed bundle builder、`resolve_floating_window_*_frame(...)` legacy helpers 和 `resolve_floating_window_projected_outer_frame_with_fallback(...)`。浮窗 bundle 现在只从 `FloatingWindowProjectionSharedSource`、有效 requested frame 或 native host bounds 取 outer frame；source guard `floating_window_projection_uses_shared_source_without_geometry_fallback` 固定禁止 `WorkbenchShellGeometry`、`.floating_window_frame(`、`geometry.region_frame` 和 `geometry.center_band_frame` 回流。相关 document-tab、floating drag pointer 和 native-window target tests 已改为传 `None`/显式 shared source/native host bounds，并用 focused gates 验证行为不回退旧坐标表。

2026-05-07 M8.1a.4 更新：`shell_pointer/bridge.rs` 删除了 `root_frames_from_geometry(...)`，不再把缺席的 shared root frames 用旧 `WorkbenchShellGeometry` 重建。`shell_pointer/drag_surface.rs` 不再接收 geometry，root/document/drawer/floating drag frames 只来自 shared root frames 和 floating projection bundle。`tab_drag/strip_hitbox.rs` 的精确命中也只消费 shared root frame resolver；`drop_resolution.rs` / `route_resolution.rs` 已删除旧 geometry 参数，当前只接收 shared root frames。`workbench_projection_cutover` 现在用 source guards 固定 bridge、drag surface、strip hitbox 三条路径不恢复旧坐标真源。

2026-05-07 M8.1a.5 更新：fresh stale-path production audit 已把 production 非测试路径和 test fixture 分桶。production 路径对旧 geometry/root-frame fallback、drawer/menu root binding、UiAssetEditor control-specific callback/property 模式均为 0 命中；保留下来的旧 geometry 字符串只在模块内 `tests.rs` 中，用来制造 stale input 并证明 shared root/projection frames 是最终 authority。

2026-05-07 M8.2a 更新：fresh focused final gates 已跑完。`runtime_ui_golden` 2 passed / 0 failed，editor `ui_debug_reflector` 14 passed / 0 failed，runtime `diagnostics` 17 passed / 0 failed，runtime `hit_grid` 12 passed / 0 failed，ignored GUI screenshot gate 1 passed / 0 failed。`target/visual-layout/editor-window-m3-*.png` 八张 artifact 刷新到 2026-05-07 12:08-12:09 +08:00。既有 runtime/editor warning noise 保留，不作为 M8.2 focused gate 失败。

2026-05-07 M8.3/M8.T 更新：broad root/plugin gates 已跑完。初次 root workspace test 失败只暴露 `zircon_runtime_interface` boundary allowlist 与 M6 text contract dependency 的记录不同步；修复 `boundary.rs` 后，focused boundary test 与完整 root workspace test rerun 均通过。后续旧 target 目录又暴露过一次 false-positive editor lib-test type split，清理污染目标后在 `D:\cargo-targets\zircon-editor-lib-fresh-m8` 复跑 editor lib 测试通过 1115 / 0 / 3 ignored，并在 `D:\cargo-targets\zircon-root-fresh-m8` 复跑 root workspace test 通过。Plugin workspace test 在 `D:\cargo-targets\zircon-plugins-m8-final` 通过；导航 runtime 的 carved-obstacle fixture 注释清理后 focused test 也通过。M8 final cleanup 没有剩余阻塞 owner。

2026-05-07 15:45 +08:00 current-target 复核：为避免低磁盘空间造成假失败，先清理已解析到 `E:\zircon-build` 下的旧 `targets` 与 `targets-ui-m6`，共释放 55.0GiB。随后在 `E:\zircon-build\targets-ui-m8-current` 复跑 runtime fixture/material 两个 focused regressions，分别 1 / 0；`zircon_runtime` lib 944 / 0；root workspace `cargo build --workspace --locked --jobs 1` 与 `cargo test --workspace --locked --jobs 1` 均退出码 0。在 `zircon_plugins` 工作区用 `E:\zircon-build\targets-ui-m8-plugins` 复跑 `cargo build --workspace --locked --jobs 1` 和 `cargo test --workspace --locked --jobs 1`，均退出码 0。该复核没有改变 M8.T 结论，只把本轮当前源码和 E-target evidence 补到 acceptance index。

2026-05-07 19:10-19:38 +08:00 scoped continuation 复核：因为 E: 当前空闲空间低于 50GB Cargo target policy 阈值，改用 `D:\cargo-targets\zircon-ui-m8-current` 重新验证 M8 shell/root-frame cleanup。`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m8-current --message-format short --color never` 通过，说明此前记录的 `UiDesignerToolMode` 阻塞已不在当前源码中。随后 focused gates：`slint_drawer_resize` 9 / 0，`workbench_projection_cutover` 8 / 0，`slint_tab_drag` 34 / 0。最终 `rustfmt --edition 2021 --check` 覆盖本轮 root-shell、shell-pointer、tab-drag、drawer-resize 源码和 focused tests，通过无输出；targeted production source guards 对 `root_shell_projection.rs`、`shell_pointer/*`、`tab_drag/*`、`floating_window_projection.rs`、`drawer_resize.rs`、`workspace_docking.rs` 中旧 geometry/root-frame fallback 模式均为 0 命中。宽 grep 仍只在 `app/tests.rs` / `ui/tests.rs` 中看到旧 geometry fixture 字符串，不计为 production 残留。

当前验证中仍然存在的非阻塞信号：

- 多个 `Cargo.toml` 里的 `toml = 1.1.2+spec-1.1.0` 继续触发 semver metadata warning
- `zircon_editor` 里仍有一批与本轮 cleanup 无关的 `unused import` / `dead code` warning
- Cargo target 目录在多轮宽验证后可能混入旧图产物；最终门槛应继续使用隔离 target，并且只清理已解析到预期 target 根下的目录
- E: 当前低于 50GB Cargo target policy 阈值；后续验证应继续使用空间充足的 D: target，或先清理确认位于预期根下的 E: target 目录

因此这轮 closeout 的验证结论是：

- 本轮 callback glue cleanup 与 root-shell geometry fallback cleanup 都已经拿到 source guard、Cargo 级验证和 fresh production stale-path audit
- `Final cleanup` 里的 `UiAssetEditor` pane-local callback/property glue 已继续往 generic schema 收口
- `Final cleanup` 的 editor/runtime golden、debug snapshot、GUI screenshot focused fresh gates 已有当前线程可捕获证据
- `Final cleanup` 的 root workspace CI-shape 与 plugin workspace CI-shape gates 已通过；M8.T 仅保留既有 warning noise、共享 dirty worktree 和后续 Cargo 目标目录空间管理作为非阻塞风险
