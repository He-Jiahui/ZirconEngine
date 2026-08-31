---
title: Runtime Scene、ECS、World 与 Level 生命周期当前源码复核
category: zircon_runtime
report_id: Runtime162
review_date: 2026-08-30
baseline_head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
verification_head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
canonical_owner: Runtime05
refreshes:
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
related_reports:
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99k-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-current-source-review.md
related_code:
  - zircon_runtime/src/scene
  - zircon_runtime/src/core/framework/scene
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime_interface/src/reflect
  - zircon_runtime/src/dynamic_api
  - zircon_editor/src/scene
  - zircon_editor/src/core/gateway
  - zircon_editor/src/core/play
  - zircon_app/src
tests:
  - zircon_runtime/src/scene/tests
  - zircon_runtime/src/scene/ecs
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_editor/src/core/play
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Level.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/LevelStreaming.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/GameInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassEntityManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassEntityQuery.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Internal/MassArchetypeData.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassArchetypeData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SceneComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/SceneComponent.cpp
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_ecs/src/storage/table/mod.rs
  - dev/bevy/crates/bevy_ecs/src/query/state.rs
  - dev/bevy/crates/bevy_ecs/src/query/access.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
  - dev/bevy/crates/bevy_ecs/src/hierarchy.rs
  - dev/bevy/crates/bevy_transform/src/systems.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world_builder.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world.rs
  - dev/bevy/crates/bevy_world_serialization/src/components.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/Fyrox/fyrox-graph/src/lib.rs
  - dev/Fyrox/fyrox-impl/src/scene/mod.rs
  - dev/Fyrox/editor/src/scene/container.rs
  - dev/godot/scene/main/node.h
  - dev/godot/scene/main/node.cpp
  - dev/godot/scene/main/scene_tree.h
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/scene/resources/packed_scene.h
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/godot/scene/3d/node_3d.h
  - dev/godot/scene/3d/node_3d.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/RenderWorld.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/ResourceReloader.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_identity_transfer_parallel_streaming_and_product_closure_incomplete
source_recheck_required: true
working_tree_drift_observed_after_snapshot: true
---

# Runtime162 · Scene、ECS、World 与 Level 生命周期

## 1. 结论

当前 Scene/ECS 并不是一个只为演示临时拼出的空壳。archetype table、sparse storage、compiled query plan、stable query order、change tick、deferred structural command、replacement epoch、bounded inspection query、DynamicScene migration、Play domain link、层级 dirty frontier、render dirty journal 和 render component change artifact 都是可以继续工程化的真实基础。尤其是当前工作树已经修正了三类旧的高风险源码形态：`Query` 的 cache-refreshing 入口要求 `&mut self`，removed-component history 变成有界 sequence window，dynamic component 注册在 publish 前完成两套 registry 的 preflight。它们应由 Runtime60/63 的 canonical owner 单独复核关闭条件，不能继续机械引用旧报告措辞。

但 Runtime05 的总体判断仍成立：公开 live entity 继续是无 World/epoch/generation 的裸 `u64`；`World::clone`、serde 与 project scene document 继续以固定 built-in map 决定数据守恒；真正读写 World 的系统仍不能进入 worker 并行；component/resource 没有逐类型持久 schema 与迁移链；DynamicScene capture 继续执行同步 `O(entities * registered types)` 反射探测并生成 pretty JSON；Level 没有 partition、cell、data layer、residency 和 budgeted activation authority；普通 Events 单帧仍可无界增长；组合式 session surface 没有产品 consumer；observer 仍可在同步 `&mut World` callback 中递归、panic 或执行无预算工作。

因此当前实现不能声称达到 Unreal、Bevy、Fyrox、Godot 的 Scene/ECS 生命周期完整度，也没有证据声称性能优于 Unreal。局部 frontier 或 journal 只能把对应 finding 判为 Partial；没有 world-aware identity、注册驱动的数据守恒、真实 access-token 并行、sealed scene delta、partition streaming 和接受的 100K/1M 规模证据之前，不能把局部代码量当作系统完成度。

