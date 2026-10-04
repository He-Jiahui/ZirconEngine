---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/192-runtime-task-execution-job-scheduler-task-graph-worker-domain-scope-cancellation-deadline-shutdown-diagnostics-product-adoption-current-working-tree-review.md
related_records:
  - docs/plans/astra/features/runtime/884-runtime02-task-diagnostic-page-capacity.md
  - docs/plans/astra/features/runtime/883-runtime02-event-task-hotpaths.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/runtime/tasks/task_graph/scope.rs
tests:
  - zircon_runtime/src/core/runtime/tasks/task_graph/scope/tests.rs
---

# Runtime885 Runtime02 Task-Graph Dependency Fence Capacity

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Scoped submit-after | `submit_after_with_scheduler` 按依赖 slice 长度一次预留 `dependency_fences`，再用 `extend` 克隆 completion fences；依赖顺序、owner 校验和 lease 保持不变。 | 新增 source contract 要求 `Vec::with_capacity(dependencies.len())`、`dependency_fences.extend`，并拒绝回退到零容量 `collect`。 |
| 分配热路径 | 深依赖或高 fan-in 的 scoped task admission 不再让 fence 向量几何扩容；依赖 lease 仍独立保留，未改变生命周期边界。 | `RUNTIME02_TASK_GRAPH_DEPENDENCY_FENCE_CAPACITY_BENCH_V1` 确定性 marker 要求旧模型增长次数大于零、预留模型为零。 |
| 行为边界 | 失败依赖、取消、terminal hooks、scope admission 和 task ordering 均不改动；只改变 fence scratch capacity。 | Runtime885 lower marker/source contract 已通过；当前 Runtime/Editor 受管 Debug/Release 批量仍待异步回执。 |

## 本地验证

- Runtime885 source contract：通过。
- Runtime885 lower marker contract：通过。
- `rustfmt --edition 2021 --check`：scope production 与 lower tests 通过。
- Runtime/Editor 非 tooling 合同批量：`Ran 18 tests ... OK`（37.287s）。
- tooling 未修改，继续按用户要求延期。

## 当前源码批量验证

Editor933 的四路提交在 Runtime885 之前，不能覆盖本次 Runtime source change。新的
Runtime/Editor Debug 与 Runtime02/Editor09 Release 四路提交后，回执会写入中央异步
日志：Runtime Debug PTY `90873`、Editor Debug PTY `20286`、Runtime02 Release PTY
`63470`、Editor09 Release PTY `10969`。wrapper 按异步约定不轮询，不从提交本身推断
编译、测试或性能阈值通过。
