---
title: Runtime Scene ECS Entity、Component、Storage、Archetype、Query、Access、Change Detection、Command、Schedule、Parallel、Event 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime60
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/scene/ecs
  - zircon_runtime/src/scene/ecs/bundle.rs
  - zircon_runtime/src/scene/ecs/lifecycle.rs
  - zircon_runtime/src/scene/ecs/internal_scene_system.rs
  - zircon_runtime/src/scene/ecs/removal.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/archetype/index.rs
  - zircon_runtime/src/scene/ecs/archetype/table/column.rs
  - zircon_runtime/src/scene/ecs/archetype/table/table.rs
  - zircon_runtime/src/scene/ecs/change_detection/change_tick.rs
  - zircon_runtime/src/scene/ecs/change_detection/wrappers.rs
  - zircon_runtime/src/scene/ecs/commands/command_queue.rs
  - zircon_runtime/src/scene/ecs/commands/inline_command_arena.rs
  - zircon_runtime/src/scene/ecs/commands/structural.rs
  - zircon_runtime/src/scene/ecs/commands/worker_command_buffer.rs
  - zircon_runtime/src/scene/ecs/component/registry.rs
  - zircon_runtime/src/scene/ecs/entity/registry.rs
  - zircon_runtime/src/scene/ecs/events/cursor.rs
  - zircon_runtime/src/scene/ecs/events/store.rs
  - zircon_runtime/src/scene/ecs/messages/cursor.rs
  - zircon_runtime/src/scene/ecs/messages/store.rs
  - zircon_runtime/src/scene/ecs/observer/store.rs
  - zircon_runtime/src/scene/ecs/query/query_data.rs
  - zircon_runtime/src/scene/ecs/query/query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cache.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only_cached.rs
  - zircon_runtime/src/scene/ecs/resource_store/store.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/store.rs
  - zircon_runtime/src/scene/ecs/system/events.rs
  - zircon_runtime/src/scene/ecs/system/query.rs
  - zircon_runtime/src/scene/ecs/system/removed_components.rs
  - zircon_runtime/src/scene/ecs/system/system_state.rs
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/scene/level_system
tests:
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/scene/tests/ecs_commands.rs
  - zircon_runtime/src/scene/tests/ecs_commands
  - zircon_runtime/src/scene/tests/ecs_events_messages.rs
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/ecs_identity_storage
  - zircon_runtime/src/scene/tests/ecs_observers_messages.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_query
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - zircon_runtime/src/scene/tests/ecs_query_structure
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_schedule
  - zircon_runtime/src/scene/tests/ecs_systems
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_tooling/21-unsafe-rust-ffi-native-memory-thread-affinity-panic-unload-safety-governance-review.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
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
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 60 · Runtime Scene ECS Entity、Component、Storage、Archetype、Query、Access、Change Detection、Command、Schedule、Parallel、Event 与 Product Integration 工程化差距

## 1. 结论

Zircon 的 ECS 已经不是临时 `HashMap<EntityId, Box<dyn Any>>` 原型。当前实现具有 generation-aware 内部实体槽、stable public entity 映射、archetype-owned contiguous table columns、sparse-set dense arrays、stable query order、compiled archetype query plan、lazy change detection、结构命令 preflight、64 KiB aligned inline command arena、稳定 worker command merge、active event channel worklist、有界 message retention、copy-on-write observer bucket 和访问冲突图。这些基础方向正确，后续重构必须保留，而不是以“工程化”为名退回全局锁、全量扫描或每组件散列存储。

但本轮发现两项此前未被其他专项唯一登记的源码级 P0。第一，system `Query::iter(&self)`、`iter_combinations(&self)`、`count(&self)`和`is_empty(&self)`把共享的`*mut QueryState`转成`&mut QueryState`，而返回的 iterator 同时借用`cached_archetype_plans`切片；调用者可在第一个 iterator 存活时再次通过共享`&Query`进入缓存更新，形成并存`&mut`和切片别名，新增 archetype 时还可能重分配 plan vector。第二，`RemovedComponentEvents`按组件类型只向`Vec`追加，World 中没有 update/clear/retention，reader 只前移私有 cursor，长期 spawn/remove/despawn 会让内存随历史总量永久增长。

此外，`QueryState/SystemState`不绑定 World identity，mutable tuple query缺失，真正访问 World 的系统仍不能进入生产 worker 并行，显式`before/after`依赖未进入 worker batch barrier，暂停虚拟时间会跳过包括事件维护在内的全部 internal system，generic command panic只丢弃剩余命令而保留已应用副作用，event reader registration没有RAII注销。它们说明当前实现是多个成熟局部件的集合，还不是能安全承载插件卸载、大型World、多核调度和长期运行的统一 ECS kernel。

本轮登记 **2项P0、72项P1、18项P2和40项验收门禁**。目标架构是`WorldIdentity + EntityAllocator + ComponentSchemaRegistry + ArchetypeStore + QueryPlan + WorldCell + SystemMeta + ScheduleGraph + EcsExecutor + DeferredWorld + EventRegistry + ObserverGraph + EcsDiagnostics`。本轮只做静态 review 和文档总账，没有修改 production、tests、Cargo、ABI 或参考源码；没有运行 Cargo、Miri、sanitizer、fuzz、真实多线程 World、soak、profiler 或 benchmark，因此不能据此宣称性能达到或超过 Unreal。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint / 说明 |
|---|---:|---|
| `scene/ecs`完整 kernel | 143 / 22,031 / 701,817 / 42 | SHA-256 `2142a8d985a23a34deb5ecfbb3f8cbfe60ef111e0249a84939fc3e305a424de6`；204行含`unsafe` |
| World、module、LevelSystem与event mirror产品接线 | 93 / 24,796 / 940,182 / 70 | SHA-256 `f1bec6a150cfd9c073779d0049ff82799186eccead16de9db5f3c517434bd056` |
| focused ECS tests与直接support | 44 / 10,614 / 400,945 / 269 | SHA-256 `67e63eff0369e106f5691c58ee39b764de909bdece5266e32aeffc6dd762c8e7`；23个文件含`include_str!`，这些文件合计200个test属性 |
| Unreal Mass、Bevy ECS、Fyrox、Godot与Unity Graphics references | 53 / 53,524 / 1,996,738 / 185 | SHA-256 `2c207fedfc1fa1f8282c42def53f66c6e8bef5974a30428a0c6d677ab5f142a6` |

