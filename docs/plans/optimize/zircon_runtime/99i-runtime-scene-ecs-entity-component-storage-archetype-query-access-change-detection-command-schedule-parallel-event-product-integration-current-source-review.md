---
title: Runtime Scene ECS Entity、Component、Storage、Archetype、Query、Access、Change Detection、Command、Schedule、Parallel、Event 与 Product Integration 当前源码工程化差距复核
category: zircon_runtime
report_id: Runtime108
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime/src/scene/ecs
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/ecs_registration
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
tests:
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/ecs_identity_storage
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_query
  - zircon_runtime/src/scene/tests/ecs_query_structure
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/scene/tests/ecs_commands.rs
  - zircon_runtime/src/scene/tests/ecs_commands
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_schedule
  - zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs
  - zircon_runtime/src/scene/tests/ecs_events_messages.rs
  - zircon_runtime/src/scene/tests/ecs_observers_messages.rs
  - zircon_runtime/src/scene/tests/ecs_systems
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_08_owner_tree.rs
plan_sources:
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassEntityManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassArchetypeTypes.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassEntityQuery.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassCommandBuffer.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassObserverManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassProcessingContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassProcessingPhaseManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassArchetypeData.cpp
  - dev/bevy/crates/bevy_ecs/src/component/info.rs
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_ecs/src/storage/table/mod.rs
  - dev/bevy/crates/bevy_ecs/src/query/state.rs
  - dev/bevy/crates/bevy_ecs/src/query/access.rs
  - dev/bevy/crates/bevy_ecs/src/query/world_query.rs
  - dev/bevy/crates/bevy_ecs/src/query/par_iter.rs
  - dev/bevy/crates/bevy_ecs/src/system/system_param.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
  - dev/bevy/crates/bevy_ecs/src/observer/runner.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/Fyrox/fyrox-core/src/pool/multiborrow.rs
  - dev/godot/core/object/object_id.h
  - dev/godot/core/object/message_queue.h
  - dev/godot/core/object/worker_thread_pool.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.Jobs.cs
doc_type: review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime108：Runtime Scene ECS 当前源码工程化差距复核

## 1. 结论

截至本轮冻结，Zircon 的 Scene ECS 已经不是“只有 HashMap 的临时样例”：它有带 generation 的内部 Entity slot、table/sparse 两类存储、按 archetype signature 建索引的 columnar table、bundle 结构事务、compiled query plan、change tick、packed deferred command、event/message/observer、system access conflict graph、worldless worker lane 与 frame diagnostics。这些基础应保留，不能为了追求表面上的 Unreal 风格而推倒重写。

但它仍未达到工程级 ECS kernel，更不能宣称性能或可靠性优于 Unreal、Bevy 或 Unity Graphics。Runtime60 登记的 **2 项 P0、72 项 P1、18 项 P2和 40 项验收门禁仍由 Runtime60 唯一拥有**。本轮没有重复创建 umbrella issue，新增计数为 **0 P0、0 P1、0 P2**；92 项差距中 **没有一项具备 closed 证据**。

两项 P0 均仍可从当前源码直接证明：

1. `system::Query::iter(&self)`、`iter_combinations(&self)`、`count(&self)`和`is_empty(&self)`继续通过共享接收器把同一`*mut QueryState`转成`&mut QueryState`。safe caller可在首个iterator仍存活时发起第二次cache refresh，触发并存可变别名，且Vec扩容可能让首个iterator引用失效。
2. `RemovedComponentEvents`继续按`TypeId -> Vec<RemovedComponentEvent>`永久追加；reader只移动自己的cursor，World没有update、retention、ack-based reclamation或clear维护。正常remove/despawn流量即可形成无界内存增长。

当前源码相对 Runtime60 的 2026-08-20 冻结有三类可验证进展：较大`UniqueEntityArray`从O(N²)改为排序验证；schedule/native/runtime system新增Real/Virtual clock domain；derived-state与worker control allocation增加诊断。它们使 RECS-P1-18、P1-42、P1-44、P1-54、P1-67和P2-16成为“**Partial，未关闭**”。其中clock domain实现仍在virtual pause时无条件跳过全部InternalSceneSystem，包括`UpdateEvents`和`ApplyDeferred`；排序去重也仍不返回duplicate index；诊断没有消除稳定帧分配或形成per-system/plugin top offender。

本轮只做review、证据冻结和重构计划维护，没有修改production、tests、Cargo、ABI或参考引擎。也没有运行Miri、sanitizer、loom、fuzz、100h soak、真实产品scene benchmark或跨引擎同负载对照，因此不能以“测试暂时通过”替代这两项P0和40项资格门。用户已暂停tooling优化，本篇不扩写tooling实现或迁移任务。

## 2. 审查边界、currentness 与 ownership

### 2.1 Canonical owner 与去重规则

| 领域 | Canonical owner | Runtime108 的作用 | 本轮不重复登记 |
|---|---|---|---|
| Scene ECS combined contract | Runtime60 | 用当前working copy重验全部kernel与产品接入事实 | RECS-P0-01..02、P1-01..72、P2-01..18、G01..40 |
| Scene/World lifecycle | Runtime05 | 核对World clone/replace、derived state与runtime ownership | World/Scene lifecycle父问题 |
| Stable identity | Runtime24 | 核对Entity/World/schema/provider generation | 全仓identity/epoch/exhaustion父问题 |
| Job scheduler | Runtime59 | 核对task pool、parallelism、shutdown与worker budget | 通用scheduler/task lifecycle父问题 |
| Runtime08 implementation plan | Runtime08 | 接收未来ECS kernel实现与失败回传 | 当前open failure artifact，不在review中冒充关闭 |
| Render-facing jobs | Graphics/Render owners | 说明ECS extract/chunk lease应如何被图形链消费 | GPU resource、frame graph、RHI父问题 |

