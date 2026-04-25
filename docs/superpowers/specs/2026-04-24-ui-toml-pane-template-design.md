---
related_code:
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/Physics + Full Animation Support 新计划.md
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor_builder.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_context.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_scene.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_editor/src/ui/slint_host/app/hierarchy_pointer.rs
  - zircon_editor/src/ui/slint_host/app/detail_scroll_pointer.rs
implementation_files:
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor_builder.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_context.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_scene.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
plan_sources:
  - user: ui.toml描述代替slint布局做窗口
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/Physics + Full Animation Support 新计划.md
tests:
  - cargo test -p zircon_editor --lib --locked --offline
  - cargo test -p zircon_editor --lib --locked template_runtime --offline
  - cargo test -p zircon_editor --lib --locked slint_hierarchy_template_body --offline
  - cargo test -p zircon_editor --lib --locked slint_animation_template_body --offline
  - cargo test -p zircon_editor --lib --locked root_workbench_slint_exports_only_generic_host_bootstrap_symbols --offline
  - cargo test -p zircon_editor --lib --locked slint_host_presentation_uses_generic_scene_data_property --offline
  - cargo test -p zircon_editor --lib --locked slint_host_scene_uses_generic_surface_metrics_and_orchestration_names --offline
  - cargo test -p zircon_editor --lib --locked slint_host_drag_and_resize_callbacks_use_generic_host_event_names --offline
  - cargo test -p zircon_editor --lib --locked host_page_pointer_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked slint_window --offline
  - cargo test -p zircon_editor --lib --locked slint_host --offline
  - cargo fmt --all
  - cargo check -p zircon_editor --locked --offline
doc_type: milestone-detail
---
# UI TOML Pane Template Design

## 背景

当前 `zircon_editor` 已经具备 `.ui.toml -> template runtime -> host projection -> Slint` 的主链；2026-04-24 cutover 后，首批 pane body 的内容定义权已经从 Rust/Slint 手工拼装层迁到 `PanePresentation` 和 `.ui.toml` 模板层，剩余 Slint DTO 只作为宿主兼容壳存在。

同一轮后续推进还开始了 root Slint generic host boundary 的第一刀：`workbench.slint` 的 bootstrap 导出已收口到 `UiHostContext`、`UiHostScaffold` 和 `HostWindowSceneData`，scene contract 也收口到 `host_scene_data`、`HostWindowSurfaceMetricsData` 和 `HostWindowSurfaceOrchestrationData`，通用拖拽/resize 宿主事件收口到 `host_drag_pointer_event` 和 `host_resize_pointer_event`，Rust host-page pointer helper 收口到 `HostPagePointer*` 和 `build_host_page_pointer_layout`，并由 `root_workbench_slint_exports_only_generic_host_bootstrap_symbols`、`slint_host_presentation_uses_generic_scene_data_property`、`slint_host_scene_uses_generic_surface_metrics_and_orchestration_names`、`slint_host_drag_and_resize_callbacks_use_generic_host_event_names`、`host_page_pointer_module_uses_generic_host_type_names` 防止 root/scene/host-event/host-page-pointer 文件重新暴露 workbench 专名。这一步只冻结宿主入口、scene DTO、通用 host event 和 host-page pointer 命名，不等价于 menu/drawer/document/floating 业务结构已经迁出。

现状中的两个事实同时成立：

- `.ui.toml` 已经能描述 editor host 的壳结构、route 声明和一部分 surface controls，见 `zircon_editor/assets/ui/editor/host/*.ui.toml`
- docking、split、drawer、floating window、focus、preset 等生命周期仍然明确属于 `zircon_editor::ui::workbench` 与 Slint host，而不是模板 runtime

当前内容拼装权主要集中在这些位置：

- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs`
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`
- `zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs`
- `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs`

这导致 pane body 的结构、字段、模板选择和 Slint 专用 DTO 仍然强耦合在宿主层，`.ui.toml` 只能描述一层薄壳，而不能成为 pane body 的权威来源。

## 目标

本设计的目标是把 `.ui.toml` 提升为 `pane body / local shell / route declaration` 的声明式权威，同时保持现有 editor host 的生命周期架构不变。

本轮目标固定为：

- `.ui.toml` 成为 pane body 和局部 shell 的布局权威
- `ViewDescriptor` 成为 pane 模板语义的注册入口
- `PaneData` 从 giant union 收敛为 `shell + body` 两段结构；实现态中 `PaneBodyCompatData` 只承载 Slint ABI 和 native slot 兼容字段
- 保留现有 `EditorUiBindingPayload` 命令体系，不引入第二套模板命令系统
- 允许高交互区域通过 hybrid native slot 挂接，而不是强行在第一轮改写控件运行时

## 非目标

以下内容明确不在本轮范围：

