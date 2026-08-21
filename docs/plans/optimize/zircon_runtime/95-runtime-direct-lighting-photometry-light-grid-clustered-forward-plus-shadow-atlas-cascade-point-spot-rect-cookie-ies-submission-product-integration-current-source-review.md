---
title: Runtime Direct Lighting、Photometry、Light Grid、Clustered Forward+、Shadow Atlas、Cascade、Point/Spot/Rect、Cookie、IES、Submission 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime95
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/asset/assets/scene/lighting.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/core/framework/render/light
  - zircon_runtime/src/core/framework/render/advanced_lighting/cookie.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr_basic.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_light_cookie.wgsl
  - zircon_plugins/rendering/features/contact_shadow/runtime/src
tests:
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs
  - zircon_runtime/src/graphics/tests/render_product_shadows/many_point_lights.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_wide.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs
  - zircon_runtime/src/scene/tests/render_extract/lighting_postprocess.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/71-runtime-scene-light-directional-point-spot-rect-photometry-layer-shadow-cookie-ies-extract-authoring-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightGridInjection.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightGrid.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowSetup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowDepthRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/LightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/LocalLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/RectLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/IESTextureManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RectLightTextureManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VirtualShadowMaps/VirtualShadowMapArray.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VirtualShadowMaps/VirtualShadowMapCacheManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VirtualShadowMaps/VirtualShadowMapCacheManager.cpp
  - dev/bevy/crates/bevy_light/src/directional_light.rs
  - dev/bevy/crates/bevy_light/src/point_light.rs
  - dev/bevy/crates/bevy_light/src/spot_light.rs
  - dev/bevy/crates/bevy_light/src/cluster/mod.rs
  - dev/bevy/crates/bevy_light/src/cluster/assign.rs
  - dev/bevy/crates/bevy_pbr/src/cluster/gpu.rs
  - dev/bevy/crates/bevy_pbr/src/cluster/cluster_allocate.wgsl
  - dev/bevy/crates/bevy_pbr/src/cluster/cluster_raster.wgsl
  - dev/bevy/crates/bevy_pbr/src/cluster/cluster_z_slice.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/light.rs
  - dev/bevy/crates/bevy_pbr/src/render/shadow_sampling.wgsl
  - dev/godot/servers/rendering/renderer_rd/cluster_builder_rd.cpp
  - dev/godot/servers/rendering/renderer_rd/cluster_builder_rd.h
  - dev/godot/servers/rendering/renderer_rd/storage_rd/light_storage.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/area_lights_inc.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/scene_forward_lights_inc.glsl
  - dev/Fyrox/fyrox-impl/src/scene/light/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/light/directional.rs
  - dev/Fyrox/fyrox-impl/src/scene/light/point.rs
  - dev/Fyrox/fyrox-impl/src/scene/light/spot.rs
  - dev/Fyrox/fyrox-impl/src/renderer/light.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shadow/csm.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/LightLoop/LightLoop.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Light/HDAdditionalLightData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/LightUnitUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/LightCookieManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/LightLoop/CookieSampling.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/LTCAreaLight/LTCAreaLight.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/ContactShadows.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/HDShadowManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/HDShadowAtlas.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/HDCachedShadowAtlas.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/HDCachedShadowManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ForwardLights.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/AdditionalLightsShadowAtlasLayout.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Direct Lighting、Photometry、Light Grid、Clustered Forward+、Shadow Atlas、Cascade、Point/Spot/Rect、Cookie、IES、Submission 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon的直接光照与阴影并非纯占位。当前已有固定128-byte `GpuLightData`、GPU Scene的light range upload、被forward/deferred/froxel实际读取的CPU z-bin/tile bitmask grid、带retention/preemption/slot generation的shadow atlas allocator、稳定cascade split与texel snapping、方向光级联/点光六面/spot视图、PCF sampling、真实WGPU shadow capture，以及同时校验light参数、static caster revision和atlas slot generation的`ShadowCache` identity kernel。这些底座应保留为characterization oracle并迁入统一generation架构。

但从普通项目资产到最终像素的工程闭环仍未成立。四类直接光组件和scene asset没有shadow、cookie、IES、单位或照明通道authoring；world extract继续对方向光、点光、聚光和矩形光写`shadow: None`。产品测试通过手工构造`RenderSceneSnapshot`和`LightShadowSettings`绕过该断链。`RenderLightReadinessReport`又把全部directional/point/spot按数量直接记为ready，无法表达字段失效、cluster截断、atlas拒绝、cookie缺失、pipeline/device失败或实际submission结果。

默认所谓“Clustered Lighting”仍存在双重truth。真正被材质消费的grid在CPU每帧重新pack、分配并三次`queue.write_buffer`；随后标成`AsyncCompute`的另一条路径只把有限方向光汇总成二维颜色buffer，render graph却宣称它写了grid params、zbins、tile masks和light list。CPU grid还会静默截断至65,535盏光，根据bitmask预算不断放大tile，按sphere处理spot/rect，并在光心`clip.w <= 0`时丢弃near-plane crossing或camera-inside的大光源；orthographic half-height又被再次乘0.5。

