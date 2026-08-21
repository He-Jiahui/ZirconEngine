---
related_code:
  - zircon_app/src/bin/zircon_shader_pbr_viewer/background_load.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/core/plugin/manager.rs
  - zircon_editor/src/core/process.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/direct_clip_worker.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
  - zircon_plugins/net/runtime/src/worker/net_worker.rs
  - zircon_plugins/plugin_sdk/src/declaration/macros.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/watch/asset_watcher.rs
  - zircon_runtime/src/asset/watch/spawn.rs
  - zircon_runtime/src/core/resource/event_stream.rs
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/tasks/timer.rs
  - zircon_runtime/src/diagnostic_log/sink/worker.rs
  - zircon_runtime/src/foundation/runtime/config_manager/worker.rs
  - zircon_runtime/src/graphics/pipeline/async_compile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/pipelined/queue.rs
  - zircon_runtime/src/operation/service.rs
  - zircon_runtime/src/scene/inspection/artifact/cache.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
tests:
  - zircon_editor/src/tests/jobs.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/controller_submission_operation_gate.rs
  - zircon_hub/tests/project_quick_actions_contract.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/navigation/runtime/src/tests/tiled_bake_context.rs
  - zircon_plugins/net/runtime/src/tests/http_routes.rs
  - zircon_plugins/net/runtime/src/tests/websocket.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime/tests.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool/lifecycle.rs
  - zircon_runtime/src/core/resource/manager/tests/concurrency.rs
  - zircon_runtime/src/core/runtime/tests/events/behavior.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior.rs
  - zircon_runtime/src/diagnostic_log/sink/tests/backpressure.rs
  - zircon_runtime/src/diagnostic_log/sink/tests/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/tests/resource_snapshot_contract.rs
  - zircon_runtime/tests/runtime_owned_result_v7.rs
  - zircon_runtime_host/src/foreign_output/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/02-pbr-viewer-tool-runtime-evidence-renderdoc-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/21-unsafe-rust-ffi-native-memory-thread-affinity-panic-unload-safety-governance-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/TaskConcurrencyLimiter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/ThreadHeartBeat.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/RunnableThread.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/QueuedThreadPool.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ScopeLock.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/LockTags.h
  - dev/bevy/crates/bevy_app/src/task_pool_plugin.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/godot/core/object/message_queue.h
  - dev/godot/core/os/safe_binary_mutex.h
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Light/HDGpuLightsBuilder.Jobs.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 24 · Concurrency、Locking、Atomic Ordering、Blocking、Thread Lifecycle、Backpressure 与 Deadlock 审查

## 1. 结论

Zircon并不是没有并发工程基础。Runtime已经有进程级IO、Async Compute、Compute三类Rayon `TaskPools`，`JobHandle`使用`Condvar`等待、允许worker在等待时协助执行，并在状态锁外调用完成回调；文本raster pool同时限制entry与completion bytes；diagnostic sink具有有界入口、批次byte budget和owned worker；pipelined render submission用可复用的一槽channel建立反馈；Editor play output和若干wizard/job入口使用有界队列；不少logging、i18n和settings路径先复制subscriber snapshot再在锁外分发。它们证明仓库知道线程命名、背压、快照发布和owned join的重要性，这些实现必须保留并提升为共同合同。

问题在于这些能力仍是各owner自行约定，不是产品级Concurrency Control Plane。仓库无法从同一份machine-readable truth回答：一个产品会启动多少线程和runtime、任务可在哪个execution domain运行、哪类调用允许阻塞、哪把锁必须先拿、一个atomic的Relaxed为什么成立、channel限制的是entry还是bytes/age、callback是否允许重入、Drop最多等待多久、hung render/network/plugin worker怎样隔离，以及哪条验证lane证明弱内存序和所有shutdown interleaving安全。

本轮对`zircon_app`、`zircon_editor`、`zircon_hub`、`zircon_plugins`、`zircon_runtime`、`zircon_runtime_host`、`zircon_runtime_interface`下Git追踪Rust代码做production-like扫描：排除明显tests/benches/examples/fixtures/generated/vendor/target和测试文件，并在首个纯`#[cfg(test)]`处截断。共覆盖11,485个文件、约1,009,937行前缀代码。该口径仍可能收进由父模块条件接入的测试文件，也看不到宏展开、cfg产品可达性、真实锁持有时长和runtime topology，所以数字是复核入口，不是缺陷数量。

