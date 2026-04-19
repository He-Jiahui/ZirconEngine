---
related_code:
  - zircon_ui/src/template/asset/legacy.rs
  - zircon_ui/src/tests/asset.rs
  - zircon_editor/src/ui/template/catalog.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/ui_asset_editor.rs
  - zircon_editor/src/core/editing/ui_asset/mod.rs
  - zircon_editor/src/core/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/core/editing/ui_asset/binding/schema_projection.rs
  - zircon_editor/src/core/editing/ui_asset/command.rs
  - zircon_editor/src/core/editing/ui_asset/document_diff.rs
  - zircon_editor/src/core/editing/ui_asset/inspector_fields.rs
  - zircon_editor/src/core/editing/ui_asset/replay_workspace.rs
  - zircon_editor/src/core/editing/ui_asset/value_path.rs
  - zircon_editor/src/core/editing/ui_asset/preview/mock_expression.rs
  - zircon_editor/src/core/editing/ui_asset/preview/mock_value_resolution.rs
  - zircon_editor/src/core/editing/ui_asset/preview_mock.rs
  - zircon_editor/src/core/editing/ui_asset/presentation.rs
  - zircon_editor/src/core/editing/ui_asset/preview_host.rs
  - zircon_editor/src/core/editing/ui_asset/preview_projection.rs
  - zircon_editor/src/core/editing/ui_asset/palette_drop.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/mod.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/resolution.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/overlay_slots.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/grid_slots.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/flow_slots.rs
  - zircon_editor/src/core/editing/ui_asset/palette_target_chooser.rs
  - zircon_editor/src/core/editing/ui_asset/session.rs
  - zircon_editor/src/core/editing/ui_asset/session/mod.rs
  - zircon_editor/src/core/editing/ui_asset/session/session_state.rs
  - zircon_editor/src/core/editing/ui_asset/session/preview_compile.rs
  - zircon_editor/src/core/editing/ui_asset/session/style_inspection.rs
  - zircon_editor/src/core/editing/ui_asset/session/hierarchy_projection.rs
  - zircon_editor/src/core/editing/ui_asset/promote_widget.rs
  - zircon_editor/src/core/editing/ui_asset/source_buffer.rs
  - zircon_editor/src/core/editing/ui_asset/source_sync.rs
  - zircon_editor/src/core/editing/ui_asset/style/theme_authoring.rs
  - zircon_editor/src/core/editing/ui_asset/style/theme_cascade_inspection.rs
  - zircon_editor/src/core/editing/ui_asset/style/theme_compare.rs
  - zircon_editor/src/core/editing/ui_asset/style_rule_declarations.rs
  - zircon_editor/src/core/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/core/host/resource_access.rs
  - zircon_editor/src/core/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/core/host/manager/workspace_state.rs
  - zircon_editor/src/core/host/manager/builtin_views/activity_views/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/activity_windows/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/core/host/manager/project_access.rs
  - zircon_editor/src/core/host/manager/ui_asset_promotion.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/mod.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/binding.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/inspector.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/navigation.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/node_ops.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/palette.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/source.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/style.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - zircon_editor/src/ui/slint_host/ui/pane_projection.rs
  - zircon_editor/assets/ui/editor/preview_state_lab.ui.toml
  - zircon_editor/assets/ui/runtime/quest_log_dialog.ui.toml
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_components.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/semantics.rs
  - zircon_asset/src/assets/mod.rs
  - zircon_asset/src/assets/imported.rs
  - zircon_asset/src/assets/ui.rs
  - zircon_asset/src/importer/service/mod.rs
  - zircon_asset/src/importer/service/import_from_source.rs
  - zircon_asset/src/importer/service/import_ui_asset.rs
  - zircon_asset/src/project/manager/mod.rs
  - zircon_asset/src/project/manager/scan_and_import.rs
  - zircon_asset/src/project/manager/asset_kind.rs
  - zircon_asset/src/project/manager/source_uri_for_path.rs
  - zircon_asset/src/project/manager/load_or_create_meta.rs
  - zircon_asset/src/editor/manager/mod.rs
  - zircon_asset/src/pipeline/manager/records/mod.rs
  - zircon_resource/src/locator.rs
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/editing/ui_asset_theme_authoring.rs
  - zircon_editor/src/tests/editing/ui_asset_palette_drop.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/src/tests/host/resource_access.rs
  - zircon_editor/src/tests/host/slint_tab_drag.rs
  - zircon_editor/src/tests/host/template_runtime.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/tests/workbench_slint_ui_asset_theme_shell.rs
  - zircon_asset/src/tests/assets/ui.rs
  - zircon_asset/src/tests/editor/boundary.rs
  - zircon_asset/src/tests/editor/manager.rs
implementation_files:
  - zircon_ui/src/template/asset/legacy.rs
  - zircon_editor/src/ui/template/catalog.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/ui_asset_editor.rs
  - zircon_editor/src/core/editing/ui_asset/mod.rs
  - zircon_editor/src/core/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/core/editing/ui_asset/binding/schema_projection.rs
  - zircon_editor/src/core/editing/ui_asset/command.rs
  - zircon_editor/src/core/editing/ui_asset/document_diff.rs
  - zircon_editor/src/core/editing/ui_asset/inspector_fields.rs
  - zircon_editor/src/core/editing/ui_asset/replay_workspace.rs
  - zircon_editor/src/core/editing/ui_asset/value_path.rs
  - zircon_editor/src/core/editing/ui_asset/preview/mock_expression.rs
  - zircon_editor/src/core/editing/ui_asset/preview/mock_value_resolution.rs
  - zircon_editor/src/core/editing/ui_asset/preview_mock.rs
  - zircon_editor/src/core/editing/ui_asset/presentation.rs
  - zircon_editor/src/core/editing/ui_asset/preview_host.rs
  - zircon_editor/src/core/editing/ui_asset/preview_projection.rs
  - zircon_editor/src/core/editing/ui_asset/palette_drop.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/mod.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/resolution.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/overlay_slots.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/grid_slots.rs
  - zircon_editor/src/core/editing/ui_asset/tree/palette_drop/flow_slots.rs
  - zircon_editor/src/core/editing/ui_asset/palette_target_chooser.rs
  - zircon_editor/src/core/editing/ui_asset/session.rs
  - zircon_editor/src/core/editing/ui_asset/session/mod.rs
  - zircon_editor/src/core/editing/ui_asset/session/session_state.rs
  - zircon_editor/src/core/editing/ui_asset/session/preview_compile.rs
  - zircon_editor/src/core/editing/ui_asset/session/style_inspection.rs
  - zircon_editor/src/core/editing/ui_asset/session/hierarchy_projection.rs
  - zircon_editor/src/core/editing/ui_asset/promote_widget.rs
  - zircon_editor/src/core/editing/ui_asset/source_buffer.rs
  - zircon_editor/src/core/editing/ui_asset/source_sync.rs
  - zircon_editor/src/core/editing/ui_asset/style/theme_authoring.rs
  - zircon_editor/src/core/editing/ui_asset/style/theme_cascade_inspection.rs
  - zircon_editor/src/core/editing/ui_asset/style/theme_compare.rs
  - zircon_editor/src/core/editing/ui_asset/style_rule_declarations.rs
  - zircon_editor/src/core/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/core/host/resource_access.rs
  - zircon_editor/src/core/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/core/host/manager/workspace_state.rs
  - zircon_editor/src/core/host/manager/builtin_views/activity_views/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/activity_windows/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/core/host/manager/project_access.rs
  - zircon_editor/src/core/host/manager/ui_asset_promotion.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/mod.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/binding.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/inspector.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/navigation.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/node_ops.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/palette.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/source.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions/editing/style.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/ui/pane_projection.rs
  - zircon_editor/assets/ui/editor/preview_state_lab.ui.toml
  - zircon_editor/assets/ui/runtime/quest_log_dialog.ui.toml
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_components.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/semantics.rs
  - zircon_asset/src/assets/ui.rs
  - zircon_asset/src/importer/service/mod.rs
  - zircon_asset/src/importer/service/import_from_source.rs
  - zircon_asset/src/importer/service/import_ui_asset.rs
  - zircon_asset/src/project/manager/mod.rs
  - zircon_asset/src/project/manager/scan_and_import.rs
  - zircon_asset/src/project/manager/asset_kind.rs
  - zircon_asset/src/project/manager/source_uri_for_path.rs
  - zircon_asset/src/project/manager/load_or_create_meta.rs
  - zircon_asset/src/editor/manager/mod.rs
  - zircon_asset/src/pipeline/manager/records/mod.rs
  - zircon_resource/src/locator.rs
