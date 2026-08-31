---
title: Runtime Navigation / NavMesh / Recast / Detour / TileCache / Crowd / Query / Pathfinding / Obstacle / Off-Mesh Link / Bake / Streaming / World / Editor 当前源码复审
category: zircon_runtime
report_id: Runtime141
review_date: 2026-08-25
baseline_head: 514d2127710757e7e991646557934469e771609b
baseline_epoch: 423
verification_head: 3af73550dd00fe4805f71e96ce199f4ab633687f
verification_epoch: 424
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
related_code:
  - zircon_runtime/src/core/framework/navigation
  - zircon_runtime/src/navigation
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/script/vm/gameplay_host/navigation.rs
  - zircon_plugins/navigation
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
plan_sources:
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/05/2026-07-13-navigation-m6-output-records.md
  - docs/plans/zircon_plugins/05/2026-07-27-navigation-m6-selected-surface-manifest.md
  - docs/plans/performance/01/2026-07-19-runtime-navigation-static-review.md
  - docs/plans/performance/01/2026-07-30-runtime-framework-animation-ai-navigation-tasks-static-review.md
  - docs/plans/zircon_plugins/05/failure-2026-07-15-navigation-bake-selection-operation-arguments.md
  - docs/plans/zircon_plugins/05/failure-2026-07-19-navigation-runtime-fallback-hotpath.md
  - docs/plans/zircon_plugins/05/failure-2026-07-27-navigation-world-scan-deserialize-value.md
  - docs/plans/zircon_plugins/05/failure-2026-07-30-navigation-overlay-frame-publication.md
  - docs/plans/zircon_plugins/05/failure-2026-08-02-navigation-editor-operation-status-v2-cutover.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Navigation
  - dev/godot/modules/navigation_3d
  - dev/godot/servers/navigation_3d
  - dev/godot/scene/3d/navigation
  - dev/Fyrox/fyrox-impl/src/scene/navmesh.rs
  - dev/Fyrox/fyrox-impl/src/utils/navmesh.rs
  - dev/Fyrox/editor/src/scene/commands/navmesh.rs
  - dev/bevy/crates/bevy_input_focus/src/lib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime141 · Navigation 当前源码复审

## 1. 结论

当前Navigation不是“完全没有实现”，但也远未形成虚幻级工程导航系统。可保留的真实底座包括：中立NavMesh/agent/surface/query/bake合同；vendored Recast、Detour、DetourCrowd和DetourTileCache；native bake/query/crowd/tile-cache C ABI；builtin预编译polygon graph、空间索引、query scratch、route cache和有界repath；plugin的scene system、typed tick report、dirty/tiled bake、off-mesh traversal、overlay generation及PIE mirror；以及近期加入的dynamic-row projection、entity-row索引、空间避障候选上限、scratch复用和若干release-only规模计数测试。这些基础必须迁入统一架构，不能退回简单A*或继续新增第四套实现。

核心问题是产品由多套互不等价的authority拼接。`zircon_app`默认仍启用builtin `navigation`，首方Navigation runtime/editor只在特定feature或project selection下出现；同一Editor Host还可同时链接builtin和source plugin。Runtime framework、builtin manager、Recast plugin与plugin legacy path共享名称和部分DTO，却不共享prepared artifact、world owner、query failure、Crowd或movement语义。plugin只要看到任意runtime obstacle、保留的obstacle world或任一off-mesh link，就清空Crowd并退回legacy路径；相同场景增加一个组件便静默改变算法和复杂度。

运行时artifact和native owner没有工程化。`NavMeshAsset`仍是弱校验的raw vertices/indices/polygons/tiles/links DTO，dynamic project又从固定`assets/navigation/main.navmesh.toml`直接读TOML并交给manager，绕过asset registry、cook artifact、依赖图和residency。plugin的常规find/sample/raycast仍从asset重建`dtNavMesh`与`dtNavMeshQuery`，C++ bridge再以polygon/edge嵌套循环重建邻接；失败、unsupported与no-path通过`Option`折叠后可静默落入O(P^2) Rust fallback。builtin虽然已把graph、空间索引和scratch持久化，却是另一套生产行为且单mutex串行，不能替plugin关闭该缺陷。

bake最严重的问题不是“缺几个shape”，而是输入和产物都不真实。Render Mesh/Cube被替成单位顶面；Box只取顶面；Sphere/Capsule/Cylinder变成顶圆盘；Convex变成AABB顶面；TriangleMesh与HeightField不产生几何；无输入时还能生成surface quad。agent radius/height/climb/slope和多数surface高级参数只进入hash/warning，native继续使用默认Recast设置。所谓tiled bake为每tile重复完整source、按centroid筛选后又合并回raw DTO；runtime没有持久Detour tile blob、salt/ref、layer、attach/detach、seam或streaming lifecycle。TileCache固定构造单个`0/0/0`layer并硬编码小容量，因此不能把存在Recast/TileCache符号解释为大世界动态导航完成。