fingerprint 算法延续 Runtime59：相对路径转`/`并排序去重，以`path|lowercase per-file SHA-256`组成LF连接且无末尾LF的UTF-8字节，再计算SHA-256。focused tests 集合为顶层`ecs_*`、`component_structure`、`ecs_systems`文件，加`component_structure/`、`ecs_commands/`、`ecs_performance_acceptance/`和`world_basics/`直接support；reference集合包含frontmatter关键路径及其同机制实现文件。fingerprint冻结本轮实际核对集合，不是未来artifact identity。

基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。工作树有大量其他会话/用户改动；本轮生产文件保持只读，报告按当前working tree读取。共享代码仍会变化，因此`source_recheck_required`保持true。

### 2.2 所有权与去重

- Runtime05继续拥有World clone/serde数据完整性、层级/派生状态、render extract、dynamic query和scene partition父边界；本篇只深审ECS kernel机制。
- Runtime08 implementation plan拥有columnar storage、bundle transaction、lazy tick、command arena、bounded events/messages与observer indexed dispatch的既有实施；本篇纠正旧状态并登记剩余合同。
- Runtime03 implementation plan拥有stage/frame loop与生产并行执行父边界；本篇新增的query共享借用P0和依赖未进入worker batch barrier是current-source具体缺陷。
- Runtime24拥有全仓identity/generation/owner/epoch规范；本篇负责Entity、Component、Archetype、QueryState、SystemState、Event/Message/Observer ID的ECS落地，不重复其父P0。
- Runtime22拥有clock、fixed-step、determinism和replay；Runtime59拥有通用JobScheduler/task substrate；本篇只规定ECS schedule graph、chunk execution和deferred apply。
- Tooling21已路由ECS unsafe safety model和Miri/sanitizer证据，但用户要求暂停tooling优化；本篇只引用owner，不扩写tooling计划。
- Unity Graphics只用于SoA/native container、显式读写和JobHandle依赖对照，不把本地Graphics package误称为完整Unity ECS源码。

## 3. 当前真实产品链

### 3.1 World数据面

```text
World
  -> EntityRegistry(index, generation, stable EntityId map)
  -> ComponentRegistry(ComponentId, storage kind, Rust TypeId/dynamic name)
  -> ArchetypeIndex(signature -> ArchetypeRecord)
       -> ArchetypeTable(entity rows + erased contiguous columns)
  -> ComponentStorage(sparse-only dense arrays + sparse locator)
  -> ResourceStore / EventStore / MessageStore / ObserverStore
  -> RemovedComponentEvents(TypeId -> ever-growing Vec)
```

Dense组件在archetype迁移时先准备目标signature和row，再移动column并修复swap-remove位置；sparse组件保留generation-aware locator。`StableQueryOrderIndex`和物理row分开，使结构移动不直接破坏公开确定性顺序。这是当前最值得保留的数据面。

### 3.2 Query、SystemParam与change tick

```text
SystemState::new(world)
  -> init Param state + compute access
  -> QueryState::new(world)
       -> compile matching archetype plans

SystemState::run(world)
  -> advance World change tick
  -> construct params through raw World pointer
  -> callback
  -> diagnostics + worker command merge
```

`Mut<T>`只在真正发生可变解引用/赋值时标记changed，wrap-aware tick window也能限制过旧change age；但QueryState和SystemState没有World identity，且system `Query`以raw pointer绕开共享接收器的独占要求。访问描述存在，不等于unsafe借用证明已经闭合。

### 3.3 Schedule与worker路径

```text
SceneScheduleStagePlan
  -> topological sorted step sequence (order/before/after)
  -> SceneScheduleRunner
       -> internal systems: main thread World
       -> native World systems: main thread World
       -> worldless worker-safe native systems: JobScheduler::join
            -> deterministic WorkerCommandBuffer merge
       -> runtime systems: serialized LevelSystem path
```

生产worker batch只能运行sealed `WorldlessSystemParam`，当前主要是`() / Commands / Local`及tuple；`Query/Res/Event/Message`系统仍串行取得`&mut World`。更严重的是batch只询问`ScheduleConflictGraph::systems_conflict`，没有把已用于拓扑排序的显式依赖edge作为barrier，两个数据不冲突但声明顺序的worldless callback可并发执行。

### 3.4 Commands、events与observers

结构命令有typed preflight、packed arena、fallback metrics和stable worker key；generic command仍是任意`FnOnce(&mut World)`。EventStore采用current/next双buffer和active-channel worklist，MessageStore有entry/byte/age上限，ObserverStore以copy-on-write bucket允许dispatch期间修改注册表。RemovedComponentEvents没有进入这套双buffer或retention模型，是独立的永久日志。

### 3.5 产品接入真实性

first-party transform、hierarchy、active、events等maintenance仍以`InternalSceneSystem`直接操作World；typed `Query/Res/EventReader`主要由测试和plugin/native registration surface认证。生产调度报告可以显示parallel batch，但这些batch不能读写World组件。当前证据只能证明API和局部executor存在，不能证明真实gameplay/render workload已经数据并行。

## 4. 可保留基础

1. Archetype table的aligned erased allocation、typed drop、row move与swap-remove修复应保留，并补proof与工具资格。
2. Sparse set的dense values/entities和generation-aware locator优于per-component HashMap，应补回收与high-water策略。
3. Stable query order与物理storage分离是确定性和cache locality可以兼容的正确方向。
4. QueryState按archetype编译column slot/sparse binding并增量吸收新archetype，避免逐实体反射查询。
5. `ChangeTick`wrap-aware age clamp及`Mut<T>`lazy marking方向与Bevy一致。
6. Bundle和dynamic spawn已有prepare/validate/apply分层，适合作为完整DeferredWorld事务的基础。
7. 64 KiB/64-byte aligned inline command arena、192-byte inline payload和4 MiB active arena/fallback统计应保留。
8. Worker command按compiled system key稳定merge，是可重复ApplyDeferred的必要基础。
9. Event active-channel worklist与bounded message retention已纠正旧的全通道扫描和永久message增长。
10. Observer copy-on-write bucket允许dispatch期间安全修改registry，适合继续扩展为generation-scoped observer graph。
11. Conflict graph已有component/resource/event访问维度和diagnostics，可升级为完整dependency/access DAG。
12. focused tests已覆盖columnar、bundle、query cache、change tick、command packing、event/message与schedule基本行为；应从source-shape升级，不应删除行为覆盖。

