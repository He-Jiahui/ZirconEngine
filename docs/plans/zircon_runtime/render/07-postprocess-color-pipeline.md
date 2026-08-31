---
related_code:
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/view_family.rs
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
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
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
`DoF(primary, pre-reconstruction) → TAA/TSR(计划06, primary→secondary) → motion blur(post-reconstruction) → bloom(能量守恒 downsample 链) → exposure(histogram compute) → SSR/fog 合成类 → blur(高斯/局部模糊复用) → LUT 烘焙(grading+tonemap) → uber(LUT 采样+vignette+grain+dither+CA) → FXAA/SMAA → primary spatial upscale(primary→secondary,仅空间路径) → secondary spatial upscale(secondary→display) → 输出转换(sRGB/HDR10)`
- 色彩空间:全链 linear,中间格式 R11G11B10F(质量档可升 RGBA16F);LUT 32^3 起步;输出 pass 统一做 transfer function;断言性测试覆盖"中间 pass 不做 gamma"。
- 动态分辨率:render scale 进相机契约,scene 系 RT 按 scale 分配(计划 01 池按缩放尺寸键控)。`RenderViewFamilyPipeline` 是唯一尺寸权威:primary/secondary logical `ViewRect` 与各自 padded allocation 同时发布；primary/secondary spatial upscale 是不同图节点，不以单一 upscale 节点作双义解释。
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
2. render scale + direct spatial upscale MVP;RT 池按缩放尺寸协同(计划 01)。
3. UE primary/secondary spatial hard cut:将 direct `Upscale` 拆成两个不兼容的 graph node 与资源身份；TSR 仅负责 primary→secondary，secondary→display 始终由独立节点决定。

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

