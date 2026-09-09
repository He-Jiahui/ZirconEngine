---
related_code:
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tree/mod.rs
implementation_files:
  - zircon_runtime/src/ui/module.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
doc_type: module-detail
---

# 架构与生命周期

## 模块职责

`UiModule` 在 `InitLevel::Scene` 注册 `UiRuntimeDriver` 与 `UiEventManager`。驱动依赖 Input、Scene、Graphics；事件管理器依赖驱动。`UiConfig` 通过 `UI_CONFIG_KEY` 注入运行时配置。

## 数据流

1. 资产加载器将 `.zui`/v2 文档解析为 `UiV2AssetDocument` 或模板文档。
2. 编译器生成 `UiV2CompiledDocument`/`UiCompiledDocument`，展开组件导入并建立节点 arena。
3. `UiV2SurfaceBuilder` 或 `UiTemplateSurfaceBuilder` 创建 `UiSurface`，建立 `UiRuntimeTree`、静态/运行时样式、文本测量缓存和组件状态。
4. 表面输入泵接收窗口事件，命中测试后沿节点路径捕获、目标、冒泡；绑定事务写回属性并设置 `UiDirtyFlags`。
5. 布局阶段按固定顺序执行测量、Taffy 排布、虚拟列表窗口和几何发布。
6. 渲染抽取将可见节点、文本 glyph artifact、图标和反馈效果转换为 `UiSurfaceFrame`；公共帧由 `PublicRuntimeFrame` 交给 Graphics。

## 保留式不变量

- 节点 ID (`UiNodeId`) 在表面会话内稳定；重建通过节点池和 dirty domain 尽量复用。
- 表面是每个 `UiTreeId` 的状态所有者，包含焦点、滚动、弹出层、文本编辑及组件状态。
- 输入可见性、禁用状态和 arranged tree 在命中测试前生效，避免向不可见/折叠节点派发事件。

## API 示例

```rust
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::ui::{UiModule, module_descriptor, UiConfig};
let descriptor = module_descriptor();
assert_eq!(UiModule.module_name(), zircon_runtime::ui::UI_MODULE_NAME);
```

## 限制与验证

UI 驱动必须由引擎模块生命周期启动；不要在业务代码手动构造 `UiEventManager` 替代服务注册。架构边界由 `runtime_absorption/ui_architecture` 与 `ui_boundary` 测试维护。
