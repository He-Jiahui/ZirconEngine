---
related_code:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/template/mod.rs
  - zircon_ui/src/template/asset/mod.rs
  - zircon_ui/src/template/asset/document.rs
  - zircon_ui/src/template/asset/loader.rs
  - zircon_ui/src/template/asset/style.rs
  - zircon_ui/src/template/asset/compiler.rs
  - zircon_ui/src/template/asset/legacy.rs
  - zircon_ui/src/template/bridge/layout_contract.rs
  - zircon_ui/src/template/bridge/tree_builder.rs
  - zircon_ui/src/template/bridge/surface_builder.rs
  - zircon_ui/src/template/document.rs
  - zircon_ui/src/tests/asset.rs
  - zircon_asset/src/assets/mod.rs
  - zircon_asset/src/assets/imported.rs
  - zircon_asset/src/assets/ui.rs
  - zircon_asset/src/importer/service.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_editor_ui/src/lib.rs
  - zircon_editor_ui/src/ui_asset_editor.rs
  - zircon_editor_ui/src/tests/activity.rs
  - zircon_editor/src/editing/ui_asset/mod.rs
  - zircon_editor/src/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/editing/ui_asset/inspector_fields.rs
  - zircon_editor/src/editing/ui_asset/session.rs
  - zircon_editor/src/editing/ui_asset/preview_host.rs
  - zircon_editor/src/editing/ui_asset/style_rule_declarations.rs
  - zircon_editor/src/host/manager/project_access.rs
  - zircon_editor/src/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/host/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/host/slint_host/app/callback_wiring.rs
  - zircon_editor/src/host/slint_host/ui/pane_projection.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_asset/src/tests/assets/ui.rs
  - zircon_asset/src/tests/editor/manager.rs
implementation_files:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/template/mod.rs
  - zircon_ui/src/template/asset/mod.rs
  - zircon_ui/src/template/asset/document.rs
  - zircon_ui/src/template/asset/loader.rs
  - zircon_ui/src/template/asset/style.rs
  - zircon_ui/src/template/asset/compiler.rs
  - zircon_ui/src/template/asset/legacy.rs
  - zircon_ui/src/template/bridge/layout_contract.rs
  - zircon_ui/src/template/bridge/tree_builder.rs
  - zircon_ui/src/template/bridge/surface_builder.rs
  - zircon_asset/src/assets/ui.rs
  - zircon_asset/src/importer/service.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_editor_ui/src/lib.rs
  - zircon_editor_ui/src/ui_asset_editor.rs
  - zircon_editor/src/editing/ui_asset/mod.rs
  - zircon_editor/src/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/editing/ui_asset/inspector_fields.rs
  - zircon_editor/src/editing/ui_asset/session.rs
  - zircon_editor/src/editing/ui_asset/preview_host.rs
  - zircon_editor/src/editing/ui_asset/style_rule_declarations.rs
  - zircon_editor/src/host/manager/project_access.rs
  - zircon_editor/src/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/host/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/host/slint_host/app/callback_wiring.rs
  - zircon_editor/src/host/slint_host/ui/pane_projection.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/panes.slint
plan_sources:
  - user: 2026-04-16 增加 activityWindow 界面作为 UI 编辑布局工具并把 UI Layout 资产化
  - user: 2026-04-16 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-16 继续把完整 zircon_editor 宿主实现补完
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
tests:
  - zircon_ui/src/tests/asset.rs
  - zircon_editor_ui/src/tests/activity.rs
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/tests/workbench_slint_shell.rs
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
  - cargo test -p zircon_ui --lib --locked tests::asset
  - cargo test -p zircon_ui --lib --locked
  - cargo test -p zircon_editor_ui --lib --locked tests::activity
  - cargo test -p zircon_editor_ui --lib --locked
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --test workbench_slint_shell --locked
  - cargo test -p zircon_asset --lib --locked
  - cargo build --workspace --locked --verbose
  - cargo test --workspace --locked --verbose
doc_type: module-detail
---

