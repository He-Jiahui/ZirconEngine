---
related_code:
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs
  - zircon_runtime/src/scene/world/schedule.rs
  - zircon_runtime/src/scene/ecs/native_system_schedule_diagnostics.rs
canonical_failure: docs/plans/zircon_runtime/runtime/03/failure-2026-07-22-production-schedule-remains-serial.md
secondary_failure: docs/plans/zircon_runtime/runtime/03/failure-2026-07-17-schedule-executor-frame-allocations.md
primary_reference:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/TaskGraph.cpp
secondary_reference:
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
status: proposed_pending_current_source_windows_measurement
created_at: 2026-08-15
---

# Scene Schedule Execution State Measurement And Deferred-Key Reuse Gate

## Scope

This is a current-source algorithm and measurement record for the product scene schedule. It is not
a timing claim. It does not create a second failure lifecycle, implement an optimization, or claim
a CPU, allocation, GPU, power, or product result. Runtime03 owns the linked canonical failures and
the currently modified source files.

The production path is:

```text
WorldDriver::tick_level
  -> SceneScheduleRunner::run_stage
  -> worker-safe native batches / explicit main-thread lanes
  -> World deferred-command merge and ApplyDeferred boundary
```

`ScheduleParallelExecutor` is deliberately outside this proposal. Current-source callers are test
only, while the product runner uses `SceneScheduleRunner` and its `JobScheduler::join` recursion.
Replacing the product path with generic `Arc<Mutex<Result>>` jobs would reintroduce a different
control plane without proving World access safety.

## Current Source Facts

The 2026-08 product repair is architecturally meaningful: `SceneScheduleRunner` groups only
worker-safe native systems whose compiled `ScheduleConflictGraph` proves no conflict. It removes
those systems from `World`, executes without a shared `&mut World`, then deterministically merges
worker-local command buffers and restores taken systems through the same unwind boundary. Internal,
runtime, non-worker-safe, and deferred-barrier steps remain explicit main-thread lanes. Any future
work must preserve those ownership and failure semantics.

The compiled `SceneScheduleStagePlan` already retains per-stage ordered native steps and conflict
graphs. A native `ScheduledSceneStep`, however, retains a `String` id, stage, and order rather
than the corresponding `DeferredSystemKey`. That produces two separate stable-frame costs:

1. A worker dispatch calls `DeferredSystemKey::compiled(stage.rank(), order, id)`. The conversion
   from `&str` to `Arc<str>` allocates a new key payload for each dispatch before the key is bound
   to the worker command buffer.
2. A main-thread native step calls `World::run_native_scene_system(id)`. That method asks
   `SceneScheduleStagePlan::native_system_deferred_key` to walk stages and native steps until it
   finds the supplied id, then rebuilds its key. With `M` scheduled native steps and `L` such
   main-thread calls, the worst-case lookup cost is `O(L * M)` inside an otherwise compiled plan.

Worker-batch vectors are also not fully observable today. The product diagnostics record a count
and capacity-byte proxy for the `systems`, `timings`, and, when present, command-buffer reference
vectors. They do not separately account for the flush-local `system_ids` vector or the stage-local
`WorkerDispatch` vector. These are coverage gaps in a proxy metric, not proof of allocator cost.
The existing values must never be presented as global allocator-call counts.

## Reference Review

Unreal is the primary lifecycle reference. `FTaskGraphInterface` explicitly starts and shuts down
the process task graph (`TaskGraphInterfaces.h`, lines 224-244; `TaskGraph.cpp`, lines 1786-1815)
and attaches named threads to that long-lived scheduler (`TaskGraphInterfaces.h`, lines 284-288).
Zircon already has the analogous long-lived `JobScheduler`; it must not create a worker pool per
frame. Unreal does not prescribe Zircon's ECS access model or key representation.

