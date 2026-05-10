---
related_code:
  - zircon_editor/Cargo.toml
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_asset_workspace.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_session_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/inspector.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/navigation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/node_ops.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/open.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/save.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/workspace_state.rs
  - zircon_editor/src/ui/host/ui_asset_promotion.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/asset_editor/contract.rs
  - zircon_editor/src/ui/asset_editor/node_projection.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_host.rs
  - zircon_editor/src/ui/asset_editor/session/preview_compile.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/mod.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/contract.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/binding.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/localization.rs
  - zircon_editor/src/ui/asset_editor/binding/schema_projection.rs
  - zircon_editor/src/ui/template/mod.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/template/service.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/target.rs
  - zircon_runtime_interface/src/ui/tree/mod.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_editor/src/ui/asset_editor/presentation.rs
  - zircon_editor/src/ui/asset_editor/shell_layout.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_mock.rs
  - zircon_editor/src/ui/asset_editor/session/mod.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/command_entry.rs
  - zircon_editor/src/ui/asset_editor/session/palette_state.rs
  - zircon_editor/src/ui/asset_editor/session/binding_state.rs
  - zircon_editor/src/ui/asset_editor/session/navigation_state.rs
  - zircon_editor/src/ui/asset_editor/session/theme_state.rs
  - zircon_editor/src/ui/asset_editor/session/promotion_state.rs
  - zircon_editor/src/ui/asset_editor/session/root_class_policy_state.rs
  - zircon_editor/src/ui/asset_editor/session/style_state.rs
  - zircon_editor/src/ui/asset_editor/session/presentation_state.rs
  - zircon_editor/src/ui/asset_editor/session/preview_state.rs
  - zircon_editor/src/ui/asset_editor/session/designer_state.rs
  - zircon_editor/src/ui/asset_editor/session/emergency_state.rs
  - zircon_editor/src/ui/asset_editor/session/runtime_report_state.rs
  - zircon_editor/src/ui/asset_editor/session/resolver_state.rs
  - zircon_editor/src/ui/asset_editor/source/mod.rs
  - zircon_editor/assets/ui/editor/asset_browser.ui.toml
  - zircon_editor/assets/ui/editor/animation_editor.ui.toml
  - zircon_editor/assets/ui/editor/assets_activity.ui.toml
  - zircon_editor/assets/ui/editor/console.ui.toml
  - zircon_editor/assets/ui/editor/hierarchy.ui.toml
  - zircon_editor/assets/ui/editor/inspector.ui.toml
  - zircon_editor/assets/ui/editor/project_overview.ui.toml
  - zircon_editor/assets/ui/editor/welcome.ui.toml
  - zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml
  - zircon_editor/src/ui/layouts/views/mod.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/layouts/views/animation_editor_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/assets_activity.rs
  - zircon_editor/src/ui/layouts/views/console_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/hierarchy_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/inspector_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/project_overview.rs
  - zircon_editor/src/ui/layouts/views/welcome_shell_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_ui_asset_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset.rs
  - zircon_editor/src/ui/retained_host/ui/tests.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/registry.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_page_chrome.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs
  - zircon_editor/src/tests/ui/boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/asset_editor_structure.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
  - zircon_editor/src/tests/ui/assets_activity/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/asset_browser/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/animation_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/project_overview/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/welcome/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/boundary/view_projection_cutover.rs
  - zircon_editor/src/tests/host/retained_window/ui_asset_editor.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/mod.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/reflection.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/editing/ui_asset/designer_tools.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/editor_layouts.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/contract_diagnostics.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/action_localization_reports.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_reports.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs
  - zircon_editor/src/tests/editing/ui_asset/runtime_report_productization.rs
  - zircon_editor/src/tests/host/template_runtime/dual_host_parity.rs
  - zircon_editor/src/tests/editing/ui_asset/emergency_shell.rs
  - zircon_editor/tests/integration_contracts/workbench_retained_shell.rs
implementation_files:
  - zircon_editor/Cargo.toml
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_asset_workspace.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_session_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/inspector.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/navigation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/node_ops.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/open.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/save.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/workspace_state.rs
  - zircon_editor/src/ui/host/ui_asset_promotion.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/asset_editor/contract.rs
  - zircon_editor/src/ui/asset_editor/node_projection.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_host.rs
  - zircon_editor/src/ui/asset_editor/session/preview_compile.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/mod.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/contract.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/binding.rs
  - zircon_editor/src/ui/asset_editor/diagnostics/localization.rs
  - zircon_editor/src/ui/asset_editor/binding/schema_projection.rs
  - zircon_editor/src/ui/template/mod.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/template/service.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/target.rs
  - zircon_runtime_interface/src/ui/tree/mod.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_editor/src/ui/asset_editor/presentation.rs
  - zircon_editor/src/ui/asset_editor/shell_layout.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_mock.rs
  - zircon_editor/src/ui/asset_editor/session/mod.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/command_entry.rs
  - zircon_editor/src/ui/asset_editor/session/palette_state.rs
  - zircon_editor/src/ui/asset_editor/session/binding_state.rs
  - zircon_editor/src/ui/asset_editor/session/navigation_state.rs
  - zircon_editor/src/ui/asset_editor/session/theme_state.rs
  - zircon_editor/src/ui/asset_editor/session/promotion_state.rs
  - zircon_editor/src/ui/asset_editor/session/root_class_policy_state.rs
  - zircon_editor/src/ui/asset_editor/session/style_state.rs
  - zircon_editor/src/ui/asset_editor/session/presentation_state.rs
  - zircon_editor/src/ui/asset_editor/session/preview_state.rs
  - zircon_editor/src/ui/asset_editor/session/designer_state.rs
  - zircon_editor/src/ui/asset_editor/session/emergency_state.rs
  - zircon_editor/src/ui/asset_editor/session/runtime_report_state.rs
  - zircon_editor/src/ui/asset_editor/session/resolver_state.rs
  - zircon_editor/src/ui/asset_editor/source/mod.rs
  - zircon_editor/assets/ui/editor/asset_browser.ui.toml
  - zircon_editor/assets/ui/editor/animation_editor.ui.toml
  - zircon_editor/assets/ui/editor/assets_activity.ui.toml
  - zircon_editor/assets/ui/editor/console.ui.toml
  - zircon_editor/assets/ui/editor/hierarchy.ui.toml
  - zircon_editor/assets/ui/editor/inspector.ui.toml
  - zircon_editor/assets/ui/editor/project_overview.ui.toml
  - zircon_editor/assets/ui/editor/welcome.ui.toml
  - zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml
  - zircon_editor/src/ui/layouts/views/mod.rs
  - zircon_editor/src/ui/layouts/views/animation_editor_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/assets_activity_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/console_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/hierarchy_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/inspector_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/project_overview_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/welcome_shell_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_ui_asset_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/ui_asset.rs
  - zircon_editor/src/ui/retained_host/ui/tests.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/registry.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_page_chrome.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
plan_sources:
  - user: 2026-04-20 目前zircon_editor有两套ui相关代码 一套在core里面需要迁移回ui
  - user: 2026-04-20 要求加载入口不允许放入src
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-20 不要re-export 直接清理core里ui部分
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/收敛缺口修复 Spec 与 Implementation Plan.md
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
  - docs/superpowers/specs/2026-05-01-ui-asset-workspace-full-watcher-design.md
  - docs/superpowers/plans/2026-05-01-ui-asset-workspace-full-watcher.md
  - .codex/plans/UI 后续产品化与验证归档计划.md
  - docs/superpowers/plans/2026-05-01-ui-asset-resource-refs-m15.md
  - docs/superpowers/plans/2026-05-01-ui-productization-editor-binding-parity.md
  - docs/superpowers/specs/2026-05-02-ui-runtime-interface-big-cutover-design.md
  - docs/superpowers/plans/2026-05-02-ui-runtime-interface-big-cutover.md
  - user: 2026-05-02 continue package/cache classification and editor template-service façade
  - .codex/plans/Zircon UI 增量布局、增量重绘与控件池优化计划.md