**动态分辨率**:`render_scale` 来自计划 09 `CameraRenderDescriptor`;当前 M4-S2 direct MVP 的 slot 1–9 瞬态 RT 按 `RenderViewFamilyPipeline` 的 primary logical rect 与 allocation extent 分配，`post.upscale` 做 primary→display。M4-S3 硬切后该单节点删除：`post.primary-upscale` 写 secondary target，`post.secondary-upscale` 写 display target；TSR 负责 primary→secondary 时不录制 primary node。`CompiledGraphCache` 键包含 logical rect、allocation extent、reconstruction kind 和两个 upscale phase mask，稳定尺寸/模式只复用已编译 artifact 与资源池，参数变化不触发 graph rebuild。任何 pass 都不得以 `ceil(viewport * render_scale)` 或 padding 自行重建 ViewRect。

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
- **与计划 06 衔接**:DoF 位于 `PreReconstructionScenePostProcess`,读取 primary `SceneColor` 并把专用输出交给 `TaaResolveExecutor`;TAA/TSR 完成 primary→secondary 后,Motion Blur 位于 `PostReconstructionScenePostProcess` 并消费重建结果。该顺序与 UE `PostProcessing.cpp` 的 BeforeDOF/DiaphragmDOF→temporal upscaler→MotionBlur 一致;TAA 关闭时后重建阶段直接消费 DoF 输出或原始 `SceneColor`,不保留 `TAA → DoF` 兼容路径。
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
| M4-S1 | `anti_alias.rs`(契约)、`feature_descriptors/anti_alias.rs` | 互斥矩阵进 `AntiAliasSettings` 校验;FXAA/SMAA 读取 `Tonemapped` 并写 transient `FinalComposited`;只有 OutputTransfer 写 external `FinalColor` | 非法组合归一化 + 诊断;终端 AA 编译路由与输出单写者正确 |
| M4-S2 | `dynamic_resolution.rs`(新)、`execute_upscale/`、`upscale.wgsl`、各 RT 描述符取缩放尺寸 | ViewFamily primary/secondary fraction 贯通; phase-specific spatial nodes cover primary→secondary and secondary→display, with temporal reconstruction replacing only the primary node;为 FSR1 插件保留位 | scale 0.5→1.0 切换池统计无泄漏 |
| M4-S3 | `view_family.rs`、`effect.rs`、`chain.rs`、`stack.rs`、`pass_graph.rs`、`graph_resource_names.rs`、`feature_descriptors/post_process.rs`、`execute_upscale/`、`upscale.wgsl`、frame submission/context | **Hard cut** `Upscale` 为 `PrimaryUpscale`/`SecondaryUpscale`，并分别引入 `PrimarySpatialUpscale`/`SecondarySpatialUpscale` phase、executor ID 与图资源。primary node 读 primary logical rect 写 secondary target；secondary node 读 secondary logical rect 写 display target；TSR 只取代 primary node。所有 allocation/scissor/history/cache key 消费同一 `RenderViewFamilyPipeline`，不得从 `render_scale` 重新推导。 | 静态 graph 能表达 spatial-only、TSR-only、TSR+secondary 和双 spatial 四种路径；同一 kind 不能再覆盖两节点；资源/presentation 均不把 alignment padding 当作 ViewRect；受管 product/RenderDoc 验收验证四路径。 |

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
- 2026-08-27 PFO-4d1q 源码切片已将 exposure adaptation 从固定 `1/60s` 改为由 runtime 唯一 outer-frame clock 经 `RenderFrameExtract::timing` 传入；synthetic/snapshot extract 明确使用 delta 0，frame timing 不进入 scene extract cache key。静态检查记录 fixed delta 0、tick capture 1、cache overlay 1、exposure consumer 1、cache timing mention 0；动态暗亮阶跃 PNG 序列、RenderDoc、profile 与功耗仍 pending，状态为 `source_implemented_static_checks_passed_dynamic_exposure_validation_pending`。
- 2026-08-27 PFO-4d1r 确认 motion-vector tile-max 的 fullscreen 参数来自唯一 immutable built-in plan 且不存在后续更新调用；参数 buffer 改为构造期 mapped initialization，删除 raw Queue、`COPY_DST` 与动态 write/layout cache。静态计数为 mapped init 1，Queue/write_buffer/COPY_DST/dynamic write 均 0；真实 WGPU 与 motion-blur 产品对拍仍 pending。
- 2026-08-29 阶段坐标与升采样切片：Pre/Post-Reconstruction、DisplayMapping、DisplayPostProcess、SpatialUpscale 的执行器统一使用 origin-zero local region，OutputTransfer 保留物理 ViewRect；新增持久 `UpscaleParams` uniform 和输入/输出逻辑尺寸采样，处理对齐 padding，避免按整张 texture 误采样/写入。SSR、DoF、MotionBlur、Bloom、Uber、FXAA/SMAA 的阶段尺寸与 Clustered Lighting/HZB/曝光路径已从旧 `target.size`/`effective_render_size` 迁移。`rustfmt --check`、`git diff --check`、`cargo metadata --locked --no-deps` 通过；受管 focused validation 在 `cargo.acquire` 超时，尚无动态编译、GPU、PNG、RenderDoc 或性能数据，状态为 `postprocess_phase_locality_and_upscale_source_implemented_static_checks_passed_dynamic_validation_pending`。
- 2026-08-29 阶段一致性复核：SSR 四个执行器的 params 与 dispatch 统一消费 PostReconstruction region；SSR history/pyramid/occlusion 资源 descriptor 同步归属该阶段，并新增 temporal secondary/half/coarse 尺寸断言。升采样 shader 修正 `@builtin(position)` 的像素中心到中心映射；OutputTransfer 改用 OutputTransform display region；通用 graph writer 仅将 `FINAL_COLOR/VIEWPORT_OUTPUT` 视为物理输出，避免中间 post-process attachment 误用 display viewport。静态格式/差异检查通过；最新受管验证返回 `cargo_reuse_pool_busy`，动态编译、GPU、PNG、RenderDoc 与性能数据仍 pending。
- 2026-08-29 M4-S3 前置结构复审：`PostProcessGraphResourceNames::view_family_pipeline_phase` 已成为 view-sized 内建纹理的唯一阶段 owner，resource descriptor allocation 与 GPU write-region 路由共同消费该表；DoF/TAA/PostReconstruction/DisplayMapping/DisplayPostProcess/SpatialUpscale/OutputTransform/Present 不再通过“未命中即 SceneLinear”推断。中间 backing 统一归零 origin，只有 `FINAL_COLOR/VIEWPORT_OUTPUT` 保留物理 ViewRect；固定尺寸 3D LUT/froxel 明确不进入 ViewFamily 尺寸表。对照 UE 5.5.4 `PostProcessing.cpp`：temporal upscaler 的 `OutputViewRect` 为 secondary，链尾 `PrimaryUpscale` 明确区分 `PrimaryToSecondary`/`PrimaryToOutput`，`SecondaryUpscale` 固定 `SecondaryToOutput`，且每段断言输出 ViewRect。因此下一硬切必须建立 `PrimarySpatialUpscale`/`SecondarySpatialUpscale`、两个 effect/slot/executor/resource identity，覆盖 spatial-only、temporal-only、temporal+secondary、dual-spatial 四路径；当前单 `SpatialUpscale/Upscale/UPSCALED` 仍未删除。精确 `rustfmt --check` 与 scoped `git diff --check` 已通过；受管动态编译仍受 Cargo pool 占用，GPU、PNG、RenderDoc、帧时与功耗证据继续 pending，状态为 `view_family_resource_phase_owner_implemented_dual_upscale_reference_review_complete_dynamic_validation_pending`。
- 2026-08-29 M4-S3 双级升采样源码切片：已硬切删除单 `SpatialUpscale` phase、`Upscale` effect/slot、`post.upscale` executor 与 `UPSCALED` 资源身份，建立 `PrimarySpatialUpscale`/`SecondarySpatialUpscale`、`PrimaryUpscale`/`SecondaryUpscale`、`post.primary-upscale`/`post.secondary-upscale`、`PRIMARY_UPSCALED`/`SECONDARY_UPSCALED` 全链路。`RenderViewFamilyPipeline` 现明确表达 primary-only、secondary-only、dual-spatial 与 temporal-plus-secondary 四路径；stack、descriptor filtering、graph validation、allocation、executor registry、GPU phase targets 与 OutputTransfer 路由共同消费该契约。GPU 审查发现延迟上传要求每个目标每帧单 CPU producer，双级 pass 因此使用两个持久 params uniform buffer，共享 pipeline/sampler/layout，避免两个 draw 都读取末次上传且不引入每帧 buffer 创建。二次稳定性审查按引擎 E10/F4 约束删除执行路径的 `panic!/expect`：非法 phase 与缺失 input target 现在返回实现标准 `Error` 的 `UpscaleExecutionError`，graph executor 在既有错误边界附带 pass identity，纯数据回归覆盖两个参数槽和两个 fail-closed 分支；相关生产段 `panic/expect/unwrap` 为 0。新增四路径、dual graph resource chain、phase allocation 与 executor routing 回归；源码旧身份扫描为 0，精确 `rustfmt --check` 与 `cargo metadata --locked --no-deps` 通过。初次受管 `spatial_upscale` 验证在 188 秒窗口无结果并超时；随后 focused `upscale_preparation_` 请求 `6d6bb68074fd4e3e8d6abaa787698b96` 被协调器接受但在 `cargo.acquire` 返回 `command_post_timeout`，仍未进入 Rust 编译。当前不能声明编译、产品、GPU、PNG、RenderDoc、性能或功耗通过；状态为 `m4_s3_dual_upscale_source_implemented_static_checks_passed_managed_dynamic_validation_pending`。
- 2026-08-29 M4-S3 graph execution 失败闭环：OutputTransfer、FXAA、SMAA 与 Bloom 的 4 个直接 phase `expect` 已统一经过 `require_post_process_render_region` 返回带 executor/pass/phase identity 的图执行错误；SceneLinear、DisplayMapping 与 allocation phase-target 的 3 个共享尺寸 `expect` 已改为 `Result`，14 个 GPU record 函数显式 `?` 传播，`record_post_process_stack` 原有两次同 phase 查询合并为一次。缺失 region 和 phase targets 的纯数据回归已写入，整个 post-process graph execution 非测试生产段的 `panic!/expect/unwrap` 计数从 7 收敛为 0；精确格式、旧签名守卫、diff 与 locked Cargo metadata 静态检查通过。该切片未重新请求已超时的 Cargo ownership，动态编译、WGPU、PNG、RenderDoc、帧时与功耗仍 pending，状态为 `postprocess_graph_phase_failure_source_implemented_dynamic_validation_pending`。
- 2026-08-30 SSR previous-history graph hard cut：SSR shader 已有 reprojection/history clamp/blend，但 resolve descriptor 原先没有 previous-history access，GPU executor 直接读取 renderer history owner。现在 previous SSR 以 View-sized `Rgba16Float` exact external read 进入 compiled access packet，binder 发布 physical texture/view/descriptor，执行只经 resolver 获取；cold-start 保留 optional fallback。源码合同、精确 rustfmt、locked metadata 与 scoped diff 通过；受管 Cargo/WGPU、连续帧 PNG/RDC、300 帧 validity 和性能/功耗未验证，状态为 `render_plan07_ssr_exact_history_source_implemented_dynamic_validation_pending`。
- 2026-08-30 SSAO descriptor single-owner hard cut：`rendering.ssao` 不再复制 runtime 的 compute pass、内建 WGSL、binding 与 dispatch；runtime built-in descriptor 经 graphics facade 成为唯一 owner，plugin 委托后重新获得此前副本遗漏的 `AMBIENT_OCCLUSION` `Rgba8Unorm` `SAMPLED | STORAGE` schema。当时建立的 previous AO exact lease 后续经 Runtime27 temporal 架构复审确认没有合格消费者，已由本节更新的 unqualified shared-history owner hard cut 取代；不得按此历史切片恢复旧物理lease。
- 2026-08-30 SSAO params / exposure fixed-buffer exact lease：SSAO 的 32-byte params ABI 由 runtime descriptor 唯一拥有，frame-scoped external buffer 声明携带 `UNIFORM | COPY_DST` schema、full range 与 compute uniform intent；post-process producer 与 compiled-scene binder 发布真实 physical descriptor。同步修正 exposure previous/current 虽有 16-byte compiled schema、binder 却清除 descriptor 的不一致，改为 physical buffer import。默认 pipeline 回归锁定 SSAO canonical `0..32` packet，history binder 锁定 exposure two-slot descriptor。静态格式、metadata、source/diff checks 通过；WGPU/画面/算法门开放，状态为 `render_plan07_fixed_external_buffer_physical_leases_source_implemented_dynamic_validation_pending`。
- 2026-08-30 Runtime27 M0 AO 产品隔离：`rendering.ssao` 的 manifest/runtime catalog 默认关闭；显式 SSAO 编译要求 depth、normal 与 HZB writer，Deferred 保留可资格化路径而 Forward+ 因没有 normal producer fail closed；`post.uber` 删除 AO read 与 `AO^2` 全场景乘法，旧“平均亮度下降”产品 oracle 改为默认请求无效时的帧输出不变。该切片未实现 GTAO、typed space/format/generation 资格、indirect-diffuse composition 或独立 specular-occlusion；精确格式/源码/diff/locked metadata 静态检查通过，受管 Cargo/WGPU/PNG/RDC/profile/power 仍 pending，状态为 `render_plan07_ssao_m0_product_containment_source_implemented_dynamic_validation_pending`。
- 2026-08-30 Runtime27 M1 AO canonical profile 源码续片：保留此前 typed depth/world-normal/HZB、projection/depth/render-rect/MSAA fail-closed 门禁；新增物理单位 `AoSourceSettings` 并贯通 camera、scene asset/TOML、Volume evaluation、frame extract、conditional graph cache key 与 final compile。输入 receipt 现持久化进 versioned `CompiledAoProfile`，由 compiled pipeline validation generation 同步标记 profile 和三类 input producer；新增受约束的 `AmbientOcclusionOutputs` 与包含 view/world-origin/pipeline/profile/render-rect/depth-normal-motion/output generation 的 `AoHistoryKey`。当前 output/history 合同尚无真实 GTAO producer/lighting consumer，旧 depth-edge WGSL 也未由 profile 驱动，temporal authoring 继续 fail closed；因此 M1、GTAO、denoise、composition 均未验收。AO 相关 36 个 Rust 文件精确 rustfmt、locked metadata 与最大 owner 778 行检查通过，无 Rust/WGPU/PNG/RDC/profile/power 验收，状态为 `render_plan07_ssao_m1_canonical_profile_source_in_progress_dynamic_validation_pending`。
- 2026-08-30 Runtime27 M1 profile authority/output receipt / M2 GTAO evaluate 源码续片：`CompiledAoProfile` 已成为 48-byte SSAO uniform 唯一来源，提供 allocation、quality-bounded work plan、米制 radius/thickness/bias/falloff、intensity、HZB mip cap 与 projected-radius cap；runtime generation/extent 不匹配 fail closed。compiled pipeline要求唯一未裁剪 AO writer并从实际 graph texture lifetime发布`AmbientOcclusionOutputs` producer/format/extent/rect/generation。evaluate descriptor删除previous-AO读取，generic compute删除按AO输出名推断history write的副作用；旧8-neighbor raw-device-depth darkener替换为unjittered inverse-projection world reconstruction、perspective/orthographic view vector、meters-per-pixel radius、footprint HZB mip与horizon-bitmask evaluate。当前默认full resolution，partial rect/half resolution在完整render-rect/downsample/spatial/bilateral链前拒绝。精确rustfmt、旧字段/常量/history source scan与locked metadata通过；受管Rust/WGPU/PNG/RDC/profile/power未验证，不计accepted，状态为`render_plan07_ssao_m1_profile_authority_output_receipt_m2_evaluate_source_implemented_dynamic_validation_pending`。
- 2026-08-30 Runtime27 M2 spatial / M3 indirect-diffuse 源码续片：evaluate改写transient raw AO，独立`ssao-spatial-denoise`通过明确graph version edge读取raw AO、qualified depth/world normal与48-byte params，以3x3 joint-bilateral写final AO；compiled output receipt指向spatial writer。显式AO profile只给唯一`deferred-lighting`增加final AO read，full binding数为29；shader只调制scene ambient/environment diffuse，direct lighting、environment specular、emissive与unlit不乘AO，无AO物理view时使用white neutral fallback。静态量化为2个AO compute pass、各5 bindings、spatial最多9邻域样本、Ultra最多54 HZB directional samples/pixel；8个续片Rust文件精确rustfmt、locked metadata、源码合同与scoped diff通过，最大owner 811行。受管Rust/Naga/WGPU、PNG/RDC、GPU profile/功耗未验证，不计accepted或性能改善；temporal、half-res bilateral、真实shader/OOM/device-loss last-good、GPU终态receipt与独立specular-occlusion仍开放，状态为`render_plan07_ssao_m1_profile_output_m2_gtao_spatial_m3_indirect_diffuse_source_implemented_dynamic_validation_pending`。
- 2026-08-30 Runtime27 AO逐帧command-record receipt / 通用Compute PSO last-good：`RenderAmbientOcclusionExecutionReport`现有16-bit failure flags，成功帧严格核对evaluate/spatial两个非零dispatch、raw AO write/read、final AO write与deferred-lighting read。Render Graph compute workload默认`Reject`；AO两pass分别以稳定family和shader interface generation 2显式选择last-good。通用cache分离bounded candidate cache与bounded family publication/LRU，只有Naga及WGPU validation error-scope成功的候选才发布；entry、workgroup、完整binding ABI、scene layout或Runtime09A device epoch不同均拒绝旧WGPU handle。generic executor把`Ready/UsingLastGood`、candidate/resolved artifact fingerprint、device identity与candidate failure写入实际dispatch receipt，AO要求两pass设备代次一致；报告透传到`RenderStats`和30个固定diagnostic paths。5个AO receipt行为测试、1个diagnostic path测试及通用policy/resolution/family ABI/WGPU fallback测试源码已补；20个相关Rust文件精确rustfmt、scoped source/diff与locked metadata通过，受管测试/WGPU/GPU终态/PNG/RDC/profile/power未验证，不计accepted或性能改善，状态为`render_plan07_ssao_compute_pipeline_last_good_source_implemented_dynamic_validation_pending`。

