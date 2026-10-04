---
title: Runtime11 Task Execution Authority And Blocking Lane Review
date: 2026-09-01
status: canonical_task_node_and_runtime_worker_domains_source_complete_static_audit_foreign_blocked
origin_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
---

# Runtime11 task execution authority and blocking lane review

## Decision

The 2026-08-28 framework DTO cutover is valid and must remain: `core/framework/tasks` owns only the
backend-neutral `ParallelSliceExecutor`, while Runtime owns `TaskDescriptor`, `TaskHandle`, and
`TaskStatus`. The remaining defect is inside Runtime. `JobScheduler` and `JobHandle` are still
public, exported through the Core facade and prelude, and used directly by production Editor, ECS,
dynamic-scene, I/O, plugin, and platform code. This is a second admitted-work model beside the
descriptor-led TaskGraph, not merely a private executor implementation.

The hard-cut target is one executable task node and one public handle:

1. `TaskDescriptor` is the only admission description. It owns stable identity, debug label,
   workload class, cancellation policy, and execution domain. Priority, affinity, and deadline
   fields remain future capabilities until a graph-owned ready queue defines their ordering and
   starvation semantics. Prerequisites are supplied to the same admission call.
2. `TaskHandle` is the only public lifecycle/result/fence authority. It owns wait, status,
   cancellation, terminal observation, prerequisite retention, and optional typed result access.
3. `JobHandle`, `JobScheduler::schedule/schedule_after/wait_all`, and their callback/dependency
   machinery become private TaskGraph implementation. Production consumers cannot construct a
   completion fence without a descriptor, scope owner, and shutdown census.
4. `TaskRecord`, completion state, and client lease are folded into one reference-counted task node.
   There is no compatibility wrapper or public conversion between `JobHandle` and `TaskHandle`.
5. Blocking file/pipe work uses the `Io` worker domain owned by the same TaskGraph. It remains under
   the same global worker budget, admission, descriptor, quota, diagnostics, cancellation, and
   shutdown authority; it is not a restoration of independent process-default `TaskPools`.

This supersedes the earlier decision in
`docs/plans/optimize/zircon_runtime/11/2026-08-28-canonical-task-model-owner-review.md` that kept
`JobHandle` as a public low-level fence. It does not authorize a new ready-queue algorithm or an
unmeasured priority implementation.

## Current-source evidence

