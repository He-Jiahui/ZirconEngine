---
title: Runtime Task Execution、Job Scheduler、Handle、Dependency、Cancellation、Thread Budget、Timer、Shutdown、Diagnostics 与 Product Integration 当前源码复核
category: zircon_runtime
report_id: Runtime114
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
refreshes_report: Runtime59
related_code:
  - zircon_runtime/src/core/framework/tasks
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/runtime/modules/tasks.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/operation/service.rs
  - zircon_runtime/src/asset
  - zircon_runtime/src/graphics/pipeline/async_compile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/pipelined/queue.rs
  - zircon_runtime/src/platform/preferences
  - zircon_runtime/src/scene
  - zircon_runtime/src/script/vm
  - zircon_runtime/src/text
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/development_watch.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/background_load.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
tests:
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/tests.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/tests
  - zircon_runtime/src/tests/runtime_absorption/job_system
  - zircon_editor/src/core/jobs/tests
  - zircon_editor/src/core/jobs/system/pending/tests
plan_sources:
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/zircon_runtime/core/job_system.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Task.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/TaskPrivate.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/TaskConcurrencyLimiter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Pipe.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/ManualPipe.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Tasks/TaskPrivate.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Tasks/TaskConcurrencyLimiter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/Fundamental/Scheduler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/Fundamental/Scheduler.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/Fundamental/Task.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/Fundamental/LocalQueue.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/Fundamental/WaitingQueue.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/Fundamental/Oversubscription.h
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - dev/bevy/crates/bevy_tasks/src/slice.rs
  - dev/bevy/crates/bevy_tasks/src/thread_executor.rs
  - dev/bevy/crates/bevy_tasks/src/executor.rs
  - dev/bevy/crates/bevy_tasks/src/edge_executor.rs
  - dev/bevy/crates/bevy_tasks/src/single_threaded_task_pool.rs
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/godot/core/object/message_queue.h
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/task.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Utilities/JaggedJobRange.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Utilities/ParallelSortExtensions.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99o · Runtime Task Execution 当前源码复核

## 1. 当前结论

Runtime59 的结论在当前源码上仍成立：Zircon 已有可保留的局部任务底座，但没有形成可承担动态 runtime、Editor、App、plugin、world 与 subsystem 生命周期的工程级 execution runtime。三类 Rayon pool、dependency continuation、worker wait-assist、panic terminalization、64-shard diagnostics、bounded keyed I/O lane，以及 EditorJobSystem 的 admission/priority/mutex/progress/cancel 都是真实实现；问题不在“完全没有功能”，而在 identity、owner、scope、admission、typed result、cancel acknowledgement、deadline、worker inventory、shutdown barrier 与产品观测没有收敛为同一个合同。

当前最严重的结构事实没有变化：`CoreRuntime`在模块激活前取得进程级`OnceLock<TaskPools>`；`TasksModule`仍只是descriptor；scheduler/pool clone没有runtime或module generation；process timer永久存活；dynamic session destroy使用零drain timeout，且不关闭task admission、不枚举task/timer/private worker、不join execution worker。EditorJobSystem 可以在业务层请求 cooperative cancel 并报告 unfinished jobs，但底层没有 scope quiescence receipt，因此不能证明卸载代码已经无人执行。

本轮账本保持 **0 项新增 P0、72 项 P1 Open、18 项 P2 Open、40 项 Gate Fail**。两项父P0仍分别由 Runtime02 与 Editor09 唯一计数：task/timer worker越过dynamic library unload，以及Editor到达deadline后仍拆卸live job依赖。新增的execution wall totals、bounded lane线性coalescing与Editor admission改进均为真实进步，但都没有闭合任一 finding 的完整验收条件，故不标记为Partial或Closed。

本轮只做review与计划记录，没有修改production、tests、Cargo、ABI或参考源码，也没有运行Cargo、动态库卸载、Editor真实关闭、压力/故障/soak、sanitizer、profiler或同负载跨引擎benchmark。当前没有证据可以宣称任务系统性能或稳定性达到、更不能宣称超过Unreal。用户要求暂缓tooling，本报告不安排脚本、生成器或现有工具优化；未来迁Rust由独立计划处理。

## 2. 当前源码冻结与可复现性

| 范围 | 文件 / 行 / 非空行 / bytes / `#[test]` | fingerprint / 选择规则 |
|---|---:|---|
| task core与Core接线 | **36 / 6,167 / 5,531 / 201,308 / 45** | `e7458fce0d1cd3da5705dce9fded72c27ebbd56336953d96450643332212b416`；framework tasks、runtime tasks及四个Core owner文件 |
| product consumer与私有worker语义扫描 | **170 / 54,858 / 49,997 / 1,940,112 / 293** | `c573e6ad2d4af923e5301df7c95b975c52ddb0e363b05555ddf297b99a999ef1`；Runtime/Editor/App/plugins production Rust中命中task pool、scheduler、timer、bounded lane、thread或join语义，排除显式test路径，保留同文件inline tests |
| focused behavior/source-shape tests | **27 / 7,057 / 6,433 / 238,450 / 142** | `8654ba155073a0b1593447908aedad18699a9c8dcaafe01dad856ab78b704142`；task、bounded lane、JobSystem absorption与Editor jobs测试 |
| 五引擎显式参考 | **31 / 13,120 / 11,234 / 504,735 / 18** | `cc65d9e4d228ea51761d6582e0c5b4035e67e83c66c50c80a4176a5f8d27fdb1` |