| 词法信号 | occurrence / 文件 | 本轮解释 |
|---|---:|---|
| `Mutex` / `RwLock` / `Condvar` | 890 / 234；98 / 20；49 / 17 | 并发状态广泛存在，但没有统一lock graph或rank |
| `.lock()` / `try_lock` / `.read()`+`.write()` | 598 / 238；11 / 6；153 / 72 | `try_lock`稀少不等于缺陷；需按latency domain分类 |
| `MutexGuard` / `Arc<Mutex<_>>` / `Arc<RwLock<_>>` | 289 / 130；161 / 90；18 / 9 | guard lifetime和共享owner边界需要AST/call graph复核 |
| atomic type | 551 / 114 | 计数器、发布位、状态机和generation语义混在同一表面 |
| Relaxed / Acquire / Release / AcqRel / SeqCst | 250 / 65；101 / 40；40 / 25；46 / 21；4 / 1 | 不能机械升级SeqCst；必须登记每个跨atomic不变量 |
| `OnceLock` / `thread_local!` | 277 / 86；20 / 20 | process lifetime、session generation和线程退出没有统一owner |
| `JoinHandle` / raw `thread::spawn` / `thread::Builder` | 24 / 12；4 / 2；8 / 8 | 测试尾部截断后的保守生产入口；仍需resolved inventory |
| Rayon / Tokio runtime / `block_on` | 14 / 5；18 / 16；21 / 13 | 多个execution runtime与调用者阻塞没有共同预算 |
| bounded / unbounded constructor | 8 / 7；12 / 10 | imported短名会漏计；必须人工区分ongoing stream与one-shot |
| `try_send` / blocking send / blocking recv | 11 / 9；37 / 27；18 / 15 | channel API无法表达owner、deadline和产品线程限制 |
| timeout recv / try recv / sleep / yield / spin | 6 / 4；16 / 11；6 / 5；5 / 3；4 / 1 | 局部退避存在，未形成forward-progress contract |

按crate family复核，Runtime占4,901个文件、519,249行、410处Mutex信号和377处atomic信号；Editor占4,490个文件、304,014行、318处Mutex；Plugins占1,469个文件、119,833行、142处Mutex。App、Hub、Runtime Host和Runtime Interface信号较少，却处在产品进程、Tauri command、DLL/FFI边界，不能按数量降级。

本篇不重复Runtime02的线程分配缺陷、Hub01的全局session锁与detached thread、Plugin08E的Tokio/NetWorker重复和lock-across-wait、Runtime11A/11B的UI channel和thread-local font问题，也不复制各Graphics报告拥有的GPU wait/drop风险。**没有新增P0，登记40项P1和12项P2**。这里拥有的是跨crate的执行域、线程拓扑、锁序、atomic invariant、channel/backpressure、blocking、shutdown和并发验证合同；具体业务owner仍在原报告修复。

## 2. 审查方法与证据边界

### 2.1 Evidence等级

| Evidence | 本轮状态 |
|---|---|
| E1 tracked inventory | 已完成；source revision `ae2be3d865a937b9ed368bf965592045346c64e3`，branch `main` |
| E2 symbol/caller/state transition阅读 | 已覆盖task pool、job、asset publication、scene cache、Editor host、Hub、network、render queue、log sink和worker lifecycle代表路径 |
| E3跨owner产品语义 | 已对线程预算、callback重入、分裂发布、atomic ordering、channel cardinality、blocking和shutdown闭环 |
| E4 controlled scheduler/model checking | 未建立；Cargo/CI/tool配置没有直接Loom、Shuttle、Miri或TSan lane |
| E5 hang/soak/topology/performance | 未建立；当前source dirty且既有Editor、Hub、WOC动态lane阻断未变化 |

### 2.2 必须避免的误判

1. `Relaxed`适合独立统计计数；只有它参与发布协议、复合状态或lifetime时才需要Acquire/Release或单atomic编码。
2. `mpsc::channel()`不一定无界失控。PBR background load和GPU readback一类单生产、单结果、cardinality=1通道可以保留，但应由schema声明one-shot。
3. `Mutex`不天然比lock-free差。冷路径、低争用owner state与需要强不变量的事务可以继续用锁；关键是rank、latency class、callback和观测。
4. `try_lock`少不代表错误。实时路径不能以try-lock静默丢状态来掩盖owner设计，应先明确deadline/fallback。
5. 本轮没有用源码词法结果推导实际线程数。动态线程数取决于product、feature、CPU、runtime配置和插件，必须由TopologyReceipt实测。

