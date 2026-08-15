---
related_code:
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/graphics/visibility/planning/build_bvh_update_plan.rs
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/visibility/view_context/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/core/framework/render/camera.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RendererScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrivate.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrivateBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneRendering.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneVisibility.cpp
tests:
  - current visibility slice 62 of 62 Rust files reviewed, 5220 lines, 38 inline tests
  - default camera spatial-prefilter formula gate passed and proves 64000 cells exceeds the 4096-cell budget
  - scoped rustfmt 55 of 62 clean; seven foreign-modified files have import/assert formatting drift
  - current-source Windows Cargo, F2 product counters, WPR, Tracy, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Graphics visibility current-source结构审查（2026-08-14）

## 当前范围与证据身份

`zircon_runtime/src/graphics/visibility/**`当前物理清单62/62个Rust文件：5,220行、4,765个非空行、38条内联测试，fingerprint为`47B15DF5D96726553B491745C4574B02EB7D37C3D1CC179F29B9AE242D6A4431`。分布为root 2/384行、context 10/1,119、culling 7/341、declarations 21/354、occlusion 2/381、planning 17/1,334、static_index 1/620、view_context 2/687。本轮以旧61文件报告为历史基线，逐项复读current 62文件、新增`spatial_query.rs`、9个foreign-modified文件及mesh/shadow直接consumer。

当前正向变化必须保留：frustum投影参数按view构造一次，extra views共享candidate slice；`VisibilityStaticIndex`的entries/cells/overflow已改为`Arc`共享并为bounds/ray查询设置cell预算，ray使用DDA；visible spatial query的1/1k/10k静态测试已锁定小范围查询的candidate/node规模。旧报告所称“每帧无条件深clone整个index”和“bounds无预算展开”已经过时，不能继续作为修复依据。

全切片`rustfmt --check --config skip_children=true`中55/62通过；其余7个失败均位于其他会话当前modified文件，表现为import顺序和断言布局漂移。本轮没有改写9个foreign-modified产品文件。

## P0：默认相机下静态预筛选必然失效

主视图只有静态primitive达到10,000才尝试空间预筛选；实现却把视锥变成以camera origin为中心的保守球，再枚举球AABB覆盖的uniform-grid cells，超过4,096 cells就返回`None`并回到全`bvh_instances`扫描。默认camera为`z_far=200`、FOV 60度、aspect 16:9，当前公式得到radius 308.987461；16-unit cell上min/max为-20/19，跨度`40^3=64,000`，是预算的15.625倍。

因此默认MVP相机即使有10,001个static meshes，`main_view_prefilter_used`也必为false。`construct/tests.rs`同一current source却断言该fixture应为true，而仅把`z_far`改为1,000的相邻测试才应false。这是可解析的current-source RED，不是缺少采样后的推测：默认产品路径与静态测试合同直接矛盾，现有空间索引对主视图通常不起作用。

修复不能只把4,096放大到64,000。应直接以frustum planes遍历hierarchy/cells，整节点inside时批量写visible bits，intersect节点才下探；或使用按scene distribution选择的BVH/octree。camera背后和球体侧面的cells不应进入候选。验收记录visited nodes、eligible candidates和fallback reason，而不是只记录“index存在”。

## P0：稳定帧仍重建整套scene visibility数据库

每个frame的`collect_batching_result`都会重建mesh alpha-mode Vec，按stable key排序mesh indices，建立all/static/dynamic三个`BTreeSet`、batch `BTreeMap`、primitive relevance、BVH instances和history entries；同一wide batch key/layer mask又进入batch、BVH与history。`FrameVisibility::from_main_view`再建relevance `BTreeMap`并把同一primitive表投影为entities、stable keys、bounds、layer masks和relevance五个Vec。

BVH update plan对previous/current history各建一张`BTreeMap`后再做insert/update/remove三遍；static和dynamic index随后各自为输入slice再建stable-key `BTreeMap`。即使delta为空，`apply_update_plan`仍建整张map并把`frame_incremental_update_count`记为1。index的Arc clone在stable generation是cheap的，但任何dirty update在旧snapshot仍存活时会触发`Arc::make_mut`复制整张entries/cells/overflow map；这不是dirty-proportional update。

目标是PERF-MVP-620/420的generation-owned packed primitive artifact：persistent slot直接持bounds/layer/mobility/relevance/batch identity，scene delta只更新dirty slots和spatial nodes；history保存generation/revision与paged Arc handle。stable frame不得排序、建tree或复制wide key，1% change的访问/复制量应接近dirty primitives及受影响nodes。

## P0：多视图工作与最终draw数据模型不一致

每个custom target和shadow view都先对全部N candidates执行frustum，再做layer/relevance过滤。TaskPool路径在N>=64时固定32一块，先复制完整`MeshFrustumWorkItem` Vec，再物化第二个结果Vec；粒度不随worker数、eligible数或view成本变化。一个开启shadow的directional light生成4 cascades，但capacity只按1个directional预留；shadow为`None`的directional仍生成1个shadow view，而point/spot会正确跳过disabled shadow。

更严重的是这些per-view结果没有保持到draw owner。mesh build把全部shadow view visible indices合成一个`HashSet`，最终每primitive只保存`shadow_view_visible: bool`；shadow command/cache只消费这个union布尔值。于是最多11个shadow views各自付出N次frustum与结果Vec成本，最终却不能表达“primitive只属于cascade/face X”，各shadow pass没有独立dense visible set。unshadowed directional产生的额外view也会让`has_shadow_views=true`并参与这次全局union，尽管shadow frame plan只为真实shadow-casting light建slot。

