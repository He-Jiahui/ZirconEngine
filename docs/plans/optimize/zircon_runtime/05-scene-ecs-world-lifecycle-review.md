---
related_code:
  - zircon_runtime/src/scene
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime_interface/src/world_sync
  - zircon_editor/src/core/gateway
  - zircon_editor/src/core/play/snapshot
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-world-derived-state-full-rebuild.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-archetype-columnar-storage.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-event-message-bounded-lifecycle.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-lazy-change-detection.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
reference_engines:
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world_builder.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/Fyrox/fyrox-graph/src
  - dev/Fyrox/fyrox-impl/src/scene
  - dev/godot/scene/main/node.cpp
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassEntityManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassEntityQuery.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Internal/MassArchetypeData.h
---

# 05 · Scene、ECS 与 World Lifecycle 工程化差距

## 1. 结论

Zircon 的 Scene/ECS 已经越过“用若干 `HashMap<EntityId, Component>` 拼出场景”的早期阶段。当前源码已经把 dense component 真值下沉到 archetype-owned table，sparse component 有 generation-aware sparse locator；`EntityLocation`、stable query order、compiled archetype query plan、lazy change tick、bundle/deferred structural preflight、schedule plan cache、bounded dynamic-scene reload、增量 inspection artifact 和有界 message retention 都是应保留的正确方向。旧 failure 中关于 table 仍为 per-component `HashMap`、所有 event channel 每帧全扫、message 永久无界、层级传播递归栈溢出的描述，已不再完整反映 current source，本文不重复登记。

但这些局部收敛还没有形成可承载大型开放世界、插件组件、编辑器 Play、远程检查和多线程帧执行的统一 World 契约。公开实体身份仍是裸 `u64`，没有 World epoch 或 entity generation；`World::clone` 与 serde 只复制硬编码内建组件组，任意 typed component 可静默丢失；真正读取 `Query/Res/Events` 的系统无法进入 worker 并行；层级、active、transform、node cache 与 render extract 仍以整个 domain 的 bool dirty 驱动全量工作；每个视图继续重新收集、排序和 clone 场景 DTO；Play 快照同步执行 `entities × registered types` 反射扫描并生成 pretty JSON；component schema 没有逐类型版本、迁移链或 missing-type policy；动态 world query 没有计划、分页、预算或响应上限；event 双缓冲只限制存活帧数，单帧 producer 仍可无限增长；scene partition/streaming 在 runtime 中没有权威数据模型，编辑器当前只有导航/反馈字符串；565 个 dynamic-scene session 文件形成巨大组合式 facade，却没有产品主链消费。

本轮登记 12 项 P1 和 2 项 P2，没有新增 P0。`World::clone`/serde 的数据丢失目前没有证据表明已导致已发布项目损坏，因此按 P1；一旦发现产品保存、checkpoint 或 rollback 已通过该路径持久化任意插件 typed component，应立即上调为 P0 数据完整性事故并暂停继续扩展场景格式。

## 2. 审查边界与物理覆盖

### 2.1 已读范围

- 对 `zircon_runtime/src/scene` 全树建立物理清单：1,075 个 Rust 文件、约 100,891 行、1,001 个 `#[test]`。其中 `tests` 154 文件/约 33,923 行，`world` 76/约 19,944，`ecs` 143/约 19,475，`dynamic_scene` 615/约 15,988，`inspection` 14/约 3,431，`reflect` 26/约 3,005。
- 深读 World identity/registry/dense storage/archetype/query/change detection/bundle/commands、schedule runner/system params、events/messages/observers、hierarchy/derived state/node cache、render extract、reflection、DynamicScene capture/spawn/reload/session、LevelSystem/frame publication 与 inspection artifact/query。
- 沿生产调用点核对 editor in-process gateway、dynamic runtime ABI `query_world`、编辑器 hierarchy/field publication、Play snapshot、LevelSystem project I/O 和 renderer extract；没有只根据 trait/API 名称判断完成度。
- 交叉核对 Runtime05、Runtime07、Runtime08 及其 open failure。对 current source 已完成的 archetype table、query plan、lazy tick、bounded messages、active event worklist、iterative hierarchy 和 inspection artifact 明确做状态纠正。
- 读取 Bevy generational entity、access-compatible multi-threaded executor、reflection-driven dynamic world；Fyrox generational `Handle<Node>` 与 graph；Godot Node owner/process group/PackedScene pack/instantiate；Unreal UWorld level/streaming/partition/tick group 与 Mass entity/archetype/chunk/batch query 的对应源码。

### 2.2 明确未覆盖