固定架构仍是`zircon_app`、`zircon_runtime`、`zircon_editor`三个public root package，runtime内部遵循`core/{runtime,framework,manager,math,resource}` spine。Scene是runtime truth，Editor只做authoring adapter；不得把Editor scene model或plugin manifest声明变成第二World authority，也不得用compatibility facade绕过新的WorldIdentity、schema和access token。

### 2.2 当前产品源码物理冻结

本轮冻结root `Cargo.toml`以及`zircon_runtime*`、`zircon_app`、`zircon_editor`、`zircon_plugins`、`examples`、`templates`、`tests`中的tracked-like产品文本扩展`.rs/.toml/.wgsl/.json/.ron/.zui/.zr`，排除`target`。算法为repo-relative path小写排序，逐文件lowercase SHA-256，以`path<TAB>hash`按LF连接且末尾无LF，再计算manifest SHA-256。

| 范围 | 文件 | 行 | 非空行 | bytes | test attrs | ignored | unsafe行 | Fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| 全产品文本 | 18,845 | 3,305,644 | 3,112,346 | 119,102,085 | 20,725 | 246 | 1,648 | `9fae89abee856f0da717ea5d54ea24e977111bbe36013ce93357ad2c175397aa` |
| `scene/ecs` | 143 | 22,173 | 19,881 | 707,022 | 45 | 1 | 204 | `5925ad607f40562cbfb66f5364a44b57bf2049beca7add6f3f064e03f7059ce3` |
| World/module/product chain | 89 | 25,491 | 23,628 | 962,473 | 80 | 1 | 0 | `431364503842d929868c5fb91a2150488897723e959e89919aa93a63b5b57b47` |
| focused ECS tests | 89 | 20,181 | 18,231 | 758,675 | 479 | 3 | 0 | `587e9980cf34287014b5739c922c6f267364400e1af9a4b826514611b3606ab6` |
| 26个显式参考文件 | 26 | 23,515 | 20,627 | 921,948 | 142 | 0 | 321 | `fa13fb44639f1829304f4131d985735c3aeeda79daa546706a6c75e56580160f` |

冻结对应HEAD `bee4c707b714738346b49bba15c59468b8bd9b39`、baseline epoch 339。共享working copy处于degraded/dirty状态，且有其他会话正在修改scene、plugin和索引文件。本报告绑定上述working-copy物理快照；implementation开始前必须对owner path重做source recheck和fingerprint，不能把本篇当永久源码事实。

### 2.3 逐文件复核方法

Runtime60已在两天前逐文件读取`scene/ecs` 143个Rust文件，并展开World/module/LevelSystem与focused tests。本轮不是用一次关键词搜索替代该工作，而是执行以下增量复核：

1. 对当前`scene/ecs` 143文件逐文件生成行数、声明、unsafe、panic/assert与temporary/TODO清单。
2. 对Runtime60 baseline `bea1acf91b909525ab1759e2c800858b0eda6528`到当前working copy做全路径diff。相关变化为47个文件、1,746 additions、200 deletions；逐一阅读全部ECS/schedule/World/product变化。
3. 对未变化文件复用Runtime60逐文件结论，并重新核对关键negative evidence：World identity、mutable tuple、periodic tick clamp、reader retirement、removed retention、parallel World access、dependency barrier、Miri/trybuild/soak。
4. 沿`World -> Schedule -> SceneScheduleRunner -> SystemState/Query -> CommandQueue -> Event/Message/Observer`走完整产品调用链，不把单元测试API当产品接入。
5. 展开26个显式参考文件，分别以Unreal Mass、Bevy ECS、Fyrox Pool、Godot object/job primitives和Unity Graphics jobs进行交叉对照。

因此本篇是Runtime60全量审查的current-source refresh，不是一个新的抽样报告，也不重置其issue编号或验收门禁。

### 2.4 精确反例搜索

| 搜索/检查 | 当前结果 | 判定 |
|---|---|---|
| `WorldIdentity/WorldId/WorldCell` in Scene ECS/World/tests | 无可用于QueryState/SystemState绑定的实现 | RECS-P1-01、13、14、24、26仍open |
| `QueryMutData` tuple impl | 无；tuple macro只实现`QueryDataAccess + QueryData` | RECS-P1-16仍open |
| `QueryData for &mut T` | 仍返回只读`&T`并登记write access | RECS-P1-15仍open |
| periodic `check_change_ticks`/stored tick clamp | 只有read window clamp；无table/sparse/resource扫描 | 超长会话wrap资格不成立 |
| `Drop`/retire for EventReaderState/EventSubscription/SystemState | 无 | RECS-P1-33、34、57仍open |
| RemovedComponent store update/retention/clear | 无 | RECS-P0-02仍open |
| worldful system worker execution | worker path调用`take_worldless_native_scene_systems` | RECS-P1-35、68、70仍open |
| dependency edge进入worker ready barrier | worker batch只检查`native_conflicts.systems_conflict` | RECS-P1-37仍open |
| trybuild/Miri/loom/fuzz under ECS | 未发现受管qualification suite | RECS-P1-28、71与P2-13/14仍open |

### 2.5 Runtime08 open failure artifact

协调器中Runtime08仍保留多项open failure，包括archetype columnar storage、single-archetype bundle transaction、dense deferred command buffer、bounded event/message lifecycle、lazy change detection、observer indexed dispatch、fixed storage/stable query index、batch mutation clone transaction、dynamic property generation和scene binding generations。当前源码对其中若干项已有明显进展，但review会话不拥有其implementation closure；必须由Runtime08 owner用当前源码、测试和性能证据逐项关闭，不能因本篇描述“基础可保留”而批量归零。

