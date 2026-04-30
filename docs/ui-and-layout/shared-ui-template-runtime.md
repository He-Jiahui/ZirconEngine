---
related_code:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/FiraMono-subset.ttf
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/asset/project/manager/asset_kind.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/surface/render/mod.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/resolved_style.rs
  - zircon_runtime/src/ui/surface/render/text_layout.rs
  - zircon_runtime/src/ui/surface/render/typography.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/geometry.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/build/mod.rs
  - zircon_runtime/src/ui/template/build/interaction.rs
  - zircon_runtime/src/ui/template/document.rs
  - zircon_runtime/src/ui/template/loader.rs
  - zircon_runtime/src/ui/template/validate.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - zircon_runtime/tests/font_asset_manifest_contract.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
  - zircon_runtime/src/ui/tests/template.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_editor/src/ui/template/mod.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/template_runtime/structure_split.rs
  - zircon_editor/src/tests/ui/template/mod.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/ui/boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/layout.rs
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_popup.ui.toml
  - zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml
  - zircon_editor/assets/ui/editor/workbench_status_bar.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml
  - zircon_editor/assets/ui/editor/host/hierarchy_body.ui.toml
  - zircon_editor/assets/ui/editor/host/module_plugins_body.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.ui.toml
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_components.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_scene.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/template_nodes.rs
  - zircon_editor/tests/integration_contracts/workbench_slint_shell.rs
  - zircon_editor/tests/integration_contracts/workbench_window_resize.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_chrome_metrics.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/slint_host/host_page_pointer/constants.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/menu_items_for_layout.rs
  - zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/activity_rail.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs
  - zircon_editor/build.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/drawer_source_projection.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs
  - zircon_editor/src/tests/host/slint_window/activity_rail_template_boundary.rs
  - zircon_editor/src/tests/host/slint_tab_drag/surface_contract.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/surface_contract.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/dispatcher.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/visual_screenshot.rs
implementation_files:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/FiraMono-subset.ttf
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/asset/project/manager/asset_kind.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/surface/render/mod.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/resolved_style.rs
  - zircon_runtime/src/ui/surface/render/text_layout.rs
  - zircon_runtime/src/ui/surface/render/typography.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/geometry.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/build/mod.rs
  - zircon_runtime/src/ui/template/build/interaction.rs
  - zircon_runtime/src/ui/template/document.rs
  - zircon_runtime/src/ui/template/loader.rs
  - zircon_runtime/src/ui/template/validate.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - zircon_runtime/tests/font_asset_manifest_contract.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
  - zircon_runtime/src/ui/tests/template.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/template_runtime/structure_split.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/layout.rs
  - zircon_editor/assets/ui/editor/host/hierarchy_body.ui.toml
  - zircon_editor/assets/ui/editor/host/module_plugins_body.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_popup.ui.toml
  - zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml
  - zircon_editor/assets/ui/editor/workbench_status_bar.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.ui.toml
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_chrome_metrics.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/slint_host/host_page_pointer/constants.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_components.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_scene.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/template_nodes.rs
  - zircon_editor/build.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/menu_items_for_layout.rs
  - zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/activity_rail.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/drawer_source_projection.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs
plan_sources:
  - user: 2026-04-15 按自定义 TOML 描述文件运行时构建 Slint 树并严格服从 Shared Layout 契约
  - user: 2026-04-15 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-18 继续下一步，推进 Runtime visual contract
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-20 不要 re-export，直接清理 core 里 ui 部分
  - user: 2026-04-21 M1 主链收口与文本底座计划，模板样式补齐 typography 字段并驱动 runtime 文本底座
  - user: 2026-04-21 继续推进 M1，默认字体入口必须成为 runtime 自有资产
  - user: 2026-04-21 继续推进 M1，把 .font.toml 正式纳入 asset/resource 主链并让 UI loader 复用公共 FontAsset
  - user: 2026-04-21 继续推进 M1，让项目内 res:// 字体资产通过 ProjectAssetManager 进入 runtime UI 文本链路
  - user: 2026-04-28 继续文本的 SDF 渲染和排版能力任务
  - user: 2026-04-24 继续编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图
  - user: 2026-04-28 继续 Generic host boundary，迁出 HostMenuChrome 业务菜单真源
  - user: 2026-04-28 继续 Generic host boundary，收回 HostMenuChrome control-specific frame ABI
  - user: 2026-04-29 激进迁移 HostMenuChrome、HostPageChrome 和 Dock header 到 ui.toml
  - user: 2026-04-29 继续 Generic host boundary status bar ui.toml cutover
  - user: 2026-04-29 继续 Generic host boundary activity rail ui.toml cutover
  - user: 2026-04-30 继续 Generic host catalog leaf-name cutover
  - user: 2026-05-01 继续 editor-only Generic host cleanup，删除 legacy builtin host document alias
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - docs/superpowers/plans/2026-04-24-ui-toml-pane-template-implementation.md
  - docs/superpowers/plans/2026-04-29-ui-cutover-move-first.md
  - docs/superpowers/plans/2026-04-29-slint-fence-ui-toml-cutover.md
tests:
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/ui/tests/template.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - zircon_runtime/tests/font_asset_manifest_contract.rs
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/template_runtime/structure_split.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/host/slint_hierarchy_template_body.rs
  - zircon_editor/src/tests/host/slint_animation_template_body.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/slint_window/activity_rail_template_boundary.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/drawer_source_projection.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/tests/host/slint_window/native_mode.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/slint_tab_drag/surface_contract.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - cargo test -p zircon_runtime render_extract_carries_visual_contract_fields_for_visible_nodes
  - cargo test -p zircon_runtime --lib ui::tests::text_layout --locked --jobs 1
  - cargo test -p zircon_runtime --lib render_extract_uses_label_when_schema_text_default_is_empty --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --nocapture
  - cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --nocapture
  - cargo test -p zircon_runtime --lib screen_space_ui_plan_uses_resolved_text_layout_lines_as_batches --locked --jobs 1
  - cargo test -p zircon_runtime --lib sdf_atlas --locked --jobs 1
  - cargo test -p zircon_runtime --lib sdf_draw_plan --locked --jobs 1
  - cargo test -p zircon_runtime --lib text_backend_routing --locked --jobs 1
  - cargo test -p zircon_runtime default_runtime_font_manifest_stays_inside_runtime_assets --locked
  - cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never
  - cargo test -p zircon_runtime --lib ui_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never
  - cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib editor_host_sources_do_not_depend_on_deleted_slint_trees --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - cargo test -p zircon_editor --lib generic_host_catalog_uses_shared_runtime_component_names --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib builtin_host_runtime_exposes_only_generic_host_window_document_id --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - git diff --check -- docs/ui-and-layout/shared-ui-template-runtime.md docs/ui-and-layout/index.md .codex/sessions/20260429-2236-ui-cutover-single-milestone.md
  - cargo test -p zircon_runtime font_asset_ --locked
  - cargo test -p zircon_runtime --test font_asset_manifest_contract project_font_manifest_resolves_through_project_asset_manager --locked
  - cargo test -p zircon_runtime --test font_asset_manifest_contract --locked
  - cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked
  - cargo check -p zircon_runtime --locked --lib
  - cargo test -p zircon_runtime template_tree_builder_projects_template_instance_into_shared_ui_tree_with_metadata --locked
  - cargo test -p zircon_editor --lib --locked tab_drag_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked drawer_resize_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked root_shell_frames_use_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib host_menu_chrome_uses_projected_menu_model_instead_of_hardcoded_business_rows --locked --jobs 1
  - cargo test -p zircon_editor --lib host_menu_chrome_keeps_menu_button_frames_internal_to_chrome_component --locked --jobs 1
  - source assertion: HostMenuChrome、HostPageChrome、side/document/bottom dock headers all render TemplatePane and no longer contain MenuBarButton/DockTabButton/TabChip
  - rustc --edition=2021 --test zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs --exact host_menu_popup_visuals_come_from_ui_toml_template_nodes
  - cargo check -p zircon_editor --locked with CARGO_TARGET_DIR=target/codex-ui-chrome-check
  - cargo build -p zircon_app --bin zircon_editor --features target-editor-host --no-default-features --locked with CARGO_TARGET_DIR=target/codex-ui-chrome-check
  - visual screenshot: target/visual-layout/editor-window-20260429-chrome-ui-toml-topmost-1280x720.png
  - visual screenshot: target/visual-layout/editor-window-20260429-chrome-ui-toml-topmost-900x620.png
  - visual screenshot: target/visual-layout/editor-window-20260429-chrome-ui-toml-topmost-700x620.png
  - visual screenshot: target/visual-layout/editor-window-20260429-menu-popup-template-1280x720.png
  - visual screenshot: target/visual-layout/editor-window-20260429-menu-popup-template-900x620.png
  - visual screenshot: target/visual-layout/editor-window-20260429-menu-popup-template-700x620.png
  - visual screenshot: target/visual-layout/editor-window-20260429-menu-popup-template-file-open-900x620.png
  - cargo test -p zircon_editor --lib slint_menu_pointer --locked --jobs 1
  - cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1
  - cargo test -p zircon_editor --lib host_menu_chrome_projects_hovered_or_open_menu_state_into_template_highlight --locked --jobs 1
  - cargo test -p zircon_editor --lib workbench_chrome_projection_uses_user_requested_asset_paths --locked --jobs 1
  - cargo test -p zircon_editor --lib slint_chrome_inputs_use_projected_control_frames_instead_of_local_geometry_math --locked --jobs 1
  - cargo test -p zircon_editor --lib workbench_chrome_heights_are_loaded_from_toml_assets_not_scene_constants --locked --jobs 1
  - cargo test -p zircon_editor --lib host_menu_chrome_uses_projected_template_control_frames_for_menu_hit_areas --locked --jobs 1
  - cargo test -p zircon_editor --lib menu_popup_nodes_project_absolute_rows_beyond_authored_slots --locked --jobs 1
  - cargo test -p zircon_editor --lib host_menu_chrome_virtualizes_scrolled_window_popup_template_rows --locked --jobs 1
  - cargo test -p zircon_editor --lib shared_menu_pointer_click_dispatches_scrolled_window_preset_selection --locked --jobs 1
  - cargo test -p zircon_editor --lib shared_menu_pointer_bridge_recomputes_hovered_item_after_window_popup_scroll --locked --jobs 1
  - cargo test -p zircon_editor --lib host_menu_chrome_keeps_popup_panel_fixed_while_virtual_rows_scroll --locked --jobs 1
  - cargo test -p zircon_editor --lib menu_popup_projection_mutes_disabled_item_labels --locked --jobs 1
  - cargo test -p zircon_editor --lib capture_scrolled_window_popup_visual_artifact --locked --jobs 1 -- --ignored
  - visual screenshot: target/visual-layout/editor-window-20260429-window-popup-scrolled-900x620.png
  - cargo test -p zircon_editor --lib host_status_bar_visuals_come_from_ui_toml_template_nodes --locked --jobs 1
  - cargo test -p zircon_editor --lib host_floating_window_headers_use_projected_template_nodes --locked --jobs 1
  - cargo test -p zircon_editor --lib native_floating_window_mode_forwards_tabs_header_and_pane_callbacks_to_root --locked --jobs 1
  - cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1
  - rustc --edition=2021 --test zircon_editor/src/tests/host/slint_window/activity_rail_template_boundary.rs --exact host_side_activity_rails_use_projected_template_nodes
  - cargo test -p zircon_editor --lib host_side_activity_rails_use_projected_template_nodes --locked --jobs 1
  - cargo test -p zircon_editor --lib slint_window --locked --jobs 1
  - cargo test -p zircon_editor --lib builtin_workbench_drawer_source_template_bridge_exports_visible_drawer_frames_from_workbench_model --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib builtin_host_window_template_bridge_exports_visible_drawer_shell_and_header_frames_from_workbench_model --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib root_host_recomputes_builtin_template_bridge_with_visible_drawer_shell_and_header_frames --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --test-threads=1 --nocapture
  - rustc --edition=2021 --test zircon_editor/src/tests/host/slint_drawer_resize/surface_contract.rs
  - cargo check -p zircon_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-resize-frame-check
  - cargo test -p zircon_editor --lib host_page_pointer --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-host-page-tabs
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-host-page-tabs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout
  - rustc --edition=2024 --test zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs ... host_menu_chrome
  - cargo fmt --package zircon_editor -- --check
  - cargo test -p zircon_runtime ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets --locked
  - cargo test -p zircon_runtime --locked
  - cargo check -p zircon_editor
  - cargo test -p zircon_editor boundary
  - cargo test -p zircon_editor template
  - cargo test -p zircon_editor --lib --locked template_runtime --offline
  - cargo test -p zircon_editor --lib --locked slint_hierarchy_template_body --offline
  - cargo test -p zircon_editor --lib --locked slint_animation_template_body --offline
  - cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - cargo test -p zircon_editor --lib builtin_pane_body_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short
  - cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short
  - cargo fmt --all
  - cargo check -p zircon_editor --locked --offline
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipTest -VerboseOutput
doc_type: module-detail
---

