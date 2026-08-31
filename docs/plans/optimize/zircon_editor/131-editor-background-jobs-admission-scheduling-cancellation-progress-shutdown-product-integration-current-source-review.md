---
related_code:
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/context
  - zircon_editor/src/core/asset/dirty/save_job_adapter.rs
  - zircon_editor/src/core/asset/import_flow
  - zircon_editor/src/core/recovery/autosave
  - zircon_editor/src/core/recovery/autosave_adapter
  - zircon_editor/src/core/recovery/autosave_service.rs
  - zircon_editor/src/core/notifications/progress
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/runtime_shutdown.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/service.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/export.rs
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_slot.rs
  - zircon_runtime/src/dynamic_api/session/registry/frame_activity.rs
  - zircon_runtime/src/scene/dynamic_scene/spawn_task/loader.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload/queue.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/IAssetCompilingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/AssetCompilingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/AsyncWork.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/AsyncTaskNotification.h
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/task.rs
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.Jobs.cs
doc_type: review-and-refactor-plan
refreshes: docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 131 · Background Jobs、Admission、Scheduling、Cancellation、Progress、Shutdown 与产品接入当前源码复核

## 1. 结论

Zircon Editor的后台作业系统已经越过“随手spawn一个线程”的最初阶段。当前源码有统一`EditorJobSystem`、有限category quota、三档公平队列、pending entry/claimed-byte/age admission、批量预留与Drop回滚、mutex group、显式依赖、typed ticket、panic containment、cooperative cancellation、priority-aware progress snapshot、observer resync和每tick有界event pump。这些基础是真实的，后续重构必须保留。

旧Editor09的三个P0中已有两个被当前源码实质关闭：

1. keyed pending merge现在会再次调用`progress.register(existing_job, &spec)`，原子刷新当前cancel token、label、priority与presentation；`keyed_pending_merge_refreshes_progress_cancellation_authority`直接验证最新token被取消、旧token不受影响。
2. lifecycle事件已进入`EditorJobEventJournal`，默认同时受4,096 entries、16 MiB retained bytes与5分钟oldest age约束；drop/coalesce/high-water/sequence exhaustion均可诊断，lifecycle丢失通过`JobJournalGap`显式要求resync。

仍然开放的P0是Editor shutdown quiescence。Runtime当前已有真实的`ExecutionRuntime/ExecutionScope`：它关闭scope admission、区分取消request/acknowledgement、等待queued/running census归零，dynamic runtime session也会在module shutdown前执行这个barrier；这是必须复用的下层基础。但`EditorJobSystem`仍直接调用从`CoreHandle`克隆的`JobScheduler`，没有通过scope schedule，因此`shutdown(deadline)`仍只广播合作式取消并等待progress active map，无法join/reap executor或child process，也没有late-commit barrier。产品还在global jobs shutdown之前调用Editor侧`shutdown_runtime_session()`，并在检查unfinished之前完成settings shutdown；project admission lease虽已延后到检查成功之后释放，仍不能证明活任务依赖的runtime/settings/project资源在deadline失败路径保持有效。

本刷新保留旧ID以便追踪，当前状态为P0：1 Open / 2 Closed；P1：42 Open / 4 Partial / 2 Closed；P2：8 Open / 2 Closed，并保留24个验收门。没有修改生产代码，也没有运行Cargo、真实Editor、线程/进程故障注入、长时间内存压力或shutdown动态验证；结论来自当前源码、测试合同与参考引擎生命周期对照。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Zircon Editor/Runtime selected | 256 / 47,380 / 43,123 nonempty / 1,644,196 bytes | 26个证据根展开后的去重物理集合；439个测试属性、18个ignored；fingerprint `0b2a98a5640e62c3a43738ed1a861fb238cd14547904e9f654afab01ff764fb1` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | 10 / 4,379 / 3,786 nonempty / 159,471 bytes | asset compile shutdown、async work、notification、task pool、worker group与GPU jobs；7个测试属性；fingerprint `fb69b4df1ffd7d985ac388c15aae60141dd4429c0b5940741c946ce5a274b999` |
| Plan/docs evidence | 8 / 2,563 / 1,837 nonempty / 367,761 bytes | engine-wide、Runtime task、Editor UI/document/asset/plugin/play与旧Editor09 owner；fingerprint `c6e180f0b4eaa5f7818c506325d433b756c78dccc78cb2ba97012426380687f1` |
| selected union | 274 / 54,322 / 48,746 nonempty / 2,171,428 bytes | 当前工作树去重物理集合；446个测试属性、19个ignored；fingerprint `ccba7c3c78838e08810fb03bcae1085d06d0ad08881f8475cff988810cce0328` |

产品集合按frontmatter列出的26个当前证据根递归展开并按物理路径去重；没有通过临时关键字结果删减文件。fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256；它只标识本轮阅读集合，不是ABI、schema或兼容性身份。

### 2.2 当前源码与动态验证隔离

本轮逐项读取job core、event journal、progress/observer、submission/pending/scheduling/lifecycle、Runtime execution scope、dynamic session destroy barrier、autosave shutdown、retained app close顺序以及产品thread/process边界。工作树中存在与本报告无关的dirty文件；本轮没有回退、格式化、暂存或提交它们。`source_recheck_required`表示实现前必须复取全文、fingerprint和测试可达性。

