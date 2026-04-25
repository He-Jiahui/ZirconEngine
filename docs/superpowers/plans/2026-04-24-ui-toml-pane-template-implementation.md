# UI TOML Pane Template Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `zircon_editor` 的 pane body/layout authority 从 Rust/Slint 手工拼装层迁移到 `.ui.toml` 模板层，同时保持 `workbench` 的 docking/window 生命周期和 Slint 的 native 交互桥不变。

**Architecture:** 先在 `ViewDescriptor` 上增加 `PaneTemplateSpec`，把 view 身份和 pane 模板语义绑定起来；再把 `workbench_host_window` 的 giant `PaneData` 收敛成 `PanePresentation = shell + body`，通过 payload builder 把数据送进 template runtime。首轮只把 `Console`、`Inspector` 和 `Hierarchy/Animation` 的外层 body 接到模板层，高交互区继续通过 hybrid native slot 挂接，不重写 pointer/scroll/timeline/graph runtime。

**Tech Stack:** Rust 2024, cargo test/check, Slint, `.ui.toml` template runtime, repo-local docs

---

## Target File Structure

```text
zircon_editor/src/ui/workbench/view/
  pane_template_spec.rs
  pane_payload_kind.rs
  pane_route_namespace.rs
  pane_interaction_mode.rs
  view_descriptor.rs
  view_descriptor_builder.rs

zircon_editor/src/ui/layouts/windows/workbench_host_window/
  pane_presentation.rs
  pane_payload.rs
  pane_payload_builders/
    mod.rs
    console.rs
    inspector.rs
    hierarchy.rs
    animation_sequence.rs
    animation_graph.rs
  host_data.rs
  pane_projection.rs

zircon_editor/src/ui/template_runtime/runtime/
  pane_payload_projection.rs
  runtime_host.rs
  projection.rs

zircon_editor/assets/ui/editor/host/
  console_body.ui.toml
  inspector_body.ui.toml
  hierarchy_body.ui.toml
  animation_sequence_body.ui.toml
  animation_graph_body.ui.toml
```

边界目标：

- `ViewDescriptor` 直接携带 `PaneTemplateSpec`
- `PanePresentation` 取代 `PaneData` 作为 pane body 主载体
- `.ui.toml` 决定 pane body 结构与 route 声明
- `Slint host` 只保留 shell、native widget、hybrid slot、pointer/scroll bridge
- `Hierarchy` 和 `Animation` 的高交互区 Phase 1 不完全模板化

## Validation Baseline

- `cargo test -p zircon_editor --lib --locked template_runtime --offline`
- `cargo test -p zircon_editor --lib --locked slint_host --offline`
- `cargo test -p zircon_editor --lib --locked inspector --offline`
- `cargo check -p zircon_editor --locked --offline`

预期：先新增 descriptor/template/presentation red tests，再逐步把实现转绿。

### Task 1: Add View Template Metadata Types And Descriptor Coverage Tests

**Files:**
- Create: `zircon_editor/src/ui/workbench/view/pane_template_spec.rs`
- Create: `zircon_editor/src/ui/workbench/view/pane_payload_kind.rs`
- Create: `zircon_editor/src/ui/workbench/view/pane_route_namespace.rs`
- Create: `zircon_editor/src/ui/workbench/view/pane_interaction_mode.rs`
- Modify: `zircon_editor/src/ui/workbench/view/mod.rs`
- Modify: `zircon_editor/src/ui/workbench/view/view_descriptor.rs`
- Modify: `zircon_editor/src/ui/workbench/view/view_descriptor_builder.rs`
- Create: `zircon_editor/src/tests/host/pane_template_descriptor.rs`
- Modify: `zircon_editor/src/tests/host/mod.rs`

- [ ] 为 `ViewDescriptor` 定义 `pane_template: Option<PaneTemplateSpec>`，并新增 `PaneShellSpec` / `PaneBodySpec` / `PanePayloadKind` / `PaneRouteNamespace` / `PaneInteractionMode`
- [ ] 写 red tests，锁死 `editor.console`、`editor.inspector`、`editor.hierarchy`、`editor.animation_sequence`、`editor.animation_graph` 必须暴露 pane template metadata
- [ ] 运行：
  - `cargo test -p zircon_editor --lib --locked pane_template_descriptor --offline`

### Task 2: Register Pane Template Metadata On Builtin Views

**Files:**
- Modify: `zircon_editor/src/ui/host/builtin_views/activity_views/console_view_descriptor.rs`
- Modify: `zircon_editor/src/ui/host/builtin_views/activity_views/hierarchy_view_descriptor.rs`
- Modify: `zircon_editor/src/ui/host/builtin_views/activity_views/inspector_view_descriptor.rs`
- Modify: `zircon_editor/src/ui/host/builtin_views/activity_windows/animation_sequence_view_descriptor.rs`
- Modify: `zircon_editor/src/ui/host/builtin_views/activity_windows/animation_graph_view_descriptor.rs`
- Modify: `zircon_editor/src/ui/host/builtin_views/activity_views/activity_view_descriptors.rs`
- Modify: `zircon_editor/src/ui/host/builtin_views/activity_windows/activity_window_descriptors.rs`
- Modify: `zircon_editor/src/tests/host/pane_template_descriptor.rs`

