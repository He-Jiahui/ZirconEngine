---
related_code:
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/material_asset/advanced_features.rs
  - zircon_runtime/src/asset/assets/material/material_asset/subsurface.rs
  - zircon_runtime/src/asset/assets/material/material_control.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/oit_buffers
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/transmission
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/oit.rs
  - zircon_runtime/src/graphics/shader/includes/zr_oit.wgsl
  - zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment_core.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_light_cookie.wgsl
  - zircon_plugins/rendering/features/light_cookies
  - zircon_plugins/rendering/features/oit
  - zircon_plugins/rendering/features/planar_reflections
  - zircon_plugins/rendering/features/subsurface_scattering
  - docs/tests/runtime/render
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/plans/optimize/zircon_runtime/09g1-volumetric-fog-froxel-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightFunctionAtlas.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightFunctionRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/OIT/OIT.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PlanarReflectionRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessSubsurface.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SubsurfaceTiles.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/TranslucentRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Substrate/SubstrateRoughRefraction.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/OIT/OITSorting.usf
  - dev/UnrealEngine/Engine/Shaders/Private/PlanarReflectionShaders.usf
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/LightCookieManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/HDProbe.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/PlanarReflectionProbe.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/HDRenderPipeline.SubsurfaceScattering.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/DiffusionProfile/DiffusionProfileSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/ScreenSpaceRefraction.cs
  - dev/bevy/crates/bevy_core_pipeline/src/oit
  - dev/bevy/crates/bevy_pbr/src/transmission
  - dev/godot/scene/3d/light_3d.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/subsurface_scattering.glsl
  - dev/Fyrox/fyrox-impl/src/renderer
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09G2 · Advanced Surface Lighting 工程化差距

## 1. 结论

Zircon当前Light Cookie、OIT、Planar Reflection、Subsurface Scattering、Clearcoat、Anisotropy与Screen-space Transmission都不是空壳。Cookie有typed projection、atlas blit和direct-light shader消费；OIT有fragment store、每像素排序与resolve；Planar Reflection有镜像相机、oblique near clip、capture camera loop和mip filter；SSS有deferred MRT、tile setup、GPU indirect scatter和recombine；高级PBR有各向异性GGX、clearcoat direct/environment lobe、透射材质路由及最多四次scene copy。重构必须保留这些真实stage和测试oracle，不能把整组能力误判为“完全没有实现”。

但这些实现目前仍是相互孤立的路径原型，不是普通项目可创建、可配置、可诊断、可扩展的工程级系统。`World::build_render_frame_extract`只向`AdvancedLightingExtract`写入fog volume和volumetric light ID，其余字段全部取默认值。仓库内Cookie、OIT、Planar probe赋值只出现在测试或手工构造extract中；四组Editor plugin也只登记名称与capability，没有scene component、asset、inspector、gizmo、transaction、save/cook或runtime preview bridge。因此Cookie、OIT、Planar Reflection在普通scene链中是production-inert，不能以“plugin已注册”视为产品功能。

透明合成没有统一owner。OIT只替换`transparent-mesh`，自己重放透明mesh与3D sprite；`particle.transparent`和half-resolution transparency仍是独立pass。Transmission在替换发生前从原透明pass复制模板并插入scene-copy/draw序列，所以它继续游离于OIT之外。Volumetric Fog又在每个透明fragment进入OIT之前应用camera-to-fragment散射。结果是mesh、sprite、particle、transmission、half-res transparency与fog各自有局部顺序，却没有一份按深度段定义的透射/吸收/散射/混合合同。

OIT存在两个不能用“质量限制”解释的正确性问题。第一，fragment颜色用`pack4x8unorm`存入两个`u32`中的color word，resolve还再次把RGB clamp到0..1；HDR、pre-exposure、emissive和bloom前能量被不可逆截断。第二，所谓`fragments_per_pixel_average`实际被`ceil`成每个pixel固定capacity，超过capacity的片元在atomic increment后直接return；resolve用`min(count, capacity)`读取，因此既无法恢复也无法统计真正溢出。默认4层在1080p约占71.19 MiB、4K约284.77 MiB，而受storage binding limit限制时compiler会静默退回sorted transparency，公开的`OitSupport`诊断API没有进入产品控制面。

Planar Reflection在单probe测试里能工作，但多probe合同确定错误。所有probe共享一张1024x1024 RGBA16F mip chain，每次capture filter覆盖同一纹理；主视图参数却从可见probe中按最小`probe_id`选择矩阵和bounds。两个probe同时捕获时，纹理通常来自最后执行的capture，参数来自最小ID probe，产生确定性的矩阵/内容错配。shader还只在硬AABB内用单probe结果整体替换environment reflection，没有priority、distance/normal/plane-side fade、overlap blend或per-probe resource identity。

SSS的三段产品链是真实的，但核心核函数不具备可伸缩性。每个有效像素分别对R/G/B执行64次采样，共192个candidate；每个candidate重复读取profile、normal、depth并重建world position。没有sample budget、half resolution、separable/bilateral quality tier、temporal stabilization或frame-time controller。厚度由最大scatter radius乘0.5推断，normal阈值固定0.55，material的`falloff`只是center和blur结果的mix。它不能替代可author的Diffusion Profile、thickness map、thin/thick transmission、border attenuation和back-lighting合同。

Clearcoat和Anisotropy有真实direct-light lobe，但仍是局部材质扩展。各向异性只作用于direct GGX，environment specular仍按普通roughness/normal采样；clearcoat F0固定0.04，缺少coat IOR/tint与独立UV transform。Screen-space Transmission基本沿用了Bevy式“排序命令分组 + scene copy”路由，但shader只用world normal的XY乘`(ior - 1) * thickness * 0.02`偏移screen UV，没有view-space Snell ray、front/back depth intersection、roughness mip、exit normal、dispersion或有效thickness。scene copy已经包含volumetric fog，透射fragment末尾又执行一次volumetric apply，背景会被二次雾化。

现有artifact只能证明若干pass曾经执行。Advanced PBR与Planar图片是简单左右差分；Cookie和Irradiance Volume共用一张单球图片，无法独立归因；OIT只覆盖640x360三个交叉平面且没有HDR/overflow/particle；SSS单球最大RGB差仅6。7个artifact exporter为`#[ignore]`，当前focused set又有26个文件存在其他Session修改，不能用2026-07-12至07-15的图片/RDC替代当前源码验收。