本报告刷新 Runtime05 的 14 项 canonical finding，**不新增唯一 finding**。12 项 P1 当前为 **8 Open、4 Partial、0 Closed**；2 项 P2 为 **2 Open、0 Partial、0 Closed**。7 项工程门为 **4 Fail、3 Partial、0 Pass**。Runtime60/61/62/63 的专项 P0 只记录当前源码移动，不在本报告重复计数或擅自关闭。

## 2. 审查边界与证据

### 2.1 当前源码选择

统计口径为 UTF-8 physical lines、non-empty lines、bytes、精确 `#[test]` / `#[ignore]` 和包含 `unsafe` 的物理行；fingerprint 为排序后的 lowercase `path<TAB>SHA-256<LF>` 再做 SHA-256。选择集包含未提交的当前工作树，因为本轮目标是复核用户正在建设的真实 current source，而不是只审 HEAD。

| 选择集 | files | lines | nonempty | bytes | tests | ignored | unsafe lines | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Runtime Scene 全树 | **1,167** | **133,180** | **121,051** | **4,839,291** | **1,291** | **0** | **205** | `130464fd1ba5f5502cc052908cbd01140e3483dbd67a6f0555d8737a7aa5ced1` |
| Scene framework / world-sync / reflect contracts | **46** | **2,738** | **2,442** | **82,332** | **9** | **0** | **0** | `c2179755043d41b76f1051093cf93253370105e6c81671cbddbba850cce92e2b` |
| Editor Scene / gateway / Play consumers | **224** | **24,442** | **22,087** | **825,893** | **220** | **0** | **65** | `ab7330e4bc6812a2b26c41ca3518d26dc7cc2599657baf258451e96b9a5219df` |
| Dynamic API / App / scene asset-project I/O product boundary | **401** | **76,449** | **70,080** | **2,814,066** | **997** | **0** | **426** | `677b9176f7920329409821e3ca73137ed41d8ca34d740384f1cc09f29e06c0b9` |
| Deduplicated Zircon union | **1,838** | **236,809** | **215,660** | **8,561,582** | **2,517** | **0** | **696** | `297b31667668a0127697bb8c48790860be1b7c0810dc6306141a234cc4d096ae` |
| Unreal / Bevy / Fyrox / Godot / Unity Graphics reference | **38** | **56,268** | **47,701** | **2,216,175** | n/a | n/a | n/a | `14300e2be65652da4cb7bb1a13704aad2dee1394303b8a14da084a779fbd8e29` |

上表是成文时冻结的可重建证据快照。静态验收期间再次观察到 Scene 工作树字节漂移，说明其他工作仍在并发写入；本报告不通过循环追逐一个持续变化的目录来伪造 terminal currentness。`source_recheck_required` 因此保持为 `true`，任何实现或关闭判断都必须重新读取对应owner文件、重取fingerprint并运行该owner的动态资格门。

本轮只做静态 review 与文档更新，没有运行 Cargo、Editor、Runtime DLL、产品进程、fault、fuzz、Miri、loom、scale、soak 或动态 benchmark。Tooling 按用户要求排除，也没有查询、轮询、等待或实时跟踪协调器状态。

### 2.2 当前产品主链

```text
Authoring World
  -> DynamicScene::from_world
      -> node_records
      -> for each entity: iterate complete type registry
      -> reflected components/resources
  -> versioned pretty JSON PlaySceneSource
  -> App-owned PlaySessionFactory / isolated runtime session
  -> PlayDomainLink { Edit | Play(instance) }
  -> runtime World / inspection artifact / prepared render extract

World mutation
  -> table/sparse/change ticks
  -> hierarchy/active/transform/node-cache dirty frontier
  -> render dirty journal
  -> mesh-render component change artifact
  -> prepared render extract / Editor projection
```