## 3. 当前真实产品链

### 3.1 World数据面

```text
World
  -> entities + entity_dense_rows + kinds + dynamic JSON components
  -> EntityRegistry(slot index, u32 generation, stable EntityId mapping)
  -> ComponentRegistry(local ComponentId, TypeId/string source, storage policy)
  -> ArchetypeIndex(signature -> ArchetypeRecord -> columnar table)
  -> ComponentStorage(sparse values only)
  -> Schedule + query/system state + command queue
  -> EventStore + MessageStore + RemovedComponentEvents + ObserverStore
  -> derived-state indices/cache/diagnostics
```

fixed scene components的runtime owner现在确实是generic table/sparse store。`fixed_components.rs`中的HashMap snapshot是clone/serde/projection rebuild期间的临时transport，并非第二个长期live owner；这项收敛应保留。仍未收敛的是stable entity Vec/Map、`kinds`、dynamic JSON map与ECS presence/storage之间的多路一致性，以及clone/serde时“先抽取再整体重建”的全量成本。

### 3.2 Query与system run

```text
register native system
  -> SystemParam::init_state(World, access)
  -> SystemState(state, access, last_run)
run
  -> advance World change tick
  -> install active tick
  -> unsafe SystemParam::get_param(*mut World, state)
  -> callback
  -> diagnostics + worker command merge
```

QueryState按component access编译matched archetype plan，增量追加新archetype，并记录table column slot/sparse binding。system Query则持有raw `*mut World`与`*mut QueryState`。当前access metadata能发现同一system参数冲突和system间读写冲突，但它不是proof-carrying capability，无法从类型上阻止声明和实际访问分离。

### 3.3 Schedule与worker

```text
SceneScheduleStagePlan
  -> topological order(order + before/after + sets)
  -> ScheduleConflictGraph(pairwise access conflicts)
SceneScheduleRunner
  -> internal system: exclusive World
  -> worldful native system: exclusive World
  -> worldless native system: JobScheduler::join + WorkerCommandBuffer
  -> runtime/plugin system: LevelSystem + mutable World context
  -> ApplyDeferred barriers
```

新增clock domain使native/runtime system能够选择Virtual或Real，并拒绝fixed-loop real-time注册，这是有效进展。但是worker readiness并未使用拓扑dependency edge；只有发生data conflict时才flush，故显式A before B但二者无冲突时仍可并行。真正访问World的system也仍全部串行。

### 3.4 Deferred与事件链

结构命令经过typed staging、descriptor import、final-state preflight、single-row publication与error aggregation；小payload进入packed inline arena，worker lane用stable compiled key归并。generic command仍可持有任意`FnOnce(&mut World)`，panic时只清理剩余queue和arena，已经commit的前序副作用保留。

Event是current/next双缓冲并有capacity shrink debounce；Message是`VecDeque`并有默认1,024 entries、256 KiB、600 frames retention和drop metrics；ObserverStore按lifecycle/event/entity-event建立indexed Arc bucket。这三项是可复用基础。RemovedComponent则未进入同一channel生命周期，Event reader/subscription也仍需手工disconnect，observer仍同步递归获得`&mut World`。

## 4. 当前源码逐域事实

### 4.1 Entity、World identity 与 component schema

可保留：`EntityRegistry`分离public stable ID和internal `(index,generation)`，despawn后更新generation并复用slot；`StableEntityLocation`同时携stable/internal/location；component table layout记录Rust layout并在dense column中集中drop/move函数。

仍不完整：

- public `EntityId`仍只是`u64`，不含World owner/epoch；World replacement或跨World缓存可能静默命中同值。
- 新slot以`slots.len() as u32`分配，没有checked capacity；generation在`u32::MAX`后回到1，保留ABA路径。
- `ComponentDescriptor`只有local id、type name、storage type与Rust `TypeId`/dynamic string source，没有stable schema key、version、layout fingerprint、provider generation、codec与migration。
- dynamic component强制SparseSet，string identity与JSON value继续承担runtime事实；plugin reload/archive无法证明schema compatibility。
- `ResourceStore`、`ComponentStorage`、`EventStore`、`MessageStore`、`ObserverStore`等Clone/PartialEq仍通过清空或恒真规避真实语义；World clone会有选择地重建行为，API形状没有明确表达unsupported participants。

### 4.2 Archetype、table、sparse storage 与 bundle transaction

可保留：Archetype signature去重排序；`by_signature`和`by_component`提供反向索引；table column拥有集中raw allocation/drop/swap-remove；dense值只由ArchetypeTable拥有，ComponentStorage只拥有sparse值；bundle insert/remove/spawn/despawn可先构造detached artifact，validate final state和commit invariant，再发布；transferred descriptor import保持target-local ComponentId。

仍不完整：

- archetype generation等同`records.len() as u64`，membership generation没有World epoch rollover协议。
- dense raw column、iterator raw pointer与transfer path的unsafe proof分散，没有统一WorldCell/column lease模型或Miri证据。
- sparse locator为按internal entity index扩张的dense vector，高水位没有page retirement/compaction policy。
- preflight没有provider generation/schema migration lease；allocation/OOM、observer panic、plugin unload和schema replacement不能保证commit infallible。
- `ArchetypeIndex::PartialEq`仍恒true，snapshot/World equality无法发现topology差异。
- transaction diagnostics以aggregate计数为主，没有owner/system/plugin budget、failure point和rollback receipt。

