---
title: Runtime Advanced Surface Lighting、Light Cookie、OIT、Planar Reflection、Subsurface Scattering、Clearcoat、Anisotropy、Transmission 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime100
review_date: 2026-08-22
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_plugins/rendering/features/light_cookies
  - zircon_plugins/rendering/features/oit
  - zircon_plugins/rendering/features/planar_reflections
  - zircon_plugins/rendering/features/subsurface_scattering
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/core/framework/render/advanced_lighting
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/scene/resources/pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/shader/includes/zr_oit.wgsl
  - zircon_runtime/src/graphics/shader/includes/zr_pbr_extras_core.wgsl
  - zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_light_cookie.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_subsurface.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_subsurface.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
tests:
  - zircon_plugins/rendering/features/light_cookies/runtime/src/tests.rs
  - zircon_plugins/rendering/features/oit/runtime/src/tests.rs
  - zircon_plugins/rendering/features/oit/runtime/src/wgpu_product_tests.rs
  - zircon_plugins/rendering/features/planar_reflections/runtime/src/tests.rs
  - zircon_plugins/rendering/features/planar_reflections/runtime/src/wgpu_product_tests.rs
  - zircon_plugins/rendering/features/subsurface_scattering/runtime/src/tests.rs
  - zircon_plugins/rendering/features/subsurface_scattering/runtime/src/wgpu_product_tests.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/tests.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/subsurface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/oit_buffers
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/tests
  - docs/tests/runtime/render/plan18_advanced_pbr_clearcoat_anisotropy_glass_dx12_renderdoc_20260714.rdc
  - docs/tests/runtime/render/plan18_advanced_pbr_clearcoat_anisotropy_glass_three_spheres_wgpu_20260714.png
  - docs/tests/runtime/render/plan18_af_m2_light_cookie_irradiance_volume_dx12_renderdoc_20260715.rdc
  - docs/tests/runtime/render/plan18_af_m2_light_cookie_irradiance_volume_wgpu_20260715.png
  - docs/tests/runtime/render/plan18_oit_three_crossing_transparent_planes_sorted_vs_oit_wgpu_20260712.txt
  - docs/tests/runtime/render/plan18_oit_three_crossing_transparent_planes_sorted_vs_oit_wgpu_20260712.png
  - docs/tests/runtime/render/plan18_planar_mirror_floor_oblique_clip_filter_wgpu_20260712.txt
  - docs/tests/runtime/render/plan18_planar_mirror_floor_oblique_clip_filter_wgpu_20260712.png
  - docs/tests/runtime/render/plan18_sss_skin_sphere_deferred_burley_wgpu_20260712.txt
  - docs/tests/runtime/render/plan18_sss_skin_sphere_deferred_burley_wgpu_20260712.png
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09g1-volumetric-fog-froxel-review.md
  - docs/plans/optimize/zircon_runtime/09g2-advanced-surface-lighting-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/95-runtime-direct-lighting-photometry-light-grid-clustered-forward-plus-shadow-atlas-cascade-point-spot-rect-cookie-ies-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99-runtime-volumetric-fog-froxel-local-fog-volume-lighting-shadow-history-temporal-reprojection-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
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
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/PlanarReflectionProbe.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/HDProbe.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/HDRenderPipeline.SubsurfaceScattering.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/DiffusionProfile/DiffusionProfileSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/ScreenSpaceRefraction.cs
  - dev/bevy/crates/bevy_core_pipeline/src/oit
  - dev/bevy/crates/bevy_pbr/src/transmission
  - dev/godot/scene/3d/light_3d.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/subsurface_scattering.glsl
  - dev/godot/servers/rendering/renderer_rd/effects/ss_effects.cpp
  - dev/godot/servers/rendering/renderer_rd/effects/ss_effects.h
  - dev/Fyrox/fyrox-impl/src/renderer/bundle.rs
  - dev/Fyrox/fyrox-impl/src/renderer/light.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/deferred_spot_light.shader
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Advanced Surface Lighting、Light Cookie、OIT、Planar Reflection、Subsurface Scattering、Clearcoat、Anisotropy、Transmission 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon Advanced Surface Lighting不是一组空接口。Light Cookie有真实GPU atlas和light-buffer metadata；OIT有fragment store、resolve、mesh/sprite replay和storage-buffer capability gate；Planar Reflection已经能从主相机派生反射相机、提交capture target并执行mip filter；SSS有Deferred MRT、8x8 tile classification、indirect dispatch和MaterialAsset生产抽取；Clearcoat、Anisotropy与Transmission也会从可见材质的有效parent链进入pipeline key和WGSL。这些实现应保留为characterization oracle，不能在重构时退回纯演示代码。

但是，现状仍未形成工程级高级材质与透明合成系统。World普通提交流程只显式构造fog与volumetric light ID，Cookie、OIT设置和Planar Probe没有Scene-owned producer；四个Editor feature package只有descriptor/capability壳。SSS与高级PBR虽然补上了生产材质抽取，但SSS不解析parent、不按selected-camera layer过滤，profile仍内嵌于MaterialAsset并靠16个数字slot。Clearcoat、Anisotropy与Transmission又把原本opaque的材质整体移入`Transparent3d`晚前向路径，丢失正常depth prepass与static command cache，而不是进入统一layered material ABI。

三个正确性问题足以阻止产品声明。第一，OIT仍以RGBA8 UNORM存储HDR片元，固定per-pixel K容量溢出即丢弃且没有telemetry。第二，Planar所有probe顺序覆盖同一张1024 RGBA16F mip chain，主视图却选择最小`probe_id`的矩阵，双probe时纹理和参数会确定性错配。第三，Transmission复制的是已经应用体积雾的scene color，forward结果又调用`zr_volumetric_apply`，同一camera segment被重复衰减和加散射。

本轮还发现一个旧09G2未单列的独立P0：`oit.fragment_store` executor会尝试读取`VOLUMETRIC_INTEGRATED`和`TRANSMISSION_SCENE_COLOR`，但OIT descriptor没有声明这两个read resource；resource resolver对未声明资源明确返回`None`。因此即使资源存在，OIT mesh路径也不能得到体积雾或透射背景，这不是偶发兼容性问题，而是图合同上的确定性断链。

