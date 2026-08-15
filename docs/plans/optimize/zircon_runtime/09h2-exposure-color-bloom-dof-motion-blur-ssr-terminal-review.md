---
related_code:
  - zircon_runtime/src/core/framework/render/post_process
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/asset/assets/scene/post_process.rs
  - zircon_runtime/src/scene/components/scene/post_process.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/scene/scene_renderer/core/constants.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain/visual_export.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_volume.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - docs/tests/runtime/render
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09g1-volumetric-fog-froxel-review.md
  - docs/plans/optimize/zircon_runtime/09g2-advanced-surface-lighting-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessEyeAdaptation.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessLocalExposure.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessCombineLUTs.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessBloomSetup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessFFTBloom.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/DiaphragmDOF.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessMotionBlur.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessDeviceEncodingOnly.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScreenSpaceRayTracing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScreenSpaceReflectionTiles.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/PostProcessing/Components
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/PostProcessing/Shaders
  - dev/godot/servers/rendering/renderer_rd/effects/bokeh_dof.cpp
  - dev/godot/servers/rendering/renderer_rd/effects/luminance.cpp
  - dev/godot/servers/rendering/renderer_rd/effects/tone_mapper.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/screen_space_reflection.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/screen_space_reflection_resolve.glsl
  - dev/bevy/crates/bevy_post_process/src/auto_exposure
  - dev/bevy/crates/bevy_post_process/src/bloom
  - dev/bevy/crates/bevy_post_process/src/motion_blur
  - dev/bevy/crates/bevy_core_pipeline/src/tonemapping
  - dev/Fyrox/fyrox-impl/src/renderer/bloom
  - dev/Fyrox/fyrox-impl/src/renderer/hdr
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09H2 · Exposure、Color、Bloom、DOF、Motion Blur、SSR 与 Terminal 工程化差距

## 1. 结论

Zircon 的通用后处理并不是一组空函数。当前 graph 已能分离 Bloom extract、曝光 histogram/resolve、DOF prepare/final、三级 motion-vector tile/neighbor max、SSR reflection pyramid/specular occlusion/resolve、scene composite、color LUT bake、Uber、upscale 与 output transfer；曝光有 64-bin histogram、百分位裁剪和双速适应，SSR 有 HZB、颜色 pyramid、hit refinement 与 temporal neighborhood clamp，Volume 有生产态 ECS 提取、global/box/sphere、priority/weight/blend distance/layer mask 和 15 个 typed component descriptor。全链 product test 也实际走过 WGPU executor，不应把这些基础误写成“完全未实现”。

但是当前颜色链存在会直接破坏 HDR 内容的 P0。启用 tonemap、color grading 或 LUT 后，color LUT bake 只在 `[0,1]` 输入域生成 32³ LUT；Uber 的 baked-LUT 分支又先把 HDR scene color clamp 到 `[0,1]` 再查表。高于 1.0 的 scene-linear highlight 因而在 tone curve 之前全部折叠到同一个 LUT 边界。若不启用这些效果，默认 `RenderTonemapOperator::None` 又把 Rgba16Float scene color 送入固定 Rgba8Unorm/Rgba8UnormSrgb terminal，仍会直接 clamp。两条默认路径都不能保证 HDR scene-linear 到 display-referred 的正确映射。

`RenderOutputTransfer` 虽公开 `SrgbNonlinear`、`LinearExtended`、`Hdr10Pq`，生产链没有消费该枚举。`output_transfer.wgsl` 只是按整数坐标 `textureLoad` 后原样返回，pipeline 只由固定 target format 决定隐式 sRGB encode；没有 PQ、Rec.2020/P3 gamut conversion、paper white、peak luminance、HDR metadata、OS/display capability 或 calibration。当前“output transfer”是资源路由节点，不是工程级 output device transform。

曝光也没有稳定的时间合同。resolve executor 硬编码 `1/60 s`，适应速度会随真实帧率变化；曝光 history 又复用 09H1 的全局 `history_available`。由于当前 history key 会在正常 camera/object/light/particle 变化时失效，曝光在动态场景中也会频繁重置。直方图虽然是真实 GPU compute，却全分辨率均匀统计所有像素，没有 metering mask、center/spot 模式、local exposure、physical camera calibration 或 compensation curve。

运行时 Volume registry 与场景资产不是同一能力面。registry 声明 15 个 component，但 `ScenePostProcessVolumeProfileAsset` 只能持久化 `volumetric_fog/bloom/color_grading/effect_stack` 四个固定字段，普通 camera settings 也只保存 Bloom、Color Grading 和六个轻量 effect。Exposure、DOF、Motion Blur、SSR、LUT、Blur 等无法由 scene asset 完整保存；LUT 的 volume apply 甚至只改变 layout/size/intensity，并保留 base texture handle。Overlay camera 虽逐相机提交并拥有独立 history key，却复用最初提取的一份 base post-process settings，overlay 自身 component 不会重新提取。

Bloom、DOF、Motion Blur 与 SSR 都有可运行 baseline，但算法与成本还远低于工程引擎。Bloom 是全分辨率 5x5、每像素 25 次 load，radius 只变成最大 4 像素 stride；DOF 在全分辨率做固定 24 gather、用启发式 CoC 并硬 clamp 12 像素；Motion Blur 最终方向读取 dilated neighbor-max 而不是中心像素 velocity，公开 `shutter_angle` 又同时出现 0.5/1.0 与 90.0 两种单位；SSR 虽构建 HZB，却用最多 128 次等距线性步进，HZB 只参与可见性而不做层级空域跳跃，最终还用 `mix(scene, reflection, alpha)` 替换 scene color，没有 material F0/metalness/Fresnel/BRDF 能量合同。

