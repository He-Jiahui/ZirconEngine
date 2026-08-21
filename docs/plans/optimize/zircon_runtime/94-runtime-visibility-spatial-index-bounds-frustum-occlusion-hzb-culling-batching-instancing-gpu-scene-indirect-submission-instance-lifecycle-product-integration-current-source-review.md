---
title: Runtime Visibility、Spatial Index、Bounds、Frustum、Occlusion、HZB、Culling、Batching、Instancing、GPU Scene、Indirect Submission、Instance Lifecycle 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime94
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
tests:
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_runtime/src/graphics/tests/visibility
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct/tests.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_workspace/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/28-hardware-ray-tracing-blas-tlas-ray-query-pipeline-sbt-denoising-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/34-vegetation-tree-foliage-grass-species-instancing-wind-animation-billboard-impostor-lod-streaming-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/66-runtime-xr-openxr-device-session-stereo-view-tracking-input-late-update-foveation-compositor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_plugins/17-first-party-virtual-geometry-source-runtime-editor-dist-catalog-asset-cook-cluster-page-streaming-culling-raster-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneVisibility.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingContext.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneCulling/SceneCulling.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/HZB.cpp
  - dev/bevy/crates/bevy_pbr/src/render/gpu_preprocess.rs
  - dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs
  - dev/bevy/crates/bevy_pbr/src/render/occlusion_culling.wgsl
  - dev/bevy/crates/bevy_render/src/view/visibility/mod.rs
  - dev/bevy/crates/bevy_pbr/src/meshlet/instance_manager.rs
  - dev/bevy/crates/bevy_pbr/src/meshlet/cull_instances.wgsl
  - dev/bevy/crates/bevy_pbr/src/meshlet/cull_bvh.wgsl
  - dev/Fyrox/fyrox-impl/src/renderer/visibility.rs
  - dev/Fyrox/fyrox-impl/src/renderer/occlusion/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/occlusion/grid.rs
  - dev/Fyrox/fyrox-impl/src/renderer/occlusion/optimizer.rs
  - dev/Fyrox/fyrox-math/src/frustum.rs
  - dev/Fyrox/fyrox-math/src/octree.rs
  - dev/godot/servers/rendering/renderer_scene_cull.h
  - dev/godot/servers/rendering/renderer_scene_cull.cpp
  - dev/godot/servers/rendering/renderer_scene_occlusion_cull.h
  - dev/godot/servers/rendering/renderer_scene_occlusion_cull.cpp
  - dev/godot/servers/rendering/renderer_geometry_instance.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/GPUResidentDrawer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/LODGroupDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceOcclusionCuller.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/OcclusionCullingCommon.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderPipelineResources/GPUDriven/InstanceOcclusionCullingKernels.compute
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Visibility、Spatial Index、Bounds、Frustum、Occlusion、HZB、Culling、Batching、Instancing、GPU Scene、Indirect Submission、Instance Lifecycle 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon的可见性路径不是空壳。`FrameVisibility`已经保留stable instance key与逐view的visible index；主相机、自定义target、方向光cascade、点光六面和spot shadow都有独立`ViewVisibilityContext`。GPU Scene已有稳定span allocator、dirty-range合并、current/previous transform、skin/morph/Virtual Geometry数据与大上传staging ring。Mesh pass也不再只是CPU indirect名义层：每phase有持久workspace，HZB compute能清零并压紧draw args、visible-instance remap与draw count，随后走multi-draw-indirect-count、多draw或逐draw降级。一个真实WGPU测试还会提交offscreen HZB剔除并readback结果。这些底座应保留并迁入目标架构。

但它仍不是工程级GPU-driven renderer。`VisibilityContext`每帧从完整`RenderFrameExtract`重新构建一组互相平行的`BTreeMap`、`BTreeSet`和`Vec`；不存在持久`RenderScene`、primitive lifecycle journal和authoritative bounds owner。资源层已有真实mesh/model local bounds及morph-expanded bounds，`RenderMeshSnapshot`却不传递它们；visibility用translation加scale-length单位球，GPU Scene又独立用translation和最大轴尺度近似。HZB shader随后把已近似成world-space的center再次乘`world_from_local`并再次缩放radius，Runtime09B记录的空间双变换P0仍可由current source直接复现。

空间预筛只是单层uniform grid：首次构建全量`BTreeMap<Cell, BTreeSet>`，增量更新先重建完整entry map，历史快照被持有时`Arc::make_mut`可能复制整张map；大bounds进入所有query共享overflow，cell过多又退化为全表候选。每个附加view继续遍历同一完整CPU候选并做sphere frustum。关闭阴影的方向光仍会产生一个visibility cascade view；shadow renderer后来虽然按精确view key过滤命令，但早期union状态和无效剔除工作仍被制造。

现有GPU indirect也没有完成GPU-driven闭环。所有mesh先完成load/prepare/material/deformation/pending-command扩展、VG/morph/light upload和全量GPU Scene sync，随后才应用visibility；每个pending draw固定以`instance_count = 1`登记，visibility产生的`gpu_instancing_candidates`、`instance_upload_plan`和`particle_upload_plan`没有产品消费者。HZB只处理若干opaque/velocity phase，并压紧CPU已经构造好的完整command/args/metadata集合；声明中的`TwoPhaseRetest`没有执行，产品只使用previous-frame HZB单次测试。它因此是“真实GPU compaction”，但还不是由持久scene和GPU preprocess产生的per-view GPU submission truth。

