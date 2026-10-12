# Editor01：Core / Particle Source 验证入口完成列表

状态：源码阶段完成；实际测试、强覆盖门、产品与性能待完成。

- [x] 提取两个既有真实驱动，保留原四个测试名称、全部正负断言和逐页 Scene 退休逻辑。
- [x] 核对实际选择源、物理源及主题的完整有序五字段 SourceReceipt；调用方持有原环境锁。
- [x] 修复测试 getter 在实际 owner 不存在时错误 expect；生产 ABI、发布及生产行为保持。
- [x] 独立核心源审通过；实际两文件应用、116 双次输入核对、114 非修改输入、两文件格式及 scoped diff 检查通过。
- [x] 共享索引、旧索引锁的内容和元数据、Cargo 与控制配置保持；历史输入与两次安全拒绝记录保留。
- [x] 写入[优化记录](../../../optimize/zircon_editor/01/2026-10-09-core-particle-source-witness-record.md)及[实际采用回执](../../../../../.codex/outbox/editor-real-component-source-witness-proposal-v2/actual-canonical-apply.json)。
- [ ] 同一批次实际执行四个精确用例，检查非零匹配和真实 Source / paint / hit / typed input / Scene 退休结果。
- [ ] strong gate 实际接入五组真实驱动；原 205 组件、53 根及 offender-empty 要求全部通过。
- [ ] 三个旧聚合资产的真实消费者迁移与实体退役决定完成。
- [ ] 通过原完整 Release 矩阵、真实 Editor 产品呈现及所有原性能阈值。
- [ ] 已激活 Jenkins 协调器提供可归属当前源码的异步批次请求和最终回执。

未增加 Cargo arm 或新测试；源码采用与格式证据不能代替 native、性能或产品验收。