fingerprint算法为：仓库相对路径转`/`并排序去重；每个文件计算lowercase SHA-256；以`path|hash`按LF连接、末尾不追加LF；再对UTF-8 payload计算SHA-256。行数使用物理行，非空行使用trim后非空计数。consumer组的大小写不敏感选择器为`TaskPool|JobScheduler|TaskTimer|AsyncTaskDescriptor|TaskPollBudget|BoundedKeyedIo|EditorJobSystem|std::thread|thread::Builder|thread::spawn|JoinHandle`，扫描`zircon_runtime/src`、`zircon_editor/src`、`zircon_app/src`与`zircon_plugins`，排除路径段`tests/test`、文件`tests.rs/test_support.rs`，保留production文件中的inline tests。冻结集证明本轮实际纳入的当前源码，不把机械命中虚构为每个文件同等深度的语义阅读；深读集中于task core、timer、bounded lane、dynamic shutdown、Editor jobs、代表性private workers及对应测试。

基线HEAD为`bee4c707b714738346b49bba15c59468b8bd9b39`，coordinator epoch为339。报告读取当前共享working tree；其中bounded lane coalescing含其他会话尚未提交的线性分区改动，本轮只评审，不接管、不回退。`source_recheck_required`保持true。

## 3. Runtime59 后的真实变化

| 变化 | 当前证据 | 账本结论 |
|---|---|---|
| execution wall metric补齐 | scheduler增加`tasks.execution_samples`与`tasks.execution_ms`，进入sharded aggregate | 只有累计总量和样本数，无分位、max、deadline、owner、task或production consumer；相关diagnostics finding仍Open |
| bounded lane coalescing降复杂度 | 当前working tree将matching-entry提取改为线性partition，并有ignored benchmark | 改善一条热点，不改变单pump串行、typed result、active cancel、pump reconciliation、observer与shutdown合同；lane findings仍Open |
| Editor job业务层增强 | 已有entry/byte/age/category quota、priority pending选择、mutex group、key merge、batch reservation、progress、cooperative cancel与unfinished report | priority不进入runtime worker queue；terminal仍走外部channel；shutdown不拥有runtime scope或worker quiescence；Runtime substrate与Editor父P0均未闭合 |
| JobSystem结构镜像继续自报无风险 | Runtime11与模块文档写`diagnostic_anchor_count = 11`、`risks = []` | 新execution metric未进入该anchor inventory；字符串/计数守卫只证明镜像一致，不能证明产品完整性或线程归属 |
| private worker面进一步确认 | Graphics async compile/render submit、Text raster、Asset watch、Editor plugin watch/play/export output、App background load、Navigation bake等均有独立线程或pool | 多数局部实现有bounded queue和Drop join，应保留；但都未进入全局budget、scope census、WorkerInventory和DLL unload barrier |

## 4. 当前真实产品链

```text
CoreRuntime::new
  -> TaskPools::default -> PROCESS_TASK_POOLS: OnceLock<TaskPools>
  -> JobScheduler::from_pool(compute.clone())
  -> later register/activate TasksModule descriptor

consumer submission
  -> raw TaskPool spawn/install/join
  -> JobScheduler spawn/schedule/schedule_after
  -> process_io/process_compute aliases
  -> subsystem-owned OS thread / dedicated TaskPool

dynamic session destroy
  -> zero task drain timeout
  -> event/watch/module/log cleanup
  -> no execution admission close, task/timer census or worker join
```

`JobScheduler::schedule`返回无TaskId、owner、scope、cancel、deadline、priority或typed result的`JobHandle`。`schedule_after`避免占用worker等待依赖，这是正确底座；但依赖可以跨任意scheduler，cycle与owner不验证，dependency failure被压成panic字符串。terminal continuation与observer同步在线程内执行，`wait()`无限等待且不等待observer完成。`spawn`返回`()`；detached panic沿Rayon fatal路径终止进程，测试还把该行为固化为预期。

Timer是固定512 registration的process singleton，单线程直接执行callback；panic被静默吞掉，cancel不能等待in-flight callback，interval使用`now + interval`而漂移。Bounded keyed lane拥有entry/byte budget、key coalescing、fence、deadline-before-start、cancel-before-start和terminal ticket，但所有key仍由一个pump串行执行，work/result未typed，active operation不可取消，pump外围panic无reconciliation guard，Drop可无限等待。

## 5. 可保留底座