# Shared UI Template Runtime

## Purpose

这一层现在承接的是“tree `.ui.toml` 资产 -> shared UI 权威模型”的正式入口，不再把 legacy `UiTemplateDocument` 当成 editor/runtime 生产链路的主文档类型。

在这轮 cutover 后，正式 authority 已经固定成：

1. tree `.ui.toml` 解析成 `UiAssetDocument`
2. `UiAssetLoader` 校验 tree authority 与稳定 `node_id`
3. `UiDocumentCompiler` 产出 `UiCompiledDocument`
4. `UiCompiledDocument` 实例化成 shared `UiTemplateInstance`
5. `UiTemplateSurfaceBuilder` 把实例树投影到 shared `UiSurface`
6. `UiSurface::compute_layout(...)` 按 shared measure/arrange 契约求 frame / clip / scroll window
7. editor/runtime adapter 再继续把 shared surface、projection 或宿主节点模型投影到宿主层

editor host 这一侧也已经同步收口：

- [`EditorTemplateRegistry`](../../zircon_editor/src/ui/template/registry.rs) 只存 `UiCompiledDocument`
- [`runtime_host.rs`](../../zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs) 的生产态只接受 `UiAssetDocument`
- builtin host 文档 [`zircon_editor/assets/ui/editor/host/*.ui.toml`](../../zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) 已改写成 tree asset authority，并继续留在 crate `src/` 之外
- `UiTemplateDocument` / `UiTemplateLoader` 仅保留在 shared template 单元测试和历史 fixture 转换测试中，不再是 editor production runtime 的 fallback authority

这意味着这一层当前负责的是“资产语义真源 + shared tree 首段落点 + 显式 layout 合同落点”，但仍然不负责 editor docking 业务、host callback ABI 或宿主专属状态机。真正的布局、命中、焦点和 route 权威仍然在 `UiTree` / `UiSurface` / shared layout contract。

## Current Asset Authority

这一轮 editor host 的最新收口点是“移除生产态 legacy loader fallback，而不是继续让 builtin host 模板在 runtime 里双解析”：

- [`EditorTemplateRegistry`](../../zircon_editor/src/ui/template/registry.rs) 删除 `UiTemplateDocument` 存储分支，只保留 `UiCompiledDocument`
- [`EditorUiHostRuntime::register_document_source(...)`](../../zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs) 只走 `parse_ui_asset_document_source(...)`，生产态不再回退到 `UiTemplateLoader`
- builtin host 文件 [`workbench_shell.ui.toml`](../../zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml)、[`workbench_drawer_source.ui.toml`](../../zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml)、[`floating_window_source.ui.toml`](../../zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml)、[`scene_viewport_toolbar.ui.toml`](../../zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml)、[`asset_surface_controls.ui.toml`](../../zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml)、[`pane_surface_controls.ui.toml`](../../zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml)、[`startup_welcome_controls.ui.toml`](../../zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml)、[`inspector_surface_controls.ui.toml`](../../zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml) 都已经改成 tree-shaped `UiAssetDocument`
- 历史 template/flat fixture 转换只允许留在 runtime test support 与 editor test support，不再允许回流进 production runtime 或 formal public template surface

当前本地验证直接锁住了这条边界：