这条链已经出现明确的 Edit/Play transport separation，但 capture、transfer policy、component persistence、entity identity 和 scene publication 仍各自使用不同粒度的真相。`PlayDomainLink` 不拥有 authoring World 是正确方向，却不能替代 RuntimeWorldFork、Checkpoint、PartitionStaging 和逐类型 transfer table。

### 2.3 可保留基础

| 区域 | 当前事实 | 保留与收束方向 |
|---|---|---|
| ECS storage/query | archetype table、sparse locator、compiled cache、stable order、duplicate mutable entity rejection 已存在 | 收束为 generation-qualified plan 与可并行 column/chunk lease |
| Structural mutation | bundle/worker command 有 preflight、deferred lane 和 barrier；DynamicScene spawn 有 staging/commit | 统一为 mutation transaction，observer/plugin 只能提交 candidate command |
| Change tracking | table/sparse/resource ticks、removed-component bounded sequence window、inspection generation 已存在 | mutation commit 一次发布 component/hierarchy/inspection/render delta |
| Derived state | active、transform、node cache 已使用 entity frontier；hierarchy topology index 已存在 | hierarchy edge 也改为 frontier/generation，消除全域修复与二次扫描 |
| Render handoff | dirty journal 和 mesh-render component artifact 支持 generation、upsert/remove、history-loss fallback | 扩展为 view-independent `SceneExtractGeneration`，覆盖全部 renderable/light/camera/particle domain |
| Dynamic scene | v2 envelope、v0->v1->v2 migration、spawn validation、bounded asset-reload queue 已存在 | 增加 per-type schema、column capture、compact artifact、background/cancel/generation fence |
| Runtime inspection | generation hint 和 items/bytes/depth/time bounded execution 已接入 Dynamic API | 公共 contract 增加 compiled plan、page/cursor、changed-since、cancel 和 permission receipt |
| Editor/Play boundary | `WorldDomain`、`PlayDomainLink`、App-owned session lease、edit policy、bounded pending edit queue 已存在 | 明确 fork/rollback/allowed authoring delta；禁止用 JSON/World clone 隐式决定 transfer |

## 3. Runtime05 canonical finding 当前重判

### 3.1 P1