本轮没有运行Cargo或动态产品验证。静态证据等级如下：

- E3：P0调用链、锁/队列边界、product shutdown顺序和所有P1实现事实均逐文件闭环。
- E2：439个选中源码测试属性表达的行为合同，包括merge cancel刷新、paused consumer journal bound/gap、backpressure restore、priority primary、scope drain、shutdown condvar与storm budget；它们未被本轮动态执行。
- 未覆盖：真实Editor压力运行、跨平台线程调度、进程强杀、GPU任务、文件系统故障、插件卸载并发和长时间内存曲线。

### 2.3 本轮追踪的产品链

1. producer构造`EditorJobSpec` -> admission/reservation/key merge -> pending queue -> quota/mutex/dependency promotion -> runtime compute scheduler。
2. runtime closure -> `JobContext` -> progress/cancel -> typed output sender -> terminal state/history -> event queue -> message bus与notification observer。
3. retained host tick -> `pump_events()` -> bounded batch/time消费 -> workbench progress projection。
4. save/import/autosave/preview/export/welcome/profile/viewport adapter -> ticket/generation/cancel/result轮询。
5. Runtime dynamic session -> 建立`ExecutionScope` -> scoped scene spawn/reload -> destroy时关闭admission并等待scope census -> module shutdown -> process log shutdown。
6. app close -> Editor runtime-session receipt -> final autosave/global job shutdown -> settings flush/shutdown -> unfinished gate -> project close/admission lease release。
7. compile host、Play output和plugin development watcher的自管线程 -> 现有thread ownership source contract -> 产品关闭边界。

完整通知卡片、历史和交互由notification专项报告拥有；本报告只审查job progress作为producer以及通知中心作为consumer的合同。Play、Plugin和Build/Export内部业务由各自专项拥有，本报告只负责它们是否受统一作业authority治理。

## 3. 已有工程基础，重构时必须保留

### 3.1 Admission不是空壳

- 默认pending上限为16,384项、64 MiB claimed bytes和5分钟oldest age，`EditorJobBatchAdmissionReservation`在真正物化任务前预留，未commit时Drop释放。
- pending ledger把可执行任务与未物化reservation计入同一entry/byte窗口，keyed replacement也会重新检查replacement bytes。
- category quota都是有限值；Thumbnail/Export/InteractiveSave/Play有独立默认与settings入口，Import/Compile/Index/Misc至少受worker parallelism派生上限，而非`usize::MAX`。
- priority slot固定为`Interactive, Interactive, Normal, Interactive, Normal, Background`，并按category扫描可运行项；这比纯FIFO更接近Editor交互需求。

### 3.2 Dependency、mutex与ticket的局部正确性

- `.after(JobId)`只接受已注册identity，terminal dependency由有界history保留，pending dependency会pin terminal record，避免简单的悬空handle。
- `MutexGroup`通过scheduler handle tail把同组任务串行化；显式dependency和mutex dependency共同进入`schedule_after`。
- `JobTicket<T>`保留typed result，job panic被`catch_unwind`转换为`JobError`，terminal callback在锁外运行。
- pending cancellation会真正移除queue entry并完成ticket，不只是设置一个UI标记。

### 3.3 Progress、observer与pump已有性能意识

- primary progress采用generation fast path；generation未变化时retained consumer不会重复克隆label/message。
- progress observer在dispatch锁外调用，observer panic被隔离；1,024项observer backlog达到上限后折叠为一次resync，而不是无限保存每个delta。
- event pump每次最多64项或1 ms，progress按job保留latest value，避免每个百分比变化都压垮UI tick。
- notification progress center把可见active job限制为64项，product tick确实调用`jobs().pump_events()`。

### 3.4 产品已有统一入口的种子

dirty save、import、autosave、UI asset refresh、preview refresh、export wizard/queue、welcome project probe、viewport lazy resolve和profile artifact export都已接入同一个context-owned `EditorJobSystem`。这说明重构应收敛现有adapter，而不是另造第二套“高级任务系统”。

## 4. P0 当前状态：1 Open / 2 Closed

### E-JOB-P0-01 · Closed：keyed merge cancellation authority 已刷新

当前`submit_admitted`在`merge_pending_admission`成功后调用`progress.register(existing_job, &spec)`；`register`替换同一JobId的`ActiveJobEntry`，同步刷新共享label、category、priority与cancel token。`keyed_pending_merge_refreshes_progress_cancellation_authority`进一步断言`request_cancel(existing_id)`只取消latest token，随后`jobs.cancel(existing_id)`使保留ticket以Cancelled结束。旧的“实际任务观察新token、progress仍保存旧token”调用链已经不存在。

仍需由P1-25/26/28处理的是结构化cancel状态、merge generation和跨promotion竞态，而不是重新打开这个已关闭P0。

### E-JOB-P0-02 · Open：shutdown deadline仍不等于quiescent barrier