- 本篇不评价 physics、animation、navigation、script 或 network 系统自身算法质量；只评价它们依赖的 World identity、system scheduling、frame publication 和 mutation 边界。
- renderer/RHI 中的 GPU buffer、barrier、device loss、material/shader 和 GPU-driven culling 归 `09/10`。本文只拥有 Scene 到 RenderExtract 的上游 generation/delta 契约。
- editor undo/redo、prefab UI、multi-user authoring 和 viewport interaction 的最终产品闭环归 `zircon_editor` 专篇；本文拥有可序列化 scene document、live entity remap 和 runtime/editor World 边界。
- 没有运行 Cargo、100k entity/hierarchy、multi-camera 或 Play latency benchmark。当前 scene/ECS 源码与既有计划存在大规模活跃修改，因此所有结论标记 `recheck_required`；本文不把静态审查写成性能胜负结论。

## 3. 当前实现闭环与应保留能力

### 3.1 ECS storage、query 与 mutation

`ArchetypeRecord -> ArchetypeTable` 当前拥有同 row 的 entity 和 erased contiguous columns；table add/remove/bundle 会准备目标 signature、验证 value/schema 后移动 row，并修复 swap-remove entity location。`ComponentStorage` 已收敛为 sparse-only；`SparseComponentStorage` 用 dense values/entities 与 generation-aware locator，不再以 `HashMap<InternalEntity, usize>` 为唯一定位。`QueryState` 保留 per-archetype compiled plan/column slots 和局部 membership generation，不再为每个 query 复制整个 entity/location projection。`StableQueryOrderIndex` 与 dense physical row 分离，允许移动后维持确定性公共枚举。

结构命令也不只是任意 closure 队列：bundle 和 worker command buffer 有 compiled key、preflight 与统一 ApplyDeferred barrier；DynamicScene spawn 会先编译 remap/write plan，在隔离且有预算的 staging world 中验证 target world/schema/component generation，再提交目标变更。这些是后续重构的基础，不应回退成 clone World 事务或 per-component 双写。

### 3.2 Change detection、events/messages 与 inspection

table/sparse/resource 的 `Mut/ResMut` 现在持有真实 changed-tick authority，只在 `DerefMut/as_mut/into_inner/set_changed` 时标记；raw `&mut` API 明确 eager mark。EventStore 维护 active channel worklist，稳定 idle 不扫描全部注册类型。Messages 使用 sequence/cursor、entry/declared-byte/age budget、drop/lag 指标与 active retention worklist。事件 ABI mirror 也有有界队列。

编辑器主 hierarchy/field 消费已经转向 `WorldInspectionArtifact`：hierarchy rows 使用 `Arc` publication，字段可按 entity 失效，name 变化能重算局部 row/ancestor hash。当前 editor viewport、host publication 和 workbench snapshot 均消费 artifact，而不是每次调用 legacy `inspect_hierarchy`。因此 finding 只针对仍保留的 legacy/dynamic gateway query 与 artifact 上游的全域 dirty，不否定这条增量链。

### 3.3 Dynamic scene、reload 与 LevelSystem

DynamicScene root document 使用 `$zircon.header.schema_version=2` 和显式 v0→v1→v2 migration；spawn 有 generation validation 和预览/提交分离；asset reload queue 有 entry/byte/time budget、single-flight、coalescing 与 reconciliation。LevelSystem 用独立 world lock、replacement epoch 和 frame-state publication约束 replacement 竞争；产品 render path直接在 World owner 上构建 prepared extract，不再必然先 clone World。

这些机制说明问题不是“没有版本、没有 transaction、没有 async queue”，而是当前版本粒度、持久化类型粒度、World 身份粒度和 frame delta 粒度仍不足以闭合产品主链。

## 4. 差距清单

### P1-1：公开 live entity 身份没有 World 与 generation，持久化身份和运行时句柄混为裸 `u64`

**证据**

- `zircon_runtime/src/scene/mod.rs:37` 与 `zircon_runtime_interface/src/world_sync/query.rs:6` 都定义 `EntityId = u64`。hierarchy、selection/highlight、object address、inspection、event、ABI query 和 scene document 均直接传播该值。
- 内部已经有 `InternalEntity { index: u32, generation: u32 }` 和 `EntityRegistry` 的 stable→internal 映射，但 generation 不随 public handle 离开 World；public identity 也不携带 `WorldId/WorldGeneration`。
- `NodeRecord`/DynamicScene 会保存 stable `u64`，spawn/restore 再通过 remap 安装到目标 World。`next_id` 只能降低单一 World 内的即时复用概率，不能阻止跨 World、World replacement、Play snapshot 或长期引用发生同值别名。

**风险与目标契约**