plan_sources:
  - user: 2026-04-16 增加 activityWindow 界面作为 UI 编辑布局工具并把 UI Layout 资产化
  - user: 2026-04-16 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-16 继续把完整 zircon_editor 宿主实现补完
  - user: 2026-04-17 继续缺漏内容补充
  - user: 2026-04-17 下一条最合理的 task 是把 Promote To External Widget Asset 接上
  - user: 2026-04-18 给多命名 slot / 低语义 slot 增加真正的 manual slot picker 或 target cycle，并恢复 zircon_editor --lib host/template 回归基线
  - user: 2026-04-18 实现计划，生成的toml要求能够可视化编辑（提供了editor UI的编辑器）
  - user: 2026-04-19 把 inverse-command 继续推进到 replay backend，并补 richer preview mock / bindings schema、theme tooling、更多迁移与自举
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
tests:
  - zircon_ui/src/tests/asset.rs
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
  - zircon_editor/src/tests/editing/ui_asset_palette_drop.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/src/tests/ui/ui_asset_editor.rs
  - zircon_editor/src/tests/ui/activity.rs
  - zircon_editor/src/tests/ui/template.rs
  - zircon_editor/src/tests/host/template_runtime.rs
  - zircon_asset/src/tests/assets/ui.rs
  - zircon_asset/src/tests/editor/manager.rs
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_session_creates_stylesheet_rule_from_selected_node --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_session_adds_and_removes_selection_classes --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_session_selects_renames_and_deletes_local_stylesheet_rules --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_session_upserts_and_deletes_local_tokens --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_session_selects_upserts_and_deletes_stylesheet_rule_declarations --locked
  - cargo test -p zircon_editor --lib editor_manager_runs_ui_asset_style_class_editing_actions --locked
  - cargo test -p zircon_editor --lib editor_manager_runs_ui_asset_style_rule_editing_actions --locked
  - cargo test -p zircon_editor --lib editor_manager_runs_ui_asset_style_token_editing_actions --locked
  - cargo test -p zircon_editor --lib editor_manager_runs_ui_asset_style_rule_declaration_editing_actions --locked
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_authoring_buttons_and_state_bindings --locked
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_class_authoring_controls_and_callback --locked
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_rule_editing_controls_and_callback --locked
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_token_editing_controls_and_callback --locked
  - cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_rule_declaration_editing_controls_and_callback --locked
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_session_projects_structured_widget_inspector_fields
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_session_updates_selected_widget_inspector_fields
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_session_projects_structured_slot_inspector_fields
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_session_updates_selected_slot_inspector_fields
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_session_extracts_selected_node_into_local_component
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_session_projects_and_updates_promote_widget_draft_fields
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_session_promotes_selected_local_component_to_external_widget_asset
  - cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_widget_inspector_editing_actions
  - cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_slot_inspector_editing_actions
  - cargo test -p zircon_editor --lib --locked editor_manager_extracts_selected_ui_asset_node_to_local_component
  - cargo test -p zircon_editor --lib --locked editor_manager_uses_custom_promote_widget_draft_values
  - cargo test -p zircon_editor --lib --locked editor_manager_promotes_selected_ui_asset_component_to_external_widget_asset
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_widget_inspector_editing_controls_and_callback
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_slot_inspector_editing_controls
  - cargo test -p zircon_editor --lib --locked --offline ui_asset_palette_drop::
  - cargo test -p zircon_editor --lib --locked --offline
  - cargo test -p zircon_editor --test workbench_slint_shell --locked --offline
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_extract_component_action_and_state_binding
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_promote_widget_action_and_state_binding
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_promote_widget_draft_controls
  - cargo test -p zircon_editor --lib --locked palette_drag_drop_to_hovered_preview_node
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_palette_drag_creation_flow -- --exact
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --test workbench_slint_shell --locked
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_asset --lib --locked
  - cargo test -p zircon_editor ui_asset_editor_subsystem_is_grouped_by_domain_folders -- --nocapture
  - cargo test -p zircon_editor editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors -- --nocapture
  - cargo test -p zircon_editor editor_template_runtime_splits_builtin_data_from_runtime_pipeline -- --nocapture
  - CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --test workbench_slint_shell --locked --offline
  - CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline
  - CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_asset --locked --offline
  - CARGO_TARGET_DIR=target/codex-ui-asset-editor-manager cargo test -p zircon_manager --locked --offline
  - CARGO_TARGET_DIR=target/codex-ui-asset-editor-resource cargo test -p zircon_resource --locked --offline
  - cargo build --workspace --locked --verbose
  - cargo test --workspace --locked --verbose
doc_type: module-detail
---

# UI Asset Editor Host Session

## Purpose

这份文档记录 `zircon_editor` 里已经落地的第一条完整 `UI Asset Editor` 宿主链路，而不是只描述 shared `zircon_ui` AST。

当前闭环已经覆盖：

- `.ui.toml` `layout/widget/style` 在 `zircon_asset` 里作为正式资产种类被导入、建 catalog、建 direct reference graph
- `editor.ui_asset` 作为真实 `ActivityWindow` 被 `EditorManager` 打开、恢复、保存和同步 dirty 状态
- `UiAssetEditorSession` 维护 source buffer、undo/redo、last-good preview、selection/style inspector 和 import hydration
- `workbench.slint` / `panes.slint` 暴露真实的 UI asset editor pane 与交互 callback，而不是只停留在 route 协议
- editor-only mock preview override 现在也走正式会话链路，不写回 `.ui.toml`

这意味着当前仓库里已经有“共享资产格式 -> editor 资产系统 -> session -> Slint host pane”这一条可运行主链。

## Responsibility Split

- `zircon_ui`
  - 仍然是 `UiAssetDocument`、`UiDocumentCompiler`、`UiCompiledDocument`、`UiSurface` 的权威层
  - 不关心 `ActivityWindow`、document tab、save/undo/redo
- `zircon_asset`
  - 负责把 `.ui.toml` 导入成 `UiLayoutAsset`、`UiWidgetAsset`、`UiStyleAsset`
  - 负责把 UI import 引用变成 editor catalog/reference graph 可以消费的 `AssetReference`
- `zircon_editor::ui`
  - 定义 `UiAssetEditorRoute`、`UiAssetEditorMode`、`UiAssetEditorReflectionModel`
  - 不实现文件读写和 preview host
- `zircon_editor`
  - 负责 session、source roundtrip、import hydration、canonical save、pane presentation、Slint callback dispatch
- `Slint`
  - 只承接 pane 展示和 callback 上传
  - 不自己解释 `.ui.toml`，也不自己成为布局真源

## Host Manager Session Tree

`zircon_editor/src/core/host/manager/ui_asset_sessions/` 现在按 host orchestration 行为拆成目录树，而不是继续把所有 editor asset 命令堆进一个 `mod.rs` 或单体 `editing.rs`：

- `mod.rs`
  - 只做结构接线与共享 re-export
- `editing.rs`
  - 只保留 descriptor、workspace entry、preview preset、外部副作用辅助函数
- `editing/source.rs`
  - source buffer 写回与 import re-hydration 入口
- `editing/style.rs`
  - 规则、token、pseudo state、class、declaration 的 host 侧编排
- `editing/inspector.rs`
  - widget / slot / layout inspector 与 semantic 编辑编排
- `editing/binding.rs`
  - binding 选择、事件、route、payload 相关 host 命令
- `editing/navigation.rs`
  - undo/redo、mode 切换、hierarchy/source outline 导航
- `editing/palette.rs`
  - palette drag target、候选切换、confirm/cancel、drop/insert
- `editing/node_ops.rs`
  - reference/component/widget promotion 与 move/reparent/wrap/unwrap

这条边界的硬约束是：`ui_asset_sessions` 仍然只属于 editor host orchestration，不迁入 `zircon_manager`，也不重新吸收 session 内部实现细节。

## Asset Registration And Project Catalog

`zircon_asset` 现在把 UI 资产作为正式 imported asset 分支处理：

- `ImportedAsset::{UiLayout, UiWidget, UiStyle}` 已加入 `zircon_asset/src/assets/imported.rs`
- `AssetImporter::import_from_source(...)` 会对 `.ui.toml` 先后尝试 `UiLayoutAsset::from_toml_str(...)`、`UiWidgetAsset::from_toml_str(...)`、`UiStyleAsset::from_toml_str(...)`
- `zircon_asset::project::ProjectManager::scan_and_import(...)` 会把三类 UI 资产映射到 `AssetKind::{UiLayout, UiWidget, UiStyle}`
- `ui_asset_references(...)` 会把 `imports.widgets` / `imports.styles` 转成 direct `AssetReference`，因此 editor catalog 和 `used_by` 图不再把 UI 资产当成黑盒

这一层的结果是：

- `EditorAssetManager` 的目录树、详情面板和引用图已经能区分 `UiLayout`、`UiWidget`、`UiStyle`
- `UI Asset Editor` 保存后触发 `AssetManager::import_asset(...)` 时，catalog 与 preview 侧会收到正常的 asset 更新链路

## Session Model

`UiAssetEditorSession` 当前是 editor 侧的最小权威对象，持有：

- `UiAssetEditorRoute`
- `UiAssetSourceBuffer`
- `UiAssetEditorUndoStack`
- `last_valid_document`
- `last_valid_compiled`
- `UiAssetPreviewHost`
- diagnostics
- `UiDesignerSelectionModel`
- `UiStyleInspectorReflectionModel`
- 已解析的 widget/style import registry

当前行为固定为：