`EditorJobSystem::shutdown(deadline)`关闭submission、drain/cancel pending、向active token广播取消，再通过condvar等待progress active map；deadline到达便返回`Vec<UnfinishedEditorJob>`。busy-spin已经消失，但它仍没有join runtime work、reap child process、撤销executor queued closure、阻止late completion side effect，也没有把`Requested`、`Acknowledged`与`Quiescent`区分为状态机。其生产构造链是`EditorManager -> core.scheduler().clone() -> EditorContextBuilder -> EditorJobSystem`，submission最终直接调用`scheduler.schedule_after`；这些任务没有进入`ExecutionScope` census。

Runtime侧已经有可保留的部分实现：`ExecutionRuntime`拥有独立pool set和scope registry，`ExecutionScope`提供capacity、close admission、CancelOnDrop、request/acknowledge与condvar drain；dynamic session创建`dynamic-session` scope，scene spawn/reload开始通过该scope调度，session registry在销毁时保留teardown-incomplete slot供重试。但`CoreRuntime::shutdown_execution`自身明确声明legacy scheduler、timer与private workers尚未scope-owned，`ExecutionShutdownReport`也明确不是worker/DLL unload receipt。故它只能作为Editor重构的下层承载，不能关闭本P0。

当前产品顺序比旧报告略有改进：`commit_project_close()`和OS-backed project admission lease释放已移动到unfinished检查成功之后。但`ui/retained_host/app.rs`仍先调用Editor侧`shutdown_runtime_session()`，其receipt只覆盖event consumer、world sync、Play session和Play gateway，不携带Runtime execution drain结果；之后才执行final autosave/global jobs shutdown。settings `flush_then_shutdown().finish()`也发生在unfinished检查之前，`EditorAutosaveService::shutdown`在deadline返回后立即清空active/retired adapter。故runtime、settings或adapter依赖仍可能在non-cooperative Editor job存活时被拆除。

Unreal的`IAssetCompilingManager`明确区分cancel hint与完成保证。Zircon必须由顶层shutdown authority依序关闭admission、按scope cancel、持续pump completion、join/reap受管executor/thread/process、禁止late publish，再释放runtime/settings/project/plugin资源；deadline失败只能进入显式quarantine/fatal策略。

### E-JOB-P0-03 · Closed：lifecycle event journal 已有硬边界与gap

旧`VecDeque + latest_progress`无界模型已被`EditorJobEventJournal`替换。默认限制为4,096 entries、16 MiB retained bytes、5分钟oldest age；每个event获得checked monotonic journal sequence，progress按JobId合并，超entry/byte/age与oversized lifecycle会记录drop并合并为`EditorJobEventJournalGap`。snapshot公开depth、retained bytes、oldest age、high-water、coalesced/dropped计数与sequence exhaustion。

`paused_consumer_keeps_the_job_journal_bounded_and_publishes_a_resync_gap`、oversized lifecycle与16,384-job pressure源码合同覆盖暂停consumer、byte bound和gap发布。剩余问题是生产侧没有明确的gap/resync consumer和shutdown drain policy，归入P1-37/38/41，不再以OOM无界队列P0描述。

## 5. P1：authority、调度、依赖、进度与产品接入缺口

当前状态映射：P1-28、P1-39 Closed；P1-12、P1-35、P1-41、P1-42 Partial；其余42项Open。表内已将这些已变化的实现事实更新为当前源码，不以旧描述重复报错。

### 5.1 Authority、scope与生命周期

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-JOB-P1-01 | Runtime已有owner字符串+task capacity的`ExecutionScope`，但`EditorJobSpec`没有app/project/document/plugin/subsystem owner、owner generation或resource identity，且Editor submission不走scope；仍只能逐ticket取消。 | 扩展并复用Runtime scope contract，使`JobScopeId + OwnerLease + Generation`进入descriptor、event、result和shutdown；支持按scope query/cancel/drain/revoke，禁止另造平行scope系统。 |
| E-JOB-P1-02 | CoreRuntime现在显式拥有worker pool，但`with_scheduler*`仍是公开constructor，可创建多个独立`EditorJobSystem`共享同一runtime scheduler；quota、mutex、ID和shutdown彼此隔离。 | process/editor instance只有一个`EditorJobAuthority`，测试通过显式fixture capability构造；domain只能取得scope-bound client。 |
| E-JOB-P1-03 | system为`Clone<Arc<...>>`，任一clone都能调用global `shutdown`并永久关闭submission。 | shutdown capability只由唯一top-level owner持有；普通client只能关闭自己的scope。 |
| E-JOB-P1-04 | 没有按状态、owner、executor、deadline查询的权威job registry，只有active progress、pending统计和256项dependency terminal record。 | bounded retained registry保存state transition、owner、资源、结果摘要和terminal reason，支持分页诊断与scope barrier。 |
| E-JOB-P1-05 | 全局shutdown挂在`EditorAutosaveService`上，domain service拥有了进程级生命周期权力。 | autosave只关闭autosave scope；`EditorShutdownCoordinator`拥有完整阶段与错误聚合。 |
| E-JOB-P1-06 | compile output、Play output和development watcher各自拥有线程，未注册到统一supervisor或closeout dependency graph。 | `WorkerSupervisor`管理thread/process/watcher owner、stop signal、join/reap、deadline与diagnostic；是否使用task pool由blocking模型决定。 |
| E-JOB-P1-07 | `EditorJobSystem::join`公开透传runtime pool，绕过JobId、admission、progress、cancel和shutdown；当前export output capture在job内嵌套使用，但API不强制该前提。 | 只暴露受限`JobExecutionContext::parallel_join`，计入父job资源；顶层product不能直接取得scheduler escape hatch。 |
| E-JOB-P1-08 | `JobTicket` Drop默认为静默detach，既不取消也不要求显式`detach()`；owner丢票后系统仍无法说明责任归属。 | ticket必须显式await/cancel/detach，detached job仍绑定scope和retention policy；debug/telemetry记录orphan。 |

