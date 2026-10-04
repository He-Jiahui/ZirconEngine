---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
---

# 导出终态错误传播

## 当前源码确认

Editor269 `ED-EXPORT-P0-002` 仍可从生产路径触发：manager 返回含 fatal diagnostics
或失败 Cargo invocation 的 `Ok(report)`，retained job 接受报告，summary 无条件显示
Exported。本项解决这条既有调用链的失败传播；不关闭两套 export authority 或完整
bundle/install/smoke 资格缺口。

## 实施与验收

- report 统一检查 materialization/plan fatal、缺少 required provider、Cargo receipt
  一致性、host/native Cargo success 与退出码。
- manager 的全部返回点将失败报告转换为 typed `ReportFailed`，保留完整报告与诊断；
  成功报告不复制。job adapter 与 UI summary 消费同一判定，取消仍为 Cancelled。
- 回归覆盖 fatal、missing provider、host/native 非零和未知退出、缺失 receipt，
  同时验证成功与取消，并确认失败报告缓冲被移动保留。
- 纳入下一次 runtime/editor 批次，不单独启动编译。正确性必须先通过，性能仍使用
  M1-M4 配对基准，不把严重失败的提前返回冒充性能提升。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M5 | 旧导出 report/job/summary 失败一致性 | implemented_pending_validation | `manager/astra_outcome_tests.rs` 与 `report/astra_outcome_tests.rs` 覆盖 fatal diagnostics、缺少 provider、host/native 非零或未知退出、缺失 receipt、成功/取消及失败报告缓冲保留；`into_result()` 在 job/summary 共用 typed terminal 判定。Rustfmt/静态复核通过；受管 Runtime/Editor Cargo 与 release 配对数据仍待执行。 |