## 3. 必须保留的工程基础

### 3.1 TaskPools与JobHandle已有正确骨架

Runtime `TaskPools`集中创建IO、Async Compute与Compute Rayon池并命名线程。`JobHandle`通过`Condvar`等待状态变化，worker等待时使用Rayon协助执行，完成回调在释放状态锁后调用，并捕获observer panic。重构应让它成为统一task scope的底座，而不是另建第四套通用job系统。

### 3.2 有界队列已经在关键路径出现

文本raster pool同时约束入口数量、完成结果bytes、timeout、cancel和shutdown；diagnostic sink有bounded entry、batch byte budget、output同步与worker join；pipelined render queue以一槽request/result channel限制在途帧；Editor play output还有entry与byte budget。这些实现比“把所有channel换成bounded”更成熟，应抽取policy与receipt而不倒退。

### 3.3 锁外回调模式已有可复用先例

`ProjectAssetManager::publish_generation_wake`在锁内只筛选subscriber并复制wake callback，随后锁外执行；logging、i18n、settings也多次复制subscriber/snapshot后释放状态锁。该模式可直接用于修复同文件`broadcast`的持锁wake，而无需引入复杂lock-free结构。

### 3.4 现有并发测试有价值但不完备

Runtime event、resource manager、dynamic API、plugin live host、task、foreign output，以及Editor viewport gate、Plugin AI/navigation/network均有真实多线程回归。它们覆盖部分并发调用和lifecycle，但操作系统调度下的重复通过不能证明所有interleaving、弱内存序或无死锁。

## 4. 已确认的具体缺口

### 4.1 Asset change在subscriber锁内执行外部wake

`project_asset_manager/runtime.rs:237-244`持有change subscriber `MutexGuard`执行每个`send`和`wake`。unbounded send本身通常不阻塞，但wake是外部callback，可能重入订阅/退订、触发任意consumer latency，或把锁序带到Runtime/UI边界。同文件`publish_generation_wake`已经展示正确做法：锁内收集callback，锁外执行。

### 4.2 Asset change ongoing stream使用unbounded channel

`subscribe_asset_changes_internal`在`runtime.rs:277-284`为每个subscriber建立`crossbeam_channel::unbounded()`。这不是one-shot；当import/watch生产速度超过Editor/runtime消费速度时，队列可以无entry、byte和age上限地增长。Runtime04记录了内部watch dispatch的bounded设计，但没有覆盖这一公开project change subscriber；本篇明确修正范围，不否定原报告对内部watcher的判断。

### 4.3 World inspection cache不是原子generation发布

`WorldInspectionArtifactCache`用六把`RwLock`分别保护artifact、fields、dirty sets、diagnostics和rebuild状态。`store()`先公开新artifact，再分别读取/清空dirty fields并更新fields；`inspection_artifact()`随后才调用`mark_hierarchy_rows_clean()`。并发reader可观察新artifact搭配旧dirty/rebuild/fields状态。应发布单个immutable generation snapshot或在一个owner transaction下提交，而不是依赖调用者记住跨六把锁的顺序。

### 4.4 Network latency的两个Relaxed atomic构成错误发布协议

`record_latency_ms`先Relaxed写`last_observed_latency_ms`，再Relaxed写`latency_observed=true`；reader先Relaxed读flag再读value。弱内存模型下，读者看到`true`不获得value写入的happens-before。可以用单atomic sentinel/packed representation，或flag Release、reader Acquire；独立byte counter仍可保留Relaxed。

### 4.5 Level与Editor host依靠人工维护多锁顺序

`LevelSystem`明确注释world先于subscription，并在world replacement期间依次进入frame、physics、animation、script state。Editor UI host拥有多把Mutex，layout/workspace/asset refresh沿调用链组合session、view registry、capability、animation/UI session和dependency state。当前没有lock rank type、debug verifier或生成lock graph，代码审查无法阻止未来反向获取。

### 4.6 Hub与部分产品线程缺少structured owner

