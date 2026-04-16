---
related_code:
  - zircon_editor_ui/src/ui_asset_editor.rs
  - zircon_editor/src/editing/ui_asset/mod.rs
  - zircon_editor/src/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/editing/ui_asset/command.rs
  - zircon_editor/src/editing/ui_asset/inspector_fields.rs
  - zircon_editor/src/editing/ui_asset/presentation.rs
  - zircon_editor/src/editing/ui_asset/preview_host.rs
  - zircon_editor/src/editing/ui_asset/session.rs
  - zircon_editor/src/editing/ui_asset/source_buffer.rs
  - zircon_editor/src/editing/ui_asset/style_rule_declarations.rs
  - zircon_editor/src/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/host/manager/project_access.rs
  - zircon_editor/src/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/host/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/host/slint_host/app/callback_wiring.rs
  - zircon_editor/src/host/slint_host/ui/pane_projection.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_asset/src/assets/mod.rs
  - zircon_asset/src/assets/imported.rs
  - zircon_asset/src/assets/ui.rs
  - zircon_asset/src/importer/service.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_asset/src/tests/assets/ui.rs
  - zircon_asset/src/tests/editor/manager.rs
implementation_files:
  - zircon_editor_ui/src/ui_asset_editor.rs
  - zircon_editor/src/editing/ui_asset/mod.rs
  - zircon_editor/src/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/editing/ui_asset/command.rs
  - zircon_editor/src/editing/ui_asset/inspector_fields.rs
  - zircon_editor/src/editing/ui_asset/presentation.rs
  - zircon_editor/src/editing/ui_asset/preview_host.rs
  - zircon_editor/src/editing/ui_asset/session.rs
  - zircon_editor/src/editing/ui_asset/source_buffer.rs
  - zircon_editor/src/editing/ui_asset/style_rule_declarations.rs
  - zircon_editor/src/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/host/manager/project_access.rs
  - zircon_editor/src/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/host/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/host/slint_host/app/callback_wiring.rs
  - zircon_editor/src/host/slint_host/ui/pane_projection.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_asset/src/assets/ui.rs
  - zircon_asset/src/importer/service.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/editor/manager.rs
plan_sources:
  - user: 2026-04-16 增加 activityWindow 界面作为 UI 编辑布局工具并把 UI Layout 资产化
  - user: 2026-04-16 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-16 继续把完整 zircon_editor 宿主实现补完
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
tests:
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor_ui/src/tests/ui_asset_editor.rs
  - zircon_editor_ui/src/tests/activity.rs
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
  - cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_widget_inspector_editing_actions
  - cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_slot_inspector_editing_actions
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_widget_inspector_editing_controls_and_callback
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_slot_inspector_editing_controls
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --test workbench_slint_shell --locked
  - cargo test -p zircon_editor_ui --lib --locked
  - cargo test -p zircon_asset --lib --locked
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

这意味着当前仓库里已经有“共享资产格式 -> editor 资产系统 -> session -> Slint host pane”这一条可运行主链。

## Responsibility Split

- `zircon_ui`
  - 仍然是 `UiAssetDocument`、`UiDocumentCompiler`、`UiCompiledDocument`、`UiSurface` 的权威层
  - 不关心 `ActivityWindow`、document tab、save/undo/redo
- `zircon_asset`
  - 负责把 `.ui.toml` 导入成 `UiLayoutAsset`、`UiWidgetAsset`、`UiStyleAsset`
  - 负责把 UI import 引用变成 editor catalog/reference graph 可以消费的 `AssetReference`
- `zircon_editor_ui`
  - 定义 `UiAssetEditorRoute`、`UiAssetEditorMode`、`UiAssetEditorReflectionModel`
  - 不实现文件读写和 preview host
- `zircon_editor`
  - 负责 session、source roundtrip、import hydration、canonical save、pane presentation、Slint callback dispatch
- `Slint`
  - 只承接 pane 展示和 callback 上传
  - 不自己解释 `.ui.toml`，也不自己成为布局真源

## Asset Registration And Project Catalog

