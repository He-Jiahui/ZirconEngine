---
related_code:
  - zircon_runtime/src/navigation
consumer_code_read_only:
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/script/vm/gameplay_host/navigation.rs
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/runtime/src/manager/query.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/fallback_query
base_reports:
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/performance/01/2026-07-19-runtime-navigation-static-review.md
owner_plans:
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
  - docs/plans/zircon_plugins/05-navigation.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationOctree.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMesh.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMeshGenerator.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMeshDataChunk.h
  - zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Include/DetourNavMeshQuery.h
doc_type: currentness-revalidation
status: static_current_revalidated_structural_and_dynamic_pending
---

# Runtime navigation fallback当前性与production owner重验（2026-08-23）

## 冻结边界

| 模块 | 已逐文件复读 | physical lines | bytes | inline tests |
|---|---:|---:|---:|---:|
| `zircon_runtime/src/navigation`根文件 | 4/4 | 678 | 23,514 | 1 |
| `zircon_runtime/src/navigation/operation` | 3/3 | 259 | 9,468 | 0 |
| `zircon_runtime/src/navigation/runtime` | 8/8 | 2,414 | 83,238 | 25 |
| 合计 | 15/15 | 3,351 | 116,220 | 26 |

ordered relative path + NUL + raw bytes + NUL SHA256为
`067c4f0966dd40ed20ae7f1ed9b5555248f4f65887f9410e81e152253bec6e0a`。runtime目录当前无未提交
改动；navigation插件consumer有其他Session的纯格式改动，本轮只读保留。Optimize08d之后runtime
operation handler有一行提交变化，故本报告重新读取当前源码，不继承旧报告的currentness状态。

产品链读到
`RuntimeDynamicSession::build -> linked plugin或BuiltinNavigationModule -> manager/scene driver ->
plugin navigation.agent_tick或script targeted tick -> query/movement/overlay`。插件/native只作为consumer证据，
不计入本报告15/15验收分母。

## 已纠正的旧算法结论

- builtin `BakedNavMesh` adjacency已使用canonical shared-edge index，不再枚举全部polygon pair。
- nearest/sample已有polygon BVH；1/约1k/约100k polygon静态测试把candidate限制到64、BVH node限制到256。
- A* scratch由mesh持有并用epoch stamp复用，不再每query重新分配/清空所有P槽；area cost也已预投影为64项表。
- builtin world projection已从`node_records + serde_json::from_value clone`改为typed dynamic component rows，
  stable world复用projection；avoidance使用2m cell index、每类最多64 candidate并旋转公平性。
- repath有默认32 query/frame预算、round-robin cursor和stable destination route cache。

因此旧PERF-MVP-437/438不能再把上述fallback局部实现写成现存瓶颈。但这些修复只在builtin路径；链接
first-party navigation插件后的production路径没有自动获得它们。

## 当前结构瓶颈

### N-P1-1：三套authority仍会按内容和错误静默换算法

dynamic session在未链接插件时装载builtin，链接时装载插件。插件有loaded mesh且没有obstacle/off-mesh时走
Crowd；没有mesh、存在runtime obstacle、保留obstacle world或任一off-mesh link时清空Crowd并进入legacy。
native query任何unsupported/create/query错误又由`Option::None`自动切纯Rust fallback。相同world只要增加
一种组件或触发backend failure，算法、移动语义和复杂度都会变化，却没有typed backend transition。

目标是插件Recast/Detour成为唯一production implementation owner；obstacle/off-mesh进入同一per-world
TileCache/Crowd/traversal generation。未链接插件发布typed Unavailable。纯Rust实现若保留，只能是显式
oracle/tool profile，不得作为错误分支或产品兼容层。

### N-P1-2：真正native query仍每次clone、展平、重建并销毁owner

插件`selected_handle_asset`每query深clone `NavMeshAsset`。`detour.rs`随后生成vertices/polygons/cost/link
数组，调用`zr_nav_detour_create_query`创建`dtNavMesh + dtNavMeshQuery`，查询结束即Drop/free。C++构造阶段
仍以polygon/edge嵌套循环重建邻接。若native返回非OK，`RecastBackend`再执行纯Rust fallback；该fallback
仍以polygon pair建图。

这使一次查询包含`O(asset bytes)`复制、`O(P^2)`最坏构造和native allocation；steady-state query无法定义
稳定p95/p99。正确目标是`NavMeshRuntimeGeneration`持久拥有tile refs、`dtNavMesh`和有界query pool，查询只
借generation/filter/scratch；load/reload/tile replace在owner边界原子发布并等待lease排空后retire。

### N-P1-3：builtin改进没有进入插件production ECS projection

插件Crowd输入仍每帧`world.node_records()`扫描全部node并解析agent JSON，同时检测obstacle。随后构建多个
`HashMap/HashSet/Vec`分组与position/writeback。obstacle/off-mesh触发的legacy又分别收集agents、positions、
obstacles并逐agent查询。runtime builtin的typed rows、spatial avoidance和repath cache优化因此不是产品热点
修复；继续扩展builtin会扩大重复owner。

目标由Runtime08提供world-scoped typed navigation slots和change admission，只投影surface/agent/obstacle/
modifier/link变化；stable frame的全world node scan、JSON decode和容器重建必须为0。

### N-P1-4：world/replacement生命周期缺失

builtin manager的loaded mesh、generated snapshot、projection和route以global handle/entity/surface数值为key，
没有session/world/replacement epoch。插件state同样由Core共享，query/crowd/bake/overlay没有统一NavWorldKey。
多Level相同entity id、world replacement、late bake和plugin reload都可能跨生命周期复用状态。