Hub command直接`thread::spawn`执行focus refresh和background action，不保存JoinHandle；focus refresh还在session锁内执行共享项目刷新。PBR background load虽命名线程并catch panic，却只向调用方返回receiver。具体问题由Hub01/App02拥有；全局缺口是没有TaskScope/ThreadOwnerReceipt统一cancel、deadline、join与产品退出资格。

## 5. P1差距：Inventory、Execution Domain 与 Topology

### CONC-P1-001 · 没有canonical ConcurrencySiteInventory

锁、atomic、channel、thread、runtime、blocking wait和callback只能靠regex重建；没有cfg-expanded product reachability、owner、latency class、shutdown phase和验证lane。必须从Resolved SourceSet/BuildSet生成source-bound inventory。

### CONC-P1-002 · 没有统一ExecutionDomainDefinition

main/editor/render/RHI/IO/compute/async/plugin/network/audio线程语义散落在名字、注释和局部enum中。缺stable domain id、允许的blocking/lock/API、可迁移性、thread capability和failure isolation unit。

### CONC-P1-003 · 没有产品级ThreadTopologyBudget

Rayon pools、Tokio runtime、NetWorker、render submit、asset/config/log/text workers、Editor/Hub线程分别决定数量。BuildSet和ProductReceipt不能给出resolved thread count、oversubscription、stack reservation或pool ownership。

### CONC-P1-004 · Task pool分配结果没有truth receipt

Runtime02已拥有`remaining_threads=0`仍被`min_threads.max(1)`夹回1、两线程预算可创建三个worker的具体缺陷。本篇要求分配器输出requested/resolved/actual topology和invariant receipt，避免只修算式后其他pool继续绕过预算。

### CONC-P1-005 · raw/Builder thread没有统一ThreadOwnerManifest

Hub、App、Editor compile/play/watch、Plugin worker及Runtime watcher/config/diagnostic/async compile/render各自spawn。线程名、purpose、owner、stack、priority、cancel source、join phase和hang policy不在同一manifest。

### CONC-P1-006 · 缺少structured concurrency / TaskScope

任务可以脱离创建它的project/session/world/window/plugin generation继续运行。没有scope close禁止新任务、传播cancel、收集completion/failure并证明所有child terminal的共同协议。

### CONC-P1-007 · Task priority与QoS不是引擎级合同

参考实现区分task priority、low-priority queue和render/main thread工作；Zircon通用task API没有交互、frame-critical、streaming、background、maintenance等priority class，也没有aging和priority inheritance。

### CONC-P1-008 · CPU affinity、stack与平台线程属性没有resolution链

Plugin ABI存在`thread_affinity`字段，但产品侧没有统一解析到execution domain/worker selection的证据；未见通用affinity、stack size、NUMA或平台QoS policy。声明元数据不能替代调度执行与拒绝证据。

### CONC-P1-009 · 没有BlockingContextContract

`block_on`、blocking send/recv、join、filesystem/process/GPU wait没有标注允许的execution domain和deadline。main/editor/render/RHI线程上的偶发阻塞无法由lint或runtime guard拒绝。

### CONC-P1-010 · nested runtime/pool没有admission authority

Network同时创建multi-thread Tokio runtime和NetWorker，其他subsystem又创建Rayon/OS worker。具体网络问题由Plugin08E拥有；全局仍缺“可复用公共pool还是必须独占runtime”的决策与总预算。

## 6. P1差距：Locking、Publication 与 Reentrancy

### CONC-P1-011 · 没有canonical LockGraph与LockRank

代码注释和调用习惯承担锁序，无法自动检测cycle、反向获取或跨crate lock chain。必须生成LockId、rank、owner、domain、recursive/reentrant policy和允许edge。

### CONC-P1-012 · Guard lifetime泄漏到调用边界

保守扫描有289处`MutexGuard`信号、130个文件。`lock_*` helper和返回guard API让锁持有长度由远端调用者决定；需要AST确认public/cross-module guard并优先改成closure、snapshot或owner operation。

### CONC-P1-013 · Asset subscriber callback在锁内执行

`ProjectAssetManager::broadcast`是已确认实例。必须先复制delivery target/wake action，释放subscriber锁，再执行发送后的外部唤醒；reentry和unsubscribe应有测试。

### CONC-P1-014 · LevelSystem多锁transaction没有机器校验

