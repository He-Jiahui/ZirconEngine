---
related_code:
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb
  - zircon_runtime/src/graphics/scene/scene_renderer/core
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider
  - zircon_plugins/virtual_geometry/runtime/src
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-14-graphics-gpu-scene-current-review.md
  - docs/plans/performance/01/2026-08-14-graphics-visibility-current-review.md
  - docs/plans/performance/01/2026-08-14-graphics-build-mesh-draws-current-review.md
  - docs/plans/performance/01/2026-08-14-graphics-mesh-pass-current-review.md
  - docs/plans/performance/01/2026-08-15-graphics-hzb-current-review.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingContext.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingMergedContext.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneVisibility.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteCullRaster.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/GPUResidentDrawer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/OcclusionCullingCommon.cs
  - dev/bevy/crates/bevy_pbr/src/render/gpu_preprocess.rs
  - dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh_preprocess.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/build_indirect_params.wgsl
  - dev/bevy/crates/bevy_pbr/src/meshlet/cull_bvh.wgsl
  - dev/bevy/crates/bevy_pbr/src/meshlet/cull_clusters.wgsl
  - dev/bevy/crates/bevy_pbr/src/meshlet/visibility_buffer_raster_node.rs
  - dev/godot/servers/rendering/renderer_scene_cull.cpp
  - dev/Fyrox/fyrox-impl/src/renderer/visibility.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: in_progress
source_recheck_required: true
---

# 09B · Renderer / Visibility / GPU Scene 工程化差距

## 1. 结论

Zircon 当前渲染器不是只有临时三角形路径。它已经有 `GpuScene` 的 primitive/instance/light storage、稳定 key 到 slot 的映射、dirty range 合并、三槽 staging ring、current/previous transform、morph、skin palette 与 Virtual Geometry payload；可见性层已经有 relevance、main/custom/shadow view、静态/动态空间索引、增量计划和局部 TaskPool frustum；mesh pass 已经有 phase command、static command cache、persistent indirect workspace、multi-draw/count replay；HZB 已经真实构建 mip、用上一帧 HZB compute compact indirect args，并把诊断回读改成默认关闭的有界异步队列。这些是后续重构必须保留的基础，不能退回逐物体直接 draw 的更简陋实现。

但这些局部机制没有形成 Unreal/Unity/Bevy 意义上的 persistent render scene 和 GPU-driven product authority。每帧仍先从当前 viewport extract 对全部 mesh 排序、克隆并展开 `PendingMeshDraw`，再做 Virtual Geometry plan、morph/skin、resident upload、light pack 和 `GpuScene` 全 draw 同步，之后才建立 mesh visibility state 和抽取 cached command。完全不可见的对象也已经支付材质解析、资产读取、变形、GPU resource 创建和 scene-data 比较。`GpuScene` 的产品注册对每个 pending draw 固定 `instance_count = 1`；visibility 生成的 `gpu_instancing_candidates`、`visible_instances`、`draw_commands`、`instance_upload_plan` 和 `particle_upload_plan` 在 graphics 产品渲染路径之外没有 consumer。当前所谓 GPU-driven 的主体只是 CPU 已构造命令的 indirect 提交格式，不是 GPU 生成可见实例、LOD、batch 和 draw command。

可见性还有直接的正确性和规模问题。`VisibilityStaticIndex` 实际是 `BTreeMap<Cell, BTreeSet<Id>>` uniform grid，不是 BVH；默认相机的保守球 AABB 会枚举 64,000 cells，超过 4,096 预算后退回全量扫描，因此 10,001 static fixture 的主视图预筛选在默认参数下不能成立。stable frame 仍重建 batch/tree/history 多份表；dirty frame在旧 snapshot 活着时会通过 `Arc::make_mut`复制整张 map。每个额外相机和 shadow view仍独立扫描候选，随后又把所有 shadow view并成一个 bool/HashSet，丢失 cascade/face identity。

更严重的是 HZB 使用的 GPU bounds 当前内部不一致。`primitive_data_for_pending_draw`把 model matrix 的 world translation写入 `bounds_center`，把 transform scale写入 `bounds_radius`；HZB shader又用 instance world matrix变换该center，并再次按world matrix缩放radius。纯平移会把center近似变成两倍平移，缩放会被近似平方；同时它完全没有消费mesh/model local bounds。这会让CPU frustum truth与GPU occlusion truth使用不同空间和几何范围，不能靠“保守bias”解释。修正并建立跨CPU/GPU bounds ABI之前，当前HZB结果不具备产品正确性证明。

Virtual Geometry 插件的体量和测试很多，但执行权威仍与名称不符。node/cluster cull pass在Rust中构造instance seed、cluster work item、最多8轮CPU hierarchy traversal、page request和traversal record，再把CPU结果创建为buffer；41行compute shader只把seed扩展成instance work item。`virtual_geometry_hardware_rasterization_pass::execute`没有开始render pass或draw，只从CPU selection收集record并创建buffer；`visbuffer64`同样从CPU selection生成entry和buffer。provider再以CPU `BTreeMap/BTreeSet`维护residency、slot、pending request并消费GPU readback。这里有真实GPU上传和少量compute，不是空模块，但“Nanite node/cluster GPU cull、hardware rasterization、VisBuffer authority”尚未成立。