旧09G2的 **12项P0全部保持开放**，其中P0-1和P0-7记录到局部进展但不满足关闭条件；本轮新增 **1项独立P0**，另登记 **36项P1、8项P2和44个资格门**。现有10个artifact共24.28 MiB，但全部来自2026-07-12至2026-07-15，画面只覆盖小尺寸合成球体、三平面或单probe；它们是路径证据，不是当前源码产品验收。在Scene/asset truth、唯一feature resolver、HDR透明合成、per-probe资源、Diffusion Profile资产、可伸缩SSS、物理分档Transmission、Editor authoring和竞争性基准闭合前，不得声称本域达到或优于当前Unreal或Unity HDRP。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / bytes / test attributes / ignored | 证据等级 | fingerprint |
|---|---:|---|---|
| Advanced Surface Lighting symbol-bearing current-source总语料 | **264 / 65,285 / 2,497,411 / 650 / 10** | E3覆盖asset、Scene submit、neutral contract、plugin、graph、GPU、shader、Editor package与tests | `64a037a6b3142818fddf2b53af64dbb12e21ec60e1b0a78ec6a896e5f22249aa` |
| production-like源码 | **196 / 45,254 / 1,736,188 / 294 / 0** | E3覆盖产品数据链和内嵌characterization tests | `cce29358bc29171c62204c2fd9117d3f3d82b62981d28eea25f3c846cf215923` |
| focused tests与test support | **52 / 19,762 / 753,056 / 355 / 10** | E3覆盖CPU、Naga、WGPU、product exporter、camera loop、probe resources与shader contracts | `785e753ac6d9ecfbbf1d917f3614fd236cd723cf8d61a15f977cc15266b4711c` |
| 四组Editor feature packages | **16 / 269 / 8,167 / 1 / 0** | E3确认只有manifest、capability、registration和descriptor壳 | `87674a393041cf5787e7135b5e04081b2b1341344438b111b504cbab97fa5eb1` |
| 当前留存产品/RenderDoc artifacts | **10 / 25,461,676 bytes** | E2读取5组PNG/TXT/RDC并完成视觉复核 | `6486960b17d651bc6ee2051bb688fc6cfbb931d30aa3a6b78a27a4f438815792` |
| 五引擎参考切片 | **33 / 19,731 / 870,665** | E2/E3读取Unreal、Unity HDRP、Bevy、Godot和Fyrox具体owner与算法入口 | `c402aaa662740a8a0af0a3a563269c705c5270fc7bdad9a11180c183f59975a6` |

fingerprint算法为：相对路径与每文件SHA-256组成排序manifest，以TAB分隔字段、LF分隔记录，再对UTF-8 manifest执行SHA-256。行数按PowerShell `Get-Content`逻辑统计。冻结对象是2026-08-22共享working tree，不是只读HEAD；基线HEAD为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator epoch为336。

Bevy、Fyrox、Godot与Unity Graphics revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal不是独立Git checkout；本轮以`LightFunctionAtlas.cpp`的SHA-256 `927f0870af60d45b20f16cb47f2787a95a490bccac1ccddbe95a9899a22821f9`作为参考切片锚点，不把父仓revision伪装为Unreal revision。

冻结时`advanced_lighting/extract.rs`、`material_features.rs`、`mod.rs`、Planar camera数学、Transmission与SSS neutral contract等相关文件在共享working tree显示modified，但相关文本diff为空或属于并发Session current blob。本报告不接管、不还原任何Rust/WGSL/Editor修改；实现前必须重新计算fingerprint并重验全部资源合同。

### 2.2 证据等级与明确未做

静态源码可以确定未声明图资源、固定格式、固定容量、选择算法、重复fog调用、Scene producer缺失和Editor symbol缺失；现有TXT/PNG可以评估旧样例覆盖度；RDC存在只能证明当时路径执行过。它们不能替代当前源码的GPU时间、跨平台画质、内存压力、device loss或多视图验收。

本轮只做review，没有修改Rust、WGSL、Cargo、plugin manifest、Scene asset、Editor UI或artifact，没有运行Cargo、Editor/App、WGPU、RenderDoc replay、cook/export、device loss、OOM、large-world、XR、dynamic resolution、透明深度复杂度、动画probe/profile或同画质参考引擎benchmark。tooling按用户要求不纳入本域审查。

### 2.3 Owner边界

| 边界 | Runtime100要求的owner | 禁止的越界 |
|---|---|---|
| Scene/asset truth | `zircon_runtime::scene`与asset层拥有versioned Light Modulator、Planar Probe、Diffusion Profile、材质layer和per-view quality intent | 不由测试手工构造`AdvancedLightingExtract`充当产品producer |
| Neutral contract | `core::framework::render`只携typed、generation-qualified、backend-neutral demand、identity和prepared handle | 不泄漏WGPU对象；不以数字slot或复用light字段冒充稳定身份 |
| Runtime graphics | graphics唯一拥有atlas/pool、透明合成、planar capture、SSS dispatch、layered PBR、residency、PSO、history和fault recovery | 不让每个feature各自修改同一pass顺序或各自解释scene color |
| Feature package | first-party package提供算法和graph contribution，服从统一resolver与prepared generation | capability字符串、pass出现或默认enabled不等于产品active |
| Editor product | `zircon_editor`拥有Inspector、gizmo、profile/material editor、quality/debug、transaction、save/reopen、cook validation | 四个43行级descriptor壳不能算authoring工具 |
| 历史finding | 旧09G2唯一计数12项父P0；Runtime100重验并只新增不同根因 | 不把同一资源/authoring问题跨层可见重复累计为多个P0 |

## 3. 当前应保留的真实基础