Runtime09B的旧结论有两项已明显改善：逐view identity现在被`FrameVisibility.views`保留，shadow pass也按view-visible entity set过滤；indirect path现在确实执行GPU compaction与indirect replay。本文不重复登记旧P0，要求原owner按current source拆分关闭已过时子句。其余持久scene、bounds双变换、未消费visibility计划、受限HZB候选、VG CPU materialization和visibility前高成本prepare仍开放或部分开放。本篇新增 **0项P0、48项P1、12项P2与48个资格门**。在100K实例、多view/shadow、camera cut、rapid spawn/despawn、reload、device loss、OOM和同画质Unreal对照证据闭合前，不得声称可见性、GPU Scene或indirect submission达到或优于Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 证据等级 | fingerprint |
|---|---:|---|---|
| Runtime visibility / GPU Scene / HZB / mesh submission产品语料 | **188 / 35,468 / 32,436 / 1,286,197 / 288** | E3逐文件覆盖visibility、GPU Scene、HZB、mesh build/pass、extract与shadow consumer | `ff7170941560799d45029169bef8b580c06f0252702b28eb2fe7873cacc0da66` |
| focused visibility tests | **4 / 2,136 / 1,979 / 77,070 / 25** | E3读取graphics visibility外部测试；内嵌GPU/HZB/indirect tests已计入产品语料 | `f70f4cb4927d84fd1061fe2340d2a8d716466f987bcede9f76ea28d3d8639b5e` |
| 五引擎参考切片 | **35 / 39,103 / 33,396 / 1,644,455 / 17** | E2/E3读取Unreal GPUScene/SceneCulling、Bevy preprocess/meshlet、Fyrox visibility/octree、Godot scene cull与Unity GPUDriven | `c121798ebe43fda7c40b3fe78ac57f55e9769885b020709735503fa284b8d465` |

冻结集合代表2026-08-21共享working tree，不是只读HEAD或实现验收receipt。Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator baseline epoch为336。Bevy、Fyrox、Godot与Unity Graphics参考revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像`Build.version`为6.0.0/UE5/changelist 0且无独立`.git`，由reference aggregate fingerprint冻结。

冻结时其他Session正在修改`scene_extract.rs`和`build_mesh_draw_build_context.rs`。本文读取并计入这些working-tree版本，但不拥有也未修改它们，因此`source_recheck_required: true`。实施前必须重新生成fingerprint，并重验extract bounds、pending draw payload、GPU Scene sync次序及indirect compatibility。

### 2.2 Owner边界

| 边界 | 本篇所有 | 既有父owner继续所有 |
|---|---|---|
| Render Scene与visibility | persistent primitive/view registry、authoritative bounds消费、spatial broadphase、frustum/occlusion/LOD candidate与per-view result | Runtime61/62拥有World/Scene/transform/bounds source；Runtime37/66拥有camera/XR view contract |
| GPU Scene与submission | primitive/instance lifecycle、dirty GPU upload、GPU preprocess、compaction、indirect args/count/remap、history与retirement | Runtime89/90拥有RenderGraph、RHI resource、queue、completion与device loss；Runtime93拥有mesh/deformation artifact |
| Material/streaming/VG | visibility只消费generation-qualified payload、residency与compatibility result | Runtime91/92拥有material/texture pipeline与residency；Plugins17拥有VG cluster/page/raster产品链 |
| 历史P0 | current-source状态、纠偏与实施前重验 | Runtime09B继续唯一计数其7项P0；本文不复制P0数量 |

### 2.3 Runtime09B current-source纠偏

| Runtime09B旧finding | 当前状态 | current-source判定 |
|---|---|---|
| 无持久Render Scene | 仍成立 | 每帧仍从完整extract重建`VisibilityContext`与并行集合；没有primitive change journal或scene generation |
| uniform grid退化且逐view identity丢失 | 部分修复 | grid退化仍成立；`FrameVisibility.views`和shadow exact view consumer已保留逐view identity，旧子句应关闭 |
| GPU HZB bounds双变换/空间错误 | 仍成立 | CPU写入translation/scale近似，shader又应用instance transform和radius scale；缺非原点/非单位尺度GPU回归 |
| visibility计划未消费、indirect只是CPU命令 | 部分修复 | instancing/upload计划仍未消费；真实GPU compaction、count与replay已存在，“仅CPU indirect”描述过时 |
| HZB只压紧CPU候选且history truth不完整 | 部分修复 | GPU压紧真实存在，但候选/args/metadata仍由CPU完整构造，只有single-frame previous HZB，无late retest |
| VG cull/raster/VisBuffer名称高估CPU materialization | 仍成立 | visibility frontier仍有BTree/排序/ordinal重扫，VG由独立CPU materialization与indirect路径承接 |
| 高成本prepare发生在visibility前 | 仍成立 | load/prepare/deformation/pending draw/GPU Scene sync在`mesh_visibility_states`过滤前完成 |

## 3. 当前产品链与断裂点