- 保留三类work domain的概念，但由`ExecutionRuntime`统一预算和生命周期，不保留任意production构造pool的能力。
- 保留dependency continuation不占worker等待、worker wait-assist、panic terminal publication与diagnostic sharding。
- 保留bounded keyed lane的entry/byte reservation、key generation coalescing、fence、terminal ticket和守恒测试。
- 保留Graphics/Text/Asset/Editor私有worker中的bounded queue、明确stop signal、Drop join、panic containment等局部工程性；迁移时把它们登记为shared lane或`DedicatedWorkerLease`，而不是重写成更弱实现。
- 保留EditorJobSystem的业务admission、priority/mutex/keyed merge/progress语义；runtime必须承接而不是复制这些语义。

## 6. 继承 P0 与所有权

Runtime02继续唯一拥有“process task/timer worker越过dynamic session与DLL unload”P0。当前新增证据仍是TasksModule无lifecycle、process pools/timer永久owner、dynamic shutdown零drain timeout、私有worker不进统一census。Runtime114的M0-M2与G01-G08是task侧关闭条件，不重复计数。

Editor09继续唯一拥有“Editor deadline到达后仍拆卸live job依赖”P0。EditorJobSystem能够返回unfinished jobs，却无法关闭runtime scope admission、获得cancel acknowledgement或证明worker quiescence。Runtime114提供底层scope/receipt；Editor09负责产品的retry/wait/force policy，不重复计数。

## 7. P1 工程化差距总账

### 7.1 Execution owner、module 与 shutdown

| ID | Status | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-01 | Open | `CoreRuntime::new`固定取得process pool，不能注入host owner或表达构造失败。 | `CoreRuntime::try_new(ExecutionRuntimeHandle)`或明确runtime-owned构造，失败返回创建与回滚receipt。 |
| RTASK-P1-02 | Open | `TasksModule`只有descriptor，activate/deactivate不改变execution状态。 | module发布service generation；activate给ready receipt，deactivate关闭admission并给quiescence receipt。 |
| RTASK-P1-03 | Open | `process_default/process_io`允许任意子系统绕过session/module owner。 | process域仅由host创建；consumer领取带scope与policy的lane capability。 |
| RTASK-P1-04 | Open | `JobScheduler::default`可按全部CPU另建compute pool。 | 删除production default；fixture显式建isolated executor，产品只解析owner capability。 |
| RTASK-P1-05 | Open | 公共`TaskPool::new`已被Graphics、Navigation等production caller使用。 | `ExecutionRuntime`统一分配domain；专用pool必须经`DedicatedWorkerLease`。 |
| RTASK-P1-06 | Open | scheduler/pool clone无runtime/module generation，可越过owner卸载提交。 | handle携`ExecutionRuntimeId + ScopeId + Generation`，stale/closed提交结构化拒绝。 |
| RTASK-P1-07 | Open | 没有session/module/plugin/world/subsystem `TaskScope`。 | scope维护admission、task census、cancel policy与retirement blocker。 |
| RTASK-P1-08 | Open | `TaskPool`没有close/drain/cancel/stop/显式join。 | `close -> cancel -> drain -> stop -> join`分阶段协议，每阶段有deadline与blocker report。 |
| RTASK-P1-09 | Open | Graphics、Text、watch、log、render、Editor/App等私有线程不在统一supervisor。 | 所有专用线程登记owner、entry、stack/QoS、stop、join与DLL code lease。 |
| RTASK-P1-10 | Open | worker无统一start/stop hook。 | WorkerDomain声明profiling、allocator、TLS、affinity与teardown hook，失败可回滚。 |

### 7.2 Admission、线程预算与 executor policy

| ID | Status | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-11 | Open | 每类pool最少1 worker；`total_threads=2`仍可创建3个worker。 | budget solver验证可满足性，实际worker conservation等于report总额或显式dedicated reservation。 |
| RTASK-P1-12 | Open | Text把async-compute预算数再次用于专用OS worker。 | 区分shared slots与dedicated reservations，同一预算不可重复消费。 |
| RTASK-P1-13 | Open | Graphics public constructor可另建完整compute pool。 | renderer领取共享render lane；device thread单独登记且有上限。 |
| RTASK-P1-14 | Open | Asset/scene/preference/VM默认构造静默抓process I/O pool。 | 构造器接收scope/lane；仅fixture helper可建isolated owner。 |
| RTASK-P1-15 | Open | Editor priority仅影响pending选择，runtime queue全同级。 | stable priority贯穿admission、queue和worker，含ageing与priority inversion诊断。 |
| RTASK-P1-16 | Open | descriptor无affinity、QoS、stack、latency/background或oversubscription policy。 | WorkerDomain提供平台验证配置与降级receipt。 |
| RTASK-P1-17 | Open | generic schedule/spawn无entry/byte/cost admission，不能返回Full/Closed/Quota。 | 所有提交先quote并原子reserve，返回typed admission与rollback lease。 |
| RTASK-P1-18 | Open | 无per-scope/plugin/world quota。 | process到lane的分层预算，可borrow但保留硬上限。 |
| RTASK-P1-19 | Open | 无topology、NUMA、locality、pinning、steal或blocking compensation政策。 | 建立可测worker topology与task class后再制定平台策略。 |
| RTASK-P1-20 | Open | raw `spawn/install/join/in_place_scope`绕过handle、diagnostics与scope census。 | raw pool API收窄至executor内部，公共入口统一生成task record与terminal receipt。 |
| RTASK-P1-21 | Open | `parallel_for`只有手工chunk size和blocking closure。 | data-parallel plan按work estimate/load切分，支持cancel、priority、typed reduction与nested policy。 |
| RTASK-P1-22 | Open | generic executor只有`FnOnce`，没有future/main-thread/thread-bound/blocking隔离。 | 明确CPU、blocking I/O、async future、main-thread continuation四类合同。 |