| ID | 状态 | 当前源码证据 | 与参考实现的差距 | 必须重构 |
|---|---|---|---|---|
| `R05-P1-01` | **Open** | `scene/mod.rs` 与 world-sync 继续公开 `EntityId = u64`；`WorldHandle(pub u64)` 不含 epoch。内部 `InternalEntity { index, generation }` 和 exhausted-slot retirement 没有进入公共 handle/wire | Bevy `Entity` 与 Fyrox `Handle<T>` 至少携带 generation；Unreal Mass handle 以 index/serial 校验 | 分离 `SceneEntityGuid`、`WorldHandle{id,epoch}`、`LiveEntityKey{index,generation}`；异步/ABI/editor address 全部 fail-closed 校验 owner epoch |
| `R05-P1-02` | **Open** | `Clone for World` 仍只 snapshot 固定 entity/render/physics/lighting/2D/animation 组；serde/project document 仍列举固定 map。任意注册 typed component 没有 structured loss report | Bevy DynamicWorld 按已注册 ReflectComponent/Resource 提取；Godot PackedScene 保存 owner/node/property/instance state | 删除 World 完整 clone/serde 假合同；component 注册声明 persistence/fork/checkpoint/play policy 与 codec，required data 丢失必须拒绝并报告 entity/type |
| `R05-P1-03` | **Open** | DynamicScene 只有 root schema v2 和 envelope migration；type path、field metadata、局部 schema generation 仍不是持久 `TypeSchemaId/version/fingerprint` | Bevy type registry 驱动反射提取；Godot/Unreal 把实际格式版本与 load/migration 绑定 | 建立 `StableTypeSchema + SceneSchemaCatalog + MigrationCatalog`；每 row 保存写入版本，缺 plugin/future/gap 支持 opaque preservation 或原子失败 |
| `R05-P1-04` | **Open** | scheduler 只有 `supports_worldless_execution` 系统进入 worker；Query、Res、Events/Messages 仍经单一 `&mut World` 串行运行。conflict graph 还没有形成受审计 borrow 权限 | Bevy executor按 `FilteredAccess` compatibility 并行非冲突系统；Unreal Mass 按 archetype chunk/context 并行 | 实现 `WorldCell/ColumnLease/ResourceLease`、compiled access token、chunk task、exclusive barrier 和 deterministic command merge；worldless 退化为普通零访问系统 |
| `R05-P1-05` | **Partial** | active/transform/node-cache 已用 `DerivedStateFrontier` 和 `mark_*_at(entity)`；但 hierarchy 仍是 bool，frontier root 计算仍分配/沿祖先扫描，repair/topology mismatch 可全量 rebuild | Bevy hierarchy relation hooks维护双向关系且 transform propagation 可跳过无 dirty subtree；Godot Node3D 有明确 transform dirty flags | 由唯一 hierarchy topology generation 发布 edge/subtree frontier；Active/Transform/Inspection/Render 共用 sealed delta，稳定帧 visit/build/clone 必须为0 |
| `R05-P1-06` | **Partial** | render dirty journal 与 `RenderComponentChangeArtifact` 已发布 generation/upsert/remove；当前 projector 只覆盖 MeshRenderer、WorldMatrix、Active、Layer、Mobility，initial/drift/history loss仍 full scan，journal generation exhaustion会 panic | Unity Graphics 使用 SoA instance arrays、job queue和增量 transform/bounds/culling；成熟引擎把 view-independent scene data 与 per-view selection 分开 | 建立完整 `SceneExtractGeneration`、stable render slot 与 added/changed/removed ranges；覆盖 sprite/light/camera/particle/visibility/post-process，多 view只引用共享 payload；删除 World clone fallback |
| `R05-P1-07` | **Open** | Play snapshot 仍同步 `DynamicScene::from_world -> to_versioned_json_pretty`；capture 对每个 entity 遍历完整 type registry。无 background job、page、cancel、deadline、sealed source generation | Bevy DynamicWorldBuilder按注册反射类型和filter构建，但 Zircon还需要面向大世界的 column/page capture | 改为 archetype-column capture 和 compact intermediate；封存 source generation，后台分页、progress/cancel/deadline，pretty JSON只做 authoring artifact，不做内部 Play transport |
| `R05-P1-08` | **Partial** | `query_world_bounded_at_replacement_epoch` 已限制 items/encoded bytes/depth/time；但 public `WorldQuery/Result` 仍返回完整 Vec，没有 page size、opaque cursor、compiled plan、changed-since或cancel；`World::query_world` 仍保留无界入口 | Bevy QueryState缓存archetype membership/access；大规模 inspection 还需要独立于 ECS API 的分页协议 | 建立 schema-generation-bound `WorldQueryPlan`、page/cursor/world epoch、byte/deadline/cancel、permission filter和delta subscription；删除产品无界入口 |
| `R05-P1-09` | **Open** | LevelSystem有 Loaded/Unloaded 等 lifecycle 和 replacement epoch，但 Scene/Interface 中没有 partition catalog、spatial cell、data layer、cross-cell reference、residency/activation budget authority | Unreal明确区分 persistent level、level collection、streaming level和World Partition cell | 建立 `WorldPartitionDescriptor + PartitionCatalogGeneration + LevelInstanceRegistry`；load/stage/activate/visible/evict是有预算、可取消、可回滚状态机 |
| `R05-P1-10` | **Open** | `Events<T>` 仍是 `current/next Vec<T>`；`send`/`send_batch`无 entry/charged-byte/per-producer budget，capacity shrink只处理事后保留内存。removed events和Messages的有界实现不能替普通 Events代偿 | 成熟 event bus 需要按 delivery class 定义 backpressure/coalesce/drop/gap | 为 critical/state/telemetry event声明策略和硬预算；写入返回 receipt/error，slow reader和drop可观测，frame flush保持 deterministic |
| `R05-P1-11` | **Open** | `dynamic_scene/session` 当前 **578 files / 13,093 lines / 464,007 bytes / 336 pub fn**；排除自身、re-export和tests后，Archive/Slot/Metadata仍没有 App/Editor 产品 consumer | 这不是参考引擎功能缺失，而是 Zircon 的无消费者组合 surface 和维护成本异常 | 收敛为 typed request：selector/source/operation/mode/options为数据；只保留一套 prepare-preview-commit 与 path/loaded adapter，产品 checkpoint workflow接通后删除组合 facade/source-shape guard |
| `R05-P1-12` | **Partial** | `WorldDomain { Edit, Play(instance) }`、PlayDomainLink、App-owned session lease、running-document lock、bounded pending-edit apply/discard 已形成明确边界；但 Play source仍是 JSON，World clone/serde/DynamicScene/Level snapshot各有不同 copy/reset语义，没有全域 transfer table、rollback token或partial report | Fyrox Editor scene container与runtime scene是明确对象边界；Godot PackedScene实例化区分owner/instance state | 定义不可混用的 `AuthoringSceneDocument`、`RuntimeWorldFork`、`FrameExtract`、`Checkpoint`、`PartitionStaging`；每种按注册policy输出 `TransferReport` 并以 world/schema epoch CAS提交 |