- 打开时先 parse 当前源码，再尝试 compile preview
- preview compile 失败不会阻止会话创建；session 会保留 diagnostics，等 import hydration 完成后再恢复 preview
- source 变成非法 TOML 时，只更新 diagnostics，不丢掉 last-good preview
- session 现在还会显式维护 `source_cursor_byte_offset + source cursor anchor(node_id + line_offset)`，所以 Hierarchy、Canvas、Source Outline 与 source-panel cursor 都能共享同一条 block-level 选中语义
- source roundtrip 现在不再只支持“跳到 block 起始行”；`select_source_line(...)` / `select_source_byte_offset(...)` 会把 block 内任意行映射回选中节点，并在 canonical rewrite、undo/redo、invalid-source fallback 后尽量保留 block 内行偏移
- source 进入非法状态时，preview / inspector / source outline 继续锚定最后一个有效 document，但 source cursor byte offset 会按当前非法 source 文本重映射；这让 source panel 能保持用户当前编辑位置，同时不丢掉 last-valid selection
- save 统一走 canonical TOML serializer，保存后重置 dirty 标志
- `zircon_ui::template::UiLegacyTemplateAdapter::layout_source(...)` 现在可以直接生成 canonical `.ui.toml` 源码，因此旧 template 文档生成出来的 TOML 也能被现有 `UiAssetEditorSession::from_source(...)` 直接打开，而不是再要求 editor 侧补一条私有转换链
- undo/redo 现在统一走 `UiAssetEditorUndoTransition` 执行后端：source edit 与 tree authoring edit 都先生成可执行的 source replace-diff；tree authoring edit 还会额外保存结构化 `UiAssetEditorTreeEdit + UiAssetDocumentDiff`，恢复时回放 `source diff + document diff` 并重建 preview/inspector，而不是整串 source snapshot 覆盖
- `UiAssetDocumentDiff` 现在已经从“整节点/整组件替换”推进到 node/component 字段级 patch，并补上 `UiChildMountListDiff` 子边列表回放：`control_id/classes/params/props/layout/style_overrides/root/style_scope` 这类变化会按字段恢复，child mount 则按 `child id` 稳定键重建目标顺序，同时保留无关额外子项，避免 undo/redo 在恢复目标字段时顺带吞掉无关的 props、children 或 component schema 扩展
- `UiAssetDocumentDiff` 这一轮继续深入到了 stylesheet/rule vector：当只修改某一条 `UiStyleRule.selector/set` 时，undo/redo 不再整张 stylesheets 向量替换，而是按 stylesheet index + rule index 回放局部 patch，未触及的 stylesheet/rule 上的临时附加内容不会被顺手冲掉；只有长度变化这类更粗粒度结构漂移时才回退到整向量替换
- 结构化 tree edit 目前已经覆盖 `InsertPaletteItem`、`MoveNode`、`ReparentNode`、`WrapNode`、`UnwrapNode`、`ConvertToReference`、`ExtractComponent`、`PromoteToExternalWidget`；其中 `PromoteToExternalWidget` 还额外携带 host-side external effect 元数据，所以 undo/redo 已不再只回滚 session 内部 source/document，而会同步删除或重建生成的外部 widget `.ui.toml`
- command-log 侧的 inverse metadata 也不再只在查询时“临时推导”有限几个动作；`UiAssetEditorUndoStack` 现在会在 entry 写入时基于 before/after document 记录显式 `UiAssetEditorInverseTreeEdit`，因此 `InsertPaletteItem`、`UnwrapNode`、`ReparentNode(outdent)`、`ConvertToReference`、`ExtractComponent`、`PromoteToExternalWidget` 都已经有稳定的逆向结构化描述，而不是退化成 source 文本级黑盒
- 选中节点后可以一键把当前节点投影成 selector rule，并直接落回 canonical source
- inline `style_overrides` 可以提取成 stylesheet rule，同时清空节点上的 inline block
- `hover/focus/pressed/disabled/selected` 伪状态预览只更新 style inspector，不落盘、不污染 source dirty
- preview root 约束预设现在已经接入会话路由：`Editor Docked`、`Editor Floating`、`Game HUD`、`Dialog`
- preview preset 属于 editor-only 会话状态，不写回 `.ui.toml`，但会跟随 `UiAssetEditorRoute` 一起持久化到 view payload，便于宿主恢复
- preview mock state 现在也是 editor-only：session 会投影可 mock 的 `props`，支持 `Text/Bool/Enum/Resource` 四类浅量值覆盖，重建 preview surface 但不修改 source buffer
- `HorizontalBox` / `VerticalBox` 的 `container.gap` 现在也已经进入 typed layout inspector、Slint pane projection 与宿主 action 路由；`layout.box.gap.set` 会稳定映射回 canonical `layout.container.gap`
- 选中节点的 `classes` 现在可以通过 session API 直接追加/删除，并保持 canonical source、style inspector 和 Slint pane 同步
- palette 当前选中项被拖进 Designer Canvas 时，session 现在会按 preview surface 坐标解析“当前 hover 到哪一个 preview frame”，并把瞬时落点投影成 `palette_drag_target_preview_index`、`palette_drag_target_action`、`palette_drag_target_label`，同时在内部解析结构化 `UiAssetPaletteInsertPlan + UiAssetPaletteInsertionPlacement`
- 当 hovered target 是共享原生容器时，session 不再只做 label-aware 提示，而是会真正合成 slot placement：`Overlay` 自动写入 `slot.layout.anchor/pivot/position`，`GridBox` 自动写入 `slot.row/column`，`FlowBox` 自动写入 `slot.break_before/alignment`
- 当 hovered target 是本地 `component` 实例或 external widget `reference` 实例时，session 也会按组件 slot schema 解析可落点 mount，不再只覆盖共享原生容器；对多命名 slot 或低语义 slot，session 会先投影显式 slot region overlay，再优先用这些 overlay 的几何命中结果决定 mount，只有没有命中任何显式 region 时才回退到 slot 名语义推断
- 这意味着当前已经有一版真正可手动选择的 slot picker：用户把 palette 拖到哪个显式 slot region，就落到哪个 mount；`slot_a/slot_b/slot_c` 这类低语义组件不再只能靠名字猜测
- 对 overlay 太小、区域重叠或低语义 slot 很多的情况，session 现在还会把当前 hover 上下文展开成结构化 `UiAssetPaletteDragResolution { candidates, selected_index }`，并投影成 `palette_drag_candidate_items + palette_drag_candidate_selected_index`；几何命中只决定默认候选，真正 drop 会使用当前被 cycle 选中的 candidate
- 对 low-semantic multi-slot target，第一次 release 现在不会立刻写 source，而是先把 chooser 置成 sticky/manual 模式并在 drop 后继续停留；只有用户明确 `Apply`、回车确认，或先手动 cycle/select 再次 drop 时，当前 candidate 才会真正提交
- `cycle_palette_drag_target_candidate_previous/next()` 已经接入会话状态，manual selection 会在 hover 上下文稳定时保持，不会被同一组候选的后续 pointer move 立刻覆盖；只有候选集合真的变了、选中切换、source roundtrip、undo/redo restore 或文档重新验证时，palette drag resolution 才会被清空/重建
- 这条 palette drag 目标状态不是 source 权威数据；它会在 selection 切换、source roundtrip、undo/redo snapshot restore 和文档重新验证后自动清空，避免上一次 hover 命中结果污染后续 authoring
- `UiAssetCanvasSurface` 对 palette 外部拖拽也不再只亮一圈 frame border；当前 child drop 会绘制内部 slot-aware `drop_inside_overlay`，after drop 会绘制独立的 `drop_after_overlay`，并且当宿主拿到结构化 slot target 时，会继续绘制 `palette_drag_slot_target_items` 驱动的显式 `Grid per-cell`、`Overlay per-anchor`、`Flow per-line/alignment` 与 semantic named-slot region overlays
- `panes.slint` 里的 source editor 现在已经切到 `UiAssetSourceTextInput` 包装层，通过 Slint 支持的 `set-selection-offsets(...)` 驱动可见光标；宿主投影的 source cursor 不再只是 pane 数据字段，而会真正反映到 source 面板里的光标位置
- 当 Palette 当前选中的是 imported widget reference entry，且当前节点不存在不可保留的 `bindings/layout/unknown props` 时，session 现在可以直接把当前节点转成 `reference` 节点，并把兼容的 `props`/`params` 收敛到目标组件参数上
- 当前选中节点现在也可以直接抽成本地 `component`：session 会把原节点树移动到新的 `[components.<Name>]` + 组件 root 节点之下，并把原位置节点替换成 `kind = "component"` 的实例节点，同时刷新 Palette 中的本地组件条目
- 如果当前选中节点已经是本地 `component` 实例，session 现在还可以继续把它提升成外部 `widget` 资产：它会生成独立 `UiAssetDocument(kind = "widget")`，复制目标组件及其本地组件依赖闭包，沿用当前文档的 imports/tokens/stylesheets，并把当前文档里所有对该本地组件的实例统一改写成 `reference`
- promote 草稿现在不再是只读默认值；session 会为当前选中的本地组件维护可编辑的 `asset_id`、`component_name`、`document_id` 三元组，并在切换选中组件时重建默认草稿，在用户已编辑当前组件草稿时保持自定义值
- 本地 stylesheet rule 现在可以在 session 内被选中、改写 selector 和删除；选中的 rule 会跨 source roundtrip 保持稳定索引并在删除后自动回退到下一个可用 rule
- 文档本地 `tokens` 现在可以在 session 内被选中、改名/改值和删除；token 编辑同样直接回写 canonical source，并在删除后自动回退到最近仍存在的 token
- 结构化 Inspector 已经能直接编辑选中节点的 `control_id` 与 `props.text`，并保持 canonical source、preview tree 和 Slint pane 同步
- 同一条 Inspector 链路现在还支持编辑当前父子边上的公共 slot 字段：`mount`、`slot.padding`、`slot.layout.width.preferred`、`slot.layout.height.preferred`
- 选中节点自身的公共 layout 字段现在也可结构化编辑：`layout.width.preferred`、`layout.height.preferred`
- 共享 `bindings` 现在已经接入宿主 Inspector，可对选中节点的 binding 列表执行选择、`Add Click`、删除，以及 `id/event/route` 三字段编辑
- slot 数值字段会在 session 层做 numeric literal 校验；空字符串表示删除对应 leaf，非法非数值输入会返回结构化错误

## Manager Lifecycle

`EditorManager` 的 `ui_asset_sessions/` 子树现在负责整个实例生命周期：

- `open_ui_asset_editor_by_id(...)`
  - 允许 `res://path.ui.toml#Component` 形式输入
  - `normalize_ui_asset_asset_id(...)` 会在文件解析前去掉 `#Component`，避免把组件后缀当成真实文件名
- `restore_ui_asset_editor_instance(...)`
  - 允许从已序列化 route 或旧 payload 恢复
- `save_ui_asset_editor(...)`
  - 写回 canonical source
  - 对 `res://` 资产立即调用 `AssetManager::import_asset(...)`
  - host recompute 现在还会把 projected floating-window frame 回写到 `EditorManager` 的 native window host registry，再生成 `FloatingWindowProjectionBundle`；这样 detached child window 的 cached `host_frame`、document-tab pointer layout 和 child callback size fallback 都会共用同一份非零 bounds，而不是在 bundle 里留下 `None`
- `undo_ui_asset_editor(...)` / `redo_ui_asset_editor(...)`
- `set_ui_asset_editor_mode(...)`
- `select_ui_asset_editor_hierarchy_index(...)`
- `select_ui_asset_editor_preview_index(...)`
- `select_ui_asset_editor_preview_mock_property(...)`
- `set_ui_asset_editor_selected_preview_mock_value(...)`
- `clear_ui_asset_editor_selected_preview_mock_value(...)`
- `create_ui_asset_editor_rule_from_selection(...)`
- `extract_ui_asset_editor_inline_overrides_to_rule(...)`
- `toggle_ui_asset_editor_pseudo_state(...)`
- `add_ui_asset_editor_class_to_selection(...)`
- `remove_ui_asset_editor_class_from_selection(...)`
- `update_ui_asset_editor_palette_drag_target(...)`
- `clear_ui_asset_editor_palette_drag_target(...)`
- `drop_ui_asset_editor_selected_palette_item_at_drag_target(...)`
- `extract_ui_asset_editor_selected_node_to_component(...)`

## 2026-04-18 Structural Split

这一轮结构化重构把热点文件先收敛成目录树，再继续保留既有行为：

- `zircon_editor/src/core/editing/ui_asset/`
  - 已拆成 `binding/`、`preview/`、`source/`、`style/`、`tree/`、`session/` 六个域，根 `mod.rs` 只做公开入口接线
  - `session/mod.rs` 现在只负责拼装 session façade，`session_state.rs`、`preview_compile.rs`、`style_inspection.rs`、`hierarchy_projection.rs` 承接真实子责任；`tree/palette_drop/` 也已把 resolution 与 overlay/grid/flow slot 几何拆开
- `zircon_editor/src/core/host/manager/ui_asset_sessions/`
  - `open.rs`、`save.rs`、`lifecycle.rs`、`sync.rs`、`imports.rs`、`hydration.rs`、`preview_refresh.rs` 现在承接 host orchestration
  - `mod.rs` 已降为目录接线层，原先堆在根文件里的编辑命令入口转入 `editing.rs`
