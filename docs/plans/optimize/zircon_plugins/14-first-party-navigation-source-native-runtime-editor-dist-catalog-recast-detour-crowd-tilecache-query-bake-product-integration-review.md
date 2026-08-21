---
title: First-Party Navigation Source、Native、Runtime、Editor、Dist、Catalog、Recast、Detour、Crowd、TileCache、Query、Bake 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins14
review_date: 2026-08-19
baseline_head: 25e09a23178000f2e783ce2143cf70a8b118d404
baseline_epoch: 333
related_code:
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/runtime/Cargo.toml
  - zircon_plugins/navigation/runtime/src
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src
  - zircon_plugins/navigation/native/native
  - zircon_plugins/navigation/native/vendor/recastnavigation
  - zircon_plugins/navigation/editor/Cargo.toml
  - zircon_plugins/navigation/editor/src
  - zircon_plugins/navigation/editor/agents_areas.zui
  - zircon_plugins/navigation/editor/bake.zui
  - zircon_plugins/navigation/editor/debug_gizmos.zui
  - zircon_plugins/navigation/editor/navigation_settings_asset.zui
  - zircon_plugins/navigation/editor/navmesh_asset.zui
  - zircon_plugins/navigation/editor/surfaces.zui
  - zircon_plugins/navigation/dist/Cargo.toml
  - zircon_plugins/navigation/dist/src
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_runtime/src/navigation
  - zircon_runtime/src/core/framework/navigation
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
  - examples/vampire/assets/navigation/main.navmesh.toml
tests:
  - zircon_plugins/navigation/runtime/src/tests
  - zircon_plugins/navigation/runtime/src/manager/traversal/tests.rs
  - zircon_plugins/navigation/native/src/tests
  - zircon_plugins/navigation/native/tests
  - zircon_plugins/navigation/editor/src/tests
  - zircon_plugins/navigation/editor/src/tests.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
  - zircon_runtime/src/core/framework/navigation/tests.rs
  - zircon_plugins/navigation/dist/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/05/failure-2026-07-15-navigation-bake-selection-operation-arguments.md
  - docs/plans/zircon_plugins/05/failure-2026-07-19-navigation-runtime-fallback-hotpath.md
  - docs/plans/zircon_plugins/05/failure-2026-07-27-navigation-world-scan-deserialize-value.md
  - docs/plans/zircon_plugins/05/failure-2026-07-30-navigation-overlay-frame-publication.md
  - docs/plans/zircon_plugins/05/failure-2026-08-02-navigation-editor-operation-status-v2-cutover.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationDirtyAreasController.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationOctree.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationInvokerComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationData.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationPath.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMesh.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMeshGenerator.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMeshDataChunk.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Navigation/CrowdManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Navigation/PathFollowingComponent.cpp
  - dev/godot/servers/navigation_3d/navigation_server_3d.h
  - dev/godot/modules/navigation_3d/nav_map_3d.h
  - dev/godot/modules/navigation_3d/3d/nav_map_builder_3d.h
  - dev/godot/modules/navigation_3d/3d/nav_mesh_queries_3d.h
  - dev/Fyrox/fyrox-impl/src/scene/navmesh.rs
  - dev/Fyrox/editor/src/interaction/navmesh/mod.rs
  - dev/bevy/crates
  - dev/Graphics/Packages
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 14 · First-Party Navigation Source、Native、Runtime、Editor、Dist、Catalog、Recast、Detour、Crowd、TileCache、Query、Bake 与 Product Integration 工程化差距

## 1. 结论

`zircon_plugins/navigation`并不是只有接口或假返回值。仓内确实编译了Recast、Detour、DetourCrowd与DetourTileCache，C++ bridge能够创建`dtNavMesh`、运行query、维护`dtCrowd`并更新tile cache；Rust侧也已经有typed Navigation组件、过滤器、off-mesh traversal、bake plan、dirty tile重建、overlay frame、operation DTO、generation与169项局部测试。这些基础应保留，不能把本轮结论误读为“重写所有算法”。

真正的问题是这些算法尚未形成同一个工程级产品。普通Client只启用core builtin Navigation，Editor Host才显式链接首方Navigation source plugin，Server虽被manifest声明支持却不启用Navigation；同名`navigation.runtime` service因此在不同target上代表不同语义。source plugin使用Recast/Detour，builtin fallback使用另一套Rust三角网格、查询、avoidance与agent tick。`Dynamic Runtime Session`只按linked package二选一，并没有provider、artifact schema、backend generation与qualification receipt。用户在Editor里看到和验证的路径不能证明Client或Server执行同一后端。

Native路径本身也被临时数据模型削弱。当前`NavMeshAsset`保存raw vertices、indices、polygon DTO、tile bounds与off-mesh link，而不是可直接装载的Detour tile数据。每次find/sample/raycast都会从DTO重新创建`dtNavMesh`和query，结束后立即释放；TileCache也从整个资产重建一个`tx=0/ty=0/layer=0`的全局layer，并硬编码walkable height、radius、climb、simplification与最多4 tiles。所谓tiled bake是对每个bounds重复运行源网格、再把多份DTO合并；没有持久tile blob、streaming chunk、build provenance、平台cook、DDC或并发query pool。