agent链已有局部改进，但仍不具统一移动authority。builtin使用空间cell、最多64候选和轮转公平；plugin native Crowd仍固定256 agents/radius 8，复杂场景退回每agent每帧寻路与O(A^2)避障。两条路径都可直接写Transform；没有mesh时还会朝目标直走。off-mesh traversal由Navigation内部用固定线性/抛物线插值，没有Character Movement、Physics、Animation、Gameplay Ability、Network或cancel/timeout握手。AI MoveTo现在会写`NavMeshAgent.destination`并读取`NavAgentTickReport`判断Arrived/NoPath，这是实质进展；但它仍以dynamic JSON与事件存储存在性推断能力，没有request handle、deadline、cancel、path-following state或world generation。

Editor公开能力与真实效果差距仍是P0。Runtime Bake operation的`prepare`固定返回“requires a pure prepare backend”，`apply`也固定拒绝；独立BakePanel/controller/surface rows只有定义、re-export和测试，没有产品owner。Surfaces、Agents/Areas、NavMesh/Settings以及五个component drawer主要是业务`Space`；OffMeshBridge甚至没有drawer。Toggle Navigation Gizmos没有handler，四个filter没有event binding，provider使用固定Default options且只消费PIE mirror。operation虽读取V2 progress，请求/结果仍是V1并固定`yield_now`轮询16次，无cancel/wake；失败后还被记录为Applied。内置Navmesh AI Workbench的tile、agent、query、rebuild文案和反馈全部固定，和Navigation plugin、document、job或artifact没有连接。

示例也没有证明产品闭环。Vampire手写一个version 1 raw TOML navmesh，script host可在单次脚本调用里直接执行`SceneNavigationRuntime::tick_world_agent`，而AI behavior tree另有`navigation.move_towards_target`文本但实际game script走私有host函数。Workbench只证明模板可达，App的typed consumer测试只证明selected plugin tick后能收到一个overlay event，均不证明真实mesh cook、artifact保存、Editor bake、Client加载和Character移动构成同一链。

因此当前没有证据支持“性能和表现优于Unreal”。7个ignored release test和若干计数器只覆盖局部clone、projection、lookup、scratch或小型synthetic mesh；没有同场景geometry、同agent profile、同query filter、真实Detour persistent owner、10k agent/百万polygon、world streaming、dynamic obstacle churn、跨平台、fault、soak或CPU/RSS/tail-latency竞争证据。本报告只刷新事实与重构门，不修改生产代码。

历史台账重判：Runtime08D的20项P1为 **13 Open、7 Partial**，5项P2全部Open；Plugins14的48项P1为 **43 Open、5 Partial**，12项P2全部Open；Editor19的5项P0、60项P1和12项P2全部Open。五份Navigation failure均有局部源码进展但仍是Open。32项综合资格门为 **26 Fail、6 Partial、0 Pass**。

## 2. 审查边界、方法与currentness

### 2.1 冻结Navigation范围

统计口径为当前工作树物理行、非空行、文件bytes、Rust `#[test]`和`#[ignore`声明。fingerprint按normalized lowercase path排序，对每个文件拼接`path + NUL + lowercase(file SHA-256) + LF`后再取SHA-256。产品consumer和五引擎集合是明确选择集，不代表所在仓库总规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Runtime Navigation中立合同全量 | **19 / 1,645 / 1,470 / 50,101 / 9 / 0** | `858db7da21a7b58159ed79a8c738b0bd26eed1c8c208cb63a181d67eb573a410` |
| Runtime builtin Navigation全量 | **15 / 3,441 / 3,154 / 119,698 / 28 / 1** | `d70083684858a0e2fe5a75e1807c5a191a4ee2777bcc0c7a9b2185cc4a8b386a` |
| Navigation插件全量（含vendor） | **183 / 48,058 / 42,585 / 1,541,779 / 148 / 6** | `8fb8abfd5eef8da4b4160a960f6db0f1b1f5e3a3774147bdba02d7ce39809375` |
| 插件自有代码（不含vendor） | **132 / 20,430 / 18,743 / 704,429 / 148 / 6** | `d89642850f839cf2fa63038f8850783a9631b68e2e191fae786c586bfb541b1e` |
| vendored Recast/Detour | **51 / 27,628 / 23,842 / 837,350 / 0 / 0** | `123b3daffb9e7c560b31b9311464739c0d0073e495ed7a2ff5def206c664de38` |
| Navigation canonical union | **217 / 53,144 / 47,209 / 1,711,578 / 185 / 7** | `6f6e4b35ed3f12d65287d361896fe7dea1d05e191b21a5f721f6c94cd4ffbacd` |
| App/catalog/AI/script/example/Workbench产品consumer | **25 / 6,032 / 5,530 / 227,975 / 85 / 0** | `9bc681319da886106bcf37ed254497133b273003fe415e9590b0f9980e293f72` |
| Zircon selected union | **240 / 59,020 / 52,601 / 1,933,760 / 268 / 7** | `f83b123fb7870c2655ca8dec031be849503d7816640d537bc81951c84d7ab36e` |
| 五引擎参考选择集 | **47 / 48,948 / 40,717 / 1,863,417 / 8 / 0** | `77a3afc1f6d855cd3ab267f5fb4f4bd2e6ef01440344bf985b76ad559087c2e3` |