```text
World / RenderFrameExtract
  -> 每帧构造 VisibilityContext
     -> transform近似 sphere bounds
     -> static/dynamic uniform grid + overflow
     -> main/custom/shadow逐view CPU sphere frustum
     -> FrameVisibility + 未消费的instancing/upload/HGI plans
  -> Mesh build遍历全部phase_ordered_meshes
     -> load/prepare/material/skin/morph/VG/pending commands
     -> 全pending draws同步进GPU Scene，每draw instance_count=1
     -> visibility state过滤、command cache与phase lists
  -> CPU建立完整indirect args / draw metadata / candidates
  -> previous-frame HZB单次compute cull与dense compaction
  -> multi-draw-indirect-count / multi-draw / per-draw fallback
  -> frame结束扫描live entries滚动previous transform/history
```

目标产品链必须变为：

```text
Scene mutation / asset generation / transform propagation
  -> RenderSceneChangeJournal
  -> PersistentRenderSceneGeneration
     -> canonical local/world/deformed/motion bounds
     -> stable primitive/instance/material/geometry handles
     -> hierarchical CPU broadphase + dirty GPU Scene scatter upload
  -> ViewFamilyDescriptors + persistent per-view history
  -> GPU preprocess: relevance / frustum / LOD / previous-HZB early cull
  -> depth/occluder update
  -> current-HZB late retest + compaction / binning / indirect count
  -> RenderGraph packet -> RHI SubmissionTicket
  -> completion-qualified history publish / retirement
```

## 4. 当前应保留的真实基础

1. `FrameVisibility`以对齐数组保存entity、stable instance key、bounds和relevance，并拥有逐view visible index，已经不是单一main-view bool。
2. `VisibilityStaticIndex`已有incremental update策略、generation/report与fail-open query，适合作为替换层的characterization oracle。
3. GPU Scene allocator、pending free merge、dirty-range merge、grow-only GPU buffer、small direct write和三槽大上传staging ring是真实资源基础。
4. current/previous transform、skin palette/source、morph weight与VG payload history已经有明确存储位置，可迁移到统一arena和generation contract。
5. per-phase `IndirectDrawWorkspace`、delta upload、GPU HZB compaction、draw count与visible-instance remap是真实indirect execution，不应被回退成纯CPU draw list。
6. HZB builder、persistent sampled resource identity、bind-group cache和真实offscreen GPU test提供了可扩展的图执行入口。
7. shadow renderer已能按`VisibilityViewKey`取得精确view visible entity set，证明逐view消费合同可落地。
8. `RenderStats`已有visibility view/culling/static-index与HZB diagnostics入口，可扩展为预算、退化原因和qualification telemetry。

## 5. 既有P0 current-source复核

本文新增0项P0。Runtime09B的P0按上表继续由原报告唯一计数；实施前必须把“逐view identity丢失”和“indirect只是CPU命令”两条已过时子句从父finding拆出，而不能因局部进展误关整个owner。Runtime93记录的mesh bounds、true instancing与deformation依赖，Runtime89/90记录的RenderGraph/RHI completion，Runtime92记录的residency也仍是本路径的硬依赖。

最危险的当前缺陷仍是bounds双重空间变换：extract没有authoritative mesh bounds，visibility和GPU Scene各自猜单位球，HZB shader又重新应用world transform。它会同时破坏遮挡保守性、LOD、streaming demand和性能统计；不能只在shader中删一行矩阵乘法，而必须先建立canonical bounds payload与空间标签，再hard cutover所有消费者。

## 6. P1：Architecture、Lifecycle、Policy 与 Truth Publication

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| R94-P1-01 | `VisibilityContext`每帧从完整extract重建renderable、mobility、relevance、batch、bounds、history与upload等平行集合 | 建立persistent `RenderSceneGeneration`和change journal，只对add/remove/change更新primitive record与view input |
| R94-P1-02 | history snapshot依附临时context，previous transform发布又扫描全部live GPU Scene entry | 将view/primitive history变成generation-owned arena，只有dirty或本帧提交成功的span进入post-submit publish |
| R94-P1-03 | `gpu_instancing_candidates`、`instance_upload_plan`、`particle_upload_plan`无产品消费者，HGI参数被忽略且输出恒空 | 删除假合同或接入唯一executor；feature未实现必须由capability/admission明确拒绝，不能返回看似完整空计划 |
| R94-P1-04 | CPU/GPU culling、HZB、multi-draw与fallback由散落布尔和backend能力临时分支决定 | 建立device/profile/view-family级requested/effective policy、拒绝原因、降级阶梯和receipt |
| R94-P1-05 | visibility、GPU Scene、HZB和indirect没有统一CPU time、GPU time、bytes、candidate、overflow与workspace预算 | 建立per-frame/per-view budget admission、bounded growth、backpressure、typed exhaustion与quality controller输入 |
| R94-P1-06 | 热路径大量使用`BTreeMap`、`BTreeSet`、排序Vec和payload clone，数据布局不围绕stable dense handle设计 | 以dense SoA、bitset、span和generation handle承载hot state；有序结构只留在离线/诊断边界 |
| R94-P1-07 | `GpuScene`文档仍称“before frame-path wiring”，`visited_node_count`实际统计grid cell，名称与产品truth漂移 | 统一术语、capability与stats schema；结构guard必须验证consumer和语义，不只匹配source token |

