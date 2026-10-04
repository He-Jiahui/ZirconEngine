---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/editor/932-editor09-enqueue-dependency-capacity.md
  - docs/plans/astra/features/editor/934-editor09-event-journal-gap-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/jobs/progress.rs
tests:
  - zircon_editor/src/core/jobs/progress.rs
---

# Editor933 Editor09 Progress Projection Capacity

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Full progress snapshot | `snapshot` 按 active-entry 数量预留输出上界，再用 `extend` 过滤 terminal 项；保留稳定 BTreeMap 顺序和共享 snapshot 字符串。 | 新增 progress source contract 要求 `Vec::with_capacity(state.active.len())` 与 `snapshots.extend`。 |
| Bounded projections | `snapshot_limit` 按 `limit.min(active.len())` 预留；`snapshot_for_ids` 按去重后的 `ids.len()` 预留；`unfinished_jobs` 按 active 数量预留。 | 既有行为测试继续覆盖 limit、ID 顺序、terminal 过滤和 unfinished 语义；新增 Editor09 ignored marker 验证旧模型有增长、预留模型增长为零。 |
| 性能门禁 | 高频 UI 进度读取不再依赖 `collect` 的几何扩容；只改变 scratch-buffer 容量，不改变排序、过滤、锁边界或数据所有权。 | 本地 Editor09 合同批量 68/68；当前源码 Debug/Release 受管四路验证仍待异步回执。 |

## 本地验证

- `rustfmt --edition 2021 --check`：`progress.rs` 通过。
- Editor09 Python 合同批量：`Ran 68 tests ... OK`（0.059s）。
- Editor933 source contract：通过。
- tooling 未修改，继续按要求延期。

## 当前源码批量验证

Runtime884 之后的四路 PTY 在本次 progress source change 前提交，不能覆盖
Editor933。新的 Runtime/Editor Debug 与 Runtime02/Editor09 Release 批次为：
Runtime Debug PTY `68251`、Editor Debug PTY `53128`、Runtime02 Release PTY
`2356`、Editor09 Release PTY `61577`。wrapper 按异步约定不轮询，不从提交本身
推断 Cargo 或性能阈值通过。

Runtime885 changed a Runtime task-graph source after this shared wave was
submitted. The Runtime lane therefore needs the replacement batch recorded in
Runtime885; the Editor source remains covered by the same current-source
submission and all wrappers stay intentionally unpolled.

Editor934 subsequently changed the event-journal gap projection after this
record's batch. The Editor933 receipt is therefore stale for the current
Editor source; Editor934 records the replacement four-lane submission.
