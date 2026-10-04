---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/192-runtime-task-execution-job-scheduler-task-graph-worker-domain-scope-cancellation-deadline-shutdown-diagnostics-product-adoption-current-working-tree-review.md
related_records:
  - docs/plans/astra/features/runtime/883-runtime02-event-task-hotpaths.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation/journal.rs
tests:
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation/tests.rs
---

# Runtime884 Runtime02 Task Diagnostic Page Capacity

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Diagnostic read page | `TaskDiagnosticJournal::read_after` 在已有有序 cursor 分区后计算实际页上界，按该上界预留 `Vec`，再用 `extend` 复制观察项；保留 zero-limit、cursor、`has_more` 和顺序语义。 | 新增 Runtime02 source contract 要求 `page_capacity`、`Vec::with_capacity(page_capacity)` 和 `observations.extend`，并保留既有 partition/zero-limit 合同。 |
| 分配热路径 | 满页诊断读取不再从零容量向量几何扩容；页不足或零页时容量严格取实际可读条目数。 | 确定性 Release marker `RUNTIME02_TASK_DIAGNOSTIC_PAGE_CAPACITY_BENCH_V1` 要求旧模型存在增长、预留模型增长为零；不把模型时间当作产品吞吐结论。 |
| 行为边界 | 该切片只改变结果 scratch buffer 的容量策略；现有 retention、dropped gap、ordered partition、cursor recovery 和 severity 行为不变。 | Runtime/Editor 非 tooling 本地批量 `18/18` 通过；受管 Cargo/Release 仍需当前源码回执。 |

## 本地验证

- Runtime884 source contract：通过。
- `rustfmt --edition 2021 --check`：production 与 lower test 文件通过。
- Runtime/Editor 非 tooling 合同批量：`Ran 18 tests ... OK`（53.500s）。
- tooling 未修改，仍按用户要求留待后续 Rust 迁移。

## 当前源码批量验证

上一轮四路验证在本切片之前提交，不能覆盖 Runtime884。新的 Runtime/Editor Debug
与 Runtime02/Editor09 Release 四路提交回执记录在中央异步日志：Runtime Debug PTY
`99767`、Editor Debug PTY `69461`、Runtime02 Release PTY `33591`、Editor09
Release PTY `77672`。wrapper 按异步约定不轮询，因此本记录不提前宣称编译、测试
数量或 Release 性能阈值达标。

Runtime885 subsequently changed the task-graph submit-after fence projection.
The four PTYs above are therefore stale for the current Runtime source; the
replacement batch and its unpolled launch are recorded in Runtime885 and the
central log.