1. 四个first-party runtime feature package和typed descriptor是合理扩展边界，应升级而非把算法重新塞回graph compiler。
2. `AdvancedLightingExtract`、`AdvancedPbrMaterialFrameUsage`、`OitSettings`、`PlanarReflectionProbeData`和`SubsurfaceProfileData`提供了可迁移的neutral DTO起点。
3. 可见材质抽取会按selected-camera layer过滤，并在最多4级parent链上生成有效高级PBR feature usage，证明生产材质链不是纯测试注入。
4. OIT已有独立fragment store/resolve executor、mesh replay、sprite支持和storage-binding capability入口，可作为统一Transparent Compositor的一条backend。
5. Planar camera reflection、oblique clip、capture target提交、成功后`mark_captured`和filter dispatch已进入生产camera loop，修正了旧报告“完全没有capture loop”的过时描述。
6. SSS已有Deferred MRT、profile index编码、8x8 tile分类、indirect dispatch、setup/scatter/recombine和MaterialAsset补全路径，可保留为画质oracle。
7. Clearcoat direct/IBL、Anisotropy direct和Transmission材质参数已真实进入shader与pipeline key，可作为layered PBR hard-cut前的数学回归样本。
8. render graph具备feature pass replacement、external resource、attachment ops和queue lane描述能力，足以承载统一owner，不必新增第二套graph。
9. 现有PNG/RDC/TXT能证明旧路径曾执行并提供少量CPU/像素统计，可保留为迁移回归样本，但必须标记stale和非验收。

## 4. 当前产品链与断点

### 4.1 Scene、材质与feature demand

```text
Scene World render extract
    +--> fog volumes / volumetric light ids
    +--> Cookie ----------------------------- X no ordinary Scene producer
    +--> OIT settings ----------------------- X project/camera truth absent
    +--> Planar probes ---------------------- X no ordinary Scene producer

visible meshes + ProjectAssetManager
    +--> effective material parent chain (max 4) --> Clearcoat / Anisotropy / Transmission usage
    +--> direct material only --------------------> SSS profile gather
                                                   X no parent resolution
                                                   X no selected-camera layer filter
```

高级PBR生产链的存在是实质进展，但它只发布“本帧见过某种feature”，没有scene/project requested quality、device eligibility、effective backend、degradation reason、resource budget或last-good generation。SSS使用另一条抽取算法，两个消费者对parent和可见性的定义已经分叉。

### 4.2 Light Cookie

Cookie atlas固定为1024x1024 `Rgba8Unorm`、单mip、8x8网格、最多64个128像素cell，基础纹理成本4 MiB。frame plan先按light ID收进`BTreeMap`，重复ID静默last-wins，再按顺序`.take(64)`；超限没有overflow receipt。每个active frame清整张atlas为白色并重画全部ready texture；未ready纹理仍向light buffer发布metadata，但blit跳过，结果是静默白光。

light packing与atlas executor分别重建等价frame plan，二者靠复制算法保持一致，没有唯一generation receipt。Directional metadata还复用`GpuLightData.position_range.xy`和`spot_angles_size.zw`。atlas bindings只对fragment可见，Volumetric Fog/Froxel不能消费。没有mip、gutter、dilation、HDR/color-space、分辨率tier、IES、Area Light或通用Light Function material。

### 4.3 OIT与透明合成

Zircon当前是固定per-pixel K-buffer风格：每layer为`vec2<u32>`，颜色用`pack4x8unorm`，深度用f32 bit。默认4 layers时，1080p layers加counts为74,649,600 bytes，即71.19 MiB；4K为298,598,400 bytes，即284.77 MiB。`fragments_per_pixel_average`实际被向上取整为每像素固定capacity，命名与分配语义不一致。

fragment count超过capacity后store直接return，resolve只取`min(count, capacity)`；没有overflow counter、pixel mask、tail representation、adaptive budget或fallback。最多精确排序32层，超出`sorted_fragment_max_count`的已存层按encounter order累积。custom material缺`fs_oit`时会让整pass失败；sprite路径每帧重建vertex buffer和bind group。

OIT只替换`transparent-mesh`，不拥有particle、half-res transparency或完整Transmission。更严重的是descriptor只声明depth、shadow、light-grid与OIT buffers，executor却可选读取Volumetric Integrated和Transmission Scene Color。`optional_texture_view_by_name`在pass未声明资源时明确返回`None`，所以这两个产品输入在OIT路径确定性不可见。

### 4.4 Planar Reflection

生产camera loop已能为待更新probe派生反射camera，先提交texture target，再提交主相机，并在整条capture/filter成功后`mark_captured`。这是应保留的真实进展。

但所有probe仍写同一个persistent 1024x1024 `Rgba16Float` mip chain和一个`GpuPlanarReflection` uniform。camera loop可顺序capture多个probe，后执行者覆盖纹理；主相机参数却从layer相交probe中取最小`probe_id`。因此texture内容和matrix/bounds可来自不同probe。主视图只支持一个硬AABB选择，没有priority、distance、plane normal、screen area、blend或per-object assignment。

filter每mip最多5x5简单blur，`radius = mip.min(2)`，不是GGX、depth/normal/distance-aware prefilter。OnDemand只响应显式request/dirty；probe transform、camera、light、visible object、material或capture setting变化不会自动失效。EveryFrame会为所有probe追加capture，没有frustum、occlusion、importance、GPU timing或frame budget。probe resolution虽被clamp，却没有与capture target extent/format/sample强校验，也没有stereo、dynamic resolution、large-world或view-family合同。

### 4.5 Subsurface Scattering

SSS是当前几组功能中最接近真实产品链的一组：Deferred写入SSS buffer与profile index，8x8 classification生成indirect workload，setup/scatter/recombine执行，生产submit会从mesh material补充profile。

但scatter对每个有效像素分别为R/G/B执行64次候选，共192次candidate loop，并重复depth、normal和world-position重建。没有quality tier、half/quarter resolution、temporal stabilization、frame-time controller或GPU timing feedback。normal阈值固定0.55；world thickness由最大scatter radius的0.5或2 pixels推断，没有authored thickness、thickness map、thin/thick transmission、border attenuation或back-light。

