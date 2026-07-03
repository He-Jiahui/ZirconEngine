---
related_code:
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/graph_resource_names.rs
  - zircon_runtime/src/core/framework/render/post_process/stack/tests.rs
  - zircon_runtime/src/core/framework/render/post_process/stack/tests/effect_stack.rs
  - zircon_runtime/src/core/framework/render/post_process/stack/tests/exposure.rs
  - zircon_runtime/src/core/framework/render/post_process/stack/tests/screen_space_reflection.rs
  - zircon_runtime/src/core/framework/render/post_process/stack/tests/temporal_history.rs
  - zircon_runtime/src/core/framework/render/post_process/stack/tests/terminal_chain.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/resolved_stack.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_profile.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component/params.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component/tests.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_registry.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_evaluator.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/mod.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process/motion_blur.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_volume.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain/fixture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/frame_effects.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/resource_routing.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection/tests.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/mod.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_post_process_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_post_process_volume_component.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_builtin_postprocess_executors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_post_process_screen_space_reflection_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_post_process_motion_blur_tests.rs
  - zircon_plugins/rendering/plugin.toml
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessEyeAdaptation.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessTonemap.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessBloomSetup.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeComponent.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeStack.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/PostProcess/UberPostProcessPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/ColorGradingLutPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/PostProcess/BloomPostProcessPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/PostProcess/ScalingSetupPostProcessPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/RTHandleSystem.cs
  - dev/bevy/crates/bevy_post_process/src/auto_exposure/auto_exposure.wgsl
  - dev/bevy/crates/bevy_post_process/src/auto_exposure/settings.rs
  - dev/bevy/crates/bevy_post_process/src/bloom/mod.rs
  - dev/bevy/crates/bevy_post_process/src/bloom/bloom.wgsl
  - dev/bevy/crates/bevy_core_pipeline/src/tonemapping/mod.rs
  - dev/bevy/crates/bevy_post_process/src/effect_stack/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/upscaling/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/hdr/mod.rs
plan_sources:
  - .codex/plans/Rendering 插件选项补齐计划.md
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
---

# 计划 07:后处理链、色彩管线与 Volume 容器框架

## 目标

1. 后处理链顺序与色彩空间权威定稿:linear 工作空间、HDR 中间格式、tonemap、输出转换一锤定音。
2. 引入 Unity 风格 Volume 容器框架:全局容器 + 局部容器(盒/球范围、混合距离、权重、优先级),按相机位置插值出每帧生效的后处理参数集 —— 覆盖"局部容器组件化、全局容器支持"需求。
3. 效果目录补齐并组件化:LUT、bloom、blur、color grading、DoF、SSR、fog(屏幕空间)、dither、vignette、grain、chromatic aberration;每个效果是独立 `VolumeComponent` 等价物 + 对应 pass/uber 槽位。
4. histogram 自动曝光、FXAA/SMAA 终端 AA、动态分辨率缩放(scaling setup + upscale)纳入链尾。

## 现状与差距

- effect stack(`stack.rs`/`effect.rs`/`pass_graph.rs`)与 bloom/DoF/SSR/motion blur/color grading 执行器已存在;PP-M2 已补 Volume DTO/evaluator,PP-M3-S1b 已补 WGPU histogram/resolve 自动曝光。剩余差距集中在:链路仍保留若干兼容构造器;LUT bake/uber 瘦身尚未硬切;tonemap 曲线与 HDR 输出档仍待 PP-M3/PP-M4 完成;dither/vignette/grain/CA 专项产物测试仍待补齐。
- 色彩空间管理不明:哪些 pass 在 linear、LUT 是否含输出转换、UI 合成在前在后,均无文档与断言。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/Graphics/.../core/Runtime/Volume/VolumeManager.cs` + `VolumeStack.cs` + `VolumeComponent.cs` | Volume 框架全貌:全局/局部容器注册、按相机位置计算权重(blend distance)、参数插值进 stack 的机制 —— 本计划容器模型的直接样板 |
| `dev/Graphics/.../universal/Runtime/Passes/PostProcess/UberPostProcessPass.cs` | uber pass 合并策略:vignette/grain/dither/CA/LUT 在单 pass 合并,避免逐效果全屏 blit |
| `dev/Graphics/.../universal/Runtime/Passes/ColorGradingLutPass.cs` | LUT 预烘焙 pass:grading + tonemap 烘进 3D LUT,运行时一次采样 |
| `dev/UnrealEngine/.../PostProcess/PostProcessing.cpp` | UE 全链顺序权威参考(translucency 后 → DoF → motion blur → bloom → exposure → tonemap → 输出) |
| `dev/UnrealEngine/.../PostProcess/PostProcessEyeAdaptation.cpp` | histogram compute 自动曝光与速度参数 |
| `dev/Graphics/.../universal/Runtime/Passes/PostProcess/ScalingSetupPostProcessPass.cs` + `core/Runtime/Textures/RTHandleSystem.cs` | 动态分辨率:RTHandle 比例缩放 + 链尾 upscale 的接法 |

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_post_process/src/auto_exposure/auto_exposure.wgsl` | `exposure_histogram/resolve` compute(PP-M3 重点) | 64 bin `array<atomic<u32>, 64>` + `var<workgroup>` 共享 bins 归并的 WGSL 直方图;百分位截断求平均与收敛写回,与本计划布局一一对应 |
| `dev/bevy/crates/bevy_post_process/src/auto_exposure/settings.rs` | `RenderExposureSettings` | `speed_brighten`(默认 3.0)/`speed_darken`(默认 1.0)/percentile 范围字段,与 UE 参数同语义的 Rust 表达 |
| `dev/bevy/crates/bevy_post_process/src/bloom/mod.rs` | bloom downsample/upsample 链多 pass 组织 | mip 链 RT 分配、downsampling/upsampling 双 pipeline 与逐 mip pass 录制 |
| `dev/bevy/crates/bevy_post_process/src/bloom/bloom.wgsl` | 能量守恒 bloom 核 | `sample_input_13_tap`(13-tap downsample)与 `karis_average`(首段抗 firefly) |
| `dev/bevy/crates/bevy_core_pipeline/src/tonemapping/mod.rs` | tonemap 曲线与 LUT 绑定 | `TonemappingLuts`(3D LUT 纹理)、tonemap 模式枚举与 shader def 注入;配 `tonemapping.wgsl` 曲线实现 |
| `dev/bevy/crates/bevy_post_process/src/effect_stack/mod.rs` | uber pass 合并(CA/vignette 单 pass) | `PostProcessingPipeline`/`PostProcessingPipelineKey` 与 `post_process.wgsl` 的多效果单 entry 合并形态 |
| `dev/bevy/crates/bevy_core_pipeline/src/upscaling/mod.rs` | `post.upscale` 链尾上采样 | `ViewUpscalingPipeline` 基于 blit 的链尾尺寸跃迁(bilinear 起步形态) |
| `dev/Fyrox/fyrox-impl/src/renderer/hdr/mod.rs` | 曝光适应的另一 Rust 形态 | `LumBuffer`(64x64 luminance 链)+ `AdaptationChain` 平均亮度法曝光(对照项:本计划取 histogram 法,勿照搬) |

`Volume` 容器框架(全局/盒/球容器、blend distance 权重插值)无 Rust 同类参照(bevy/Fyrox 均无 Volume 概念),实现时以 Unity `VolumeManager.cs`/`VolumeStack.cs` 为唯一样板,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:Volume 框架契约进 `core/framework/render/post_process/`(纯数据,无 wgpu);求值与 pass 在 `scene_renderer/post_process/`;可选效果继续经 rendering 插件 feature 注册。

核心设计:

- `VolumeComponentDescriptor`:每个效果的参数 schema(字段、默认值、插值方式),由内建效果与插件效果共同注册;ECS 侧 volume 实体(全局/盒/球 + blend distance + priority + layer mask)经 extract 输出 `PostProcessVolumeExtract`。
- `VolumeEvaluator`:每帧按相机位置求权重 → 插值出 `ResolvedPostProcessStack`(替代现有静态 stack 输入);per-camera 求值(多相机各自独立,接计划 09)。
- 链顺序定稿(backbone 固定,效果可关):
  `TAA(计划06) → DoF → motion blur → bloom(能量守恒 downsample 链) → exposure(histogram compute) → SSR/fog 合成类 → blur(高斯/局部模糊复用) → LUT 烘焙(grading+tonemap) → uber(LUT 采样+vignette+grain+dither+CA) → FXAA/SMAA → upscale(动态分辨率) → 输出转换(sRGB/HDR10)`
- 色彩空间:全链 linear,中间格式 R11G11B10F(质量档可升 RGBA16F);LUT 32^3 起步;输出 pass 统一做 transfer function;断言性测试覆盖"中间 pass 不做 gamma"。
- 动态分辨率:render scale 进相机契约,scene 系 RT 按 scale 分配(计划 01 池按缩放尺寸键控),链尾 upscale(bilinear 起步,FSR 类留插件位)。
- uber 合并:轻量像素效果合入单 pass;需要中间 RT 或链路复用的效果以独立 pass 表达(DoF/motion blur/bloom/SSR-fog composite/blur/LUT/upscale/output-transfer)。

## 里程碑

### PP-M1 链顺序与色彩空间定稿