### 7.3 Task identity、result、cancel、dependency 与 completion

| ID | Status | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-23 | Open | framework descriptor/status与runtime scheduler是两套状态机。 | canonical `TaskDescriptor`由scheduler admission消费，成为status/trace/shutdown唯一来源。 |
| RTASK-P1-24 | Open | `CancelOnDrop/DetachOnDrop/FinishOnShutdown`只有enum，无执行语义。 | Task Drop和scope shutdown执行policy并返回cancel/ack/too-late/finished receipt。 |
| RTASK-P1-25 | Open | `TaskPollBudget`无production consumer。 | MainThreadExecutor按frame budget轮询并报告deferred、age、deadline miss与starvation。 |
| RTASK-P1-26 | Open | JobHandle无TaskId，日志、trace与shutdown blocker无法定位。 | generation-qualified `TaskId`从handle、event、diagnostics一致可查且不复用。 |
| RTASK-P1-27 | Open | handle无owner/scope/scheduler identity。 | `TaskHandle<T>`携owner identity，cross-owner dependency需显式bridge。 |
| RTASK-P1-28 | Open | tracked task只能返回`()`，业务另建Mutex/channel/map。 | `Task<T,E>`保存typed terminal或bounded result locator，支持take/late observer。 |
| RTASK-P1-29 | Open | detached `spawn`返回`()`，无admission、id、terminal或failure receipt。 | 默认spawn返回task；detach显式声明owner、failure sink、retention与shutdown policy。 |
| RTASK-P1-30 | Open | JobHandle无cancel API，queued/running/dependency-waiting无法统一请求取消。 | token与scheduler state集成；queued可retract，running cooperative，terminal有ack。 |
| RTASK-P1-31 | Open | handle Drop隐式detach，与framework默认CancelOnDrop矛盾。 | Drop policy来自descriptor且危险detach在调用处可见可审计。 |
| RTASK-P1-32 | Open | generic task无start/completion deadline，lane只能外置timer。 | descriptor持两类deadline与timeout policy；timer只负责wake。 |
| RTASK-P1-33 | Open | panic压成字符串，wait重新panic，不能typed恢复。 | terminal区分Succeeded/Cancelled/Deadline/DependencyFailed/Panicked/Rejected。 |
| RTASK-P1-34 | Open | detached panic终止进程，而tracked API隔离panic，故障域互相矛盾。 | panic policy显式；普通task隔离上报，仅host批准fatal task可终止。 |
| RTASK-P1-35 | Open | 非字符串panic丢失backtrace、task、owner、stage。 | FailureRecord保存分类、摘要、backtrace policy、owner、worker与trace correlation。 |
| RTASK-P1-36 | Open | 无attempt/retry/checkpoint/idempotence/cleanup owner。 | descriptor可声明retry/transaction artifact，默认不重试非幂等工作。 |
| RTASK-P1-37 | Open | `wait()`只有无限等待，shutdown无法得到unfinished原因。 | `wait_until/try_join`返回typed wait outcome；产品Drop不得偷偷无限等待。 |
| RTASK-P1-38 | Open | wait不接收scope cancellation。 | wait可被scope/deadline中断，同时严格区分wait timeout与task terminal。 |
| RTASK-P1-39 | Open | `schedule_after`接受任意scheduler handles，无owner/executor/lifetime验证。 | edge验证identity、retirement与允许的cross-domain transition。 |
| RTASK-P1-40 | Open | dependency graph无cycle检测。 | admission增量检测cycle，或使用只向前引用的frozen DAG builder。 |
| RTASK-P1-41 | Open | dependency panic把child压成相同panic字符串。 | `DependencyFailed`保留上游TaskId与terminal，policy明确skip/fallback/run。 |
| RTASK-P1-42 | Open | `combine`借首个handle diagnostics，混合scheduler归属随机且只保留首个panic。 | JoinSet有独立owner与ordered terminal vector，跨scope组合显式声明。 |
| RTASK-P1-43 | Open | terminal同步递归发布continuation，宽fanout/深chain放大worker latency与stack。 | bounded continuation queue或迭代trampoline，记录fanout/depth/dispatch wall。 |
| RTASK-P1-44 | Open | observer同步运行；wait不等observer，slow observer阻塞worker，panic仅局部计数。 | observer进入受控lane，completion barrier明确，panic汇入统一failure sink。 |