# UI Asset Documents And Editor Protocol

## Purpose

这条文档现在记录的不再只是 shared AST 和最薄的 route 协议，而是已经进入可运行状态的 V1 主链：

- `zircon_ui` 提供正式 `layout/widget/style` AST、legacy adapter、selector stylesheet、component/reference/slot 编译器，以及到 `UiSurface` 的桥接
- `zircon_asset` 把 `.ui.toml` 正式注册为 `UiLayout` / `UiWidget` / `UiStyle` 三种资产，并把 `imports` 转成 editor catalog/reference graph 能消费的依赖
- `zircon_editor_ui` 提供 `editor.ui_asset` window descriptor、route、mode、selection 和 style inspector reflection types
- `zircon_editor` 已经具备真实 `UiAssetEditorSession`、source roundtrip、undo/redo、preview host、recursive import hydration，以及 Slint pane callback 接线

也就是说，当前仓库已经完成了“shared UI asset model -> project asset pipeline -> editor host session -> Slint pane”这一条首个可编辑闭环；真正还未完成的是更高层的可视化拖拽 authoring，而不是基础宿主接线。

## Shared Asset Model

### `UiAssetDocument`

`zircon_ui::template::asset` 现在提供正式的资产文档模型：

- `UiAssetKind::{Layout, Widget, Style}`
- `UiAssetHeader`
- `UiAssetImports`
- `UiAssetRoot`
- `UiNodeDefinition`
- `UiChildMount`
- `UiComponentDefinition`
- `UiComponentParamSchema`
- `UiNamedSlotSchema`
- `UiStyleSheet`
- `UiStyleRule`
- `UiStyleDeclarationBlock`

源码权威格式已经切到 `.ui.toml` 风格的 TOML 结构：

- `[asset]` 提供 `kind/id/version/display_name`
- `[imports]` 注册外部 widget/style 依赖
- `[tokens]` 提供文档级 token
- `[root]` 指向稳定 `node_id`
- `[nodes.*]` 使用注册表式节点表，而不是深度 inline nesting
- `[components.*]` 表示本地复用组件
- `[[stylesheets]]` / `[[stylesheets.rules]]` 表示嵌入式样式表

### Legacy Compatibility

`UiLegacyTemplateAdapter::layout_document(...)` 现在可以把旧 `UiTemplateDocument` 转成新 `UiAssetDocument`：

- legacy `component` 节点映射到 `UiNodeDefinitionKind::Native`
- legacy `template` 调用映射到本地 `component` 实例
- legacy `slot` placeholder 映射到新 `slot` 节点
- `attributes.layout` 被拆回新文档的 `layout`
- 旧 binding ref、`control_id`、children/slot fill 保持稳定

这意味着迁移期可以继续加载旧模板文档，再通过 adapter 进入新编译器，而不是要求现有 editor template 资产一次性全部改写。

## Compiler Flow

### `UiDocumentCompiler`

共享编译器新增了两类注册表：

- `register_widget_import(reference, UiAssetDocument)`
- `register_style_import(reference, UiAssetDocument)`

`compile(&UiAssetDocument)` 的当前固定输出是 `UiCompiledDocument`，它内部已经包含：

1. reference/component/slot 展开后的 `UiTemplateInstance`
2. 经过 stylesheet + inline override 求值后的属性树
3. 原始 `asset` header，便于宿主继续识别 `layout/widget/style`

### Expansion Semantics

当前已实现的实例语义包括：

- `native` 节点直接落成 `UiTemplateNode.component`
- `component` 节点调用当前文档内的本地 `components.*`
- `reference` 节点调用注册过的外部 `widget` 资产组件
- `slot` 节点在编译期用调用方提供的 `mount` 内容替换
- `params` 支持 `$param.*` 形式的常量替换
- 文档或被引用资产的 `tokens` 支持 `$token_name` 常量替换
- reference/component 根实例可以覆盖 `control_id`、追加 `classes`、增加 `style_overrides`

