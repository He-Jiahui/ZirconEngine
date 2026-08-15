---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: job-pump-budget-and-pending-scan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/event.rs
  - zircon_editor/src/core/jobs/event_sink.rs
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/core/jobs/progress.rs
  - zircon_editor/src/core/jobs/spec.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/jobs/system/construction.rs
  - zircon_editor/src/core/jobs/system/lifecycle.rs
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/core/jobs/system/progress_observer.rs
  - zircon_editor/src/core/jobs/system/scheduling.rs
  - zircon_editor/src/core/jobs/system/submission.rs
  - zircon_editor/src/core/jobs/system/state.rs
  - zircon_editor/src/core/jobs/tests/admission_scaling_contract/indexed.rs
  - zircon_editor/src/core/jobs/tests/background_storm_contract.rs
  - zircon_editor/src/core/jobs/tests/scheduling_contract.rs
tests:
  - cargo test -p zircon_editor --lib cloned_events_share_the_job_stable_label_allocation --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib ready_background_job_is_selected_within_one_weighted_fairness_round --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib system_root_is_a_structural_leaf_module_entry --locked --jobs 1 -- --test-threads=1
  - 1000/10000 job promotion complexity benchmark
  - count/time-budgeted main-thread pump storm
  - progress coalescing preserves started and terminal edges
---

# Editor14：job pump 无帧配额且 pending admission 接近 O(n²)

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：Editor jobs 19 个生产 Rust 文件静态审查与 1,000-job storm 合同复核
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：job admission、worker→main 回流配额与 progress 合并属于 Editor14 调度契约。

## 失败现象与复现证据

`JobEventPump::pump` 对 unbounded MPSC 一直 drain 到空，并逐事件同步发布；retained host 每 tick 调用它。现有 1,000-job storm 明确输出 `numeric_budget=undefined`，未设置每帧 count/time SLA。

pending 使用 `Vec`；每次 promote 全量 `filter/min_by_key`，再 `remove(index)`，每个 job 完成又重跑。长队列累计扫描/搬移接近 O(n²)。

## 最低共享层根因

Editor job model 只有 active-category 并发配额，没有主线程 delivery 配额、queue age/peak 或按 priority/category 可增量准入的数据结构。上层 retained tick 无法安全决定何时停止 drain。

## 架构修复验收

- 1,000/10,000 job 分离测 submit、promotion、completion 和 pump p50/p95/allocations。
- 定义 count 与 time 双预算；Started/terminal 保序不可丢，Progress 可按 JobId latest-value 合并。
- 选择 priority/category/dependency-aware ready queues 或索引，证明大队列不再二次增长。
- 当前源码 editor WPR 验证主线程帧预算、worker 利用率、shutdown 和公平性。

## 禁止临时方案

- 不得通过降低 worker 并发掩盖主线程 pump；不得丢 terminal/cancel/error 事件。
- 不得把 wall-clock 观察值当通过阈值，必须冻结可复现预算。

## 修复结果与回传

Open state: `Editor14 源码已建立 pump SLA、progress 合并、索引化 admission 与 enum 自有扫描库存；current-source Cargo、性能/WPR 和独立复审尚未完成`。

2026-07-18 静态复核发现并修复一项后续扩展饥饿风险：priority/category enum 与各自 `ALL` 由同一 `define_job_enum!` 输入原子生成，扫描库存不再由 `pending.rs` 私有复制，最大 probe 数由库存长度派生；测试拒绝恢复本地常量、固定长度 `ALL` 或非宏声明源。`rustfmt --check`、atomic inventory guard 7/7 与 scoped diff check 已通过。

当前 managed Cargo 被 Coordinator01 的 `pending-cpu-reservation-absolute-expiry-not-enforced` failure 阻塞：两个 executable-owner reservation 在持久化绝对过期后仍保持 pending/jobless 并占用 FIFO head。本 failure 保持 `open`；Editor14 不释放 foreign reservation，也不在缺失 current-source Cargo、1k/10k 原始输出、WPR 与独立复审时生成 fixed return。

2026-07-22 current-source补充：indexed admission与64 events/1ms pump仍成立，本轮把`TOPIC_JOB` parse从每tick收为构造期一次。新的最低open条件是submit/lifecycle queue entry+bytes+age背压，以及event sink三锁/稳定label clone收敛；只证明progress coalesce或pending scan已修不能关闭PERF-MVP-020。

2026-07-22全量tests复核补充：`background_storm_contract`仍用“must accept every storm job”锁定1,000个请求全部无条件入队，并把wall-clock明确降为非pass/fail baseline；`admission_scaling_contract`只证明1k/10k probe线性，不记录payload/label bytes、oldest age或100k/1M重复请求。因此验收必须先把submit结果建模为accepted/merged/backpressured，并冻结entry+bytes+age/RSS与pump p95硬门；不得以线性promotion或60秒watchdog替代有界性。