## 5. P0 阻断

### 5.1 Query共享接收器制造并存可变别名

`system/query.rs`中的`Query::iter(&self)`把`self.state: *mut QueryState`转成`&mut *self.state`并调用`iter_cached_with_ticks`；后者会`update_cache`并返回同时借用`cached_archetype_plans`切片和state指针的`QueryIter`。由于入口只要求`&self`，安全调用者可以写出“保留iterator A，再创建iterator B”。第二次调用会在A存活期间再次创建`&mut QueryState`，刷新所有plan；若出现新archetype还会`Vec::extend`并可能重分配，使A保存的slice悬空。`iter_combinations/count/is_empty`重复同一模式，注释声称run item唯一拥有state，但Rust类型没有把这个前提编码到方法receiver。

| ID | 阻断 | 唯一关闭条件 |
|---|---|---|
| RECS-P0-01 | safe API可通过两个共享`&Query`调用触发同一QueryState的并存可变访问，并可能让首个iterator持有被第二次cache update重分配的plan slice。 | 立即把会更新cache/返回借用cache的入口收紧为`&mut self`，或拆分不可变frozen plan与受控interior state；建立compile-fail双iterator、Miri、archetype growth/reallocation和drop diagnostics测试。未经资格不得让该query进入多线程executor。 |

### 5.2 RemovedComponentEvents永久累积

`removal.rs`只有`push/push_type_id/events`，每类组件持有`Vec<RemovedComponentEvent>`；reader的`read/clear`只把自身`cursor`推进到当前长度。World和UpdateEvents没有清理、双buffer、reader watermark、容量、byte budget、age、overflow receipt或slow-reader policy。因此每次组件移除和实体despawn都会永久增加World驻留内存，即使没有任何reader；长期编辑器会话、开放世界streaming或反复Play可稳定放大RSS。

| ID | 阻断 | 唯一关闭条件 |
|---|---|---|
| RECS-P0-02 | 移除事件按历史总量无界保留，正常production mutation即可造成不可恢复的World内存增长。 | 统一进入generation-scoped bounded event registry；按双buffer或reader watermark回收，显式定义capacity/bytes/age/overflow/slow-reader policy，World update与system removal必须推进生命周期；以百万次remove/despawn和100h soak证明驻留上界。 |

## 6. P1 工程化差距

### 6.1 Entity、component、storage与archetype（P1-01 至 P1-12）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RECS-P1-01 | public `EntityId=u64`不携World owner/epoch，跨World或replacement可静默命中新实体。 | 分离`LiveEntityKey(index,generation)`、`WorldIdentity(id,epoch)`与持久`SceneEntityGuid`，所有跨帧/ABI句柄校验owner。 |
| RECS-P1-02 | EntityRegistry用`slots.len() as u32`分配index，没有checked exhaustion。 | allocator显式保留invalid值，耗尽时fail-closed并返回typed capacity error。 |
| RECS-P1-03 | entity generation到`u32::MAX`后回到1，足够长寿命下允许ABA。 | generation checked exhaustion或扩大identity domain；耗尽slot退休，不静默复用。 |
| RECS-P1-04 | archetype membership generation用wrapping/saturating序列，没有epoch rollover协议。 | `ArchetypeGeneration`绑定World epoch并在耗尽时关闭结构mutation或重建带新epoch的World。 |
| RECS-P1-05 | ComponentDescriptor只有id/type name/storage/source，没有schema version、layout fingerprint和migration。 | `ComponentSchema`记录stable key、version、layout/drop/move ABI、provider generation、serialization和migration chain。 |
| RECS-P1-06 | dynamic component身份依赖字符串，typed身份依赖进程`TypeId`，无法跨DLL reload和archive稳定比较。 | provider-qualified stable component key映射到每代runtime ComponentId；TypeId只作当前进程加速。 |
| RECS-P1-07 | dynamic组件默认SparseSet，registration没有按profile/usage验证storage policy。 | registration提供table/sparse/external policy、size/alignment/expected cardinality与可观测迁移成本。 |
| RECS-P1-08 | `ResourceStore::clone()`返回default且`PartialEq`恒true，API形状伪装数据已复制/相等。 | 删除欺骗性trait实现，或通过schema clone/compare policy完整复制并报告不可克隆资源。 |
| RECS-P1-09 | `ComponentStorage::clone()`同样清空数据，`PartialEq`恒true。 | snapshot/clone走显式component capture transaction；普通Clone不存在或保证完整语义。 |
| RECS-P1-10 | `ArchetypeIndex::PartialEq`恒true，World比较无法发现storage topology差异。 | 定义结构identity/equality receipt，测试只比较明确snapshot，不以恒真绕过derive。 |
| RECS-P1-11 | sparse locator扩到最高entity index后不收缩，稀疏高水位可永久占用大数组。 | chunked sparse pages、retirement/compaction policy和locator bytes diagnostics。 |
| RECS-P1-12 | `World::spawn_node`等stable id序列使用unchecked `+= 1`。 | 所有public/stable identity统一checked allocator、reserved invalid、owner epoch和exhaustion terminal。 |