### 7.4 Diagnostics、trace 与 product truth

| ID | Status | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-45 | Open | diagnostics默认关闭，CoreRuntime未构造`with_diagnostics`。 | profile显式配置；development默认可观测，shipping可sampling。 |
| RTASK-P1-46 | Open | production无`record_diagnostics/diagnostic_report` consumer。 | frame diagnostics采样execution owner并进入Editor/profile/status snapshot。 |
| RTASK-P1-47 | Open | enable后只有无generation的部分累计值。 | report带enabled_at、reset_generation与sample window。 |
| RTASK-P1-48 | Open | 指标仅scheduler aggregate，无TaskId/scope/owner/domain/priority/label/failure class。 | bounded label registry与scope aggregation，提供top offenders。 |
| RTASK-P1-49 | Open | 新execution metric仍只有总毫秒与samples，无p50/p95/p99/max/deadline lateness。 | per-domain bounded histogram与slow-task sample，并资格化采样成本。 |
| RTASK-P1-50 | Open | 无rejected/quota/steal/park/utilization/capacity/saturation/blocking compensation。 | execution health同时报告需求、容量、排队、执行、等待、取消和shutdown阶段。 |
| RTASK-P1-51 | Open | TaskPoolReport只含声明线程数，不含真实线程或private worker。 | WorkerInventory成为shared/dedicated线程唯一事实。 |
| RTASK-P1-52 | Open | 聚合重试失败静默返回last stable snapshot。 | snapshot带captured_at、stale、retry_exhausted与age。 |
| RTASK-P1-53 | Open | 无task DAG、parent/child、scope lifetime、critical path或trace span。 | 低成本按需trace可离线重建dependency、queue、execution与retirement blocker。 |

### 7.5 Timer 与 deadline service

| ID | Status | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-54 | Open | 所有runtime共享固定512项process timer，一个owner可耗尽全局capacity。 | DeadlineService按runtime/scope配额，registration返回owner-qualified id。 |
| RTASK-P1-55 | Open | registration无TaskId、owner、priority、purpose或shutdown policy。 | registration绑定scope/target，owner retirement可批量cancel并等quiescence。 |
| RTASK-P1-56 | Open | 单timer线程串行执行任意callback。 | timer线程只维护时序和wake，业务投递control/target lane。 |
| RTASK-P1-57 | Open | callback panic被catch后静默丢弃。 | typed callback outcome与failure sink关联registration/owner。 |
| RTASK-P1-58 | Open | cancel只能移除未取出deadline，不能等待in-flight callback。 | receipt区分removed/in-flight/completed，并可等待callback lease归零。 |
| RTASK-P1-59 | Open | interval按`now + interval`重排而漂移，无miss policy。 | 明确fixed-rate/fixed-delay/coalesce/skip/catch-up及最大补偿。 |
| RTASK-P1-60 | Open | 无scheduled/fired/late/callback wall/queue delay指标。 | lateness histogram、miss count与slow callback offender。 |
| RTASK-P1-61 | Open | explicit timer同名且不进WorkerInventory。 | thread identity含runtime/domain generation并进入统一inventory与shutdown report。 |

### 7.6 Bounded keyed operation lane

| ID | Status | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| RTASK-P1-62 | Open | 每lane一个pump串行执行全部key。 | max concurrency可配置，per-key串行、cross-key公平并发且保留fence。 |
| RTASK-P1-63 | Open | work固定`Result<(), static-code failure>`，typed结果仍外置。 | `BoundedOperationLane<K,T,E>`保存bounded result/error locator。 |
| RTASK-P1-64 | Open | deadline仅worker开始前检查，running work无completion deadline。 | 分开start/completion deadline，运行中发cancel并报告ack。 |
| RTASK-P1-65 | Open | cancel authority只取消before-start，active work无法协作取消。 | closure接收CancellationContext，backend声明可中断性与safe commit fence。 |
| RTASK-P1-66 | Open | ticket id与epoch使用`saturating_add`，上限后可重复或冻结。 | checked exhaustion关闭lane并返回terminal error，identity不复用。 |
| RTASK-P1-67 | Open | 仅work closure被catch；pump外围panic可卡住pump与reservation。 | pump-level reconciliation guard校验守恒并fail-closed或安全restart。 |
| RTASK-P1-68 | Open | shutdown guard Drop无限wait，单worker/self-wait可死锁。 | Drop非阻塞fail-closed；显式owner以deadline shutdown并检测self-wait。 |
| RTASK-P1-69 | Open | 每entry只有一个terminal observer槽，可被替换。 | bounded multi-subscriber/cursor或统一result journal。 |
| RTASK-P1-70 | Open | observer同步运行且panic静默吞掉。 | 受控lane记录dispatch/observer wall、panic与slow-consumer policy。 |
| RTASK-P1-71 | Open | lane diagnostics局部且不进入DiagnosticStore，缺scope/rejection/deadline/fence成本。 | 注册ExecutionDiagnostics，低基数聚合并支持top-N key hash sample。 |
| RTASK-P1-72 | Open | lane/ticket/timer/handle对poison多用`into_inner`继续运行。 | cache可重建；admission/accounting state需journal校验或进入Poisoned并关闭提交。 |

