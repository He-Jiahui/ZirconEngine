---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/editor/930-editor09-promotion-dependency-capacity.md
  - docs/plans/astra/features/editor/934-editor09-event-journal-gap-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/jobs/system/state.rs
tests:
  - zircon_editor/src/core/jobs/tests/scheduling_contract.rs
---

# Editor935 Editor09 Scheduling Dependency Capacity

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Scheduling dependency projection | `scheduling_dependencies` 按显式 `after` 数量加可选 mutex-group tail 一次性预留 `JobHandle` 容量，再用 `extend` 写入显式依赖；保留依赖顺序、terminal handle 与 mutex tail 语义。 | 新增 scheduling source contract 要求上界容量与 `dependencies.extend`；既有 mutex-tail 行为测试继续覆盖依赖数量和完成状态。 |
| 性能门禁 | 普通调度路径不再让显式依赖集合在 `collect` 中几何扩容；只改变 scratch-vector 容量，不改变依赖解析、锁范围或调度顺序。 | 新增 Editor09 ignored marker `EDITOR09_SCHEDULING_DEPENDENCY_CAPACITY_BENCH_V1`，对 4,096 个依赖验证旧模型有扩容、预留模型为零；managed Debug/Release 仍待异步回执。 |

## 本地验证

- `rustfmt --edition 2021 --check`：`state.rs` 与 `scheduling_contract.rs` 通过。
- Editor09 Python 合同批量：`Ran 68 tests ... OK`（0.067s）；本次 Rust source contract 与 marker 随 Editor09 包测试提交。
- tooling 未修改，继续按要求延期。

## 当前源码批量验证

Editor934 四路 PTY 在本次 `state.rs`/`scheduling_contract.rs` source change
之前提交，不能覆盖 Editor935。新的批量验证已提交：Runtime Debug PTY
`65144`、Editor Debug PTY `30236`、Runtime02 Release PTY `91656`、Editor09
Release PTY `87829`。旧批次与本批 wrapper 均按约定不轮询，提交本身不推断
Cargo 或性能门禁结果。

合同断言随后按 rustfmt 的多行容量表达式进行了收窄，所以上述批次也不覆盖
当前测试源。替换批次使用 Runtime Debug PTY `17377`、Editor Debug PTY
`93752`、Runtime02 Release PTY `65523`、Editor09 Release PTY `39730`；所有
wrapper 仍按约定不轮询，提交本身不推断 Cargo 或性能门禁结果。
