---
related_code:
  - zircon_editor_ui/src/ui_asset_editor.rs
  - zircon_editor/src/editing/ui_asset/mod.rs
  - zircon_editor/src/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/editing/ui_asset/command.rs
  - zircon_editor/src/editing/ui_asset/inspector_fields.rs
  - zircon_editor/src/editing/ui_asset/preview_mock.rs
  - zircon_editor/src/editing/ui_asset/presentation.rs
  - zircon_editor/src/editing/ui_asset/preview_host.rs
  - zircon_editor/src/editing/ui_asset/preview_projection.rs
  - zircon_editor/src/editing/ui_asset/palette_drop.rs
  - zircon_editor/src/editing/ui_asset/session.rs
  - zircon_editor/src/editing/ui_asset/promote_widget.rs
  - zircon_editor/src/editing/ui_asset/source_buffer.rs
  - zircon_editor/src/editing/ui_asset/source_sync.rs
  - zircon_editor/src/editing/ui_asset/style_rule_declarations.rs
  - zircon_editor/src/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/host/manager/workspace_state.rs
  - zircon_editor/src/host/manager/builtin_views/activity_views/mod.rs
  - zircon_editor/src/host/manager/builtin_views/activity_windows/mod.rs
  - zircon_editor/src/host/manager/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/host/manager/project_access.rs
  - zircon_editor/src/host/manager/ui_asset_promotion.rs
  - zircon_editor/src/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/host/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/host/slint_host/app/pointer_layout.rs
  - zircon_editor/src/host/slint_host/floating_window_projection.rs
  - zircon_editor/src/host/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/host/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/host/slint_host/app/callback_wiring.rs
  - zircon_editor/src/host/slint_host/app/tests.rs
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
  - zircon_editor/src/tests/editing/ui_asset_palette_drop.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/src/tests/host/slint_tab_drag.rs
  - zircon_editor/src/tests/host/template_runtime.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_asset/src/tests/assets/ui.rs
  - zircon_asset/src/tests/editor/manager.rs