| Path | Current SHA-256 | Finding |
| --- | --- | --- |
| `core/runtime/tasks/job_handle.rs` | `67818D50112F526AB49A67BD0E6C034D98BFDD1EECA67DB9FB9826C5FE2A231D` | `JobHandle` and canonical `TaskHandle` now project one `Arc<TaskNode>` lifecycle, but the low-level fence is still publicly exported. The root is 462 lines after moving node synchronization into its folder-backed owner and removing the obsolete graph-only completion constructor. |
| `core/runtime/tasks/job_handle/task_node.rs` | `30789C4C6A5FD8069D08EA2938CE5A462099B10CE89369E1E1BB312C4462AE83` | The 167-line canonical node owner contains descriptor identity, graph-owner identity, lifecycle synchronization, dependency/observer queues, cancellation flags and public-handle reference counting. Owner comparison upgrades the weak token before pointer equality, preventing stale graph/allocator ABA matches. A test-only poison hook keeps lock-recovery coverage on the canonical mutex without restoring the removed state wrapper. |
| `core/runtime/tasks/job_scheduler.rs` | `1412610D7964B5F51A825514F66BB75A38CB521DDCF6FE65B80450EEB0B104B9` | TaskGraph can submit an existing canonical node and inject its callback dispatcher while retaining scheduler diagnostics; public raw closure-only scheduling remains an unmigrated bypass. The graph can also construct a lightweight scheduler facade over its existing pool/dispatcher for direct descriptor-led dependency submission. Terminal retirement hooks are delegated to the folder-backed pending owner, dependency registration now short-circuits after terminal work is consumed, and graph-owned nodes use an explicit owner-bearing constructor. |
| `core/runtime/tasks/task_descriptor.rs` | `df7ed9c689ad29d91c0e3bd5575451c02bf5c9d792ae273485ae45c500ff2051` | Canonical descriptor's `TaskPoolKind` is now the concrete TaskGraph execution-domain selector; priority and affinity remain future measured work. |
| `core/runtime/tasks/task_graph/task_handle.rs` | `DFF3B2D7638E90DFBDD621CCF1D2AE0A23997811FF3318F140ED63F5C1FD0330` | Canonical handle delegates to the same node-backed completion projection, no longer stores a per-handle graph weak reference, and no longer owns `TaskRecord` or a per-handle client lease. Graph ownership is queried through the canonical node; `wait_all(&[TaskHandle])` is the canonical multi-handle synchronization point. |
| `core/runtime/tasks/task_graph/scope.rs` | `F2B1966F1A3A31897771F04E22EBE065C50878B48E1F8BE2599D935C6A7E2E3B` | Direct, direct dependency, and scheduler-backed scope admission select the descriptor domain in O(1); scheduler and descriptor domains must share the same physical owner, prerequisite handles must belong to the same graph, graph-owned terminal delivery remains on `AsyncCompute`, and scope retirement follows terminal publication. Direct `submit` now delegates to the same scheduler-backed pipeline as `schedule` and `submit_after`. |
| `core/runtime/tasks/task_graph/scope/tests.rs` | `E2A102E02B7A2F8ED178184DF7D371E86E6E20C5629A833D022B05B2E01BC82` | The 810-line scope regression route retains shared fixtures and core lifecycle behavior, including direct descriptor-led dependency submission and dependency-failure propagation, while delegating ownership and dependency-scale coverage to focused child files. |
| `core/runtime/tasks/task_graph/scope/tests/ownership.rs` | `DAF95302AA5A65D4DDA783103E4C6A786A8F70A5D92A0D9E84EC331560FF390A` | The 159-line ownership regression owner rejects cross-graph, detached, and stale post-shutdown prerequisites and proves public handle clones do not duplicate owner references. |
| `core/runtime/tasks/task_graph/scope/tests/dependency_scale.rs` | `6E4F0C4F7A3DF54A39520F2E13AC4A5355FA8D395C0DD0A2835896A9B1F5506F` | The 196-line dependency-scale owner retains deep-chain ordering and wide fan-in completion coverage, including canonical `TaskHandle::wait_all` synchronization and failure-after-all-terminal propagation, without inflating the lifecycle test route. |
| `core/runtime/tasks/task_graph/admission.rs` | `31C9E2CA6C09BDF494BE01C073F2C73F19C272479C7DAFD5C129951FEE0A2AD2` | Admission errors distinguish scheduler-owner mismatch from prerequisite graph-owner mismatch before submission or scope capacity is consumed. |
| `core/runtime/tasks/task_graph/engine_task_graph.rs` | `DADA716F48D1168748ED1D7CE9FDFC9C04AF371CDE9C160386152D0E8C61FB51` | One TaskGraph authority owns the budgeted `Io`, `AsyncCompute`, and `Compute` domains, its single callback dispatcher, one `Arc<()>` owner identity, and closes/joins all three under one lifecycle. The production root is 277 lines after moving tests to a folder-backed owner, adding the graph-owned scheduler adapter, and removing the obsolete direct completion constructor. |
| `core/runtime/tasks/task_graph/engine_task_graph/tests.rs` | `E32B03BC7E251D147235AE6572FBA1B4ED3EA5C5D05E137CB0BAA76D679DAD7F` | Folder-backed owner for the five graph inventory/shutdown behavior regressions; no production logic or test anchor was removed. |
| `src/tests/tasks.rs` | `6AB52A44823DA653AFE05ED58D76F487705DEC116CA3775D2D5420DC8A676267` | The shared task regression root is now 371 lines; diagnostics and terminal-observer suites are explicit folder-backed child modules, keeping the test owner below the structural split threshold while preserving all anchors. |
| `src/tests/tasks/diagnostics.rs` | `E347DF60CB0CF283994262C463A5F8EB7E98A604B081B0E8D9137954F8B76F55` | The 442-line diagnostics owner retains schedule, queue, dependency, lifecycle-conservation, concurrent-admission, worker-wait, panic and cancellation coverage. |
| `src/tests/tasks/terminal_observers.rs` | `33C7B61C5869E1E6EB9B5017B7ACC5E8A4458CB90CE78DEEF83BA5E15653CE38` | The 145-line observer owner retains registration timing, ordering, panic containment, dependency continuation ordering and handle re-entry coverage. |
| `src/tests/runtime_absorption/job_system/mirror_docs.rs` | `99944D458ABA09AC7341120F4C9F01CCBE01E984C9DA454292D033EBDD749029` | The 356-line Rust mirror reads the folder-backed task, graph and scope test owners plus the level-manager owner, matching the Python audit's 76-source-anchor visibility. |
| `core/runtime/tasks/job_handle/tests.rs` | `49BB372B66184354CF08D7825F521E96F242BBCF76A0758B4AD8B0B4826F2698` | Private JobHandle regressions now poison and inspect the canonical TaskNode path; no stale `JobState` or `handle.state` references remain. |
| `core/runtime/tasks/bounded_stream_io/lane.rs` | `a307572ae50584cd7d1d5fbd8efd5109544c887faed1cc7cc64e1a394c6e1370` | Pipe readers use the physical `Io` domain; lane capacity is the minimum of configured readers and actual IO parallelism. |
| `core/runtime/tasks/task_pool_kind.rs` | `84e59b0c4dd8e48ec14fe85ca766b97261b8eb0e2dc8ca3ca217c2956c43f7e6` | `Compute`, `AsyncCompute`, and `Io` are explicit TaskGraph execution domains. |
| `core/runtime/runtime.rs` | `b5c421eced8b9b8ee6818448924cad1d5347e77265d9667792a84b355c6cea53` | Core still exposes both `scheduler()` and `task_graph()`; its worker inventory regression now requires three domains under the conserved total budget. |
| `core/runtime/handle/core_handle.rs` | `0cac0c5bb6f7fe8a3e9c996cef6ac9a91a20ad25778752a8ababf8ff4c474452` | Cloneable Core handles also expose the raw scheduler. |
| `zircon_editor/src/core/jobs/system/state.rs` | `a96408a514c74bbb783ddaa7854a5aec730fea1849e991d748274b1c6520b5e4` | Editor job dependencies and mutex-group tails retain raw `JobHandle` values. |
| `scene/ecs/schedule_parallel_executor.rs` | `390b4254de938ab1dc6c416a9a07b0312b90a4b0cab966de0e146d033a1d78d0` | ECS still builds a raw `JobHandle` dependency chain and blocks on its tail outside TaskGraph scope accounting. |
| `scene/dynamic_scene/asset_reload/stage_task.rs` | `f6b80f8a9e045cb31c648c2fd64f07de6079e2435fb736cd868162385caf845d` | Asset reload still stores a raw completion handle and publishes business state separately. |