### 4.3 Query、access 与 borrowing

可保留：QueryAccess能拒绝同一query中的read/write或write/write冲突；QueryState编译table slot与sparse binding；新增archetype可增量追加；cached direct path避免逐entity dynamic downcast；`Mut<T>`按实际`DerefMut/as_mut/into_inner`惰性标记changed；unique many query对N>16改为stack array排序，避免O(N²)。

仍不完整：

- **RECS-P0-01仍open**：共享`Query::iter/count/is_empty/iter_combinations`可变刷新同一QueryState。
- QueryState/SystemState均不保存WorldId/schema/provider generation；相同local ComponentId在错误World中可能偶然命中。
- `QueryData for &mut T`的read item与write access语义矛盾；tuple没有`QueryMutData`，常见多组件可变query不能组成。
- tuple arity在bundle/query/filter/system/ParamSet多处手写到8，复杂产品system只能嵌套或拆facade。
- cache hit仍逐个refresh全部matched plan membership；schema/provider变化不参与key。
- many/combinations路径建立candidate/location Vec；combination没有`n choose k` admission、item/byte/deadline budget。
- 没有chunk partition token、parallel query iterator、compile-fail alias suite或Miri reallocation regression。
- 大N去重保留“请求顺序第一个duplicate entity”，但error仍没有duplicate index；ignored benchmark不是跨机器受管performance gate。

### 4.4 Change detection

可保留：`ChangeTickWindow`以wrapping age比较并夹紧过旧last_run；component/resource保留added/changed ticks；`Mut<T>`只在真正取得mutable access时更新；query记录candidate/matched与change scan统计。

仍不完整：代码定义`CHECK_TICK_THRESHOLD`和`MAX_CHANGE_AGE`，但没有像Bevy那样周期遍历table/sparse/resource并夹紧stored ticks。只夹紧reader window无法在tick完整wrap后维持长期正确性。active tick的恢复也不是RAII：`SystemState::run`先设置active tick，再在`catch_unwind`之外构造param；param构造panic可遗留错误active tick。

### 4.5 SystemParam、schedule 与 parallel executor

可保留：SystemParamAccess覆盖component/resource/event/message/command；ParamSet限制分支顺序借用；topological stage plan能检测missing reference/cycle；conflict graph能表达conservative World access；worker callback panic会丢弃lane并恢复registry；clock domain区分real/virtual；native schedule diagnostics记录main/worker callback、ready delay、batch与控制面buffer。

仍不完整：

- SystemParam/SystemState没有通用retire hook，event reader、plugin lease和future observer state无法在system unregister/rebind时回收。
- worldful Query/Res/Event/Message system不实现WorldlessSystemParam，production worker仍只处理`() / Commands / Local`等无World参数。
- standalone `ScheduleParallelExecutor`与`SceneScheduleRunner`不是同一真实executor，测试DAG和产品DAG仍有语义分叉。
- before/after拓扑顺序没有变成worker completion dependency；不冲突system会进入同一batch。
- conflict graph按system pair O(S²)编译，access是sorted Vec；没有condition、ambiguity policy、partition或affinity capability graph。
- virtual pause在loop入口直接skip全部InternalSceneSystem，`UpdateEvents`、`ApplyDeferred`与pending scene maintenance停滞；现有测试明确接受pending state留到unpause。
- 每个worker flush仍分配system IDs、systems、timings和command buffer引用Vec；新增diagnostics只测量成本，没有将其变为compiled persistent scratch。

### 4.6 Deferred commands 与事务

可保留：inline arena减少small-command boxing；fallback和growth有metrics；worker lane有deterministic key；deferred spawn先统一reserve token；连续structural command进入batch preflight；panic时剩余payload按storage kind正确drop并清空resolution。

仍不完整：

- generic `Command/FnCommand`仍允许任意opaque World closure，无法预声明access、cost、schema、journal或rollback。
- command panic前已执行的generic command和已完成structural batch不会回滚，也没有partial applied range/poison receipt。
- opaque command切断structural batch，事务边界由实现偶然决定而不是caller contract。
- deferred token/ordinal/Message/Observer等allocator exhaustion仍混用expect、saturate、wrap和unchecked increment。
- command quota没有world/system/plugin层级entry/byte/alignment预算与typed rejection。
- worker callback的外部副作用不受DeferredWorld约束；merge失败只能丢command，不能撤销外部effect。

### 4.7 Event、Message、Removed 与 Observer

可保留：Event双buffer有明确frame update；EventCursor逐item推进，部分iterator Drop不会吞尾部；active-channel worklist避免扫描全部空channel；Message retention有entry/byte/age和drop count；ObserverStore按触发类型和entity索引，并在despawn时detach/restore observer bucket支持事务。

仍不完整：

- **RECS-P0-02仍open**：RemovedComponent按历史总量永久保留，每次reader read还复制新的EntityId Vec。
- Event queue单帧send仍无entry/byte budget；generation使用saturating add，EventTypeId用`channels.len() as u32`。
- EventReaderState在init时`register_reader`，SystemState Drop不disconnect；手动EventSubscription Drop也不能接触store，reader_count可永久泄漏。
- Event observer在send boundary同步运行，bool rejection不阻止事件入队，validation/tap/delivery语义混合。
- MessageId exhaustion直接expect panic；Message generation饱和；MessageCursor在创建iterator时把cursor直接推进到queue末端，部分消费会吞未读尾部。
- Observer callback同步获得`&mut World`，可递归触发，没有depth/work/deadline budget；panic没有按ObserverId/owner隔离；ObserverId unchecked，unregister没有in-flight lease。

### 4.8 Product integration