直接着色也没有达到物理或形状正确性。forward basic、forward full和deferred各自复制punctual loop与`(1-d/r)^2`衰减；Rect Light虽然pack了width/height，着色只把它当带单面dot项的point light。layer mask不被surface/light或caster消费，shadow `strength`和`normal_bias`只被打包而未进入最终采样。cookie具备手工extract DTO和固定atlas，但没有scene authoring producer；atlas每帧重建、固定8x8/64格、无priority/residency/overflow receipt。仓内没有IES runtime contract、artifact、atlas或shader consumer。

shadow planner与visibility仍是两套独立authority。planner只选择第一盏投影阴影的方向光，固定0.1 near和默认4级/150米级联；visibility却提前为所有方向光建立视图，关闭shadow的方向光仍建一个，同时为所有enabled point/spot建立六面/单面视图而不等待atlas接受。方向光级联绕过allocator并伪造每帧generation。`ShadowCache`只有类型、policy与测试，生产renderer不读取decision；graph每帧`clear_store`整张atlas，逐slot创建uniform buffer/bind group/pass name，按`BTreeSet`过滤后重放完整shadow command stream。缓存内容不可能在当前提交模型中留存。

可选Contact Shadow插件也不具备逐光源语义。它没有world reconstruction、light ID/type/direction或逐光源ray参数，只比较12个邻域depth、一个HZB mip和`normal.z`，输出单通道visibility，随后post process把该值乘到已经包含ambient、IBL、baked、reflection与emissive的整张scene color。这不是Unreal/HDRP/Bevy意义上的contact shadow，而是全画面后置遮蔽。

Runtime09E登记的10项P0在当前源码中全部仍开放；Runtime71的两项light authoring P0也仍可复现。本文不重复登记这些父owner P0，只给出current-source状态和可执行子要求。本篇新增 **0项P0、48项P1、12项P2与48个资格门**。在authoring roundtrip、真实GPU cluster、统一shadow assignment/view/cache、物理光度、面积光、cookie/IES、过载/fault/device-loss和同硬件同画质基准闭合前，不得声称直接光照、阴影性能或表现达到或优于当前Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 证据等级 | fingerprint |
|---|---:|---|---|
| Zircon direct lighting / grid / shadow / cookie / contact产品语料 | **97 / 21,581 / 20,101 / 848,142 / 146** | E3逐文件覆盖authoring、extract、ABI、shader、grid、plan、visibility、atlas、cache、graph与submission | `41878c168236e1c703bd1c402e10dbaab67f061ef2e08db6f457bb5f3813f545` |
| focused product tests | **7 / 2,997 / 2,836 / 107,404 / 22** | E3读取shadow graph、capture、wide、many-light与scene extract产品测试 | `f8bf45553f362bb5e74ee27c98b55ad091323b0be0f6c22b35e274d769ac7db8` |
| 五引擎参考切片 | **48 / 50,572 / 43,415 / 2,180,867 / 0** | E2/E3读取Unreal light-grid/shadow/VSM、Unity HDRP/URP、Bevy cluster/shadow、Godot RD、Fyrox light/CSM | `46c6ee7e634b80f60a93d4adbe728365cdcefad20dd5c822dc3e3e58fd2d4846` |

Zircon语料由frontmatter `related_code`中的目录及显式文件，加上两条scene renderer shadow-plan caller、light-grid stats和froxel direct-light consumer组成；focused tests不重复计入独立测试表。fingerprint按规范化小写路径与逐文件SHA-256组成manifest后再次SHA-256。冻结代表2026-08-21共享working tree，不是只读HEAD或实现验收receipt。

Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator epoch为336。Bevy、Fyrox、Godot与Unity Graphics revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`和`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像由本表aggregate fingerprint冻结。

冻结时`core/framework/render/light/mod.rs`与`readiness.rs`存在其他Session的working-tree修改。本文读取并计入这些版本，但不拥有也未修改它们，因此`source_recheck_required: true`。实施前必须重算fingerprint并重验readiness字段、light public export、pack ABI和所有shadow plan调用点。

### 2.2 Owner边界

| 边界 | 本篇所有 | 既有owner继续所有 |
|---|---|---|
| Scene light source | 只定义renderer所需有效descriptor与receipt | Runtime71拥有component/asset/property/editor authoring、默认值、transform与migration；其2项P0不在本文重复计数 |
| Direct lighting | prepared light generation、photometry conversion消费、shape evaluation、light linking与shared shader module | Runtime91拥有shader module/permutation/PSO；Runtime92拥有texture/cookie/IES residency |
| Light assignment | CPU characterization、真实GPU clustered/Forward+、overflow/capacity、per-view list generation | Runtime94提供persistent RenderScene、bounds和view generation；Runtime89/90提供graph/RHI/queue truth |
| Shadow | policy arbitration、planned views、atlas/page allocation、cache、depth submission、sampling与receipt | Runtime65拥有quality/device profile/frame budget；Runtime62/94拥有caster lifecycle/visibility source |
| 历史P0 | current-source重验、拆分资格门、禁止错误关闭 | Runtime09E继续唯一计数10项P0；Runtime71继续唯一计数2项authoring P0 |