文件扩展分布为`.cpp` 34、`.h` 23、`.md` 1、`.rs` 138、`.toml` 5、`.txt` 5、`.zui` 11。参考revision为Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`；`dev/UnrealEngine`没有独立Git元数据，以24个选择文件及参考集合fingerprint冻结。

### 2.2 检查方法

1. 逐文件读取framework、builtin、plugin runtime/native/editor/dist/vendor与测试，并沿DTO、manager、scene system、C ABI和native owner双向追踪。
2. 沿`App target -> project selection -> first-party catalog -> provider registration -> module/service -> World system -> asset/query/movement`检查普通产品可达性，不从feature名或registration test推断功能完成。
3. 沿`Scene geometry -> bake snapshot -> Recast config -> tile/layer artifact -> runtime load -> query/crowd/tile-cache -> movement intent`核对真实数据是否守恒。
4. 沿`Editor contribution -> ZUI binding -> operation factory -> Runtime handler -> artifact transaction -> PIE mirror/overlay`检查authoring闭环和false surface。
5. 对Runtime08D、Plugins14、Editor19、五份failure和两份performance review逐项重判；局部降分配或增加DTO最多记Partial。
6. 对照Unreal NavigationSystem/Recast/AIModule、Godot NavigationServer/Map、Fyrox runtime/editor navmesh；Bevy和Unity Graphics只在其真实边界内参考。

### 2.3 动态证据边界

- Session基线为`514d2127710757e7e991646557934469e771609b` / epoch 423；静态验收时仓库推进到`3af73550dd00fe4805f71e96ce199f4ab633687f` / epoch 424，但所选Navigation源码相对注册基线没有提交差异。
- 当前仓库存在大量用户或其他Session改动；本轮只写本报告和三处索引，不覆盖、不回退任何现有内容。
- 本轮review-only，没有运行Cargo、native build、App、Editor、PIE、真实mesh cook、WPR、sanitizer、fuzz、scale、soak或竞争benchmark。
- 静态调用图足以证明的固定Bake失败、synthetic geometry、per-query native rebuild、silent backend fallback、产品controller缺失和固定Workbench反馈不因未跑Cargo而改变。
- Tooling按用户要求排除；只使用session coordinator保护文档写入范围，不审查或优化其实现。

## 3. 当前真实产品链路

```text
ordinary Client
  zircon_app default -> builtin navigation
  -> raw TOML NavMeshAsset loader -> builtin manager
  -> compiled Rust polygon graph/query + lightweight agent movement

selected first-party Navigation
  project selection -> runtime/editor catalogs
  -> source plugin manager + Recast native bridge
  -> navigation.agent_tick -> Crowd OR legacy fallback
  -> direct Transform / DesiredVelocity

Editor Host
  matching views/templates are reachable
  -> Bake operation handler always rejects production bake
  -> standalone BakePanel/controller has no product owner
  -> overlay uses PIE mirror with fixed options
  -> most authoring surfaces are empty Space placeholders

example/product
  Vampire -> hand-authored raw v1 navmesh TOML
  script -> private nav_move_towards_entity/tick_world_agent path
  Workbench -> fixed tile/agent/query/rebuild state
