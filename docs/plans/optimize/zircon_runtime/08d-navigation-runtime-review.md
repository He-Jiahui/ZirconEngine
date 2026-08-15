---
related_code:
  - zircon_runtime/src/core/framework/navigation
  - zircon_runtime/src/navigation
  - zircon_runtime/src/navigation/operation/handler.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/runtime/src/manager/state.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/geometry.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/task_pool.rs
  - zircon_plugins/navigation/runtime/src/overlay_frame.rs
  - zircon_plugins/navigation/native/src/bake.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/fallback_query
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/editor/src/bake_panel.rs
  - zircon_plugins/navigation/editor/src/operation_command/command.rs
  - zircon_plugins/navigation/editor/src/viewport_overlay_provider.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/performance/01/2026-07-19-runtime-navigation-static-review.md
  - docs/plans/performance/01/2026-07-30-runtime-framework-animation-ai-navigation-tasks-static-review.md
  - docs/plans/zircon_plugins/05/failure-2026-07-15-navigation-bake-selection-operation-arguments.md
  - docs/plans/zircon_plugins/05/failure-2026-07-19-navigation-runtime-fallback-hotpath.md
  - docs/plans/zircon_plugins/05/failure-2026-07-27-navigation-world-scan-deserialize-value.md
  - docs/plans/zircon_plugins/05/failure-2026-07-30-navigation-overlay-frame-publication.md
  - docs/plans/zircon_plugins/05/failure-2026-08-02-navigation-editor-operation-status-v2-cutover.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationOctree.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMesh.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMeshGenerator.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMeshDataChunk.h
  - dev/godot/modules/navigation_3d/nav_map_3d.h
  - dev/godot/modules/navigation_3d/nav_agent_3d.h
  - dev/godot/modules/navigation_3d/3d/nav_map_iteration_3d.h
  - dev/godot/modules/navigation_3d/3d/nav_map_builder_3d.h
  - dev/godot/modules/navigation_3d/3d/nav_mesh_queries_3d.h
  - dev/godot/modules/navigation_3d/3d/nav_mesh_generator_3d.h
  - dev/Fyrox/fyrox-impl/src/scene/navmesh.rs
  - zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Source/DetourNavMeshQuery.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Source/DetourCrowd.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/Recast/Source/Recast.cpp
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 08D · Navigation Runtime 工程化差距

## 1. 结论

Zircon Navigation 已经越过“只有接口和假返回值”的阶段。当前 first-party 插件确实 vendored 并编译 Recast、Detour、DetourCrowd 与 DetourTileCache；native bridge 可以烘焙、查询、创建 crowd 和 tile-cache obstacle；runtime 有六类导航组件、`navigation.agent_tick` scene system、repath budget、off-mesh traversal 状态机、异步 tiled bake、dirty tile rebuild、generated bake snapshot 与 Editor PIE overlay mirror。最近 current-source 修改还补入了带 session/owner generation 的 `NavigationOverlayFrame` 和真实 viewport provider。这些能力应被保留，不能在重构时退回简单网格 A*、全量同步 bake 或 Editor 自建第二份导航真相。

但当前架构仍不是一套可用于大型工程的统一导航系统，而是至少三套行为叠加：`zircon_runtime/src/navigation` 的自研 builtin manager、插件中的 Recast/Crowd 主路径，以及插件内的 legacy/fallback path/avoidance/off-mesh 路径。动态 session 会按插件是否链接选择 core 或 plugin；插件又会因为“存在 obstacle、保留的 obstacle world 或任一 off-mesh link”在运行中清空 crowd并退回 legacy。相同场景只要增加一种组件，寻路、避障、移动与性能特征就会静默改变。builtin fallback 更严重：它能加载和查询 navmesh，却没有注册 `navigation.agent_tick`、`NavAgentTickReport`、`NavRepathBudget` 和 debug resource；产品没有自动 agent 更新，AI 又用 `NavAgentTickReport` event storage 是否存在判断 navigation 是否可用。因此 fallback 的公开描述和实际产品能力不一致。

最严重的热路径问题在 native owner。`DefaultNavigationManager` 只保存可克隆的原始 `NavMeshAsset`。每次 path/sample/raycast 都先深 clone asset，再把 vertices/polygons/cost/link 展平，创建新的 C++ `dtNavMesh` 和 `dtNavMeshQuery`，完成一个查询后立即销毁。C++ 创建阶段还以 polygon pair/edge 的嵌套循环重建邻接。任何 native build/unsupported/error 又被 `Option::None` 吞掉并静默切到纯 Rust fallback；该 fallback 每 query 重建 O(P²) 邻接，并以线性选最小值的 Dijkstra 继续 O(P²)。这不是“有 fallback 更稳健”，而是 backend failure、合法 no-path 与不支持配置被混成同一分支，既无法诊断，也无法建立稳定性能合同。

烘焙链同样存在产品真实性问题。Render Mesh 输入并不读取 mesh vertices/indices，而是对每个 Cube/Mesh 生成单位顶面；Box collider 只取顶面，Sphere/Cylinder/Capsule 变成 12 边顶圆盘，ConvexHull 变成 AABB 顶面，TriangleMesh/HeightField 不产出任何几何。无输入时还会警告后生成 surface volume quad，使错误场景得到看似可用的 navmesh。agent radius/height/climb/slope、surface voxel/min-region/height-mesh等字段没有进入 Recast config；后者甚至明确只被写入 settings hash。tiled bake 虽然并行分 tile，却把完整 source mesh交给每个 tile worker，之后将结果合并回原始 Rust polygon asset；运行时仍重建一个 single-tile `dtNavMesh`，并没有可 attach/detach、可 streaming、带 salt/ref 生命周期的 Detour tile generation。

Editor 公开了 Bake Scene/Surface 命令，但 runtime handler 的 `prepare` 和 `apply` 对 bake 明确返回错误；单独的 `NavigationBakePanelController` backend 抽象只有测试实现。同步 edit command 只连续 poll 16 次，也无法承载真实异步 tile bake。与此同时，插件选项 `navigation.default_agent_type`、`navigation.default_settings_asset`、`navigation.debug_gizmos` 和 `navigation.bake_backend` 只注册 manifest，没有生产 consumer。当前 overlay 修复解决了“数据源不存在”，但 scene system无论是否有人订阅都会每帧把全部已加载 navmesh转成 triangle DTO并发送；编辑器再 clone一遍三角形和线段，稳定大网格仍是 O(T) 分配与镜像带宽。