实施切片:
1. backbone 顺序固化进 pipeline asset(forward/deferred 模板同步);现有效果归位。
2. 中间格式常量收敛 + 输出转换 pass;"linear 全链"测试断言。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime post_process --locked` + `render_product` 回归
- 验收证据:链顺序 graph dump 与本文档一致;灰阶测试图经全链后线性度断言通过。

### PP-M2 Volume 容器框架

实施切片:
1. `VolumeComponentDescriptor` 注册表与 `PostProcessVolumeExtract`;全局/盒/球容器与权重求值。
2. `VolumeEvaluator` 替换静态 stack 输入;现有效果参数迁移为 volume 组件 schema。
3. 编辑器侧容器组件面板对接(只做 runtime 契约,编辑器 UI 另由 editor 计划承接)。

测试阶段:
- `cargo test -p zircon_runtime volume --locked`(权重求值单测:相机在局部容器边界处插值正确、priority 覆盖)
- 验收证据:相机移入局部容器时 bloom 强度平滑过渡(逐帧参数 readback 断言)。

### PP-M3 曝光、LUT 与 uber 效果补齐

实施切片:
1. histogram compute 自动曝光(min/max/speed 参数)+ 手动曝光覆盖。
2. ColorGradingLut pass(grading+tonemap 烘焙,ACES/neutral 两曲线);uber pass 合并 LUT/vignette/grain/dither/CA。
3. blur(高斯,供 UI/局部模糊复用)效果组件。

测试阶段:
- `cargo test -p zircon_runtime post_process --locked`(曝光收敛、LUT 烘焙 readback、各效果开关产物差异)
- 验收证据:暗→亮场景曝光按 speed 收敛曲线变化;效果全开/全关抓帧对比记录。

### PP-M4 终端 AA 与动态分辨率

实施切片:
1. FXAA/SMAA 终端 pass;与 TAA 的互斥/共存策略定稿。
2. render scale + 链尾 upscale;RT 池按缩放尺寸协同(计划 01)。

测试阶段:
- `cargo test -p zircon_runtime post_process --locked` 与 `render_graph` 回归
- 验收证据:scale 0.5→1.0 切换无资源泄漏(池统计);FXAA/SMAA 开关产物对比。

## 工程落地细化

本章节为计划 07 的实施权威(见 index.md §8.7)。bind group 槽位、GPU 数据布局基线、WGSL include 前缀、测试命名直接引用 index.md §8,不再重述。所有契约类型落在 `zircon_runtime::core::framework::render`(facade 不变,无 wgpu);所有 pass 经 RenderGraph 声明 + executor 执行,无旁路提交;只消费 `RenderFrameExtract`。

### 模块与文件落点

现状基线:契约层 `core/framework/render/post_process/` 已有 `effect.rs`(`PostProcessEffectKind`/`PostProcessEffectSettings`)、`graph_resource_names.rs`(`PostProcessGraphResourceNames`)、`stack.rs`(`PostProcessStackDescriptor` 与 graph descriptor 构造/验证)、`stack/tests/*`(stack graph contract 行为测试)、`pass_graph.rs`/`pass_node.rs`/`validation.rs`、`resolved_stack.rs`(`RenderResolvedPostProcessSettings`)、`volume_profile.rs`(`RenderPostProcessVolumeProfile`)、`volume_component.rs`/`volume_component/{params,tests}.rs`/`volume_registry.rs`(组件 schema、参数值/插值 owner、行为测试与内建注册表)、`volume_extract.rs`(`PostProcessVolumeExtract`/global/box/sphere 形状快照)、`volume_evaluator.rs`(`VolumeEvaluator` 每相机求值)、`effect_stack_settings/`(`RenderPostProcessEffectStackSettings` 含 tonemap/color_lookup/blur/motion_blur/DoF/SSR/vignette/grain/dither/CA/fog 全套字段)。实现层 `graphics/scene/scene_renderer/post_process/resources/` 已有 `execute_bloom`、`execute_depth_of_field`、`execute_motion_blur`、`execute_scene_composite`、`execute_blur`、`execute_fxaa`、`execute_smaa` 与 `execute_post_process`/`post.uber`;`post_process.wgsl` 仍承载共享采样函数与若干轻量 uber 效果,但 DoF/motion blur、SSR/fog composite、通用 blur、terminal FXAA/SMAA 已拆到独立 WGPU passes。

**新增文件**:

| 文件(相对 `zircon_runtime/src/`) | 内容 | 层 |
|---|---|---|
| `core/framework/render/post_process/volume_component.rs` | `VolumeComponentDescriptor`、内建 component descriptor 表与 read/apply 写回映射 | 契约 |
| `core/framework/render/post_process/volume_component/params.rs` | `VolumeParamSchema`/`VolumeParamValue`/`VolumeParamInterpFn` 与参数插值/默认值工厂 | 契约 |
| `core/framework/render/post_process/volume_component/tests.rs` | volume component descriptor、默认值、插值与错误路径行为测试 owner | 契约测试 |
| `core/framework/render/post_process/volume_registry.rs` | `VolumeComponentRegistry`(内建+插件效果注册,id 去重) | 契约 |
| `core/framework/render/post_process/graph_resource_names.rs` | `PostProcessGraphResourceNames` 资源名表 owner,与 `PostProcessStackDescriptor` 构造解耦 | 契约 |
| `core/framework/render/post_process/resolved_stack.rs` | `RenderResolvedPostProcessSettings` 强类型 resolved stack | 契约 |
| `core/framework/render/post_process/volume_profile.rs` | `RenderPostProcessVolumeProfile` 运行时 volume profile 旧数据入口 | 契约 |
| `core/framework/render/post_process/volume_extract.rs` | `PostProcessVolumeExtract`/`VolumeShapeExtract`/`VolumeComponentOverride` | 契约 |
| `core/framework/render/post_process/volume_evaluator.rs` | `VolumeEvaluator`/`ResolvedPostProcessStack`(每相机求值) | 契约 |
| `core/framework/render/post_process/exposure_settings.rs` | `RenderExposureSettings`/`RenderExposureMode` | 契约 |
| `core/framework/render/post_process/chain.rs` | `PostProcessChainSlot` 枚举 + backbone 定稿表构建(`PostProcessStackDescriptor::from_resolved`) | 契约 |
| `core/framework/render/post_process/color_space.rs` | `RenderOutputTransfer`、中间格式常量 `INTERMEDIATE_HDR_FORMAT_*`、LUT 尺寸常量 | 契约 |
| `core/framework/render/post_process/dynamic_resolution.rs` | `DynamicResolutionSettings`/`UpscaleFilter` | 契约 |
| `graphics/scene/scene_renderer/post_process/params/exposure_params.rs` | `ExposureParams` GPU 上传布局与默认 exposure buffer words | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/execute_exposure/mod.rs` | histogram/resolve compute executor(`post.exposure.histogram`/`post.exposure.resolve`) | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/{exposure_histogram,exposure_resolve}.rs` | exposure 两个 compute layout | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/{exposure_histogram_pipeline,exposure_resolve_pipeline}.rs` | exposure 两个 compute pipeline | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/new/create_buffer_bundle/{exposure_params_buffer,default_exposure_buffer,default_exposure_histogram_buffer}.rs` | exposure 参数与 fallback buffer | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/execute_color_lut_bake/mod.rs` | grading+tonemap 烘 3D LUT(compute,`post.color-lut-bake`) | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/execute_uber/mod.rs` | uber 单 pass(`post.uber`) | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/execute_depth_of_field/mod.rs` | DoF gather/composite(`post.depth-of-field`,prepare 已有) | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/execute_motion_blur/mod.rs` | motion blur 独立 pass(`post.motion-blur`,拆自 `post.stack`) | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/execute_blur/mod.rs` | 通用 blur 独立 pass(`post.blur`,写 `postprocess.blurred`) | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/execute_smaa/mod.rs` | 内建 SMAA terminal AA pass(`post.smaa`,读 `postprocess.terminal-aa-input`,写 `final-color`) | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/execute_upscale/mod.rs` | 链尾 upscale(`post.upscale`,bilinear;FSR1 留插件 executor 位) | 实现 |
| `graphics/scene/scene_renderer/post_process/resources/execute_output_transfer/mod.rs` | 输出转换(`post.output-transfer`,sRGB/HDR10 PQ) | 实现 |
| `graphics/scene/scene_renderer/post_process/shaders/{exposure_histogram,exposure_resolve,color_lut_bake,uber,depth_of_field,motion_blur,blur,fxaa,smaa,upscale,output_transfer}.wgsl` | 对应 WGSL;公共函数下沉 `zr_color.wgsl` include(§8.3) | 实现 |
| `graphics/feature/builtin_render_feature_descriptor/feature_descriptors/{post_process,compute_workload}.rs` | exposure 两 pass 的 feature descriptor 与 compute workload | 实现 |

**修改文件**:

| 文件 | 改动 |
|---|---|
| `core/framework/render/post_process/effect.rs` | `PostProcessEffectKind` 增 `Taa`/`DepthOfField`/`MotionBlur`/`ExposureHistogram`/`ExposureResolve`/`SceneComposite`/`ColorLutBake`/`Uber`/`Upscale`/`OutputTransfer`;删 `HistoryResolve`/`EffectStack`/`ColorGrading`/`FinalComposite`(硬切换,迁移同变更内完成) |
| `core/framework/render/post_process/stack.rs` | `from_extract_settings*` 四个构造器收敛为单一 `from_resolved(...)`;`PostProcessGraphResourceNames` 迁入 `graph_resource_names.rs`;inline graph contract tests 迁入 `stack/tests/*`;父 owner 只保留 `PostProcessStackDescriptor` 构造、validated graph 与 history-resource stripping |
| `core/framework/render/post_process/volume.rs` | 删除整文件:`RenderPostProcessVolume`/`RenderPostProcessVolumeStack`/`local_blend` 由 `volume_extract.rs`+`volume_evaluator.rs` 接管;`RenderResolvedPostProcessSettings` 迁入 `resolved_stack.rs`, `ResolvedPostProcessStack` 作为 evaluator 别名保留 |
| `core/framework/render/post_process/mod.rs` | re-export 表按新 owner 路径更新(保持 thin) |
| `core/framework/render/frame_extract.rs` | `PostProcessExtract` 增 `volumes: Vec<PostProcessVolumeExtract>` 字段与 `resolved_settings_for_camera(...)`;删旧 `volume_stack` 与 `resolved_settings_for_layers(...)` |
| `core/framework/render/camera.rs` | `exposure_ev100` 保留为 Manual 模式来源;`render_scale`/`volume_mask` 落在计划 09 `CameraRenderDescriptor`,本计划只消费 |
| `graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs` | pass 表按 backbone 重排;`post.stack` 改为 `post.scene-composite`(SSR 合成+屏幕空间雾) |
| `feature_descriptors/{bloom.rs,anti_alias.rs}` | bloom 改能量守恒 downsample/upsample 链多 pass;FXAA/SMAA 输入改 `postprocess.terminal-aa-input` |
| `feature_descriptors/{color_grading.rs,history_resolve.rs}` | 删除(grading 并入 LUT bake;history 路径归计划 06 TP-M4) |
| `resources/{execute_post_process/,execute_bloom/}` + `shaders/{post_process.wgsl,bloom.wgsl}` | `post.stack` 瘦身为 scene-composite;bloom 链改造 |
| `graphics/pipeline/declarations/compiled_render_pipeline.rs` + forward/deferred pipeline asset 模板 | backbone 顺序固化进模板;经计划 01 `CompiledGraphCache` 键控 |
| `graphics/scene/resources/post_process_lut_texture/` | 用户 LUT 资产(2D strip)继续走 `PostProcessLutTextureResource` 流送;烘焙时叠加进内部 3D LUT |

### 核心类型与接口

全部为契约层(`core/framework/render/post_process/`),无 wgpu 类型;`Real`/`Vec3`/`Quat` 来自 `core::math`。

```rust
// volume_component/params.rs —— 参数 schema 与逐参数插值函数指针(避免反射式分支,见风险节)
pub type VolumeParamInterpFn =
    fn(from: VolumeParamValue, to: VolumeParamValue, weight: Real) -> VolumeParamValue;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VolumeParamValue {
    Float(Real),
    Vec3(Vec3),
    Bool(bool),
    Uint(u32),
    Enum(u32),          // 离散参数(如 RenderTonemapOperator),weight>=0.5 取 to(blend_discrete 语义)
}

pub struct VolumeParamSchema {
    pub name: &'static str,          // 如 "intensity"
    pub default: VolumeParamValue,
    pub interp: VolumeParamInterpFn, // interp_lerp / interp_discrete / interp_bool
}

pub struct VolumeComponentDescriptor {
    pub component_id: &'static str,  // 如 "post.bloom"、"post.vignette";插件用 "<plugin>.<effect>"
    pub params: &'static [VolumeParamSchema],
    /// 把插值后的参数写回强类型 ResolvedPostProcessStack(内建效果各自实现;插件写入 sideband)
    pub apply: fn(&mut ResolvedPostProcessStack, &[VolumeParamValue]),
}

// volume_registry.rs
pub struct VolumeComponentRegistry { /* Vec<VolumeComponentDescriptor> + id 索引 */ }
impl VolumeComponentRegistry {
    pub fn register(&mut self, descriptor: VolumeComponentDescriptor)
        -> Result<(), VolumeRegistryError>;            // 重复 component_id 报错
    pub fn default_stack(&self) -> ResolvedPostProcessStack; // 全部 schema 默认值
}
```

```rust
// volume_extract.rs —— ECS 容器组件经 extract 输出的快照;形状进 extract,权重由求值器算
#[derive(Clone, Debug, PartialEq)]
pub enum VolumeShapeExtract {
    Global,
    Box { center: Vec3, half_extents: Vec3, rotation: Quat, blend_distance: Real },
    Sphere { center: Vec3, radius: Real, blend_distance: Real },
}

#[derive(Clone, Debug, PartialEq)]
pub struct VolumeComponentOverride {
    pub component_id: String,
    /// 与 descriptor.params 同序;None = 该参数未覆写(Unity overrideState 语义)
    pub values: Vec<Option<VolumeParamValue>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PostProcessVolumeExtract {
    pub active: bool,
    pub shape: VolumeShapeExtract,
    pub priority: Real,
    pub weight: Real,                      // [0,1],求值时 clamp
    pub volume_mask: RenderLayerSet,       // 计划 09 定稿的掩码类型(RenderLayer 保持 layer 索引语义),与相机 volume_mask 相交过滤
    pub overrides: Vec<VolumeComponentOverride>,
}
```

```rust
// volume_evaluator.rs —— 每相机求值;多相机各自独立(接计划 09 CameraRenderDescriptor)
pub struct ResolvedPostProcessStack {       // 替代 RenderResolvedPostProcessSettings(硬切换更名)
    pub bloom: RenderBloomSettings,
    pub color_grading: RenderColorGradingSettings,
    pub exposure: RenderExposureSettings,
    pub effect_stack: RenderPostProcessEffectStackSettings, // 既有字段集原样复用
}

pub struct VolumeEvaluator { /* Arc<VolumeComponentRegistry> */ }
impl VolumeEvaluator {
    /// 算法(对照 VolumeManager.Update):
    /// 1. ReplaceData 等价:从 registry.default_stack() 出发;
    /// 2. 收集:active && volume_mask.intersects(camera_volume_mask) && weight > 0;
    /// 3. influence:Global = clamp01(weight);Box/Sphere 求 camera_position 到形状最近点
    ///    距离平方 d2(Box 先逆旋转到局部空间再 clamp),blend2 = blend_distance^2;
    ///    d2 > blend2 → 跳过;blend2 > 0 → interp = 1 - d2/blend2,否则 1;
    ///    influence = interp * clamp01(weight);
    /// 4. 排序:priority 升序,同 priority 按 extract 序稳定;
    /// 5. 叠加:逐 volume 逐 override 逐参数 value = schema.interp(current, override, influence),
    ///    最后 descriptor.apply 写回强类型 stack。
    pub fn evaluate(
        &self,
        camera_position: Vec3,
        camera_volume_mask: RenderLayerSet,
        volumes: &[PostProcessVolumeExtract],
    ) -> ResolvedPostProcessStack;
}
```

```rust
// exposure_settings.rs
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderExposureMode { Manual, Histogram }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderExposureSettings {
    pub mode: RenderExposureMode,    // Manual 时取相机 exposure_ev100,链上仍走同一 ExposureBuffer
    pub manual_ev100: Real,
    pub compensation_ev: Real,
    pub min_ev100: Real,             // 默认 -8
    pub max_ev100: Real,             // 默认 8
    pub low_percent: Real,           // 直方图截断下百分位,默认 0.10
    pub high_percent: Real,          // 默认 0.90
    pub speed_brighten: Real,        // EV/s,默认 3.0
    pub speed_darken: Real,          // 默认 1.0
}

// chain.rs —— backbone 槽位(顺序即定稿顺序,构建时只做开关不重排)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PostProcessChainSlot {
    TaaResolve,        // 计划 06 TaaResolveExecutor 占位,本计划不实现
    DepthOfField, MotionBlur, Bloom,
    ExposureHistogram, ExposureResolve,
    SceneComposite,    // SSR 合成 + 屏幕空间雾(高度雾归计划 11,边界见下)
    ColorLutBake, Uber, TerminalAntiAlias, Upscale, OutputTransfer,
}

// color_space.rs / dynamic_resolution.rs
pub enum RenderOutputTransfer { SrgbNonlinear, LinearExtended, Hdr10Pq } // 后两者仅定接口
pub enum UpscaleFilter { Bilinear, Fsr1 }   // Fsr1 经 rendering 插件 RenderFeature 提供 executor
pub struct DynamicResolutionSettings { pub render_scale: Real /* 0.25..=1.0 */, pub upscale_filter: UpscaleFilter }
```

**后处理链定稿表**(backbone 固定;"并入 uber"= 无独立 pass;Volume 组件列为 `component_id`):

| 序 | 效果 | 输入 | 输出 | 槽位/executor | 并入 uber | Volume 组件 |
|---|---|---|---|---|---|---|
| 1 | TAA | scene-color、velocity、depth、history | history.current(计划 06 语义) | `TaaResolve`(计划 06) | 否 | —(AA 模式非 volume) |
| 2 | DoF | history.current、scene-depth、DoF CoC/bokeh(prepare 已有) | `postprocess.dof` | `post.depth-of-field` | 否(需半分辨率中间 RT) | `post.depth-of-field` |
| 3 | Motion blur | 上游 color、neighbor-max、scene-depth | `postprocess.motion-blurred` | `post.motion-blur` | 否 | `post.motion-blur` |
| 4 | Bloom | 上游 color | `bloom-texture`(mip 链) | `post.bloom-extract` + downsample/upsample 链 | 否 | `post.bloom` |
| 5 | Exposure | 上游 color | `EXPOSURE_HISTOGRAM` → `EXPOSURE_CURRENT`(前帧为 `EXPOSURE_PREVIOUS`) | `post.exposure.histogram`/`post.exposure.resolve`(compute) | 否 | `post.exposure` |
| 6 | SSR 合成+屏幕空间雾 | SSR 六件套产物、scene-depth、fog 参数 | `postprocess.scene-composited` | `post.scene-composite`(改造自 `post.stack`) | 否 | `post.screen-space-reflection`、`post.screen-space-fog` |
| 7 | LUT bake | grading+tonemap 参数、用户 LUT(可选) | `COLOR_LUT`(3D 32^3 rgba16float) | `post.color-lut-bake`(compute) | 否 | `post.color-grading`、`post.tonemap` |
| 8 | LUT 采样+vignette+grain+dither+CA | scene-composited、bloom、COLOR_LUT、`EXPOSURE_CURRENT` | `TONEMAPPED` | `post.uber` | 是(本体) | `post.vignette`/`post.grain`/`post.dither`/`post.chromatic-aberration`/`post.color-lookup` |
| 9 | FXAA/SMAA | `postprocess.terminal-aa-input` | `final-color` | `FXAA_EXECUTOR_ID`/`SMAA_EXECUTOR_ID` | 否 | — |
| 10 | Upscale | anti-aliased(render_scale 尺寸) | `UPSCALED`(全尺寸) | `post.upscale` | 否 | — |
| 11 | 输出转换 | UPSCALED | `viewport-output` | `post.output-transfer` | 否 | — |

与计划 11 的边界:`FogSettings`(解析雾/高度雾,场景光照域)归 11,在 forward/deferred 光照合成中生效;本计划的屏幕空间雾只做 depth 驱动的后处理雾(现 `RenderFogSettings` 字段),两者参数类型不同名不互引,Volume 均可覆写。

### GPU 数据布局与 WGSL 约定

bind group 全链遵守 §8.1:group0 frame/view(相机矩阵、时间、jitter),group1 pass 级输入,group2/3 后处理不用。Exposure 的 histogram/resolve compute pass 使用私有 group0 layout;最终 uber/post-process pass 在 group1 binding 28 只读绑定 resolved exposure storage buffer。数据布局遵守 §8.2(storage 一律 std430、显式 padding 注释)。

**LUT**:内部 LUT 为 3D 纹理 `32x32x32`,格式 `rgba16float`(质量档高可升 `64^3`;常量 `COLOR_LUT_SIZE_DEFAULT=32`/`COLOR_LUT_SIZE_HIGH=64` 进 `color_space.rs`)。经 compute 烘焙(storage texture 写,workgroup `4x4x4`,dispatch `(size/4)^3`),避开 wgpu 对 3D color attachment 的兼容性差异;tonemap(ACES/neutral 双曲线)与 grading 全部烘入,运行时 uber 单次 `textureSampleLevel` 取回。uber 端 `lut_params = (size, 1/size, (size-1)/size, intensity)`,采样坐标 `uvw = saturate(color_log) * lut_params.z + 0.5 * lut_params.y`(URP `lutParameters` 同构)。用户 LUT(既有 `PostProcessLutTextureResource`,2D strip)在 bake pass 内按 `color_lookup.intensity` 叠加,不进 uber。

**exposure histogram compute 布局**(对照 UE PostProcessHistogram.cpp / PostProcessEyeAdaptation.cpp):

```wgsl
// exposure_histogram.wgsl —— private group0
@group(0) @binding(0) var scene_color_tex: texture_2d<f32>;
@group(0) @binding(1) var<uniform> params: ExposureParams;
@group(0) @binding(2) var<storage, read_write> exposure_histogram: array<atomic<u32>, 64>; // 256 B
// workgroup 16x16;workgroup 私有 var<workgroup> bins 累加后 atomicAdd 归并
// bin = 0 保留暗值,bin 1..63 对应 clamp(log2(luminance), min_ev100, max_ev100)