本轮登记 8 项 P0、30 项 P1、9 项 P2。重构顺序必须先冻结 scene-linear/working-space/display-space 与 output device 合同，修复 HDR LUT domain 和默认 tone curve；再拆分曝光独立 history/真实 delta time；随后收敛可序列化 Volume schema；之后才能分别升级 Bloom、DOF、Motion Blur、SSR 和终端 HDR。09H1 继续拥有共同的 resolution/history/velocity authority，09H2 不建立第二套 temporal truth。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | 文件 / 物理行 | 本轮判定 |
|---|---:|---|
| production focused set | 239 / 25,878 | E3：authoring schema、Volume evaluation、pass graph、resource format、executor、WGSL、camera stack、terminal output |
| production focused fingerprint | 239 / 25,878 | `cd83813b175061a628d312e382771831816d8d2da2c9af2eb9bd24492d7d4dc9` |
| production 文件内 test 属性 | 185 | E2：settings、params、shader parse、resource/cache、source contract |
| dedicated focused tests | 25 / 9,469 | E2：96 个 `#[test]`，其中 1 个 ignored artifact exporter |
| 名称命中的历史 artifact | 67 / 48,982,850 bytes | E1-E2：26 exit、32 log、1 PNG、8 RDC；均早于本轮源码指纹 |
| Reference engine 主链 | Unreal 10 组、Unity HDRP 2 目录、Godot 5 组、Bevy 4 组、Fyrox 2 组 | E3：color pipeline、exposure、effect algorithm、volume/output contract |

focused fingerprint 的算法与 09H1 相同：路径排序后逐文件 SHA-256，再对 UTF-8 `path<TAB>hash<LF>` 清单计算 SHA-256。范围包括 core post-process ABI、scene/asset persistence、Volume/camera stack、graph routing、全部 production post-process executor/WGSL 与 terminal owner；test-only 文件单列。当前 42 个 focused production 文件处于其他 Session 的 modified/deleted/untracked 状态，本文不修改或回退它们，实施前必须重新取指纹并复核结论。

### 2.2 数据链读取深度

本轮从 camera component 和 scene document 开始，追踪 project I/O、World ECS、volume collider/layer extraction、per-camera evaluation、camera-loop source restore、effective settings、feature/budget gate、stack/pass graph、resource descriptor、pipeline/bind group、settings upload、shader sample/composite、history/readback、terminal attachment、overlay/UI 顺序、stats、product tests 与 artifact exporter。

shader 审查不以“pass 存在”为完成。曝光核对 histogram weighting、percentile、时间积分和 history validity；LUT 核对输入域、采样/插值和 tone/output placement；Bloom 核对 prefilter、soft knee、pyramid 与能量；DOF 核对物理 CoC、near/far 分层、tile/gather/scatter；Motion Blur 核对 velocity space、shutter unit、tile/classification、中心/邻域权重；SSR 核对 traversal、rough ray、denoise、history rejection、BRDF composition 与 fallback；terminal 核对 gamut/EOTF/display metadata/UI。

### 2.3 当前工作区边界

关键重叠修改集中在 post-process stack/resource/shader、HZB、motion tile、LUT bake、DOF prepare、terminal cache 和 `scene/world/render_post_process.rs`。另有旧 SSAO 文件删除。本轮只写 `docs/plans/optimize`，不把这些并发改动整理成自己的 change，也不假定它们已经通过集成验收。

### 2.4 与相邻审查的 owner 边界

- 09A 拥有 Render Graph、transient alias、persistent GPU object、queue/fence/device loss；09H2 定义每个 effect 的资源 format/extent/lifetime 语义。
- 09C 拥有 shader/material/PSO ABI 与 custom pass compile；09H2 定义 post-process parameter、working color space、injection point 与 permutation contract。
- 09D 拥有 LUT/curve/mask/lens texture artifact、resident generation 与 streaming；09H2 消费 ready generation 并发布 degraded reason。
- 09F1/09G2 拥有 reflection probe、planar reflection、transmission 与 surface BRDF；09H2 的 SSR 只能贡献 screen-space indirect specular，不能覆盖其 fallback/energy owner。
- 09G1 拥有 volumetric fog；09H2 的 lightweight screen fog 必须明确是独立产品还是被 hard cut，不能重复介质积分。
- 09H1 拥有 ViewFamily resolution、velocity、jitter、history domain、dynamic resolution 与 upscaler phase；Exposure、SSR、DOF temporal data 必须注册到同一个 typed history system。

### 2.5 明确未做

本轮没有运行 Cargo、WGPU、Editor、RenderDoc 或参考引擎，没有重导出 artifact。没有执行 HDR monitor/PQ、camera pan exposure、bright/dark transition、high-nit highlight、LUT domain chart、Bloom firefly、foreground DOF、fast thin object motion blur、rough/offscreen SSR、split viewport、camera stack、4K/8K、stereo/XR、device loss、VRAM pressure或同画质 benchmark。静态源码足以证明颜色与持久化合同冲突，但不能替代修复后的产品验收。

## 3. 当前可保留的真实基础

### 3.1 Pass graph 与资源命名已形成可迁移骨架

Bloom、Exposure、DOF、Motion Vector Tile/Neighbor、Motion Blur、SSR pyramid/occlusion/resolve、Scene Composite、LUT Bake、Uber、Upscale 和 Output Transfer 都有独立 kind、resource name、executor 与顺序测试。后续应升级资源语义和算法，不能退回一个无法分析/计时的巨型隐式 post pass。

### 3.2 Histogram exposure 是真实 compute baseline

64-bin workgroup histogram、global accumulation、low/high percentile、manual/histogram mode、separate brighten/darken speed、双缓冲 exposure buffer和 readback parser 都存在。其问题是 metering/time/history/physical semantics，不是完全没有 auto exposure。

### 3.3 Motion-vector dilation 三阶段值得保留

2x2 half-resolution tile max、2x2 coarse max 与 full-resolution 3x3 neighbor max 已形成独立 pipeline。它可迁移为更完整的 velocity hierarchy；问题是最终 filter 把 neighbor max 当中心 velocity，以及缺少 tile classification/quality/coverage contract。

### 3.4 SSR 不应被误判为占位 shader

