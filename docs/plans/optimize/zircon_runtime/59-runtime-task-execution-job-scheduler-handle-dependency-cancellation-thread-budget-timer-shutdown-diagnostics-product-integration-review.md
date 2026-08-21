---
title: Runtime Task Execution、Job Scheduler、Handle、Dependency、Cancellation、Thread Budget、Timer、Shutdown、Diagnostics 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime59
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/framework/tasks
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/runtime/modules/tasks.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/operation/service.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
  - zircon_runtime/src/graphics/pipeline/async_compile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/pipelined/queue.rs
  - zircon_runtime/src/platform/preferences/persistence/adapter.rs
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/module/scene_artifact_io.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery/io.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/text/sdf/generation_scheduler.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/context
  - zircon_editor/src/core/settings/persistence.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
tests:
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/tests.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/tests
  - zircon_runtime/src/tests/runtime_absorption/job_system
  - zircon_editor/src/core/jobs/tests
  - zircon_editor/src/core/jobs/system/pending/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/35-ownership-graph-shared-weak-borrow-lease-callback-subscription-raii-cycle-detach-leak-isolation-review.md
  - docs/plans/optimize/zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Task.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/TaskPrivate.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/TaskConcurrencyLimiter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Pipe.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/ManualPipe.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Tasks/TaskPrivate.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Tasks/TaskConcurrencyLimiter.cpp
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - dev/bevy/crates/bevy_tasks/src/slice.rs
  - dev/bevy/crates/bevy_tasks/src/thread_executor.rs
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/task.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Utilities/JaggedJobRange.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 59 · Runtime Task Execution、Job Scheduler、Handle、Dependency、Cancellation、Thread Budget、Timer、Shutdown、Diagnostics 与 Product Integration 工程化差距

## 1. 结论

Zircon 的任务层不是空壳。当前代码已经具备三类 Rayon pool、依赖 continuation、worker 内协助等待、panic 捕获、64-shard scheduler diagnostics，以及 entry/byte budget、key coalescing、fence、deadline-before-start、cancel authority、terminal ticket 和 shutdown guard 较完整的 bounded keyed I/O lane。EditorJobSystem 又在其上补了类别配额、pending admission、priority、mutex group、cooperative cancel、progress 和 shutdown。上述基础应保留，尤其不能在重构时退化 bounded admission、依赖不占 worker 等待和诊断写侧分片。

但它们尚未组成一个工程级 execution runtime。`TasksModule` 只是可被激活和卸载的描述符，既不创建 execution owner，也不关闭 admission、取消 scope、排空 task、停止 timer 或 join worker；`CoreRuntime` 则在模块系统之外直接取得进程级 `OnceLock<TaskPools>`。因此 AssetModule 对 TasksModule 的硬依赖只能证明一个名字已激活，不能证明执行服务已就绪或卸载已静默。`AsyncTaskDescriptor`、`TaskCancellationPolicy` 和 `TaskPollBudget` 又与真正的 `JobScheduler/JobHandle` 完全断开，scene 等子系统只好手工重建状态机。

线程预算也不是产品事实。线程分配器在剩余为零时仍把每类 pool clamp 到至少一个 worker，`total_threads=2`可以实际创建三个 worker；`JobScheduler::default()`还能另建一个完整 compute pool。Text 把 async-compute 的预算数字当作自身专用 OS worker 数再次创建线程，Graphics、diagnostic log、config、watcher 和 render submission 也各自管理线程。`TaskPoolReport`只报告三类 pool 的配置，无法回答进程中实际有多少线程、归谁、排了多少工作、能否退出。

本轮不新增跨报告 P0 计数。Runtime02 已拥有 task/timer worker 越过 dynamic session 与 DLL unload 的总 P0；Editor09 已拥有 shutdown deadline 后仍继续拆 project/settings 的 Editor P0。本篇补齐这两个阻断在 task subsystem 内的直接原因和验收合同，但不重复累计。本轮登记 **72项P1、18项P2和40项验收门禁**。目标不是继续为每个子系统加一个私有 pool，而是建立 `ExecutionRuntime + WorkerDomain + TaskScope + TaskDescriptor + Task<T> + DependencyGraph + DeadlineService + BoundedOperationLane + DedicatedWorkerLease + ExecutionDiagnostics`，让每项异步工作都能回答 identity、owner、admission、result、cancel、deadline、shutdown 和 observation。

本轮只做静态 review 和文档总账，没有修改 production、tests、Cargo、ABI 或参考源码；没有运行 Cargo、动态库 unload、真实 Editor shutdown、stress、soak、sanitizer、profiler 或 benchmark。静态结构不能证明性能已达到或超过 Unreal。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint / 说明 |
|---|---:|---|
| framework DTO、runtime scheduler/pool/timer/bounded lane 与 Core 接线 | 36 / 6,040 / 196,351 / 43 | SHA-256 `610eb404f451426d2b0c04dcee452cae3d3a225ce01f2109ece1e24b3b806bd2` |
| Runtime、Editor、App production consumers 与私有 worker | 82 / 32,222 / 1,164,302 / 161 | SHA-256 `34c749adb75de82b37560a174dcd2b0e1690a6b8d0c578ad1224a434b9654a8d` |
| focused behavior、pressure、source-shape 与 Editor job tests | 27 / 7,057 / 238,450 / 142 | SHA-256 `8654ba155073a0b1593447908aedad18699a9c8dcaafe01dad856ab78b704142` |
| Unreal、Bevy、Godot、Fyrox 与 Unity Graphics references | 29 / 9,738 / 357,657 / 18 | SHA-256 `67c0d07427e0e44fca9d8a563e23746292d9132ea78828fa78785ce30ca3ff6d` |

fingerprint 算法延续 Runtime58：相对路径转`/`并排序去重，以`path|lowercase per-file SHA-256`组成LF连接且无末尾LF的UTF-8字节，再计算SHA-256。它冻结本轮实际读取集合，不是未来 task identity 或 artifact identity。

基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。工作树存在大量其他会话/用户改动；本轮生产文件保持只读，报告按当前working tree读取。共享实现和索引仍会继续变化，因此`source_recheck_required`保持true。