- [ ] 给首批五个 view descriptor 绑定稳定的 `body document id / payload kind / route namespace / interaction mode`
- [ ] 明确 `Console`、`Inspector` 为 `TemplateOnly`，`Hierarchy`、`AnimationSequence`、`AnimationGraph` 为 `HybridNativeSlot`
- [ ] 运行：
  - `cargo test -p zircon_editor --lib --locked pane_template_descriptor --offline`

### Task 3: Add Builtin Body Documents And Template Runtime Registration Tests

**Files:**
- Create: `zircon_editor/assets/ui/editor/host/console_body.ui.toml`
- Create: `zircon_editor/assets/ui/editor/host/inspector_body.ui.toml`
- Create: `zircon_editor/assets/ui/editor/host/hierarchy_body.ui.toml`
- Create: `zircon_editor/assets/ui/editor/host/animation_sequence_body.ui.toml`
- Create: `zircon_editor/assets/ui/editor/host/animation_graph_body.ui.toml`
- Modify: `zircon_editor/src/ui/template_runtime/builtin/template_documents.rs`
- Modify: `zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs`
- Modify: `zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs`
- Create: `zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs`
- Modify: `zircon_editor/src/tests/host/template_runtime/mod.rs`

- [ ] 为五个 pane body 新建 `.ui.toml` 文档，首轮只定义 header/section/body slot，不承诺高交互区纯模板化
- [ ] 在 builtin template runtime 注册这五个 document、component descriptor 和 route bindings
- [ ] 写 red tests，锁死 runtime 可以加载并投影这些 document，且 hierarchy/animation 模板带稳定 slot 名
- [ ] 运行：
  - `cargo test -p zircon_editor --lib --locked pane_body_documents --offline`
  - `cargo test -p zircon_editor --lib --locked template_runtime --offline`

### Task 4: Introduce PanePresentation And Payload Builders

**Files:**
- Create: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_presentation.rs`
- Create: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs`
- Create: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/mod.rs`
- Create: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/console.rs`
- Create: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/inspector.rs`
- Create: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/hierarchy.rs`
- Create: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/animation_sequence.rs`
- Create: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/animation_graph.rs`
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/mod.rs`
- Create: `zircon_editor/src/tests/host/pane_presentation.rs`
- Modify: `zircon_editor/src/tests/host/mod.rs`

- [ ] 新建 `PanePresentation = shell + body` 和 typed `PanePayload` 结构，避免直接退回 `serde_json::Value`
- [ ] 让 builder 明确输出 `ConsoleV1`、`InspectorV1`、`HierarchyV1`、`AnimationSequenceV1`、`AnimationGraphV1`
- [ ] 写 red tests，锁死 builder 会为对应 view 输出正确 `document_id / payload_kind / route namespace / interaction mode`
- [ ] 运行：
  - `cargo test -p zircon_editor --lib --locked pane_presentation --offline`

### Task 5: Rework `pane_projection.rs` To Resolve Template Spec And Build PanePresentation

**Files:**
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs`
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/mod.rs`
- Modify: `zircon_editor/src/tests/host/pane_presentation.rs`

- [ ] 把 `pane_projection.rs` 从 `ViewContentKind` 巨型 body 分支收缩成：找 tab -> 找 descriptor -> 找 `PaneTemplateSpec` -> 调 payload builder
- [ ] 让 `host_data.rs` 先并存旧 `PaneData` 和新 `PanePresentation`，避免一次性拆光 Slint host
- [ ] 更新测试，锁死 `pane_projection` 不再直接手工拼 `Console/Inspector/Hierarchy/Animation` body DTO
- [ ] 运行：
  - `cargo test -p zircon_editor --lib --locked pane_presentation --offline`
  - `cargo check -p zircon_editor --locked --offline`

### Task 6: Add Template Runtime Payload Projection For Pane Bodies

**Files:**
- Create: `zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs`
- Modify: `zircon_editor/src/ui/template_runtime/runtime/mod.rs`
- Modify: `zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs`
- Modify: `zircon_editor/src/ui/template_runtime/runtime/projection.rs`
- Create: `zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs`
- Modify: `zircon_editor/src/tests/host/template_runtime/mod.rs`

- [ ] 在 runtime 内新增 pane body payload 注入层，让 `document_id + payload + bindings` 能为首批 body 模板生成 host nodes
- [ ] 先保证 `Console` 和 `Inspector` 能完全模板投影，`Hierarchy/Animation` 至少能暴露 hybrid slot 挂载锚点
- [ ] 写 red tests，锁死 slot 与 route metadata 会进入 host projection
- [ ] 运行：
  - `cargo test -p zircon_editor --lib --locked pane_payload_projection --offline`
  - `cargo test -p zircon_editor --lib --locked template_runtime --offline`

### Task 7: Switch Console To Template-Driven Body

**Files:**
- Modify: `zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs`
- Modify: `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs`
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`
- Create: `zircon_editor/src/tests/host/slint_console_template_body.rs`
- Modify: `zircon_editor/src/tests/host/mod.rs`