当前 SSR 已读取 min/max HZB、构建 half-resolution reflection color pyramid、执行最多 4 次 hit refinement、按 roughness 选 mip、做 temporal reprojection/neighborhood clamp，并有 specular occlusion pass。这些阶段可保留。需要替换的是线性 trace、单镜面方向、弱 history validation 与非 BRDF composite。

### 3.5 Volume evaluator 的 typed descriptor 方向正确

15 个 builtin descriptor 有 parameter schema、type check、read/apply function 与 interpolation policy。World 路径会提取 active volume、形状、priority、weight、blend distance 和 layer。应让持久化/Editor/profile asset真正承载这套 schema，而不是删除 registry 回到四字段结构体。

### 3.6 Camera stack 与 terminal ownership 有工程基础

Base/Overlay 会逐相机提交，camera history key 按 descriptor 分离，attachment clear policy 与 stack/viewport terminal output owner明确，屏幕 UI 只交给最后的 terminal camera。这一套 ownership 可保留；需要补的是逐相机 base settings和 HDR/display-space UI contract。

### 3.7 Product tests 至少证明 executor 可达

2026-07-05 的旧日志记录 12 个 post-process product tests 全通过，耗时 72.46 秒，覆盖 motion blur、tonemap/grading、light effects、user LUT、blur、DOF、full chain、fog、SSR、terminal AA/upscale 和 volume transition。这能证明当时二进制执行过相关节点，不能证明当前指纹或画质正确。

## 4. P0：必须先修复的正确性与产品合同

### P0-1：Baked LUT 在 tone curve 前把 HDR 输入 clamp 到 `[0,1]`

`color_lut_bake.wgsl` 只为 normalized cube 生成 `apply_color_grading(apply_tonemap(source_color))`。Uber baked mode 调 `sample_effect_lut_3d(color)`，采样函数先 `clamp(color, 0, 1)`。scene-linear 1、10、100 的 highlight 会命中同一 LUT 边界，无法恢复。当前 full-chain test只检查 frame delta/chromatic pixels，没有 HDR ramp oracle，因此不会发现该错误。

必须定义 HDR shaper domain，例如 log2/LogC/ACEScct-like working encoding，LUT 存储与 shader sample使用同一 transform；或把 tone curve 留在 analytic pass、LUT 只处理明确的 grading domain。验收必须用超 1.0 ramp、wide-gamut primaries、negative/over-range值和 CPU reference。

### P0-2：默认 `TonemapOperator::None` 会把 scene HDR 直接写入固定 LDR target

`TONEMAPPED_SDR_FORMAT` 固定 Rgba8Unorm，`FINAL_COLOR_FORMAT` 固定 Rgba8UnormSrgb。默认 operator 是 `None`，无 effect 时 output transfer也只是 copy。任何超过 1.0 的 scene color在 attachment conversion时 clamp。引擎不能把“用户未显式配置 tone mapper”解释为“允许破坏 HDR”。

应引入明确的 scene-linear working format与 display transform默认值。`None` 只能用于 HDR-linear capture、debug或显式 external transform target，并由 target capability决定；普通 SDR surface必须有稳定默认 tone curve与 exposure/pre-exposure contract。

### P0-3：`Hdr10Pq`/`LinearExtended` 是未接生产链的 false surface

`RenderOutputTransfer` 只在定义/测试/re-export出现，executor没有参数，shader没有 transform。当前不存在 PQ EOTF、Rec.2020/P3 matrix、paper white/peak nits、display min/max、HDR metadata、surface negotiation或fallback。选择 target format也不能等价于 HDR10 mastering/display encoding。

在真正接线前，公开 capability必须报告 unsupported；不得以 enum存在声称 HDR10。重构后 output device plan需由surface/capture target解析，包含working/output gamut、transfer function、luminance、bit depth、alpha、UI policy与metadata owner。

### P0-4：Exposure 复用全局 history invalidation 且时间步固定 1/60 秒

`EXPOSURE_ADAPTATION_DELTA_SECONDS` 硬编码 `1.0/60.0`。同一 speed在30/60/144 Hz表现不同。更严重的是曝光 history 在全局 history unavailable时被统一 invalid；09H1 已证明正常 camera/scene变化会触发该状态。因此动态场景中的 auto exposure无法稳定按真实时间连续适应。

Exposure 必须成为独立 history domain，只有camera cut、manual reset、metering schema/range change、target generation等事件重置；真实 clamped delta time由frame timing传入。M1/M2由09H1 typed history owner提供基础，09H2定义 exposure-specific reset/transition。

### P0-5：Scene/Volume 持久化能力远小于运行时 registry

运行时注册 Exposure、DOF、Motion Blur、SSR、Fog、Color Grading、Tonemap、Vignette、Grain、Dither、Chromatic Aberration、LUT、Blur、Bloom、Volumetric Fog；`ScenePostProcessVolumeProfileAsset` 只有四个 Option，普通 settings又只保存 Bloom/Color Grading和六个 effect字段。用户无法可靠 save/reload/cook 主要功能，旧资产也没有 versioned migration。

必须以 component id + schema version + override state + typed values/resource references形成可序列化 profile asset，builtin只是registry内容。未知 component要保留opaque payload，migration失败要可诊断。固定四字段结构完成 hard cut 后，Editor/runtime/cook读取同一 profile truth。

### P0-6：Motion Blur 的 shutter unit与最终 velocity source互相错误

shader把 `shutter_angle` 当 0..1 shutter fraction；调用与测试同时出现 0.5/1.0 和 90.0，upload没有 `/360`。90会把 exposure vector放大后依赖component clamp，单位没有类型保护。最终 gather又只读 `motion_vector_neighbor_max`，中心/sample direction都使用 dilated field；静止背景会被邻近高速前景拖拽，薄物体边缘出现跨层 bleed。

API应明确 `shutter_angle_degrees` 或 `shutter_fraction`，序列化/Editor显示同一单位并提供 frame-rate/shutter-time mode。filter必须同时读取per-pixel velocity/depth和tile/neighbor envelope，邻域只决定 search radius，不替代中心surface velocity。