### 2.2 所有权与去重

- Runtime02继续拥有 event/task 总体分层、进程 task/timer worker 与 dynamic-library unload 的父P0；本篇拥有当前 task execution 纵向的具体合同与迁移序列。
- Runtime11 implementation plan继续承接已在实施的 bounded I/O、diagnostics、asset、scene、plugin discovery 与 preference failure handoff；本篇不把已有局部实现误写成空白。
- Runtime01/46拥有 module/service 总生命周期；本篇只要求 TasksModule 成为真实 execution service owner，而非 descriptor-only dependency。
- Runtime24拥有全仓 handle/generation 规范；本篇负责 `TaskId/TaskScopeId/WorkerDomainId/TimerRegistrationId` 的落地。
- Editor09拥有 EditorJobSystem 的 keyed merge、progress、event queue、product shutdown 和业务 adapter；本篇只审它依赖的 runtime task substrate。
- Tooling35/24拥有全仓 ownership 和 concurrency 横切治理；本篇拥有 runtime execution product implementation。用户已要求暂停 tooling 优化，因此本轮不扩写 tooling 报告。
- Unity Graphics 只用于 data-parallel dependency 与资源访问声明对照，不把 package 内 job 使用误当作完整 Unity engine scheduler 源码。

## 3. 当前真实产品链

### 3.1 Core、module 与 pool

```text
CoreRuntime::new
  -> TaskPools::default
  -> TaskPools::process_default
  -> PROCESS_TASK_POOLS: OnceLock<TaskPools>
       -> IO Rayon pool
       -> AsyncCompute Rayon pool
       -> Compute Rayon pool
  -> JobScheduler::from_pool(compute.clone())

builtin profile
  -> register/activate TasksModule
  -> TasksModule descriptor has no lifecycle/service/factory
  -> AssetModule dependency on "TasksModule" is considered satisfied
```

Core 构造在 module activation 前已经创建/取得执行资源；TasksModule activation 不改变 scheduler 状态，deactivation 也不改变 admission。`CoreHandle::scheduler/task_pools/task_pool`返回可 clone 的 pool/scheduler，clone 没有 module generation 或 scope lease，可越过 runtime/module lifetime。

### 3.2 Scheduler、handle 与 dependency

```text
schedule(task)
  -> untyped JobHandle
  -> TaskPool::spawn
  -> catch_unwind(task)
  -> mark_complete / mark_panicked(String)
  -> synchronously publish every continuation
  -> synchronously run terminal observers

schedule_after(handles, task)
  -> register private continuation on every handle
  -> last dependency launches task
  -> first dependency panic marks child as panicked
```

`JobHandle`没有 task id、owner、scheduler identity、typed result、cancel token、deadline、priority 或 shutdown policy。跨 scheduler handle 可任意组合，依赖图不检查 cycle。`wait()`只有无限等待并在失败时 panic；`spawn()`返回`()`且不 catch panic，测试明确认证真实 Rayon detached panic 保留进程终止行为。

### 3.3 Framework task contract 是平行状态机

`AsyncTaskDescriptor { handle, pool, label, cancellation_policy }`、`AsyncTaskStatus`、`TaskPollBudget`看似定义了任务协议，但 scheduler 不接收它们。production 中只有 dynamic scene spawn/reload 局部使用 descriptor/status，自己维护 atomic cancel、mutex status/result 和独立全局 ID。`CancelOnDrop/DetachOnDrop/FinishOnShutdown`没有 generic runtime executor 实施者，`TaskPollBudget`没有 production consumer。

### 3.4 Bounded keyed I/O 是成熟孤岛

```text
try_admit(key, generation, retained_bytes, deadline, work)
  -> reserve entry + bytes
  -> optional coalesce / fence prerequisite pins
  -> activate ticket
  -> one scheduled pump drains the whole lane serially
  -> terminal ticket + one observer + local diagnostics
  -> shutdown guard closes admission and waits for every reservation/handle
```

该 lane 已覆盖大量 1,000/100,000 admission、fence、deadline、cancel、panic 和 observer race 测试，方向正确。但它仍不是 `TaskScope`：work只能返回`Result<(), static code>`，deadline只在开始前生效，一个 lane 同时只有一个 active work；pump 外层 panic没有 reconciliation guard，shutdown guard 的 Drop 又无限阻塞。

### 3.5 生产消费者没有统一 owner

- Asset worker有请求/observer/retained-result容量和诊断，但最终调用`TaskPool::spawn`；Drop取消 ticket publication，不等待实际 decode worker退出。
- Text SDF复用process compute pool，bitmap raster却按 async-compute thread count再次创建同数量专用OS线程；二者都不进入Core task census。
- Graphics async pipeline compile和pipelined render submission各有私有线程及局部join；它们不是 TasksModule scope，也不进入统一预算。
- Scene artifact、runtime archive、preferences、Editor settings和VM discovery分别创建 bounded lane，有的显式finish，有的在Drop里直接无限等待guard。
- Runtime operation prepare使用公开的detached `core.scheduler().spawn`，当前闭包自行 catch handler panic，但 substrate 本身仍是 fatal-on-unwind API。
- EditorJobSystem正式复用`core.scheduler()`并在UI退出时经autosave service调用shutdown，但底层 scheduler没有停止 admission/cancel/drain；deadline返回unfinished后产品仍继续close project，父P0归Editor09。
- Dynamic session shutdown以`Duration::ZERO`关闭modules，只证明module lifecycle进入Unloaded，不检查scheduler、pool、timer、private workers或callbacks。

## 4. 可保留基础