本轮登记12项P0、27项P1、8项P2。重构顺序必须先冻结真实feature状态与唯一scene contract，再建立统一transparent compositor和feature-demand/capability truth；随后分别升级Cookie atlas、HDR OIT、per-probe Planar资源与调度、Diffusion Profile/SSS和物理Transmission，最后补齐Editor、scalability、telemetry、migration与竞争性产品gate。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | 文件 / 物理行 | 本轮判定 |
|---|---:|---|
| production focused set | 93 / 15,053 | E3：scene/extract、material、compile routing、GPU resource/executor/shader、plugin runtime/editor |
| production focused fingerprint | 93 / 15,053 | `11f8ed352f54dd8504f495cb9ab9f8a0fa86a44f19367e39714eb3f0674b4618` |
| production文件内test属性 | 58 | E2：packing、数学、source inclusion与局部stage合同 |
| dedicated focused tests | 14 / 3,756 | E2：54个test属性，其中7个ignored exporter/capture |
| Advanced Surface artifacts | 10 / 25,461,676 bytes | E2：2个RDC、5个PNG、3个文本报告；仅能证明简化场景路径 |
| Reference engine主链 | Unreal 10、Unity HDRP 6、Bevy 2组、Godot 2、Fyrox renderer spot check | E3：资源、调度、shader、authoring与降级边界 |

focused fingerprint按路径排序，对每个文件计算SHA-256，再对UTF-8的`path<TAB>hash<LF>`清单计算SHA-256。范围包括高级材质与core ABI、五个GPU feature目录、pipeline/submission/graph consumer、关键WGSL，以及四组runtime/editor plugin。当前其中26个文件存在本轮之外的modified状态，摘要只绑定当前工作区快照；进入实现或重新验收前必须重取。

### 2.2 数据链读取深度

本轮从MaterialAsset和`AdvancedLightingExtract`开始，追踪World producer、可见材质usage resolve、pipeline feature裁剪、pass插入/替换、command list分区、persistent resource、bind group、WGSL存储/采样、final scene-color合成和artifact exporter。OIT额外追踪到sprite重建与particle独立pass；Planar额外追踪capture camera derivation、update state、shared environment resource和主视图probe选择；SSS额外追踪profile gather、GBuffer packing、tile list、indirect dispatch和recombine。

shader审查不是只确认binding存在。Cookie核对atlas cell、projection和wrap；OIT核对color/depth packing、atomic overflow、排序方向和blend；Planar核对mip filter、bounds和environment替换；SSS核对每通道采样、world reconstruction、profile/normal/depth rejection与falloff；Transmission核对UV推导、scene-copy fallback、Beer attenuation、alpha与volumetric apply顺序。

### 2.3 与相邻审查的owner边界

- 09A拥有render graph resource truth、persistent GPU object、alias、queue、fence、device loss与budget。09G2定义Cookie/OIT/Planar/SSS/Transmission资源语义，不能在executor里再建立未版本化资源owner。
- 09B拥有view family、GPU Scene、visibility、draw command和particle/sprite分类。Planar capture visibility和transparent contributor必须复用该体系。
- 09C拥有material/shader ABI、permutation、PSO generation/cache与prewarm。Clearcoat/Anisotropy/Transmission/SSS/OIT shader contract不得维护第二套临时编译规则。
- 09D拥有cookie/profile/thickness/capture derived artifact的streaming、residency、fallback和cook。09G2只消费ready generation。
- 09E拥有light photometry、cluster、shadow、light layer和Rect Light模型。Cookie/Light Function是light extension，不复制light authority。
- 09F1拥有environment、IBL和reflection probe。Planar是同一Reflection Environment中的view-dependent provider；各向异性/clearcoat必须消费其versioned输出。
- 09F2拥有Irradiance Volume和baked lighting。AF-M2 artifact中Irradiance Volume不归09G2重复规划；SSS只定义baked diffuse进入scatter/retained的能量规则。
- 09G1拥有介质沿深度积分。09G2拥有透明表面与折射；二者共同定义每一透明深度段只应用一次介质传输。
- 09H拥有motion vector、jitter、camera cut、history、upscale、DOF和motion blur。Planar/SSS/transparent temporal路径必须接入统一history generation。

### 2.4 参考引擎边界

- Unreal提供工程上限。Light Function Atlas按系统需求启用，并被Deferred、Volumetric Fog、Lumen、Translucency等consumer共享；Planar Reflection有独立scene proxy/render target、frustum/occlusion筛选、prefilter roughness distance和stereo参数；SSS具有tile classification、Burley/Separable、half-res、sample set、bilateral、profile cache与多档CVar；Translucency按Before Distortion/After DOF/After Motion Blur/Holdout等pass分层，并显式集成OIT和ray-traced路径。
- Unity HDRP提供资源和authoring下限。Cookie Manager有可配format/size、cache/update、relayout、mip/border、area light与IES；HDProbe有Baked/Custom/Realtime、EveryFrame/OnEnable/OnDemand、time slicing、per-probe color/depth texture、influence/proxy和完整Editor handles；Diffusion Profile是独立asset，包含scatter distance、world scale、IOR、thickness remap、transmission mode/tint、border attenuation、preview与migration；SSS有sample budget和downsample steps。
- Bevy与Zircon当前结构最接近，因而也最能揭示Zircon的退化。Bevy OIT使用全屏共享linked-list node pool，RGB用RGB9E5保留HDR，capacity overflow至少是全局池语义；其Transmission同样按命令数分step复制scene color，但额外提供roughness quality/taps和depth-prepass rejection。Bevy可作为Rust边界参考，不能作为超过Unreal/HDRP的画质上限。
- Godot提供轻量级产品下限：Light projector是scene property并贯穿renderer；SSS有独立effect与quality控制。Godot没有同等级专用Planar Reflection/OIT体系，不应把其缺失当作降低Zircon目标的理由。
- Fyrox主要用于Rust renderer/material/editor ownership与传统transparent fallback抽查；当前没有与Unreal/HDRP等价的全套高级表面光照，不能作为能力上限。