- `zircon_asset/src/editor/manager/`
  - `default_editor_asset_manager.rs` 已降成 declaration-only 根入口，`default_editor_asset_manager/` 子树拆开 `EditorAssetState`、catalog/details façade、trait bridge、record/reference projection 与 parse/error helper
  - `preview_refresh.rs` 与 `project_sync.rs` 也都已降成 structural root，分别把 preview mutation/generation/palette 与 project hydration/path/meta helper 下沉到各自子树；`folder_projection.rs`、`reference_analysis.rs` 继续各守单一职责
- `promote_ui_asset_editor_selected_component_to_external_widget(...)`
- `set_ui_asset_editor_selected_widget_control_id(...)`
- `set_ui_asset_editor_selected_widget_text_property(...)`
- `set_ui_asset_editor_selected_slot_mount(...)`
- `set_ui_asset_editor_selected_slot_padding(...)`
- `set_ui_asset_editor_selected_slot_width_preferred(...)`
- `set_ui_asset_editor_selected_slot_height_preferred(...)`
- `set_ui_asset_editor_selected_layout_width_preferred(...)`
- `set_ui_asset_editor_selected_layout_height_preferred(...)`
- `select_ui_asset_editor_binding(...)`
- `add_ui_asset_editor_binding(...)`
- `delete_ui_asset_editor_selected_binding(...)`
- `set_ui_asset_editor_selected_binding_id(...)`
- `set_ui_asset_editor_selected_binding_event(...)`
- `set_ui_asset_editor_selected_binding_route(...)`
- `select_ui_asset_editor_stylesheet_rule(...)`
- `rename_ui_asset_editor_selected_stylesheet_rule(...)`
- `delete_ui_asset_editor_selected_stylesheet_rule(...)`
- `select_ui_asset_editor_style_token(...)`
- `upsert_ui_asset_editor_style_token(...)`
- `delete_ui_asset_editor_selected_style_token(...)`

最关键的行为是递归 import hydration：

- open / restore / source update / undo / redo / save 之后都会重新收集当前文档的 `imports`
- manager 会递归加载 nested widget/style imports
- 然后统一调用 `session.replace_imports(...)`

这样做避免了两个宿主各维护一套 import registry，也保证引用 widget 资源被热更新后，session 下一次 revalidate 会走同一条 shared compiler path。

`Promote To External Widget Asset` 额外补上的宿主语义是：

- manager 只允许在当前存在打开 project 的情况下执行 promote，因为目标文件会落到 `<project>/assets/ui/widgets/`
- session 会先为当前组件生成默认 promote 草稿，例如 `SaveButton -> res://ui/widgets/save_button.ui.toml + ui.widgets.save_button`，但 inspector 允许用户在真正 promote 前改写 `asset_id/component_name/document_id`
- manager 在执行 promote 前会规范化这三个字段：资产路径会收敛到 `res://...ui.toml`，组件名会收敛到合法的导出组件名，文档 id 会收敛到 dotted id；若目标文件已存在，则只对最终落盘的 asset/document id 自动追加数字后缀
- manager 会立即写出新的 widget `.ui.toml` 并触发 `AssetManager::import_asset(...)`
- 当前正在编辑的文档仍保持正常 dirty 生命周期，引用改写先停留在 session/source buffer，等待用户显式保存当前文档
- `undo_ui_asset_editor(...)` / `redo_ui_asset_editor(...)` 现在会在 session 结构化回放之外，再执行对应的 host-side external effect：undo promote 时删除生成的 widget 文件并触发 `reimport_all()` 清理资产目录状态，redo promote 时重写该文件并重新 `import_asset(...)`

## Pane Presentation And Slint Callbacks

`UiAssetEditorPane` 现在已经是 workbench 里的真实 pane，而不是 placeholder 文本。

当前 pane contract 已经从扁平 `PaneData.ui_asset_*` 收口成结构化 `UiAssetEditorPaneData`：

- `workbench.slint::PaneData` 只保留 `ui_asset: UiAssetEditorPaneData`
- `UiAssetEditorPane` 只接收 `pane: root.pane.ui_asset`，不再由宿主逐项转发几十个 `ui_asset_*` 字段
- 选择与列表状态统一进入 `UiAssetStringSelectionData`
- source / preview mock / theme / style / matched-rule / declaration / token / inspector / palette drag 都各自下沉到独立 detail DTO：
  - `UiAssetSourceDetailData`
  - `UiAssetPreviewMockData`
  - `UiAssetThemeSourceData`
  - `UiAssetStyleRuleData`
  - `UiAssetMatchedStyleRuleData`
  - `UiAssetStyleRuleDeclarationData`
  - `UiAssetStyleTokenData`
  - `UiAssetInspectorWidgetData`
  - `UiAssetInspectorSlotData`
  - `UiAssetInspectorLayoutData`
  - `UiAssetInspectorBindingData`
  - `UiAssetPaletteDragData`
- `panes.slint` 里的本地属性现在从 `root.pane.*` 和 `root.pane.<detail>.*` 做投影；palette target preview / candidate / sticky chooser 也统一从 `root.pane.palette_drag.*` 读取，而不是保留额外 host 绑定

这对应方案里的六区基础骨架：

- Palette
- Hierarchy
- Designer/Preview 区
- Inspector
- Stylesheet 区
- Source 区

当前 `panes.slint` 与 `workbench.slint` 已经收口到更稳定的 callback 边界：

- `ui_asset_action(instance_id, action_id)`
- `ui_asset_style_class_action(instance_id, action_id, class_name)`
- `ui_asset_detail_event(instance_id, detail_id, action_id, item_index, primary, secondary)`
- `ui_asset_collection_event(instance_id, collection_id, event_kind, item_index)`
- `ui_asset_source_edited(instance_id, value)`
- `ui_asset_source_cursor_changed(instance_id, byte_offset)`
- `ui_asset_palette_drag_hover(instance_id, surface_x, surface_y)`
- `ui_asset_palette_drag_drop(instance_id)`
- `ui_asset_palette_drag_cancel(instance_id)`
- `ui_asset_palette_target_confirm(instance_id)`
- `ui_asset_palette_target_cancel(instance_id)`

其中：

- `detail_id` 当前覆盖 `inspector_widget`、`style_rule`、`style_rule_declaration`、`style_token`、`preview_mock`、`binding_payload`
- `collection_id` 当前覆盖 `palette`、`hierarchy`、`preview`、`source_outline`、`preview_mock`、`matched_style_rule`、`slot_semantic`、`layout_semantic`、`binding`、`binding_event`、`binding_action_kind`、`binding_payload`、`palette_target_candidate`
- 这意味着 host 不再保留 `ui_asset_inspector_widget_action(...)`、`ui_asset_style_rule_action(...)`、`ui_asset_style_token_action(...)`、`ui_asset_style_rule_declaration_action(...)`、`ui_asset_preview_mock_action(...)`、`ui_asset_binding_payload_action(...)` 这类逐面 ABI

宿主侧 `app/ui_asset_editor.rs` 目前支持的动作集合是：

- `save`
- `undo`
- `redo`
- `reference.open`
- `canvas.convert.reference`
- `canvas.extract.component`
- `canvas.promote.widget`
- `preview.preset.editor_docked`
- `preview.preset.editor_floating`
- `preview.preset.game_hud`
- `preview.preset.dialog`
- `palette.target.previous`
- `palette.target.next`
- `mode.design`
- `mode.split`
- `mode.source`
- `mode.preview`
- `style.rule.create`
- `style.rule.extract_inline`
- `style.state.hover`
- `style.state.focus`
- `style.state.pressed`
- `style.state.disabled`
- `style.state.selected`
- `style.class.add`
- `style.class.remove`

额外的 stylesheet rule 编辑现在通过 `ui_asset_detail_event(..., "style_rule", ...)` 进入宿主：

- `style.rule.select`
- `style.rule.rename`
- `style.rule.delete`

token 编辑也通过 `ui_asset_detail_event(..., "style_token", ...)` 进入宿主：

- `style.token.select`
- `style.token.upsert`
- `style.token.delete`

rule declaration 编辑也通过 `ui_asset_detail_event(..., "style_rule_declaration", ...)` 进入宿主：

- `style.rule.declaration.select`
- `style.rule.declaration.upsert`
- `style.rule.declaration.delete`

结构化 Inspector 编辑当前复用同一条 detail-event 宿主 callback：

- `ui_asset_detail_event(instance_id, "inspector_widget", action_id, item_index, primary, secondary)`
- `widget.control_id.set`
- `widget.text.set`
- `slot.mount.set`
- `slot.padding.set`
- `slot.layout.width.preferred.set`
- `slot.layout.height.preferred.set`
- `layout.width.preferred.set`
- `layout.height.preferred.set`
- `binding.add`
- `binding.delete`
- `binding.id.set`
- `binding.event.set`
- `binding.route.set`

这样 Slint 不需要再为 slot/layout/binding/theme/style/payload 各自引入私有 ABI；宿主只根据 `detail_id + action_id` 或 `collection_id + event_kind` 路由到 manager/session 的字段编辑 API。

`Source` 区已经使用 multiline `TextEdit`，不是单行输入框。当前 Source 面板还新增了 `Source Outline`，会按当前 source buffer 中实际存在的 `[nodes.*]` block 和 line number 投影 outline 列表；Hierarchy、preview 列表、preview canvas 和 source outline 都已经汇合到同一条 session 选中链路。

这一轮补上的 palette target-cycle / picker 闭环是：

- `UiAssetEditorPanePresentation` / `PaneData` / `UiAssetEditorPane` 已经把 `palette_drag_candidate_items` 和 `palette_drag_candidate_selected_index` 显式投影到 Slint pane
- `panes.slint` 在 palette 外部拖拽 overlay 里新增了 `Target Cycle` 面板；当候选数大于 1 时，会显示当前可落点列表，而不是只显示一个几何默认 label
- overlay 内与 sticky chooser 内的 `FocusScope` 都已接入键盘轮换：`Left/Up` 选上一个，`Right/Down/Tab` 选下一个，`Enter` 应用当前 candidate，`Escape` 取消 palette drag 或 sticky chooser
- `drop_selected_palette_item_at_palette_drag_target()` 现在不再盲目依赖 hover 默认值，而是会读取当前 selected candidate 对应的 `UiAssetPaletteInsertPlan`
- 低语义 slot 进入 sticky chooser 后，drag pointer 已结束也能继续保留 `Target Cycle` 面板；这条路径专门覆盖 overlay 太小、区域重叠或需要键盘轮换目标的场景