1. `TaskPools`对Compute/AsyncCompute/IO的显式域划分，以及 pool clone 共享同一 Rayon owner 的机制。
2. `schedule_after`通过 terminal continuation 释放依赖，不用 worker 阻塞等待依赖。
3. worker 内`wait`会尝试执行当前 Rayon pool 工作，已有单worker防死锁测试。
4. tracked task捕获panic并保证handle到达terminal；combine、deep chain和wide fanout已有行为测试。
5. 64个cache-aligned diagnostics shard把高频写分散到线程本地选择的shard，并对聚合快照做epoch复核。
6. bounded keyed I/O 的两阶段admission、entry/byte conservation、generation coalescing、global fence epoch和cancel authority。
7. bounded lane 对deadline-before-start、panic terminal、shutdown report和高压admission的测试矩阵。
8. Asset、Text SDF和Editor jobs已有局部entry/byte/category预算，可迁移为统一lane policy，而不是删除重写。
9. Text raster、Graphics compile/render submission和diagnostic log展示了专用线程确有合理场景；目标是注册和监督，不是强迫所有工作进入同一CPU pool。
10. Editor正式路径已经在UI结束后先请求job shutdown、再关闭project/settings，这是可扩展为统一host shutdown choreography的基础。

## 5. P0 阻断与归并

### 5.1 继承 P0-A：task/timer worker 越过 dynamic session 与 DLL unload

Runtime02 P0-1继续拥有该阻断，本篇不重复计数。新增直接证据是：TasksModule没有lifecycle；CoreRuntime scheduler在module之外构造；process pools/timer由`OnceLock`永久强持；dynamic shutdown使用零drain timeout且只检查modules；TaskPool没有close/join；多个私有worker也不在统一census。Runtime59的M0-M2和G01-G08是该父P0的task-side关闭条件。

### 5.2 继承 P0-B：Editor deadline 到达后仍拆卸 live job 依赖

Editor09 E-JOB-P0-02继续拥有该阻断，本篇不重复计数。正式`run_editor`经autosave调用共享`EditorJobSystem::shutdown`，但只收到unfinished列表；随后仍执行`close_project`与settings shutdown，最后才把unfinished转成error。Runtime task handle没有owner/cancel acknowledgement/quiescence，导致Editor layer无法建立更强barrier。Runtime59负责底层scope/receipt，Editor09负责产品关闭决策。

## 6. P1 工程化差距

### 6.1 Execution owner、module 与 shutdown（P1-01 至 P1-10）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RTASK-P1-01 | `CoreRuntime::new`固定取得process pool，不能注入host-owned execution service，也不能表达构造失败。 | `CoreRuntime::try_new(ExecutionRuntimeHandle)`或明确runtime-owned构造，错误包含requested/created worker及rollback结果。 |
| RTASK-P1-02 | TasksModule只有descriptor，activate/deactivate不改变execution状态，却作为AssetModule硬依赖。 | TasksModule发布真实service generation；activate建立ready receipt，deactivate关闭admission并返回quiescence receipt。 |
| RTASK-P1-03 | `TaskPools::process_default`和`JobScheduler::process_io`允许任意子系统绕过session/module owner。 | process域只能由host owner创建；consumer领取带scope和policy的lane capability。 |
| RTASK-P1-04 | `JobScheduler::default`按全部CPU另建compute pool，调用点可无意成倍扩张线程。 | 删除production default；测试fixture显式建isolated executor，production只能解析owner capability。 |
| RTASK-P1-05 | 公共`TaskPool::new`让Asset/Graphics/Text/测试风格扩散到产品代码。 | `ExecutionRuntime`统一分配worker domain；专用pool走受审计的`DedicatedWorkerLease`。 |
| RTASK-P1-06 | scheduler/pool clone没有runtime/module generation，持有者可越过owner卸载继续提交。 | handle携`ExecutionRuntimeId + ScopeId + Generation`，stale/closed提交结构化拒绝。 |
| RTASK-P1-07 | 没有session/module/plugin/world/subsystem `TaskScope`，shutdown无法枚举谁仍有任务。 | 每项工作必须属于scope，scope维护admission gate、task census、cancel policy和retirement blocker。 |
| RTASK-P1-08 | `TaskPool`没有close、drain、cancel、stop或显式join；Arc最后释放不等于产品shutdown receipt。 | 分阶段`close_admission -> cancel -> drain -> stop -> join`，每阶段有deadline和blocker report。 |
| RTASK-P1-09 | Graphics、Text、config、watcher、log和render submission私有线程不进入统一supervisor。 | 所有专用线程登记owner、entry point、stack/QoS、stop signal、join handle和DLL code lease。 |
| RTASK-P1-10 | worker没有统一thread start/stop hook，无法装配profiling、allocator、script TLS、affinity或plugin callback禁入。 | WorkerDomain descriptor定义startup/teardown hook，失败回滚并进入diagnostics。 |

### 6.2 Admission、线程预算与 executor policy（P1-11 至 P1-22）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RTASK-P1-11 | 每个pool的min=1会在remaining=0时继续分配，report的`total_threads`小于实际worker和。 | budget solver先验证可满足性；实际worker conservation必须等于reported total或明确reserved dedicated count。 |
| RTASK-P1-12 | Text把async-compute预算数再次创建bitmap raster线程，`TaskPoolReport`看不到这批线程。 | budget区分shared slots与dedicated reservations；同一数字不可被两个owner重复消费。 |
| RTASK-P1-13 | Graphics public constructor可另建完整compute pool，renderer数量会线性放大worker。 | renderer领取共享render lane；真正device thread由graphics owner单独登记且有上限。 |
| RTASK-P1-14 | Asset/scene/preference/VM的Default或`new`静默抓process IO pool，测试与产品owner语义混杂。 | 构造器必须接收scope/lane；只允许显式fixture helper创建isolated owner。 |
| RTASK-P1-15 | scheduler没有priority，Editor `JobPriority`只影响pending选择，进入runtime后全部同级。 | stable priority class一路传递到queue/worker，支持ageing和priority inversion诊断。 |
| RTASK-P1-16 | descriptor不含affinity、QoS、stack、background/latency class或oversubscription policy。 | WorkerDomain提供platform-validated配置和降级报告，实时/渲染/IO域不得共享隐式默认。 |
| RTASK-P1-17 | `schedule/spawn`没有entry/byte/cost admission，提交永不返回Full/Closed/QuotaExceeded。 | 所有异步提交先quote并原子reserve，返回typed admission outcome和rollback lease。 |
| RTASK-P1-18 | 没有per-scope/per-plugin/per-world quota；一个owner可占满全局queue或timer。 | hierarchical budget：process -> runtime -> module/plugin -> scope -> lane，支持borrow但保留硬上限。 |
| RTASK-P1-19 | 无CPU topology、NUMA、cache locality、worker pinning、work stealing或blocking compensation政策。 | 先建立可测worker topology和task class，再按平台选择steal/locality；不能用线程数替代调度设计。 |
| RTASK-P1-20 | `TaskPool::spawn/install/join/in_place_scope`绕过JobScheduler handle、diagnostics和scope census。 | raw pool API收窄到executor内部；公共入口均生成task record和terminal receipt。 |
| RTASK-P1-21 | `parallel_for`只有手工chunk size和blocking slice closure，没有阈值、cancel、priority、result或嵌套预算。 | data-parallel plan根据work estimate/worker load切分，支持scope cancel、typed reduction和nested parallelism policy。 |
| RTASK-P1-22 | generic executor只接受`FnOnce`，没有future/waker、main-thread executor、thread-bound continuation或blocking task隔离。 | 明确CPU task、blocking IO、async future、main-thread continuation四类executor合同及adapter。 |

