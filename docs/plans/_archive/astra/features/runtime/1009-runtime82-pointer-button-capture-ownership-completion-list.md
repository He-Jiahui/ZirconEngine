---
related_code:
  - zircon_runtime/src/ui/surface/input/state/pointer_capture.rs
  - zircon_runtime/src/ui/surface/input/state/pointer_press.rs
  - zircon_runtime/src/ui/surface/surface/event_routing/pointer_ownership.rs
implementation_files:
  - zircon_runtime/src/ui/surface/input/pointer.rs
  - zircon_runtime/src/ui/surface/input/window_pump.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-pointer-button-capture-ownership.md
tests:
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/pointer_capture_routes/ownership_lifecycle.rs
doc_type: milestone-detail
---

# Runtime1009 完成列表：pointer button ownership

状态：`implemented_reviewed_managed_validation_pending`。

- [x] 最低共享 Runtime owner 修复：press/capture 分开记录 pointer/button；异键边沿正常派发并保留原归属。
- [x] 保留公共 UiFocusState、同 Primary 重按、programmatic 任意 Up、显式 Release 与多 pointer 独立性。
- [x] 跨 owner capture admission、事务 rollback、同节点多指 pressed、detach/disable、window blur/deactivate/destroy、hot reload 合同已实现。
- [x] 新增真实 Surface、reply、text/range/scrollbar 与生命周期回归；提取语义 child，结构门继续 `<800`。
- [x] [优化记录](../../../optimize/zircon_runtime/82/2026-09-27-pointer-button-capture-ownership.md)与 canonical 输入合同同步；保留原 N/O manifest，记录 input.md 的 P successor。
- [x] 独立源码审查已绑定 v2 的 28 路径 manifest；补齐失效 owner 双 pressed 标记与真实文本选区断言。
- [x] 3 个既有合同测试与 4 个既有滚动测试完整迁至语义子模块；父 0 测试、gate 所列 child 合计 46、全部文件 `<800`，保留所有旧断言。
- [ ] managed P 合批编译、动态回归及实际执行回执。
- [ ] Editor59 G06 capture generation/native、G31/G33 与 100k/1M、1kHz、4/16 viewport allocation/p95/p99 原性能门槛。

未直接运行 Cargo，未提交或轮询编译协调器。完整测试名单、静态检查和 preimage 逆向证据随 P source manifest 归档。