本轮登记 20 项 P1、5 项 P2，没有新增 P0。P1 首先收敛唯一 authority、world/generation 生命周期、真实 geometry/config、prepared Detour tile artifact、持久 query/crowd owner、增量 ECS projection、movement ownership、AI/Editor产品闭环；world partition、hierarchical navigation、mass crowd和多运动域进入P2。完成这些重构之前，当前“tiled bake/crowd/editor complete”的历史 milestone只能说明局部 API 或测试落地，不能作为工程级导航完成声明，更不能支持性能优于 Unreal 的结论。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 范围 | 文件 | 行数 | `#[test]` | 证据等级 |
|---|---:|---:|---:|---|
| `core/framework/navigation` | 19 Rust | 1,645 | 9 | E3：asset、agent、surface、query、bake、settings、gizmo 与 manager contract |
| `zircon_runtime/src/navigation` | 15 Rust | 3,351 | 26 | E3：builtin module/manager、compiled fallback mesh、projection、avoidance、operation |
| navigation runtime plugin `src` | 55 Rust | 8,110 | 66 | E3：scene system、Recast/Crowd与legacy agent、bake、obstacle、off-mesh、overlay |
| native Rust bridge | 25 Rust | 3,716 | 33 | E3：bake、Detour、Crowd、TileCache、asset FFI、Rust fallback query |
| native C++ bridge | 10 C/C++/header | 3,394 | 0 | E3：Recast bake、Detour query/crowd/tile-cache C ABI；vendored upstream另计 |
| navigation editor Rust | 21 Rust | 3,308 | 29 | E3：registration、bake panel、operation command、PIE mirror、overlay/provider |
| editor ZUI/dist | 13 physical files | 570 | 2 | E2：agents/areas、bake、debug、settings、surface/component drawers与dynamic entry |

物理统计以 2026-08-15 当前工作区为准。上述 Rust 子树共 163 个 `#[test]`，没有 `#[ignore]`；共发现 5 次 `include_str!`、64 次 `.contains(...)`，未发现 Criterion/`#[bench]`。测试对 asset migration、简单 path/sample/raycast、Recast bridge、crowd、repath budget、obstacle、off-mesh、tile bake和Editor registration有真实行为保护；但没有真实 mesh import-to-bake、10k agent/100k polygon scale、steady-state allocation、world replacement、多world、App reactive cadence、async Editor bake、world partition、跨平台 native soak或产品导出证据。

产品调用点额外覆盖 dynamic session linked/unlinked module选择、默认 navmesh TOML加载、frame demand、AI MoveTo、Runtime operation gateway、Editor retained operation和viewport provider。`zircon_plugins/navigation` 当前有其他 Session 的未提交修改，集中在 overlay frame/provider、operation command、plugin registration和runtime manager/state；`zircon_runtime/src/navigation/operation/handler.rs` 也在修改中。本报告按 current source承认其新generation保护，但实现前必须重取fingerprint、复核 overlapping diff和对应 failure 状态，故标记 `source_recheck_required`。

### 2.2 参考边界

- Unreal `NavigationSystem` 将 world-scoped navigation data、navigation octree、dirty-area controller、invoker、async/time-sliced tile generator、fixed tile pool、active tile set、data chunk attach/detach、world partition和path invalidation置于一个生命周期中。Zircon不需要复制 Actor/UObject层次，但必须吸收“持久 navmesh owner、真实 tile ref、增量 world input、可取消 build、query/bake budget和streaming generation”。
- Godot `NavMap3D`/iteration builder用map iteration snapshot、region/link/agent/obstacle owner、有限 query slot与worker build分离mutable更新和查询读取；generator也明确区分source geometry parse、sync/async bake、task状态与callback。它证明 server-style API仍需要不可变generation和真实任务生命周期，不能靠一个全局 `Mutex<HashMap>` 替代。
- upstream Recast/Detour是算法与native内存语义的第一参考。`dtNavMesh`、tile refs、`dtNavMeshQuery`、node pool、Crowd和TileCache都设计为可复用owner，不应每query重建。Zircon vendored了这些源码，却在bridge上把它们退化成一次性query object。
- Fyrox当前参考navmesh是较轻量的scene node + `Arc<RwLock<Navmesh>>`，并明确让agent每帧只提供运动目标，由游戏角色执行实际移动。它适合Rust ownership和编辑器节点对照，不足以作为大世界、Crowd或tile streaming上限。
- Bevy主仓没有同等级first-party 3D navigation stack，因此不把“Bevy没有某API”当设计依据。Unity `dev/Graphics`是渲染参考树，不是navigation authority；其内容留给09/10图形审查，只有debug overlay/visibility buffer边界会在本篇定义接口，不用graphics代码反推寻路算法。

### 2.3 明确未做

- 没有修改 production code，没有运行 Cargo、App、Editor、native build、WPR、sanitizer、soak或规模性能测试。本篇是静态 current-source 审查和重构计划，不是实现完成证明。
- 没有因为 Zircon 缺少 Unreal 某个类名就直接登记缺陷。P1依据是产品闭环、正确性、owner/lifecycle、能力真实性和规模复杂度；高级玩法能力进入P2。
- 没有否定当前 Recast/Crowd/TileCache bridge、repath budget、generated snapshot、off-mesh queue和overlay generation。它们是可迁移基础，但必须进入单一 per-world runtime generation，而不是与 legacy/fallback长期并存。

## 3. 当前闭环与必须保留的能力

### 3.1 native第三方基础是真实的

构建脚本已编译 Recast、Detour、DetourCrowd和DetourTileCache，C ABI有显式 status/message DTO，Rust wrapper对native owner使用Drop。simple/tiled bake、path/sample/raycast、Crowd add/remove/target/update/state read与TileCache obstacle都有行为测试。重构应复用vendored upstream和现有ABI分层，扩大bridge为persistent world/tile/query owner，而不是另写第四套寻路算法。

### 3.2 plugin scene system、预算和typed结果已有正确骨架

插件在Update注册 `navigation.agent_tick`，位于 `navigation.main` system set并排在 `ai.behavior_tick`之后；每帧有 `NavRepathBudget`，agent支持Transform或DesiredVelocity写回，report包含arrival/no-path/off-mesh/debug状态。off-mesh bridge已有容量队列、approach/traverse/exit与started/completed event。后续应保持system identity、typed report、fair repath cursor和DesiredVelocity模式，将其接到真实NavWorld/character movement合同，而不是删除这些API后让AI直接调用native query。