profile固定16个数字slot，内嵌在MaterialAsset；显式profile先进入`BTreeMap`，重复ID在进入resolver诊断前已经折叠。material冲突采用first/explicit-wins，diagnostics没有进入RenderStats或Editor。与高级PBR抽取不同，SSS只查直接material handle，不解析parent，也不按selected-camera layers过滤。Forward或MSAA大于1时feature可能从产品graph静默消失。

### 4.6 Clearcoat、Anisotropy与Transmission

任何Clearcoat、Anisotropy或Transmission都会触发`requires_forward_path`。opaque材质因而从普通opaque/depth-prepass/static-command-cache路径移入`Transparent3d` late-forward；Transmission另建command list。这是用phase搬运绕过deferred ABI，不是统一layered material模型。

Clearcoat已有direct与IBL，但F0固定0.04，只支持独立coat normal；没有coat IOR、tint、absorption、独立UV transform或完整map suite。Anisotropy只影响direct light，environment仍用isotropic roughness；没有strength/rotation map和tangent-frame authoring。

Transmission允许0至4次全分辨率`Rgba16Float` scene copy。单次1080p约15.82 MiB，4K约63.28 MiB；四次分别为63.28 MiB和253.12 MiB每帧拷贝量。partition按command count切分，不看tile coverage、depth overlap或material dependency。shader只用world normal XY乘`(ior - 1) * thickness * 0.02`偏移一次scene sample，并clamp屏幕边缘；没有view-space Snell ray、Hi-Z intersection、exit surface、roughness mip/color pyramid、off-screen visibility、temporal或ray-query backend。复制背景已经fogged，forward模板又调用一次`zr_volumetric_apply`。

## 5. 旧09G2 P0 current-source重验

| finding | 当前状态 | current-source证据与关闭要求 |
|---|---|---|
| 09G2-P0-1 Cookie、OIT与Planar无普通Scene producer/authoring | **部分进展但开放** | 高级PBR与SSS已有MaterialAsset生产抽取，Planar有camera loop；Cookie/OIT/Planar仍无Scene/project producer，四个Editor package仍是壳。关闭需versioned component/asset、transaction、save/reopen、cook与生产extract |
| 09G2-P0-2 activation/capability/fallback无唯一真值 | **开放** | OIT、SSS、Cookie各自静默移除、截断或白回退；没有requested/eligible/active/degraded/disabled、reason、quality、cost与generation统一receipt |
| 09G2-P0-3 透明合成无统一owner | **开放** | OIT只替换transparent mesh，particle/half-res/transmission/fog仍分散；graph resource断链进一步证明pass级拼接不能维持组合合同 |
| 09G2-P0-4 OIT HDR压为8-bit UNORM | **开放** | store仍`pack4x8unorm`，resolve仍clamp RGB 0..1；HDR emissive、pre-exposure和bloom energy不可逆丢失 |
| 09G2-P0-5 OIT capacity overflow丢片元且不可观测 | **开放** | slot超capacity立即return；无counter、mask、tail、adaptive budget或降级receipt |
| 09G2-P0-6 Planar共享纹理与最小ID参数错配 | **开放** | filter写唯一persistent mip chain；main params仍`.min_by_key(probe_id)`，双probe确定性错配 |
| 09G2-P0-7 Planar无Scene-owned invalidation/scheduler | **部分进展但开放** | camera loop和success-only mark已落地；依赖图、visibility、importance、time slicing、last-ready fallback和Scene producer仍缺失 |
| 09G2-P0-8 Transmission不是工程级折射 | **开放** | 单次normal.xy屏幕偏移仍无Snell ray、depth hierarchy、exit、rough mip或off-screen fallback；只能定义为Low近似 |
| 09G2-P0-9 Transmission重复应用Volumetric Fog | **开放** | scene copy位于opaque/sky fog之后，forward template对透射结果再次调用volumetric apply |
| 09G2-P0-10 SSS固定192 candidate/像素且无预算 | **开放** | 三通道各64 loop保持不变；没有sample tier、downsample、temporal和frame-time controller |
| 09G2-P0-11 SSS profile/thickness合同不足 | **开放** | 16数字slot、Material内嵌profile、pre-resolver折叠、推断thickness保持；生产gather又缺parent/layer语义 |
| 09G2-P0-12 证据不足以关闭产品gate | **开放** | 10个artifact均为2026-07旧样例，覆盖LDR三平面、单probe、平滑球和合成球体；export/capture tests仍可ignore或skip |

## 6. 新增P0

### Runtime100-P0-13：OIT pass未声明Volumetric与Transmission资源，executor的可选读取确定性返回None

`zircon_plugins/rendering/features/oit/runtime/src/lib.rs`的fragment-store descriptor只声明Scene Depth、Shadow Atlas、Light Grid Params/ZBins/TileMasks和OIT buffers；`graph_execution/render_pass_execution_context/gpu/oit.rs`却调用`optional_texture_view_by_name`读取`VOLUMETRIC_INTEGRATED`与`TRANSMISSION_SCENE_COLOR`。resolver对未声明资源在`resource_lookup.rs`直接`return Ok(None)`。结果不是“资源尚未ready”，而是graph ABI禁止executor看到它们。

关闭要求：Transparent Compositor统一声明每个backend需要的surface/volume/transmission segment资源；compiler验证descriptor与executor access一致；OIT、sorted、particle、sprite、transmission在clear/foggy/multi-layer场景产生同一物理组合oracle；缺资源时输出明确degraded/disabled reason，不能静默绑定fallback。

## 7. P1差距清单