P1 合计：**8 Open、4 Partial、0 Closed**。

### 3.2 P2

| ID | 状态 | 当前源码证据 | 必须重构 |
|---|---|---|---|
| `R05-P2-01` | **Open** | ObserverStore 用 Arc bucket避免 registry borrow冲突，但 callback仍同步获得 `&mut World`；没有 dispatch depth、callback/command/time budget、panic transaction、reentrant policy；`next_id += 1` 仍可 overflow | 默认 observer只读 event context并写 deferred lane；exclusive observer在barrier执行；foreign/plugin panic隔离candidate commands，所有超限/递归/失败产生 receipt |
| `R05-P2-02` | **Open** | 去重选择集有2,517个测试属性，却有290处 `include_str!/include_bytes!` source-shape读取；没有本轮动态执行、100K hierarchy、multi-view、1M event、Miri/loom/fuzz/soak验收 | 保留少量owner guard；增加property/generative、controlled concurrency、visit/alloc/clone counters和managed Windows product benchmark，terminal receipt前不得写成通过 |

P2 合计：**2 Open、0 Partial、0 Closed**。

## 4. 专业 owner 的当前源码移动，不重复计数

| Canonical owner | 旧 current-source 状态 | 本轮观察到的源码移动 | 处理要求 |
|---|---|---|---|
| Runtime60 / Runtime108 `RECS-P0-01` | 旧报告称共享 `Query::iter/count/is_empty` 用 `&mut QueryState` alias | 当前 `Query` 对所有 cache-refresh/lend-cache入口使用 `&mut self`；只读 `QueryState::iter(&self, &World)`不刷新共享cache | 这是**源码修复候选**；Runtime60需重跑 alias/compile-fail/Miri或等价测试并独立重判，Runtime162不关闭P0 |
| Runtime60 / Runtime108 `RECS-P0-02` | 旧报告称 removed history永久 Vec保留 | 当前 `RemovedComponentChannel` 使用 `VecDeque`、sequence/generation、默认1024 entries/256 KiB/600 frames、drop metrics和slow-reader gap | 这是**源码修复候选**；需验证payload charge、reader churn、generation reset和产品schedule后由Runtime60重判 |
| Runtime63 / Runtime111 `RSR-P0-001` | 旧报告称先改 component registry、后续reflect失败留下半注册 | 当前 `register_component_type` 先验证descriptor、duplicate type、runtime registration和descriptor import，再一次publish | 这是**源码修复候选**；需以失败后两registry/generation/instance eligibility不变及立即重试测试由Runtime63重判 |
| Runtime62 / Runtime110 | 两项P0旧报告均Open | public `insert/get_mut/remove<T>`仍可对Hierarchy/WorldMatrix/Active等保护类型操作；raw get_mut只在借出前mark dirty，不能验证最终值 | 专项风险仍有直接源码证据；由Runtime62拥有mutation capability hard cut和关闭验证 |
| Runtime61 / Runtime109 | 五项P0旧报告均Open | 固定typed projection的World clone/serde和Play/DynamicScene数据守恒缺口仍可复现于源码；本轮未对五项逐一做独立关闭审计 | Runtime61继续唯一计数；Runtime162只拥有父级identity/transfer/persistence架构，不复制P0 |