## 7. P1：Bounds、Spatial Index 与 Query

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| R94-P1-08 | `RenderMeshSnapshot`不携带资源层已有的mesh/model local bounds和morph-expanded bounds | extract显式携带`BoundsHandle + BoundsGeneration + local bounds kind`，并与mesh/deformation generation一致 |
| R94-P1-09 | visibility `mesh_bounds`只从translation和scale-length生成单位球 | 消费authoritative local AABB/sphere，按完整affine transform保守地产生world bounds |
| R94-P1-10 | GPU primitive center/radius仍是transform代理，material payload slot固定invalid | `GpuPrimitiveData`消费canonical world bounds与resolved material payload handle，不再从draw transform猜数据 |
| R94-P1-11 | CPU/GPU没有明确local/world/view/clip空间标签、origin和transform次数合同 | 定义bounds ABI、large-world origin、矩阵方向、radius scale和shader layout；父P0以CPU/GPU parity关闭 |
| R94-P1-12 | skin、morph、cloth/VFX等deformation没有统一dynamic bounds provider | 建立bind/deformed/predicted bounds generation，按feature、LOD和quality选择CPU/GPU reduction或cooked envelope |
| R94-P1-13 | 没有velocity、animation uncertainty、camera jitter和temporal occlusion需要的bounds expansion policy | 将motion margin、history validity与occlusion confidence纳入view-specific conservative bounds |
| R94-P1-14 | broadphase和frustum只使用sphere，无法表达细长mesh、OBB、AABB或cluster bounds | 以AABB为持久空间索引基础，允许sphere/OBB/cluster bounds作分层narrow test，并保留保守性证明 |
| R94-P1-15 | static/dynamic索引共享单层uniform grid，不是分层BVH、octree或spatial hash blocks | 建立large-world分层空间索引，static build与dynamic refit分离，支持AABB/sphere/frustum/ray query |
| R94-P1-16 | incremental update先重建完整entry map；历史snapshot存活时`Arc::make_mut`可能复制整图 | 使用page/chunk级copy-on-write或epoch snapshot，dirty update成本随changed primitives增长 |
| R94-P1-17 | query枚举cell到Vec，再用`BTreeSet`去重和排序candidate | 使用visited generation/bitset、arena scratch与连续candidate spans，steady query禁止按candidate堆分配 |
| R94-P1-18 | 大bounds进入全局overflow并参与每次query；超过cell上限又退化为所有entry | 建立oversized primitive层、层级节点和有界fallback；overflow必须有数量/bytes/top offender/拒绝或降级策略 |
| R94-P1-19 | 10K启用阈值、cell size、4096 cell上限和parallel阈值是固定常量，report把cell称node | 改成profile/capability/budget参数，以benchmark选择并发布effective值和真实算法指标 |

## 8. P1：Multi-view、Shadow 与 View Family

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| R94-P1-20 | main/custom/cascade/cubemap face/spot view对同一CPU candidate Vec重复完整sphere frustum | 先做scene/view-family共享broadphase，再用SIMD/job/GPU per-view bitmask或compact list分流 |
| R94-P1-21 | `directional_shadow_ranges(None)`仍返回一个range，关闭阴影的方向光也产生visibility view | shadow disabled必须产生零shadow view；新增无shadow、多light和mixed setting产品回归 |
| R94-P1-22 | shadow view只检查`shadow_caster`，没有light-specific render-layer/culling-mask合同 | 将light channels/layers、receiver/caster mask、cascade split和view key写入同一shadow view descriptor |
| R94-P1-23 | mesh build把所有shadow view可见性union成单个`shadow_view_visible` bool | 缓存/prepare只能使用per-view demand或精确view mask；union仅可作保守共享工作提示，不能作为submission truth |
| R94-P1-24 | 没有正式ViewFamily对stereo、cube、portal、editor multi-viewport和共享history进行归组 | 建立stable View/ViewFamily identity、generation、parent/child relation、shared resource与history invalidation |
| R94-P1-25 | GPU HZB/indirect以phase workspace为中心，没有per-view GPU candidate/visible-list owner | 每个view/subview拥有可追踪的candidate、early-visible、late-retest、final-visible与indirect range |

## 9. P1：GPU Scene、Instance 与 Memory

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| R94-P1-26 | allocator内部有stable span，但外部以stable key map查找，缺generation-bearing primitive/instance handle | 引入`PrimitiveHandle`、`InstanceHandle`与generation；remove/reuse/stale引用必须typed reject |
| R94-P1-27 | GPU buffer只增长不缩，缺budget pressure、relocation、fragmentation和fence-qualified compaction | 建立arena telemetry、high-water/shrink policy、relocation table与提交完成后retirement |
| R94-P1-28 | 成功帧后扫描全部live entries滚动previous transform并标记changed span | 维护moved/visible/submitted dirty queues，仅更新需要history的实例；camera cut和first frame显式invalid |
| R94-P1-29 | current/previous skin palette、source、morph以多组per-instance HashMap和CPU shadow Vec维护 | 收敛为typed component arena/SoA，统一allocation、dirty bitset、upload、history和retirement |
| R94-P1-30 | morph与VG通过完整Vec比较发现变化，payload大时CPU和内存带宽随总数据增长 | producer发布versioned dirty rows/ranges；GPU upload按generation delta，不比较完整payload |
| R94-P1-31 | 12-binding mega scene bind group跨primitive/instance/light/skin/remap/morph/VG耦合，任一buffer grow重建全局组 | 分离稳定global scene表、feature arena和per-pass table；以bindless/descriptor generation最小化全局失效 |
| R94-P1-32 | 每个skinned stable key创建current/previous专用storage buffer和command-local scene bind group | 建立共享palette/source arena与offset handle；skin实例能进入统一batch/indirect兼容路径 |
| R94-P1-33 | 产品为每个pending draw调用`register(..., 1)`，没有消费visibility instancing候选 | `MeshDrawPacketCompiler`按geometry/material/pass/deformation compatibility形成instance span并上传dense records |
| R94-P1-34 | GPU Scene material payload slot保持invalid，draw state仍由CPU command持有 | 建立generation-qualified material table/parameter block handle，使GPU preprocess/binning可安全读取material state |