### P0-7：SSR composite 没有 material specular/BRDF 能量合同

resolve产生的 alpha被硬乘 `0.18` 并 clamp到 `0.35`，scene composite随后 `mix(scene, reflection.rgb, reflection.a)`。路径不消费 F0、metalness、specular、Fresnel或 indirect-specular energy，也没有“替换 probe/IBL 的哪一部分”的定义。结果可能把 diffuse/lighting整体替换为 screen color，或与 probe/IBL双重计能。

SSR应输出 radiance + hit confidence/roughness/distance，由统一 indirect-specular compositor结合 material BRDF、visibility和 probe/ray fallback。magic cap必须被物理/质量参数替换，并用 dielectric/metal/roughness矩阵验证能量和fallback连续性。

### P0-8：当前 artifact 无法证明颜色与效果正确，最新导出未完成

唯一命中的 post full-chain PNG 是 320×240，画面大面积洋红/青色静态颗粒，不能作为 tone/Bloom/DOF/SSR golden。8 个 RDC 和旧日志来自 2026-07-29/30 或更早；2026-08-01 current-source exporter日志显示 session coordinator `command_post_timeout`，没有产出声明的 current-source PNG/JSON。旧 12-test pass又绑定不同二进制路径与源码。

在没有当前 fingerprint 的 HDR/SDR image sequence、GPU profile、graph/resource manifest和 reference metric之前，任何“postprocess complete/production ready”状态都必须保持 false。

## 5. P1：算法、性能、扩展与闭环差距

### P1-1：Histogram 全分辨率均匀计量，没有 metering mask

每个 full-resolution pixel都参与64-bin atomic histogram。缺少 center-weighted、spot、mask texture、ignore-material/UI/sky policy和downsample/reduction hierarchy。Bevy baseline已支持 metering mask，Unreal还有material ignore和black bucket influence；Zircon当前既更慢又更难控制。

### P1-2：曝光语义没有物理相机/校准

shader直接把 `log2(luminance)` 当 EV100，未定义cd/m²、middle gray、lens attenuation、ISO/shutter/aperture与pre-exposure。manual EV只是倍率接口，不能和physical camera/render light units形成可验证关系。

### P1-3：缺少 Local Exposure 与 compensation curve

没有局部亮度分解、bilateral/local contrast、highlight/shadow detail、曝光补偿曲线和debug view。Unreal将 Local Exposure独立成pass并提供可视化；Zircon只能用全局标量应对高动态范围。

### P1-4：所谓 `Aces` 只是逐通道经验拟合曲线

当前2.51/0.03/2.43/0.59/0.14公式不是完整 ACES working-space/gamut/tone/output pipeline，没有 chromatic adaptation、gamut compression、output transform或ACES version。应准确命名为拟合 curve，或实现明确版本的 ACES/AgX/自有 tone pipeline并用 reference vector验证。

### P1-5：Color Grading 只有五个全局标量/向量

只有 exposure、contrast、saturation、gamma、tint；缺少 white balance、lift/gamma/gain、offset、shadows/midtones/highlights、channel mixer、curves、hue-vs-hue/sat/luma、working-space selection。Unreal Combine LUT和HDRP components显示了工程级 authoring下限。

### P1-6：2D strip LUT 用 nearest `textureLoad`

2D strip路径没有 trilinear/tetrahedral interpolation，色阶会按格点跳变；3D LUT虽用 sampler，也没有明确 filter/capability/precision contract。应统一 LUT sampling oracle、layout validation与边界测试。

### P1-7：LUT 固定 32³，64³常量未进入质量/预算策略

`COLOR_LUT_SIZE_DEFAULT` 为32，高质量64常量没有 production consumer。缺少按output/gamut/quality选择的尺寸、cache key、async bake、dirty component tracking和 resident generation。每次变化何时重烘、失败后用哪一代也未成为产品状态。

### P1-8：Bloom 是全分辨率固定 25-load filter

一次 fullscreen pass对每像素做5x5 `textureLoad`，radius只映射到最多约4像素stride。没有 half/quarter pyramid、downsample/upsample/scatter，因此大半径既不真实又浪费带宽。Bevy轻量实现也已有 mip pyramid，HDRP分为 prefilter/blur/upsample。

### P1-9：Bloom threshold公式会产生超线性亮斑

当前 `sample_color * max(luminance-threshold,0)` 使高亮颜色再乘亮度差，容易产生平方式能量增长。没有 soft knee、anti-firefly、exposure-aware threshold或energy normalization。

### P1-10：Bloom 缺少工程级 lens/output 特征

没有 scatter/tint、anamorphic ratio、dirt mask、convolution/FFT kernel、starburst、lens flare integration和 quality tiers。Unreal同时有standard Bloom与FFT Bloom；这些应作为可组合档位，不应全部塞进同一固定shader。

### P1-11：DOF enable predicate 与 prepare predicate矛盾

settings在 `aperture > 0 || max_blur_radius > 0` 时enabled，prepare却要求两者同时大于epsilon。某些合法输入会插入pass但 prepare写空，产生无效成本/意外无效果。唯一sanitize/enable合同应在CPU resolver完成。

### P1-12：DOF CoC 是启发式，不是镜头模型

CoC由 `(depth-focus)/focus_range * aperture * focal_length/50 * max_radius` 计算，没有sensor size、f-number、focal distance、viewport pixel pitch、near/far sign和camera projection的物理关系。focal_length字段因此只是一种倍率。

### P1-13：DOF 全分辨率固定 gather且硬上限12像素

prepare全分辨率写CoC/Bokeh，final对 procedural 与 prepared各12次sample，最大半径被 `DOF_MAX_FINAL_PASS_RADIUS=12` 截断，不论authoring值。没有tile max、half/quarter gather、mip sampling、indirect dispatch或大CoC scatter。

### P1-14：DOF 没有稳定的 near/far layer 与透明语义

