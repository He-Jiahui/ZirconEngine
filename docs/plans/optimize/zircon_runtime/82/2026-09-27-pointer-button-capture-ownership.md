---
related_code:
  - zircon_runtime/src/ui/surface/input/state/pointer_capture.rs
  - zircon_runtime/src/ui/surface/input/state/pointer_press.rs
  - zircon_runtime/src/ui/surface/input/pointer.rs
  - zircon_runtime/src/ui/surface/surface/event_routing/pointer_ownership.rs
implementation_files:
  - zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs
  - zircon_runtime/src/ui/surface/input/window_pump.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
plan_sources:
  - user: 2026-09-27 Runtime/Editor optimization with asynchronous batched validation
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
tests:
  - zircon_runtime/src/ui/tests/event_routing/pointer_state.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/pointer_capture_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/pointer_capture_routes/ownership_lifecycle.rs
  - zircon_runtime/src/ui/tests/widget_range_navigation.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
doc_type: milestone-detail
---

# Runtime82：按键归属与指针捕获

| 里程碑 | 范围 | 状态 | 日期 | 证据 |
|---|---|---|---|---|
| Runtime1009 | 共享 press/capture 按 pointer/button 归属、生命周期与真实控件回归 | `implemented_reviewed_managed_validation_pending` | 2026-09-27 | [完成列表](../../../astra/features/runtime/1009-runtime82-pointer-button-capture-ownership-completion-list.md)、[运行时合同](../../../../zircon_runtime/ui/surface/pointer_capture.md)；P 批统一验证待回执 |

## 源码缺陷与修复

原 `route_pointer_event_with_details` 在任何 Down 覆盖 pressed，在任何 Up 清 press/capture；Secondary Down 后 Primary Up 可误触 Primary click，捕获拖动中异键 Up 使后续越界 Move 丢路由。文本 Up 无归属判断还会结束选区拖拽或打开右键菜单。

Runtime 分别保留每个 pointer 的 press 和 capture button；异键边沿正常派发，但不触发默认激活或自动释放。已保存同 Primary 重按迁移与无 button 的 programmatic capture 任意 Up 释放兼容合同。跨 owner 异键 CapturePointer reply 在 mutation 前拒绝；原事务快照包含新增字段，后续 effect 失败可完整恢复。补齐多指同节点 pressed 投影、非活跃 owner 移除、真实 window blur/deactivate/destroy 和 hot reload 清理。

原 `event_routing.rs` 为 884 行；将捕获与路由所有权完整迁至语义子模块，保留 `<800` 结构门。既有结构测试同时对齐已存在的 state invalidation 子 owner 与 hover scratch 函数名，不放宽断言阈值。

本地参考：`dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InputRouter.cpp:244–254,320–375`，已活跃 capture 继续接收输入，捕获替换需要显式终止/接管；修饰与普通输入继续处理。Zircon 的 pointer/button 记录由自身真实 Surface/manager 合同决定。

独立源码审查已完成并绑定 v2 的 28 路径 manifest。审查补齐失效 owner 的节点/组件 pressed 标记清理，文本回归同时检查真实 caret/selection。另修复同一测试树既有结构门失配：3 个源码合同测试迁入 `event_routing/contracts.rs`，4 个滚动测试迁入 `event_routing/component_events/scroll_defaults.rs`，所有原断言保留；父模块仍为 0 个测试，gate 所列 child owners 合计 46 个测试，文件继续严格 `<800`，原 852 行的 component child 收敛为 758 行。此计数不代表整棵递归测试树；17 个新增行为回归与 7 个既有测试搬迁均待实际 Rust 执行。

## 验收与边界

新增回归先落源码，dynamic red 与最终动态通过均待 managed P 批，未直接运行 Cargo。测试过滤器为 `pointer_button_ownership_`；完整名单及静态命令/哈希、lease、preimage、逆变换证据位于 `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-p-runtime82-pointer-ownership-*`。计划写入授权为 `eeede5ecb05648cfb5c189b85d8ae0db` / `75c3096757a54fcf8ab8fb94e36d1d5d`。

Canonical `input.md` 是明确授权的 P 后继版本；O 封存 SHA 为 `7a2fd48c4a1ce5dbc3ee3dabeae9d43ca98de50994071f5989da41a4fd5daf33`。O 首项 admission 已终止且未生成 Cargo ticket；旧 N/O manifest 保持不可变，P 保存新输入哈希及继承原因。

Editor59 原门槛仍待验收：G06 `release只终止同pointer/button/capture generation的interaction`；G31 `pointer event hot path不构造完整render packet或复制全量mesh payload`；G33 `125/500/1000Hz motion可合并但不跨press/release/cancel改变edge order`。100k/1M selectable、1kHz pointer、large selection、4/16 viewport、allocation 与 p95/p99 的产品/native/性能证据仍 pending；本次不宣称 generation/native 或性能总门槛达标。