### 2.3 明确未做

本轮没有修改production code、shader、Cargo、资产或插件，没有运行Cargo、Editor、真实GPU、PIX/RenderDoc、WPR、device loss、atlas thrash、过载、camera-inside、跨平台视觉golden或长时间soak。静态审查能证明字段、调用链、资源访问和算法契约断裂，不能证明任何硬件上的最终耗时、画质、闪烁或稳定性。

## 3. 当前应保留的真实基础

1. `GpuLightData`具有固定stride、对齐与布局测试，GPU Scene具备light dirty-range upload；它应升级为versioned `PreparedLightingGeneration`的单一payload，而不是废弃。
2. CPU z-bin/tile bitmask grid确实被deferred、forward PBR与froxel读取，可作为GPU cluster重构的correctness oracle和低端fallback。
3. shadow atlas allocator已有tier downgrade、retention、preemption hysteresis、rejection与slot generation，是可用的policy kernel。
4. cascade split、frustum corner fit与texel snapping具备数学测试，可迁入统一planned shadow view owner。
5. `ShadowCacheInput`的light hash、static caster revision、slot generation三因子identity方向正确，应接入真实persistent depth内容和submission receipt。
6. shadow graph、forward/deferred receiver、directional capture、subtexel stability、mixed spot/directional和64/128 point light测试提供了有价值的characterization，但必须从手工snapshot扩展到真实authoring入口。

## 4. 历史P0 current-source状态

### 4.1 Runtime09E十项P0

| 父finding | 当前状态 | current-source证据与关闭要求 |
|---|---|---|
| 09E-P0-1 shadow authoring不可达 | 开放 | 四类component/asset无shadow字段，world extract四处`shadow: None`；必须用asset -> World -> extract -> plan -> capture roundtrip关闭 |
| 09E-P0-2 Clustered Lighting compute是假契约 | 开放 | CPU写真实grid，方向光颜色compute另写`LIGHT_LIST`，descriptor虚报四资源write和AsyncCompute；必须删除假路径或以真实GPU assignment硬切换 |
| 09E-P0-3 camera-inside/near/ortho grid错误 | 开放 | `clip.w <= 0`直接拒绝，ortho half-height再次乘0.5；必须用canonical projection conservative bounds和GPU/CPU差分关闭 |
| 09E-P0-4 light layer未被receiver/caster消费 | 开放 | mask只pack到`shadow_slot_layer.y`，direct/shadow WGSL不读；必须贯穿surface、cluster、caster、volumetric和debug |
| 09E-P0-5 photometry/attenuation非物理 | 开放 | unitless defaults与三份`(1-d/r)^2`；必须有lux/lumen/candela/nit contract、inverse-square和reference fixture |
| 09E-P0-6 Rect Light仍是point近似 | 开放 | width/height进入ABI但shader仅做单面dot；必须以LTC或经验证等价算法消费真实area geometry |
| 09E-P0-7 strength/normal bias/readiness是假完成 | 开放 | strength和normal bias不进采样，readiness按count；必须以最终submitted generation与pixel effect关闭 |
| 09E-P0-8 shadow plan/visibility双authority | 开放 | planner只取第一方向光，visibility提前建全部view；必须先仲裁allocation再生成唯一planned view set |
| 09E-P0-9 shadow cache未接入且atlas全清 | 开放 | cache仅自身/plan tests消费，graph每帧`clear_store`；必须证明跨帧内容保留、静态reuse和动态overlay |
| 09E-P0-10 contact shadow是全画面post occlusion | 开放 | shader无light/world ray语义，post乘整张scene color；必须迁入逐光源direct visibility或删除错误能力名 |

### 4.2 Runtime71两项P0

| 父finding | 当前状态 | current-source证据与关闭要求 |
|---|---|---|
| RSL-P0-001 Spot asset默认与runtime相反/退化 | 开放 | `SceneSpotLightAsset`仍复用Rect的1,000,000强度、20 range和+Y/零cone默认；必须version migration并做partial asset parity |
| RSL-P0-002 单一NodeKind隐藏多个同时生效light component | 开放 | renderer按component store分别收集，多light component可同时生效；必须强制互斥或显式multi-emitter schema |

## 5. 当前产品链与目标架构

当前链路：

```text
Scene component / asset
  -> World按family全量collector
     -> shadow=None，camera layer预过滤
  -> mesh build重新pack light + cookie metadata
     -> shadow planner独立分配slot
     -> GPU Scene light upload
  -> visibility提前独立猜shadow views
  -> light-grid pass再次pack -> CPU Vec grid -> 3x queue.write_buffer
  -> 假cluster compute写未参与direct loop的directional color summary
  -> shadow graph每帧清atlas -> per-slot alloc/bind/replay
  -> forward/deferred/froxel各自direct loop与shadow sampling
  -> post process把Contact Shadow乘到整张scene color
```

