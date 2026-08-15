# Editor14 Job Event Delivery Reservation Analysis

## Scope

This analysis continues the open
`failure-2026-07-17-job-pump-budget-and-pending-scan.md`. It covers only the
worker-to-main lifecycle delivery path:
`EditorJobSystem -> JobEventSink -> JobEventQueue -> JobEventPump -> EditorMessageBus`.
It does not replace the existing pending-admission, worker-category, or
progress-observer limits.

## Current-Source Trace

- `PendingJobQueue::ensure_admissible` bounds pending entry and estimated-byte
  state, while `take_next` removes a job from that state before it runs.
- `JobEventQueue` retains every lifecycle event in `VecDeque`; it only
  coalesces progress events by `JobId` in `latest_progress`.
- `JobEventPump` applies the correct 64-event / 1-ms delivery budget, but it
  does not bound retained lifecycle records between calls.
- A stalled host can therefore allow a job to finish, submit another job after
  its pending entry is removed, and repeat this sequence. Pending admission
  remains within its budget while Started and terminal event records grow with
  the number of completed jobs.
- `Arc<str>` removes repeated label-buffer allocation from each event emission,
  but it does not bound the number of retained event objects.
- `EditorJobSpec::label`, `JobContext::report_progress` message, and failed
  event text are currently unbounded `String` inputs. A lifecycle-count cap
  alone would therefore still permit one retained event to consume arbitrary
  memory.
- `SharedEditorMessageBus::publish` returns a dispatch report. Its lossless
  inbox preflight can report `backpressured`, but `JobEventPump` currently
  ignores that report after removing the event from its queue. A full inbox can
  therefore also lose a Started or terminal edge at the bus boundary.

## Reference Check

**Primary reference -- Unreal TaskGraph.** `FTaskGraphInterface` exposes
`QueueTask` separately from named-thread processing
(`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h`),
and its implementation gives each named thread a priority-aware
`FStallingTaskQueue` plus a re-entry guard
(`dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/TaskGraph.cpp`).
That split is the transferable architectural rule: worker routing and wake-up
are a scheduler concern, while named-thread execution/observation is a
separate, explicit queue concern. `ProcessThreadUntilIdle` is deliberately an
idle-drain API, not a per-frame presentation SLA; Zircon must therefore not
copy it into retained-host ticks.

Zircon's `JobEventPump` is the latter concern. Its lifecycle queue needs a
bounded, lossless delivery reservation whose release is driven by successful
message-bus acknowledgement, not a replacement task scheduler or a
worker-blocking queue. This keeps `JobScheduler` as the only execution owner
and gives the retained host an independently measurable count/time budget.

**Contrast -- Godot CallQueue.** Godot's `CallQueue` uses a fixed page size
and a `max_pages` capacity. Its enqueue path explicitly returns an
out-of-memory error when the page cap is reached instead of silently growing
the main-thread queue (`dev/godot/core/object/message_queue.h` and
`dev/godot/core/object/message_queue.cpp`). Zircon cannot copy its
drop-on-full behavior because this failure requires Started and terminal job
edges to remain observable. The transferable principle is an explicit bounded
delivery resource with an auditable backpressure outcome.

## Cross-Plan Ownership

The non-consuming lossless dispatch outcome belongs to the already-open
Editor02 message-owner failure
`docs/plans/zircon_editor/editor/02/failure-2026-07-17-message-inbox-backpressure-and-fanout.md`.
That record already owns `bus.rs`, `shared.rs`, lossless inbox admission and
fanout payload ownership. It must expose a way for a producer to retain its
original message when lossless preflight backpressures, without forcing an
extra deep clone. Editor14 owns the consumer-side queue-front retry and the
job delivery reservation after that contract is available. No duplicate
Editor02 failure or compatibility wrapper is permitted.

## Required Design

Add a job-event delivery reservation owned by `EditorJobSystem`:

1. Admission reserves two lifecycle delivery slots and their worst-case text
   bytes for every accepted job. The reservation covers Started plus one
   terminal edge; a pending cancellation still keeps both slots until its
   terminal edge is delivered, which is safe and deliberately conservative.
   `submit_batch` validates and reserves `2 * request_count` in the same state
   transaction before registering any job. A keyed `Merged` outcome changes no
   reservation, and dependency/admission failure rolls back without a leak.
2. The reservation remains held after job completion. It is released only
   after `JobEventPump` has received a lossless-success dispatch report for
   that job's terminal event from `EditorMessageBus`. A stalled or
   backpressured pump therefore applies backpressure to new submissions before
   lifecycle queue growth becomes unbounded.
3. `EditorJobSpec` validates a maximum UTF-8 label size at submission. Progress
   and failed-event text are normalized to an explicit UTF-8-safe display cap
   with a truncation diagnostic before enqueue, so queue byte accounting has a
   real upper bound. The original typed job error remains the ticket result;
   display text is not the error authority.
4. Submission with no remaining delivery entry, byte, or oldest-age capacity
   returns a new typed backpressure error. It must not block a worker, a
   retained-host callback, or `cancel`; it must not drop or overwrite a
   lifecycle event. Delivery age starts when the first not-yet-losslessly-
   dispatched queue record is enqueued, not when the job is admitted. Started
   acknowledgement removes its age contribution; a later terminal or first
   Progress record starts a new contribution. Replacing an already-queued
   Progress record retains its first enqueue time.
5. Progress remains one coalesced queue record per not-yet-released lifecycle
   reservation. A terminal event does not erase an already queued Progress
   record because callers still observe ordered Started/Progress/terminal
   edges. `EditorJobLimits` therefore reserves one bounded Progress headroom
   for every lifecycle reservation, not merely for active jobs. Total retained
   event-queue capacity is the sum of two lifecycle records plus one Progress
   record per outstanding reservation; progress does not consume either
   lifecycle slot.
6. Pump dispatch preserves a lifecycle event until the bus report has no
   `backpressured`, dropped, or dispatch-error outcome. On lossless inbox
   backpressure it retains that event at queue front and ends the pump pass;
   it must not consume the terminal reservation or advance later lifecycle
   edges. It move-outs the queue-front event while holding only the queue lock,
   releases that lock before bus dispatch or reservation/state work, and on a
   failed report reacquires only the queue lock to push the same event back to
   front. This preserves the existing `state -> progress -> queue` producer
   ordering and forbids `queue -> state` lock acquisition. Progress may remain
   coalesced under the existing latest-value rule: a backpressured Progress
   delivery token may be discarded without releasing the lifecycle reservation,
   because `EditorJobProgressSource` remains the authoritative latest progress;
   Started and terminal edges never take that path.
7. Capacity and text caps are explicit `EditorJobLimits` values with
   entry/byte/oldest-age snapshot counters. They are not inferred from arbitrary
   queue length or hidden in `JobEventPump`.

## Rejected Alternatives

- A bounded queue that drops lifecycle events violates the established
  Started/terminal edge contract.
- Blocking `JobEventSink::emit` can deadlock or stall main-thread cancellation
  paths because pending cancellation emits synchronously.
- Releasing capacity after Started delivery allows terminal records to grow
  again while a long-running job completes after new submissions consume the
  released slots.
- A second UI-side queue or timer duplicates the job system's admission truth.
- A count-only cap leaves unbounded label/progress/error strings in retained
  records and cannot prove the required byte budget.
- Consuming an event before inspecting the message-bus dispatch report silently
  loses lifecycle edges whenever a lossless subscriber inbox is full.

## Regression Matrix

- With a two-slot delivery limit and a stalled pump, one immediate job may run
  and terminalize; a second submission must return typed delivery backpressure.
- Pumping the first job's Started and terminal edges releases its reservation;
  the second submission then succeeds and both lifecycle edges remain ordered.
- A failed batch admission, dependency validation failure, or mutex validation
  failure leaves delivery reservations unchanged; an accepted `N`-job batch
  reserves exactly `2 * N`; keyed merge leaves the existing reservation intact.