### 5.2 Admission、资源与优先级

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-JOB-P1-09 | pending bytes由caller自报，默认固定4 KiB；没有测量、上限校准、实际峰值或结果大小reconciliation。 | typed resource claim覆盖input capture、working set、output、child process和GPU staging；完成后记录estimated/actual偏差并治理异常producer。 |
| E-JOB-P1-10 | bytes只在pending阶段计账，promotion后立即释放，即使runtime executor尚未真正开始或任务正在消耗大内存。 | resource lease跨Queued/Running/Publishing全阶段，terminal commit/cleanup后才释放。 |
| E-JOB-P1-11 | Import/Compile/Index/Misc默认各等于worker parallelism，category quota之和可远超worker width；它限制分类，不是全局running容量。 | 先有executor global capacity，再叠加category、scope和resource-class quota；快照区分admitted、executor queued和running。 |
| E-JOB-P1-12 | Partial：Started event已移动到runtime closure真正开始执行后发送；但`state.mark_started`和category running accounting仍发生在`schedule_after`之前，executor queued与running继续混为一态。 | 明确Admitted/Ready/ExecutorQueued/Running/Publishing/Terminal状态，真正开始执行时才占running metrics。 |
| E-JOB-P1-13 | priority只决定Editor pending选择；`JobScheduler::schedule_after`没有priority参数，低优先级任务进入runtime queue后不可重排或抢占。 | executor接受priority/deadline/aging；queued work支持retract/reschedule，Interactive latency有可测SLO。 |
| E-JOB-P1-14 | 文件IO、child-process等待、import/compile、preview compute都复用一个compute scheduler，blocking work可占满compute worker。 | 明确Compute/BlockingIO/Process/GPU/MainThreadCommit executor class，各自容量、thread policy、shutdown与telemetry。 |
| E-JOB-P1-15 | settings只开放Thumbnail/Export/InteractiveSave/Play四类，值1..64且重启后生效；没有per-project/plugin fairness或动态负载调整。 | validated runtime policy按hardware、executor和scope配置，支持热更新、安全上下限及配置来源诊断。 |
| E-JOB-P1-16 | oldest pending age是全局oldest；incoming request的max age可被另一category/owner的旧job触发，造成跨域反压。 | age budget按scope/category/queue计算并返回具体阻塞owner；global emergency gate另行定义。 |
| E-JOB-P1-17 | admission key和mutex group只做非空验证，label也无长度；没有namespace、owner或字符/byte上限。 | validated `JobKey/ResourceLockId/LocalizedJobLabel`绑定owner namespace并有严格bytes/count预算。 |
| E-JOB-P1-18 | spec没有execution deadline、queue deadline、timeout action、retry/backoff或retry safety。`max_pending_age`只控制后续admission，不终止旧job。 | deadline属于状态机；区分queue expiry、execution timeout和publish timeout，retry必须声明idempotence与cleanup策略。 |
| E-JOB-P1-19 | 没有CPU weight、IO bandwidth、open files、process slots、GPU memory或device queue预算，category无法表达真实竞争。 | resource vector由admission broker原子授予，支持多资源避免死锁，并输出每类饱和原因。 |
| E-JOB-P1-20 | 每次promotion最多64项是固定数量预算，没有基于耗时、executor queue depth或frame hitch反馈的治理。 | promotion使用time/work budget与backpressure，记录扫描、promotion、queue latency分位数。 |
| E-JOB-P1-21 | batch reservation可由持有者无限期占据entries/bytes，只有commit、显式release或Drop；没有lease deadline和stale reclamation。 | reservation带owner、expiry和renewal；authority自动回收并记录abandoned materialization。 |