2026-08-10 forward repair：新增 1,000 thumbnail requests 的 bounded-storm contract，显式统计 accepted 与 `AdmissionEntryLimitExceeded` backpressured 请求。它以 8-entry/1,024-byte/60-second admission limits 断言 retained tickets、pending/scheduled records、snapshot entry/byte/age 均在预算内；gate release 后按同一 60-second deadline 以 `try_take` 收敛，避免调度回归使 Cargo lane 无限挂起。该合同只验证逻辑预算，不把 wall-clock 观测值升级为性能通过阈值。受管 Cargo、规模矩阵与 WPR 尚未完成，failure 保持 open。

2026-08-10 当前源码性能分析补充：pending admission 的 entry/bytes 上限不能限制 stopped UI pump
期间已完成 job 的 Started/terminal backlog；完成后释放 pending entry，再持续 submit 可以在不违反
pending budget 的情况下增长 `JobEventQueue` lifecycle `VecDeque`。同时 label、progress 和 failed display
text 都是未限长 String，count cap 不能证明 bytes 有界。后续实现必须采用 job admission 持有、terminal event
成功发布后释放的 lifecycle delivery entry/byte/age reservation，并在 enqueue 前限制/诊断 display text；禁止
drop、worker blocking 或第二 UI queue。`JobEventPump` 还必须处理 EditorMessageBus 的 lossless backpressure
report：lifecycle edge 只有获得 lossless dispatch acknowledgement 后才能出队/释放 reservation，不能在 full
subscriber inbox 时静默丢弃。设计、反例和回归矩阵见
`14/2026-08-10-job-event-delivery-reservation-analysis.md`。本项尚未进入实现或受管 profile。
非消费式 lossless dispatch outcome 的消息总线 owner 已存在于 Editor02
`failure-2026-07-17-message-inbox-backpressure-and-fanout.md`；Editor14 只能消费该 contract 后实现 queue-front
retry/reservation release，不能复制 bus/inbox owner 或建立 compatibility wrapper。

2026-08-11 source-level contract audit narrows that prerequisite: the current
`SharedEditorMessageBus::publish` takes `EditorMessage` by value and
`prepare_dispatch` immediately moves it into `EditorMessageDelivery::with_sequence` before
lossless inbox preflight. Its later `EditorMessageDispatchReport` can describe
`backpressured`, but it cannot return the original producer payload. Editor02 must therefore
own one atomic lossless-producer operation which either admits the original message into the
shared delivery payload or returns that unchanged message with typed backpressure, without
allocating a delivery sequence, mutating inboxes, or dirtying views on rejection. This is not a
new Editor14 bus wrapper: without that owner contract, queue-front retry would require a deep
clone of `JobEventKind::{Progress, Failed}` text and violate the retained-payload bound.

2026-08-11 independent forward repair: ready admission is now a deterministic weighted cycle
(`Interactive, Interactive, Normal, Interactive, Normal, Background`) rather than strict
priority. A ready Background item with category capacity is therefore selected within one six-slot
cycle; category capacity, dependencies, mutex tails and per-priority `JobId` FIFO remain
authoritative. `EditorJobSpec` now owns one `Arc<str>` label which event sinks clone without a
String-to-Arc conversion. The former mixed `system/mod.rs` implementation is hard-cut into
construction, submission, lifecycle, scheduling and progress-observer leaf modules. This forward
repair is source-level only until the listed managed current-source tests and the existing
performance/WPR gates complete; the failure remains `open` because the Editor02 lossless producer
contract is still a required lower-layer dependency.

2026-08-11 current-source closure correction: the original M4 candidate listed only 22 input paths
while `system/mod.rs` declared ten leaf modules. It omitted M1 `admission_ledger.rs`,
`admission_reservation.rs`, `pending_task.rs`, and the corresponding M1 admission/test source, so
it cannot be used for a source-bound compile or behavior receipt. The successor
`2026-08-11-m1-m3-m4-current-source-manifest.md` freezes the complete 40-path union; the old
candidate is historical only. Three M1 Rust formatting drifts were mechanically repaired, but no
Cargo, scale matrix, WPR, fixed return, or Editor02 contract result is claimed here.

## 产出记录与时间

请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前 failure 保持 `open`：已有 admission/pump 的静态前向修复和设计复审，但 lifecycle delivery
reservation 仍依赖 Editor02 的 non-consuming lossless producer contract，且受管 Cargo、规模矩阵和
Windows WPR 尚未完成。完整、唯一的 open 状态记录见
[`2026-08-11-job-pump-output-records.md`](2026-08-11-job-pump-output-records.md)。