精确caller扫描得到：`register_native_system` 43个命中中production 8个，均主要是registry/API定义；`register_worldless_native_system` 10个命中中production 5个，也均为framework入口。`query::<` 45个命中中production只有5个，集中在animation `parameter_apply.rs`的独立查询；EventReaderParam/MessageReaderParam的production命中是re-export与定义，不是first-party system workload。

transform、hierarchy、active、event maintenance仍由`InternalSceneSystem`直接调用World函数；render path也直接要求derived state。plugin注册已经能声明sets、constraints、order、access和clock domain，但这些metadata没有生成不可伪造的World view。native/dynamic plugin仍可声明窄access却通过host callback走更宽路径，或退化为conservative World writer。

所以现有API tests不能证明ECS已被animation、physics sync、visibility、render extract、AI/nav等第一方产品链采用，更不能证明worker speedup。Runtime60的product adoption与competitive benchmark门禁仍是硬阻断。

## 5. 两项 P0 当前证据

### 5.1 RECS-P0-01：共享Query接收器制造并存可变别名

`zircon_runtime/src/scene/ecs/system/query.rs`中的`Query`持有`world: *mut World`和`state: *mut QueryState`。`iter(&self)`先构造`&World`，再把`self.state`构造为`&mut QueryState`并调用cache-refreshing iterator；`iter_combinations(&self)`、`count(&self)`和`is_empty(&self)`重复同一模式。注释声称“run item uniquely owns system state”，但safe API的receiver没有编码该唯一性。

| 证据 | 当前状态 | 必须修复为 |
|---|---|---|
| 两个共享`&Query`调用可同时借用同一raw state | Open P0 | cache update需要`&mut self`，或run前独占prepare后只暴露immutable compiled plan |
| 首个iterator借用cached plan Vec时第二次refresh可能extend/reallocate | Open P0 | iterator持有generation-pinned immutable plan/chunk lease |
| 无Miri/compile-fail/reallocation regression | Open gate | safe alias regression、trybuild和Miri纳入受管Windows/WSL资格 |

不能仅把unsafe注释写得更长；必须让Rust receiver、WorldCell proof与executor生命周期共同排除第二次可变refresh。

### 5.2 RECS-P0-02：RemovedComponentEvents永久累积

`zircon_runtime/src/scene/ecs/removal.rs`当前只有`push/push_type_id/events/registered_type_names`。每个TypeId的Vec只增不减；`RemovedComponentReader::read`把未读slice复制到新Vec后把自己的cursor移到末尾，`clear`也只移动该reader cursor。World/schedule没有removed-channel maintenance。

| 证据 | 当前状态 | 必须修复为 |
|---|---|---|
| 正常remove/despawn使World retained entries单调增长 | Open P0 | generation page/ring + entry/byte/age hard budget |
| 无reader和慢reader都无法触发store reclamation | Open P0 | owner-qualified cursor ack、lag policy与overflow receipt |
| clone会复制全部历史removed event | Open P0 | snapshot明确选择drop/retain window，不复制无限历史 |
| 无million-mutation RSS slope test | Open gate | bounded mutation soak验证entries/bytes/RSS守恒 |

建议与Event/Message共用channel/cursor/budget基础，但不要简单套Event双buffer：Removed consumer需要明确“本帧可见性、慢reader、despawn顺序和overflow”合同。

## 6. Runtime60 issue 状态刷新

| Runtime60 owner | 当前状态 | 当前源码新增基础 | 为什么仍不能关闭 |
|---|---|---|---|
| P0-01 Query alias | **Open P0** | query plan/cache更完整 | 共享receiver仍制造同一state的`&mut` |
| P0-02 Removed retention | **Open P0** | Event/Message已有生命周期基础 | removal store仍永久Vec |
| P1-01..12 identity/schema/storage | **Open** | columnar ownership、bundle transfer较完整 | 无World owner/epoch、checked exhaustion、stable schema/provider generation；Clone/Eq仍欺骗 |
| P1-13..17 query identity/mutable tuples/arity | **Open** | cached direct与change wrappers可用 | 无WorldId、`&mut T` read语义矛盾、无mutable tuple、arity仍8 |
| P1-18 unique validation | **Partial** | N>16改为stack sort，避免O(N²) | error无duplicate index；benchmark ignored且无manifest |
| P1-19..28 query allocation/budget/WorldCell/qualification | **Open** | plan统计更细 | 无budget、chunk lease、parallel iterator、trybuild/Miri |
| P1-29..41 SystemParam/schedule/executor | **Open** | panic lane cleanup与conflict diagnostics存在 | 无run guard/retire、worldful并行、统一executor、dependency-ready graph |
| P1-42 pause/clock policy | **Partial** | native/runtime system新增Real/Virtual domain | internal maintenance无domain且virtual pause全部skip |
| P1-43 schedule clone truth | **Open** | runtime registry能重建部分descriptor | World/Schedule clone仍丢callback语义 |
| P1-44 worker control allocations | **Partial** | 新增temporary buffer count/bytes diagnostics | 每次flush仍创建多个Vec |
| P1-45..53 deferred transaction/schema/quota | **Open** | structural batch preflight和arena明显增强 | opaque command、partial commit、无schema/journal/hierarchical quota/provider lease |
| P1-54 command diagnostics | **Partial** | frame diagnostics可发布queue/bundle/native指标 | 无per-system/plugin top offender和budget outcome |
| P1-55..66 event/message/observer lifecycle | **Open** | Event双buffer、Message retention、observer index可保留 | ID/generation exhaustion、reader leak、message ack、observer recursion/panic仍在 |
| P1-67 first-party maintenance | **Partial** | derived-state新增细粒度work counters | maintenance仍是exclusive direct World函数 |
| P1-68..72 product adoption/ABI/tests/benchmark | **Open** | animation有少量standalone query使用 | 没有first-party executor matrix和竞争性artifact |
| P2-01..15 consistency/maintenance/tests | **Open** | 局部模块拆分与注释改善 | identity、tuple生成、unsafe docs、model/fault suite仍缺 |
| P2-16 performance acceptance | **Partial** | unique validation新增ignored P95证据 | 无硬件/OS/build/workload manifest或CI regression policy |
| P2-17..18 platform/soak | **Open** | 无可关闭新证据 | 无跨平台核数矩阵和100h RSS slope |