### 6.2 Query、access、borrowing与change detection（P1-13 至 P1-28）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RECS-P1-13 | QueryState只缓存component IDs与archetype generation，不保存创建它的WorldId。 | state保存WorldIdentity并在每次使用验证；跨World返回`QueryStateWorldMismatch`。 |
| RECS-P1-14 | SystemState也没有World identity，可把旧component/resource/event IDs用于另一个World。 | SystemState绑定World/schema generation；显式rebind重新初始化并注销旧reader/observer。 |
| RECS-P1-15 | `QueryData for &mut T`返回只读`&T`，但access登记write，类型语义和调度权限不一致。 | read path只接受`&T/Ref<T>`；mutable query只通过独占receiver产生`&mut T/Mut<T>`。 |
| RECS-P1-16 | QueryData tuple支持到8，但QueryMutData没有tuple实现，不能一次安全修改多个组件。 | GAT/derive生成可组合mutable fetch，编译期拒绝重复mutable component并支持常用宽度。 |
| RECS-P1-17 | QueryData、QueryFilter、SystemParam和ParamSet均手写固定到arity 8。 | 统一tuple generation/variadic abstraction，至少覆盖产品系统宽度并有compile-time budget。 |
| RECS-P1-18 | UniqueEntityArray重复验证是O(N²)。 | 小N内联、较大N排序或bounded hash验证，并输出duplicate index而非只返回失败。 |
| RECS-P1-19 | mutable many/query combination先快照candidate和location Vec，热路径按调用分配。 | iterator/chunk直接遍历compiled storage range；temporary来自bounded scratch arena。 |
| RECS-P1-20 | get/contains等路径反复创建component location Vec，宽query分配与组件数线性。 | plan内固定slot map和small-vector/scratch复用，统计fallback allocation。 |
| RECS-P1-21 | combination query没有K、candidate、输出数量或时间预算，组合爆炸只能由caller自律。 | admission估算`n choose k`，设item/byte/deadline/cancel上限并返回truncation receipt。 |
| RECS-P1-22 | standalone QueryState read-only API仍有whole-world uncached路径，system path与公开路径性能语义分裂。 | 明确`PreparedQuery`为默认；一次性scan必须命名并要求budget，diagnostics区分两者。 |
| RECS-P1-23 | cache hit仍逐个刷新所有matched plan membership，稳定帧成本与archetype数线性。 | membership delta/generation log只触达changed archetypes，稳定generation执行零metadata写。 |
| RECS-P1-24 | cache失效只看archetype generation，不看component schema/provider generation。 | plan key包含World、schema、provider与archetype generations；任一stale都拒绝fetch。 |
| RECS-P1-25 | incremental append后依赖plan按archetype id保持排序，binary search不在类型/API中编码该不变量。 | 使用sorted insertion或indexed plan map；debug/release都验证唯一和排序。 |
| RECS-P1-26 | raw World pointer、NonNull<QueryState>和storage pointer的Safety前提散落在多个iterator。 | 建立唯一`WorldCell/UnsafeWorldCell`和proof-carrying access token，unsafe边界记录alias/lifetime/thread条件。 |
| RECS-P1-27 | 没有按archetype/table chunk的parallel query iterator和partition token。 | compiled plan生成不重叠chunk lease，changed ticks与deferred lane按chunk归并。 |
| RECS-P1-28 | 没有compile-fail/Miri测试证明重复mutable query、ParamSet交错和iterator lifetime被拒绝。 | 建立trybuild UI suite、Miri alias suite和archetype reallocation regression，纳入受管验证。 |

### 6.3 SystemParam、schedule与parallel executor（P1-29 至 P1-44）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RECS-P1-29 | SystemParam tuple固定到8，复杂系统被迫嵌套tuple或拆出无owner facade。 | derive/tuple generator支持产品宽度，编译diagnostic精确指出冲突param。 |
| RECS-P1-30 | ParamSet仅在运行时允许冲突param顺序访问，缺少set级别访问/条件元数据。 | ParamSet state产生分支访问token并限制同时存活item；schedule知道其最坏访问集。 |
| RECS-P1-31 | `SystemState::run`在`catch_unwind`前构造param；param构造panic会让World active change tick未恢复。 | RAII run guard负责active tick、command lane和reader lease，无论任何阶段panic都恢复。 |
| RECS-P1-32 | callback后diagnostics或command merge panic没有统一system terminal/rollback record。 | system execution stage化为prepare/run/diagnostics/merge/apply，每阶段有failure class和cleanup guard。 |
| RECS-P1-33 | EventReaderState注册reader后没有Drop/disconnect，删除system仍永久增加reader_count。 | reader registration返回generation lease，SystemState Drop/rebind/unregister等待in-flight并回收。 |
| RECS-P1-34 | SystemState没有通用param cleanup hook，resource/event/message/observer state无法参与retirement。 | `SystemParamState::retire(world)`或owner registry统一执行cleanup并返回quiescence receipt。 |
| RECS-P1-35 | 真正访问Query/Res/Event/Message的native system不实现WorldlessSystemParam，生产仍串行。 | executor使用validated WorldCell access token并行非冲突World systems；exclusive/foreign形成barrier。 |
| RECS-P1-36 | ScheduleParallelExecutor是通用闭包批测试器，不是SceneScheduleRunner真实World executor。 | 合并为一个compiled EcsExecutor，测试与产品走同一DAG、worker、panic和ApplyDeferred路径。 |
| RECS-P1-37 | 显式`before/after`只用于step拓扑排序，worker batch仅按data conflict判断，可违反声明顺序。 | ScheduleGraph把dependency和access edge共同编译；dependency未完成的system永不进入ready set。 |
| RECS-P1-38 | stable command merge只确定副作用提交顺序，不能修复并发callback自身的外部副作用顺序。 | worker callback禁止未声明外部副作用；thread-bound/external effect通过exclusive capability或deferred operation。 |
| RECS-P1-39 | conflict graph pairwise O(S²)，访问集是sorted Vec，系统规模增长时compile成本无边界。 | component/resource/event bitset、incremental graph cache和compile time/memory diagnostics。 |
| RECS-P1-40 | 没有run condition、set condition、ambiguity policy和condition access冲突。 | SystemMeta包含condition/set/ambiguity，executor像普通system一样调度condition依赖。 |
| RECS-P1-41 | send/non-Send、exclusive、main-thread、foreign callback和world partition只以少量bool表达。 | typed executor class与affinity capability，编译期产生合法lane和barrier。 |
| RECS-P1-42 | virtual pause时所有InternalSceneSystem无条件跳过，包括UpdateEvents；real-clock系统仍可运行。 | 每个internal system声明clock/lifecycle domain；event/message maintenance按明确frame policy推进。 |
| RECS-P1-43 | schedule/registry clone和serde丢弃native/runtime callback，克隆World可静默改变行为。 | schedule是owner-qualified runtime artifact；snapshot只保存stable descriptors，restore重新解析并验证缺失provider。 |
| RECS-P1-44 | 每个worker flush分配system id、system box、timing和command-buffer引用Vec。 | compiled batch持久保存indices/scratch容量，稳定帧零控制面heap allocation并记录fallback。 |

