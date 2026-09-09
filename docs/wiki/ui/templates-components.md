---
related_code:
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/build/surface_builder.rs
  - zircon_runtime/src/ui/component/catalog/mod.rs
  - zircon_runtime/src/ui/component/state_reducer.rs
  - zircon_runtime/src/ui/binding/mod.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/component/catalog/registry.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/ui/tests/template_pipeline.rs
  - zircon_runtime/src/ui/tests/template/interaction_bindings.rs
  - zircon_runtime/src/ui/tests/v2_asset/composite_components.rs
doc_type: module-detail
---

# 模板、组件与绑定

## 模板管线

`UiDocumentCompiler` 解析文档、校验节点与 slot contract、收集资源/本地化依赖，输出 `UiCompiledDocument`。`UiTemplateTreeBuilder` 生成运行时树，`UiTemplateSurfaceBuilder` 进一步附加表面状态。`UiTemplateInstance` 表示可重建实例。

编译缓存 (`UiAssetCompileCache`、`UiCompiledArtifactStore`) 由内容指纹索引；资源变化经过 `UiInvalidationGraph` 生成热重载计划，`UiAssetHotReloadExecutor` 仅重建受影响表面。

## 组件目录与状态

`UiComponentDescriptorRegistry` 保存组件描述、调色板入口和参数 schema。Material Foundation、MUI/MUI-X、编辑器导航和表单控件在目录中注册。`UiComponentStateRuntimeExt::apply_component_event` 将组件事件折叠到状态 reducer（selection、slider、tree view、text input、toast 等）。

## 数据绑定

`UiModelSchemaRegistry` 注册反射模型字段和类型转换；`UiBindingConversionRegistry` 执行 `UiValue` 转换。更新报告区分 retained attribute、runtime state、component event 等来源，并携带 previous value、dirty flags 与 Applied/Unchanged/Rejected 状态。

```rust
use zircon_runtime::ui::binding::{UiModelSchemaRegistry, binding_update_report};
use zircon_runtime::ui::component::apply_component_event;
let report = binding_update_report(vec![update]);
```

## 限制

组件参数必须通过 descriptor 校验；widget alias 更新被拒绝并产生诊断。热重载期间先取得 `UiBindingQuiescenceReceipt`，确保绑定事务完成后再替换树。