状态计数保持：**Open P0 2；Closed 0；Partial P1 5；Partial P2 1；其余P1/P2 Open**。Partial不减少Runtime60登记总数，也不允许跳过对应gate。

## 7. 参考引擎交叉证据

### 7.1 Unreal Mass：主参考是完整处理链，不是命名

Unreal Mass的`FMassEntityManager`集中entity/archetype mutation并把processor mutation路由到deferred command；`FMassArchetypeVersionedHandle`显式携archetype version；`FMassEntityQuery`保存requirements与per-archetype mapping，提供`ForEachEntityChunk`和`ParallelForEachEntityChunk`，并为并行job提供独立command buffer；`FMassObserverManager`用creation context/observer lock把通知延迟到结构事务完成；phase manager维护dependency-solved processing graph、dynamic processor注册/注销和phase completion event。

Zircon应吸收“requirement -> versioned plan -> chunk execution -> deferred commit -> observer publication -> phase dependency”整链。不能只复制Mass类名，也不应照抄Unreal的global/UObject ownership、check/assert exhaustion或历史single-thread兼容形态。

### 7.2 Bevy ECS：Rust safety与executor的最低参考

Bevy QueryState显式保存`WorldId`、archetype generation、matched table/archetype bitset、matched storage与FilteredAccess；safe update先validate world。`UnsafeWorldCell`集中表达World别名责任；WorldQuery/SystemParam通过state与fetch分离构造访问；ParIter按storage batch切分；multi-threaded executor维护ready/running/completed状态并以dependency completion解锁successor；change tick有周期维护而不是只夹紧reader window。

Zircon不需要机械复制Bevy所有trait和panic语义，但同为Rust引擎，不能低于其World identity、集中unsafe cell、mutable tuple、chunk partition、dependency-ready executor和change-tick长期正确性基线。

### 7.3 Fyrox 与 Godot：identity/borrow和生命周期反例

Fyrox pool的Handle把index和generation组合，pool负责spawn/free/borrow校验，MultiBorrow显式拒绝重复mutable handle；它适合作为stable handle与multi-borrow的补充参考，不是archetype ECS模板。Godot ObjectID提供对象身份域，MessageQueue把push/flush和线程同步放在明确owner中，WorkerThreadPool区分TaskID/GroupID、completion、collaborative wait和runlevel shutdown；它们说明queue/job的lifecycle与shutdown必须是一等合同。

Zircon不能照抄Godot的global singleton、unchecked monotonic ID或Variant/message形态，也不能把Fyrox graph pool直接当columnar ECS；应吸收其明确invalid/stale检查、multi-borrow失败和任务终止协议。

### 7.4 Unity Graphics：render-facing并行访问必须显式

Unity Graphics GPUDriven jobs大量使用`IJobParallelFor/Batch`、`NativeArray/NativeList`、`ReadOnly/WriteOnly/NoAlias`、parallel writer和显式job range，把render instance transform、visibility、probe和draw list工作切成可调度batch。部分hot job会关闭container safety restriction，这反而说明关闭检查前必须由range partition、dependency和ownership提供外部证明。

Zircon当前没有ECS chunk lease或render extract job dependency，不能仅凭JobScheduler::join声称等价；未来Graphics owner应消费runtime生成的immutable/partitioned extract artifact，而不是让renderer持raw World或重新扫描dynamic JSON。

### 7.5 差距矩阵

| 能力 | Zircon当前 | Unreal Mass | Bevy ECS | 结论 |
|---|---|---|---|---|
| World/query identity | local IDs，无World binding | manager/versioned handle | QueryState WorldId | 必须先补WorldIdentity/schema generation |
| mutable multi-component query | 无QueryMutData tuple | requirement mapping/chunk views | tuple WorldQuery | 不能进入worldful并行前置 |
| parallel World execution | 仅worldless callback | chunk parallel + per-job commands | dependency executor + par iter | 当前不是真实ECS并行 |
| dependency ordering | topo sequence，worker可越过edge | processing graph completion | ready/running/completed | dependency必须成为ready条件 |
| deferred transaction | structural preflight，opaque partial commit | deferred buffer + observer context | Commands/apply_deferred | 保留基础，收紧opaque path与receipt |
| removed/event lifecycle | Removed无界；Event两帧；Message有界 | observer manager/context | event/observer schedules | Removed P0优先 |
| product adoption | maintenance direct World | processors走Mass query | first-party systems走ECS | 必须迁移真实产品链 |
| qualification | unit/perf counters | mature product/trace生态 | compile/safety/perf生态 | 无Miri/soak/competitive artifact不能宣称完成 |

## 8. 目标架构

Runtime60目标架构不变：