- Cancelling a pending job holds its conservative reservation until Cancelled
  is published, then releases it without blocking `cancel`.
- `shutdown -> cancel_pending` follows the same rule: every pending Cancelled
  edge retains its reservation until lossless pump success, while mutex and
  dependency terminal bookkeeping can still make progress.
- Repeated Progress for an active job remains one retained record while no
  lifecycle edge is dropped. A stopped-pump sequence in which every job emits
  Progress then terminal proves the three-record-per-outstanding-reservation
  entry and byte bound.
- A full lossless job subscriber inbox leaves the Started or terminal edge at
  queue front, publishes no later lifecycle edge, and does not release the
  reservation until a later successful pump.
- A backpressured Progress delivery may be removed under the latest-value
  policy while its lifecycle reservation remains held; a later terminal still
  follows the lossless retry rule and the progress authority retains the latest
  state for resynchronization.
- A synthetic clock proves that a long-running job whose Started edge was
  already acknowledged does not trigger delivery-age backpressure, while an
  undelivered terminal or first Progress record does; Progress replacement
  preserves the original queue-age start time.
- With `Started(t0) -> Terminal or first Progress(t1) -> lossless
  Started acknowledgement(t2)`, the remaining oldest delivery age is still
  measured from `t1`, not reset at `t2`.
- A watchdog-controlled interleaving of worker emit, lossless-bus
  backpressure, `cancel` or shutdown, and retry pump proves the queue is not
  held while bus/state/progress locks are acquired and the front event is
  restored ahead of concurrently appended events.
- Oversized label submission returns a typed input error; oversized progress
  and failed display text are UTF-8-safely bounded and record truncation while
  ticket error identity remains intact.
- A 1/1k/10k stopped-pump stress run records lifecycle reservations, retained
  queue entries/bytes/oldest age, rejected submissions, truncation, terminal
  latency and RSS. The managed Windows profile, not a static estimate,
  determines the production capacity.

## Non-Goals

- No global interner, worker blocking, consumer-specific drop policy, or
  EditorMessageBus compatibility path.
- No Editor14 copy of Editor02's lossless inbox/fanout implementation.
- No performance-pass claim before the managed stress/profile evidence exists.

## Performance Conclusion

The current implementation does not satisfy a bounded main-thread delivery
memory contract: it has an unbounded lifecycle accumulation path, unbounded
event display text, and a lossless-inbox acknowledgement gap. The `Arc<str>`
label change is a narrow allocation reduction only; it is not evidence that
this failure is closed.