这一步把“React 式组件参数 + Unreal named slot”压成现有 shared runtime 能消费的 `UiTemplateNode` 树，而没有重新起一套并行宿主树。

## Style And Layout Bridge

### Selector Styles

V1 样式系统已经能处理：

- 类型选择器，如 `Button`
- class，如 `.primary`
- id，如 `#OpenButton`
- 状态/作用域 token 的 AST 入口，如 `:hover`、`:host`
- 后代和直接子代组合器

当前样式求值顺序已经固定成：

1. 被引用 widget 资产自带 stylesheet
2. imported 外部 style 资产
3. 当前文档内嵌 stylesheet
4. 节点 inline `style_overrides`

样式规则现在可以同时改：

- `self.*` -> `UiTemplateNode.attributes`
- `slot.*` -> `UiTemplateNode.slot_attributes`

### Slot-Aware Layout Merge

现有 shared layout solver 只有一套 `UiTreeNode.constraints/anchor/pivot/position` 真源，所以 `slot.*` 不能再被丢掉。当前桥接采取了最小过渡策略：

- `UiTemplateNodeMetadata` 保存原始 `slot_attributes`
- `template/bridge/layout_contract.rs` 在读取 layout 时合并 `attributes.layout` 和 `slot_attributes.layout`
- 对线性容器先按“交叉轴优先吃 slot，主轴保留 self”的策略落到当前 solver
- `UiTemplateSurfaceBuilder::build_surface_from_compiled_document(...)` 允许新资产编译结果直接走现有 `UiSurface` 构建

这不是最终的完整 PanelSlot 模型，但已经足够让 V1 资产测试验证：

- imported widget reference 展开
- named slot fill 生效
- stylesheet 命中与优先级正确
- slot height 能进入现有 layout 求解

## Editor Protocol Entry Point

`zircon_editor_ui` 仍然是 `UI Asset Editor` 的协议层入口：

- `UI_ASSET_EDITOR_WINDOW_ID = "editor.ui_asset"`
- `ui_asset_editor_window_descriptor()`
- `UiAssetEditorMode::{Design, Split, Source, Preview}`
- `UiAssetEditorRoute { asset_id, asset_kind, mode }`

当前 descriptor 已经锁住方案要求的窗口级能力：

- `multi_instance = true`
- `supports_document_tab = true`
- `supports_exclusive_page = true`
- `supports_floating_window = true`

在此基础上，`UiDesignerSelectionModel`、`UiStyleInspectorReflectionModel` 和 `UiAssetEditorReflectionModel` 也已经成为 editor/runtime host 之间的稳定反射面。

当前 Slint pane 额外暴露了一组 host-only projection 字段来承载结构化 stylesheet 编辑：

- `style_rule_items`
- `style_rule_selected_index`
- `style_selected_rule_selector`
- `style_can_edit_rule`
- `style_can_delete_rule`

这组字段仍然建立在同一个 `UiAssetDocument` + canonical TOML roundtrip 上，没有引入第二套私有样式文档模型。

## Asset Registration And Host Handoff

`zircon_asset` 现在已经把 UI 资产从“未归类文本文件”提升成正式 imported asset：

- `ImportedAsset::{UiLayout, UiWidget, UiStyle}` 已加入统一 imported asset 枚举
- `AssetImporter::import_from_source(...)` 会识别 `.ui.toml` 并按 `[asset.kind]` 分流
- `ProjectManager::scan_and_import(...)` 会把它们映射到 `AssetKind::{UiLayout, UiWidget, UiStyle}`
- `ui_asset_references(...)` 会把 `imports.widgets` / `imports.styles` 转成 direct references，进入 `EditorAssetManager` 的 catalog/reference graph

对应的 editor 宿主链路也已经接上：

