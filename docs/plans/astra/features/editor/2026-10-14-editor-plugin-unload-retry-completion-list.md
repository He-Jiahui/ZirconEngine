# Editor：插件卸载重试夹具完成列表

状态：`source_reviewed / native_performance_pending`；详见[优化记录](../../../optimize/zircon_editor/2026-10-14-plugin-unload-retry-fixture-record.md)。

- [x] CR-EDITOR-SERVICES-0003 最终事件序列改为 Loaded/Enabled/Disabled/Unloaded/Unloaded。
- [x] 保留首次 Unloaded 错误/Faulted/候选无事件和重试成功的全部断言。
- [x] 生产逻辑、原6 ordinary/0 ignored 身份和所有性能门槛保持。
- [x] source/packet 校验、fmt/diff、root guard 和独立 delta 静态审查完成。
- [x] 补选唯一实际修复 case，第14批 Editor ordinary 下限96。
- [ ] 实际原生回执、full-input seal、case与匹配下限通过。
- [ ] 适用原性能/资源门槛通过并关闭历史 CR-EDITOR-SERVICES-0003。
- [ ] 更广 CR-EDITOR-SERVICES-0002 与 Plugin Manager 主计划 P0/P1 独立修复验收。

仅源码和静态阶段完成；保留必要交付证据，无构建/缓存/夹具或源码副本，实际释放0B。