### 5.3 Dependency、取消与结果

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-JOB-P1-22 | `.after(JobId)`只保证先后；现有测试明确固化“dependency失败后dependent仍运行”。 | dependency edge声明`RequireSuccess/Always/OnFailure/Finally`，失败和取消传播由图策略决定。 |
| E-JOB-P1-23 | terminal dependency history固定256；稍晚提交会得到`ExpiredDependency`，没有durable build/session graph identity。 | scope内DAG拥有graph lifetime和retention，edge不依赖全局最近N个terminal job。 |
| E-JOB-P1-24 | dependency不携typed result，不能表达fan-in、artifact、partial result或main-thread commit阶段；生产代码当前也没有真正使用`.after`。 | `JobNode<I,O>`和typed artifact handle构成显式DAG，图在admission前校验资源、edge和commit policy。 |
| E-JOB-P1-25 | `CancellationToken`只有atomic bool，没有request reason、requester、deadline、phase或generation。 | structured cancellation request进入audit/state machine，token只作为executor端快速观察缓存。 |
| E-JOB-P1-26 | Editor `cancel(id)`返回true只表示设置了token，不表示任务观察、停止副作用、回滚或完成；Runtime scope虽已有request/acknowledge区分，Editor事件、ticket和UI没有接入。 | 复用Runtime cancellation语义并在Editor层分离Requested/Acknowledged/Quiescent；UI和shutdown以quiescent为安全门。 |
| E-JOB-P1-27 | 所有active progress snapshot硬编码`cancellable=true`，即使任务从不检查token或已进入不可中断commit。 | descriptor声明cancel capability，运行时随phase更新；UI只在确实可接受取消时提供操作。 |
| E-JOB-P1-28 | Closed：keyed merge后重新`progress.register(existing_job, &spec)`，current cancel、label、category、priority与primary projection会一起刷新；回归源码验证latest token。 | 保留该行为并在未来owner/generation改造时维持原子replace，不再按旧差异重复实施。 |
| E-JOB-P1-29 | 已admit任务不能pause、resume、reprioritize或从runtime queue retract；Interactive请求无法修正队列反转。 | handle支持合法状态下的priority change、queued retraction和domain-defined pause/checkpoint。 |
| E-JOB-P1-30 | `JobTicket::wait(self)`没有deadline、取消、UI pump或线程约束，调用者可以无限阻塞。 | 提供nonblocking poll/async await/deadline wait；UI线程的blocking wait由lint和runtime assertion禁止。 |
| E-JOB-P1-31 | JobId没有公开的typed result query或retained result handle；ticket丢失后只能看progress，不可恢复完成结果。 | bounded result store按scope/owner policy保留typed artifact/status，支持late observer和restart-safe result receipt。 |
| E-JOB-P1-32 | `JobEventKind::Failed`只携String，丢失错误stage、code、retryability、artifact、owner和source chain。 | versioned `JobFailure`携结构化分类、cause chain摘要、diagnostic attachments和redaction policy。 |
| E-JOB-P1-33 | 没有统一的partial artifact cleanup、retry、resume/checkpoint或crash recovery协议；每个adapter自行处理残留。 | job descriptor声明transaction/attempt directory、cleanup owner、resume token与idempotence，terminal由authority确认发布。 |

### 5.4 Progress、event、observer与诊断

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-JOB-P1-34 | `report_progress`接受total=0、completed>total、倒退、total变化和无界message，没有phase/subtask。 | validated progress model支持indeterminate、phase、nested work、monotonic或显式reset语义及message byte cap。 |
| E-JOB-P1-35 | Partial：primary已由`visible_by_priority: BTreeSet<(u8, JobId)>`选择，不再纯取最小JobId；同优先级仍按JobId，且没有recent activity、user focus、phase或用户pin策略。 | presentation policy按visibility、priority、recent activity和user focus选primary，并可展示多任务摘要。 |
| E-JOB-P1-36 | terminal时active progress立即移除；notification observer只retire active card，没有成功/失败/取消结果、retry或artifact入口。 | terminal outcome按severity和policy保留，通知消费结构化result；完整通知生命周期由后续专项报告实现。 |
| E-JOB-P1-37 | 非测试production未发现通用`TOPIC_JOB/EditorMessagePayload::Job`订阅者；message bus事件当前主要是未消费旁路。 | 明确唯一event journal和订阅API，删除无owner的重复publication或为其定义真实consumer与backpressure。 |
| E-JOB-P1-38 | product `pump_events()`只在retained host tick调用；headless、commandlet或替代host没有canonical pump owner。 | job authority提供host-neutral driver，GUI/CLI/test显式注册消费策略；shutdown也拥有drain phase。 |
| E-JOB-P1-39 | Closed：`event_journal_snapshot()`已公开depth、retained bytes、oldest age、high-water、coalesced/dropped lifecycle/progress与sequence exhaustion，gap承担resync信号。 | 保留snapshot并把这些指标接入profile/status/性能gate；后者由P1-43继续拥有。 |
| E-JOB-P1-40 | progress observer虽然在锁外且panic-isolated，但同步运行在submit/finish线程；slow observer会阻塞producer。系统只允许一个observer。 | bounded async fan-out或snapshot subscription，支持多个lease-bound observer、per-subscriber cursor和slow-consumer策略。 |
| E-JOB-P1-41 | Partial：event已有checked monotonic `journal_sequence`并在丢失时发布gap；仍没有timestamp、scope、owner generation、attempt、executor或correlation。 | versioned event envelope携完整身份与时序，consumer能去重、检测gap和拒绝stale generation。 |
| E-JOB-P1-42 | Partial：label/progress message已用`Arc<str>`共享，primary有generation fast path，notification可用`snapshot_for_ids`；通用`snapshot()`仍全量克隆snapshot，且无paged registry/delta cursor。 | immutable generation snapshot或paged view共享Arc数据，delta消费不反复物化全量集合。 |
| E-JOB-P1-43 | 没有可查询的queue/execution/publish latency、per-owner terminal rate、cancel latency或deadline miss；runtime scheduler只有聚合诊断。 | JobId到stage timing的bounded telemetry，按executor/category/scope输出分位数并关联profile trace。 |