本轮登记7项P0、14项P1、5项P2。P0先建立persistent render scene/change journal、正确bounds与generation identity、view-owned visibility bitset、early visibility ordering、真正被产品消费的GPU preprocess/cull/indirect，以及真实Virtual Geometry traversal/raster authority；P1再收敛GPUScene arena/upload、空间层次、多视图task、command/material cache、HZB pass、residency、诊断和测试；P2才进入Nanite级persistent-thread/两阶段遮挡、software/mesh-shader raster、large-world、多GPU和ray tracing scene同步。真实100k/1M规模、GPU capture、camera cut、device loss和soak证据完成前，不能把当前“GPU Scene/GPU-driven/HZB/Virtual Geometry complete”作为工程完成或性能优于Unreal的结论。

2026-08-26 P0-1 局部实现：`graphics/scene/render_scene/` 已建立独立 CPU scene owner，使用稳定代际 handle、密集 payload、swap-remove relocation、与 static-mesh eligibility 解耦的显式 revision、确定性 delta 与不可变 journal。进一步复核 extract 全链确认 camera LOD 会替换完整 model/mesh/material/primitive source，而不只是 `mesh_lod` 字段；因此新 primitive 使用 component-level identity，持有 base + 全部 LOD camera-neutral source，view 侧 O(log L) 选择，避免多视图污染 scene generation。19 个 focused tests 已编写且 scoped rustfmt/静态结构检查通过；Runtime04/extract delta producer、GPUScene journal consumer、view-family bitset、managed Cargo、规模基线和真实产品证据仍 open。详细记录见 `docs/plans/zircon_runtime/render/03/2026-08-26-persistent-render-scene-generation-architecture.md`。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 范围 | 文件 / 物理行 | `#[test]` | 证据等级 |
|---|---:|---:|---|
| `graphics/scene/gpu_scene` | 18 / 4,733 | 44 | E3：slot、history、upload、morph/skin/VG与产品sync |
| `graphics/visibility` | 62 / 5,212 | 38 | E3：context、relevance、view、grid、plan、query与VG plan |
| `scene_renderer/mesh` | 113 / 26,848 | 268 | E3：pending draw、cache、phase、indirect、replay、pipeline直接consumer |
| `scene_renderer/hzb` | 7 / 1,660 | 17 | E3：culler、workspace、bind cache、WGSL与readback |
| `scene_renderer/core` | 99 / 14,072 | 116 | targeted E3：构造、runtime prepare、compiled scene、submit和history owner |
| Virtual Geometry runtime plugin | 225 / 36,875 | 211 | E3 production 197 / 14,523行；E2 tests 28 / 22,352行 |

这些统计是本轮current worktree快照，不是稳定提交基线。上述四个主要renderer目录合计至少51处 `include_str!`；大量测试通过源码字符串、文件结构和 `.contains(...)`锁定实现形态。也存在有价值的headless WGPU、second-frame零upload、indirect workspace identity、visibility query和hierarchy行为测试，但测试数不能替代产品消费链、像素正确性、GPU capture和规模复杂度。

当前 `gpu_scene`、`visibility`、`mesh`、`hzb`、`core` 和 Virtual Geometry provider/plugin均有其他Session tracked modified或untracked文件；mesh/core尤其是大面积重构中。报告以读取时current source为准，不覆盖这些改动。实现前必须重取fingerprint、复核overlap diff、当前failure/output record和已有2026-08-14/15 current review，故标记 `source_recheck_required`。

### 2.2 参考引擎边界

- Unreal是persistent render scene、GPU Scene、Instance Culling和Nanite上限主参考。`GPUScene.cpp`从dirty primitive/instance filter生成pre-sized scatter upload，full scene upload是明确例外；`SceneVisibility.cpp`遍历scene octree并写packed visibility map，再只为visible/relevant primitive gather mesh；Instance Culling把visible instance compaction和indirect command build放进GPU pass并支持合并context；Nanite在GPU完成instance/node/cluster cull、两阶段HZB、page request、hardware/software raster和visibility buffer。Zircon不需要复制UE类层次，但必须复制“scene delta -> persistent slots -> per-view bitsets -> GPU compaction -> indirect execution”的所有权顺序。
- Unity Graphics是GPU resident scene与产品变更输入的辅助参考。`GPUResidentDrawer`通过ObjectDispatcher跟踪renderer/material/LOD/transform的update/deletion batch，持久拥有InstanceDataSystem、LODGroupDataSystem、InstanceCullingBatcher和OcclusionCullingCommon；occlusion compute直接更新indirect draw。它证明GPU resident不是“每帧先重建全部draw，再让最后一次write为0”。
- Bevy GPU preprocessing是WGPU可行性参考。它明确区分direct preprocess、GPU frustum和early/late occlusion preprocess，compute shader写instance output与indirect metadata，`build_indirect_params.wgsl`再生成indirect args；meshlet路径有persistent buffer、BVH/cluster cull和真实visibility-buffer hardware/software raster。Zircon不能用WGPU能力限制解释未消费的visibility plan或CPU物化的raster record。
- Godot `renderer_scene_cull`和Fyrox visibility是较小CPU引擎基线。它们没有Unreal级GPU-driven上限，但scene instance、dirty update、spatial tree/octree和renderer bundle/cache都有持久owner。Zircon当前uniform grid fallback和每帧多表重建甚至没有稳定跨过这个较低基线。

### 2.3 明确未做