Bevy is the direct schedule-state reference. Its `MultiThreadedExecutor::init` allocates conflict
metadata, dependency counts, ready/running bitsets, completion queue capacity, and per-system
metadata once for the compiled `SystemSchedule`. Its subsequent `run` resets and reuses that state
while scheduling only access-compatible systems. The transferable rule is compiled schedule identity
plus reusable execution state. Zircon must retain its safe World take/restore and deterministic
worker-command merge rather than importing Bevy's unsafe world-cell implementation.

## Measurement Contract

Before any state reuse, record a managed Windows current-source baseline from the real
`WorldDriver` path. Store artifacts only beneath an approved `D:\ZirconBuilds`,
`E:\ZirconBuilds`, or `F:\ZirconBuilds` session root. No `C:` artifact is permitted.

For every sample, retain source fingerprint, frame index, stage, schedule-plan identity, workload
identity, timestamp, and a single completed-batch association. Capture at least three repetitions
for each of these dimensions:

| Dimension | Values |
| --- | --- |
| Native system count | 1, 16, 256, 10k |
| Parallel width | 1, 2, 8, 64 worker-safe systems |
| Conflict shape | all disjoint, one conflict boundary, all main-thread/exclusive |
| Change mode | cold schedule build, unchanged schedule, one-percent schedule change |
| Deferred work | no commands, ordered commands, worker panic, merge/apply failure |

The existing metrics remain required: worker batch count, conflict count, callback count, ready
delay, worker utilization, callback p95, and the documented temporary-control-buffer capacity
proxy. The next observability slice must add separate fields for `system_ids` capacity, stage
dispatch capacity, compiled-key build count, and compiled-key payload bytes. Those fields remain
allocation proxies unless an allocator tool reports allocator events independently.

The report must independently retain stage wall time, worker callback wall time, ready delay, merge
and ApplyDeferred wall time, World lock acquire/hold counts and time, jobs/waits, command count,
and the state/event/deferred ordering result. CPU timing cannot establish GPU time or system power;
those measurements require their own supported capture sources and equivalent workloads.

## Decision Gate

Do not optimize from static complexity alone.

1. If stage wall time is dominated by callbacks, World lock, merge, or deferred application, repair
   that measured owner rather than cache keys or vectors.
2. If unchanged schedules show material key construction, plan lookup, or control-buffer proxy work
   on the user path, implement the smallest compiled-plan key reuse cut below.
3. Compare cold build, one-percent change, unchanged-frame p50/p95, RSS, and allocation evidence
   before and after. Reject a change that shifts frame work into schedule publication or weakens the
   current panic/ordering guarantees.
4. Do not route the generic closure executor into production, add a global World mutex, or suppress
   diagnostics to make a result appear cheaper.

## Candidate Implementation Cut

This design is conditional on the measurement gate.

1. Compile one `DeferredSystemKey` into each native `ScheduledSceneStep`, using an `Arc<str>`
   created at schedule publication. `ScheduledSceneStepRef` borrows that key. Worker dispatches
   clone only the `Arc` handle when a command buffer needs ownership; they do not reconstruct the
   payload from `&str` every frame.
2. Add a narrow `World` native-system invocation that accepts the already compiled key. It performs
   the existing take/run/restore transaction without calling `native_system_deferred_key`; the old
   public/direct lookup remains for callers that genuinely lack a compiled step.
3. Only after the new counters prove temporary control buffers material, attach reusable scratch
   state to the compiled plan or its owning runner, keyed by schedule generation. Scratch reuse must
   be exclusive to the frame runner, cleared on panic/error, and must not retain taken systems,
   worker command payloads, or a second World authority.
4. Preserve non-Send/exclusive/main-thread routing, conflict checks, stable command key order,
   batch-level ApplyDeferred, panic cleanup, and restore-before-rethrow behavior. Add focused
   regression coverage before changing the first source line.

## Current Status

The four implementation files required for this candidate are currently modified by another
Runtime03 change set, so this record deliberately makes no source edit. The immediately useful
non-validation work is complete: product-path identification, key-lifecycle analysis, observability
gap definition, primary/secondary engine comparison, and a falsifiable measurement gate. No Cargo,
Pester, product launch, screenshot, profiler capture, or performance claim was performed.