Tracked plus current-source inventory found `JobScheduler` in 38 production-like files and
`JobHandle` in 15. By contrast, descriptor-led `TaskDescriptor` appears in 15 production-like files
and canonical Runtime `TaskHandle` in only the render-manifest, bounded-stream, and dynamic-scene
families; Navigation's similarly named bake handle is a separate domain ticket. The inventory is a
route count, not proof that every textual occurrence is a public call.

## Unreal alignment

Local Unreal `Core/Public/Tasks/Task.h` binds prerequisites, priority, wait/completion, and result
access to the same `TTask`/`FTask` handle returned by launch. `TaskPrivate.h` stores prerequisites in
the executable task node, retains prerequisite references until execution, and makes waiting a task
operation. The transferable rule is not Unreal's C++ allocator or exact queue: admitted work has one
identity and one executable lifecycle object, while low-level scheduling state stays private.

Zircon already follows this rule at the new `TaskGraphScope` boundary. The required work is to make
that boundary universal and remove direct consumer access to its lowering implementation.

Unreal also keeps blocking storage execution out of the ordinary worker path: local
`IoDispatcherConfig.cpp` defines bounded I/O buffer memory and an explicit decompression worker
count, while `TaskGraphInterfaces.h` reports foreground/background worker domains separately. The
transferable rule is separate physical progress domains with one engine lifecycle and resource
budget, not a second public task API. Zircon therefore reuses its existing `TaskPools` assignment
policy inside `EngineTaskGraph`; it does not add an IO-only scheduler, handle, or process-global
owner.

## Isolated task-node allocation model

An E-drive optimized Rust model used 21 alternating samples and marker
`RUNTIME11_TASK_HANDLE_AUTHORITY_MODEL_V1`. It compared the current ownership shape
(`TaskRecord Arc + JobState Arc + HandleLease Arc`) with one `Arc<UnifiedTaskNode>`. Both lanes kept
the same descriptor and synchronization categories; callback/prerequisite payload growth and task
execution were excluded.

| tasks | lane | allocations | allocated bytes | P50 construction | P95 construction |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1 | current dual handle | 3 | 128 B | 200 ns | 600 ns |
| 1 | unified node | 1 | 88 B | 200 ns | 300 ns |
| 1,000 | current dual handle | 3,000 | 128,000 B | 246.2 us | 317.6 us |
| 1,000 | unified node | 1,000 | 88,000 B | 125.7 us | 150.8 us |
| 100,000 | current dual handle | 300,000 | 12,800,000 B | 37.62 ms | 69.94 ms |
| 100,000 | unified node | 100,000 | 8,800,000 B | 13.89 ms | 20.01 ms |

The model predicts two fewer allocations and 40 fewer modeled bytes per admitted task. It is not a
current-source Cargo benchmark and does not include queue, callback, result, label `String`, worker,
or power costs. Production mutation still requires allocation counters around actual TaskGraph
admission and a 1/1k/100k dependency/cancellation matrix.

## Isolated blocking-lane saturation model

A second E-drive optimized model used 21 samples and marker
`RUNTIME11_BLOCKING_IO_STARVATION_MODEL_V1`. It saturated every worker with a blocking operation,
submitted one compute marker, and released blockers after 5 ms. The separate lane is a topology
proof only; the target production design remains one TaskGraph authority with a bounded blocking
lane, not independent schedulers.