- `cargo check -p zircon_editor`
- `cargo test -p zircon_editor boundary`
- `cargo test -p zircon_editor template`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipTest -VerboseOutput`

## Builtin Pane Body Projection

2026-04-24 的 pane body cutover 把首批 workbench pane 的 body authority 又从 Slint/Rust 手写 DTO 往 `.ui.toml -> PanePresentation -> host projection` 推进了一层。`Console`、`Inspector` 和 `RuntimeDiagnostics` 走 template body；`Hierarchy`、`AnimationSequenceEditor`、`AnimationGraphEditor` 和 `ModulePlugins` 走 hybrid body，native slot 只承载尚未退场的复杂 pane 内部交互。

当前合同如下：

- [`hierarchy_body.ui.toml`](../../zircon_editor/assets/ui/editor/host/hierarchy_body.ui.toml) 保留 `SelectionCommand.SelectSceneNode` route，并把树主体声明成 `hierarchy_tree_slot`；Rust-owned host contract 只继续承载已有 tree native slot 与 pointer bridge。
- [`animation_sequence_body.ui.toml`](../../zircon_editor/assets/ui/editor/host/animation_sequence_body.ui.toml) 保留 `AnimationCommand.ScrubTimeline` route，并把 timeline 主体声明成根级 `animation_timeline_slot`。
- [`animation_graph_body.ui.toml`](../../zircon_editor/assets/ui/editor/host/animation_graph_body.ui.toml) 保留 `AnimationCommand.AddGraphNode` route，并把 graph canvas 主体声明成根级 `animation_graph_canvas_slot`。
- [`module_plugins_body.ui.toml`](../../zircon_editor/assets/ui/editor/host/module_plugins_body.ui.toml) 保留 `DockCommand.FocusView` route，并把 plugin list 主体声明成根级 `module_plugin_list_slot`；对应的 `ModulePluginsPaneBody/FocusModulePlugins` binding 由 [`template_bindings.rs`](../../zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs) 注册到 `editor.module_plugins#1`。
- [`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs) 现在对 hierarchy/animation 优先读取 `PanePresentation.body`，调用 `EditorUiHostRuntime::project_pane_body(...)` 注入 payload 与 hybrid slot anchor，再把 payload 还原成 Rust-owned host contract/native view 所需的 rows、track、parameter、node、state 和 transition 数据。
- [`apply_presentation.rs`](../../zircon_editor/src/ui/slint_host/ui/apply_presentation.rs) 会把每个 dock/floating pane 的可见内容尺寸传给转换层，让 template body projection 至少拥有宿主 content bounds；dock/window 生命周期和 native pointer bridge 不在这一层重写。
- former pane body Slint sources are no longer active assets, test inputs, or migration references for this path; current pane body truth continues from `.ui.toml -> PanePresentation -> host projection` and consumes stable control id / anchor id while hierarchy tree、timeline、graph canvas 和 plugin list 的细交互留给对应 native slot。

Task 10 的 host-side cleanup 把 [`PaneData`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs) 中原本平铺的 `Console` / `Inspector` / `Hierarchy` / `Animation` / asset body DTO 收进 `PaneNativeBodyData`。`PaneData` 现在的正式 body authority 是 `presentation: PanePresentation`；`native_body` 只作为 Rust-owned host ABI 和 native slot 所需的宿主数据投影存在。对应地，[`pane_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs) 只在生成 `PanePresentation` 后填充 native body，[`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 读取 native 数据时也显式标注为宿主投影，而不是继续把 giant union 当成结构真源。

验证覆盖：

- `cargo test -p zircon_editor --lib --locked template_runtime --offline` 锁住 builtin body document 注册、hybrid slot 根级声明、route namespace 和 payload projection。
- `cargo test -p zircon_editor --lib --locked slint_hierarchy_template_body --offline` 锁住 hierarchy body 从 payload 和 template anchor 投影到 Slint view data。
- `cargo test -p zircon_editor --lib --locked slint_animation_template_body --offline` 锁住 sequence timeline / graph canvas hybrid anchors 与 animation payload 投影。
- `cargo test -p zircon_editor --lib builtin_pane_body_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --nocapture` 锁住 builtin pane body document 注册和 route namespace，其中包含 `ModulePluginsPaneBody/FocusModulePlugins`。
- `cargo test -p zircon_editor --lib --locked slint_host --offline` 确认现有真实宿主桥仍然保持可运行。
- `cargo check -p zircon_editor --locked --offline` 确认 `PaneNativeBodyData` 收束后 production crate 仍然编译通过。

## No-Slint Generic Host Cutover

Task 8 的文档收口基于当前 hard fence 状态，而不是早期 plan 中的临时 moved `.slint` 目标：active [`zircon_editor/ui`](../../zircon_editor/ui) tree 现在必须保持 0 个 `.slint` 文件，former deleted Slint copies 也不再是实现、测试或迁移参考源。

当前 editor host authority 链路是：[`zircon_editor/assets/ui/editor/**/*.ui.toml`](../../zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) -> shared `UiAssetLoader` / `UiDocumentCompiler` / `UiTemplateSurfaceBuilder` -> `PanePresentation` / `ViewTemplateNodeData` -> [`zircon_editor::ui::slint_host::host_contract`](../../zircon_editor/src/ui/slint_host/host_contract/mod.rs)。[`zircon_editor/build.rs`](../../zircon_editor/build.rs) 不再运行 `slint_build`、不再 staging migration Slint trees 到 `OUT_DIR`，[`slint_host/mod.rs`](../../zircon_editor/src/ui/slint_host/mod.rs) 也不再调用 `slint::include_modules!()`；former generated DTO/callback seam 由 [`host_contract/window.rs`](../../zircon_editor/src/ui/slint_host/host_contract/window.rs)、[`host_contract/globals.rs`](../../zircon_editor/src/ui/slint_host/host_contract/globals.rs) 和 [`host_contract/data/**`](../../zircon_editor/src/ui/slint_host/host_contract/data/mod.rs) 承接。

Rust projection owner 在本次 DTO cutover 中仍是 [`src/ui/layouts/windows/workbench_host_window`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/mod.rs)，而不是 plan 早期样例里的 `host_window` move target。`generic_host_layout_paths` 因此同时守住两件事：active `zircon_editor/ui` 无 `.slint` 源，以及当前 Rust host projection seam 继续存在，避免新代码为了“补回入口”而恢复 `ui/workbench.slint`、`temp/slint-migration/**`、generated Slint include 或 `as slint_ui` 兼容别名。

Runtime 侧的 cutover 不需要 editor host 的 Slint bootstrap：[`RuntimeUiManager`](../../zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs) 从 runtime fixture `.ui.toml` 构建 owned `UiSurface`，输入事件通过 shared surface dispatch，`build_frame()` 只把 `UiRenderExtract` 交给 graphics。`render_framework_submits_all_builtin_runtime_ui_fixtures` 已覆盖 `HudOverlay`、`PauseMenu`、`SettingsDialog` 和 `InventoryList` 经 `WgpuRenderFramework::submit_runtime_frame(...)` 进入 screen-space UI pass，并检查 command 与 quad/text payload stats。

## Runtime Typography Metadata

M1 之后，template metadata 已经开始承担 runtime 文本底座所需的最小 typography 真源，而不再只有 `background/foreground/border/text/icon/opacity` 这类视觉占位字段。

`resolve.rs` 当前会把下面这些键直接解析进 `UiResolvedStyle`：

- `font`
- `font_family`
- `font_size`
- `line_height`
- `text_align`
- `wrap`
- `text_render_mode`

除了直写属性，还支持 `[font]` table 的同义字段，例如：

```toml
font = "res://fonts/default.font.toml"
font_family = "Fira Mono"
font_size = 18.0
line_height = 24.0
text_align = "center"
wrap = "word"
text_render_mode = "auto"
```

如果需要显式覆盖字体资产默认值，也可以写成：

```toml
text_render_mode = "sdf"
```

这组 metadata 现在已经有 capture 级证据，而不只是 shared tree / planner 级证据：[runtime_ui_text_render_contract.rs](/E:/Git/ZirconEngine/zircon_runtime/tests/runtime_ui_text_render_contract.rs) 直接证明 `native` / `sdf` 两条文本路径都能把 template typography metadata 落到最终 glyph 输出，并且 `clip_frame`、`wrap` 和 `opacity` 都会继续进入最终 glyph 采样结果，而不是在 shared render contract 里中途丢失。

这份回归现在已经分成两类最终证据：

- 手写 `UiRenderCommand` 路径，证明 runtime text backend 本身不会把字形重新退化成占位矩形带
- 正式模板资产路径，证明 `.ui.toml -> UiAssetLoader -> UiDocumentCompiler -> UiTemplateSurfaceBuilder -> UiSurface.render_extract -> RenderFramework capture_frame(...)` 这条主链也能把 `wrap` 和 `opacity` 保到最终 glyph 像素差异上

这让 shared template runtime 的 typography 合同不再只是“字段已解析进 `UiResolvedStyle`”，而是已经证明正式资产链路能把这些字段保真送到最终 renderer。

`UiSurface.render_extract` 现在还会在每条文本命令上生成 `UiResolvedTextLayout`。这份 layout DTO 是 shared runtime 数据，而不是 renderer 私有状态：它记录 word/glyph wrapping 后的可见文本行、每行 `UiFrame`、对齐方式、字号、行高和 `overflow_clipped`。因此模板链路在进入 graphics 前已经具备基础排版事实，screen-space UI planner 会优先按 resolved line 生成独立 text batch，glyphon/SDF backend 只负责最终 shaping、atlas/cache 和提交，而不再是唯一知道文本如何分行和裁剪的层。

SDF 路径现在还有一个 renderer-local atlas/cache planning owner：[`sdf_atlas.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs)。shared template runtime 仍然只写 `text_render_mode = "sdf"`、`font`、`font_family` 和字号这些中性样式字段；进入 graphics 后，`ScreenSpaceUiSdfAtlas` 会把 resolved SDF text batches 收敛成 glyph slot plan。slot key 包含 glyph、font asset、font family 和字号，因此不同字体或字号的同一字符不会在后续专用 SDF atlas 中错误共用 cache entry；atlas rect 按 key 排序分配，空白字符只保留 advance，不会占用 atlas slot。

SDF 可见输出也已经从 glyphon fallback 替换为 renderer-local GPU path：[`sdf_render.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs) 根据同一个 atlas plan 上传 `R8Unorm` SDF atlas texture，生成 screen-space glyph quads，并通过 [`sdf_text.wgsl`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl) 在 UI pass 内 alpha blend 输出。SDF quad planning 会同时受 text batch frame、`text_align`、显式 `clip_frame` 和 viewport 约束；当前 atlas mask 仍是最小 CPU placeholder SDF，用来固定 texture/bind group/quad/UV/clip 合同；后续真实字体轮廓 SDF bake 只需要替换 mask 生成，不需要让 template runtime 暴露 GPU 细节。

普通文本渲染兼容性由 `ResolvedScreenSpaceUiTextBatches` 负责守住。`Native` 批次和解析为 `Native` 的 `Auto` 批次只进入 normal glyphon backend；`Sdf` 批次和解析为 `Sdf` 的 `Auto` 批次进入 SDF atlas owner 与 GPU SDF renderer。这个 routing contract 让专用 SDF shader 不会误把普通文本迁到 SDF cache，也不会让 SDF atlas state 污染 native-only frame。

这条边界由 `ui::tests::text_layout`、`screen_space_ui_plan_uses_resolved_text_layout_lines_as_batches`、`sdf_atlas`、`sdf_draw_plan` 和 `text_backend_routing` 锁住：word wrap + center align 会输出稳定的 line frames，`clip_frame` 会裁掉不可见行并标记 overflow，graphics planner 会把这些行 frame 作为最终 text batch 输入，SDF atlas owner 会按字体身份和字号生成稳定 glyph slot plan 并把空白保留为 advance，GPU SDF renderer 会按 `text_align` 为可见 glyph 生成 textured quads，普通 native 文本不会进入 SDF atlas input。

editor viewport 那条 runtime-style HUD 现在也加入了这条 capture 证据链：[render_frame_submission_hud_text_renders_through_runtime_glyph_capture](/E:/Git/ZirconEngine/zircon_editor/src/tests/editing/state.rs) 直接把 `EditorState::render_frame_submission()` 产出的 `UiRenderExtract` 交给 runtime render framework capture，并用“有字 HUD / 去字 HUD”的像素差异证明 shared template/runtime 写出的 typography 字段没有在 editor 宿主路径里被旁路掉。

或者：

```toml
[font]
asset = "res://fonts/default.font.toml"
family = "Fira Mono"
size = 18.0
line_height = 24.0
align = "center"
wrap = "word"
render_mode = "native"
```

这里的默认语义已经固定下来：

- `UiResolvedStyle::text_render_mode` 默认值是 `Auto`
- `text_render_mode = "auto"` 表示把具体 native/sdf 选择延后到 runtime text backend
- text backend 会优先读取字体资产 manifest 的 `render_mode`
- 如果字体资产没有声明默认值，则稳定回落到 `Native`
- 如果模板样式显式写 `native` 或 `sdf`，显式样式优先于字体资产默认值

作为 M1 的默认可用闭环，`res://fonts/default.font.toml` 现在还带有一条更硬的资源归属规则：

- manifest 内部的 `source` 已收口到 `zircon_runtime/assets/fonts/FiraMono-subset.ttf`
- shared template runtime 继续只暴露字体资产引用，不把 dev tree 相对路径泄露进样式合同
- `default_runtime_font_manifest_stays_inside_runtime_assets` 会把这条默认资产归属锁成测试

在这之上，字体 manifest 解析边界也被收紧成正式合同：

- `source` 必须是相对路径，不接受绝对文件路径
- `res://` 字体 manifest 的 `source` 解析后必须仍落在 runtime `assets/` 根内
- 非 `res://` 的外部 manifest 也只能解析到 manifest 自己目录作用域内，不允许继续越级逃逸

这轮又把这份合同从“renderer 私有 loader”继续推成公共资产语义：

- `.font.toml` 现在有正式的 [`FontAsset`](../../zircon_runtime/src/asset/assets/font.rs) 模型，而不是只在 `graphics::...::font_asset` 内部保留匿名 TOML 结构
- runtime asset pipeline 已经把 `.font.toml` 识别成 `ImportedAsset::Font` / `AssetKind::Font`，所以 project scan、artifact store、runtime resource registry 都能看见字体资产
- [`font_asset.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs) 现在直接复用 `FontAsset::from_toml_str(...)`，并向 text backend 返回强类型 `UiTextRenderMode`
- [`ScreenSpaceUiTextSystem`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs) 现在会持有 `ProjectAssetManager`，因此 shared template/runtime 写出的 `font = "res://fonts/project.font.toml"` 不再只会回到 runtime crate 自带 `assets/`，而是能优先命中当前项目里正式导入的字体资产
- 项目里的原始字体二进制 `.ttf/.otf/.woff/.woff2` 现在被 [`collect_files.rs`](../../zircon_runtime/src/asset/project/manager/collect_files.rs) 视为 manifest source auxiliary，而不是独立 asset；这保证 shared template runtime 可以安全引用项目字体而不会把 `scan_and_import()` 直接炸掉

这样 shared template/runtime 样式仍然只持有 `font = "res://..."` 这类中性引用，但 runtime text backend 已经不再信任 manifest 内部的任意文件系统跳转。

这样 shared template runtime 现在已经能把“字体资源引用、字号、行高、对齐、换行、auto/native/sdf 选择”一路投影到 shared render contract，而不需要 editor/runtime 再各自做一套文本样式解释。

## Legacy Compat Model

### `UiTemplateDocument`

- `version`
- `components: BTreeMap<String, zircon_runtime::ui::template::UiComponentTemplate>`
- `root: zircon_runtime::ui::template::UiTemplateNode`

文档拥有一个真正的入口 root，以及一组可被重复装配的命名 component template。

### `zircon_runtime::ui::template::UiComponentTemplate`

- `root: zircon_runtime::ui::template::UiTemplateNode`
- `slots: BTreeMap<String, zircon_runtime::ui::template::UiSlotTemplate>`

component template 是“复合组件装配层”的最小权威单元。它不直接描述最终像素 frame，只描述宿主树和 shared tree 应该如何拼装。

### `zircon_runtime::ui::template::UiTemplateNode`

当前节点固定只有三种互斥形态：

- `component`
  - 表示一个真实宿主/共享组件节点
- `template`
  - 表示对命名 `zircon_runtime::ui::template::UiComponentTemplate` 的调用
- `slot`
  - 表示 template 内部的插槽占位点

额外携带的通用字段包括：

- `control_id`
- `bindings`
- `children`
- `slots`
- `attributes`
- `style_tokens`

这里的 `bindings` 目前不是宿主 callback 名称，而是稳定的 `zircon_runtime::ui::template::UiBindingRef`：

- `id`
- `event`
- `route`

`id` 用来承载诸如 `WorkbenchMenuBar/SaveProject` 这类稳定命名空间；`route` 只是稳定 route key，不是桌面宿主私有函数名。

`UiComponentTemplate` / `UiSlotTemplate` / `UiBindingRef` / `UiActionRef` 现在都统一经 `zircon_runtime::ui::template::*` 暴露，`zircon_runtime::ui` root 不再继续平铺这组 template document model。

## Legacy Template TOML Shape

当前实现支持的最小 TOML 形态如下：

```toml
version = 1

[root]
template = "WorkbenchShell"
slots = { menu_bar = [{ template = "MenuBar" }] }

[components.WorkbenchShell]
slots = { menu_bar = { required = true } }
root = { component = "WorkbenchShell", children = [{ slot = "menu_bar" }] }

[components.MenuBar]
root = { component = "UiHostToolbar", children = [
  { component = "IconButton", control_id = "SaveProject", bindings = [
    { id = "WorkbenchMenuBar/SaveProject", event = "Click", route = "MenuAction.SaveProject" }
  ] }
] }
```

2026-04-30 generic host catalog cutover removed the host-specific icon-button and label transitional component names from active editor host assets and adapter tests. Host-owned shells such as `UiHostWindow` and `UiHostToolbar` remain as bootstrap/container roles, while leaf visuals use shared Runtime UI names such as `IconButton` and `Label`. The editor host adapter follows the same text resolution contract as Runtime UI render extraction: a non-empty `text` prop wins, and an authored non-empty `label` remains the fallback when component schema defaults inject `text = ""`.

这个结构已经满足第一阶段目标：

- component template 可以嵌套 component template
- slot 内容由调用点提供
- binding 引用保留稳定命名空间
- 运行时实例展开后不会丢掉这些 binding ref

仓库里这组 builtin host 模板资产现在已经放在 [workbench_shell.ui.toml](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml)。它先覆盖 workbench shell 的复合装配骨架：

- `WorkbenchShell`
- `MenuBar`
- `ActivityRail`
- `DocumentHost`
- `StatusBar`

## Layout Contract By Attribute

当前模板文档不会额外引入第二套 layout 节点类型，而是固定通过 `attributes.layout` 把 shared contract 写进模板节点。

已落地的字段包括：

- `width` / `height`
  - 对应 shared `AxisConstraint { min, max, preferred, priority, weight, stretch }`
- `anchor` / `pivot` / `position`
  - 直接映射到 shared `Anchor` / `Pivot` / `Position`
- `boundary`
  - 对应 `LayoutBoundary::{ContentDriven, ParentDirected, Fixed}`
- `clip` / `clip_to_bounds`
  - 控制节点 clip 链入口
- `z_index`
  - 控制 shared draw order 的层级偏置
- `input_policy`
  - 支持 `Inherit` / `Receive` / `Ignore`
- `container`
  - 显式声明 shared 容器语义，而不是强依赖 component 名字

`container` 目前支持：

- `Container`
- `Overlay`
- `Space`
- `HorizontalBox { gap }`
- `VerticalBox { gap }`
- `ScrollableBox { axis, gap, scrollbar_visibility, virtualization }`

这一步的关键点是：editor-only component 名字不再需要和 shared primitive 名字一一重合。像 `WorkbenchShell`、`DocumentHost`、`ActivityRail` 这样的 composite，可以继续保留自己的宿主身份，但它们的 shared layout 行为已经由 `attributes.layout.container` 显式给出。

## Validation Rules

`UiTemplateValidator` 当前已经把以下约束钉死：

- 每个节点必须且只能声明 `component` / `template` / `slot` 其中一种
- `template` 调用必须引用已注册的 component template
- required slot 必须由调用点提供
- 不允许给单值 slot 塞多个子节点
- template 内部出现的 slot placeholder 必须先在 `slots` 中声明
- slot placeholder 不能再额外携带 bindings、children、slot fills 或 control id
- template 调用不允许直接再挂 `children`，slot 才是唯一的复合内容注入口

这一步的意义是避免 editor host 或后续 Slint projection 再去容忍一堆“能跑但不清晰”的隐式模板结构。

## Instance Expansion

`zircon_runtime::ui::template::UiTemplateInstance::from_document(...)` 当前会：

- 先跑完整 `UiTemplateValidator`
- 再把 `template` 调用展开成真实 component 子树
- 再把 slot placeholder 替换成调用点提供的内容
- 最终得到一个已经没有 `template`/`slot` 占位歧义的运行时模板实例树

目前实例层还提供 `binding_refs()`，按树遍历顺序收集稳定 binding 引用。这正是后续 editor/runtime adapter 把模板树映射成 typed command/binding、再投影给 Rust-owned host contract 的入口。

## Shared Tree Bridge

这一轮新增了 shared-core 桥接器：

- `UiTemplateTreeBuilder`
- `UiTemplateSurfaceBuilder`
- `UiTemplateBuildError`
- `zircon_runtime::ui::tree::UiTemplateNodeMetadata`

### `zircon_runtime::ui::tree::UiTemplateNodeMetadata`

`UiTreeNode` 现在可以携带模板元数据快照，用来保留后续 shared core 和宿主投影都会需要的稳定信息：

- `component`
- `control_id`
- `attributes`
- `style_tokens`
- `bindings`

这一步很关键，因为之前如果直接把模板实例铺进 `UiTree`，会丢掉：

- 稳定 binding id
- 宿主 icon / label 等属性
- style token
- component/control identity

那样后续再想从 shared tree 做 route 或宿主投影，就必须重新回头读模板实例，等于 shared tree 不是真正的中继层。

### `UiTemplateTreeBuilder`

当前 builder 会把 `zircon_runtime::ui::template::UiTemplateInstance` 转成 `UiTree`，并做两类 shared-core 推断：

- 节点按 preorder 分配稳定 `UiNodeId`
- `UiNodePath` 使用 control/component 名称加顺序索引生成可读路径
- 模板元数据挂入每个 `UiTreeNode`
- 显式 `attributes.layout` 映射到 shared `BoxConstraints` / `Anchor` / `Pivot` / `Position` / `LayoutBoundary` / `UiInputPolicy` / `z_index`
- `attributes.layout.container` 优先映射到 `UiContainerKind`
- 当模板没有显式 `container` 时，再退回到已知共享容器名映射
- 可交互节点根据 bindings / 已知交互 primitive 推断 `clickable` / `hoverable` / `focusable`
- 带 bindings 的节点默认设置 `UiInputPolicy::Receive`
- `ScrollableBox` 自动初始化 `UiScrollState::default()` 并开启 `clip_to_bounds`

当前 layout contract 采用“显式字段优先、组件名仅作回退”的规则。也就是说：

- 如果模板写了 `attributes.layout.container.kind = "VerticalBox"`，shared tree 就直接按 `VerticalBox` 处理
- 如果模板没写 layout 容器，但 component 名字本身就是 `HorizontalBox` / `ScrollableBox` 这类 shared primitive，builder 仍然会做兼容映射
- 如果两者都没有，节点保持 `UiContainerKind::Free`

对于 layout 字段值，builder 现在会在 bridge 阶段做基本结构校验；不支持的 enum 值或错误的 table 形态会直接返回 `UiTemplateBuildError::InvalidLayoutContract`，避免把畸形模板延后到 layout pass 或宿主投影时才暴露。

当前已知容器映射只覆盖 shared primitive 名称：

- `Container`
- `Overlay`
- `Space`
- `HorizontalBox`
- `VerticalBox`
- `ScrollableBox`

未知 component 目前不会被强行解释成布局容器，而是保留 `UiContainerKind::Free`。

### `UiTemplateSurfaceBuilder`

`UiTemplateSurfaceBuilder` 只是 `UiTemplateTreeBuilder` 的轻封装：

- 先构建 `UiTree`
- 再放入 `UiSurface`
- 最后调用 `rebuild()` 生成 hit-test index 和初始 `UiRenderExtract`

这让 shared template runtime 现在已经具备了“模板实例 -> shared retained surface -> shared layout 求 frame -> shared visual draw list”的最低闭环，而不是停留在纯文档/纯实例层。

当前 `rebuild()` 输出的 `UiRenderExtract` 会直接把模板属性里已经 resolved 的视觉字段带出来，而不再只保留几何：

- `background` / `foreground` / `border` 会落到 `UiResolvedStyle`
- 非空 `text` 会落到 render command 的 `text`；当 schema default 注入的是空 `text = ""` 时，会回退到非空 `label`
- `icon` / `image` 会落到 `UiVisualAssetRef`
- `opacity` 会落到 render command 的 `opacity`

这意味着 style asset 和 inline override 在 template compiler 里完成归并以后，shared surface 已经能把这些视觉结果继续传给 preview/runtime consumer；文本测量/分行/对齐/裁剪已经先以 `UiResolvedTextLayout` 进入 render extract，SDF 字形 cache key / slot run / whitespace advance / atlas rect / GPU quad path 也已经在 renderer-local `ScreenSpaceUiSdfAtlas` + `ScreenSpaceUiSdfRenderer` 边界收敛。后续还没做的部分是更完整的 shaping/fallback、真实字体轮廓 SDF bake、图片资源装载和更完整 GPU pass，而不是再回头重建另一套 visual payload 模型。

## Current Scope And Deliberate Gaps

这一轮刻意没有把以下能力塞进 `zircon_runtime::ui::template`：

- Rust-owned host tree 自动投影
- repeat/tree data projection
- 样式 token 继承/覆盖求值
- 模板参数求值和表达式系统
- 文本/图片测量服务
- 基于样式 token 或表达式的动态 layout 合同求值
- runtime widget 级 visual primitive 的完整模板化

原因很直接：这里先锁住模板装配契约，并把显式 layout 合同落进 shared tree，但不在 token、表达式、测量服务都还没定型之前就发明第二层隐式布局公式。

## Why This Boundary Matters

如果没有这层共享模板语义，后续 editor 迁移很容易退回两条错误路线：

- 让 Slint `.slint` 业务树继续做真正的模板权威
- 或者在 `zircon_editor` 里直接把 WorkbenchLayout/ViewModel 拼成另一套 host-only 树

现在 `zircon_runtime::ui::template` 已经先把文档、slot、binding 命名、运行时实例展开，以及 shared tree 的第一段桥接统一下来。后续无论是 runtime UI 还是 editor shell，都必须从同一份模板真源继续向 shared layout 求解和宿主投影层推进。

## Builtin Root Document Identity

`zircon_editor` 这一轮又把 shared template runtime 的 builtin root 文档身份往 generic host 边界推进了一步：

- builtin root host 模板现在只以 `ui.host_window` 注册；旧 workbench shell document alias、测试 re-export 和重复 builtin document entry 已删除，同一份 [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) 不再被双身份注册
- `UiHostWindow` 相关 component descriptor 也同步改成指向 `ui.host_window`
- `editor_ui_host_runtime_registers_only_generic_host_window_document_id` 会验证 runtime projection 和 shared surface tree id 都走 `ui.host_window`，`builtin_host_runtime_exposes_only_generic_host_window_document_id` 会守住 `template_runtime` 与对应测试不再恢复旧 alias 常量或旧 literal
- `EditorUiHostRuntime` 新增 generic `load_builtin_host_templates()`，把“加载一组 builtin host template”与“加载 workbench shell”两个概念拆开
- former `workbench.slint` 的导出 root 在 fence 前已经跟着这个 identity 收口；当前 active Rust surface 由 `host_contract/window.rs` 承接：`UiHostWindow` 只剩 window/bootstrap wrapper，bootstrap 符号统一为 `UiHostContext`、`UiHostScaffold` 和 `HostWindowSceneData`
- host presentation / scene contract 也同步去掉首批 workbench 专名：`workbench_scene_data` 已改为 `host_scene_data`，`HostWorkbenchSurfaceMetricsData` / `HostWorkbenchSurfaceOrchestrationData` 已改为 `HostWindowSurfaceMetricsData` / `HostWindowSurfaceOrchestrationData`
- host drag / resize pointer event 也已经从 `workbench_drag_pointer_event` / `workbench_resize_pointer_event` 收口为 `host_drag_pointer_event` / `host_resize_pointer_event`
- 2026-04-29 追加拖拽释放修正：原 tab 输入控件现在会在自己收到 `PointerEventKind.up` 时提交 `host_drag_pointer_event(2, ...)` 并清空 `UiHostContext.drag_state`，不再依赖后显示的全屏 `HostTabDragOverlay` 捕获释放；旧 `TabChip` / `DockTabButton` 也改为只要处于 suppress-click 的拖拽状态就发出 `drag_finished`，避免跨过移动阈值后本地 press capture 被清掉导致释放丢失。
- 2026-04-29 追加 resize 帧驱动修正：`HostResizeLayer` 现在在 `UiHostContext.resize_state.resize_active` 时启动 16ms `Timer`，鼠标 move 只更新 `resize_pointer_x/y`，真正的 `host_resize_pointer_event(1, ...)` 由帧脉冲统一提交；释放鼠标仍立即提交 `host_resize_pointer_event(2, ...)`，避免布局大小只跟随鼠标事件批次跳动。
- Rust host-page pointer helper 也已经从 `WorkbenchHostPagePointer*` / `build_workbench_host_page_pointer_layout` 收口为 `HostPagePointer*` / `build_host_page_pointer_layout`
- Rust menu pointer helper 也已经从 `WorkbenchMenuPointer*` / `build_workbench_menu_pointer_layout` 收口为 `HostMenuPointer*` / `build_host_menu_pointer_layout`
- former `HostMenuChrome` 的业务 menu row 已经从 Slint 硬编码迁到 Rust-owned [`HostMenuChromeData.menus`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/data/host_components.rs)，由 [`scene_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 从 `WorkbenchViewModel.menu_bar` 投影 `HostMenuChromeMenuData` / `HostMenuChromeItemData`
- menu pointer layout 同步接收同一份 `MenuBarModel`，让 popup 可见 rows 与 `HostMenuPointerLayout.menus` 的 hit-test/action rows 不再各自维护一套硬编码业务列表；Window menu 仍在 projection 中追加 preset save/load rows，保持现有 `SavePreset.*` / `LoadPreset.*` action contract
- `HostMenuChrome` 的六个 top-level menu button frame 现在由 `workbench_menu_chrome.ui.toml` 的 `MenuSlot*` frame 投影成 `HostChromeControlFrameData`；popup anchor 和 menu pointer hit-test 共用这些 TOML frame，不再保留 Slint/Rust 侧的 button width/gap/height 常量
- `HostMenuChrome` 的 popup row 视觉也已经切到 [`workbench_menu_popup.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_menu_popup.ui.toml)：[`scene_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 先把 `HostMenuChromeMenuData.items` 投影为 `popup_nodes`，Rust-owned host projection 只用 `TemplatePane` 渲染这些节点并保留整窗透明 pointer 转发，不再用 `MenuItemRow`/`VerticalLayout` 手画业务菜单项。Window menu 超过 16 个 `.ui.toml` authored row slots 时，[`chrome_template_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs) 会把 `MenuPopupItemRow*` / label / shortcut slots 当作 TOML row stencil 投影成 absolute control id；`TemplatePane` 再按 `UiHostContext.menu_state.window_menu_scroll_px` 和 popup viewport height cull offscreen rows，所以 row 16+ 的 visual control id、hover highlight 和 pointer/action item index 保持同一套 absolute row 语义。
- 2026-04-29 post-review overflow hardening 继续保持 popup panel 视觉来自 TOML：`TemplatePane` 新增 `fixed_control_id` / `fixed_y_offset`，Window menu 滚动时只让 item rows 随 content offset 滚动，`WorkbenchMenuPopupPanel` 用同一份 TOML node counter-scroll 留在 viewport 外框位置；`chrome_template_projection.rs` 同步把 `HostMenuChromeItemData.enabled == false` 投影成 label/shortcut `text_tone = "muted"`，让 disabled row 的视觉状态和 pointer dispatch 的 disabled 状态一致；Window popup scroll 后 `HostMenuPointerBridge` 还会重新 dispatch 当前 pointer `Move`，让 hover row index 立即从新的 scroll offset 下重新求值。
- Host shell chrome 的视觉尺寸权威已经继续从 `.slint` 迁到 root `.ui.toml`：[`workbench_menu_chrome.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml)、[`workbench_page_chrome.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml)、[`workbench_dock_header.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml)、[`workbench_status_bar.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_status_bar.ui.toml) 和 [`workbench_activity_rail.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml) 现在描述顶部菜单、Workbench/Page tab、dock header、floating window header、status bar 和 side activity rail 的 label/frame/font/spacing；[`chrome_template_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs) 负责把这些 asset 经过 shared `UiTemplateSurfaceBuilder` / layout pass 投影成 `ViewTemplateNodeData`、`HostChromeControlFrameData` 和 `HostChromeTabData`
- former host Slint files are no longer active assets or test references: their last active responsibilities have been replaced by `TemplatePane` nodes plus Rust-owned transparent input forwarding; current Rust-owned host contract continues to expose the callback/DTO surface, while input frame、activity rail visual、status text 和 floating header chrome visual 不再由 Slint button/chip/text 控件手写
- `HostPageChrome` 的透明 page-tab hitbox 现在把 tab frame 和 pointer fact 转成相对 `page_data.tab_row_frame` 的坐标再发给 `host_page_pointer_clicked`；`HostPagePointerBridge` 仍负责叠加 strip frame，因此 Slint adapter 不会把 top chrome/page strip origin 重复加一次
- 2026-04-29 追加修正：[`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) 现在把 `HostPageStripRoot` 作为 `WorkbenchMenuBarRoot` 与 `WorkbenchBody` 之间的真实 vertical band，而不是隐藏的 overlay marker；该 band 为 page strip 保留 32px，并额外保留 1px separator，因此正文、activity rail、drawer source 与 floating-window source 都从 59px 的 top chrome boundary 开始
- [`workbench_page_chrome.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml) 的 `WorkbenchPageBar` 已从 24px 提升到 32px，page tab hitbox 同步为 108x30；当前 Rust-owned `HostPageChromeData` 的 fallback `host_bar_height_px` 也同步为 32px，避免首帧/空数据状态短暂回落到旧高度；[`WorkbenchChromeMetrics`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/autolayout/workbench_chrome_metrics.rs) 的 `host_bar_height` 与 `status_bar_height` 也和这条 shell/template contract 对齐，避免 asset 视觉尺寸和 Rust fallback geometry 再次分叉
- Rust activity rail pointer helper 也已经从 `WorkbenchActivityRailPointer*` / `build_workbench_activity_rail_pointer_layout` 收口为 `HostActivityRailPointer*` / `build_host_activity_rail_pointer_layout`
- Rust document tab pointer helper 也已经从 `WorkbenchDocumentTabPointer*` / `build_workbench_document_tab_pointer_layout` 收口为 `HostDocumentTabPointer*` / `build_host_document_tab_pointer_layout`
- Rust drawer header pointer helper 也已经从 `WorkbenchDrawerHeaderPointer*` / `build_workbench_drawer_header_pointer_layout` 收口为 `HostDrawerHeaderPointer*` / `build_host_drawer_header_pointer_layout`
- Rust shell pointer helper 也已经从 `WorkbenchShellPointer*` / `workbench_shell_pointer_*` 收口为 `HostShellPointer*` / `host_shell_pointer_*`
- Rust tab drag helper 也已经从 `WorkbenchDragTarget*` / `ResolvedWorkbenchTabDrop*` / `resolve_workbench_*` 收口为 `HostDragTarget*` / `ResolvedHostTabDrop*` / `resolve_host_*`
- Rust resize target helper 也已经从 `WorkbenchResizeTargetGroup` / `resolve_workbench_resize_target_group` 收口为 `HostResizeTargetGroup` / `resolve_host_resize_target_group`
- Rust root shell frames DTO 也已经从 `BuiltinWorkbenchRootShellFrames` / `workbench_body_frame` 收口为 `BuiltinHostRootShellFrames` / `host_body_frame`
- Rust builtin host projection builder 也已经从 `build_builtin_workbench_host_projection` 收口为 `build_builtin_host_window_projection`
- Rust drawer source bridge 也已经从 `BuiltinWorkbenchDrawerSource*` / `build_builtin_workbench_drawer_source_surface` 收口为 `BuiltinHostDrawerSource*` / `build_builtin_host_drawer_source_surface`
- 2026-04-29 drawer source projection 又把 compact bottom drawer height limit 下沉到 shared drawer-source layout：[`workbench_drawer_source/layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source/layout.rs) 在 host projection 消费前就对 bottom drawer frame 做同一套 compact clamp，因此 root host、host window bridge 和 zeroed legacy geometry 路径都消费同一份 shared frame truth，而不是分别在上层重算 bottom/center extents
- 内置 template runtime 现在会递归解析 `res://...` widget imports，并允许 layout root 通过 `#RootControlId` 作为嵌入组件使用
- `active_editor_ui_tree_contains_no_slint_sources` 会守住 active `zircon_editor/ui` tree 不再恢复 `.slint` 源。
- `editor_ui_toml_assets_replace_former_workbench_source_roles` 会守住 `workbench_shell.ui.toml`、menu/page/dock header、activity rail、status bar、welcome、UI asset editor 和 component showcase 这些 `.ui.toml` 资产继续替代 former workbench source roles。
- `rust_projection_module_remains_the_editor_host_layout_authority` 会守住当前 Rust projection seam 仍在 `workbench_host_window`，防止文档样例里的 `host_window` move target 被误当成已经完成的代码事实。
- `editor_host_sources_do_not_depend_on_deleted_slint_trees` 会守住 deleted Slint tree、generated Slint include/build seam、`temp/slint-migration/**` 和 `as slint_ui` 兼容别名不再被 editor host source 读取。
- `builtin_host_runtime_exposes_only_generic_host_window_document_id` 会守住 builtin host template registration 只暴露 generic `ui.host_window` 文档身份，不再保留旧 workbench shell alias。
- `slint_host_module_exports_rust_owned_contracts_without_generated_modules` 与 `rust_owned_host_contract_declares_window_globals_and_projection_data` 会守住 `slint_host` glue 只导出 Rust-owned `host_contract/**` DTO/callback seam，而不是恢复 generated Slint modules。
- `editor_ui_toml_assets_are_the_host_chrome_authority` 会守住 host chrome 视觉 authority 来自 `.ui.toml` 资产和 Rust projection。
- `runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface` 会守住 runtime input 只通过 owned `UiSurface` 的 shared dispatch/focus path。
- `render_framework_submits_all_builtin_runtime_ui_fixtures` 会守住 runtime fixture `.ui.toml -> UiSurface -> UiRenderExtract` 输出进入 screen-space UI pass，而不新建 fixture-specific renderer 分支。
- `host_menu_chrome_uses_projected_menu_model_instead_of_hardcoded_business_rows` 会守住 business menu labels/items 只来自 Rust host projection 和 menu model，不再回到任何 deleted Slint shell
- `host_menu_chrome_uses_projected_template_control_frames_for_menu_hit_areas` 会守住 menu hit areas 来自 `.ui.toml` projected control frames 和 menu pointer layout，不再复制 top-level menu button frame 常量
- `host_status_bar_visuals_come_from_ui_toml_template_nodes` 会守住 status bar root/panel/primary/secondary/viewport label 来自 `workbench_status_bar.ui.toml` 和 `HostStatusBarData.template_nodes`，并禁止 status text、panel background 或 separator visual 回到 hand-written host chrome
- `host_floating_window_headers_use_projected_template_nodes` 会守住 embedded/native floating window header 的 `header_nodes`、`header_frame` 和 `tab_frames` 来自 `workbench_dock_header.ui.toml` 投影，并禁止 floating header 重新使用 `TabChip`、`text: window.title` 或 header `palette.chrome_bg` 视觉
- `host_side_activity_rails_use_projected_template_nodes` 会守住 side activity rail 的 `rail_nodes`、`rail_button_frames` 和 `rail_active_control_id` 来自 `workbench_activity_rail.ui.toml` 投影，并禁止 rail visual 回到 `RailButton` 或 `palette.rail_bg` 手写视觉
- `activity_rail_pointer_module_uses_generic_host_type_names` 会守住 activity rail pointer helper 不再回到 workbench 专名
- `document_tab_pointer_module_uses_generic_host_type_names` 会守住 document tab pointer helper 不再回到 workbench 专名
- `drawer_header_pointer_module_uses_generic_host_type_names` 会守住 drawer header pointer helper 不再回到 workbench 专名
- `shell_pointer_module_uses_generic_host_type_names` 会守住 shell pointer helper 不再回到 workbench 专名
- `tab_drag_module_uses_generic_host_type_names` 会守住 tab drag helper 不再回到 workbench 专名
- `drawer_resize_module_uses_generic_host_type_names` 会守住 resize target helper 不再回到 workbench 专名
- `root_shell_frames_use_generic_host_type_names` 会守住 root shell frames DTO 不再回到 workbench 专名
- `drawer_source_bridge_uses_generic_host_type_names` 会守住 drawer source bridge 不再回到 workbench 专名
- 这层 wrapper 目前仍通过属性别名、typed event 名称和 callback forwarding 暂时保留部分 workbench 业务 ABI，因此 shared template/runtime 的 generic root identity 可以先稳定下来，而不需要一次性重写所有 host/slint 业务接线

2026-05-01 继续清理 builtin root document identity 后，focused validation 重新覆盖当前 active path：`builtin_host_runtime_exposes_only_generic_host_window_document_id` 通过 1 passed / 0 failed / 847 filtered out，`template_runtime` 通过 36 passed / 0 failed / 812 filtered out，`generic_host_boundary` 通过 8 passed / 0 failed / 840 filtered out，`catalog_registry` 通过 3 passed / 0 failed / 845 filtered out。本轮触及 Rust 文件的 `rustfmt --edition 2021 --check` 通过；exact source sweeps confirmed the old builtin host document literal and `LEGACY_HOST_WINDOW_DOCUMENT_ID` are gone from active `zircon_editor/src` and docs, while broader `workbench.shell` regex matches only unrelated `editor.workbench.shell_pointer.*` routing ids under shell pointer surfaces. Touched-file `git diff --check` only reported Windows LF-to-CRLF warnings.

当前验证状态：该 menu chrome guard 已先按 TDD RED 确认会在缺少 projected menu DTO 时失败。2026-04-29 root `.ui.toml` chrome cutover 的最新验证在当前 workspace 重新跑完：lower shared runtime 路径已经从旧 `GpuCompletions` 收敛到 `collect_runtime_feedback(...)` / `RuntimeFeedbackBatch::into_parts(...)`，两个 submit entry point 只拆包 `HybridGiRuntimeFeedback` 与 `VirtualGeometryRuntimeFeedback` 后进入 `record_submission(...)`。focused chrome guards 通过 `workbench_chrome_projection_uses_user_requested_asset_paths`、`slint_chrome_inputs_use_projected_control_frames_instead_of_local_geometry_math`、`workbench_chrome_heights_are_loaded_from_toml_assets_not_scene_constants`、`host_menu_chrome_uses_projected_template_control_frames_for_menu_hit_areas`。外部 review 暴露 TemplatePane cutover 后 top-level menu hover/open highlight 会丢失，本轮先新增 RED guard `host_menu_chrome_projects_hovered_or_open_menu_state_into_template_highlight`，确认缺少 `highlighted_control_id` 投影会失败；随后 `HostMenuChrome` 从 `UiHostContext.menu_state.hovered_menu_index` / `open_menu_index` 解析 highlighted `control_id`，`TemplatePane` 对匹配 Label 节点绘制 `palette.chrome_bg_soft` 高亮并提升文本色。复跑 `cargo test -p zircon_editor --lib host_menu_chrome_projects_hovered_or_open_menu_state_into_template_highlight --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 1 test / 0 failed / 889 filtered out；`cargo test -p zircon_editor --lib slint_menu_pointer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 12 tests / 0 failed / 878 filtered out；`cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 29 tests / 0 failed / 861 filtered out。`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never`、`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never` 和 `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never` 均通过；当前 warnings 是既有 `sdf_atlas::slot_count`、`editor_asset_manager::editor_meta::save` 和 `showcase_demo_state` dead-code。

历史 host page tabs target 补充验证：2026-04-29 早前复跑 `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-host-page-tabs --message-format short -- --test-threads=1 --nocapture` 时覆盖 TemplatePane root chrome asset guard、projected input frame guard、chrome height asset guard 和 generic host naming guards；之后 popup/highlight guard 扩充改变了 focused `generic_host_boundary` 的测试数量，因此这里不再把旧 target 的通过数量当作当前 guard 规模。随后 `cargo test -p zircon_editor --lib host_page_pointer --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-host-page-tabs --message-format short -- --test-threads=1 --nocapture` 通过 7 passed / 0 failed / 881 filtered out，确认 page tab pointer bridge 仍按 shared strip frame route 命中。`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-host-page-tabs --message-format short` 通过，仅保留既有 `editor_meta::save` 与 `showcase_demo_state` dead-code warnings；`cargo fmt --package zircon_editor -- --check` 与当前 editor chrome slice 文件的 `git diff --check` 通过，diff-check 仅报告 Windows LF-to-CRLF 提示。

Post-review guard hardening 又把 `generic_host_boundary.rs` 的 source guard 从简单 substring 检查推进到两个更严格条件：`TemplatePane` 必须是实际 `TemplatePane { ... }` render block 且 block 内含对应 `nodes:` binding；`HostPageChrome` 的 `host_page_pointer_clicked(...)` 必须按完整 argument order 传入相对 `page_data.tab_row_frame` 的 tab x、tab width、pointer x、pointer y。该 guard fix 经只读 re-review 无 findings。早前 Cargo 复跑曾在 focused tests 执行前被另一个 active Runtime UI showcase slice 阻断：`zircon_editor/src/ui/template_runtime/showcase_demo_state.rs:142` 缺少 `UiComponentShowcaseDemoState::project_state_panel_node`，用户选择不在本 chrome slice 接管该 blocker；后续相关 lower-layer source 已收敛。再后续的 Hybrid GI resolve DTO private-field compile blocker 也已通过既有 `HybridGiResolveProbeSceneData` / `HybridGiResolveTraceRegionSceneData` constructor/accessor seam 收口。当前 `D:\cargo-targets\zircon-editor-host-page-tabs` focused lane 已重新通过：`cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short` green，`generic_host_boundary` 31 passed / 0 failed / 873 filtered out，`host_page_pointer` 7 passed / 0 failed / 897 filtered out，`cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short` green。格式 caveat：触及的 Hybrid GI runtime 大文件仍有既有 rustfmt drift，因此这里不声称 workspace fmt green。

本轮 page-strip 高度补丁的轻量验证先按 RED/GREEN 复跑 `rustc --edition=2021 --test zircon_editor\src\tests\host\slint_window\generic_host_boundary.rs` 生成的 source guard：`host_page_strip_reserves_real_shell_space_before_workbench_body` 与 `host_page_chrome_asset_uses_full_height_tabs` 在旧布局下分别失败，修正后 `generic_host_boundary` 31 tests 全部通过；最新复跑还把 host page chrome `.ui.toml` / Rust-owned fallback 32px 高度纳入 `host_page_chrome_asset_uses_full_height_tabs`。随后对本轮触及的 Rust 文件执行 `rustfmt --edition 2021 --check --config skip_children=true ...` 通过，针对本轮触及文件的 `git diff --check` 通过且只报告 Windows LF-to-CRLF 提示。完整 Cargo 投影测试 `cargo test -p zircon_editor builtin_host_window_template_bridge_recomputes_surface_backed_frames_with_shell_size --locked --no-run` 在当前机器上 124s 超时，且同一 target 下已有其他 session 的 cargo/rustc 编译进程；因此本轮未声称完成可执行截图验证。

2026-04-29 popup wrapper 收口补充验证：`host_menu_popup_visuals_come_from_ui_toml_template_nodes` 先守住 `HostMenuChromeData.popup_nodes`、`workbench_menu_popup.ui.toml`、`menu_popup_nodes` 投影入口，并禁止 deleted host chrome shell 重新拥有 `MenuItemRow`、popup panel `border-radius`、`background`、`border-width`、`border-color` 这些视觉 token。补充复跑 `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never` 通过，仅保留 Hybrid GI / Virtual Geometry / UI SDF 既有 warning；随后用 standalone source guard `rustc --edition=2021 --test zircon_editor\src\tests\host\slint_window\generic_host_boundary.rs -o E:\cargo-targets\zircon-editor-workbench-chrome-layout\generic_host_boundary_tests.exe` 并执行 `generic_host_boundary_tests.exe --exact host_menu_popup_visuals_come_from_ui_toml_template_nodes --nocapture`，结果 1 passed / 0 failed / 30 filtered out。早前完整 `cargo test -p zircon_editor --lib host_menu_popup_visuals_come_from_ui_toml_template_nodes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 曾两次在 Windows editor test binary 编译阶段超出 120s 与 600s 工具超时；当时并行存在其他 `zircon_editor` / Hybrid GI Cargo 编译进程，后续复跑结果见下段。

同日后续复跑已经补齐上面的 Cargo 证据：`cargo test -p zircon_editor --lib host_menu_popup_visuals_come_from_ui_toml_template_nodes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 1 passed / 0 failed / 897 filtered out；`cargo test -p zircon_editor --lib slint_menu_pointer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 12 passed / 0 failed / 886 filtered out；`cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 31 passed / 0 failed / 867 filtered out。复跑中曾先暴露 active Hybrid GI cutover 的 lower-layer private-field compile blocker；修正点是让 `build_resolve_runtime.rs` 与 `scene_trace_support.rs` 消费 `HybridGiRuntimeProbeSceneData` / `HybridGiRuntimeTraceRegionSceneData` 已有 accessor，而不是重新读取 owner-private quantized fields。随后 `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never`、`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never`、`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never` 与 `rustfmt --edition 2021 --check zircon_runtime\src\graphics\runtime\hybrid_gi\build_resolve_runtime.rs zircon_runtime\src\graphics\runtime\hybrid_gi\scene_trace_support.rs` 均通过；剩余输出是 `selection_collection` / `seed_backed_execution_selection` / `sdf_atlas` / `editor_meta::save` / `showcase_demo_state` 的既有 dead-code warnings。

2026-04-29 Window menu popup overflow 补充验证：外部 review 指出 `workbench_menu_popup.ui.toml` 只 author 了 16 个 row slots，而 Window presets 可以超过 16，旧实现会让 pointer/action row 16+ 可点但视觉仍停在前 16 行。本轮先新增 RED `menu_popup_nodes_project_absolute_rows_beyond_authored_slots`，确认旧 projection 缺失 `MenuPopupItemLabel16`；修正后复跑 `cargo test -p zircon_editor --lib menu_popup_nodes_project_absolute_rows_beyond_authored_slots --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 1 passed / 0 failed / 899 filtered out。随后新增 RED source guard `host_menu_chrome_virtualizes_scrolled_window_popup_template_rows`，让 `HostMenuChrome` 把 Window popup scroll offset/viewport height 传给 `TemplatePane`，`TemplatePane` 对 offscreen rows 设置 `visible: false`；GREEN 复跑同名 test 通过 1 passed / 0 failed / 899 filtered out。`shared_menu_pointer_click_dispatches_scrolled_window_preset_selection` 已扩到 20 个 presets 并点击 absolute item index 17，复跑通过 1 passed / 0 failed / 899 filtered out；`cargo test -p zircon_editor --lib slint_menu_pointer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 13 passed / 0 failed / 887 filtered out；`cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 31 passed / 0 failed / 869 filtered out。默认 runtime check 首次遇到 active render-plugin session 正在收敛的 `HybridGiRuntimeState.resident_slots` private-field drift；当前源已通过 accessor 收敛后，重新运行 `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never`、`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never` 和 `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never` 均通过；仅保留 `resident_slot` 未使用 warning。`rustfmt --edition 2021 --check` 覆盖本轮三个 Rust 文件，`git diff --check` 覆盖本轮 Rust/Slint 文件且只输出 Windows LF-to-CRLF 提示。

2026-04-29 Window menu popup overflow post-review closure：review 又指出三处风险，分别是 scroll 后 hover index 不会按新 offset 重算、`WorkbenchMenuPopupPanel` 和 rows 共用一个 translated `TemplatePane` 导致 panel 跟着内容滚动、disabled item 只在 pointer dispatch 里禁用但视觉仍像 enabled。本轮按 RED/GREEN 增补 `shared_menu_pointer_bridge_recomputes_hovered_item_after_window_popup_scroll`、`host_menu_chrome_keeps_popup_panel_fixed_while_virtual_rows_scroll`、`menu_popup_projection_mutes_disabled_item_labels`，并把 `menu_popup_nodes_project_absolute_rows_beyond_authored_slots` 扩到 disabled row 17 的 muted text-tone 断言。最新验证：standalone source guard `rustc --edition=2021 --test zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs` 通过 7 passed / 0 failed；`cargo test -p zircon_editor --lib menu_popup_nodes_project_absolute_rows_beyond_authored_slots --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 1 passed / 0 failed / 902 filtered out；`cargo test -p zircon_editor --lib slint_menu_pointer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 16 passed / 0 failed / 887 filtered out；`cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short -- --test-threads=1 --nocapture` 通过 31 passed / 0 failed / 872 filtered out；`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never`、`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never`、`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-chrome-layout --message-format short --color never` 均通过；`rustfmt --edition 2021 --check` 覆盖本轮触及 Rust 文件通过；`git diff --check` 覆盖本轮触及文件通过且只报告 Windows LF-to-CRLF 提示。最终补充的 ignored visual test `capture_scrolled_window_popup_visual_artifact` 用 real Slint software renderer 写出 `target/visual-layout/editor-window-20260429-window-popup-scrolled-900x620.png`，截图显示 Window menu 在 360px scroll offset 下保持 fixed panel 外框、rows 10-15 可见且 hover 高亮落在可见 row 上。

最终 review 还指出 enabled shortcut text-tone 被 disabled-state helper 误覆盖成 `default`，会破坏 `workbench_menu_popup.ui.toml` 对 shortcut 的 muted 视觉权威。随后把 `menu_popup_nodes_project_absolute_rows_beyond_authored_slots` 扩成 RED，确认 `MenuPopupItemShortcut16` 从 `muted` 变成 `default`；GREEN 后只在 `!item.enabled` 时覆盖 label/shortcut 为 `muted`，enabled shortcut 保留 TOML-authored tone。复跑该 focused test、`slint_menu_pointer`、`generic_host_boundary`、`cargo check -p zircon_editor --lib` 和 `rustfmt --edition 2021 --check` 均通过。

2026-04-29 status bar cutover 继续削掉 host scene 的 Slint visual authority：新增 [`workbench_status_bar.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_status_bar.ui.toml) 定义 `WorkbenchStatusBarRoot`、separator、panel、primary/secondary/viewport labels；[`chrome_template_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs) 通过 `status_bar_nodes(...)` 把 `HostStatusBarData.status_primary/status_secondary/viewport_label` 覆盖进 template labels。当前 status bar visual authority 是 `.ui.toml` asset + Rust-owned host projection；deleted Slint copy is non-authoritative and is not a render/test source. TDD 证据：`host_status_bar_visuals_come_from_ui_toml_template_nodes` 先在缺少 `workbench_status_bar.ui.toml` 时 RED，GREEN 后 standalone source guard 通过；随后 `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short -- --test-threads=1 --nocapture` 通过 32 passed / 0 failed / 874 filtered out，`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short --color never` 通过。

2026-04-29 floating window header cutover 继续复用 dock header template seam：[`chrome_template_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs) 新增 `floating_window_header_nodes(...)`，把 floating window tab titles 和 window title 覆盖进 [`workbench_dock_header.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml) 的 `DockTab*` / `DockSubtitle` stencil；[`scene_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 再把 `header_nodes`、`header_frame` 和 `tab_frames` 写入每个 `FloatingWindowData`。当前 floating header authority 是 `workbench_dock_header.ui.toml` + Rust-owned host projection/input forwarding；deleted floating-window Slint copies are non-authoritative. TDD 证据：`host_floating_window_headers_use_projected_template_nodes` 先在缺少 `floating_window_header_nodes` 时 RED，GREEN 后 standalone source guard 通过；随后 `cargo test -p zircon_editor --lib host_floating_window_headers_use_projected_template_nodes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short -- --test-threads=1 --nocapture` 通过 1 passed / 0 failed / 906 filtered out，`cargo test -p zircon_editor --lib native_floating_window_mode_forwards_tabs_header_and_pane_callbacks_to_root --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short -- --test-threads=1 --nocapture` 通过 1 passed / 0 failed / 906 filtered out，`cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short -- --test-threads=1 --nocapture` 通过 35 passed / 0 failed / 872 filtered out，最终格式后复跑 `cargo test -p zircon_editor --lib slint_window --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short -- --test-threads=1 --nocapture` 通过 41 passed / 0 failed / 866 filtered out，`cargo test -p zircon_editor --lib slint_tab_drag --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short -- --test-threads=1 --nocapture` 通过 35 passed / 0 failed / 872 filtered out，`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short --color never` 通过；触及 Rust 文件的 `rustfmt --edition 2021 --check` 通过，targeted `git diff --check` 仅报告 Windows LF-to-CRLF 提示。

2026-04-29 side activity rail cutover 继续削掉 side dock 的 Slint visual authority：新增 [`workbench_activity_rail.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml) 定义 `ActivityRailPanel`、两行 `ActivityRailButton*` stencil 和 `ActivityRailButtonLabel*` label stencil；[`chrome_template_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs) 新增 `activity_rail_nodes(...)`、`activity_rail_button_frames(...)` 和 `activity_rail_active_control_id(...)`，把 side dock tabs 投影为 absolute rail control ids、active inset surface variant 和 active highlighted label；[`scene_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 把这些 rail nodes/frame/id 写入 left/right `HostSideDockSurfaceData`。当前 side rail visual/frame authority 是 `workbench_activity_rail.ui.toml` + Rust-owned host projection；deleted side-dock Slint copy is non-authoritative. TDD 证据：`host_side_activity_rails_use_projected_template_nodes` 先在缺少 `workbench_activity_rail.ui.toml` 时 RED，随后又暴露 helper 名称残留 `RailButton` 的 guard failure；GREEN 后 standalone source guard 通过，`cargo test -p zircon_editor --lib host_side_activity_rails_use_projected_template_nodes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short -- --test-threads=1 --nocapture` 通过 1 passed / 0 failed / 907 filtered out，`cargo test -p zircon_editor --lib slint_window --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short -- --test-threads=1 --nocapture` 通过 42 passed / 0 failed / 866 filtered out，`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-statusbar-ui-toml --message-format short --color never` 通过；触及 Rust 文件 `rustfmt --edition 2021 --check` 通过，targeted `git diff --check` 仅报告 LF-to-CRLF warning。

2026-04-29 后续 editor/template closeout 把 stale Slint/template expectations 与 drawer shared-source projection 收口到当前 shared truth：document tab/native floating source guards 改为检查 `UiHostContext` generic callbacks，repository snapshot 跟随 `workbench_shell.ui.toml` 的真实 root children，welcome mount 和 floating-window projection 断言改为消费 shared pane/source frame，drawer source bottom compaction 则在 shared `workbench_drawer_source/layout.rs` 里完成。最新 broad editor lib 复跑在 `D:\cargo-targets\zircon-render-plugin-final` 通过 `cargo test -p zircon_editor --lib --locked --jobs 1 --color never -- --test-threads=1`，结果 906 passed / 0 failed / 1 ignored；同一延续的 Runtime UI suite 通过 `cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final --color never -- --nocapture`，结果 126 passed / 0 failed，并覆盖 `render_extract_uses_label_when_schema_text_default_is_empty` 的 shared text/label fallback。

2026-04-30 Slint fence root seam 收口：active `zircon_editor/ui/**/*.slint` 继续保持 0 个文件，source guard 现在通过 `generic_host_layout_paths.rs` 明确断言 active tree 无 Slint 源，并且 `generic_host_boundary.rs` 禁止 editor host source 回读 deleted Slint trees、`slint::include_modules!()`、`slint_build` / `slint-build` build seam 或 `as slint_ui` 兼容 alias。`zircon_editor/build.rs` 不再编译 active `ui/workbench.slint`，不再 staging deleted Slint tree 到 `OUT_DIR`，也不再调用 `slint_build`；`zircon_editor/src/ui/slint_host/mod.rs` 已移除 `slint::include_modules!()`，改为导出 `host_contract/**` 里的 Rust-owned `UiHostWindow`、`UiHostContext`、`PaneSurfaceHostContext`、`FrameRect`、`PaneData` 和 host presentation DTO。`apply_presentation.rs`、`pane_data_conversion/**` 与 `template_node_conversion.rs` 现在把 Rust-owned surface 命名为 `host_contract`。验证：`cargo test -p zircon_editor --lib editor_host_sources_do_not_depend_on_deleted_slint_trees --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` 先因 `as slint_ui` RED，改名后通过 1 passed / 0 failed / 841 filtered out；`cargo test -p zircon_editor --lib editor_host_source_guard_rejects_hyphenated_generated_build_dependency --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` 先因未禁止 `slint-build` RED，补 guard 后通过 1 passed / 0 failed / 842 filtered out；`cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` 通过 6 passed / 0 failed / 837 filtered out，`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` 通过，`cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` 通过 27 passed / 0 failed。后续 cleanup 已把局部 conversion helper 收敛到 `to_host_contract_*` terminology；当前不再存在 active 或 deleted `.slint` source reader。

2026-04-30 后续验证补齐了 module plugins pane body binding 与 no-Slint fence 的完整 editor lib 证据：`ModulePluginsPaneBody/FocusModulePlugins` 现在由 `template_bindings.rs` 注册为 `DockCommand::FocusView { instance_id: "editor.module_plugins#1" }`，`builtin_pane_body_` focused tests 通过 2 passed / 0 failed；`cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --nocapture` 通过 841 passed / 0 failed / 1 ignored；`cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --nocapture` 通过 27 passed / 0 failed。浮窗 child-window hierarchy pointer 的回归期望也已从 legacy `shell_geometry` 改为 cached shared `FloatingWindowProjectionBundle.content_frame`，并用 geometry-difference guard 防止再把旧 outer/content geometry 当成 callback sizing authority。

同一 Module Plugins slice 又补了 scoped focused evidence：`module_plugins_body.ui.toml` 作为 `pane.module_plugins.body` 进入 builtin document/component registry，`module_plugin_list_slot` 是 stable hybrid slot，`PanePayload::ModulePluginsV1` 会携带 plugin catalog payload。`cargo test -p zircon_editor --lib builtin_hybrid_pane_body_documents_declare_stable_native_slot_names --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` 通过 1 passed / 0 failed，`cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` 通过 1 passed / 0 failed。

2026-04-30 Runtime UI dispatch acceptance now proves `RuntimeUiManager` does not bypass the shared surface when receiving host input. The manager exposes crate-local pointer/navigation forwarding methods, while `UiSurface` still owns hit-test capture, focus state, and navigation handled results. Validation: `cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed 1 passed / 0 failed / 1190 filtered out, `cargo test -p zircon_runtime --lib ui_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed 17 passed / 0 failed / 1174 filtered out, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed. The corresponding no-Slint editor fence reruns also passed: `generic_host_layout_paths` 3 passed / 0 failed and `generic_host_boundary` 6 passed / 0 failed on `E:\cargo-targets\zircon-ui-cutover-move-first`.

2026-04-30 Runtime UI graphics fixture acceptance now submits all builtin Runtime UI fixtures through `WgpuRenderFramework::submit_runtime_frame(...)` under the `runtime-ui-integration-tests` feature, proving the shared `.ui.toml -> UiSurface -> UiRenderExtract` path reaches the screen-space UI pass for HUD, pause menu, settings dialog, and inventory list fixtures. Validation: `cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture` passed 1 passed / 0 failed / 1195 filtered out, and `cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture` passed 7 passed / 0 failed.

2026-04-30 generic host catalog leaf-name cutover moved active host leaf visuals from transitional `UiHost*` component names to shared Runtime UI `IconButton` / `Label` names. The adapter rejects legacy leaf-name aliases and now resolves visible host text the same way as `UiRenderExtract`: non-empty `text` wins, then non-empty `label` is used when schema defaults inject `text = ""`. Focused evidence on `E:\cargo-targets\zircon-ui-cutover-move-first`: `generic_host_catalog_uses_shared_runtime_component_names` passed 1 / 0, editor `template_runtime` passed 36 / 0 after the label fallback fix, runtime `ui::tests` passed 126 / 0, full editor `generic_host_boundary` passed 7 / 0, `screen_space_ui_plan` passed 4 / 0, `ui_boundary` passed 17 / 0, and `render_framework_submits_all_builtin_runtime_ui_fixtures` with `runtime-ui-integration-tests` passed 1 / 0. The broader `cargo test -p zircon_runtime --lib runtime_ui_integration --features runtime-ui-integration-tests ...` command did not reach Runtime UI assertions because active native-plugin-loader work currently fails compilation before the tests run.

2026-04-30 host-contract helper naming cleanup completed the local conversion helper cutover that followed the root no-Slint fence. `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs`, `pane_data_conversion/**`, and `template_node_conversion.rs` now use `to_host_contract_*` helper names for Rust-owned host-contract DTO projection, while the test-facing re-exports and boundary assertions also point at the new names. The cleanup did not restore active `.slint` sources, generated Slint modules, or compatibility aliases; it only removes stale local naming debt around the Rust-owned `host_contract/**` surface. Validation: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed; final `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir E:\cargo-targets\zircon-ui-cutover-move-first` passed workspace build and test with `--locked` after one stale test-only `to_slint_host_scene_data` import was renamed; targeted `rustfmt --edition 2021 --check` passed; `git diff --check` reported only LF-to-CRLF warnings; and `zircon_editor/ui/**/*.slint` remained empty.

这样 shared template runtime 对外暴露的默认 root 入口已经不再是 workbench 业务名；旧 builtin host document alias 已删除，`workbench` 只剩资产文件名、业务域名和未迁完的 editor shell 子结构，而不是 root document identity。后续继续做 `Generic host boundary` 时，就可以在不改 shared runtime 主入口命名的前提下，逐步削掉 former Slint business shell 和 builtin projection 里的业务壳结构。
