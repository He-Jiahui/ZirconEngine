---
title: Runtime Navigation 当前工作树 Bake、Artifact、Query、Crowd、Off-Mesh 与 Runtime Boundary 复审及重构计划
category: zircon_runtime
report_id: Runtime169
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zp-runtime-navigation-navmesh-recast-detour-tilecache-crowd-query-pathfinding-obstacle-off-mesh-link-bake-streaming-world-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
related_code:
  - zircon_runtime/src/core/framework/navigation
  - zircon_runtime/src/navigation
  - zircon_runtime/src/scene/navigation.rs
  - zircon_plugins/navigation/runtime
  - zircon_plugins/navigation/native
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs
  - zircon_app/src/entry
plan_sources:
  - docs/plans/optimize/zircon_runtime/99zp-runtime-navigation-navmesh-recast-detour-tilecache-crowd-query-pathfinding-obstacle-off-mesh-link-bake-streaming-world-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/05/failure-2026-07-19-navigation-runtime-fallback-hotpath.md
  - docs/plans/zircon_plugins/05/failure-2026-07-27-navigation-world-scan-deserialize-value.md
  - docs/plans/zircon_plugins/05/failure-2026-07-30-navigation-overlay-frame-publication.md
  - docs/plans/zircon_plugins/05/failure-2026-08-02-navigation-editor-operation-status-v2-cutover.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavMesh/RecastNavMeshGenerator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/AI/Navigation/NavigationSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/AI/Navigation/DetourCrowdAIController.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/WorldPartition/WorldPartitionNavigationDataBuilder.cpp
  - dev/godot/modules/navigation_3d/3d/navigation_mesh_generator.cpp
  - dev/godot/scene/resources/navigation_mesh.cpp
  - dev/godot/servers/navigation_3d/navigation_server_3d.cpp
  - dev/Fyrox/fyrox-impl/src/scene/navmesh.rs
  - dev/Fyrox/editor/src/interaction/navmesh/mod.rs
  - dev/Fyrox/editor/src/scene/commands/navmesh.rs
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime169 · Navigation 当前工作树复审

## 1. 结论

Navigation 已从旧版的多个明显占位推进到一个可运行的局部底座：runtime framework 有版本化 `NavMeshAsset`、query/filter、agent、surface、off-mesh 和 operation DTO；builtin manager 有持久化 polygon graph、空间索引、query scratch、repath cache 与 typed world projection；first-party plugin 有 Recast/Detour/DetourCrowd C ABI、异步 tiled bake、代际发布、dirty-bake 状态、off-mesh capacity/traversal、typed tick report、overlay frame 和 PIE mirror；Crowd native scratch 也已按配置容量分配，而不是固定 64 个输出槽。这些部分值得保留。

但当前仍不是工程级导航系统。核心问题从“有没有路径函数”转成了 authority、几何真实性、artifact 生命周期和算法切换的不可证明性。builtin manager 与 first-party plugin 仍是两套生产行为；builtin 明确不 bake、plugin 的 bake 在缺源时会造出一个简单 quad；真实 render/collider 几何仍被压缩成单位顶面/圆盘；Recast 参数大量只进 hash/warning；tiled bake 共享了计划内存却仍把完整 source buffer 传给每个 tile 并合并回 raw DTO；生成结果没有 cook、CAS、依赖、residency 或持久 Detour tile blob。

运行时查询默认选择 `loaded` 中最小 handle，不能表达 world/scene、agent profile、artifact generation 和 provider ownership。只要场景出现 runtime obstacle、obstacle world 或 off-mesh link，plugin Crowd 就被清空并退回 legacy 路径；同一输入因此会改变算法、复杂度和运动语义。Crowd 仍固定 256 agents / radius 8，legacy 分支直接写 Transform 并用简化避让；off-mesh 由导航内部直接插值，没有 Character Movement、Physics、Animation、Gameplay、Network 或 cancel/timeout handoff。不能据此宣称与 Unreal、Godot 或 Fyrox 等价，更不能宣称性能优于 Unreal。

## 2. 复审范围与物理统计