| ID | 差距 | 必须重构为 |
|---|---|---|
| P1-1 | Cookie固定1024、8x8、64个128 cell | device/project预算驱动的variable-size atlas、稳定slot与容量receipt |
| P1-2 | Cookie每帧清整图并重画全部ready texture | dirty-region update、persistent generation、upload/copy budget |
| P1-3 | Cookie缺图仍发布metadata并静默得到白光 | missing/not-ready/evicted显式状态与last-good/neutral策略 |
| P1-4 | Cookie无mip、gutter、dilation、HDR/color-space | filter-safe mip chain、border policy、format与source transform合同 |
| P1-5 | Cookie只覆盖有限projection | Punctual、Directional、Area、IES与material Light Function统一modulator ABI |
| P1-6 | Cookie只在fragment stage可见 | deferred、forward、froxel、GI等消费者共享generation-qualified atlas |
| P1-7 | Cookie复用light buffer其他字段 | 独立`LightModulationHandle`和versioned GPU struct |
| P1-8 | OIT默认显存成本无预算 | per-view memory planner、viewport/dynamic-resolution感知和hard cap |
| P1-9 | OIT只有sorted fallback与固定K | sorted、WBOIT、MLAB/PPLL/K-buffer的capability/quality ladder |
| P1-10 | custom shader缺入口导致整pass错误 | per-material compatibility classification和局部fallback |
| P1-11 | OIT sprite每帧重建buffer/bind group | persistent batch/cache、dirty upload和统一contributor packet |
| P1-12 | capability helper与compiler gate分裂 | 唯一Feature Resolver发布effective backend与reason |
| P1-13 | Planar只按最小ID选择一个probe | priority、distance、plane、screen coverage、layer与blend决策 |
| P1-14 | Planar固定共享1024 RGBA16F | per-probe pool/atlas slice、resolution tier、residency与eviction |
| P1-15 | Planar filter仅简单box-like blur | roughness-aware GGX prefilter和depth/normal/distance rejection |
| P1-16 | Planar capture无frustum/occlusion/importance预算 | view-family scheduler、time slicing和GPU-ms feedback |
| P1-17 | authored resolution与target extent无强校验 | capture descriptor与resource generation原子验证 |
| P1-18 | Planar无stereo、DRS、large-world合同 | per-eye/view-family、camera-relative和resolution-generation设计 |
| P1-19 | SSS在Forward或MSAA > 1时静默消失 | support matrix、explicit fallback与产品状态报告 |
| P1-20 | SSS只有16数字slot | stable content identity、dedup、allocator、overflow与hot reload |
| P1-21 | SSS三通道重复几何拒绝且阈值固定 | shared bilateral cache、profile-driven thickness/normal policy |
| P1-22 | `falloff_rgb`物理语义不清 | diffusion profile参数、单位、能量归一与CPU/GPU oracle |
| P1-23 | 缺皮肤、蜡、叶、眼等workflow | material presets、profile library、thickness authoring和preview |
| P1-24 | SSS与Baked/GI/Volumetric能量归属未冻结 | lighting decomposition与single-application composition contract |
| P1-25 | Anisotropy只影响direct light | direct/IBL/reflection统一anisotropic BRDF和tangent maps |
| P1-26 | Clearcoat层合同不完整 | coat IOR/tint/absorption、normal、roughness与独立UV/map suite |
| P1-27 | Transmission scene-copy step过粗 | coverage/depth/material dependency驱动的copy/resolve planner |
| P1-28 | Cookie重复light ID和超64只靠BTreeMap/take | duplicate/overflow diagnostic、deterministic priority与budget UI |
| P1-29 | light packing与atlas各自重建frame plan | 单一prepared generation同时驱动metadata与atlas publication |
| P1-30 | OIT sorted-limit以外的已存层按encounter order混合 | 有误差界的tail approximation与deterministic oracle |
| P1-31 | OIT不拥有particle和half-res contributor | 统一Transparent Contributor Registry和phase routing |
| P1-32 | SSS生产gather不解析material parent也不按camera layer过滤 | 复用唯一effective-material与visibility resolver |
| P1-33 | SSS冲突/overflow diagnostics停留在临时table | RenderStats、Editor Problems、capture metadata与CI receipt |
| P1-34 | 高级PBR opaque材质被整体移入Transparent3d | layered GBuffer/forward specialization，不牺牲depth/static cache |
| P1-35 | Transmission按command count分step | tile coverage、overlap、depth order和copy bandwidth模型 |
| P1-36 | 全域无统一GPU ms、memory、overflow、residency、quality debug | Advanced Lighting Diagnostics schema、overlay与machine-readable capture |

## 8. P2差距清单

| ID | 差距 | 收敛要求 |
|---|---|---|
| P2-1 | 名称未表达近似层级 | 将screen warp、fixed-K OIT、simple planar blur标成明确quality/backend名 |
| P2-2 | 缺schema version与hard-cut migration | Scene、Material、Diffusion Profile、project quality都必须versioned且无compat shim |
| P2-3 | 缺跨CPU/GPU数学oracle | BRDF、Burley、projection、blend、fog segment与refraction建立同输入oracle |
| P2-4 | 缺feature-specific debug view | atlas occupancy、OIT depth complexity、probe selection、profile/thickness、transmission ray |
| P2-5 | 缺稳定性能数据格式 | GPU timestamp、VRAM、copy bytes、dispatch/draw、overflow和quality generation |
| P2-6 | 缺平台/feature-interaction矩阵 | Deferred/Forward、MSAA/TAA、OIT/fog/transmission、XR/DRS、low/high-end GPU |
| P2-7 | 缺authoring validation与修复建议 | Inspector/Problems提供冲突、容量、unsupported、stale capture的actionable fix |
| P2-8 | 缺持续参考基线与误差记录 | 固定场景、当前源码fingerprint、图像误差、GPU ms和reference provenance |

## 9. 五引擎参考证据与正确借鉴方式