目标链路：

```text
SceneLightDescriptorGeneration + asset dependency generations
  -> PreparedLightingGeneration
     stable light handle / canonical photometry / validated shape / channels
     cookie + IES residency result / source and effective quality
  -> ViewFamilyGeneration
  -> LightAssignmentGeneration
     GPU cluster lists + overflow/capacity receipt + deterministic fallback
  -> ShadowPolicyGeneration
     importance/cost/history/budget arbitration
  -> ShadowAllocationGeneration
     atlas/page allocation + stable slot/page generation
  -> PlannedShadowViewGeneration
     only accepted light/face/cascade views + qualified caster sets
  -> ShadowDepthSubmissionGeneration
     cached static reuse + dynamic overlay + batched depth packets
  -> DirectLightingPacketGeneration
     shared BRDF/shape/attenuation/cookie/IES/shadow/channel module
  -> RenderGraph -> RHI SubmissionTicket -> completion-qualified receipt/history
```

目标owner不得用裸数组位置连接阶段。每个阶段至少携带`scene_generation`、`view_family_generation`、`lighting_generation`、`assignment_generation`、`shadow_policy_generation`、`allocation_generation`、`resource_generation`与`device_generation`，任何不匹配必须fail closed、保留last-good或显式degrade，不能静默消费过期slot或texture。

## 6. P1差距与重构完成定义

### 6.1 Source、generation、ABI与readiness

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RDL-P1-001 | component/asset到render snapshot没有共同versioned effective descriptor | Runtime71发布validated descriptor generation；graphics只消费它，不再按family重新解释字段 |
| RDL-P1-002 | shadow、cookie、IES、photometry与channel没有完整source presence/provenance | 每字段保留source/effective值、默认/迁移来源、dependency generation与degrade reason |
| RDL-P1-003 | mesh build和light-grid各自调用pack，存在多份frame-local light truth | 单一`PreparedLightingGeneration`只pack一次，所有consumer按generation只读同一buffer/index table |
| RDL-P1-004 | cookie metadata覆盖directional offset与rect/spot共用ABI槽位 | versioned ABI为shape、cookie、IES、shadow、channel提供独立字段或typed indirection，layout reflection测试闭合 |
| RDL-P1-005 | 64-bit light ID只保留低32位写GPU字段 | 使用generation-qualified dense index与完整stable handle映射；截断、复用和wrap有检测 |
| RDL-P1-006 | volumetric membership以`Vec::contains`逐光查找且作为sideband truth | participation预编译成bit/typed index，zero-change frame不重复O(light×volumetric)查询 |
| RDL-P1-007 | readiness对directional/point/spot按总数直接ready | per-light readiness来自validate、dependency、assignment、shadow allocation、pipeline、device和submit receipt |
| RDL-P1-008 | 无跨cluster/shadow/cookie/device的generation一致性检查 | packet materialization逐代校验，stale/missing输入返回typed failure并进入diagnostics，不得猜默认成功 |

### 6.2 Photometry、shape、shader与channels

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RDL-P1-009 | 方向/点/聚光/矩形光共享unitless `intensity`但默认尺度不一致 | canonical lux/lumen/candela/nit及solid-angle/area转换，source unit可roundtrip，GPU只读effective radiometric量 |
| RDL-P1-010 | 三套shader复制punctual attenuation和light loop | 由Runtime91维护一个可组合direct-light module，forward/deferred/froxel对共同case的数值差异有容差测试 |
| RDL-P1-011 | bounded falloff替代了inverse-square物理衰减 | inverse-square为默认，range仅做平滑cutoff；pre-exposure、单位缩放和极近距离source radius处理明确 |
| RDL-P1-012 | Rect Light width/height不参与direct evaluation | 采用LTC或经reference fixture验证的area integration，支持orientation、sidedness、barn door、source texture |
| RDL-P1-013 | Spot/Rect cluster bounds只用range sphere | spot使用conservative cone/large-angle fallback，rect使用oriented box/frustum，CPU/GPU bounds差分为0漏光 |
| RDL-P1-014 | `shadow_params.x` strength与slot normal bias不影响最终visibility | strength参与visibility blend；normal/slope/receiver bias在统一坐标和单位中进入投影/采样并有像素回归 |
| RDL-P1-015 | lighting layer只做camera预过滤，surface与caster无共同mask | `LightingChannelMask`贯穿primitive/light/cluster/direct/caster/volumetric；camera只做候选粗滤 |
| RDL-P1-016 | color没有temperature/tint、working-space与negative/finite资格 | scene admission完成color conversion与finite validation；shader不再对非法值临时容错或传播NaN |