### 5.5 产品接入与治理

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-JOB-P1-44 | `EditorUiHost::save_document_toolkit`提交`ForegroundDocumentSaveJob`后立刻`ticket.wait()`；animation和UI asset保存路径调用它，主UI线程失去响应。 | Save返回operation handle，UI持续pump并显示真实progress/cancel/close gate；最终commit仍走Editor 02的统一dirty/save authority。 |
| E-JOB-P1-45 | 各adapter手工保存ticket、cancel、generation和result channel，stale-result规则重复且不一致。 | scope-aware adapter基类/组合器统一latest-wins、generation check、cancel replacement、terminal publish和owner teardown。 |
| E-JOB-P1-46 | thread ownership source test宣称production必须经`EditorJobSystem`，但compile host、Play output和development watcher仍直接创建线程；合同与产品现实相互矛盾。 | 定义允许的worker类型并全部注册supervisor；source/lint test检查“有owner capability”，而非粗暴禁止一切线程。 |
| E-JOB-P1-47 | ownership scanner是762行手写Rust token scanner，依赖源码形状，可能漏掉macro/generated/间接spawn，也会把合理blocking worker与逃逸混为一谈。 | Clippy/compiler lint、封装capability和link-time inventory共同治理；source scan只作补充。 |
| E-JOB-P1-48 | Runtime dynamic scene spawn/reload已开始使用`ExecutionScope`，但Editor job、Play output、live watcher和compile capture不在同一quota/progress/cancel/status/shutdown registry，project close仍无法证明全部后台活动已停。 | 建立product adoption matrix；所有长任务、线程、process和watcher必须有scope、supervisor、status与close barrier，允许不同executor但不允许不同lifecycle authority。 |

## 6. P2：一致性、极端边界与维护性

| ID | 当前差距 | 建议 |
|---|---|---|
| E-JOB-P2-01 | `next_id`使用`saturating_add`；到`u64::MAX`后会重复JobId并覆盖record。 | checked terminal error或128-bit scoped identity，永不复用。 |
| E-JOB-P2-02 | admission reservation ID与terminal order也使用saturating arithmetic，极限时排序/identity不再唯一。 | 与JobId共用不可回绕epoch/sequence策略。 |
| E-JOB-P2-03 | primary progress generation用`checked_add(...).expect`，极限时直接panic；与其他counter的饱和策略不一致。 | 统一typed generation overflow终态，不在产品进程panic。 |
| E-JOB-P2-04 | 多个独立JobSystem都从JobId 1开始，event/progress没有authority ID；跨system日志可歧义。 | `JobAuthorityId + JobSequence`构成全局可诊断身份。 |
| E-JOB-P2-05 | `JobCategory`是封闭enum，plugin/domain不能登记带policy metadata的新category，只能挤入Misc。 | built-in稳定ID加validated extension category/owner policy，或改用resource class与tags组合。 |
| E-JOB-P2-06 | fairness slot、terminal retention 256、pump 64/1 ms、observer backlog 1,024都是硬编码，缺来源和现场反馈。 | 记录设计预算、暴露diagnostics，并只对确需运维的项开放validated配置。 |
| E-JOB-P2-07 | Closed：shutdown已使用`Condvar::wait_timeout`，旧`yield_now` busy-spin不存在。 | 在后续scope barrier改造中保留事件驱动等待与可注入deadline测试。 |
| E-JOB-P2-08 | Closed：progress snapshot与JobEvent均共享`Arc<str>` label，progress message也以`Arc<str>`保留。 | 保留共享不可变文本；本地化、message byte cap与分页分别由P1-34/P2-10处理。 |
| E-JOB-P2-09 | `EditorJobProgressSource::default`和snapshot constructor可公开构造与真实authority无关的source/value。 | 构造权收窄到authority；测试通过fixture builder获得。 |
| E-JOB-P2-10 | priority/category rank和事件显示依赖Rust enum/Debug语义，没有稳定wire/localization ID。 | 为诊断、持久化和UI定义稳定ID、localized message key与schema version。 |

## 7. 与参考引擎的差距

| 参考 | 可复用的工程原则 | Zircon当前差距 | 不应照搬 |
|---|---|---|---|
| Unreal `IAssetCompilingManager/AssetCompilingManager` | 按asset type注册manager、对象级finish、cancel只是hint、Shutdown阻塞到安全、per-frame apply、专用thread pool | 无owner/type manager、对象级barrier、safe shutdown和结构化post-compile结果 | 不复制其全局singleton、UObject/宏体系或历史兼容层 |
| Unreal `AsyncWork` | pool/priority/required memory/debug name、queued retract cancel、EnsureCompletion、reschedule | priority不进入executor、只算pending bytes、无法retract/reprioritize、cancel无ack | 不把每类Editor业务都退化为裸`FAsyncTask`式模板 |
| Unreal `AsyncTaskNotification` | cancellable是动态属性，区分progress/prompt/completion、headless/unattended与keep-open策略 | Zircon硬编码cancellable且terminal立即消失；通知专项仍待审 | presentation不能反向成为execution authority |
| Bevy `TaskPool` | pool拥有worker生命周期，scoped task在返回前完成，spawn task显式await/cancel/detach | Zircon ticket Drop隐式detach，Editor authority不拥有底层pool shutdown | Bevy面向runtime task，不提供Editor project/plugin/admission语义 |
| Godot `WorkerThreadPool` | singleton/named pool、normal/low队列、high priority、TaskID/GroupID、group processed count、协作wait与退出runlevel | Zircon没有executor group、group progress、priority传递和lifecycle runlevel | 不复制Godot全局对象模型；需要scope/owner隔离 |
| Fyrox core/engine task | UUID task result channel；engine handler把completion绑定plugin UUID或scene/node/script owner | Zircon ticket更typed但没有owner binding；product adapter手工防stale | Fyrox core pool的无界result channel和薄封装只可作最低基线 |
| Unity Graphics Jobs | `JobHandle`依赖组合、Burst data-parallel job、ReadOnly/WriteOnly资源声明 | Zircon dependency无typed data/resource access，不能形成可验证并行DAG | Graphics package不是通用Editor job/notification/shutdown authority |