- 让 `.ui.toml` 接管 docking、split、floating window、drawer extent、focus、preset 生命周期
- 移除 Slint 作为 native renderer 和平台窗口宿主
- 为 hierarchy tree、animation timeline、graph canvas 重写底层交互运行时
- 用开放式 `Value/Table` 或字符串脚本系统替代 Rust 的强类型 pane payload 和 binding payload

## 设计结论

系统采用三层权责模型：

1. `workbench`
2. `template_runtime`
3. `Slint host`

### workbench

`workbench` 继续拥有：

- `ViewDescriptorId -> ViewInstance -> ViewHost` 生命周期
- document center、drawer slot、floating window、attach/detach、split ratio、focus、preset
- `ViewContentKind` 在 autolayout、callback surface size、closeability 上的宿主语义

### template_runtime

`template_runtime` 负责：

- 接收 `document_id + payload + bindings`
- 将 pane body 模板实例化并投影为 host node tree
- 在 hybrid 模式下暴露 native slot 所需的挂载锚点

### Slint host

`Slint host` 继续负责：

- native widget 渲染
- 平台窗口与 floating window 承载
- pointer/scroll/selection bridge
- hierarchy、animation timeline、graph canvas 等高交互区域的 native 桥接

因此，本设计不是“用 `.ui.toml` 替掉 Slint”，而是“用 `.ui.toml` 替掉 pane body 的手工拼装权”。

## 核心模型

### ViewDescriptor 扩展

`ViewDescriptor` 从只描述 workbench 语义，扩展为同时描述 pane 模板语义。

新增元数据固定为：

- `pane_template: Option<PaneTemplateSpec>`

`PaneTemplateSpec` 至少包含：

- `shell: PaneShellSpec`
- `body: PaneBodySpec`

`PaneShellSpec` 描述 pane 外壳级语义：

- `document_id`
- `show_toolbar`
- `supports_empty_state`
- `viewport_slot`

`PaneBodySpec` 描述 body 模板级语义：

- `document_id`
- `payload_kind`
- `route_namespace`
- `interaction_mode`

其中：

- `payload_kind` 决定 body payload 的 Rust 类型
- `route_namespace` 决定模板控件绑定应落到哪类命令面
- `interaction_mode` 决定该 pane body 是纯模板实现还是 hybrid native slot

### PanePresentation

现有 `PaneData` 是一个跨多个 pane 的 giant union。设计上将其拆成：

- `PanePresentation`
- `PaneShellPresentation`
- `PaneBodyPresentation`
- `PanePayload`

`PaneShellPresentation` 只承载：

- title
- icon
- subtitle
- info
- empty state
- toolbar
- viewport chrome

`PaneBodyPresentation` 承载：

- `document_id`
- `payload_kind`
- `route_namespace`
- `interaction_mode`
- `payload`

### PanePayload

`PanePayload` 保持 Rust 强类型 enum，而不是降级为开放式表结构。首批 payload kind 固定为：

- `ConsoleV1`
- `HierarchyV1`
- `InspectorV1`
- `AnimationSequenceV1`
- `AnimationGraphV1`

对应首批 payload 族：

- `ConsolePanePayload`
- `HierarchyPanePayload`
- `InspectorPanePayload`
- `AnimationSequencePanePayload`
- `AnimationGraphPanePayload`

这条规则用于防止 pane body 数据模型滑向弱类型脚本化结构，保持与 `EditorUiBindingPayload` 现有风格一致。

## Route 与命令边界

本设计不引入第二套模板命令系统。

模板层只声明 route，行为仍由现有 `EditorUiBindingPayload` 家族执行。命名空间固定为：

- hierarchy -> `SelectionCommand`
- inspector -> `DraftCommand` 与 `InspectorFieldBatch`
- animation -> `AnimationCommand`
- shell / docking / menu -> `DockCommand`、`MenuAction`、`ViewportCommand`

因此 `.ui.toml` 的职责是：

- 声明控件树
- 声明 binding id / route
- 声明 slot

而不是：

- 决定 editor command 语义
- 直接持有 dock/focus/window lifecycle 逻辑

## 模板注册面

模板注册继续走现有 `template_runtime::builtin` 入口，不新建平行注册系统。

首批新增 document 固定为：

- `pane.console.body`
- `pane.hierarchy.body`
- `pane.inspector.body`
- `pane.animation.sequence.body`
- `pane.animation.graph.body`

这些模板将通过以下入口注册：

- `template_documents.rs`
- `component_descriptors.rs`
- `template_bindings.rs`

如需 hybrid native slot，body 模板必须提供稳定 slot 名称，例如：

- `hierarchy_tree_slot`
- `animation_timeline_slot`
- `animation_graph_canvas_slot`

## Hybrid 边界

Phase 1 不追求所有 pane 完全模板化。

以下区域固定先走 hybrid：

- hierarchy tree 主体
- animation sequence 的高交互 timeline 细节区
- animation graph 的 canvas 区

