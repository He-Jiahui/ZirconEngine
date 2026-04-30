---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_session_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/ui_asset_promotion.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
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
  - zircon_editor/src/ui/asset_editor/session/style_state.rs
  - zircon_editor/src/ui/asset_editor/session/presentation_state.rs
  - zircon_editor/src/ui/asset_editor/session/preview_state.rs
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
  - zircon_editor/src/ui/layouts/views/assets_activity.rs
  - zircon_editor/src/ui/layouts/views/console_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/hierarchy_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/inspector_shell_layout.rs
  - zircon_editor/src/ui/layouts/views/project_overview.rs
  - zircon_editor/src/ui/layouts/views/welcome_shell_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/ui/workbench/fallback_pane.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/pane_data.slint
  - zircon_editor/ui/workbench/pane_surface_host_context.slint
  - zircon_editor/ui/workbench/template_pane.slint
  - zircon_editor/ui/workbench/tool_window_empty_state.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/ui/workbench/ui_asset_editor_center_column.slint
  - zircon_editor/ui/workbench/ui_asset_editor_data.slint
  - zircon_editor/ui/workbench/ui_asset_editor_palette_target_chooser.slint
  - zircon_editor/ui/workbench/ui_asset_editor_pane.slint
  - zircon_editor/ui/workbench/ui_asset_editor_stylesheet_panel.slint
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_surface_contract.slint
  - zircon_editor/ui/workbench/host_root.slint
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/host/manager/mod.rs
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
  - zircon_editor/src/tests/ui/ui_asset_editor/mod.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/reflection.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/editor_layouts.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/tests/workbench_slint_shell.rs
implementation_files:
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_session_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/ui_asset_promotion.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
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
  - zircon_editor/src/ui/asset_editor/session/style_state.rs
  - zircon_editor/src/ui/asset_editor/session/presentation_state.rs
  - zircon_editor/src/ui/asset_editor/session/preview_state.rs
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
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/ui/workbench/asset_panes.slint
  - zircon_editor/ui/workbench/animation_editor_pane.slint
  - zircon_editor/ui/workbench/assets_activity_pane.slint
  - zircon_editor/ui/workbench/console_pane.slint
  - zircon_editor/ui/workbench/fallback_pane.slint
  - zircon_editor/ui/workbench/hierarchy_pane.slint
  - zircon_editor/ui/workbench/inspector_pane.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/pane_data.slint
  - zircon_editor/ui/workbench/pane_surface_host_context.slint
  - zircon_editor/ui/workbench/project_overview_pane.slint
  - zircon_editor/ui/workbench/tool_window_empty_state.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/ui/workbench/ui_asset_editor_center_column.slint
  - zircon_editor/ui/workbench/ui_asset_editor_data.slint
  - zircon_editor/ui/workbench/ui_asset_editor_palette_target_chooser.slint
  - zircon_editor/ui/workbench/ui_asset_editor_pane.slint
  - zircon_editor/ui/workbench/ui_asset_editor_stylesheet_panel.slint
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_surface_contract.slint
  - zircon_editor/ui/workbench/host_root.slint
plan_sources:
  - user: 2026-04-20 目前zircon_editor有两套ui相关代码 一套在core里面需要迁移回ui
  - user: 2026-04-20 要求加载入口不允许放入src
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-20 不要re-export 直接清理core里ui部分
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/收敛缺口修复 Spec 与 Implementation Plan.md
tests:
  - zircon_editor/src/tests/editing/ui_asset/structure_split.rs
  - zircon_editor/src/tests/editing/ui_asset/source_projection.rs
  - zircon_editor/src/tests/editing/ui_asset/reference_and_promotion.rs
  - zircon_editor/src/tests/editing/ui_asset_theme_authoring.rs
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/host/manager/ui_asset_session_preview.rs
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
  - zircon_editor/src/tests/host/slint_detail_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/slint_detail_pointer/template_callbacks.rs
  - zircon_editor/src/tests/host/slint_list_pointer/pane_surface_actions.rs
  - zircon_editor/src/tests/host/slint_list_pointer/surface_contract.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/editor_layouts.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/tests/workbench_slint_shell.rs
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
  - cargo test -p zircon_editor --locked host_scene_projection_converts_host_owned_panes_to_slint_panes --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::boundary::template_assets:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::boundary::view_projection_cutover:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::ui::boundary::workbench_projection_cutover:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::host::slint_detail_pointer:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo test -p zircon_editor --locked tests::host::slint_list_pointer:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture
  - cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-a
  - F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe ui::slint_host::ui::tests::apply_presentation_projects_welcome_shell_layout_into_global_context --exact --nocapture
  - cargo test -p zircon_editor --locked ui_asset_editor_stylesheet_panel_consumes_shell_layout_for_header_rows --lib --target-dir F:/cargo-targets/zircon-codex-a
  - cargo test -p zircon_editor --lib tests::ui::ui_asset_editor --locked --offline --message-format short
  - cargo test -p zircon_editor --lib workbench_slint_entry_stays_on_generic_host_bootstrap_files --locked
  - cargo test -p zircon_editor --locked --offline --test workbench_slint_shell
doc_type: module-detail
---

# UI Asset Editor Host Session

## Purpose

这份文档记录 `zircon_editor` 在本轮 cutover 后的当前真相：

- editor UI 宿主实现统一收口到 `zircon_editor/src/ui`
- `core` 不再拥有 workbench/layout/window/session 的真实实现
- `UI Asset Editor` 相关代码不再分裂在 `core/editing/ui_asset` 和 `core/host/manager/ui_asset_sessions`