| 域 | Unreal | Unity HDRP / Graphics | Bevy | Godot / Fyrox | 对Zircon的约束 |
|---|---|---|---|---|---|
| Light modulation | `LightFunctionAtlas`定义scene/view级atlas、slot/light mapping、平台尺寸限制，并被Deferred、Volumetric Fog、Lumen等消费者显式请求 | `LightCookieManager`有reserve、relayout、cache、NeedsUpdate、mip、border、2D/cube/area/IES组合和无空间错误 | 无同等级统一atlas切片 | Fyrox把cookie作为SpotLight资产属性贯穿bundle、texture-ready与white fallback；Godot Light3D提供场景属性入口 | Zircon必须先有资产身份、更新/容量/诊断和多消费者合同，再扩展shader花样 |
| OIT | 提供pass type、max sample、debug、method和transmittance threshold控制，默认可选MLAB/regular | 本切片不作为OIT主参考 | Bevy明确标注内存权衡，使用全局node pool、heads、atomic counter、camera最大容量，并列出支持alpha mode与MSAA限制 | 不以传统透明排序冒充OIT目标 | 技术可不同，但必须有quality/backend选择、限制、内存预算、overflow与debug |
| Planar probe | 有独立Scene proxy、reflection view、clip plane、view rect、prefilter与occlusion相关路径 | `HDProbe`持有per-probe realtime color/depth、EveryFrame/OnEnable/OnDemand、pending render steps、time slicing和valid rendered data | 无同等级built-in目标 | Godot/Fyrox不作为本域上限 | 关键不是复制算法，而是per-probe身份、资源、调度、有效性和last-ready publication |
| SSS | 支持Full/Half/Bypass、sample override、checkerboard、profile cache、tile与多种路径 | FrameSettings提供`sampleBudget`和`downsampleSteps`；独立DiffusionProfile有world scale、thickness remap、IOR、transmission tint、filter radius和unique hash | 无同等级built-in目标 | Godot SSS有Low/Medium/High质量和Forward+限制 | Zircon必须把质量、profile资产、单位、thickness、平台fallback和预算做成产品合同 |
| Transmission/refraction | Translucency与Substrate Rough Refraction按material/tile组织，明确与SSS等layer关系 | Screen Space Refraction明确区分Planar/Sphere/Thin模型与edge fade | Bevy明确称Screen Space Transmission，支持0步environment fallback、Low/Medium/High/Ultra taps、depth rejection、exposure compensation与TAA建议 | 传统实现只作fallback参考 | 屏幕空间近似可以保留，但必须明确命名、质量档、depth/roughness处理与off-screen fallback |
| 工程启示 | 场景代理、render graph/RDG、scalability、debug与多消费者共同构成系统 | 资产、Volume/FrameSettings、atlas/probe manager和Editor settings闭环 | 简洁实现也明确限制、资源模型与配置 | 较小引擎仍把功能接入场景资产而非只靠测试DTO | Zircon目标应综合owner纪律与可伸缩性，不把任何单一参考实现当最终上限 |

## 10. 目标架构

```text
Scene components / assets / project quality
    |
    v
AdvancedLightingSceneCompiler
    |-- LightModulatorSnapshot
    |-- TransparentContributorSnapshot
    |-- PlanarProbeSnapshot
    |-- DiffusionProfileSnapshot
    |-- LayeredMaterialSnapshot
    v
AdvancedLightingFeatureResolver
    requested -> eligible -> admitted -> prepared -> active/degraded/disabled
    |
    +--> LightModulationAtlasService
    +--> TransparentCompositor
    |      +--> Sorted / WBOIT / MLAB-PPLL-K backend
    |      +--> TransmissionTracingService
    |      +--> Particle / Sprite / Half-res contributors
    |      +--> depth-segment Volumetric composition
    +--> PlanarReflectionService
    |      +--> Scheduler + ResourcePool + Prefilter + Selection
    +--> DiffusionProfileService + SubsurfaceScatteringService
    +--> LayeredPbrService
    +--> AdvancedLightingResourceService
    +--> AdvancedLightingDiagnosticsService
    |
    v
RenderGraph prepared generations + Editor authoring/debug/capture
```

### 10.1 `AdvancedLightingSceneCompiler`

只从versioned Scene/asset/project truth生成immutable snapshot。它统一解析material parent、render layers、component enable、asset identity、quality intent和mutation generation，禁止Cookie/Planar/SSS各自扫描一遍World并得出不同可见性。

### 10.2 `AdvancedLightingFeatureResolver`

唯一计算requested、device support、render-path eligibility、budget admission、effective backend、fallback、resource cost和last-good generation。feature package只提交需求与算法实现，不得自行静默删除pass或把缺资源解释成白纹理。

### 10.3 `LightModulationAtlasService`

管理2D/cube/octa/IES/area/light-function entry、variable resolution、mip/gutter、dirty update、residency、eviction、generation和多consumer metadata。light buffer只保存typed handle，不复用position/spot字段。

### 10.4 `TransparentCompositor`

统一收集mesh、sprite、particle、transmission和half-res contributor，选择sorted/OIT backend，持有HDR-safe radiance/coverage/depth representation，并按相邻深度段只积分一次participating media。OIT是backend，不再替换产品pass并猜测其他feature资源。

### 10.5 `PlanarReflectionService`

以`probe_id + resource_generation + matrix_generation + capture_generation`原子发布。Scheduler依据view visibility、importance、mutation、age和GPU budget选择capture；ResourcePool提供per-probe slice/texture、last-ready和eviction；Filter和Selection消费同一identity。

### 10.6 `DiffusionProfileService`与`SubsurfaceScatteringService`

Diffusion Profile成为独立content-addressed asset，包含单位、scatter、thickness remap、IOR、transmission、border和版本。SSS按tile/profile/quality调度，支持half/quarter、adaptive sample、bilateral cache、temporal与GPU time feedback；Forward/Deferred/MSAA差异由resolver显式报告。

### 10.7 `LayeredPbrService`与`TransmissionTracingService`

Clearcoat、Anisotropy、Base、Subsurface、Transmission进入统一layered material ABI和能量分配，不再把opaque整体伪装成透明。Transmission至少分为`ScreenWarpLow`、`ScreenRayHiZ`和可选`RayQuery`，具有roughness pyramid、front/back thickness、off-screen environment和透明/fog组合合同。

### 10.8 Resource、Diagnostics与Editor