### 6.3 Task identity、result、cancel、dependency 与 completion（P1-23 至 P1-44）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RTASK-P1-23 | framework `AsyncTaskDescriptor`没有进入JobScheduler，描述和执行是两套状态机。 | canonical `TaskDescriptor`由scheduler admission消费，并成为status/trace/shutdown唯一来源。 |
| RTASK-P1-24 | `CancelOnDrop/DetachOnDrop/FinishOnShutdown`只是enum，无generic执行语义。 | Task Drop和scope shutdown执行policy，返回cancel requested/acknowledged/too-late/finished receipt。 |
| RTASK-P1-25 | `TaskPollBudget`没有production consumer，也不与frame/main-thread continuation关联。 | MainThreadExecutor按frame budget轮询，输出deferred count、age、deadline miss和starvation。 |
| RTASK-P1-26 | JobHandle没有TaskId，无法在日志、trace、shutdown blocker或依赖图中定位工作。 | generation-qualified `TaskId`永不静默复用，可从handle、event和diagnostics一致查询。 |
| RTASK-P1-27 | handle没有owner/scope/scheduler identity；跨runtime handle看起来完全相同。 | `TaskHandle<T>`携owner identity，跨owner dependency必须经过显式bridge/fence。 |
| RTASK-P1-28 | tracked task只能返回`()`；业务结果另建Mutex/channel/map，重复terminal协议。 | `Task<T, E>`保存typed result或bounded result locator，支持borrow/take/late observer。 |
| RTASK-P1-29 | detached `spawn`返回`()`，调用者没有admission、task id、terminal或failure receipt。 | 默认spawn返回task；detach必须显式指定owner、failure sink、retention和shutdown policy。 |
| RTASK-P1-30 | JobHandle没有cancel API，running/queued/dependency-waiting都不能统一请求取消。 | cancellation token与scheduler state集成，queued可retract，running cooperative，terminal有ack。 |
| RTASK-P1-31 | handle Drop隐式detach，和framework默认`CancelOnDrop`矛盾。 | Drop policy来自descriptor并有静态默认；危险detach在调用处可见且可审计。 |
| RTASK-P1-32 | generic task无deadline；bounded lane只能在外层自行安排timer。 | descriptor持start deadline、completion deadline和timeout policy，timer只负责wake而不执行业务。 |
| RTASK-P1-33 | tracked panic压成字符串，`wait()`重新panic，调用者不能做typed恢复。 | terminal为`Succeeded/Cancelled/Deadline/DependencyFailed/Panicked/Rejected`及结构化cause。 |
| RTASK-P1-34 | detached panic沿Rayon默认路径终止进程，同一scheduler的两种API故障域相反。 | panic policy显式声明；普通task隔离并上报，只有host批准的fatal task可终止进程。 |
| RTASK-P1-35 | 非字符串panic被压成固定文本，backtrace、task identity、owner和stage丢失。 | FailureRecord保存分类、panic payload摘要、backtrace policy、owner、worker和trace correlation。 |
| RTASK-P1-36 | 没有attempt、retry、checkpoint、idempotence或cleanup owner；业务层各自重做。 | descriptor可声明retry policy和transaction artifact，但默认不自动重试非幂等工作。 |
| RTASK-P1-37 | `wait()`只有无限等待，host shutdown无法传deadline并得到unfinished原因。 | `wait_until/try_join`返回typed wait outcome；产品Drop不得偷偷转成无限等待。 |
| RTASK-P1-38 | wait不接收cancellation token，外部scope关闭后等待者仍可永久阻塞。 | wait可被scope/deadline中断，但task terminal与observer timeout必须严格区分。 |
| RTASK-P1-39 | `schedule_after`接受任意scheduler的handles，无owner、executor或lifetime兼容性验证。 | dependency edge验证identity、retirement和allowed cross-domain transition。 |
| RTASK-P1-40 | dependency graph没有cycle检测；动态互依会永久pending且无diagnostic。 | admission增量检测cycle，或使用只能向前引用的builder/frozen DAG。 |
| RTASK-P1-41 | dependency panic把child标为相同panic字符串，取消、上游失败和自身panic不可区分。 | child terminal记录`DependencyFailed { dependency, terminal }`，policy决定skip/fallback/run。 |
| RTASK-P1-42 | `combine`从首个handle借diagnostics，混合scheduler时统计归属随机且结果只保留首个panic。 | join-set有独立owner和ordered terminal vector，跨scope组合显式声明。 |
| RTASK-P1-43 | completion在线程内同步递归发布所有continuation，宽fanout/深chain可放大terminal延迟和stack。 | bounded continuation queue或迭代trampoline，记录fanout、depth、dispatch wall和backpressure。 |
| RTASK-P1-44 | terminal observer同步运行；`wait()`不等待observer，slow observer阻塞worker，panic只留在handle局部计数。 | observer进入受控continuation lane；completion barrier语义明确，panic汇入统一failure sink。 |