编辑器选中实体、异步任务、插件 callback、网络/远程 query 结果或 render instance 若跨过 World replacement，当前只能用“目标 World 里是否还有同一个 u64”判断，无法区分原对象与新对象。目标必须分离三种身份：`LiveEntityKey(index,generation)` 只在一个 World epoch 内寻址；`WorldHandle(id,epoch)` 约束句柄所属 World；`SceneEntityGuid`/authoring object id 用于持久化、prefab override、跨加载引用。instantiate 明确产出 persistent→live remap；ABI/watch/highlight/object address 都携带 world epoch 并在过期时返回 `StaleWorld/StaleEntity`，不能静默命中新实体。

Bevy `Entity` 明确编码 index+generation，Fyrox `Handle<T>` 也用 pool index+generation 校验。Zircon 还需要补上多 World/replace epoch 和独立 persistent GUID，不能只把 internal 64-bit bits 公开后继续序列化 live handle。

### P1-2：`World::clone` 与 serde 是硬编码内建组件投影，会静默丢失任意 typed component

`World::clone` 在 `world/world.rs:143-221` 只取得 persistent entity/render/physics/lighting/2D/animation 的固定 snapshot，重建 registry/table，再复制 dynamic JSON、resources、schedule、observers 等部分 runtime state。`WorldPersistentState`/Ref 在 `226-307` 明列 Name、Hierarchy、Transform、camera/mesh/light/physics/animation 等固定 map；`Serialize` 在 `309-351` 只写这些字段，`Deserialize` 在 `354-462` 重置 registry/resource/event/observer/schedule 等并注册 builtins。

`Component` trait 本身没有 clone/persistence/transient/checkpoint policy。插件或业务代码注册的任意 typed table/sparse component，即使已有 reflection registration，也不会自动进入 `World::clone`/serde；调用成功且没有 structured loss report。与此同时 `LevelSystem::snapshot()` 直接 `self.lock_world().clone()`，多个 dynamic-session level/world capture facade 依赖这条深 clone；公开 `World::build_viewport_render_packet` 也仍通过 clone 构造可变 prepared world。

目标删除“`Clone for World` 等于完整世界副本”和“`Serialize for World` 等于完整持久化”的假契约。组件注册必须声明 `PersistencePolicy`、`Clone/ForkPolicy`、`PlayTransferPolicy`、`CheckpointPolicy` 和 codec/schema；canonical scene save 只经过 versioned scene document，遇到 required but nonserializable/unknown component 必须失败并列出 entity/type。运行时 fork/checkpoint 是独立事务，按注册策略封存 immutable component columns和 runtime-domain snapshot，不复用 authoring JSON，也不复制 observer/task/queue 等不可转移状态。

### P1-3：component/resource 只有 scene root 版本，没有逐类型 schema identity 与迁移链

`DynamicComponent` 只保存 `type_path`、`plugin_owned` 和 fields；`ReflectTypeRegistration` 保存 type path、field info、serialization/visibility/plugin id，但没有 type schema id/version、layout hash、minimum supported version或 migration owner；plugin `ComponentTypeDescriptor` 同样只有 type id/plugin/display/properties。DynamicScene 的全局 v2 migration 能迁移 document envelope，却无法表达“某个插件组件从 v3 字段布局升级到 v4”。

插件缺失或新版本加载时，document 中的 dynamic component 只能按字符串 path 和当前 field metadata解释；字段重命名、type change、拆分/合并、entity reference语义变化没有连续迁移链。World serde deserialization又只恢复 builtins和 dynamic JSON，不会保存/验证原插件 schema catalog。结果是 root document可读不代表其中每个组件可正确读取。

目标为 component/resource 注册稳定 `TypeSchemaId + version + fingerprint`，scene row记录写入时版本；registry提供连续 `N -> N+1` migration、missing-plugin opaque preservation policy和 deterministic diagnostics。load分为 parse envelope、resolve type catalog、migrate per type、validate references、prepare instantiate、commit；任何未来版本、缺失中间 step、duplicate field或 incompatible entity-reference type 都在 commit 前失败。热重载时新旧 plugin schema generation 必须与 staging plan绑定。

### P1-4：真正读取 World 的 ECS 系统不能并行，访问冲突图目前只并行“worldless command producer”

`SceneSystem::supports_worker_dispatch` 要求 WorkerSafe、`supports_worldless_execution()`、无 ordering constraints 且没有 conservative World access。`WorldlessSystemParam` 是 sealed marker；实现仅有 `()`、`CommandsParam`、`LocalParam<T>` 及其 tuple。`Query`、`Res/ResMut`、Events/Messages 都只实现普通 `SystemParam`。`SceneScheduleRunner::run_stage` 因而把这些真实 ECS 系统逐个放进 `level.with_world_mut` 调用 `run_native_scene_system`；能组成 worker batch 的系统不能读取 World，只能依赖 local state并返回 deferred commands。