### 3.3 generated snapshot和Editor mirror开始具备generation语义

generated bake replace会移除旧handle、插入新asset并推进overlay generation；Editor PIE mirror拒绝wrong session、stale sequence和较旧owner generation。viewport provider使用同一mirror而非全局第二份cache。这是正确方向。下一步需要把generation扩大为world replacement + navmesh tile generation + bake ticket，并把overlay从每帧full snapshot变成generation-owned geometry + bounded delta。

## 4. P1 差距清单

### P1-1：core builtin、plugin Recast和plugin legacy是三个authority，scene内容会触发静默换算法

dynamic session检测到linked navigation插件时解析 `DefaultNavigationManager`，否则注册 `BuiltinNavigationModule`。插件内部 `tick_world_agents`又在无loaded asset、存在runtime obstacle、存在retained obstacle world或任一off-mesh link时清空Crowd并调用 `tick_world_agents_legacy`。core builtin还有独立 `BakedNavMesh`、projection、avoidance和movement。三条路径共享部分DTO与module名称，却不共享compiled artifact、空间索引、query error、Crowd或移动语义。

目标硬切为一个first-party Navigation implementation owner：framework保留稳定contract，plugin runtime拥有NavWorld/Recast执行与scene system；未链接插件时发布typed unavailable capability，不启动功能不同的自研manager。obstacle/off-mesh必须成为同一NavWorld内的TileCache/Crowd/traversal feature，不能作为退回legacy的开关。迁移完成后删除core自研query/movement和plugin legacy算法；如纯Rust查询仍用于工具或故障诊断，必须由显式backend/profile选择、独立capability和一致性门管理，绝不自动fallback。

### P1-2：builtin fallback只提供查询API，没有产品agent system，公开能力与AI判断互相冲突

`BuiltinNavigationModule`只注册driver/manager；它不注册 `navigation.agent_tick`、`NavAgentTickReport`、`NavRepathBudget`、`NavigationDebugCapture`或off-mesh event。全仓production对 `tick_world_agents` 的scene caller只有plugin system。dynamic project仍会向builtin加载 `assets/navigation/main.navmesh.toml`，形成“资产加载成功但agent永不移动”的状态。AI MoveTo又以world是否注册 `NavAgentTickReport` event storage判断navigation availability，因此builtin环境会直接报告unavailable。

目标由capability registry发布 `NavigationRuntimeStatus { availability, backend, world_generation, navmesh_readiness, agent_tick_registered, reason }`。App、AI、Editor和脚本都读取同一状态，不以event类型存在、manager可解析或文件存在推断能力。唯一plugin owner缺失时应快速、可诊断地拒绝navigation组件和MoveTo；不得保留“能调用find_path但产品系统不运行”的半能力fallback。

### P1-3：manager状态不是per-world，entity id、surface id和异步结果可跨world/replacement污染

`DefaultNavigationManager`在CoreRuntime中是一个共享driver；state maps只以 `NavMeshHandle`、entity `u64`或 `Option<surface u64>`为key。crowd、agent motion、obstacle world、off-mesh traversal、bake context/task和generated snapshot都没有 `WorldId`、play session或replacement epoch。同一Core下多个Level/Editor preview可以复用entity id；world replacement后旧bake worker仍持有clone的World并可能发布到同surface key。当前bake generation只能区分同一surface的新旧request，不能证明结果属于当前world。

目标建立 `NavWorldKey { session, world, replacement_epoch }` 和 `NavWorldRegistry`。每个world拥有独立navmesh generations、query pool、Crowd/TileCache、agent slots、build scheduler、events和debug stream。所有handle/ticket包含world与generation；world replace/unload先停止admission、取消/隔离task、retire旧generation，再创建新owner。stale query/bake/overlay/result必须返回typed StaleGeneration，不得凭entity/surface数值碰撞继续apply。

### P1-4：active agent、Crowd和bake backlog没有进入reactive frame demand

plugin scene system只在host产生frame时执行。dynamic session `frame_demand`只合并asset reload和animation demand；navigation没有production caller请求Immediate/After。一个只有navigation agent移动的反应式App可能在首帧后idle，异步bake完成也没有明确wake ticket。现有agent/crowd测试直接调用manager，不覆盖host cadence。

目标由每个NavWorld发布immutable `NavigationFrameDemandSnapshot`：active moving/repath/traversing agents、Crowd step deadline、dirty geometry、pending bake/apply、event/overlay backlog和next wake。session accumulator按generation合并；worker完成通过现有wake sink唤醒owner。pause、无target、arrival、blocked/no-path、manual traversal、world unload和task cancel都需有准确idle transition，避免停帧与busy loop。App/Editor product test要证明只含navigation的scene持续移动并在稳定后回Idle。

### P1-5：NavMeshAsset是弱校验raw DTO，加载、替换、卸载和平台artifact边界不成立

公开asset保存raw vertices/indices/polygons/tile metadata/off-mesh links。`from_bytes`只按version反序列化，不校验finite bounds、index/polygon ranges、三角形退化、tile id/count一致、area表、duplicate link id、link端点或settings compatibility。plugin `load_nav_mesh`直接插入HashMap，core只检查非空。dynamic project从固定 `assets/navigation/main.navmesh.toml`直接serde，绕开versioned bytes/cook。manual load没有unload/refcount/replacement generation；settings变更只清generated bake，手工loaded asset继续存活。

目标分离authoring source、validated cook receipt和runtime artifact。`NavMeshCookArtifactV3`至少包含world/profile/agent/config hash、Recast params、area/filter schema、真实native tile blobs、tile coord/layer/bounds、off-mesh table、source dependency generations、platform/endianness/bridge ABI和完整性hash。loader先bounded validate再prepare；publish用generation swap，handle有lease/unload/retire。TOML只可作为authoring/debug格式，产品runtime不得直接把它当compiled navmesh。

### P1-6：每个path/sample/raycast重建整个dtNavMesh/dtNavMeshQuery，属于灾难性hot path

manager `selected_handle_asset`深clone `NavMeshAsset`。`DetourQuery::from_asset`随后分配并展平vertices、polygons、area costs和links，C++执行polygon neighbour重建、`dtCreateNavMeshData`、`dtAllocNavMesh`和`dtAllocNavMeshQuery`。一个query结束即Drop并释放两者。Crowd创建也从asset重新建query再把ownership转交native。大navmesh每次查询都支付asset复制、O(P²)邻接和native构建成本。