CoC虽分channel，最终仍在一个gather里合成；没有foreground dilation/occlusion、far precombine、near/far independent blur、transparency/particle DOF或highlight scatter。Unreal Diaphragm DOF和HDRP均有tile/dilate/gather/combine分层。

### P1-15：Motion Blur sample count固定且没有quality model

所有像素按authoring 1..32固定采样，不随velocity length/tile class/quality自适应；sample为整数间隔，缺少blue-noise/jitter、half-res gather、separable quality、camera/object policy和rolling shutter。

### P1-16：Motion Blur 的depth rejection是magic constant

1像素speed threshold、0.01/0.05 depth threshold直接写在shader，未与projection/depth linearization/scene scale关联；没有velocity confidence、coverage或transparency alpha策略。应由versioned filter settings和linear depth contract控制。

### P1-17：SSR 的 HZB 没有承担层级 traversal

trace仍把max distance均分为最多128步，每步再读取多个HZB mip做visibility；没有cell boundary advance、mip ascend/descend或empty-space skipping。Godot local reference会根据hit在mip层级上下移动，说明Zircon当前“有HZB”不等于hierarchical tracing。

### P1-18：SSR 全分辨率全像素trace，没有material tile classification

默认64步并对所有像素执行，roughness/validity没有在dispatch前产生tile list/indirect args。Unreal有SSRT tile classification与indirect路径；Zircon应按roughness/material/screen region和quality分配ray budget。

### P1-19：Rough SSR 仍只追一条perfect reflection ray

roughness只控制color mip与fade，没有GGX/VNDF方向采样、multiple ray quality、spatial reservoir或roughness-aware denoiser。高roughness只是模糊单条镜面命中，不能代表microfacet积分。

### P1-20：SSR temporal history验证信息不足

reprojection使用 neighbor-max velocity，没有previous depth/normal/roughness/material/moments/confidence；history RGB按当前scene-color 3x3 bounds clamp，而不是reflection-domain statistics。current miss时history也被current trace alpha压掉，无法稳定填充稀疏ray。

### P1-21：SSR 没有显式 offscreen/miss fallback

Scene Composite只接resolved SSR；reflection probe/IBL在其他阶段独立相加，没有screen hit/probe/planar/ray fallback的单一优先级和能量替换规则。边缘fade、miss和粗糙表面会出现不连续或double lighting。

### P1-22：General Blur 复用昂贵的DOF gather family

Blur只有radius一个参数，却走全分辨率DOF-style固定gather并同样clamp 12像素。没有separable Gaussian/Kawase/mip/bilateral、edge mode、quality/cost或mask。它不应作为任意工程blur primitive被继续复用。

### P1-23：Film Grain 与 Dither 是静态、单通道相关噪声

`effect_noise(coord, scale)`没有frame index/blue-noise sequence，grain和dither每帧固定，RGB共享同一噪声，并在tone mapping之前应用。没有film response/color、output bit-depth或temporal decorrelation，容易形成固定纹理和错误量化位置。

### P1-24：Chromatic Aberration 与 screen fog 是轻量启发式

Chromatic Aberration只做水平正负offset，不是径向lens/spectral model；screen fog由UV.y和normalized depth推导，不做world-height密度积分，还与独立volumetric fog并存。名称和authoring必须反映算法，或升级/合并owner。

### P1-25：Local Volume 只支持Box/Sphere，其他Collider静默消失

Capsule、Cylinder、ConvexHull、TriangleMesh、HeightField、Compound均返回 `None`。现有测试甚至固化capsule“不投影到extract”。运行时/Editor没有清晰degraded diagnostic，用户只会看到volume无效果。

### P1-26：Volume registry 没有可持久化resource parameter

schema type只有float/vec3/bool/uint/enum等值类型；LUT apply保留base texture handle，volume无法覆盖具体纹理。后续metering mask、dirt、curves、custom effect resource也无统一引用/插值/ready-generation合同。

### P1-27：Overlay camera复用base camera的post-process base settings

camera loop逐descriptor选择camera，但 `CameraLoopPostProcessSourceState` 只恢复最初extract的Bloom/ColorGrading/EffectStack/Volumes。World extraction只从初始 `scene_camera_entity` 读取一次 `PostProcessSettingsComponent`。Overlay的volume位置/mask正确，Overlay自身base component不正确。

### P1-28：Terminal后的Overlay/UI没有HDR composition policy

SDR下 output transfer之后绘制debug overlay/UI是合理起点；但HDR output需要UI reference white、gamut/EOTF、alpha/composition plane和capture policy。当前固定sRGB8掩盖了该问题，真正接HDR时不能继续在未知encoding上直接blend。

### P1-29：缺少统一quality/scalability与per-effect预算

Budget degrade只做有限feature gate和SSR mip bias。没有Bloom mip/quality、DOF gather/scatter budget、Motion Blur quality、SSR ray/denoise tier、LUT precision、exposure resolution，以及按GPU timing/VRAM的结构化fallback。

### P1-30：测试以pass order和单帧pixel delta为主

96个dedicated tests大量检查节点/资源/changed pixels。没有HDR ramp oracle、tone vector、exposure时间序列、Bloom energy/firefly、DOF foreground occlusion、velocity edge、rough SSR、output PQ、camera stack profile、4K GPU budget或device loss。旧WGPU pass不能替代这些验收。

## 6. P2：完整产品能力与可维护性差距

### P2-1：缺少lens distortion、panini、lens flare统一链

当前只有水平chromatic offset和Bloom。工程相机需要可排序的distortion/undistortion、panini、vignette、chromatic、data-driven/screen-space flare与calibration asset，并明确TAA/DOF/UI前后位置。

### P2-2：缺少display calibration与working color space authoring

没有project working primaries、white point、chromatic adaptation、display profile/EDID override、calibration chart、capture EXR path和content mastering metadata。

### P2-3：Custom post-process injection point未与Volume/profile统一

plugin graph有扩展基础，但Volume registry、scene asset、shader/material ABI、resource readiness和before/after tone/transparency/UI injection没有形成一套用户可author、可cook、可诊断的合同。