schedule plan cache、access descriptor和 conflict graph本身是有价值的，但它们尚未提供并行 borrow/column partition的安全证明。两个分别只读/写不相交 archetype columns或resources的系统仍争同一 `&mut World`，无法兑现 ECS 数据布局的并行收益。

目标 executor 从 compiled plan 取得 component/resource/event access token，以 `UnsafeWorldCell` 等受审计边界把互不冲突的 table column/resource slot 分区借用；结构变更、non-Send/foreign callback和明确 exclusive system构成 barrier。query可进一步按 archetype chunk拆分任务，worker只写本 chunk changed ticks/command lane；ApplyDeferred合并使用 deterministic system/chunk key。Bevy multi-threaded executor以 `component_access_set().is_compatible` 锁定运行中访问集；Unreal Mass以 archetype chunk query/batch command为并行边界。Zircon应吸收其证明方法，而不是复制 unsafe 细节。

### P1-5：层级和派生状态仍是 domain-global dirty，单点变化会触发全 World 重建链

`DerivedStateDirty` 只有 `hierarchy/active/transforms/node_cache/render_extract` bool；mark hierarchy会同时置脏后四域，mark transform也会置脏整个 transform/node/render 域。`run_internal_scene_system` 依次执行 `rebuild_hierarchy_validity`、`rebuild_active_in_hierarchy`、`rebuild_world_matrices`、`refresh_node_cache` 与 `prepare_render_extract`。

current source 已经引入 `HierarchyMutationIndex`，维护 stable roots/children，并用显式 DFS 取代递归；这修复了重复建临时 child map和深链栈风险。但 index 的 dirty/current 仍是 whole-domain，现有 Runtime07 failure也明确记录 dirty frontier、NodeCache/render/inspection增量投影未完成。任何 rename/reparent/transform/active变化仍可能扫描/复制与受影响 subtree无关的实体，NodeCache仍维护第二份宽 `Vec<SceneNode>`。

目标由 hierarchy topology generation拥有 parent、ordered child range、topological/depth与dirty roots；reparent事务只验证 changed edge和affected ancestors/subtrees。Active/WorldTransform按dirty frontier迭代传播，changed ticks产出 added/changed/removed entity ranges；NodeCache、inspection、render extract消费同一sealed delta，而不是各自维护全域 bool。稳定 generation 的 visit/build/clone计数必须为0，单点变化成本接近受影响 subtree。

### P1-6：Scene→RenderExtract 仍按视图全量扫描、排序与 clone，公开 World 路径还会深 clone World

产品 `LevelSystem` 路径已经避免必然 clone World，这是进展；但 `World::build_prepared_render_frame_extract_for_request` 每个 request仍收集 meshes/phase/material overrides、sprites、particles、五类 lights、visibility、post process、volumetric ids与camera DTO，并排序/clone morph weights、layer masks、material overrides等。多 camera/view会重复执行 view-independent scene materialization。`World::build_viewport_render_packet` 在 `world/render.rs:43-50` 仍先 `self.clone()`，再运行 prepared systems。

当前没有唯一的 `WorldExtractGeneration`、stable instance slot、changed/removed ranges和多 view共享 payload。`world_generation` 只能判断整个 World是否变化，不能让 renderer知道哪些 instance/light/material row应重用或移除；这也使GPU scene与增量upload无法从上游获得可靠delta。

目标在frame seal只构建一次view-independent `SceneExtractGeneration`，内部用Arc/column pages和stable render instance key发布 added/changed/removed ranges；每个view只选择camera、culling layers和view-dependent visibility，引用共享payload。render owner以generation验证旧资源退休，不从World clone获得可变权限。公开 `World` render API迁移到只读 sealed extract request后删除 clone fallback。Renderer/GPU上传细节由`09/10`拥有，但scene必须提供唯一上游delta。

### P1-7：DynamicScene capture 与编辑器 Play snapshot 是同步 `O(entities × registered types)` 扫描并生成 pretty JSON

`dynamic_scene/scene/capture.rs:14-23` 先 `world.node_records()` materialize全体 SceneNode；每个 entity在 `97-121` 遍历完整 `type_registry`，逐 type调用 adapter.contains/read_fields，再过滤serializable field并排序。因此组件稀疏时仍按 N×T探测。捕获完成后，`zircon_editor/src/core/play/snapshot/source.rs:17-22` 在调用线程执行 `DynamicScene::from_world` 和 `to_versioned_json_pretty`，把完整文本存入 `Arc<str>`。

进入 Play、未保存场景预览或大型 checkpoint会在编辑器主流程同时承担全量 node clone、反射探测、field allocation、JSON tree和pretty string成本；当前没有 sealed source generation、进度、取消、预算或background snapshot。根文档字节限制也不能解决生成期间的CPU/working-set峰值。