| 范围 | files | lines | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Runtime framework + builtin navigation (`zircon_runtime/src/core/framework/navigation`, `zircon_runtime/src/navigation`) | 37 | 5,551 | 185,950 | 46 | 4 | `40e8378614d37ad0c221f05d538e50ebc91acfb52a79f711ca2d9cf4a30d3a92` |
| First-party navigation runtime + native (`zircon_plugins/navigation/runtime`, `zircon_plugins/navigation/native`) | 83 | 12,961 | 434,802 | 116 | 6 | `73882c92cdd0d57204c95800c3cd89923c85a45f1c11a351083a0bbccd0961f3` |
| Navigation editor + UI session path | 23 | 3,714 | 128,698 | 34 | 1 | `9595eacaf8de3b0acbe27a81f503878bc222be471ec5efc5245fec21e9af8e7e` |
| Catalog/App integration | 218 | 30,815 | 1,140,226 | 484 | 1 | `d27c4cd49d65088edafa0214653e878aebf1bcda404a87e4834a6a2c66ca274f` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference selection | 11 | 15,780 | 639,843 | 0 | 0 | n/a |

统计按当前工作树的 Rust 文件与显式测试属性计算。Navigation runtime/editor/native/catalog 存在用户或会话修改及未跟踪文件，以上 fingerprint 只用于本轮文档证据，不是未来实现的稳定基线。

## 3. 当前真实链路

1. `NavigationRuntimePlugin` 注册组件、事件、`NavRepathBudget`、`NavigationDebugCapture` 与 `navigation.agent_tick`；tick 在 AI behavior 之后运行，发布 `NavAgentTickReport` 和可选 overlay frame。
2. plugin manager 以 `Arc<Mutex<NavigationRuntimeState>>` 持有 settings、loaded assets、generated snapshots、crowds、obstacle worlds、bake contexts/tasks、diagnostics 和 traversal state。`replace_generated_snapshot` 会替换 raw `NavMeshAsset` 并生成新的 handle，但没有资产管理器 ticket 或持久 artifact identity。
3. `prepare_bake` 收集 surface、agent/settings、world nodes 与 geometry；`bake_nav_mesh_asset` 选择 triangle mesh、tiled mesh 或 simple-surface fallback；finish 阶段 stamp settings hash、嵌入 off-mesh links、发布 generated snapshot。
4. `start_tiled_bake` 已有 shared `RecastTiledBakePlan`、worker 结果槽和 generation supersession；harvest 时再 merge tile assets。该任务只有 Pending/Ready，缺少 cancel、progress、deadline、owner shutdown 和持久失败 receipt。
5. 查询从 `selected_handle_asset` 选择 loaded asset 后交给 Recast backend；builtin manager 另有 `BakedNavMesh` graph/query 实现。两者的 filter、off-mesh、failure、streaming 和 movement 语义并未统一。

## 4. P0：产品真实性阻断

### P0-01：双 authority 没有硬切 owner

`BuiltinNavigationManager::bake_surface` 仍直接返回“不支持 bake surfaces”，而 first-party `DefaultNavigationManager` 提供另一个 bake/query/crowd/obstacle 实现。App/catalog 的 feature 选择决定哪个路径可见，不能证明一个项目在 client、server、editor host 和 PIE 使用同一个 world owner、artifact、query filter 或 movement contract。实现阶段必须指定唯一 `NavigationProvider`，builtin 只能作为明确的 fallback/read-only loader，不能继续与 plugin 并列消费同一 DTO。

### P0-02：Bake 在缺几何时仍生成伪造可行走面

`zircon_plugins/navigation/runtime/src/manager/bake/asset.rs:12-52` 在无源几何时调用 `bake_simple_surface`，诊断虽记录 warning，但仍可发布一块由 `half_extent` 决定的 quad。没有 source geometry 时，生产系统应该返回可解释的 `NoSourceGeometry` 或空 artifact，而不是生成可以让 AI 行走的虚假世界。该 fallback 会把错误延迟到 gameplay，并污染 bake report、debug overlay 与性能测量。

### P0-03：输入几何不是场景几何

`manager/bake/geometry.rs:40-75,185-248` 的 render mesh/Cube 路径统一 `push_quad_from_matrix(..., Vec3::splat(0.5))`；Box 只写顶面；Sphere/Capsule/Cylinder 只写 12 段圆盘；ConvexHull 取 AABB 顶面；TriangleMesh/HeightField 明确不收集。这样无法表达斜坡、台阶、墙、洞、悬空体、真实碰撞网格或 heightfield，Recast 的结果即使成功也不是项目场景的导航结果。

### P0-04：Bake settings 没有进入 native compiler