- 初始审查没有修改production code；当前 P0-1/P0-3 已有局部 production 实现，但尚未接成 persistent product authority。没有运行Cargo、Editor、App、真实GPU、RenderDoc/PIX、WPR、GPU timestamp、camera-cut capture、device-loss、soak或规模benchmark。本篇仍不是实现验收。
- 没有在本篇逐审material compiler、shader permutation、texture/mesh streaming、lighting/shadow算法、post/temporal和runtime UI；它们进入09C以后单元。这里仅记录其与visibility/GPU Scene/command owner直接交叉的边界。
- 没有要求第一轮就实现Unreal全部Nanite、work graph、mesh shader或multi-GPU。P0要求的是正确authority、identity、bounds、delta、view和submission闭环；高级吞吐路径在这些边界稳定后进入P2。

## 3. 当前闭环与必须保留的能力

### 3.1 GPU Scene ABI、dirty queue和stable zero-write是可迁移基础

primitive/instance结构有显式size/alignment测试，update queue会原地排序并合并相邻dirty range，稳定shadow能把最终GPU write降到0，大upload有三槽grow-only staging ring。morph/VG/palette也已有current/previous和局部稳定判定。目标应把这些组件迁入scene generation owner和统一upload artifact，而不是删除后重新写逐draw uniform。

### 3.2 Indirect workspace与replay已经具备真实产品价值

phase-local workspace会grow-only复用args、metadata、visible remap、compacted args和draw count buffer；测试要求首次prepare创建、稳定二次prepare零create/零upload，单args变化只写20 bytes。replay支持multi-draw-indexed-indirect-count、固定multi-draw和逐drawfallback，并缓存pipeline/material/GPUScene/geometry状态。目标是让GPU culling生成其内容和range，而不是把这套基础退回direct draw。

### 3.3 HZB build/readback已从同步停机路径迈出

共享HZB mip确实由compute构建并服务SSAO/SSR与occlusion；occlusion diagnostics默认关闭，启用后通过最多4帧的异步readback queue，满时drop。params workspace和64项bind-group cache也减少稳定资源创建。这些机制应进入09A统一submission/ticket和本篇view generation，不应恢复`wait_indefinitely`或每帧bind group。

### 3.4 可见性与VG类型词汇可保留，但必须删掉平行假权威

typed view key、relevance bits、history snapshot、page request、runtime provider、residency slot和GPU completion DTO为后续边界提供了词汇。问题不是类型少，而是同一事实被batch/set/map/plan/debug snapshot多次物化，且若干输出没有consumer。重构应保留一个canonical artifact，其余改为borrowed view或删除。

## 4. P0 差距清单

### P0-1：没有persistent render scene，稳定帧仍从viewport draw重建场景数据库

`VisibilityContext::from_extract...`每帧按stable key排序mesh、构造renderable/static/dynamic集合、relevance、batch、BVH instance和history；`build_mesh_draws`再次展开pending draws、phase、material、deformation、VG与GPUScene entry。`GpuScene`注册和删除权属于当前viewport draw集合，camera变化也会进入camera-neutral数据比较。第二viewport会重复同一scene准备，而不是只生成第二套view visibility。

目标由Runtime04/Render03建立唯一 `RenderSceneGeneration`：persistent dense primitive/instance/material/geometry slots、slot generation、added/changed/removed journal、packed SoA和spatial hierarchy。game/world extract只发布delta与immutable asset handle；viewport只创建view descriptor和visibility result，不再register/retain GPU Scene。stable scene generation的sort/tree/key clone/register/retain/history scan必须为0，camera-only变化不得触发camera-neutral artifact rebuild。

### P0-2：空间预筛选不能降低全场景访问，多视图结果在提交边界退化为全流扫描

`VisibilityStaticIndex`是16-unit uniform grid。主视图仅在static >=10,000时尝试prefilter，却用camera-centered保守球AABB枚举cell；默认far=200、60度FOV、16:9得到40^3=64,000 cells，超过4,096预算后返回None。即使查询成功，`cull_main_view_with_static_index`仍遍历全部`bvh_instances`并对候选`BTreeSet`逐项查询，只减少部分frustum test而不减少scene-row访问。所谓incremental update又先把全部实例重建为`BTreeMap`；`collect_batching_result`和`build_bvh_update_plan`也逐帧重建全量集合，因此稳定场景仍是O(scene log scene)。每个custom/shadow view随后继续对全候选数组分配work/result `Vec`并独立frustum，且昂贵bounds test发生在layer/relevance过滤之前。

current source已纠正旧审查中的一处事实：`FrameVisibility`保留每个cascade/point face/spot的`Vec<u32>`与`VisibilityViewKey`，`ShadowAtlasSlotPass`也把该key传到产品pass，所以身份并未完全丢失。但mesh build先把所有shadow view union为`shadow_view_visible: bool`来生成一条全局shadow command stream；每个atlas slot随后重新物化`BTreeSet<EntityId>`、禁用global indirect、再扫描整条command stream过滤。问题是identity没有成为可直接消费的dense command range，而不是pass完全看不到view identity。