world、subscription、frame、physics、animation、script的顺序由注释和函数组织维持。新增调用链可轻易形成逆序；world replacement还需要明确哪些retired object析构必须在锁外完成。

### CONC-P1-015 · EditorUiHost多锁组合没有snapshot transaction

layout/workspace/asset refresh组合多个registry/session/dependency guard。应按owner operation建立prepare snapshot、锁外工作、generation-checked commit，或至少用lock rank与contention instrumentation约束。

### CONC-P1-016 · WorldInspectionArtifactCache分裂发布

六把RwLock保护一个逻辑generation，new artifact可能和旧fields/dirty/rebuild状态同时可见。必须以单`Arc<Snapshot>`、ArcSwap或单transaction lock发布一致代次。

### CONC-P1-017 · callback/reentrancy policy没有类型表达

subscriber、wake、plugin callback、UI emission和job completion有的锁外执行、有的锁内执行。接口未声明MayReenter、NoReentry、OwnerThreadOnly或CallbackLease，调用者无法证明锁与module generation安全。

### CONC-P1-018 · poison recovery没有与锁内不变量绑定

Tooling23已拥有poison继续/终止策略分裂。本篇要求每把保护transactional state的锁声明panic后state是否可继续、需重建还是隔离；`into_inner()`不能作为通用并发恢复。

## 7. P1差距：Atomic、Global 与 Memory Layout

### CONC-P1-019 · 没有AtomicInvariant registry

250处Relaxed信号分布65个文件，但代码没有登记atomic保护的事实、参与者、memory order证明、ABA/wrap规则和关联字段。reviewer只能逐次重新推导。

### CONC-P1-020 · Network latency发布缺少happens-before

两个Relaxed atomic组成value+valid复合状态，reader可看到valid但读到旧value。这是具体ordering defect；修复后需弱内存模型测试，而不是仅在x86上循环运行。

### CONC-P1-021 · OnceLock/process singleton缺少generation与reset owner

277处OnceLock信号跨86个文件。缓存、pool、registry和平台singleton在project/session/plugin reload、device recreate与测试隔离时是否可复用没有共同lifetime contract。

### CONC-P1-022 · thread_local state缺少worker生命周期协议

20个文件使用thread-local信号。pool扩缩、plugin unload、font/cache generation变化和线程退出时，per-thread copy的内存、generation与cleanup没有统一inventory；Runtime11B拥有font具体问题。

### CONC-P1-023 · 热atomic与共享队列没有false-sharing基线

Unreal limiter显式cache-line padding热任务slot；Zircon没有统一cache layout annotation、hot counter packing策略或跨核心争用profile。不能在没有测量时盲目padding，但必须有基线和证据门。

## 8. P1差距：Channel、Admission 与 Backpressure

### CONC-P1-024 · 没有ChannelPolicy registry

bounded/unbounded/sync/channel由owner直接选型，缺ChannelId、producer/consumer domain、cardinality、entry/byte/age budget、overflow、wake、shutdown和metrics schema。

### CONC-P1-025 · Asset change subscriber是无界ongoing stream

每个subscriber使用unbounded queue，慢消费者可无限积压`AssetChange` clone。必须合并/coalesce generation、限制entry+bytes+age，并定义overflow后full resync或disconnect语义。

### CONC-P1-026 · 仅有entry bound不足以控制资源

不同大小的日志、play output、asset change、GPU readback和network payload不能只按消息数预算。正确的text/log局部实现应推广为entry、bytes、age、in-flight work和retained artifact多维admission。

### CONC-P1-027 · one-shot与stream没有schema级区分

PBR background result和GPU readback即使用unbounded API也被cardinality限制；机械lint会误报。ChannelPolicy必须区分OneShot、LatestValue、CoalescedGeneration、BoundedStream和LosslessJournal。

### CONC-P1-028 · overflow fairness与starvation策略缺失

drop newest/oldest、block、coalesce、reject、spill、disconnect由局部代码决定，没有producer fairness、tenant/project quota、priority aging或关键控制消息保留槽。

### CONC-P1-029 · cancel/deadline不是admission的一部分

任务和消息进入队列后常只有receiver断开或owner Drop才终止。缺deadline、cancel generation、obsolete-work rejection和queued/inflight cancellation receipt。

## 9. P1差距：Shutdown、Forward Progress 与 Observability

### CONC-P1-030 · Thread shutdown/join没有统一deadline