本篇重点说明 ownership、会话边界和 Slint 宿主入口约束，而不是重复 shared `.ui.toml` 资产格式本身。资产格式见 [`UI Asset Documents And Editor Protocol`](../ui-and-layout/ui-asset-documents-and-editor-protocol.md)。

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
  - 负责把 Slint callback 或 workbench action 路由成稳定 session 调用
- `ui::asset_editor::UiAssetEditorSession`
  - 负责 source/document/preview 三角同步
  - 负责 selection、tree edit、binding/style authoring、undo/replay
  - 负责 canonical TOML 输出和 last-good preview 语义

也就是说，host 层只保留“会话编排”和“工作台整合”；真正的 UI asset authoring 行为已经回到 `ui::asset_editor` 域内，而不是继续夹在 `core` 和 `ui` 之间。

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

## Slint Entry Boundary

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

这一轮又把 `workbench_host_window` 内部最后一批深层 pane payload Slint DTO 拔到了真正的宿主边界：

- [`host_data.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs) 里的 [`PaneData`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs) 现在不再直接持有生成态 `UiAssetEditorPaneData` / `AnimationEditorPaneData`
- `ui_asset` 字段直接退回 [`UiAssetEditorPanePresentation`](../../zircon_editor/src/ui/asset_editor/presentation.rs) 这份 asset-editor 域内的纯 Rust presentation
- `animation` 字段改成 [`AnimationEditorPaneViewData`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs)，宿主拥有的 Rust-owned payload 现在既包含业务 string/model，也包含从 bootstrap asset 萃取出来的 `shell_layout`
- [`pane_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs) 现在只负责选择哪个 pane/session presentation 进入当前 tab，不再在 workbench host 内部拼装 `UiAsset*` / `AnimationEditorPaneData` 这类 Slint 生成 struct
- Slint 边界仍然由 [`apply_presentation.rs`](../../zircon_editor/src/ui/slint_host/ui/apply_presentation.rs) 暴露：
  - `to_slint_ui_asset_pane(...)`
  - `to_slint_animation_editor_pane(...)`