目标 capture按archetype/registered column遍历，只访问实际存在且serializable的component columns；entity reference在同一remap pass处理。frame/authoring owner先封存只读 scene generation，后台job按page产出binary/compact intermediate，支持progress/cancel/deadline和generation mismatch重试；pretty JSON仅作为最终人类可读authoring artifact，不作为Play进程内部传输。进入Play必须有10k/100k实体和100/1000注册类型的延迟、clone bytes和峰值内存门槛。

### P1-8：dynamic runtime `query_world` 是无计划、无分页、无预算的全量反射查询

`WorldQuery` 只有 `with/without/select/generation_hint`，没有limit、cursor、projection byte cap、deadline或cost class；结果是完整 `Rows(Vec<EntityRow>)`。`World::query_world` 先 `node_records()`，再对每个实体调用 `inspect_fields`；字段构建继续遍历可见reflection registration并分配value，最终对全部结果排序、转 `serde_json::Value`。dynamic ABI直接解析request、同步执行query并一次性JSON encode到owned byte buffer。

generation_hint只优化“整个 World完全不变”，不能处理小变化、慢consumer或超大response。一个本地editor/plugin query即可长时间占用World read/mutable gateway lane并分配巨型buffer；remote-visible与editor-visible字段也没有在query plan层形成独立权限/预算。

目标 `WorldQueryPlan` 在schema generation上编译 type ids/column slots，按matched archetype执行；接口必须有page size、opaque cursor(world/schema/query generation)、max result bytes、deadline/cancel与structured `BudgetExceeded/StaleCursor`。changed-since查询消费inspection/world delta，只返回affected rows；ABI使用有界chunk/batch writer或subscription，不构造单个无限JSON buffer。权限过滤在plan compile前完成并计入diagnostics。

### P1-9：runtime 没有 scene partition/level streaming 权威，编辑器“Level Streaming”目前只是预览控制与反馈文本

对 runtime scene/interface 搜索没有找到 world partition、streaming level、data layer、spatial cell、sublevel 或scene partition的数据模型和生产owner。命中集中在 editor retained-host navigation/template bindings和诸如“preview queued/load queued”的feedback字符串；这些控制没有连接到runtime residency、dependency、activation或unload transaction。

单一 `World` + DynamicScene整体spawn不能承担开放世界。缺少cell/level identity、spatial bounds、dependency/data layer、desired/resident/visible/active状态、异步load priority、activation budget、cross-cell entity reference、HLOD proxy、origin rebasing和失败回退。若UI先宣称功能存在，会把导航规格误当成产品能力。

目标建立 `WorldPartitionDescriptor` 与不可变catalog generation；cell/level有persistent id、bounds、data layers、dependencies、cook artifact sections和state machine `Unloaded -> Requested -> Loading -> Staged -> Activated -> Visible`，失败/取消/evict有明确回滚。spawn在staging world/partition lane完成schema/reference validation，frame boundary按预算激活，cross-cell soft reference通过persistent GUID resolver。Unreal `UWorld`明确区分persistent level、streaming levels、World Partition和tick group；Zircon不必复制Actor历史模型，但必须具备等价的ownership/residency边界。Editor Level Streaming UI只能在runtime contract和真实product workflow接通后解除preview标记。

### P1-10：Events只在“保留两代”意义上有界，单帧entry/byte仍可无限增长

`Events<T>` 使用 `current/next Vec<T>`；`send`直接push，`send_batch`按producer size_hint reserve并push全部事件。capacity policy只在后续空闲帧debounce shrink，metrics只有len/capacity/high-water/shrink。producer可在一次frame或一次callback中写入任意条数/任意payload，既没有entry/byte cap，也没有error/backpressure/drop/coalesce policy。

Messages已经有entry/declared-byte/age hard budget，EventStore也有active worklist；因此旧failure可关闭“messages永久增长”和“idle全channel扫描”的源级问题，但不能把event双缓冲称为严格bounded。目标按channel声明delivery class：critical lossless采用有界producer error/backpressure，state change采用key coalesce/latest，telemetry采用drop oldest/newest；每类有entry、charged-byte、per-frame producer budget、dropped/gap和largest-payload diagnostics。stage flush和reader generation语义保持deterministic。

### P1-11：DynamicScene session 形成组合式 API 爆炸，却没有对应产品工作流消费

`dynamic_scene/session` 当前有565个Rust文件、约9,399行、547个public function、367个唯一public function名。目录把path/loaded/source_path、named/selected、basic/metadata、commit/preview、world/level、retention/global/tag、copy/import/export/merge/restore等维度做笛卡尔展开，大量mod节点只有少量转发。排除session自身、re-export和tests后，没有在zircon_app/editor/runtime产品代码中找到`RuntimeSessionArchive/Slot/Metadata`消费者；命中主要是runtime absorption结构测试。