### 6.4 Deferred commands与事务（P1-45 至 P1-54）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RECS-P1-45 | generic Command/FnCommand可执行任意`FnOnce(&mut World)`，无法preflight、序列化或审计。 | public deferred API由typed operation enum/trait描述read/write/schema/cost/rollback；opaque closure仅限内部exclusive lane。 |
| RECS-P1-46 | apply中某命令panic只丢弃剩余queue，之前命令副作用保留，调用者拿不到partial receipt。 | transaction window先prepare全体，再commit；失败返回applied range和compensation/poison状态。 |
| RECS-P1-47 | generic closure会切断连续structural batch，bundle级原子性不能跨opaque command。 | command compiler按access和transaction boundary分段，调用者显式声明barrier。 |
| RECS-P1-48 | CommandQueue clone返回default且equality恒true，快照/比较可静默丢pending work。 | 删除Clone/Eq，或导出只读descriptor snapshot；pending executable payload不可伪比较。 |
| RECS-P1-49 | deferred token/ordinal/generation耗尽使用`expect`或无统一checked policy。 | owner-qualified DeferredToken allocator在admission阶段返回Exhausted并关闭lane。 |
| RECS-P1-50 | inline/fallback有局部上限和metrics，但无per-system/plugin/world quota与rejection strategy。 | hierarchical command entry/byte/alignment budget，overflow可defer/reject/fail system并有receipt。 |
| RECS-P1-51 | worker command merge失败会丢payload，但callback已发生的外部副作用不能撤销。 | worker-safe contract只允许local state与DeferredWorld；外部effect在成功commit后发布。 |
| RECS-P1-52 | command payload没有stable schema/operation ID，不能用于replay、network prediction或crash journal。 | typed DeferredOperation携schema、owner、deterministic key和optional journal codec。 |
| RECS-P1-53 | structural preflight覆盖已有命令族，不保证任意组件provider卸载、schema迁移和observer副作用安全。 | preflight验证provider generation、observer plan、capacity和all affected archetypes，再取得commit lease。 |
| RECS-P1-54 | command metrics未形成first-party frame/profile consumer和top offender。 | EcsDiagnostics按world/system/plugin报告entry/bytes/fallback/apply wall/panic/rollback与budget outcome。 |

### 6.5 Events、messages与observers（P1-55 至 P1-66）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RECS-P1-55 | event generation使用saturating add，到上限后cursor无法区分后续更新。 | checked generation exhaustion触发channel retirement/new epoch，cursor收到StaleGeneration。 |
| RECS-P1-56 | EventTypeId来自`channels.len() as u32`，没有容量和exhaustion检查。 | registry使用checked slot+generation并支持provider卸载后的stale拒绝。 |
| RECS-P1-57 | event reader只有register，没有disconnect/slow-reader ownership和reader census回收。 | RAII EventSubscription记录system owner、cursor、lag、Drop和retirement barrier。 |
| RECS-P1-58 | active-channel worklist减少空扫描，但单channel单帧event数量/bytes仍可无限增长。 | per-channel/per-owner/global entry+byte budget和overflow策略，send返回typed outcome。 |
| RECS-P1-59 | send-boundary observer先同步运行并只聚合bool，observer拒绝也不阻止事件入队，语义不清。 | 区分validation、tap和delivery observer；是否可拒绝、是否入队、失败传播在类型中明确。 |
| RECS-P1-60 | MessageId exhaustion直接panic，长期server/editor会话没有terminal policy。 | checked identity allocator和channel epoch，耗尽关闭admission并输出owner report。 |
| RECS-P1-61 | message generation饱和后冻结，cursor freshness失真。 | generation rollover protocol与stale cursor error，不用saturation隐藏overflow。 |
| RECS-P1-62 | MessageCursor在创建read iterator时前移到窗口末尾，部分消费后Drop会确认未读消息。 | cursor逐item commit，或显式`read_batch/ack`事务；Drop语义有测试。 |
| RECS-P1-63 | observer callback同步获得`&mut World`，可递归触发observer且无depth/work budget。 | ObserverGraph维护dispatch queue、depth、event/byte budget和cycle/recursion diagnostic。 |
| RECS-P1-64 | observer panic没有按subscription隔离、禁用或terminal receipt。 | catch并关联ObserverId/owner/trigger/entity；policy决定disable/retry/fail transaction。 |
| RECS-P1-65 | ObserverId用unchecked sequence，注册/注销无provider generation和in-flight callback lease。 | slot+generation identity、callback lease、unregister quiescence与plugin unload barrier。 |
| RECS-P1-66 | EventStore、MessageStore和ObserverStore clone清空且equality恒真，World clone行为不可见。 | 删除欺骗性trait；snapshot明确选择保留队列/cursor/registration或返回unsupported participant。 |

### 6.6 Product integration、diagnostics与资格（P1-67 至 P1-72）

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| RECS-P1-67 | first-party hierarchy/transform/active/event maintenance仍是直接World函数，不使用typed query/system access。 | 逐域迁移到同一SystemMeta/QueryPlan路径，保留必要exclusive system并说明原因。 |
| RECS-P1-68 | typed Query/Res/Event系统的production caller薄，API测试不能证明gameplay/render workload adoption。 | 建立first-party adoption matrix，至少transform、animation、physics sync、visibility/extract使用真实ECS executor。 |
| RECS-P1-69 | plugin可声明access，但registration metadata不是不可伪造的实际fetch capability。 | 系统只能从注册access token构造WorldCell view；未声明fetch在运行前拒绝。 |
| RECS-P1-70 | worldless external system拿不到World，dynamic/native plugin又常退化为conservative World writer。 | plugin SDK暴露versioned typed query/resource/event descriptors和host-validated chunk callback。 |
| RECS-P1-71 | 23个focused test文件含source-shape断言，且没有trybuild/Miri/fuzz覆盖unsafe query/storage。 | behavior、compile-fail、model、Miri、sanitizer和fault tests分层；source scan只守owner边界。 |
| RECS-P1-72 | 没有真实产品scene的schedule speedup、query tail、allocation、RSS和UE/Bevy同负载基线。 | 发布固定机器/build/workload/profile artifact，记录p50/p95/p99、alloc、cache、worker利用率和结果一致性。 |