// exposure_resolve.wgsl —— private group0,单 workgroup
@group(0) @binding(0) var<uniform> params: ExposureParams;
@group(0) @binding(1) var<storage, read> exposure_histogram: array<u32, 64>;
@group(0) @binding(2) var<storage, read> previous_exposure: array<vec4<f32>, 1>;
@group(0) @binding(3) var<storage, read_write> current_exposure: array<vec4<f32>, 1>;
// current_exposure[0] = (multiplier, resolved_ev100, average_ev100, valid_flag)
// 前缀和 → 按 low_percent/high_percent 截断求加权平均 → 与上帧 ExposureBuffer 按
// 1 - exp2(-dt * speed) 收敛 → clamp 到 [min_ev100, max_ev100] → 写本帧 buffer。
// Manual 模式:跳过 histogram pass,resolve 直接用相机 exposure_ev100 写 buffer,链上消费方无分支。
```

两个 compute pass 用计划 16 的 `ComputePassDescriptor` 形态声明(dispatch = `PerPixel(half_res, 16x16)` 与 `Fixed(1,1,1)`),在 graph 中与 render pass 同等参与 culling 与生命周期。

**uber pass binding 编号表**(group1;group0 由帧契约统一提供,含 ExposureBuffer):

| binding | 资源 | WGSL 类型 |
|---|---|---|
| 0 | scene-composited 输入 | `texture_2d<f32>` |
| 1 | linear clamp sampler | `sampler` |
| 2 | bloom mip 链 | `texture_2d<f32>` |
| 3 | 内部 3D LUT | `texture_3d<f32>` |
| 4 | LUT sampler(linear clamp) | `sampler` |
| 5 | `UberParams` | `var<uniform>` |
| 6 | grain/dither 噪声纹理(内建小纹理) | `texture_2d<f32>` |

```wgsl
struct UberParams {                  // uniform,6 x vec4 = 96 B
    lut_params: vec4<f32>,           // x=size, y=1/size, z=(size-1)/size, w=intensity
    vignette_params: vec4<f32>,      // x=intensity, y=smoothness, z=roundness, w=aspect
    grain_params: vec4<f32>,         // x=intensity, y=response, zw=帧随机 uv 偏移
    dither_params: vec4<f32>,        // x=intensity, y=scale, zw=保留(注释占位)
    chromatic_params: vec4<f32>,     // x=intensity, y=sample_spread, zw=保留
    output_params: vec4<f32>,        // x=RenderOutputTransfer 编码, y=HDR 白点 nits, zw=保留
}
// 像素流程(单 entry point,无分支版本由 specialization 常量裁剪):
// CA 偏移采样(HDR)→ +bloom → ×exposure_scale(group0)→ LUT(含 tonemap+grading)
// → vignette → grain → dither → 输出 TONEMAPPED
```

**HDR 与色彩空间**:全链 linear;scene color 与 DoF/motion blur/bloom 中间 RT 统一 `rg11b10ufloat`(`INTERMEDIATE_HDR_FORMAT_DEFAULT`),质量档高或精度问题时升 `rgba16float`(`INTERMEDIATE_HDR_FORMAT_HQ`);velocity 维持计划 06 的 RG16F。uber 输出 `TONEMAPPED`:SDR 档 `rgba8unorm`(uber 末尾做 sRGB 编码,FXAA 在感知空间取 luma 正确);HDR 输出档 `rgba16float` 线性。"中间 pass 不做 gamma" 断言覆盖 slot 1–7;`post.output-transfer` 是唯一 transfer function 归属点(SDR 直拷已编码内容,HDR10 做 PQ + Rec.2020,仅定接口)。

**动态分辨率**:`render_scale` 来自计划 09 `CameraRenderDescriptor`;slot 1–9 的瞬态 RT 一律按 `ceil(viewport * render_scale)` 尺寸经计划 01 `TransientResourcePool` 键控分配(`RgTextureHandle` 描述符带缩放后尺寸,池天然按 (size, format, usage) 复用);`post.upscale` 是唯一尺寸跃迁点,之后 `UPSCALED`/`viewport-output` 为全尺寸。scale 变化触发 `CompiledGraphCache` 重编译(viewport 尺寸在缓存键内,计划 01 既有约定)。

**AA 模式互斥矩阵**(`AntiAliasMode` 单选定稿,既有枚举扩展):

| 启用 | MSAA | FXAA | SMAA | TAA |
|---|---|---|---|---|
| MSAA | — | 允许(forward,FXAA 在 resolve 后) | 允许 | 互斥(TAA 选中强制 1x) |
| FXAA | 允许 | — | 互斥(同槽 TerminalAntiAlias) | 允许(TAA 后低成本清理,默认关) |
| SMAA | 允许 | 互斥 | — | 允许(同上) |
| TAA | 互斥 | 允许 | 允许 | — |

约束在契约层 `AntiAliasSettings` 校验(非法组合归一化并出诊断),graph 端不再判断。

### 帧时序与集成点

- **挂入方式**:链定稿表的每个 slot 对应 `feature_descriptors/` 中一条 `RenderFeaturePassDescriptor`(stage = `RenderPassStage::PostProcess`,带 executor id 与 IO 声明),由 `PostProcessStackDescriptor::from_resolved` 给出的开关集决定哪些 pass 进 compiled graph;关闭的效果不进图(§6.4),其上游孤立 pass 由计划 01 pass culling 裁掉(如 bloom 关闭时 downsample 链)。插件效果(FSR1、NN 后处理等)经 RenderFeature descriptor 注册同名槽位 executor,并随 descriptor 携带 `VolumeComponentDescriptor` 注册进 `VolumeComponentRegistry`;SMAA V1 已作为内建 terminal AA executor 接入,后续质量升级不再依赖外部插件 pass。
- **每帧 CPU 时序**:extract(volume 实体 → `PostProcessVolumeExtract`)→ 每相机 `VolumeEvaluator::evaluate`(在 submit_frame_extract 的 prepare 段,按计划 09 相机序)→ `from_resolved` 产出 stack descriptor → 与 `CompiledGraphCache` 键比对(效果开关集变化才重编译;**参数值变化只更新 uniform,不触发重编译**)→ executor 录制。
- **与计划 06 衔接**:`TaaResolveExecutor` 输出即本链 slot 1 输入;DoF/motion blur 在 TAA 之后,顺序定稿为 TAA → DoF → motion blur(对照 UE PostProcessing.cpp,计划 06 正文与细化章节已同步此顺序);jitter 去除由 TAA 完成,slot 2 起全部无 jitter;TAA 关闭时 slot 1 直通,后续无分支。
- **硬切换删除项清单**:① `volume.rs` 全文件(`RenderPostProcessVolume`/`RenderPostProcessVolumeStack`/`local_blend` 预算路径);② `stack.rs` 的 `from_extract_settings`/`from_extract_settings_with_anti_alias`/`from_extract_settings_with_effect_stack_and_anti_alias`;③ `PostProcessEffectKind::{EffectStack, ColorGrading, FinalComposite, HistoryResolve}` 及对应资源名;④ `feature_descriptors/color_grading.rs`、`feature_descriptors/history_resolve.rs`(后者与计划 06 TP-M4 同帧变更);⑤ `post_process.wgsl` 中 tonemap/LUT/vignette/grain/dither/CA/motion blur 代码段(迁入 uber.wgsl/motion_blur.wgsl 后删除);⑥ `execute_post_process` 内 motion blur 与 tonemap 分支。每项删除与新路径落地同一变更。

### 实施切片细化

**PP-M1 链顺序与色彩空间定稿**(里程碑节的切片展开;切片期只 `cargo check -p zircon_runtime --lib --locked`):

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M1-S1 | `chain.rs`(新)、`effect.rs`、`stack.rs` | `PostProcessChainSlot` + `from_resolved` 重建 backbone;effect kind 增删;资源名增删 | check 通过;graph dump 顺序与定稿表一致 |
| M1-S2 | `color_space.rs`(新)、各 executor RT 描述符 | 中间格式常量收敛 `rg11b10ufloat`;`execute_output_transfer` + `output_transfer.wgsl`;FinalComposite 删除 | 全部中间 RT 经常量取格式;输出 pass 唯一做 transfer |
| M1-S3 | `feature_descriptors/post_process.rs`、pipeline asset 模板、`compiled_render_pipeline.rs` | pass 表重排进 forward/deferred 模板 | 两模板 dump 一致;旧 pass 名清零 |

**PP-M2 Volume 容器框架**:

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M2-S1 | `volume_component.rs`/`volume_component/{params,tests}.rs`/`volume_registry.rs`(新) | descriptor/schema/插值函数表;内建 11 个效果组件注册(定稿表"Volume 组件"列);参数值/插值与测试 owner 分离 | 每个内建效果有 component_id 与默认值 schema |
| M2-S2 | `volume_extract.rs`(新)、`frame_extract.rs`、scene 侧 volume 实体投影 | global/box/sphere 形状进 extract;`volume_mask` 用计划 09 `RenderLayerSet` | extract 含 volumes 字段,box/sphere 携 blend_distance |
| M2-S3 | `volume_evaluator.rs`(新)、`submit_frame_extract/submit/submit.rs`、删除 `volume.rs` | 每相机求值替换静态 stack 输入;"全局容器默认值 = 旧 stack 值"迁移等价 | 旧 resolve 路径删除;默认场景产物不变 |

**PP-M3 曝光、LUT 与 uber 效果补齐**:

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M3-S1 | `exposure_settings.rs`(新)、`execute_exposure/`、两 wgsl、`feature_descriptors/post_process.rs`、history exposure buffer | histogram + 收敛 compute;ExposureBuffer 双缓冲持久;Manual 直写 | resolved exposure buffer 进 post-process binding 28;手动/自动同一消费路径 |
| M3-S2 | `execute_color_lut_bake/`、`color_lut_bake.wgsl`、`post_process_lut_texture` 叠加路径 | 3D LUT compute 烘焙,ACES/neutral 双曲线;用户 LUT 叠加 | neutral 恒等 LUT readback 误差 < 1/1024 |
| M3-S3 | `execute_uber/`、`uber.wgsl`、`execute_motion_blur/`、`execute_depth_of_field/`、瘦身 `execute_post_process/` | uber 合并五效果;motion blur/DoF 拆独立 pass;`post.stack` → `post.scene-composite` | 旧 post_process.wgsl 代码段删除;binding 表与本文档一致 |
| M3-S4 | blur 组件(`execute_bloom` 旁挂高斯 blur executor 复用 downsample 设施) | `post.blur` 供 UI/局部模糊复用 | blur 单测产物方差下降断言 |

**PP-M4 终端 AA 与动态分辨率**:

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M4-S1 | `anti_alias.rs`(契约)、`feature_descriptors/anti_alias.rs` | 互斥矩阵进 `AntiAliasSettings` 校验;FXAA/SMAA terminal pass 读取 `postprocess.terminal-aa-input` 并写 `final-color` | 非法组合归一化 + 诊断;终端 AA 编译路由正确 |
| M4-S2 | `dynamic_resolution.rs`(新)、`execute_upscale/`、`upscale.wgsl`、各 RT 描述符取缩放尺寸 | render_scale 贯通;bilinear upscale;FSR1 插件位 | scale 0.5→1.0 切换池统计无泄漏 |

### 测试与验收清单

单测位置:契约层测试与类型同文件 `#[cfg(tests)]`(现状惯例);执行器/产物测试在 `scene_renderer/post_process` 测试模块。命名遵守 §8.6。

| 测试函数 | 断言 | 位置 |
|---|---|---|
| `render_volume_registry_rejects_duplicate_component_id` | 重复注册返回 Err | `volume_registry.rs` |
| `render_volume_evaluator_blends_global_volumes_by_priority_order` | 既有 volume.rs 同名场景迁移,数值不变 | `volume_evaluator.rs` |
| `render_volume_evaluator_box_blend_distance_weight` | 相机在 box 边界 blend_distance 半程处 interp = 0.75(1 - d2/blend2) | `volume_evaluator.rs` |
| `render_volume_evaluator_sphere_boundary_zero_influence` | d2 > blend2 跳过;blend_distance=0 且在体内 → influence = weight | `volume_evaluator.rs` |
| `render_volume_evaluator_respects_camera_volume_mask` | mask 不相交 volume 不参与 | `volume_evaluator.rs` |
| `render_volume_evaluator_per_camera_independent` | 两相机不同位置得不同 stack | `volume_evaluator.rs` |
| `render_post_chain_backbone_order_is_stable` | 全开时 node 序 == 定稿表序 | `chain.rs` |
| `render_post_chain_disabled_effects_absent_from_graph` | 逐效果关闭后对应 pass 与孤立上游不在 compiled graph(接计划 01 culled 统计) | `chain.rs` + graph 测试 |
| `render_post_linear_chain_no_gamma_before_output_transfer` | 灰阶测试图过 slot 1–7 线性度偏差 < 1e-3 | 产物测试 |
| `render_post_exposure_histogram_buffer_layout` | bins=64、ExposureBuffer 16 B 偏移断言 | `execute_exposure`/bind group layout |
| `render_post_exposure_converges_with_speed` | 暗→亮阶跃后第 N 帧 exposure_scale_smoothed 按 exp2(-dt*speed_up) 曲线,误差 < 5% | 产物测试(readback) |
| `render_post_lut_bake_neutral_identity` | neutral + 默认 grading 烘出恒等 LUT | `execute_color_lut_bake` |
| `render_post_uber_binding_layout_matches_contract` | binding 编号表静态断言 | `execute_uber` |
| `render_post_anti_alias_mutex_matrix` | 互斥矩阵全组合归一化结果 | 契约层 anti_alias |
| `render_post_dynamic_resolution_scale_swap_releases_pool` | scale 切换两帧后池峰值条目回落 | graph/池测试 |
| `render_product_post_full_chain_all_effects_on` | 全效果开 vs 全关产物差异快照;RenderDoc 抓帧 pass 名与 dump 一致 | `render_product_*` |
| `render_product_post_volume_camera_transition` | 相机移入局部容器 bloom 强度逐帧平滑(PP-M2 验收证据的自动化形态) | `render_product_*` |