这不是否定archive core、atomic writer、retention policy或preview report本身，而是产品需求尚未闭环时先维护数百个组合入口。多条world/level capture wrapper又调用`LevelSystem::snapshot`，把P1-2的深clone和数据策略缺失扩散到巨大surface。结构测试会进一步锁死文件树与方法组合，使任何合理hard cut成本上升。

目标收敛为一个typed request/command model：selector、source、operation、mode、metadata/retention options是数据，不是每种组合一个方法/文件。统一prepare→preview/report→commit transaction和一套path/loaded source adapter；先接通一个真实产品save/checkpoint/restore workflow并做故障恢复，再按消费者证据增加convenience API。完成consumer迁移后删除组合facade、root re-export与source-shape guards，不保留deprecated wrapper。

### P1-12：World 生命周期没有明确的 authoring/runtime fork、rollback 与 domain transfer policy

目前`LevelSystem::snapshot/replace_world_if_generation/transaction`、DynamicScene capture/spawn、PlaySceneSource JSON和World clone各自承担一部分“复制/替换世界”语义。它们对resource、schedule、observer、event/message、command queue、change tick、inspection cache和subsystem frame state采用不同的copy/reset策略，且没有一张权威transfer table说明进入Play、退出Play、checkpoint、undo preview、hot reload和level streaming分别保留什么。

`World::clone`会clone schedule/resources/observers/command errors等，重置subscriptions/deferred resolutions，events/messages的clone语义又由各store决定；serde则重置更多状态。调用方无法仅从API名称判断这是authoring document、runtime fork、frame snapshot还是disaster checkpoint。失败时也没有统一rollback token和partial domain report。

目标定义五个不可混用的边界：AuthoringDocument、RuntimeWorldFork、FrameExtract、Checkpoint、PartitionStaging。每种由注册表声明component/resource/domain policy和reference remap，输出`TransferReport`；prepare期间不修改目标，commit以World epoch和schema/catalog generation做compare-and-swap，失败不发布partial world。退出Play只应用显式允许的authoring delta，不把runtime-only component/event/task回写场景。

### P2-1：Observer callback 同步持有 `&mut World`，没有递归、panic、工作量和事务边界

public `observe_event/observe_entity_event/observe_component_lifecycle`接收可任意修改World的callback。dispatch会先取得Arc callbacks snapshot，再同步调用；这避免了observer registry自身的可变借用冲突，但callback可递归触发同类event/lifecycle、执行结构变更或panic。当前没有max dispatch depth、per-trigger callback/command/time budget、panic isolation、reentrant policy或失败后的World mutation report。

目标默认observer只接收只读event context并写入deferred command lane；需要exclusive mutation的observer在明确barrier执行。dispatch context携带depth/trace id/budget，超限产生structured failure；foreign/plugin callback必须catch panic/ABI error并隔离candidate commands。是否允许同event递归要按event type显式声明，不能依赖调用栈自然终止。

### P2-2：测试数量很大，但source-shape守卫与未执行规模验收削弱了工程置信度

scene全树有1,001个`#[test]`，至少47个文件使用`include_str!`读取源码；可见大量`.contains(...)`、目录/owner tree、方法名和源码片段断言。它们适合守住hard cut和owner边界，但不能证明aliasing、并行system correctness、World transfer数据完整性、100k hierarchy增量成本、multi-camera extract复用或single-frame event budget。

现有Runtime07/08 failure也多次明确“source实现已闭包但managed validation/performance receipt未完成”。目标保留少量结构守卫，同时用property/generative tests覆盖archetype move/serde/schema migration/entity stale handle，用loom/shuttle或受控thread harness覆盖scheduler，使用确定性visit/clone/alloc counters和managed Windows benchmark证明规模行为。没有terminal receipt前不把source字符串测试计为feature complete。

## 5. 参考实现适配矩阵

| 目标边界 | 参考源码证据 | Zircon 应吸收 | 不应照搬 |
|---|---|---|---|
| live entity identity | Bevy `Entity` index+generation；Fyrox `Handle<T>` pool generation | stale handle可判定、typed live key | 把live key直接当长期scene GUID |
| access-based scheduler | Bevy multi-threaded executor的access compatibility/running access；Unreal Mass chunk query | compiled access token、conflict-aware并行、chunk task | 未审计unsafe或Actor tick全部历史层 |
| archetype/batch mutation | Unreal Mass archetype manager、chunk iteration、BatchCreate/Destroy/command | chunk-owned rows、batch structural barrier | 把Mass和authoring scene强行统一成一种对象模型 |
| scene serialization | Bevy reflection dynamic world；Godot PackedScene owner/property usage/instance state | 注册驱动的持久化策略、owner/instance边界、remap | 仅靠字符串type path或全局document版本 |
| hierarchy/process partition | Godot Node process thread group；Unreal tick group | 明确affinity/group/barrier和层级owner | 允许任意node callback隐式并行 |
| world/level streaming | Unreal persistent level、streaming levels、World Partition | partition catalog、cell state machine、budgeted activation | 复制UE兼容性债务和全Actor序列化历史 |
| frame/extract | Unreal Mass chunk与World tick group提供阶段边界；Unity Graphics细节留后篇 | sealed generation、delta range、多view共享 | 在scene报告中臆测GPU实现优越性 |

