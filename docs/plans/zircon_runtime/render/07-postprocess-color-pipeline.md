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

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`07/2026-07-09-postprocess-color-pipeline-output-records.md`](07/2026-07-09-postprocess-color-pipeline-output-records.md)

## 性能审阅交接

- 2026-07-18 effect-stack统计性能交接：frame stats每帧为active/approximated/missing效果与资源物化多组`Vec<String>`，与sealed graph/executor状态重复；resource detection的3轮graph+5轮executor扫描已止损为各一轮。Render07联动Render17发布dense effect/resource bitset+counts并按generation共享，String只在UI/capture/log导出时生成；见PERF-MVP-361及`docs/plans/performance/01/2026-07-18-runtime-core-framework-render-post-process-effect-settings-static-review.md`。
- 2026-07-18 post stack性能交接：每帧descriptor构造、compile-options clone、graph validate及extract/context多owner深clone已回链。Render07联动Render01/17以camera post settings+history/AA+size/upscale+feature generation为key发布唯一compiled artifact，history变化只切variant或精确失效；见PERF-MVP-362。
- 2026-07-18 post Volume性能交接：per-call evaluator/registry重建与产品已排序输入重复sort已直接止损；同camera仍会被main submission及多个froxel/history消费者重复求值，scene extract仍每帧展开String/参数Vec且camera-loop深clone。PP-M2联动Runtime07/Render17改为scene generation维护的priority-ordered immutable compiled Volume set，每camera submission只发布一次resolved artifact，builtin component热路径使用dense identity；见PERF-MVP-363/364及Volume静态证据。
- 2026-07-18 post执行GPU对象交接：默认post pass每camera新建参数uniform buffer和29-entry bind group；probe三类count=0/full-capacity上传已直接改为只写active prefix。PP-M3应把binding 4切为persistent dynamic-uniform/ring，并以physical view/history/LUT/depth-mode generation缓存post binding bundle；warm stable buffer/bind-group create=0、bundle≤1/generation、params≤1 packed upload/camera frame。见PERF-MVP-369及`docs/plans/performance/01/2026-07-18-graphics-post-process-execute-static-review.md`。
- 2026-07-18 effect executor补充交接：PERF-MVP-369的参数ring/binding bundle须覆盖bloom/cluster/LUT/DoF/exposure/FXAA/MV/SSR/SMAA/SSAO/upscale全部ABI。SMAA中间纹理由Render01管理，disabled effect不再录制clear pass；color LUT将dynamic exposure移出generation bake，稳定grading下32³ bake≤1/relevant generation。见PERF-MVP-370及`docs/plans/performance/01/2026-07-18-graphics-post-effect-executors-static-review.md`。
- 2026-07-18 pipeline构造交接：9条split post entry的重复WGSL转换/shader module/相同layout已从9/9/9收敛为1/1/1；但`ScenePostProcessResources::new`仍同步创建约27条含大量optional effect的pipelines。Render07联动Render08按compiled post artifact需求queue/single-flight，F2必需加载期prewarm，optional首用不得阻塞frame thread；见PERF-MVP-371及construct静态证据。
- 2026-07-18 pass-graph记录交接：normal路径克隆全部executor ID并构建String tree已RED→GREEN改为单遍18-bit effect mask；fallback仍构建两份String tree，`record_post_process_graph`仍逐camera/frame深clone整图及node labels。PP-M2复用PERF-MVP-362的compiled artifact发布dense node/effect identity与executed bitset，stable generation不得重建字符串图；见PERF-MVP-372及`docs/plans/performance/01/2026-07-18-graphics-post-process-root-static-review.md`。
- 2026-07-18 history variant交接：compiled-scene原每frame无条件深clone`ViewportRenderFrame`已止损为history稳定时borrow；history unavailable/cut/resize仍clone完整frame并调用`without_history_resources`+`validated_graph`。PP-M2联动Render01/17预编译history-ready/historyless variants并只切dense handle，执行期graph build/clone=0；见PERF-MVP-374。
- 2026-07-18 post history资源交接：GI/metadata/AO/SSR CPU整图初始化已改两次GPU clear；PP-M2须让各history按compiled effect mask独立创建与resize，不因TAA/HZB/froxel变化重建，feature-off真实texture=0，stable graph bind不clone handles。见PERF-MVP-395。
- 2026-07-18 volumetric resolved-settings交接：advanced-lighting三个froxel pass及shading apply仍可能为同一camera重复调用Volume evaluator。Render07须把PERF-MVP-363的per-camera resolved post artifact直接发布给`PreparedAdvancedLightingFrame`，executor只读typed volumetric settings，stable submission evaluator调用≤1且无String错误路径构造；见PERF-MVP-403。
- 2026-07-18 offscreen可选slot交接：Render07向Render01的`OffscreenResourceMask`声明bloom/AO/post中间纹理的真实compiled需求；effect-off时真实slot=0，单effect toggle只创建/销毁对应slot，render-scale变化只重建render-size资源，不让final output或其他history/advanced资源整包重建。见PERF-MVP-408。
- 2026-07-18 post descriptor compile交接：active resource set已从每filtered pass重建降为每descriptor一次；Render07继续把effect enabled、latest scene-color route和resource mask编译进PERF-MVP-422唯一post artifact，stable stack扫描/字符串物化=0，changed stack每generation构建≤1并由Render01复用resource analysis。
