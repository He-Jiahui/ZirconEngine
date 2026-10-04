---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
---

# Runtime 验证前置修复

## 当前诊断

2026-09-05 合并 release 诊断 job `938b6ebb2b784e57a1ba3a3860a3ff08`、run
`5e88cd0079d24558a2e427a15b45cee6` 的 runtime lib test 因 278 个编译错误失败，
测试与性能基准未执行。日志位于 `.codex/state/session-coordinator/cargo-runs/`
对应 job/run 目录。错误包括迁移后的旧路径、类型及测试接口，不能把批次失败当作
优化性能结果。

## 修复范围

- 单服务卸载阻断集合恢复为借用 `RegistryName`，只在最终错误输出处分配字符串。
- 材质依赖投影为借用键补全生命周期约束，保留引用与 locator 的不同去重语义。
- 插件工厂 panic 回归使用受 owner 管理的 plugin handle；事件订阅测试导入现有
  trait，移除与 blanket impl 冲突的测试 Event 实现。
- readiness 回归用不可变 row identity 判断依赖变更，替换已移除的数值 revision API。
- 不恢复已删除的兼容 API；后续按同一错误组继续修复剩余调用方。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M7 | 借用集合、生命周期与迁移后的测试调用方 | implemented_pending_validation | 合并诊断已定位原因，待剩余编译错误修复及统一验证 |