`zircon_asset` 现在把 UI 资产作为正式 imported asset 分支处理：

- `ImportedAsset::{UiLayout, UiWidget, UiStyle}` 已加入 `zircon_asset/src/assets/imported.rs`
- `AssetImporter::import_from_source(...)` 会对 `.ui.toml` 先后尝试 `UiLayoutAsset::from_toml_str(...)`、`UiWidgetAsset::from_toml_str(...)`、`UiStyleAsset::from_toml_str(...)`
- `ProjectManager::scan_and_import(...)` 会把三类 UI 资产映射到 `AssetKind::{UiLayout, UiWidget, UiStyle}`
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
- save 统一走 canonical TOML serializer，保存后重置 dirty 标志
- undo/redo 目前按 source 文本编辑粒度工作，不做结构化 tree command
- 选中节点后可以一键把当前节点投影成 selector rule，并直接落回 canonical source
- inline `style_overrides` 可以提取成 stylesheet rule，同时清空节点上的 inline block
- `hover/focus/pressed/disabled/selected` 伪状态预览只更新 style inspector，不落盘、不污染 source dirty
- 选中节点的 `classes` 现在可以通过 session API 直接追加/删除，并保持 canonical source、style inspector 和 Slint pane 同步
- 本地 stylesheet rule 现在可以在 session 内被选中、改写 selector 和删除；选中的 rule 会跨 source roundtrip 保持稳定索引并在删除后自动回退到下一个可用 rule
- 文档本地 `tokens` 现在可以在 session 内被选中、改名/改值和删除；token 编辑同样直接回写 canonical source，并在删除后自动回退到最近仍存在的 token
- 结构化 Inspector 已经能直接编辑选中节点的 `control_id` 与 `props.text`，并保持 canonical source、preview tree 和 Slint pane 同步
- 同一条 Inspector 链路现在还支持编辑当前父子边上的公共 slot 字段：`mount`、`slot.padding`、`slot.layout.width.preferred`、`slot.layout.height.preferred`
- 选中节点自身的公共 layout 字段现在也可结构化编辑：`layout.width.preferred`、`layout.height.preferred`
- 共享 `bindings` 现在已经接入宿主 Inspector，可对选中节点的 binding 列表执行选择、`Add Click`、删除，以及 `id/event/route` 三字段编辑
- slot 数值字段会在 session 层做 numeric literal 校验；空字符串表示删除对应 leaf，非法非数值输入会返回结构化错误

## Manager Lifecycle

`EditorManager` 的 `ui_asset_sessions.rs` 现在负责整个实例生命周期：

- `open_ui_asset_editor_by_id(...)`
  - 允许 `res://path.ui.toml#Component` 形式输入
  - `normalize_ui_asset_asset_id(...)` 会在文件解析前去掉 `#Component`，避免把组件后缀当成真实文件名
- `restore_ui_asset_editor_instance(...)`
  - 允许从已序列化 route 或旧 payload 恢复
- `save_ui_asset_editor(...)`
  - 写回 canonical source
  - 对 `res://` 资产立即调用 `AssetManager::import_asset(...)`
- `undo_ui_asset_editor(...)` / `redo_ui_asset_editor(...)`
- `set_ui_asset_editor_mode(...)`
- `select_ui_asset_editor_hierarchy_index(...)`
- `select_ui_asset_editor_preview_index(...)`
- `create_ui_asset_editor_rule_from_selection(...)`
- `extract_ui_asset_editor_inline_overrides_to_rule(...)`
- `toggle_ui_asset_editor_pseudo_state(...)`
- `add_ui_asset_editor_class_to_selection(...)`
- `remove_ui_asset_editor_class_from_selection(...)`
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

## Pane Presentation And Slint Callbacks

`UiAssetEditorPane` 现在已经是 workbench 里的真实 pane，而不是 placeholder 文本。

当前 pane 暴露的数据面包括：