- `EditorManager::open_ui_asset_editor_by_id(...)` 支持 `res://path.ui.toml#Component`
- `project_access.rs` 会在文件解析前规范化 `#Component` 后缀，避免路径解析错误
- `UiAssetEditorSession` 在 `session.rs` 中维护 source buffer、undo stack、selection/style inspector、last-good preview 与 import registry
- `ui_asset_sessions.rs` 会在 open / restore / source update / undo / redo / save 后重新递归 hydration widget/style imports
- `UiAssetEditorSession` 现在还支持：
  - `create_rule_from_selection()`
  - `extract_inline_overrides_to_rule()`
  - `toggle_pseudo_state_preview()`
  - `add_class_to_selection()`
  - `remove_class_from_selection()`
  - `set_selected_widget_control_id()`
  - `set_selected_widget_text_property()`
  - `set_selected_slot_mount()`
  - `set_selected_slot_padding()`
  - `set_selected_slot_width_preferred()`
  - `set_selected_slot_height_preferred()`
  - `set_selected_layout_width_preferred()`
  - `set_selected_layout_height_preferred()`
  - `select_binding()`
  - `add_binding()`
  - `delete_selected_binding()`
  - `set_selected_binding_id()`
  - `set_selected_binding_event()`
  - `set_selected_binding_route()`
  - `select_stylesheet_rule()`
  - `rename_selected_stylesheet_rule()`
  - `delete_selected_stylesheet_rule()`
  - `select_style_token()`
  - `upsert_style_token()`
  - `delete_selected_style_token()`
- `save_ui_asset_editor(...)` 保存 canonical TOML 后会重新触发 `AssetManager::import_asset(...)`

更完整的 editor-only 细节见 [UI Asset Editor Host Session](../editor-and-tooling/ui-asset-editor-host-session.md)。

## Style Authoring Follow-up

`UI Asset Editor` 的 Stylesheet 区现在已经不再是只读摘要：

- pane projection 新增 `can_create_rule` / `can_extract_rule` 以及五个伪状态激活标记
- `panes.slint` 已经增加 `Rule` / `Extract` / `Hover` / `Focus` / `Pressed` / `Disabled` / `Selected` 按钮
- `panes.slint` 现在还会投影当前选中节点的 `style_class_items`，并提供 `class-name` 输入框加 `Add` / `Remove` 动作
- `panes.slint` 现在还会投影本地 `style_rule_items`，提供 `Rules` 列表、selector 输入框，以及 `Apply` / `Delete` 动作
- `panes.slint` 现在还会投影本地 `style_token_items`，提供 `Tokens` 列表、`token-name` / `token-value` 输入框，以及 `Apply` / `Delete` 动作
- `dispatch_ui_asset_action(...)` 会把这些动作映射到 manager/session，而不是再走 source 文本手写编辑
- `dispatch_ui_asset_style_class_action(...)` 会把 `style.class.add` / `style.class.remove` 映射到 manager/session 的 class 编辑 API
- `dispatch_ui_asset_style_rule_action(...)` 会把 `style.rule.select` / `style.rule.rename` / `style.rule.delete` 映射到 manager/session 的本地 rule 编辑 API
- `dispatch_ui_asset_style_rule_declaration_action(...)` 会把 `style.rule.declaration.select` / `style.rule.declaration.upsert` / `style.rule.declaration.delete` 映射到 manager/session 的本地 declaration 编辑 API
- `dispatch_ui_asset_style_token_action(...)` 会把 `style.token.select` / `style.token.upsert` / `style.token.delete` 映射到 manager/session 的本地 token 编辑 API
- 结构化 Inspector 现在已经能直接改写 `control_id`、`props.text`，以及当前父子边上的 `mount`、`slot.padding`、`slot.layout.width.preferred`、`slot.layout.height.preferred`
- 节点自身的公共 layout 约束也已经进入同一条编辑链路：`layout.width.preferred`、`layout.height.preferred`
- 共享 `bindings` 已经被投影成宿主 Inspector 的可选列表，支持绑定项选择、`Add Click`、删除，以及 `id/event/route` 三字段编辑
- 这些 Inspector 字段继续复用同一个 `ui_asset_inspector_widget_action(instance_id, action_id, value)` callback；Slint 只上传 action id，manager/session 再决定它是 widget、slot、layout 还是 binding 字段，binding 列表选中则单独走 `ui_asset_binding_selected(instance_id, item_index)`
- slot 数值字段会在 session 层做 numeric literal 校验；空字符串表示删掉对应 leaf，非法输入不会被悄悄降格成字符串 TOML
- 规则创建默认从当前选中节点生成 selector；优先使用 `#control_id`，否则退回类型 + class 组合
- inline override 提取会把节点上的 `style_overrides` 移入本地 stylesheet rule，然后立刻重建 style inspector
- 伪状态预览只更新 inspector 命中链，不改 source、不改 preview dirty 状态
- class 增删会直接落回 canonical TOML，同时刷新 style inspector 和 Slint pane class 列表
- 本地 rule 删除后会自动把选中索引回退到仍然存在的下一条 rule，避免 pane 进入悬空选择状态
- 选中 rule 后，rule body 会被 flatten 成 `self.*` / `slot.*` dotted path 列表，供 editor 结构化编辑
- declaration 编辑支持把 `set.self` / `set.slot` 的嵌套 table 改写成 leaf path，并在删除后自动回收空 table，保持 canonical TOML 稳定
- 本地 token 删除后会自动把选中项回退到仍然存在的相邻 token；裸字符串输入如 `#223344` 会自动按 TOML string literal 落盘，而 `12` 这类值会保持数值 token