## 6. 目标架构

### 6.1 Identity 与 World epoch

```text
SceneEntityGuid (persistent, document/prefab/cross-cell)
        |
        | instantiate/remap
        v
WorldHandle { world_id, epoch }
LiveEntityKey { index, generation }
        |
        +-- EditorObjectAddress { world, entity }
        +-- Runtime ABI entity handle
        +-- RenderInstanceKey (separate stable render slot)
```

任何跨异步/ABI/frame边界的live handle都验证world epoch和entity generation；任何持久化引用都经SceneEntityGuid和soft resolver，不保存dense row/index。

### 6.2 World data 与 scheduler

- World拥有archetype tables、sparse sets、resources、hierarchy topology和schema catalog generation的唯一authority。
- Schedule compile产出stage DAG、system access sets、archetype/resource bindings和exclusive barriers；run时只借用计划允许的columns/slots。
- structural commands按system/chunk稳定key进入per-worker lane，barrier统一preflight和commit；observer/plugin callback不能绕过该边界。
- component tick、hierarchy dirty frontier、inspection delta和render delta从同一mutation commit产出，不允许每个consumer再次扫描World推断变化。

### 6.3 Scene document、fork 与 partition

- `AuthoringSceneDocument`：版本化type catalog、persistent GUID、component columns、prefab/instance/override和soft references。
- `RuntimeWorldFork`：从sealed authoring generation创建live entity remap，按注册policy复制/初始化runtime domains。
- `Checkpoint`：显式列出可恢复runtime state和不可恢复项，带schema/version/epoch，不等价于scene save。
- `PartitionCatalog`：cell/level bounds、data layers、dependency/cook section和状态机；staging与active World分离。
- `SceneExtractGeneration`：只读frame publication，含稳定slot与delta；多view只产生轻量selection。

## 7. 硬切范围

1. 引入World-aware generational live handle并迁移所有生产消费者后，删除public裸`EntityId=u64`作为live handle的接口；wire兼容迁移只允许一次离线/document migration，不保留运行时双身份。
2. 删除`Clone for World`和直接`Serialize/Deserialize for World`；调用点必须选择authoring document、runtime fork、checkpoint或test fixture builder。
3. 删除以worldless为唯一worker条件的scheduler路径；完成access-token executor后，worldless只作为普通零访问系统，不再拥有平行执行语义。
4. 删除domain-global derived bool作为最终dirty authority；过渡期可由dirty frontier派生“是否有工作”，不能长期双写两套真值。
5. 删除公开render clone fallback和per-view view-independent全量extract；只保留sealed generation入口。
6. DynamicScene session request model接管后，删除组合式path/named/selected/basic/metadata/preview/commit wrappers、re-export和对应source-shape测试。
7. runtime partition owner接通前，Editor Level Streaming入口必须明确为preview/nonfunctional；接通后删除hardcoded fake feedback，不能保留双路径。

## 8. 测试先行的重构里程碑

| 里程碑 | 最低层工作 | 先失败的验收 |
|---|---|---|
| M0 | 固化current source、owner与benchmark基线 | 记录1/1k/100k entity、type、depth、view和event规模的visit/clone/alloc/latency counters |
| M1 | World epoch、generational live key、persistent GUID与remap | stale world/entity、cross-world same-index、replace race、serialized live-key rejection |
| M2 | component/resource transfer policy与per-type schema migration | custom typed component clone/save不再静默丢失；future/missing/migration-gap原子失败 |
| M3 | 删除World clone/serde，建立AuthoringDocument/RuntimeFork/Checkpoint | Play/save/checkpoint各自golden corpus和domain transfer report |
| M4 | access-token scheduler与chunk task | disjoint Query/Res真实并行；conflict/exclusive/ApplyDeferred确定性；panic取消不发布partial commands |
| M5 | hierarchy dirty frontier与统一delta | stable visit=0，single transform/reparent近affected subtree，100k深链无栈溢出 |
| M6 | sealed SceneExtractGeneration与multi-view reuse | view-independent build每frame最多1次，stable upload delta=0，removed instance精确退休 |
| M7 | archetype-column DynamicScene capture、Play background snapshot、bounded query plan | N×T探测归零；分页/byte/deadline有效；cancel/generation mismatch不发布旧snapshot |
| M8 | bounded events与observer dispatch policy | 1M producer严格按policy受限；drop/gap/backpressure可观测；递归/panic不破坏World commit |
| M9 | partition catalog、cell staging/activation/eviction与Editor真实工作流 | cell依赖、跨cell soft ref、失败rollback、activation budget、重新打开项目一致 |
| M10 | session facade hard cut与产品checkpoint workflow | 产品consumer迁移完成，565-file组合surface删除，无compat shim/source-shape残留 |