## 5. 参考引擎逐项对照

| 参考源码 | 可吸收的工程合同 | Zircon 当前差距 | 采用边界 |
|---|---|---|---|
| Bevy ECS entity/query/access/executor | entity index+generation、table row失效规则、QueryState archetype generation、FilteredAccess冲突、非冲突系统并行 | public entity无generation/world；真实World系统串行；cache/plan虽有但没变成并行borrow证明 | 吸收identity/access/plan证明方法，不照抄unsafe实现或API命名 |
| Bevy hierarchy/transform/world serialization | relationship hook维护ChildOf/Children，transform可跳过无dirty subtree；DynamicWorld按ReflectComponent/Resource和filter提取 | hierarchy保护写入口仍开放；capture N*T；持久化仍built-in whitelist | 吸收注册驱动提取、关系authority与dirty skip；另加Zircon持久GUID/partition需求 |
| Unreal UWorld/ULevel/LevelStreaming/World Partition | persistent level、level collection、streaming visibility、partition cell、world subsystem/lifecycle边界 | Level lifecycle只是单World容器状态，没有catalog/residency/staging/activation transaction | 采用ownership/state/budget/failure模型，不复制Actor/Package历史债务 |
| Unreal Mass | index+serial handle、archetype-owned rows/chunks、cached query requirements、parallel execution flag/context | internal generation不出World；scheduler不能给Query/Res worker lease | 采用chunk/access/command barrier思想，不把Mass object model等同authoring scene |
| Fyrox Pool/Graph/Scene/Editor container | typed generational handle、graph handle remap、scene作为graph/animation/physics容器、Editor scene entry独立管理 | public u64与World clone混合live/persistent；fork/remap/Editor ownership不是一个显式artifact | 吸收typed handle、remap和Editor/runtime scene边界，不复制具体Graph节点类型 |
| Godot Node/SceneTree/PackedScene/Node3D | owner/parent/children、process group、packed node/property/instance state、local/global transform dirty | Zircon的关系可被raw mutation破坏，process affinity和scene instance/override持久语义不足 | 吸收owner/instance/property usage及thread group边界，不复制Node callback的全部动态行为 |
| Unity Graphics GPUDriven | SoA NativeArray、renderer-to-instance map、IJobParallelFor、transform/bounds/update queues、per-camera data | Zircon mesh delta刚起步，完整extract仍广泛materialize/sort/clone，view-independent数据不能充分复用 | 作为Scene->Render增量consumer资格线；RHI/GPU实现仍由Render owner负责 |

参考源码证明的是成熟系统必须具备可验证owner、identity、generation、state和failure contract；不能据此直接宣称任一参考引擎更快。性能对比必须冻结相同硬件、场景、线程预算、正确性门、视图数、缓存冷热和统计方法。

## 6. 目标架构与所有权

### 6.1 固定包边界