## 8. P2 一致性、可维护性与资格差距

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| RTASK-P2-01 | Open | AsyncTaskHandle是裸`u64`，默认0可构造。 | 不可空、owner-qualified、checked sequence identity。 |
| RTASK-P2-02 | Open | failure message与label无byte cap、message key或redaction。 | bounded diagnostic attachment，UI本地化留在presentation。 |
| RTASK-P2-03 | Open | poll count与counter饱和后不暴露overflow。 | report带overflow bit，关键identity checked exhaustion。 |
| RTASK-P2-04 | Open | thread assignment百分比错误通过assert panic。 | 配置解析返回typed validation error并保留provenance。 |
| RTASK-P2-05 | Open | 独立pool复用线程名前缀，profile/crash dump难辨owner。 | 名称含domain短ID，完整owner由registry查询。 |
| RTASK-P2-06 | Open | pool descriptor无schema version或来源。 | versioned config snapshot记录default/config/CLI/platform provenance。 |
| RTASK-P2-07 | Open | `shares_execution_owner_with`只比较Rayon Arc。 | 返回结构化owner identity，pointer equality仅内部优化。 |
| RTASK-P2-08 | Open | JobHandle Debug只显示complete。 | bounded摘要包含id/owner/state/dependency count，不泄露payload。 |
| RTASK-P2-09 | Open | timer overflow用`unwrap_or(now)`改成立即触发。 | 返回DeadlineOutOfRange，不改变语义。 |
| RTASK-P2-10 | Open | lane `queue_entries`实际是reserved entries。 | 分开reserved/suspended/queued/active/completion-retained。 |
| RTASK-P2-11 | Open | Duration/counter饱和后无标记。 | 窗口化metric与overflow/staleness metadata。 |
| RTASK-P2-12 | Open | “observer必须bounded”等限制只写在注释。 | typed executor/queue policy与资格测试强制预算。 |
| RTASK-P2-13 | Open | standalone task tests集中千行文件，contract ownership混杂。 | 按contract folder拆分，保留behavior而非source-string耦合。 |
| RTASK-P2-14 | Open | Runtime11 mirror以字符串、anchor数量和`risks=[]`自证；遗漏新指标仍可通过。 | compiler capability boundary加runtime WorkerInventory；mirror只作辅助且必须覆盖product behavior。 |
| RTASK-P2-15 | Open | pressure tests无长期RSS、allocator、tail latency或fairness基线。 | 固定机器/平台baseline、noise policy与可复验artifact。 |
| RTASK-P2-16 | Open | detached panic子进程测试固化“应终止”的危险行为。 | typed fatal policy；普通detached panic隔离且可观测。 |
| RTASK-P2-17 | Open | 无loom/shuttle、TSAN/helgrind、fault injection或poison transaction矩阵。 | handle/lane/timer/shutdown模型测试与平台并发资格。 |
| RTASK-P2-18 | Open | 无跨OS/CPU规模/DLL reload/100h soak证据。 | 性能稳定性结论绑定环境、构建、采样与artifact。 |

## 9. 五参考引擎对照与适用边界

| 参考 | 当前源码可证能力 | Zircon必须吸收 | 不应误抄/过度声称 |
|---|---|---|---|
| Unreal | `TTask<T>` typed result、debug name、priority/extended priority、prerequisite graph、Pipe/ManualPipe、TaskEvent、TryRetractAndExecute、timeout wait、concurrency limiter；低层scheduler有foreground/background worker、affinity、stack、oversubscription、local/global queue、steal与完整StopWorkers/join序列 | identity/priority/dependency/concurrency limit、worker inventory、分阶段停机、deadline wait与诊断 | Unreal task cancellation以cooperative为主，文档明确不能把任意已launch task跳过执行；其process-global scheduler也不能直接解决Zircon DLL owner问题 |
| Bevy | TaskPoolBuilder配置thread count/name/stack/start-stop hook；pool拥有executor、shutdown channel和JoinHandle，Drop关闭并join；`Task<T>`有cancel/detach；scoped borrowed task在Scope Drop时取消 | typed result、scope、pool-owned join、thread hooks、main-thread/thread-bound executor与按worker数切片 | Bevy global pools面向常驻app，不能原样跨dynamic library session |
| Godot | TaskID/GroupID、high/low priority、group progress、caller task/group identity、collaborative wait与潜在死锁检测；NORMAL/PRE_EXIT/EXIT runlevel；finish切换退出、join并报告遗留task | group/scope census、退出阶段、协助等待、join后遗留报告、thread enter/exit hook | Godot不是typed Rust result模型；named pool存在但其文档仍偏向singleton |
| Fyrox | 核心pool返回UUID与结果channel；engine handler把结果回投plugin或scene/node/script owner | owner-aware result delivery与产品对象retirement检查 | pool本身对shutdown、capacity、type safety较薄，只能作为最低owner回投参考，不能作为目标架构 |
| Unity Graphics | package job structs使用ReadOnly/WriteOnly/native container access、JobHandle dependency、ScheduleParallel/CombineDependencies、按`JobsUtility.JobWorkerCount`切分，并在render pipeline形成具体DAG | data access声明、resource hazard、批切分、dependency artifact与deferred disposal | 本地只有Graphics package消费者，不是Unity core scheduler源码；不得从中推断worker lifecycle、cancel或全引擎性能 |