### 6.3 Light grid、clustered Forward+与过载

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RDL-P1-017 | CPU builder每帧分配zbins、tile masks、min/max和统计scratch | persistent per-view workspace按capacity增长并复用；steady frame allocation与全量upload预算可验收 |
| RDL-P1-018 | light count静默截断到`u16::MAX` | capacity由设备/quality profile决定，溢出返回accepted/rejected名单、原因和可预测fallback，不得静默丢灯 |
| RDL-P1-019 | bitmask随light count线性增长，预算不足只不断放大tile | GPU compact index list/linked-list/two-level策略按平台选择，memory/cell overflow有feedback与resize/degrade |
| RDL-P1-020 | `clip.w <= 0`丢弃camera-inside和near-crossing光 | canonical projection library产生conservative bounds；perspective/ortho/reversed-Z/jitter/stereo/viewport subrect全覆盖 |
| RDL-P1-021 | orthographic `ortho_size`被再次乘0.5 | grid、visibility、cascade只消费同一projection matrix/derived parameters，不再复制half-height约定 |
| RDL-P1-022 | 所有light/tile/bin统计在CPU做完整交集重扫 | 热路径只采集廉价GPU counters/readback或采样统计；详细debug按需启用且不改变产品assignment |
| RDL-P1-023 | 假cluster compute与CPU真实grid并存且graph虚报资源write | hard cutover到唯一assignment owner；graph access、queue lane、barrier和consumer read与真实encoder命令一致 |
| RDL-P1-024 | 没有按view history、动态range和稳定capacity policy | assignment generation支持view family/XR、camera cut、resolution change、overflow feedback和hysteresis resize |

### 6.4 Shadow policy、view、atlas与sampling

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RDL-P1-025 | 只选第一盏shadow directional，其他方向光无typed rejection | shadow policy按view、importance、cost和quality仲裁，所有accepted/rejected light都有稳定receipt |
| RDL-P1-026 | directional固定0.1 near、默认4级/150米，source不可配置 | near、distance、cascade count/split/fade/stabilization来自validated policy并受camera/quality/device约束 |
| RDL-P1-027 | directional cascade绕过allocator且伪造slot generation | directional/local light进入同一allocation generation或明确分区owner，generation只由物理内容身份产生 |
| RDL-P1-028 | visibility在allocation前独立创建所有shadow view | planner先完成acceptance/allocation，再发布唯一`PlannedShadowViewGeneration`给visibility和renderer |
| RDL-P1-029 | point/spot/cascade view math在plan和visibility复制 | 一个tested projection/bounds owner生成view/projection/frustum/caster query，所有consumer只读descriptor |
| RDL-P1-030 | atlas无gutter/padding，shader把UV clamp到槽边界 | 分配器包含filter footprint/gutter，采样使用half-texel与safe rect；邻槽强对比测试无bleed |
| RDL-P1-031 | PCF只有固定1/5/9 taps，point按dominant face硬切 | filter policy与source size/quality/device关联，point seam、cascade blend、rotation/noise/temporal稳定性可验收 |
| RDL-P1-032 | priority仅`intensity*range`且point全六面成败，未表达成本/历史 | 屏幕影响、shadow cost、mobility、cache hit、distance、importance、hysteresis共同驱动，原子light group结果可解释 |

### 6.5 Cache、depth submission、cookie、IES与contact shadow

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RDL-P1-033 | `ShadowCache` decision没有生产consumer | cache manager持有真实persistent depth/page content，evaluate/commit/retain接入frame graph并产生hit/miss/invalidation receipt |
| RDL-P1-034 | graph每帧`clear_store`整张atlas，静态内容无法留存 | cached/static与dynamic overlay采用load/copy/page preservation；clear只作用于需要重绘的物理区域 |
| RDL-P1-035 | 每slot创建uniform buffer、bind group和字符串并启动pass | view constants进入ring/dynamic offsets，按pipeline/material/view批量提交，CPU allocation和pass count受预算约束 |
| RDL-P1-036 | 每slot重放完整shadow command stream并用`BTreeSet`过滤 | planned caster packet按view/phase预编译或GPU cull，static/dynamic/VG/alpha-mask路径有独立批量与统计 |
| RDL-P1-037 | render path用`expect`处理pipeline缺失 | pipeline/device/resource失败返回typed pass outcome，保留last-good或显式无阴影degrade，不得在渲染线程panic |
| RDL-P1-038 | cookie固定8×8/64格且每帧重建1024 RGBA8 atlas | persistent slot/page residency、priority、resolution/mips/gutter/color policy、incremental upload与overflow receipt成立 |
| RDL-P1-039 | cookie没有scene producer，IES完全不存在 | Runtime71/92提供qualified cookie/IES artifact与authoring；spot/point/rect projection、normalization、reload/eviction闭环 |
| RDL-P1-040 | Contact Shadow是无light语义的全画面post multiply | 迁入逐光源direct visibility，重建world position并沿light ray march，支持mask/history/disocclusion；否则删除能力声明 |