### P2-4：缺少per-effect debug/visualization

需要 histogram/meter mask、exposure EV、local exposure、Bloom mip、CoC/near/far/tile、velocity/tile/neighbor、SSR ray/hit/confidence/history/rejection、working/output gamut clipping和tone curve view。

### P2-5：缺少效果级GPU timestamp与带宽/occupancy证据

graph有pass profile基础，但报告没有标准的per-effect ms、dispatch/draw尺寸、samples/pixel、read/write bytes、transient peak、history bytes和cache bake counters。

### P2-6：缺少asset migration与unknown-component round trip测试

从固定字段迁移到schema profile时必须覆盖旧scene、未知plugin component、missing resource、rename/version、undo/redo、save/reload、cook/pack和runtime hot reload。

### P2-7：缺少2D/stereo/XR/orthographic边界合同

DOF/SSR/motion/fog默认透视3D假设。需要明确2D camera、orthographic、stereo eye history、foveated/dynamic-resolution、subrect/split viewport和multi-display输出。

### P2-8：缺少高精度capture与离线reference路径

普通capture只得到final RGBA8。需要scene-linear EXR、pre/post tone、effect intermediates、histogram/EV metadata和deterministic offline reference，才能做客观差异和回归。

### P2-9：缺少竞争性同场景基线

没有在冻结硬件/driver/resolution/quality/content下对照Unreal、HDRP、Godot以及Bevy/Fyrox轻量路径的质量、GPU时间、VRAM和稳定性。当前不能从pass数量推导“超过Unreal”。

## 7. 参考引擎对照

### 7.1 能力矩阵

| 域 | Zircon 当前 | Reference contract | 必须补齐 |
|---|---|---|---|
| exposure | 64-bin全屏histogram、固定1/60、全局history | metering mask/curve/physical calibration、真实dt、独立history/local exposure | metering owner、frame timing、physical EV、local exposure |
| tone/color | normalized 32³ LUT、简化curve、五个grading参数 | HDR shaper/working space、white balance、range grading、gamut/output transform | HDR-safe LUT或analytic tone、versioned color pipeline |
| output | shader copy到固定sRGB8 | target/display capability、SDR/HDR EOTF、gamut、paper white/metadata | output device plan与surface/capture negotiation |
| Bloom | full-res 5x5/25-load | soft-knee prefilter、mip pyramid、scatter/convolution、lens assets | pyramid与quality/budget |
| DOF | heuristic CoC、full-res 24 gather、12px cap | physical lens、CoC tile/dilate、near/far gather/scatter/recombine | physical camera + layered scalable pipeline |
| Motion Blur | 三级max、固定sample、neighbor-max direction | typed shutter、per-pixel velocity + tile envelope、classification/quality | velocity-aware filter与单位contract |
| SSR | HZB visibility + linear steps、single ray、scene mix | hierarchical traversal、tile/ray quality、rough denoise、BRDF/fallback | indirect-specular compositor与denoiser |
| Volume | typed runtime registry、fixed asset profile | serializable component override schema、resource refs、custom components | registry/asset/editor/cook同一truth |
| evidence | pixel delta、旧PNG/RDC/log | HDR sequence/golden/reference、GPU/VRAM、capture manifest | current fingerprint artifact gate |

### 7.2 Unreal 提供工程上限

`PostProcessEyeAdaptation.cpp`使用真实 `DeltaWorldTime`，提供histogram百分位、speed up/down、compensation settings/curve LUT、pre-exposure和更多metering控制；`PostProcessLocalExposure.cpp`把局部曝光独立成可观察阶段。`PostProcessCombineLUTs.cpp`包含working color space、white temp/tint、global与shadow/midtone/highlight saturation/contrast/gamma/gain/offset、ACES参数、多个LUT blend和output device。

`DiaphragmDOF.cpp`提供half-resolution gather、CoC tile/dilate、foreground/background、hybrid scatter、recombine、ring/mip/quality与bokeh LUT。`PostProcessMotionBlur.cpp`有velocity flatten、gather/scatter dilation、tile classify、half-res/separable和quality档。`ScreenSpaceRayTracing.cpp`有roughness/quality/ray count、tile classification、indirect执行和denoiser输出；`PostProcessDeviceEncodingOnly.cpp`消费output format/gamut/max luminance/paper white/HDR target。

Zircon不需要复制所有CVar，但必须达到同等级的typed contract、可观测状态、scalable algorithm和output truth。

### 7.3 Unity HDRP 提供模块化产品下限

HDRP将Bloom拆为Prefilter/Blur/Upsample compute；DOF有CoC、TileMax、Dilate、Mip、Prefilter、Gather、PreCombine/Combine和indirect args；Motion Blur有motion-vector prep、tile generation/merge/neighborhood与filter；Exposure有fixed/automatic/curve/physical相关authoring，Tonemapping/Color Curves/LUT Builder则形成独立color pipeline。Custom Post Process还有明确injection point/volume component boundary。

这说明即便不采用Unreal架构，工程级后处理也应有可分层、可裁剪、可扩展和可budget的stage，而不是每个效果一个固定全屏kernel。

### 7.4 Godot、Bevy、Fyrox 提供轻量实现下限

Godot SSR shader会沿HiZ cell/mip ascend/descend，resolve按depth/normal/roughness权重组合，并根据roughness估计blur mip；另有luminance reduction、tone mapper与Bokeh DOF owner。Bevy auto exposure已经支持metering mask，Bloom使用soft threshold和downsample/upsample mip，Motion Blur文档明确shutter为0..1 fraction。Fyrox有独立HDR luminance/adaptation与Bloom blur链。

因此Zircon当前某些问题并不是“只有大型商业引擎才需要”：Bloom pyramid、metering mask、明确shutter单位和层级SSR在仓内轻量参考中也已有直接证据。

## 8. 目标架构

### 8.1 唯一的 color pipeline plan