这一轮补上的 preview workflow 闭环是：

- `UiAssetPreviewPreset` 已经进入 `UiAssetEditorRoute`，作为 editor-only 宿主状态参与序列化/恢复
- `UiAssetEditorSession::set_preview_preset(...)` 会重建 shared preview surface，但不会污染 source dirty 或触发 canonical source 变更
- `EditorManager::set_ui_asset_editor_preview_preset(...)` 已经把这条行为接入正式会话管理链路
- `UiAssetEditorPanePresentation` / `PaneData` / `UiAssetEditorPane` 已经投影当前 preset，并在工具条里提供 `Docked` / `Float` / `HUD` / `Dialog` 四个切换按钮
- 预设切换后 preview summary 会显示新的 surface 约束尺寸，例如 `1920x1080` 的 `Game HUD` 或 `640x480` 的 `Dialog`

这一轮补上的 mock preview workflow 闭环是：

- `preview_mock.rs` 已经成为独立 session-only 模块，按 `node_id + prop key` 保存临时 override，不进入 route 序列化，也不会标记 source dirty
- session 会从当前选中节点的 `props` 里筛出可 mock 的浅量字段，并按 `Text/Bool/Enum/Resource` 分类投影到 pane
- `set_selected_preview_mock_value(...)` / `clear_selected_preview_mock_value(...)` 会对 `last_valid_document` 的克隆应用 override，再重新编译 shared preview surface；source 非法时仍可继续基于 last-good document 调整 preview
- `UiAssetEditorPane` 现在已经提供 `Mock Preview` 列表、当前 prop/kind 标签、value 输入框，以及 `Apply` / `Clear` 操作

这一轮补上的资源复用导航闭环是：

- `UiAssetEditorSession::selected_reference_asset_id()` 会从当前选中的 `reference` 节点提取 source asset id，并剥离 `#ComponentName`
- `UiAssetEditorPanePresentation` / `PaneData` 新增 `can_open_reference`，Slint pane 在 tree authoring 工具条里暴露 `Open Ref`
- `UiAssetEditorPanePresentation` / `PaneData` 现在还会投影 `can_convert_to_reference`，Slint pane 在同一组 tree authoring 工具条里新增 `To Ref`
- `reference.open` 通过 `SlintEditorHost -> EditorManager::open_ui_asset_editor_selected_reference(...) -> open_ui_asset_editor_by_id(...)` 打开被引用 widget 资产
- `canvas.convert.reference` 通过 `SlintEditorHost -> EditorManager::convert_ui_asset_editor_selected_node_to_reference(...) -> UiAssetEditorSession::convert_selected_node_to_reference(...)` 把当前节点就地转换成 Palette 当前选中的 imported widget reference，并保持选中与 undo/redo 一致
- hierarchy 列表现在也支持 `double-clicked` 激活：宿主会先同步选中，再直接尝试打开当前 reference 节点的源 widget 资产
- preview 列表现在也支持 `double-clicked` 激活：宿主会先同步 preview 选中，再复用同一条 `Open Reference` 链路打开源 widget 资产
- 这让 V1 已经具备“按钮打开 + hierarchy 双击打开 + preview 双击打开”的正式工作流；真正的 canvas frame overlay 手势仍待补齐

为了让这条链路在当前 dirty workspace 下保持可验证，workbench host helper 还顺手把 `layout_hosts` 与 `builtin_views` 的同名函数导入改成显式 module-path 访问，避免同名私有 module/re-export 冲突把 `zircon_editor` 编译入口打断；这不改变产品行为，只恢复宿主编译可达性。

Stylesheet 区现在不再只是只读摘要：

- 顶部有 `Rule` / `Extract` 动作按钮
- 下方有 `Hover` / `Focus` / `Pressed` / `Disabled` / `Selected` 伪状态切换按钮
- 选中节点的 class 列表会投影到 `style_class_items`，并提供 `class-name` 输入框加 `Add` / `Remove` 动作
- 本地 rule 列表会投影到 `style_rule_items`，并附带 `style_rule_selected_index`、`style_selected_rule_selector`、`style_can_edit_rule`、`style_can_delete_rule`
- rule editor 现在提供 `Rules` 可选列表、selector 输入框，以及 `Apply` / `Delete` 动作
- matched rule 链现在会投影到 `style_matched_rule_items`，并附带 `style_matched_rule_selected_index`、`style_selected_matched_rule_origin`、`style_selected_matched_rule_selector`、`style_selected_matched_rule_specificity`、`style_selected_matched_rule_source_order`、`style_selected_matched_rule_declaration_items`
- matched rule inspector 现在提供 `Matched Rules` 可选列表，并展示 origin、selector、specificity/source order 与 declaration 明细
- 选中 local rule 后会继续投影 `style_rule_declaration_items`，并附带 `style_rule_declaration_selected_index`、`style_selected_rule_declaration_path`、`style_selected_rule_declaration_value`、`style_can_edit_rule_declaration`、`style_can_delete_rule_declaration`
- declaration editor 现在提供 `Declarations` 可选列表、`self.background.color` 这种路径输入框、value 输入框，以及 `Apply` / `Delete` 动作
- 本地 token 列表现在会投影到 `style_token_items`，并附带 `style_token_selected_index`、`style_selected_token_name`、`style_selected_token_value`、`style_can_edit_token`、`style_can_delete_token`
- token editor 现在提供 `Tokens` 可选列表、`token-name` / `token-value` 输入框，以及 `Apply` / `Delete` 动作
- token 编辑对裸字符串字面量已经补了容错回退：像 `#223344` 这种不能直接作为 TOML literal 解析的输入，会自动按 string token 落盘，而不是把整个 token 编辑链打成错误状态
- pane projection 会把 rule availability 和当前伪状态活跃标志显式投影到 Slint `PaneData`
- declaration path 解析支持 `self.*` 和 `slot.*`，并会把嵌套 TOML table flatten 成可编辑的 dotted path；删除 leaf 后会自动回收空 table，保持 canonical source 简洁

## Theme Source Authoring

Theme 面板现在已经从“只读来源摘要”推进到第一条真正可执行的 theme authoring 工作流：

- `theme_source_items` 会同时列出 local theme 和 imported style source，并继续投影 `selected_theme_source_reference/kind/token_count/rule_count/available`
- 选中 imported source 时，面板允许直接 `Open` 对应 style asset；选中 local source 时，面板会把 token/rule 列表继续联动到现有本地 token/rule editor，而不是另起一套私有编辑模型
- local theme 的提升流程现在还多了一层 draft：`theme_promote_asset_id`、`theme_promote_document_id`、`theme_promote_display_name`
- 这三个字段由 session 先按当前 layout/widget asset id 和 display name 自动生成默认值，然后允许用户在 promote 之前显式改写
- Slint 仍然不直接决定任何文件系统行为；draft 字段通过既有 `ui_asset_inspector_widget_action(...)` callback 上传 action id/value，由 manager/session 更新 draft，再由 host-side `resolve_external_style_target(...)` 在落盘前处理路径冲突和 suffix 去重
- 因此 `Promote Local Theme` 现在和 `Promote To External Widget Asset` 一样，具备“默认建议值 + 可视化覆盖 + host-side 唯一路径解析 + canonical TOML 落盘”的一致语义，而不是固定把本地 theme 导出到单一路径

## Preview Projection

`UiAssetPreviewHost` 当前直接消费 `UiCompiledDocument`：

- 通过 `UiTemplateSurfaceBuilder::build_surface_from_compiled_document(...)` 构建 shared `UiSurface`
- 立刻执行 `compute_layout(preview_size)`
- `preview_projection.rs` 会把 shared `render_extract` 和 `template_metadata` 投影成 `preview_items`、`preview_canvas_items`、`preview_surface_width`、`preview_surface_height`
- `preview_selected_index` 会同时驱动 preview 列表、canvas frame 高亮和 session 选中节点映射，不再由不同 pane 各自维护一套 index

为了让 reference widget 场景可读，preview 列表不再只显示最终渲染出来的底层 `Button` 类型：

- 如果当前 preview node 对应文档里的 `component_ref = "...#ToolbarButton"`，列表会优先显示 `ToolbarButton`
- 如果底层实际渲染组件类型和文档组件身份不同，会显示 `ToolbarButton/Button` 这类组合标签

当前 `Designer Canvas` 也已经消费这份投影：

- 画布会按 shared layout frame 渲染可点击的 frame block，而不是 editor-only 假列表
- 选中 frame 后会在画布里弹出一组 selected-frame overlay 快捷动作，直接复用既有 `palette.insert.*`、`canvas.move.*`、`canvas.reparent.*`、`canvas.wrap.vertical_box`、`canvas.unwrap`、`reference.open`、`canvas.convert.reference`、`canvas.extract.component`、`canvas.promote.widget`
- 选中 frame 周围现在还会出现 contextual target pad：`Insert In`、`Insert After`、`Into Prev`、`Into Next`、`Outdent`，这些 target 直接贴着当前 frame 呈现，比下方工具按钮更接近真正 designer authoring 的落点语义
- 这些 canvas target 和 overlay button 现在会先经过 session 内部的 dry-run legality 检查，再把 `can_*` 布尔量投影给 Slint；因此非法的插入、重排、reparent、wrap/unwrap 动作会直接在 UI 上禁用，而不是点下去才 no-op
- child insert 现在也不再把 palette 节点盲插到任何 widget 下面；至少要求当前目标是宿主已知可承载 children 的容器型 widget，这和后续 slot/drop contract 收紧的方向保持一致
- canvas 内部现在还新增了最小 drag authoring 手势：拖动当前 frame 时会在 Slint 本地解析 pointer 是否落进这些 contextual target，并在 release 后复用同一条 `action_requested(...)` 路由触发树编辑，不需要为此额外增加宿主 ABI
- 这些 overlay 动作不会绕过 session/manager；它们仍然走与工具条按钮相同的宿主 action 路由，所以 undo/redo、selection reconcile 和 source roundtrip 语义保持一致
- 进一步地，Palette 拖拽现在不再只能对“当前选中的 frame”做上下文插入；Slint 会把 pointer 反投影回 preview surface 坐标，由宿主/session 按 hovered preview frame 计算真实 palette drop target，并把 hover 高亮与 `Insert In` / `Insert After` 文案再投影回 canvas
- 对有结构化 slot 语义的目标，hover 之外的候选落点现在也会一起投影回 canvas；这让 palette 外部拖拽第一次具备“所见即所得”的 slot region authoring，而不是只看一条 label