里程碑测试命令沿用里程碑节(`cargo test -p zircon_runtime post_process --locked`、`volume`、`render_graph` 过滤词);新增过滤词 `render_volume` 与 `render_post`。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|---|---|---|---|---|---|
| 2026-06-24 | Render product anti-alias particle/reactive tests owner split | 完成: anti-alias 粒子透明与 TAA reactive-mask 产品守卫已从 over-budget anti-alias 产品父测试迁入专属子 owner | `graphics/tests/render_product_anti_alias.rs` 从 1005 行降到 566 行,只保留通用 AA/FXAA/TAA 产品守卫、共享 capture/frame/assert helpers 和 `mod particle;`/`mod reactive_mask;`;新增 192 行 `graphics/tests/render_product_anti_alias/particle.rs` 承接粒子透明 pass、previous-state velocity gap stats 和粒子 motion-blur/TAA fixtures;新增 279 行 `graphics/tests/render_product_anti_alias/reactive_mask.rs` 承接 authored/transparent reactive-mask material writer path、material registration 和 reactive-mask graph assertions;新增 `runtime_15_render_product_anti_alias_focused_tests_are_child_owners` 结构守卫 | `render_plan07_product_anti_alias_particle_reactive_tests_owner_split_static_passed_cargo_deferred_implementation_cadence`;scoped rustfmt/static/line-count/docs-anchor/stale-path/whitespace/diff-check 通过;Cargo/WGPU/RenderDoc 按 milestone implementation cadence 暂缓,不声明新的通过 | 继续 Plan 07 terminal AA/upscale RenderDoc、blur/DoF/scene-composite RenderDoc 与更宽 product gates |
| 2026-06-24 | Render product post-process motion-blur test owner split | 完成: motion-blur 产品守卫已从 post-process 产品父测试迁入专属子 owner | `graphics/tests/render_product_post_process.rs` 从 913 行降到 609 行,只保留 uber light-effects、non-neutral tonemap/grading、user LUT 产品守卫、共享 viewport/capture/assert helpers 和 `mod motion_blur;`;新增 323 行 `graphics/tests/render_product_post_process/motion_blur.rs` 承接 `render_product_post_motion_blur_split_uses_velocity_and_changes_final_frame`、particle velocity fixture、motion-blur product framework、local executor/order/scene-velocity readback assertions;新增 `runtime_15_render_product_post_process_motion_blur_tests_are_child_owner` 结构守卫 | `render_plan07_product_post_process_motion_blur_test_owner_split_static_passed_cargo_deferred_active_compile_lane`;scoped rustfmt/static/line-count/docs-anchor/whitespace/diff-check 通过;当前有其他 Cargo/Rust 编译通道活跃,本切片未声明新的 Cargo/WGPU/RenderDoc 通过 | 继续 Plan 07 terminal AA/upscale RenderDoc、blur/DoF/scene-composite RenderDoc 与更宽 product gates |
| 2026-06-24 | Plan 07 SSR GPU context test owner split | 完成: SSR GPU graph bridge production owner 与 mip-pass tests 已分离 | `graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs` 降到 656 行并只保留 SSR resolve/specular occlusion/reflection pyramid/reflection pyramid coarse WGPU graph bridge helpers 与 `mod tests;`;新增 `graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection/tests.rs` 承接 3 个 `ssr_parent_pyramid_mip_passes` 行为测试;新增 `runtime_15_ssr_gpu_context_tests_are_child_owner_split` 结构守卫 | `render_plan07_ssr_gpu_context_test_owner_split_static_passed_cargo_deferred_active_compile_lane`;scoped rustfmt/static/line-count/docs-anchor/whitespace/diff-check 通过;当前有其他 Cargo/Rust 编译通道活跃,本切片未声明新的 Cargo/WGPU/RenderDoc 通过 | 继续 Plan 07 terminal AA/upscale RenderDoc、blur/DoF/scene-composite RenderDoc 与更宽 product gates |
| 2026-06-23 | Plan 07 post-process stack owner split | 完成: stack 描述符 owner、资源名表 owner 与行为测试 owner 已分离 | 新增 `core/framework/render/post_process/graph_resource_names.rs` 承接 `PostProcessGraphResourceNames`;`stack.rs` 降到 586 行并只挂载 `mod tests;`;17 个原 inline stack tests 迁入 `core/framework/render/post_process/stack/tests/{exposure,terminal_chain,screen_space_reflection,temporal_history,effect_stack}.rs`;新增 `runtime_15_post_process_stack_is_folder_backed` 结构守卫 | `render_plan07_post_process_stack_owner_split_static_passed`;scoped rustfmt/static/line-count/docs-anchor/diff-check 通过;本切片未声明新的 Cargo/WGPU/RenderDoc 通过 | 继续 Plan 07 terminal AA/upscale RenderDoc、blur/DoF/scene-composite RenderDoc 与更宽 product gates |
| 2026-06-23 | Plan 07 volume component owner split | 完成: volume component 根文件、参数契约 owner 与行为测试 owner 已分离 | 新增 `core/framework/render/post_process/volume_component/params.rs` 承接 `VolumeParamValue`/`VolumeParamSchema`/`VolumeParamInterpFn` 与 `interp_*`/参数默认值工厂;新增 `core/framework/render/post_process/volume_component/tests.rs` 承接 5 个原 inline tests;`volume_component.rs` 降到 642 行并只通过 `mod params;` 与 `mod tests;` 挂载子 owner;新增 `runtime_15_post_process_volume_component_is_folder_backed` 结构守卫 | `render_plan07_volume_component_owner_split_static_passed`;scoped rustfmt/static/line-count/docs-anchor/diff-check 通过;focused locked `render_volume_component` Cargo 测试 184s 超时无结果且未发现本 target 残留,未计 Cargo 通过;本切片未声明新的 WGPU/RenderDoc 通过 | 继续 Plan 07 terminal AA/upscale RenderDoc、blur/DoF/scene-composite RenderDoc 与更宽 product gates |
| 2026-06-23 | Plan 07 volume camera transition product guard | 部分完成: 计划表 `render_product_post_volume_camera_transition` 产品守卫源码已补,focused Cargo 未返回结果 | 新增 `graphics/tests/render_product_post_process_volume.rs` 与 `mod.rs` 挂载;测试用真实 headless WGPU 三视口提交 camera 在 volume 外、blend 区和中心的帧,通过 `PostProcessVolumeExtract` + `VolumeShapeExtract::sphere` 驱动 `post.vignette`,断言 `post.uber`/`post.output-transfer` executor 与角落 luma 逐步下降、最终帧 delta 增长;避免继续扩大 913 行 `render_product_post_process.rs` | `render_plan07_volume_camera_transition_product_guard_static_passed_cargo_timeout_no_result`;scoped rustfmt/static/line-count 通过;focused locked `render_product_post_volume_camera_transition` Cargo 测试 245s 超时无结果且无 cargo/rustc 残留;本切片未声明新的 Cargo/WGPU/RenderDoc 通过 | 重新跑 focused Cargo;继续 `render_product_post_full_chain_all_effects_on` 与 Plan 07 RenderDoc evidence |
| 2026-06-23 | Plan 07 full-chain all-effects product guard | 部分完成: 计划表 `render_product_post_full_chain_all_effects_on` 产品守卫源码已补,focused Cargo 测试未返回结果 | 新增 `graphics/tests/render_product_post_process_full_chain.rs` 与 `graphics/tests/render_product_post_process_full_chain/fixture.rs`,并挂到 `graphics/tests/mod.rs`;测试用真实 headless WGPU mesh + particle velocity 场景,同时打开 histogram exposure、bloom、DoF、motion blur、SSR/fog scene-composite、blur、color LUT bake/tonemap/user LUT、vignette/grain/dither/CA、dynamic-resolution upscale 与 SMAA terminal AA,断言 executor 顺序、active families、关键 alias/backing、scene-velocity readback 和最终帧 delta;主测试 542 行、fixture 414 行,避免扩大 913 行 `render_product_post_process.rs` | `render_plan07_full_chain_all_effects_product_guard_static_passed_cargo_timeout_no_result`;scoped rustfmt/static/line-count 通过;locked core-min `cargo check` 通过(既有 warnings);focused locked `render_product_post_full_chain_all_effects_on` Cargo 测试 604s 超时无结果且未发现本 target-dir 残留;本切片未声明新的 WGPU/RenderDoc 通过 | 重新跑 focused Cargo test;继续 Plan 07 full-chain/terminal AA/upscale/blur/DoF/scene-composite RenderDoc evidence |
| 2026-06-23 | Plan 07 built-in post-process executor owner split | 完成: 内建后处理 executor 根文件已拆成 folder-backed owner,注册表可见执行器入口不变 | `builtin_postprocess_executors.rs` 降到 574 行并只保留 bloom/color-lut/TAA/output-transfer/upscale/FXAA/SMAA/motion-blur/blur/DoF/scene-composite/exposure/SSAO/clustered/HZB/velocity/motion-vector/SSR/uber 等 executor 函数;新增 `builtin_postprocess_executors/frame_effects.rs` 承接 frame effect predicate,`graph_resources.rs` 承接 `product_postprocess_executor(...)` 与 graph resource kind/external binding 校验,`resource_routing.rs` 承接 terminal/bloom/uber 输入输出选择和 8 个原 inline 路由测试;新增 `runtime_15_builtin_postprocess_executors_are_folder_backed` 结构守卫 | `render_plan07_builtin_postprocess_executor_owner_split_static_passed`;scoped rustfmt/static/line-count/docs-anchor/diff-check 通过;locked core-min `cargo check` 通过(既有 warnings);focused locked `runtime_15_builtin_postprocess_executors_are_folder_backed` Cargo test 被当前 `Cargo.lock` 更新需求在编译前阻断,未计 test/WGPU/RenderDoc 通过 | 继续 Plan 07 focused Cargo rerun、full-chain/terminal AA/upscale/blur/DoF/scene-composite RenderDoc evidence |
| 2026-06-23 | Render index 当前状态总览拆分 | exposure 完成,LUT bake compute、neutral/user LUT readback reference path、非中性 tonemap/grading CPU reference/曝光读回产品守卫、真实 user LUT WGPU 产品守卫、DoF/motion blur 独立 WGPU pass、motion blur split WGPU 产品守卫、DoF split WGPU 产品守卫、split pass HDR 中间格式修正、`post.scene-composite` SSR/fog split 与 fog/SSR wider 产品守卫、`post.blur` 通用 blur split、terminal FXAA/SMAA、SMAA 内部三阶段 edge/blend/resolve、dynamic-resolution upscale、terminal AA/upscale 产品守卫、render-scale 池统计和 uber 轻效果产品守卫已接入但仍部分完成 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 07 PostProcess/Color 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：terminal AA/upscale RenderDoc、blur/DoF/scene-composite RenderDoc；验收缺口：需要 terminal AA/upscale RenderDoc、DoF/scene-composite/blur RenderDoc,以及更宽 product scene/RenderDoc;当前整包 fmt 被无关 UI 文件格式漂移阻塞 |
| 2026-06-15 | PP-M1-S1 post-process chain slot contract | 部分完成: 源码契约已接入, 源码测试编译未完成 | 新增 `post_process/chain.rs`;公开 `PostProcessChainSlot`;为 `PostProcessPassNode` 增加 `chain_slot` 与 `planned_chain_executor_id`;为 `PostProcessPassGraph` 增加 `planned_backbone_slots` 与 `active_chain_slots`;测试夹具改走 `PostProcessPassGraph::from_ordered_nodes(...)` | `rustfmt --edition 2021 --check` 通过 scoped Rust 文件;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-post-chain-0615 --message-format short --color never` 通过, 70 个既有 warning;`cargo test -p zircon_runtime --lib render_post_chain ...` 两次超过 604s/604s,未返回过滤测试结果,遗留 cargo/rustc 已停止;符号扫描确认链槽位写入 `chain.rs`、`pass_node.rs`、`pass_graph.rs`;`git diff --check` 仅 LF/CRLF 提示 | 重新跑 `render_post_chain` 过滤测试或生成 graph dump;PP-M1-S2 继续中间格式与 output-transfer 硬切;PP-M1-S3 再把旧 `FinalComposite`/`ColorGrading` pass 名清零 |
| 2026-06-15 | PP-M1-S2a color-space/intermediate HDR format contract | 部分完成: color-space/RHI/RT 描述符已接入, output-transfer executor 未切 | 新增 `post_process/color_space.rs` 与 `RenderPostProcessTextureFormat`/`RenderOutputTransfer`;公开 `INTERMEDIATE_HDR_FORMAT_DEFAULT = rg11b10ufloat`;RHI 增加 `TextureFormat::Rg11b10Ufloat`;图编译将 HDR `scene-color`、`TAA_OUTPUT`、SSR reflection pyramid 走统一中间 HDR 常量,保留 HZB/TAA history/运动向量为高精度 `rgba16float`;TAA resolve 与 SSR reflection pyramid WGPU target 常量同步;渲染设备请求 `RG11B10UFLOAT_RENDERABLE`;WGPU transient 创建和 pool key 支持新格式 | `cargo fmt --package zircon_runtime -- --check` 通过;scoped `git diff --check` 仅 LF/CRLF 提示;符号扫描确认 `Rg11b10Ufloat` 覆盖 framework/RHI/compile/WGPU 常量/TAA pipeline/request_device;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-post-color-s2-0615 --message-format short --color never` 通过,71 个既有 warning;`cargo test -p zircon_runtime --lib render_post_color_space ...` 超过 904s,停在 shared lib-test 编译,遗留 cargo/rustc 已停止 | PP-M1-S2b 继续落 `execute_output_transfer` + `output_transfer.wgsl`,并删除/替换旧 `FinalComposite`;PP-M1-S3 再清 pass 表旧名和模板顺序 |
| 2026-06-15 | PP-M1-S2b output-transfer pass hard cut | 部分完成: WGPU output-transfer 已接入, terminal AA/uber 旧职责仍待 PP-M3/PP-M4 瘦身 | 删除 `PostProcessEffectKind::FinalComposite`,改为 `OutputTransfer`;`PostProcessPassGraph` 改报 `output_transfer_node`;运行时统计/诊断键同步为 output-transfer;新增 `PostProcessGraphResourceNames::TONEMAPPED`;`post.stack` 由写最终目标改为写 `postprocess.tonemapped`;新增 `output_transfer.wgsl`、output-transfer bind-group layout、pipeline、`execute_output_transfer` 与 `post.output-transfer` executor;post-process pass 表新增 `output-transfer` pass 写 `FINAL_COLOR`;旧 `post.final-composite` executor 名清零;FXAA 旧 no-op descriptor 在本切片过滤掉,等待 PP-M4 terminal AA 重接 | `cargo fmt --package zircon_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-post-color-s2-0615 --message-format short --color never` 通过,71 个既有 warning;`cargo test -p zircon_runtime --lib output_transfer ...` 超过 304s,停在 shared lib-test 编译,匹配本 target 的 cargo/rustc 已停止;scoped `git diff --check` 仅 LF/CRLF 提示;源码扫描确认 `FinalComposite`/`post.final-composite`/`output-transferd` 清零,output-transfer shader/pipeline/executor/TONEMAPPED 符号存在 | PP-M1-S3 继续清 `ColorGrading`/`EffectStack` 旧 pass 名并同步模板 graph dump;PP-M3/PP-M4 再把 uber、FXAA 与 transfer 的职责彻底拆开;重新跑 `output_transfer`/`render_post_color_space` 过滤测试 |
| 2026-06-15 | PP-M1-S3 planned pass table and legacy name cleanup | 部分完成: planned chain 名称已切到运行时契约,focused lib-test 仍待补跑 | `PostProcessEffectKind::{ColorGrading, EffectStack}` 硬切为 `ColorLutBake`/`Uber`;`feature_descriptors/color_grading.rs` pass 改 `color-lut-bake`/`post.color-lut-bake`;post-process 主 pass 改 `uber`/`post.uber`;执行器注册表移除旧 `post.color-grade`、`post.effect-stack`、`post.stack`;pipeline compile 过滤、运行时统计、post-process graph fixtures、forward/deferred 期望 pass dump 同步到 planned 名称;保留 `RenderColorGradingSettings` 与 `RenderPostProcessEffectStackSettings` 数据 schema,等待 PP-M2/PP-M3 schema 迁移 | `cargo fmt --package zircon_runtime` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-post-color-s2-0615 --message-format short --color never` 通过,71 个既有 warning;源码扫描确认 `PostProcessEffectKind::ColorGrading`、`PostProcessEffectKind::EffectStack`、`post.color-grade`、`post.effect-stack`、`post.stack` 清零,仅 `RenderPhase::PostProcess` 的 phase label 仍为 `"post-process"` | 重新跑 `render_post_chain`/`pipeline_compile`/`render_pass_executor_registry` focused lib-tests;PP-M3 再把当前 `record_post_process_stack`/`execute_post_process` 内部实现拆成 `execute_uber`、`execute_motion_blur`、`execute_depth_of_field`、`execute_color_lut_bake`;PP-M4 重接 terminal AA |
| 2026-06-15 | PP-M2-S1 Volume component registry contract | 部分完成: schema/registry 契约已落地,尚未接入 volume extract/evaluator | 新增 `post_process/volume_component.rs` 与 `post_process/volume_registry.rs`;公开 `VolumeParamValue`/`VolumeParamSchema`/`VolumeComponentDescriptor`/`VolumeComponentRegistry`;内建注册 14 个 component_id: DoF、motion blur、bloom、exposure、SSR、screen-space fog、color grading、tonemap、vignette、grain、dither、chromatic aberration、color lookup、blur;descriptor apply 可写回现有 `RenderResolvedPostProcessSettings`,exposure 先占 schema 位等待 PP-M3 | `cargo fmt --package zircon_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-post-color-s2-0615 --message-format short --color never` 通过,当前 warning 仍为既有未清理项;scoped `git diff --check` 仅 LF/CRLF 提示;源码扫描确认新 registry/schema 符号与全部计划 component_id 存在,PP-M1-S3 退休名扫描仍清零 | PP-M2-S2 增 `PostProcessVolumeExtract` 与 global/box/sphere 形状;PP-M2-S3 用 evaluator 替换旧 `volume.rs` 的 `local_blend` 预算路径;focused `render_volume_registry`/`render_volume_component` lib-tests 待共享 lib-test 编译窗口稳定后补跑 |
| 2026-06-15 | PP-M2-S2 Volume extract shape snapshot | 完成: frame extract 已携 planned volume DTO,空间求值交由 PP-M2-S3 evaluator | 新增 `post_process/volume_extract.rs`,公开 `PostProcessVolumeExtract`/`VolumeShapeExtract`/`VolumeComponentOverride`;`PostProcessExtract` 增 `volumes: Vec<PostProcessVolumeExtract>`;scene world 提取拆出 `scene/world/render_post_process.rs`,将全局 volume、box collider、sphere collider 投影为 planned shape snapshot,并把旧 profile 映射到 component override;capsule 按本阶段 planned DTO 范围不投影 | `cargo fmt --package zircon_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-post-color-s2-0615 --message-format short --color never` 通过,71 个既有 warning;PP-M2-S2 符号扫描确认 extract/shape/override/frame volumes 接入;scoped `git diff --check` 仅 LF/CRLF 提示;`scene/world/render.rs` 由 979 行降至 793 行,post-process volume 提取职责移入 192 行新模块 | PP-M3 继续把 exposure/LUT/uber 运行时参数接入 volume component schema;需要 capsule/convex volume 时另开形状扩展切片 |
| 2026-06-15 | PP-M2-S3 VolumeEvaluator hard cut | 完成: evaluator 接入提交路径,旧 `volume.rs`/`volume_stack` 公共求值面已删除 | 新增 `post_process/volume_evaluator.rs` 与 `VolumeEvaluator`/`VolumeEvaluationRequest`/`VolumeEvaluationError`/`ResolvedPostProcessStack`;新增 `resolved_stack.rs` 与 `volume_profile.rs` 作为拆分 owner;`VolumeComponentDescriptor` 增 `read` 函数指针与 `read_values`,evaluator 以 registry schema 读取当前强类型参数、应用 overrideState(None 保持当前值)、按 global/box/sphere shape 权重插值后写回;`PostProcessExtract::resolved_settings_for_camera(...)` 成为唯一 per-camera 求值入口;删除 `RenderPostProcessVolume`/`RenderPostProcessVolumeStack`/`PostProcessExtract.volume_stack`/`resolved_settings_for_layers(...)`;scene/submit/测试全部转向 planned `volumes` DTO | `cargo fmt --package zircon_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-post-color-s2-0615 --message-format short --color never` 通过,当前 warning 仅既有未清理项;`cargo test -p zircon_runtime --lib render_volume_evaluator ...` 通过 6 个过滤测试;`cargo test -p zircon_runtime --lib render_post_process_extract ...` 通过 9 个过滤测试;`render_frame_extract_carries_scene_post_process_volumes_for_camera_layers`、`inactive_post_process_volume_hierarchy_is_excluded_from_frame_extract`、`render_framework_stats_report_volume_effect_stack_product_node_when_authored` 过滤测试均通过 | PP-M3 继续 exposure histogram/LUT bake/uber pass;PP-M4 重接 terminal AA/dynamic resolution;后续再补 `render_volume_extract` 与更宽 `render_post_chain`/`pipeline_compile`/executor registry sweep |
| 2026-06-15 | PP-M3-S1a exposure settings and volume contract | 部分完成: framework/submit 曝光参数契约已接入, WGPU histogram/resolve executor 未落地 | 新增 `post_process/exposure_settings.rs` 与 `RenderExposureSettings`/`RenderExposureMode`/histogram buffer 常量;`RenderResolvedPostProcessSettings`、`PostProcessExtract`、`VolumeEvaluationRequest`、`FrameSubmissionContext` 增 `exposure`;scene extract 将相机 `exposure_ev100` 写入 manual exposure;`post.exposure` descriptor 从 schema-only/no-op 改为真实 read/apply,支持 mode/manual_ev100/compensation/min/max/percent/speed 参数 | scoped `rustfmt --edition 2021` 通过 touched Rust 文件;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-post-color-s2-0615 --message-format short --color never` 通过,71 个既有 warning;`render_exposure` 过滤测试通过 2 个;`render_volume_component_descriptor_applies_exposure_values`、`render_volume_evaluator_blends_exposure_component`、`render_extract_projects_scene_camera_component_product_fields` 过滤测试均通过 | PP-M3-S1b 继续落 `execute_exposure_histogram`/`execute_exposure_resolve`、双缓冲 `ExposureBuffer`、WGSL 与 group0 消费路径;PP-M3-S2 再接 LUT bake |
| 2026-06-15 | PP-M3-S1b WGPU exposure histogram/resolve | 完成: WGPU exposure histogram/resolve、历史双缓冲和最终 pass 消费路径已接入 | `PostProcessEffectKind` 增 `ExposureHistogram`/`ExposureResolve`;`PostProcessGraphResourceNames` 增 `EXPOSURE_HISTOGRAM`/`EXPOSURE_PREVIOUS`/`EXPOSURE_CURRENT`;histogram mode 编译 `exposure-histogram`,manual mode 跳过 histogram 但仍执行 resolve;`feature_descriptors/post_process.rs` 声明 `post.exposure.histogram`/`post.exposure.resolve` async compute workload;新增 exposure params/bind-group layouts/pipelines/default buffers/WGSL;`SceneFrameHistoryTextures` 持有 exposure read/write buffer 并在成功帧 flip;`post_process.wgsl` 通过 binding 28 读取 resolved exposure multiplier;`RenderHistoryCopyReport` 与 runtime diagnostics 增 `exposure_copied` | scoped `rustfmt --edition 2021` 通过 touched Rust 文件;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-wgpu-exposure-0615 --message-format short --color never` 通过,71 个既有 warning;`cargo test -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-wgpu-exposure-0615 exposure_ -- --nocapture` 通过 9 个过滤测试;`effective_post_process_stack_culls_disabled_optional_post_process_passes`、`rendering_plugin_default_features_restore_legacy_forward_plus_pass_order`、`runtime_diagnostics` 过滤测试通过 | PP-M3-S2 继续 `color_lut_bake` compute 与 3D LUT 烘焙;PP-M3-S3 再把当前 uber/scene-composite 职责拆薄并补全 vignette/grain/dither/CA 专项产物测试 |
| 2026-06-16 | PP-M3-S2 COLOR_LUT graph resource bootstrap | 部分完成: 图资源、WGPU 3D transient 与 neutral LUT 上传路径已接通,真实 grading/tonemap compute 烘焙未完成 | 新增 `PostProcessGraphResourceNames::COLOR_LUT`;`ColorLutBake` 节点产出 `postprocess.color-lut`,`OutputTransfer` 经 `post.uber` 读取 tonemapped 输出;`feature_descriptors/post_process.rs` 将 `post.color-lut-bake` 插到 `post.uber` 前并写 storage texture;no-stack 编译过滤 `post.color-lut-bake`/`COLOR_LUT`;pipeline compile 将 `COLOR_LUT` 描述为 32x32x32 `Rgba16Float` 3D transient,带 sampled/storage/copy usage;新增 `execute_color_lut_bake` 上传 neutral identity LUT;`record_post_process_stack` 在资源存在时把 baked 3D LUT 传给 `post.uber` | `cargo fmt --package zircon_runtime` 通过;`git diff --check` 无空白错误,仅仓库既有 LF/CRLF 提示;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-color-lut-0616 --message-format short --color never` 通过,71 个既有 warning;`cargo test -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-color-lut-0616 color_lut -- --nocapture` 超过 604s,未返回测试结果,匹配该 target-dir 的残留 cargo/rustc 已停止 | PP-M3-S2 继续实现 `color_lut_bake.wgsl` compute,把 color grading、tonemap、user LUT 叠加烘入 3D LUT,并补 readback 误差验收;PP-M3-S3 再瘦身 `execute_post_process`/uber 职责 |
| 2026-06-16 | PP-M3-S2 color_lut_bake WGPU compute path | 部分完成: WGPU compute 烘焙路径已接入,readback/真实产品场景验收未完成 | 新增 `color_lut_bake.wgsl`、`ColorLutBakeParams`、params buffer、bind-group layout 与 compute pipeline;`post.color-lut-bake` 改为 async compute workload,固定 workgroup `[4,4,4]`、dispatch `[8,8,8]`,读 `EXPOSURE_CURRENT`,按 color grading、tonemap 与用户 2D/2D-strip/3D LUT 写 32³ `COLOR_LUT`;runtime 录制 `zircon-color-lut-bake-pipeline` compute dispatch 并把内部 baked LUT 以 `post.uber` mode 4 绑定,避免二次 tonemap/grading;graph contract 收紧为 `ColorLutBake` 只依赖 `EXPOSURE_CURRENT`,不再伪依赖 `SCENE_COLOR` | `cargo fmt --package zircon_runtime` 通过;`cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-color-lut-0616` 被无关 plugin/native-loader 编译漂移阻塞: `crate::plugin::{native_bridge_method_descriptors_from_manifest,NativeBridgeMethodBinding,NativeHostBridgeCallScope,NativePluginCandidate,NativePluginLoadReport}` re-export 缺失并引发后续类型推断错误;本次阻塞发生在 `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/*`,未到达 render 新改动 | 补跑 render-only `cargo check`/`color_lut_bake` 过滤测试;增加 readback: neutral identity 与用户 LUT 误差 < 1/1024;做 RenderDoc/product scene 捕获;随后进入 PP-M3-S3 瘦身 `execute_post_process`、拆 `execute_uber`/motion blur/DoF |
| 2026-06-16 | PP-M3-S2 color_lut_bake dispatch diagnostics | 部分完成: 调度诊断能区分曝光输入与 LUT 输出,readback/产品场景验收仍未完成 | `RenderGraphComputeDispatchRecord` 增 `resource_accesses` 诊断面并保持旧 `storage_write_resources`;`post.color-lut-bake` dispatch 现在记录 `EXPOSURE_CURRENT` buffer read 和 `COLOR_LUT` texture write;新增 `color_lut_bake_dispatch_reports_exposure_read_and_lut_write` 与 compute dispatch metadata 覆盖 | `cargo fmt -p zircon_runtime -- --check` 通过;scoped `git diff --check` 仅 LF/CRLF 提示;`cargo test -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-render-dispatch-0616 color_lut_bake_dispatch_reports_exposure_read_and_lut_write -- --nocapture` 超过 184s 停在编译阶段,已结束匹配该 target-dir 的残留 cargo/rustc | 继续补 readback: neutral identity 与用户 LUT 误差 < 1/1024;补 RenderDoc/product scene 捕获;随后推进 PP-M3-S3 `execute_uber`/motion blur/DoF 拆分 |
| 2026-06-16 | PP-M3-S2 neutral LUT readback validation path | 部分完成: neutral identity readback 通路与产品断言已落地,过滤测试仍受共享编译耗时阻塞 | 新增 `RenderColorLutReadbackReport` 与 `COLOR_LUT_IDENTITY_EPSILON_MICRO=977`;新增测试专用 `read_texture_rgba16float_3d` 读回 helper,按 3D slice/row 剥离 WGPU copy padding;`RenderGraphExecutionRecord`、renderer last report、`RenderStats.last_color_lut_readback_report` 接入 LUT 报告;`render_framework_stats_report_neutral_color_lut_readback_identity` 断言 `post.color-lut-bake` 执行且 32³ neutral LUT identity 误差不超过 1/1024 | `cargo fmt -p zircon_runtime` 通过;scoped `git diff --check` 仅 LF/CRLF 提示;`cargo test -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-color-lut-readback-0616 color_lut_readback_report -- --nocapture` 超过 304s 停在编译阶段,已结束匹配该 target-dir 的残留 cargo/rustc | 补跑 `color_lut_readback_report` 与 `render_framework_stats_report_neutral_color_lut_readback_identity`;继续补用户 LUT readback 误差与 RenderDoc/product scene 捕获;随后推进 PP-M3-S3 `execute_uber`/motion blur/DoF 拆分 |
| 2026-06-16 | PP-M3-S2 color_lut readback diagnostics | 部分完成: readback report 已进入 runtime diagnostics,过滤测试仍待共享编译窗口补跑 | 新增 `core/runtime/diagnostics/render_stats_store/post_process.rs`,把 `RenderStats.last_color_lut_readback_report` 记录到 `render.post_process.color_lut.readback.*`;覆盖 available、identity_within_epsilon、尺寸、byte_len/expected_byte_len、sample_count、invalid_byte_len、invalid_sample_count、identity_max_abs_error_micro、identity/alpha out-of-tolerance counts;`render_stats_store.rs` 注册独立 post-process 诊断模块,避免继续扩大 product 统计文件 | `cargo fmt -p zircon_runtime` 通过;`cargo fmt -p zircon_runtime -- --check` 通过;scoped `git diff --check` 无空白错误,仅 LF/CRLF 提示;不立即重跑 focused Cargo test,此前 `color_lut_readback_report` 过滤测试 304s 编译超时,按里程碑实施节奏延后到测试阶段 | 补跑新增 `post_process_diagnostics_record_color_lut_readback_identity_report`、`color_lut_readback_report`、`render_framework_stats_report_neutral_color_lut_readback_identity`;继续补用户 LUT readback 误差与 RenderDoc/product scene 捕获;随后推进 PP-M3-S3 `execute_uber`/motion blur/DoF 拆分 |
| 2026-06-16 | PP-M3-S2 user LUT readback reference path | 部分完成: 用户 LUT 参考路径已接入测试构建,产品场景/RenderDoc 与非中性曝光-tonemap 组合验证仍待后续 | `RenderColorLutReadbackReport` 增 `RenderColorLutReadbackReference::{Identity,UserLut}`、reference-space max error/out-of-tolerance 字段与 `reference_within_epsilon()`/`user_lut_within_epsilon()`;diagnostics 增 `render.post_process.color_lut.readback.reference_*`;`attach_color_lut_readback_stats` 现在读取当前 `ViewportRenderFrame` 与 `ResourceStreamer`,当已绑定同尺寸 RGBA8 用户 LUT 且 tonemap 为默认时,从同一 CPU texture asset 按 shader 契约计算 1D 2D、2D strip 或 3D 用户 LUT 参考,再用 `from_raw_rgba16_float_user_lut_bytes(...)` 校验 baked 3D LUT;无法可靠生成参考时保持 identity 报告,避免误报 | touched-file `rustfmt --edition 2021` 通过;`cargo check -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-user-lut-readback-check-0616` 通过(70 个既有 warning);此前 `cargo test ... color_lut_readback_report` 超过 304s 停在共享 lib-test 编译,用户明确测试可延后,本切片不继续长测试 | 后续补真实产品场景与 RenderDoc;补曝光 buffer 读回后扩展非中性色彩分级/tonemap 的 CPU reference;再推进 PP-M3-S3 `execute_uber`、motion blur、DoF 拆分与 SMAA 产品验收/多阶段质量升级 |
| 2026-06-21 | PP-M3-S2 non-neutral tonemap/grading WGPU product guard | 部分完成: 非中性 ACES/曝光/色彩分级产品守卫已自动化,CPU reference 与 RenderDoc 仍待后续 | `render_product_post_non_neutral_tonemap_grading_changes_final_frame` 用真实 headless WGPU 提交 baseline 与非中性 tonemap/grading 双视口;断言 `post.color-lut-bake`、`post.uber`、`post.output-transfer` executor 执行,`RenderColorLutReadbackReport` 报告 32^3 LUT、样本数正确、无无效 sample、且不再是 identity;最终 `CapturedFrame` 全帧 RGB delta 与中心 luma delta 证明 baked LUT 影响产品输出。 | `rustfmt --edition 2021 zircon_runtime/src/graphics/tests/render_product_post_process.rs zircon_runtime/src/graphics/tests/mod.rs` 已执行;`cargo test -p zircon_runtime --lib render_product_post_non_neutral_tonemap_grading_changes_final_frame --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1,既有 warnings 保留;首次 240s 调用超时发生在共享 lib-test 编译阶段,等待残留 cargo/rustc 结束并复用目标目录后同一 exact filter 通过。 | PP-M3-S2 仍需补 exposure buffer readback 后再把非中性 tonemap/grading 纳入 CPU reference 误差校验;PP-M3-S3/PP-M4 仍需 DoF/motion blur/scene-composite/blur/SMAA/terminal AA/upscale 产品和 RenderDoc 验收。 |
| 2026-06-21 | PP-M3-S2 user LUT WGPU product guard | 部分完成: 真实 2D-strip 用户 LUT 产品守卫已自动化,非中性 CPU reference 与 RenderDoc 仍待后续 | `render_product_post_user_lut_texture_changes_final_frame_and_matches_readback_reference` 在 `ProjectAssetManager` 中注册 32^3 对应的 1024x32 线性 RGBA8 2D-strip 用户 LUT,用真实 headless WGPU 提交 baseline 与 user-LUT 双视口;断言 LUT asset 经 ResourceStreamer 准备成功(`request=1/ready=1/fallback=0/2d_strip=1`),`post.color-lut-bake`、`post.uber`、`post.output-transfer` executor 执行,`RenderColorLutReadbackReport.reference == UserLut` 且 `user_lut_within_epsilon()`,并用最终帧 RGB delta 与红/绿通道下降证明用户 LUT 影响产品输出。 | `rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_post_process.rs zircon_runtime/src/graphics/tests/mod.rs` 通过;`cargo test -p zircon_runtime --lib render_product_post_user_lut_texture_changes_final_frame_and_matches_readback_reference --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1;随后 `cargo test -p zircon_runtime --lib render_product_post_ --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 15/15,既有 warnings 保留。 | PP-M3-S2 剩余为非中性 CPU reference/曝光 readback 与 RenderDoc;PP-M3-S3/PP-M4 仍需 DoF/motion blur/scene-composite/blur/SMAA/terminal AA/upscale 产品和 RenderDoc 验收。 |
| 2026-06-21 | PP-M3 graph contract focused refresh | 完成: framework post-process graph 合同测试已对齐 exposure/scene-composite planned chain | `zircon_runtime/src/core/framework/tests.rs` 中旧 post-process graph 断言刷新为当前链路:默认 disabled-effects/history 场景保留 `ExposureResolve -> OutputTransfer`,fog 场景在 `ColorLutBake` 后加入 `SceneComposite`,SSR 场景加入 reflection pyramid/specular/resolve 后的 `SceneComposite`,并断言 `post.uber` 从 `SCENE_COMPOSITED`/`COLOR_LUT` 读取而不再重复要求 `SCENE_DEPTH`。因该文件 1682 行且本次只更新既有断言,未在此切片拆分大测试文件。 | `cargo test -p zircon_runtime --lib render_product_post_ --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 15/15,覆盖 12 个 framework post-process graph 合同测试和 3 个 WGPU 产品守卫,既有 warnings 保留。 | 后续仍需更宽 `post_process`/SMAA/terminal/upscale focused sweep、CPU reference 扩展和 RenderDoc 验收。 |
| 2026-06-16 | PP-M3-S3 DoF/motion blur split | 部分完成: DoF 与 motion blur 已从 `post.uber` 拆为独立 WGPU fullscreen passes,但剩余轻效果和产品验收仍未收口 | `PostProcessEffectKind::{DepthOfField,MotionBlur}` 接入链槽与 pass graph;新增 `postprocess.depth-of-fielded`、`postprocess.motion-blurred` 中间资源;`post.depth-of-field`/`post.motion-blur` pass、executor registry、编译过滤、HDR transient 描述与 graph resource 路由已接入;新增 `fs_depth_of_field`/`fs_motion_blur` shader entry、`depth_of_field_pipeline.rs`/`motion_blur_pipeline.rs`、`execute_depth_of_field`/`execute_motion_blur`;`record_post_process_stack` 在这些节点存在时跳过 uber 内部对应职责,后续 `post.uber` 读取 split 后的场景色 | touched-file `rustfmt --edition 2021` 通过;`cargo check -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-motion-blur-split-check-0616` 通过(70 个既有 warning);`cargo check -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-dof-split-check-0616` 通过(70 个既有 warning);按用户要求本轮不继续长 Cargo tests | 继续瘦身 `post.uber`:SSR/fog、chromatic aberration、vignette、grain、dither、通用 blur 仍在 uber/scene composite 路径;补 DoF/motion blur 产品场景、RenderDoc 截帧、focused graph/registry/shader tests;继续 SMAA 产品验收/多阶段质量升级 |
| 2026-06-16 | PP-M3-S3 bloom source routing after split passes | 部分完成: bloom 下游输入已跟随 TAA/DoF/motion blur 后的最新 scene color,但产品场景与 RenderDoc 仍待后续 | pipeline compile 在 `post.bloom-extract` 过滤后调用 `route_bloom_to_latest_scene_color_input(...)`,按 MotionBlur → DepthOfField → TAA → scene-color 优先级把 bloom read resource 从 `SCENE_COLOR` 改写为最新 split 输出;`bloom_extract_executor` 同步用 pass graph resource metadata 选择 `MOTION_BLURRED`/`DEPTH_OF_FIELDED`/`TAA_OUTPUT`/`SCENE_COLOR`,避免执行阶段仍硬编码原始 scene-color;新增 compile/executor 路由单测覆盖 motion-blurred 输入与 fallback | `rustfmt --edition 2021` 通过 touched Rust 文件;`cargo check -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-bloom-route-check-0616` 通过(70 个既有 warning);按用户要求 focused tests 暂不运行 | 后续继续 PP-M3-S3:拆 `post.scene-composite` 承载 SSR/fog 或继续 CA/vignette/grain/dither 轻效果归位;补 `compile_routes_bloom_extract_after_split_scene_color_passes`、`bloom_executor_reads_motion_blurred_source_when_declared`、产品场景和 RenderDoc 验收 |
| 2026-06-16 | PP-M3-S3 scene-composite split | 部分完成: SSR/fog 合成已从 `post.uber` 拆到独立 WGPU fullscreen pass,但剩余轻效果与产品验收仍未收口 | `PostProcessEffectKind::SceneComposite` 接入链槽与 pass graph;新增 `postprocess.scene-composited` 中间资源和 `post.scene-composite` executor;pipeline compile 将 scene-composite 输入路由到 MotionBlur/DepthOfField/TAA/scene-color 最新源,并让 `post.uber` 优先读取 `SCENE_COMPOSITED` 且移除重复 SSR history 读依赖;WGPU 新增 `fs_scene_composite`、`scene_composite_pipeline.rs`、`execute_scene_composite` 与 graph recording;`post.uber` 在 scene-composite split 下清零内部 SSR/fog 参数并绑定 fallback SSR texture,避免重复合成 | touched-file `rustfmt --edition 2021` 通过;`cargo check -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-scene-composite-check-0616` 通过(70 个既有 warning);按用户要求 focused tests 暂不运行 | 继续 PP-M3-S3:确认/拆分 chromatic aberration、vignette、grain、dither、通用 blur 的最终归属;补 scene-composite graph/compile/executor/shader focused tests、产品场景与 RenderDoc 验收;继续 SMAA 产品验收/多阶段质量升级 |
| 2026-06-16 | PP-M3-S3 blur split | 部分完成: 通用 blur 已从 `post.uber` 拆为独立 WGPU fullscreen pass,但剩余轻效果与产品验收仍未收口 | `PostProcessEffectKind::Blur` 接入链槽与 pass graph;新增 `postprocess.blurred` 中间资源和 `post.blur` executor;pipeline compile 将 blur 输入路由到 `SCENE_COMPOSITED` 或 MotionBlur/DepthOfField/TAA/scene-color 最新源,并让 `post.uber` 优先读取 `BLURRED`;WGPU 新增 `fs_blur`、`blur_pipeline.rs`、`execute_blur` 与 graph recording;`execute_blur` 清零 DoF 参数并绑定 fallback DoF/bloom textures,`post.uber` 在 blur split 下清零通用 blur 半径,避免重复模糊 | touched-file `rustfmt --edition 2021` 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-blur-split-check-0616` 通过(既有 warning 仍存在);按用户要求 focused tests 暂不运行 | 继续 PP-M3-S3:确认/拆分 chromatic aberration、vignette、grain、dither 的最终归属;补 blur graph/compile/executor/shader focused tests、产品场景与 RenderDoc 验收;继续 SMAA 产品验收/多阶段质量升级 |
| 2026-06-16 | PP-M4-S1a terminal FXAA graph/writeback reconnect | 部分完成: FXAA 节点、transient terminal 输入与最终颜色写回已闭环,真实 FXAA shader 独立化和互斥诊断未完成 | `PostProcessStackDescriptor` 不再硬编码关闭 FXAA,`AntiAliasSettings::fxaa()` 会让 `OutputTransfer` 产出 `postprocess.terminal-aa-input` 并启用 `Fxaa`;`feature_descriptors/anti_alias.rs` 将 FXAA 输入改为 transient texture;pipeline compile 在后处理栈启用 FXAA 时保留 `AntiAlias` pass,并把 `post.output-transfer` 写目标改为 transient terminal 输入;`post.fxaa` executor 读取 terminal 输入并通过现有 fullscreen transfer path 写回外部 `FINAL_COLOR`;新增栈/编译/执行器路由单测 | `cargo fmt -p zircon_runtime` 通过;`cargo fmt -p zircon_runtime -- --check` 通过;`cargo test -p zircon_runtime --lib terminal -- --nocapture` 超过 120s 停在 shared lib-test 编译阶段,未返回测试结果;该超时后只停止匹配本次 `terminal` 过滤词的 cargo 进程,其余早先/其他会话进程保留 | 补跑 `fxaa_terminal_anti_alias_routes_output_transfer_through_terminal_input`、`compile_routes_output_transfer_through_fxaa_terminal_input` 与 output-transfer executor 路由测试;把真实 FXAA shader 从 `post.uber` 拆到 terminal pass;补 `AntiAliasSettings` 非法组合归一化/诊断和 RenderDoc/product scene 验收 |
| 2026-06-16 | PP-M4-S1b terminal FXAA shader executor | 部分完成: 真实 WGPU FXAA pass 已接入,但 PP-M4 的互斥诊断、SMAA terminal pass、动态分辨率 upscale 与产品场景/RenderDoc 验收仍未完成 | 新增 `fxaa.wgsl` 与 `FXAA_SHADER`;新增 `fxaa_pipeline.rs`、`ScenePostProcessResources::fxaa_pipeline`、`execute_fxaa` 和 `record_fxaa_to_resource`;`post.fxaa` executor 不再复用 output-transfer fallback,改为采样 `postprocess.terminal-aa-input` 后写 `FINAL_COLOR`;pipeline compile 将 `postprocess.terminal-aa-input` 固定为 `Rgba8UnormSrgb` 单采样 renderable/sampled transient;graph execution 在该资源为 transient 时覆盖预导入 final alias,避免 output-transfer 与 FXAA 读写同一最终目标;新增 shader parse/source contract/materialization 防线测试 | `cargo fmt -p zircon_runtime` 通过;`cargo fmt -p zircon_runtime -- --check` 通过;scoped `git diff --check` 无空白错误,仅 LF/CRLF 提示;`cargo test -p zircon_runtime --lib fxaa_shader -- --nocapture` 超过 184s 停在 shared lib-test 编译阶段,未返回测试结果,未发现本次 `fxaa_shader` 残留 cargo/rustc | 补跑 `fxaa_shader_parses`、`fxaa_shader_source_declares_terminal_input_and_neighbor_filter`、`materialization_overrides_preimported_terminal_aa_input_with_owned_transient`、FXAA graph/compile/executor 路由测试;继续补 AntiAlias 非法组合归一化/诊断、SMAA terminal pass、dynamic resolution/upscale、产品场景与 RenderDoc 截帧验收 |
| 2026-06-16 | PP-M4-S1c anti-alias normalization diagnostics | 部分完成: 非法组合/互斥诊断面已接入,但 SMAA terminal pass、dynamic resolution/upscale 与产品场景/RenderDoc 验收仍未完成 | `AntiAliasFallbackReport` 增 requested/effective graph sample count、`graph_sample_count_normalized`、`terminal_slot_normalized` 与 `taa_msaa_conflict_normalized`;`AntiAliasSettings::resolve_with_requested_graph_sample_count(...)` 接收相机 MSAA 请求;`build_frame_submission_context` 将 `extract.view.camera.msaa_samples` 传入解析;runtime diagnostics 增 `render.anti_alias.normalization.{active,count,graph_sample_count,taa_msaa_conflict,terminal_slot}`;契约测试覆盖 TAA+camera MSAA 与 unsupported SMAA terminal slot fallback | `cargo fmt -p zircon_runtime` 通过;`cargo fmt -p zircon_runtime -- --check` 通过;scoped `git diff --check` 无空白错误,仅 LF/CRLF 提示;`cargo test -p zircon_runtime --lib anti_alias::settings --locked --jobs 1 --message-format short --color never` 超过 184s 停在 shared lib-test 编译阶段,本轮启动的残留 cargo/rustc 已停止,无关 VM 后端测试保留 | 补跑 `taa_resolution_reports_camera_msaa_sample_count_normalization`、`unsupported_terminal_aa_reports_slot_normalization` 与 runtime diagnostics 聚合测试;继续实现 SMAA terminal pass、dynamic resolution/upscale、产品场景与 RenderDoc 截帧验收 |
| 2026-06-16 | PP-M4-S2a dynamic resolution upscale chain | 部分完成: render-scale 链尾 `post.upscale` 已闭环,产品场景/RenderDoc 与池统计切换验收仍未完成 | `build_frame_submission_context` 比较 `effective_render_size` 与 `effective_view_size`,当内部 render size 低于 viewport 时启用 `PostProcessEffectKind::Upscale`;`PostProcessStackDescriptor` 声明 `postprocess.upscaled` 并保证 `post.uber` 产出 `TONEMAPPED`、`post.upscale` 读取 `TONEMAPPED` 写 full-viewport `UPSCALED`、`post.output-transfer` 再读取 `UPSCALED`;pipeline compile 保留 `post.upscale`,将 `UPSCALED` 描述为 viewport 尺寸 `Rgba8Unorm` transient,并在 no-stack 路径剔除;新增 `upscale.wgsl`、filtered sampler bind group layout、`upscale_pipeline.rs`、`execute_upscale`、`record_upscale_to_resource` 与 `post.upscale` executor/registry 覆盖;动态分辨率编译测试断言 0.5 scale 下内部资源 160x120、upscaled 资源 320x240、output-transfer 读 `UPSCALED` | `cargo check -p zircon_runtime --lib` 通过;`cargo fmt -p zircon_runtime -- --check` 通过;scoped `git diff --check` 无空白错误,仅 LF/CRLF 提示;`rg` 在当前桌面环境被拒绝启动,改用 `Select-String` 源码扫描确认 stack/compile/shader/pipeline/executor 符号 | 补跑 `dynamic_resolution_declares_upscale_before_output_transfer`、`dynamic_resolution_scales_internal_graph_resources_without_resizing_viewport_output`、`upscale_shader_parses`、`upscale_shader_source_declares_filtered_source_sampling` 与 output-transfer upscaled 路由测试;继续 SMAA terminal pass、scale 0.5->1.0 池统计、产品场景抓帧和 RenderDoc 验收 |
| 2026-06-16 | PP-M4-S1d built-in SMAA terminal pass | 部分完成: 内建 `post.smaa` WGPU terminal pass 已闭环,但当前是单 pass 近似边缘滤波,产品场景/RenderDoc 与多阶段 SMAA 质量升级仍未完成 | 新增 `smaa.wgsl`、`SMAA_SHADER`、`smaa_pipeline.rs`、`execute_smaa`、`record_smaa_to_resource` 与 `post.smaa` executor;`feature_descriptors/anti_alias.rs` 同时声明 FXAA/SMAA 两个 terminal pass;pipeline compile 按有效 stack 在 FXAA/SMAA 间保留其一,不再要求外部插件 `post.smaa`;registry 覆盖 `post.smaa`;插件编译测试改为内建 SMAA 路由断言 | touched-file `rustfmt --edition 2021` 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-smaa-check-0616` 通过(既有 warnings 仍存在);按用户要求 focused tests 暂不运行 | 后续补 `smaa_shader_parses`、`compile_routes_output_transfer_through_smaa_terminal_input`、内建 SMAA 路由/registry focused tests、产品场景抓帧、RenderDoc 验收和多阶段 SMAA 质量升级。 |
| 2026-06-16 | PP-M4-S2b render-scale pool-stat validation | 完成: 0.5→1.0→0.5 动态分辨率池切换验收已自动化 | `TransientResourcePool` 新增 `render_post_dynamic_resolution_scale_swap_releases_pool` 回归测试;测试用真实 offscreen WGPU device 连续 materialize 半分辨率 `scene-color`、全分辨率 `scene-color`、再回半分辨率 graph frame,并通过 `RenderGraphExecutionResources::materialize_transient_resources_with_pool(...)`/`release_transient_backings_into_pool(...)` 走同一池路径;断言首个 0.5 frame 创建 1 个 texture、1.0 frame 再创建 1 个且池条目为 2、回到 0.5 frame 不再创建并复用半分辨率 backing,池条目仍为 2。 | touched-file `rustfmt --edition 2021 --check` 通过;`cargo check -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-dynres-pool-check-0616` 通过;首次 filtered test 编译超过 304s 未返回摘要但进程自然退出,复用目标目录后 `cargo test -q -p zircon_runtime --lib render_post_dynamic_resolution_scale_swap_releases_pool --target-dir D:\cargo-targets\zircon-runtime-dynres-pool-0616 -- --nocapture` 通过。 | PP-M4 剩余为产品场景抓帧、RenderDoc 验收;SMAA 多阶段质量升级已由 2026-06-17 PP-M4-S1e 行推进;PP-M3 仍需 user LUT readback 与 `execute_uber` 轻效果归属验收。 |
| 2026-06-17 | PP-M4-S1e SMAA multi-stage terminal pass | 部分完成: `post.smaa` 已从单 pass 近似边缘滤波升级为内部三阶段 WGPU 执行,产品场景/RenderDoc 与 focused tests 仍按用户要求后置 | 新增 SMAA 专用双纹理 bind group layout;`ScenePostProcessResources` 持有 edge/blend/resolve 三条 pipeline;`smaa_pipeline.rs` 以 `fs_edge`/`fs_blend`/`fs_resolve` 构建三段管线,edge/blend 阶段写 `Rgba8Unorm` 内部权重纹理,resolve 阶段写外部 `final-color`;`execute_smaa` 每帧创建 `zircon-smaa-edges` 与 `zircon-smaa-blend` 临时纹理并依次录制 `SmaaEdgePass`、`SmaaBlendPass`、`SmaaResolvePass`;`smaa.wgsl` 拆为边缘检测、权重计算、邻域混合三个 entry;`record_smaa_to_resource` 传入 viewport target size 供内部纹理分配 | touched-file `rustfmt --edition 2021` 已执行;`cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-smaa-multistage-0617` 通过,仅既有 warnings;按用户要求未运行 focused tests/product/RenderDoc | 补跑 `smaa_shader_parses`、`smaa_shader_source_declares_terminal_input_and_edge_filter`、`compile_routes_output_transfer_through_smaa_terminal_input`、内建 SMAA registry/graph focused tests;补产品场景抓帧与 RenderDoc 验收;继续 PP-M3 user LUT/非中性 tonemap-grading 与剩余 uber 轻效果归属验收。 |
| 2026-06-21 | PP-M3-S3 uber light-effects WGPU product guard | 部分完成: vignette/grain/dither/chromatic 最终帧产品守卫已自动化,但 RenderDoc 与更宽产品场景仍待 | 新增 `render_product_post_process.rs`,用真实 headless WGPU 提交同尺寸 baseline 与启用轻效果的 post-process extract;断言 `post.uber` 与 `post.output-transfer` executor 执行、effect-stack active families 覆盖 `vignette`/`film-grain`/`dither`/`chromatic-aberration` 且无缺失资源,并比较最终 `CapturedFrame` 左上角 luma 与 RGB 全帧 delta,证明轻效果已进入产品输出。`tests/mod.rs` 接入新模块,避免继续扩大已有超千行 anti-alias 产品测试文件。 | `rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_post_process.rs zircon_runtime/src/graphics/tests/mod.rs` 通过;`cargo test -p zircon_runtime --lib render_product_post_uber_light_effects_change_final_frame --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1,既有 warnings 保留。红跑曾暴露默认 `exposure-resolve` 会出现在执行节点里,最终断言改为按 graph contract/executor/final-frame product evidence 验收。 | PP-M3 仍需非中性 CPU reference/曝光 readback、DoF/motion blur/scene-composite/blur 更宽产品抓帧、RenderDoc 截帧;PP-M4 仍需 SMAA focused rerun和终端 AA/upscale 产品/RenderDoc 验收。 |
| 2026-06-21 | PP-M3-S3 motion blur split WGPU product guard | 部分完成: motion blur split 已有真实 WGPU 产品守卫,DoF/scene-composite/blur 更宽产品与 RenderDoc 仍待 | 新增 `render_product_post_motion_blur_split_uses_velocity_and_changes_final_frame`,用真实 headless WGPU 提交 baseline 与 motion-blur 双帧捕获;断言 `temporal.velocity-object`、`particle.velocity`、motion-vector tile/coarse/neighbor-max、`post.motion-blur`、`post.uber`、`post.output-transfer` executor 顺序和 active family;读取 `scene-velocity` 证明 motion vector 非零,并比较最终帧 RGB delta 证明 split motion blur 进入产品输出。红跑暴露 `post.motion-blur` 目标格式仍用最终 sRGB target,已把 DoF/motion-blur/blur/scene-composite split 输出管线改为 `POST_PROCESS_INTERMEDIATE_HDR_FORMAT`;同时 effect-stack resource status 改为扫描全 post-process graph node,避免 split motion blur 的 motion-vector 输入被误报缺失。 | `rustfmt --edition 2021 --check` 通过 touched Rust 文件;`cargo test -p zircon_runtime --lib effect_stack_resource_status_detects_split_motion_blur_node_motion_vectors --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1;`cargo test -p zircon_runtime --lib render_product_post_motion_blur_split_uses_velocity_and_changes_final_frame --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1;`cargo test -p zircon_runtime --lib render_product_post_ --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 16/16,既有 warnings 保留。 | PP-M3 仍需非中性 CPU reference/曝光 readback、DoF/scene-composite/blur 更宽产品抓帧、RenderDoc 截帧;PP-M4 仍需 SMAA focused rerun和终端 AA/upscale 产品/RenderDoc 验收。 |
| 2026-06-21 | PP-M3-S3 blur split WGPU product guard | 部分完成: blur split 已有真实 WGPU 产品守卫,但 DoF/scene-composite 更宽产品与 RenderDoc 仍待 | 新增 `render_product_post_blur_split_changes_final_frame`,用真实 headless WGPU baseline/blur 双帧捕获验证 `post.blur`、`post.uber`、`post.output-transfer` 执行顺序和 `blur` active family,并断言 `postprocess.blurred` 与 `postprocess.tonemapped` 为不同资源别名,再以最终帧 RGB delta 证明 split blur 进入产品输出。红跑暴露所有 split pass 与 uber 共用同一个 `post_process_params_buffer`:同一 command submission 内后续 uber 参数写入会覆盖 `post.blur` 已绑定参数,导致 blur 半径被归零;修复为 blur/motion-blur/DoF/scene-composite/SSR/uber 每次录制创建 pass-local post-process params uniform buffer,并移除过时的共享 params buffer 字段。 | `rustfmt --edition 2021 --check` 通过 touched Rust 文件;`cargo test -p zircon_runtime --lib compile_routes_blur_split_through_uber_and_output_transfer --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --color never -- --test-threads=1 --nocapture` 通过 1/1;`cargo test -p zircon_runtime --lib render_product_post_blur_split_changes_final_frame --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --color never -- --test-threads=1 --nocapture` 通过 1/1;`cargo test -p zircon_runtime --lib render_product_post_motion_blur_split_uses_velocity_and_changes_final_frame --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --color never -- --test-threads=1 --nocapture` 复跑通过 1/1;`cargo test -p zircon_runtime --lib effect_stack_resource_status_detects_split_motion_blur_node_motion_vectors --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --color never -- --test-threads=1 --nocapture` 复跑通过 1/1;`cargo test -p zircon_runtime --lib render_product_post_ --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --color never -- --test-threads=1 --nocapture` 通过 17/17,既有 warnings 保留。 | PP-M3 仍需非中性 CPU reference/曝光 readback、DoF/scene-composite 更宽产品抓帧、blur/DoF/scene-composite RenderDoc 截帧;PP-M4 仍需 SMAA focused rerun和终端 AA/upscale 产品/RenderDoc 验收。 |
| 2026-06-21 | PP-M3-S3 DoF split WGPU product guard | 部分完成: DoF split 已有真实 WGPU 产品守卫,但 scene-composite 更宽产品与 RenderDoc 仍待 | 新增 `render_product_post_depth_of_field_split_changes_final_frame`,单独放入 `render_product_post_process_depth_of_field.rs`,避免继续扩大既有 post-process 产品测试文件;用真实 headless WGPU 提交 baseline 与 DoF 双视口,验证 `post.depth-of-field-prepare`、`post.depth-of-field`、`post.uber`、`post.output-transfer` 顺序,确认 `depth-of-field` active family 无缺失资源,断言 `DEPTH_OF_FIELD_COC`/`DEPTH_OF_FIELD_BOKEH` alias 已存在且 `postprocess.depth-of-fielded` 与 `postprocess.tonemapped` 是不同 backing,并用最终帧 RGB delta 证明 split DoF 进入产品输出。 | `rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_post_process_depth_of_field.rs zircon_runtime/src/graphics/tests/mod.rs` 通过;首次 exact Cargo 调用在 604s 超时但后台编译自然完成,复用 `target\codex-runtime-postprocess-0621` 后 `cargo test -p zircon_runtime --lib render_product_post_depth_of_field_split_changes_final_frame --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --color never -- --test-threads=1 --nocapture` 通过 1/1;随后 `cargo test -p zircon_runtime --lib render_product_post_ --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --color never -- --test-threads=1 --nocapture` 通过 18/18,既有 warnings 保留。 | PP-M3 仍需非中性 CPU reference/曝光 readback、scene-composite 更宽产品抓帧、blur/DoF/scene-composite RenderDoc 截帧;PP-M4 仍需 SMAA focused rerun和终端 AA/upscale 产品/RenderDoc 验收。 |
| 2026-06-21 | PP-M3-S3 scene-composite split WGPU product guard | 完成: scene-composite fog 与 SSR wider 产品守卫已自动化,RenderDoc 仍待 | 新增 `render_product_post_scene_composite_fog_changes_final_frame` 与 `render_product_post_scene_composite_ssr_changes_final_frame`,单独放入 `render_product_post_process_scene_composite.rs`;fog 路径验证 `post.scene-composite`/`post.uber`/`post.output-transfer` 顺序、`fog` active family、独立 `SCENE_COMPOSITED` backing 与最终帧 luma/RGB delta。SSR 路径用真实材质/几何/灯光场景验证 reflection pyramid/coarse/specular occlusion/resolve、`post.scene-composite`、`post.uber`、`post.output-transfer` 执行顺序,断言 SSR pyramid/specular/history backing 存在、active family 为 `screen-space-reflection` 且无缺失资源,并通过最终帧区域/全帧 RGB delta 证明 SSR split 输出进入产品帧。红跑还修复两处生产合同:`product_postprocess_executor(...)` 按 graph resource kind/external binding type 区分 texture/buffer 校验;SSR reflection-pyramid/coarse descriptor 显式读取 `SCENE_DEPTH`,避免共用 post-process bind group 在 RawDepth 后端拿到错误 sample type。`execute_post_process_pass_graph(...)` 现在优先按真实 `executed_executor_ids` 反查 post-process node,资源模拟仅作旧路径兜底,避免 SSR split 实际执行后统计漏报 `scene-composite`。 | focused `cargo test -p zircon_runtime --lib render_product_post_scene_composite_ssr_changes_final_frame --locked --target-dir target\codex-runtime-postprocess-0621 -- --test-threads=1 --nocapture` 通过 1/1;`cargo test -p zircon_runtime --lib post_process_pass_graph_records_nodes_from_executed_executor_ids --locked --target-dir target\codex-runtime-postprocess-0621 -- --test-threads=1` 通过 1/1;broader `cargo test -p zircon_runtime --lib render_product_post_ --locked --target-dir target\codex-runtime-postprocess-0621 -- --test-threads=1 --nocapture` 通过 20/20;`cargo check -p zircon_runtime --lib --locked --target-dir target\codex-runtime-postprocess-0621` 通过;scoped `rustfmt`、`git diff --check` 与冲突标记扫描通过,仅既有 warnings/LF-CRLF 提示保留。 | PP-M3 剩余: blur/DoF/scene-composite RenderDoc 截帧;PP-M4 剩余: SMAA focused rerun、terminal AA/upscale 产品与 RenderDoc 验收。 |
| 2026-06-21 | PP-M3-S2 non-neutral CPU reference/exposure readback | 完成: 非中性 tonemap/color-grading LUT CPU reference 与曝光读回产品守卫闭环,但 RenderDoc 与 PP-M4 终端链仍待 | 新增 `RenderExposureReadbackReport` 与 test-only `read_buffer_f32x4`,将 `history.current.exposure` 真实 GPU buffer 读回到 `RenderStats.last_exposure_readback_report` 与诊断路径;`RenderColorLutReadbackReport` 增 `ColorTransform` reference,CPU 公式镜像 `color_lut_bake.wgsl` 的 ACES/Reinhard/Filmic/None tonemap、曝光 multiplier、saturation/contrast/gamma/tint grading;`render_product_post_non_neutral_tonemap_grading_changes_final_frame` 现在同时断言曝光 multiplier/EV100、`RenderColorLutReadbackReference::ColorTransform`、`color_transform_within_epsilon()`、LUT 非 identity 与最终帧 delta。红跑暴露历史曝光 buffer 实际 usage 只有 `STORAGE|COPY_DST`,已补 `COPY_SRC` 使图资源描述和真实持久 buffer 合同一致。 | `cargo check -p zircon_runtime --lib --locked --target-dir target\codex-runtime-postprocess-0621` 通过;focused filter `cargo test -p zircon_runtime --lib render_product_post_non_neutral_tonemap_grading_changes_final_frame --locked --target-dir target\codex-runtime-postprocess-0621 -- --test-threads=1 --nocapture` 通过 1/1;broader `cargo test -p zircon_runtime --lib render_product_post_ --locked --target-dir target\codex-runtime-postprocess-0621 -- --test-threads=1 --nocapture` 通过 19/19,既有 warnings 保留。 | PP-M3 剩余: blur/DoF/scene-composite RenderDoc 截帧;PP-M4 剩余: SMAA focused rerun、terminal AA/upscale 产品与 RenderDoc 验收。 |
| 2026-06-22 | PP-M4-S3 terminal AA/upscale WGPU product guards | 部分完成: FXAA/SMAA terminal AA 与 dynamic-resolution upscale 产品守卫已自动化,RenderDoc 仍待 | 新增 `render_product_post_process_terminal.rs` 并接入 `tests/mod.rs`;`render_product_post_terminal_fxaa_changes_final_frame_after_output_transfer` 用真实 headless WGPU baseline/FXAA 双视口验证 `post.output-transfer -> post.fxaa` terminal 链、`postprocess.terminal-aa-input` backing、particle executor 与最终帧 RGB delta;`render_product_post_dynamic_resolution_upscale_feeds_smaa_terminal_frame` 以 160x120 viewport/0.5 render-scale 验证内部 80x60 渲染、full-viewport `post.upscale -> post.output-transfer -> post.smaa` 链、`postprocess.upscaled`/terminal input backing、Auto->SMAA capability resolution、FXAA 未执行与最终帧可见。红跑暴露 `RenderStats.last_anti_alias_graph_executed_pass_count` 只统计 FXAA/TAA,已在 `base_stats.rs` 将 `post.smaa` 也纳入 terminal AA 执行计数。 | scoped `rustfmt --edition 2021` 通过 `render_product_post_process_terminal.rs`、`base_stats.rs`、`tests/mod.rs`;`cargo test -p zircon_runtime --lib render_product_post_process_terminal --locked` 通过 2/2;`cargo test -p zircon_runtime --lib smaa --locked` 通过 11/11;`cargo test -p zircon_runtime --lib dynamic_resolution --locked` 通过 8/8;`cargo test -p zircon_runtime --lib render_product_post_process --locked` 通过 22/22;`cargo check -p zircon_runtime --lib --locked` 通过,仅既有 warnings。 | PP-M4 剩余: terminal AA/upscale RenderDoc 证据;PP-M3 剩余: blur/DoF/scene-composite RenderDoc 截帧。 |