更严重的是烘焙输入并非真实场景几何。Render Mesh和Cube被投影成1x1顶面quad，box collider只取顶面，sphere/capsule/cylinder用12段顶面圆盘，ConvexHull退化成AABB顶面，TriangleMesh与HeightField直接跳过。Area/Modifier按node origin或整node应用，obstacle bake按中心相交删除整个node；空输入还会生成synthetic quad并返回成功。public agent、surface和advanced bake settings多数只进入hash或warning，没有进入`RecastBakeSettings`。`output_asset`只是字符串与报告字段，没有asset writer或原子发布。

Runtime同样没有稳定world authority。manager用共享`Arc<Mutex<NavigationRuntimeState>>`存所有asset与snapshot，查询时克隆完整asset，tick时克隆全部loaded assets；agent system每帧全World扫描和动态JSON解析。任意obstacle或off-mesh link会清空全部native crowd并退回legacy路径，legacy路径进行逐agent寻路和O(A² + A×O) avoidance，然后直接改写Transform。obstacle的stationary/move threshold字段未生效，off-mesh traversal由Navigation内部插值，没有与角色移动、物理、动画、网络authority建立握手。

Editor已有11份ZUI和较完整的Bake panel model，但大多数资源是`Space`业务占位；panel/controller没有产品构造者，debug controls没有event binding，overlay每帧复制全部三角形。更关键的是Runtime operation handler明确让BakeScene/BakeSurface在prepare阶段失败，提示需要pure prepare backend；注册的Bake按钮无法形成真实asset transaction。示例产品则硬编码读取`examples/vampire/assets/navigation/main.navmesh.toml`，内容是一个version 1、单tile、`settings_hash = 0`的可编辑TOML三角集合，并非Editor bake产生的cooked `.znavmesh`。

Navigation本体最高优先级由Runtime08D管理，authoring由Editor19管理，package/catalog/native ABI由Plugins01/06管理，asset/cook由Plugins07管理，通用operation、identity、time与builtin composition由Runtime02/22/24/42等父owner管理。本篇不重复累计父报告P0，登记 **0项新增P0、48项P1、12项P2**；本篇唯一拥有Navigation单包从manifest、source/native runtime、core fallback、Editor、dist、catalog、App target、product asset到参考后端的纵向交付合同。

## 2. 审查边界、规模与currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes | 冻结事实 |
|---|---:|---|
| 三个canonical root合计 | 217 / 52,108 / 1,671,981 | `zircon_plugins/navigation`、`zircon_runtime/src/navigation`与`zircon_runtime/src/core/framework/navigation`；共169项`#[test]` |
| Zircon自有代码 | 166 / 24,480 / 834,631 | core fallback 15、framework 19、package 1、native-owned 40、dist 2、editor 33、runtime 56 |
| vendored RecastNavigation | 51 / 27,628 / 837,350 | 只包含Recast、Detour、DetourCrowd、DetourTileCache；没有RecastDemo/sample源码 |
| core fallback | 15 / 3,351 / 116,220 | Rust baked mesh、projection cache、query scratch、repath budget、avoidance grid与agent tick |
| framework contracts | 19 / 1,645 / 50,101 | asset、agent、area、filter、surface、modifier、obstacle、off-mesh link与runtime service |
| native-owned | 40 / 7,404 / 249,997 | Rust FFI、bake/query/crowd/tile-cache wrapper与C++ bridge；不含51个vendored文件 |
| plugin runtime / editor / dist | 56 / 8,127 / 279,282；33 / 3,780 / 132,797；2 / 115 / 4,128 | runtime manager/system/bake/traversal；11份ZUI与registration/model；Native ABI v3 metadata shell |
| package fingerprint | `3f9544fc4771cb9da09706c2575ddf55f465c3fbb4d533baa53271b3f038ba07` | 217个tracked path排序，以小写path、空格与file SHA-256组成LF串，无末尾LF后重算SHA-256 |

源revision为`25e09a23178000f2e783ce2143cf70a8b118d404`，coordinator baseline epoch为333。三个canonical source root在冻结时没有tracked working-tree差异；App、catalog、共享Runtime与计划文档存在其他会话或用户改动，所以本文保持`source_recheck_required: true`。实施前必须在同一BuildSet重算source/native/fallback、App target features、runtime/editor catalog、builtin row、示例asset和五份开放failure。

### 2.2 测试库存不等于产品资格

169项test attribute分布为plugin runtime 66、native 37、editor 29、core fallback 26、framework 9、dist 2。测试覆盖asset migration、path/sample/raycast、crowd/tile cache、bake/tiled bake、off-mesh traversal、overlay、operation DTO、registration和局部fallback预算。这些回归对保留现有底座有价值。

但仓内没有Criterion、bench、property、fuzz、Loom、Miri或soak入口。native tests使用小型手造mesh，Editor tests主要驱动isolated model/controller，dist只检查descriptor；没有普通Client/Editor/Server provider parity、真实render/physics/terrain geometry、Editor bake落盘再重启、Detour tile artifact roundtrip、world partition streaming、native fault injection、DLL unload、跨平台ABI、多人网络authority、1k agents或长时间动态障碍资格。本轮只做E3静态审查，没有运行Cargo、C++ toolchain、App、Editor、NativeDynamic或性能测试。

### 2.3 五份开放failure保持open

