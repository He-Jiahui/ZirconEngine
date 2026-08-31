# Runtime11 Execution Worker Join Owner Architecture And Profiling Plan

- Date: 2026-08-27
- Owner plan: `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- Status: `superseded_by_engine_task_graph_single_worker_set`
- Superseded by: `2026-08-27-engine-task-graph-shared-worker-owner.md`

> 该记录保留三域 join 机制的调研历史；其“三个物理 pool domain”终局设计已被
> 单一 `EngineTaskGraph` worker set 硬切取代，不能再作为当前源码结构说明。

## Scope

This slice closes the lifecycle gap between `ExecutionScope` task-body
quiescence and actual Runtime-owned worker termination. It does not migrate the
process-default pool, timer, or private subsystem workers, and it makes no
startup, frame-time, shutdown-latency, RSS, or power improvement claim before a
source-bound product measurement.

The first production consumer cutover in this slice removes
`ProjectAssetManager::default()` from the `AssetModule` factory and injects the
activating `CoreRuntime` `Io` domain. This does not delete the process-default
owner; it proves the intended module-to-runtime injection route before the
remaining consumers migrate.

The second production cutover applies the same rule to `PlatformModule`:
`PlatformDriver` now receives the activating Runtime's `Io` pool and uses it
for preference persistence. `PlatformDriver::default()` remains an explicit
standalone/process-owner constructor rather than the module factory path.

The third production cutover removes `JobScheduler::process_io()` from editor
settings persistence. `EditorManager` now derives an I/O scheduler from the
active `CoreHandle` pool and `EditorContextBuilder` requires both its compute
and settings-I/O schedulers explicitly.

## Production Consumer Cutover Status

| Consumer | Current source result | Remaining work |
|---|---|---|
| Asset module | Complete at source level: the module injects `CoreRuntime::Io` into `ProjectAssetManager`; activation test compares physical execution-owner identity. | Managed Cargo and product profiling. The expiry deadline still uses the process timer. |
| Platform preference persistence | Complete at source level: the module declares the Tasks dependency and injects `CoreRuntime::Io` through `PlatformDriver`; activation test compares the bounded lane's execution owner. | Managed Cargo. Deadline-bearing lane entries still use the process timer pending the deadline-owner decision. |
| Editor settings persistence | Complete at source level: `EditorManager` derives `JobScheduler` from the active Core `Io` pool; Builder and persistence constructors require the route explicitly, and the production service contains no process-default lookup. | Managed Cargo and product shutdown profiling. |
| Scene level manager | Already compliant: the production scene module uses `DefaultLevelManager::with_core`; process-default construction is confined to standalone/test paths. | No worker-pool cutover required. |
| Native-plugin discovery | Process-lifetime authority retained intentionally: discovery precedes Core composition and matches Unreal's application-global plugin manager. A nonblocking prepared-root ticket/last-good snapshot source contract is implemented. | Migrate Editor12 request paths and run managed validation. Profile the process-default three-pool materialization before choosing an application execution owner or dedicated I/O owner; never bind the authority to one Runtime generation. |
| Renderer/UI text | Research complete: product SDF and shape prewarm use process Compute, while raster copies the AsyncCompute count into an additional private thread set. | Run the Windows product matrix in `2026-08-27-renderer-text-execution-owner-research.md`; then inject render execution resources and converge private workers only if the profile supports it. |
| Font SDF build tool | Intentionally standalone: the CLI uses the process compute owner. | Keep separate from Runtime-lifetime acceptance. |

## Current-Source Finding

`ExecutionRuntime` owns three explicit Rayon pools (`Compute`,
`AsyncCompute`, and `Io`) and already closes scope admission before waiting for
queued/running task bodies. The lower pool model is still structurally unable
to prove unload safety:

- every `TaskPool` clone retains `Arc<rayon::ThreadPool>`, so a scheduler or
  subsystem handle can silently extend the physical worker lifetime;
- `ExecutionRuntime::shutdown` marks the runtime stopped after scope
  quiescence without releasing or joining the pools;
- Rayon 1.13 `ThreadPool::drop` calls `Registry::terminate`, which requests
  eventual worker termination but does not wait; its worker-stop waiter is
  available only to Rayon tests;
- unscoped work admitted through an already cloned pool is absent from
  `ExecutionScopeCensus`, so scope counters alone cannot be a DLL-unload
  receipt.

The defect is ownership and lifecycle complexity rather than a lock or polling
micro-hotspot. With `H` retained pool handles and `W` physical workers, current
worker lifetime is unbounded by the runtime owner (`O(H)` independent owners),
and shutdown publishes no `O(W)` join evidence.

## Reference Evidence

### Unreal Engine

`dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/Fundamental/Scheduler.cpp`
implements `FScheduler::StopWorkers` as an owner-only transition: stop both
waiting queues, exchange every worker slot to null, join and delete every
thread, finish queue shutdown, then reset TLS, local queues, events, and the
global registry. `TaskGraph.cpp` calls this after its shutdown callbacks and
named-thread quit requests. Zircon must preserve the same ordering even though
its execution backend remains Rayon.

### Rayon

The locked workspace uses Rayon 1.12 / rayon-core 1.13. The local dependency
source confirms that custom `ThreadPoolBuilder::spawn_handler` owns each
`std::thread::JoinHandle`, while the default handler intentionally discards it.
This is the narrow integration point needed for a real join receipt without
forking or replacing Rayon's work-stealing scheduler.

### Bevy

`dev/bevy/crates/bevy_tasks/src/task_pool.rs` keeps task-pool creation and
executor driving in one owner. It is useful for thread-budget and executor
separation, but its process-global pools are not an unload receipt and are not
copied as Zircon's runtime-instance lifecycle model.

## Chosen Architecture

1. The `TaskPool` retained by the explicit runtime is the sole strong Rayon
   owner. Every cloned `TaskPool` or `JobScheduler` route is a weak execution
   handle, so consumer lifetime cannot become worker lifetime.
2. Runtime lifecycle is monotonic: `Running -> Closing -> Stopped`. Closing
   release-stores one shared admission bit, so old handles cannot reopen work.
3. A pool call performs two acquire admission checks around one `Weak` upgrade.
   The second check is the admission linearization point. An already admitted
   call temporarily retains the backend until that call returns; a rejected
   call neither locks nor touches the Rayon scheduler.
4. Rayon custom-spawn workers retain real `JoinHandle`s. Dropping the sole pool
   owner requests termination after outstanding spawned work completes;
   shutdown waits for worker-exit observations and then joins every handle.
5. The runtime order is fixed: close scopes, reach task-body quiescence, close
   all three pool domains, signal termination, join workers, then publish
   `Stopped`.
6. A timeout leaves the runtime and pools in `Closing`. A later shutdown call
   continues the same transition; it never recreates workers or publishes a
   false stopped state.
   Shutdown invoked from an owned worker returns the same incomplete receipt
   immediately for that domain instead of waiting for the caller to join
   itself; an external owner must retry after the task returns.
7. `ExecutionShutdownReport` carries per-domain expected/exited/joined counts
   and whether termination was signalled. Only exact joined equality for every
   domain is an unload-capable worker receipt.

## Complexity And Bounds

For `W` workers, `D = 3` runtime domains, `S` live scopes, and `T` admitted
tasks:

- admission: `O(1)` per call, with two atomic loads and one weak upgrade, no
  per-call mutex or allocation;
- scope shutdown: existing `O(S + T)` accounting and cooperative drain;
- pool close: `O(D)` state transitions;
- worker termination and join: `O(W)` time/receipt storage;
- added persistent memory: `O(W)` join handles plus `O(D)` lifecycle state;
- no per-task allocation, queue node, polling loop, or diagnostic lock is added
  to the scheduler hot path.

## Test-First Contract

- retaining a cloned `TaskPool` handle does not retain the physical worker
  owner after runtime shutdown;
- a blocked unscoped task prevents a joined receipt, produces a typed timeout,
  and a later retry joins all workers after the task is released;
- shutdown reports all three domains and conserves expected/exited/joined
  counts;
- post-close admission through stale handles is rejected deterministically;
- repeated successful shutdown is idempotent and does not join twice.
- worker-side shutdown cannot self-join or consume the full requested deadline;
  an external retry completes the same owner transition.

## Product Profiling Gate

After managed source-bound compilation, compare the same build before and after
the hard cut with 1/3/8/32 workers and 0/1/1,000/100,000 no-op tasks plus one
blocked unscoped task. Record:

- shutdown p50/p95/max latency and timeout accuracy;
- expected/exited/joined workers by domain;
- process thread count before start and after the joined receipt;
- CPU time, context switches, RSS, and package/process power during idle and
  shutdown;
- scheduler submission/queue/execution counters to prove no steady-state
  per-task regression.

Acceptance requires zero Runtime-owned workers after a successful receipt,
zero post-close admissions, no detached worker after timeout/retry, and no
statistically material steady-state submission regression. Until that matrix
runs, this slice claims lifecycle correctness by source contract only and does
not claim that a performance or power bottleneck has disappeared.

## Static Evidence (2026-08-27)

- focused `rustfmt --check` and `git diff --check` passed for the Platform and
  editor settings injection slices;
- runtime domain dependency audit passed 11/11;
- five editor settings source-contract suites passed 24/24;
- preference persistence lane audit passed 6/7; its only failure is the
  pre-existing WOC client manifest expectation, not a Platform owner mismatch;
- JobSystem audit passed 2/3; its only failure remains the pre-existing mesh
  builder direct-Rayon path;
- managed Cargo and product profiling were deliberately not claimed.