| workers | lane | compute started only after blocker release | compute P50 | compute P95 |
| ---: | --- | ---: | ---: | ---: |
| 1 | shared blocking pool | 21/21 | 11.66 ms | 113.25 ms |
| 1 | separate bounded lane model | 0/21 | 37.2 us | 566.5 us |
| 2 | shared blocking pool | 21/21 | 7.05 ms | 12.38 ms |
| 2 | separate bounded lane model | 0/21 | 66.0 us | 641.8 us |
| 8 | shared blocking pool | 21/21 | 8.94 ms | 52.44 ms |
| 8 | separate bounded lane model | 0/21 | 43.5 us | 358.0 us |

This proves the starvation condition of putting blocking operations on every shared worker. It does
not measure Zircon's real pipe frequency, Windows scheduler wakeups, RSS, CPU, energy, or the exact
Rayon behavior. Those measurements decide lane capacity and whether async OS I/O is justified.

## Required hard-cut order

1. Fold `TaskRecord`, `JobState`, and handle lease into one executor task node while preserving the
   current dependency-race, callback-order, worker-wait-assist, panic, cancellation and shutdown
   tests. Keep `JobHandle` private during the transition.
2. Add descriptor-led TaskGraph admission for detached, scoped, and prerequisite work. Add priority
   and execution-domain fields only with fixed ordering and starvation tests; do not implement a
   new scheduler queue yet.
3. Migrate Editor jobs, ECS schedule batches, dynamic-scene reload/staging, bounded keyed I/O,
   operation service, animation direct workers, plugin discovery, and platform persistence to
   owned TaskGraph scopes and canonical handles.
4. Add a runtime-owned bounded blocking lane admitted through the same TaskGraph. Move pipe/file
   blocking consumers only after current-source WPR/ETW confirms their blocking duration and
   concurrency; preserve one global worker budget and exact shutdown receipt.
5. Delete public `CoreRuntime/CoreHandle::scheduler`, public `JobScheduler` scheduling methods,
   public `JobHandle` exports, and production `TaskPool::spawn` bypasses. Retain only private backend
   primitives needed by the executor.
6. Run managed Windows Runtime/Editor/App gates, actual 1/1k/100k task profiles, 1/2/N worker
   blocking-I/O saturation, WPR context-switch/wakeup/CPU/RSS, and product power measurements before
   claiming the algorithm or power bottleneck is closed.

## Implemented source slice (2026-09-01)

The first hard-cut stage now folds descriptor ownership, lifecycle/completion state, cooperative
cancellation flags, prerequisite continuations, terminal observers, and public-handle reference
counting into one `Arc<TaskNode>`. `TaskHandle`, its completion fence, and the worker cancellation
token all reference that same node; the old `TaskRecord`, `TaskRecordState`, and separate
per-handle client lease have been deleted. Scope census retains a private `JobHandle` projection of
the same node rather than a second lifecycle record.

Descriptor-led direct, scheduler-backed, detached, and prerequisite admission now allocate the
canonical node before admission and submit that existing node to the executor. Dependency wiring
continues to retain public prerequisite handles until launch or prelaunch terminal propagation, so
`CancelOnDrop` cannot cancel a prerequisite merely because its caller released the original handle.
The new focused regression asserts that the public handle, completion authority, and worker token
have identical node identity and one monotonic `Completed` state.

The plan's missing M3 dependency-scale anchors now exercise a 64-node prerequisite chain and a
64-task fan-in fence through canonical `TaskGraphScope::schedule_after`. The chain holds its head
until every successor is admitted, then requires exact execution order; the fan-in holds all
prerequisites until the final fence is registered, then requires the fence to observe all 64
terminal predecessors. The ownership child also rejects detached and stale post-shutdown handles
before admission. They are current-source regressions, not compiled acceptance evidence.

The canonical node has also been moved from the 553-line `job_handle.rs` blob into the
folder-backed `job_handle/task_node.rs` synchronization owner. After adding the canonical
graph-owned constructor and deleting the obsolete no-diagnostics graph constructor, the public
projection root is 462 lines and the node owner is 167 lines. This remains
a module-boundary split: there is still one
`Arc<TaskNode>`, the public type paths are unchanged, and no compatibility state or forwarding
overload was added.

This slice deliberately does not claim the universal public API hard cut. `JobScheduler` and
`JobHandle` remain public because their 38/15 production-like consumer routes have not yet been
migrated to owned TaskGraph scopes. Removing those exports before consumer migration would only
replace the dual model with a compile break, while preserving them as a permanent compatibility
route would violate the decision above.

## Implemented execution-domain slice (2026-09-01)

`EngineTaskGraph` now owns the existing budget allocator's three physical domains instead of
materializing only a compute pool. `EngineTaskGraphOptions::with_worker_threads(N)` remains one
total budget: combined domain minimums raise `N=1/2` to three workers, `N=3` resolves to
`Io=1, AsyncCompute=1, Compute=1`, and `N=8` resolves to `2/2/4`. Domain selection is the existing
`TaskPools::get(TaskPoolKind)` match, so admission remains O(1) and allocates no extra queue or task
node.