目标由`RenderSceneGeneration`持有dense primitive SoA和只消费change journal的空间结构；view输出以scene slot为索引的bitset加compact slot/command ranges，shadow slot直接消费自己的ranges，不再构造entity set或扫描全流。策略不能机械固定为octree：UE同时保留packed `PrimitiveBounds`、per-view `PrimitiveVisibilityMap`和scene octree，且`r.Visibility.FrustumCull.UseOctree`默认关闭。Zircon应按实测成本在小场景/高可见率的dense parallel scan与大场景/低可见率/多view的hierarchy traversal间选择；hierarchy路径直接遍历frustum planes，inside node批量接受，intersect才下探，overflow有独立有界列表。两条路径必须从同一view descriptor、layer/relevance、bounds ABI和scene generation派生，结果逐slot对拍一致。

2026-08-26 current-source review完成、实现未开始。优化前必须用WPR/xperf和renderer counters覆盖1/1k/10k/100k primitives，1/4/11 views，compact/sparse/large-bounds分布，0/1/1% dirty和0.1%/10%/100% visible；记录scene rows、node/leaf tests、frustum tests、set/map admissions、allocated bytes、task count、shadow command visits、CPU p50/p95和GPU pass time。当前managed产品构建仍被外部UI资产迁移阻塞，且visibility spatial-query、mesh build和shadow renderer存在其它worktree owner修改；在可复现baseline与owner边界稳定前不改索引算法，也不把HashSet替换或cell阈值调整记作P0-2完成。

### P0-3：GPU occlusion bounds发生双重变换，CPU与GPU可见性事实不一致

`gpu_scene_sync.rs`把model matrix translation写为primitive `bounds_center`，把matrix最大列长写为`bounds_radius`。`hzb_occlusion_cull.wgsl`随后用current/previous instance world matrix再次变换center，并用同一matrix再次缩放radius。它也没有读取mesh local center/radius。纯平移中心可近似变成2T，uniform scale半径近似变成S²；旋转/非均匀缩放误差更复杂。此数据进入previous-frame HZB false-cull判断，属于画面正确性问题，不只是性能问题。

目标由compiled mesh/model artifact发布local AABB/sphere和bounds revision；instance只持world transform。CPU frustum与GPU shader共享同一packed bounds ABI、finite/negative scale policy、skinned/morph conservative expansion和large-world origin。新增translated/scaled/rotated/nonuniform/skinned/morph fixture，逐实例对拍CPU reference、GPU readback和像素产物；修复前HZB不得作为final visibility authority。

2026-08-26 局部实施状态：GPUScene producer 已硬切为 `local_bounds_center/radius`，HZB 只按 instance current/previous transform 变换一次；skin、变化 morph、CPU morph、shear、退化/非法 bounds 在缺少 conservative history 时 fail-open。CPU visibility 仍使用资源解析前的代理球，CPU/GPU 对拍、managed WGPU/Naga、像素证据与性能证据尚未完成，因此 P0-3 保持未完成。实施证据见 [`../../zircon_runtime/render/03/2026-08-26-gpu-scene-local-bounds-hzb-abi.md`](../../zircon_runtime/render/03/2026-08-26-gpu-scene-local-bounds-hzb-abi.md)。

### P0-4：GPU-driven/instancing计划未进入产品路径，indirect只是CPU命令的提交格式

产品sync对每个pending draw调用 `gpu_scene.register(..., 1)`并写一个instance。visibility构造 `gpu_instancing_candidates`、`visible_instances`、`draw_commands`和`instance_upload_plan`，但graphics产品渲染路径没有读取这些字段；particle plan也没有renderer consumer。mesh pass仍由CPU逐draw解析pipeline/material/geometry，CPU batcher只把相邻同state command写成indirect args。带command-local GPUScene bind group的skin/palette draw直接拒绝indirect；transparent/transmission等路径也保留direct/fallback。

目标删除未消费的平行plan，或让其成为唯一 `GpuPreprocessInput`。persistent slot按compiled state bucket和geometry/material handle分组；GPU对per-view candidate执行frustum/LOD/occlusion，compact visible instance IDs，生成batch counts、instance remap和indirect args。CPU只建立changed generation的state bucket/range，不逐visible draw生成owned command。fallback必须有typed reason和counter，不能静默把“支持GPU-driven”解释为存在indirect buffer。

### P0-5：HZB只压缩CPU候选子集，算法和truth publication都没有工程闭环

HZB只处理opaque、alpha-mask、advanced opaque和velocity的indirect-capablephase；custom GPUScene bind group的draw被迫direct，因此不会被HZB compact。shader一个global invocation处理一个arg，却在单lane loop里串行遍历该arg全部instances；每个sphere只取中心UV一个mip sample。当前每arg通常又只有一个instance，隐藏了串行问题。结果只修改phase indirect workspace；`FrameVisibility`的 `occlusion_culled_count`始终为0，visible batches、spatial query、VG和Editor仍使用pre-HZB集合。

目标明确两套合法truth：same-frame GPU execution visibility和last-completed CPU-readable visibility。GPU cull使用workgroup/subgroup scan或分层prefix sum处理instances，投影screen rect并按closest/furthest约定做保守多点/区域测试；camera cut、resize、teleport、new instance、velocity和history invalidation有明确policy。所有indirect-capablephase可共享一次candidate/compaction dispatch，direct fallback也必须解释为何不能参与。readback只更新延迟diagnostics/query snapshot，不反向阻塞当前帧。

### P0-6：Virtual Geometry 的cull/raster/VisBuffer名称超前于实际执行权威