这意味着 `.ui.toml` 只接管这些 pane 的：

- header
- toolbar
- filter / sidebar / panel
- status / summary / section layout
- empty state

而不接管这些 pane 的：

- row hit test runtime
- graph canvas interaction runtime
- pointer bridge / scroll bridge / selection bridge

这样可以避免 Phase 1 退化为控件运行时重写。

## 迁移顺序

### Phase 1a: 引入模板元数据

在 `workbench/view` 新增以下类型：

- `PaneTemplateSpec`
- `PaneShellSpec`
- `PaneBodySpec`
- `PanePayloadKind`
- `PaneRouteNamespace`
- `PaneInteractionMode`

并将它们挂入 `ViewDescriptor` 与 builder。

### Phase 1b: 引入 PanePresentation

在 `workbench_host_window` 中新增：

- `PanePresentation`
- `PaneShellPresentation`
- `PaneBodyPresentation`
- 各 payload builder

`pane_projection.rs` 改为：

- 解析当前 tab/view
- 读取 `PaneTemplateSpec`
- 调用对应 payload builder
- 生成 `PanePresentation`

它不再直接充当所有 pane body 的巨型分支拼装器。

### Phase 1c: 注册首批 body 模板

在 `template_runtime::builtin` 中新增首批 body document 与 bindings：

- console
- inspector
- hierarchy
- animation sequence
- animation graph

### Phase 1d: 切换 Console 与 Inspector

先切换 `Console` 与 `Inspector`，因为它们更接近模板化表单或展示布局，风险较低。

### Phase 1e: 切换 Hierarchy 与 Animation Hybrid

切换 `Hierarchy`、`AnimationSequence`、`AnimationGraph` 的外层 body 模板，但保留核心高交互区为 native slot。

### Phase 1f: 削薄旧 DTO 转换链

随着 pane body authority 迁移完成，逐步削薄：

- `host_data.rs` 中的 giant union 字段
- `pane_data_conversion.rs` 中的 pane-specific Slint DTO 转换
- `apply_presentation.rs` 中直接灌入 pane body 细节的逻辑

当前落点是 `PaneData` 保留 shell、viewport、presentation 和一个显式命名的 `PaneBodyCompatData`。首批 pane 的结构真源不再是平铺 union 字段；Slint 生成类型仍保持扁平 ABI，因此 compat 壳在 Rust host 边界内负责喂给 native tree、timeline、graph canvas 与尚未完全模板化的 asset/project body。

## 文件级改造落点

### 新增

建议新增：

- `zircon_editor/src/ui/workbench/view/pane_template_spec.rs`
- `zircon_editor/src/ui/workbench/view/pane_payload_kind.rs`
- `zircon_editor/src/ui/workbench/view/pane_route_namespace.rs`
- `zircon_editor/src/ui/workbench/view/pane_interaction_mode.rs`
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_presentation.rs`
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs`
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/*.rs`
- `zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs`

### 扩展

直接扩展：

- `view_descriptor.rs`
- `view_descriptor_builder.rs`
- `hierarchy_view_descriptor.rs`
- `inspector_view_descriptor.rs`
- `console_view_descriptor.rs`
- `animation_sequence_view_descriptor.rs`
- `animation_graph_view_descriptor.rs`
- `template_documents.rs`
- `component_descriptors.rs`
- `template_bindings.rs`

### 变薄

明确要变薄的文件：

- `host_data.rs`
- `pane_projection.rs`
- `pane_data_conversion.rs`
- `apply_presentation.rs`

### 保留职责

明确保留现职责，不在本轮替换的文件：

- `slint_adapter.rs`
- `hierarchy_pointer.rs`
- `detail_scroll_pointer.rs`

## 验收标准

本设计完成时必须满足：

- 任一 `ViewDescriptorId` 都能解析到唯一的 `PaneTemplateSpec`
- pane body 不再由 `ViewContentKind` 大分支直接手工拼装
- `.ui.toml` 可以决定 body 结构、section 结构、route 声明与 hybrid slot 布局
- `DockCommand`、`DraftCommand`、`SelectionCommand`、`AnimationCommand` 的现有语义不变
- floating window、split、drawer、focus、preset 行为无回归
- hierarchy 与 animation 的高交互区域在 hybrid 模式下仍保持可用

## 默认假设

- `ViewContentKind` 在本轮之后仍然保留，但只服务于宿主策略与 autolayout，不再承担 pane body 结构选择权
- `Slint` 在本轮之后仍然是 editor 的 native renderer 与 platform host，而不是要被 `.ui.toml` 替换掉
- 动画 editor 的 sequence 与 graph 在 view 身份上已经分离，因此 payload 也应分离，不再继续依赖单一 `mode` 进行 body 结构切换
- 如果某个 pane 的高交互区在第一轮无法稳定模板化，则优先保留 native slot，而不是把整个 pane body 权限留在旧 union DTO 体系中