## 10. 目标架构与硬切原则

```text
Host / Runtime Session
  -> ExecutionRuntime
       -> WorkerInventory
       -> WorkerDomain { Cpu, BlockingIo, Async, MainThread, Control }
       -> ExecutionDiagnostics / FailureSink
       -> DeadlineService
       -> TaskScope { runtime, module, plugin, world, subsystem, operation }
            -> TaskDescriptor
            -> Task<T, E> / TaskId / CancellationContext
            -> DependencyGraph / JoinSet / ContinuationLane
            -> BoundedOperationLane<K, T, E>
            -> DedicatedWorkerLease
```

固定crate边界保持`zircon_app + zircon_runtime + zircon_editor`，execution spine归`zircon_runtime::core::{runtime,framework,manager,math,resource}`中的runtime/framework owner。不得新建server/facade兼容层，不保留旧`AsyncTaskDescriptor`与新`TaskDescriptor`双轨，也不以`pub use`、shim trait、process singleton alias继续开放旧路径。迁移采用调用点、测试、文档同时硬切；旧路径无production consumer后删除。

## 11. 分阶段重构计划

| Milestone | 目标 | 退出条件 |
|---|---|---|
| M0 | 冻结全部pool/scheduler/timer/thread caller与dynamic unload路径 | caller/worker inventory可机械复核；两项父P0拥有者和关闭条件明确 |
| M1 | 引入`ExecutionRuntime + WorkerInventory + WorkerDomain`与可失败构造 | thread budget守恒；构造失败回滚；TasksModule成为真实service owner |
| M2 | 引入`TaskScope + TaskDescriptor + Task<T,E>` | identity/owner/result/cancel/deadline/drop/shutdown policy统一，普通panic不杀进程 |
| M3 | 重建dependency、JoinSet、continuation与data parallel | cross-owner/cycle fail-closed；100k chain/fanout、single-worker wait与nested parallel资格通过 |
| M4 | 收敛DeadlineService与`BoundedOperationLane<K,T,E>` | timer只wake；cross-key并发、active cancel、typed result、pump reconciliation与deadline合同完成 |
| M5 | 迁移Asset/Scene/Preferences/VM/Text/Graphics/Navigation/App/Editor | 无production raw pool/default/process alias/direct thread旁路；必要专用线程均有lease |
| M6 | 接入ExecutionDiagnostics、trace与产品UI | generation/window/stale、owner/domain/priority/failure、histogram与shutdown blocker进入真实consumer |
| M7 | 完成fault、model、跨平台、soak与跨引擎同语义benchmark | artifact冻结环境和阈值，满足后才允许性能优于Unreal的结论 |

首个实现切片必须从M0/M1开始，不能先给`JobHandle`零散加cancel或priority字段。否则旧process owner、私有worker和dynamic unload缺口仍存在，新API只会成为第三套平行状态机。

## 12. 验收门禁