## 10. P1：Preparation、Batching 与 Indirect Submission

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| R94-P1-35 | load/prepare/material/deformation/VG扩展发生在visibility过滤前 | 分层为cheap retained primitive update、visibility/residency demand、async prepare和final draw packet；不可见对象不预付昂贵工作 |
| R94-P1-36 | `sync_gpu_scene_pending_draws`以全部pending draw为live set，不以persistent scene dirty journal或final residency为输入 | GPU Scene sync消费add/remove/change journal和generation-ready payload，view visibility不决定对象生死 |
| R94-P1-37 | CPU先生成完整command、indirect args、batch metadata和candidate列表，GPU只做末端筛除 | GPU preprocess从persistent scene records构造visible instances、bin/count/args；CPU只提交bounded work descriptors |
| R94-P1-38 | indirect batcher只合并相邻兼容command，对排序和state binning依赖上游偶然顺序 | 建立stable draw key、radix/binning或GPU binning，明确transparent order与不可重排约束 |
| R94-P1-39 | 每个primitive保留CPU command object，depth/shadow/opaque/alpha/PBR/transparency/velocity等重复phase plan | 以shared draw packet + phase mask/permutation metadata生成各pass工作，避免多份对象和重复validation |
| R94-P1-40 | command-local GPU Scene bind group、已有indirect、skinned和VG路径会退出通用batcher | 定义可解释compatibility matrix和typed fallback reason；各feature必须接入统一instance/indirect ABI或明确不支持 |

## 11. P1：HZB、Occlusion 与 Validation

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| R94-P1-41 | 产品只执行`SingleFrameReproject`，`TwoPhaseRetest`枚举没有executor | 实现previous-HZB early pass、depth/occluder更新、current-HZB late retest及final publish状态机 |
| R94-P1-42 | HZB candidate只覆盖部分opaque/alpha/PBR/velocity phase，不覆盖depth、shadow和透明策略 | 建立phase capability matrix；depth/shadow采用保守candidate，transparent明确排序/occlusion policy与fallback |
| R94-P1-43 | shader以`world_radius / clip.w`估projected radius并采一个texel，没有完整projection/FOV/axis与4-corner conservative test | 采用screen-space rect、mip选择、多sample/furthest-depth规则和near-plane/behind-camera保守处理，并做CPU oracle对照 |
| R94-P1-44 | builder声称多mip reduction，执行却逐mip开pass/bind group；params/bind group生命周期仍有frame churn | 使plan与execution一致，复用persistent params/bindings或single-dispatch multi-mip方案，并计入RenderGraph resource lifetime |
| R94-P1-45 | white fallback纹理让history缺失时fail-open，但camera cut、resolution change、view identity change和occluder version没有统一有效性合同 | 建立per-view HZB history generation、invalid reason、warmup、cut/reset和resolution migration状态 |
| R94-P1-46 | 多数HZB测试是source-string/DTO检查；真实GPU测试使用原点center和单位尺度，绕开空间双变换 | 增加非原点、非单位/负尺度、near-plane、jitter、camera cut、resize、多view与device fallback的CPU/GPU语义测试 |
| R94-P1-47 | 没有10K/100K static/dynamic、多个shadow view、spawn/despawn churn和历史快照长持有的产品scale test | 建立固定scene generator、warmup/steady/soak基线，记录CPU/GPU time、alloc、upload、candidate、visible、draw与VRAM |
| R94-P1-48 | 没有同一primitive在extract/CPU bounds/GPU Scene/HZB/streaming/shadow间的代际与空间一致性门，也没有性能退化自动阻断 | 建立cross-stage trace ID、buffer readback/capture oracle和BuildSet-bound threshold；越界构建不得通过qualification |

## 12. P2长期能力