这些参考位于不同层级，没有一个可以整套复制。目标是用Zircon自身的`EditorJobAuthority`统一产品owner、admission、executor、result和shutdown，向下复用现有`ExecutionRuntime/ExecutionScope`，再把runtime scheduler、OS process/thread、GPU queue作为受管执行后端。

## 8. 目标架构

```text
EditorShutdownCoordinator (唯一global shutdown capability)
        |
        v
EditorJobAuthority ---- bounded Job Registry / Event Journal / Telemetry
        |
        +-- JobScopeLease(App / Project / Document / Plugin / Tool)
        |       `-- JobDescriptor(identity, generation, resources,
        |                         priority, deadlines, cancel policy,
        |                         presentation, result policy)
        |
        +-- Admission Broker
        |       +-- global + executor + scope + category quotas
        |       +-- resource-vector leases and expiring reservations
        |       `-- priority aging / queue deadline / dedup generation
        |
        +-- Typed Job Graph
        |       +-- success/failure/finally edges
        |       `-- artifact/result/main-thread commit nodes
        |
        +-- Worker Supervisor
        |       +-- Compute executor
        |       +-- Blocking I/O executor
        |       +-- Child process executor
        |       +-- GPU/device executor
        |       `-- long-lived watcher/thread owner
        |
        `-- Presentation Adapter
                +-- progress snapshots / cancel state
                +-- terminal notification / retry / artifact action
                `-- GUI, CLI and diagnostics cursors
```

关键约束：

- owner lease被撤销后，旧generation任务可以清理但不能提交产品状态。
- cancellation request不等于quiescence；关闭必须等待scope barrier或进入显式fatal/quarantine状态。
- admission lease覆盖完整资源生命周期，不能在executor queue前提前释放。
- lifecycle事实只在一个bounded registry/journal中发布；notification、message bus和status都是consumer。
- blocking线程和child process可以保留专用实现，但必须受同一scope、supervisor和shutdown合同治理。

## 9. 分阶段重构路线

### M0 · 先封闭三个P0

- 让keyed merge原子更新current cancellation endpoint和presentation generation，并启用现有started-merge回归测试。
- 将lifecycle queue改为有entry/byte/age上限的state journal，补consumer暂停压力测试和high-water diagnostics。
- 让app closeout在unfinished job存在时停止拆卸或进入明确fatal流程；把全局shutdown capability从autosave service移到top-level coordinator。

### M1 · 建立唯一authority与scope lease

- 在现有`ExecutionRuntime/ExecutionScope`上补齐`JobAuthorityId/JobScopeId/OwnerLease/OwnerGeneration`，不创建平行scope authority。
- 收窄constructors、Clone client和shutdown capability；实现scope query/cancel/drain/revoke。
- 建立bounded job registry及完整state machine。

### M2 · 资源感知admission与多executor

- 定义Compute/IO/Process/GPU/MainThreadCommit资源class和resource-vector claim。
- 全局capacity叠加scope/category quota，资源lease覆盖queued/running/publishing。
- priority/deadline进入executor，提供queue retraction、aging和Interactive latency gate。

### M3 · Typed graph、cancel acknowledgement与result

- 将`.after`升级为带edge policy的typed DAG，支持artifact fan-in和main-thread commit。
- 引入cancel request/ack/quiescent状态及phase-dependent cancelability。
- result/failure/attempt/cleanup/checkpoint形成versioned合同。

### M4 · Progress、journal、notification与telemetry

- progress支持indeterminate、phase、subtask和message budget。
- bounded journal提供sequence/cursor/resync；GUI、CLI和status共用。
- notification呈现terminal outcome、retry和artifact；trace记录stage latency、resource wait和cancel latency。

### M5 · 产品迁移与线程治理

- 依次迁移save/autosave、import/preview、export/process、welcome/viewport/profile。
- 将Play output、compile capture和plugin watcher登记到Worker Supervisor。
- 删除public scheduler escape，替换手写source scanner为capability + lint + inventory。

### M6 · 规模、故障与跨平台验收

- 长时间job storm、slow/paused consumer、memory pressure、priority inversion、owner revoke和shutdown fault injection。
- Windows/Linux/macOS进程/线程关闭、文件锁、child tree、GPU device loss矩阵。
- 建立与固定硬件、build identity绑定的Interactive latency、throughput和memory baseline；只有可复算数据才能讨论优于Unreal。

