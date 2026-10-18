# Runtime 输入测试依赖注册：源码完成列表

- [x] 修复动作管理器集成测试的模块图：真实 TasksModule → PlatformModule → InputModule。
- [x] 保留实际动作求值、动作表重绑定断言及现有测试模块接线。
- [x] 保留 CR-INPUT-0003 的待验收标记。
- [ ] Jenkins 合批执行原生测试并绑定实际源码、配置与工具链。
- [ ] 公开完整变更检查及真实测试通过后关闭 BUG。

优化记录：[源码修复记录](../../../optimize/zircon_runtime/56/2026-10-17-input-fixture-platform-dependency-source-record.md)。源码完成不等于测试通过；本次未创建构建产物，释放 0 字节。Runtime220 功能与性能验收继续开放。