### 2.5 明确未做

本轮没有修改production code，没有运行Cargo、Editor、WGPU或RenderDoc，没有重导出当前源码artifact，也没有运行参考引擎。未执行HDR > 1、OIT overflow、粒子/half-res/transmission混合、双Planar probe、camera/scene mutation、rough glass、厚物体折射、皮肤/耳廓back-light、MSAA/forward fallback、4K/ultrawide、stereo、dynamic resolution、VRAM pressure、device loss或同画质GPU benchmark。

## 3. 可保留并迁移的基础

### 3.1 typed feature ABI和pass边界可保留

Cookie projection、OIT settings、Planar probe/update mode、SSS profile和Transmission settings均为typed contract。Cookie atlas build、OIT store/resolve、Planar filter、SSS setup/scatter/recombine、Transmission copy/draw的stage边界清楚，适合升级为正式resource owner和验收点。

### 3.2 Planar镜像相机与oblique clip数学方向正确

reflection matrix、镜像camera transform和WGPU oblique near-plane projection有CPU测试。应保留这部分数学oracle，把单共享纹理和手工extract替换为scene-owned probe scheduler/resource pool。

### 3.3 OIT的排序与近端精确、远端近似策略可升级

resolve保留最多32个精确排序片元，并把超出`sorted_fragment_max_count`但仍已存储的远端层近似合成。策略本身可作为一个quality tier；必须先改HDR存储、真实overflow、全局pool和透明贡献者覆盖。

### 3.4 SSS的diffuse/specular分离和tile indirect链可保留

deferred lighting将SSS diffuse与retained/specular分离，setup构建active tile list，scatter使用GPU indirect，recombine回写scene color。阶段划分与workload audit值得保留，核函数、profile asset和quality controller需要重写。

### 3.5 Clearcoat与Anisotropy direct BRDF已有正确起点

各向异性tangent旋转与alpha-x/alpha-y GGX、clearcoat独立normal/roughness和base energy scale已真实进入direct lighting。后续应统一到material lobe graph并补足IBL、layer energy、texture transform和import/cook。

### 3.6 Transmission的独立command list避免普通透明重放

透射材质有独立render queue和command list，最多四组copy/draw；普通透明、sprite和OIT不会随每个transmission step重复绘制。这是正确的调度基础，但分组语义、scene representation与物理折射仍需升级。

### 3.7 feature descriptor与空输入inert测试可保留

多组测试验证“注册但无输入”不改变graph/frame，并检查owned pass执行。这些测试应保留为L2 contract gate，再增加普通scene authoring、failure fallback和产品画质/性能gate。

## 4. P0 差距清单

### P0-1：Cookie、OIT与Planar没有普通scene producer或authoring闭环

World只写fog和volumetric light ID。Cookie/OIT/Planar probe只能由测试或外部手工构造extract注入；Editor plugin只有capability字符串。必须建立versioned scene component/asset、project setting、runtime extraction、transaction/undo/save/cook和debug surface，并删除“capability存在即功能可用”的状态推导。

### P0-2：feature activation、capability与fallback没有唯一真值

四组plugin默认disabled；OIT可能因storage binding size静默移除，SSS可能因forward/MSAA静默移除，Cookie超过64或texture未ready也静默退化。产品必须输出requested/eligible/active/degraded/disabled五态、reason code、effective quality、resource cost和last-good generation；不能只从pass是否出现反推功能状态。

### P0-3：透明合成没有统一owner，跨mesh/sprite/particle/transmission/fog顺序不成立

OIT替换mesh pass并手工纳入3D sprite，却不接管`particle.transparent`、half-res transparency或Transmission。Volumetric又在片元存储前应用。必须建立Transparent Compositor：统一收集contributor、depth/coverage/material mode，明确sorted/OIT/WBOIT/transmission/particle/half-res路由，并按相邻深度段计算介质transmittance与in-scattering。

### P0-4：OIT把HDR颜色不可逆压为8-bit UNORM

`zr_oit.wgsl`用`pack4x8unorm(color)`，resolve再次clamp RGB到0..1。任何高亮、emissive、pre-exposed radiance和bloom energy都被破坏。至少改为RGB9E5/FP16或分离radiance/alpha的HDR-safe node format，并用>1、负pre-exposure边界和bloom输出做golden/readback gate。

### P0-5：OIT真实capacity overflow直接丢片元且不可观测

每像素slot超过固定capacity立即return；count虽然继续增长，resolve却截成capacity，GPU/CPU没有overflow counter、pixel mask或降级策略。`fragments_per_pixel_average`名称也与固定per-pixel allocation不符。必须改为共享node pool或明确固定K-buffer，提供overflow counter/heatmap、deterministic tail approximation和per-view adaptive budget。

### P0-6：Planar多probe时共享纹理与最小ID参数会确定性错配

所有capture写同一mip chain，主视图按最小`probe_id`选matrix/bounds。必须以probe resource identity管理texture slice/atlas entry、generation和params，capture发布必须原子绑定`probe_id + texture_generation + matrix_generation`；双probe overlap与非overlap都是发布前必测场景。

### P0-7：Planar没有scene-owned update/invalidation/scheduling生命周期

OnDemand只在显式`request_planar_reflection_capture`/`mark_dirty`后更新，而production没有scene probe producer，也没有在probe transform、camera、light、visible object、material或capture setting变化时自动失效。EveryFrame则可能为全部probe追加capture camera。必须建立visibility/importance/time-slicing scheduler、mutation dependency和last-ready fallback。

### P0-8：Screen-space Transmission不是工程级折射模型

world normal XY乘固定0.02比例不能表达Snell折射、view orientation、厚物体exit point或遮挡。必须至少实现view-space refracted ray、hierarchical depth intersection、front/back thickness、roughness-aware color pyramid和off-screen/environment fallback；高档可接hardware/software ray query。当前路径只能标为low-quality approximate screen warp。

### P0-9：Transmission对已雾化背景再次应用Volumetric Fog

scene copy来自已经完成opaque/sky volumetric apply的scene color，透射材质合成后又执行一次forward volumetric apply，背景沿同一camera segment被重复衰减/加散射。必须让Transparent Compositor持有未雾化surface radiance或深度分段数据，保证每段介质只积分一次，并建立clear/foggy/multi-layer对照oracle。