## 7. P2 一致性、可维护性与资格差距

| ID | 当前差距 | 建议 |
|---|---|---|
| RECS-P2-01 | 多种ID仍是可默认构造的整数newtype，invalid/owner语义不一致。 | 统一NonZero或显式Invalid，Debug输出owner/generation。 |
| RECS-P2-02 | ComponentDescriptor的type name同时承担诊断和dynamic identity。 | 分离stable key、display name、Rust type name和localized presentation。 |
| RECS-P2-03 | tuple实现散布在bundle/query/filter/system/ParamSet多个macro，最大arity易漂移。 | 单一generation source和compile matrix锁定支持宽度。 |
| RECS-P2-04 | Query cache counter和多种generation饱和后没有overflow bit。 | 窗口化metric携overflow/stale/reset generation。 |
| RECS-P2-05 | registered component/removal type name查询每次分配Vec并排序。 | registry维护immutable sorted snapshot generation。 |
| RECS-P2-06 | RemovedComponents read每次复制EntityId到新Vec。 | 修复retention后提供借用page/iterator和明确ack。 |
| RECS-P2-07 | EventCursor与MessageCursor消费语义相似但Drop/ack行为不同。 | 命名和文档明确peek/read/ack/clear，建立一致contract suite。 |
| RECS-P2-08 | QueryState Debug不展示World/schema identity、plan generation和access摘要。 | bounded Debug/diagnostic snapshot包含稳定身份和cache currentness。 |
| RECS-P2-09 | observer注册允许目标entity当前不存在，错误只在未来dispatch表现。 | registration policy显式选择RequireLive或DeferredPersistentTarget。 |
| RECS-P2-10 | 多个public unsafe/helper附近缺少完整Safety章节。 | 每个unsafe说明validity、aliasing、lifetime、unwind、thread和postcondition。 |
| RECS-P2-11 | source-shape test以字符串锁定方法名和目录，不验证间接调用或语义。 | compiler lint/visibility contract加行为测试，字符串只作补充。 |
| RECS-P2-12 | ECS test按历史feature增长，结构、性能、行为和owner guard混杂。 | 按identity/storage/query/schedule/events/unsafe/qualification拆folder-backed suite。 |
| RECS-P2-13 | 没有property-based随机spawn/insert/remove/despawn/query序列与model oracle。 | 与简单reference model差分并保留seed/minimized artifact。 |
| RECS-P2-14 | 没有command/observer/event panic点的systematic fault injection。 | 每个prepare/commit/publish/cleanup边界可注入失败并验证conservation。 |
| RECS-P2-15 | schedule diagnostics以全局字符串counter为主，缺world/system generation。 | 使用低基数stable ID registry并保留top-N offender。 |
| RECS-P2-16 | performance acceptance主要断言内部计数，没有机器、噪声和回归阈值artifact。 | benchmark manifest记录硬件、OS、编译、scene、warmup、样本和置信区间。 |
| RECS-P2-17 | 没有Windows/Linux/macOS、1/2/8/32/64核的同一ECS workload矩阵。 | 平台矩阵验证结果确定性、tail latency、NUMA/oversubscription和shutdown。 |
| RECS-P2-18 | 没有100h mutation/streaming/plugin reload soak和RSS斜率证据。 | 长测输出entity/component/event/reader/observer/command conservation与RSS slope。 |

## 8. 参考引擎对照与适用边界

| 参考 | 已核对机制 | Zircon应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal MassEntity | archetype/chunk storage、requirements与cached archetype version、`ForEachEntityChunk/ParallelForEachEntityChunk`、typed batched command、composition add/remove observer pipeline、processing context/phase | chunk作为并行和统计边界；query requirement与archetype version；typed deferred mutation；composition observer plan | UObject反射、宏和UE全局subsystem lifetime不能替代Rust借用、DLL generation与World owner |
| Bevy ECS | QueryState绑定WorldId，保存matched table/archetype bitset与storage list；UnsafeWorldCell集中借用证明；multi-threaded executor同时维护conflicts、dependents、remaining dependency、ready/running/completed/unapplied sets；change detection与parallel query | World/schema identity、proof-carrying access、dependency+conflict统一DAG、send/exclusive/condition、chunk parallel、compile-fail安全面 | 不照抄全部API或默认global App lifetime；Zircon还需动态plugin卸载、stable ABI和产品archive合同 |
| Fyrox | generational Handle、pool free stack与borrow diagnostics、multiborrow、Graph owner | generation/borrow error必须是句柄合同；scene graph和pool identity分层 | Fyrox graph不是archetype ECS，也不能作为多核schedule性能证明 |
| Godot | ObjectID/registry、thread-local MessageQueue及flush reentrancy检查、WorkerThreadPool task/group identity和join、Node/SceneTree lifecycle | deferred queue线程owner与flush guard、task/group census、对象生命周期与树通知边界 | singleton、Callable/Variant和Node主导模型不是Zircon ECS数据面目标 |
| Unity Graphics package | NativeArray/NativeList、ReadOnly/WriteOnly/NoAlias、IJobParallelFor/Batch与JobHandle依赖组织GPU-driven实例数据 | render extract/instance update使用SoA、显式读写、批任务与依赖句柄，避免每entity对象调用 | 本地仓库不是Unity Entities或核心Job System完整源码，不能据此推断其全部安全/lifecycle实现 |

## 9. 目标架构

```text
WorldIdentity(id, epoch)
  -> EntityAllocator(slot, generation, persistent remap)
  -> ComponentSchemaRegistry(stable key, runtime id, version, provider generation)
  -> ArchetypeStore(table pages, sparse pages, structural generation log)
  -> QueryPlan(world/schema generation, matched bitsets, compiled fetch)
  -> WorldCell(access token, partition/chunk lease)
  -> SystemMeta(owner, access, dependencies, conditions, affinity)
  -> ScheduleGraph(dependency DAG + access conflicts + apply barriers)
  -> EcsExecutor(ready/running/completed, chunk tasks, panic terminal)
  -> DeferredWorld(typed prepare/commit/publish, stable merge key)
  -> EventRegistry / ObserverGraph(bounded lifecycle, subscription lease)
  -> EcsDiagnostics(generation snapshot, budgets, conservation, trace)
```