implementation_files:
  - zircon_editor_ui/src/ui_asset_editor.rs
  - zircon_editor/src/editing/ui_asset/mod.rs
  - zircon_editor/src/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/editing/ui_asset/command.rs
  - zircon_editor/src/editing/ui_asset/inspector_fields.rs
  - zircon_editor/src/editing/ui_asset/preview_mock.rs
  - zircon_editor/src/editing/ui_asset/presentation.rs
  - zircon_editor/src/editing/ui_asset/preview_host.rs
  - zircon_editor/src/editing/ui_asset/preview_projection.rs
  - zircon_editor/src/editing/ui_asset/palette_drop.rs
  - zircon_editor/src/editing/ui_asset/session.rs
  - zircon_editor/src/editing/ui_asset/promote_widget.rs
  - zircon_editor/src/editing/ui_asset/source_buffer.rs
  - zircon_editor/src/editing/ui_asset/source_sync.rs
  - zircon_editor/src/editing/ui_asset/style_rule_declarations.rs
  - zircon_editor/src/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/host/manager/workspace_state.rs
  - zircon_editor/src/host/manager/builtin_views/activity_views/mod.rs
  - zircon_editor/src/host/manager/builtin_views/activity_windows/mod.rs
  - zircon_editor/src/host/manager/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/host/manager/project_access.rs
  - zircon_editor/src/host/manager/ui_asset_promotion.rs
  - zircon_editor/src/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/host/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/host/slint_host/app/pointer_layout.rs
  - zircon_editor/src/host/slint_host/floating_window_projection.rs
  - zircon_editor/src/host/slint_host/shell_pointer/bridge.rs
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
  - user: 2026-04-17 继续缺漏内容补充
  - user: 2026-04-17 下一条最合理的 task 是把 Promote To External Widget Asset 接上
  - user: 2026-04-18 给多命名 slot / 低语义 slot 增加真正的 manual slot picker 或 target cycle，并恢复 zircon_editor --lib host/template 回归基线
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
tests:
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/editing/ui_asset_palette_drop.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/src/host/slint_host/app/tests/floating_window_projection.rs
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
- editor-only mock preview override 现在也走正式会话链路，不写回 `.ui.toml`

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
- undo/redo 现在分两条恢复路径：普通 source edit 仍使用 source snapshot；tree authoring edit 会额外保存结构化 `UiAssetEditorTreeEdit + before/after UiAssetDocument`，恢复时优先直接回放文档状态并重建 preview/inspector，而不是重新把 source 当成唯一真源
- 结构化 tree edit 目前已经覆盖 `InsertPaletteItem`、`MoveNode`、`ReparentNode`、`WrapNode`、`UnwrapNode`、`ConvertToReference`、`ExtractComponent`、`PromoteToExternalWidget`；这一步还是 snapshot-based，不是最终的 inverse command log
- 选中节点后可以一键把当前节点投影成 selector rule，并直接落回 canonical source
- inline `style_overrides` 可以提取成 stylesheet rule，同时清空节点上的 inline block
- `hover/focus/pressed/disabled/selected` 伪状态预览只更新 style inspector，不落盘、不污染 source dirty
- preview root 约束预设现在已经接入会话路由：`Editor Docked`、`Editor Floating`、`Game HUD`、`Dialog`
- preview preset 属于 editor-only 会话状态，不写回 `.ui.toml`，但会跟随 `UiAssetEditorRoute` 一起持久化到 view payload，便于宿主恢复
- preview mock state 现在也是 editor-only：session 会投影可 mock 的 `props`，支持 `Text/Bool/Enum/Resource` 四类浅量值覆盖，重建 preview surface 但不修改 source buffer
- 选中节点的 `classes` 现在可以通过 session API 直接追加/删除，并保持 canonical source、style inspector 和 Slint pane 同步
- palette 当前选中项被拖进 Designer Canvas 时，session 现在会按 preview surface 坐标解析“当前 hover 到哪一个 preview frame”，并把瞬时落点投影成 `palette_drag_target_preview_index`、`palette_drag_target_action`、`palette_drag_target_label`，同时在内部解析结构化 `UiAssetPaletteInsertPlan + UiAssetPaletteInsertionPlacement`
- 当 hovered target 是共享原生容器时，session 不再只做 label-aware 提示，而是会真正合成 slot placement：`Overlay` 自动写入 `slot.layout.anchor/pivot/position`，`GridBox` 自动写入 `slot.row/column`，`FlowBox` 自动写入 `slot.break_before/alignment`
- 当 hovered target 是本地 `component` 实例或 external widget `reference` 实例时，session 也会按组件 slot schema 解析可落点 mount，不再只覆盖共享原生容器；对多命名 slot 或低语义 slot，session 会先投影显式 slot region overlay，再优先用这些 overlay 的几何命中结果决定 mount，只有没有命中任何显式 region 时才回退到 slot 名语义推断
- 这意味着当前已经有一版真正可手动选择的 slot picker：用户把 palette 拖到哪个显式 slot region，就落到哪个 mount；`slot_a/slot_b/slot_c` 这类低语义组件不再只能靠名字猜测
- 对 overlay 太小、区域重叠或低语义 slot 很多的情况，session 现在还会把当前 hover 上下文展开成结构化 `UiAssetPaletteDragResolution { candidates, selected_index }`，并投影成 `palette_drag_candidate_items + palette_drag_candidate_selected_index`；几何命中只决定默认候选，真正 drop 会使用当前被 cycle 选中的 candidate
- `cycle_palette_drag_target_candidate_previous/next()` 已经接入会话状态，manual selection 会在 hover 上下文稳定时保持，不会被同一组候选的后续 pointer move 立刻覆盖；只有候选集合真的变了、选中切换、source roundtrip、undo/redo restore 或文档重新验证时，palette drag resolution 才会被清空/重建
- 这条 palette drag 目标状态不是 source 权威数据；它会在 selection 切换、source roundtrip、undo/redo snapshot restore 和文档重新验证后自动清空，避免上一次 hover 命中结果污染后续 authoring
- `UiAssetCanvasSurface` 对 palette 外部拖拽也不再只亮一圈 frame border；当前 child drop 会绘制内部 slot-aware `drop_inside_overlay`，after drop 会绘制独立的 `drop_after_overlay`，并且当宿主拿到结构化 slot target 时，会继续绘制 `palette_drag_slot_target_items` 驱动的显式 `Grid per-cell`、`Overlay per-anchor`、`Flow per-line/alignment` 与 semantic named-slot region overlays
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

`EditorManager` 的 `ui_asset_sessions.rs` 现在负责整个实例生命周期：

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
- 因此当前 V1 的 undo/redo 只回滚 session 内部文档改写，不会自动删除已经创建的外部 widget 文件；即使 tree edit 已切到结构化文档恢复，这个限制仍然保留

## Pane Presentation And Slint Callbacks

`UiAssetEditorPane` 现在已经是 workbench 里的真实 pane，而不是 placeholder 文本。

当前 pane 暴露的数据面包括：