### P0-10：SSS固定192 candidate/有效像素，没有质量或frame-time预算

R/G/B各64次采样且重复depth/normal/world reconstruction，在大面积皮肤、4K或多视图下无法伸缩。必须支持adaptive sample budget、half/quarter resolution、separable或importance-sampled Burley、bilateral cache、tile classification、temporal stabilization和GPU timing feedback，并给出每档误差与毫秒预算。

### P0-11：SSS profile/thickness合同不足且profile冲突可被静默折叠

profile内嵌在MaterialAsset，默认ID容易冲突；可见材质gather使用first-wins折叠后，profile table已无法报告同ID不同参数。厚度又由scatter radius推断，没有thickness map、thin/thick transmission、border attenuation或back-light。必须建立独立DiffusionProfile asset、稳定content identity、slot allocator/conflict diagnostic和材质thickness/transmission字段。

### P0-12：现有证据不能关闭产品正确性和性能gate

Cookie artifact与Irradiance Volume混合，OIT只有三平面LDR，Planar只有单probe，SSS只有平滑球，Transmission没有depth/roughness/fog专用场景。7个export/capture test被ignore，旧RDC/PNG也未绑定当前26个modified文件。必须把artifact区分为path evidence和acceptance evidence，并为每个P0建立当前源码自动gate。

## 5. P1 差距清单

### P1-1：Cookie atlas固定为1024、8x8、64个128像素cell

固定格子忽略源纹理尺寸、投影类型、屏幕贡献和quality tier；第65个cookie按light ID排序后被静默截断。应改为可配置format/extent、变尺寸packing、importance/residency、稳定slot和明确overflow reason，而不是继续扩大常量。

### P1-2：Cookie每个active frame全atlas清白并重绘全部ready texture

executor每帧clone cookie列表并`rebuild`，4 MiB atlas被整张clear，所有cookie重新blit。应按texture generation和projection recipe维护dirty entry，只更新变更区域；atlas relayout必须有迁移generation和last-good表。

### P1-3：Cookie缺图时仍发布metadata并得到静默白光fallback

texture未ready会跳过blit，但light仍获得slot metadata，白色clear使画面看似“cookie无效”。必须区分pending、missing、failed、evicted，选择显式neutral/last-good/debug fallback，并让Editor和stats显示reason。

### P1-4：Cookie没有mip、gutter、dilation、HDR/color-space或filter contract

单mipRGBA8与紧邻cell会在缩小、斜投影和高频图案时闪烁/串色，也不能表达HDR light function。应为2D、cube/octahedral、area cookie定义导入格式、mip filter、border/gutter、channel/intensity、color-space和压缩策略。

### P1-5：Cookie只覆盖有限projection，没有IES、Area Light或通用Light Function材质

当前只有Directional/Spot/PointOctahedral，且light cookie与IES/profile/material function没有统一调制合同。应把Cookie视为Light Modulation provider，支持texture、IES、area convolution和受限material graph，并保持photometry与shadow能量一致。

### P1-6：Cookie binding只对fragment可见，Volumetric Fog无法消费

direct pixel lighting能采样cookie，但froxel compute不能访问atlas。应由Light Function resource owner向Deferred/Forward/Volumetric/Lumen-like GI/Translucency声明同一versioned binding，consumer capability必须可查询。

### P1-7：Cookie metadata复用`GpuLightData`其他字段，ABI语义脆弱

directional cookie把offset/scale写入本来对该light类型未使用的position/range或spot字段。短期节省空间会阻碍area light、world-origin和shader ABI演进。应使用显式modulation index/record，并由09E light ABI版本化。

### P1-8：OIT默认显存成本过高且没有预算控制

默认4层在1080p约71.19 MiB、4K约284.77 MiB，尚未计入scene color、depth和其他透明资源。应按view extent、dynamic resolution、平台budget和历史overflow自适应，支持共享全局pool、resolution fraction和预算回收。

### P1-9：OIT只提供sorted fallback，没有technique ladder

复杂scene需要exact PPLL/K-buffer、WBOIT、moment-based、depth peeling或ray tracing等不同trade-off。应按material/scene/platform选择technique，而不是“storage buffer不够就整view退回object sorted”。

### P1-10：OIT的custom shader失败是整pass错误，缺少per-material兼容状态

任一透明shader没有`fs_oit`合同就记录unsupported并让pass返回Err。应在shader compile/prewarm阶段生成compatibility status，为不兼容材质选择sorted/subpass fallback，并在Editor显示原因；不能运行时才让整帧失败。

### P1-11：OIT sprite每帧重建vertex buffer、texture bind group和临时列表

`prepare_oit_sprite_draws`按帧生成vertices、buffer和bind group。应进入09B/09D统一sprite GPU stream、bindless/atlas和draw compaction，OIT只消费prepared contributor range。

### P1-12：OIT capability API与实际compiler gate分裂

`OitSupport`只在core自身测试使用，产品compiler另用backend capability和binding-size条件裁剪。应合并为一份resolution report，包含required/available limits、memory plan、selected fallback和user-facing diagnostic。

### P1-13：Planar主视图只按最小ID选一个probe，没有空间选择与混合

硬AABB内单probe整体替换environment reflection；overlap时不看priority、distance、normal facing、screen coverage或roughness。应使用influence/proxy volume、priority、blend distance与parallax-validity评分，必要时每tile选择/混合多个provider。

### P1-14：Planar固定共享1024 RGBA16F mip chain，不尊重per-probe预算

256/512 probe仍依赖同一1024资源，所有probe没有独立last-ready generation。应建立texture array/atlas/pool，按probe resolution、quality、visibility和budget分配；每个entry有color/depth/mip/generation与eviction state。

### P1-15：Planar filter不是roughness-aware物理prefilter

当前逐mip最多5x5简单权重blur，不按GGX分布、reflection distance、depth discontinuity或normal过滤，也没有temporal稳定。应实现距离/roughness-aware prefilter或GGX convolution，保留depth并避免clip plane揭露区漏色。

### P1-16：Planar capture缺少frustum/occlusion/importance/time slicing