目标让 `NavMeshRuntimeGeneration`在asset publish时一次创建持久 `dtNavMesh`，按实际tile blob attach；只读generation由Arc/lease持有。`dtNavMeshQuery`来自有上限的per-world/per-thread pool，复用node pool与scratch；filter是dense/immutable或query-local小对象。path/sample/raycast只借generation和query slot，不复制asset、不重建邻接、不触发navmesh allocation。telemetry必须记录query wait、node visits、pool miss、partial/out-of-nodes和generation age。

### P1-7：native failure、unsupported和no-path被Option吞并并静默切到另一套算法

`detour::find_path/sample/raycast`以嵌套Option表达“创建失败、调用失败、没有命中”；backend在None时调用Rust fallback。任何off-mesh `cost_override`都会让DetourQuery直接返回None；C++ build/init错误也走同一分支。最终用户可能得到一条由polygon centroid连接的路径，却不知道native backend未运行。错误枚举缺少UnsupportedFeature、InvalidArtifact、OutOfNodes、StaleGeneration、Cancelled、BudgetExceeded和Partial等关键原因。

目标定义分层result：`QueryOutcome::{Success, Partial, NoPath}`与 `NavigationFailure::{Unsupported, InvalidArtifact, Backend, OutOfNodes, Stale, Cancelled, Budget}`分离。native ABI保留detail code/message/counters；backend不得把failure转成success。显式fallback profile需要同一golden corpus和能力矩阵，调用者必须能看到backend identity。cost override等尚不支持的功能应在cook/load阶段拒绝或编译成filter/link语义，不在第一帧查询时暗换算法。

### P1-8：query入口同步、无统一队列/优先级/取消，fallback复杂度和scratch仍不可控

manager trait所有查询都是同步返回owned Vec。AI、gameplay、debug capture和agent repath可在同一帧抢占owner；只有Crowd target repath有简单计数预算。debug capture会为每个agent额外find_path，而当前每次都重建native query。Rust fallback每query用polygon pair共享顶点建图、线性选最小Dijkstra并新分配容器；core fallback虽预编译空间索引和scratch，却被单一mutex串行且与plugin不共享。

目标建立 `NavigationQueryScheduler`：immediate small query、batched projection、async path、agent repath和debug request分lane；ticket包含world/generation、priority、deadline、cancel、node/byte/time预算和结果owner。短query可以借query slot同步完成，长query必须可分片/异步且不持world写锁。批量sample/project共享filter和tile候选。每帧按gameplay/AI/debug公平调度，debug永远不能挤占shipping路径。删除两套fallback scratch前，以1/1k/100k polygon和1/100/10k query验证work counters而非只测wall time。

### P1-9：bake geometry不是场景真实几何，单位quad/disc和空输入fallback会生成错误导航

RenderMeshes分支只检查 `node.mesh`或NodeKind，然后无条件生成transform后的1x1顶面。Physics分支把Box/ConvexHull压成顶面，把Sphere/Capsule/Cylinder压成12边圆盘；TriangleMesh和HeightField为空分支。modifier/area volume按node position或近似bounds判定，carving obstacle用球半径近似并删除整个source node。无source时生成surface volume simple quad。楼梯、坡道、多层、凹形、terrain、mesh collider、门洞和旋转复杂体都会得到与画面/碰撞不一致的navmesh。

目标让geometry collector消费canonical render mesh cook artifact或physics collision cook artifact，包含真实indices/vertices/submesh/material/navigation flags、world transform、winding、scale、bounds和source generation。复杂collider使用cook后的triangle/heightfield/convex数据；primitive可以生成完整解析几何，但不能只取顶盖。source extraction先建立空间索引，按surface/tile bounds只收候选。空输入默认BakeFailed；只有显式 `TestFixture/PlaneSource` authoring node才允许simple quad。golden场景必须覆盖多楼层、楼梯、斜坡、terrain、concave、moving door和modifier边界。

### P1-10：agent与surface的Recast参数没有实际进入bake，settings hash记录了虚假差异

`NavigationAgentSettings`有radius/height/max_climb/max_slope，surface有voxel/tile/min-region/height-mesh等字段；native `RecastBakeSettings::default`却固定cell 0.2/0.1、slope 45、height 2、climb 0.4、radius 0、region 0。simple和planned tile都只用default。diagnostic明确说明voxel/min-region/height-mesh“记录在hash但未应用”，而agent profile字段连该warning都没有。不同agent类型可以获得不同hash/文件名，却烘焙出同样拓扑。

目标由validated `NavigationBakeProfile`推导完整 `rcConfig`：world unit到voxel的离散规则、border、walkable radius/height/climb/slope、region/partition、edge、detail mesh、tile size和max polys必须明确。每个agent profile生成独立artifact或可证明兼容的共享layer；hash只覆盖真正生效且已canonicalized的参数。unsupported knob在authoring/prepare阶段阻止Bake，不能“hash后忽略”。增加窄门、低顶、台阶、坡度边界和多agent golden，验证拓扑确实随参数改变。

### P1-11：所谓tiled bake只在Rust层切块，runtime没有真实Detour tile与streaming生命周期

`prepare_tiled_bake`扫描每个tile是否与mesh相交；每个worker仍接收整份source mesh buffer。tile结果是普通 `NavMeshAsset`，合并时按坐标量化去重vertices并重写raw polygons。runtime `DetourQuery::from_asset`再把全部polygon构造成单个navmesh data并 `nav_mesh->init`；`tiles`字段只是metadata，不是dtNavMesh tile blob/ref。没有tile salt、layer、origin、add/remove、active set、跨tilepath invalidation、streaming residency或tile memory budget。

目标让Recast每tile输出可直接喂给 `dtNavMesh::addTile` 的owned blob及compressed TileCache layer；artifact保存tile coord/layer/config/source hash。runtime generation可以原子add/remove/replace tile并产生changed tile set，路径/corridor按poly ref generation失效。dirty rebuild只收集受影响tile几何，并保持未变tile blob/refs；world streaming attach/detach data chunk。不能再把tile结果合并回一个raw triangle soup后宣称tiled runtime完成。

### P1-12：bake task无取消、容量、shutdown和world epoch，并长期保留整World及所有tile结果

`start_tiled_bake`按值接收完整 `World`。Pending状态在整个任务期间持有World、BakePreparation、plan和每tile `Option<Result<NavMeshAsset>>`；prepare后为每tile向同一pool spawn closure。没有cancel API、queue/worker/memory上限、timeout、shutdown/drain或progress subscription。新generation只从manager map移除旧task，已spawn closure仍因Arc继续运行。spawn结果未进入task状态；完成依赖外部poll/harvest。锁poison统一继续执行。