| Failure | 当前source drift | 本轮处理 |
|---|---|---|
| bake selection operation arguments | Bake ZUI现已有payload binding，但没有产品producer/controller装配 | 保持open；不能用resource文本代替真实提交与artifact结果 |
| runtime fallback hotpath | core fallback已有projection cache、spatial index与预算改进，plugin legacy仍全扫描、重复寻路和O(A² + A×O) | 保持open；需同workload产品profiling与硬切验证 |
| world scan deserialize value | 当前source已直接调用`NavMeshAgentDescriptor::deserialize(value)`，旧`.as_ref()`症状局部漂移 | 保持open；没有受管Cargo gate与产品回归，不能静态关闭 |
| overlay frame publication | source已发布frame/provider，但全量复制、产品binding与受管验证仍缺 | 保持open；publication存在不等于可用debug产品 |
| editor operation status v2 cutover | production command已使用V2 progress，测试与端到端operation仍未完成资格 | 保持open；需按原required gates关闭 |

## 3. 当前真实产品链与断点

~~~text
ordinary zircon_app Client
  -> enables zircon_runtime/navigation contracts
  -> does not link first-party Navigation runtime plugin
  -> Dynamic Runtime Session installs BuiltinNavigationModule
  -> Rust fallback mesh/query/agent behavior

zircon_app Editor Host
  -> explicitly links Navigation runtime + editor source plugins
  -> session skips builtin when linked package is selected
  -> plugin manager + RecastBackend + optional native bridges
  -> Editor surfaces exist, but Bake operation prepare always rejects

zircon_app Server
  -> package manifest says Navigation supports Server
  -> target-server does not enable navigation contracts/provider
  -> no equivalent product path

Editor/source bake request
  -> isolated BakePanel / controller DTO
  -X product producer and durable asset transaction absent
  -X Runtime operation handler rejects BakeScene/BakeSurface

example product
  -> hard-coded assets/navigation/main.navmesh.toml
  -> raw triangles / one logical tile / settings_hash=0
  -> manager clones asset
  -> each native query rebuilds dtNavMesh/query then destroys it

NativeDynamic dist
  -> ABI v3 descriptor + registration manifest
  -X no navigation commands/events/state/bridge/lifecycle behavior
~~~

这条链说明“源码中存在Recast调用”“Editor中存在Bake按钮”“manifest声明NativeDynamic/Server”和“普通产品执行同一可烘焙、可部署、可诊断Navigation系统”是四件不同的事。当前只有第一件成立。

## 4. 可保留的底座

1. framework已有相对完整的Navigation组件和查询DTO，不应退回匿名JSON或裸C ABI结构。
2. vendored Recast/Detour/Crowd/TileCache是真实上游算法实现，C++ bridge已有最小ownership和drop路径。
3. core fallback的`BakedNavMesh` spatial index、projection cache、scratch复用、repath budget和avoidance grid可转为reference/backend qualification用例。
4. plugin runtime已有generation、tiled plan、dirty tile范围、pending bake task、off-mesh selection与overlay frame概念，可作为新scheduler与artifact receipt的输入。
5. Editor Bake panel已有surface row、progress、diagnostic、cancel/retry模型，适合接入真实operation，而不是另造第二个面板。
6. 169项局部测试可以迁移为单一backend contract suite、artifact roundtrip与source/native differential基础。

保留这些底座不等于保留当前owner边界。DTO重建navmesh、全局manager、synthetic geometry、silent fallback、直接Transform写回和metadata-only NativeDynamic都必须硬切退出。

## 5. 参考实现给出的适用约束

### 5.1 Recast/Detour是算法内核，不是完整产品架构

仓内vendored源码提供Recast voxel/raster/poly mesh生成、Detour navmesh/query、DetourCrowd和TileCache，但没有RecastDemo/sample application。它证明Zircon可以复用成熟算法，不证明geometry extraction、artifact、world ownership、streaming、scheduler、Editor UX、network authority或product qualification已经完成。上游固定limits与filter model也必须由Zircon在artifact admission和capability truth中显式暴露，不能在bridge内静默折叠。

### 5.2 Unreal约束工程级tile lifecycle与移动authority

Unreal的`FNavigationDirtyAreasController`累积带flags、source element和debug reason的有效bounds，支持build lock、oversized warning、update frequency与World Partition dynamic mode。`NavigationData`区分Static、DynamicModifiersOnly与Dynamic generation，并让path持有filter、invalidation与repath状态。`FRecastTileGenerator`保存dirty layer mask、compressed layer、geometry cache，并把gather/raster/filter/build拆为可续跑的time-sliced状态；Navigation Invoker、data chunk、PathFollowing与CrowdManager把large-world residency、path lifecycle和movement authority放在NavMesh算法之外。

Zircon不能只增加“dirty=true”或“async=true”布尔位。必须建立可追踪dirty source、tile/layer identity、bounded queue、time budget、generation publish、path/corridor lifetime与角色移动结果协议。

### 5.3 Godot约束稳定server boundary与iteration snapshot

Godot `NavigationServer3D`用RID分别创建map、region、link、agent与obstacle，map/region暴露iteration id和async iteration策略，query使用typed parameters/result并可callback。`NavMap3D`维护同步/异步dirty request、RWLock保护的iteration slots、worker task、query slots、performance counters以及显式add/remove API。它表明即使不采用Unreal的actor体系，Navigation也应有独立server、稳定owner ID、immutable iteration和增量同步，而不是在每个manager call或frame tick扫描整个World。

### 5.4 Fyrox约束最小可用Editor也必须落到真实对象与命令