```text
WorldIdentity(id, epoch)
  -> EntityAllocator(live key, stable guid, checked exhaustion)
  -> ComponentSchemaRegistry(stable key, version, layout, provider generation)
  -> ArchetypeStore(table/sparse pages, structural delta, capacity receipt)
  -> QueryPlan(world/schema/archetype generations, compiled bindings)
  -> WorldCell(access token, chunk lease, thread/alias proof)
  -> SystemMeta(access, dependency, condition, affinity, owner)
  -> ScheduleGraph(ready/running/completed + barriers)
  -> EcsExecutor(chunk tasks + deterministic DeferredWorld lanes)
  -> DeferredWorld(typed operation, quota, prepare/commit/receipt)
  -> EventRegistry(Event/Message/Removed generations + bounded cursors)
  -> ObserverGraph(indexed dispatch + depth/work/panic/retirement)
  -> EcsDiagnostics(world/system/plugin top-N + qualification artifacts)
```

关键不变量：

1. 所有live handle先验证WorldIdentity，再验证slot/schema/provider generation。
2. 所有unsafe fetch只从WorldCell/lease产生，调用者不能从metadata自行伪造raw pointer。
3. Schedule dependency和data conflict共同决定ready set；“排序更早”不等于“允许并发开始”。
4. worker-safe callback只能修改lease内数据或写DeferredWorld，外部effect在commit成功后发布。
5. Event/Message/Removed/Observer注册都是owner-qualified generation lease，Drop/unregister能等待in-flight并回收。
6. Scene runtime truth只有一个；Editor、plugin、render、serialization都消费descriptor/snapshot/delta，不持第二live store。

## 9. 依赖顺序重构计划

### M0：立即止血两项 P0

- RECS-P0-01：让cache refresh只接受独占Query/QueryState；run前prepare immutable plan；补双iterator、plan reallocation、archetype append、Miri与compile-fail回归。
- RECS-P0-02：Removed进入generation page/ring，配置entry/byte/age硬上限、cursor ack/lag和overflow receipt；补无reader、慢reader、多reader、despawn batch与百万mutation RSS slope。
- 在M0完成前禁止扩大worldful worker并行，避免把当前alias风险送入多线程。

### M1：WorldIdentity、allocator 与 schema

- 分离`LiveEntityKey(index,generation)`、`WorldIdentity(id,epoch)`和persistent `SceneEntityGuid`。
- slot/index/generation/ComponentId/EventTypeId/MessageId/ObserverId/DeferredToken全部checked exhaustion；耗尽slot退休或关闭admission。
- ComponentSchema增加stable key、version、layout/drop/move ABI、provider generation、storage policy、codec与migration。
- 删除恒真PartialEq和清空Clone；snapshot/clone用显式participant policy和unsupported receipt。

### M2：ArchetypeStore 与结构事务

- 把table/sparse/entity location/schema lease统一为结构delta plan；preflight admission容量、provider generation、observer publication和所有affected archetype。
- sparse locator改paged/compact policy，输出table/sparse/locator实际bytes。
- commit boundary使用不可失败publication或明确poison/compensation receipt；opaque closure不得插入atomic structural window。
- 将分散raw column safety收敛到审计过的storage cell/row transfer primitive。

### M3：QueryPlan、WorldCell 与 SystemState

- QueryPlan绑定World/schema/provider/archetype generations，safe API跨World返回typed mismatch。
- 统一read-only和mutable query语义；生成tuple QueryMutData/SystemParam/ParamSet宽度，compile-time拒绝重复`&mut T`。
- compiled plan直接携column slots/bitsets/chunk ranges；many/combinations使用bounded scratch与admission。
- SystemState加入RAII run guard和retire/rebind协议，任何param/callback/diagnostics/merge panic都恢复tick、lane与reader lease。

### M4：ScheduleGraph 与真实 EcsExecutor

- 合并ScheduleParallelExecutor与SceneScheduleRunner，测试和产品只走一个DAG executor。
- access edge、before/after、condition、set、exclusive/foreign/affinity共同编译ready/running/completed状态。
- WorldCell发放不重叠archetype/table chunk lease，first-party Query/Res/Event system能够真实并行。
- compiled batch持久保存indices和scratch；稳定帧控制面heap allocation为0，overflow有typed fallback/receipt。
- internal maintenance声明clock/lifecycle domain；pause时明确推进event/message/removal和deferred policy。

### M5：DeferredWorld 与 lifecycle channel

- public deferred API改为typed operation，声明owner/schema/access/cost/deterministic key和可选journal codec。
- hierarchical quota覆盖world/system/plugin entry/byte/alignment；prepare全窗口，commit返回applied range和failure class。
- Event/Message/Removed统一generation、entry/byte/age budget与逐item/batch ack contract。
- reader/observer registration使用generation lease；observer dispatch queue限制depth/work/deadline并隔离panic。

### M6：迁移第一方产品链

- 先迁transform/hierarchy/active maintenance，再迁animation/physics sync/visibility/render extract，逐项记录为何parallel或exclusive。
- plugin SDK暴露versioned typed query/resource/event descriptor与host-validated chunk callback，声明直接生成capability。
- Editor只消费runtime catalog/snapshot/delta；render只消费immutable extract artifact；删除旧direct World旁路，不保compat shim。

### M7：资格与竞争性证据

- 建立behavior、compile-fail、model/property、Miri、sanitizer、fault、soak、benchmark分层套件。
- 固定真实产品scene、硬件/OS/build/profile/warmup/sample manifest，记录p50/p95/p99、alloc、RSS、cache、worker利用率和结果一致性。
- 与Unreal Mass和Bevy在同实体数、组件分布、mutation率、query mix和worker数下比较；Unity Graphics只用于render extract/job负载。
- “优于Unreal”必须由可复现artifact证明，不能由API数量、单个microbenchmark或内部counter推断。

## 10. 验收门禁刷新

Runtime60的G01-G40全部保留。本轮最先阻断implementation扩张的门禁如下：

