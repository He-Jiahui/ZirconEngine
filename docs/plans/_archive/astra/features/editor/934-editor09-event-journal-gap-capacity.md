---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/editor/933-editor09-progress-projection-capacity.md
  - docs/plans/astra/features/editor/935-editor09-scheduling-dependency-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/jobs/event_journal/journal.rs
tests:
  - zircon_editor/src/core/jobs/event_journal/journal.rs
---

# Editor934 Editor09 Event Journal Gap Capacity

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Gap merge projection | `merge_gap` 先保留覆盖区间，再按 BTreeMap range 的 `size_hint` 预留序列 scratch buffer，使用 `extend` 快照后移除事件；保留 gap 合并、事件顺序、计数与 finish/remove 语义。 | 新增 source contract 要求使用 `covered_range.size_hint().0`、`Vec::with_capacity` 与 `covered_sequences.extend`；既有 gap/回压恢复测试继续覆盖行为。 |
| 性能门禁 | 覆盖事件较多的 gap 合并不再依赖 `Vec` 的几何扩容；只改变临时序列容器容量，不改变 BTreeMap 锁边界、删除顺序或诊断计数。 | 新增 Editor09 ignored marker `EDITOR09_EVENT_JOURNAL_GAP_CAPACITY_BENCH_V1`，对 4,096 个覆盖事件验证旧模型有扩容、预留模型为零；managed Debug/Release 仍待异步回执。 |

## 本地验证

- `rustfmt --edition 2021 --check`：`journal.rs` 通过。
- Editor09 Python 合同批量：`Ran 68 tests ... OK`（0.073s）。
- Editor934 source contract：通过。
- tooling 未修改，继续按要求延期。

## 当前源码批量验证

Editor933/Runtime885 之后的四路 PTY 在本次 `journal.rs` source change
之前提交，不能覆盖 Editor934。新的 Runtime/Editor Debug 与 Runtime02/
Editor09 Release 批次已重新提交并记录在中央异步日志；wrapper 按约定不轮询，
不从提交本身推断 Cargo 编译、测试或性能阈值通过。
