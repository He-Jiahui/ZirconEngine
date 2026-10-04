---
status: in_progress
local_validation_receipt: "paired Runtime task-failure contract 4/4; expanded non-tooling Runtime/Editor source-contract batch 4072/4072 across 962 files in 139.499s (performance-or-contract filename filter, tooling/export/coordinator excluded); recent-record Rustfmt batch 169/169; managed Windows Cargo/Release and product gates pending"
plan_sources:
  - docs/plans/optimize/zircon_editor/300-editor-engineering-gap-synthesis-review.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
---

# Editor 任务进度派发优化

## 已确认问题

`EditorJobSystemInner::deliver_progress_observer_events` 在任务接收、完成、取消路径
调用 `ProgressObserverDispatch::take_all`，每轮 `mem::take` 后释放缓冲。
连续进度批次因此重复分配；重入 callback 产生下一批时也不能复用已排空的缓冲。
队列上限和 overflow resynchronization 已存在，必须保留。

## 实施与验收

- 在原 dispatch owner 内保留两个有界批次缓冲；交换批次并回收已排空缓冲。
- callback 顺序、重入防护、overflow resynchronization、panic 恢复保持原合同。
- 预热后重复 1/1k 批次保留两块缓冲容量；10k 输入仍折叠为一个 resynchronization。
  容量回归不是 allocator 计数，不据此虚构实测分配次数。
- release 同进程对照原实现，预热后采样 p50/p95/p99；p95 回退不超过 5%。
- 同 [Runtime 任务诊断](../runtime/01-task-failure-progress.md) 批量验证。
  源码计数和队列微基准不等同于产品帧率达标。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M1 | 复用进度通知缓冲，覆盖重入与有界性 | implemented_pending_validation | 见 [Runtime 批次交接](../runtime/01-task-failure-progress.md#验证交接)；快照封存被外部 dirty 依赖阻止，尚无性能结果 |
