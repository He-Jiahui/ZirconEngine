# Runtime02：事件容量计数查表优化完成列表

状态：源码阶段完成；原生功能与完整性能待完成。

- [x] 成功 charge 路径复用两个不同 map 字段的 mutable entry，四次查询减为两次，移除临时 Arc clone/drop。
- [x] 保留三个 checked-add 的失败原子性、原提交/peak 顺序和未知订阅者、缺 topic、overflow、零值、关闭及 poison 语义。
- [x] 独立源审及实际一文件应用通过；45 双次输入核对、44 非修改输入、实际格式与 scoped diff 检查通过。
- [x] 保留原 17 业务用例、7 原 EventBus Release producer、所有断言和预算；现有 #2 / #9 覆盖，新增 arm / selector / case 均为零。
- [x] 保留共享索引、旧索引锁内容及元数据、Cargo 与控制配置；不完善 tooling。
- [x] 写入[优化记录](../../../optimize/zircon_runtime/02/2026-10-09-admission-charge-entry-reuse-record.md)与[实际采用回执](../../../../../.codex/outbox/runtime02-admission-charge-entry-reuse-v1/actual-canonical-apply.json)。
- [ ] 批量编译和功能测试实际通过，核对精确名称及非零数量。
- [ ] 原完整 tasks / DLL / fanout / payload / batch / 10K / 100K / 1M 矩阵通过。
- [ ] actual allocation、CPU、锁等待、延迟、retained bytes 与 RSS 满足原性能阈值。
- [ ] 已激活 Jenkins 协调器提供可归属当前源码的异步批次请求及最终回执。
- [ ] 功能、性能及真实产品验收完成。

查表和 Arc 计数减少尚未转化为实测性能结论。