- `palette_items`
- `hierarchy_items`
- `preview_items`
- `inspector_items`
- `style_class_items`
- `stylesheet_items`
- `source_text`
- 结构化 inspector 字段：`inspector_selected_node_id`、`inspector_parent_node_id`、`inspector_mount`、`inspector_control_id`、`inspector_text_prop`、`inspector_slot_padding`、`inspector_slot_width_preferred`、`inspector_slot_height_preferred`、`inspector_layout_width_preferred`、`inspector_layout_height_preferred`
- binding inspector 字段：`inspector_binding_items`、`inspector_binding_selected_index`、`inspector_binding_id`、`inspector_binding_event`、`inspector_binding_route`

这对应方案里的六区基础骨架：

- Palette
- Hierarchy
- Designer/Preview 区
- Inspector
- Stylesheet 区
- Source 区

当前 `panes.slint` 与 `workbench.slint` 已经打通一组真实交互 callback：

- `ui_asset_action(instance_id, action_id)`
- `ui_asset_style_class_action(instance_id, action_id, class_name)`
- `ui_asset_style_rule_action(instance_id, action_id, item_index, selector)`
- `ui_asset_style_token_action(instance_id, action_id, item_index, token_name, token_value)`
- `ui_asset_source_edited(instance_id, value)`
- `ui_asset_hierarchy_selected(instance_id, item_index)`
- `ui_asset_preview_selected(instance_id, item_index)`
- `ui_asset_binding_selected(instance_id, item_index)`

宿主侧 `app/ui_asset_editor.rs` 目前支持的动作集合是：

- `save`
- `undo`
- `redo`
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

额外的 stylesheet rule 编辑通过独立 callback 进入宿主：

- `style.rule.select`
- `style.rule.rename`
- `style.rule.delete`

token 编辑也通过独立 callback 进入宿主：

- `style.token.select`
- `style.token.upsert`
- `style.token.delete`

rule declaration 编辑也通过独立 callback 进入宿主：

- `style.rule.declaration.select`
- `style.rule.declaration.upsert`
- `style.rule.declaration.delete`

结构化 Inspector 编辑当前复用同一条宿主 callback：

- `ui_asset_inspector_widget_action(instance_id, action_id, value)`
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

这样 Slint 不需要再为 slot/layout/binding 各自引入私有 ABI；宿主只根据 action id 路由到 manager/session 的字段编辑 API，而 binding 列表选择则走 `ui_asset_binding_selected(...)`。

`Source` 区已经使用 multiline `TextEdit`，不是单行输入框。Hierarchy 与 preview 列表也都可以反向驱动 session 选中状态。

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
- pane projection 会把 rule availability 和当前伪状态活跃标志显式投影到 Slint `PaneData`
- declaration path 解析支持 `self.*` 和 `slot.*`，并会把嵌套 TOML table flatten 成可编辑的 dotted path；删除 leaf 后会自动回收空 table，保持 canonical source 简洁

## Preview Projection

`UiAssetPreviewHost` 当前直接消费 `UiCompiledDocument`：

- 通过 `UiTemplateSurfaceBuilder::build_surface_from_compiled_document(...)` 构建 shared `UiSurface`
- 立刻执行 `compute_layout(preview_size)`
- `preview_items` 读取 shared `render_extract` 和 `template_metadata`

为了让 reference widget 场景可读，preview 列表不再只显示最终渲染出来的底层 `Button` 类型：

- 如果当前 preview node 对应文档里的 `component_ref = "...#ToolbarButton"`，列表会优先显示 `ToolbarButton`
- 如果底层实际渲染组件类型和文档组件身份不同，会显示 `ToolbarButton/Button` 这类组合标签

这让 preview 列表既能保持对宿主节点的可追踪性，也不会丢失 shared surface 的真实渲染节点类型。

## Current V1 Limits

当前实现完成了“宿主闭环”，并且已经有第一批结构化 Inspector 编辑，但还没有完成整个产品计划。

仍未落地的高层 authoring 能力包括：

- 真正的可视化 canvas frame/slot overlay 与拖拽重排
- Palette 拖入创建节点
- Wrap/Unwrap、Extract Component、Promote To External Widget Asset、Convert To Reference
- Layout / Bindings / parent-specific slot 字段的更完整结构化编辑；当前只覆盖 widget 基础字段、preferred-size 级 layout 字段和基础 binding/id/event/route 编辑
- Stylesheet selector/slot/self 的更高层结构化编辑，以及跨 asset token/theme 视图
- 结构化 undo/redo，而不是当前的 source 文本级 undo/redo