Render04/05/09必须共同把`ViewId/ShadowSlot -> dense primitive bitset/range/draw span`作为唯一合同：disabled shadow view数为0；每个启用的cascade/face保持自己的结果直到pass execution；共享工作只共享scene artifact、frustum planes和可复用scratch，不把输出提前并集。cheap layer/relevance/static-cell过滤先于exact frustum，CPU task粒度按eligible primitives和worker预算计算，大规模再由GPU scene cull接管。

## P0：visible spatial query重复发布且逐query建树

每次成功发布时，`VisibleSpatialQuery::from_context`先从main view visible indices建立`BTreeSet`，再全扫BVH instances建立visible-entry `BTreeMap`；同一main-view key set在visible batches和VG plan处还会重复物化。bounds/ray query又分别合并static/dynamic结果为candidate `BTreeSet`，按entity建第二个去重`BTreeSet`和结果Vec。oversized query会全扫visible-entry map。

该snapshot名义上的visible只包含CPU relevance/frustum结果；`ViewCullingStats.occlusion_culled_count`在visibility模块始终写0，HZB readback仅进入后续stats。因此editor selection必须显式选择frustum-eligible、last-completed HZB-visible或authoring-all语义，不能把当前pre-HZB集合称为最终可见性。

目标snapshot直接借用frame generation的primitive slots、per-view bitset和static/dynamic spatial pages；pointer/selection owner持reused epoch bitset与hit scratch，query不建立BTree集合。oversized工具查询必须有显式budget/cancel/continuation，而不是在UI event锁域同步全扫V。

## Unreal Engine本地源码依据

- `ScenePrivate.h:1561-1595`以packed primitive index维护transforms/bounds/flags/visibility/octree等dense arrays；`RendererScene.cpp:1570-1707`把transform变化入队并跳过redundant update，支持persistent scene artifact与dirty command，而非每帧重建多表。
- `RendererScene.cpp:5712-5742`在compact/swap时同步更新packed arrays与persistent-to-packed映射；这是Zircon stable slot与camera-neutral scene artifact的直接工程依据，不要求复制UE对象模型。
- `SceneVisibility.cpp:731-768`先遍历primitive octree nodes，父节点完全inside时避免重复containment；`861-900`直接按word写`PrimitiveVisibilityMap`。这比camera球AABB枚举uniform cells再全量filter更符合view-frustum问题。
- `ScenePrivateBase.h:18-20,129-160,291-317`与`SceneRendering.h:1281-1315`使用typed packed bit arrays表示per-view primitive/static-mesh visibility；`SceneRendering.h:1965-1969`还保留per-primitive view masks。Zircon不应把多个shadow view结果压成一个bool。
- `SceneVisibility.cpp:3612-3708`按primitive数、worker数和schedule计算frustum/occlusion/relevance task粒度并限制范围，`3735-3738`显式串联frustum到relevance依赖。Zircon固定64/32阈值只能作为临时值，不能作为最终算法。

## 目标算法与实施顺序

1. Render03/04先定义persistent primitive slot、packed SoA、dirty command stream、spatial hierarchy和generation owner；PERF-MVP-620的extract只发布camera-neutral delta/handle。
2. main/custom/shadow view从prepared view descriptor开始，先做layer/relevance/class过滤，再遍历hierarchy并把结果写view-owned dense bitset/ranges；默认camera不再构造64,000-cell球AABB。
3. Render05/09让shadow slot与visibility view一一对应，disabled light不建view，cascade/face bitset一直保留到draw execution；删除全shadow union bool。
4. stable generation复用scene/index/batch/history artifact；dirty generation用slot/page级copy-on-write或双buffer delta，禁止`Arc::make_mut`复制全scene map。
5. visible batches、VG、draw prepare、editor point/rectangle query借用同一generation bitset/slot table；query用epoch/scratch去重，明确pre-HZB、last-completed HZB与authoring语义。
6. 最后才按eligible count、worker budget和测得的task overhead选择serial/CPU parallel/GPU路径；不得用多线程掩盖全量DTO/tree重建。

## 动态验收矩阵与阻塞

规模：primitives 0/1k/10,001/100k/1M，static 0/50/100%，dirty 0/1/100%，views main/custom/cascade/point/spot为1/4/12/64，threads 1/2/8/64，camera far 10/200/1,000，bounds normal/overflow/nonfinite，stable/transform/layer/material/add-remove。记录scene artifact builds、sort/tree/key clone bytes、index page copies、visited nodes/cells/candidates、per-view exact tests/result bytes、view allocations、task count/queue age、shadow pass draw counts、query allocations、CPU p50/p95/p99、RSS、CSwitch/ReadyThread、GPU timestamp与energy。

硬门：默认10,001-static fixture使用hierarchy且无64,000-cell fallback；stable frame scene/batch/index/history rebuild=0；1% change访问/复制近dirty+affected nodes；shadow off directional view=0，shadow on directional=4且每cascade draw set独立；frustum visits近eligible candidates；per-view临时full-N work/result Vec=0；main visible bitset发布1/generation且consumer deep clone=0；point query无BTree allocation，oversized query有界可取消；TaskPool粒度随eligible/worker变化。

当前source formula gate通过并量化为64,000/4,096；但同源Rust测试无法在current workspace执行。最近managed Windows lib-test在843.4秒后因361个共享foreign编译错误结束，0 tests执行；workspace与`E:\ZirconBuilds`均无`zircon_app.exe`，所以WPR/Tracy/energy、F2产品counter、GPU timestamp与RenderDoc没有可信样本。本记录不进入`review.md`，由Render03/04/05/09/17与Runtime07/11继续承接。