### 6.4 Diagnostics、trace 与 product truth（P1-45 至 P1-53）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RTASK-P1-45 | scheduler diagnostics默认关闭，CoreRuntime构造没有`with_diagnostics`。 | diagnostics policy由profile显式配置，development默认可观测，shipping支持sampling而非全空。 |
| RTASK-P1-46 | production中没有scheduler `record_diagnostics`或`diagnostic_report` consumer，公开常量不等于产品接线。 | Runtime frame diagnostics统一采样execution owner并进入Editor/profile/status snapshot。 |
| RTASK-P1-47 | 提交工作后再enable会得到无generation的部分累计值，无法知道窗口起点。 | report携enabled_at/reset_generation/sample window，禁止把partial counter伪装成lifetime total。 |
| RTASK-P1-48 | 指标仅按scheduler聚合，没有TaskId、scope、owner、domain、priority、label或failure class。 | bounded label registry与scope aggregation，支持top offenders而不造成高基数字符串爆炸。 |
| RTASK-P1-49 | 只累计总纳秒和samples，没有p50/p95/p99、max、histogram或deadline lateness。 | per-domain bounded histogram和slow-task sample，基线说明采样成本。 |
| RTASK-P1-50 | 没有rejected、quota、steal、park、utilization、queue capacity、worker saturation或blocking compensation。 | execution health同时报告需求、容量、排队、执行、等待、取消和shutdown阶段。 |
| RTASK-P1-51 | TaskPoolReport只反映声明线程数，不包含真实线程ID、alive/parked、private worker或queue。 | WorkerInventory成为唯一线程事实，shared和dedicated统一列账。 |
| RTASK-P1-52 | 聚合重试失败时静默返回last stable snapshot，consumer不知道数据过旧。 | snapshot携captured_at/stale/retry_exhausted及age；超龄时产品健康度降级。 |
| RTASK-P1-53 | 没有task DAG、parent/child、scope lifetime、critical path或trace span关联。 | 低成本task trace支持按需capture，离线重建依赖、queue wait、execution和retirement blocker。 |

### 6.5 Timer 与 deadline service（P1-54 至 P1-61）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RTASK-P1-54 | 所有runtime共享固定512项process timer，单个子系统可耗尽全局registration。 | DeadlineService按runtime/scope配额，注册返回owner-qualified id与capacity report。 |
| RTASK-P1-55 | timer registration没有TaskId、owner、priority、purpose或shutdown policy。 | registration绑定scope和target task/operation，owner retirement可批量cancel并等待quiescence。 |
| RTASK-P1-56 | 单一timer线程串行执行任意callback，慢callback阻塞所有deadline。 | timer线程只维护时序和wake；业务callback投递到control/target lane并受预算。 |
| RTASK-P1-57 | callback panic被`catch_unwind`后静默丢弃，没有terminal或diagnostic。 | timer callback有typed outcome和failure sink，panic关联registration与owner。 |
| RTASK-P1-58 | subscription cancel只移除未取出的deadline；已取出的callback可与cancel/Drop并发执行。 | cancel receipt区分removed/in-flight/completed，并可等待callback lease归零。 |
| RTASK-P1-59 | interval在取出时以`Instant::now()+interval`重排，slow callback会漂移；多callback又无miss policy。 | 明确fixed-rate/fixed-delay/coalesce/skip/catch-up政策和最大补偿次数。 |
| RTASK-P1-60 | 没有scheduled_at/fired_at/late_by/callback_wall/queue_delay指标，deadline功能不可资格化。 | 每类deadline输出lateness histogram、miss count和slow callback offender。 |
| RTASK-P1-61 | explicit timer实例仍使用同一thread name，且不进入WorkerInventory，现场无法区分owner。 | thread identity包含runtime/domain generation，所有实例进入统一inventory和shutdown report。 |

### 6.6 Bounded keyed I/O lane（P1-62 至 P1-72）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RTASK-P1-62 | 每lane只有一个pump串行执行全部key；慢key阻塞无关key，无法配置并行度。 | lane声明max concurrency与per-key serialization，公平调度不同key且保留fence语义。 |
| RTASK-P1-63 | work固定`Result<(), BoundedKeyedIoFailure { static code }>`，typed结果和错误细节仍在外置Mutex。 | `BoundedOperationLane<K,T,E>`统一保存bounded result/error locator和retained bytes。 |
| RTASK-P1-64 | deadline只在worker开始前检查，running work没有completion deadline或timeout/cancel token。 | 分开start/completion deadline；运行中发cancel并报告是否ack，不能伪称已终止OS I/O。 |
| RTASK-P1-65 | cancel authority只能取消before-start；active work和shutdown只改外围publication。 | operation closure接收CancellationContext，backend定义可中断/不可中断及safe commit fence。 |
| RTASK-P1-66 | ticket id和若干epoch使用`saturating_add`，到上限后静默重复/冻结identity。 | checked exhaustion关闭lane并返回terminal error；identity永不复用。 |
| RTASK-P1-67 | pump只在work closure周围catch panic；`next_entry`、accounting、observer准备等外层panic会让`pump_active`和reservation永久卡住。 | pump-level reconciliation guard校验conservation、发布failed terminal并决定fail-closed/restart。 |
| RTASK-P1-68 | shutdown guard Drop无限`wait`；若在同一单worker executor且pump仍排队，可先卡Condvar而没有机会协助执行。 | Drop只做非阻塞fail-closed记录；显式shutdown由非worker owner以deadline驱动，检测self-wait。 |
| RTASK-P1-69 | 每entry只有一个`terminal_observer`槽，组合层无法安全注册多个独立consumer。 | ticket提供bounded multi-subscriber/cursor或统一result journal，不以替换单槽实现fan-out。 |
| RTASK-P1-70 | observer在线程内同步运行且panic静默吞掉；worker wall与observer wall也没有分开。 | observer投递受控lane，记录dispatch/observer wall、panic和slow-consumer策略。 |
| RTASK-P1-71 | lane diagnostics是局部snapshot，没有scope/key class、rejection reason、deadline miss、observer或fence cost，也未统一进入DiagnosticStore。 | 注册到ExecutionDiagnostics，按低基数owner/lane聚合并支持top-N key hash采样。 |
| RTASK-P1-72 | lane、ticket、timer、JobHandle多处统一`poisoned.into_inner()`继续运行，没有invariant检查。 | cache可重建；admission/accounting state必须journal校验或进入Poisoned并关闭新提交。 |