```

目标产品链只能有一个：App提交project/profile selection；Runtime解析唯一provider并创建per-World generation；Editor提交versioned source document和operation；compiler从真实render/physics/terrain artifact生成可直接装载的Detour tile/layer artifact；Runtime持久装载、调度query/Crowd/TileCache并只发布movement intent与observation；Character/Physics/Network负责最终移动。

## 4. 必须保留的基础

1. 保留framework的中立Navigation DTO、typed report和manager/service边界，但给identity、owner、generation、deadline、cancel与budget补完整语义。
2. 保留vendored Recast/Detour/Crowd/TileCache和当前Rust/C++ RAII桥接；升级为persistent world/tile/query owner，不另写替代算法。
3. 保留builtin已验证的预编译graph、空间索引、scratch、route cache、repath轮转与空间避障思想，按需要迁入唯一plugin owner或test oracle。
4. 保留`navigation.main`、`navigation.agent_tick`、typed tick report、DesiredVelocity模式和公平repath预算。
5. 保留dirty/tiled task、selected-surface参数、generation guard、panic containment和last-good publication骨架。
6. 保留dynamic-row projection、entity-row索引和borrowed parse的局部优化，继续硬切到typed incremental component projection。
7. 保留off-mesh started/completed事件与容量概念，改由smart-link/traversal ticket统一管理。
8. 保留overlay frame、owner generation、PIE session/sequence拒旧与demand-driven publication；删除full snapshot复制和固定options。
9. 保留Editor contribution/operation/toolkit基础和V2 progress消费，接入真实document、job、artifact transaction与runtime observation。
10. 保持Navigation为Partial/default-off或明确degraded。G01-G32通过前不得升级成熟度或宣称优于Unreal。

## 5. P0：产品真实性阻断

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| NAV-P0-001 | Open | Runtime Bake Scene/Surface的`prepare`与`apply`固定返回错误，而focused tests把该失败当成当前合同 | 实现pure prepare、bounded build、validated artifact和owner apply；成功/失败测试必须与生产行为一致 |
| NAV-P0-002 | Open | Surfaces、Agents/Areas、NavMesh/Settings和五个drawer等11个业务面大量发布`Space` | 未闭合面默认隐藏或typed unavailable；可见面必须有真实model/binding/validation/empty/error/loading状态 |
| NAV-P0-003 | Open | BakePanel、surface rows、controller、job、artifact publish、scene reference和undo/recovery没有产品owner | 建立Editor NavigationAuthoringGateway和单一operation/artifact transaction，删除测试专用第二提交链 |
| NAV-P0-004 | Open | Toggle/filter/query preview没有handler或binding，overlay固定Default且只消费PIE mirror | 控件驱动同一runtime observation/filter/query ticket，并受document/world/backend generation与预算约束 |
| NAV-P0-005 | Open | Navmesh AI Workbench固定显示tile/agent/query/rebuild状态并返回预写feedback | 默认入口撤销或标Demo/Unavailable；只有真实Navigation document/runtime/job receipt可驱动状态 |

## 6. P1：Package、Catalog、Artifact与Carrier

| ID | 状态 | 当前差距 | 需要重构 |
|---|---|---|---|
| NNAV-P1-001 | Open | ordinary Client仍运行builtin而非selected Recast provider | 由`NavigationActivationPlan`在build/launch前解析唯一provider、carrier、artifact schema和资格receipt |
| NNAV-P1-002 | Open | Editor source Recast与Client builtin语义不等价 | Editor/Client/export运行同一admitted provider与scenario corpus，target差异必须显式 |
| NNAV-P1-003 | Open | manifest声明Server支持，普通`target-server`不启用Navigation | 正式接入headless provider/strip policy，或声明Unsupported并阻断依赖功能 |
| NNAV-P1-004 | Open | core/plugin以同module/service维持两套manager | 硬切单一implementation owner；degraded backend也必须同contract且显式选择 |
| NNAV-P1-005 | Open | 缺失或未链接selection可被catalog跳过 | required selection fail-close并记录requested/linked/admitted/activated reason |
| NNAV-P1-006 | Open | generic Partial无法说明query/bake/crowd/tile-cache真实性 | capability按facet、limits、provider generation和qualification evidence发布 |
| NNAV-P1-007 | Open | NativeDynamic dist只有metadata，无command/event/state/bridge/unload | 实现等价service与quiescence，或明确metadata-only/Unsupported |
| NNAV-P1-008 | Open | Source/LibraryEmbed/NativeDynamic没有同输入同结果矩阵 | 建立carrier conformance、schema/ABI skew、save/restore、fault与unload gate |
| NNAV-P1-009 | Open | activation receipt不含Recast版本、flags、limits和fallback | 输出不可伪造的provider/artifact/backend/generation receipt |
| NNAV-P1-010 | Open | vendor NOTICE/provenance/patch/upgrade证据不完整 | 冻结upstream revision、patch ledger、license、flags、CVE与upgrade corpus |
| NNAV-P1-011 | Open | `NavMeshAsset`仍是raw DTO，不是prepared runtime data | 定义chunked cook artifact，保存per-agent Detour tile/layer blob、semantic/link/stream index |
| NNAV-P1-012 | Open | project loader固定读取`assets/navigation/main.navmesh.toml` | 通过asset registry、stable ID、map/profile引用和dependency graph加载多个nav data |
| NNAV-P1-013 | Open | `output_asset`无writer、transaction和reference update | temp write、bounded validate、atomic publish、reference update、undo/rollback与receipt闭环 |
| NNAV-P1-014 | Open | artifact无magic/checksum/backend ABI/platform/provenance | 增加schema、endianness、digest、ABI/toolchain/source revision、bounds与migration |
| NNAV-P1-015 | Open | agent/profile/area依赖String和裸整数 | 编译stable IDs、schema generation、permission/cost table和unknown policy |
| NNAV-P1-016 | Open | manager无World owner、remove/unload/retire/stale handle | 建立per-World `NavigationWorld`与generation-qualified handles及terminal lifecycle |

## 7. P1：Bake、Native、Query与Runtime

| ID | 状态 | 当前差距 | 需要重构 |
|---|---|---|---|
| NNAV-P1-017 | Open | Render Mesh/Cube被替为1x1顶面，不读真实mesh/LOD/instance | geometry provider消费admitted render/collision artifact、transform generation与material area |
| NNAV-P1-018 | Open | collider只取顶面，Convex退AABB，TriangleMesh/HeightField为空 | physics/terrain提供真实triangles/heightfield/convex cook chunk及精度/budget |
| NNAV-P1-019 | Open | Surface/Area/Modifier以origin/AABB/whole node近似 | 在rasterization按shape/triangle/voxel overlap生成area/exclude/link mask |
| NNAV-P1-020 | Open | bake obstacle按中心相交删除整个source node | 静态modifier进入tile build；动态obstacle进入TileCache/RVO并保留局部bounds |
| NNAV-P1-021 | Open | 空source生成synthetic quad并warning success | 无有效source typed fail；fixture plane只能存在于test namespace |
| NNAV-P1-022 | Open | agent和advanced settings多数只进hash/warning，native用默认值 | 全部可见setting进入validated compiler/Recast config；unsupported在admission拒绝 |
| NNAV-P1-023 | Open | `force_full_rebuild`没有生产语义 | receipt明确full/dirty mode、source generation、reason和实际执行范围 |
| NNAV-P1-024 | Open | tiled bake对每tile重跑全source后合并raw DTO | per-tile/layer gather/raster/build并生成直接`addTile` blob和seam验证 |
| NNAV-P1-025 | Open | TileCache伪造单`0/0/0`layer、零height和极小固定容量 | 持久多tile compressed layers、artifact config、bounded obstacle request和增量publish |
| NNAV-P1-026 | Open | dirty rebuild仍重扫完整World，bounds由caller手工给出 | NavigationOctree/geometry registry/change stream自动产生并合并dirty areas |
| NNAV-P1-027 | Open | async bake持有完整World/plan/result且按tile无界派发 | immutable source snapshot、items/bytes/time/concurrency预算、priority/cancel/deadline |
| NNAV-P1-028 | Open | sync bake和`update_until_current`可在caller完整执行 | build后台或time-sliced；main thread只做有预算commit/publish |
| NNAV-P1-029 | Open | plugin每次query clone asset并重建/销毁Detour owner | per-World持久`dtNavMesh`、bounded query pool/scratch和immutable generation lease |
| NNAV-P1-030 | Open | native`Option`混合failure/unsupported/no-path并可静默切Rust | typed outcome/failure；backend只能由activation policy选择并留下receipt |
| NNAV-P1-031 | Open | 单个off-mesh `cost_override`可让整个native query退fallback | 编译link cost/flags/user ID或在artifact admission明确拒绝 |
| NNAV-P1-032 | Open | corridor/straight path固定512，u16与partial处理不完整 | limit进入capability；支持partial/truncated/continuation/budget和tile refs |
| NNAV-P1-033 | Open | 64 area中16以上折叠到一个Detour flag | 分离area type、poly flags与query permissions并在compiler验证容量 |
| NNAV-P1-034 | Open | query只有同步find/sample/raycast，无filter asset或ticket | typed sync/batch/async request、priority/deadline/cancel/corridor handle |
| NNAV-P1-035 | Open | global Mutex state无World/PIE/session隔离 | 每World独立immutable generations、mutable Crowd/TileCache和scheduler |
| NNAV-P1-036 | Open | asset只能load且隐式选最小handle为default | 显式map/profile选择，支持stream/replace/unload和stale reference |

## 8. P1：Agent、Crowd、Obstacle、Off-Mesh、Editor与Qualification

| ID | 状态 | 当前差距 | 需要重构 |
|---|---|---|---|
| NNAV-P1-037 | Partial | dynamic-row与borrowed projection减少`node_records`热路径，但仍解析JSON，malformed值可`unwrap_or_default`静默归零 | typed ECS change stream、compiled access plan、strict diagnostics和增量projection |
| NNAV-P1-038 | Open | 任意obstacle/off-mesh link会clear Crowd并退legacy | Crowd、TileCache和links在同一generation协作；不支持组合在admission fail-close |
| NNAV-P1-039 | Partial | crowd state开始复用scratch/容量，但仍固定256/radius 8且复杂场景退fallback | workload admission、generation slot、SoA caller scratch和overflow policy |
| NNAV-P1-040 | Open | obstacle stationary/carve参数无完整生产状态机 | 按transform generation实现moving/stationary/carve/pending/remove/re-add |
| NNAV-P1-041 | Open | Navigation可直接写Transform，且无mesh时直走目标 | 只发布movement intent/path outcome，由Character/Physics/Network确认 |
| NNAV-P1-042 | Open | off-mesh由内部固定插值器执行 | smart-link provider、traversal ticket、动画/能力/物理/network握手与恢复 |
| NNAV-P1-043 | Partial | overlay已按demand发布并有generation/mirror拒旧，但仍全量clone triangles/links/paths | generation edge/index cache、viewport/frustum/LOD及items/bytes/time预算 |
| NNAV-P1-044 | Open | 11份ZUI仍有大量无binding业务`Space` | 所有可见资源接真实model/validation/empty/error/loading/permission状态 |
| NNAV-P1-045 | Partial | selected-surface args和V2 progress消费已存在，但controller无产品owner、Runtime Bake固定失败 | 单一Editor gateway接operation、scheduler、artifact transaction、cancel和undo |
| NNAV-P1-046 | Open | source/native/editor/client/server没有共享scenario corpus | 同geometry/artifact/query/agent/obstacle/link场景做跨carrier differential |
| NNAV-P1-047 | Partial | 已有6个ignored release test及scale counters，但多为synthetic局部算法 | 加入real geometry、large tiles、1k/10k agents、bad artifact、OOM/cancel/unload/soak |
| NNAV-P1-048 | Open | ABI、vendor升级、allocator/thread/fault和三平台无完整receipt | 固化layout/compiler/allocator/thread contract、sanitizer和平台矩阵 |

## 9. P2：P1闭环后的竞争能力

| ID | 状态 | 能力 | 工程目标 |
|---|---|---|---|
| NNAV-P2-001 | Open | World Partition与Navigation Invoker | interest驱动tile residency/prefetch/eviction、chunk continuation与server policy |
| NNAV-P2-002 | Open | Hierarchical NavMesh | cluster/portal graph、长距离搜索、局部corridor refinement与动态失效 |
| NNAV-P2-003 | Open | Smart Link生态 | 门/电梯/跳跃/攀爬/传送/Ability条件、预约、拥塞和失败恢复 |
| NNAV-P2-004 | Open | Query Filter资产与策略编译 | 权限、动态cost、危险/掩体/噪声/阵营语义、SIMD batch与可解释trace |
| NNAV-P2-005 | Open | Massive Crowd LOD | 多精度simulation、flow field、lane/group、spatial partition和server determinism |
| NNAV-P2-006 | Open | 多移动域 | vehicle/flying/swimming/climbing/2D及混合domain transition |
| NNAV-P2-007 | Open | Runtime geometry增量编译 | destructible/construction/moving platform/procedural world局部tile rebuild |
| NNAV-P2-008 | Open | Deterministic network/replay | policy version、server path、client prediction、rollback和trace replay |
| NNAV-P2-009 | Open | 多agent多层nav data | shared source cook、layer dedup、cross-profile link和budgeted residency |
| NNAV-P2-010 | Open | GPU辅助bake与分析 | CPU语义正确后加入GPU voxelization/distance/heatmap并保留CPU oracle |
| NNAV-P2-011 | Open | Navigation质量数据库 | property/fuzz/reference differential、optimality/clearance/reachability与bisect |
| NNAV-P2-012 | Open | Live observability与竞争档案 | tile/query/crowd/link timeline、remote capture、heatmap和同负载benchmark archive |

## 10. 历史台账与failure重判

### 10.1 Runtime08D

20项P1中，P1-2、P1-6、P1-8、P1-13、P1-14、P1-17、P1-18为Partial：builtin现有agent tick/compiled graph/scratch/route cache，projection、avoidance、Crowd scratch、AI MoveTo feedback与overlay generation均有真实源码进展。其余13项仍Open，尤其多authority、per-World owner、frame demand、artifact、typed native failure、真实geometry/config/tile、task lifecycle、movement/off-mesh和Editor operation未闭合。5项P2全部Open。

### 10.2 Plugins14与Editor19

Plugins14的48项P1以本报告表为唯一current status：43 Open、5 Partial；12项P2全部Open。Editor19的5项P0、60项P1、12项P2全部Open。存在view/template registration、V2 progress、overlay provider或测试并不能关闭业务`Space`、固定Bake失败、无产品controller和固定Workbench状态。

### 10.3 五份failure

| Failure | 当前重判 | 保持Open的原因 |
|---|---|---|
| bake selection operation arguments | Partial | selected surface payload已正确，但surface row producer和managed产品gate不存在 |
| runtime fallback hotpath | Partial | builtin有compiled graph/spatial/scratch，plugin native仍每query重建且legacy仍O(A^2) |
| world scan deserialize value | Partial | 原compile症状有guard，但runtime仍逐帧dynamic JSON，malformed value会静默default |
| overlay frame publication | Partial | provider/frame source与demand publication存在，但仍全量物化/clone且UI控制断开 |
| editor operation status V2 cutover | Partial | Editor消费V2 progress，但request/result V1、16次spin poll、无cancel/wake且Runtime Bake失败 |

failure文件只能在其原required gates和managed validation通过后关闭；本报告的静态重判不能替代修复记录。

## 11. 参考引擎差异

### 11.1 Unreal

Unreal NavigationSystem以World为owner，使用NavigationOctree与DirtyAreasController管理增量source；invoker/active tiles/world partition控制residency；RecastNavMeshGenerator消费真实tri mesh、convex、heightfield与custom geometry，维护compressed multilayer TileCache、dirty layer bits、time-sliced/async task预算并直接产出Detour tile data；RecastNavMeshDataChunk支持attach/detach；query filter、batch projection、path following、CrowdManager、rendering component和testing actor共享同一nav data lifecycle。Zircon目前只有这些名词的零散DTO或bridge，缺统一owner、artifact和产品闭环。

### 11.2 Godot

Godot用NavigationServer/Map稳定RID owner管理region/link/agent/obstacle，mutable updates与只读map iteration snapshot分离；worker task、RWLock、iteration ID和explicit sync/finish/free定义生命周期；source geometry parse与sync/async bake有明确task状态；2D/3D avoidance有layers/masks/priority；Editor region/link/obstacle工具连接真实server对象。它证明server facade仍需generation、query slot、worker和retirement，而不是global raw-handle Mutex。

### 11.3 Fyrox、Bevy与Unity Graphics边界

Fyrox runtime navmesh规模较轻且仍有octree TODO，不能作为大世界或Crowd上限；但其Editor有真实vertex/edge selection、viewport picking、connect/delete/move command及undo/revert，足以证明最小工程Editor也不能用空`Space`替代。Bevy主仓没有同等级first-party gameplay Navigation，本文只参考其ECS/schedule所有权，不把缺失功能当目标。Unity Graphics也没有Navigation实现，只参考DebugManager/DebugDisplaySettings的runtime-backed panel、widget、reset/refresh和query组织；不从渲染仓反推寻路算法。

## 12. 目标架构与Hard Cutover

```text
zircon_app
  Project/Profile Selection
      -> NavigationActivationPlan + immutable ActivationReceipt