目标建立`NavWorldRegistry<session, world, replacement_epoch>`，所有mesh generation、query/crowd/tile-cache、
agent slot、bake ticket、overlay cursor都带world/generation；stop/replacement先关admission，再cancel或隔离任务，
最后retire generation。stale结果必须是typed outcome。

### N-P1-5：fallback几何和movement合同不应成为产品语义

builtin polygon只保留AABB/center；contains/project不使用真实polygon边界，raycast只判断终点是否落入某AABB，
path通过polygon center且没有funnel。一个mesh的path query还共享单个scratch mutex，所有并发find-path串行。
agent找不到loaded mesh时把destination当作path target继续移动，并直接`world.update_transform`；插件legacy也有
同类直达destination与Transform写回。script host又可在单次调用中手工`tick_world_agent`，绕过统一scene
system cadence。

这些行为不适合继续局部修补。Navigation必须输出path/corridor/desired movement intent；CharacterMovement/
Physics应用碰撞后的realized motion并回执。无backend/no-path/partial/blocked各自有typed terminal outcome，
禁止“没有navmesh仍直线移动”。

### N-P1-6：产品调度、Bake和Editor证据仍不闭环

builtin module只注册lazy driver/manager，没有`navigation.agent_tick`、tick report/debug resource；AI仍以
`NavAgentTickReport` event storage存在性推断navigation availability。active agent和pending bake没有进入App
frame demand。Runtime operation虽然注册Bake Scene/Surface，但prepare/apply固定返回“需要pure prepare
backend”；Clear/Restore才有snapshot transaction。

插件overlay只在有reader时发布是正确进步，但每次enabled frame仍把全部loaded asset物化为triangle DTO。
最终目标是typed runtime status + reactive demand/wake、真实async bake ticket/progress/cancel/commit，以及static
generation tile pages + dynamic agent delta的debug流。

## Unreal与Detour源码裁决

- `NavigationSystem.h:290-306,436-478,605-640`把NavDataSet、world、invoker、active tile、dirty事件和tick放在
  同一world生命周期；`218-233`还记录tile wait与time-slice预算。
- `NavigationOctree.h:141-195`提供持久octree和Add/Append/Update/Remove node，证明world输入应增量维护，
  不应每帧全world JSON扫描。
- `RecastNavMesh.h:1518-1521,1652-1661,1763-1816`持有Recast实现并提供batch query/path/raycast入口，
  不是每次请求重建navmesh。
- `RecastNavMeshGenerator.h:279-352,369-438,648-808`定义time-sliced tile状态、异步tile task、dirty layers、
  discard/cancel和生成结果；这支持bounded build scheduler和atomic generation publish。
- `RecastNavMeshDataChunk.h:12-40,68-78,90-122`保存tile/cache raw data并支持attach/detach ownership；Zircon
  当前把tiled Rust asset合回一次性query object，不等价于tile runtime。
- vendored `DetourNavMeshQuery.h:169-194,219-259,519-580`要求query对持久`dtNavMesh`初始化并持有node pool，
  还提供sliced path；Zircon bridge已经使用上游库，但生命周期用法相反。

Zircon不复制Unreal Actor/UObject层次。需要吸收的是world owner、incremental source、persistent tile/query、
bounded async build、generation/lease和typed failure。没有同地图、Recast参数、agent行为和质量容差的同机数据，
不声明耗时或功耗接近/优于Unreal。

## 实施顺序

1. **M0能力真相与唯一owner**：定义`NavigationRuntimeStatus`；unlinked为Unavailable；冻结builtin/legacy，
   禁止silent fallback和内容触发backend切换。
2. **M1 NavWorld lifecycle与typed projection**：world/replacement key、dense component slots、reactive demand、
   cancel/retire；stable frame全world scan/JSON decode为0。
3. **M2真实cook/tile artifact**：真实mesh/collision geometry、完整Recast profile、version/config/ABI receipt、
   Detour tile blob和atomic tile generation。
4. **M3 persistent query/build scheduler**：query pool、sync small/batched/async lanes、deadline/cancel/fairness、
   dirty tile rebuild与attach/detach。
5. **M4 Crowd/TileCache/movement**：obstacle/off-mesh不换backend；navigation只发intent，physics/character
   movement回执realized motion。
6. **M5 AI/Editor闭环**：request/outcome journal、async bake transaction、generation page overlay。
7. **M6动态资格**：current-source产品二进制上测1/100/10k requests与agents、1/1k/100k polygons/tiles，记录
   query builds必须0、clone bytes、alloc、pool wait、node visits、queue age、main/worker CPU、RSS和功耗。

## 当前验收状态

本轮完成15/15 current-source静态复读、产品caller、插件/native热点与Unreal/Detour源码核对。没有production
代码修改：可见的简单fallback局部优化不在目标production owner，继续改它会延长双实现；正确修复需要先完成
Plugins05/Runtime14的唯一owner硬切。

可执行的静态门结果：`test_plugins_05_navigation_overlay` 1/1通过（0.003s）；15/15 navigation Rust文件的
`rustfmt +1.94.1 --edition 2021 --check`与scoped diff check通过。docs convention当前扫描3,156份文档、
83,807条路径，复现全仓既有801项违规，本轮两份navigation文档owned violation为0。

managed Cargo执行身份已归档，Rust test执行数为0；当前源码没有可启动产品二进制，WPR、功耗和RenderDoc
采样数为0。RenderDoc仅在navigation debug/skinned scene可见输出需要验证时使用，不能替代CPU/query/build
profile。本模块保持`static_current_revalidated_structural_and_dynamic_pending`，不得写入`review.md`。
