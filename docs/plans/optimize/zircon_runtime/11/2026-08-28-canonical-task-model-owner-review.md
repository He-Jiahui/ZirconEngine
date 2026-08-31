# Runtime11 Canonical Task Model Owner Review And Profiling Gate

- Date: 2026-08-28
- Owner plan: `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- Status: `source_cutover_complete_managed_validation_and_profile_pending`

## Decision Boundary

This review was completed before changing the task scheduler algorithm. The
current defect is an ownership split, not evidence that a lock, queue, worker
count, or scheduling policy is the performance bottleneck. This slice may
converge contract ownership and remove duplicate task state. It does not
authorize priority queues, work-stealing changes, reusable execution slots, or
new worker threads without the measurement matrix below.

## Current Source Finding

The tracked plus current-source inventory contains:

- 16 Rust files referring to the framework `AsyncTask*` model;
- 20 Rust files referring to the executable `JobHandle` model;
- one `TaskGraphScope` record with its own mutable `AsyncTaskStatus` while the
  scheduled `JobHandle` independently owns completion, cancellation, panic,
  wait, prerequisite, and terminal-observer state;
- a dynamic-scene spawn task with a third mutable status lock and a separate
  cancellation bit beside its `JobHandle`;
- `TaskPollBudget` with no production consumer and `TaskCancellationPolicy`
  with no executor outside the Runtime task graph.

The split permits two authorities to disagree. A scoped task can be terminal
in `JobHandle` while its copied scene status is still pending or running, and
`TaskGraphScope::schedule` currently discards the scoped task handle it just
admitted and returns only the independent job handle. Poll count is consumer
observation data, not task lifecycle state, and does not belong in the
canonical status record.

## Reference Evidence

### Unreal Engine

`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Task.h` binds launch,
prerequisites, priority, completion, wait, and result access to one task
handle. Prerequisites are attached before launch and the returned handle is the
later wait/result authority. `TaskGraphInterfaces.h` similarly makes graph
event completion the prerequisite and wait authority. The transferable rule
is one executable handle per admitted task, not Unreal's exact C++ storage.

### Bevy And Fyrox

Bevy's task pools return executor-owned task handles and do not publish a
second framework DTO state machine. Fyrox routes asynchronous resource work
through its resource/task owner rather than asking each dynamic consumer to
maintain a parallel lifecycle record. These engines do not justify copying
Zircon's current framework DTO plus scheduler-state dual track.

## Chosen Architecture

1. `core::framework::tasks` retains only the backend-neutral
   `ParallelSliceExecutor` trait.
2. Runtime11 owns `TaskId`, `TaskDescriptor`, `TaskState`, `TaskStatus`,
   `TaskCancellationPolicy`, `TaskPoolKind`, and `TaskPoolDescriptor` under
   `core::runtime::tasks`.
3. The old `AsyncTask*` names and files are deleted in the same cutover; there
   is no re-export, alias, or compatibility module.
4. `TaskPollBudget` is deleted because no Runtime executor consumes it. A
   future main-thread executor must introduce its budget with the executor and
   measured caller, not preserve an unused DTO.
5. `TaskHandle` binds descriptor, executor-owned status, cooperative
   cancellation, completion, wait, terminal observation, and scoped
   prerequisites. `JobHandle` remains the private lowering target for scoped
   dependency fences and the public low-level fence for unscoped scheduler
   work; it is not a second admitted-task identity.
6. `TaskPoolKind` and `TaskPoolDescriptor` move with their only implementation
   owner. The neutral parallel trait does not expose either type.
7. `TaskDescriptor.kind` is a logical workload class for the single shared
   TaskGraph, not a physical pool selector. The misleading legacy `pool` field
   name is removed without an alias.

## Complexity And Performance Boundary

The ownership move is behavior-neutral and adds no hot-path work. The target
canonical model must retain:

- average `O(1)` task admission and status lookup;
- `O(prerequisites)` launch wiring and `O(1)` prerequisite completion work;
- `O(1)` state per live task plus explicit bounded observer/dependent storage;
- no per-poll mutation, duplicate status mutex, global status scan, or worker
  allocation;
- no task label, failure string, or consumer payload in low-cardinality hot
  diagnostics.

## Required Measurement Before Scheduler Optimization

Use one source-bound Windows release build with target and trace output on D or
E. Measure 1/2/8 workers and 1/1,000/100,000 tasks for no-op, dependency chain,
wide fan-out, cancellation-before-start, active cancellation, panic, and
terminal-observer lanes. Record:

- admission, queue, execution, explicit-wait, and shutdown p50/p95/p99/max;
- allocations and retained bytes per task/prerequisite/observer;
- worker active/parked time, wakeups, context switches, CPU, RSS, and power;
- submitted/queued/running/completed/failed/cancelled conservation;
- status disagreement count, which must be zero after single-authority cutover.

Only a material measured term may authorize a scheduler algorithm change.
Acceptance of the full cutover additionally requires zero `AsyncTask*` source
references, one terminal state authority, managed Runtime/Editor/App compile
gates, focused task/dynamic-scene behavior gates, and an independent review.

## 2026-08-28 Source Result

- framework task files: `9 -> 2` (`mod.rs` plus `ParallelSliceExecutor`);
- legacy `AsyncTask*` Rust references: `16 files -> 0`;
- public scoped submission return types: `JobHandle`/`TaskGraphTask` split ->
  one `TaskHandle` for `submit`, `schedule`, and `schedule_after`;
- dynamic-scene lifecycle authorities: task-graph status plus scene status plus
  cancellation bit -> one Runtime task record;
- poll clocks in task status: `1 -> 0`;
- final review corrections make detached cancellation/panic update the same
  typed dependency fence, retain prerequisite handle leases until launch or
  prelaunch terminal, bind scoped terminal callbacks to the TaskGraph worker
  owner, deliver late observers safely after worker join, and move the entire
  pending/running/terminal lifecycle into the same synchronized `JobHandle`
  state used by status, completion, wait, and terminal observation;
- current structural mirror: owner modules `22/22`, behavior anchors `73/73`,
  missing anchors/modules/declarations `0`, oversized owners `0`, risks `0`;
- independent follow-up review: Critical `0`, Important `0`; the reviewer
  confirmed all public lifecycle observations read `JobStateInner.lifecycle`
  and `TaskRecordState` no longer owns a terminal state;
- `rustfmt` parse and `git diff --check`: passed;
- locked offline Windows compile reached `zircon_runtime` but the shared
  worktree has 152 unrelated current-source errors. Filtering the compiler
  output found no errors in the migrated contract, task-graph, or
  dynamic-scene paths; only existing diagnostics-owner and render-manifest
  validation errors remain in the broader filtered modules. This is not a
  managed Cargo pass.