目标用Runtime11 structured task scope承载 `NavigationBakeTicket`：world/generation/source snapshot、cancel token、deadline、priority、estimated bytes、tile worklist和bounded result pages。geometry snapshot只含所需immutable cook leases，不clone World。supersede/world unload/plugin stop会取消admission并等待或隔离旧task；每tile完成可流式提交staging artifact，最终在owner thread做generation compare和atomic publish。Editor订阅progress/wake而非忙poll。OOM、spawn reject、panic、cancel、partial failure和shutdown各有terminal state。

### P1-13：agent/obstacle/surface扫描仍依赖node_records与动态JSON，每帧重建多组容器

Crowd主路径每帧 `collect_agents(world)`遍历 `world.node_records()`，取dynamic component并serde解析；runtime obstacles、legacy tick、bake几何、modifier和area volume还有各自扫描。随后构造loaded asset clone map、groups、positions、owners、active set、writebacks、entities-by-handle和agents-by-entity。Crowd `read_states`每次按capacity分配Vec。core fallback已有typed generation projection，但plugin没有复用；历史failure也记录过projection编译漂移。

目标建立per-world `NavigationSceneProjection`：用ECS typed query/change tick把surface、agent、obstacle、modifier和link增量编译成dense slots与spatial indices。stable frame只遍历active/moving agents与dirty sources，不clone/deserialize JSON。Crowd binding直接以slot/generation索引，readback使用复用buffer；source change形成dirty tile bounds。projection replacement与scene transaction同generation发布，禁止把现有全量scan简单搬到worker。规模门记录node visits、JSON decode、slot churn、alloc bytes和dirty fan-out。

### P1-14：Crowd容量硬编码且与obstacle/off-mesh不兼容，任何复杂场景都退回O(A²+AO)近似避障

每navmesh Crowd固定 `max_agents=256`、`max_agent_radius=8`，没有配置、分片、LOD或超额策略。任何runtime obstacle/TileCache owner/off-mesh link都会清空crowd并进入legacy。legacy每agent遍历所有agent/obstacle做简单repulsion，直接path/sample/writeback，不是DetourCrowd的collision-free velocity模型；大量临时map/set和debug二次path进一步放大成本。超出256的add failure没有产品级shard/降级合同。

目标让NavWorld按navmesh generation和spatial cell管理Crowd shards，容量/agent radius来自profile与预算。TileCache obstacle和off-mesh corridor必须与同一dtNavMesh/Crowd集成；不能因为功能存在就换算法。远距离/不可见agent采用明确navigation LOD、lower-frequency corridor或server policy，近场保持Crowd。超额产生typed admission result。验收覆盖256边界、1k/10k agents、动态obstacle、link、不同半径/优先级、窄道对穿与长期deadlock，报告avoidance quality和frame budget。

### P1-15：navigation可以直接写Transform，绕过character controller、physics、animation与network authority

Transform writeback把Crowd position直接写入scene transform；legacy/off-mesh也直接update_transform。该路径不执行sweep、step-up、floor、slope、penetration recovery、root motion或server authority，可能穿过物理碰撞。DesiredVelocity模式是正确方向，但只是动态JSON组件，缺少消费ack、realized velocity、blocked reason和corridor correction合同。arrival按Crowd目标点距离判断，不一定代表角色物理位置到达。

目标确定唯一移动责任：Navigation产生corridor、desired velocity、facing intent和path status；CharacterMovement/Physics消费并执行碰撞移动，回传actual transform/velocity/block/floor；Animation/RootMotion在同一movement transaction中仲裁；Network决定authority/prediction。Transform直写仅允许显式kinematic/test profile，不能是默认。arrival/repath依据realized position与navigation projection，并带request id/world generation避免旧反馈。

### P1-16：off-mesh traversal是导航内部插值器，没有gameplay/animation/physics握手

当前link状态机可排队并发出started/completed event，但auto traversal会在Approach/Traverse/Exit中直接线性或抛物线写Transform。Manual只会阻塞。没有能力/动画蒙太奇/root motion/character controller ticket、失败/cancel/timeout、link disable、owner unload、network authority或save/replay语义。桥容量按entity数字管理，也未绑定world generation。

目标把off-mesh解析与实际动作分离。Navigation发出 `TraversalRequest { request_id, world_generation, agent, link, entry, exit, motion_hint, reservation }`；gameplay/character/animation接受、拒绝或异步完成，Navigation仅维护corridor reservation和状态。内置jump可由标准movement ability实现，但仍走同一ticket。link动态disable、agent death、teleport、world unload、network correction和timeout必须释放容量并发布typed terminal outcome。

### P1-17：AI MoveTo用event storage判断能力，并全量扫描report，没有稳定request/outcome合同

AI integration在构造host时调用 `world.events::<NavAgentTickReport>()`；存在即认为navigation可用。每tick把全部arrival/no-path report整理成map，通过generic property path/JSON写agent destination。system顺序是AI先、navigation后，因此反馈通常跨帧；report没有MoveTo command id，entity id复用或连续target可把旧arrival/no-path归到新请求。abort只改组件，缺少对pending query/traversal的cancel。

目标提供typed `NavigationAgentCommand`和 `NavigationAgentOutcomeJournal`，以world/agent slot/request id/generation关联SetDestination、Cancel、Arrived、Partial、NoPath、Blocked、Traversal和Superseded。AI只提交命令并查询自己的最新outcome，不扫描全world event。capability/status来自P1-2。behavior tree abort必须取消同request的query/corridor/traversal，late result被generation拒绝。系统顺序和同帧/跨帧可见性写成contract测试。

### P1-18：event catalog、telemetry和overlay不是可扩展观察面，稳定navmesh仍每帧full copy

plugin每个agent tick都会调用 `navigation_overlay_frame(report.clone())`，遍历全部loaded asset，把每个polygon转成triangle DTO，再 `world.send_event`；即便没有mirror reader也执行。Editor收到后clone完整 `NavigationGizmoSnapshot`，`to_scene_gizmo_overlay`每triangle再生成三条line，共享边重复。debug reader只控制agent path捕获，不能阻止navmesh复制。event catalog声明path query completed/failed/navmesh baked等，但直接manager API没有统一producer；stats也不包含query wait/build/cache/alloc/tile/crowd预算。