- 这两个函数现在只是薄 wrapper，真正的大块 pane DTO 映射实现已经拆到 [`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs)，避免 `apply_presentation.rs` 再次演变成新的 1k+ 行 owner 文件

这条边界的意义很直接：

- `workbench_host_window` 现在只消费 editor UI 域自己的 Rust 数据，不再 import 任何深层 `UiAsset*` / `AnimationEditorPaneData`
- Slint 生成类型继续存在，但它们只作为 `UiHostWindow` presentation apply 的末端 ABI，而不是 workbench host projection 的内部工作模型
- 后续如果要删 `ui_asset_editor_pane.slint` 或 `animation_editor_pane.slint`，就不需要再先拆一轮 `workbench_host_window -> slint_host generated type` 依赖链

## Bootstrap Shell Layout Authority

`UiAssetEditor` 的 `.slint` 真源这轮继续往下缩，但还没有直接跳到完整 generic renderer。当前先切掉的是 pane shell 的外层几何 owner：

- [`ui_asset_editor.ui.toml`](../../zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml) 现在不再承载旧的 footer-row 试验形态，而是直接描述当前工作中的 `UiAssetEditor` pane shell：`HeaderPanel`、`LeftColumn`、`CenterColumn`、`RightColumn` 以及 `PalettePanel`、`HierarchyPanel`、`DesignerPanel`、`ActionBarPanel`、`SourcePanel`、`InspectorPanel`、`StylesheetPanel`
- [`shell_layout.rs`](../../zircon_editor/src/ui/asset_editor/shell_layout.rs) 作为新的生产 owner，从 crate `assets/` 读取 bootstrap `.ui.toml`、注册 widget/style import、编译 shared `UiSurface`，再把上述 control frame 萃取成 `UiAssetEditorShellLayout`
- [`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 不再让 `UiAssetEditorPanePresentation` 只携带纯业务字段，而是在知道 dock/floating content size 后，把 `UiAssetEditorShellLayout` 灌回 `pane.ui_asset.shell_layout`
- [`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs) 继续作为 Slint 边界 owner，把 `UiAssetEditorShellLayout` 转成 `UiAssetEditorShellLayoutData`
- [`ui_asset_editor_pane.slint`](../../zircon_editor/ui/workbench/ui_asset_editor_pane.slint) 现在只消费这些 shell frame 来放置 header/left column/center column/right column；旧的 outer `HorizontalLayout { x: 10px; y: 74px; ... }` 固定几何不再是唯一 authority

这一步的目的不是宣称 `UiAssetEditor` 已经完全摆脱业务 `.slint`，而是先把最外层 pane shell 的几何真源从手写 Slint 改成树形 `.ui.toml`。列内的具体 leaf widget、draft state、interaction callback 还暂时留在 `.slint`，下一阶段才能继续削减到真正的 projection/generic host。

随后 header 本身也不再保留那组三段固定偏移作为真源：

- [`editor_widgets.ui.toml`](../../zircon_editor/assets/ui/editor/editor_widgets.ui.toml) 新增 `EditorHeaderShell`，把 summary/status/actions 三段行壳定义成共享 widget 资产，而不是继续借 `EditorToolbar` 单 slot 结构硬塞业务 header
- [`ui_asset_editor.ui.toml`](../../zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml) 现在继续下钻到 `HeaderAssetRow`、`HeaderStatusRow`、`HeaderActionRow`
- [`shell_layout.rs`](../../zircon_editor/src/ui/asset_editor/shell_layout.rs) / [`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs) / [`ui_asset_editor_data.slint`](../../zircon_editor/ui/workbench/ui_asset_editor_data.slint) 已把这三块 frame 当成正式宿主边界字段
- [`ui_asset_editor_pane.slint`](../../zircon_editor/ui/workbench/ui_asset_editor_pane.slint) 只在各 row host 里摆标题文本、状态文本和 mode/preset/save/undo/redo 控件，不再自己保留 `x: 10 / y: 6 / 16 / 28` 这组 header row 偏移真源

随后这条边界又继续往 `SourcePanel` 内部推进了一层，不再只停在“整个 source panel 是一个大矩形”：

- [`ui_asset_editor.ui.toml`](../../zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml) 现在除了 `SourceInfoPanel`、`SourceOutlinePanel`、`MockWorkspacePanel`、`SourceTextPanel` 之外，还明确给 `MockWorkspacePanel` 拆出了 `MockSubjectsPanel`、`MockEditorPanel`、`MockStateGraphPanel`
- 同一份 bootstrap asset 也开始继续拆 `DesignerPanel` 内部几何，新增 `DesignerCanvasPanel` 和 `RenderStackPanel`，避免 `.slint` 自己继续保留 `preview_canvas y: 28px`、`render stack y: parent.height - 90px` 这类内部纵向真源
- [`shell_layout.rs`](../../zircon_editor/src/ui/asset_editor/shell_layout.rs) 和 [`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs) 也同步把这三块 frame 作为正式 `UiAssetEditorShellLayout` / `UiAssetEditorShellLayoutData` 字段传到 Slint 边界
- [`ui_asset_editor_center_column.slint`](../../zircon_editor/ui/workbench/ui_asset_editor_center_column.slint) 不再自己决定 mock workspace 的大段纵向分区；它现在只消费这三个 host band，再在每个 band 内用局部 layout 摆叶子控件
- `Designer` 区域也开始遵守同一个规则：`.slint` 只在 `DesignerCanvasPanel` / `RenderStackPanel` 里面放 preview canvas 与 render-stack list，不再自己决定这两个 band 的相对外框
- `ActionBarPanel` 现在也不再把三排按钮组的分段几何留在 `.slint` 里：
  - bootstrap `.ui.toml` 继续下钻到 `ActionInsertRow`、`ActionReparentRow`、`ActionStructureRow`
  - `shell_layout.rs` / `pane_data_conversion/mod.rs` / `UiAssetEditorShellLayoutData` 把这三块 frame 当成正式宿主边界字段传进 Slint
  - `ui_asset_editor_center_column.slint` 只在每个 row host 里摆对应 `ShellButton`，旧的 `VerticalLayout { x: 10px; y: 8px; width: parent.width - 20px; spacing: 4px; }` 已不再是 action bar 的真正分组 authority
- `StylesheetPanel` 的顶部壳层也开始遵守同一原则：
  - bootstrap `.ui.toml` 现在继续下钻到 `StylesheetActionRow`、`StylesheetStatePrimaryRow`、`StylesheetStateSecondaryRow`、`StylesheetContentPanel`
  - `ui_asset_editor_stylesheet_panel.slint` 不再自己保留 `y: 26/54/82/118` 这组 header/body 分区真源，而是直接消费 shell-layout frame
  - `ui_asset_editor_pane.slint` 只负责把 `shell_layout` 和 `panel_frame` 透传给 `UiAssetEditorStylesheetPanel`，不再在右侧 panel owner 里重算这组 offsets
- `InspectorPanel` 现在也开始把内容壳层交回 bootstrap 资产：
  - bootstrap `.ui.toml` 不再只放一个占位 `InspectorLabel`，而是继续下钻到 `InspectorContentPanel`
  - `shell_layout.rs` / `pane_data_conversion/mod.rs` / `UiAssetEditorShellLayoutData` 把这块 frame 当成正式宿主边界字段传进 Slint
  - `ui_asset_editor_pane.slint` 只负责把 `shell_layout` 和 `panel_frame` 透传给 `UiAssetEditorInspectorPanel`
  - `ui_asset_editor_inspector_panel.slint` 不再把 `Rectangle { x: 0px; y: 26px; width: parent.width; height: parent.height - 26px; ... }` 和内部 `VerticalLayout { x: 10px; y: 8px; ... }` 当成唯一壳层几何 authority，而是直接消费 `InspectorContentPanel` frame

这样 cutover 的真实结果是：`UiAssetEditor` 还没有完全删除业务 `.slint`，但 source/mock 这块最明显的“绝对 y 偏移串”已经不再是最终几何 authority。`.slint` 这时只剩 leaf interaction、draft field 和局部微布局，而不是继续拿一串 `y: 78/94/126/...` 当真正的 panel 结构。

根 pane 自己持有的 popup 壳层也继续缩了一步：

- [`ui_asset_editor_pane.slint`](../../zircon_editor/ui/workbench/ui_asset_editor_pane.slint) 不再内联 palette target chooser overlay 的 candidate list / footer actions / keyboard cycling 这整块 popup owner
- 这块动态 overlay 已经下沉到 [`ui_asset_editor_palette_target_chooser.slint`](../../zircon_editor/ui/workbench/ui_asset_editor_palette_target_chooser.slint)，root pane 只保留 placement 和 host-level callback forwarding
- 这一步还没把 chooser shell 交给 bootstrap `.ui.toml`，因为它的尺寸和可见性仍然直接依赖 palette drag 候选集与 preview 指针状态；但至少 root pane 不再继续吞这块动态 popup 业务壳

这一步对应的 focused evidence 也已经补上：

- [`workbench_projection_cutover.rs`](../../zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs)
  - source guard 明确禁止 `workbench_host_window` 重新 import `AnimationEditorPaneData`、`AnimationEditorShell*Data` 和整组 `UiAsset*` 生成 DTO
  - 同时要求 `apply_presentation.rs` 保留 `to_slint_animation_editor_pane(...)` 与 `to_slint_ui_asset_pane(...)`
- [`ui/tests.rs`](../../zircon_editor/src/ui/slint_host/ui/tests.rs)
  - host-scene projection 回归现在会把非默认 `UiAssetEditorPanePresentation` 样本推进 Slint 边界，确认 header/palette 数据不是只在源码层存在
- [`template_assets.rs`](../../zircon_editor/src/tests/ui/boundary/template_assets.rs)
  - source-panel guard 继续确认 `ui_asset_editor_center_column.slint` 必须直接消费 `mock_subjects_panel` / `mock_editor_panel` / `mock_state_graph_panel`
  - 同时禁止旧的 mock workspace 绝对偏移重新回流进 source panel owner
  - action-bar guard 继续确认 `ui_asset_editor_center_column.slint` 必须直接消费 `action_insert_row` / `action_reparent_row` / `action_structure_row`
  - 同时禁止旧的 action-bar `VerticalLayout { x: 10px; y: 8px; ... }` 行分组重新回流
  - stylesheet guard 继续确认 `ui_asset_editor_stylesheet_panel.slint` 必须直接消费 `stylesheet_action_row` / `stylesheet_state_primary_row` / `stylesheet_state_secondary_row` / `stylesheet_content_panel`
  - 同时禁止旧的 stylesheet `y: 26/54/82/112/118` 顶部壳层 offsets 重新回流
- [`bootstrap_assets.rs`](../../zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs)
  - bootstrap shell-layout 回归现在要求 `MockSubjectsPanel` / `MockEditorPanel` / `MockStateGraphPanel` 在编译后的 shared `UiSurface` 中都能导出有效 frame
  - 同时也要求 `DesignerCanvasPanel` / `RenderStackPanel` 导出有效 frame，避免 designer panel 内层几何重新回流到 Slint
  - `ActionInsertRow` / `ActionReparentRow` / `ActionStructureRow` 也必须从 bootstrap shell-layout 导出有效 frame，避免 action bar 分组再退回成 Slint-only 布局
  - `StylesheetActionRow` / `StylesheetStatePrimaryRow` / `StylesheetStateSecondaryRow` / `StylesheetContentPanel` 也必须导出有效 frame，避免右侧 stylesheet panel 再退回成手写 Slint 分区
  - `InspectorContentPanel` 也必须导出有效 frame，避免 inspector body 再退回成手写 `y: 26 + x: 10/y: 8` 的 Slint 壳层偏移
- [`template_assets.rs`](../../zircon_editor/src/tests/ui/boundary/template_assets.rs)
  - inspector guard 现在要求 `ui_asset_editor_inspector_panel.slint` 必须直接消费 `shell_layout.inspector_content_panel`
  - 同时禁止旧的 inspector content `Rectangle { x: 0px; y: 26px; ... }` 和 `VerticalLayout { x: 10px; y: 8px; ... }` 壳层几何重新回流
  - palette-target-chooser guard 现在要求 root pane 只能 import `ui_asset_editor_palette_target_chooser.slint` 并委托 overlay owner；旧的 candidate list / apply-cancel footer / sticky hint 文案不能再内联回 `ui_asset_editor_pane.slint`

同一轮 `.slint -> ui.toml` 收口现在也已经推进到 `AssetBrowserPane` 的宿主壳层：

- [`asset_browser.ui.toml`](../../zircon_editor/assets/ui/editor/asset_browser.ui.toml) 不再只是一个粗粒度 page shell，而是把 `ToolbarPanel`、`ImportPanel`、`MainPanel`、`UtilityPanel` 以及 `ToolbarTitleRow`、`ToolbarSearchRow`、`ToolbarKindPrimaryRow`、`ToolbarKindSecondaryRow`、`UtilityTabsRow`、`UtilityContentPanel`、`ReferenceLeftPanel`、`ReferenceRightPanel` 都固定成 bootstrap asset authority
- [`asset_browser_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/asset_browser_shell_layout.rs) 作为新的读取 owner，从 crate `assets/` 编译这份 tree asset 并把 frame 萃取成 `AssetBrowserShellLayout`
- [`host_data.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs)、[`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 和 [`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs) 继续把这份 layout 作为 `PaneData.asset_browser -> AssetBrowserPaneData -> AssetBrowserShellLayoutData` 的正式投影链路
- [`pane_content.slint`](../../zircon_editor/ui/workbench/pane_content.slint) 现在只把 `root.pane.asset_browser` 透传进 [`asset_panes.slint`](../../zircon_editor/ui/workbench/asset_panes.slint)，而 `AssetBrowserPane` 本身不再保留旧的 outer-margin / toolbar-height / sources-width / details-width 公式；它只消费 `root.pane.shell_layout.*` frame，再在每个 host band 里摆 Search、kind chip、details rail 和 utility tab 叶子控件
- 这一步先只收口 shell/topology authority。`metadata` / `plugins` tab 的叶子排布仍然保留在 pane owner 内部，但它们已经退到 `UtilityContentPanel` 这个稳定宿主壳层之下，不再决定 page 级 panel 分区

这条 asset browser seam 的 focused evidence 也已经补上：

- [`bootstrap_assets.rs`](../../zircon_editor/src/tests/ui/asset_browser/bootstrap_assets.rs)
  - `asset_browser_bootstrap_layout_self_hosts_shell_sections` 继续锁定 bootstrap 资产必须导出 toolbar/import/main/utility 以及 reference 双栏 frame
- [`template_assets.rs`](../../zircon_editor/src/tests/ui/boundary/template_assets.rs)
  - `asset_browser_pane_consumes_shell_layout_for_top_level_sections` 和 `asset_browser_pane_consumes_shell_layout_for_toolbar_and_utility_sections` 现在直接禁止 `asset_panes.slint` 回流旧的 top-level geometry formula 与 toolbar/reference 绝对坐标
- `cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-a`
  - 证明 asset browser 的 host projection、DTO 转换和 Slint consumer 同步后，`zircon_editor` production 代码仍能正常编译

同一轮 `.slint -> ui.toml` 收口随后也推进到 `AnimationEditorPane`：

- [`animation_editor.ui.toml`](../../zircon_editor/assets/ui/editor/animation_editor.ui.toml)
  - 现在把 `AnimationEditorHeaderPanel` / `AnimationEditorBodyPanel` 以及 `HeaderModeRow`、`HeaderPathRow`、`HeaderStatusRow`、`AnimationSequence*`、`AnimationGraph*`、`AnimationStateMachine*` 这些 mode band 固定成 bootstrap asset authority
  - `BodyPanel` 使用 overlay container，让 sequence / graph / state-machine 三套 mode shell 共享同一块内容区，而不是在 `.slint` 里各自保留一组 inset/offset 公式
- [`animation_editor_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/animation_editor_shell_layout.rs)
  - 作为新的读取 owner，从 crate `assets/` 编译 tree asset 并把 frame 萃取成 `AnimationEditorShellLayout`
- [`host_data.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs)、[`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 和 [`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs)
  - 把这份 layout 作为 `AnimationEditorPaneViewData -> AnimationEditorPaneData -> AnimationEditorShellLayoutData` 的正式投影链路
- [`animation_editor_pane.slint`](../../zircon_editor/ui/workbench/animation_editor_pane.slint)
  - 现在只消费 `root.pane.shell_layout.*` frame，再在对应 host band 内摆 mode text、timeline/selection 文本和 track/node/state/transition 列表
  - 旧的 `height: 64px` header、`width: parent.width - 24px` mode panel、`y: 140px / 148px` graph-state-machine band offset 已不再是 animation pane 的真正壳层 authority

这条 animation seam 的 focused evidence 也已经补上：

- [`bootstrap_assets.rs`](../../zircon_editor/src/tests/ui/animation_editor/bootstrap_assets.rs)
  - `animation_editor_bootstrap_layout_self_hosts_shell_sections` 和 `animation_editor_shell_layout_exposes_mode_shell_regions` 现在锁定 bootstrap 资产必须导出 header/body 以及三种 mode shell frame
- [`template_assets.rs`](../../zircon_editor/src/tests/ui/boundary/template_assets.rs)
  - `animation_editor_pane_consumes_shell_layout_for_top_level_sections` 与 `animation_editor_pane_consumes_shell_layout_for_mode_inner_sections` 直接禁止 `animation_editor_pane.slint` 回流旧的 `64px/140px/148px` 硬编码壳层公式
- [`ui/tests.rs`](../../zircon_editor/src/ui/slint_host/ui/tests.rs)
  - `host_scene_projection_converts_host_owned_panes_to_slint_panes` 现在也会把非默认 animation shell-layout frame 样本推进 Slint 边界，确认 animation pane DTO 转换不会把新字段丢在宿主内部
- `cargo test -p zircon_editor --locked tests::ui::animation_editor:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
- `cargo test -p zircon_editor --locked tests::ui::boundary::template_assets:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
- `cargo test -p zircon_editor --locked host_scene_projection_converts_host_owned_panes_to_slint_panes --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`

同一轮 `.slint -> ui.toml` 收口现在也推进到 `AssetsActivityPane`：

- [`assets_activity.ui.toml`](../../zircon_editor/assets/ui/editor/assets_activity.ui.toml)
  - 现在把 `AssetsActivityToolbarPanel`、`AssetsActivityMainPanel`、`AssetsActivityUtilityPanel` 以及 `ToolbarTitleRow`、`ToolbarSearchRow`、`ToolbarKindPrimaryRow`、`ToolbarKindSecondaryRow`、`TreePanel`、`ContentPanel`、`UtilityTabsRow`、`UtilityContentPanel`、`PreviewPanel`、`ReferenceLeftPanel`、`ReferenceRightPanel` 固定成 bootstrap asset authority
  - `UtilityContentPanel` 继续使用 overlay 容器，让 preview 和 references 双栏共享同一块宿主内容区，而不是继续在 `.slint` 里保留 `y: 50`、`(parent.width - 12px) / 2` 这类壳层公式
- [`assets_activity.rs`](../../zircon_editor/src/ui/layouts/views/assets_activity.rs)
  - 现在的读取 owner 不再萃取 `AssetsActivityShellLayout`，而是通过 `build_view_template_nodes(...)` 直接把 tree asset 编译成 `ViewTemplateNodeData`
  - `AssetsActivityPaneViewData` 正式承接 `nodes: ModelRc<ViewTemplateNodeData>`，把 `Assets` pane 固定在“runtime tree asset -> neutral node projection -> Slint leaf owner”的主链上
- [`host_data.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs)、[`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 和 [`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs)
  - 现在把这条链路固定成 `AssetsActivityPaneViewData -> AssetsActivityPaneData { nodes } -> Slint [TemplatePaneNodeData]`
  - `pane_with_assets_activity_projection(...)` 负责在 host scene 投影阶段注入当前尺寸下的 mount frame，而不是把 authoring 几何写死回 `.slint`
- [`pane_content.slint`](../../zircon_editor/ui/workbench/pane_content.slint) 与 [`pane_data.slint`](../../zircon_editor/ui/workbench/pane_data.slint)
  - `PaneData.assets_activity` 现在直接暴露 `nodes`
  - `AssetsActivityPaneView` 保持 inline owner，但只消费 `root.pane.nodes`；各个 toolbar/main/utility 子面通过 `control_id` 过滤的重复元素读 mount frame，不再依赖旧的 shell-layout DTO，也没有单独的 `assets_activity_pane.slint`

这条 assets activity seam 的 focused evidence 也已经补上：

- [`bootstrap_assets.rs`](../../zircon_editor/src/tests/ui/assets_activity/bootstrap_assets.rs)
  - `assets_activity_bootstrap_layout_self_hosts_shell_sections` 和 `assets_activity_projection_maps_bootstrap_asset_into_mount_nodes` 锁定 bootstrap 资产必须导出 toolbar/main/utility 以及 preview/reference mount，并确认运行时投影会产出真实 frame
- [`template_assets.rs`](../../zircon_editor/src/tests/ui/boundary/template_assets.rs)
  - `assets_activity_pane_consumes_template_mount_nodes_for_top_level_sections` 与 `assets_activity_pane_consumes_template_mount_nodes_for_toolbar_and_utility_sections` 直接禁止 `pane_content.slint` 回流到 shell-layout 依赖
- [`ui/tests.rs`](../../zircon_editor/src/ui/slint_host/ui/tests.rs)
  - `host_scene_projection_converts_host_owned_panes_to_slint_panes` 现在会把非默认 `AssetsActivity` node frame 样本推进 Slint 边界，确认 assets activity pane DTO 转换不会把节点投影留在宿主内部
- `cargo test -p zircon_editor --locked tests::ui::assets_activity:: --lib --target-dir F:/cargo-targets/zircon-codex-d -- --nocapture`
- `cargo test -p zircon_editor --locked tests::ui::boundary::template_assets:: --lib --target-dir F:/cargo-targets/zircon-codex-d -- --nocapture`
- `cargo test -p zircon_editor --locked host_scene_projection_converts_host_owned_panes_to_slint_panes --lib --target-dir F:/cargo-targets/zircon-codex-d -- --nocapture`
- `cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-d`

同一轮 `.slint -> ui.toml` 收口现在也推进到 `ProjectOverviewPane`：

- [`project_overview.ui.toml`](../../zircon_editor/assets/ui/editor/project_overview.ui.toml)
  - 现在把 `ProjectOverviewOuterPanel`、`ProjectOverviewHeaderTitleRow`、`ProjectOverviewHeaderPathRow`、`ProjectOverviewDetailsPanel`、`ProjectOverviewCatalogPanel` 固定成 bootstrap asset authority
  - 这份 layout 同时保留标题、详情值和 CTA 的 control id，让 project snapshot 的文本覆写直接落到 neutral node projection，而不是回到 `.slint` 壳层坐标
- [`project_overview.rs`](../../zircon_editor/src/ui/layouts/views/project_overview.rs)
  - 作为读取 owner，通过 `build_view_template_nodes(...)` 把 tree asset 编译成 `ViewTemplateNodeData`
  - `project_overview_pane_data(...)` 在投影阶段直接把 project snapshot 文本写进对应 control node，正式去掉 `ProjectOverviewShellLayout`
- [`host_data.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs)、[`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs)、[`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs) 和 [`apply_presentation.rs`](../../zircon_editor/src/ui/slint_host/ui/apply_presentation.rs)
  - 把 project pane 链路固定成 `ProjectOverviewPaneViewData -> ProjectOverviewPaneData { nodes }`
  - 继续禁止 `workbench_host_window` 内部回流 Slint 生成 DTO，只允许 host 侧保留 Rust-owned neutral node projection
- [`pane_content.slint`](../../zircon_editor/ui/workbench/pane_content.slint)、[`pane_data.slint`](../../zircon_editor/ui/workbench/pane_data.slint) 与 [`template_pane.slint`](../../zircon_editor/ui/workbench/template_pane.slint)
  - `PaneData.project_overview` 现在直接暴露 `nodes`
  - `Project` pane 已经改成 generic `TemplatePane` 消费者；不存在单独的 `project_overview_pane.slint` owner，也不再需要 page shell DTO

这条 project overview seam 的 focused evidence 也已经补上：

- [`bootstrap_assets.rs`](../../zircon_editor/src/tests/ui/project_overview/bootstrap_assets.rs)
  - `project_overview_bootstrap_layout_self_hosts_shell_sections` 和 `project_overview_projection_maps_bootstrap_asset_into_template_nodes` 锁定 bootstrap 资产必须导出 outer/title/path/details/catalog node，并确认 snapshot 文本会进到投影结果
- [`template_assets.rs`](../../zircon_editor/src/tests/ui/boundary/template_assets.rs)
  - `project_overview_pane_routes_through_generic_template_owner_file` 与 `project_overview_pane_consumes_shell_layout_for_top_level_sections` 一起固定住 current cutover：壳层 authority 在 `.ui.toml`，Slint 只保留 generic template owner
- [`workbench_projection_cutover.rs`](../../zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs)
  - 继续禁止 `workbench_host_window` 内部回流 `ProjectOverviewPaneData` 这类 Slint 生成 DTO，并要求 `apply_presentation.rs` 保留 `to_slint_project_overview_pane(...)` wrapper
- [`ui/tests.rs`](../../zircon_editor/src/ui/slint_host/ui/tests.rs)
  - `host_scene_projection_converts_host_owned_panes_to_slint_panes` 现在会把非默认 `ProjectOverview` node 样本推进 Slint 边界，确认 project overview pane DTO 转换不会把模板节点留在宿主内部
- `cargo test -p zircon_editor --locked tests::ui::project_overview:: --lib --target-dir F:/cargo-targets/zircon-codex-d -- --nocapture`
- `cargo test -p zircon_editor --locked tests::ui::boundary::template_assets:: --lib --target-dir F:/cargo-targets/zircon-codex-d -- --nocapture`
- `cargo test -p zircon_editor --locked tests::ui::boundary::workbench_projection_cutover:: --lib --target-dir F:/cargo-targets/zircon-codex-d -- --nocapture`
- `cargo test -p zircon_editor --locked host_scene_projection_converts_host_owned_panes_to_slint_panes --lib --target-dir F:/cargo-targets/zircon-codex-d -- --nocapture`
- `cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-d`

同一轮 `.slint -> ui.toml` 收口现在也推进到 `WelcomePane`：

- [`welcome.ui.toml`](../../zircon_editor/assets/ui/editor/welcome.ui.toml)
  - 现在把 `WelcomeOuterPanel`、`WelcomeRecentPanel`、`WelcomeRecentHeaderPanel`、`WelcomeRecentListPanel`、`WelcomeMainPanel`、`WelcomeHeroPanel`、`WelcomeStatusPanel`、`WelcomeNewProjectHeaderPanel`、`WelcomeProjectNameField`、`WelcomeLocationField`、`WelcomePreviewPanel`、`WelcomeValidationPanel`、`WelcomeActionsRow` 固定成 bootstrap asset authority
  - 这份 layout 只收口欢迎页的 page-level shell bands，不把 recent item 行内叶子控件和表单卡片内部排版提前塞进资产
- [`welcome_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/welcome_shell_layout.rs)
  - 作为新的读取 owner，从 crate `assets/` 编译 tree asset 并把 frame 萃取成 `WelcomeShellLayout`
- [`view_data.rs`](../../zircon_editor/src/ui/layouts/views/view_data.rs)、[`welcome_presentation.rs`](../../zircon_editor/src/ui/layouts/views/welcome_presentation.rs) 与 [`apply_presentation.rs`](../../zircon_editor/src/ui/slint_host/ui/apply_presentation.rs)
  - 把欢迎页链路固定成 `WelcomePaneSnapshot -> WelcomePaneData -> WelcomeShellLayoutData -> PaneSurfaceHostContext.welcome_pane`
  - `apply_presentation.rs` 现在会从可见的 `Welcome` pane/document/floating surface 解析真实内容区尺寸，再在 Slint 边界前补齐 `shell_layout`，避免继续让 `welcome.slint` 自己推导整页壳层
- [`welcome.slint`](../../zircon_editor/ui/workbench/welcome.slint)
  - 现在只消费 `root.welcome.shell_layout.*` frame，再在 recent/header/list、hero/status、new-project/form/preview/actions 各 band 内摆叶子文本和按钮
  - 旧的 `18px` outer inset、固定 `320px` recent column 和 `28px` 右侧 hero/form band 坐标已经从欢迎页 owner 主链移除

这条 welcome seam 的 focused evidence 也已经补上：

- [`bootstrap_assets.rs`](../../zircon_editor/src/tests/ui/welcome/bootstrap_assets.rs)
  - `welcome_bootstrap_layout_self_hosts_shell_sections` 和 `welcome_shell_layout_exposes_pane_shell_regions` 锁定 bootstrap 资产必须导出 recent/main/hero/status/form/preview/actions 这组 frame
- [`template_assets.rs`](../../zircon_editor/src/tests/ui/boundary/template_assets.rs)
  - `welcome_pane_consumes_shell_layout_for_top_level_sections` 直接禁止 `welcome.slint` 回流旧的 outer panel、left column、hero/status/form 绝对坐标
- [`view_projection_cutover.rs`](../../zircon_editor/src/tests/ui/boundary/view_projection_cutover.rs)
  - 继续禁止 `layouts::views` 回流 `WelcomeShellFrameData` / `WelcomeShellLayoutData` 这类 Slint 生成 DTO，并要求 `apply_presentation.rs` 保留 `to_slint_welcome_pane(...)` wrapper
- [`ui/tests.rs`](../../zircon_editor/src/ui/slint_host/ui/tests.rs)
  - `apply_presentation_projects_welcome_shell_layout_into_global_context` 直接验证 `apply_presentation` 会按可见 welcome surface 的内容区尺寸投影真实 `WelcomeShellLayout`
- `cargo test -p zircon_editor --locked tests::ui::welcome:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
- `cargo test -p zircon_editor --locked tests::ui::boundary::template_assets:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
- `cargo test -p zircon_editor --locked tests::ui::boundary::view_projection_cutover:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe ui::slint_host::ui::tests::apply_presentation_projects_welcome_shell_layout_into_global_context --exact --nocapture`
- `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe ui::slint_host::ui::tests::host_scene_projection_converts_host_owned_panes_to_slint_panes --exact --nocapture`
- `cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-a`
  - 当前这条 `zircon_editor` crate-local production check 已重新转绿，welcome shell-layout cutover 不再被外部运行时编译漂移阻断

同一轮 `.slint -> ui.toml` 收口最后也把 `panes.slint` 的剩余 seam 全部切干净：

- [`panes.slint`](../../zircon_editor/ui/workbench/panes.slint) 已删除；[`pane_content.slint`](../../zircon_editor/ui/workbench/pane_content.slint) 现在直接路由到 [`hierarchy_pane.slint`](../../zircon_editor/ui/workbench/hierarchy_pane.slint)、[`inspector_pane.slint`](../../zircon_editor/ui/workbench/inspector_pane.slint)、[`console_pane.slint`](../../zircon_editor/ui/workbench/console_pane.slint) 这三个真实 pane owner，同时把空态和兜底 owner 拆到 [`tool_window_empty_state.slint`](../../zircon_editor/ui/workbench/tool_window_empty_state.slint) 与 [`fallback_pane.slint`](../../zircon_editor/ui/workbench/fallback_pane.slint)，避免继续让一个 legacy owner 同时持有业务 pane、空态卡和 fallback 文案。
- [`hierarchy.ui.toml`](../../zircon_editor/assets/ui/editor/hierarchy.ui.toml)、[`inspector.ui.toml`](../../zircon_editor/assets/ui/editor/inspector.ui.toml)、[`console.ui.toml`](../../zircon_editor/assets/ui/editor/console.ui.toml)
  - 分别把 `HierarchyListPanel`、`InspectorContentPanel/HeaderPanel/NameRow/ParentRow/PositionRow/SeparatorRow/ActionsRow`、`ConsoleTextPanel` 固定成新的 bootstrap asset authority
- [`hierarchy_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/hierarchy_shell_layout.rs)、[`inspector_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/inspector_shell_layout.rs)、[`console_shell_layout.rs`](../../zircon_editor/src/ui/layouts/views/console_shell_layout.rs)
  - 作为新的读取 owner，从 crate `assets/` 编译这三份 tree asset，并把 frame 萃取成 Rust-owned shell layout DTO
- [`pane_data.slint`](../../zircon_editor/ui/workbench/pane_data.slint)、[`host_data.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs)、[`pane_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs)、[`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs)、[`pane_data_conversion/mod.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs) 与 [`apply_presentation.rs`](../../zircon_editor/src/ui/slint_host/ui/apply_presentation.rs)
  - 现在把剩余三类 pane 的合同固定成 `Rust host DTO -> Slint boundary DTO -> dedicated pane owner`
  - `PaneData.hierarchy` 现在直接携带 `hierarchy_nodes` 和 `shell_layout`
  - `PaneData.inspector` 现在直接携带 `info`、`inspector_name`、`inspector_parent`、`inspector_x/y/z`、`delete_enabled` 和 `shell_layout`
  - `PaneData.console` 现在直接携带 `status_text` 和 `shell_layout`
- [`pane_surface_host_context.slint`](../../zircon_editor/ui/workbench/pane_surface_host_context.slint)
  - 现在只保留 hover/scroll 状态、共享 backing collection 和 pointer/template callback，不再继续充当 hierarchy/inspector/console 业务语义字段的总线
- 这条 seam 的 focused evidence 也已经补上：
  - [`bootstrap_assets.rs`](../../zircon_editor/src/tests/ui/hierarchy/bootstrap_assets.rs)、[`bootstrap_assets.rs`](../../zircon_editor/src/tests/ui/inspector/bootstrap_assets.rs)、[`bootstrap_assets.rs`](../../zircon_editor/src/tests/ui/console/bootstrap_assets.rs) 分别锁定三份 bootstrap 资产必须导出 list/content/text 这组 shell frame
  - [`template_assets.rs`](../../zircon_editor/src/tests/ui/boundary/template_assets.rs) 直接禁止 `pane_data.slint` 再从 `panes.slint` 导入剩余 pane DTO，并禁止 hierarchy/inspector/console owner 回流旧的手写壳层几何
  - [`workbench_projection_cutover.rs`](../../zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs) 继续禁止 `workbench_host_window` 内部重新导入 `Hierarchy*` / `Inspector*` / `Console*` 这类 Slint 生成 DTO，并要求 `apply_presentation.rs` 保留对应 `to_slint_*_pane(...)` wrapper
  - [`surface_contract.rs`](../../zircon_editor/src/tests/host/slint_detail_pointer/surface_contract.rs)、[`template_callbacks.rs`](../../zircon_editor/src/tests/host/slint_detail_pointer/template_callbacks.rs)、[`surface_contract.rs`](../../zircon_editor/src/tests/host/slint_list_pointer/surface_contract.rs) 与 [`pane_surface_actions.rs`](../../zircon_editor/src/tests/host/slint_list_pointer/pane_surface_actions.rs) 追加锁定 detail/list surface 继续走通用 pointer/template callback 合同，而不是回退到 legacy business callback ABI
  - `cargo test -p zircon_editor --locked tests::ui::hierarchy:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - `cargo test -p zircon_editor --locked tests::ui::inspector:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - `cargo test -p zircon_editor --locked tests::ui::console:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - `cargo test -p zircon_editor --locked tests::ui::boundary::template_assets:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - `cargo test -p zircon_editor --locked tests::ui::boundary::workbench_projection_cutover:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - `cargo test -p zircon_editor --locked tests::host::slint_detail_pointer:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - `cargo test -p zircon_editor --locked tests::host::slint_list_pointer:: --lib --target-dir F:/cargo-targets/zircon-codex-a -- --nocapture`
  - `F:/cargo-targets/zircon-codex-a/debug/deps/zircon_editor-0e7c5fdfee4db764.exe ui::slint_host::ui::tests::host_scene_projection_converts_host_owned_panes_to_slint_panes --exact --nocapture`
  - `cargo check -p zircon_editor --lib --locked --target-dir F:/cargo-targets/zircon-codex-a`

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
- `cargo test -p zircon_editor --lib workbench_slint_entry_stays_on_generic_host_bootstrap_files --locked`
  - 证明 `workbench.slint` 入口不再倒回业务壳 import
- `cargo test -p zircon_editor --locked --offline --test workbench_slint_shell`
  - 证明 bootstrap Slint 合同已经稳定收敛到 `HostWindowPresentationData -> HostWorkbenchWindowSurfaceContract -> scene/native split`，不会回退到旧的散装 surface passthrough
- `cargo test -p zircon_editor --locked workbench_host_window_keeps_generated_slint_shell_dtos_at_ui_boundary_only`
  - 证明 `workbench_host_window` 仍未重新吃回深层 `UiAsset*` / `AnimationEditorPaneData`，而且 `apply_presentation.rs` 对外 wrapper 还在
- `cargo test -p zircon_editor --locked host_scene_projection_converts_host_owned_panes_to_slint_panes`
  - 证明新的 `pane_data_conversion/mod.rs` 真实接管了 pane 投影，而不是只通过源码守卫
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

这组测试组合起来，覆盖了“代码物理位置”“owner 边界”“shared 资产链路”“导航/光标行为回归”和“Slint 入口约束”几个最关键的验收面。