| Gate | Status | 验收条件 |
|---|---|---|
| RTASK-G01 | Fail | 每个production worker、timer、task可查询runtime/scope/module/plugin owner。 |
| RTASK-G02 | Fail | TasksModule activate返回ready receipt，deactivate返回quiescence receipt。 |
| RTASK-G03 | Fail | dynamic session destroy在worker/task/callback未退出时fail-closed，禁止DLL unload。 |
| RTASK-G04 | Fail | ExecutionRuntime构造失败返回typed error并join全部已创建worker。 |
| RTASK-G05 | Fail | close admission后clone、process alias、private lane均拒绝新工作。 |
| RTASK-G06 | Fail | shutdown按policy处理task并列出unfinished TaskId与owner。 |
| RTASK-G07 | Fail | shared和dedicated线程与WorkerInventory严格守恒。 |
| RTASK-G08 | Fail | 无未登记production `TaskPool::new/JobScheduler::default/direct thread`。 |
| RTASK-G09 | Fail | framework descriptor/status与runtime task是同一状态机。 |
| RTASK-G10 | Fail | `Task<T,E>`提供typed terminal，普通panic不终止进程。 |
| RTASK-G11 | Fail | Drop/cancel/shutdown policy可测试并返回ack。 |
| RTASK-G12 | Fail | start/completion deadline与observer timeout可区分。 |
| RTASK-G13 | Fail | cross-owner dependency无bridge即拒绝，cycle admission失败。 |
| RTASK-G14 | Fail | dependency failure保留上游TaskId和terminal class。 |
| RTASK-G15 | Fail | 100,000深链/宽fanout不栈溢出、不长期占用单worker。 |
| RTASK-G16 | Fail | wait_until、JoinSet、late observer与result retention有race/timeout测试。 |
| RTASK-G17 | Fail | 1/2/4/8/32/64核实际worker不超budget且低核可前进。 |
| RTASK-G18 | Fail | priority贯穿Editor/runtime queue，含ageing与inversion测试。 |
| RTASK-G19 | Fail | per-scope entry/byte/cost quota在并发admission严格守恒。 |
| RTASK-G20 | Fail | blocking I/O、CPU、main-thread、control work不能互相耗尽。 |
| RTASK-G21 | Fail | nested parallel与worker wait在单worker/跨domain矩阵无死锁。 |
| RTASK-G22 | Fail | ECS/Graphics data conflict形成可验证DAG，错误声明fail-closed。 |
| RTASK-G23 | Fail | benchmark输出throughput、queue p95/p99、steal、utilization、allocation。 |
| RTASK-G24 | Fail | dedicated worker有QoS/affinity/stack证据与完整join测试。 |
| RTASK-G25 | Fail | timer capacity按scope隔离，单owner storm不能饿死其他owner。 |
| RTASK-G26 | Fail | timer线程不执行无界业务callback，slow/panic有diagnostic与terminal。 |
| RTASK-G27 | Fail | cancel registration可等待in-flight callback lease归零。 |
| RTASK-G28 | Fail | fixed-rate/fixed-delay/coalesce/skip有fake-clock测试。 |
| RTASK-G29 | Fail | bounded operation支持per-key串行、cross-key并发、typed result、active cancel。 |
| RTASK-G30 | Fail | pump任意阶段panic后守恒恢复或lane fail-closed。 |
| RTASK-G31 | Fail | shutdown guard在同worker、timeout、Drop路径不死锁或无限等待。 |
| RTASK-G32 | Fail | execution snapshot带generation/window/stale并进入真实DiagnosticStore consumer。 |
| RTASK-G33 | Fail | Asset/Scene/Preferences/VM/Text/Graphics/Navigation/App/Editor adoption无旁路。 |
| RTASK-G34 | Fail | runtime operation、plugin、script task panic不杀Editor且可关联owner。 |
| RTASK-G35 | Fail | Editor close在unfinished task时保持依赖存活并提供retry/wait/force。 |
| RTASK-G36 | Fail | App、Editor、headless、commandlet共用execution shutdown protocol。 |
| RTASK-G37 | Fail | dynamic library load/run/unload 10,000轮无旧worker/callback进入卸载代码。 |
| RTASK-G38 | Fail | 100h soak无task/result/timer/observer/thread增长且identity不复用。 |
| RTASK-G39 | Fail | model/sanitizer/helgrind等覆盖handle/lane/timer/shutdown关键状态机。 |
| RTASK-G40 | Fail | 与Unreal/Bevy/Godot同语义benchmark满足预冻结性能稳定性阈值。 |

## 13. Owner 路由

- Runtime02保留event/task父边界和dynamic unload P0；Runtime114拥有execution具体合同、迁移顺序和资格门。
- Runtime24提供全仓identity/generation规则；Runtime114落地TaskId、ScopeId、WorkerDomainId、TimerRegistrationId。
- Runtime01/46拥有module/service总生命周期；Runtime114要求TasksModule成为真实execution owner。
- Runtime11实施计划可继续承接已排定的scheduler、asset、scene、preference切片，但其局部`risks=[]`不能覆盖本报告产品门禁。
- Editor09拥有EditorJobSystem业务adapter与产品关闭决策；Runtime114提供底层scope、cancel acknowledgement和quiescence receipt。
- Graphics/Text/Asset/Navigation/App的owner报告负责各自业务迁移；统一worker/budget/shutdown合同由Runtime114定义。
- Tooling优化按用户要求排除，不把本轮差距转交给Python/source-scan工具解决。

## 14. 复核与未执行项

本轮完成了当前working tree的静态源码与参考源码复核、文件集指纹、finding逐项对账和40项门禁判定。未运行Cargo是有意的：本轮不改生产或测试代码，且全工程MVP baseline门仍未完成；静态review不能把历史或其他会话的Cargo结果冒充本轮证据。

下一轮实现前必须重新检查HEAD、coordinator epoch、两项父P0、private worker caller、process pool/timer入口与dynamic shutdown。任何实现若只增加字段、wrapper或compat re-export，而没有移除旧owner旁路，不得计为本报告进度。