目标把静态navmesh debug geometry缓存于 `NavigationDebugGeometryGeneration`：indexed unique edges/triangles、tile bounds、area ids和LOD pages，只在navmesh generation变化时构建。mirror先发generation metadata，按viewport/frustum/selection请求bounded tile pages；agent frame只发delta并有entity/bytes/rate/age预算。零reader时不materialize任何debug payload。所有query/bake/crowd/traversal outcome从同一owner产出typed telemetry/event，catalog由真实producer验证。shipping可关闭高成本debug但保留低成本计数。

### P1-19：Editor bake是公开的false surface，异步operation、选择、undo和资产提交均未闭环

Editor注册Bake Scene、Bake Selected Surface与Clear命令。Runtime handler对Bake的snapshot仅回传request，`prepare`固定错误“requires a pure prepare backend”，`apply`固定错误“cannot reach owner apply”。`NavigationOperationCommand`在edit apply内submit后最多yield/poll 16次，无法等待真实tile任务。`NavigationBakePanelController`有progress模型和backend trait，却只有测试backend，production registration没有创建它。已有open handoff还指出ZUI selected按钮缺surface_entity投影；current controller逻辑不能证明retained click真实使用它。

目标实现Editor-owned authoring workflow但保持runtime execution authority：选择稳定surface entity/generation，提交异步BakeTicket，显示逐阶段/逐tile进度、诊断、cancel和资源预算；完成后先生成staging artifact与差异预览，再以一个transaction原子写入asset/reference/generated snapshot。undo/redo引用content-addressed artifact generation，不深clone world/navmesh；scene/world变化使ticket stale并要求显式rebase/retry。Clear/Restore与Bake走同一事务合同。真实ZUI点击、gateway、worker、publish、save、undo、PIE reload和viewport更新必须端到端验证。

### P1-20：plugin options、默认资产、capability状态和产品验收未接入真实配置生命周期

manifest注册四个navigation option，但全仓production只在registration report中读取它们。dynamic project硬编码默认navmesh TOML，不读取default settings/agent/backend/debug option。manager `load_navigation_settings`立即清部分状态，没有config generation、persist、validation receipt、last-good或manual asset compatibility；debug option也不控制capture/provider。manifest把navigation标为Beta/Partial，却同时公开client/server/editor support，未区分bake、runtime query、Crowd、streaming和authoring readiness。

目标将options接入Config 03的layered snapshot，生成validated `NavigationRuntimeConfigGeneration`并在world admission时冻结。default settings/navmesh由asset GUID/profile解析，backend是实际可用能力而不是唯一enum装饰。配置reload prepare新NavWorld generation，成功后atomic swap，失败保留last-good。capability按query/crowd/dynamic obstacle/bake/editor/streaming分别报告Supported/Partial/Unavailable及原因。产品门必须从项目创建、真实mesh bake、保存、App/Editor运行、AI MoveTo、obstacle/link、导出和重启加载全链验证，不能再以registration/source-shape测试关闭milestone。

## 5. P2 能力差距

### P2-1：缺少large-world partition、navigation invoker和层级路径

当前所有loaded asset常驻，默认选择最小handle；没有world partition cell、active tile set、invoker radius、tile streaming priority、跨cell data chunk或hierarchical coarse graph。大世界会在内存、bake和长路径查询上同时失控。目标在P1真实tile generation之上加入partition-owned nav data chunk、invoker/streaming policy、coarse cluster graph和异步partial corridor；参考Unreal active tiles/data chunk/world partition，而不是在raw asset HashMap上增加LRU。

### P2-2：语义区域、smart link和动态cost仍不足以支持复杂gameplay

area目前主要是u8/cost/walkable，modifier和off-mesh link缺少tag/profile/class、enter cost、fixed cost、team/ability/filter predicate、time-varying danger/cost field和智能link callback。目标建立compiled area/filter schema与gameplay-owned smart-link contract；动态危险/拥堵使用bounded field/grid或query overlay，不通过每帧重烘焙整tile实现。

### P2-3：缺少Mass级群体导航、flow field和可观测LOD质量模型

DetourCrowd适合近场局部避障，不是十万实体的完整方案。目标在同一NavWorld/corridor authority上提供多级agent simulation：近场Crowd、中场稀疏corridor、远场coarse/flow-field或schedule simulation，并定义升级/降级位置连续性、确定性和CPU/内存预算。不能另建与NavMesh无generation关系的“轻量AI移动器”。

### P2-4：只有地面3D人形模型，车辆、飞行、游泳、攀爬与2D没有统一路由

当前profile虽可命名agent type，算法仍是Recast ground navmesh。未来需要按locomotion domain选择ground navmesh、lane/road graph、voxel/volume、water layer、climb surface或2D map，并用统一typed query/route segment组合多模态路径。P1不应提前实现这些算法，但handle、request、traversal和movement合同不得把所有路径硬编码成单层ground polygon。

### P2-5：缺少server determinism、prediction/replay和离线导航质量工具

当前Crowd/native float update、HashMap遍历和event时序没有跨机器确定性声明；network snapshot、rollback、prediction和replay尚未定义。目标区分server-authoritative deterministic-enough模式与客户端visual avoidance，记录query/profile/navmesh generation和command log以重放；Editor提供reachability/island、clearance、link、agent profile、tile memory和regression heatmap离线分析。网络协议细节归08E，但Navigation必须提供stable state/input boundary。

## 6. 目标架构

### 6.1 唯一owner与层次

```text
zircon_runtime::core::framework::navigation
  stable DTO / typed error / capability / query + movement contracts
                    |
zircon_plugin_navigation_runtime
  NavWorldRegistry keyed by session + world + replacement epoch
    NavigationSceneProjection  <- typed ECS change stream
    NavBuildScheduler          <- immutable geometry/collision cook leases
    NavMeshRuntimeGeneration   <- persistent dtNavMesh + real tile refs
    NavigationQueryScheduler   <- bounded dtNavMeshQuery pools/tickets
    CrowdRuntime               <- shards + TileCache + corridor state
    AgentCommand/Outcome       -> CharacterMovement/Physics contract
    Debug/Telemetry            -> bounded generation/delta stream
                    |
navigation editor plugin
  surface/profile authoring, async bake transaction, diagnostics, overlay pages
```

`zircon_runtime/src/navigation`在迁移后只保留absorption/operation glue或被framework吸收的共享合同，不再拥有独立manager/query/movement算法。不得新增root crate；native bridge仍在现有navigation native package中扩展。