有的worker在Drop中join、有的detached、有的忽略join panic。没有StopAccepting -> Cancel -> Drain -> Join -> Escalate phase、每阶段deadline和剩余thread receipt。

### CONC-P1-031 · GPU/render/plugin等待可无限延长Drop

pipelined render queue虽然bounded且owned，但worker若卡在GPU/execute，Drop join可无限等待；plugin watcher/hot reload也可能在外部工作中阻塞退出。具体owner报告继续修实例，本篇要求产品级shutdown budget与isolation。

### CONC-P1-032 · yield/spin退避没有统一CPU预算

diagnostic worker在control queue和deadline循环中多次`yield_now`；局部deadline是正向基础，但高负载/teardown时仍可能持续占用CPU。需要event/Condvar或经测量的bounded backoff policy。

### CONC-P1-033 · 没有thread heartbeat/hang detector

没有像Unreal `FThreadHeartBeat`那样按thread/function/checkpoint记录前进、允许平台暂停并区分stuck/hang的服务。产品只能表现为卡死，不能生成owner、stack/symbol、last checkpoint和blocking cause receipt。

### CONC-P1-034 · 没有统一contention与queue telemetry

缺lock wait/hold percentile、queue depth/bytes/age、task latency、steal/assist、cancel lag、join duration和thread utilization的稳定指标。局部diagnostic counter不能回答跨subsystem priority inversion。

### CONC-P1-035 · 没有deterministic schedule/replay

现有多线程测试依赖OS调度，失败interleaving不可记录、缩减和重放。需要seeded scheduler或event barrier trace，至少覆盖task、event、asset publication、plugin lifecycle和foreign output。

### CONC-P1-036 · 没有weak-memory/model-checking lane

直接配置中未找到Loom、Shuttle、Miri或TSan；`Cargo.lock`中的Loom只是传递依赖，不构成Zircon验证。Atomic发布、lock-free/once初始化和cancel state machine没有状态空间探索。

### CONC-P1-037 · 并发stress/soak缺少产品拓扑矩阵

已有单元级并发测试，但没有1/2/4/8/16+ core、oversubscription、slow consumer、device loss、plugin reload、project close、window close和shutdown storm组合，也没有hang-free duration gate。

### CONC-P1-038 · Task completion没有统一identity与receipt

JobHandle、channel result、thread join、operation和plugin/network future各自表达完成。缺TaskId、scope、generation、submitted/started/terminal time、outcome、cancel reason和failure domain，无法证明旧代任务未发布结果。

### CONC-P1-039 · Plugin thread affinity声明缺少执行证据

SDK/ABI可以声明main-thread-only或worker-safe，但没有统一scheduler admission、runtime assertion、callback thread receipt和违规隔离。Tooling21拥有unsafe/thread-affinity boundary；本篇拥有调度执行闭环。

### CONC-P1-040 · Concurrency truth没有进入产品资格

BuildSet、Capability Truth和Release qualification没有要求resolved topology、零detached owner、bounded channel、shutdown receipt、hang-free soak和contention/performance baseline。功能通过仍可能隐藏不可发布的并发风险。

## 10. P2改进项

### CONC-P2-001 · 生成AST/CFG-aware并发inventory

从resolved Cargo graph生成lock/atomic/channel/thread/blocking/callback sites，输出definition、caller、feature、product和source digest，替代regex长期账本。

### CONC-P2-002 · 建立typed `TaskScope`与`ScheduledTask`

统一scope owner、domain、priority、deadline、cancel、generation和terminal receipt；允许Rayon/OS/Tokio作为backend，不强迫所有任务改成async。

### CONC-P2-003 · 建立ExecutionDomain scheduler adapter

把Main、Editor UI、Render Submit、RHI、Compute、IO、Plugin和Background编码为capability，并在blocking/affinity违规时debug fail、release降级并记录。

### CONC-P2-004 · 引入LockId/Rank与debug lock graph

为多锁owner提供typed rank guard、动态edge采样、超时stack capture和CI cycle检查；先覆盖LevelSystem、EditorUiHost、asset/plugin lifecycle。

### CONC-P2-005 · 建立AtomicInvariant与litmus/model test模板

每个复合atomic协议声明writer/reader、order、wrap/ABA和state diagram，并能转成Loom/Shuttle测试；独立Relaxed counter用显式exemption保留。