### 9.1 Identity与schema

所有live handle先校验WorldIdentity，再校验slot generation；persistent SceneEntityGuid不直接寻址storage。ComponentSchemaRegistry把stable component key映射到当前provider generation的ComponentId，query、snapshot和command都冻结schema generation。provider retirement先关闭registration/query/command admission，再等待WorldCell、system和observer lease归零。

### 9.2 Query与WorldCell

QueryPlan是不可变、generation-qualified artifact；运行时只刷新独立membership snapshot，不允许shared receiver制造`&mut QueryState`。WorldCell由ScheduleGraph依据access产生，read、write、non-Send、exclusive和chunk partition是不可伪造token。mutable iterator独占相应column/chunk，compile-fail与Miri证明重复借用不可构造。

### 9.3 Schedule与DeferredWorld

ScheduleGraph同时编译显式dependency、run condition和access conflict。executor只从dependency归零且访问兼容的ready set启动system；实际World query按chunk进一步并行。所有worker副作用先进入typed DeferredWorld，成功后按stage/system/chunk/ordinal稳定commit；foreign/main-thread/exclusive operation成为显式barrier。

### 9.4 Events与observers

Event、Message和RemovedComponent统一使用generation channel、entry/byte/age budget、cursor ack与overflow receipt。subscription是owner-qualified RAII lease。ObserverGraph将validation、tap和delivery分开，dispatch有queue/depth/budget、panic policy和plugin retirement barrier。

### 9.5 Diagnostics与产品事实

每帧snapshot至少包含world/schema/schedule generation、entity/archetype/table/sparse bytes、query cache/currentness、system ready/run/wait、parallelism、command bytes/fallback、event/message/removal backlog、reader lag、observer depth/panic和budget outcomes。Editor、headless和profile capture消费同一snapshot。

## 10. 分阶段重构计划

### M0：止血与current-source资格

- 修复RECS-P0-01：共享Query入口不得更新可变cache；补compile-fail/Miri/reallocation测试。
- 修复RECS-P0-02：RemovedComponentEvents进入有界生命周期；补百万mutation和RSS slope测试。
- 冻结92项finding的source owner、现有failure路由和first-party caller matrix。

### M1：WorldIdentity、allocator与ComponentSchemaRegistry

- 引入WorldId/epoch、checked entity/component/event/observer allocator和stale errors。
- schema记录stable key/version/layout/provider generation；迁移dynamic/typed registration。
- 删除恒真PartialEq和清空Clone，snapshot改走显式participant。

### M2：ArchetypeStore与结构delta

- 保留columnar table，补page allocator、sparse high-water/compaction和structural generation log。
- bundle/insert/remove/despawn统一prepare/commit/publish，observer plan在commit前冻结。
- 建随机model differential、Miri和sanitizer资格。

### M3：QueryPlan、WorldCell与SystemState

- QueryPlan绑定World/schema/provider generation，mutable tuple与wide param支持收敛。
- 建立唯一WorldCell/UnsafeWorldCell safety model、chunk lease和parallel iterator。
- SystemState RAII run/retire guard恢复tick、reader、commands与plugin lease。

### M4：ScheduleGraph与真实EcsExecutor

- dependency、condition与access冲突编译成同一DAG；修复worker batch顺序漏洞。
- 生产Query/Res/Event system进入WorldCell executor，non-Send/exclusive/foreign显式barrier。
- compiled batch复用控制面scratch，稳定帧零heap allocation。

### M5：DeferredWorld与事件观察者收敛

- generic closure收窄，typed operation支持preflight、stable merge、partial failure receipt与optional journal。
- Event/Message/Removed统一channel/cursor/budget；EventReader/Observer registration改RAII generation lease。
- observer dispatch queue加入depth/work/panic和retirement policy。

### M6：迁移first-party产品链

- 先迁移transform/hierarchy/active和event maintenance，再迁animation/physics sync/visibility/extract。
- plugin SDK只暴露host验证的typed access/query/chunk capability，移除conservative World writer常态化路径。
- World snapshot、dynamic scene和render extract通过schema/delta artifact接线，Runtime05/08继续拥有上层格式和性能计划。

### M7：性能、故障与跨平台资格

- 固定scene在1/2/8/32/64核测schedule speedup、query p95/p99、allocation、cache miss和RSS。
- 执行panic、OOM/admission、provider unload、generation exhaustion、slow reader、observer recursion和shutdown fault injection。
- Windows/Linux/macOS、100h soak、Miri/sanitizer/fuzz/profile artifact齐全后，才允许讨论达到或超过Unreal。

## 11. 验收门禁

### 11.1 Identity、schema与storage（G01-G08）

| Gate | 验收条件 |
|---|---|
| G01 | 任一Entity/Component/Query/System/Event/Observer handle在错误World/provider generation上返回typed stale error。 |
| G02 | 所有ID exhaustion fail-closed，无wrap、saturate后复用或panic。 |
| G03 | ComponentSchema包含stable key/version/layout/provider/migration，archive和plugin reload可验证。 |
| G04 | World clone/snapshot对每个participant明确Copied/Skipped/Unsupported，无清空Clone和恒真Eq。 |
| G05 | columnar move/drop/alignment/ZST在Miri和sanitizer矩阵通过。 |
| G06 | sparse high-water有预算和compaction，恶意高index不会永久线性占用。 |
| G07 | 100万随机结构mutation与reference model实体/组件结果一致。 |
| G08 | structural generation delta只包含changed archetype，稳定帧metadata write为0。 |

### 11.2 Query、borrow与change detection（G09-G16）

| Gate | 验收条件 |
|---|---|
| G09 | safe API无法在iterator存活时再次可变刷新同一QueryState；RECS-P0-01回归在Miri通过。 |
| G10 | QueryPlan跨World/schema/provider generation必定拒绝，不能偶然命中同ID。 |
| G11 | 重复`&mut T`、mutable tuple重复类型和非法ParamSet同时借用有compile-fail证据。 |
| G12 | read query不登记write，mutable query不通过共享World产生可变引用。 |
| G13 | cached query稳定generation零heap allocation，archetype delta只增量编译。 |
| G14 | combination/many query遵守item/byte/deadline/cancel预算并返回receipt。 |
| G15 | parallel query chunk地址范围不重叠，change tick和command key可重放一致。 |
| G16 | wrap附近added/changed过滤、未实际mutation和长间隔system均符合定义。 |