### 6.2 资产与运行generation

```text
Scene component + mesh/collision cook generations
  -> incremental tile source snapshot
  -> validated NavigationBakeProfile / rcConfig
  -> per-tile Recast build + compressed layer + dtNavMesh tile blob
  -> staged NavMeshCookArtifactV3
  -> owner-thread atomic publish
  -> NavMeshRuntimeGeneration(tile refs, query pools, crowd/tile cache)
  -> retire after query/crowd/debug leases drain
```

每个artifact与runtime generation都有world/profile/source/config/ABI identity。bake、load、stream和reload共用publish路径；Editor undo只是切换artifact reference，不能建立第二套generated truth。

### 6.3 agent与movement数据流

```text
AI/gameplay SetDestination(request_id)
  -> query/corridor owner
  -> Crowd/local avoidance desired velocity
  -> CharacterMovement/Physics apply + collision
  -> realized motion feedback
  -> corridor correction / arrival / blocked outcome
  -> AI/gameplay journal
```

off-mesh traversal在corridor和movement之间产生typed ticket；Animation/Gameplay完成动作后回执。Navigation不再默认写Transform。

### 6.4 debug与Editor数据流

静态navmesh geometry按generation构建一次、按tile/LOD分页；动态agent只发布bounded delta。viewport provider提交visible tile request，mirror按session/world/generation缓存page。bake operation使用async ticket + progress event + wake，不在edit command内自旋poll。所有UI状态来自同一NavWorld和artifact owner。

## 7. 必须硬切的旧实现

1. 删除产品路由中的 `BuiltinNavigationManager/BuiltinNavigationModule`独立算法；未链接plugin改为typed unavailable。
2. 删除plugin根据obstacle/off-mesh自动进入 `tick_world_agents_legacy` 的分支；相关能力迁入同一Recast/Crowd world。
3. 禁止 `Option` native failure自动调用Rust fallback；改为typed outcome/failure和显式backend profile。
4. 删除每query `NavMeshAsset` clone + `DetourQuery::from_asset`重建路径；所有production query借persistent generation。
5. 删除render mesh单位quad、collider顶盖/disc、TriangleMesh/HeightField空实现与无source volume quad产品fallback。
6. 删除“参数只写hash不生效”的配置；不支持字段在prepare阶段拒绝。
7. 删除raw tiled assets合并后single-tile runtime路径；切换到真实tile blobs/add-remove refs。
8. 删除全world `node_records` + JSON per-frame agent/obstacle projection；切换typed incremental slots。
9. 删除默认navigation Transform直写与内部off-mesh插值；接movement/traversal ticket。
10. 删除Editor bake固定错误handler、16次busy poll和test-only backend；切换async transaction。
11. 删除稳定navmesh每帧full overlay event；切换generation geometry/page + agent delta。
12. 删除manifest-only options；每个公开option必须有config consumer、状态与行为测试，否则从产品surface移除。

硬切不允许compat alias、双写、shadow manager、silent fallback或“先保留旧路由以后再删”。如需要分阶段迁移，阶段内只能有一个production owner，旧实现仅存在于不参与产品构建的oracle/test fixture。

## 8. 分阶段重构计划

### M0：能力真实性与唯一authority

- 冻结current-source inventory/fingerprint，核对当前overlay Session和所有open/fixed failure。
- 引入分项 `NavigationRuntimeStatus`；unlinked状态明确Unavailable。
- 选定plugin Recast为唯一production owner，停止builtin/legacy新功能增长。
- 建linked/unlinked、client/server/editor capability matrix和产品bootstrap测试。

退出门：同一session/world不可能解析到第二manager；加入obstacle/link不改变backend identity；AI不再以event storage判断能力。

### M1：world/replacement生命周期与增量scene projection

- 建NavWorldKey/NavWorldRegistry和world-scoped handles/tickets。
- 将surface/agent/obstacle/modifier/link编译为typed dense projection与spatial index。
- 接world replace/unload、plugin stop、frame demand与wake。
- 移除per-frame node_records/JSON扫描。

退出门：双world相同entity id互不污染；replacement后旧query/bake/overlay全部stale；stable frame JSON decode和world scan为0。

### M2：真实geometry、profile与cook artifact

- 接render mesh/physics collision cook leases，完成triangle mesh/heightfield/compound/primitive语义。
- 从agent/surface settings推导完整rcConfig并做golden边界测试。
- 定义NavMeshCookArtifactV3、bounded validator、dependency/config/ABI identity。
- 取消空几何simple quad产品fallback。

退出门：楼梯/坡道/多层/terrain/凹形/门的bake与collision/render一致；不同agent profile产生可解释差异；corrupt artifact在load前拒绝。

### M3：真实tile generation、异步build与streaming publish

- native bridge输出并接收real Detour tile blob/compressed layer。
- build scheduler实现bounded queue、cancel、progress、deadline、memory accounting和shutdown。
- dirty bounds只重建受影响tile；staging完成后atomic publish。
- 支持tile attach/detach、data chunk、path/corridor invalidation与retire。

退出门：未变tile bytes/ref保持；cancel/supersede无late publish；world unload没有worker/lease遗留；跨tile路径连续。

### M4：persistent query runtime与typed scheduler

- 建NavMeshRuntimeGeneration、persistent dtNavMesh、query pools和filter cache。
- 分离outcome/failure，删除silent fallback。
- 实现sync small、batched project和async path lanes，加入cancel/budget/fairness。
- 补node visit/out-of-nodes/partial/query wait telemetry。

退出门：steady-state query不clone asset、不build navmesh、无非预算分配；1/100/10k并发query保持generation安全；debug负载不影响gameplay lane。

### M5：Crowd、TileCache、agent与movement合同

- 让Crowd/TileCache共享M4的persistent navmesh generation。
- obstacle/off-mesh不再触发legacy；crowd sharding/capacity/LOD可配置。
- 切换typed agent slots和复用FFI readback buffers。
- 默认输出DesiredMovementIntent并接CharacterMovement/Physics feedback。

退出门：1k/10k agent规模曲线、动态obstacle/link、超容量和不同半径有明确结果；navigation不直接穿透physics。

### M6：off-mesh、AI与gameplay合同

- 实现TraversalRequest/terminal outcome/reservation/cancel。
- AI切换request-id command/outcome journal；abort取消完整链。
- 补animation/root motion/network authority接口，具体Network实现留08E。
- 明确arrival/partial/no-path/blocked/repath语义。