Fyrox的`NavigationalMesh`是Scene Graph node，内部Navmesh可通过`Arc<RwLock<_>>`用于off-thread query；Editor有Navmesh selection、vertex/edge picking、move gizmo、edge duplication、connect/delete与`CommandGroup`撤销。它的功能规模小于Unreal，但产品链是真实的：用户操作同一scene object，命令修改真实mesh，并进入undo。Zircon当前11份ZUI和isolated BakePanel仍未达到这个更低但完整的基线。

### 5.5 Bevy与Unity Graphics的适用边界

本地Bevy `crates`没有Navigation/NavMesh领域crate，不能把ECS schedule或generic asset pattern冒充Navigation语义参考。本地`dev/Graphics/Packages`是Unity Graphics/render pipeline源码，不含Unity AI Navigation package，也不能用于证明bake、agent或obstacle设计。两者只可约束通用ECS、render debug和artifact integration；Navigation语义以vendored Recast、Unreal、Godot与Fyrox为主。

## 6. P0归属说明

本篇新增P0为0。以下最高优先级阻断已经有canonical owner，保持原计数：

| 阻断 | Canonical owner |
|---|---|
| product Bake operation被Runtime handler明确拒绝 | Editor19 / Runtime02 |
| 11份Navigation ZUI大面积业务`Space`，可见但不可操作 | Editor19 |
| Bake job、artifact publish、asset reference与undo/recovery不闭合 | Editor19 / Plugins07 |
| overlay/filter/query debug没有真实产品控制器 | Editor19 / Runtime08D |
| builtin fallback和source plugin产品语义分裂 | Runtime08D / Runtime42 / Plugins06 / App01 |
| native package admission、ABI、carrier lifecycle与source/native parity | Plugins01 / Plugins06 / Runtime Interface owners |

本篇P1只描述这些P0下面Navigation单包必须交付的具体纵向合同，不复制P0所有权。

## 7. P1差距：Package、Catalog、Artifact 与 Carrier（16项）

| ID | 当前事实 | 必须重构为 |
|---|---|---|
| NNAV-P1-001 | ordinary Client启用Navigation contract却不链接Recast plugin，实际运行builtin fallback | `NavigationActivationPlan`在build/launch前解析唯一provider，并给出backend、carrier、artifact schema、generation与qualification receipt |
| NNAV-P1-002 | Editor Host运行source Recast，Client运行Rust fallback，同一project预览与发布语义不等价 | Editor/Client/source export必须运行同一scenario corpus与同一admitted provider；差异只能是显式target policy |
| NNAV-P1-003 | manifest声明Server支持，`target-server`却不启用Navigation | Server要么进入正式headless provider/strip policy，要么manifest与capability明确Unsupported并阻止依赖功能启动 |
| NNAV-P1-004 | core/plugin都注册`navigation.runtime` module/service，但维护两套实现 | 硬切到单一implementation owner；fallback只能是同contract、显式degraded backend，不得复制manager/system |
| NNAV-P1-005 | catalog对缺失或未链接selection可静默跳过 | required Navigation selection必须fail-close，记录resolved package/carrier/provider和missing reason |
| NNAV-P1-006 | capability只表达generic partial/available，无法说明query/bake/crowd/tile-cache真实性 | capability拆成可验证facet，并绑定provider instance、artifact version、limits、platform与qualification evidence |
| NNAV-P1-007 | NativeDynamic dist是stateless metadata shell，command/event为空且无state/lifecycle/bridge | 要么实现同语义navigation service与unload quiescence，要么明确metadata-only/Unsupported，不能宣称行为parity |
| NNAV-P1-008 | Source、LibraryEmbed、NativeDynamic均列为package form，但没有同输入同结果矩阵 | 建立carrier conformance suite、schema/ABI skew、save/restore、fault、unload与differential gate |
| NNAV-P1-009 | activation receipt不记录Recast版本、build flags、limit、backend fallback或generation | 输出不可伪造的provider/artifact/backend receipt，并纳入diagnostics与crash evidence |
| NNAV-P1-010 | vendored Recast无明确upstream commit、patch ledger、license/source archive与upgrade test | 冻结vendor provenance、patch set、compiler flags、license、CVE/upgrade policy和known-limit corpus |
| NNAV-P1-011 | `NavMeshAsset`是raw vertex/index/polygon DTO，不是runtime-prepared nav data | 定义chunked `NavigationCookArtifact`，保存per-agent Detour tile/layer blobs、semantic table、off-mesh records与streaming index |
| NNAV-P1-012 | 示例和project loader依赖固定`assets/navigation/main.navmesh.toml` | 通过project asset registry、stable asset ID、world/map/profile引用和dependency graph加载任意多个nav data |
| NNAV-P1-013 | `output_asset`只在settings/report里流转，没有writer、transaction或scene reference update | Bake operation必须prepare、write temp、validate、atomic publish、update reference、undo/rollback并返回artifact receipt |
| NNAV-P1-014 | asset只有版本、raw arrays与settings hash，无magic/checksum/backend/platform/provenance | header加入schema/endianness/checksum/content digest/backend ABI/toolchain/source revision/agent config/chunk bounds与migration |
| NNAV-P1-015 | `agent_type`、profile与部分area语义依赖字符串和裸整数 | 编译stable agent/profile/area IDs，记录schema generation、permissions、cost table与unknown-ID policy |
| NNAV-P1-016 | manager只有load/query/stats，没有map/world owner、remove、unload、generation或stale handle防御 | 建立per-world `NavigationWorld`与generation-qualified map/navdata handles，支持add/remove/replace/retire/stream与terminal receipt |

## 8. P1差距：Bake、Native、Query 与 Runtime（20项）