| ID | 能力 | 前置条件 |
|---|---|---|
| R94-P2-01 | hardware occlusion query与HZB hybrid | P1 bounds、view history、async result和fail-open合同 |
| R94-P2-02 | software occlusion raster fallback / server visibility | canonical bounds、deterministic fixed-point或受控float oracle |
| R94-P2-03 | portal、room、sector与visibility cell | persistent scene与hierarchical spatial owner |
| R94-P2-04 | large-world multi-level spatial hash、world partition visibility streaming | Runtime23/29、origin rebasing与partition generation |
| R94-P2-05 | GPU Scene在线defrag、page relocation和incremental compaction | generational handles、indirection table与submission fence |
| R94-P2-06 | stereo/multiview/foveated shared culling和view-mask indirect | Runtime66 ViewFamily、per-view history与backend capability |
| R94-P2-07 | meshlet/cluster hierarchy、cluster HZB与software raster | Runtime93与Plugins17 canonical mesh/VG artifact |
| R94-P2-08 | GPU draw sorting、state binning、command compression与mesh shader dispatch | material/geometry stable tables和transparent constraints |
| R94-P2-09 | predictive occlusion、motion confidence和temporal hysteresis | valid velocity、history generation和false-negative telemetry |
| R94-P2-10 | async-compute preprocess/HZB overlap与load balancing | Runtime89/90 queue ownership、barrier和timestamp evidence |
| R94-P2-11 | scene/primitive/view/HZB/indirect可视化、capture与offline replay | stable trace identity、artifact schema与privacy/budget policy |
| R94-P2-12 | 根据scene density、view count和GPU反馈自适应选择CPU/GPU broadphase | P1完整telemetry、quality policy和可复现实验基线 |

## 13. 五引擎差异证据

### 13.1 Unreal Engine

`FGPUScene`拥有持久primitive/instance/payload/lightmap buffers、span allocator、dirty state、pre/post scene update、scatter upload与dynamic primitive collector；`SceneCulling`不是单层grid，而是多level spatial hash、block/cell/chunk、static/dynamic compressed chunk、async update/query及节点summary。`InstanceCullingContext`进一步拥有load balancer、screen-size/LOD/HZB cull、draw command descriptor、compaction prefix sum、order preservation与indirect args。Zircon应借鉴的是persistent ownership、hierarchical broadphase、GPU preprocess和submission lifecycle，不是复制UE宏、global state或特定RHI封装。

### 13.2 Bevy

普通mesh GPU preprocess支持GPU frustum与early/late/main occlusion phase、previous input mapping、sparse current input/free slots、changed previous ranges和phase-specific persistent work buffers；shader同时处理OBB/frustum、visibility range/LOD、previous HZB early cull、current HZB late retest和indirect metadata。meshlet路径还拥有instance AABB、per-view visibility、BVH/cluster queues与两阶段cull。需要保留边界判断：当前meshlet `InstanceManager`仍每帧reset/rebuild并留有改成change event的TODO，因此本报告只引用其GPU层次和normal preprocess增量模型，不把它当成完整生命周期答案。

### 13.3 Fyrox

Fyrox以observer/cell visibility cache和异步occlusion query提供fail-open语义，tile occlusion tester消费world AABB与projected rectangle；octree则提供正式AABB/sphere/ray/point query。它不是GPU-driven复杂度标杆，但证明基础scene query、world bounds和pending occlusion result不应由draw builder临时拼装。

### 13.4 Godot

Godot `Scenario`持有`DynamicBVH`、paged `InstanceBounds`/`InstanceData`和visibility range；instance拥有local/transformed/previous AABB、custom AABB、visibility margin/range/parent dependency、dirty list与occlusion timeout。`_update_instance`按真实AABB增量更新motion bounds和BVH，AABB/ray/convex query也先收敛dirty instance。其可借鉴点是scenario-owned lifecycle、真实bounds、dirty propagation和稳定query边界，而不是具体BVH实现。

### 13.5 Unity Graphics

Unity GPU Resident Drawer把MeshRenderer/LODGroup update与deletion batches送入persistent `InstanceDataSystem`；系统有renderer-to-handle map、archetype allocator、free/reallocate、grow/shrink policy、local/world AABB、current/previous transform、moved/visible bitset和GPU scatter update。`CullingJob`按world AABB执行split frustum、receiver、LOD/crossfade与可选occlusion，随后构造direct/indirect输出。遮挡路径为每view保存occluder context，compute first pass把被遮挡实例写入second-pass list，occluder更新后以indirect dispatch重测并直接更新draw instance count/remap。它也保留CPU frustum/LOD和staging成本，因此应作为生命周期、AABB和two-pass indirect旁证，而不是性能结论。

## 14. 目标架构与唯一所有权

| Owner | 唯一职责 | 禁止泄漏 |
|---|---|---|
| `RenderSceneService` | persistent primitive/instance registry、change journal、scene generation与add/remove/change transaction | 不由每帧extract或mesh draw list重建对象生死 |
| `RenderBoundsService` | canonical local/world/deformed/motion bounds、空间标签、generation与CPU/GPU ABI | 不允许visibility、GPU Scene、LOD、streaming分别猜bounds |
| `SpatialSceneIndex` | static build、dynamic refit、hierarchical query、snapshot epoch、overflow budget与telemetry | 不允许单层grid退化成无诊断全表扫描 |
| `ViewFamilyService` | stable view/family identity、frustum/cascade/subview descriptor、history validity和mask | 不允许shadow、XR、editor viewport各自造临时view语义 |
| `GpuSceneService` | stable generational handles、component arenas、dirty scatter、memory budget、relocation与fence retirement | 不允许per-draw私有palette/buffer成为常规路径 |
| `VisibilityPipeline` | relevance、broadphase、frustum、LOD、early/late occlusion、per-view visible spans与receipt | 不拥有asset I/O、material compile或draw submission |
| `MeshDrawPacketCompiler` | generation-ready geometry/material/deformation兼容键、instance spans、phase masks和fallback reason | 不以完整World/asset clone为输入 |
| `GpuSubmissionPlanner` | GPU preprocess、bin/count/compaction、indirect args/remap及RenderGraph packet | 不直接提交queue；交给Runtime89/90 RHI owner |
| `VisibilityHistoryService` | previous transform/view/HZB/LOD/occlusion confidence的commit-after-submit publish | 不在failed/cancelled frame推进history |