## 10. 验收门

1. keyed pending merge在pending、promotion竞态、started和shutdown四阶段都只取消current generation token，旧token不影响新任务。
2. `cancel`状态区分Requested、Acknowledged和Quiescent；UI不把设置bool显示为已经停止。
3. lifecycle journal在consumer完全暂停且持续提交短任务时，entry、bytes和oldest age始终受硬上限约束。
4. journal超窗后consumer收到typed resync-required，并能从authority snapshot恢复无重复终态。
5. app/project/plugin/document各有scope lease；revoke后旧generation completion不能写入当前产品状态。
6. 非top-level client在编译期无法关闭global job authority。
7. shutdown先关闭admission，再cancel、pump、join/reap、禁止late commit，最后释放project/plugin/settings/runtime。
8. deadline到达且有non-cooperative work时，产品不得继续普通closeout；错误包含scope、job、executor和phase。
9. 所有受管thread、watcher和child process在shutdown后均已join/reap，OS handle与临时目录无泄漏。
10. pending reservation有lease deadline，owner崩溃或忘记Drop后能自动回收并留下diagnostic。
11. admission同时限制global、executor、scope、category及memory/IO/process/GPU资源，任何维度饱和都返回typed原因。
12. resource claim从admission持续到publish/cleanup结束，executor排队不会提前释放预算。
13. Interactive任务在background saturation下满足固定硬件上的queue-start p95/p99门槛，priority进入真实executor。
14. blocking IO和child wait不占满compute workers；各executor starvation和backpressure有独立测试。
15. dependency edge可声明success/failure/finally，失败与取消传播在fan-in图中确定且可测试。
16. typed graph能传递artifact/result并在main-thread commit前拒绝stale owner generation。
17. progress拒绝非法total/completed、超长message和未声明reset；nested phase可正确聚合。
18. cancellable UI状态随job phase变化，non-cancelable commit不显示虚假取消按钮。
19. terminal failure保留structured stage/code/retryability/cause摘要，notification和CLI不依赖任意String解析。
20. `JobTicket`必须显式await/cancel/detach；UI线程blocking wait触发测试或lint失败。
21. save、autosave、import、preview、export、welcome、viewport、profile、Play output和plugin watcher全部出现在统一registry与adoption matrix。
22. headless、GUI和shutdown driver都能推进event/result消费；没有只依赖retained tick的隐藏前提。
23. 10万短任务、1万并发progress更新、slow observer和observer panic压力下，内存、tick时间、sequence和terminal count满足基线。
24. Windows/Linux/macOS fault matrix覆盖worker panic、child hang、file lock、disk full、plugin unload、project switch和device loss，且关闭后无late product commit。

## 11. 实施边界与交叉计划

- Runtime worker pool、`ExecutionRuntime/ExecutionScope`、JobHandle和底层trace由[Runtime 02](../zircon_runtime/02-core-runtime-events-tasks-review.md)拥有；Editor本轮要求复用这些能力并定义其上层authority、产品接入与executor合同。
- foreground save、dirty generation、atomic publication和recovery仍由[Editor 02](02-document-transaction-save-autosave-recovery-review.md)拥有；本报告负责取消UI blocking wait和提供scope/barrier。
- import/thumbnail/reference产品语义由[Editor 04](04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md)拥有；本报告负责资源、generation与job result治理。
- plugin owner、unload/reload与watcher由[Editor 06](06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md)拥有；本报告要求它们进入统一scope和supervisor。
- Play child/session/recovery由[Editor 07](07-play-session-process-pie-game-view-live-edit-recovery-review.md)拥有；本报告负责output worker、process slot和shutdown barrier。
- notification center的完整history、toast、action、accessibility和presentation performance另开Editor 10；这里不重复审查其全域。

禁止的实施捷径：

- 不得在后续merge generation改造中回退当前cancel/label/priority原子刷新，或让adapter保存第三份token重新分裂authority。
- 不得移除event journal的entry/byte/age硬边界、gap与high-water诊断，也不得只增大pump 64或observer 1,024常量掩盖consumer缺失。
- 不得把所有blocking IO、process和watcher强塞进compute pool以满足source scanner。
- 不得以“Rust会保住Arc内存”为由把scope quiescence等同于安全关闭；文件、进程和外部系统副作用仍必须停止。
- 不得用一次绿色小测试宣称达到Unreal级任务系统；必须通过第10节的资源、故障、关闭和长时间压力门。

## 12. 当前审查状态

- `review_status: current_source_refresh_complete`：26个证据根展开为256个当前Zircon物理文件，P0调用链、两个已关闭差异、Runtime scope部分基础及shutdown产品顺序已重新闭环。
- `implementation_status: pending`：本轮没有修改任何production job、runtime scheduler、product adapter、thread owner或测试源码。
- `source_recheck_required: true`：实现前必须复取256文件集合、fingerprint和相关dirty diff。
- 动态验证：本轮按review-only边界未运行Cargo、真实Editor、thread/process fault、paused consumer soak或shutdown deadline测试；测试源码只能作为静态合同。
- 后续审查继续覆盖notification center、history、toast/action、retention、accessibility与job/diagnostic integration，以及其他authoring边界。