| Gate | 当前 | 关闭证据 |
|---|---|---|
| G01/G02 World/ID stale与exhaustion | Fail | cross-World/replacement/rollover模型测试全部fail-closed |
| G03 schema/provider migration | Fail | version/layout/provider/migration archive与plugin reload矩阵 |
| G09 Query alias | **Fail P0** | safe双iterator不可构造，Miri reallocation通过 |
| G10 Query cross-World | Fail | QueryPlan world/schema mismatch typed rejection |
| G11 mutable tuple | Fail |合法tuple运行，重复mutable compile-fail |
| G15 chunk/change/deferred determinism | Fail |分区地址不重叠、tick和merge重放一致 |
| G17 dependency DAG | Fail |无冲突before/after仍严格等待predecessor completion |
| G19 first-party worldful parallel | Fail |真实transform/animation/physics/extract workload worker trace |
| G21 panic conservation | Fail |param/callback/diagnostics/merge/apply fault injection守恒 |
| G24 pause maintenance | Fail |real/virtual/fixed/pause组合下channel/deferred policy符合声明 |
| G25 Removed boundedness | **Fail P0** |无/慢/多reader均有entry/byte硬上限与RSS slope |
| G28 reader retirement | Fail |system/plugin Drop后reader census归零并等待in-flight |
| G30 observer recursion/panic | Fail |depth/work/deadline与owner terminal receipt |
| G33-G36 product/diagnostics | Fail |第一方adoption matrix与world/system/plugin diagnostics |
| G37-G40 qualification | Fail |Miri/sanitizer/fuzz/platform/100h/competitive artifacts |

### 10.1 必须新增或升级的测试矩阵

| 层 | 必测场景 |
|---|---|
| Identity | cross-World、World replace、slot generation exhaustion、stable GUID roundtrip、provider reload |
| Storage | random spawn/insert/remove/despawn与reference model差分；table/sparse move；allocation/failure-point conservation |
| Query safety | shared iterator reentry、archetype Vec reallocation、mutable duplicate tuple、ParamSet交错、Miri |
| Change tick | threshold clamp、u64 wrap模型、长期未运行system、resource/table/sparse一致性 |
| Schedule | dependency+access混合DAG、condition、panic、cancel/shutdown、1/2/8/32/64 worker deterministic result |
| Commands | opaque path收口、batch prepare failure、partial commit receipt、quota overflow、worker external effect rejection |
| Lifecycle | Event/Message/Removed partial iteration、slow reader、Drop retire、generation exhaustion、observer recursion/panic |
| Product | transform/hierarchy/animation/physics/visibility/extract走同一executor并与旧结果逐帧对照 |
| Performance | archetype churn、stable frame、high sparse index、wide query、command burst、event burst、real product scene |
| Soak | 100h mutation/streaming/plugin reload，entity/component/channel/reader/observer/command守恒与RSS slope |

## 11. 实施约束与 owner 路由

| 工作 | Owner | 协作/阻断 |
|---|---|---|
| 两项P0与ECS combined contract | Runtime60 / Runtime08 implementation | Runtime24 identity、Runtime59 scheduler |
| WorldIdentity/schema/provider generation | Runtime24 | Runtime60、Plugins/Interface owners |
| table/sparse/bundle/dynamic projection | Runtime08 | Runtime05、serialization/dynamic scene owners |
| task pool/shutdown/thread budget | Runtime59 | Runtime60 executor |
| first-party transform/hierarchy/active | Runtime05/08 | Runtime60 access/executor完成后迁移 |
| animation/physics/AI/nav adoption | 各plugin owner | 不得自建第二ECS或raw World bypass |
| render extract/chunk job | Graphics/Render owners | 消费Runtime60 immutable artifact，不反向拥有Scene |
| Editor scene authoring | zircon_editor owner | 只消费runtime schema/snapshot/delta |

实施必须hard cutover：新WorldCell/QueryPlan/EventLease接线后删除旧raw入口和compat re-export；若旧路径暂时无法删除，应把迁移分为更小owner slice，而不是永久保留“双轨都能写World”。触及约1000行的大文件时按domain拆folder-backed module，不能继续把调度、存储、诊断和产品逻辑堆进`World`或单个registry文件。

## 12. 复核限制与未执行项

- 已完成：当前ECS逐文件inventory、baseline diff逐项阅读、World/product caller追踪、92项canonical差距分组复核、两项P0源码重验、26个参考文件交叉对照、物理freeze与fingerprint。
- 未执行：Cargo build/test、Miri、sanitizer、loom、fuzz、fault injection、产品运行、跨平台、soak和benchmark。原因是本轮只修改review文档，且这些命令不能替代尚未实现的门禁。
- 未接管：共享working copy中其他会话对scene、plugin、docs索引和tests的修改；本轮不回滚、不重排、不宣称拥有其实现结果。
- source recheck：Runtime08/60开始修复前必须重新读取两项P0所在文件、所有dirty owner path与open failure artifacts；本篇fingerprint漂移后不能直接作为验收证据。

## 13. 当前状态

- Canonical owner：Runtime60。
- Current-source refresh：Runtime108 / 本篇。
- 新增umbrella差距：P0 0、P1 0、P2 0。
- Canonical状态：P0 open 2；92项差距closed 0；P1 partial 5；P2 partial 1；其余open；40 gates全部未通过。
- Implementation：pending；先完成M0两项P0，再按M1-M7依赖顺序推进。
- 竞争性结论：当前无证据证明Zircon Scene ECS达到或优于Unreal/Bevy/Unity Graphics；目标保留，但只能由同负载、同机器、可复现资格artifact关闭。