| 包 | 必须拥有 | 明确禁止 |
|---|---|---|
| `zircon_runtime` | World/Scene truth、live identity、ECS storage/schedule、scene schema/fork/checkpoint、partition、inspection/extract generation | 不把运行时truth上移给Editor；不新增第四顶层package或非网络server |
| `zircon_editor` | authoring document、operation/undo、Play policy、diagnostic projection、partition workflow UI | 不持有第二套World/ECS/partition/extract authority，不通过clone/JSON猜transfer policy |
| `zircon_app` | host composition、runtime session factory、window/process lifecycle、terminal shutdown结果 | 不解析scene业务schema，不成为Level或ECS owner |

### 6.2 Runtime 内部权威脊柱

```text
SceneIdentityCatalog
  -> SceneEntityGuid
  -> WorldIdentity { id, epoch }
  -> LiveEntityKey { index, generation }

SceneSchemaCatalogGeneration
  -> ComponentPolicy { persistence, fork, checkpoint, play, extract }
  -> MigrationCatalog
  -> AuthoringSceneDocument / RuntimeWorldFork / CheckpointArtifact

WorldMutationTransaction
  -> Archetype / Sparse / Resource commit
  -> HierarchyTopologyDelta
  -> ComponentChangeDelta
  -> InspectionGeneration
  -> SceneExtractGeneration

ScheduleCompiler
  -> AccessTokenPlan
  -> Column/Resource/Chunk leases
  -> Parallel executor
  -> Deterministic structural barrier

PartitionCatalogGeneration
  -> Cell request/load/stage/activate/visible/evict
  -> Cross-cell persistent reference resolver
  -> Budget / cancellation / rollback receipt
```

`LevelSystem`应成为一个 generation-qualified runtime level instance facade，而不是把World lock、snapshot clone、replacement和scene persistence全部混成隐式语义。`World`不再实现“完整 clone/serde”；每个跨域操作必须选择一个明确artifact和policy。

## 7. 依赖顺序重构计划

| Milestone | 工作 | 完成证据 |
|---|---|---|
| M0 · Characterization | 冻结Runtime05/60/61/62/63 owner、identity grammar、component policy matrix、规模/性能计数器 | custom typed component、stale handle、Play transfer、hierarchy mutation、query alias和event burst RED corpus |
| M1 · Identity | 引入World identity/epoch、generational live key、persistent GUID和instantiate remap；迁移ABI/editor/object address | cross-world同index、replacement、despawn/respawn、async stale、serialized live key全部fail-closed |
| M2 · Schema/data conservation | 注册逐类型schema/version/fingerprint、persistence/fork/checkpoint/play policy和migration | builtin/custom/plugin/missing/future全量round-trip；required data静默丢失=0 |
| M3 · Explicit artifacts | 删除World完整clone/serde生产入口；实现AuthoringDocument、RuntimeFork、Checkpoint和TransferReport | enter/exit Play、save/reopen、checkpoint/restore各有golden corpus、rollback和domain report |
| M4 · Parallel ECS | access-token plan、column/resource/chunk lease、exclusive barrier、worker command lane | disjoint Query/Res真实并行；冲突串行；panic/cancel不发布partial commands；同seed顺序一致 |
| M5 · Mutation/delta | hierarchy edge/frontier和component change由一次mutation commit发布；保护derived component写入口 | stable frame visits=0；single change近affected set；raw mutation无法破坏关系/derived truth |
| M6 · Scene extraction | 完整SceneExtractGeneration、stable slots、added/changed/removed ranges和multi-view共享 | view-independent build每frame最多1次；stable upload delta=0；removed slot精确退休 |
| M7 · Capture/query/events | column/page background capture；compiled paged query；普通Events delivery class/hard budget；observer deferred policy | N*T探测归零；cursor/deadline/cancel有效；1M event受限；observer递归/panic不破坏commit |
| M8 · Partition/Level | partition catalog、cell artifact、dependency/data layer、staging/activation/eviction和cross-cell resolver | load/cancel/failure/evict/reopen一致；activation按预算；Editor UI消费真实Runtime状态 |
| M9 · Product/scale hard cut | session facade收敛，App/Editor/Runtime只走新identity/artifact/query/partition路径 | 旧clone/serde/unbounded query/组合wrapper删除；10K/100K/1M、多view、soak和对照benchmark达标 |