关键ABI至少包含：`PrimitiveHandle{index,generation}`、`InstanceHandle{index,generation}`、`RenderSceneGeneration`、`BoundsGeneration`、`ViewFamilyGeneration`、`GpuSceneGeneration`、`GeometryGeneration`、`MaterialGeneration`、`SubmissionTicket`与`HistoryCommitToken`。每个GPU record必须能追溯到同一组generation，任何跨代组合在admission阶段typed reject，而不是在shader中fail-open。

## 15. 依赖顺序与重构里程碑

| 里程碑 | 内容 | 依赖/退出证据 |
|---|---|---|
| M0 | current behavior characterization与Runtime09B P0拆分 | 冻结现有CPU/GPU可见结果、bounds空间错误和indirect fallback matrix |
| M1 | canonical bounds ABI与RenderScene journal | Runtime62/93；asset local bounds到extract/CPU/GPU/shadow/streaming一致 |
| M2 | stable primitive/instance handle与GPU Scene arenas | Runtime24/90；remove/reuse/churn、budget、dirty scatter和fence retirement闭合 |
| M3 | hierarchical spatial index与ViewFamily | Runtime23/29/37/66；static/dynamic增量、多view/shadow/XR query闭合 |
| M4 | visibility-first prepare、true instance spans与shared draw packet | Runtime91-93；不可见对象不预付，64/10K实例形成instance span |
| M5 | GPU preprocess、two-phase HZB与per-view compaction | M1-M4；early/depth/late状态、phase coverage、camera cut和indirect count闭合 |
| M6 | RenderGraph/RHI completion、history publish与failure recovery | Runtime89/90；failed submit不推进history，device loss可重建并保留last-good |
| M7 | product qualification与Unreal公平对照 | 100K、多view、dynamic churn、fault/soak/capture/visual parity/benchmark绑定BuildSet |

M5不得绕过M1直接重写compute shader。没有authoritative bounds、stable handles、persistent scene和visibility-first preparation时，GPU culling只会更快地产生错误或把CPU重建成本藏到上传前。M7也不能只比较draw call或单场景FPS；必须先固定相同可见物、LOD、shadow、material、resolution和画质。

## 16. 资格门

| Gate | 必须形成的证据 |
|---|---|
| VIS94-G01 | RenderScene add/remove/change journal有transaction、generation、rollback与deterministic replay |
| VIS94-G02 | steady frame无scene变化时primitive registry和visibility input不全量重建 |
| VIS94-G03 | entity/stable key/primitive/instance handle remove-reuse不会命中旧generation |
| VIS94-G04 | asset local bounds经extract、world transform、GPU Scene、shader readback完全一致 |
| VIS94-G05 | 非原点、非单位/负/非均匀尺度和large-world origin有CPU/GPU bounds parity |
| VIS94-G06 | skin/morph/动态变形产生generation-qualified conservative deformed bounds |
| VIS94-G07 | velocity、jitter、near-plane与temporal uncertainty expansion有固定oracle |
| VIS94-G08 | spatial static build与dynamic refit成本随dirty count增长，不随全scene增长 |
| VIS94-G09 | 历史snapshot长持有不会触发整图copy或无界retention |
| VIS94-G10 | oversized primitive、cell overflow和fallback有bounded policy与top offender telemetry |
| VIS94-G11 | AABB/sphere/frustum/ray query与brute-force oracle在随机/边界case一致 |
| VIS94-G12 | spatial阈值/cell level来自effective profile并记录选择原因 |
| VIS94-G13 | no-shadow directional light创建0个shadow visibility view |
| VIS94-G14 | cascade、point face、spot、custom target逐view identity和visible set不串线 |
| VIS94-G15 | light layer/caster/receiver mask在visibility与shadow submission一致 |
| VIS94-G16 | stereo/multiview共享工作但保持per-view history、mask与final visible result |
| VIS94-G17 | per-view candidate/early/late/final visible counts及GPU ranges可追踪 |
| VIS94-G18 | GPU Scene handle有generation，stale CPU/GPU引用被拒绝而非读新对象 |
| VIS94-G19 | primitive/instance/component arena有bytes、capacity、fragmentation和high-water stats |
| VIS94-G20 | grow/shrink/relocation在submission completion后执行，in-flight frame不读已回收span |
| VIS94-G21 | previous transform只更新moved/submitted实例，failed/cancelled frame不推进history |
| VIS94-G22 | skin/morph/VG dirty upload按changed range，不比较/上传完整payload |
| VIS94-G23 | scene bind-group generation变化范围最小，单feature buffer增长不失效无关资源 |
| VIS94-G24 | 共享skin palette/source arena支持current/previous且无per-instance常规buffer |
| VIS94-G25 | 64同mesh/material实例形成共享draw packet和`instance_count > 1` |
| VIS94-G26 | 10K/100K实例GPU Scene sync成本按dirty count与visible demand增长 |
| VIS94-G27 | resolved material payload进入GPU table并与pipeline/layout generation匹配 |
| VIS94-G28 | 不可见且无residency prefetch实例不执行material/deformation/draw packet构造 |
| VIS94-G29 | GPU Scene live set由scene journal与generation ready决定，不由临时view结果决定 |
| VIS94-G30 | CPU只提交bounded preprocess descriptors，不为所有candidate预建完整GPU args |
| VIS94-G31 | draw key/binning对可重排opaque确定，transparent order有明确不可重排约束 |
| VIS94-G32 | shared draw packet通过phase mask服务depth/shadow/base/velocity，避免对象复制 |
| VIS94-G33 | skinned/VG/custom bind group进入兼容路径或输出typed fallback reason |
| VIS94-G34 | direct/multi-draw/count fallback matrix按device capability测试且结果等价 |
| VIS94-G35 | previous-HZB early pass、depth/occluder update、current-HZB late pass真实执行 |
| VIS94-G36 | first pass被遮挡实例进入second-pass compact list，late visible能恢复draw |
| VIS94-G37 | HZB projected rect/mip/sample/depth规则与CPU oracle在边界case一致 |
| VIS94-G38 | camera cut、view ID复用、resolution/FOV change和history缺失均fail-open且有原因 |
| VIS94-G39 | HZB builder plan与实际dispatch/pass数一致，无逐mip临时bind-group增长 |
| VIS94-G40 | depth/shadow/opaque/alpha/velocity/transparent的occlusion policy矩阵完整 |
| VIS94-G41 | HZB空间双变换父P0由非原点/非单位尺度真实GPU测试关闭 |
| VIS94-G42 | 100K static、10K dynamic、多cascade/point shadow benchmark绑定固定scene hash |
| VIS94-G43 | rapid spawn/despawn、reload、camera churn与snapshot retention soak无无界增长 |
| VIS94-G44 | OOM、workspace exhaustion、shader/pipeline failure与device loss有typed降级/恢复 |
| VIS94-G45 | CPU/GPU capture含scene update、bounds、broadphase、cull、upload、draw、VRAM/RSS |
| VIS94-G46 | cross-stage trace可从entity追到primitive/instance/view/indirect command与ticket |
| VIS94-G47 | regression gate阻止CPU/GPU time、allocation、upload、candidate amplification越过基线 |
| VIS94-G48 | 同场景同画质同硬件与Unreal比较visual parity、CPU/GPU/frame/memory/upload；未胜出不得宣称优于Unreal |

