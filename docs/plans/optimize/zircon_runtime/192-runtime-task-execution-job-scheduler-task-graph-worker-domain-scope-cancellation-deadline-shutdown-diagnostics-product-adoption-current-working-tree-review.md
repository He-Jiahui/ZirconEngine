---
title: Runtime Task Execution、Job Scheduler、Task Graph、Worker Domain、Scope、Cancellation、Deadline、Shutdown、Diagnostics 与 Product Adoption 当前工作树复核
category: zircon_runtime
report_id: Runtime192
review_date: 2026-08-30
baseline_head: cc5cadbd597c3707954ebd6109fad0fd5643a152
doc_type: current-working-tree-review-and-refactor-plan
review_status: current_working_tree_refresh_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
refreshes:
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99o-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/158-runtime-core-events-tasks-timer-event-bus-task-graph-current-source-review.md
related_plans:
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-27-native-plugin-discovery-bounded-refresh-publication.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-08-05-task-diagnostics-editor-log-source-bridge.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-08-10-blocking-io-process-output-budget.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-08-23-task-terminal-delivery-bounded-dispatch.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphDefinitions.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/TaskGraph.cpp
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_tasks/src/thread_executor.rs
  - dev/bevy/crates/bevy_tasks/src/edge_executor.rs
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/PassesData.cs
---

# Runtime192 当前源码审查

## 1. 当前裁决

本轮针对当前工作树重新扫描了任务执行、JobScheduler、TaskGraph、Timer、callback dispatcher、bounded keyed/stream I/O、OperationService、CoreRuntime teardown，以及 asset、scene、plugin、graphics、text、platform、app、editor 和 network 的生产消费者。任务/操作核心闭包为 **93 个 Rust 文件、16,543 行、561,403 bytes、165 个测试标记（11 个 ignored）**。参考引擎 scheduler 核心选择集为 **11 个关键文件、7,098 行、247,097 bytes**，另核对 **3 个 Unity Graphics JobHandle/RenderGraph 消费文件**。统计只作为审计范围记录，不是性能结论。

当前源码已经具备可保留的工程底座：

- EngineTaskGraph 能创建一个 runtime-owned Rayon worker set；TaskGraphScope 具有 admission、capacity、取消策略、依赖前置失败处理、census 和 quiescent wait。
- JobHandle 具备依赖计数、terminal observer、panic containment、continuation dispatcher 和 worker 协助等待；Runtime11 failure-2026-08-23 的深链、宽 fan-out、同 deadline timer 测试已有 managed Windows 证据。
- bounded keyed/stream lane 已有 entry/byte reservation、generation/fence、固定 read/queue/drain 上限及 shutdown terminal；诊断 journal 已有 256 条 terminal 观察上限、64 条批量上限和 4 KiB 消息上限。
- OperationService 已把原始 JSON admission、prepare/apply owner-thread边界、completion channel、TTL/harvest 和 retained bytes 分开建模。

这些局部能力仍没有形成 Unreal、Bevy、Godot、Fyrox 或 Unity Graphics 那种可证明的统一 execution contract。核心问题是 owner、identity、capacity、deadline、result、cancellation 和 teardown 仍各自有多套实现：

1. TaskPools::process_default 继续持有三类进程级 pool，TaskTimer::process_default 和 callback dispatcher 又依赖该静态 owner；EngineTaskGraph::worker_inventory 明确不统计它们及 dedicated threads。
2. TaskPool::new/spawn/install/join、JobScheduler::from_pool/spawn/install/join、OperationService 的 raw scheduler、Graphics/Text/asset watcher 等 private thread 仍是可达的生产旁路。
3. TaskDescriptor 没有 owner、generation、priority、deadline、result contract 或 affinity；TaskState 没有 timeout/dependency-failed/panicked/rejected/shutdown terminal；JobHandle::wait() 无界且无 timeout outcome。
4. timer 只有单线程排序和 callback 投递，没有 registration owner、miss policy、callback budget、in-flight cancellation receipt 或统一 failure sink；dispatcher 只限制每次运行数量，队列本身无 entry/byte/age admission。
5. OperationService 是第二套任务状态机，prepare 可以长期阻塞 graph worker，cancel/deadline 不能中断 in-flight work，tick 没有 owner time/bytes budget，且没有统一 graph shutdown receipt。
6. bounded stream read 的 token 未被读取循环消费，阻塞 Read 可无限占用 graph worker；一个 capture 可以占满所有 worker，跨 capture 没有公平调度。

因此：继承的 dynamic session/DLL unload execution-owner P0 仍 **Open**；Runtime192 不新增独立 P0，但确认其根因仍由 process-static pool/timer、private worker 和不完整 shutdown census 构成。稳定的 72 项 P1 本轮重判为 **58 Open / 14 Partial / 0 Closed**，18 项 P2 为 **16 Open / 2 Partial / 0 Closed**，40 项资格门为 **40 Fail / 0 Partial / 0 Pass**。没有任何依据可以宣称性能或表现优于 Unreal；本轮没有运行跨平台 Release benchmark、RSS/allocator、DLL reload、100h soak 或真实产品关闭验证。

## 2. 当前源码冻结与证据边界