### 6.6 Diagnostics、tests、fault与性能资格

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RDL-P1-041 | stats只记录light count、grid occupancy和atlas write次数 | per-stage receipt覆盖accepted/rejected/degraded light、cluster overflow、atlas pressure、cache hit、draw/byte/time与generation |
| RDL-P1-042 | allocator的`rejected`、`scale_factor`、`reused_previous`未进入产品readiness | 每帧公开结构化allocation report并可按light追踪到authoring/Editor diagnostic |
| RDL-P1-043 | shadow产品测试全部手工构造snapshot | 新增asset/component -> save/reopen -> World -> extract -> plan -> WGPU capture闭环，禁止仅靠fixture宣称可达 |
| RDL-P1-044 | many-light只测64/128，未覆盖65,535、cell overflow或分布极值 | 建1/64/128/4K/overflow梯度、dense/spread/camera-inside/near/ortho/rect/spot压力矩阵与明确预算 |
| RDL-P1-045 | 无multi-directional、atlas rejection/thrash、cache hit/invalidations产品测试 | 持续竞争、移动/静止切换、slot reuse、static/dynamic caster变化、quality切换和camera cut均有receipt/pixel回归 |
| RDL-P1-046 | 无cookie/IES缺失、reload、eviction和错误profile测试 | dependency等待/失败/last-good/恢复、atlas overflow、非法profile和device generation变化均可预测 |
| RDL-P1-047 | 无device loss/OOM/resize/surface reconfigure后的lighting/shadow恢复证据 | Runtime90 fault harness验证resource重建、stale packet拒绝、cache invalidation、无panic/泄漏/黑帧谎报 |
| RDL-P1-048 | 没有同硬件同场景Unreal/Unity/Fyrox/Godot对照基准 | 固定资产、曝光、光度、分辨率、shadow filter和画质，记录CPU/GPU/RAM/VRAM/像素误差及capture provenance |

## 7. P2演进项

| ID | 演进项 | 启动前提 |
|---|---|---|
| RDL-P2-001 | Virtual Shadow Map/page table/clipmap与physical page cache | atlas、generation、cache invalidation、GPU Scene和budget receipt先合格 |
| RDL-P2-002 | stochastic many-light/MegaLights式采样、reservoir与denoise | 正确cluster/direct baseline、稳定光度和reference capture先成立 |
| RDL-P2-003 | hardware ray-traced shadows与hybrid per-light method | Runtime28 HRT capability、BLAS/TLAS、SBT与fallback parity闭合 |
| RDL-P2-004 | ray-query contact/soft shadow与screen-space补洞 | per-light contact semantics、history与disocclusion基线合格 |
| RDL-P2-005 | EVSM/MSM/VSM moment filtering与可变penumbra | depth atlas边界、precision、light bleeding和temporal稳定性有产品指标 |
| RDL-P2-006 | disk/tube/sphere/polygon/mesh emitter直接光 | Rect LTC、source geometry、photometry/solid angle和bounds owner先完成 |
| RDL-P2-007 | spectral/SPD photometry与wavelength-aware BRDF/medium | RGB物理单位、working color space和能量守恒reference先闭合 |
| RDL-P2-008 | adaptive light LOD、GPU importance与quality prediction | acceptance receipt、hysteresis、overflow与同画质性能baseline成立 |
| RDL-P2-009 | multi-GPU、explicit async compute overlap与copy-queue shadow streaming | Runtime89/90真实queue/fence/ownership和跨GPU资源模型闭合 |
| RDL-P2-010 | XR multiview/foveated cluster与shadow allocation | Runtime66 view family、foveation map和双眼一致性资格完成 |
| RDL-P2-011 | offline/reference path-traced direct-light oracle与自动差分 | 资产单位、材质BRDF、exposure、capture和容差contract稳定 |
| RDL-P2-012 | neural/stochastic shadow reconstruction实验 | 正确非神经baseline、训练/模型provenance、fallback和平台成本证据成立 |

## 8. 实施里程碑

### M95-1：父P0 characterizations与authoring可达性

- 将09E十项和Runtime71两项P0转成current-source failing tests；
- 建shadow/cookie/IES/light-channel asset roundtrip fixture；
- 冻结旧shader/grid/atlas capture作为迁移oracle。

### M95-2：单一Prepared Lighting Generation

- 统一stable handle、canonical photometry、shape、channels和dependency readiness；
- 单次pack并让mesh/grid/froxel/shadow只读同一generation；
- hard cutover旧多次pack与低32位identity。

### M95-3：真实GPU Light Assignment

- 以现有CPU grid为oracle实现GPU z-slice/cluster count、allocate、populate与consumer；
- 建overflow feedback、capacity hysteresis、GPU/CPU differential和低端fallback；
- 删除方向光颜色summary、假AsyncCompute与虚报resource write。

### M95-4：共享物理Direct Lighting

- 建共享BRDF/attenuation/shape/channel/cookie/IES module；
- forward/deferred/froxel切到同一实现；
- 完成inverse-square、Rect LTC、photometric fixtures和NaN/finite资格。

### M95-5：统一Shadow Policy、Allocation与View

- 先按importance/cost/history/budget仲裁，再分配slot/page并发布唯一view generation；
- directional/local light统一物理identity与generation；
- 删除visibility和plan的重复projection/cascade逻辑。

### M95-6：Atlas正确性与批量Depth Submission

- 加gutter/filter footprint、point seam与cascade blend；
- 建view constant ring、caster packet、alpha/static/dynamic/VG分相批量；
- 移除per-slot allocation、字符串和render-thread panic。