## Validation Evidence

当前这条链路重新验证了：

- `cargo test -p zircon_ui --lib --locked tests::asset`
- `cargo test -p zircon_ui --lib --locked`
- `cargo test -p zircon_editor_ui --lib --locked tests::activity`
- `cargo test -p zircon_editor_ui --lib --locked`
- `cargo test -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked`
- `cargo test -p zircon_asset --lib --locked`
- `cargo build --workspace --locked --verbose`
- `cargo test --workspace --locked --verbose`

这轮样式 authoring 跟进额外验证了：

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

最新一轮稳定可重复的验证证据仍然是：

- `cargo check -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_binding_inspector_editing_controls`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_pane_declares_`

更宽的 `cargo test -p zircon_editor --lib --locked ...` 过滤在当前 dirty workspace 下会被别处的并行改动间歇性打断，例如 `slint_viewport_toolbar_pointer.rs` 对 `BuiltinViewportToolbarTemplateBridge::recompute_layout(...)` 的外部编译错误；这不是 `ui_asset` editor host 本轮新增的失败，但会影响把 unit-test 过滤结果作为唯一验证证据。

为了让这些验证恢复可运行，本轮还做了最小兼容修复：

- `zircon_graphics/src/visibility/*` 的 helper 可见性补线
- `zircon_editor/src/host/slint_host/app/native_windows.rs` 与 `app.rs` 的 native-window re-export 可见性补线
- `zircon_editor/src/editing/state/*` 的缺失导入修复

这些验证覆盖了 shared asset/compiler、本地 component/reference 展开、正式资产注册、editor catalog/reference graph、session source roundtrip、preview projection 和 Slint pane callback 声明。

## Remaining Work

当前剩下的工作已经从“先把资产和宿主接起来”变成“补高层 authoring 体验”：

- 可视化 designer canvas 的真实 frame/slot overlay、拖拽插入和重排
- Palette 拖入创建节点和引用节点
- Layout / Bindings / parent-specific slot 语义的更完整结构化编辑；当前只覆盖 widget 基础字段、preferred-size 级 layout 字段和基础 binding/id/event/route 编辑
- Stylesheet rule body 的更高层结构化编辑，以及跨 asset token/theme 视图
- `Wrap/Unwrap`、`Extract Component`、`Promote External Widget Asset`、`Convert To Reference`
- 更细粒度的结构化 undo/redo，而不仅是当前 source 文本级撤销
- runtime/editor 对更多现有 screen/window 的正式迁移，以及 `UI Asset Editor` 自举