Resource Service拥有atlas/pool/buffer/PSO生命周期、预热、内存压力、device loss和generation切换。Diagnostics发布GPU ms、VRAM/copy bytes、occupancy、overflow、rejection、stale/fallback reason。Editor为每类资产提供transactional Inspector、gizmo、preview、debug、Problems、save/reopen和cook validation。

## 11. 重构owner与依赖

| 交付物 | 主owner | 前置依赖 | 不应归属 |
|---|---|---|---|
| Scene components、Diffusion Profile、material schema | `zircon_runtime::scene` / asset | serialization、ResourceId、hard-cut migration | graphics executor、Editor plugin字符串 |
| Neutral snapshots与feature receipt | `core::framework::render` | Scene compiler、capability schema | WGPU、shader binding细节 |
| Atlas、OIT、Planar、SSS、Layered PBR backend | `zircon_runtime::graphics` | render graph、resource streamer、pipeline cache | Scene component、project document |
| Algorithm graph contribution | first-party rendering packages | Feature Resolver、prepared resources | 独立feature activation真值 |
| Authoring与debug workflow | `zircon_editor` | Scene/asset schema、diagnostics API | runtime descriptor壳 |
| Product host/capture | `zircon_app` | runtime stable API、artifact schema | 重新实现render feature |

依赖顺序必须是Scene/asset schema -> neutral snapshot/receipt -> feature resolver/resource plan -> backend hard-cut -> Editor/product gate。不得先继续堆shader，再用兼容层补身份与生命周期。

## 12. 分层实施里程碑

### M0：冻结完成度、artifact与feature status

重算current-source manifest；把旧artifact标记为path-only；建立13项P0、36项P1、8项P2和44项gate的machine-readable baseline。任何“完成”状态必须绑定source fingerprint、device、quality、scene和artifact。

### M1：Scene/asset schema与hard-cut migration

新增Light Modulator、Planar Probe、Diffusion Profile、material layer和project quality schema；提供transaction/save/reopen/cook；迁移后删除手工DTO作为唯一产品入口，不保留compat module或双写。

### M2：统一Feature Resolver与prepared generation

发布requested/eligible/admitted/prepared/active/degraded/disabled receipt、reason、quality、cost、generation。Cookie/OIT/Planar/SSS删除独立静默gate。

### M3：工程化Light Modulation Atlas

实现variable entries、mip/gutter、dirty update、residency、capacity、IES/area/light function和多consumer binding；加入overflow、missing、eviction与GPU成本证据。

### M4：建立Transparent Compositor骨架

统一mesh/sprite/particle/transmission/half-res contributor和HDR radiance/coverage/depth packet；冻结fog depth-segment和scene-color阶段语义。

### M5：HDR OIT与overflow治理

替换RGBA8 payload；实现technique ladder、shared pool或明确K-buffer、overflow telemetry、tail approximation、per-material fallback和memory planner；删除pass replacement式孤岛。

### M6：Planar per-probe ResourcePool与原子发布

每probe获得stable resource identity、resolution、generation、last-ready与eviction；texture、matrix、bounds和capture状态必须同generation发布。

### M7：Planar Scheduler、Prefilter与Selection

加入mutation dependency、visibility、importance、time slicing、GPU budget、roughness-aware prefilter、多probe selection/blend、stereo/DRS/large-world。

### M8：Diffusion Profile资产与table

建立content identity、单位、profile library、allocator/dedup/conflict/overflow、hot reload、thickness/transmission参数；SSS与material复用唯一effective-material resolver。

### M9：可伸缩SSS kernel

按质量提供sample/downsample/temporal策略，复用几何拒绝，支持tile/profile dispatch和frame-time feedback；建立皮肤/耳朵/蜡/叶/眼产品场景。

### M10：统一Layered PBR direct/IBL

让Clearcoat与Anisotropy在direct、IBL、reflection和baked/GI上使用同一layer contract；恢复opaque depth/prepass/static cache，补齐map/UV/energy conservation。

### M11：物理分档Transmission

落地ScreenWarpLow、ScreenRayHiZ与可选RayQuery；加入depth hierarchy、exit/thickness、roughness pyramid、edge/off-screen fallback、copy planner和明确质量命名。

### M12：Volumetric与Temporal集成

以Transparent Compositor按segment组合fog，确保每段只积分一次；定义TAA/DRS/camera cut、OIT、SSS、Planar和Transmission的history validity与generation切换。

### M13：Resource/PSO生命周期、Editor与debug闭环

完成PSO预热、cache、内存压力、device loss、last-good fallback；交付Inspector、gizmo、profile/material preview、debug views、Problems、undo/redo、save/reopen和cook validation。

### M14：竞争性产品gate与hard cut

删除旧固定atlas、共享planar texture、RGBA8 OIT、推断thickness、normal.xy transmission和分散feature gate；在同场景、同分辨率、同质量下记录Zircon与参考实现的画质误差、GPU ms、VRAM和稳定性。没有可重复证据不得宣称优于参考引擎。

## 13. 验收资格门