`zircon_plugins/navigation/native/src/bake.rs:70-115` 每次使用 `RecastBakeSettings::default()`；surface 的 voxel、region、height mesh 等选项在 `manager/bake/diagnostics.rs:68-91` 只产生 warning，并且仅进入 settings hash。agent radius/height/climb/slope、tile border、area flag 与 build-height-mesh 没有形成编译输入。相同 geometry 改变 settings 可能得到相同 native mesh，settings hash 只是掩盖了语义缺失。

### P0-05：Editor Bake operation 仍然不能执行

`zircon_runtime/src/navigation/operation/handler.rs:115-165` 对 Bake Scene/Surface 的 `prepare` 返回“requires a pure prepare backend”，`apply` 继续拒绝“cannot reach owner apply”。因此 UI 已能提交 operation descriptor，但真正纵向链路仍在 operation owner 前失败，不能进入 undo、reopen、PIE 或 artifact commit。

## 5. P1：Artifact、Bake 与 Query

### P1-01：tiled bake 不是可流式的 tile artifact

`native/src/bake.rs:180-310` 的 tile plan 共享 `Arc` buffer 是内存优化，但每个 `bake_planned_tile` 仍将完整 source vertex/index buffer 传入 native；`tile_intersects_mesh` 是三角形 AABB 粗筛；merge 以量化顶点 key 将 tile raw vertices/polygons 合并。结果没有 Detour tile data、salt/ref、layer compression、cross-tile link、attach/detach、rebuild priority 或 stream residency。Unreal 的 Recast generator 与 World Partition builder 把 dirty tile、优先级、压缩和运行时装载作为产品边界；当前实现只完成了并行计算的一个内存细节。

### P1-02：异步任务不可取消且不受 owner 生命周期约束

`manager/bake/task_pool.rs:64-188` 将 World、prepared geometry 和每 tile result 放入 `Arc<Mutex>`，handle 状态只有 Pending/Ready。`advance_bake_context` 可以移除旧任务引用，但 worker 仍继续执行；没有 cancellation token、worker admission、progress event、shutdown join、memory budget 或 partial-result discard receipt。长 bake、关闭 world、重复点击 bake 和 editor disconnect 会造成不可见工作和 stale side effects。

### P1-03：generated snapshot 不是资产提交

`NavigationGeneratedBakeSnapshot` 只包含 `surface_entity`、完整 `NavMeshAsset` 和 `output_asset: Option<String>`。`replace_generated_snapshot` 直接把 asset clone 进 manager 的 loaded map；没有 source dependency fingerprint、compiler version、settings schema version、CAS key、atomic file publish、asset registry import、load/unload ticket 或 stale artifact rejection。Undo 也只能复制完整 raw asset，不能恢复一个可验证的 cooked artifact。

### P1-04：查询选择与生命周期过弱

`DefaultNavigationManager::selected_handle_asset` 在无 handle 时取 `state.loaded.keys().next()`；query 只传 `NavMeshHandle`、agent type 和 area mask，没有 world/scene identity、artifact generation、provider epoch 或 query deadline。loaded map 同时承载 generated 与外部 loaded assets，替换 settings 会清空 crowds、obstacle worlds、bake tasks 与 generated state，读者无法获得稳定的 snapshot lease。

### P1-05：fallback query 与 native query 语义未闭合

builtin `BakedNavMesh` 维护自己的 polygon adjacency/spatial index/query scratch，而 plugin backend 维护 Recast/Detour。`find_path_with_filter`、sample、raycast、off-mesh link、partial/no-path 和 visited-node 统计没有一份 conformance suite 证明等价。任何 runtime obstacle/off-mesh link 触发 legacy 分支时，query、avoidance、completion 结果会改变，构成静默算法降级。

## 6. P1：Agent、Crowd、Obstacle 与 Off-Mesh

### P1-06：Crowd 容量仍是硬编码并且按场景切换算法

`zircon_plugins/navigation/runtime/src/agent.rs:20-24,229-245` 固定 `NAV_CROWD_MAX_AGENTS=256`、`NAV_CROWD_MAX_AGENT_RADIUS=8.0`。`tick_world_agents` 在存在 obstacle/off-mesh 时清空 Crowd 并调用 `tick_world_agents_legacy`；legacy 每帧重查路径并做简化 obstacle/agent 避让。真实项目的 agent profile、world partition、crowd partition、overflow policy 与 capacity telemetry 都缺失，256 不是可证明的上限而是隐藏故障阈值。

### P1-07：运动 authority 仍由 Navigation 直接写世界