每个view必须解析一份immutable plan：scene working color space、pre-exposure、tone/grading implementation、LUT shaper/domain/precision、output gamut/EOTF/luminance、alpha/UI/capture policy与quality。Graph、shader、surface和stats都消费该plan，禁止分别从format/default enum推断。

### 8.2 Typed post-process component/profile asset

Profile保存component id、schema version、enabled/override状态与typed parameters/resource references；registry负责default/interpolation/sanitize/apply/editor metadata/migration。builtin与plugin使用同一机制，未知payload round-trip，resource readiness有generation/degraded reason。

### 8.3 Per-effect runtime plan与budget

resolver把authoring + quality + capability + ViewFamily生成 effect plan：phase、input/output/history、resolution tier、algorithm variant、sample/ray/mip budget、async eligibility、fallback reason。Executor不自行二次判断magic constants。

### 8.4 Shared typed history，不共享单一valid bool

Exposure、SSR、DOF temporal、TAA/TSR、volumetric各自声明schema、extent、reset reason和success generation。Camera cut可以广播事件，普通motion不是全域reset。所有consumer可独立失效/迁移/观察。

### 8.5 Indirect-specular compositor

SSR/planar/probe/IBL/ray provider输出radiance、visibility/confidence与coverage；compositor结合material BRDF/Fresnel/roughness和fallback priority，明确替换/补充哪部分indirect specular。不得让screen pass直接mix整个scene。

### 8.6 Output device与UI composition owner

Surface/capture target提供format、gamut、EOTF、luminance/metadata能力；output stage执行tone/output transform。UI/overlay声明scene-linear或display-referred及reference white，HDR/SDR/headless/EXR都走可验证分支。

## 9. 依赖顺序重构里程碑

### M0：冻结feature truth与当前快照

- 标记HDR10、LinearExtended、ACES、当前SSR/DOF/Bloom为experimental/degraded；
- 记录239-file fingerprint与42个dirty owner；
- 验收：docs/capability/Editor/stats不再将enum/pass存在解释为product-ready。

### M1：冻结scene-linear/working/display color contract

- 定义working primaries/white、pre-exposure、tone、grading、output device和capture space；
- 禁止fixed format/default enum隐式决定color semantics；
- 验收：每个graph resource都有color-space/encoding/precision metadata。

### M2：修复HDR LUT domain与默认tone路径

- 引入HDR shaper或analytic tone + grading LUT分层；
- 普通SDR target强制合法默认tone/output transform；
- 验收：0..10000 nit/range ramp、wide gamut与negative/over-range对CPU oracle。

### M3：Output device plan与surface/capture接线

- 实现sRGB/linear extended/PQ及gamut/luminance/metadata；
- 缺capability时结构化fallback；
- 验收：SDR/HDR10/scRGB/headless/EXR和UI reference white矩阵。

### M4：Exposure独立history与真实frame timing

- 真实delta time、camera cut/reset、success generation；
- 移除global history bool依赖；
- 验收：30/60/144 Hz同时间常数，normal motion不reset，cut精确reset。

### M5：Exposure metering/physical/local能力

- downsample histogram、mask/center/spot、curve、physical calibration、local exposure；
- 验收：bright object crossing、backlit/window、dark-to-light序列和EV/debug输出。

### M6：Volume/profile schema hard cut

- component id/version/override/resource ref、unknown round-trip、migration；
- camera与overlay逐自有settings解析；
- 验收：15 builtin与plugin component均可save/reload/cook/undo/hot reload。

### M7：Color grading/LUT production pipeline

- white balance、range grading、curves/mixer、multiple LUT blend、quality/cache；
- 验收：reference vectors、strip/3D插值、missing/stale generation与async bake。

### M8：Bloom scalable pipeline

- soft-knee/anti-firefly prefilter、mip down/up、scatter、lens dirt/convolution tier；
- 验收：radius/energy invariance、firefly、1080p/4K GPU/bandwidth和artifact。

### M9：Physical/scalable DOF

- camera lens/sensor CoC、tile/dilate、near/far、gather/scatter/recombine、transparent policy；
- 验收：foreground occlusion、focus pull、highlight bokeh、dynamic resolution和quality tiers。

### M10：Motion Blur contract与filter

- typed shutter、per-pixel velocity + tile envelope、depth/coverage、adaptive/jitter samples；
- 验收：static background/fast foreground、camera/object/skin/particle、thin edge、30/60/120fps。

### M11：Hierarchical rough SSR与denoiser

- tile classify/indirect、HiZ traversal、GGX ray tiers、spatial/temporal denoise、history validation；
- 验收：roughness矩阵、thin/offscreen/disocclusion、GPU ray budget和debug。

### M12：Indirect-specular integration与fallback

- material BRDF、SSR/planar/probe/IBL/ray provider priority和energy replacement；
- 验收：dielectric/metal、roughness、screen edge/miss无能量跳变或double count。

### M13：Light effects、custom injection与quality tooling

- blur family重构、grain/dither时域噪声、lens distortion/flare、custom injection；
- per-effect quality/GPU/VRAM stats与debug view；
- 验收：missing provider/resource与budget degradation均结构化可见。

### M14：Artifact、竞争性benchmark与hard cut

- 当前fingerprint导出HDR/SDR sequence、EXR/PNG、RDC、profile/manifest；
- 删除normalized HDR LUT、copy-only HDR transfer、fixed profile、shutter双单位与scene-mix SSR旧路；
- 同场景对照Unreal/HDRP/Godot/Bevy/Fyrox并完成独立review。

## 10. 验收矩阵