tests:
  - zircon_editor/src/tests/editing/ui_asset/structure_split.rs
  - zircon_editor/src/tests/editing/ui_asset/source_projection.rs
  - zircon_editor/src/tests/editing/ui_asset/reference_and_promotion.rs
  - zircon_editor/src/tests/editing/ui_asset_theme_authoring.rs
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/editing/ui_asset/emergency_shell.rs
  - zircon_editor/src/tests/editing/ui_asset/preview.rs
  - cargo test -p zircon_editor --lib ui_asset_preview_host_resizes_retained_surface_without_rebuilding_tree_state --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-incremental-layout-render --message-format short --color never (2026-05-08 attempted; blocked before test execution by unrelated editor host/runtime compile drift)
  - cargo test -p zircon_editor --lib ui_asset_editor_session_switches_preview_presets_and_rebuilds_preview_surface --locked --jobs 1 --target-dir <milestone-target> --message-format short --color never (planned milestone validation; test name predates retained resize behavior)
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/host/manager/ui_asset_session_preview.rs
  - zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs
  - zircon_editor/src/tests/host/manager/ui_asset_style_and_inspector.rs
  - zircon_editor/src/tests/host/asset_manager_boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/asset_editor_structure.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - zircon_editor/src/tests/ui/assets_activity/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/asset_browser/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/animation_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/console/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/hierarchy/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/inspector/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/project_overview/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/welcome/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/boundary/view_projection_cutover.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/host/retained_detail_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_window/ui_asset_editor.rs
  - zircon_editor/src/tests/host/retained_detail_pointer/template_callbacks.rs
  - zircon_editor/src/tests/host/retained_list_pointer/pane_surface_actions.rs
  - zircon_editor/src/tests/host/retained_list_pointer/surface_contract.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/editor_layouts.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/contract_diagnostics.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/binding_semantics.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/action_localization_reports.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_reports.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs
  - zircon_editor/src/tests/editing/ui_asset/runtime_report_productization.rs
  - zircon_editor/src/tests/host/template_runtime/dual_host_parity.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
  - zircon_editor/tests/integration_contracts/workbench_retained_shell.rs
  - cargo test -p zircon_editor --lib editor_asset_boundary_lives_in_editor_crate --locked
  - cargo test -p zircon_editor --lib editor_manager_becomes_thin_facade_over_editor_ui_host --locked
  - cargo test -p zircon_editor --lib editor_module_owner_moves_under_ui_host --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_moves_into_a_folder_backed_ui_subsystem --locked
  - cargo test -p zircon_editor --locked ui_asset_editor_subsystem_is_grouped_by_domain_folders --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked ui_asset_editor_lifecycle_owns_document_validation_and_apply_state --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked ui_asset_editor_theme_state_owns_theme_replay_helpers --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked ui_asset_editor_style_state_owns_style_replay_helpers --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked ui_asset_editor_promotion_state_owns_promotion_helpers --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked ui_asset_editor_command_entry_owns_document_replay_helpers --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked ui_asset_editor_presentation_state_owns_view_projection --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --lib editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_bootstrap_assets_parse_and_compile_with_imports --locked
  - cargo test -p zircon_editor --locked ui_asset_editor_bootstrap_shell_layout_exposes_pane_shell_regions --lib --target-dir F:/cargo-targets/zircon-codex-a
  - cargo test -p zircon_editor --locked ui_asset_editor_action_bar_consumes_shell_layout_for_internal_rows --lib --target-dir F:/cargo-targets/zircon-codex-a
  - cargo test -p zircon_editor --locked tests::ui::assets_activity:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked asset_browser_bootstrap_layout_self_hosts_shell_sections --lib --target-dir F:/cargo-targets/zircon-codex-a
  - cargo test -p zircon_editor --locked tests::ui::animation_editor:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::console:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::hierarchy:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::inspector:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::project_overview:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::welcome:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked host_scene_projection_converts_host_owned_panes_to_host_contract --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture (historical name has since moved to retained host-contract wording)
  - cargo test -p zircon_editor --locked tests::ui::boundary::template_assets:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::boundary::view_projection_cutover:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::boundary::workbench_projection_cutover:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::host::retained_detail_pointer:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::host::retained_list_pointer:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-a
  - F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe ui::retained_host::ui::tests::apply_presentation_projects_welcome_shell_layout_into_global_context --exact --nocapture
  - cargo test -p zircon_editor --locked ui_asset_editor_stylesheet_panel_consumes_shell_layout_for_header_rows --lib --target-dir F:/cargo-targets/zircon-codex-a
  - cargo test -p zircon_editor --lib tests::ui::ui_asset_editor --locked --offline --message-format short
  - cargo test -p zircon_editor --lib ui_asset_editor_bootstrap --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib workbench_retained_entry_stays_on_generic_host_bootstrap_files --locked
  - cargo test -p zircon_editor --locked --offline --test integration_contracts workbench_retained_shell
  - cargo test -p zircon_editor --lib contract_diagnostics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never (3 passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never (passed with unrelated graphics warnings)
  - cargo test -p zircon_editor --lib ui_asset_editor_host_genericizes_detail_event_dispatch --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never
  - cargo test -p zircon_editor --lib asset_browser_projection_maps_bootstrap_asset_into_mount_nodes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never
  - cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never
  - cargo test -p zircon_editor --lib ui_asset_editor_session_projects_and_updates_root_class_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never (passed after one cold compile timeout)
  - cargo test -p zircon_editor --lib asset_editor_component_adapter_updates_selected_component_root_class_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never (passed)
  - cargo test -p zircon_editor --lib editor_component_adapter_registry_advertises_reflection_and_asset_editor_sources --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never (passed)
  - cargo test -p zircon_editor --lib ui_asset_editor_host_genericizes_detail_event_dispatch --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never (passed)
  - cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never (204 passed; 0 failed; 675 filtered out)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never (blocked by unrelated runtime-interface manifest/lock drift)
  - cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never (blocked by unrelated runtime-interface manifest/lock drift)
  - cargo test -p zircon_editor --lib ui_asset_editor_projects_runtime_binding_diagnostic_and_schema_items --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (passed: 1 passed; 0 failed; 888 filtered out)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-big-cutover --message-format short --color never (blocked before compilation by unrelated workspace lock drift: `web-sys` selected `js-sys` 0.3.97 while `Cargo.lock` selected 0.3.95)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-editor-check --message-format short --color never (2026-05-02 editor tree DTO/runtime extension split: passed with existing runtime graphics warnings and 3 editor warnings)
doc_type: module-detail
---

# UI Asset Editor Host Session

## Purpose

这份文档记录 `zircon_editor` 在本轮 cutover 后的当前真相：

- editor UI 宿主实现统一收口到 `zircon_editor/src/ui`
- `core` 不再拥有 workbench/layout/window/session 的真实实现
- `UI Asset Editor` 相关代码不再分裂在 `core/editing/ui_asset` 和 `core/host/manager/ui_asset_sessions`

本篇重点说明 ownership、会话边界和 Rust-owned retained host 约束，而不是重复 shared `.ui.toml` 资产格式本身。资产格式见 [`UI Asset Documents And Editor Protocol`](../ui-and-layout/ui-asset-documents-and-editor-protocol.md)。

## Ownership After Cutover

### `core` 还保留什么

`zircon_editor::core` 现在只保留 editor 内核而不是 UI 实现本体：

- 编辑状态机、intent、history 和 editor-event runtime
- 非 UI 的 editor 资产状态与 command/runtime contract

本轮明确删除了旧的 [`zircon_editor/src/core/host/manager.rs`](../../zircon_editor/src/core/host/manager.rs) owner 角色，而且 `core::host` 整个子树都已经退场。`core` 里不再存在兼容性的 host façade 或模块 owner。

### `ui::host` 现在拥有什么

`zircon_editor/src/ui/host/` 现在是 editor UI 宿主编排的唯一 owner，覆盖：

- `EditorUiHost` 作为统一宿主 owner，真实持有 `CoreHandle`、`ViewRegistry`、`LayoutManager`、`WindowHostManager`、`EditorSessionState` 和 UI asset session 账本
- `EditorManager` 退化为薄 façade，只暴露 editor-facing API 并把状态访问委托给 `EditorUiHost`
- `module.rs` 作为 `EditorModule`、service-name 常量和 `module_descriptor()` 的唯一 owner
- `editor_asset_manager/` 与 `resource_access.rs` 也已经并入 `ui::host`，负责 asset workspace catalog/details/reference/preview sidecar 与宿主资源句柄解析
- builtin view 和 builtin layout 注册
- layout command / layout host / layout persistence
- startup、welcome、recent project、workspace session bookkeeping
- native floating window host 账本
- UI asset session orchestration 与 promotion/workspace sync

这意味着 view registry、layout manager、window host manager、startup/welcome/workspace 持久化都已经从旧 `core::host::manager` 目录下迁回 `ui::host`，而 `EditorManager` 本身不再继续成为这些对象的直接 owner。

### `ui::asset_editor` 现在拥有什么

`zircon_editor/src/ui/asset_editor/` 现在承接 UI Asset Editor 自身的领域实现，覆盖：

- route / reflection / window descriptor contract
- source buffer 与 canonical save
- session state、preview compile、presentation
- binding/style/tree edit authoring
- undo、document replay、external effect replay
- promotion draft 与 preview host
- session 子树下的 folder-backed authoring 实现：`ui_asset_editor_session.rs` 只保留主类型、共享 replay/helper 与少量 tree façade；`lifecycle.rs`/`command_entry.rs`/`palette_state.rs`/`binding_state.rs`/`preview_state.rs`/`style_state.rs`/`theme_state.rs`/`promotion_state.rs`/`presentation_state.rs` 分别承接生命周期、命令回放、palette drag/insert、binding authoring、preview mock authoring、style/semantic/rule/token authoring、theme authoring、reference/promotion draft、view-projection 组装

原先分散在 `core/editing/ui_asset/*` 的逻辑现在按 `binding/preview/session/source/style/tree` 子树收进同一 UI 域，避免“编辑器 UI 领域逻辑在 core 里继续长大”。

## No Core Re-export Shim

本轮刻意没有采用“`core -> ui` 兼容 re-export”做过渡。

当前链路是直接改 owner：

- [`zircon_editor/src/ui/host/module.rs`](../../zircon_editor/src/ui/host/module.rs) 直接实例化 `crate::ui::host::EditorManager` 并持有 `EditorModule` wiring
- [`zircon_editor/src/lib.rs`](../../zircon_editor/src/lib.rs) 的公开 editor host 类型直接从 `ui::host::module` 导出
- [`zircon_editor/src/core/mod.rs`](../../zircon_editor/src/core/mod.rs) 已不再声明 `host` 子树，`core::host` 目录也已删除

这样做的效果是，后续再清理 `core` 时不会被一层历史兼容命名绑住，也不会让调用方误以为 `core` 仍然是 UI owner。

## Session And Host Split

当前 UI asset editor 的职责边界固定成两层：

- `ui::host::asset_editor_sessions`
  - 负责打开/保存、asset hydration、project/workspace 同步、host-level orchestration
  - 负责把 retained host callback 或 workbench action 路由成稳定 session 调用
- `ui::asset_editor::UiAssetEditorSession`
  - 负责 source/document/preview 三角同步
  - 负责 selection、tree edit、binding/style authoring、undo/replay
  - 负责 canonical TOML 输出和 last-good preview 语义

也就是说，host 层只保留“会话编排”和“工作台整合”；真正的 UI asset authoring 行为已经回到 `ui::asset_editor` 域内，而不是继续夹在 `core` 和 `ui` 之间。

### Workspace Watcher And Conflict State

M5 的 workspace owner 现在也收口在 host 层，而不是塞回 `UiAssetEditorSession`：

- `EditorUiHost` 在 project open/save 后重启 `UiAssetWorkspaceWatcher`，watch `project_root/assets` 下的 `.ui.toml` 文件，并把路径规整成 `res://...` asset id
- `refresh_ui_asset_workspace_for_changes(...)` 是 deterministic refresh 入口；真实 watcher poll、保存后的 dependent refresh、promotion undo/redo 外部 effect 都走同一条入口
- `UiAssetWorkspaceEntry` 持有 disk baseline、external conflict、diff snapshot 和 stale import diagnostics；`UiAssetEditorSession` 仍只负责 source/document/preview 与 authoring state
- clean direct asset change 会从磁盘重建 session、更新 baseline、清理 conflict，并重新 hydrate imports
- dirty direct asset change 不覆盖本地 source，而是记录 `UiAssetExternalConflict`；pane/reflection 暴露 `has_external_conflict`、reload/keep-local/save-local-copy/diff snapshot affordance
- save-local-copy 只把当前本地 canonical source 写入显式路径或原文件旁的 `*.local-copy.toml`，不更新 disk baseline、不清理 dirty/conflict，也不覆盖外部变更
- removed direct asset change 也进入 conflict state，而不是让 promotion undo 或外部删除因为 `NotFound` 中断 session 流程
- import asset change 会尝试重新收集 imported widget/style documents；解析失败、kind mismatch 或 missing file 会变成 stale import diagnostic，并保留 last-good preview/session，直到 import 恢复后清空 stale state

对外 editor manager 只新增薄 façade 方法：`reload_ui_asset_editor_from_disk`、`keep_ui_asset_editor_local_and_save`、`revert_ui_asset_editor_to_last_valid`、`save_ui_asset_editor_local_copy`、`save_ui_asset_editor_local_copy_next_to_source`、`refresh_ui_asset_workspace_for_changes`、`poll_ui_asset_workspace_watcher`，以及 crate 内诊断用的 diff snapshot accessor。UI shell 可以通过 pane presentation 决定是否展示 reload/keep-local/revert/save-local-copy/diff/stale import 操作，但真正的冲突裁决和 emergency recovery 仍由 host workspace pipeline 执行。

### Emergency Shell And Last-Valid Recovery

M24 的 recovery owner 也收口在同一条 session/host 边界上，而不是新增 Slint 真源或独立 watcher policy：

- `UiAssetEditorShellState::{Valid, Stale, Invalid, Emergency}` 是 editor contract 的状态出口；`Stale` 来自外部冲突，`Emergency` 表示当前 source 有诊断但仍保留 last-valid preview host。
- `UiAssetEditorSession` 持有 `last_valid_source_text`、`last_valid_document`、`last_valid_compiled` 和 `preview_host`；当前 source 解析失败时只更新 diagnostics/source buffer，不覆盖 last-valid document/compiled/preview。
- `emergency_state.rs` 负责 shell-state 计算、diagnostic summary、revert affordance 和 `revert_source_to_last_valid()`；revert 仍走 `UiAssetEditorCommand::edit_source(...)`，所以 undo/replay 语义保持一致。
- `UiAssetEditorReflectionModel`、pane presentation、retained/native host-contract DTO 和 `pane_ui_asset_conversion.rs` 投影 `shell_state`、`emergency_summary`、`can_emergency_reload`、`can_emergency_revert`、`can_emergency_open_asset_browser`。
- `dispatch_ui_asset_action(...)` 把 `emergency.reload_from_disk`、`emergency.revert_last_valid`、`emergency.open_asset_browser` 分别路由到现有 `EditorUiHost` reload/revert/open-view 方法；reload/keep-local/diff 继续复用 M5 workspace conflict pipeline。

M24 focused validation on 2026-05-07:

- `cargo test -p zircon_editor --lib emergency_shell --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never -- --nocapture --test-threads=1` passed after the first cold compile timed out and the warmed rerun completed, `4 passed; 0 failed; 1128 filtered out`.
- `cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never -- --nocapture --test-threads=1` passed, `6 passed; 0 failed; 1126 filtered out`.
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never -- --nocapture --test-threads=1` passed, `211 passed; 0 failed; 921 filtered out`.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never` passed with existing runtime/editor warnings.

### Session Folder Split

当前 `zircon_editor/src/ui/asset_editor/session/` 已经开始按作者态职责继续下沉，而不是让 `ui_asset_editor_session.rs` 持续吞下所有 session 逻辑：

- `ui_asset_editor_session.rs`
  - 保留 `UiAssetEditorSession` / `UiAssetEditorSessionError` / `UiAssetEditorReplayResult`、少量跨文件共享 cursor/serialization/label helper 与少数 tree façade；不再直接承接 style/theme/promotion，也不再直接承接 source revalidate / valid-document apply、promotion normalize/reference helper、tree/binding replay bundle、reflection/pane projection 这批具体子流程；当前已收窄到 227 行，作为这一轮 session split 的 stop line
- `lifecycle.rs`
  - 负责 `from_source`、source/import 生命周期、source revalidate、valid-document apply/reconcile、preview refresh/rebuild、canonical save、外部 widget/style source 恢复
- `command_entry.rs`
  - 负责 `apply_command`、undo/redo、document replay 记录与 undo transition 回放，以及 tree/binding document replay bundle 与 widget/component/node replay command builder
- `palette_state.rs`
  - 负责 palette 选中、drag target chooser、preview drop target、节点插入、move/reparent
- `binding_state.rs`
  - 负责 binding 选中、event/route/action/payload 编辑与 suggestion 应用
- `navigation_state.rs`
  - 负责 hierarchy/preview/source-outline/source-line/source-byte-offset 选中，以及 source cursor snapshot/restore/reconcile 与选中节点联动
- `preview_state.rs`
  - 负责 preview mock subject/property/nested entry 选中、值修改、suggestion 应用，以及 preview rebuild 触发
- `style_state.rs`
  - 负责 class、control/text、slot/layout semantic、style token、stylesheet rule/declaration、pseudo-state preview 等 style authoring mutation，以及对应的 replay-aware document edit 与 stylesheet edit replay bundle
- `theme_state.rs`
  - 负责 theme source 选中、imported/local theme detach/clone/adopt helper、theme refactor 应用、theme promotion 使用的 theme replay bundle，以及 style import/token/stylesheet/rule diff replay helper
- `promotion_state.rs`
  - 负责 reference 转换、component extract、external widget/style promotion draft、promotion 输入 normalize、reference asset 解析、外部资产 source upsert/restore 与 promotion replay 协调
- `presentation_state.rs`
  - 负责 `reflection_model` / `pane_presentation`、preview summary，以及 source/preview/inspector/style/theme/palette/binding 的 view-projection 组装；session 主文件不再继续吞下整块 UI projection owner

这条拆分线的目标不是把 `session/` 一次性拆到极限，而是先把最明显的状态机簇从 4k+ 主文件里切走，让后续剩余的 style/theme/source/promotion 再继续按责任下沉时，不需要重新挪动已经稳定的 palette/binding/command 生命周期边界。

### Structured Contract Diagnostics

`ui::asset_editor` now has a dedicated `diagnostics/` child module for editor-facing diagnostic DTOs. The current production path maps runtime-owned `UiComponentContractDiagnostic` and `UiBindingDiagnostic` values into `UiAssetEditorDiagnostic` without making the editor parse compiler error display strings or invent separate contract/binding semantics.

The session keeps two diagnostic surfaces intentionally:

- `diagnostics: Vec<String>` remains the legacy status/error surface used by existing presentation code and editability checks.
- `structured_diagnostics: Vec<UiAssetEditorDiagnostic>` carries stable code, severity, source path, target node id, target control id, and target binding id for contract-aware and binding-aware panels.

`lifecycle.rs` refreshes structured diagnostics whenever preview compilation fails or imports change. It combines component-contract diagnostics from `zircon_runtime::ui::template::asset::component_contract` with interface-owned binding diagnostics returned by runtime binding validation. `presentation_state.rs` projects them into `structured_diagnostic_items` and uses a diagnostic `target_node_id` to select the matching source-outline row when there is no current user selection. This keeps runtime component-contract and binding behavior authority in `zircon_runtime` while neutral binding diagnostic DTOs come from `zircon_runtime_interface::ui::template`, allowing the editor to deep-link private selector, API mismatch, closed-root-class, invalid binding target, invalid value kind, unresolved reference, and unsupported operator failures.

Binding inspector schema projection is also runtime-report driven. `binding/schema_projection.rs` imports `UiBindingDiagnostic`, `UiBindingTarget`, and `UiBindingTargetKind` from `zircon_runtime_interface::ui::template`, lists authored target assignments as `target[index] [kind.name] = expression`, then appends matching runtime diagnostic rows for that target index. The projection intentionally uses `collect_asset_binding_report(...)` rather than reimplementing value-kind or descriptor-prop rules in editor code.

Template-node projection keeps render extraction as runtime behavior. `asset_editor/node_projection.rs` and `layouts/views/view_projection.rs` now call runtime extraction when they need command/style/text rows for host projection; they do not call an interface-owned `UiRenderExtract::from_tree(...)` constructor. This preserves `zircon_runtime_interface::ui::surface::UiRenderExtract` as a data-only DTO while letting editor host projection reuse the canonical runtime extraction path. The UI Asset Editor shell projection now also keeps a process-local retained `UiSurface` session for `ui_asset_editor_node_projection(size)`: the first call loads and compiles the editor `.ui.toml` documents, while later size-only calls mark the retained surface roots layout dirty and use `UiSurface::rebuild_dirty(size)` instead of reloading the asset files.

Preview preset resize follows the same retained-surface rule. `UiAssetPreviewHost::rebuild(...)` is still the source/import/hot-reload path and rebuilds from the compiled document, but `rebuild_with_size(...)` updates `preview_size`, marks retained roots layout/hit/render dirty, and runs runtime dirty rebuild on the existing surface. This preserves focus/input/session state across preview-preset size changes while still allowing source edits, mock preview changes, import refresh, and last-valid recovery to replace the preview host through `compile_preview(...)` when the compiled document actually changes.

The first retained-preview validation attempt on 2026-05-08 used `cargo test -p zircon_editor --lib ui_asset_preview_host_resizes_retained_surface_without_rebuilding_tree_state --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-incremental-layout-render" --message-format short --color never`. The command did not reach the filtered editor regression because the shared checkout failed earlier in unrelated code: first the retained host presentation initializer was missing the new `text_input_focus` field, then later scoped editor checks were blocked by unrelated runtime asset-facade generic-bound errors and native plugin ABI callback drift. The runtime half of this retained slice was validated separately through `surface_dirty_domains` and `zircon_runtime_interface` checks documented in `docs/ui-and-layout/slate-style-ui-surface-frame.md`.

### Component Root-Class Authoring

Root-class policy authoring now lives in `session/root_class_policy_state.rs` instead of being folded into generic style or promotion state. The session resolves the selected local component from the current component node, projects the runtime-owned `UiRootClassPolicy` as `append_only` or `closed`, validates accepted editor literals, and applies changes through the same replay-aware document edit path as other authoring mutations. `pane_presentation()` exposes both `inspector_component_root_class_policy` and `inspector_can_edit_component_root_class_policy`, and the inspector summary includes a `root class policy: ...` item so the product surface is visible without inventing editor-owned contract semantics.

The host and adapter path stays on the shared Runtime UI envelope route. The retained detail action `component.root_class_policy.set` dispatches a `Commit` envelope to the `asset_editor` adapter, the adapter accepts `component.root_class_policy`, and `EditorManager` forwards the mutation to `EditorUiHost::set_ui_asset_editor_selected_component_root_class_policy(...)`. This keeps closed-root-class validation in runtime component contracts while giving the UI Asset Editor a replayable authoring control for the contract value.

Focused evidence from 2026-05-02 used `E:\cargo-targets\zircon-ui-m10-root-class-authoring`: `ui_asset_editor_session_projects_and_updates_root_class_policy`, `asset_editor_component_adapter_updates_selected_component_root_class_policy`, `editor_component_adapter_registry_advertises_reflection_and_asset_editor_sources`, and `ui_asset_editor_host_genericizes_detail_event_dispatch` passed as focused gates; `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1` also passed with `204 passed; 0 failed; 675 filtered out`. Broader `cargo check -p zircon_editor --lib --locked` and `cargo test -p zircon_runtime --lib asset_component_contract --locked` did not reach compilation because the shared checkout had unrelated runtime-interface manifest/lock drift and Cargo refused to update `Cargo.lock` under `--locked`.

### UI Asset Detail Component Adapter Dispatch

UI Asset Editor detail edits now share the Runtime UI component-adapter envelope instead of dispatching field-specific manager mutations directly from retained host callbacks. `zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs` builds `UiComponentEventEnvelope` commits for widget, slot, layout, and semantic field edits, then calls `dispatch_ui_asset_component_adapter_commit(...)`; the `asset_editor` adapter in `zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs` maps supported target paths onto the existing `EditorManager` mutation APIs. Delete actions remain direct manager commands because they are structural operations rather than value commits.

The guard `ui_asset_editor_host_genericizes_detail_event_dispatch` now checks the real source route and the adapter-owned field paths: `widget.control_id`, `widget.text`, `component.root_class_policy`, `slot.mount`, `slot.width_preferred`, `slot.height_preferred`, `layout.width_preferred`, `layout.height_preferred`, `slot.semantic.value`, and `layout.semantic.value`. This keeps detail editing on the same typed adapter seam as Inspector and future reflection bindings while preserving the existing UI Asset Editor session owner boundary.

## Current Retained Host Boundary

The active editor UI boundary is now Rust-owned. `zircon_editor/src/ui/retained_host` owns the host loop, retained input/callback dispatch, host-contract DTOs, pane projection, and native presentation glue. Deleted generated UI sources are no longer an implementation, test, or documentation authority.

The current host-contract entry points are:

- [`retained_host/app.rs`](../../zircon_editor/src/ui/retained_host/app.rs) and [`retained_host/app/ui_asset_editor.rs`](../../zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs) for retained host lifecycle and UI Asset Editor event dispatch
- [`retained_host/host_contract/mod.rs`](../../zircon_editor/src/ui/retained_host/host_contract/mod.rs), [`host_contract/window.rs`](../../zircon_editor/src/ui/retained_host/host_contract/window.rs), and [`host_contract/globals.rs`](../../zircon_editor/src/ui/retained_host/host_contract/globals.rs) for the Rust-owned window/context/presentation seam
- [`retained_host/ui/apply_presentation.rs`](../../zircon_editor/src/ui/retained_host/ui/apply_presentation.rs) and [`retained_host/ui/pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs) for presentation-to-host-contract projection
- [`template_runtime/retained_adapter.rs`](../../zircon_editor/src/ui/template_runtime/retained_adapter.rs) for the retained template-runtime adapter

The retained boundary deliberately keeps `workbench_host_window` on editor-owned Rust data. Pane selection and scene projection stay in [`host_data.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs), [`pane_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs), and [`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs); those modules pass `UiAssetEditorPanePresentation`, `AnimationEditorPaneViewData`, neutral template nodes, and host shell-layout data into retained host contracts rather than rebuilding deleted generated DTOs.

Editor panes get their structural frames from `.ui.toml` assets and runtime/shared UI projection, not from generated source files. `ui_asset_editor.ui.toml`, the pane assets under `zircon_editor/assets/ui/editor`, and host assets under `zircon_editor/assets/ui/editor/host` are the current structure authorities; retained host projection consumes their shared-surface frames.

The focused retained evidence for this boundary lives in `workbench_projection_cutover.rs`, `template_assets.rs`, the pane-specific `bootstrap_assets.rs` tests, `retained_host/ui/tests.rs`, `retained_detail_pointer`, `retained_list_pointer`, and the `workbench_retained*` integration-contract readers.

## Historical Slint Entry Boundary

The following section is retained only as historical cutover context for reviewers reading older validation logs. It does not define current owner paths; current implementation and tests must use `retained_host`, `.ui.toml`, and Rust-owned `host_contract` paths above.

`.slint` cutover 在 editor 侧先冻结了入口边界：

- [`workbench.slint`](../../zircon_editor/ui/workbench.slint)
- [`host_scaffold.slint`](../../zircon_editor/ui/workbench/host_scaffold.slint)
- [`host_surface.slint`](../../zircon_editor/ui/workbench/host_surface.slint)
- [`host_surface_contract.slint`](../../zircon_editor/ui/workbench/host_surface_contract.slint)
- [`host_root.slint`](../../zircon_editor/ui/workbench/host_root.slint)

这些 bootstrap 文件现在只能保留 generic host window / scaffold / surface 职责。边界测试明确禁止它们重新 import `assets.slint`、`panes.slint`、`welcome.slint` 这类业务壳文件。

当前这层 generic bootstrap 还有一个固定合同：

- `host_root.slint` 里的 `HostWindowPresentationData` 统一分组 `host_shell`、`host_layout`、`workbench_scene_data`、`native_floating_surface_data`
- `host_scaffold.slint` 只接收整份 `host_presentation`，不再把 surface/layout/native payload 扇出成一组松散属性
- `host_surface_contract.slint` 只负责从 `host_presentation` 投影出 `workbench_scene_data` 和 `native_floating_surface_data`
- `host_surface.slint` 只消费 contract 输出，再把 scene/native floating surface 分流到真正的 host scene surface

这一步的目标不是一次性删除所有业务 `.slint` 文件，而是先钉死“入口层不能再成为业务真源”。更深层的 pane catalog 和残余业务 Slint 仍是后续 slice 的清理对象。

## Latest Pane DTO Boundary

The current boundary is Rust-owned end to end. `workbench_host_window` selects editor-owned pane presentations, `retained_host/ui/apply_presentation.rs` builds `HostWindowPresentationData`, and `retained_host/ui/pane_data_conversion/mod.rs` converts pane data into `retained_host/host_contract/**` DTOs. The deleted generated pane DTOs are not an internal work model and must not be restored as compatibility ABI.

The practical split is:

- [`host_data.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs), [`pane_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs), and [`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) keep `PaneData` on editor-owned Rust presentations such as `UiAssetEditorPanePresentation`, `AnimationEditorPaneViewData`, `ViewTemplateNodeData`, and host shell-layout data.
- [`apply_presentation.rs`](../../zircon_editor/src/ui/retained_host/ui/apply_presentation.rs) owns the final presentation apply step into `HostWindowPresentationData`.
- [`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs) owns conversion from workbench pane data to retained host-contract panes.
- [`host_contract/data/panes.rs`](../../zircon_editor/src/ui/retained_host/host_contract/data/panes.rs), [`host_contract/data/ui_asset.rs`](../../zircon_editor/src/ui/retained_host/host_contract/data/ui_asset.rs), [`host_contract/window.rs`](../../zircon_editor/src/ui/retained_host/host_contract/window.rs), and [`host_contract/globals.rs`](../../zircon_editor/src/ui/retained_host/host_contract/globals.rs) are the host-facing data seam.
- [`retained_host/ui/tests.rs`](../../zircon_editor/src/ui/retained_host/ui/tests.rs) validates that host-owned panes convert to host-contract panes, including non-default shell-layout samples.

This keeps `workbench_host_window` from importing retained host DTO declarations directly and prevents the retained host from becoming the owner of editor business sessions. The only data that crosses into the retained host is already shaped as presentation or host-contract data.

## Bootstrap Shell Layout Authority

All pane shell topology now comes from `.ui.toml` assets and shared runtime UI projection. The retained host can render or route the resulting frames, but it does not invent page-level panel geometry.

- [`ui_asset_editor.ui.toml`](../../zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml) describes the `UiAssetEditor` shell: header rows, left/center/right columns, palette, hierarchy, designer, action bar, source, inspector, stylesheet, emergency, and render-stack regions. [`shell_layout.rs`](../../zircon_editor/src/ui/asset_editor/shell_layout.rs) extracts those regions into `UiAssetEditorShellLayout`; scene projection injects the layout back into `UiAssetEditorPanePresentation`; retained pane conversion publishes it through host-contract `UiAssetEditorShellLayoutData`.
- [`asset_browser.ui.toml`](../../zircon_editor/assets/ui/editor/asset_browser.ui.toml) owns the Asset Browser toolbar/import/main/utility/reference shell. [`asset_browser_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/asset_browser_shell_layout.rs) extracts `AssetBrowserShellLayout`; host projection and retained conversion keep it as pane data instead of local coordinate formulas.
- [`animation_editor.ui.toml`](../../zircon_editor/assets/ui/editor/animation_editor.ui.toml) owns animation header/body and sequence/graph/state-machine mode regions. [`animation_editor_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/animation_editor_shell_layout.rs) extracts `AnimationEditorShellLayout`; `retained_host/ui/pane_data_conversion/mod.rs` carries it into `AnimationEditorPaneData`.
- [`assets_activity.ui.toml`](../../zircon_editor/assets/ui/editor/assets_activity.ui.toml) and [`assets_activity.rs`](../../zircon_editor/src/ui/layouts/views/assets_activity.rs) use neutral `ViewTemplateNodeData` mount nodes rather than a bespoke shell-layout DTO. Scene projection injects current frames during host projection.
- [`project_overview.ui.toml`](../../zircon_editor/assets/ui/editor/project_overview.ui.toml) and [`project_overview.rs`](../../zircon_editor/src/ui/layouts/views/project_overview.rs) route Project Overview through generic template nodes with snapshot text overrides.
- [`welcome.ui.toml`](../../zircon_editor/assets/ui/editor/welcome.ui.toml), [`welcome_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/welcome_shell_layout.rs), and [`welcome_presentation.rs`](../../zircon_editor/src/ui/layouts/views/welcome_presentation.rs) keep Welcome page shell bands in assets and pass visible-surface sizing through retained presentation apply.
- [`hierarchy.ui.toml`](../../zircon_editor/assets/ui/editor/hierarchy.ui.toml), [`inspector.ui.toml`](../../zircon_editor/assets/ui/editor/inspector.ui.toml), and [`console.ui.toml`](../../zircon_editor/assets/ui/editor/console.ui.toml) own the remaining tool-pane shell frames, extracted by the corresponding `*_shell_layout.rs` modules and carried as Rust-owned pane data.

The current source guards lock this authority split instead of preserving old generated names:

- [`workbench_projection_cutover.rs`](../../zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs) rejects workbench-host imports of retained/generated pane DTO families and checks direct owner paths.
- [`template_assets.rs`](../../zircon_editor/src/tests/ui/boundary/template_assets.rs) rejects fallback to deleted hardcoded pane geometry by checking the `.ui.toml` shell regions that each pane must consume.
- Pane bootstrap tests under `src/tests/ui/*/bootstrap_assets.rs` assert each authored asset exports the required shell regions or mount nodes.
- [`retained_host/ui/tests.rs`](../../zircon_editor/src/ui/retained_host/ui/tests.rs) checks host-scene projection into host-contract panes.
- Retained pointer tests under `src/tests/host/retained_detail_pointer` and `src/tests/host/retained_list_pointer` keep scroll/list/detail surfaces on shared pointer/template callback contracts rather than restoring pane-specific callback ABI.

The accepted focused gates for this boundary are the pane bootstrap tests, `tests::ui::boundary::template_assets`, `tests::ui::boundary::workbench_projection_cutover`, retained detail/list pointer tests, `host_scene_projection_converts_host_owned_panes_to_host_contract_panes`, `workbench_host_window_keeps_retained_shell_dtos_at_ui_boundary_only`, and `cargo check -p zircon_editor --lib --locked` when the surrounding workspace is not blocked by unrelated active lanes.

## Tree-Native Session Helpers

当前 `UI Asset Editor` 的生产代码已经不再把 `document.nodes` 当成作者态真源。

这一轮实际落地的是：

- `UiAssetEditorSession`、tree edit、undo/replay、style inspection、source sync、promotion 和 preview projection 都直接走 `UiAssetDocument` 的递归 helper
- 典型访问路径已经统一成 `contains_node`、`node`、`node_mut`、`iter_nodes`、`parent_of`、`child_index_in_parent`、`replace_node`、`remove_node`、`insert_child`、`push_child`、`swap_children`
- component root 也不再通过旧的根节点字符串索引消费，而是直接把内嵌树根当成正式节点数据处理
- preview mock subject 的默认回退现在按 UI 实际展示顺序选首项，不再因为树遍历顺序和 subject 列表排序不同而出现“初始选中项错位”

剩余 legacy 兼容只留在 runtime 模板层的 `#[cfg(test)]` 迁移 helper，以及 editor 自己的 `src/tests/support.rs` 夹具迁移 helper；production editor authoring path 只接受 tree authority，它已经不再是 editor authoring session 的内部工作模型。

## Acceptance Evidence

本轮与 ownership 收口直接对应的验证有几条：

- `cargo test -p zircon_editor --lib ui_asset_editor_moves_into_a_folder_backed_ui_subsystem --locked`
  - 证明 UI asset editor 已经物理迁入 `src/ui/asset_editor`
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe tests::editing::ui_asset::structure_split::ui_asset_editor_subsystem_is_grouped_by_domain_folders --exact --nocapture`
  - 证明 `session/` 继续保持 folder-backed 结构，`lifecycle.rs`、`command_entry.rs`、`palette_state.rs`、`binding_state.rs`、`navigation_state.rs`、`preview_state.rs`、`style_state.rs`、`theme_state.rs`、`promotion_state.rs` 不会再退化回单文件堆积
- `cargo test -p zircon_editor --locked ui_asset_editor_lifecycle_owns_document_validation_and_apply_state --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 证明 `revalidate` / `apply_valid_document` 已经迁入 `lifecycle.rs`，`ui_asset_editor_session.rs` 不再继续吞下 document validation/apply-state 这批生命周期逻辑
- `cargo test -p zircon_editor --locked ui_asset_editor_theme_state_owns_theme_replay_helpers --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 证明 `theme_document_replay_bundle` 已经迁入 `theme_state.rs`，`ui_asset_editor_session.rs` 不再继续吞下 theme import/token/stylesheet/rule replay helper
- `cargo test -p zircon_editor --locked ui_asset_editor_style_state_owns_style_replay_helpers --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 证明 `editable_stylesheet` / `style_rule_{insert,remove,move}_replay_bundle` 已经迁入 `style_state.rs`，`ui_asset_editor_session.rs` 不再继续吞下 style replay helper
- `cargo test -p zircon_editor --locked ui_asset_editor_promotion_state_owns_promotion_helpers --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 证明 promotion normalize helper、`reference_asset_id` 与 external asset restore/remove helper 已经迁入 `promotion_state.rs`，`ui_asset_editor_session.rs` 不再继续吞下 promotion helper 簇
- `cargo test -p zircon_editor --locked ui_asset_editor_command_entry_owns_document_replay_helpers --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 证明 `tree_document_replay_bundle` / `binding_document_replay_bundle` 已经迁入 `command_entry.rs`，`ui_asset_editor_session.rs` 不再继续吞下 command replay helper
- `cargo test -p zircon_editor --locked ui_asset_editor_presentation_state_owns_view_projection --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 证明 `reflection_model` / `pane_presentation` 已经迁入 `presentation_state.rs`，`ui_asset_editor_session.rs` 不再继续吞下整块 view-projection owner
- `cargo test -p zircon_editor --locked tests::editing::ui_asset::structure_split:: --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 追加证明 `lifecycle.rs` / `theme_state.rs` / `style_state.rs` / `promotion_state.rs` / `command_entry.rs` / `presentation_state.rs` 这批 owner 边界现在已经能在同一轮 lib-test 编译里一起转绿
- `cargo test -p zircon_editor --locked tests::editing::ui_asset::preview:: --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 追加验证 11 个 preview 行为回归，确认 `presentation_state.rs` / `preview_state.rs` 收口后 preview compile、projection、mock authoring 与 last-good preview 语义仍然稳定
- `cargo test -p zircon_editor --locked tests::editing::ui_asset::inspector:: --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 追加验证 15 个 inspector 行为回归，确认 view-projection owner 下沉后 widget/slot/layout/binding inspector 流程未回归
- `cargo test -p zircon_editor --locked tests::editing::ui_asset_replay:: --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 追加验证 17 个 replay 回归，确认 style/promotion/command helper 下沉后 executable replay command 与外部 effect 回放语义仍然稳定
- source-boundary assertion: `presentation_state.rs` contains `pub fn reflection_model(&self) -> UiAssetEditorReflectionModel` and `pub fn pane_presentation(&self) -> UiAssetEditorPanePresentation`, while `ui_asset_editor_session.rs` no longer contains either signature
  - 作为额外守卫，直接证明 view-projection owner 已经迁入 `presentation_state.rs`
- `cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-a`
  - 证明当前 `style_state.rs` / `promotion_state.rs` / `command_entry.rs` / `presentation_state.rs` ownership 收口后的 production 代码可以重新编译
- `cargo test -p zircon_editor --lib editor_manager_becomes_thin_facade_over_editor_ui_host --locked`
  - 证明 `EditorManager` 已退化为统一 `EditorUiHost` 的薄 façade，不再直接持有 host/layout/view/window/session 状态
- `cargo test -p zircon_editor --lib editor_module_owner_moves_under_ui_host --locked`
  - 证明 `EditorModule` / `module_descriptor()` owner 已迁入 `ui::host::module`，crate root 不再从 `core::host::module` 导出
- `cargo test -p zircon_editor --lib editor_asset_boundary_lives_in_editor_crate --locked`
  - 证明 editor asset manager 与 resource access 宿主服务已经迁入 `ui::host`，`core::host` 子树已删除
- `cargo test -p zircon_editor --lib editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors --locked`
  - 证明 `EditorManager` 和 `ui::host::asset_editor_sessions` 的职责边界已经稳定
- `cargo test -p zircon_editor --lib ui_asset_editor_bootstrap_assets_parse_and_compile_with_imports --locked`
  - 证明 editor bootstrap 资产仍能经 shared loader/compiler 打开
- `cargo test -p zircon_editor --lib ui_asset_editor_bootstrap --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture`
  - 证明 `ui_asset_editor.ui.toml` 的 tool-mode row、diagnostic overlay 和 emergency shell band 继续能经 shared loader/compiler/surface projection 导出有效 frame，当前结果为 9 passed / 0 failed / 839 filtered out
- `cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked workbench_retained_shell`
  - 证明 retained workbench shell 合同稳定收敛到 Rust-owned `host_contract` scene/native split，不会回退到旧的散装 surface passthrough
- `cargo test -p zircon_editor --locked workbench_host_window_keeps_retained_shell_dtos_at_ui_boundary_only`
  - 证明 `workbench_host_window` 仍未重新吃回深层 generated pane DTO，而是通过 retained host-contract projection 对外输出
- `cargo test -p zircon_editor --locked host_scene_projection_converts_host_owned_panes_to_host_contract`
  - 证明新的 `retained_host/ui/pane_data_conversion/mod.rs` 真实接管了 pane 投影，而不是只通过源码守卫
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe tests::editing::ui_asset:: --nocapture`
  - 在邻近 `zircon_runtime` Nanite compile drift 阻断普通 `cargo test -p zircon_editor --locked tests::editing::ui_asset:: ...` 时，复用刚编出的 `zircon_editor` lib-test 二进制完成了 66 个 `editing::ui_asset` 行为回归，确认这次 session split 没有破坏现有 authoring 语义
- `cargo test -p zircon_editor --locked tests::editing::ui_asset::source_projection:: --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 追加验证 11 个 source projection / navigation 行为回归，确认 `navigation_state.rs` 切分后 hierarchy、preview、source-outline、source-line、source-byte-offset 选中与 source cursor reconcile/undo 语义保持不变
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe tests::editing::ui_asset::reference_and_promotion:: --nocapture`
  - 追加验证 11 个 reference / external promotion 行为回归，确认 `promotion_state.rs` 切分后 reference convert、component extract、promote draft 与外部资产写回语义保持不变
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe tests::editing::ui_asset_theme_authoring:: --nocapture`
  - 追加验证 21 个 theme authoring 行为回归，确认 `theme_state.rs` 切分后 imported/local theme helper、theme compare、theme refactor 与 promote-theme draft 语义保持不变
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe tests::editing::ui_asset_replay::ui_asset_editor_session_theme_ --nocapture`
  - 追加验证 theme promotion/refactor replay 命令与外部 source restore 语义，确认拆分后共享 replay helper 仍然输出可执行 undo/redo 合同
- `cargo test -p zircon_editor --locked tests::editing::ui_asset_theme_authoring:: --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - 追加验证 21 个 theme authoring 行为回归，确认 `theme_state.rs` 接管 replay helper 后 imported/local theme detach/clone/adopt、theme compare、theme refactor 与 promote-theme draft 语义仍然稳定
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe tests::editing::ui_asset_replay:: --nocapture`
  - 追加验证 17 个 replay 回归，确认 `style_state.rs` 切分后 style rule insert/delete/reorder、undo/redo replay 与跨文件 effect 重放语义保持不变
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe tests::host::manager::ui_asset_style_and_inspector:: --nocapture`
  - 追加验证 10 个 host-facing style/inspector 回归，确认 editor manager 继续通过稳定 session API 驱动 class、rule、declaration、semantic、widget/slot/layout inspector 作者态流程
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe tests::editing::ui_asset_preview_binding_authoring:: --nocapture`
  - 追加验证 21 个 preview/binding authoring 回归，确认 `preview_state.rs` 切分后 preview mock subject/property/nested value/suggestion 流程仍然稳定驱动 preview rebuild 与表达式求值
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe tests::host::manager::ui_asset_session_preview:: --nocapture`
  - 追加验证 8 个 host-facing preview/session 回归，确认 editor manager 继续通过稳定 session API 驱动 preview preset、mock preview、source byte offset 选中与交互式 session command
- `cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 5 个 M5 workspace watcher/conflict 回归，覆盖 clean external reload、dirty conflict preservation、save-local-copy 不解除冲突、diff snapshot、reload/keep-local resolution、stale import failure/recovery
- `cargo test -p zircon_editor --lib emergency_shell --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 4 个 M24 emergency shell/host 回归，覆盖 invalid source -> Emergency、last-valid preview retention、manager revert 和 retained/native emergency action routing；首次冷编译超时后，后台编译完成，warmed rerun 通过
- `cargo test -p zircon_editor --lib ui_asset_workspace_watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证同一 recovery target 下 6 个 workspace conflict/recovery 回归，覆盖 clean reload、dirty conflict、diff snapshot、reload/keep-local、save-local-copy、stale import recovery 和 manager emergency revert
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 211 个 UI Asset Editor authoring/session 回归，确认 emergency shell state、host action routing 和 last-valid recovery 没有破坏 source/preview/tree/style/binding/replay 语义
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-emergency-m24 --message-format short --color never`
  - 证明 M24 emergency shell state、host action routing、retained/native DTO projection 和 editor session surface 在当前 editor lib 下可编译；输出仍包含既有 runtime/editor warning
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 205 个 UI Asset Editor authoring/session 回归，确认 workspace conflict action projection 没有破坏 source/preview/tree/style/binding/replay 语义
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-hot-reload-m5 --message-format short --color never`
  - 证明 M5 hot reload/conflict façade、retained/native action dispatch 和 pane presentation surface 在当前 editor lib 下可编译；输出仍包含既有 runtime/editor warning
- `cargo test -p zircon_editor --lib ui_asset_reference_and_promotion --locked -- --nocapture`
  - 追加验证 12 个 reference/promotion host 回归，确认 promotion undo/redo 删除或恢复外部 `.ui.toml` 时 dependent refresh 不再把 missing file 当作 fatal host error
- `cargo test -p zircon_editor --lib ui_asset_replay --locked -- --nocapture`
  - 追加验证 17 个 replay 回归，确认 workspace refresh wiring 没有破坏 external effect replay source maps 与 undo/redo 合同
- `cargo test -p zircon_editor --lib contract_diagnostics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never`
  - 追加验证 3 个 UI Asset Editor structured contract diagnostic 回归，覆盖 private selector code/control target、API mismatch source-outline target、closed root class code/source path/node target
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-productization-contract --message-format short --color never`
  - 证明 editor diagnostic DTO、session storage 和 pane projection surface 可以通过 `zircon_editor` lib type-check；输出仍包含 unrelated runtime graphics warnings
- `cargo test -p zircon_editor --lib ui_asset_editor_host_genericizes_detail_event_dispatch --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never`
  - 证明 UI Asset Editor detail callbacks 通过 `dispatch_ui_asset_component_adapter_commit` 和 `asset_editor` component adapter mutation route，而不是回退成直接 detail-manager field dispatch
- `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never`
  - 作为 Milestone 0 editor closeout package gate 通过，当前结果为 876 passed / 0 failed / 1 ignored；runtime/workspace broad green 仍未声明
- `cargo test -p zircon_editor --lib ui_asset_editor_bootstrap --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 9 个 UI Asset Editor bootstrap/projection 回归，确认 `ui_asset_editor.ui.toml` authored shell、header rows、panel columns 和 route 仍能打开并投影
- `cargo test -p zircon_editor --lib designer_tools --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 3 个 M6 designer tool 行为：Select/Resize Slot/Preview Interact 工具状态投影、slot preferred size 单次 undoable transaction、preview canvas node binding dispatch
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 208 个 UI Asset Editor authoring/session 回归，确认 designer tool state、slot resize 和 preview interact 投影没有破坏 source/preview/tree/style/binding/replay 语义
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-designer-m6 --message-format short --color never`
  - 证明 M6 designer tool session state、host façade、retained/native action DTO 和 pane presentation surface 在当前 editor lib 下可编译；输出仍包含既有 runtime/editor warnings
- `cargo test -p zircon_editor --lib action_localization_reports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 2 个 M21/M14 runtime report projection 回归，覆盖 action policy report rows 和 localization diagnostic/report rows
- `cargo test -p zircon_editor --lib runtime_reports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 1 个 UI Asset Editor runtime-report projection 回归，覆盖 action-policy rows、locale dependency rows、locale extraction rows 和 diagnostics-empty happy path
- `cargo test -p zircon_editor --lib runtime_report_productization --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 1 个 M21/M14/M15 productization 回归，覆盖 action policy、capability explanations、locale preview selection 和 resource dependency/diagnostic rows
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 219 个 UI Asset Editor authoring/session 回归，确认 runtime-report projection 没有破坏 source/preview/tree/style/binding/replay/emergency/designer 语义
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m21-m14-editor-reports --message-format short --color never`
  - 证明 `runtime_report_state.rs`、`diagnostics/localization.rs`、pane presentation、retained/native DTO conversion 和 editor template-service façade 在当前 editor lib 下可编译；输出仍包含既有 runtime/editor warnings
- `rustfmt --edition 2021 --check zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs zircon_editor/src/ui/asset_editor/session/presentation_state.rs zircon_editor/src/ui/asset_editor/session/runtime_report_state.rs zircon_editor/src/ui/asset_editor/presentation.rs zircon_editor/src/ui/retained_host/host_contract/data/ui_asset.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_ui_asset_conversion.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs`
  - 追加验证 M15 editor dependency view 相关 session/presentation/DTO/test 文件格式，通过且无输出
- `cargo test -p zircon_editor --lib resource_dependency_view --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m15-resource-ux --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 4 个 M15 editor dependency-view 回归，覆盖 `UiAssetEditorSession` typed resource dependency/diagnostic accessors、source edit 刷新和资源编译失败后清空
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m15-resource-ux --message-format short --color never`
  - 证明 `UiAssetEditorSession::resource_dependencies()`、`UiAssetEditorSession::resource_diagnostics()` 和 preview compile refresh wiring 在当前 editor lib 下可编译；输出仍包含既有 runtime/editor warnings
- `cargo test -p zircon_editor --lib dual_host_parity --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m22-parity --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 2 个 M22 runtime/editor parity 回归，覆盖 generic host window、UI Asset Editor shell、Component Showcase、Workbench 在 runtime `UiSurface`、editor host model、retained/native projection 下的 layout frame、稳定模板属性、style token、binding/route id parity，以及 Material value change/commit/action route 的状态 reducer
- `cargo test -p zircon_editor --lib template_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m22-parity --message-format short --color never -- --nocapture --test-threads=1`
  - 追加验证 44 个 template runtime 回归，确认 parity fixture 没有破坏 host model、shared surface、pane payload、Component Showcase state、viewport toolbar、welcome/inspector/asset surface 等投影路径

这组测试组合起来，覆盖了“代码物理位置”“owner 边界”“shared 资产链路”“导航/光标行为回归”和“retained host 入口约束”几个最关键的验收面。

## Designer Canvas Tools Status

M6 designer canvas tools now have a focused behavior closure inside the UI Asset Editor session. `UiDesignerToolMode` is part of the shared editor contract and is projected through the reflection model, pane presentation, retained/native DTO conversion, and action dispatch IDs for Select, Resize Slot, and Preview Interact.

`designer_state.rs` owns the session behavior: it reports resize/interact capability from the selected node and preview host, changes the active tool mode without dirtying source, resizes selected slot preferred width/height through one replay-aware document edit labeled `Resize Slot`, and dispatches preview canvas interactions by selecting the preview node and projecting the matched `.ui.toml` binding/action payload into `UiDesignerPreviewInteractDispatch`.

This is intentionally editor authoring behavior. It does not replace runtime input dispatch or add RHI/rendering dependencies. Remaining UI productization can now build recovery UX, policy/locale inspector projection, resource UX, and runtime/editor parity on top of a tested designer command surface instead of treating the canvas as a read-only projection.

## Runtime Report Productization Status

M21/M14 editor productization now has a focused UI Asset Editor closure. `runtime_report_state.rs` is the session owner for runtime-owned report projection: it validates the last-valid `.ui.toml` document against both `UiActionHostPolicy::runtime_default()` and `UiActionHostPolicy::editor_authoring()`, collects localization dependency/extraction/diagnostic rows, collects resource dependency/diagnostic rows, and exposes locale preview selection state through `UiAssetEditorPanePresentation`.

For action safety, the pane now carries four distinct report groups. `action_policy_items` keeps the editor-authoring diagnostics used by the existing action report; `capability_explanation_items` keeps the static side-effect allow/block summary; `host_enforcement_items` shows runtime-default and editor-authoring profile enforcement side by side; and `unsafe_action_guidance_items` gives per-binding authoring guidance for editor-only asset IO and side effects that require an explicit host capability, such as network or external process work.

The 2026-05-07 host-enforcement presentation slice is covered by focused editor validation in `D:\cargo-targets\zircon-ui-m14-m15-resolver`: `runtime_reports` passed 1 test, `runtime_report_productization` passed 1 test, the broader `ui_asset_editor` filter passed 221 tests, and `cargo check -p zircon_editor --lib` passed with existing warnings.

`resolver_state.rs` now wires the runtime-owned locale/resource resolvers into the editor session. The editor can register external locale table keys for a selected preview locale and configure a `UiResourcePathResolver`; it then projects missing locale tables, missing keys, and missing resource files as pane rows without taking over localization/resource classification semantics.

M15 editor resource dependency view is also focused-accepted at the resolver-backed read-only scope. `UiAssetEditorSession` stores the latest successful `UiCompiledDocument.resource_dependencies()` and `resource_diagnostics()` vectors, refreshes them after valid source edits, clears them when resource validation prevents compile, extends diagnostics with configured resolver file-existence results, and exposes the data through session accessors plus `resource_dependency_items` / `resource_diagnostic_items` rows.

Localization report diagnostics are also mapped into `UiAssetEditorDiagnostic` through `diagnostics/localization.rs`, so invalid localized text refs carry source-path and node-target metadata instead of remaining only pane row strings.

This keeps policy, localization, and resource semantics in runtime/interface-owned DTOs. The editor does not classify side effects, infer localization references, or rescan resources independently; it formats the report rows and forwards them through the existing pane presentation and retained/native data conversion surfaces.

The accepted scope is intentionally narrow: action policy rows, capability explanation rows, host enforcement rows, unsafe action guidance rows, locale preview/dependency/extraction/missing-key diagnostic rows, and resource dependency/file-existence diagnostic rows are visible to the editor host. Resource browser UX, watcher-driven reload, runtime loader backends, and graphics/RHI resource consumption remain separate follow-up milestones.

## Resource Dependency Accessors

M15 now has a resolver-backed editor session view in addition to the runtime-report pane rows. `UiAssetEditorSession` stores `Vec<UiResourceDependency>` and `Vec<UiResourceDiagnostic>` from the latest preview compile and exposes them through `resource_dependencies()` and `resource_diagnostics()`. The vectors are refreshed after a successful preview rebuild or source edit, can be refreshed when configured resolver roots or files change, and are cleared when parse/resource validation fails so stale dependency rows cannot survive a bad compile.

This remains a read-only editor boundary. The editor does not infer resource references, watch resources, load GPU resources, or rescan `.ui.toml` independently; it consumes `UiCompiledDocument::resource_dependencies()` and `UiCompiledDocument::resource_diagnostics()` from the runtime-owned compiler surface, then asks runtime `UiResourcePathResolver` to validate configured roots.

## Dual Host Parity

M22 now has a focused representative parity fixture in `zircon_editor/src/tests/host/template_runtime/dual_host_parity.rs`. The test builds one runtime `UiSurface`, computes layout once, then compares that same frame authority against the editor host model and retained/native host projection for the generic host window, UI Asset Editor shell, Component Showcase, and Workbench templates.

The parity check treats `UiSurface` as the layout/render/hit/input frame source of truth: frame entries, stable authored attributes, style tokens, binding ids, and registered route ids must survive the host/native projection path. Component Showcase intentionally overlays retained demo state in the editor host model, so the fixture filters only those state-owned overlay fields (`selected`, `selection_state`, `value_text`, validation rows, generated `collection_items`) from the raw-template attribute comparison and then verifies the overlay separately through projected collection rows, virtualization metadata, world-space metadata, and material event dispatch.

Accepted focused evidence is `dual_host_parity` passing 2 tests, `template_runtime` passing 44 tests, and `cargo check -p zircon_editor --lib` passing in `D:\cargo-targets\zircon-ui-m22-parity`. This does not claim Graphics/RHI rendering parity or broad workspace green.

## Runtime Interface UI DTO Cutover Status

The 2026-05-02 UI runtime-interface audit separates remaining editor `zircon_runtime` usage into two groups.

Concrete runtime services still intentionally come from `zircon_runtime::ui`: `UiSurface`, `UiEventManager`, `UiTemplateBuildError`, `UiComponentDescriptorRegistry`, `UiAssetDocumentRuntimeExt`, runtime pointer/surface dispatchers, and runtime layout/render behavior exposed through those services. `EditorTemplateRuntimeService` now owns the high-level editor façade for `.ui.toml` parsing, document compilation, registry registration/instantiation, surface construction, render extraction, and binding diagnostic collection; that façade is the editor-owned boundary around `UiAssetLoader`, `UiDocumentCompiler`, `UiTemplateSurfaceBuilder`, and `extract_ui_render_tree(...)`. These dependencies are runtime behavior APIs, not neutral DTO ownership.

The latest Milestone 4 audit found 134 `zircon_runtime::ui` hits and 431 `zircon_runtime_interface::ui` hits under `zircon_editor/src`. The editor is already broadly consuming interface DTOs for IDs, layout geometry, component values, binding reports, dispatch records, and template asset records, but the remaining runtime hits cannot be treated as one mechanical import rewrite.

The tree/surface identity blocker has been cut over. `zircon_runtime_interface::ui::tree` owns neutral `UiTree`, `UiTreeNode`, `UiInputPolicy`, and `UiTreeError` declarations, and `zircon_runtime::ui::surface::UiSurface` stores that interface `UiTree` directly. Editor files that construct a `UiSurface`, insert `UiTreeNode`s, or set `UiInputPolicy` import those DTOs from `zircon_runtime_interface::ui::tree`; files that call insertion, query, mutation, routing, focus, scroll, or other tree behavior import the needed `zircon_runtime::ui::tree::UiRuntimeTree*Ext` trait instead of importing DTOs through runtime.

The follow-up non-tree DTO audit did not find a safe remaining DTO-only import that is still exported from `zircon_runtime::ui`. Targeted searches for runtime-owned `binding::Ui*`, `event_ui::Ui*`, `layout::Ui*`, `surface::UiRender*/UiResolved*/UiText*`, `component::UiValue/UiComponentState/UiComponentEvent/UiDrag*`, dispatch DTOs, and template asset DTOs only found runtime service or behavior names such as `UiEventManager`, `UiComponentDescriptorRegistry`, and `UiAssetDocumentRuntimeExt`. Current editor source still has mixed files where interface DTOs appear near runtime services, but those DTOs already import from `zircon_runtime_interface::ui`; the remaining runtime imports should stay until concrete service APIs move or narrower capability facades replace them. The new `EditorTemplateRuntimeService` narrows the most common high-level runtime-template seams without pretending that low-level pointer/surface dispatchers are interface DTOs. `EditorTemplateRegistry` now stores compiled documents only; callers must use the service façade for compilation and instantiation so registry code no longer owns runtime compiler behavior directly. The 2026-05-02 19:44 rerun `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-interface-package-cache-opencode --message-format short --color never` passed with existing runtime graphics warnings and 3 editor warnings. That proves the UI DTO split and editor template service façade type-check in the current worktree; it does not claim the editor no longer depends on runtime services, and it does not imply workspace-test green.