Crowd writeback (`agent/writeback.rs`) 与 legacy `tick.rs` 都可以直接 `world.update_transform`；DesiredVelocity 只是动态组件。没有 CharacterController/Physics sweep、root motion、animation handoff、network prediction、authority conflict 或 movement receipt。off-mesh `traversal/advance.rs` 用固定线性/抛物线插值直接提供 position，capacity 只在导航内部管理，Gameplay 无法取消、暂停、拒绝或完成 traversal。

### P1-08：动态障碍和局部避让不是真正的增量 nav update

runtime obstacle 以 world projection + 空间 cell 收集，plugin 复杂障碍直接触发 legacy；builtin avoidance 只检查最多 64 个候选，plugin Crowd 也只从 native state 回读。没有 DetourTileCache 的 obstacle add/remove/update、dirty tile scheduler、局部重烘焙、世界 generation fence 或对 query/crowd 的原子切换。障碍数量一变，系统复杂度和结果会跳变。

### P1-09：AI integration 仍缺 request lifecycle

agent tick 会报告 `arrived_agents`、`no_path_agents`，这是实质改善；但 `NavMeshAgentDescriptor.destination` 仍是动态 JSON 组件，事件消费者根据 tick report 推断状态，没有 typed request handle、path-following state、deadline、cancel、stale world generation、partial path policy 或 server/client parity。AI 行为与 nav job 之间不存在可追踪 receipt。

## 7. P1：可观测性、插件与产品装配

### P1-10：Overlay frame 有生产发布，但过滤器未成为 runtime contract

`NavigationDebugCapture` 通过 mirrored-event reader count 开关 capture，tick 才发布 frame；frame 是整个 loaded asset 的 debug triangles/off-mesh links 加上 agent path/velocity。没有 bounded payload、demanded tile/agent subset、backpressure、sequence gap recovery 或 render-time lifetime lease。editor provider 还使用固定 default options，因此 UI 的四个过滤器不能可靠改变 runtime 数据量。

### P1-11：Catalog/dist/capability 不能证明可用 provider

runtime manifest 有 native distribution、Recast capability、driver 和 event catalog，这是装配底座；但 editor/runtime catalog 的 closure、默认 feature、asset loader/cook registration、server/headless provider 与 generated artifact owner 仍需要逐目标验证。`PluginMaturity::Beta`/`CapabilityStatus::Partial` 与测试 manifest 不能被解释成 production qualification。

### P1-12：性能证据只覆盖局部容器优化

当前 release-only 测试主要比较单次 world projection、Arc plan clone、scratch capacity、overlay copy 和 stats scan。没有 bake wall time、每 tile source bandwidth、native query latency/p99、crowd overflow、dirty update latency、cross-world contention、memory residency、server tick budget、replay determinism 或 100K agents 的实测，因此不能支持“优于 Unreal”的性能结论。

## 8. 参考引擎差异

* Unreal `RecastNavMeshGenerator.cpp` 将 agent 参数、tile border、dirty priority、压缩、debug tile 和异步 generation 放入明确的 generator/runtime 边界；World Partition builder 进一步要求分区 artifact 与可重建的 tile ownership。Zircon 的 shared plan 是必要优化，但还不是 persistent tile pipeline。
* Godot 的 `NavigationMeshGenerator` 将 SceneTree source parsing 与后台 `bake_from_source_geometry_data` 分开，要求明确的 source geometry resource；Zircon 当前仍由 bake stage 直接遍历节点并在缺 source 时造 quad。
* Fyrox 的 editor navmesh mode 以真实 Navmesh 顶点/边/三角形编辑 command 支持 undo/selection；Zircon editor 的 bake operation 仍不能 prepare/apply，asset view 也没有真实 mesh editing。
* Bevy 的 schedule 文档明确区分 PreUpdate、FixedUpdate、Update、PostUpdate 和 render 交换边界；Zircon navigation tick 虽有 system anchor，但 bake/query/crowd/overlay 没有统一 fixed-step、observation 和 render ownership。
* Unity Graphics `DebugManager` 体现集中注册、可重置 debug data/panel/widget 的产品边界；Zircon overlay 只有 provider extract 与 event mirror，缺少可变过滤器、重置、面板状态和 frame budget。

## 9. 目标架构与重构顺序

### R0：Authority 与失败合同