Direct scope submission acquires the descriptor's physical domain. Scheduler-backed admission now
rejects a scheduler whose physical owner differs from the descriptor domain instead of silently
running `Io` work on compute. Completion, cancellation, dependencies and scope accounting still
use the same canonical `TaskNode`. Scheduler-backed admission retains the external scheduler's
diagnostics but injects the graph's callback dispatcher, so an `Io` task body runs on `Io` while
its dependency and terminal-observer delivery runs on the graph's `AsyncCompute` domain. The
dispatcher falls back to synchronous accepted-callback delivery after domain admission closes,
preserving shutdown ownership. The crate-private unscoped detached helper still uses its explicit
scheduler owner because it has no TaskGraph; that route remains part of the later public bypass
hard cut rather than being mislabeled as graph-owned.

Shutdown closes all three domains before draining scopes, then releases and joins `Io`,
`AsyncCompute`, and `Compute` within the one deadline. The public report hard-cuts the old singular
`worker_shutdown` field to typed per-domain `worker_shutdowns`; each entry carries kind,
active-submission, expected/exited/joined and termination-signal counts. Retained weak pool handles
cannot reopen any joined domain.

The focused source regressions now cover the previously inconsistent bounded-stream assumption:
a three-worker total budget exposes exactly one physical IO worker, and a blocking IO capture
reaches terminal while the only compute worker remains deliberately blocked. This proves
progress-domain separation at the source-contract level. A scheduler-domain regression also
rejects an IO descriptor paired with the Compute scheduler, verifies the admitted IO task runs on
a `zircon-io-task-*` worker, and requires its terminal observer to run on a
`zircon-async-compute-task-*` worker; neither regression is product WPR or power evidence.

## Current-source performance gate

`zircon_runtime/tests/runtime_task_node_allocation_performance.rs` is an ignored, coordinator-only
Windows Release target with its own counting allocator. It profiles the real
`EngineTaskGraph -> TaskGraphScope::submit -> TaskHandle::wait -> shutdown` lifecycle at 1, 1,000,
and 100,000 tasks over five samples, and emits
`RUNTIME11_CANONICAL_TASK_NODE_PROFILE_V1` with elapsed P50/P95, allocation-count P50, requested-byte
P50, allocations per task, and bytes per task. This is the production-path counterpart to the
isolated ownership-shape model above; no current-source numbers are recorded until the managed
Release target actually runs.

`zircon_runtime/tests/runtime_task_domain_isolation_performance.rs` adds a second ignored,
coordinator-only Windows Release target over the current implementation. For total budgets 3 and
8 it saturates every physical IO worker for 21 samples and requires a compute marker to start
before the blockers are released. Marker `RUNTIME11_TASK_DOMAIN_ISOLATION_PROFILE_V1` reports the
resolved IO/compute worker counts and compute latency P50/P95. No current-source values are
recorded until that managed target runs; WPR context switches/wakeups, RSS and product power remain
separate required evidence.

The focused managed compile requests
`runtime11-canonical-task-node-check-20260901-v1` and
`runtime11-task-domains-check-20260901-v1` were both rejected before ticket creation with
`validation_ticket_external_worktree_dirty` because the external dependency worktree
`E:\Git\zr_vm` contains foreign uncommitted changes. Runtime11 did not clean or modify that
repository. Rustfmt, `git diff --check`, and locked Cargo metadata pass, but they are not substitutes
for compilation, behavior tests, Release profiling, WPR, or power evidence.

After converging scheduler-backed terminal delivery on the graph dispatcher, exact-manifest request
`runtime11-task-graph-callback-owner-test-20260901-v1` was rejected at the same immutable-source
preflight with `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`. Cargo did not start
and no target artifact was created. The IO-body/AsyncCompute-observer regression therefore remains
source-only pending a clean managed validation snapshot.

## Deferred High-Frequency Admission Profile

Source review found one measurable candidate after the direct-submit hard cut:
`TaskGraphScope::submit(...)` and `submit_after(...)` call the graph's
`scheduler_for(...)` for each admission. That facade reuses the graph's physical
pool and callback dispatcher, but it clones those references and creates a fresh
`Arc<JobSchedulerDiagnosticsState>` retained by the task handle. This is an
ownership-preserving observation, not evidence of a bottleneck. Before changing
it, the existing Windows Release task-node profile must compare 1/1,000/100,000
direct and dependency submissions with facade-construction allocation counts,
retained bytes, p50/p95 admission time, and task completion/retirement
conservation. A cached per-domain facade is only valid if the profile shows
material cost and the cache retains no independent worker, dispatcher,
diagnostic, or shutdown authority.

## Priority and affinity gate (2026-09-01)

The current `TaskDescriptor` has stable id, debug label, execution domain and cancellation policy;
it has no priority, affinity or task deadline. Local Unreal review confirms that
`UE::Tasks::FTaskHandle::Launch` stores `ETaskPriority` in the executable task before launch and the
low-level scheduler consumes it when ordering ready work. Zircon currently submits directly into
Rayon domain pools, which expose no per-task priority contract. Adding a serialized `priority`
field now would therefore advertise behavior the executor cannot provide.

