---
status: architecture_research_complete_test_layout_static_validation_complete_cargo_validation_pending_source_optimization_not_admitted
created_at: 2026-08-23
summary_slug: schedule-execution-architecture-and-performance-admission
origin_plan: docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
related_code:
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/world/schedule.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/EngineBaseTypes.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LevelTick.cpp
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
tests:
  - zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs
  - zircon_runtime/src/scene/ecs/schedule_runner/tests/
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop/split_layout.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
---

# Runtime03 Schedule Execution Architecture And Performance Admission

## Decision

No production performance edit is admitted by this record. Runtime03 already has the correct
authority split: `zircon_app` hosts the loop, `zircon_runtime::scene` owns the world and schedule,
and `JobScheduler` is the only worker execution surface. A future change must preserve that split;
it must not send a shared `&mut World` through generic jobs, add a second scheduler, or move scene
authority into the app host.

The source review identifies a credible stable-frame control-plane candidate, not a measured
bottleneck. `Schedule` caches an `Arc<SceneScheduleStagePlan>` and rebuilds it only after schedule
definition changes. The plan already caches per-stage steps and conflict graphs. In contrast,
`SceneScheduleRunner::run_stage` still forms a worker batch every frame, compares every candidate
worker-safe step against the current batch, and `flush_worker_batch` creates system-id, taken-system,
timing, and command-buffer-reference vectors for every worker batch. For a contiguous W-system,
conflict-free worker region, the greedy admission has O(W^2) conflict queries per frame; the graph
build itself is correctly a schedule-mutation-time O(N^2) operation. Whether the per-frame work is
material for a product schedule remains unproven.

The generic `ScheduleParallelExecutor` is not the product scene executor. Its per-batch
`Arc<Mutex<...>>`, id-vector, abort flag, and job-handle chain are measurement candidates only and
must not be used as product-scene performance evidence.

## Reference Evidence

| Reference | Evidence | Adopted constraint |
|---|---|---|
| Unreal Engine | `EngineBaseTypes.h` gives each `FTickFunction` a tick-group range and explicit prerequisites. `LevelTick.cpp` calls `StartFrame`, runs ordered groups, intentionally leaves `TG_DuringPhysics` non-blocking, establishes later barriers, then calls `EndFrame`. | Make worker/main lanes and deferred barriers compiled schedule data, with deterministic group boundaries. Do not derive runtime ordering from ad hoc per-frame registration or task completion order. |
| Bevy | `MultiThreadedExecutor::init` preallocates dependency, conflict, ready, running, completed, and deferred metadata when a schedule initializes. `run` resets reusable state and applies deferred work at controlled boundaries. Its tests cover skipped dependencies and panic-to-error behavior. | Cache execution metadata with the schedule generation and reuse frame scratch. Preserve explicit deferred application and failure restoration rather than optimizing them away. |
| Current Zircon | `SceneScheduleStagePlan` already owns mutation-time topology and conflict compilation; `SceneScheduleRunner` takes worker-safe systems out of `World`, runs them through `JobScheduler::join`, merges `WorkerCommandBuffer`s by compiled key, and restores every system across success and panic. | Keep safe take/run/merge/restore ownership as the starting point. Do not import Bevy's unsafe concurrent world-borrow mechanism until Zircon has an equivalent independently-audited access proof. |

This is an intentional divergence from Bevy's executor internals: Zircon preserves its existing
worldless-system and worker-command-buffer contract. It aligns with Unreal's explicit group and
barrier semantics while remaining compatible with the current Rust ownership model.

## Required Structural Landing Zone

If profiling admits a source change, the change must be an execution-plan extension owned by
`SceneScheduleStagePlan`, not a local cache inside `SceneScheduleRunner`.

1. Introduce a private compact native-system slot in the scene schedule registry. Public
   registration and diagnostics may keep stable string ids, but a compiled execution lane must not
   repeatedly hash or clone ids in the steady frame.
2. Compile each stage into ordered `MainLane`, `WorkerLane`, and deferred-barrier entries when the
   schedule generation changes. A `WorkerLane` contains only mutually compatible, worker-safe slots;
   exclusive, internal, runtime, non-Send, and barrier work remains in `MainLane`.
3. Precompute the `DeferredSystemKey` with the lane. The frame runner only borrows the compiled
   lane, takes its slots, invokes `JobScheduler`, merges worker-local buffers in stable lane order,
   and restores the exact slots.
4. Keep reusable timing and command-buffer-reference scratch scoped to the runner/execution plan
   lifecycle. Do not introduce a global mutable cache or retain world-system ownership across a
   panic boundary.
5. Hard-cut the present runner into a folder-backed subsystem before adding that responsibility:
   `schedule_runner/mod.rs` for private wiring, `stage_execution.rs` for orchestration,
   `worker_lane.rs` for take/run/merge/restore, and `tests/` by behavior family. The existing file
   is 293 production lines but carries 514 test lines; it is not over the R1.4 production limit,
   yet the behavior tests exceed the R4.2 inline-test threshold and must move with the refactor.
   No compatibility re-export or parallel runner is permitted.

This is deliberately a two-phase design. First prove the control plane is significant; only then
introduce stable slots and compiled lanes. A low-cost workload should retain the simpler current
implementation rather than pay persistent plan complexity without evidence.

## Profiling Protocol

WPR is available on this Windows host and was confirmed idle. No trace has been started and no
profile artifact exists: capture requires a coordinator-managed receipt and an E-drive output path.
All future artifacts must remain under a managed `E:\ZirconBuilds\...` directory, never C:.