EveryFrame可为所有probe追加完整capture camera；没有主视图frustum、screen area、occlusion history、update budget或多帧调度。应复用09B visibility，按贡献排序并发布last-ready结果，超预算时保留旧generation而不是随机停更。

### P1-17：Planar probe resolution与capture target extent没有强一致性验证

filter使用probe声明resolution计算mip和dispatch，但capture target是独立ResourceId。必须在prepare时验证format/extent/sample count/usage，必要时重建或拒绝，并让artifact记录effective resolution。

### P1-18：Planar缺少stereo、dynamic resolution与large-world合同

当前单camera矩阵和单params不足以表达双眼view rect、camera-relative transform、world-origin rebasing和view-dependent capture。应由09B view family与09H temporal authority提供per-view参数和失效规则。

### P1-19：SSS在Forward或MSAA > 1时静默消失

compile只保留Deferred且single-sample的SSS descriptor；虽然plugin helper能生成diagnostic，产品路径没有上报。应定义forward/MSAA resolve方案或明确quality fallback，并在effective feature report中可见。

### P1-20：SSS profile slot只有16个且缺少稳定分配、dedup和overflow策略

16槽本身可接受，但当前用material profile ID直接索引，缺少content-hash dedup、default/neutral slot、跨streaming generation remap和overflow fallback。应建立per-view profile table builder及GPU material remap。

### P1-21：SSS几何拒绝每个RGB通道重复，且阈值/厚度不可配置

normal/depth/world reconstruction和profile匹配被执行三遍，固定normal dot 0.55、推断thickness容易跨薄边或漏采样。应先计算共享geometry acceptance/offset，再按channel radius/weight采样；阈值来自profile/quality并有edge leakage指标。

### P1-22：SSS的`falloff`不是清晰的物理profile参数

shader把RGB falloff作为center diffuse与scattered结果的线性mix，名称容易被理解为扩散尾部或吸收。应迁移为明确的scatter distance/shape、albedo texturing mode、border attenuation和strength，旧字段只在migration中解释。

### P1-23：SSS缺少皮肤、蜡、叶片、眼球等material workflow

只有自定义lighting model名字`subsurface`和内嵌数值，没有Diffusion Profile asset浏览器、材质slot、厚度预览、transmittance曲线或profile debug。应提供domain-specific preset与可视化，但preset不能替代可编辑物理参数。

### P1-24：SSS与Baked/GI/Volumetric的能量归属没有冻结

deferred baked diffuse曾被写入emissive/retained路径，SSS又对已应用volumetric transmittance的diffuse做空间扩散。必须定义direct diffuse、indirect diffuse、emissive、specular、fog的先后和哪些能量参与scatter，并以white furnace与分量debug验证。

### P1-25：Anisotropy只影响direct light，IBL仍是各向同性

environment specular没有使用anisotropic tangent frame或directional roughness，材质在直接光和环境光下外观不一致。应实现anisotropic IBL/LUT或明确低档近似，并用旋转tangent和roughness sweep验证。

### P1-26：Clearcoat层参数和纹理坐标合同不完整

coat F0固定0.04，没有coat IOR/tint/absorption；clearcoat normal缺少独立UV set/transform，layer energy只覆盖当前有限lobe。应进入统一layered material contract，支持KHR clearcoat导入、独立texture transform和多光源/IBL能量守恒。

### P1-27：Transmission的scene-copy预算与分step语义过于粗糙

单次全屏RGBA16F copy在1080p约15.82 MiB、4K约63.28 MiB；最多4次仅copy带宽就约63.28/253.12 MiB每帧。命令按数量分连续区间，不按pixel overlap、depth complexity或material dependency。应使用color pyramid/partial rect/tile demand、GPU depth-layer classification和明确step budget；零step/超预算必须有可见fallback。

## 6. P2 差距清单

### P2-1：名称没有表达近似层级

`fragments_per_pixel_average`、`ScreenSpaceTransmission`、`PlanarReflectionQuality`和`SubsurfaceProfile`容易暗示比实际更强的合同。完成迁移前应在API/Editor中标明fixed K、screen-space approximate、single-provider或prototype状态。

### P2-2：缺少schema version与hard-cut migration

Cookie projection、OIT setting、Planar probe、SSS profile和高级材质字段都需版本、default provenance、upgrade diagnostic和旧值映射；迁移完成后删除旧DTO/隐式default，避免双owner长期共存。

### P2-3：缺少跨CPU/GPU数学oracle

应为cookie projection、OIT blend/sort、oblique clip、Burley profile、Snell/refraction和Beer attenuation建立相同输入的CPU oracle、WGSL readback与容差，不能只断言source包含某个字符串。

### P2-4：缺少feature-specific debug view和capture metadata

需要Cookie atlas/slot、OIT layer/overflow、Planar probe ID/generation/influence、SSS profile/tile/radius、Transmission ray/hit/mip/fallback视图；截图/RDC旁应自动写source fingerprint、GPU、driver、quality、effective settings和pass timing。

### P2-5：缺少稳定的性能数据格式

当前文本混合CPU submit、graph profile与资源字节，字段不统一。应输出机器可读JSON/CSV，区分CPU build/record/submit、GPU timestamp、VRAM committed/resident/transient、overflow和quality，并进入回归阈值。

### P2-6：缺少平台和feature-interaction矩阵

至少覆盖DX12/Vulkan/Metal/WebGPU能力、forward/deferred、MSAA/TAA、HDR/pre-exposure、DOF/motion blur、dynamic resolution、stereo、particles、fog、GI和device loss，Unsupported必须是显式状态。

### P2-7：缺少authoring validation与修复建议

Editor应在cookie格式/atlas overflow、probe target不匹配、profile冲突、OIT预算超限、transmission无depth支持时给出对象定位和可执行修复，而不是通用“queued”反馈或静默白色/退回sorted。

### P2-8：缺少持续参考基线与画质误差记录

应固定一组Unreal/HDRP/Godot/Bevy等价场景，记录目标设置、限制和图像/性能差异。参考源码用于设计，不代表复制其默认值；Zircon要超过目标必须以同画质、同硬件、同场景数据证明。

## 7. 目标架构

### 7.1 唯一scene与asset合同