## 17. 禁止的临时实现

1. 禁止只在HZB shader删除一次transform而不建立canonical bounds空间与generation合同。
2. 禁止继续用translation/scale单位球代表mesh bounds，或让CPU、GPU、LOD、streaming和shadow分别推导。
3. 禁止把单层grid改成更大固定cell/阈值并称为hierarchical spatial index。
4. 禁止以`Arc::make_mut`、全量clone/sort/map rebuild伪装incremental update。
5. 禁止关闭shadow后仍构造shadow visibility view，再依赖后端丢弃。
6. 禁止把union shadow bool当作逐viewsubmission truth。
7. 禁止保留无consumer的instancing/upload/HGI plan并以DTO存在声称feature完成。
8. 禁止把multi-draw称为true instancing；必须证明共享draw packet与`instance_count > 1`。
9. 禁止每个skinned实例持有专用palette buffer或以command-local bind group永久退出batcher。
10. 禁止在visibility前同步load、clone、变形、构造全部draw command和GPU args。
11. 禁止把GPU只过滤CPU完整command list称为完整GPU-driven renderer。
12. 禁止声明two-phase occlusion而只执行previous-frame单pass。
13. 禁止只用source-string测试证明shader空间、保守性或indirect写入正确。
14. 禁止在failed/cancelled/device-lost frame推进previous transform、HZB或LOD history。
15. 禁止用更大grow-only buffer掩盖allocator fragmentation、budget和retirement问题。
16. 禁止用compat facade、双写、永久CPU fallback保留旧scene/bounds authority；迁移必须hard cutover。
17. 禁止高级meshlet/software raster/async compute抢跑并绕过M1-M4基础合同。
18. 禁止以单场景FPS、draw count或单vendor路径证明达到/优于Unreal。

## 18. 本轮输出边界

本篇完成Runtime Visibility/Spatial Index/Bounds/Frustum/Occlusion/HZB/Culling/Batching/Instancing/GPU Scene/Indirect Submission/Instance Lifecycle的current-source E3静态审查，未实施production重构。Runtime09B的7项历史P0继续由原owner唯一计数，其中逐view identity和纯CPU indirect描述已有current-source纠偏；本文新增0项P0、48项P1、12项P2和G01-G48。

本轮未运行Cargo、Editor/App、真实交互场景、RenderDoc、Nsight、device loss、OOM、10K/100K soak或benchmark；现有真实WGPU测试只作为源码证据，不是本次动态验收。tooling按用户要求暂不纳入。实施必须从M0 characterization和M1 bounds/scene authority开始，再按M2-M7推进；在G01-G48形成BuildSet-bound产品证据前，不得把本报告标记为implemented，也不得声称可见性、GPU Scene、HZB或indirect submission达到或超过Unreal。