zircon_runtime
  NavigationWorldRegistry
    -> NavigationWorldKey(session, world, replacement_epoch)
    -> NavigationSourceProjection (typed incremental render/physics/terrain)
    -> NavigationBuildScheduler (bounded/cancel/deadline)
    -> NavigationArtifactStore (versioned Detour tile/layer chunks)
    -> NavigationRuntimeGeneration
         persistent dtNavMesh + query pool + TileCache + Crowd
    -> QueryScheduler / PathFollowingContract / MovementIntent
    -> ObservationStream / FrameDemand / Diagnostics

zircon_editor
  NavigationDocument + AuthoringGateway
    -> operation ticket -> Runtime prepare/build/publish
    -> artifact/reference transaction + undo/recovery
    -> same-generation preview/query/overlay
```

硬切规则：删除builtin与plugin legacy作为普通生产authority；删除query-time raw DTO native rebuild和automatic backend fallback；删除fixed TOML product loader；删除synthetic geometry success；删除Navigation直接Transform writeback；删除Editor第二提交链、16次spin poll与固定Workbench反馈。兼容层、re-export或“先保留旧路径兜底”都不能让旧行为继续可达。

## 13. 分层重构计划

### M0：Truth Freeze与RED基线

- 固化五项P0、dual authority、synthetic geometry、per-query rebuild、silent fallback、direct Transform、fixed Workbench与五份failure的可重复RED。
- 建立provider/consumer/asset/operation/owner/deletion matrix；保持capability Partial/default-off。

### M1：唯一Provider、World Owner与Identity

- 实现`NavigationActivationPlan/Receipt`和per-World registry/generation handles。
- ordinary Client/Editor/Server/export只解析一个provider；缺失required capability时fail-close。

### M2：Typed Source Geometry与Dirty Registry

- Render/Physics/Terrain提供真实versioned geometry chunks和change stream。
- 建立NavigationOctree/geometry registry、dirty area合并及source/profile dependency digest。

### M3：Compiler、Cook Artifact与Tile Build

- 全部可见agent/surface/area setting进入validated Recast config。
- per-tile/layer生成Detour/TileCache blobs，完成schema/checksum/provenance/seam与bounded artifact validation。

### M4：Persistent Runtime与Query Scheduler

- generation加载持久`dtNavMesh`，query pool复用scratch/node pool。
- 建立typed sync/batch/async outcome、filter artifact、priority/deadline/cancel与telemetry；禁止silent fallback。

### M5：Agent、Crowd、Obstacle与Movement

- typed incremental agent/obstacle/link projection；Crowd、TileCache和links同generation组合。
- Navigation只发布intent/outcome，接入Character/Physics/Animation/Network确认链和reactive frame demand。

### M6：Smart Link与Path Following

- 将内部插值器硬切为provider-owned traversal ticket、reservation、timeout/cancel/recovery。
- AI MoveTo/脚本统一使用generation request、path-following state和terminal outcome。

### M7：Editor Operation、Artifact与Observation

- 单一AuthoringGateway构造BakePanel/surface rows，接operation wake/progress/cancel/result。
- 完成atomic artifact/reference transaction、undo/recovery、toolkits、inspectors、query preview与budgeted overlay。

### M8：Streaming、Carrier、Failure与Platform

- 完成tile attach/detach、World Partition/invoker基础、world unload/plugin reload/shutdown fence。
- source/library/native与三平台共享scenario，关闭五份failure及ABI/vendor/sanitizer门。

### M9：产品与竞争资格

- Vampire/WOC或专用产品fixture必须从真实scene geometry bake、保存、reload、Client/Server运行到movement outcome。
- 在功能、安全、artifact和场景相同条件下测1/10k agents、百万polygons、streaming/churn、CPU/RSS/P50/P95/P99与soak，再讨论优于Unreal。

## 14. G01-G32 综合资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 | Fail | Client/Editor/Server/source/native没有同schema activation receipt |
| G02 | Fail | builtin、plugin Recast和legacy仍是多个authority |
| G03 | Fail | 缺provider/artifact仍可形成半能力或Ready表象 |
| G04 | Fail | capability仍是generic Partial而非facet/limit/evidence |
| G05 | Fail | product bake仍存在synthetic quad/disc/empty fallback |
| G06 | Fail | TriangleMesh/HeightField与spatial modifier语义未闭合 |
| G07 | Fail | 多数可见setting仍被默认值或warning忽略 |
| G08 | Fail | artifact header/provenance/platform/chunk directory不完整 |
| G09 | Fail | runtime仍从raw DTO重建单体Detour owner |
| G10 | Partial | dirty plan/generation存在，但source仍全World扫描且bounds非change-stream authority |
| G11 | Partial | task pool、shared plan和panic guard存在，缺全局items/bytes/time/cancel/deadline/shutdown预算 |
| G12 | Fail | Runtime Bake operation固定失败，无artifact transaction/recovery |
| G13 | Fail | force-full与dirty/runtime policy没有可靠可观察差异 |
| G14 | Fail | 无per-World query/Crowd/TileCache/streaming双World lifecycle |
| G15 | Partial | framework有typed结果骨架，native/backend仍以`Option`折叠关键状态 |
| G16 | Fail | native failure或单link仍可静默切Rust fallback |
| G17 | Fail | 64 area到16-bit flags仍静默折叠 |
| G18 | Partial | dynamic-row/borrowed projection降低部分成本，仍无typed incremental projection |
| G19 | Fail | obstacle/off-mesh仍会全局clear Crowd并退legacy |
| G20 | Partial | scratch/capacity复用有进展，容量仍硬编码且无generation/overflow合同 |
| G21 | Fail | obstacle stationary/carve状态机未完整执行 |
| G22 | Fail | Navigation仍可直接写Transform |
| G23 | Fail | smart-link无跨Gameplay/Animation/Physics/Network ticket |
| G24 | Fail | 11份ZUI未形成可验收产品交互 |
| G25 | Fail | BakePanel无产品owner且Runtime handler拒绝Bake |
| G26 | Partial | overlay有generation与demand publication，仍全量复制且无UI/filter/budget闭环 |
| G27 | Fail | 示例仍依赖固定raw TOML navmesh |
| G28 | Fail | 无跨carrier/target共享真实scenario corpus |
| G29 | Fail | corrupt/oversize/fault/unload/reload/shutdown无完整terminal receipt |
| G30 | Fail | ABI/vendor/allocator/thread/sanitizer/三平台矩阵不完整 |
| G31 | Fail | 五份failure均未通过原required gates关闭 |
| G32 | Fail | 本轮只完成静态文档门；实现、managed validation与动态证据仍待办 |

## 15. 禁止的临时修补

1. 禁止保留builtin、Recast和legacy三套生产authority并用priority或feature flag掩盖差异。
2. 禁止把native failure/no-path/unsupported继续折叠为`None`后自动调用Rust fallback。
3. 禁止用quad、disc、AABB顶面或空source平面伪造真实scene geometry。
4. 禁止仅把agent/surface setting写入hash或warning却宣称已支持。
5. 禁止把Rust层tile循环、centroid过滤或raw DTO metadata称为Detour tile streaming。
6. 禁止每query重建navmesh/query、每frame全World JSON扫描或全量overlay clone后只做微基准优化。
7. 禁止Navigation直接控制Transform，或在无navmesh时把直走目标当成功降级。
8. 禁止为BakePanel、surface rows、query preview再建第三套test-only controller/backend。
9. 禁止用空`Space`、固定Workbench数据、registration shape test或ignored benchmark升级capability。
10. 禁止在MVP baseline前实现P2高级能力；当前允许的下一步是RED证据、架构硬切和P1基础闭环。

## 16. 本轮完成定义

- 已逐文件检查240个Zircon selected文件，并抽查47个五引擎参考文件；vendor与产品consumer分别冻结。
- 已将Runtime08D、Plugins14、Editor19、五份failure和局部performance进展统一重判。
- 已登记5项P0、48项P1、12项P2、M0-M9和G01-G32；finding保持canonical owner，不重复扩大计数。
- 本轮只修改review和索引，没有实现生产修复，也没有把静态审查写成动态通过。
- 任何实施Session开始前必须重取HEAD/fingerprint、检查failure owner与lease，并从M0/M1的truth和唯一authority开始。