The structural gate is explicit: do not add priority/affinity/deadline fields until one graph-owned
ready queue has fixed FIFO-within-priority ordering, bounded starvation/aging semantics, dependency
release ordering, cancellation-before-launch behavior, and a deterministic 1/2/N-worker work-count
matrix. The queue also needs current-source scheduling/steal/latency profiles before replacing
direct pool submission. Execution domain is already real and O(1); it remains the only scheduling
class in the descriptor until those gates are met.

## Quiescence and terminal-order correction (2026-09-01)

Source review found that a scoped worker previously retired its scope record before the outer
scheduler published the `JobHandle` terminal state. That allowed `wait_until_quiescent()` to return
while a public handle was still non-terminal, and released the `TaskId` capacity slot one ordering
step too early. The correction keeps one scope record, but adds an internal post-terminal hook to
the scheduler's existing completion path: `TaskNode` terminal publication now happens first;
scope counters, active `TaskId`, and quiescence notification retire immediately afterwards. The
dependency prelaunch path follows the same order by marking the handle before invoking the scope
retirement hook. No extra executor, queue, compatibility status, or per-task diagnostic lock was
introduced.

Two focused source regressions cover direct `TaskGraphScope::submit` and scheduler-backed
`TaskGraphScope::schedule`. Both close admission, wait for quiescence, then assert the public
handle is already `Completed` and the scope has zero queued/running work. Rustfmt and scoped diff
checks pass. Managed Cargo behavior remains unavailable because the immutable external dependency
worktree `E:\Git\zr_vm` is dirty; this correction is therefore source evidence only.

The retirement hook does not retain a second completion-handle clone. The scheduler closure and
scope activity table already keep the canonical node alive, so removing that redundant capture
avoids one Arc increment/decrement per scheduler-backed scoped task without changing ownership or
shutdown order. This is a static operation-count reduction, not an elapsed-time or power claim.

## Dependency admission short-circuit (2026-09-01)

Review of the prerequisite registration path found an avoidable retention case: when a dependency
was already cancelled or panicked before `schedule_after` entered the registration loop, the
pending work was terminalised but the loop still registered callbacks on later live dependencies.
Those callbacks could not launch work after the pending record was consumed, yet each retained the
dependent handle until an unrelated dependency reached terminal state. The loop now checks the
dependent handle before each registration and breaks immediately after a terminal dependency is
observed. This preserves the existing first-terminal-winner semantics while making preexisting and
racing terminal dependencies O(number of callbacks registered before the winner), rather than
always O(all dependencies).

The focused regression asserts that a pre-cancelled first dependency leaves a later pending
dependency with zero registered continuations after the dependent handle reaches `Cancelled`.
This is a deterministic ownership/work-count guard, not a throughput or power measurement; the
wide-fan-in source profile remains the required follow-up for current-source elapsed and allocation
evidence.

## TaskGraph prerequisite owner boundary (2026-09-01)

The canonical `TaskHandle` now retains a weak identity of the `EngineTaskGraph` that admitted it.
`TaskGraphScope::schedule_after` validates every prerequisite against that owner before acquiring a
worker submission or inserting a scope record. A handle from another graph, including a detached or
synthetic handle with no graph owner, is rejected with `DependencyOwnerMismatch`. Cross-scope
dependencies within one graph remain valid, so the check closes only the unsafe cross-owner route.

This prevents a continuation from being registered on a foreign callback dispatcher and then
outliving the scheduling graph's shutdown census. The focused regression admits a completed
prerequisite in a second graph and proves the primary scope rejects it without creating queued work;
both graphs then close and join through their own reports. The owner check is O(number of
prerequisites), adds no scheduler queue or compatibility fallback, and is source-level evidence
until the managed Cargo lane becomes available.

Current owner-boundary receipt: `task_handle.rs` is 174 lines
(`DFF3B2D7638E90DFBDD621CCF1D2AE0A23997811FF3318F140ED63F5C1FD0330`), `scope.rs` is 461 lines
(`F2B1966F1A3A31897771F04E22EBE065C50878B48E1F8BE2599D935C6A7E2E3B`), `admission.rs` remains 50 lines
(`31C9E2CA6C09BDF494BE01C073F2C73F19C272479C7DAFD5C129951FEE0A2AD2`), and the regression source
is now an 810-line route (`E2A102E02B7A2F8ED178184DF7D371E86E6E20C5629A833D022B05B2E01BC82`) with
ownership and dependency-scale children at 159 and 196 lines. The scope path now also includes
the direct descriptor-led `submit_after` ordering and failure-propagation regressions, while the
dependency-scale child owns canonical multi-handle synchronization.