| Phase | Workload | Required measurements | Decision use |
|---|---|---|---|
| A | Current product `WorldDriver` route with 1, 16, 256, and 10,000 native systems; conflict-free, conflict-dense, and explicit-barrier variants | frame CPU p50/p95, worker overlap, ready delay, callback p95, `temporary_control_buffer_count/bytes`, system take/restore count, `JobScheduler::join` count, schedule-plan rebuild count | Distinguish meaningful workload, control-plane cost, and mutation-time compilation. |
| B | Same scenarios after 300 warm-up frames, with 31 independent timed samples and a separate cold schedule-build pass | ETW CPU stacks, allocations, context switches, worker utilization, RSS, and package power/wake data where hardware supports it | Prove or reject per-frame allocation and quadratic admission as the dominant cost; do not mix cold compilation with steady frames. |
| C | Serial and worker-lane routes on the same deterministic scene fixture | final world state, event/deferred-command order, panic restoration, and all counters above | Ensure a measured speedup does not exchange correctness or determinism for throughput. |

The existing temporary-control-buffer byte metric is a capacity proxy, not an allocator measurement.
Phase B therefore requires ETW heap allocation data before calling allocation churn a bottleneck.
No power or cross-engine comparison is meaningful until the machine, driver, profile, scene, frame
count, and sampling procedure are recorded beside the trace.

## Admission And Validation Gates

Source optimization is admitted only if a current-source Phase A/B trace shows either (a) per-frame
worker admission/temporary-buffer work on the critical path with superlinear growth in W, or (b)
steady-frame allocation/context-switch cost that is material relative to the same fixture's serial
baseline. Otherwise, retain the existing implementation and close only the test-layout convention
repair when separately authorized.

An admitted implementation requires all of the following:

- Static plan compilation increments only after registration/removal/mutation, never on a stable
  frame; compiled-lane construction preserves explicit before/after constraints.
- A 1/16/256/10,000-system behavioral matrix proves worker overlap only for compatible slots,
  zero per-step `World` mutex sharing, deterministic deferred merge order, main-thread execution
  for exclusive/non-worker-safe lanes, and restoration after callback or merge panic.
- Warm stable-frame diagnostics prove the chosen target: zero runner-owned control-plane allocations
  when reusable scratch is enabled, or a documented nonzero lower bound with its owner and reason.
- WPR/ETW evidence reports before/after CPU p50/p95, allocation, context-switch, worker-utilization,
  wake, and available power data using the same fixture. Unsupported power counters are recorded as
  unavailable, not fabricated.
- The Runtime03 `ecs_schedule` gate, Runtime11 scheduler regressions, structural audits, `rustfmt`,
  and `git diff --check` pass through the coordinator's Windows validation route.

## Current Status

`architecture_research_complete_test_layout_static_validation_complete_cargo_validation_pending_source_optimization_not_admitted`.
The separately authorized R4.2 test-layout repair is complete: `schedule_runner.rs` is now a
296-line production owner, and its behavior tests are routed through six focused files under
`schedule_runner/tests/` (largest: `panic_recovery.rs`, 259 lines). The existing Runtime03
split-layout guard now prevents reintroducing inline or `include!` behavior tests and requires the
worker-dispatch, panic-recovery, typed-worker, callback-order, and shared-support owners.

Static verification completed with `rustfmt --check`, `git diff --check`, an exact 11-name
pre/post function-set comparison, and the Runtime03 schedule/frame-loop boundary audit reporting
19/19 source files, 10/10 guard files, no missing runner or behavior anchors, and no risks.
An ephemeral Windows `core-min` Cargo lane was started on F: but remained in the crates.io index
refresh before any Zircon crate compiled or test executed, then was cancelled. Dynamic Cargo
validation therefore remains pending; this is not a test-pass claim.

The next performance source change still needs a fresh scope covering the schedule registry, stage
plan, runner subtree, schedule tests, relevant structure audit owners, and the Runtime03 failure
record. This report does not close any Runtime03 failure, does not claim a benchmark, and does not
authorize a commit or WeCom notification.

## Completion Record

| Work item | Status | Evidence / remaining condition |
|---|---|---|
| Current-source execution-path review | complete | `SceneScheduleRunner::run_stage` performs per-frame greedy worker admission and `flush_worker_batch` allocates control vectors; `SceneScheduleStagePlan` retains schedule-definition-time topology and conflicts. |
| Unreal/Bevy architecture comparison | complete | Unreal tick groups/prerequisites and Bevy initialization-time executor metadata support compiled lanes with explicit barriers, while Zircon retains its safe worldless-system contract. |
| Structural target and hard-cut boundary | complete | The required landing zone is a folder-backed `schedule_runner` subsystem with compiled stage lanes, not a runner-local cache or a second scheduler. |
| R4.2 schedule-runner test-layout repair | static verification complete; Cargo pending | The 514-line inline test body moved without function-set changes to six folder-backed owners (25-259 lines); the production source is 296 lines and a Runtime03 split-layout guard prevents regression. `rustfmt`, whitespace, and the schedule/frame-loop boundary audit pass; the ephemeral Cargo lane did not reach compilation because its external index refresh stalled. |
| Product profiling and optimization admission | pending | No WPR/ETW trace or hardware-power receipt exists yet; capture the defined warm/cold matrix to an E-drive managed directory before source changes. |
| Runtime03 source optimization | not admitted | Admission requires profile evidence showing the control plane is materially on the steady-frame critical path. |