M1-M4是Scene正确性底座，必须先于开放世界、GPU优化或继续扩展Editor表面。M5-M8可在identity/schema/transaction稳定后分层推进，但不得以另建兼容facade维持双真相。

## 8. 工程资格门

| Gate | 状态 | 当前缺口 | 通过标准 |
|---|---|---|---|
| Unit / property | **Partial** | 单测很多且三项专项P0出现修复候选，但290处source-shape读取不能证明数据/alias/transaction正确性 | allocator/archetype/schema/remap/transfer/delta/event均有property/generative与negative corpus；关键owner无source-string替代行为测试 |
| Concurrency | **Fail** | 真实World Query/Res系统不能worker并行；observer/replace/capture缺统一并发模型 | access token、chunk lease、barrier、panic/cancel/replace race受控测试通过，data race/partial publish=0 |
| Scale | **Fail** | 没有接受的100K/1M entity、1000 type、100K depth、64 view、1M event证据 | 冻结数据集/硬件，记录p50/p95/p99、visits、alloc/clone bytes、lock wait、working set与回归阈值 |
| Serialization | **Fail** | arbitrary typed component仍可被clone/serde/project document静默丢失；逐类型migration缺失 | builtin/custom/plugin/missing/future/cross-cell reference corpus全通过，未声明数据丢失=0 |
| Product | **Partial** | App-owned Play session、Edit/Play domain和bounded pending edits是真实接线；但JSON capture、session archive无consumer、partition无产品链 | save/reopen、Play fork/exit、checkpoint、paged inspect、cell load/unload只走新authority，UI不假成功 |
| Performance | **Partial** | frontier、journal、component delta和局部counter存在；完整extract/capture/query/scheduler仍缺accepted动态证据 | 稳定帧work=0或明确定额，single mutation近affected set，多view共享；与UE/参考baseline在同正确性门下测量 |
| Fault injection | **Fail** | schema gap、callback panic、snapshot cancel、partition activation、World replacement和OOM/admission缺端到端矩阵 | 任意prepare阶段失败不发布partial World/document/extract；last-good authoring/runtime/partition保持且receipt可诊断 |

Gate 合计：**4 Fail、3 Partial、0 Pass**。

## 9. 实施约束与验收交接

1. Runtime05继续拥有本报告14项finding；Runtime60/61/62/63拥有其专项P0/P1/P2。源码移动必须回到canonical owner重判，不用Runtime162复制关闭计数。
2. 先写identity、schema、transfer、mutation、schedule和partition failure model，再改代码；type alias、空driver、第二份cache或更多facade都不构成完成。
3. 迁移采用hard cutover。新live handle、artifact、query和partition产品接通后，裸u64跨域、World完整clone/serde、无界query、假Level Streaming和组合session wrapper必须删除。
4. 每个milestone都要按底层unit/property、transaction/reopen、concurrency/fault、scale/product自底向上验收；上层demo不能替代底层失败测试。
5. “优于Unreal”只能作为同场景、同正确性、同线程/内存预算下的测量结果。没有原始benchmark artifact、统计方法和回归阈值时不得写入完成状态。
6. 本报告只完成review；没有修改Runtime/Editor/App生产代码，也没有把未执行的Cargo、产品或性能验证写成通过。

下一实现会话应从M0/M1开始，把Runtime05、Runtime24、Runtime60和Runtime61的identity/data-conservation contract合并成一份可编译设计记录；在live identity和custom typed component守恒通过前，不应先扩展更多Level Streaming UI或GPU scene临时入口。