Current-source receipt after this correction: `job_scheduler.rs` is 456 lines
(`1412610D7964B5F51A825514F66BB75A38CB521DDCF6FE65B80450EEB0B104B9`), terminal execution and
post-terminal hook code is owned by `job_scheduler/pending.rs` at 172 lines
(`7C39859503F4AB8EBA678B3F4CF7EDDFCDF52F82F7253189D4520A0F9F0F0D2A`); scheduler regressions
are in `job_scheduler/tests.rs` (`FFC86E3FA2121440619CE434A01F76C016C5BE548042D247AAF47604AE17B600`),
and the scope/test
receipts are `F2B1966F1A3A31897771F04E22EBE065C50878B48E1F8BE2599D935C6A7E2E3B`,
`E2A102E02B7A2F8ED178184DF7D371E86E6E20C5629A833D022B05B2E01BC82`,
`DAF95302AA5A65D4DDA783103E4C6A786A8F70A5D92A0D9E84EC331560FF390A`, and
`6E4F0C4F7A3DF54A39520F2E13AC4A5355FA8D395C0DD0A2835896A9B1F5506F`. The updated anchor audit is
`4EFDF91A7A9677FCB2A952C392EC5D873F908B865EF053BEB728BCF9922AD79D`; the companion boundary
script is `513AFA7CD7C6D412D74C3B34CE78C08C806C6FD5525925EECFEAEC0D35E7ADAF`. Its only remaining
risk is the pre-existing foreign `tasks/report.rs` size (617 lines).

The owner identity correction is source-only: `EngineTaskGraphInner` creates one `Arc<()>`, each
graph-owned `TaskNode` stores one `Weak<()>`, and public `TaskHandle` clones only the scope and
canonical completion node. Cross-graph admission compares the node's owner pointer in O(1) and
graph-less detached/synthetic nodes remain rejected. The 64-clone regression demonstrates that
handle projection no longer increments the graph-owner weak count. This is a reference-count
operation reduction, not a claim about elapsed time, RSS, or power.

The bounded-dispatch Failure structure is now current-source complete for this slice: the shared
task test root is 371 lines with diagnostics and terminal-observer suites under explicit child
owners, and the private JobHandle tests use the canonical TaskNode mutex plus a `cfg(test)` poison
hook. This removes the stale `JobState`/`handle.state` test contract without introducing a
compatibility state field. The source and audit receipts are recorded above; managed Cargo remains
the required gate for compilation and runtime behavior.

The local incremental `cargo check -p zircon_runtime --lib --locked --offline --jobs 1`
reached `zircon_runtime_interface` and then stopped in the pre-existing foreign WGPU crate at
`zr_rhi_wgpu/src/render_pass_validation.rs:455` (`texture_view_layer_count` is unresolved). No
Runtime11 source diagnostic was emitted before that error, and the command left no Cargo/rustc
process running. A separate `core-min` offline check remained without a compiler result for 180
seconds and timed out, also leaving no Cargo/rustc process. These are compilation observations
only; neither is a passing Runtime11 gate.

The managed Windows `core-min` lib-test request for the exact current snapshot used target pool
`E:\\cargo-targets\\zircon-engine\\pool\\512c70185e168c08f5932544e6456e4327df2ef2717a85aa184d3337f9b3c6eb` and
the filter `core::runtime::tasks`. The validator wrapper timed out after 604 seconds while the
corresponding Cargo/rustc process was still compiling and produced no terminal test result. That
run was stopped only by exact PID/command-line identity; no other Cargo process was touched. This
is an environment-timeout receipt, not a passing or failing Runtime11 source diagnostic.

## Cross-plan compile-blocker source receipts

Two delegated compile blockers were reviewed without taking ownership of foreign dirty paths. The
current `zircon_runtime_host/src/foreign_output/item_count.rs` (`7E643D75BC34A77A5A596A59351184AD679A10197B613A7C7255B7D00F66E322`)
has an explicit `WorldQueryResult::TransformSnapshot { .. } => 1` arm. Its existing
`world_query_item_count_covers_every_result_variant` regression enumerates `ComponentRows`,
`HierarchyRows`, `InspectionFields`, `TransformSnapshot`, `EntityMissing`, and `NotModified`.
The current `zircon_runtime_interface/src/reflect/schema_catalog/admission.rs`
(`18D866B7ECBAD235A8C83D34FEA59D6A28CCC10F3275E0F5E462C90E0ABB2BA7`) validates aliases in a
second immutable loop, drops `field_names`, and only then mutably sorts aliases, removing the
reported E0502 borrow overlap. Both are current-source receipts only; the managed focused Cargo
validation and required upward notification remain pending while the coordinator/Cargo lane is
unavailable.

## Repository gate status

The read-only `check_conventions.py` gates were run after the source slice. The guard gate passed
with `violation_count=0` across 63 rules. The exemption gate reported seven pre-existing missing
markers, all under `zircon_runtime_interface`; the documentation gate reported 1,604 pre-existing
missing-path/repository-escape references across the workspace. Neither gate points at a Runtime11
owned path, and no foreign file was edited to make them appear green. The layering gate did not
return within its 90-second budget and the later structure gate did not return within 120 seconds,
so both are recorded as unverified rather than passed.