| 能力 | Scene / Asset authority | Render extract | GPU identity | 必需状态 |
|---|---|---|---|---|
| Light Modulation | Light component引用Cookie/IES/LightFunction asset | visible light modulation demand | atlas entry + content generation | pending/ready/degraded/overflow |
| Transparency | Camera/quality setting + material transparency mode | unified contributor ranges + technique demand | compositor generation + per-view buffers | sorted/OIT/WBOIT/transmission/fallback |
| Planar Reflection | PlanarReflectionProbe component + capture settings | visible probe request与mutation generation | probe ID + color/depth entry + matrix generation | dirty/scheduled/capturing/ready/evicted |
| SSS | DiffusionProfile asset + material profile/thickness | active profile hash table + material remap | profile table generation + tile resources | active/remapped/overflow/fallback |
| Advanced PBR | versioned material lobe graph | required lobe/permutation/quality | material payload + PSO generation | resident/compiled/degraded |

World只能产出语义数据和stable resource handle，不携带ready GPU object，也不能在缺省结构体里静默抹掉feature。所有extract字段必须有producer测试：从scene创建对象，经save/reload、visibility、submission、graph到pixel或GPU readback。

### 7.2 Advanced Lighting Feature Resolver

建立每view唯一resolver，输入scene demand、material demand、camera quality、backend limits、resident generations和frame budget，输出：

1. `requested_features`：用户/scene/material请求的能力；
2. `eligible_features`：backend、pipeline、MSAA、view family允许的能力；
3. `active_features`：本帧真实执行且资源ready；
4. `degraded_features`：选择了何种fallback、原因、误差/质量层级；
5. `resource_plan`：显存、transient、copy bandwidth、dispatch/draw预算；
6. `diagnostics`：对象、asset、material、limit和修复建议；
7. `generation`：任何影响graph/resource/shader的变化都更新，用于cache和history失效。

compile不能再靠`Option::is_some`或Vec是否为空分别裁剪feature。resolver report同时提供给render graph、stats、Editor和artifact metadata，避免控制面与执行面分裂。

### 7.3 Unified Transparent Compositor

透明系统需要一份按view编译的contributor table：mesh、sprite、particle、decal-like transparent、transmission和half-res effect都登记bounds、depth range、material mode、lighting mode、velocity、fog participation与preferred technique。compositor负责：

- 把可精确排序对象路由到sorted phase；
- 把兼容片元路由到HDR OIT/WBOIT等GPU technique；
- 为screen-space transmission建立depth/color pyramid和layer scheduling；
- 决定particle/half-res在DOF、motion blur、TAA前后的composition domain；
- 以opaque depth、transparent fragment depth和fog integral计算每个depth segment；
- 把unsupported material局部降级，不让一个shader破坏整pass；
- 输出layer count、overflow、copy bandwidth、fallback和GPU timing。

OIT、Transmission和Volumetric不能再各自采样/覆盖最终scene color。最终合成至少需要`surface_radiance + coverage/alpha + depth + medium segment`语义，具体buffer布局由09A/09H共同确定。

### 7.4 Light Modulation Atlas Manager

目标manager按content hash和recipe缓存Cookie/IES/Light Function，支持变尺寸packing、mip/gutter/dilation、HDR format和per-platform compression。visibility/importance只影响residency，不改变stable asset identity；relayout以双表或generation原子发布。Deferred、Forward、Volumetric、Translucency和GI consumer读取同一entry record，并能判断neutral/last-good/failure。

### 7.5 Planar Probe Scheduler与Resource Pool

每个probe保留独立capture state和published generation。scheduler按frustum、occlusion、screen area、roughness contribution、update mode、last update age与GPU budget排序；capture color/depth、filter、publish是一个原子job。resource pool提供per-probe entry或明确atlas slice，eviction保留last-good或回退environment。主视图按influence/proxy/priority选择，而不是按ID；多view/stereo共享可共享内容但保留per-view projection参数。

### 7.6 Diffusion Profile与SSS Quality Controller

DiffusionProfile必须成为独立asset，至少包含scatter distance/shape、world scale、IOR/F0、texturing mode、thickness remap、thin/thick transmission、tint、border attenuation和版本。per-view table按content hash dedup、稳定slot remap并报告overflow。quality controller按屏幕面积、profile radius和GPU时间选择full/half resolution、sample set、Burley/separable、temporal与bilateral配置；所有档位共享CPU oracle和profile-fit误差。

### 7.7 Layered PBR与Transmission provider

Clearcoat、Anisotropy、Base、Sheen/未来lobe应进入统一layered BSDF合同，direct与IBL使用同一参数解释、Fresnel和能量归一。Transmission provider按quality选择：

- Low：明确标记的screen warp，受限场景使用；
- Medium：view-space ray + depth pyramid + color pyramid + roughness mip；
- High：front/back depth或thickness asset、multi-step overlap、temporal stabilization；
- Ultra：hardware/software ray query与off-screen scene representation。

每档都必须定义fog、shadow、GI、emissive、velocity、DOF、OIT和alpha coverage规则，不能只增加采样次数。

### 7.8 Resource、PSO与history生命周期

所有atlas/pool/profile table/pyramid/history由09A资源owner创建并按view/device generation复用。executor只记录已解析资源，不创建长期owner；逐帧small uniform进入ring/dynamic buffer，sprite/mesh复用prepared GPU stream。PSO与shader variant进入09C prewarm/cache，`cache: None`只可作为backend明确不支持时的实现细节。device loss后从scene/asset truth重建，不读取悬空Arc或沿用旧generation。

### 7.9 Editor、debug与scalability

Editor至少提供Light Cookie/IES slot、Planar probe component与handles、Diffusion Profile asset preview、Transparency camera/quality设置、material lobe inspector和feature status面板。所有编辑进入transaction/undo/save/cook；viewport显示runtime resolver的effective state。Scalability不只枚举Low/Medium/High，而是解析为atlas、OIT memory、probe update、SSS samples、Transmission rays/copies等可度量预算。

## 8. 重构owner与依赖