## 7. P2 一致性、可维护性与资格差距

| ID | 当前差距 | 建议 |
|---|---|---|
| RTASK-P2-01 | AsyncTaskHandle只是裸`u64`，默认0也可构造，没有invalid/scoped identity。 | 使用不可空、owner-qualified、checked sequence identity。 |
| RTASK-P2-02 | AsyncTaskStatus的failure message和descriptor label无byte cap、message key或redaction。 | 内部使用bounded diagnostic attachment，UI本地化在presentation层。 |
| RTASK-P2-03 | poll_count和大量diagnostic counter饱和后继续显示同一数值，不暴露overflow。 | report携overflow bit，关键identity使用checked exhaustion。 |
| RTASK-P2-04 | `TaskPoolThreadAssignmentPolicy::thread_count`用assert校验百分比，配置错误直接panic。 | 配置解析阶段返回typed validation error并保留原值来源。 |
| RTASK-P2-05 | 多个独立compute/io pool复用相同线程名前缀，profile和crash dump难定位owner。 | worker name包含domain短ID，完整owner通过thread registry查询。 |
| RTASK-P2-06 | TaskPool descriptor只有kind/thread count/name，没有schema version或来源。 | versioned config snapshot记录default/config/CLI/platform override provenance。 |
| RTASK-P2-07 | `shares_execution_owner_with`只比较Rayon Arc，不能判断runtime/scope generation。 | 调试API返回结构化owner identity，pointer equality只留内部优化。 |
| RTASK-P2-08 | `JobHandle::Debug`只输出complete，现场看不到id/owner/result/dependencies。 | bounded debug摘要包含稳定身份和状态，不泄露payload。 |
| RTASK-P2-09 | timer的`checked_add(...).unwrap_or(now)`在Instant溢出时把远期deadline改成立即触发。 | 返回DeadlineOutOfRange，不改变请求语义。 |
| RTASK-P2-10 | bounded diagnostics里queue_entries实际是reserved entries，名称会误导运维。 | 分开reserved/suspended/queued/active/completion retained counters。 |
| RTASK-P2-11 | worker wall、queue age等Duration累加饱和后无标记，长期进程统计不可解释。 | 使用窗口化metric和overflow/staleness metadata。 |
| RTASK-P2-12 | source中存在“observer必须bounded”等注释，但API不编码或强制预算。 | 将约束变成typed executor/queue policy和测试门，而不是调用者自律。 |
| RTASK-P2-13 | standalone task tests集中在一个千行文件，ownership、diagnostics、dependency、panic混在一起。 | 按contract folder拆分，但保留behavior而非source-string耦合。 |
| RTASK-P2-14 | 多项结构测试只搜索字符串或禁止某API，不能证明间接spawn、macro或运行时线程归属。 | compiler lint/capability封装加运行时WorkerInventory，source scan仅补充。 |
| RTASK-P2-15 | 当前pressure tests主要验证计数守恒，没有长期RSS、allocator、tail latency或fairness基线。 | 建立固定机器/平台baseline和noise policy，输出artifact而非只assert。 |
| RTASK-P2-16 | detached panic依赖子进程测试证明“应终止”，它固化了危险行为而不是安全合同。 | 改为typed fatal policy测试；普通detached panic必须被隔离并可观测。 |
| RTASK-P2-17 | 没有loom/shuttle式状态空间、TSAN/helgrind、fault injection或poison-mid-transaction矩阵。 | 为handle/lane/timer/shutdown建立模型测试与平台并发资格。 |
| RTASK-P2-18 | 没有Windows/Linux/macOS不同CPU规模、单核、进程退出、DLL reload和100h soak证据。 | 性能/稳定性结论必须绑定环境、构建、采样和可复验artifact。 |

## 8. 参考引擎对照与适用边界

| 参考 | 已核对机制 | Zircon 应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal Tasks | typed `TTask<T>`、debug name、priority、prerequisite、nested task、wait/retract-and-execute；Pipe提供串行链；ConcurrencyLimiter把并发上限作为一等控制 | task identity/result、priority贯通、依赖/嵌套语义、bounded continuation、并发限制器和可诊断等待 | Unreal全局scheduler面向常驻进程，不能替代Zircon动态DLL owner和Rust panic/result合同 |
| Bevy tasks | TaskPoolBuilder配置线程数/名字/stack；spawn返回可cancel/detach的typed Task；scope保证借用任务在返回前结束；pool拥有worker shutdown/join；Compute/AsyncCompute/IO使用域明确 | typed task、显式detach/cancel、scope cleanup、worker ownership、domain usage guidance | Bevy global pools同样面向常驻App；其默认不等于Zircon可卸载session安全 |
| Godot WorkerThreadPool | normal/low priority、TaskID/GroupID、group processed count、caller task/group identity、worker内协作wait、language-thread runlevel和finish join | priority lane、group task/progress、caller identity、显式runlevel与join barrier | Godot singleton/Callable/Variant和脚本线程模型不是Rust API目标 |
| Fyrox | core pool返回UUID result；engine handler把completion closure绑定plugin UUID或scene/node/script owner | 即使较薄的pool也说明result必须重新关联产品owner；插件/节点完成回调不能无主 | Fyrox无界result channel、Any downcast和较弱shutdown只能作为最低基线 |
| Unity Graphics Jobs | Graphics package使用JobHandle依赖组合、IJob/IJobFor、NativeArray和ReadOnly/WriteOnly资源声明组织高吞吐数据并行 | render/ECS热路径需要data access declaration、dependency handle和批量schedule，而不是闭包黑盒 | 本地Graphics仓库不是Unity核心job scheduler源码，不能据此推断其全部lifecycle/worker实现 |