退出门：旧result不能完成新MoveTo；link disable/death/teleport/unload释放容量；manual/auto/gameplay traversal共用一套状态机。

### M7：Editor bake、资产事务与debug overlay

- production接通surface row/selection/typed invocation。
- async bake ticket驱动progress/cancel/diagnostic；staging artifact原子commit。
- undo/redo/save/reload/PIE使用content-addressed generation。
- overlay改为static generation pages + dynamic bounded delta与viewport culling。

退出门：真实ZUI点击到runtime bake再到asset/viewport闭环；stable navmesh不每帧重建；PIE结束/能力关闭立即清理。

### M8：large-world与高级能力

- world partition/nav invoker/hierarchical graph和tile residency。
- semantic cost fields/smart links；Crowd LOD/flow field。
- 多locomotion domain的可扩展request/segment模型。
- server/replay boundary与离线质量分析。

退出门按独立P2项目设定，不能阻塞M0-M7基础硬切，也不能在P1 owner未收敛时并行新增第二套系统。

### M9：产品与性能资格

- App、Editor、server、export、reload、device/profile config的真实产品矩阵。
- Windows优先的受管native/Cargo验证，再按具体Linux要求进入WSL。
- WPR/heap/native sanitizer/fault/soak/long-running crowd与streaming capture。
- 与固定Unreal/Fyrox/Godot场景同硬件、同agent/geometry规模、同质量目标对比；报告方法和误差，不能只写“更快”。

## 9. 验收门

### 9.1 正确性门

- asset：corrupt version/hash/tile/index/area/link/NaN、cross-platform round-trip、generation retire和unload。
- bake：stairs、slope阈值、clearance、low ceiling、multi-floor、terrain、concave、modifier、door、multiple agent profiles与dirty tile oracle。
- query：path/sample/raycast/partial/no-path/cost/filter/off-mesh、tile replace、out-of-nodes、cancel和stale generation。
- crowd：collision corridor、deadlock、obstacle add/remove、link queue、capacity/LOD、deterministic ordering和world replacement。
- movement：sweep/step/floor/root motion、blocked/repath、teleport、arrival和server correction。
- Editor：selection、bake/cancel/fail/commit、undo/redo/save/reload/PIE、overlay lifecycle。

### 9.2 性能门

| 维度 | 场景 | 必报指标 |
|---|---|---|
| artifact/load | 1 / 1k / 100k tiles，1 MiB / 1 GiB source | validate/build wall、peak RSS、copy bytes、tile residency、publish pause |
| query | 1 / 1k / 100k polygons，1 / 100 / 10k requests | navmesh builds/query必须0、query alloc、node visits、pool wait、p50/p95/p99、partial/out-of-nodes |
| agents | 1 / 256 / 1k / 10k agents，0/10k obstacles | projection visits、JSON decode必须0、Crowd/avoidance ms、FFI bytes、alloc bytes、repath fairness、deadlock |
| bake | 1 / 1k / 100k source meshes，1 / 1k tiles | source candidates、triangle bytes、task queue/age、cancel latency、peak result bytes、main-thread apply |
| overlay | hidden/stable/dirty，1k / 1M triangles | rebuild count、mirrored bytes、unique edges、commands、viewport extract p95、drop/age |
| lifecycle | 1k world replace/plugin reload cycles | stale apply必须0、live tasks/queries/leases、RSS drift、shutdown p99 |

性能通过需要steady-state allocation与work counter证据、CPU profile和产品trace。微基准只能定位算法，不能替代App/Editor帧时间。与Unreal比较必须冻结build配置、地图、Recast参数、agent行为和质量容差；若质量或能力不同，结果不得标注“优于”。

### 9.3 故障与平台门

- native allocation/build/query failure、worker panic、spawn reject、OOM预算、corrupt tile、asset reload失败和plugin unload。
- Editor在bake中关闭项目、切场景、删除surface、修改profile、undo和崩溃恢复。
- Windows MSVC为首个受管门；随后验证Linux clang/GCC和目标平台endianness/ABI。ASan/UBSan覆盖C++ bridge与vendored调用边界。
- 8小时Crowd/obstacle/link soak、连续tile streaming和频繁dirty rebuild，无增长队列、handle泄漏、generation错用或busy loop。

## 10. 与既有计划的关系

`docs/plans/zircon_plugins/05-navigation.md`仍是实现owner，但其M1-M6“完成”记录需要按本篇current-source重新打开以下内容：M1真实geometry和settings、M2真实Detour tile artifact/任务生命周期、M3 persistent Crowd/query和scale、M4 obstacle不退legacy、M5 movement-owned traversal、M6真实Editor bake与bounded overlay。已有output record保留历史证据，不改写为失败；新实现应以新的milestone/return record覆盖current status。

open handoff中，fallback hotpath、world projection、selected surface参数、operation status和overlay publication都与本篇相交。当前overlay source已经比handoff初始状态前进，但未完成性能与最终managed gate；不能只因文件存在就标fixed。operation command已使用V2 progress shape，但Bake handler仍固定失败；应在source recheck后分别判断旧ABI handoff是否可关闭、产品Bake gap是否新建/并入M7。避免用一个“全部fixed”记录掩盖不同生命周期键。

## 11. 完成定义

Navigation只有在以下条件同时成立后，才能从“Beta/Partial API实现”提升为工程级runtime：

1. 只有一个production manager/backend authority，obstacle/off-mesh不会静默换算法，unlinked状态明确Unavailable。
2. 每个world/replacement有独立NavWorld generation，所有query/bake/crowd/debug result都可拒绝stale。
3. bake消费真实mesh/collision geometry，所有生效参数进入rcConfig，空输入不会生成假navmesh。
4. runtime加载真实Detour tile artifact，持久复用dtNavMesh/query pools，并支持tile replace/stream/retire。
5. steady-state agent tick是typed incremental projection，不做node_records/JSON全扫；Crowd有容量、LOD和规模证据。
6. Navigation默认只产出movement intent，CharacterMovement/Physics/Animation/Network共同完成移动与off-mesh动作。
7. AI使用request/outcome合同；Editor Bake/undo/save/PIE/overlay是真实产品闭环。
8. 配置、capability、telemetry和debug surface都由真实consumer/producer支持，不再有manifest或catalog装饰项。
9. correctness、fault、platform、soak和规模性能门全部有新鲜证据；任何“优于Unreal”结论都有可复现的同质量对照。

在这些门完成前，本篇状态保持 `implementation_status: pending`。