`execute_virtual_geometry_node_and_cluster_cull_pass`在CPU构造global state、seed、launch worklist、cluster work item、最多8轮child traversal、decision、page request和record；compute shader仅复制seed字段。`executed_cluster_selection`以CPU seed/indirect draw收集selection；所谓hardware rasterization pass只pack record到storage buffer，没有render pipeline/pass/draw；VisBuffer64同样pack CPU entry，没有栅格化attachment或atomic visibility write。snapshot rebuild还可从execution segment或snapshot fallback重建结果。大量test source证明DTO与顺序，却不能证明GPU authority。

目标把hierarchy node/child/cluster bounds、instance seeds、resident page table和work queues放入persistent GPU buffers；GPU persistent/bounded work queue完成node/cluster cull、LOD error、page request和visible cluster append。真实hardware/mesh-shader或compute software raster写depth+64-bit visibility buffer，material resolve消费该buffer。CPU reference只用于oracle/debug，不能参与正常结果生产；readback是延迟诊断/streaming feedback，不是下一步draw list的同步必要输入。未实现前应把pass/source名改成`CpuReference...`或明确`RecordMaterialization`，防止false green。

### P0-7：昂贵准备发生在可见性之后，完全不可见对象仍消耗资产、CPU和GPU资源

current `build_mesh_draws`顺序是phase ordered meshes、全部pending draw展开、VG indirect/resident、morph、light/GPUScene sync，然后才构造`mesh_visibility_states`和抽取cache。phase ordering只做camera layer，不消费main/shadow bitset。model/skinned路径可在render preparation同步load model/skeleton、重建bone map、CPU skin/clone primitive；CpuMorphed fallback还会在GPUScene sync中 `GpuMeshResource::from_asset`。material override和普通draw可创建多套bind group。late cache hit只省command尾部，fully culled对象已经支付前述工作。

目标入口改为 `RenderSceneGeneration + ViewVisibilityRanges`。静态cache/material/binding/geometry/deform artifact在scene或asset generation变化时构建；当前view只对visible ranges和真实shadow ranges产生dynamic command。GPU skin路径禁止先CPU skin，VG feature off为零长度borrowed artifact。fully culled对象的material probe、asset load、deform、VG、GPUScene compare、GPU object create和command build必须为0。

## 5. P1 差距清单

### P1-1：GPU Scene stable zero-upload仍付全draw、全map和全history访问

每帧为pending draws新建live `HashSet`和entry `HashMap`，逐项register、构造/比较primitive和instance、stage skin/morph并写revision；之后扫描stale key并retain多张map。submit返回后previous transform扫描全部entry/span，palette/source/morph做整map roll。`uploaded_bytes=0`只证明最终写省掉，不能证明稳定帧算法省掉。目标消费scene dirty journal，history以slot epoch/revision翻转；stable generation相关访问计数必须为0。

### P1-2：slot allocator、identity和资源退役没有generation/budget合同

primitive/instance id是可复用数值span，无slot generation；first-fit在线性free spans查找，commit pending frees会排序/全量归并，high-water buffer只增不减。free、previous roll和staging ring复用绑定CPU submit-return/帧序，而不是09A的device generation与GPU completion ticket。目标使用generation handle、paged/size-class allocator、fragmentation与budget telemetry；resize/compact产生relocation table并在相关submission完成后退役旧range/resource。

### P1-3：skin palette按实例保留多份固定16 KiB大对象

一个palette storage固定256个4x4 matrix加params，约16,400 bytes；每stable key有current/previous GPU buffer，CPU map和committed/staged storage又保留多份。1/64 active joints仍按最大容量拥有对象，变化上传也非统一arena。目标用device级current/previous suballocated arena，slot只持active range/revision；按dirty active prefix上传，buffer对象数不随skinned instance x2增长。

### P1-4：morph/VG payload与upload cost model仍是全量比较和单阈值

morph/VG stable frame仍slice equality扫描全部shadow；变化时找dirty runs后再复制完整shadow。staging只按256 KiB总bytes在direct和copy间切换，忽略range count/contiguity；morph/VG/palette还绕开统一staging。目标producer发布revision/dirty pages，形成唯一 `CompiledGpuSceneUpload`，按bytes、range count、contiguous ratio和backend测量选择direct、staging或GPU scatter。

### P1-5：uniform grid/BTree容器和Arc COW不是真正dirty-proportional spatial index

static/dynamic index都用同一grid，每cell是BTreeSet；query构造cell Vec和candidate BTreeSet。更新前先把current slice转BTreeMap；旧snapshot仍存活时任一 `Arc::make_mut`会复制整张entries/cells/overflow。目标以paged BVH/octree或Morton hierarchy维护slot leaf，dirty只重建受影响page/node；query用epoch bitset/reused scratch，避免per-query tree/set allocation。

### P1-6：多视图task策略固定且重复物化全N work/result Vec

frustum N>=64即并行、固定chunk=32，先构造完整work item Vec再构造结果Vec；views本身串行循环。阈值不看worker数、eligible数、view类型或task开销。目标先做cheap mask/hierarchy，再按eligible和worker budget切serial/parallel/GPU；task只写预分配bit ranges，禁止每view复制全N DTO。

### P1-7：visibility公开多组没有consumer或恒空的功能面

除未消费的instancing/draw/upload/particle plans外，`_hybrid_global_illumination`参数被明确忽略，active probes/update/feedback/requested全部构造成空值，却仍作为 `VisibilityContext` public fields发布。目标要么由HGI owner发布真实generation artifact并让visibility只消费必要probe bounds/bitset，要么删除这些字段；禁止长期保留看似已集成的恒空surface。

