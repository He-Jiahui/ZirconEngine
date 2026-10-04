---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/root_normalization.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-detached-subtree-parent-cycle-rejection.md
tests:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/raw_cycle_tests.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/root_normalization_profile.rs
---

# Runtime1012：detached subtree parent-cycle rejection

状态：`implemented_static_review_managed_validation_pending`。

- [x] 真实 raw tail/self/2/3-cycle 和 empty-success/partial-delete 问题有回归源码。
- [x] 批次局部 memo 先验证再 collapse，保留错误优先级、拒绝计数和 Runtime08 owned-row 事务边界。
- [x] 合法 covered roots、table/sparse 值与 ticks、直接 persistence、deferred 通知边界和唯一 parent reads 有真实 World 用例。
- [x] 完整旧 prepare API baseline 与新 Release paired profile 已保存；未用缩减函数替代基线。
- [x] [公共合同](../../../../zircon_runtime/scene/world/detached_entity_batch.md) 与 [Runtime62 记录](../../../optimize/zircon_runtime/62/2026-09-27-detached-subtree-parent-cycle-rejection.md) 同步；Q error 为只读依赖。
- [x] 冻结当前源 SHA 后完成独立代码审查；根节点检查了七路径 SHA、exact inverse、完整旧 API baseline、实际 parent memo、raw-cycle 断言与有效 generation 语义。
- [ ] managed R 完整 lib 与 grouped Release 动态回执：`runtime62_detached_parent_cycle_release_profile`、`prepared_camera_subtree_managed_scale_fixture`、`detached_entity_batch_managed_scale_fixture`。
- [ ] 原 RSH-G03 的其余 corrupt-graph walkers、restore 合并图校验及 protected-component authority 后续合同。

无 Cargo/编译器运行、验证提交、编译状态轮询或外部通知。测试先写入但动态 red 未执行；raw p50/p95/p99、计数和性能验收仍 pending。inverse、preimage、归属/授权和源 manifest 由 R 批次记录绑定，不改 Q 已冻结文件。