所以当前阶段应把它视为“正式可打开/编辑/保存/预览 UI 资产的宿主骨架”，而不是已经具备完整 Widget Blueprint 级 authoring 体验。

## Validation Evidence

此前主链基线已经完整验证过：

- `cargo test -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked`
- `cargo test -p zircon_editor_ui --lib --locked`
- `cargo test -p zircon_asset --lib --locked`
- `cargo build --workspace --locked --verbose`
- `cargo test --workspace --locked --verbose`

这轮样式 authoring 跟进已经新增验证：

- `cargo check -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --lib ui_asset_editor_session_creates_stylesheet_rule_from_selected_node --locked`
- `cargo test -p zircon_editor --lib ui_asset_editor_session_adds_and_removes_selection_classes --locked`
- `cargo test -p zircon_editor --lib ui_asset_editor_session_selects_renames_and_deletes_local_stylesheet_rules --locked`
- `cargo test -p zircon_editor --lib ui_asset_editor_session_upserts_and_deletes_local_tokens --locked`
- `cargo test -p zircon_editor --lib ui_asset_editor_session_selects_upserts_and_deletes_stylesheet_rule_declarations --locked`
- `cargo test -p zircon_editor --lib editor_manager_runs_ui_asset_style_class_editing_actions --locked`
- `cargo test -p zircon_editor --lib editor_manager_runs_ui_asset_style_rule_editing_actions --locked`
- `cargo test -p zircon_editor --lib editor_manager_runs_ui_asset_style_token_editing_actions --locked`
- `cargo test -p zircon_editor --lib editor_manager_runs_ui_asset_style_rule_declaration_editing_actions --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_authoring_buttons_and_state_bindings --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_class_authoring_controls_and_callback --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_rule_editing_controls_and_callback --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_token_editing_controls_and_callback --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell ui_asset_editor_pane_declares_style_rule_declaration_editing_controls_and_callback --locked`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_projects_matched_style_rules_into_stylesheet_summary_items`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_selects_matched_style_rules_and_projects_details`
- `cargo test -p zircon_editor --lib --locked editor_manager_projects_matched_style_rule_summaries_into_stylesheet_items`
- `cargo test -p zircon_editor --lib --locked editor_manager_projects_selected_matched_style_rule_details`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_matched_rule_inspection_controls_and_callback`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_projects_structured_widget_inspector_fields`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_updates_selected_widget_inspector_fields`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_projects_structured_slot_inspector_fields`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_updates_selected_slot_inspector_fields`
- `cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_widget_inspector_editing_actions`
- `cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_slot_inspector_editing_actions`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_widget_inspector_editing_controls_and_callback`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_slot_inspector_editing_controls`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_projects_structured_layout_inspector_fields`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_session_updates_selected_layout_inspector_fields`
- `cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_layout_inspector_editing_actions`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_layout_inspector_editing_controls`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_binding_inspector_editing_controls`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_`

最新一轮 `layout` / `bindings` Inspector 跟进里，稳定可重复的验证证据是：

- `cargo check -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_binding_inspector_editing_controls`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_`

unit-test crate 的更宽过滤在当前 dirty workspace 下会被其他并行改动间歇性打断，例如 `slint_viewport_toolbar_pointer.rs` 里对 `BuiltinViewportToolbarTemplateBridge::recompute_layout(...)` 的外部编译错误；这不是本轮 `ui_asset` host slice 引入的失败，但会暂时限制 `cargo test -p zircon_editor --lib --locked` 作为这条 slice 的单独验证入口。

为了让这些验证重新绿灯，这轮还顺手确认并恢复了 dirty workspace 下 `viewport/controller` 模块分拆后的 crate 级编译可达性，但没有接管那条外部重构的功能边界：

- `cargo check -p zircon_editor --lib --locked` 重新可运行