No lifecycle-reservation code may be accepted until all of the following are
available: the Editor02 non-consuming lossless-dispatch contract, a focused
current-source correctness gate, and a managed Windows 1/1k/10k stopped-pump
profile reporting entry/byte/age/RSS/terminal latency. The profile selects the
default capacity; it must not be invented from a queue-length constant.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-08-10 | `analysis_complete / design_review_pending` | 完成 stopped-pump lifecycle backlog 的当前源码推导、Godot bounded queue 对照和 reservation-based nonblocking design；补足 label/progress/error text 的 byte cap、UTF-8 truncation diagnostic、bus lossless dispatch acknowledgement 与 entry/byte/age 回归。 | `spec.rs` label、`context.rs` progress message、`pending.rs` admission removal、`pump.rs` lifecycle `VecDeque`、`system/mod.rs` completion promotion、`editor_message/bus.rs` lossless preflight；bus non-consuming outcome 已路由既有 Editor02 failure，未改生产代码，尚无 profile。 |
| 2026-08-10 | `performance_conclusion_not_accepted` | 明确当前 queue 不能证明 lifecycle/text/bus-boundary memory or lossless semantics；冻结进入实现前必须满足的 Editor02 contract、focused gate 和 managed Windows profile。 | 本文 Current-Source Trace、Cross-Plan Ownership 与 Regression Matrix；未把静态推导、queued ticket 或 allocation 预期升级为性能结论。 |
| 2026-08-10 | `design-review-findings-forward-fixed / second-review-pending` | 补齐 batch/merge reservation 原子性、active-progress headroom 的总 entry/byte 上界，以及 shutdown pending-cancel 的 terminal-release 回归。 | 独立设计复审 `Critical/Important/Minor = 0/2/1`；本表与 Required Design/Regression Matrix 的前向修正，未改生产代码。 |
| 2026-08-10 | `second-design-review-finding-forward-fixed / final-review-pending` | 定义 delivery oldest-age 为最早未 lossless dispatch 的 queue record age，而非 job admission age；Started ack 后清除贡献，Progress replace 保持首次入队时间。 | 二次独立设计复审 `Critical/Important/Minor = 0/1/0`；新增 long-running/undelivered/progress-replace synthetic-clock 回归，未改生产代码。 |
| 2026-08-10 | `final-design-review-minor-forward-fixed / closeout-review-pending` | 补充 `Started(t0) -> Terminal/Progress(t1) -> ack Started(t2)` 时 remaining oldest-age 仍起算于 `t1` 的合成时钟回归，锁定队首 ack 不会重置已经排队记录的真实年龄。 | 最终独立设计复审 `Critical/Important/Minor = 0/0/1`；未改生产代码。 |
| 2026-08-10 | `closeout-review-important-findings-forward-fixed / re-review-pending` | 将 Progress headroom 从 active-job 纠正为每个 outstanding reservation 一条，保留 Started/Progress/terminal 顺序；明确 queue-front retry 的 move-out/dispatch/reinsert 无锁序协议。 | closeout review `Critical/Important/Minor = 0/2/0`；新增 stopped-pump complete-progress 和 concurrent retry/cancel watchdog 回归，未改生产代码。 |
| 2026-08-10 | `design-closeout-review-clean / implementation-prerequisites-open` | 独立复审确认每个 outstanding reservation 的三记录上界、Progress latest-value backpressure 处理、queue-front retry FIFO 与无锁序环、batch/merge、cancel/shutdown、oldest-age 和 watchdog 回归彼此一致。 | closeout re-review `Critical/Important/Minor = 0/0/0`；`git diff --check` 通过。仍需 Editor02 non-consuming lossless dispatch contract、focused current-source gate 和 managed Windows 1/1k/10k profile，未改生产代码。 |
| 2026-08-10 | `unreal-taskgraph-architecture-reference-recorded` | 以 Unreal TaskGraph 复核 worker 路由与 named-thread queue 的职责边界：Zircon 的 `JobEventPump` 是主线程交付层，不复制 worker task graph，也不使用 idle-drain 取代每帧 SLA。 | Unreal `FTaskGraphInterface::QueueTask`/`FStallingTaskQueue` 与当前 `JobEventPump`/`EditorMessageBus` 对照；保留 Editor02 lossless acknowledgement 为实现前置，尚未进入生产优化或性能结论。 |
| 2026-08-11 | `current-source-audit-complete / editor02-contract-prerequisite-open` | 复核当前 `JobEventPump`、`SharedEditorMessageBus`、现有 1k storm baseline 及 Fyrox task completion path。确认 Pump 在队首 `pop` 后只能调用消耗 `EditorMessage` 所有权的 `publish`，再事后读取 dispatch report；lossless inbox backpressure 时无法将原事件安全放回队首。现有 1k baseline 仅记录 wall-clock 观察，不能替代 stopped-pump 的 entry/byte/age/RSS/terminal-latency profile。 | `zircon_editor/src/core/jobs/pump.rs`、`zircon_editor/src/core/editor_message/shared.rs`、`bus.rs` lossless preflight、`tests/background_storm_contract.rs`；Unreal `TaskGraphInterfaces.h`/`TaskGraph.cpp` 作为主参考，Fyrox `fyrox-core/src/task.rs` 与 `fyrox-impl/src/engine/task.rs` 作为 Rust 对照。未新增 Editor14 workaround、兼容 wrapper 或第二队列；继续由 Editor02 提供 non-consuming lossless dispatch contract。 |
