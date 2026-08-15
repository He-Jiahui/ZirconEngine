# Editor14 Job Event Sink Stable Label Ownership Analysis

## Scope

This analysis continues the open `failure-2026-07-17-job-pump-budget-and-pending-scan.md` without treating a queued validation receipt as acceptance. It is limited to the worker-to-main `JobEventSink -> JobEventQueue -> JobEventPump -> EditorMessageBus` path.

## Current-Source Findings

- 变更前，`JobEventSink` owns a job-stable `label: String`, but `emit` clones that string into every `JobEvent` while holding the lifecycle lock.
- A normal job produces Started, zero-or-more Progress, and one terminal event. The 1,000-job storm therefore performs at least 2,000 stable-label allocations before Progress events, and 3,000 for its current one-progress fixture.
- The queue retains a latest Progress event by `JobId`; replacing a Progress event releases the old cloned label but still allocates before coalescing. `JobEventPump` then moves the retained event into `EditorMessagePayload::Job`; consumers expose the label only as `&str`.
- `JobEvent` is serialized, and the workspace `serde` dependency enables `rc`. `Arc<str>` therefore preserves the serialization surface and string equality while allowing lifecycle events to share a single job-stable allocation.
- The lifecycle, progress-source, and queue locks have a single observed order (`lifecycle -> progress -> queue`). No deadlock evidence or contention metric justifies collapsing these independently-owned locks. This cut must not change their ownership.

## Decision

Replace the private event-label representation with `Arc<str>` when creating `JobEventSink`. `JobEvent::label()` remains `&str`; public event fields and event ordering remain unchanged. Each emission clones only the `Arc` control block, including coalesced Progress replacement. The construction conversion can retain one setup-time allocation/copy for a dynamically built `String`; this cut intentionally makes no stronger allocator-count claim without a profile.

The expected steady-state reduction is from `O(lifecycle + progress events)` event-label buffer allocations per job to zero event-label buffer allocations after sink construction. Setup remains `O(1)` per job, and atomic reference-count traffic remains `O(events)`; neither becomes a performance result until a managed CPU profile demonstrates its cost.

## Validation Plan

- Add a focused behavior regression proving cloned events share the same stable label allocation while retaining string-visible equality.
- Run scoped formatting and static diff checks locally.
- Submit the focused managed jobs event test against an immutable snapshot; collect a Windows performance profile before considering lock or queue-structure changes.

## Non-Goals

- No new job descriptor registry, compatibility field, event format version, or second queue.
- No lock aggregation, raw pointers, interning global, or test-only allocator shortcut.
- No performance pass claim without a managed current-source profile.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-08-10 | `analysis_complete / implementation_started` | 完成 event sink、queue、pump、message bus 与 serde ownership 调研；选择 `Arc<str>` stable label sharing 作为唯一低风险结构性优化，明确保留三段锁 owner。 | 当前源码调用链与 1,000-job storm事件数量；workspace `serde` 启用 `rc`。尚未运行 Cargo 或宣称性能收益。 |
| 2026-08-10 | `implementation_complete / static_validation_passed / independent_review_pending` | `JobEvent` 与 `JobEventSink` 的私有稳定标签改为 `Arc<str>`；sink 构造后事件发射与事件克隆共享该分配，`label()` 可见语义不变。 | 新增克隆共享分配回归；`rustfmt --edition 2021 --check` 与 scoped `git diff --check` 通过；受管快照 `1595`（`be9e1b5a1dc3424d878cf433df62e35e`）。构造转换的精确 allocation 数待 profile，尚未主张性能结果。 |
| 2026-08-10 | `independent_review_clean / validation_queued` | 独立复审确认 `Arc<str>` 不改变 serde 表面、`label()` 观察语义、事件顺序或 `lifecycle -> progress -> queue` 锁顺序；无需额外前向修复。 | 独立复审 `Critical/Important/Minor = 0/0/0`；静态格式与差异检查复核通过；focused managed receipt `ea722162afbd42b180b3f6cbd6a4c2cc`，immutable source snapshot `1597`，尚未取得 terminal Cargo 或 profile evidence。 |
