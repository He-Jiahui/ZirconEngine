# Runtime51 前缀查询修复完成列表

本轮状态：`source_applied_native_validation_pending`。

- [x] 在当前正式源码移除前缀候选 HashSet；返回 Vec 准确预分配，保留完整过滤和原顺序。
- [x] 核对构建、加载、更新、重命名的索引不变量，保留外国测试目录迁移。
- [x] 保留原断言，增加 empty／scheme／suffix／相似路径／无结果的 authoritative 对照回归。独审、增量应用、换行核对、格式检查和 diff 检查完成。
- [ ] 运行合并后的原测试批次，修复实际失败。
- [ ] 按原 10K／100K／1M、G39 和 CPU／RSS／I/O／allocation／P50／P95／P99 条件完成性能验收。

证据：[所属优化记录](../../../optimize/zircon_runtime/51/2026-10-05-prefix-query-scratch-allocation.md)。源码完成项不等同于测试或性能达标。