共同工程原则是：executor owner必须知道worker和task归属；task handle必须携结果、依赖和终态；并发限制、优先级、资源访问与shutdown不是业务层补丁；专用线程可以存在，但必须进入同一生命周期和预算总账。

## 9. 目标架构

```text
Host / RuntimeOwner
  -> ExecutionRuntime
       -> WorkerInventory
       -> WorkerDomain { Compute, AsyncCompute, Io, Control, MainThread }
       -> DeadlineService
       -> ExecutionDiagnostics / TraceCapture
       -> TaskScope(runtime/module/plugin/world/subsystem)
            -> admission + quota + cancellation + generation
            -> TaskDescriptor
            -> Task<T, E>
            -> DependencyGraph / JoinSet
            -> BoundedOperationLane<K, T, E>
            -> DedicatedWorkerLease (only when required)
```

### 9.1 Identity 与 owner

`ExecutionRuntimeId + ScopeId + TaskSequence + Generation`构成task identity。scope绑定runtime session、module/plugin generation和shutdown policy；handle不能脱离scope静默继续提交。TasksModule只是Core内的service adapter，真正的process/DLL owner必须由host与动态加载边界共同决定。

### 9.2 Task 与 result

`Task<T,E>`统一admission、queued/running/terminal state、typed result、cancel acknowledgement、deadline、panic isolation和observer。`detach`是显式转换，必须指定failure sink与retirement owner。业务层不再用Mutex<Option<Result>>重建相同状态机。

### 9.3 Scheduling 与 data parallel

WorkerDomain负责priority/QoS/affinity/stack和共享预算；TaskScope负责公平配额；DependencyGraph负责DAG和cross-domain edge。ECS/Graphics数据并行另有resource access plan与批量schedule，不能仅依赖opaque closure。

### 9.4 Timer、bounded operation 与 dedicated worker

DeadlineService只做时间排序和wake，不串行业务callback。BoundedOperationLane泛化现有key/fence/byte budget，并支持per-key串行、跨key并发、typed result、active cancel和deadline。确需阻塞API或设备线程时使用DedicatedWorkerLease，统一登记、停止和join。

### 9.5 Diagnostics 与 product truth

WorkerInventory是线程数唯一事实；ExecutionDiagnostics输出按domain/scope的capacity、queued、active、rejected、cancel、deadline、latency histogram和shutdown blockers。按需trace重建DAG和critical path。App/Editor capability只能在owner ready且诊断接线后声明Tasks active。

## 10. 分阶段重构计划

### M0：冻结 owner、caller 与 shutdown truth

- 禁止新增`TaskPool::new/JobScheduler::default/process_default/direct thread`生产调用，建立allowlist与owner inventory。
- TasksModule readiness改为真实execution capability；dynamic/Editor shutdown报告所有worker/task/timer blocker。
- 保持Runtime02与Editor09 P0 owner，不在本篇另造并行总事务。

### M1：ExecutionRuntime、WorkerDomain 与可失败构造

- 引入可注入、可停止、可join的ExecutionRuntime。
- 修正thread budget conservation；统一shared/dedicated worker inventory。
- 为platform QoS/affinity/stack/start-stop hook建立配置和降级报告。

### M2：TaskScope、TaskDescriptor 与 typed Task

- 合并framework descriptor/status和runtime handle。
- 实施owner/generation、typed result、cancel/drop/shutdown policy、deadline和failure record。
- 将raw pool API收窄为executor内部能力。

### M3：DependencyGraph、completion 与 data parallel

- 增加cycle/cross-owner验证、typed dependency failure和JoinSet。
- continuation改为bounded trampoline/queue；observer有明确completion barrier。
- 为Scene ECS、Graphics和Text shape建立resource-aware batch execution。

### M4：DeadlineService 与 bounded operation convergence

- 替换process timer的inline callback和全局512共享上限。
- 将bounded keyed I/O泛化为typed、可并行、可active-cancel的operation lane。
- 加pump reconciliation、checked identity和非阻塞Drop。

### M5：迁移 production consumers

- 先迁移dynamic scene、operation、asset、preferences/settings、VM discovery。
- 再迁移Text raster/SDF、Graphics compile/render submission和watch/config/log dedicated workers。
- EditorJobSystem保留业务admission，但底层改用scope-aware task；Editor09继续拥有UI/product行为。

### M6：Diagnostics、trace 与 product integration

- Core按profile启用并记录execution metrics；Editor runtime diagnostics展示同一generation快照。
- App/Editor/commandlet/headless共用shutdown choreography与blocker输出。
- capability、status和module activation不再报告descriptor-only成功。

### M7：性能、故障与跨平台资格

- 单核到高核、Windows/Linux/macOS、不同task mix和oversubscription矩阵。
- panic、poison、timer saturation、worker spawn failure、stuck IO、plugin unload、deadline storm和shutdown timeout fault injection。
- 发布可复验benchmark/profile/soak/sanitizer artifact后，才能讨论达到或超过Unreal。

## 11. 验收门禁

### 11.1 Owner 与 lifecycle（G01-G08）

| Gate | 验收条件 |
|---|---|
| G01 | 每个production worker、timer和task都可查询runtime/scope/module/plugin owner。 |
| G02 | TasksModule activate返回execution ready receipt，deactivate返回quiescence receipt。 |
| G03 | dynamic session destroy在任一worker/task/callback未退出时fail-closed，不允许DLL unload。 |
| G04 | ExecutionRuntime构造失败返回typed error并join所有已创建worker。 |
| G05 | close admission后所有入口，包括clone、process alias和private lane，均拒绝新工作。 |
| G06 | shutdown按cancel/finish policy处理task，并列出未完成TaskId与owner。 |
| G07 | shared和dedicated线程总和与WorkerInventory conservation一致。 |
| G08 | 不存在未登记的production `TaskPool::new/JobScheduler::default/direct thread`调用。 |

### 11.2 Task model 与 dependency（G09-G16）