| ID | 当前事实 | 必须重构为 |
|---|---|---|
| NNAV-P1-017 | Render Mesh/Cube被收集为1x1顶面quad，没有读取真实mesh asset、LOD或instance geometry | 建立geometry provider contract，消费admitted render collision/source mesh、transform generation、material area与dependency digest |
| NNAV-P1-018 | collider只近似顶面，ConvexHull退化AABB，TriangleMesh/HeightField跳过 | physics/terrain owner提供真实triangles/heightfield或预烘焙geometry chunk，并声明精度、winding、scale与budget |
| NNAV-P1-019 | Surface/Area/Modifier按node origin、AABB或整个node应用，不能表达triangle/voxel重叠 | 在geometry rasterization阶段按volume/shape/triangle overlap生成area、exclude与link-generation mask |
| NNAV-P1-020 | bake obstacle以中心相交删除整个source node | 静态modifier与动态obstacle分离；前者进入voxel/tile build，后者进入TileCache/RVO且保留局部bounds |
| NNAV-P1-021 | 空几何时生成synthetic surface quad并返回warning/success | 无有效source必须typed fail/empty result；fixture helper只能在test namespace且不得进入product artifact |
| NNAV-P1-022 | agent radius、height、profile及advanced settings多数只进入hash/warning，native使用`RecastBakeSettings::default()` | 所有可见setting进入validated compiler input与Recast config；未支持字段在admission时拒绝，不能静默忽略 |
| NNAV-P1-023 | `force_full_rebuild`只存在于UI/DTO/test，没有改变production rebuild | 命令必须明确选择full/dirty模式、source snapshot generation与reason，并在receipt报告实际执行范围 |
| NNAV-P1-024 | tiled bake对每个bounds重复运行全source后合并DTO，不产出持久Detour tile | 每个tile/layer独立gather/raster/build并生成可直接`addTile`的blob、dependency digest与neighbor seam validation |
| NNAV-P1-025 | TileCache从全asset重建一个`0/0/0`layer，hardcode walkability并限制4 tiles | 持久加载多tile compressed layers，配置来自artifact，支持bounded obstacle requests、incremental update与tile publish |
| NNAV-P1-026 | dirty rebuild仍重扫全World并重新prepare，dirty bounds由caller手工给出 | 建立NavigationOctree/geometry registry与DirtyAreasController，按source change自动合并bounds、flags、reason和affected profiles |
| NNAV-P1-027 | async bake克隆整个World/plan/result，每tile一个task，没有Navigation队列容量、优先级或取消 | immutable source snapshot + bounded scheduler + work stealing/priority/deadline/cancel + generation-aware early abort |
| NNAV-P1-028 | sync bake与`update_until_current`可以在caller线程完整跑完 | 全量和动态build都必须time-sliced或后台执行，主线程只做有预算的commit/publish |
| NNAV-P1-029 | 每个find/sample/raycast克隆asset并重新创建/销毁`dtNavMeshQuery` | per-world加载持久`dtNavMesh`，使用bounded query pool/scratch，query只持immutable generation lease |
| NNAV-P1-030 | native返回`Option`，unsupported、allocation/ABI failure、invalid asset、no path与partial path可静默切Rust | 使用typed result/disposition；backend切换只由activation policy决定，并记录原因、语义与qualification |
| NNAV-P1-031 | 任一off-mesh link带`cost_override`时整个native query/TileCache返回None | 将link cost/flags/user ID编译到Detour data或显式拒绝该artifact，不能让单link改变全局backend |
| NNAV-P1-032 | bridge固定512 corridor/straight points，u16 vertex限制和partial corridor处理不完整 | limit进入capability/artifact admission，结果支持partial/truncated状态、continuation/budget与大场景tile refs |
| NNAV-P1-033 | 64个area ID映射到Detour flags时16以上折叠到同一bit | 分离area type、poly flags和query permissions；compiler验证映射容量并为超限提供明确错误/多filter策略 |
| NNAV-P1-034 | public query只有同步单次find/sample/raycast，无batch/async/cancel/corridor handle | typed query request/result、filter asset、priority/deadline/cancel、batch、corridor/path handle与generation invalidation |
| NNAV-P1-035 | manager state是共享Mutex，asset/snapshot/tiled plan没有World/PIE/session隔离 | 每World实例拥有immutable nav generations、mutable obstacle/crowd state与独立scheduler；支持双World和PIE并行 |
| NNAV-P1-036 | asset只能load，不能remove/unload/stream，最小handle被隐式选为default | map/profile选择必须显式；stream-in/out、replace、world unload与stale reference全部有确定lifecycle |

## 9. P1差距：Agent、Crowd、Obstacle、Off-Mesh、Editor 与 Qualification（12项）