### P1-8：static command cache生命周期和命中成本不闭合

cache lookup命中会clone完整 `MeshDrawCommand`；cache在pending draw、material/deform/VG/GPUScene之后才生效。`retain_generation`只有定义和测试，没有产品调用，因此renderer生命周期内删除/卸载对象的旧command/resource handle没有显式retirement；若未来每帧调用它又会把暂时不可见项误当stale。目标由scene/material/geometry generation精确invalidate/retire，cache hit只返回arena range/handle、不clone raw WGPU resources；visibility变化不决定cache寿命。

### P1-9：dynamic geometry和binding仍可在per-draw/per-frame创建GPU对象

CpuMorphed skin fallback在GPUScene sync创建 `GpuMeshResource`；material override、normal/standard material和skin palette可按draw创建uniform/bind group。目标由asset/deform/material generation持久拥有resident geometry、palette slot和binding bundle，draw command只引用dense handle/dynamic offset；稳定unique material/mesh/palette组合create=0。

### P1-10：phase/indirect artifact虽有workspace，CPU plan与pass边界仍重复放大

CPU仍为多phase扫描command并构造batch/args/metadata；HZB四个phase分别clear三个buffer、开compute pass和dispatch。execution-owned原args、phase workspace、compaction和stats存在多层投影。目标以 `(view generation, command generation, HZB identity)`封存唯一CPU/GPU plan，phase只是range/header；同HZB下合并clear/dispatch，stats直接读sealed counters。

### P1-11：phase/material/pipeline resolution仍在frame热路径重复

material-adjusted phase queue按camera/frame重新物化和排序；pending draw和command build多次查询material common/revision/texture/uniform/variant并clone owned keys。目标让material/static-state generation发布compiled material id、pipeline variant id、binding handle和phase mask；view只排序透明深度或合并预编译range，opaque stable frame不重做variant/key/string工作。

### P1-12：HZB build计划与执行不一致，四phase也没有合批

`HzbBuildPlan`声明每pass最多4 mips，1080p附近应3个reduction batches；产品执行仍逐mip创建view/bind group/compute pass并dispatch11/12次。occlusion又按4phase重复clear/dispatch。目标实现1-4 mip batch kernel和UAV array/permutation，warm frame view/bind/upload create=0；同HZB identity的phase cull合成一次compute pass。

### P1-13：Virtual Geometry extract/residency/debug仍有同步CPU和重复事实源

provider自动extract按camera接收全mesh slice并允许同步load model，prepare以CPU visible clusters/segments构造resident/pending/available/evictable Vec；residency用BTreeMap/BTreeSet维护slot和request，GPU结果回读后再重建snapshot/selection/VisBuffer debug fallback。目标asset/scene generation预编译cluster/node/page dense tables，streaming owner持有唯一page table和ticket；camera只发布GPU seed/view，debug off不构造snapshot，debug on只采样/分页读取真实GPU结果。

### P1-14：测试与诊断不能证明产品GPU authority或规模复杂度

当前有大量源码字符串测试；HZB current review还发现测试仍要求已删除的params staging copy文本，执行即RED。Virtual Geometry测试代码22,352行，超过production 14,523行，但真实raster、visibility attachment、GPU traversal和capture并未因此成立。目标把source-shape tests降为少量architecture lint，增加CPU/GPU differential、pixel golden、indirect args readback、camera-cut、resource generation、device-loss、100k/1M benchmark和RenderDoc/PIX marker验证。

## 6. P2 差距清单

### P2-1：Nanite级两阶段遮挡与persistent-thread hierarchy traversal

核心P0稳定后，再实现previous-HZB main pass、current-depth post pass、occluded candidate replay、persistent thread/work queue、overflow retry和GPU budget scaling。不能在CPU 8-wave loop上继续增加Nanite命名。

### P2-2：hardware mesh shader与compute software raster双路径

按triangle size/material capability选择hardware/mesh-shader/software raster，统一写64-bit visibility/depth并做material resolve；需要wave/subgroup、barycentric、atomics与平台fallback矩阵。普通indexed hardware draw仍是baseline fallback。

### P2-3：large-world、portal/room与超大view-family spatial hierarchy

支持world partition cell、origin rebasing、streaming bounds、portal/room/PVS、reflection/probe/scene-capture view family和64+ shadow views；hierarchy和visibility bitset必须分页、可调度、可取消，不能复制单世界全表。

### P2-4：ray tracing instance scene与raster visibility共享persistent identity

BLAS/TLAS build/refit、instance mask/SBT/material identity应消费同一scene delta和slot generation；不得从raster draw重新构造第二套scene。具体实现依赖09A真实RHI acceleration-structure contract。

### P2-5：multi-GPU、work graph与meshlet streaming specialization

device-group residency、cross-GPU visibility、work graph/shader bundle、meshlet page compression和platform-specific fast path只能在单device正确性、budget和fallback稳定后开展；每条高级路径必须保留相同artifact合同和可对拍fallback。

## 7. 目标架构

### 7.1 唯一产品链