指定唯一 `NavigationProvider`/world owner；builtin 只能作为显式 read-only loader 或被硬切移除。定义 `NavigationWorldId`、`NavigationArtifactId`、`NavigationGeneration`、`NavQueryReceipt`、`NavBakeFailure`、capacity/overflow policy 和 no-source 行为。所有 query、tick、overlay、AI consumer 必须通过 provider lease 读取同一 immutable snapshot。

### R1：真实 source geometry 与 compiler input

建立 typed `NavigationSourceGeometry`：render mesh vertex/index/submesh、physics triangle mesh/heightfield、transform、material/area、modifier volume、obstacle carve 和 source dependency fingerprint。完整实现 box/sphere/capsule/cylinder/convex/triangle mesh/heightfield 的几何转换，禁止单位 quad/顶面/圆盘伪造。把 agent profile、voxel、region、height mesh、tile border、area flags 传入 native compiler，并对每个 ignored knob 失败而非静默 warning。

### R2：Cooked artifact、CAS 与 tiled lifecycle

将 Recast 输出保存为 versioned tile blob/mesh header/link table，包含 compiler/settings/source fingerprints、tile coordinates、salt/ref、compression、dependency graph 和 schema migration。实现 atomic cook publish、CAS dedupe、load/unload/stream attach-detach、dirty tile priority、partial failure、cancel、shutdown join 和 memory budget；raw `NavMeshAsset` 仅作为 authoring DTO，不再作为 runtime residency owner。

### R3：Query scheduler 与 agent/crowd

以 immutable artifact snapshot 构造 per-world Detour query/crowd instances；query 支持 filter、deadline、cancel、generation、partial policy、scratch lease 与 deterministic receipt。将 crowd capacity 变成 settings/partition 资源，定义 overflow/backpressure，不在障碍或 off-mesh 出现时清空 Crowd 退回 O(A²) legacy。运行时障碍使用 dirty tile/cache，运动通过 movement authority/physics adapter，off-mesh 通过 typed gameplay handoff。

### R4：Operation、AI、PIE 与 debug product

Bake prepare 必须只生成 immutable plan，apply 只提交 artifact receipt，clear/restore 只操作 artifact identity；所有状态有 revision conflict、undo/redo、cancel 和 durable failure。AI MoveTo 改为 request handle + path-following state。Overlay 使用 demand-driven bounded extraction、filter state、sequence/generation 和 provider retirement，PIE/server/editor 共用同一事件 schema。

### R5：资格化

至少建立：真实几何 golden scenes；settings sensitivity；tile seam/stream attach-detach；no-source/invalid geometry；cancel/shutdown/stale generation；query conformance（builtin/provider/native）；crowd capacity/overflow；obstacle dirty update；off-mesh gameplay handoff；server/client determinism；100K agents/large world memory；editor bake undo/reopen/PIE；故障注入、长时 soak、p99 latency 和 render overlay backpressure。

## 10. 资格门

| Gate | 当前状态 | 必须证明 |
|---|---|---|
| RT-NAV-1 唯一 provider/world owner | Fail | builtin 与 plugin 不再并列改变语义 |
| RT-NAV-2 真实 source geometry | Fail | render/collider/heightfield/volume 全部进入 compiler |
| RT-NAV-3 settings-sensitive native bake | Fail | agent/voxel/region/height/tile 参数改变结果且可复现 |
| RT-NAV-4 persistent cooked tile artifact | Fail | CAS、依赖、压缩、stream attach/detach、migration |
| RT-NAV-5 cancel/generation/shutdown | Fail | stale bake 无 side effect，关闭可回收 worker |
| RT-NAV-6 query/crowd contract | Partial | filter 有 DTO，但 scheduler、capacity、overflow、conformance 缺失 |
| RT-NAV-7 movement/off-mesh authority | Fail | physics/animation/network/gameplay handoff 与 cancel |
| RT-NAV-8 editor operation/artifact undo | Fail | Bake prepare/apply、receipt、undo/reopen/PIE |
| RT-NAV-9 debug/PIE bounded observation | Partial | mirror/provider 存在，但 demand/filter/backpressure 缺失 |
| RT-NAV-10 scale/performance evidence | Fail | p99、memory、large world、crowd、soak 与 Unreal 对照 |

本轮为 review-only；没有修改 production Rust、测试、Cargo、ABI 或 ZUI，也没有运行 Cargo、native Recast、Editor、PIE、真实 bake、scale、fault、soak 或动态 benchmark。Tooling 按用户要求排除，未查询、轮询、等待或实时跟踪协调器。实施前必须重新冻结 source fingerprint。