M1-M4是共享正确性层，必须先于partition/editor/renderer扩展。M5-M7可在identity/transfer contract稳定后分工推进；M9不得通过硬编码演示cell绕过asset semantic section和World transaction。

## 9. 验证矩阵

| 层级 | 必测内容 | 关键指标/失败条件 |
|---|---|---|
| unit/property | entity allocator、archetype move、schema migration、remap、dirty frontier、event policy | stale alias=0、row/value/tick守恒、迁移deterministic/idempotent |
| concurrency | scheduler access、structural barrier、World replace、snapshot cancel、observer panic | data race/partial publish=0；同seed command order一致 |
| scale | entities 1/1k/100k，types 1/100/1000，depth 1/1k/100k，views 1/8/64 | stable visit/clone=0；single change近affected set；stack overflow=0 |
| serialization | builtin/custom/plugin/missing/future component，prefab/cross-cell refs | 未声明数据丢失=0；错误含entity/type/version/provenance |
| product | save/reopen、enter/exit Play、runtime process gateway、checkpoint/restore、cell load/unload | UI不假成功；失败可恢复；旧World handle确定失效 |
| performance | schedule parallel speedup、Play snapshot、query pages、extract reuse、event burst | 记录p50/p95/p99、alloc/clone bytes、lock wait、visits、builds；无数据不宣称优于UE |
| fault injection | plugin missing/schema gap、I/O cancel、OOM/admission reject、callback panic、cell activation失败 | last-good World/partition保持；无partial document/World publish |

## 10. 与既有计划的冲突和需要重开项

- `05-scene-editor-boundary-closeout.md` 的completed状态只证明scene authoring/editor ownership和命名边界的既有里程碑，不代表entity identity、World transfer、partition或ECS execution完整。它不应被用作整个Scene系统已完成的证据。
- Runtime08 archetype columnar storage与lazy change detection在current source已完成主要production hard cut；本文不重开已删除的per-component table或eager wrapper问题，只要求managed validation receipt和在新scheduler/extract中的集成验证。
- Runtime08 event/message failure需要拆分状态：message budget和event active worklist可标记source closure；event单帧entry/byte hard cap仍open，由本文P1-10继续追踪。
- Runtime07 world-derived-state failure的2026-08-13更新准确：iterative traversal与共享HierarchyMutationIndex已完成，dirty frontier、NodeCache/render/inspection统一delta仍open。本文P1-5是其current-source E3重申，不另造第二实现owner。
- Runtime07 frame snapshot已有replacement epoch和部分domain publication，但SceneExtractGeneration、多view共享和World transfer policy未完成；本文P1-6/P1-12提供上游契约，不能由frame-state API存在而宣称关闭。
- performance计划当前有活跃会话租约并覆盖World/ECS/frame extract相关未来子计划。本文只写review文档，不修改其文件；实现前必须由coordinator重新授权并二次复核current source。

## 11. 需要避免的临时实现

- 不通过给裸`u64`再配一张全局generation map来兼容旧API；身份必须在handle和wire contract中可验证。
- 不在硬编码WorldPersistentState后继续追加每个新组件map；持久化由注册策略和column codec驱动。
- 不把World锁换成`RwLock`后就宣称ECS并行；必须证明column/resource access与structural barrier。
- 不把全量derived rebuild扔进worker掩盖work量；先建立dirty frontier和唯一delta owner。
- 不以缓存上一帧巨大DTO代替sealed generation；缓存必须有stable key、changed/removed ranges和退休规则。
- 不用后台线程生成同样的N×T pretty JSON后宣称Play snapshot完成；算法、working set、取消和generation验证都必须收敛。
- 不用Editor mock cells、固定名称或feedback字符串宣称level streaming；runtime residency/activation/failure才是完成条件。
- 不保留565个session wrapper作为“兼容层”；无外部稳定consumer时应直接hard cut。

## 12. 状态与产出记录

| 日期 | 产出 | 状态 | 证据边界 |
|---|---|---|---|
| 2026-08-15 | Scene/ECS/World lifecycle首轮current-source差距审查 | review_complete；implementation_pending；recheck_required | 静态生产链、测试与仓内参考源码E3；未运行Cargo/scale/product benchmark，不声明性能或产品验收通过 |