```text
World / asset changes
        |
        v
RenderSceneChangeJournal
        |
        v
Persistent RenderSceneGeneration
  primitive/instance slots + bounds + geometry/material/deform handles
  spatial hierarchy + cached command/state buckets + resident page tables
        |
        +-------------------+
        |                   |
        v                   v
ViewFamily descriptors   GPU Scene dirty upload artifact
        |                   |
        v                   v
CPU hierarchy mask / GPU candidate ranges
        |
        v
GPU preprocess: frustum + LOD + HZB + instance/cluster compaction
        |
        +----------------------+
        |                      |
        v                      v
Indirect phase ranges     Virtual Geometry raster/VisBuffer
        |                      |
        +-----------+----------+
                    v
            Render Graph execution
                    |
                    v
      submission ticket / delayed feedback / diagnostics
```

只有 `RenderSceneChangeJournal`可以新增、更新、删除persistent scene slot；只有view family可以产生per-view visibility；只有GPU preprocess/明确CPU fallback可以产生本帧visible remap和indirect count；只有09A submission coordinator可以提交与退役资源。Debug、Editor、stats和readback只能观察canonical artifact，不能重演cull或另建权威draw list。

### 7.2 核心身份

- `RenderSceneSlot { index, generation }`：跨帧primitive/instance稳定身份，删除后旧generation不可复用。
- `RenderSceneGeneration`：camera-neutral scene artifact版本；记录dirty pages与asset/material/deform revisions。
- `ViewGeneration { family, view, history_epoch }`：camera、layer、frustum、HZB history和shadow slot身份。
- `GpuSceneAllocation { device_generation, arena, range, slot_generation }`：只由GPU Scene arena owner分配和退役。
- `MeshCommandGeneration`：compiled state bucket、phase range、binding/geometry handles和cache revisions。
- `VirtualGeometryPageTicket`：page generation、physical slot、upload submission和completion/eviction终态。

## 8. 重构里程碑

### M0：冻结事实源、修正命名与建立测量基线

重新取current source fingerprint；把现有Render02/03/04与performance current review中的open项映射到本篇P0/P1；删除/标记未消费visibility plan，给CPU物化VG pass改准确名称；修复stale source-shape tests。新增stable/dirty/full、pre/post-cull、per-view、GPUScene、indirect、HZB、VG、GPU object与bytes counters。此阶段不以微调BTree/Hash阈值代替架构修复。

### M1：建立persistent RenderScene与change journal

Runtime04发布asset/scene delta；Render03持有packed primitive/instance slots、bounds/material/geometry/deform handles和remove journal；Render04空间hierarchy只消费delta。迁移期间旧frame rebuild与新artifact逐帧对拍，最终硬切并删除旧owner。stable generation rebuild/sort/map/key clone=0。

### M2：修正bounds、slot generation和GPU lifetime

compiled asset发布local bounds；CPU/GPU共享ABI并通过differential tests。所有scene/GPU/command/page handle加入slot/device generation；free/resize/history roll接09A submission completion，旧resource deferred retire。完成camera cut、device recreate和stale handle fault tests。

### M3：view family、spatial hierarchy与per-view dense result

替换uniform-grid球查询与全row membership scan，发布同一scene generation上的dense scan/hierarchy双策略；只有量化数据决定分界。main/custom/cascade/face各有slot-indexed bitset、compact command ranges和stats。cheap layer/relevance先行，hierarchy inside node批量接受、intersect下探；TaskPool按eligible/workers预算。删除shadow union bool对全局command admission的控制，shadow pass直接消费view-local ranges；spatial query借用generation bitset和有界scratch。

### M4：early visibility与static command/material artifact硬切

让view visibility在dynamic material/deform/VG准备前生效。Render08/02按material/static-state generation编译pipeline variant、binding和command range；visible static只引用range，dynamic visible才进入frame arena。删除per-draw stable GPU object创建、late cache clone和无retirement cache。

### M5：GPU Scene arena与统一dirty upload

primitive/instance/palette/morph/VG/light进入paged/grow-only arena；scene delta生成唯一upload artifact。按range/bytes/contiguity选择direct/staging/scatter；history用epoch/range，不全map roll。记录fragmentation、capacity、retired bytes和upload cause，stable scene访问/compare/upload均为0。

### M6：产品GPU preprocess、instance cull和indirect authority

把compiled state buckets与per-view candidate ranges送GPU，完成frustum/LOD/HZB、visible instance scan/compact、batch count和indirect args build。支持same-frame early/late occlusion与合法fallback；所有phase消费同一generation artifact，indirect workspace继续复用。删除没有consumer的CPU draw/upload plans。

### M7：Virtual Geometry GPU authority硬切

asset generation编译hierarchy/cluster/page表；GPU bounded/persistent queue做node/cluster traversal与page request；真实raster写depth/VisBuffer64，material resolve消费。CPU reference只保留test/debug oracle。residency与page table由ticket/completion驱动，debug snapshot不参与结果生产。

### M8：预算、fallback、故障与多viewport闭环

为scene slots、visible candidates、indirect args、VG queues/pages、readback、cache、task和GPU bytes设软/硬预算；overflow有typed fallback、retry或quality degradation。覆盖2/8 viewport、64 views、resize/cut、device loss、OOM、page starvation、readback backlog和shutdown，所有ticket exactly-one terminal。

### M9：产品验收与旧路径删除

运行focused/unit/property/headless WGPU、F2像素回归、100k/1M benchmark、WPR/xperf、GPU timestamp、RenderDoc/PIX和长时soak。只有动态门通过后删除旧frame-rebuild、CPU VG materialization和parallel side plans；更新Render02/03/04状态，不以测试文件存在或局部marker作为完成证据。