- 2026-08-30 Runtime27 half-resolution GTAO / bilateral upsample 源码续片：共享Render Graph texture schema新增以Render或View为基准的有理数相对尺寸与显式floor/ceil策略，拒绝零比例和`u32`溢出；AO profile ABI升级为64-byte并同时携work/full extent及resolution divisor。full路径保持evaluate→spatial，half路径按ceil(full/2)执行evaluate→spatial，再以full depth/world normal进行2x2 joint-bilateral upsample写唯一final AO，deferred-lighting只读取该final writer；work域shader坐标显式映射回full-domain depth/normal/HZB。三类AO compute family使用shader interface generation 3与兼容last-good，逐帧报告按full/half分别要求3/4个相关recorded pass，扩展为22-bit failure flags和38个固定diagnostic paths。当前仅完成源码、合同测试与静态复核；受管Rust/Naga/WGPU、PNG/RDC、GPU timestamp、transient-memory、p50/p95/p99、功耗及RenderDoc证据仍未验证，不计accepted或性能改善，状态为`render_plan07_ssao_half_resolution_bilateral_source_implemented_dynamic_validation_pending`。此前48-byte、两pass、16-bit/30-path与interface generation 2条目仅为历史切片，均由本条状态取代。
- 2026-08-30 Runtime09A scene submission terminal journal / Runtime27 AO completion correlation：`SceneRenderer`在正常渲染的单次frame-begin completion poll下新增有界scene ticket日志，容量直接取RHI unresolved-submission limit；pending ticket以复用scratch和一次批量status查询推进，空队列不取submission锁。公共`RenderSceneSubmissionCompletionReport`与当前`RenderAmbientOcclusionExecutionReport`分离，保留frame generation、ticket、poll receipt及真实terminal status，并公开pending/capacity/observed/terminal backlog计数，AO通过generation关联而不建立feature私有poller。审查发现的显式readback/capture额外poll已统一在每次推进后同步路由journal、IBL、typed query和timer；提交失败返回前发布最新completion stats，框架边界保留typed completion error。capture/readback职责迁入148行folder-backed owner，pipeline主文件由860降至724行。7个状态机测试以及frame/readback owner顺序、错误发布和11个固定diagnostic path测试源码已补；相关owner精确rustfmt、scoped diff/whitespace与locked metadata通过。受管Cargo、真实WGPU/device-loss、PNG/RDC/profile/锁竞争/功耗未验证，不计accepted milestone或性能改善，状态为`render_plan07_scene_submission_terminal_journal_source_implemented_dynamic_validation_pending`。
- 2026-08-30 Runtime27 AO descriptor test owner split：继续 temporal 算法前先按结构规范处理超过800行review门的`screen_space_ambient_occlusion.rs`；3个既有descriptor/topology/WGSL合同测试原样迁到folder-backed `screen_space_ambient_occlusion/tests.rs`，仅将3条`include_str!`改为迁移后的等价相对路径。生产owner由811行降至607行，测试owner为202行；两文件精确rustfmt、3个测试声明、3个shader路径存在性与scoped whitespace检查通过。该切片不改变graph、shader、pipeline ABI或算法，受管Rust/Naga/WGPU、PNG/RDC、profile与功耗均未验证，状态为`render_plan07_ssao_descriptor_test_owner_split_static_passed_dynamic_validation_pending`。
- 2026-08-30 Runtime27 AO unqualified shared-history owner hard cut：temporal重审确认`CompiledAoProfile`仍拒绝temporal，现有evaluate/spatial/upsample不读previous AO，旧共享`Rgba8Unorm`却仍无条件随scene history创建。已删除该texture/view/init attachment、history binder lease、帧尾copy/write-intent，并让spatial SSAO不再单独激活共享history；当前帧AO和indirect-diffuse消费不变。静态容量模型减少4 bytes/render-pixel的无消费者持久纹理，1080p约7.91 MiB、4K约31.64 MiB，但未实测显存/带宽/帧时/功耗。未来只能在motion/profile/key资格完成后建立独立`AoTemporalHistoryStore`，且需depth/normal/confidence与显式invalidate；静态rustfmt、locked metadata、负向源码扫描和owner行数门通过，动态证据仍pending，状态为`render_plan07_ao_unqualified_history_owner_hard_cut_static_passed_dynamic_validation_pending`。
- 2026-08-30 Scene history demand-compiled physical allocation：`SceneFrameHistoryRequirements`现在从compiled writer、temporal资格和froxel quality精确决定TAA、hybrid GI、SSR、HZB、exposure与volumetric具名owner；feature-off物理owner为0，空需求释放当前handle。仅exposure/volumetric不再随viewport resize重建，仅HZB不再触发无附件clear submission；binder、epilogue copy和HZB identity消费均显式处理optional lease。静态容量模型：TAA双缓冲与GI lighting+metadata各为16 bytes/history-pixel（1080p约31.64 MiB、4K约126.56 MiB），SSR为8 bytes/history-pixel（1080p约15.82 MiB、4K约63.28 MiB），固定froxel history约5.27/7.03/10.55 MiB。该切片尚未完成按domain增量reconcile，需求变化仍替换aggregate并扩大state/cache失效；动态Cargo/WGPU、PNG/RDC、profile与功耗均pending，状态为`render_plan07_scene_history_demand_compiled_allocation_source_implemented_static_checks_passed_dynamic_validation_pending`，不计accepted milestone或性能改善。
- 2026-08-30 Scene history per-domain reconcile：上一条遗留的整aggregate替换已由固定6域`SceneHistoryAllocationChanges`取代。history extent只重建TAA/GI/SSR，render extent只重建HZB，froxel quality只重建volumetric，exposure不随extent变化；toggle只创建/释放对应owner，未变化physical identity与domain state保留。frame transaction只对变化域标记`AllocationChanged`；TAA bind-group cache仅在TAA allocation变化时失效。稳定路径固定`O(6)`且无资源创建/clear command。精确rustfmt、source contracts、scoped diff、locked metadata和低于800行门通过；受管Cargo/WGPU、RDC/PNG、VRAM/GPU timestamp/功耗仍pending，状态为`render_plan07_scene_history_per_domain_reconcile_source_implemented_static_checks_passed_dynamic_validation_pending`，不计accepted milestone或性能改善。该切片当时遗留的pre-scene独立clear submit已由下一条scene-packet fusion源码切片取代。
- 2026-08-30 Scene-ticket history initialization fusion：history构造/reconcile只编码并返回可选`wgpu::CommandBuffer`；frame resource uploads进入既有pre-scene ledger后，terminal packet把clear放在scene command buffers索引0，与draw/diagnostic/surface tail共享scene ticket。旧`HistoryInitialization` producer已删除；scene接受前失败删除本帧history handle，`FrameFailedAfterSceneSubmission`保留已接受history，稳定帧不创建buffer也不走插入冷路径。该owner选择对齐UE RDG clear-pass进入统一graph/command stream，不复制Lumen compute复刻工程的逐pass submit。F16同步收敛为`render.rs`481、binding153、submit627及低于300行的具名child，既有严格预算未放宽。精确rustfmt、source contract、旧producer扫描、预算、scoped diff、尾随空白和locked metadata通过；受管Cargo/WGPU、native submit计数、PNG/RDC、VRAM/GPU timestamp/功耗仍pending。状态为`render_plan07_history_initialization_scene_packet_fusion_source_implemented_static_checks_passed_dynamic_validation_pending`，不计accepted milestone或性能改善。