### M95-7：真实Shadow Cache

- 接入`ShadowCache` decision、persistent depth/page owner、static reuse与dynamic overlay；
- graph按区域load/copy/clear，不再全atlas清空；
- 完成slot/page generation、caster/resource revision与device-loss invalidation。

### M95-8：Cookie、IES与Contact Shadow hard cutover

- persistent cookie/IES residency与增量atlas/page upload；
- scene/editor authoring、reload/eviction/overflow receipt闭环；
- Contact Shadow迁入逐光源direct visibility或删除错误产品能力。

### M95-9：Fault、scale与跨引擎资格

- 完成many-light、multi-view、atlas pressure、camera edge、cache soak和fault matrix；
- 采集CPU/GPU/RAM/VRAM、allocation/upload/draw/cache counters；
- 在同硬件、同画质、同曝光与同光度下对照Unreal，并记录Unity/Bevy/Godot/Fyrox辅助差分。

## 9. 资格门

| Gate | 必须通过的证据 |
|---|---|
| RDL-G01 | Directional/Point/Spot/Rect shadow设置可从scene asset保存、重开并抵达render plan |
| RDL-G02 | cookie和IES source/dependency/generation可roundtrip，缺失或失败状态不伪装ready |
| RDL-G03 | Spot partial/legacy asset默认迁移后与runtime canonical default一致 |
| RDL-G04 | 一个node上的light family组合满足明确互斥或multi-emitter schema，不再隐藏生效 |
| RDL-G05 | light stable handle在删除/复用/wrap/重载后不发生低32位别名 |
| RDL-G06 | zero-change frame产生0 repack、0 light upload和0 cookie atlas rebuild |
| RDL-G07 | lux/lumen/candela/nit转换与reference数值在规定容差内 |
| RDL-G08 | point/spot inverse-square与range cutoff不产生NaN、爆亮或非单调断层 |
| RDL-G09 | Rect width/height/orientation/source texture真实改变辐照度与高光形状 |
| RDL-G10 | forward/deferred/froxel共享case输出在规定数值容差内 |
| RDL-G11 | light/receiver/caster channel mask在direct、shadow和volumetric中一致生效 |
| RDL-G12 | shadow strength、depth/normal/slope bias均有独立可见像素效应与边界回归 |
| RDL-G13 | GPU cluster output与CPU oracle在randomized perspective/ortho scene逐cluster等价 |
| RDL-G14 | camera-inside、near-crossing、behind-center大光源不会整灯消失 |
| RDL-G15 | orthographic、reversed-Z、jitter、stereo和viewport subrect assignment正确 |
| RDL-G16 | spot cone、wide-angle fallback和rect oriented bounds无false negative |
| RDL-G17 | cluster overflow返回typed receipt并按policy扩容/降级，不静默截断 |
| RDL-G18 | render graph声明的resource access、queue lane和实际encoder命令逐项一致 |
| RDL-G19 | 1/64/128/4K/overflow light梯度满足固定CPU/GPU/VRAM预算 |
| RDL-G20 | dense/spread distribution不会因bitmask预算无限放大tile或产生不可解释画质跳变 |
| RDL-G21 | assignment capacity resize有hysteresis，持续边界负载无逐帧抖动 |
| RDL-G22 | 假directional color cluster buffer、zero-weight consumer与旧重复authority被删除 |
| RDL-G23 | 多方向光shadow仲裁产生accepted/rejected receipt且顺序稳定 |
| RDL-G24 | camera near/far、cascade count/split/fade/source policy逐字段进入planned view |
| RDL-G25 | visibility只为实际allocated face/cascade创建view，禁用shadow产生0 shadow view |
| RDL-G26 | plan与visibility对point/spot/cascade view matrix及frustum逐bit/容差一致 |
| RDL-G27 | directional和local allocation generation来自物理slot/page owner，不按frame伪造 |
| RDL-G28 | point六面原子接受/拒绝及spot/rect策略在atlas不足时可预测 |
| RDL-G29 | atlas相邻高反差slot在最大filter footprint下无采样bleed |
| RDL-G30 | point cube seam、cascade transition与subtexel camera move满足稳定性阈值 |
| RDL-G31 | static shadow第二帧命中cache且不重绘静态caster，像素与全重绘一致 |
| RDL-G32 | dynamic caster overlay不会破坏已缓存static depth，也不会遗漏移动遮挡者 |
| RDL-G33 | light/caster/resource/slot/page/device任一generation变化只失效必要cache范围 |
| RDL-G34 | graph不再每帧清空整atlas，clear/copy/load区域可由capture与receipt证明 |
| RDL-G35 | shadow depth submission无per-slot heap allocation/bind-group creation/string formatting |
| RDL-G36 | planned caster packet避免每slot全命令流扫描，static/dynamic/alpha/VG统计可分解 |
| RDL-G37 | pipeline/resource缺失、OOM和device loss不会触发render-thread panic |
| RDL-G38 | cookie atlas有persistent slot、gutter/mip/priority/overflow和incremental upload证据 |
| RDL-G39 | IES point/spot/rect profile normalization与photometric unit保持能量/峰值contract |
| RDL-G40 | cookie/IES reload、eviction、失败恢复不消费stale resource generation |
| RDL-G41 | Contact Shadow读取world position与逐光源direction/type/mask，不再全画面乘ambient/emissive |
| RDL-G42 | contact history在camera/light/object运动、disocclusion和cut下无持久ghost |
| RDL-G43 | 真实authoring入口的forward/deferred WGPU capture均显示shadow和cookie/IES效果 |
| RDL-G44 | atlas rejection、thrash、quality切换和多view竞争有稳定per-light diagnostics |
| RDL-G45 | stats报告accepted/rejected/degraded、overflow、cache、draw、upload byte与CPU/GPU time |
| RDL-G46 | fault恢复后stale packet被拒绝，新device generation重建resource且无泄漏 |
| RDL-G47 | 长时间移动/静止/增删光源soak无slot leak、capacity leak、history污染或闪烁 |
| RDL-G48 | 同硬件同画质Unreal基准记录场景、revision、曝光、光度、capture与CPU/GPU/RAM/VRAM数据 |