| 范围 | 当前证据 |
|---|---|
| canonical task model | zircon_runtime/src/core/runtime/tasks/mod.rs、task_id.rs、task_descriptor.rs、task_state.rs、task_status.rs、task_cancellation_policy.rs |
| worker owner | pool.rs、pools.rs、thread_assignment.rs、task_graph/engine_task_graph.rs |
| graph/scope/handle | task_graph/scope.rs、task_graph/task_handle.rs、job_handle.rs、job_scheduler.rs |
| callback/timer | callback_dispatcher.rs、timer.rs 及其同目录 tests |
| bounded lanes | bounded_keyed_io/*、bounded_stream_io/*、retained_byte_budget.rs |
| diagnostics | diagnostics.rs、diagnostic_observation/*、report.rs |
| second operation state machine | zircon_runtime/src/operation/* |
| CoreRuntime teardown | core/runtime/runtime.rs、core/runtime/state/core_runtime_state.rs、core/runtime/handle/core_handle.rs、dynamic session shutdown path |
| production consumers | asset、scene、plugin、graphics、text、platform、zircon_app、zircon_editor、zircon_plugins/net |

本轮只做静态源码、调用点、focused test、既有 failure record 和仓内参考源码审查。没有修改 Rust、Cargo、ABI、ZUI、测试或 tooling；没有把历史会话的 Cargo 结果当作本轮验收证据。当前工作树本身存在其他修改，本文只记录它们对 execution contract 的影响，不回滚或覆盖这些修改。

## 3. 当前实现闭环与断点

### 3.1 Framework 与 runtime task model

zircon_runtime/src/core/framework/tasks/mod.rs 现在只保留 ParallelSliceExecutor，提供 parallel_for、serial fallback 的 parallel_map_indices 和 parallel_map_ordered。旧的 framework async descriptor 已删除，这是方向正确的 hard cut；但 framework fast path 仍是 raw blocking closure，未产生 task record、cancel token、cost estimate 或 diagnostics。

runtime TaskDescriptor 目前只有 TaskId、logical kind、label 和 TaskCancellationPolicy。TaskId 是单调 u64，没有 generation/owner/epoch；TaskState 只有 Pending、Running、Completed、Failed、Cancelled。缺少结果类型、priority、queue/start/completion deadline、affinity、dependency failure 和 typed rejection。

### 3.2 Pool、graph 与 scheduler

EngineTaskGraph 的确只创建一个 worker_pool，并在 create_scope 时维护 scope census；shutdown 可从 Running 进入 Closing，关闭 admission，等待 scope quiescence，再 close/join graph pool，超时可重试。Drop 只关闭 scope admission，不能替代显式 shutdown。

同一模块又保留 TaskPools 的 io、async_compute、compute 三个独立 Rayon pool及 PROCESS_TASK_POOLS: OnceLock。TaskPool 对外公开 spawn/install/in_place_scope/join，且 submission_or_panic 把关闭/容量错误转为 panic。JobScheduler 可从任意 TaskPool 构造，spawn 返回 ()，install/join 仍是无 task identity 的 raw API。EngineTaskGraph::worker_pool() 被 app、asset、graphics、platform、editor 直接调用，形成新的 escape hatch。

### 3.3 Handle、依赖和 completion

TaskGraphScope::schedule_after 会检查 scheduler 与 graph pool 的 pointer ownership，并用依赖 handle lease 将 terminal callback 绑定到 child；scope 记录可在依赖失败或取消前置时结束。这是可保留的局部实现。

但 JobHandle::combine 只使用首个 handle 的 dispatcher，不验证混合 scheduler/owner；schedule_after 不做 cross-owner 依赖、generation 或 cycle 检测。wait() 只有无限等待，worker 内部以协助执行和 1 ms park 避免部分死锁；没有 wait timeout 与 typed TimedOut/Rejected/DependencyFailed outcome。observer 虽已有有界 dispatcher和 panic containment，但 observer queue 无 admission cap、bytes cap、age cap 或 owner census。

### 3.4 Timer 与 callback dispatcher

TaskTimer 使用固定容量 512 的进程级控制线程 zircon-runtime-timer，将 deadline 放入 BTreeMap，periodic tick 通过 delivery_pending 合并，再把 callback 投递到 TaskCallbackDispatcher。同 deadline 隔离、周期 coalescing、timer drop 后抑制 queued callback 均有 focused tests。

断点仍是：TaskTimer::new 默认连接 TaskCallbackDispatcher::process_default；registration 只有内部 u64 id 和 closure；callback panic 被 catch 后没有统一 terminal/failure record；取消只能移除尚未取出的 registration，不能等待 in-flight callback lease；interval 以 now + interval 重排，没有 fixed-rate/fixed-delay/miss/catch-up policy。dispatcher 的 VecDeque 没有队列容量/字节/年龄限制，只有每 runner 64 callback 和最多 2 runner 的执行预算。

### 3.5 Bounded keyed/stream I/O

BoundedKeyedIoLane 的 reservation、coalescing、fence、deadline-before-start 与 terminal enum 是可靠基础；但每 lane 单 pump 串行所有 key，active work 不能被 cancellation/deadline 中断，ticket/epoch 使用 saturating increment，terminal observer 是单槽并可覆盖，shutdown guard Drop 仍可能无界等待。

BoundedStreamIoLane 以 graph scope 和 reader budget 限制 reader 数、chunk、line、queue entry/bytes；然而 reader 的 token 未进入阻塞 Read，一条坏 pipe 可永远占住 worker。单 capture 可消耗整个 graph worker set，多个 capture 没有 priority/fairness；drain(max_bytes=0) 仍可返回第一条记录；failures 集合按 reader 数增长，没有 lane-level terminal journal。

### 3.6 OperationService 第二状态机

zircon_runtime/src/operation 自有 Queued、Preparing、ReadyToApply、Completed、Failed、Cancelled、Expired、Harvested 以及 queue index、retained bytes、TTL 和 maintenance timer。raw JSON 在 decode 前做 byte admission，owner-thread apply 的原则是正确的。

它仍通过 core.scheduler().spawn 启动 prepare，没有 TaskGraphScope/TaskDescriptor/owner generation；prepare cancellation 只是标志位，运行中的 future/同步工作不能停止。completion channel 是 bounded count 而非 owner time/bytes budget，tick 会扫描任务 map，维护 timer 使用 process default。没有 service close、shutdown fence、统一 task terminal receipt，也没有 Editor/Navigation/Neural 之外的真实 adoption。

### 3.7 Diagnostics 与 product truth

diagnostics 有 64 shard stable snapshot、lifecycle counters、queue/execution/dependency/wait duration、bounded terminal journal、typed cursor/identity 和 editor host bridge。failure observation 目前只区分 Cancelled 与 Panicked，label/owner/scope/domain 不进入 observation。

缺口是 dispatcher depth/age、timer late/miss/callback wall、scope census、private worker inventory、deadline miss、priority fairness、rejected/quota、histogram/p95/p99、stale snapshot metadata。CoreRuntime 默认不构造 with_diagnostics，生产没有统一 record_diagnostics/diagnostic_report consumer。Runtime11 failure-2026-08-05 已实现 bridge，但 managed validation 和完整 product matrix 仍 pending。

## 4. 生产 owner 与旁路矩阵

| 生产路径 | 当前入口 | 工程化差异 |
|---|---|---|
| CoreRuntime / editor manager / app viewer | JobScheduler::from_pool(core.task_graph().worker_pool().clone())；app 暴露 worker_pool() | raw pool clone 无 owner/generation capability，关闭只覆盖 graph pool |
| Asset | ProjectAssetManager、open_project.rs 的 TaskPool::new(io)、scene writer | 注入与 process/default pool 并存；asset pipeline 没有统一 scope/cost/deadline census |
| Plugin discovery | TaskPools::process_default().io() | native plugin refresh 可在 session teardown 后保留 static worker/closure；failure record 只覆盖 admission/terminal，不覆盖 host unload |
| Operation | core.scheduler().spawn | 第二状态机绕过 graph descriptor、scope、result receipt 和 shutdown owner |
| Graphics | render framework / pipeline construction 领取 graph pool；visibility culling 仍可新建 TaskPool；async compile 与 pipelined render 各有 OS worker | GPU/device thread、compile worker、render-submit worker 不在 WorkerInventory，Drop join 无统一 timeout/failure receipt |
| Text | TextRenderState 使用 process pools；raster_pool.rs 另起每 worker OS thread | shared budget 被重复解释成 dedicated workers；Drop 对 raster threads 无界 join |
| Scene / Platform / VM | 多数路径已注入 graph pool，但 scene loader 仍有 process compute helper，preferences/scene artifact 有独立 scheduler construction | source migration 不等于 policy migration，调用点仍能绕过 module/scope owner |
| App viewer / watcher / logging / config / network | viewer background thread、asset watcher、diagnostic sink、config persistence、net worker | 各自有局部 queue/timeout，但没有共同 shutdown receipt、generation 或 inventory |
| Editor | export_cargo_process、output capture 各自 TaskPool::new；Editor async operation 仍同步调用 Runtime OperationService | Editor close 无法列出全部 unfinished worker/task；Editor256 的 async command 不能获得 typed runtime task terminal |

这些旁路是本报告把多数 P1 保持 Open 的理由。单个局部测试通过，只能证明局部实现，不代表动态 session、跨模块、跨平台和产品关闭闭环。

## 5. 五参考引擎对照

| 参考 | 可直接核实的合同 | Zircon 应吸收 | 不应误抄 |
|---|---|---|---|
| Unreal TaskGraph | FTaskGraphInterface 提供 QueueTask、named thread/AnyThread、GetNumWorkerThreads、WaitUntilTasksComplete、协作处理、TriggerEventWhenTasksComplete；FGraphEventRef/TGraphTask 将 prerequisites、desired thread、completion event 放在同一图；优先级有 high/background fallback | graph event identity、named/worker domain、依赖边、协作等待、优先级/affinity、停止和 worker join 的统一接口 | Unreal scheduler 常驻 process；不能直接解决 Zircon dynamic library generation |
| Bevy tasks | TaskPoolBuilder 配置线程数、名字、stack/start-stop hook；pool 拥有 executor 和 worker JoinHandle；Task<T> 支持 cancel/detach；scope 处理 borrowed task；ThreadExecutor 由外部 tick | typed result、scope lifetime、pool-owned join、thread hook、main-thread executor、显式 cancel/detach | Bevy global pool 适合常驻 App，不能原样作为每 session DLL owner |
| Godot WorkerThreadPool | TaskID/GroupID、high/low priority、group progress、caller task id、collaborative wait、runlevel exit、finish() 与遗留 task 处理；named pool 明确是特殊用途 | group/scope census、runlevel close、协作等待、priority promotion、join census 和 thread enter/exit | Godot Callable/Variant 不是 Zircon typed Rust result；named pool 也不能成为随意旁路 |
| Fyrox | TaskPool 用 UUID 标识异步任务，以结果 channel 回投；TaskPoolHandler 把完成结果在下一 update 交给 plugin、scene node 或 script | owner-aware result delivery、下一帧主线程 apply、对象 retirement 检查 | Fyrox pool 对 capacity/cancel/shutdown 较薄，只能作为 owner 回投参考 |
| Unity Graphics | GPUDriven jobs 使用 IJob/IJobParallelFor、Schedule/ScheduleParallel、JobHandle.CombineDependencies、NativeContainer deferred dispose；RenderGraph compiler 记录 pass dependency、async compute 和 graphics fence | 数据访问声明、resource hazard、批切分、依赖 artifact、GPU/CPU 生命周期边界 | dev/Graphics 是 package consumer，不是 Unity core scheduler；不能从中推断 Zircon worker shutdown 或全引擎性能 |

共同原则是：任务不是 FnOnce 加一个线程池；它必须带 owner、可枚举 identity、资源/数据依赖、bounded admission、typed terminal、可观察 failure 和可证明 join。

## 6. P0 继承项

### RTASK-P0-01（Inherited，Open）：process-static worker 越过 dynamic session / DLL unload

PROCESS_TASK_POOLS、PROCESS_TIMER、PROCESS_CALLBACK_DISPATCHER 是 OnceLock 长寿命 owner；private text raster、pipeline compile、render submit、asset watcher、log/config/network/viewer threads 不在 EngineTaskGraph::worker_inventory。动态 session shutdown 只关闭 graph scope、module 和 graph pool，不能证明这些 worker/callback 已经停止。任何回调仍捕获旧 cdylib generation 时，destroy success 不能作为 FreeLibrary barrier。该项由 Runtime11/Runtime158 持有，本报告不重复计数，但 Runtime192 将其列为所有重构的前置条件。

## 7. P1 工程化差距总账

下表保留 Runtime59/99o 的稳定 ID，状态以当前源码为准；Partial 仅表示有局部底座，不表示产品完成。

### 7.1 Owner、module、worker 与 admission

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-01 | Open | CoreRuntime 只注入 graph pool，不能注入 host owner 或返回统一创建/回滚 receipt | ExecutionRuntime::try_new(host_owner)，构造失败带已创建 worker census |
| RTASK-P1-02 | Open | TasksModule 仍主要是 descriptor，activate/deactivate 不拥有 execution generation | module service generation、ready receipt、deactivate quiescence receipt |
| RTASK-P1-03 | Open | process_default/process_io 仍可被任意子系统取得 | host 唯一创建，consumer 只能领取 scoped lane capability |
| RTASK-P1-04 | Open | JobScheduler::default/from_options 可另建完整 pool | 删除 production default，fixture 显式 isolated executor |
| RTASK-P1-05 | Open | public TaskPool::new 已被 graphics、editor、asset 等生产路径使用 | dedicated worker lease 或共享 WorkerDomain，禁止裸构造 |
| RTASK-P1-06 | Open | clone 只保存 Arc/Weak，没有 runtime/module generation | handle 携 owner、scope、generation，stale 提交结构化拒绝 |
| RTASK-P1-07 | Partial | TaskGraphScope 已有 capacity/census，但没有 module/plugin/world/subsystem owner scope | 所有业务 scope 注册 parent owner、retirement blocker 和 quota |
| RTASK-P1-08 | Partial | graph pool 有 close/join；process/private pool 没有统一协议 | close -> cancel -> drain -> stop -> join，阶段均有 deadline |
| RTASK-P1-09 | Open | private workers 不在统一 supervisor/inventory | 登记 owner、generation、QoS、stop、join、code lease |
| RTASK-P1-10 | Open | worker 没有统一 start/stop/TLS/allocator hook | WorkerDomain 声明平台 hook、失败回滚和 provenance |
| RTASK-P1-11 | Open | 三类 process pool 的 minimum worker 可超过总 budget | 可满足性 solver 与实际 worker conservation |
| RTASK-P1-12 | Open | text async budget 再次创建 dedicated raster OS worker | shared slot 与 dedicated reservation 分离且不可重复消费 |
| RTASK-P1-13 | Open | graphics public constructor 可另建 full compute pool | renderer lane 与 device worker 显式登记并限额 |
| RTASK-P1-14 | Partial | asset/scene/preferences/VM 有部分注入，但仍存在 process helper | 全部 constructor 接收 scope/lane；fixture 才能 isolated |
| RTASK-P1-15 | Open | priority 只影响 editor pending，runtime queue 同级 | stable priority、ageing、inversion diagnostics |
| RTASK-P1-16 | Open | descriptor 无 affinity/QoS/stack/background/oversubscription | WorkerDomain 平台验证和降级 receipt |
| RTASK-P1-17 | Partial | bounded lanes 有 reservation，generic spawn 无 typed Full/Closed/Quota | quote -> atomic reserve -> rollback lease -> typed admission |
| RTASK-P1-18 | Open | 无 per-scope/plugin/world quota | 分层硬上限，有限 borrow 也须可审计 |
| RTASK-P1-19 | Open | 无 NUMA/locality/steal/blocking compensation policy | 先形成可测 topology，再声明调度策略 |
| RTASK-P1-20 | Open | raw spawn/install/join/in_place_scope 绕过 census | raw API 限制在 executor 内部，公共入口生成 record |
| RTASK-P1-21 | Partial | parallel_for 有 small-input fast path，但只有手工 chunk/blocking closure | cost-aware split、cancel、priority、typed reduction、nested policy |
| RTASK-P1-22 | Open | generic executor 没有 future/main-thread/blocking 隔离 | CPU、blocking I/O、async future、main-thread 四类合同 |

### 7.2 Identity、result、cancel、dependency 与 completion

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-23 | Open | framework 与 runtime 仍是两套 descriptor/status | canonical descriptor 由 scheduler admission 消费 |
| RTASK-P1-24 | Partial | CancelOnDrop/DetachOnDrop/FinishOnShutdown 已进入 scope，但 JobScheduler/raw pool 无语义 | policy 产生 cancel/ack/too-late/finished receipt |
| RTASK-P1-25 | Open | TaskPollBudget 无 production consumer | main-thread executor 按 frame budget tick 并报 age/deferred |
| RTASK-P1-26 | Open | JobHandle 无稳定 TaskId/generation | handle/event/diagnostics/shutdown 使用同一 qualified identity |
| RTASK-P1-27 | Open | handle 无 owner/scope/scheduler identity | TaskHandle<T> 携 owner，跨 owner 需显式 bridge |
| RTASK-P1-28 | Open | tracked task 只能 ()，业务结果另走 Mutex/channel/map | bounded typed result 或 result locator |
| RTASK-P1-29 | Open | detached spawn 返回 ()，失败不可观察 | detach 显式 sink、retention、shutdown policy；fatal 单独 API |
| RTASK-P1-30 | Partial | TaskGraph queued cancel 可用，running 只有 cooperative 标记 | token 与 scheduler state 集成，terminal 有 ack |
| RTASK-P1-31 | Open | handle Drop 与 framework CancelOnDrop 语义矛盾 | descriptor 决定 policy，危险 detach 可审计 |
| RTASK-P1-32 | Open | descriptor 无 start/completion deadline | timer 只 wake，deadline 属于 task contract |
| RTASK-P1-33 | Partial | panic message/terminal observer 已有局部处理，但状态没有 typed panic/dependency/deadline | Succeeded/Cancelled/Deadline/DependencyFailed/Panicked/Rejected |
| RTASK-P1-34 | Open | detached panic 可触发 Rayon 进程终止，tracked panic 被隔离 | 普通 panic 隔离，显式 fatal policy 才升级 |
| RTASK-P1-35 | Open | failure 丢失 backtrace、owner、stage、worker correlation | bounded FailureRecord 与 trace correlation |
| RTASK-P1-36 | Open | 无 retry/checkpoint/idempotence/cleanup owner | descriptor 声明 retry/transaction artifact，非幂等默认不重试 |
| RTASK-P1-37 | Partial | bounded lanes 有 wait_until，JobHandle::wait 仍无界 | typed wait outcome、timeout、unfinished blocker report |
| RTASK-P1-38 | Open | wait 不接受 scope cancellation | wait 可被 scope/deadline 中断并区分 timeout/terminal |
| RTASK-P1-39 | Open | schedule_after 不验证 cross-owner/lifetime | edge admission 验证 identity、retirement、bridge |
| RTASK-P1-40 | Open | dependency graph 无 cycle detection | 增量 cycle reject 或 frozen forward-only DAG |
| RTASK-P1-41 | Open | dependency panic 被压为 child panic string | 保留上游 TaskId/terminal 与 skip/fallback/run policy |
| RTASK-P1-42 | Open | combine 借首个 dispatcher，混合 owner 语义随机 | owner-aware JoinSet 与 ordered terminal vector |
| RTASK-P1-43 | Partial | continuation dispatcher 已做 64-run budget 和 panic containment | bounded queue、iterative trampoline、fanout/depth metrics |
| RTASK-P1-44 | Open | observer 不在 task completion barrier，slow observer 会占 worker | controlled observer lane、barrier、failure sink |

### 7.3 Diagnostics 与 timer

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-45 | Partial | diagnostics store/journal 存在，但 CoreRuntime 默认关闭 | profile 显式配置，development 可观测、shipping sampling |
| RTASK-P1-46 | Open | production 没有统一 record/report consumer | frame/runtime snapshot 接入 Editor/profile/status |
| RTASK-P1-47 | Open | enable 后无 enabled_at/reset generation/sample window | report 带窗口和 generation |
| RTASK-P1-48 | Open | aggregate 缺 TaskId/scope/owner/domain/priority/label | bounded label registry、scope aggregation、top offenders |
| RTASK-P1-49 | Open | 只有总毫秒/samples，没有 p50/p95/p99/max/deadline lateness | per-domain bounded histogram 与采样成本 |
| RTASK-P1-50 | Open | 无 rejected/quota/steal/park/utilization/capacity/blocking compensation | execution health 全面报告需求、容量、等待、取消、shutdown |
| RTASK-P1-51 | Open | TaskPoolReport 只含声明线程数，排除 timer/private worker | WorkerInventory 成为 shared/dedicated 唯一事实 |
| RTASK-P1-52 | Partial | stable snapshot/retry 已有，但缺 stale/retry_exhausted/age metadata | snapshot 携 captured_at、stale、age、retry outcome |
| RTASK-P1-53 | Open | 无 DAG parent/child、scope lifetime、critical path/trace span | 可按需重建 dependency、queue、execution、retirement |
| RTASK-P1-54 | Open | 固定 512 process timer，单 owner 可耗尽全局 capacity | per runtime/scope quota 的 DeadlineService |
| RTASK-P1-55 | Open | registration 无 TaskId/owner/priority/purpose/shutdown policy | owner-qualified registration 与批量 retirement |
| RTASK-P1-56 | Partial | timer 线程只排序并投递 callback，已有 callback dispatcher | timer wake 与 target lane 完全分离，业务不在 control thread |
| RTASK-P1-57 | Open | callback panic 被 catch 后无统一诊断 | typed callback outcome 关联 registration/owner |
| RTASK-P1-58 | Open | cancel 不等待 in-flight callback | removed/in-flight/completed receipt 与 callback lease |
| RTASK-P1-59 | Open | interval 用 now + interval，无 miss policy | fixed-rate/fixed-delay/coalesce/skip/catch-up 明确化 |
| RTASK-P1-60 | Open | 无 scheduled/fired/late/callback wall/queue delay metrics | lateness histogram、miss count、slow callback |
| RTASK-P1-61 | Open | explicit timer 不入 WorkerInventory | thread identity 含 runtime/domain generation 并纳入 shutdown |

### 7.4 Bounded keyed lane

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-62 | Partial | 单 pump 串行所有 key，已有 fence/coalescing | 可配并发；per-key 串行、cross-key 公平 |
| RTASK-P1-63 | Open | work 结果固定 ()/static failure code | BoundedOperationLane<K,T,E> 保存 bounded result |
| RTASK-P1-64 | Open | deadline 只检查 before-start | start/completion deadline 分离并报告 ack |
| RTASK-P1-65 | Open | active work 无 CancellationContext，不能中断 | backend 声明可中断性和 safe commit fence |
| RTASK-P1-66 | Open | ticket/epoch 使用 saturating increment | checked exhaustion 关闭 lane，identity 不复用 |
| RTASK-P1-67 | Open | pump 外围 panic 可能遗留 reservation | reconciliation guard，fail-closed 或安全 restart |
| RTASK-P1-68 | Open | shutdown guard Drop 无界等待，self-wait 风险 | Drop 非阻塞，显式 deadline shutdown/self-wait 检测 |
| RTASK-P1-69 | Open | 每 entry 只有一个 observer 槽 | bounded multi-subscriber/cursor 或 result journal |
| RTASK-P1-70 | Open | observer 同步执行，panic 静默 | controlled lane 记录 wall/panic/slow consumer |
| RTASK-P1-71 | Open | lane diagnostics 不进 DiagnosticStore | 注册 ExecutionDiagnostics，低基数 top-N key sample |
| RTASK-P1-72 | Open | 多处 poison lock into_inner 后继续 admission | accounting journal 校验，Poisoned 后关闭新提交 |

## 8. P2 一致性、可维护性与资格差距

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RTASK-P2-01 | Open | AsyncTaskHandle 是可构造的裸 u64，默认 0 合法 | 不可空、owner-qualified、checked sequence |
| RTASK-P2-02 | Open | failure message/label 无 bytes、key、redaction policy | bounded attachment，presentation 层本地化 |
| RTASK-P2-03 | Open | counters 饱和不暴露 overflow | report overflow bit，identity checked exhaustion |
| RTASK-P2-04 | Partial | thread assignment 已 clamp，但输入错误仍有 assert/panic 路径 | typed validation error 与 provenance |
| RTASK-P2-05 | Open | pool 线程名前缀可复用，crash dump 难归 owner | 名称含 domain short id，registry 查询完整 owner |
| RTASK-P2-06 | Open | descriptor 没有 schema version/source | versioned config snapshot 与来源 |
| RTASK-P2-07 | Open | owner equality 只比较 Rayon Arc 指针 | 结构化 owner identity，指针只作内部优化 |
| RTASK-P2-08 | Open | JobHandle Debug 只显示 complete | bounded id/owner/state/dependency 摘要 |
| RTASK-P2-09 | Open | timer deadline overflow unwrap_or(now) 变成立即触发 | 返回 DeadlineOutOfRange |
| RTASK-P2-10 | Open | lane queue_entries 实际是 reservations | 分开 reserved/suspended/queued/active/retained |
| RTASK-P2-11 | Open | Duration/counter 饱和无标记 | windowed metric 与 overflow/staleness metadata |
| RTASK-P2-12 | Open | observer bounded 只存在注释 | typed queue policy 与资格测试 |
| RTASK-P2-13 | Partial | job_handle/timer 测试文件接近/超过 owner size 阈值 | 按 contract folder 拆测试，减少 source-string coupling |
| RTASK-P2-14 | Open | Runtime11 mirror 以字符串/anchor 自证，遗漏新指标仍可通过 | compiler boundary + WorkerInventory，mirror 仅辅助 |
| RTASK-P2-15 | Open | pressure test 缺 RSS、allocator、tail latency、fairness baseline | 固定机器/构建/采样与 artifact |
| RTASK-P2-16 | Open | detached panic child test 固化危险进程终止 | typed fatal policy，普通 panic 隔离 |
| RTASK-P2-17 | Open | 无 loom/shuttle、sanitizer、fault injection、poison matrix | 并发状态模型与平台资格测试 |
| RTASK-P2-18 | Open | 无跨 OS/CPU/DLL reload/100h soak 证据 | 结论绑定环境、阈值和可复验 artifact |

## 9. 目标架构与硬切顺序

~~~text
Host / Runtime Session
  -> ExecutionRuntime
       -> WorkerInventory
       -> WorkerDomain { Cpu, BlockingIo, Async, MainThread, Control }
       -> DeadlineService
       -> FailureSink / ExecutionDiagnostics
       -> TaskScope { runtime, module, plugin, world, subsystem, operation }
            -> TaskDescriptor { id, owner, generation, domain, priority, deadline, policy }
            -> Task<T, E> / CancellationContext / DependencyGraph / JoinSet
            -> BoundedOperationLane<K, T, E>
            -> DedicatedWorkerLease
~~~

必须保留 zircon_app + zircon_runtime + zircon_editor crate 边界。ExecutionRuntime 是唯一生产创建者；process_default、TaskPool::new、裸 JobScheduler::spawn/install/join、未登记 spawn_named_thread 只能在 executor 内部或 test fixture。旧 AsyncTaskDescriptor 与新 descriptor 不得并存，不使用 shim/re-export 延长旧路径寿命。

固定 shutdown transaction：

~~~text
Running
 -> ClosingAdmission
 -> CancelQueued
 -> DrainRunningAndCallbacks
 -> StopTimerAndPrivateWorkers
 -> JoinWithDeadline
 -> ReleaseModuleAndCodeLeases
 -> Stopped / TeardownIncomplete
~~~

任何 TeardownIncomplete、unaccounted worker、callback queue 非空、panic worker 或 stale generation 都必须阻止 dynamic library unload。Drop 只能 fail-closed，不能隐式无限等待。

## 10. 分阶段重构计划

| Milestone | 目标 | 退出条件 |
|---|---|---|
| M0 | 冻结所有 pool/scheduler/timer/private-thread caller | 生产调用矩阵、WorkerInventory 和父 P0 owner 可机械复核 |
| M1 | ExecutionRuntime + WorkerDomain + WorkerInventory | 构造失败回滚；configured/created/live/busy/retired 守恒 |
| M2 | TaskScope + TaskDescriptor + Task<T,E> hard cut | owner/generation/result/cancel/deadline/drop/shutdown 统一 |
| M3 | dependency/JoinSet/continuation/data-parallel convergence | cross-owner/cycle reject；100k chain/fanout/single-worker wait 通过 |
| M4 | DeadlineService 与 bounded operation convergence | timer 只 wake；active cancel、typed result、pump reconciliation 完成 |
| M5 | Asset/Scene/Plugin/Preferences/VM/Text/Graphics/App/Editor adoption | 无 production raw pool/process alias/direct thread 旁路，专用线程有 lease |
| M6 | diagnostics/trace/product bridge | owner/domain/priority/failure/histogram/shutdown blocker 可查询 |
| M7 | fault/model/cross-platform/soak/benchmark | artifact 冻结环境与阈值，才可谈性能比较 |

首个实现切片必须从 M0/M1 开始。只给 JobHandle 增加 cancel/priority 字段而不删除 process owner、OperationService 第二状态机和 private-thread 旁路，不能改变本报告判定。

## 11. 验收门禁

| Gate | 状态 | 验收条件 |
|---|---|---|
| RTASK-G01 | Fail | 每个 task/timer/worker/private thread 可查询 runtime/scope/module/plugin owner |
| RTASK-G02 | Fail | TasksModule activate/deactivate 返回 ready/quiescence receipt |
| RTASK-G03 | Fail | dynamic destroy 在未退出 task/callback/worker 时 fail-closed |
| RTASK-G04 | Fail | ExecutionRuntime 构造失败 typed 返回并 join 已创建 worker |
| RTASK-G05 | Fail | close admission 后 process alias/private lane/clone 拒绝新工作 |
| RTASK-G06 | Fail | shutdown 按 policy 处理并列出 unfinished TaskId/owner |
| RTASK-G07 | Fail | shared/dedicated worker 与 WorkerInventory 严格守恒 |
| RTASK-G08 | Fail | 无未登记 production TaskPool::new/JobScheduler::default/direct thread |
| RTASK-G09 | Fail | framework descriptor/status 与 runtime task 是同一状态机 |
| RTASK-G10 | Fail | Task<T,E> typed terminal，普通 panic 不终止进程 |
| RTASK-G11 | Fail | Drop/cancel/shutdown policy 可测且返回 ack |
| RTASK-G12 | Fail | start/completion deadline 与 observer timeout 可区分 |
| RTASK-G13 | Fail | cross-owner dependency 无 bridge 即拒绝，cycle admission 失败 |
| RTASK-G14 | Fail | dependency failure 保留上游 TaskId 与 terminal class |
| RTASK-G15 | Fail | 100,000 深链/宽 fan-out 无栈溢出和单 worker 长占用 |
| RTASK-G16 | Fail | wait_until/JoinSet/late observer/result retention 有 race/timeout 测试 |
| RTASK-G17 | Fail | 1/2/4/8/32/64 核 worker 不超预算且低核可前进 |
| RTASK-G18 | Fail | priority 贯穿 runtime/editor queue，ageing/inversion 有测试 |
| RTASK-G19 | Fail | per-scope entry/byte/cost quota 并发守恒 |
| RTASK-G20 | Fail | CPU/blocking I/O/main-thread/control work 不能互相耗尽 |
| RTASK-G21 | Fail | nested parallel、worker wait 单 worker/跨 domain 无死锁 |
| RTASK-G22 | Fail | ECS/Graphics data hazard 形成可验证 DAG |
| RTASK-G23 | Fail | benchmark 输出 throughput、queue p95/p99、steal、utilization、allocation |
| RTASK-G24 | Fail | dedicated worker 有 QoS/affinity/stack 与完整 join 证据 |
| RTASK-G25 | Fail | timer capacity 按 scope 隔离，owner storm 不饿死其他 owner |
| RTASK-G26 | Fail | timer 不执行无界业务 callback，slow/panic 有 terminal/diagnostic |
| RTASK-G27 | Fail | cancel registration 可等待 in-flight callback lease 归零 |
| RTASK-G28 | Fail | fixed-rate/fixed-delay/coalesce/skip 有 fake-clock 测试 |
| RTASK-G29 | Fail | bounded operation 支持 per-key 串行、cross-key 并发、typed result、active cancel |
| RTASK-G30 | Fail | pump 任意阶段 panic 后守恒恢复或 lane fail-closed |
| RTASK-G31 | Fail | shutdown guard 同 worker/timeout/Drop 不死锁或无界等待 |
| RTASK-G32 | Fail | execution snapshot 带 generation/window/stale 并进入真实 DiagnosticStore consumer |
| RTASK-G33 | Fail | Asset/Scene/Plugin/Text/Graphics/App/Editor adoption 无旁路 |
| RTASK-G34 | Fail | operation/plugin/script panic 不杀 Editor 且关联 owner |
| RTASK-G35 | Fail | Editor close 对 unfinished task 提供 retry/wait/force 且保持依赖存活 |
| RTASK-G36 | Fail | App/Editor/headless/commandlet 共用 shutdown protocol |
| RTASK-G37 | Fail | dynamic load/run/unload 10,000 轮无旧 callback/worker 进入卸载代码 |
| RTASK-G38 | Fail | 100h soak 无 task/result/timer/observer/thread 增长且 identity 不复用 |
| RTASK-G39 | Fail | model/sanitizer/helgrind 覆盖 handle/lane/timer/shutdown 状态机 |
| RTASK-G40 | Fail | 与 Unreal/Bevy/Godot 同语义 benchmark 满足预冻结阈值 |

## 12. Owner 路由与未执行项

- Runtime11 继续拥有 job-system 实施；Runtime02/158 继续拥有 event、timer 与 dynamic unload 父边界；Runtime192 只负责当前 task execution 差异和跨 consumer 重构顺序。
- Runtime24 提供全仓 identity/generation 规则；Runtime192 要求其覆盖 TaskId、ScopeId、WorkerDomainId、TimerRegistrationId 和 operation handle。
- Runtime01/46 拥有 module/service 总生命周期；Runtime192 要求 TasksModule 成为真实 execution owner，而不是 descriptor catalog。
- Editor09/Editor256 拥有 Editor async operation 与 close UX；Runtime192 提供 typed task terminal、quiescence receipt 和 private worker inventory。
- Graphics/Text/Asset/Scene/Plugin/App/Network 的业务报告负责迁移各自调用点；它们不能各自定义第二套 worker budget 或 shutdown semantics。
- Tooling 按用户要求排除，不把 tooling 迁移或 Python/source-scan 设计混入本报告。

本轮完成了静态源码逐文件复核、生产调用点核对、参考源码比对、稳定 finding ID 重判和 40 项门禁登记。未运行 Cargo、Editor host/UI automation、GPU/IO fault、DLL reload、cross-platform benchmark、allocator/RSS、sanitizer、soak 或协调器状态查询；这些是后续实现切片的验收工作，不得由本报告的静态结论代替。
