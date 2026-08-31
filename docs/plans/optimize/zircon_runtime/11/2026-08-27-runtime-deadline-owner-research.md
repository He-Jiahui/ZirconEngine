# Runtime11 Deadline Owner Architecture And Profiling Gate

- Date: 2026-08-27
- Owner plans:
  - `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
  - `docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- Status: `research_complete_measurement_required_before_structural_optimization`

## Decision

Do not mechanically move the current process timer thread into every
`ExecutionRuntime`. The current defect is mixed ownership, while the preferred
engine model has two distinct services:

1. frame-bound gameplay and runtime timers are owned by a runtime/world and
   advanced from its authoritative tick;
2. headless wall-clock lifecycle deadlines use at most one explicitly owned
   wait service, with a close/join receipt before its callback executor stops.

The existing timer remains unchanged in this slice. The first safe preparatory
cut removes `ProjectAssetManager::default()` from the production `AssetModule`
factory and injects the activating runtime's `Io` pool. Timer restructuring
must follow product measurement and explicit consumer injection, not precede
them.

## Current Source

`core/runtime/tasks/timer.rs` owns one bounded process service:

- `PROCESS_TIMER` is `OnceLock<Result<TaskTimer, String>>`, so the product
  process retains the timer owner until process exit;
- one named OS thread sleeps on a condition variable and a `BTreeMap<Instant,
  ...>` deadline index; registration/removal is `O(log N)` and capacity is 512;
- a `HashMap` gives expected `O(1)` registration lookup/cancellation;
- periodic callbacks are rescheduled from the current time and one
  `delivery_pending` bit coalesces slow delivery, so no catch-up backlog is
  created;
- the worker blocks on a condition variable rather than polling, so source
  inspection alone does not establish a CPU or power hotspot;
- `TaskTimer::new(...)` is nominally explicit but still constructs
  `TaskCallbackDispatcher::process_default()`, so callback execution escapes
  the explicit owner;
- clone lifetime is tracked by a manual owner count and only the last
  non-worker drop joins. The process `OnceLock` means product shutdown never
  reaches that last drop.

## Consumer Matrix

| Consumer | Deadline meaning | Current owner problem | Required direction |
|---|---|---|---|
| Asset worker completion expiry | headless wall-clock retention bound | process timer and process callback pool | inject one runtime lifecycle deadline handle beside the runtime `Io` pool |
| Bounded keyed I/O admission/fence expiry | headless wall-clock admission bound | lane constructor receives a scheduler but reacquires the process timer | inject scheduler and deadline service from the same runtime owner |
| Runtime operation maintenance | session-owned task/terminal TTL | process timer outlives the dynamic session | make the session/runtime own the deadline handle; cancel and drain before runtime workers |

Gameplay/world timers are not current `TaskTimer` consumers and must not be
folded into this control-plane service as an accidental compatibility path.

## Unreal Reference

`dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TimerManager.cpp`
constructs `FTimerManager` for an owning `UGameInstance`. Timers live in
pending/active heap state and `FTimerManager::Tick` advances them once per
engine frame. The manager also removes expired timer delegates when a library
is unmounted. It does not create one private deadline thread per game instance.

`dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Containers/Ticker.cpp`
implements the process core ticker as a host-driven `Tick` over an incoming
queue and retained elements. Removal coordinates with a currently executing
callback, and `Reset` clears queue/time/state. It is a process service but is
still driven by the owner thread rather than an invisible polling worker.

The applicable Zircon rule is ownership and shutdown ordering, not a literal
port of Unreal containers: runtime/world timers are tick-owned; genuinely
headless deadlines share one explicit wait owner; library unload removes
callbacks before code and worker teardown.

## Candidate Architecture

### Runtime Tick Timers

- owner: `CoreRuntime` or a world/session service below it;
- clock: the runtime's authoritative monotonic/frame time policy;
- storage: indexed min-heap or the current ordered deadline map at the measured
  maximum registration count;
- execution: bounded callbacks drained by the owner tick, never a hidden
  process worker;
- shutdown: close admission, remove callbacks by runtime/module owner, drain or
  cancel under a bounded per-frame policy.

### Runtime Lifecycle Deadlines

- owner: one explicit `ExecutionRuntime` auxiliary owner, not each consumer;
- clock: monotonic wall clock, independent of paused or absent frames;
- execution: callback dispatcher bound to that runtime's `AsyncCompute` or
  `Io` domain;
- handles: weak consumer routes so retained subscriptions cannot retain the
  wait thread or callback executor;
- shutdown order: close deadline admission, cancel registrations, wake and
  join the wait thread, drain already admitted callbacks, then close/join pool
  domains;
- receipt: expected/exited/joined wait-worker count, pending registration count,
  and admitted callback count must be exact before unload.

One Runtime lifecycle deadline owner is the MVP ceiling. A timer wheel, per-CPU
shards, or multiple wait threads is unjustified at the current 512-registration
bound without measured contention.

## Profiling Matrix

Use one source-bound product build and compare the current process timer,
tick-driven prototype, and explicit wait-owner prototype at 0/1/64/512 active
deadlines with 1 ms, 16 ms, 1 s, and 60 s periods. Record:

- timer worker wakeups and context switches per second;
- process and timer-thread CPU time while idle and under expiry load;
- package/process power during a five-minute idle window;
- deadline error p50/p95/max and callback queue delay p50/p95/max;
- registrations, cancellations, coalesced deliveries, pending deliveries, and
  peak retained callback bytes;
- process thread count before runtime start and after the shutdown receipt;
- shutdown p50/p95/max with zero, queued, executing, and self-cancelling
  callbacks.

The prototype may replace the current thread only if it preserves headless
deadline semantics, leaves zero owned workers/callbacks after shutdown, and
does not materially regress deadline error. Power or CPU improvement must be
reported from the matrix; source structure alone is not evidence that the
current condition-variable sleeper is a performance bottleneck.

## Remaining Work

- expose explicit runtime deadline injection without a process-default
  fallback in production module/session constructors;
- profile the current timer before changing its algorithm or thread model;
- implement the selected tick/wait split and add its shutdown census;
- migrate asset expiry, bounded keyed I/O, and operation maintenance;
- remove the process timer only after source scans prove no production
  consumer and managed shutdown tests prove zero retained callbacks/workers.