这让 preview 列表和 preview canvas 都能保持对 shared surface 的真实渲染节点可追踪，同时开始具备第一批真正的可视 authoring affordance。

## Current V1 Limits

当前实现完成了“宿主闭环”，并且已经有第一批结构化 Inspector 编辑，但还没有完成整个产品计划。

已经从“未落地”列表中移除、且目前真实可用的能力包括：

- Palette 选择后直接插入 native node 或 imported widget reference node
- 选中 imported widget palette entry 后可把当前节点 `Convert To Reference`
- `Wrap VBox` / `Unwrap`
- sibling move 与 reparent `into previous / into next / outdent`
- preview canvas frame 点击选中与列表/Hierarchy/Source Outline 同步
- preview canvas selected-frame overlay 快捷动作，可直接触发插入、重排、reparent、wrap/unwrap、open reference、convert/extract/promote
- preview canvas contextual insert/reparent target pad，可直接在 frame 周边执行 `Insert In`、`Insert After`、`Into Prev`、`Into Next`、`Outdent`
- canvas authoring 操作现在已经带 legality gating；例如非容器节点不会再显示可用的 `Insert In`，顶层节点不会错误暴露 `Outdent`，无父节点的 root 也不会再显示可用的 `Wrap`
- selected-frame drag gesture 已经可以直接落到现有 contextual target 上触发插入/reparent authoring，不再只能点击 overlay 快捷按钮
- palette drop 现在已具备真正的 slot placement synthesis：`Overlay` 会自动推导 `anchor/pivot/position`，`GridBox` 会自动推导 `row/column`，`FlowBox` 会自动推导 `break_before/alignment`
- palette drop 目标已经扩展到本地 `component` mount 和 external widget named slot，不再只限共享原生容器
- palette drop 现在还会把这些 slot synthesis 结果可视化成显式 overlay：`GridBox` 的 per-cell、`Overlay` 的 per-anchor、`FlowBox` 的 per-line/alignment，以及 `header/body`、`leading/trailing` 这类 named-slot region；对 low-semantic slot，真正的落点解析也已经先走这些 overlay 几何命中，而不是只看语义
- 多命名 slot / 低语义 slot 现在还额外有独立的 `target cycle` 第二通道：当 hover 默认命中不够稳定时，用户可以在 drag overlay 里看见候选列表，并用键盘轮换当前 target
- sticky/manual chooser 现在已经是 drop 后仍可停留的工作流，而不只是 drag-time 瞬时 picker；这让“先松手再精确改落点”的 slot authoring 真正可用
- 结构化 bindings inspector
- parent-specific slot/layout semantic inspector，已经覆盖 `Overlay/Grid/Flow/ScrollableBox` 与线性容器 `gap`
- `Open Ref` 打开外部 widget 引用源资源，以及 hierarchy 双击激活该导航
- preview 双击激活 `Open Ref`
- preview root surface presets：`Editor Docked`、`Editor Floating`、`Game HUD`、`Dialog`
- editor-only mock preview：文本/布尔/枚举/资源引用四类浅量 `props` override
- structured binding inspector 已经扩到 `event` 枚举选择、`route/action` 目标切换、`action kind` 切换与 `payload key/value` 编辑，不再只是字符串 route 输入
- Theme panel 现在已经具备 imported theme `Detach` / `Clone` 动作、跨 asset token/rule cascade inspector，以及 `Merge Preview` 列表；在真正落盘前可以直接看到 detach / clone 两条路径将如何改写本地 `imports`、token rename 和 stylesheet/rule 顺序
- Theme panel 现在还会额外投影 `theme_compare_items`，把 imported/local 两侧 token/rule 的 `shared` / `inherited` / `local-only` / `shadowed` / `overrides imported` 摘要和 cascade inspector 的 `active` / `shadowed` 链配对展示，便于真正做 theme diff / refactor 前先看清语义差异
- 这轮还顺手修复了两条真实宿主 Slint 漂移：`workbench/host_scaffold.slint` 重新显式导出 `WorkbenchHostScaffold`，`UiAssetCanvasSurface` 的快捷按钮状态不再错误引用不存在的 `root.pane.*`

与此同步，editor template runtime 的消费边界也开始向同一份资产合同收敛：

- `EditorTemplateRegistry` 现在既能注册 legacy `zircon_ui::template::UiTemplateDocument`，也能注册正式 `UiAssetDocument`
- asset 文档会先编译成 `UiCompiledDocument`，再在实例化时回收到同一种 `UiTemplateInstance`
- `EditorUiHostRuntime::register_document_source(...)` / `register_document_file(...)` 现在按“asset 优先、legacy 回退”处理输入源码，所以后续把 builtin shell 源逐步切到 `.ui.toml` 时，不需要再改 session/pane/projection 这条高层宿主链

仍未落地的高层 authoring 能力包括：

- stylesheet selector / declaration block 的更高层结构化编辑体验；当前 theme 侧已经有 imported detach/clone、local merge preview、跨 asset cascade/compare、批量 theme refactor，以及按 token/rule 逐条采纳 imported theme body 的 helper，但还缺更强的 compare-cascade 联动、theme-wide multi-rule helper 和更可视化的 rule-body authoring UX
- preview mock 目前已扩到 number / collection / object / expression、轻量函数求值和跨节点 state graph 摘要；binding payload 这一侧也已经开始支持基于当前选中 object/collection 的相对结构化编辑和 collection append，但还缺更完整的 schema-aware collection/object editor 和跨节点交互联动
- 更细粒度的 tree-command undo/redo 执行后端；当前 `UiAssetDocumentDiff` 已经具备 node/component + child-mount-list 级 patch，且 promote 外部文件副作用也已进入第一批 compensation flow，但 stylesheet rule 向量 diff、更多树命令的真 inverse execution 与跨文件复合副作用日志仍未落地
- 更大范围的 runtime/editor screen/window 迁移还没完成；当前闭环证明的是 asset/editor host 主链可行，不代表现有 runtime HUD、editor ActivityWindow 已全部切到这条路径
- `UI Asset Editor` 自身也还没有完全自举；当前仍有一部分 workbench/window chrome 逻辑停留在现有宿主壳层，而不是由 `.ui.toml` UI 资产完全描述

所以当前阶段应把它视为“正式可打开/编辑/保存/预览 UI 资产的宿主骨架”，而不是已经具备完整 Widget Blueprint 级 authoring 体验。

## Validation Evidence

此前主链的宿主/资产闭环已经做过宽验证，至少包括：

2026-04-18 这一轮继续补的证据，主要针对“test-only 编译漂移是否已经收回、以及 `ui_asset_sessions` 结构测试是否重新稳定”为准：

- `cargo test -p zircon_editor --lib editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo -- --nocapture`
- `cargo test -p zircon_editor --lib editor_manager_promotes_selected_ui_asset_component_to_external_widget_asset -- --nocapture`
- `cargo test -p zircon_editor --lib editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors -- --nocapture`

这三条回归在当前目录树和 host/session 切分下都重新回到绿色，说明本轮 `ui_asset_sessions` 子树重排没有再次引入 test-only 编译断裂。

- `cargo test -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked`
- `cargo test -p zircon_editor --lib --locked`
- `cargo test -p zircon_asset --lib --locked`
- `cargo build --workspace --locked --verbose`
- `cargo test --workspace --locked --verbose`

本轮 selected-frame canvas overlay 跟进的新增证据是：

- `cargo test -p zircon_ui ui_legacy_template_adapter_emits_canonical_asset_source_that_roundtrips -- --nocapture`
- `cargo test -p zircon_editor editor_template_registry_instantiates_registered_asset_documents -- --nocapture`
- `cargo test -p zircon_editor --lib editor_ui_host_runtime_projects_asset_document_source_into_slint_projection -- --nocapture`
- `cargo test -p zircon_editor --lib generated_legacy_template_asset_source_opens_in_ui_asset_editor_session -- --nocapture`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_projects_canvas_insert_and_wrap_availability`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_projects_canvas_move_and_reparent_availability`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_canvas_declares_selected_frame_authoring_overlay_controls -- --exact`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_canvas_declares_contextual_insert_and_reparent_targets -- --exact`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_canvas_declares_drag_authoring_state_and_drop_resolution -- --exact`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked menu_popups_anchor_to_actual_menu_buttons_instead_of_hardcoded_offsets -- --exact`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked`
- `cargo check -p zircon_editor --lib --locked`

本轮 target-cycle / picker 与 child-window projection 修复的新增证据是：

- `cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_exposes_palette_drag_target_cycle_candidates_for_low_semantic_slots`
- `cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_drop_uses_cycled_palette_drag_target_candidate`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked --offline ui_asset_editor_pane_declares_palette_target_cycle_panel_and_keyboard_controls`
- `cargo test -p zircon_editor --lib --locked --offline host::slint_host::app::tests::floating_window_projection::child_window_host_recompute_caches_floating_window_projection_bundle_for_detached_window`
- `cargo test -p zircon_editor --lib --locked --offline`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked --offline`

这表示当前 dirty workspace 下，至少 `zircon_editor` 的 Slint pane 壳层和 crate-local lib check 都已经覆盖到这次改动，没有再出现之前那种由外部并行重构导致的 `zircon_editor` 自身编译入口被打断的情况。

本轮没有重新跑 workspace-wide `cargo build/test --workspace --locked --verbose`，原因不是 `ui_asset` 逻辑本身阻塞，而是这条 slice 只修改了 `zircon_editor` 的 Slint pane 文本与对应 shell 回归测试；当前最有价值的验证面是 pane 壳层回归和 `zircon_editor` crate check。

本轮 palette-hover drop target slice 新增了两条目标验证：

- `cargo test -p zircon_editor --lib --locked palette_drag_drop_to_hovered_preview_node`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_palette_drag_creation_flow -- --exact`

2026-04-17 这条下层阻塞已经被收敛并恢复成可编译状态；本地用独立 `CARGO_TARGET_DIR=target/codex-ui-asset-editor` 跑过：

- `cargo check -p zircon_graphics --lib --locked`

对应修复点包括：

- `RuntimePreviewRenderer` 对 `wgpu 29` 的 `CurrentSurfaceTexture` surface acquire 语义迁移
- pipeline/render-pass descriptor 对 `immediate_size`、`bind_group_layouts: &[Option<&...>]`、`multiview_mask`、`DepthStencilState` 新字段形态的兼容
- `winit 0.31` 的 `Arc<dyn Window>` + `surface_size()` trait-object 适配

