---
related_code:
  - zircon_editor/src/ui/template_runtime/component_adapter/mod.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/registry.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/inspector.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/reflection.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/command.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
implementation_files:
  - zircon_editor/src/ui/template_runtime/component_adapter/registry.rs
tests:
  - rustfmt --edition 2021 --check zircon_editor/src/ui/template_runtime/component_adapter/registry.rs
  - editor UI 10 component-adapter dead_code suppression scan
doc_type: module-doc
status: active
---

# Editor UI Template Runtime Component Adapter

## 职责

`component_adapter` 是 editor UI 模板运行时与 runtime `UiComponent*` 数据契约之间的适配层。它负责把 editor 侧 inspector、reflection、asset editor 的字段数据源登记到 runtime access，并把 UI 组件事件 envelope 分发到对应 editor 状态处理器。

## 当前结构

- `registry.rs` 是唯一注册入口：`EditorUiComponentAdapterRegistry::data_sources()` 产出 inspector、reflection、asset_editor 三类数据源描述，`apply_envelope(...)` 按 target domain 路由到具体处理器。
- `inspector.rs` 处理 selected-entity inspector 事件。
- `reflection.rs` 处理 reflection 数据源事件。
- `asset_editor.rs` 处理 UI Asset widget/layout/slot/binding/style 字段事件。
- `command.rs` 承载 component adapter command 辅助类型。
- `component_drawer.rs` 与 `showcase.rs` 是模板运行时 component 展示/抽屉侧的使用域，不拥有注册器数据源声明。

## 接线契约

`EditorUiComponentAdapterRegistry::data_sources()` 由 `ui/host/editor_event_runtime_access.rs` 暴露给 editor event runtime access，测试覆盖位于 `src/tests/ui/component_adapter.rs`。因此 registry helper 不是半成品死代码，不能通过 `#[allow(dead_code)]` 逃避结构审计。

## 2026-06-22 结构治理记录

按 `engine-code-structure-convention.md` E6 与 Editor UI 10 M5.T2，`registry.rs` 已移除 5 处陈旧生产 `#[allow(dead_code)]` 抑制：

- `EditorUiComponentAdapterRegistry::data_sources()`
- `reflection_source(...)`
- `asset_editor_source(...)`
- `reflection_fields(...)`
- `asset_editor_fields(...)`

窄范围验证确认 registry 无剩余 `dead_code` 抑制，数据源入口仍由 runtime access 和 component adapter 测试直接使用。原 Editor UI 10 S8 的 `EditorWorldSlot` 私有旁路也已硬删除，并由 boundary regression 守卫不得恢复。