| 域 | 必测场景 | 正确性 gate | 性能/预算 gate | 证据 |
|---|---|---|---|---|
| color/tone | HDR ramp、wide gamut、negative/over-range、LUT blend | CPU/reference vector一致，无pre-tone clamp | LUT bake/cache ms、bytes | EXR + vector log + shader hash |
| output | SDR/PQ/scRGB、paper white、monitor fallback、UI | EOTF/gamut/luminance/alpha正确 | terminal GPU ms、bandwidth | probe chart + metadata + capture |
| exposure | fixed/manual/auto/mask/local、30/60/144Hz、cut | EV/time constant/metering一致 | histogram resolution/atomics/ms | EV sequence + histogram dump |
| Bloom | soft threshold、firefly、small/large radius、dirt/FFT | energy/radius稳定，无square blowout | mip bytes、GPU ms、tier曲线 | HDR sequence + mip atlas |
| DOF | focus pull、near occluder、far highlight、transparent | physical CoC、near/far occlusion正确 | tile occupancy、gather/scatter ms | CoC atlas + EXR sequence |
| Motion Blur | camera/object/skin/particle、fast thin edge | shutter单位、中心surface、不拖静态背景 | samples/tile、GPU ms | velocity/tile view + sequence |
| SSR | smooth/rough metal/dielectric、offscreen、disocclusion | hit/fallback/BRDF/history连续 | traced tiles/rays、denoise ms | ray/debug atlas + RDC |
| Volume | global/local/unsupported shape、overlay、plugin/resource | save/reload/cook/migration/unknown round-trip | eval candidates、CPU us | scene asset + transition trace |
| Cross-feature | exposure+bloom+DOF+motion+SSR+TAA+fog+UI | phase/color/history不冲突 | peak transient/history/total ms | full-chain EXR/PNG/RDC/profile |

所有产品gate至少包含feature-off exact baseline、registered-but-empty inert、active visible effect、invalid/missing resource degraded path、连续动态sequence、resize/quality/camera/provider切换、device loss与当前source fingerprint。单帧 changed-pixel threshold、shader parse或pass name存在不能独立证明算法正确。

“超过Unreal”必须限定硬件、driver、resolution、output、scene、quality target、warmup、metric和统计方法。只有同画质下GPU/VRAM/稳定性更优，或同预算下质量显著更高且第三方可复现，才能对具体场景/档位作出结论。

## 11. 现有测试与 artifact 判定

### 11.1 Product tests证明可达，不证明reference correctness

`render_product_post_non_neutral_tonemap_grading_changes_final_frame`、user LUT readback、Blur/DOF/Motion Blur/SSR/Fog/Volume与full-chain tests会检查node/executor顺序、资源backing、frame delta、RGB sum/chromatic pixel。它们能抓pass未执行、资源别名错误和完全无输出，不能抓HDR domain clamp、错误tone curve、DOF occlusion、motion bleed或SSR energy。

### 11.2 Full-chain artifact exporter被ignore且当前重导失败

唯一 exporter用 `#[ignore]` 写入PNG/JSON/RDC。目录保留1张旧PNG和8个旧RDC，但没有源码声明的 `current_source_20260801` PNG/JSON；对应日志停在session registration timeout。不能把accepted command或旧capture当当前源码验收。

### 11.3 旧PNG本身暴露非golden特征

320×240图像大面积分布洋红/青色颗粒，中心场景信号很弱。该图可以作为“某次全链产生过frame”的E1旁证，不能作为color/tone/grain/dither/Bloom/DOF/SSR视觉golden，也没有baseline、EXR、settings、GPU/driver和metric manifest。

### 11.4 旧12-test pass只绑定旧二进制

2026-07-05日志显示12 passed、0 failed、72.46s，执行文件位于旧cargo target目录。它没有本轮239-file fingerprint、commit/worktree diff、GPU identity、profile或artifact hash。本报告保留该事实，但不将其升级为当前E3动态证据。

### 11.5 本轮验证声明

本轮只做static review、图像人工查看与文档校验，未运行Cargo/WGPU/Editor/RenderDoc。25个dedicated test文件共9,469行、96个test、1个ignored exporter；运行状态未知。67个命中artifact共48,982,850 bytes，其中9个视觉/capture文件也不能覆盖本报告第10节矩阵。

## 12. 完成定义与退出条件

09H2只有同时满足以下条件才可从`pending`改为完成：

1. scene-linear HDR在tone/LUT前不被`[0,1]` clamp，默认SDR路径有合法tone/output transform；
2. working color space、pre-exposure、grading、tone、output gamut/EOTF/luminance和capture metadata有唯一plan；
3. sRGB、linear extended与HDR10能力由真实surface/capture支持决定，unsupported不会以enum伪装；
4. Exposure使用真实delta time和独立history/reset，metering/physical/local exposure通过动态序列；
5. 15个builtin及plugin post component可由versioned profile save/reload/cook/migrate，resource reference与unknown payload可闭环；
6. Base/Overlay camera分别解析自身base settings、volume与history，terminal/UI ownership在SDR/HDR均正确；
7. Bloom使用可扩展pyramid/soft-knee/anti-firefly与quality budget；
8. DOF使用物理CoC、near/far分层和scalable tile/gather/scatter/recombine，透明边界明确；
9. Motion Blur单位唯一，per-pixel surface velocity与tile envelope职责分离，动态场景无邻域拖影；
10. SSR具备hierarchical traversal、tile/ray quality、roughness-aware sampling/denoise与可靠history rejection；
11. SSR/planar/probe/IBL/ray fallback在统一indirect-specular compositor按material BRDF能量合成；
12. Grain/dither/fog/chromatic/blur/lens/custom injection的算法身份、phase与quality得到明确实现或保持unsupported；
13. per-effect debug、GPU timing、resource/history bytes、degraded reason与Editor/runtime stats同真值；
14. 第10节矩阵由当前source fingerprint自动执行，artifact含EXR/PNG/RDC/profile/settings/GPU/driver/hash；
15. 旧normalized HDR LUT、copy-only HDR output、fixed volume profile、shutter双单位、neighbor-only motion direction和scene-mix SSR完成hard cut；
16. 与参考引擎同场景的质量/性能差异有可复现记录，未超过项继续公开为gap；
17. 独立code、visual、color-science和performance review均无Critical/Important遗留。

在这些退出条件之前，capability、Editor、docs、plugin manifest、release note不得使用“完整HDR”“ACES compliant”“cinematic DOF”“physically based SSR”“production-ready postprocess”或“超过Unreal”描述本组能力。