本轮 slot placement synthesis / named-slot palette targeting / graphics unblock slice 用独立 `CARGO_TARGET_DIR=target/codex-ui-asset-editor` 串行跑过的验证面包括：

- `cargo check -p zircon_graphics --lib --locked`
- `cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_projects_slot_aware_palette_drag_target_labels`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked --offline ui_asset_editor_pane_declares_slot_aware_external_palette_drop_overlays`
- `cargo test -p zircon_editor --lib --locked --offline palette_drag_drop_to_hovered_preview_node`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked --offline ui_asset_editor_pane_declares_palette_drag_creation_flow`
- `cargo test -p zircon_editor --lib --locked --offline ui_asset_palette_drop::`
- `cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_projects_explicit_ -- --nocapture`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked --offline ui_asset_editor_pane_declares_explicit_palette_slot_target_overlay_projection -- --exact --nocapture`

这些命令说明：

- `zircon_graphics` 的 `wgpu 29` / `winit 0.31` 兼容性已经恢复到可编译状态
- session 层的 slot placement synthesis 是可执行的，而不只是文档/源码约定
- Slint canvas 已经声明并消费 `drop_inside_overlay` / `drop_after_overlay`，并且对结构化 slot target 继续消费 `UiAssetCanvasSlotTargetData` 的显式 overlay 列表
- `ui_asset_palette_drop::` 已经直接覆盖 `Overlay anchor/pivot/position`、`Grid row/column`、`Flow break/alignment`、local component mount 和 external widget named slot 五条回归路径

随后继续把证据面扩大到更完整的 `zircon_editor` 宿主回归，并把中途暴露的 shared host/template drift 全部收敛掉：

- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline host::slint_host::app::tests::root_activity_rail_pointer_click_prefers_shared_projection_surface_when_left_region_geometry_is_stale -- --exact --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline tests::host::slint_tab_drag::resolve_workbench_tab_drop_route_uses_shared_root_projection_tab_strip_when_drawers_are_collapsed -- --exact --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline tests::host::template_runtime -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline low_semantic_component_mounts -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline`
  - 当前结果：`404 passed; 0 failed`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --test workbench_slint_shell --locked --offline`
  - 当前结果：`41 passed; 0 failed`

本轮 sticky/manual chooser、structured binding inspector、source-diff undo backend 与 canonical resource boundary 收口后的新增证据是：

- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --test workbench_slint_shell --locked --offline`
  - 当前结果：`44 passed; 0 failed`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline`
  - 当前结果：`423 passed; 0 failed`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_asset --locked --offline`
  - 当前结果：`44 passed; 0 failed`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor-manager cargo test -p zircon_manager --locked --offline`
  - 当前结果：`8 passed; 0 failed`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor-resource cargo test -p zircon_resource --locked --offline`
  - 当前结果：`11 passed; 0 failed`

这一轮新增收口的真实行为包括：

- `binding.route.set` 重新作为通用 binding target 编辑入口暴露到 Slint pane，同时保留 `binding.route_target.set` / `binding.action_target.set` 的结构化字段
- source roundtrip undo/redo 不再依赖整串 source snapshot 替换，而是执行 char-boundary 安全的 source replace-diff，再与 `UiAssetDocumentDiff` 一起回放
- `zircon_editor` 的 resource access 边界与 `zircon_asset` pipeline records 继续收敛到 canonical `zircon_resource::{ResourceRecord, ResourceState}`
- `zircon_resource::ResourceLocator` 现在具备稳定排序能力，保证 `zircon_asset` 在扩回更大面 cargo 验证时不会因为 locator 排序漂移卡住更高层编辑器回归

在本轮把 low-semantic manual slot targeting 和 host/template baseline 一起收口后，又重新扩大验证面：

- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_projects_explicit_ -- --nocapture`
  - 当前结果：`3 passed; 0 failed`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --test workbench_slint_shell --locked --offline ui_asset_editor_pane_declares_explicit_palette_slot_target_overlay_projection -- --exact --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_uses_explicit_slot_overlay_regions_for_low_semantic_component_mounts -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --test workbench_slint_shell --locked --offline`
  - 当前结果：`41 passed; 0 failed`
- `CARGO_TARGET_DIR=target/codex-ui-asset-editor cargo test -p zircon_editor --lib --locked --offline`
  - 当前结果：`404 passed; 0 failed`

这一轮收敛掉的真实宿主/模板问题包括：

- `activity_rail` bridge 现在会优先尝试 strip-local 坐标，再在必要时接受投影后的全局点位，避免 shared pointer surface 与宿主回调输入空间漂移时直接漏点
- `root_shell_projection` 在 visible drawer / collapsed drawer 的 fallback 上，不再错误依赖完整 `shell_frame`；只要 shared `workbench_body_frame` 还在，就能继续合成 `document/right/bottom/activity rail` 等根区域 frame
- `template_runtime` 的快照断言已经跟上当前 builtin template 真实结构：
  - `WorkbenchShell` 现在包含 `WorkbenchScaffold + Left/Right/Bottom Drawer Overlay`
  - `SceneViewportToolbar` 现在按 `LeftGroup/RightGroup` 分组，而不是把所有 control 直接挂在 root 下
- `native_window_hosts` 现在已经沿着 `host_lifecycle -> pointer_layout -> document_tab_pointer / shell_pointer / ui projection` 统一传递，child-window floating projection 与 template-runtime 测试不再各自维护漂移签名
- `SlintEditorHost::new_for_test(...)` 在 test build 下增加了 startup-state fallback：如果启动会话要求 project mode，但 scene service `SceneModule.Manager.DefaultLevelManager` 在当前 test fixture 中不可用，就先回退到 welcome state；这让并行 `zircon_editor --lib` 不再因为 test-only fixture 依赖而偶发失败

因此，这份文档前面记录的 palette drop / slot synthesis / named-slot targeting / source roundtrip / canvas authoring 行为，当前已经不只是 crate-local 新功能通过，而是已经和 `zircon_editor` 的 shared host projection、template runtime、real host app regression 一起恢复到更大面的全绿基线。

本轮继续把 `HorizontalBox` / `VerticalBox` 的线性 `gap` 宿主链路与 token 编辑边角 case 一起收口后，又补了一轮更扎实的验证：

- 使用独立 `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-diff CARGO_INCREMENTAL=0`
- `cargo check -p zircon_editor --lib --locked --offline`
- `cargo test -p zircon_editor --lib --locked --offline layout_semantic_action_path_maps_linear_box_gap_action`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked --offline ui_asset_editor_pane_declares_typed_parent_specific_slot_layout_and_binding_fields -- --exact`
- `cargo test -p zircon_editor --lib --locked --offline sticky_palette_target_chooser`
  - 当前结果：`4 passed; 0 failed`
- `cargo test -p zircon_editor --lib --locked --offline parent_specific_slot_and_layout_semantics`
  - 当前结果：`2 passed; 0 failed`
- `cargo test -p zircon_editor --lib --locked --offline upserts_and_deletes_local_tokens -- --nocapture`
- `cargo test -p zircon_editor --lib --locked --offline editor_manager_runs_ui_asset_style_token_editing_actions -- --nocapture`
- `cargo test -p zircon_editor --lib --locked --offline -- --test-threads=1`
  - 当前结果：`430 passed; 0 failed`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked --offline`
  - 当前结果：`44 passed; 0 failed`

这一轮对应的真实收口点包括：

- `layout.box.gap.set` 已经贯通 `UiAssetEditorPane -> pane projection -> host action dispatch -> session/document`
- `ui_asset_sessions` 的宿主子模块拆分漂移已经稳定下来，不再让更大面的 `zircon_editor --lib` 回归在 host/session 边界上先崩
- token 编辑对裸字符串字面量现在会自动回退到 `Value::String(...)`，恢复了 UI 主题 token 编辑里 `#223344` 这类常见值的稳定输入路径
- `UiAssetDocumentDiff` 的新增字段级回放测试已经覆盖“node/component 更新时保留无关字段”和“children 变化时保留无关 child mount”的场景，undo document replay 不再默认依赖整节点替换
- `Promote To External Widget` 已经进入第一批宿主 compensation flow，undo 时会清理生成的外部 widget 文件，redo 时会按记录的 canonical source 重建它
- 默认并行 `cargo test -p zircon_editor --lib` 在当前环境里仍可能因为共享锁/环境噪声产生误导性失败，因此当前可靠证据以单线程全量 lib 结果为准

本轮 source-panel cursor roundtrip 与更大面 crate 回归补充的最新证据是：

- 使用独立 `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-zircon-editor-lib-roundtrip-red CARGO_INCREMENTAL=0`
- `cargo test -p zircon_editor --lib --offline ui_asset_editor_session_ -- --nocapture`
  - 当前结果：`71 passed; 0 failed`
- `cargo test -p zircon_editor --lib --offline editor_manager_selects_ui_asset_nodes_from_source_byte_offsets -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `cargo test -p zircon_editor --lib --offline -- --nocapture`
  - 当前结果：`453 passed; 0 failed`
- `cargo test -p zircon_editor --test workbench_slint_shell --offline -- --nocapture`
  - 当前结果：`46 passed; 0 failed`
- `cargo test -p zircon_editor --lib --offline -- --nocapture`
  - 当前结果：`29 passed; 0 failed`
- `cargo test -p zircon_asset --lib --offline -- --nocapture`
  - 当前结果：`47 passed; 0 failed`

本轮 theme promote draft authoring 的新增证据是：

- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-theme-details CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline theme_ -- --nocapture`
  - 当前结果：`7 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-theme-details CARGO_INCREMENTAL=0 cargo test -p zircon_editor --test workbench_slint_ui_asset_theme_shell --locked --offline -- --nocapture`
  - 当前结果：`2 passed; 0 failed`

本轮继续把 local theme merge preview 和相邻 Slint 宿主漂移一起收口后的新增证据是：

- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-theme-details CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_projects_local_theme_layer_merge_preview_for_imported_source -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-theme-details CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline theme_ -- --nocapture`
  - 当前结果：`12 passed; 0 failed`
- `cargo test -p zircon_editor --test workbench_slint_ui_asset_theme_shell --locked --offline -- --nocapture`
  - 当前结果：`6 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-theme-details CARGO_INCREMENTAL=0 cargo check -p zircon_editor --lib --locked --offline`
  - 当前结果：`finished dev profile without errors`

这一轮额外确认的真实行为包括：

- source-panel cursor 投影已经不再停留在 Rust session/pane 数据模型；Slint source editor 可以实际接收宿主回推的 cursor byte offset
- invalid-source fallback 现在明确采用“last-valid preview/selection + current-source cursor remap”的语义，测试已经覆盖 block 内行偏移在 invalid edit 后的保持
- `zircon_editor::ui` 的 repository template 基线也重新对齐到了当前真实宿主壳：`workbench_shell.toml` 的根实例现在是 `UiHostWindow`，不再是旧测试中假定的 `WorkbenchShell`

本轮继续把 undo backend 从“source diff 可回放”推进到“command-log 有显式逆向语义”后的新增证据是：

- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-undo-red CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_tracks_explicit_inverse_tree_edits -- --nocapture`
  - 当前结果：`3 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-undo-red CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline stylesheet_diff_preserves_unrelated_existing_rules_when_one_rule_changes -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-undo-red CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_ -- --nocapture`
  - 当前结果：`100 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-undo-red CARGO_INCREMENTAL=0 cargo check -p zircon_editor --lib --locked --offline`
  - 当前结果：`finished dev profile without errors`

本轮继续往更可执行的 replay backend、richer preview/binding schema、theme tooling 和迁移资产推进后的新增证据是：

- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_evaluates_function_preview_expressions_and_binding_payload_previews -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_projects_cross_asset_theme_rule_cascade_activity -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_theme_refactor_uses_style_rule_vector_replay_commands -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_additional_editor_preview_state_lab_asset_compiles_and_opens -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_runtime_quest_log_dialog_asset_opens_as_shared_runtime_preview_session -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_ -- --nocapture`
  - 当前结果：`181 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline -- --test-threads=1`
  - 当前结果：`563 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next CARGO_INCREMENTAL=0 cargo test -p zircon_editor --test workbench_slint_shell --locked --offline -- --nocapture`
  - 当前结果：`55 passed; 0 failed`

本轮继续把 preview mock suggestion authoring 与 theme rule-body helper 补齐后的新增证据是：

- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next5 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_projects_preview_mock_suggestions_relative_to_selected_nested_container_and_applies_them -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next5 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline editor_manager_runs_ui_asset_preview_mock_suggestion_actions_relative_to_selected_container -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next5 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --test workbench_slint_shell --locked --offline ui_asset_editor_pane_declares_preview_mock_suggestion_controls_and_callback -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next5 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_adopts_imported_theme_rule_body_helper_items -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next5 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline editor_manager_applies_theme_rule_body_helper_items_for_imported_sources -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next5 CARGO_INCREMENTAL=0 cargo check -p zircon_editor --lib --locked --offline`
  - 当前结果：`finished dev profile without errors`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next5 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline theme_ -- --nocapture`
  - 当前结果：`29 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next5 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline -- --test-threads=1`
  - 当前结果：`573 passed; 0 failed`
- `TMP=D:\codex-temp TEMP=D:\codex-temp CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next5 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --test workbench_slint_ui_asset_theme_shell --locked --offline -- --nocapture`
  - 当前结果：`8 passed; 0 failed`

## Latest Extensions

这一轮补上的宿主闭环能力主要有五组：

- Preview mock / binding schema 现在不再只会解析简单 `=Node.prop` 引用。共享预览求值已经支持 `concat(...)`、`coalesce(...)`、`count(...)` / `len(...)`、`first(...)`、`last(...)`、`join(...)`、`eq(...)`、`if(...)` 这类轻量函数表达式，并且会继续把结果投影到 preview panel 的 expression result 与 binding schema preview。
- Binding payload editor 的宿主后端现在开始利用“当前选中 payload 容器”这层上下文：选中 object payload 后可以直接用相对 key 写入子字段，选中 collection payload 后可以用空 key 追加一项，而不必总是手写完整 dotted path；这一层能力已经进入 session + crate 回归。
- collection payload suggestion 的 append 位现在也做了稳定化：当当前选中 collection 还是空数组时，session 会按 `max(current_len, template_len)` 生成 append 候选，避免把模板索引和 append 索引都投成重复的 `[0]`。
- Theme cascade inspector 现在除了按层列出 imported/local 顺序，还会额外标出跨 asset rule 的 `active` / `shadowed` 状态，直接显示最终生效规则和被覆盖的上游规则体，便于 theme compare / refactor 前先看清 cascade 真相。
- Theme helper 不再只停留在 whole-theme `Detach` / `Clone`。选中 imported theme source 时，session 现在还会投影 `Adopt imported token ...` / `Adopt imported rule ...` 项，允许把 imported token 值和 rule body 逐条拉进本地 theme layer，而不必先整层 detach/clone 再手动删改。
- Theme helper 与 compare 现在也真正接上了：选中 imported theme source 后，helper 列表会额外给出 `Adopt compare diffs from selected theme (...)` 和 `Prune compare duplicates shared with selected theme (...)` 两类批量动作，直接把 compare state 转成多条 token/rule 的批处理 authoring 操作。
- Theme/document replay backend 继续从“整张 stylesheet 替换”向“可执行 rule 向量回放”推进。当 stylesheet id 稳定且 selector 唯一时，session 现在会优先合成 `InsertStyleRule` / `RemoveStyleRule` / `MoveStyleRule`，而不是把整张表退化成 `ReplaceStyleSheet`。
- 共享 UI asset 迁移样例又扩大了一步：新增了 editor 侧 [`preview_state_lab.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/preview_state_lab.ui.toml) 和 runtime 侧 [`quest_log_dialog.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/runtime/quest_log_dialog.ui.toml)，两者都已经能走同一条 `UiAssetEditorSession -> UiDocumentCompiler -> preview host` 链路打开与预览。
- Preview mock authoring 现在还额外补上了 schema-aware suggestion/apply 流：选中 object/collection 容器后，会按当前 nested container 投影 `[n] = ...` / `field = ...` 建议，并能直接把建议应用回对应 nested selection，而不是只能手敲结构化值。
- Preview mock nested tree 的显示也已经收口成两层语义：当当前 subject 就是层级选中的节点时，object/collection editor 继续展示简洁的相对 key（如 `enabled`、`[1]`）；当 authoring 目标来自跨节点 subject 或 schema-target 选择时，同一套 nested list 会自动切成限定路径（如 `StatusLabel.context.dialog.steps[1].label`），这样 Source/Hierarchy/Canvas 联动选择不会丢失深层定位能力。
- `UiAssetEditorUndoStack` 的 document replay 现在不再依赖“source text 回退才是最后真相”。session 会直接暴露最新 edit 的 redo replay/外部副作用预览；执行时先尝试 `Insert/Remove/Move*` 这类显式 document commands，再用 AST diff 计算权威目标态，仅在命令结果未完全覆盖目标文档时才回落到目标文档替换。这样 widget/theme promotion、stylesheet vector、cross-file asset source effect 都能保持可执行 replay，同时避免重复插入 stylesheet/rule。
- 这轮还顺手修复了一条更大面验证时暴露的 support-layer 编译漂移：[`zircon_runtime/src/scene/mod.rs`](/E:/Git/ZirconEngine/zircon_runtime/src/scene/mod.rs) 重新显式指向现有 [`semantics.rs`](/E:/Git/ZirconEngine/zircon_runtime/src/scene/semantics.rs)，避免新 `CARGO_TARGET_DIR` 下 editor host 回归被无关 runtime 模块路径错误阻断。

这轮还顺手把 Slint host shell 的结构 seam 重新对齐到当前更大的 `workbench_slint_shell` 文本回归基线：

- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 现在已经反过来删掉显式 `AssetBrowserPane` import probe，继续把 root 缩到纯 bootstrap；业务 pane catalog 可见接缝改由 `pane_surface.slint` 和 `workbench_slint_shell` 源码守卫锁定。
- [`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 现在承接 drag overlay label、document edge split target 与 `HostTabDragOverlay` 这组高层 host-shell contract；[`host_scaffold.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_scaffold.slint) 则继续收薄成 mode switch。
- [`host_components.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 补回显式 `PaneSurface` / `PaneData` import seam，保证 pane surface catalog 抽离后的结构测试继续能看到共享 host component 依赖边界。
- [`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 也补齐了 `HostMenuChrome.menu_state` 与结构化 `drag_state` / `overlay_data` 对接，消除了这一轮 preview/mock 验证中暴露出的 host-shell DTO 漂移。
- [`runtime_ui_manager.rs`](/E:/Git/ZirconEngine/zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs) 和 [`runtime_ui_manager_error.rs`](/E:/Git/ZirconEngine/zircon_runtime/src/ui/runtime_ui/runtime_ui_manager_error.rs) 现在显式从 `zircon_ui::template` 导入 `UiTemplateSurfaceBuilder` / `UiTemplateBuildError`，收敛了验证中暴露出的旧 root re-export 漂移。

本轮把“剩余 replay/preview/theme/migration/self-hosting 缺口”进一步收口后的新增证据是：

- `TMP=D:\codex-temp-next TEMP=D:\codex-temp-next CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next7 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_widget_promotion_emits_executable_widget_import_replay_commands -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp-next TEMP=D:\codex-temp-next CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next7 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_edits_preview_mock_object_entries_structurally -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp-next TEMP=D:\codex-temp-next CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next7 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_session_edits_preview_mock_collection_entries_structurally -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp-next TEMP=D:\codex-temp-next CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next7 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline ui_asset_editor_replay_workspace_applies_stylesheet_insert_and_cross_file_effects -- --nocapture`
  - 当前结果：`1 passed; 0 failed`
- `TMP=D:\codex-temp-next TEMP=D:\codex-temp-next CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next7 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --offline`
  - 当前结果：`593 passed; 0 failed`
- `TMP=D:\codex-temp-next TEMP=D:\codex-temp-next CARGO_TARGET_DIR=D:\codex-ui-asset-editor-next7 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --test workbench_slint_shell --locked --offline`
  - 当前结果：`58 passed; 0 failed`

对应的实际闭环现状也进一步明确了：

- runtime/editor window migration 已不再只停留在 session/test 层能打开。当前回归面已经覆盖 editor 侧 asset browser、binding browser、theme browser、layout workbench、preview state lab，以及 runtime 侧 HUD、inventory dialog、pause dialog、quest log dialog、settings dialog 这些共享 `.ui.toml` 入口。
- UI Asset Editor self-hosting 现在已经不是“单个 bootstrap layout 可以 parse”。editor 自身的 bootstrap layout/style/widget 资产已经进入 crate 回归，并通过 `UiAssetEditorSession -> UiDocumentCompiler -> preview host -> Slint pane` 跑通打开、import hydration 与 reflection/state projection。