- `palette_items`
- `hierarchy_items`
- `preview_items`
- `preview_canvas_items`
- `palette_drag_candidate_items`
- `palette_drag_candidate_selected_index`
- `preview_preset`
- `preview_mock_items`
- `source_outline_items`
- `inspector_items`
- `style_class_items`
- `stylesheet_items`
- `source_text`
- action availability：`can_save`、`can_undo`、`can_redo`、`can_insert_child`、`can_insert_after`、`can_move_up`、`can_move_down`、`can_reparent_into_previous`、`can_reparent_into_next`、`can_reparent_outdent`、`can_open_reference`、`can_convert_to_reference`、`can_extract_component`、`can_promote_to_external_widget`、`can_wrap_in_vertical_box`、`can_unwrap`、`can_create_rule`、`can_extract_rule`
- 结构化 inspector 字段：`inspector_selected_node_id`、`inspector_parent_node_id`、`inspector_mount`、`inspector_control_id`、`inspector_text_prop`、`inspector_slot_padding`、`inspector_slot_width_preferred`、`inspector_slot_height_preferred`、`inspector_layout_width_preferred`、`inspector_layout_height_preferred`
- promote inspector 字段：`inspector_promote_asset_id`、`inspector_promote_component_name`、`inspector_promote_document_id`、`inspector_can_edit_promote_draft`
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
- `ui_asset_palette_selected(instance_id, item_index)`
- `ui_asset_palette_drag_hover(instance_id, surface_x, surface_y)`
- `ui_asset_palette_drag_drop(instance_id)`
- `ui_asset_palette_drag_cancel(instance_id)`
- `ui_asset_hierarchy_selected(instance_id, item_index)`
- `ui_asset_inspector_widget_action(instance_id, "promote.asset_id.set" | "promote.component_name.set" | "promote.document_id.set", value)`
- `ui_asset_preview_selected(instance_id, item_index)`
- `ui_asset_preview_mock_selected(instance_id, item_index)`
- `ui_asset_preview_mock_action(instance_id, action_id, value)`
- `ui_asset_binding_selected(instance_id, item_index)`

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

`Source` 区已经使用 multiline `TextEdit`，不是单行输入框。当前 Source 面板还新增了 `Source Outline`，会按当前 source buffer 中实际存在的 `[nodes.*]` block 和 line number 投影 outline 列表；Hierarchy、preview 列表、preview canvas 和 source outline 都已经汇合到同一条 session 选中链路。

这一轮补上的 palette target-cycle / picker 闭环是：

- `UiAssetEditorPanePresentation` / `PaneData` / `UiAssetEditorPane` 已经把 `palette_drag_candidate_items` 和 `palette_drag_candidate_selected_index` 显式投影到 Slint pane
- `panes.slint` 在 palette 外部拖拽 overlay 里新增了 `Target Cycle` 面板；当候选数大于 1 时，会显示当前可落点列表，而不是只显示一个几何默认 label
- overlay 内的 `FocusScope` 已接入键盘轮换：`Left/Up` 选上一个，`Right/Down/Tab` 选下一个，`Enter` 应用当前 candidate，`Escape` 取消 palette drag
- `drop_selected_palette_item_at_palette_drag_target()` 现在不再盲目依赖 hover 默认值，而是会读取当前 selected candidate 对应的 `UiAssetPaletteInsertPlan`

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
- pane projection 会把 rule availability 和当前伪状态活跃标志显式投影到 Slint `PaneData`
- declaration path 解析支持 `self.*` 和 `slot.*`，并会把嵌套 TOML table flatten 成可编辑的 dotted path；删除 leaf 后会自动回收空 table，保持 canonical source 简洁

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
- 结构化 bindings inspector
- parent-specific slot/layout semantic inspector
- `Open Ref` 打开外部 widget 引用源资源，以及 hierarchy 双击激活该导航
- preview 双击激活 `Open Ref`
- preview root surface presets：`Editor Docked`、`Editor Floating`、`Game HUD`、`Dialog`
- editor-only mock preview：文本/布尔/枚举/资源引用四类浅量 `props` override

仍未落地的高层 authoring 能力包括：

- palette drag 目前仍是“拖拽期间的瞬时 picker”，不是 drop 后可继续停留的 sticky/manual chooser；当用户希望先释放鼠标再精确改落点，仍缺少更重型的 target picker 工作流
- Layout / Bindings / parent-specific slot 字段的更完整结构化编辑；当前只覆盖 widget 基础字段、preferred-size 级 layout 字段和基础 binding/id/event/route 编辑
- Stylesheet selector/slot/self 的更高层结构化编辑，以及跨 asset token/theme 视图
- preview mock 目前只覆盖节点 `props` 的浅量 scalar/string 覆盖，不涉及表达式绑定、集合型数据或跨节点 state graph
- 更细粒度的 tree-command undo/redo 执行后端；当前虽然 tree edit 已从纯 source 回放推进到结构化文档 snapshot 恢复，但真正的 tree-diff / inverse command-log 回放仍未落地

所以当前阶段应把它视为“正式可打开/编辑/保存/预览 UI 资产的宿主骨架”，而不是已经具备完整 Widget Blueprint 级 authoring 体验。

## Validation Evidence

此前主链的宿主/资产闭环已经做过宽验证，至少包括：

- `cargo test -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked`
- `cargo test -p zircon_editor_ui --lib --locked`
- `cargo test -p zircon_asset --lib --locked`
- `cargo build --workspace --locked --verbose`
- `cargo test --workspace --locked --verbose`

本轮 selected-frame canvas overlay 跟进的新增证据是：

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