### CONC-P2-006 · 建立ChannelPolicy与AdmissionTicket

统一entry/byte/age/inflight预算、overflow、coalesce、fairness、cancel和metrics，并让one-shot/cardinality exemption机器可读。

### CONC-P2-007 · 将分裂状态迁移为immutable generation snapshot

优先修World inspection、Editor view/session projection和只读高频状态，用`Arc<Snapshot>`/ArcSwap或generation-checked commit减少读锁与混代观察。

### CONC-P2-008 · 建立产品ThreadTopology resolver

按CPU、平台、产品、feature和插件解析pool/runtime/worker数量、stack、priority与affinity，输出requested/resolved/actual receipt并设总预算。

### CONC-P2-009 · 建立Heartbeat/ForwardProgress service

线程和长任务登记checkpoint、expected wait和suspend reason；超时生成owner、domain、last progress、queue/lock/GPU状态及symbolized stacks。

### CONC-P2-010 · 建立contention profiler与可视化

把lock wait/hold、queue age、task latency、worker utilization和priority inversion接到统一trace，并在Editor/Hub提供产品诊断视图，而不是仅日志文本。

### CONC-P2-011 · 建立deterministic scheduler与fault injection suite

支持seed replay、barrier perturbation、slow/stuck consumer、panic、disconnect、cancel、poison、device loss和shutdown storm，失败输出最小event trace。

### CONC-P2-012 · 建立固定workload scalability基线

在1到多核心、固定frame/asset/network/text场景测吞吐、tail latency、CPU、context switch、cache miss、内存峰值与shutdown时间，比较Zircon版本和参考架构，而不是宣称抽象上更快。

## 11. 目标架构

```text
Resolved SourceSet / BuildSet
  -> ConcurrencySiteInventory
  -> ExecutionDomainDefinition + ThreadOwnerManifest
  -> ProductThreadTopologyBudget
  -> LockGraph / AtomicInvariant / ChannelPolicy
  -> TaskScope + AdmissionTicket + ScheduledTask
  -> Completion / Failure / ShutdownReceipt
  -> ConcurrencyEvidenceReceipt
  -> CapabilityTruth / ProductQualification / Release
```

关键约束：

1. Scheduler backend可以是Rayon、Tokio、OS thread、platform main loop或GPU queue，但owner、domain、budget和terminal receipt必须统一。
2. 锁、atomic和channel是实现选择，不是architecture owner；它们必须服务于同一generation/state transaction。
3. callback、foreign call、filesystem/process/GPU wait默认不得在未知锁内或实时domain阻塞；例外必须definition-bound并可观测。
4. 所有产品退出都必须证明scope停止接收、任务终止、线程已join或被明确隔离；detached best-effort不能铸成clean shutdown。
5. 性能优化必须由contention/scalability证据驱动；禁止为了“工程化”机械替换为lock-free或SeqCst。

## 12. 分阶段重构计划

### M0 · Inventory与动态基线

1. 生成cfg/product-aware concurrency inventory，给所有thread/runtime/pool/channel/lock/atomic/blocking site分配owner。
2. 在Editor、Runtime Preview、Hub、PBR Viewer和plugin product采集actual thread topology、queue、join和lock wait基线。
3. 固化本报告三个具体回归：asset wake锁外、inspection单代发布、network latency memory ordering。

### M1 · Domain、Topology与Blocking contract

1. 定义ExecutionDomain、ThreadCapability、TaskPriority和BlockingClass。
2. 修复TaskPoolThreadAssignment并输出requested/resolved/actual topology。
3. 把现有Rayon/OS/Tokio入口接入ThreadOwnerManifest，不立即替换backend。

### M2 · Lock与Publication收敛

1. 为LevelSystem、EditorUiHost、asset/plugin lifecycle建立首批LockGraph/rank。
2. callback默认锁外执行；对reentry建立contract test。
3. 将World inspection迁移为immutable generation snapshot，并验证reader看不到混代。

### M3 · Channel、Backpressure与TaskScope

1. 建立ChannelPolicy，先接asset changes、Editor jobs/play output、diagnostic、text和render queues。
2. asset changes改为bounded/coalesced generation，overflow触发full resync receipt。
3. 引入TaskScope，收编Hub/App detached thread和plugin/runtime owned worker lifecycle。

### M4 · Atomic、Global lifetime与验证

