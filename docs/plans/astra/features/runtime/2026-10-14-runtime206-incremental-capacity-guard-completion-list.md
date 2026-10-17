# Runtime206：增量容量守卫修复完成列表

状态：`source_reviewed / native_performance_pending`。

[优化记录](../../../optimize/zircon_runtime/206/2026-10-14-incremental-capacity-guard-scope-record.md)记录原计划范围、目标函数/兄弟函数误报原因、最终源码哈希、补丁和静态证据。[原计划](../../../optimize/zircon_runtime/206/2026-09-15-incremental-publication-capacity.md)的容量及性能验收没有放宽。

- [x] prepare/commit 唯一边界与函数体范围失败时拒绝。
- [x] 保留四项 reserve、四项 dedup/materialize 正例与目标 new 负例。
- [x] 在原 source-shape 用例保留 sibling、退化、移出 witness、无效边界的反例。
- [x] 最终源码/packet 校验、fmt/diff、root change-guard 与独立静态复核完成。
- [x] 原 2 ordinary + 1 ignored 身份及 upper_bound / P95 producer 内容不变。
- [x] 核对两个 ordinary 与旧批次所有 Runtime unit arm 无有效选择重叠。
- [ ] 第14批批量请求实际执行并返回 exact-source 原生通过证据。
- [ ] 全输入 seal、普通匹配数及所有适用原始性能门槛通过。
- [ ] 原生回执核验后关闭历史 CR-ASSET-PIPELINE-SOURCE-GUARD-0001 和整体验收。

仅源码和静态阶段完成；pending 请求、静态检查、allocation-shape 模型不代表产品性能达标。交接后继续功能修复，不等待或实时监控协调器。未创建构建/缓存/夹具或源码副本，实际释放 0B；保留必要交付证据。