- [ ] 让 `Console` body 从新 `PanePresentation` 和 `.ui.toml` 文档生成，不再依赖旧 `ConsolePaneViewData` 直转 Slint DTO
- [ ] 保留 shell、窗口和 scroll host 逻辑不变
- [ ] 运行：
  - `cargo test -p zircon_editor --lib --locked slint_console_template_body --offline`
  - `cargo test -p zircon_editor --lib --locked slint_host --offline`

### Task 8: Switch Inspector To Template-Driven Body Without Changing DraftCommand Semantics

**Files:**
- Modify: `zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs`
- Modify: `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs`
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`
- Modify: `zircon_editor/src/tests/editing/inspector.rs`
- Create: `zircon_editor/src/tests/host/slint_inspector_template_body.rs`
- Modify: `zircon_editor/src/tests/host/mod.rs`

- [ ] 让 Inspector body 结构改由 `pane.inspector.body` 驱动，但字段编辑仍走 `DraftCommand.SetInspectorField` 和 `InspectorFieldBatch`
- [ ] 保留 inspector scroll bridge，不在本任务重写底层交互
- [ ] 运行：
  - `cargo test -p zircon_editor --lib --locked inspector --offline`
  - `cargo test -p zircon_editor --lib --locked slint_inspector_template_body --offline`

### Task 9: Switch Hierarchy And Animation To Hybrid Template Bodies

**Files:**
- Modify: `zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs`
- Modify: `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs`
- Modify: `zircon_editor/src/ui/slint_host/app/hierarchy_pointer.rs`
- Modify: `zircon_editor/src/ui/slint_host/app/detail_scroll_pointer.rs`
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`
- Create: `zircon_editor/src/tests/host/slint_hierarchy_template_body.rs`
- Create: `zircon_editor/src/tests/host/slint_animation_template_body.rs`
- Modify: `zircon_editor/src/tests/host/mod.rs`
- Modify: `zircon_editor/src/tests/host/animation_editor.rs`

- [x] 让 hierarchy 与 animation 改为模板驱动的 outer body，但树主体、timeline 细交互、graph canvas 继续走 native slot / pointer bridge
- [x] 锁死 hierarchy 使用 `SelectionCommand`，animation 使用 `AnimationCommand`，不新建平行 route 家族
- [x] 运行：
  - `cargo test -p zircon_editor --lib --locked slint_hierarchy_template_body --offline`
  - `cargo test -p zircon_editor --lib --locked slint_animation_template_body --offline`
  - `cargo test -p zircon_editor --lib --locked slint_host --offline`
  - `cargo test -p zircon_editor --lib --locked template_runtime --offline`
  - `cargo check -p zircon_editor --locked --offline`

### Task 10: Remove Remaining Giant Union Responsibilities And Update Docs

**Files:**
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`
- Modify: `zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs`
- Modify: `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs`
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs`
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs`
- Modify: `docs/ui-and-layout/shared-ui-template-runtime.md`
- Modify: `docs/ui-and-layout/index.md`
- Modify: `docs/superpowers/specs/2026-04-24-ui-toml-pane-template-design.md`

- [x] 删除已被 `PanePresentation` 取代的 giant union 责任，让旧 `PaneData` 只保留必须的兼容壳或彻底退场
- [x] 更新相关文档，说明 `.ui.toml` 现在拥有 pane body authority，而 Slint 保留 native host/hybrid bridge
- [x] 运行：
  - `cargo test -p zircon_editor --lib --locked template_runtime --offline`
  - `cargo test -p zircon_editor --lib --locked slint_hierarchy_template_body --offline`
  - `cargo test -p zircon_editor --lib --locked slint_animation_template_body --offline`
  - `cargo test -p zircon_editor --lib --locked slint_host --offline`
  - `cargo fmt --all`
  - `cargo check -p zircon_editor --locked --offline`

## Sequencing Rules

- 先加 `ViewDescriptor -> PaneTemplateSpec`，再改 `PanePresentation`
- 先注册 body 模板，再让运行时和 Slint host 消费这些模板
- 先切 `Console` 和 `Inspector`，再切 `Hierarchy/Animation hybrid`
- `Hierarchy` 和 `Animation` 的高交互区在 Phase 1 不允许被错误规划成纯模板运行时
- `DockCommand / DraftCommand / SelectionCommand / AnimationCommand` 语义必须保持不变
- 文档只在实现和测试稳定后更新

## Completion Checklist

- `ViewDescriptor` 已能为首批 pane 暴露稳定的 `PaneTemplateSpec`
- `PanePresentation` 已取代 giant `PaneData` 作为 pane body 主表示
- `template_runtime` 已能加载首批 pane body 模板并注入 payload/route metadata
- `Console` 与 `Inspector` 已完全切到模板驱动 body
- `Hierarchy` 与 `Animation` 已切到 hybrid template body
- `pane_projection.rs` 不再直接手工拼装 `Console/Inspector/Hierarchy/Animation` body
- `Slint host` 仍保留 docking/window/native interaction bridge 责任