| Owner | 必须提供 | 09G2消费 / 交付 |
|---|---|---|
| 09A RHI/Graph/Lifetime | typed resource class、pool、budget、fence、device-loss、timestamp | atlas/OIT/probe/SSS/pyramid/history生命周期 |
| 09B Renderer/Visibility | persistent scene、view family、contributor visibility、GPU draw stream | transparent contributor、probe/cookie importance与capture culling |
| 09C Material/Shader/PSO | layered material ABI、variant generation、prewarm/cache、compile diagnostic | Clearcoat/Anisotropy/SSS/OIT/Transmission shader合同 |
| 09D Streaming/Residency | texture/profile/capture artifact、generation、fallback、cook | Cookie/IES/thickness/profile资源ready truth |
| 09E Direct Lighting | light ABI、photometry、cluster/shadow/layer | modulation index和direct lobe消费 |
| 09F1 Reflection Environment | provider selection、IBL、probe资源语义 | Planar provider与anisotropic/coat IBL |
| 09F2 Baked Lighting | baked diffuse/irradiance energy contract | SSS参与规则，不重复Irradiance Volume |
| 09G1 Volumetric | depth-segment medium integral | transparent/transmission只应用一次fog |
| 09H Post/Temporal | velocity、history、camera cut、DOF/MB/upscale顺序 | Planar/SSS/Transmission temporal与composition domain |
| Editor/Project/Cook后续计划 | schema、transaction、inspector、gizmo、build | 普通项目可author、保存、运行、调试与迁移 |

任何里程碑若需要上述owner尚未交付的底层合同，应写跨计划failure handoff，不能在09G2目录复制临时resource manager、visibility或material ABI。

## 9. 分层实施里程碑

### M0：冻结完成度、artifact与feature status

- 将Cookie/OIT/Planar标记为prototype/non-authorable，将SSS/Transmission标记为integrated prototype；
- 固定当前source manifest、artifact hash、有效/无效证据和已知P0；
- 实现统一Feature Resolver report，但先不改变画面；
- 验收：所有silent disable/truncation/fallback有reason code，Editor/stats/JSON一致。

### M1：建立scene/asset schema与hard-cut migration

- 新增Light Modulation asset reference、PlanarReflectionProbe component、DiffusionProfile asset、Transparency camera setting；
- material只引用profile/texture handle，不内嵌冲突ID作为最终truth；
- scene save/reload/cook和旧asset migration闭环；
- 验收：普通项目无需手工构造extract即可请求所有能力。

### M2：统一feature demand、capability和resource plan

- scene/material/view/backend/budget只通过resolver生成effective plan；
- graph cache key纳入generation和effective technique；
- 验收：forward/MSAA/storage limit/VRAM压力的降级可重复、可查询、无黑帧。

### M3：工程化Light Modulation Atlas

- 变尺寸packing、mip/gutter/HDR、dirty update、last-good和overflow；
- Cookie/IES/Area/Light Function统一entry，接入direct与volumetric；
- 验收：动态streaming、relayout、64+ lights、atlas bleed和device loss场景通过。

### M4：建立Transparent Compositor骨架

- 统一mesh/sprite/particle/transmission/half-res contributor；
- 冻结composition domain、depth segment、velocity、fog、DOF/MB/TAA顺序；
- 保留sorted fallback，先确保功能不丢失；
- 验收：关闭OIT时与既有sorted路径等价，所有contributor仍可见。

### M5：HDR OIT与overflow治理

- HDR-safe node format、共享pool或正式K-buffer、overflow counter/tail approximation；
- per-material compatibility与technique fallback；
- 验收：HDR emissive、16+交叉层、particle/sprite/custom material、4K预算和overflow heatmap通过。

### M6：Planar per-probe resource pool与原子发布

- 每probe color/depth/mip/generation，双probe不再共享错误内容；
- target descriptor validation、last-ready、eviction；
- 验收：两个不同动画probe同时显示，矩阵/纹理generation严格配对。

### M7：Planar visibility scheduler、filter与selection

- frustum/occlusion/importance/time slicing、OnDemand mutation invalidation；
- roughness/distance/depth-aware filter、influence/proxy/priority blend；
- 验收：多probe、移动对象、probe overlap、roughness sweep、stereo/dynamic resolution和预算退化通过。

### M8：Diffusion Profile asset与table

- profile authoring/preview/migration、hash dedup、slot remap/overflow；
- thickness/transmission/border字段贯穿material/GBuffer/GPU；
- 验收：同ID不同内容被拒绝或重映射，save/reload/cook保持同一外观。

### M9：可伸缩SSS kernel

- 共享geometry acceptance、importance-sampled Burley/Separable、half-res、bilateral、temporal和quality budget；
- direct/indirect/baked/emissive/fog能量归属冻结；
- 验收：skin/ear/wax/leaf/eye边界、profile fit、4K GPU毫秒和camera motion稳定性达到门槛。

### M10：统一Layered PBR direct/IBL

- Anisotropic IBL、clearcoat IOR/tint/UV、layer energy与KHR import/cook；
- white furnace、multi-light、roughness/tangent sweep；
- 验收：direct与IBL外观连续，energy error在定义容差内。

### M11：物理分档Transmission

- depth/color pyramid、view-space ray、front/back thickness、roughness mip、off-screen fallback；
- multi-layer scheduling从command count升级为overlap/depth需求；
- 验收：Snell角度、厚度、rough glass、遮挡、off-screen和性能档位通过。

### M12：Volumetric与Temporal集成

- transparent depth segments只应用一次fog；
- Planar/SSS/Transmission接入统一motion/history/camera-cut；
- 验收：多层玻璃穿雾、移动probe、皮肤运动和camera cut无double fog/ghosting。

### M13：资源/PSO生命周期、Editor和debug闭环

- persistent pool、ring buffer、prewarm/cache、device loss；
- inspector/gizmo/profile preview/feature status/debug views；
- 验收：编辑可撤销保存、runtime preview同真值，device recovery与24h soak无泄漏/持续增长。

### M14：竞争性产品gate与hard cut

- 删除旧手工extract入口、共享Planar纹理、RGBA8 OIT、内嵌profile ID和临时fallback；
- 同场景对照Unreal/HDRP/Bevy/Godot，发布画质、GPU、VRAM和限制；
- 只有自动gate、artifact、性能和独立review全部通过后，feature状态才可改为product-ready。

## 10. 验收矩阵