| ID | 当前事实 | 必须重构为 |
|---|---|---|
| NNAV-P1-037 | plugin agent/obstacle系统每帧`world.node_records()`全扫描并动态JSON解析 | typed ECS query/change stream、component access plan与incremental projection；steady frame不解析JSON或重建全量vector |
| NNAV-P1-038 | 场景存在任意obstacle、obstacle world或off-mesh link就清空所有crowd并退legacy | Crowd、TileCache、links在同一nav generation上协作；不支持组合必须在activation/artifact admission时fail-close |
| NNAV-P1-039 | native crowd固定最多256 agents、radius 8，`read_states`每帧按capacity分配Vec | capacity由workload/admission配置，slot具generation，state通过caller scratch/batched SoA返回并有overflow policy |
| NNAV-P1-040 | obstacle `move_threshold`、`time_to_stationary`、`carve_only_stationary`没有production语义 | 维护observed transform generation与stationary state machine，按策略选择RVO、TileCache carve、pending与remove |
| NNAV-P1-041 | agent系统直接写Transform或DesiredVelocity，Transform绕过角色、物理和网络authority | Navigation发布movement intent/path outcome；Character Movement/Physics/Network owner消费并回报accepted/progress/failure |
| NNAV-P1-042 | off-mesh traversal由Navigation内部位置插值，没有动画、能力、物理、取消或失败握手 | smart-link provider + traversal ticket/state machine + gameplay/animation/physics/network callbacks与timeout/rollback |
| NNAV-P1-043 | runtime/editor overlay每帧物化并复制全部triangles/links，每三角三条线，无预算/LOD | nav generation维护debug edge/index cache；overlay按viewport、selection、frustum、LOD、items/bytes/time budget增量投影 |
| NNAV-P1-044 | 11份ZUI中surfaces/settings/agent/area/asset/5个drawer/debug viewport主要是`Space` | 每个可见资源挂真实model、binding、validation、empty/error/loading/permission state与可验收交互 |
| NNAV-P1-045 | BakePanel/controller只在测试中组装，Runtime Bake operation明确不能成功 | Editor19的document/operation owner构造真实controller，连接source snapshot、scheduler、artifact transaction、progress/cancel与undo |
| NNAV-P1-046 | source/native/editor/client/server没有共享scenario corpus或result schema | 同scene/source/artifact/query/agent/obstacle/link scenario跨carrier、target、restart与fault做differential qualification |
| NNAV-P1-047 | 测试以手造小mesh、mock World和registration shape为主，没有真实性能/失败/规模证据 | 加入real geometry、cook/reload、large tile set、dynamic churn、1k agents、long path、bad artifact、OOM/cancel/unload与soak |
| NNAV-P1-048 | C++ ABI、vendored升级、allocator/thread affinity、panic/fault与跨平台构建没有完整receipt | 记录layout/size/alignment/compiler/runtime flags、allocator ownership、thread contract、sanitizer与Windows/Linux/macOS matrix |

## 10. P2差距：在P1闭环后用于超过现有引擎（12项）

| ID | 能力 | 工程目标 |
|---|---|---|
| NNAV-P2-001 | World Partition与Navigation Invoker | 基于玩家/AI interest的tile residency、prefetch、eviction、跨chunk path continuation与server policy |
| NNAV-P2-002 | Hierarchical NavMesh | cluster/portal graph、长距离分层搜索、局部corridor refinement与动态失效 |
| NNAV-P2-003 | Smart Link生态 | 门、电梯、跳跃、攀爬、传送与Gameplay Ability provider，支持条件、成本、预约、拥塞和失败恢复 |
| NNAV-P2-004 | Query Filter资产与策略编译 | 多agent权限、动态cost layer、危险/掩体/噪声/阵营语义、SIMD batch evaluation与可解释trace |
| NNAV-P2-005 | Massive Crowd LOD | high/medium/low fidelity simulation、flow field、lane/group behavior、spatial partition与server determinism |
| NNAV-P2-006 | 多移动域 | vehicle、flying、swimming、climbing、2D及混合domain transition，共享stable path/traversal合同 |
| NNAV-P2-007 | Runtime geometry增量编译 | destructible、construction、moving platform与procedural world的局部voxel/tile rebuild及last-good publication |
| NNAV-P2-008 | Deterministic network/replay | query/build policy version、server authoritative path、client prediction、rollback和trace replay |
| NNAV-P2-009 | 多agent多层nav data | 不同radius/height/slope/ability的shared source cook、layer dedup、cross-profile link与budgeted residency |
| NNAV-P2-010 | GPU辅助bake与分析 | 在CPU artifact语义正确后引入GPU voxelization/distance field/quality heatmap，并保留deterministic CPU oracle |
| NNAV-P2-011 | Navigation质量数据库 | 随机场景/property/fuzz、reference differential、path optimality/clearance/reachability指标与回归bisect |
| NNAV-P2-012 | Live Navigation observability | tile/query/crowd/link timeline、remote capture、decision trace、hotspot flame/heatmap与同workload benchmark archive |

这些能力必须建立在唯一provider、真实geometry、prepared artifact、per-world generation、稳定movement authority和产品Editor闭环上。新增空DTO、feature flag、固定fixture或没有consumer的GPU descriptor不计为P2进度。

## 11. 目标架构与Owner收敛

~~~text
NavigationSourceRegistry per World
  RenderGeometryProvider / PhysicsGeometryProvider / TerrainProvider
  Surface / Area / Modifier / Link / Obstacle change streams
        |
        v
DirtyAreasController + NavigationCompiler
  validated agent/profile configs
  bounded tile/layer build DAG
  dependency digest + provenance + diagnostics
        |
        v
NavigationCookArtifact
  header/schema/backend ABI/source revision/checksum
  per-agent tile directory
  Detour navmesh tile blobs + compressed TileCache layers
  semantic area/flag table + smart-link records + streaming chunks
        |
        v
NavigationActivationPlan / Receipt
  target + carrier + provider + backend + artifact compatibility
        |
        v