## 性能审阅交接

- 2026-07-18 effect-stack统计性能交接：frame stats每帧为active/approximated/missing效果与资源物化多组`Vec<String>`，与sealed graph/executor状态重复；resource detection的3轮graph+5轮executor扫描已止损为各一轮。Render07联动Render17发布dense effect/resource bitset+counts并按generation共享，String只在UI/capture/log导出时生成；见PERF-MVP-361及`docs/plans/performance/01/2026-07-18-runtime-core-framework-render-post-process-effect-settings-static-review.md`。
- 2026-07-18 post stack性能交接：每帧descriptor构造、compile-options clone、graph validate及extract/context多owner深clone已回链。Render07联动Render01/17以camera post settings+history/AA+size/upscale+feature generation为key发布唯一compiled artifact，history变化只切variant或精确失效；见PERF-MVP-362。
- 2026-07-18 post Volume性能交接：per-call evaluator/registry重建与产品已排序输入重复sort已直接止损；同camera仍会被main submission及多个froxel/history消费者重复求值，scene extract仍每帧展开String/参数Vec且camera-loop深clone。PP-M2联动Runtime07/Render17改为scene generation维护的priority-ordered immutable compiled Volume set，每camera submission只发布一次resolved artifact，builtin component热路径使用dense identity；见PERF-MVP-363/364及Volume静态证据。
- 2026-07-18 post执行GPU对象交接：默认post pass每camera新建参数uniform buffer和29-entry bind group；probe三类count=0/full-capacity上传已直接改为只写active prefix。PP-M3应把binding 4切为persistent dynamic-uniform/ring，并以physical view/history/LUT/depth-mode generation缓存post binding bundle；warm stable buffer/bind-group create=0、bundle≤1/generation、params≤1 packed upload/camera frame。见PERF-MVP-369及`docs/plans/performance/01/2026-07-18-graphics-post-process-execute-static-review.md`。
- 2026-07-18 effect executor补充交接：PERF-MVP-369的参数ring/binding bundle须覆盖bloom/cluster/LUT/DoF/exposure/FXAA/MV/SSR/SMAA/SSAO/upscale全部ABI。SMAA中间纹理由Render01管理，disabled effect不再录制clear pass；color LUT将dynamic exposure移出generation bake，稳定grading下32³ bake≤1/relevant generation。见PERF-MVP-370及`docs/plans/performance/01/2026-07-18-graphics-post-effect-executors-static-review.md`。
- 2026-07-18 pipeline构造交接：9条split post entry的重复WGSL转换/shader module/相同layout已从9/9/9收敛为1/1/1；但`ScenePostProcessResources::new`仍同步创建约27条含大量optional effect的pipelines。Render07联动Render08按compiled post artifact需求queue/single-flight，F2必需加载期prewarm，optional首用不得阻塞frame thread；见PERF-MVP-371及construct静态证据。
- 2026-07-18 pass-graph记录交接：normal路径克隆全部executor ID并构建String tree已RED→GREEN改为单遍18-bit effect mask；fallback仍构建两份String tree，`record_post_process_graph`仍逐camera/frame深clone整图及node labels。PP-M2复用PERF-MVP-362的compiled artifact发布dense node/effect identity与executed bitset，stable generation不得重建字符串图；见PERF-MVP-372及`docs/plans/performance/01/2026-07-18-graphics-post-process-root-static-review.md`。
- 2026-07-18 history variant交接：compiled-scene原每frame无条件深clone`ViewportRenderFrame`已止损为history稳定时borrow；history unavailable/cut/resize仍clone完整frame并调用`without_history_resources`+`validated_graph`。PP-M2联动Render01/17预编译history-ready/historyless variants并只切dense handle，执行期graph build/clone=0；见PERF-MVP-374。
- 2026-08-29 history availability hard-cut：运行时不再通过`without_history_resources()`克隆并剥离已构造的post-process graph；`PostProcessStackDescriptor`直接以`history_available=false`构造cold-start图，TAA/history资源在descriptor阶段省略，motion blur/SSR等独立速度消费者仍按显式需求保留`SCENE_VELOCITY`。对应测试与结构吸收合同已迁移到`stack/tests/temporal_history.rs`，动态Cargo/WGPU验证仍待受管执行。
- 2026-07-18 post history资源交接：GI/metadata/SSR CPU整图初始化已改GPU clear，无消费者AO history已删除；2026-08-30源码已完成compiled requirements驱动的feature-off物理owner归零、extent依赖解耦和各domain独立reconcile，TAA cache失效也收窄到TAA allocation变化。PERF-MVP-395仍需把旧pre-scene独立clear submit迁入统一frame packet/completion-retirement owner，并用真实WGPU/VRAM/GPU timestamp证明stable graph bind无handle clone/创建及toggle只影响对应domain。
- 2026-07-18 volumetric resolved-settings交接：advanced-lighting三个froxel pass及shading apply仍可能为同一camera重复调用Volume evaluator。Render07须把PERF-MVP-363的per-camera resolved post artifact直接发布给`PreparedAdvancedLightingFrame`，executor只读typed volumetric settings，stable submission evaluator调用≤1且无String错误路径构造；见PERF-MVP-403。
- 2026-07-18 offscreen可选slot交接：Render07向Render01的`OffscreenResourceMask`声明bloom/AO/post中间纹理的真实compiled需求；effect-off时真实slot=0，单effect toggle只创建/销毁对应slot，render-scale变化只重建render-size资源，不让final output或其他history/advanced资源整包重建。见PERF-MVP-408。
- 2026-07-18 post descriptor compile交接：active resource set已从每filtered pass重建降为每descriptor一次；Render07继续把effect enabled、latest scene-color route和resource mask编译进PERF-MVP-422唯一post artifact，stable stack扫描/字符串物化=0，changed stack每generation构建≤1并由Render01复用resource analysis。
