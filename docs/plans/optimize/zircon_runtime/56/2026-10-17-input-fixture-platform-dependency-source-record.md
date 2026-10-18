# Input 动作映射测试依赖注册修复

对应计划：[Runtime 输入产品集成 56](../56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md)、[Runtime220](../220-runtime-input-device-event-frame-action-context-trigger-modifier-user-focus-gamepad-rumble-record-replay-host-editor.md)。

`input_action_manager_resolves_from_runtime_module_descriptor` 原来仅注册 InputModule，激活前缺少声明的 PlatformModule；PlatformModule 又依赖 TasksModule。CoreRuntime 的模块表初始为空，因此测试在动作求值和重绑定断言前就会失败。

本次在真实夹具中按 TasksModule、PlatformModule、InputModule 注册，保持输入模块配置及既有动作求值、重绑定断言。没有另造模块描述符，也没有添加跳过条件。保留 CR-INPUT-0003 标记，注明源码已修复但验收待完成。

- [x] 补齐真实公开模块描述符及传递依赖。
- [x] 保留现有 action/rebind 行为断言；通过现有 tests/mod.rs 到达该测试。
- [ ] 合批原生测试通过：`input::tests::action_mapping::input_action_manager_resolves_from_runtime_module_descriptor`。
- [ ] 当前完整源码集合的公开变更检查及测试验收。

源码写入记录：`.codex/outbox/reports/optimize-20261013-01a13451/runtime-input-platform-fixture-source-write-receipt.json`。本项为测试夹具修复，没有性能提升实测结论；不能据此关闭 Runtime220 的输入驱动、生命周期或性能条目。编译、缓存、夹具产物创建为 0；实际释放空间 0 字节。测试随 Runtime、Editor 后续验证批次提交，保持待验收。