NavigationWorld per World/PIE generation
  immutable NavData generations + persistent dtNavMesh
  bounded query pool / query scheduler
  mutable TileCache + Crowd + obstacle/link state
        |
        +-> MovementIntent / PathOutcome -> Character/Physics/Network
        +-> Editor/Runtime debug projection with viewport budgets
        +-> Trace/metrics/fault/retirement receipts
~~~

| Owner | 唯一职责 | 禁止继续承担 |
|---|---|---|
| Runtime08D | neutral Navigation contracts、per-world runtime、query/path/crowd/obstacle/link lifecycle | Editor widget、package selection、asset file transaction、第二套fallback manager |
| Editor19 | settings/surface/area/link authoring、Bake document/job UX、artifact transaction、query/debug tools | 私有mock bake、固定demo overlay或绕过operation service |
| Plugins14 | Navigation package source/native/editor/dist/catalog/App纵向closure与carrier parity | 复制Runtime/Editor父finding或在bridge内静默改语义 |
| Plugins07 / Runtime asset owners | source snapshot、dependency graph、cook/DDC、artifact publish/load/migration | frame query、crowd step与Editor view state |
| Plugins01/06 + Runtime42 + App | provider/carrier/profile selection、activation admission与effective capability | domain bake/query算法 |
| Character/Physics/Network owners | movement authority、collision-integrated locomotion、prediction/replication与outcome | 由Navigation直接写Transform代替自身状态机 |
| Tooling/Qualification owners | compiler/native matrix、sanitizer、benchmark、crash/evidence archive | 用path存在或test count宣称产品完成 |

## 12. 分层重构里程碑

### M0 · Truth Freeze与失败基线

- 重算217文件fingerprint、App target/provider矩阵、示例asset与五份failure source drift；
- 冻结新增synthetic geometry、silent fallback、direct Transform write和metadata-only Ready；
- 建立ordinary Client、Editor Host、Server、Source Export、NativeDynamic effective Navigation receipt。

### M1 · 唯一Provider与per-world owner

- 将core fallback与plugin manager硬切到一个implementation owner和backend interface；
- 引入`NavigationWorldId/NavMapHandle/NavDataGeneration`与add/remove/replace/retire；
- catalog/App/profile在启动前选择唯一provider，missing required fail-close。

### M2 · 真实Source Geometry

- 接入render mesh、physics collider、terrain/heightfield与instance geometry provider；
- Surface/Area/Modifier/Obstacle/Link改为change stream和空间重叠语义；
- 建立geometry snapshot、dependency digest、budget、finite/index/winding/scale admission。

### M3 · Cook Artifact与Tile Build

- 编译per-agent Recast config、真实tile/layer与Detour blobs；
- 建立chunk header、checksum、provenance、platform/backend ABI、streaming index和migration；
- Editor Bake通过operation service原子发布asset并更新scene/project reference。

### M4 · Persistent Runtime与Query

- 每NavData generation持久装载`dtNavMesh`与TileCache layers；
- 建立bounded query pool、async/batch/cancel/deadline、typed partial/error result；
- 禁止per-query asset clone/rebuild与runtime算法静默切换。

### M5 · Agent、Crowd与Movement Authority

- typed ECS change projection替代World scan/JSON；
- Crowd、TileCache obstacle与off-mesh link在同一generation运行；
- Navigation只发布intent/outcome，Character/Physics/Network执行并回报状态。

### M6 · Editor产品闭环

- 11份ZUI绑定真实document/model/controller和所有状态；
- create/edit/bake/cancel/error/retry/undo/save/reopen/query/debug形成同一artifact/runtime闭环；
- overlay按viewport与budget读取runtime/editor generation，不复制全量mesh。

### M7 · Carrier、Failure与Compatibility

- Source/LibraryEmbed/NativeDynamic运行同一contract suite；
- world unload、plugin reload、bad artifact、worker/native fault、cancel、OOM与shutdown有terminal receipt；
- 完成vendor provenance、ABI matrix、sanitizer和跨平台构建。

### M8 · Large World与竞争性资格

- 实施invoker/streaming、hierarchical path、mass crowd LOD、smart links与deterministic server/replay；
- 建立真实大场景、动态障碍、1k agents、长时间soak与Editor/runtime differential workload；
- 只有correctness、quality、failure、memory、CPU/frame latency与artifact完整性同时达标，才允许与Unreal同口径比较。

## 13. 资格门

