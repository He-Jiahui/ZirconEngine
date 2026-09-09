---
related_code:
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/tree/node/routing.rs
  - zircon_runtime/src/ui/event_ui/manager/ui_event_manager.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
implementation_files:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/event_ui/manager/ui_event_manager.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/ui/tests/shared_core/navigation.rs
  - zircon_runtime/src/ui/tests/shared_core/input_visibility/pointer_routes.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
doc_type: module-detail
---

# UI 树与事件路由

## 节点模型

`UiRuntimeTree*Ext` 扩展 trait 将布局、交互、焦点、渲染顺序、路由和滚动操作拆分。节点保存父子关系、布局盒、输入策略、伪状态和语义元数据。`UiHitTestIndex` 以网格索引加 arranged 可见性执行点查询，返回 `UiHitTestResult` 与祖先路径。

## 路由阶段

窗口事件先转换为 `UiInputEvent`，再由 `UiEventManager` 按 `UiNodePath` 路由：捕获阶段从根向目标，目标阶段处理控件行为，冒泡阶段从目标回到根。处理结果通过 `UiComponentEventReport` 汇总；指针捕获、焦点转移和弹窗栈可改变后续目标。

支持指针/鼠标、键盘、文本、IME、导航、模拟量、拖放、弹窗/工具提示计时器及无障碍动作事件。每个事件都带 `UiInputEventMetadata`，包括窗口、设备、时间和来源。

## Rust API

```rust
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::event_ui::UiEventManager;

let result = surface.hit_test(point);
let (subscription_id, notifications) = event_manager.subscribe();
```

具体派发函数由表面输入层调用；业务组件应注册订阅或实现组件状态 reducer，而不是直接修改路由索引。需要同步回调时可使用 `UiEventManager::register_route` 注册 `UiEventBinding`。

## 焦点、滚动与限制

焦点导航使用 `UiRuntimeTreeFocusExt` 和 navigation index；模态作用域会限制候选节点。滚动状态支持指针滚轮、键盘导航和虚拟列表。命中测试要求几何为有限正数，超出离散布局值上限（`MAX_UI_LAYOUT_DISCRETE_VALUE=4096`）的轨道/跨度会被拒绝。