### 参考实现精读笔记

| 来源(真实符号) | 要点 | Zircon 对应物 | 取舍 |
|---|---|---|---|
| VolumeManager.cs:`Update(VolumeStack, Transform trigger, LayerMask)` | 先 `ReplaceData(stack)` 重置为默认值,再 `GrabVolumes(layerMask)` 取按 layer 缓存且排序的列表,逐 volume 叠加 | `VolumeEvaluator::evaluate` 步骤 1/2/4 | 不做 Unity 的 per-layer 缓存(volume 数 < 百级,每帧线性过滤足够) |
| VolumeManager.cs:局部体权重 `collider.ClosestPoint` → `closestDistanceSqr`、`blendDistSqr`、`interpFactor = 1f - (closestDistanceSqr / blendDistSqr)`、`OverrideData(stack, volume, interpFactor * Mathf.Clamp01(volume.weight))` | 距离平方域的 blend distance 权重;全局体直接 `Mathf.Clamp01(volume.weight)` | `VolumeShapeExtract::Box/Sphere` 最近点距离 + 同公式 | 不依赖物理 collider:形状自带解析最近点(box 局部空间 clamp、sphere 半径差),无物理模块耦合 |
| VolumeComponent.cs:`Override(VolumeComponent state, float interpFactor)` 逐参数 `stateParam.Interp(stateParam, toParam, interpFactor)`,带 overrideState | 参数级覆写开关 + 插值多态 | `VolumeComponentOverride.values: Vec<Option<VolumeParamValue>>` + `VolumeParamSchema.interp` 函数指针 | 用函数指针表替代虚调用/反射,契约层零动态派发 |
| VolumeStack.cs:`GetComponent<T>()`、`Reload(Type[] componentTypes)` | stack 是组件实例容器 | `ResolvedPostProcessStack` 为强类型 struct,`descriptor.apply` 写回 | 不做类型擦除容器:效果集编译期已知,插件走 sideband |
| UberPostProcessPass.cs:`RecordRenderGraph` 从 `volumeStack.GetComponent<ChromaticAberration/Vignette/FilmGrain>()` 取参,PassData 含 `ChromaticAberrationParams/VignetteParams/FilmGrainParams/DitheringParams`,材质槽 `_InternalLut/_UserLut/_Lut_Params/_Bloom_Texture/_Bloom_Params/_LensDirt_*` | 轻效果单 pass 合并;LUT 与 bloom 作为纹理输入进 uber | `UberParams` 六 vec4 + binding 表;bloom/LUT 同为 group1 输入 | lens dirt 不进 V1;用户 LUT 移到 bake 期叠加,uber 只采一次 3D LUT(URP 采内部+用户两张) |
| ColorGradingLutPass.cs:`lutWidth = lutHeight * lutHeight` 2D strip;`m_HdrLutFormat` 优选 `R16G16B16A16_SFloat` 降级 `B10G11R11_UFloatPack32`;tonemap 经 `ShaderKeywordStrings.TonemapNeutral/TonemapACES` 烘入;`lutParameters = (lutHeight, 0.5/lutWidth, 0.5/lutHeight, lutHeight/(lutHeight-1))` | LUT 预烘焙 + 半 texel 校正参数 | compute 烘 3D 纹理 32^3 rgba16float;`lut_params` 同构 | 弃 2D strip(wgpu 有真 3D 纹理采样);弃 B10G11R11 降级(LUT 很小,固定 rgba16float) |
| PostProcessing.cpp:`AddPostProcessingPasses` 用 `TOverridePassSequence<EPass>`,枚举序 `MotionBlur → PostProcessMaterialBeforeBloom → Tonemap → FXAA → … → PrimaryUpscale → SecondaryUpscale`;`DiaphragmDOF::AddPasses` 在 `FTranslucencyComposition`(ComposeToNewSceneColor)之前,TAA/TSR 经 `ITemporalUpscaler::AddPasses`;bloom 走 `AddGaussianBloomPasses` + downsample 链,`AddHistogramPass` → `AddHistogramEyeAdaptationPass`/`AddBasicEyeAdaptationPass` | 全链权威顺序:TAA/TSR → DoF/translucency → motion blur → bloom+exposure → tonemap → FXAA → upscale | `PostProcessChainSlot` 序即此序的简化(无 separate translucency 合成,DoF 置 TAA 后) | 不做 PassSequence 的 override-last-pass 机制(graph culling 已覆盖);不做 PostProcessMaterial 链(插件 RenderFeature 承担) |
| PostProcessEyeAdaptation.cpp:`GetEyeAdaptationParameters` 产出 `FEyeAdaptationParameters{ExposureLowPercent, ExposureHighPercent, MinAverageLuminance, ExposureSpeedUp, ExposureSpeedDown, HistogramScale = 1/HistogramLogDelta, HistogramBias = -HistogramLogMin * HistogramScale, ExposureCompensationSettings/Curve}`;`FEyeAdaptationCS` 写 `RWStructuredBuffer<float4> RWEyeAdaptationBuffer`,`View.SwapEyeAdaptationBuffers()` 双缓冲、RDG 注册 MultiFrame | 直方图 scale/bias 编码、百分位截断、上下行速度不对称、float4 结果 buffer 跨帧 | `RenderExposureSettings` 字段一一对应;`ExposureBuffer` 16 B 双缓冲持久资源 | 不做曝光补偿曲线资产(`ExposureCompensationCurve`)与 local exposure(双边网格),只留 `compensation_ev` 标量 |
| PostProcessHistogram.cpp:`LoopCountX/Y = 8`、`HistogramBucketsPerTexel = 4`、threadgroup `(8,4)`、`HISTOGRAM_SIZE` 宏 | 每 group 覆盖 64x32 texel 的分块直方图再归并 | workgroup 16x16 + workgroup 共享 bins + atomicAdd 归并,bins 固定 64 | 不做 per-group 中间纹理两段归约(WGSL workgroup 原子 + 半分辨率输入,单段够用) |

## 风险与回退

- Volume 求值进热路径:参数插值是 CPU 端小数据,按 descriptor 预编译插值函数表避免反射式逐字段分支。
- 效果迁移到 volume schema 是硬切换:迁移期以"全局容器默认值 = 旧 stack 值"保证产物不变,迁完删除旧 stack 输入。
- HDR 显示输出(HDR10/scRGB)只定接口(输出转换 pass 可插换),具体落地视 wgpu surface 能力另立切片。

## 2026-07-01 Runtime 15 Structure Mirror

Plan 07 post-process stack owner split is mirrored for the current structure guard: `render_plan07_post_process_stack_owner_split_static_passed`, `core/framework/render/post_process/graph_resource_names.rs`, `core/framework/render/post_process/stack/tests/effect_stack.rs`, and `runtime_15_post_process_stack_is_folder_backed`. The stack parent remains the descriptor construction owner, graph resource names stay in the dedicated resource-name owner, and stack behavior tests remain folder-backed under `core/framework/render/post_process/stack/tests/`.