| Gate | 验收要求 |
|---|---|
| G01 | Light Modulator、Planar Probe、Diffusion Profile和layered material均可Scene/asset创建、序列化、save/reopen |
| G02 | 旧schema通过hard-cut migration后无compat module、双写或旧路径引用 |
| G03 | material parent、cycle、missing parent和depth limit由唯一resolver处理并输出diagnostic |
| G04 | selected-camera layers对高级PBR与SSS得出一致可见集合 |
| G05 | Editor undo/redo、multi-edit、copy/paste与cook保留所有高级光照属性 |
| G06 | 空Scene不产生未author的Cookie、Planar、SSS、OIT或Transmission demand |
| G07 | 每feature发布requested/eligible/admitted/prepared/active/degraded/disabled状态 |
| G08 | capability、render path、quality、memory和fallback reason可由Editor与capture读取 |
| G09 | generation切换原子，prepare失败保留last-good且无半更新metadata/resource |
| G10 | graph compiler验证descriptor access与executor resource lookup完全一致 |
| G11 | Cookie支持variable resolution、mip、gutter/dilation和明确format/color-space |
| G12 | Cookie dirty update不再每帧清整图，上传/copy bytes受budget控制 |
| G13 | duplicate、capacity overflow、missing、not-ready、evicted均有counter和reason |
| G14 | Deferred、Forward、Volumetric至少消费同一Light Modulation generation |
| G15 | Punctual、Directional、Area、IES和通用Light Function具有产品场景与golden |
| G16 | OIT payload保持HDR与pre-exposed radiance，>1 emissive和bloom energy通过readback |
| G17 | 深度复杂度超过capacity时无静默丢失，overflow counter/mask与tail error可验证 |
| G18 | 1080p/4K/XR/DRS OIT内存低于项目hard cap并记录实际VRAM |
| G19 | sorted、OIT backend和per-material fallback在同输入上满足误差阈值 |
| G20 | mesh、sprite、particle、half-res和transmission都经Transparent Contributor Registry |
| G21 | clear/foggy/multi-layer场景中每个camera depth segment只应用一次体积介质 |
| G22 | OIT descriptor显式声明并成功消费所需Volumetric/Transmission资源 |
| G23 | 双probe non-overlap时各自纹理、matrix、bounds和generation完全匹配 |
| G24 | 双probe overlap按priority/distance/coverage稳定选择或混合，无最小ID偶然语义 |
| G25 | transform/camera/light/object/material/setting变化按依赖图触发OnDemand invalidation |
| G26 | Scheduler在多probe压力下遵守capture count、GPU ms和age budget |
| G27 | Planar prefilter在roughness sweep上通过reference/golden和mip能量检查 |
| G28 | stereo、DRS、camera-relative large-world与resource resize不产生stale publication |
| G29 | Diffusion Profile具有stable content identity、unique hash、dedup与hot reload |
| G30 | profile slot overflow/conflict不在BTreeMap预折叠，Editor/CI可见完整diagnostic |
| G31 | authored thickness/map、thin/thick transmission和border/back-light进入shader合同 |
| G32 | Low/Medium/High/Cinematic SSS有sample/downsample/temporal定义和GPU ms预算 |
| G33 | 4K大面积皮肤、多视图和动画profile不超过预算且无闪烁/漏色 |
| G34 | skin/ear/wax/leaf/eye场景具有当前源码非ignored画质与能量golden |
| G35 | Clearcoat支持IOR/tint/absorption/normal/roughness/map/UV并满足energy conservation |
| G36 | Anisotropy在direct、IBL和reflection使用一致tangent frame与roughness模型 |
| G37 | 高级opaque材质保留depth prepass、occlusion与static command cache资格 |
| G38 | Transmission Low/HiZ/RayQuery backend有明确capability、quality与off-screen fallback |
| G39 | Transmission copy按coverage/dependency规划，4K多layer copy bytes受budget并可观测 |
| G40 | GPU timestamp覆盖atlas、OIT store/resolve、planar capture/filter、SSS和transmission |
| G41 | debug views覆盖atlas occupancy、depth complexity、probe selection、profile/thickness与ray hit |
| G42 | device loss、OOM/pressure、shader compile failure、missing asset均恢复或显式fail closed |
| G43 | required-GPU lane无adapter时失败或输出受管skip receipt，export/capture验收不得默认ignore |
| G44 | 同场景同质量对Unreal/HDRP记录图像误差、GPU ms、VRAM、稳定性与source provenance |

## 14. 现有测试与artifact判定

四类runtime package和graphics内部已有大量source-inclusion、pass-name、CPU math、Naga/WGPU和product exporter测试；它们能锁住当前ABI与执行骨架。问题是许多测试手工构造`RenderFrameExtract`、feature descriptor、GPU resource或probe list，正好绕过普通Scene producer、Editor transaction和save/cook链。10个ignored attribute也说明部分capture/export不是默认验收。

视觉复核结论如下：

1. Advanced PBR PNG是三个合成球体，只能观察基础clearcoat/anisotropy/glass差异，没有depth、roughness、fog、IBL方向性或透明层交互；RDC不能绑定当前modified source。
2. Cookie/Irradiance PNG把两个feature混在同一对照中，只看到球体颜色变化，不能证明atlas多entry、mip、overflow、missing、volumetric或IES。
3. OIT PNG只有640x360三张交叉LDR平面；TXT记录changed pixels 12,954、MAE 2.2195，CPU submit sorted 192,701us、OIT 302,135us，graph CPU 7,279us对31,441us，fragment store 11,845us、resolve 15,419us。这些不是GPU timestamp，也不覆盖HDR/overflow/particle/fog/transmission。
4. Planar PNG只有640x360单mirror floor与boxes；TXT只保留CPU统计，最终camera stats不含独立filter成本。没有双probe、移动失效、budget或resource identity证据。
5. SSS PNG是640x360平滑球且差异近乎不可见；TXT为changed 27,821、brightened 581、darkened 657、red gain 2,047、MAE 0.0534、max error 6。没有真实皮肤/耳朵/蜡/叶/眼、运动、4K或GPU timing。

因此现有artifact统一定级为L2/L3 path evidence，不得记作G01-G44任何产品gate通过。新的artifact必须携source fingerprint、scene hash、device/driver、resolution、quality、feature receipt、GPU timestamps、VRAM/copy bytes、image error和non-ignored test identity。

## 15. 完成定义与退出条件

Runtime100只在以下条件同时成立时关闭：13项P0全部由当前源码和产品artifact关闭；36项P1完成或有被接受且带期限的明确降级；8项P2进入持续治理；G01-G44全部有machine-readable证据；旧固定atlas、RGBA8 OIT、共享Planar texture、推断SSS thickness、normal.xy transmission和分散activation路径完成hard cut；Editor authoring、save/reopen、cook、debug和fault recovery走真实产品入口；同质量参考基准可重复且没有把CPU提交时间冒充GPU性能。

在上述退出条件前，`implementation_status`保持`not_started`，现有功能应在产品状态中标为`experimental/degraded`，不能因pass存在、PNG有变化、RDC可打开或测试在无GPU时skip而提升为production-ready。