## 9. 验收矩阵与硬门

### 9.1 场景矩阵

| 维度 | 样本 |
|---|---|
| primitives / instances | 0、1、1k、100k、1M |
| visible ratio | 0、0.1%、1%、10%、100% |
| scene dirty | 0、1 item、1%、100%、50% churn |
| views | 1、4 cascades、12 mixed、64、2/8 viewports |
| geometry | static、dynamic、skinned 1/64/256 joints、morph 0/8/64、VG 1k/1M clusters |
| materials/state buckets | 1、100、1k；opaque/masked/transparent/custom binding |
| HZB | off、history absent/stable、resize、cut、teleport、MSAA1/4 |
| faults | buffer grow、allocator fragmentation、queue backlog、OOM、device loss、page starvation |

### 9.2 必须记录

- scene delta counts、stable full-scan visits、sort/tree/hash probes、key/DTO clone bytes；
- spatial visited nodes/cells/candidates、per-view tests、bitset/range bytes和fallback reason；
- material/asset/deform probes、CPU vertex/bone/morph visits、GPU object/bind create；
- GPUScene dirty ranges/bytes、history visits、allocator comparisons/fragmentation/retired bytes；
- GPU preprocess candidates/visible/culled、dispatch、scan/compact bytes、indirect/direct fallback reason；
- HZB build/cull pass、dispatch、mip sample、history invalidation和false-positive/false-negative oracle；
- VG node/cluster work、queue high-water、page request/residency/eviction、raster/VisBuffer pixels；
- main/render/worker CPU p50/p95/p99、task queue age、CSwitch/ReadyThread、GPU timestamp、VRAM、energy。

### 9.3 硬门

- stable scene generation：scene/batch/index/history/GPUScene/command/VG rebuild、sort、full scan、payload compare、GPU create/upload均为0；
- camera-only变化：camera-neutral artifact work/upload为0，只产生view artifact；
- fully culled对象：material/asset/deform/VG/GPUScene/command准备为0；
- 1% dirty：CPU访问、copy、upload与受影响slot/page/node近线性，不复制全scene map；
- translated/scaled/rotated bounds：CPU/GPU可见性对拍，HZB不得出现false occlusion；
- per-shadow-view：4 cascades/6 point faces保持独立draw set，disabled shadow view为0；
- real instancing：64同mesh/material实例不注册64个one-instance primitive，GPU compact并生成正确args；
- HZB：camera cut/resize/new instance保守正确，diagnostics off无copy/map，四phase同HZB可合批；
- VG：正常产品帧CPU hierarchy traversal/selection/raster record build为0，真实GPU pass写depth/VisBuffer；
- stats与RenderDoc/PIX events、GPU readback和最终像素一致，任何fallback有typed cause/counter。

## 10. 禁止的临时实现

- 禁止把每帧全scene扫描并行化后称为persistent render scene；
- 禁止继续增加没有product consumer的plan/DTO/manager并以类型存在表示完成；
- 禁止用固定阈值、扩大cell预算或更换HashMap掩盖错误owner/复杂度；
- 禁止CPU先生成完整visible/draw/raster结果，再让GPU复制、compact或回读后称为GPU-driven；
- 禁止用world-space center/radius写入local-space ABI，或让CPU/GPU使用两套bounds；
- 禁止GPU skin路径先CPU skin/clone primitive，禁止stable draw创建mesh/bind group/buffer；
- 禁止把debug snapshot、neutral fallback或source-shape test作为产品执行证据；
- 禁止用submit返回代替GPU completion，或用裸WGPU handle/raw address做跨generation cache identity；
- 禁止为通过旧测试恢复同步readback、per-mip/per-phase资源创建或已经删除的staging copy；
- 禁止在P0 owner/identity/bounds完成前投入work graph、mesh shader或多GPU表面工程。

## 11. 既有计划需要重开的状态

| 既有计划 | 需要重开的内容 | 关闭条件 |
|---|---|---|
| Render02 Mesh Draw Command | cache太晚、命中clone、无retirement、parallel只覆盖尾部、phase arena非唯一 | M4/M6 stable command generation和visible range产品证据 |
| Render03 GPU Scene/GPU-driven | viewport draw注册、one-instance、全history scan、palette/VG arena、CPU indirect authority | M1/M2/M5/M6 dynamic gates |
| Render04 Visibility/Culling | 默认prefilter失败、per-view结果丢失、HZB truth未发布、CPU VG plan | M3/M6/M7 dynamic gates |
| Render05 Lighting/Shadows | shadow visibility union、view/slot不一一对应 | 每cascade/face独立bitset到pass replay |
| Render09 Camera Ordering | multi-camera重复scene prepare、view family无共享artifact | camera-neutral build一次、view-owned ranges |
| Runtime04 Asset Pipeline | render prepare同步load/compile mesh/skeleton/morph/VG | compiled artifact按asset generation发布一次 |
| Render17 Performance | source-shape/局部counter无法证明product authority | 第9节全counter、capture与规模基线 |
| Optimize09A RHI/GPU Lifetime | device generation、submission ticket、completion retirement未接入renderer | M2/M8故障与retirement证据 |

这些计划继续是implementation owner；本篇不是新建平行renderer架构。实现记录应回写对应编号计划，并在本篇只维护current-source差距状态。