| 域 | 必测场景 | 正确性gate | 性能/预算gate | 证据 |
|---|---|---|---|---|
| Cookie | directional/spot/point/area、IES、动画、65+、stream in/out | projection、mip/gutter无bleed、last-good、volumetric一致 | atlas update bytes、resident entries、GPU ms、overflow | atlas debug + pixel golden + RDC |
| OIT | HDR emissive、1/4/8/16/64层、交叉mesh、sprite、particle、custom shader | HDR不clamp、front/back oracle、overflow可测且deterministic | 1080p/4K VRAM、GPU ms、node utilization、fallback | readback + layer heatmap + JSON + RDC |
| Planar | 2/8 probes、overlap、移动对象/光、OnDemand/EveryFrame、stereo | probe ID/matrix/texture generation一致、clip/fade/selection正确 | captures/frame、pool bytes、filter ms、time-slice age | probe debug + sequence PNG + RDC |
| SSS | skin/ear/wax/leaf/eye、profile conflict、thin/thick、motion | CPU profile fit、edge leakage、energy、thickness/back-light | samples/pixel、active tiles、GPU ms、history rejection | profile plot + golden + timing JSON |
| Clearcoat/Anisotropy | white furnace、tangent rotation、direct/IBL、texture transforms | energy、Fresnel、direct/IBL连续、import round-trip | variants/PSO、GPU ms | parameter sweep atlas + readback |
| Transmission | Snell slab、sphere、rough glass、multi-layer、off-screen、fog | ray hit/depth/thickness、roughness mip、fog exactly once | copies/rays/taps、bandwidth、GPU ms、fallback | ray debug + sequence golden + RDC |
| Cross-feature | OIT + particles + fog + DOF + TAA + GI + Planar | contributor不丢失、顺序与history正确、无double count | graph resources、VRAM peak、frame budget | full-chain capture + machine report |

所有产品gate必须至少包含：feature-off exact baseline、registered-but-empty inert、active visible delta、failure/degraded path、动态sequence、resize/quality切换、device loss重建和当前source fingerprint。单个`changed_pixel_count > N`不能独立判定画质正确。

## 11. 现有测试与artifact判定

### 11.1 OIT

`plan18_oit_three_crossing_transparent_planes_sorted_vs_oit_wgpu_20260712.txt`记录640x360、4层capacity、最多8层精确排序，changed pixel为12,954、mean RGB error为2.2195、max error为88。sorted submit CPU为192,701 us，OIT为302,135 us；graph profile为7,279/31,441 us。资源约7.03 MiB layers加0.88 MiB counts。该证据能证明交叉平面顺序变化与owned passes，不能证明HDR、真实GPU时间、overflow、particle、sprite纹理、custom shader或4K预算。

### 11.2 Planar Reflection

`plan18_planar_mirror_floor_oblique_clip_filter_wgpu_20260712.txt`记录单个256 capture、640x360、changed pixel 9,452、mean error 2.4133、camera loop从1增到2。图片只显示三个方块在单一镜面地板中的反射。它能证明mirror camera/capture/filter/consumer路径，不能触发共享纹理与最小ID参数错配，也不覆盖OnDemand mutation、overlap、roughness、stereo或预算。

### 11.3 Subsurface Scattering

`plan18_sss_skin_sphere_deferred_burley_wgpu_20260712.txt`记录64 samples、changed pixel 27,821，但mean error仅0.0534、max error 6，brighter/darker pixel为581/657。图片中的左右平滑球差异非常轻微。测试额外检查红通道gain与三pass/workload执行，可作为路径证据；它不是皮肤边缘、耳透光、厚度、profile fit、漏光或4K性能验收。

### 11.4 Advanced PBR / Transmission

现有PNG比较baseline/advanced三球与两个背景参照球；历史output record记录clearcoat/anisotropy/glass区域changed pixels为6,403/7,028/1,876，mean error为2.7136/1.9503/0.1489，并有13,158,422-byte RDC。它能证明late-forward opaque、scene copy、transmission draw顺序和材质差分，不能证明Snell/depth/thickness/roughness、anisotropic IBL、clearcoat energy、multi-layer或double fog。

### 11.5 Light Cookie

AF-M2 PNG/RDC把Cookie与Irradiance Volume同时应用到一个球体。非ignored测试分别比较baseline->cookie与cookie->cookie+volume且要求各自changed pixels >2,000，这是有价值的路径隔离；但导出的最终图片没有cookie-only panel、atlas view、投影图案或错误场景，无法从artifact独立判断cookie正确性、bleed、streaming或64-entry overflow。

### 11.6 测试形态风险

14个dedicated focused test文件共3,756行、54个test属性，7个export/capture被ignore。大量测试手工构造`RenderFrameExtract`、资源和feature descriptor，这恰好绕过P0-1的普通scene断链。source inclusion、pass name、changed pixel和简化CPU helper适合L1/L2，不得替代L3 scene round-trip、L4 WGPU sequence、L5产品画质/性能、L6 soak/device-loss。

## 12. 完成定义与退出条件

09G2只有同时满足以下条件才可从`pending`改为完成：

1. 普通scene可author、保存、重载、cook并触发Cookie、OIT/transparent mode、Planar probe、Diffusion Profile与高级材质；
2. Feature Resolver对所有requested/active/degraded/disabled状态提供唯一truth和reason；
3. Transparent Compositor统一覆盖mesh、sprite、particle、Transmission、half-res与Volumetric depth segment；
4. OIT为HDR-safe且overflow可测，Planar为per-probe generation-safe，SSS有profile/thickness/quality budget，Transmission至少有depth-aware物理分档；
5. Editor具备真实transaction、gizmo/preview、debug和budget surface；
6. 当前源码自动通过验收矩阵，artifact带source fingerprint、GPU/driver、effective settings、资源和GPU timing；
7. 1080p/4K、dynamic resolution、stereo、device loss、VRAM pressure与24h soak达到明确门槛；
8. 与Unreal/HDRP同场景的画质/性能差异有可复现记录，任何未超过项都作为公开gap保留；
9. 旧RGBA8 OIT、共享Planar texture、手工extract-only入口和内嵌冲突profile完成hard cut，不保留双实现；
10. 独立code review与visual review均无Critical/Important遗留。

在这些退出条件之前，文档、capability或plugin manifest不得使用“complete”“production-ready”或“超过Unreal”描述本组能力。