### 11.3 Schedule、commands与parallel（G17-G24）

| Gate | 验收条件 |
|---|---|
| G17 | dependency未完成的system绝不启动，即使访问集不冲突。 |
| G18 | run/set condition访问进入冲突计算；non-Send/exclusive/foreign只在合法lane运行。 |
| G19 | first-party Query/Res/Event system实际在worker并行，非worldless测试替身。 |
| G20 | 1线程与多线程最终World、events和command order一致，声明为非确定的输出单独记录。 |
| G21 | 任意param/callback/diagnostics/merge/apply panic后active tick、lane和system registry守恒。 |
| G22 | DeferredWorld全体preflight后commit；partial failure返回applied range、poison和compensation状态。 |
| G23 | 稳定schedule帧控制面零heap allocation，fallback按system/world可观测。 |
| G24 | pause/real/virtual/fixed组合中event/message/removal maintenance按声明domain推进。 |

### 11.4 Events、observers与lifecycle（G25-G32）

| Gate | 验收条件 |
|---|---|
| G25 | RemovedComponent backlog在无reader、慢reader和多reader下均有严格entry/byte上界。 |
| G26 | 百万次remove/despawn后回收完成，RSS不随历史总量线性增长。 |
| G27 | Event/Message/Removed cursor逐item或显式batch ack，部分迭代Drop不吞未读数据。 |
| G28 | reader/system/plugin Drop注销subscription并等待in-flight lease，reader census归零。 |
| G29 | event/message generation与type/message ID耗尽返回terminal，不panic或冻结。 |
| G30 | observer递归、slow callback和panic受depth/work/deadline policy限制并可定位owner。 |
| G31 | entity-target observer对despawn、ID reuse和World replacement无stale delivery。 |
| G32 | event validation/tap/delivery语义、拒绝与入队结果在API和测试中一致。 |

### 11.5 Product、fault与性能资格（G33-G40）

| Gate | 验收条件 |
|---|---|
| G33 | transform/hierarchy/animation/physics/visibility/extract adoption matrix无未声明World旁路。 |
| G34 | plugin系统只能取得registration授权的access token，provider unload前lease归零。 |
| G35 | Editor、runtime、headless和dynamic session消费同一EcsDiagnostics generation snapshot。 |
| G36 | trybuild、Miri、sanitizer、fuzz、fault和model tests进入受管验证并保存artifact。 |
| G37 | Windows/Linux/macOS及1/2/8/32/64核结果一致，无deadlock、oversubscription和shutdown leak。 |
| G38 | 100h soak中entity/component/event/reader/observer/command conservation成立，RSS slope在预算内。 |
| G39 | 固定产品scene发布p50/p95/p99、alloc、cache、worker利用率和speedup，不用内部counter替代。 |
| G40 | 只有同负载、同画质/结果、同硬件和可复验artifact优于Unreal后，才允许宣称性能领先。 |

## 12. Owner路由与实施约束

| 内容 | 主owner | 协同owner |
|---|---|---|
| query alias止血、WorldCell和SystemState safety | Runtime60 | Runtime03、Runtime08、Tooling21 |
| removed/event/message/observer lifecycle | Runtime60 | Runtime02、Runtime05、Runtime08 |
| Entity/Component/World/schema identity | Runtime24 | Runtime60、Runtime04、Plugins01、Interface01 |
| ScheduleGraph与生产worker执行 | Runtime03 | Runtime60、Runtime22、Runtime59 |
| World snapshot、dynamic scene、derived state、extract | Runtime05/07/08 | Runtime60、Runtime39、Runtime52/53 |
| task pool、worker budget、shutdown | Runtime59 | Runtime03、Runtime60、App01 |
| unsafe CI与证据治理 | Tooling21/10 | Runtime60；用户暂停tooling优化期间只登记依赖 |

实施时必须遵守：

- 先修两项P0并建立资格门，再扩大World并行；不能让潜在悬空query iterator进入worker。
- 不保留清空Clone、恒真Eq、旧raw EntityId或旧Query shared-cache入口作为兼容shim。
- 不以全局`RwLock<World>`假装并行；并行资格来自access token和不重叠chunk。
- 不以source-string test、内部counter或微型synthetic closure证明产品性能。
- Runtime08已完成的columnar、lazy tick、bounded message、command arena和stable merge不得回退。
- tooling迁移到Rust前不扩写tooling实现；Miri/sanitizer等仍是Runtime60关闭P0所需外部资格依赖。

## 13. 复核与未执行项

本轮逐文件读取`scene/ecs`143个Rust文件，并沿World/module/LevelSystem/event mirror和44个focused test文件核对调用链；对2项P0又回读了query receiver、cache update、iterator borrowed plan、removed event push/reader和schedule runner具体实现。参考对照定点读取Unreal MassEntity query/command/observer/phase、Bevy query state/access/multi-threaded executor/change detection、Fyrox pool/handle/multiborrow、Godot object/message queue/worker pool及Unity Graphics GPU-driven jobs。

未执行Cargo是有意的：MVP gate仍阻止以普通Cargo成功替代当前review milestone，而且本轮没有production改动。也未执行Miri、sanitizer、fuzz、stress、soak、动态plugin unload、真实产品scene profile或跨引擎benchmark；它们已被写成G09、G25-G26和G36-G40硬门禁。静态review能证明当前源码合同和缺口，不能证明运行时未触发UB、长期RSS上界或性能领先。

## 14. 当前状态

- Runtime60 review：`review_complete`。
- production/tests/Cargo/ABI实现：`pending`，本轮零改动。
- 新增唯一P0：2；P1：72；P2：18；验收门禁：40。
- source recheck：required；进入实现前必须重算四组fingerprint、open failure和first-party caller。
- 全引擎review goal仍为`in_progress`；Runtime60只是ECS kernel纵切面，不表示zircon_runtime或整个引擎审查完成。