| Gate | 验收条件 |
|---|---|
| G09 | framework descriptor/status和runtime task是同一状态机，无scene私有平行实现。 |
| G10 | `Task<T,E>`提供typed terminal，普通panic不终止进程。 |
| G11 | Drop/cancel/shutdown policy行为可测试并返回ack，不以注释约束。 |
| G12 | start/completion deadline与observer timeout可区分，running timeout不伪装成已终止。 |
| G13 | cross-owner dependency无显式bridge时拒绝，cycle在admission阶段失败。 |
| G14 | dependency failure保留上游TaskId和terminal class，child policy明确。 |
| G15 | 100,000深链/宽fanout不会递归栈溢出或让单worker长时间执行observer。 |
| G16 | wait_until、JoinSet、late observer和result retention均有race/timeout测试。 |

### 11.3 Scheduling、budget 与 data parallel（G17-G24）

| Gate | 验收条件 |
|---|---|
| G17 | 1/2/4/8/32/64核配置中实际worker不超budget，低核仍可前进。 |
| G18 | priority贯穿Editor/runtime queue，包含ageing与priority inversion测试。 |
| G19 | per-scope entry/byte/cost quota在并发admission下严格守恒。 |
| G20 | blocking IO、CPU、main-thread和control work不会互相耗尽worker。 |
| G21 | nested parallelism和worker-side wait在单worker与交叉domain矩阵无死锁。 |
| G22 | ECS/Graphics data access冲突形成可验证DAG，错误声明fail-closed。 |
| G23 | task scheduling基准输出throughput、queue p95/p99、steal、utilization和allocation。 |
| G24 | dedicated worker有平台QoS/affinity/stack evidence和完整join test。 |

### 11.4 Timer、bounded operation 与 diagnostics（G25-G32）

| Gate | 验收条件 |
|---|---|
| G25 | timer capacity按scope隔离，一个owner的storm不能饿死其他owner。 |
| G26 | timer线程不执行无界业务callback；slow/panic callback有diagnostic和terminal。 |
| G27 | cancel registration可等待in-flight callback lease归零。 |
| G28 | fixed-rate/fixed-delay/coalesce/skip语义各有deterministic fake-clock测试。 |
| G29 | bounded operation支持per-key串行、cross-key并发、typed result和active cancel。 |
| G30 | pump任意阶段panic后capacity/handle/ticket conservation恢复或lane fail-closed。 |
| G31 | shutdown guard在同worker、timeout和Drop路径不死锁、不无限隐式等待。 |
| G32 | execution snapshot标记generation/window/stale，并进入真实DiagnosticStore consumer。 |

### 11.5 Product、fault 与性能资格（G33-G40）

| Gate | 验收条件 |
|---|---|
| G33 | Asset/Scene/Preferences/VM/Text/Graphics/Editor adoption matrix无未声明旁路。 |
| G34 | Runtime operation、plugin和script task panic不会杀死Editor，failure可关联owner。 |
| G35 | Editor close在unfinished task存在时保持依赖存活并提供retry/wait/force policy。 |
| G36 | App、Editor、headless、commandlet使用同一execution shutdown protocol。 |
| G37 | dynamic library连续load/run/unload至少10,000轮，无旧worker/callback进入卸载代码。 |
| G38 | 100h soak无task/result/timer/observer/thread增长，identity无复用。 |
| G39 | loom/model、sanitizer/helgrind或等价并发资格覆盖handle/lane/timer/shutdown关键状态机。 |
| G40 | 与Unreal/Bevy/Godot对照的公开benchmark artifact满足预先冻结的性能和稳定性阈值。 |

## 12. Owner 路由与实施约束

| 责任 | Canonical owner |
|---|---|
| ExecutionRuntime、TaskScope、Task/Handle、DeadlineService、WorkerInventory | Runtime59 / Runtime11 |
| dynamic session destroy、DLL unload permission与host code lease | Runtime02 + App01 + RuntimeInterface01 |
| Core module/service lifecycle与TasksModule service publication | Runtime01/46，Runtime59提供execution adapter |
| EditorJobSystem业务admission、progress、notification与close UX | Editor09 |
| stable task/scope/generation identity规范 | Runtime24 |
| Asset/Scene/Text/Graphics/VM具体算法和业务结果 | 各自现有专项；只迁移executor合同 |
| 全仓direct-thread lint、并发工具和CI资格 | Tooling35/24，当前按用户要求暂停扩写 |

实施必须硬切到canonical task合同，不保留`AsyncTaskDescriptor`与JobHandle长期双轨，不用`pub use`/compat wrapper掩盖旧路径。任何专用线程必须先证明为何不能使用共享domain，再取得DedicatedWorkerLease。不得用“tests pass”替代真实worker census、shutdown receipt或dynamic unload证据。

## 13. 复核与未执行项

- 已逐文件复核36个canonical core文件，并按符号扫描82个production consumer/private worker文件。
- 已核对`src/tests/tasks.rs`、bounded keyed I/O pressure/fence测试、runtime absorption job inventory与Editor jobs tests共27个文件。
- 已核对Unreal Tasks、Bevy tasks、Godot WorkerThreadPool、Fyrox core/engine task与Unity Graphics jobs共29个参考文件。
- 静态确认scheduler diagnostics没有production record consumer；TasksModule没有lifecycle；process pool/timer和多批private worker不进入统一shutdown。
- 未运行Cargo，因为当前MVP门禁只允许review/documentation，且工作树有大量其他会话改动。
- 未运行真实Editor、dynamic DLL unload、profiling、benchmark、stress、soak、sanitizer或跨平台测试；所有性能判断保持待证。
- 用户已要求暂停tooling优化；本篇只引用既有Tooling owner，不新增或修改tooling专项。

## 14. 当前状态

- Review：完成。
- Production实现：未开始。
- 新增P0：0。
- 继承P0：2，分别归Runtime02与Editor09，不重复计数。
- P1：72。
- P2：18。
- 验收门禁：40。
- 下一实施入口：先完成M0 owner/caller/shutdown truth和M1可停止ExecutionRuntime，再允许扩展新的异步功能。