| Gate | 验收内容 |
|---|---|
| G01 | ordinary Client、Editor Host、Server、Source Export与NativeDynamic均输出同schema `NavigationActivationReceipt` |
| G02 | 每World/generation最多一个Navigation provider；core/plugin不再维护两套manager/system实现 |
| G03 | required selection缺provider/artifact/backend时fail-close，不发布Available/Ready |
| G04 | capability逐facet声明query/bake/crowd/tile-cache/off-mesh/runtime-generation及limits/evidence |
| G05 | 真实render/physics/terrain geometry进入compiler；product代码不存在synthetic quad成功路径 |
| G06 | Surface/Area/Modifier按空间/triangle/voxel语义生效，TriangleMesh/HeightField有正式provider |
| G07 | 每个可见agent/bake setting进入validated Recast config；不支持字段在admission时拒绝 |
| G08 | artifact包含magic/schema/checksum/provenance/backend ABI/platform/source digest与per-agent tile directory |
| G09 | Detour navmesh tile与TileCache layer可直接装载，不从raw DTO在query时重建 |
| G10 | dirty area来自change stream并含bounds/flags/source/reason/profile，支持合并、锁与oversize诊断 |
| G11 | bake scheduler有items/bytes/time/concurrency预算、priority/deadline/cancel与generation early abort |
| G12 | Bake operation完成prepare/build/validate/atomic publish/reference update/undo/rollback/restart recovery |
| G13 | `force_full_rebuild`、dirty rebuild与runtime dynamic policy具有可观察且不同的执行语义 |
| G14 | per-world nav generation、query pool、TileCache、Crowd和streaming lifecycle通过双World/PIE测试 |
| G15 | query返回typed success/partial/truncated/no-path/invalid/stale/unsupported/fault，不使用`Option`折叠 |
| G16 | backend不得因单个link/filter/native fault静默切换；任何degraded选择由activation policy记录 |
| G17 | area/flag/filter容量经过compiler验证，64 area语义不会静默折叠为16 flags |
| G18 | agent/obstacle/link steady frame使用typed incremental projection，无全World JSON扫描 |
| G19 | Crowd与动态obstacle/off-mesh link组合不触发全局legacy fallback或crowd clear |
| G20 | crowd capacity、scratch、overflow与agent slot generation明确，steady tick无按capacity Vec分配 |
| G21 | obstacle stationary/carve policy由状态机实现并有移动、停止、删除、re-add与tile update测试 |
| G22 | Navigation不直接写Transform；movement intent/outcome接入Character、Physics和Network authority |
| G23 | smart-link traversal有ticket、provider callback、动画/能力/物理握手、timeout/cancel/failure recovery |
| G24 | 11份ZUI全部具真实binding、validation、loading/empty/error/permission与input/render visual evidence |
| G25 | BakePanel由产品构造并消费真实operation progress/cancel/result，Runtime handler不再拒绝Bake |
| G26 | overlay使用generation cache、unique edges、viewport/frustum/LOD/items/bytes/time预算并受UI控制 |
| G27 | 示例和新项目通过asset registry引用bake产物，不硬编码单一TOML三角mesh |
| G28 | source/native/editor/client/server运行同一geometry/artifact/query/crowd/link scenario corpus |
| G29 | bad/corrupt/old/oversized artifact、worker/native fault、world unload、plugin reload与shutdown有terminal receipt |
| G30 | C++ ABI layout、allocator ownership、thread affinity、vendor provenance、sanitizer与三平台matrix通过 |
| G31 | 五份开放Navigation failure分别通过原required gates关闭，不能由source grep或本报告替代 |
| G32 | `git diff --check`、frontmatter/path、finding唯一性、fingerprint、索引/coverage与plan-output audit通过 |

## 14. 明确禁止的临时修复

1. 不在core fallback和plugin runtime之间继续复制manager、query或agent代码；迁移后删除旧owner。
2. 不把Recast函数调用存在、169项局部测试或vendor行数当作产品完成度。
3. 不用synthetic quad、top-face collider、AABB hull或跳过HeightField来让Bake返回成功。
4. 不让未支持setting只进入hash、warning或diagnostic；可见即必须生效或fail-close。
5. 不扩大512 path、256 crowd、4 tile等固定数字掩盖无limit contract和无capacity admission。
6. 不用`Option`、空path或Rust fallback吞掉native ABI、allocation、unsupported link或invalid artifact错误。
7. 不在query hot path克隆完整asset、重建`dtNavMesh`或为debug再次执行find path。
8. 不以全World扫描、动态JSON解析或更大的Vec cache代替typed change projection。
9. 不由Navigation直接改写Transform来绕过Character Movement、Physics或Network owner。
10. 不用`Space`、静态表格、按钮descriptor、固定progress或test-only controller冒充Editor功能。
11. 不把NativeDynamic metadata shell、空commands/events或registration snapshot称为native Navigation支持。
12. 不关闭五份failure，除非各自原required gate在同BuildSet、同product path下通过。
13. 不从Bevy generic ECS或Unity Graphics render pipeline外推不存在的Navigation参考能力。
14. 不在同场景、同几何、同agent、同硬件、同平台、同采样与同失败条件前宣称超过Unreal。

## 15. 状态与产出记录

| 项目 | 状态 | 证据 |
|---|---|---|
| 物理扫描 | review_complete | 217 tracked files / 52,108 lines / 1,671,981 bytes；Zircon-owned与vendored Recast分开核算 |
| 测试库存 | review_complete | 169项test attribute；0 property/fuzz/benchmark/soak入口；不是本轮通过数 |
| 产品纵向链 | review_complete | Client fallback、Editor source Recast、Server缺席、NativeDynamic metadata-only、Vampire TOML资产 |
| Native算法审查 | review_complete | Recast/Detour/Crowd/TileCache真实存在；DTO rebuild、单layer cache、固定limits与silent fallback未工程化 |
| Bake/Editor审查 | review_complete | 假几何、settings未生效、无artifact writer、operation明确拒绝Bake、11份ZUI大面积占位 |
| 参考实现 | review_complete | Recast、Unreal、Godot、Fyrox适用事实；Bevy/Unity Graphics负向边界 |
| 本轮登记 | review_complete | 0 P0 / 48 P1 / 12 P2 / 32 gates；P0与五份failure沿用canonical owner |
| Production/tests修改 | pending | 本篇只写review与重构计划，没有修改production/tests或运行Cargo/C++/App/Editor |

本报告完成的是Navigation纵向事实冻结和可验收重构设计，不是功能修复。下一阶段必须从M0/M1的唯一provider、per-world owner和产品truth开始，不能从补按钮、扩固定容量或继续丰富临时DTO开始。