## 10. 参考引擎映射与适用性

| 目标能力 | 主参考 | 可迁移原则 | 不应机械复制 |
|---|---|---|---|
| 大规模light grid | Unreal `LightGridInjection.cpp`、Unity HDRP `LightLoop.cs` | GPU assignment、HZB/shape refinement、two-level/linked list、16-bit bandwidth、async与overflow feedback | UE RDG/RHI对象和Unity native不可见部分不能直接映射为Zircon API |
| 可移植cluster下限 | Bevy cluster GPU/CPU、Godot `ClusterBuilderRD` | point/spot/probe/decal typed lists、GPU overflow readback/resize、sphere/cone/box volume与debug | Bevy/Godot的固定上限和平台策略不是“优于Unreal”的最终目标 |
| 物理authoring与area light | Unreal LightComponent/Rect、Unity `LightUnitUtils`/HDRP LTC | 单位、source geometry、barn door、temperature、LTC与texture/IES residency | 不复制编辑器字段命名；先定义Zircon versioned descriptor |
| 传统shadow policy | Unreal `ShadowSetup.cpp`、Unity HD shadow managers、Fyrox light/CSM | screen influence、cache/update budget、static/dynamic split、distance/fade、quality和可解释降级 | Fyrox传统逐光pass只作为完整性下限，不作为规模目标 |
| page-based shadow终点 | Unreal VSM array/cache manager | physical page pool、light/primitive invalidation、page pressure feedback、dynamic resolution与cache stats | VSM必须在atlas/generation/cache/visibility底座合格后启动，不能用名称替代闭环 |
| cookie/IES/contact | Unreal IES/rect texture managers、Unity HDRP cookie/contact | persistent residency、atlas generation、world-space逐光源ray与mask | 当前Zircon全画面post occlusion不得保留兼容名或假完成度 |

Unity Graphics仓只包含SRP可见源码，不包含Unity native scene/RHI全部实现；本文不根据缺失源码推断其native core。Unreal源码规模和目标最接近最终工程级渲染器；Bevy、Godot与Fyrox用于验证基本完整性、可移植性和较低复杂度fallback，而不是性能上限。

## 11. 禁止的临时修法

1. 禁止只给scene struct加一个`casts_shadow: bool`，却不完成schema version、property、save/reopen、extract和receipt。
2. 禁止保留CPU真实grid和GPU假cluster两个authority，再用名称或统计宣称GPU clustered完成。
3. 禁止仅提高`u16::MAX`、tile mask常量或atlas尺寸来掩盖overflow和预算契约缺失。
4. 禁止让visibility继续猜测shadow view，再由planner晚期丢弃。
5. 禁止用`pub use`、兼容wrapper或旧buffer同步双写延长hard cutover。
6. 禁止把Rect Light继续按point light着色，仅通过更亮默认值伪装面积光。
7. 禁止用source-string tests替代GPU resource access、cache content、pixel或performance证据。
8. 禁止把Contact Shadow改名为AO但继续作为shadow产品卖点；语义不正确就删除能力声明。
9. 禁止在render thread用`expect`处理可由pipeline/resource/device状态触发的失败。
10. 禁止在没有同画质同硬件数据时写“优于Unreal”结论。

## 12. 状态

本文是review与重构计划，不是实现完成声明。Runtime09E的10项P0和Runtime71的2项P0仍由原报告唯一计数；Runtime95新增0项P0、48项P1、12项P2和48个资格门。实施必须按M95-1至M95-9顺序推进，并在每个阶段保留旧行为characterization、generation receipt、fault证据与可复现实测结果。

`source_recheck_required: true`在以下条件全部满足前不得改为false：冻结语料fingerprint重算一致或差异已审计；并发修改的light readiness/export已复查；父P0有对应failing/passing evidence；真实GPU、Editor、fault、scale和跨引擎基准均形成带revision与hardware provenance的验证记录。