1. 修复network latency ordering；为发布位、generation、cancel state建立AtomicInvariant。
2. 登记OnceLock/thread-local owner、generation和reset/unload策略。
3. 增加Loom/Shuttle或等价model lane、TSan平台lane和deterministic schedule replay。

### M5 · Forward progress与产品资格

1. 建立heartbeat、lock/queue/task telemetry和shutdown receipt。
2. 跑core-count、slow consumer、reload/close/device-loss/shutdown storm矩阵。
3. 让ConcurrencyEvidenceReceipt成为产品qualification与release required input。

### M6 · 性能收敛

1. 用固定workload定位oversubscription、priority inversion、false sharing和tail latency。
2. 仅对已测热点选择snapshot、sharding、batching、work stealing、lock-free或cache-line layout。
3. 与Unreal/Bevy/Godot/Fyrox/Unity Graphics参考机制比较合同与证据，不以源码形状替代性能结果。

## 13. 验收标准

| Gate | Required evidence |
|---|---|
| Inventory | 当前BuildSet所有生产thread/runtime/pool/lock/atomic/channel/blocking site都有owner与source digest |
| Topology | 每个产品有requested/resolved/actual线程清单，总数不越预算，独占runtime有理由 |
| Locking | required lock graph无cycle；关键锁有wait/hold观测；外部callback不在未知锁内 |
| Publication | inspection与其他generation snapshot不可观察混代；旧代task不能发布当前结果 |
| Atomic | 每个复合协议有AtomicInvariant和model test；独立Relaxed counter有exemption |
| Backpressure | ongoing stream有entry/byte/age/inflight预算与overflow语义；one-shot有cardinality证明 |
| Blocking | main/editor/render/RHI domain的blocking operation被拒绝或有显式bounded exemption |
| Lifecycle | project/window/world/plugin/product关闭后所有task/thread terminal；join有deadline与receipt |
| Forward progress | hang detector能定位stuck owner/checkpoint；shutdown storm无永久等待 |
| Validation | deterministic replay、model checking、TSan、stress/soak和core topology矩阵进入required lane |
| Performance | 固定workload tail latency、CPU、memory、context switch与shutdown time不回退，并绑定BuildSet |

## 14. Reference engines带来的约束

| Reference | 可借鉴约束 | 不应照搬 |
|---|---|---|
| Unreal | concurrency limiter有slot、priority、timeout、task lifetime和cache-line padding；heartbeat有per-thread/checkpoint hang诊断 | 不复制宏/API命名，也不把大型全局task graph直接移植到Rust |
| Bevy | App统一创建IO/compute pools并有spawn/destroy callback；ECS executor按dependency/access并行，处理main-thread executor和panic | Bevy当前task pool只能作为基础下界，不能替代Zircon产品线程、GPU/plugin/Editor资格 |
| Godot | WorkerThreadPool有TaskId/group、low priority、wait/assist和native callback；SafeBinaryMutex显式owner语义 | 不继承其所有global singleton和兼容性折中 |
| Fyrox | 简洁TaskPool/engine executor展示最小可用async result pump | 只能当下界，不足以满足目标中的大型工程调度和诊断 |
| Unity Graphics | JobHandle、NativeArray read/write标注和Burst job把依赖、数据访问、并行粒度显式化 | 不把C# Job System表面API直接映射成Rust类型 |

参考源码说明的共同点不是“线程越多越快”，而是并发必须有任务身份、依赖、访问权限、优先级、等待/完成、lifetime、诊断和性能证据。Zircon已有若干优秀局部实现，下一步应先收敛这些共同合同，再在热点上选择更激进的数据并行与lock-free结构。

## 15. 本轮验证与限制

本轮仅新增review与索引，未修改production、tests、manifest或workflow。没有重跑Cargo、Editor、Hub、WOC、GPU、network、soak或benchmark；已知Editor编译、Hub调用签名、WOC native/typed contract与plugin locked metadata阻断没有变化，重复运行不会增加证据。

直接Cargo/CI/tool配置扫描未发现Zircon自有Loom、Shuttle、Miri或ThreadSanitizer lane；`Cargo.lock`中的Loom是传递依赖，不能算验证能力。动态线程拓扑、锁争用和内存序结果尚未采集；实施前必须在source drift后重取AST/Cargo inventory，并按产品BuildSet执行E4/E5验收。