The focused JobSystem structure audit was updated from the obsolete single `worker_pool` and
compute-pool stream-capacity anchors to the current `TaskPools` owner and explicit `Io` domain. It
now finds all current Runtime11 API and behavior anchors, including the deep-chain and wide-fan-in
regressions. Its four Python tests pass 4/4 while requiring the one remaining risk to stay exact:
foreign-owned `core/runtime/tasks/report.rs` is 617 lines. The Runtime11-owned `job_handle.rs`
oversize finding is closed; `report.rs` was not absorbed or edited by this lifecycle. The current
audit script hash is `513AFA7CD7C6D412D74C3B34CE78C08C806C6FD5525925EECFEAEC0D35E7ADAF` and the
behavior anchor count is now 76/76 after the test split, direct submit-after route, direct
dependency-failure regression, and canonical multi-handle wait regression. The Rust mirror now
reads the same folder-backed child sources rather than silently scanning only parent modules.

## Acceptance state

- [x] Framework DTO migration rechecked; only the neutral parallel trait remains.
- [x] Runtime descriptor/handle/status and all public scheduler/fence leakage reviewed.
- [x] Unreal task ownership and prerequisite lifetime model compared from local source.
- [x] E-drive task-node allocation model recorded.
- [x] E-drive blocking-I/O saturation model recorded after worker-ready correction.
- [x] Exact Runtime11 source ownership established without absorbing active diagnostics changes.
- [x] `TaskRecord`, completion state, cancellation token, prerequisite state, and handle lease folded
      into one canonical task node in current source.
- [x] Real 1/1k/100k TaskGraph allocation/elapsed Release profile target added.
- [x] Descriptor domains hard-routed to one Runtime-owned three-domain worker budget.
- [x] Scheduler-backed scoped nodes retain scheduler diagnostics but use the graph's single
      `AsyncCompute` callback owner; IO body and terminal-delivery thread domains are asserted.
- [x] Per-domain worker inventory and exact shutdown census implemented.
- [x] Real saturated-IO/compute-progress Release profile target added.
- [x] Deep-chain and wide-fan-in dependency-scale source regressions added.
- [x] Canonical task-node synchronization moved to a folder-backed owner; `job_handle.rs` reduced
      from 553 to 462 lines without changing the public path or adding compatibility state.
- [x] JobSystem source anchors converged to the physical `Io` lane and `TaskPools` domain owner;
      the only remaining oversize audit risk is foreign-owned `tasks/report.rs` at 617 lines.
- [x] Scope quiescence now follows canonical `TaskNode` terminal publication for direct,
      scheduler-backed, and dependency-prelaunch paths; focused ordering regressions added.
- [x] `schedule_after` short-circuits dependency callback registration after preexisting or racing
      terminal failure/cancellation; focused continuation-retention regression added.
- [x] `schedule_after` rejects prerequisites from a different or graph-less owner before admission;
      cross-scope same-graph dependencies remain supported and the owner-boundary regression is added.
- [x] Graph owner identity is stored once on the canonical task node; cloning a public handle does
      not duplicate the owner weak reference, and graph-owned construction has an explicit API.
- [x] `engine_task_graph.rs` inline shutdown/inventory tests moved to the folder-backed
      `engine_task_graph/tests.rs` owner; production root is 277 lines and all five anchors remain.
- [x] Shared JobSystem tests moved to explicit `tasks/diagnostics.rs` and
      `tasks/terminal_observers.rs` owners; root is 371 lines and all 76 behavior anchors remain.
- [x] JobHandle lock-recovery tests hard-cut from removed `JobState`/`handle.state` storage to the
      canonical TaskNode mutex through a test-only poison hook; no compatibility state field added.
- [x] Graph-owned `submit_after` has a dependency-failure regression proving the dependent reaches
      canonical `Failed` without launching user work, matching the scheduler-backed path.
- [x] Direct graph `submit` now delegates to the same scheduler-backed admission and terminal
      pipeline as `schedule`/`submit_after`; the obsolete graph-only completion constructor was removed.
- [ ] One task node and one public handle implemented with no compatibility route.
- [ ] Production blocking consumers moved to a TaskGraph-owned bounded lane.
- [ ] Public scheduler/fence/pool-spawn bypasses removed from production callers.
- [ ] Managed current-source behavior, WPR, allocation, RSS and power evidence accepted.

Current dirty `diagnostic_observation/journal.rs`, `diagnostic_observation/tests.rs`, and
`tasks/report.rs` remain foreign and untouched. The canonical node and execution-domain source
slices are `implementation_complete_validation_blocked`; they are not an accepted Runtime11
milestone and are not eligible for coordinator commit or WeCom completion publication yet.
