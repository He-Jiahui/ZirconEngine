---
related_code:
  - zircon_runtime/src/core/framework/render/anti_alias
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs
  - zircon_runtime/src/core/framework/render/view_family.rs
  - zircon_runtime/src/core/framework/render/post_process
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/runtime/history
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/graphics/scene/scene_renderer/history
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal
  - zircon_runtime/src/graphics/shader/template/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_taa_reactive_mask.wgsl
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias
  - zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs
  - docs/zircon_runtime/core/framework/render/anti_alias.md
  - docs/zircon_runtime/graphics/scene/scene_renderer/temporal/taa.md
  - docs/tests/runtime/render
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09g1-volumetric-fog-froxel-review.md
  - docs/plans/optimize/zircon_runtime/09g2-advanced-surface-lighting-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VelocityRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalAA.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalSuperResolution.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/DynamicResolution.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/DynamicResolutionProxy.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/STP/STP.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/STP/StpPreTaa.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/STP/StpTaa.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Upscaling/DLSSIUpscaler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Upscaling/FSR2IUpscaler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/PostProcessing/Shaders/TemporalAntialiasing.hlsl
  - dev/bevy/crates/bevy_anti_alias/src/fxaa
  - dev/bevy/crates/bevy_anti_alias/src/smaa
  - dev/bevy/crates/bevy_anti_alias/src/taa
  - dev/bevy/crates/bevy_anti_alias/src/contrast_adaptive_sharpening
  - dev/godot/servers/rendering/renderer_rd/effects/taa.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/taa_resolve.glsl
  - dev/godot/servers/rendering/renderer_rd/effects/smaa.cpp
  - dev/godot/servers/rendering/renderer_rd/effects/fsr2.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/fsr2
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/fxaa.shader
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09H1 · Temporal AA、Velocity、History 与 Upscaling 工程化差距

## 1. 结论

Zircon 当前并非完全没有时域渲染基础。它已有 typed `AntiAliasSettings` 与结构化 fallback report，TAA 有真实 WGPU fullscreen resolve、Rgba16Float 双缓冲 history、success-only flip、YCoCg variance clip、closest-depth velocity、material reactive mask 和 bind-group identity cache；mesh 路径保留 previous transform、previous skinning palette 与 previous morph state，camera/object/particle velocity 也各有实际 GPU executor。新加入的 `RenderViewFamilyPipeline` 还表达了 display、primary、secondary、phase target、temporal history extent 和 GPU-time dynamic-resolution decision。这些 stage、数据结构和局部测试都值得迁移，不能在重构中退回“当前帧与上一帧简单 lerp”的临时实现。

但是标准产品链目前有三处互相叠加的致命矛盾。第一，`FrameHistoryValidationKey` 比较完整 camera、每个 mesh transform/material、全部 lighting、animation pose、post-process、particles 与 feature list。任何正常相机运动、物体运动、骨骼动画、粒子变化或灯光变化都会得到 `FrameInputsChanged`，进而把 `previous_history_available` 置 false，并使 TAA history invalid。也就是说，velocity 数据虽然被生成，TAA 在最需要它的连续运动帧却被强制当作 seed frame，时域积累只在输入逐字相等时成立。

第二，ViewFamily contract 在 `build_frame_submission_context` 中被计算并用于判断是否插入 spatial upscale、校验 post-process phase，却没有通过 `.with_view_family_pipeline(...)` 放入 `FrameSubmissionContext`。生产代码也没有调用 `view_family_pipeline()`。资源描述仍只按 `effective_render_size` 与 `effective_view_size` 二分：`TAA_OUTPUT` 是 render size，TAA 双缓冲 history 是 display size，executor 又把 `stack.target.size` 作为 viewport 传给 shader。动态分辨率下同一 TAA render pass 同时绑定不同尺寸的两个 color attachment，违反 WGPU attachment extent contract；即使后端容忍，shader 也没有 input/output ratio，只会把 display 坐标直接 `textureLoad` 到低分辨率输入，不构成 temporal upscaling。

第三，公开 AA 功能面与真实运行能力分裂。WGPU capability 硬编码 `supports_smaa = false`、`supports_cas = false`、`supports_dlss = false`、`max_supported_msaa_samples = 1`；`Auto` 永远优先选择 FXAA。与此同时，仓库已有三 pass 的 SMAA 命名实现和可令 graph texture 变成 4x 的 MSAA compile 路径，但 SMAA 没有 area/search LUT、pattern search、diagonal/corner detection，MSAA 没有任何 multisampled pipeline 或 resolve target。所有 scene pipeline 都使用 `MultisampleState::default()`，所有实际 `resolve_target` 都是 `None`。一旦只放开 capability gate，运行时不是获得 SMAA/MSAA，而是得到错误算法身份或 WGPU validation failure。

当前 TAA shader 是可工作的 baseline，不是 Unreal TAA/TSR、Unity STP、Godot FSR2 级 temporal reconstruction。它用 round + `textureLoad` 取单个 history texel，用“当前中心 depth 与当前 3x3 最近 depth 之差”冒充 disocclusion，没有 previous depth、previous velocity、normal/material/history moments、exposure correction、input/output transform、jitter delta、lock/status、sample count、thin geometry/flicker、translucency composition 或 sharpening。空间 upscale 则只有一次 bilinear `textureSampleLevel`。因此不能把当前 `Taa`、`Smaa`、`Msaa`、`Cas`、`Dlss` 枚举视为与参考引擎同名功能已经存在。

本轮登记 10 项 P0、26 项 P1、8 项 P2。重构顺序必须先修复 history validity 与 ViewFamily 唯一分辨率合同，再统一 velocity/camera-cut/reactive producer，随后升级 baseline TAA 与 temporal reconstruction/provider ABI；FXAA、SMAA、CAS、MSAA 必须按真实算法和资源合同分别实现；最后接入 GPU timing dynamic resolution、Editor/scalability/debug、artifact 和同场景竞争性 gate。没有这些前置条件，继续在现有 shader 上堆阈值会扩大错误合同。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | 文件 / 物理行 | 本轮判定 |
|---|---:|---|
| production focused set | 108 / 18,701 | E3：AA ABI、ViewFamily、history、submission、graph allocation、velocity/reactive producer、GPU executor、WGSL |
| production focused fingerprint | 108 / 18,701 | `7d2743c2adc128dc2c5d850cea728b8fe8250bb110d263bddc5b7c233cc5f292` |
| production 文件内 test 属性 | 149 | E2：数学、source contract、shader parse、history state、compile route |
| dedicated focused tests | 9 / 4,382 | E2：55 个 test 属性，其中 1 个 ignored artifact exporter |
| 名称命中的历史 artifact | 24 / 154,612 bytes | E1-E2：16 个 velocity 日志/图片、2 个 GI-MSAA、6 个 GI/volumetric temporal；直接 AA/upscale artifact 为 0 |
| Reference engine 主链 | Unreal 4 组、Unity 6 组、Bevy 4 组、Godot 4 组、Fyrox FXAA | E3：history、reprojection、AA algorithm、upscaler provider、DRS、debug 与质量档 |

focused fingerprint 按路径排序，对每个文件计算 SHA-256，再对 UTF-8 的 `path<TAB>hash<LF>` 清单计算 SHA-256。范围由 AA/history/temporal 命名目录与 `AntiAlias|Taa|FXAA|SMAA|Upscale|Temporal|Msaa` 符号追踪共同构成，排除 dedicated test owner，但保留 production 文件内 `#[cfg(test)]`。当前 33 个 focused production 文件存在其他 Session 的 modified/untracked 状态；其中 `view_family.rs` 与 `taa_resolve_bind_group_cache.rs` 尚未跟踪。结论绑定当前工作区精确快照，进入实现前必须重取 fingerprint 并逐项复核。

### 2.2 数据链读取深度

本轮从 camera/quality profile 的 requested AA 开始，追踪 capability resolution、effective mode、post-process stack、ViewFamily phase validation、compile options、graph resource descriptor、persistent history allocation、frame-history compatibility、previous camera rollover、mesh/skin/morph/particle previous state、velocity/reactive command selection、bind group、render pass attachment、WGSL sampling、history flip、statistics 和 artifact exporter。

shader 审查不只确认“有 velocity/history binding”。TAA 逐项核对 current/history coordinate domain、history filter、depth/velocity dilation、disocclusion source、color clipping、confidence、reactive response、exposure、jitter、dynamic-resolution transform 与 MRT extent；FXAA 核对 edge orientation/end search/subpixel；SMAA 核对 LUT/search/diagonal/corner/stencil/preset；upscale 核对 reconstruction filter、sharpness、ringing、viewport transform 与 quality tier。

### 2.3 当前工作区的并发修改边界

关键 modified 文件包括 submission `build.rs`、`frame_submission_context.rs`、post-process stack/pass graph、resource filtering、history texture owner、TAA executor/shader/store、mesh velocity/reactive pipeline、particle velocity builder 与 graph executor registry。两个关键文件是 untracked 状态。本文只做 review，不修改、整理或回退这些外部改动，也不把它们误认为已经接受的主干能力。

`source_recheck_required: true` 不是形式字段。尤其是 ViewFamily handoff、TAA attachment extent、particle velocity descriptor 与 history key，只要相关 Session 继续编辑，P0 的精确代码位置可能变化。实现 owner 必须先确认问题是否仍存在，再按本文 acceptance contract 修复，不能机械套 patch。

### 2.4 与相邻审查的 owner 边界

- 09A 拥有 render graph resource truth、transient alias、persistent GPU object、queue/fence、device loss 与 readback。09H1 定义 temporal resource 的 extent、format、generation 和 history semantic。
- 09B 拥有 ViewFamily、GPU Scene、draw command 与 visibility。09H1 要求其 previous transform/deformation 输出成为 velocity 唯一 producer，不在 TAA 内复制 scene state。
- 09C 拥有 material/shader ABI 与 PSO。WPO/deformation velocity、reactive/material classification 和 SMAA/MSAA permutations 必须复用 09C 的 versioned ABI。
- 09D 拥有 LUT、vendor model、derived artifact 与 resident generation。SMAA LUT、upscaler binary/model 和 calibration profile 由 09D 管理，09H1 只消费 ready generation。
- 09G1/09G2 拥有 volumetric 和透明表面。09H1 定义其 velocity/reactive/history input 与 composition phase，不复制透明 compositor。
- 09H2 将拥有 exposure、color grading、bloom、DOF、motion blur、SSR 与 terminal composition。09H1 先冻结共同的 resolution/history/velocity contract，09H2 不得建立第二套 temporal validity。

### 2.5 明确未做

本轮没有修改 production code，没有运行 Cargo、Editor、WGPU、RenderDoc 或参考引擎，没有重导出 artifact。没有执行 camera pan、moving/skinned/morph/particle、disocclusion、subpixel geometry、specular shimmer、HDR exposure jump、dynamic resolution、split viewport、ultrawide、4K/8K、stereo/XR、device loss、VRAM pressure、GPU timing 或同画质 benchmark。静态源码已足以证明当前 P0 合同冲突，但不能替代修复后的 GPU 产品验收。

## 3. 当前产品链与可保留基础

### 3.1 requested/effective AA 与 fallback report 可以保留

`AntiAliasMode`、`AntiAliasSettings`、`TaaQualityPreset` 和 `AntiAliasFallbackReport` 已把请求模式、实际模式、reason、requested/effective graph sample count 与 normalization 分开。这是正确方向。后续需要扩成 provider、phase、quality、resource cost 与 failure reason 的唯一状态，而不是删除结构化诊断退回 bool。

### 3.2 TAA 双缓冲 owner 与 success-only flip 可以保留

`TemporalHistoryStore` 用 key、两个 texture、read index 与 valid bit 管理 Rgba16Float history。执行成功后才 flip，invalidate 不破坏槽位。这比每帧 copy scene color 更合理。应升级为 typed temporal history set，增加 extent domain、generation、previous depth/velocity/moments/locks 和 per-history validity，而不是放弃 owner。

### 3.3 mesh deformation previous state 是真实工程基础

mesh command 已能基于 stable instance key 取得 previous transform，并保存 previous skinning palette、morph weights/source。velocity pass 使用 unjittered matrix，alpha-mask 有对应 shader template。这些是对标工程引擎不可缺少的数据入口。问题在于 coverage 与 history policy 没有让这些数据真正发挥作用，而不是完全缺少 previous state。

### 3.4 camera/object/particle velocity stage 边界清楚

camera fullscreen velocity、mesh object velocity、particle sprite velocity 已分别成为 executor。camera pass 对 missing previous/cut/invalid 有 structured status；particle pairing 能处理 stable sprite key、duplicate key 与 previous billboard basis。后续可在不改变上层意图的情况下迁移到统一 velocity contract。

### 3.5 reactive mask writer 与 bind-group cache 值得保留

TAA reactive mask 有 graph resource、clear、mesh writer、material-authored strength 与 transparent alpha path。新 bind-group cache 用 sampled texture identity 与 history current identity 避免稳定帧重复创建。这些不是最终 reactive/composition 系统，但可作为 producer registry 和 persistent descriptor cache 的起点。

### 3.6 ViewFamily 类型设计方向正确

`RenderViewFamilyPipeline` 已表达 display viewport、primary/secondary fraction、alignment、phase targets、temporal history extent、GPU sample scope 与 bounded square-root DRS controller。问题是 production handoff 与 resource materialization 没有完成。应把它 hard-cut 成唯一权威，而不是重新回到散落的 `target_size/render_size` 标量。

### 3.7 product stats 与 executor order 测试可迁移

现有测试会检查 effective AA、graph executor id、TAA resolve、reactive command、velocity readback、particle ordering、history bind-group reuse 与 sample-count normalization。这些适合作为 L1/L2 contract gate。修复时应保留并扩展到普通 scene、连续运动、动态分辨率和 artifact/perf，而不是只保留 pixel changed count。

## 4. 参考引擎对照

### 4.1 能力矩阵

| 域 | Zircon 当前 | Reference contract | 必须补齐 |
|---|---|---|---|
| history validity | 全 frame input 相等才 valid | camera cut、resize、format/provider change 等结构事件失效；正常 motion 靠 reprojection | per-history generation、结构事件 reset、正常 motion 保留 |
| TAA history | color + alpha confidence | color、depth/velocity、moments/lock/coverage/luma 等分层 history | typed history set 与独立 validity |
| disocclusion | 当前 depth 与当前邻域 depth 差 | previous depth/velocity/material/coverage reprojection | previous-surface consistency 与 thin/translucent policy |
| history sampling | round 后单 texel load | Catmull-Rom/bicubic/quality filter、history UV bounds | filter tier、input/output transform |
| temporal upscale | TAA 名义上选择 Temporal ViewFamily，但未消费 | primary-to-output reconstruction、history output space、reactive/exposure/sharpen | provider ABI、phase/resource contract |
| DRS | authored fixed scale + budget min；controller 类型未接线 | delayed GPU timing、hysteresis、scope/generation、fallback | timing owner、decision telemetry、history transition |
| FXAA | 5 tap edge blur | FXAA 3.11 edge orientation、endpoint search、subpixel | canonical implementation与preset |
| SMAA | 3 个自定义邻域 pass | area/search LUT、pattern/diagonal/corner search、stencil、preset | SMAA 1x identity 或改名 |
| MSAA | graph texture 可标多采样，pipeline/resolve 均单采样 | matching pipeline sample count、resolve、depth、postprocess policy | 真正 multisample render/resolve 或删除入口 |
| upscaler ecosystem | Spatial/Temporal 二元 enum；Cas/Dlss 只 fallback | STP/TSR + FSR2/DLSS provider、capability、quality、reset | provider registry、vendor/open implementation、fallback |

### 4.2 Unreal 提供目标上限

`TemporalAA.cpp` 显式处理 Catmull-Rom、history screen percentage、history pre-exposure correction、camera cut、velocity texture 与 history viewport transform。`TemporalSuperResolution.cpp` 不只是“高质量 TAA”：它有 history update quality、sample-count metadata、flicker/moire 分析、thin geometry、translucency rejection、history guide/coverage/uncertainty、resurrection 与可视化资源。`VelocityRendering.cpp` 将 velocity 视为 renderer-wide pass contract，`DynamicResolution.cpp`/proxy 则把 GPU timing 与 game/render-thread state 隔离。

Zircon 的超越目标不能靠复制 Unreal 的 CVar 数量完成，但至少要达到同等级的可观测 history model、输入覆盖、phase/resource consistency 和可复现 quality/perf gate。当前单 color history + eight constants 还不在同一工程层级。

### 4.3 Unity STP/HDRP 提供 provider 与 history 下限

STP configuration 同时接收 current、previous、previous-previous non-jittered matrices，input color/depth/motion、optional stencil、pre/post upscale size、frame timing 与 valid-history signal。其 history context 分配 `DepthMotion`、`Luma`、`Convergence`、`Feedback` 等多个 history type。Core RP 还以 `DLSSIUpscaler`、`FSR2IUpscaler` 暴露 provider boundary，HDRP 有独立 TAA shader/pass。

这说明工程级 temporal upscaler 需要稳定 provider ABI 与多资源 history，不应把 `AntiAliasMode::Dlss` 作为一个最终枚举分支硬编码在 core AA resolver 中。

### 4.4 Godot 提供轻量级 TAA/FSR2 产品下限

Godot TAA 保存 previous velocity，使用 9-tap Catmull-Rom history sampling，并按 current/previous velocity 差做 disocclusion。其 FSR2 路径包含 reconstruct previous depth、depth clip、lock、luma history、reactive/autoreactive、exposure 与 RCAS。即使不追求 Unreal TSR 的全部复杂度，这些也是 temporal reconstruction 的基础资源闭环。

### 4.5 Bevy/Fyrox 提供 Rust 边界与算法身份下限

Bevy TAA 要求 jitter、MipBias、depth prepass 与 motion-vector prepass，并提供 explicit reset；shader 至少使用 5-sample Catmull-Rom-style history sampling。Bevy SMAA 带 area/search KTX2 LUT、Low/Medium/High/Ultra preset、stencil 与真实三 pass。Fyrox FXAA 明确基于 NVIDIA FXAA 3.11，采样完整 3x3 邻域、判断 edge orientation、执行最多 12 级 endpoint search 并计算 subpixel offset。

Zircon 当前 FXAA/SMAA 与这些轻量参考仍有显著算法身份差异。因此在超过 Unreal 之前，必须先达到 Rust/轻量引擎同名功能的基本真实性。

## 5. P0 差距清单

### P0-1：全帧输入相等被错误用作 temporal history compatibility

`FrameHistoryValidationKey` 包含 world、完整 camera descriptor、mesh entity/transform/model/material/tint/mobility/layer、完整 lighting、animation poses、post-process、particles 与 effective features。`ViewportFrameHistory::incompatibility_reason` 对任何不等返回 `FrameInputsChanged`；`resolve_history_handle` 又要求 invalidation reason 为 `None` 才发布 `previous_history_available`。

这不是保守的 cut policy，而是取消正常时域运动。camera pan、dynamic object、skinning、morph、particle simulation、light animation、exposure/post-process mutation 每帧都可能使 history 无效。必须把“结构不兼容”与“场景内容变化”分开：正常内容变化依靠 motion/depth/reactive reprojection，只有 resize、format/provider、camera identity/cut、projection discontinuity、device generation 等事件 reset。

验收必须包含 300 帧连续 camera pan、moving rigid/skinned/morph/particle/light sequence，并证明 valid-history ratio、reprojection coverage 与 rejection reason，而不是只测静态相等帧。

2026-08-29 源码状态：P0-1 的全帧相等根因已完成 source hard cut。`FrameHistoryValidationKey`不再复制或比较mesh、animation、lighting、post-process、particle、camera transform和world generation；normal motion依赖velocity/depth/reactive/domain metadata。camera cut与velocity不再各自判断：`ViewportCameraSnapshot::supports_temporal_reprojection_from`成为submission history发布和camera velocity的共享合同。已有错误测试已反转，并补world identity、feature topology、projection kind与large camera cut回归。精确格式、源码守卫、diff和locked metadata通过；focused validator在外层等待244秒后无输出超时，没有request id或Cargo/rustc证据。尚无Cargo测试结果、300帧序列、GPU/RenderDoc/PNG、valid-history ratio或性能/功耗数据，故P0-1只记`source_implemented_dynamic_acceptance_pending`，P0-2及本计划整体保持pending。

### P0-2：单个全局 previous-history bit 误伤所有 temporal consumer

`prepare_history_textures` 以同一个 `previous_history_available` 决定 TAA、volumetric、exposure 等 history validity；resize/recreate 又在同一 `SceneFrameHistoryTextures` owner 中联动。HZB、SSR、SSAO、GI、exposure、volumetric、TAA 的失效原因和允许重投影条件不同，不能共享一个“整帧是否完全相同”的 bool。

2026-08-30 AO history 结构复核：SSAO 的 physical allocation 不再与 history validity 混为一谈。只要 SSAO active，`prepare_history_textures` 会在 pre-scene 阶段创建 SceneLinear primary/render-sized `Rgba8Unorm` AO history，初始化 clear 为 1.0；`SceneFrameHistoryTextures` 单独保存 AO `render_size`，避免 temporal/display history size 在动态分辨率下污染 AO domain。`SceneHistoryDomain::AmbientOcclusion` 只控制本帧是否允许采样，以及 AO 写入成功后是否复制/提交。descriptor 通过 Render-sized exact schema 与 compute sampled access packet 固定物理尺寸，binder 仅发布该 owner 的 texture/view/descriptor。该做法保留 domain-specific invalidation，同时避免为 cold-start 再常驻一张 1x1 或伪造 View descriptor；仍需受管 WGPU/PNG/RDC 与 profile 验证，不能据此宣称算法或性能完成。

必须建立 `TemporalHistoryDomain`/generation table：每个 history 声明 producer、extent space、format/schema、camera key、valid rect、reset reason、last successful frame 与 dependencies。一个 light parameter 改变不应自动删除 camera HZB；一个 exposure curve 改变也不应销毁几何 velocity history。跨 consumer 的共享只能发生在明确 versioned resource 上。

### P0-3：ViewFamily 被计算但没有进入生产提交上下文

`build.rs` 在构造 post-process stack 前生成 `view_family_pipeline`，用它判断 `SpatialUpscale` 并调用 `validate_stack_for_view_family`，但最终 `FrameSubmissionContext::new(...)` 链只设置 material mip bias、anisotropy 和 IBL reservation，没有调用 `.with_view_family_pipeline(view_family_pipeline)`。`FrameSubmissionContext::new` 将字段初始化为 `None`，getter 会 panic；production grep 没有 getter consumer，builder 只在 tests 中调用。

这使 ViewFamily 成为验证旁路，而不是资源/执行权威。必须 hard-cut：context 构造函数直接要求非 optional ViewFamily；graph compile、resource descriptor、history allocation、render region、post-process executor 与 presenter 全部只消费 phase target。删除独立推导 `target_size/render_size/effective_view_size` 的 fallback。

### P0-4：动态分辨率 TAA 绑定不同尺寸 MRT，且没有 temporal reconstruction

`resource_descriptors.rs` 只让 `UPSCALED`/`FINAL_COMPOSITED` 使用 display size，其余包括 `TAA_OUTPUT` 使用 render size。`TemporalHistoryStore` 却由 target `size` 分配 display-size 双缓冲。`record_taa_resolve_to_resources` 把 `stack.target.size` 传给 shader，并在一个 render pass 中同时绑定 render-size `taa_output_view` 与 display-size `taa_history_current_view`。

WGPU 要求同一 render pass color attachments 的 extent/sample count 兼容。动态 scale 小于 1 时该链应直接 validation failure。即使把 history 临时改成 render size，shader仍按 output coordinate `textureLoad` current color/depth/velocity，没有 primary-to-secondary transform、filter 或 reconstruction，所以不能称 temporal upscale。

必须由 ViewFamily 明确 TAA input/output/history allocation extent 与 viewport rect。TAAU/TSR provider 输出 secondary/display space，普通 TAA 输出 primary space；两者不能共用隐式 `viewport_size`。新增 0.5/0.67/0.77/1.0、奇数/对齐 extent、split viewport 产品 gate。

### P0-5：camera velocity 在 dynamic/sub-view 下混用 display 与 render 坐标域

`record_velocity_camera_to_resource` 也传 `stack.target.size`，而 scene depth/velocity 是 render-size graph texture。`velocity_camera.wgsl` 用这个 display viewport 除 fragment coordinate，导致低分辨率 target 只映射到 display UV 左上区域。`SceneUniform::from_frame` 的 current matrix 用 `render_region().local_size()`，`previous_motion_view_projection` 又以 `frame.viewport_size` 构建 previous matrix，current/previous matrix aspect/viewport contract不一致。

必须建立一个 motion transform uniform：current/previous jittered 与 unjittered clip transform、current/previous render viewport、display viewport、input/output scale、jitter delta、camera-relative origin delta。shader 不得从一个裸 `viewport_size` 猜坐标域。验收要覆盖 aligned allocation、non-zero viewport origin、asymmetric subrect 与动态 scale。

### P0-6：particle velocity 在默认产品图中不可达

renderer 已注册 `particle.velocity` executor，CPU/GPU 实现也存在；但 production `maybe_insert_core_scene_particle_descriptor` 只插入 `particle-render` color pass，没有 `particle-velocity`。带 velocity 的 descriptor 只在 `plugin_render_feature_fixtures.rs`，产品测试通过 `new_for_test_with_plugin_render_features(...)` 注入该 fixture。标准 runtime/plugin 没有对应 descriptor producer。

因此测试证明的是“手工注入的 pass 能运行”，不是普通项目粒子能写 velocity。默认路径必须由 particle render feature 根据 TAA/motion blur/provider requirements 声明 velocity/reactive outputs；plugin/cook/runtime 必须用同一 descriptor。没有 previous state 时也要有明确 first-frame policy 和 telemetry。

### P0-7：透明与动态内容的 velocity/reactive coverage 不完整

transparent mesh 不写 object velocity，只能得到 depth 背景的 camera velocity；particle 默认不写 velocity，也没有 particle reactive-mask pass。reactive resource 只收 mesh writer，空 mesh stream 则绑定 black。sprite、UI surface、half-res transparency、OIT、transmission、procedural/deformed material 均没有统一贡献合同。

必须由 Transparent Compositor 和 GPU Scene 发布 `VelocityContribution`、`ReactiveContribution`、`CompositionPhase`。按材质/primitive 选择 write depth、write velocity、reactive strength、translucency mask 与 exclude-from-TAA，禁止 TAA executor自行猜 producer。动态内容 acceptance 必须覆盖透明粒子、玻璃、屏幕材质、WPO、sprite、OIT 与 half-res。

### P0-8：MSAA graph surface 与实际 GPU pipeline/resolve 合同不一致

compiler 可以把 scene/gbuffer/depth graph texture 标成 sample count 4，compile test 也只断言 descriptor count；但 mesh/depth/gbuffer/deferred/velocity/particle/post-process pipeline 全部使用 `wgpu::MultisampleState::default()`，production `resolve_target: Some(...)` 搜索结果为 0。capability 硬编码 max 1 只是把错误路径挡住，不是实现。

在完整 multisampled attachment/pipeline/depth resolve/scene-color resolve/post-process/camera stacking/writeback policy完成前，必须保持 capability disabled，并禁止测试用 fake capability 把 descriptor-only 视为 MSAA 功能。实现后需在 pipeline key 中包含 sample count，明确 alpha-to-coverage、resolve precision、deferred/forward、velocity、depth sampling、UI 与 TAA互斥规则。

### P0-9：AA/upscaler requested、supported、active、provider 与算法身份没有唯一真值

SMAA shader与三 stage代码存在但 capability 永远 false；CAS/DLSS只有 enum/fallback；TAA capability 仅等于 `supports_offscreen`，没有验证 attachment format、filterability、sampled bindings、history memory、velocity/depth producer或 provider readiness。FXAA/SMAA 使用同名但非标准算法。diagnostics 只报告 effective enum，不能说明 provider、phase、quality、history state或失败边界。

必须建立 `AntiAliasRuntimePlan`/`UpscalerRuntimePlan`，至少输出 requested、eligible、supported、active、degraded、provider id/version、algorithm level、input/output extent、history generation、resource bytes、GPU timing与 reason code。同名算法必须通过 identity gate；实验性自定义 filter 应改名而不是借用 FXAA/SMAA。

### P0-10：没有能支撑“超过 Unreal”的质量、性能与回归证据

focused tests 有路径、stats 与局部像素断言，但 `docs/tests/runtime/render` 中直接 TAA/FXAA/SMAA/upscale artifact 为 0。ignored reactive-mask exporter 声明 `render18_taa_reactive_mask_wgpu_20260813.png`，仓库内并无该文件。已有 velocity 图片和 GI/volumetric temporal artifact不验证 AA reconstruction。

必须建立固定 scene corpus、camera path、reference image/sequence、GPU timing、VRAM/bandwidth、history/rejection heatmap与 capture manifest。每个 artifact 绑定 source fingerprint、GPU/driver/backend、provider/version、effective settings与 shader hashes。没有同场景可复现数字，任何“production-ready”或“超过 Unreal”都只是描述，不是验收结果。

## 6. P1 差距清单

### P1-1：TAA 没有 previous depth 或 surface history

history 只有 color/confidence。新显露区域、moving occluder、thin geometry、反射/透明变化无法验证 previous surface。至少增加 previous depth 或 depth-motion history，并定义 normal/material/coverage 可选 tier。

### P1-2：当前 `depth_delta` 不是 disocclusion test

shader 计算中心当前 depth 与当前 3x3 closest-depth 的绝对差。这只能检测当前邻域深度边界，不能判断 reprojected history 来自上一帧不同表面。应在 history coordinate 比较 previous depth/velocity/material consistency。

### P1-3：history sampling 使用 round + 单 texel load

subpixel reprojection 被量化为最近像素，运动时会抖动和模糊。至少提供 5/9 tap Catmull-Rom 或受控 bicubic tier，并正确处理 history valid rect、border 与 allocation padding。

### P1-4：uniform 缺少 input/output/history transform 与 jitter delta

`TaaResolveParams` 只有 viewport、8 个 quality constants 与 valid flag。没有 current input extent、output extent、history extent、viewport origin、UV min/max、current/previous jitter、pre-exposure ratio。必须改为 versioned reconstruction constants，禁止从 textureDimensions 隐式拼合同。

### P1-5：没有 history sample count、moments、lock/status 与 confidence model

alpha confidence 每 valid frame固定 +0.15，失效时设0.25，无法区分稳定纹理、thin geometry、flicker或重复 rejection。需要 history age/sample count、luma moments/variance、lock/coverage 或 provider等价 metadata。

### P1-6：quality preset 只是在同一 baseline 上切 8 个阈值

Low/Medium/High 没有改变 history filter、neighborhood kernel、metadata、sharpen、resolution strategy或cost。quality tier 必须绑定可测算法/cost差异，并输出 effective tier与降级原因。

### P1-7：camera cut 仅靠硬编码启发式

translation > far plane 20%、rotation > 60°、FOV > 15°、ortho 25%、clip 50% 才当 cut。缺少 authored cut、camera stack identity、teleport generation、world-origin shift、sequence discontinuity 与 provider reset。启发式可保留为 safety net，不能是唯一真值。

### P1-8：jitter 固定为 8 周期 Halton(2,3)

没有按 primary/output ratio、sample phase count、provider recommendation、projection type或quality选择 pattern，也没有 blue-noise/rotated grid tier。需要 provider-owned jitter contract，并记录 current/previous sample。

### P1-9：temporal frame index 未绑定实际 history write generation

viewport index 在成功 terminal submit 后推进，即使当前 effective AA 不是 TAA；history reset/cut 不重置 pattern，TAA pass是否真正写入也不参与 phase。应由 active temporal provider的 successful write generation驱动，non-temporal frame不消耗 temporal phase。

### P1-10：dynamic resolution controller 类型存在但没有 runtime owner

`RenderDynamicResolutionController/GpuSample/Decision/Scope` 除未跟踪的 `view_family.rs` 与 re-export/tests 外没有 production consumer。当前所谓 dynamic resolution 仍是 authored fixed scale 与 budget min。需要 timestamp query/readback、delayed sample queue、scope/generation validation、hysteresis、upper/lower bound与 unavailable/timed-out fallback。

### P1-11：render-size 变化只会整组 reallocate/reset

scale改变返回 `RenderSizeChanged` 并分配新 handle，没有 history resample、valid rect migration、phase-preserving transition或短期 last-good。DRS连续调节会反复失去 temporal benefit。需要 provider声明 resize tolerance、history output-space固定或 controlled migration。

### P1-12：static mobility 物体的违规运动没有防护

mesh velocity只对 dynamic mobility生成。若 static-marked entity transform变化，history key会使整帧失效，但没有 contract error、自动 promotion、diagnostic或 last-known velocity。应在 scene/GPU Scene 边界检测 mobility invariant，明确 reject/promotion策略。

### P1-13：透明 mesh 不写自身 object velocity

透明表面只保留 camera background velocity，导致玻璃、alpha blend、屏幕、透明角色随物体移动时 history reprojection错误。需要 per-material velocity mode、depth/coverage policy与透明 compositor集成。

### P1-14：particle velocity 每帧 CPU 展开并创建新 vertex buffer

matched sprite 被展开为 6 vertices，`record_velocity` 每帧 `create_buffer_init`。大规模粒子会产生 CPU、allocation与上传热点。应复用 GPU-resident particle state、indirect draw与ring/arena，不在 velocity stage复制整个 billboard几何。

### P1-15：particle reactive 与 translucency classification 缺失

粒子颜色变化可能由 animated texture、soft particle、additive emission 引起，单纯 velocity不能防 ghost。需要 particle/material输出 reactive/transparency/composition mask，并覆盖 additive、alpha blend、flipbook、soft depth intersection。

### P1-16：reactive mask 用 alpha 直接代表历史拒绝过于粗糙

transparent mesh writer取 `max(alpha, authored_strength)`。opacity、coverage、emissive变化与history rejection不是同一量；高 alpha 静态玻璃可能被过度拒绝，低 alpha快速发光可能拒绝不足。需要 reactive、transparency/composition mask分离和 material guidance。

### P1-17：procedural deformation/WPO velocity 没有统一 contract

当前 previous transform/skin/morph基础很好，但 material vertex deformation、procedural animation、cloth、water、foliage wind等没有 versioned previous-parameter输入。09C/09B必须提供 current/previous deformation ABI，09H1只消费结果。

### P1-18：缺少 camera-relative/LWC motion precision策略

camera matrices直接以 world transform构建，cut阈值也依赖 far plane。大型世界、origin rebasing与远距离物体会损失 previous/current精度。需要 camera-relative origin、origin delta与双精度/分块 world identity contract。

### P1-19：FXAA 实际是 5 tap 邻域模糊

shader只采 N/S/E/W，固定 luma threshold 0.03125，选择轴向邻居平均后混合。缺少完整3x3、edge endpoint search、subpixel aliasing、quality/console preset和HDR/tonemap placement。必须实现可识别的 FXAA版本或改名为 custom edge blur。

### P1-20：SMAA 实际是三 pass 自定义 edge blur

虽有 edge/blend/resolve三阶段，但没有 area/search LUT、horizontal/vertical pattern search、diagonal/corner detection、predication、stencil或preset；resolve仍是四邻域平均。它不能以 `Smaa` 名称对外。应接入 SMAA 1x reference contract或改名。

### P1-21：spatial upscale 只有 bilinear sample

`upscale.wgsl` 只执行一次 `textureSampleLevel`，没有 EASU/RCAS、CAS、Lanczos、ringing clamp、content adaptive kernel、alpha/premultiplied policy、viewport params或sharpness。至少提供 bilinear、quality spatial、sharpen三个明确 tier。

### P1-22：没有真实 CAS 产品路径

`AntiAliasMode::Cas` 被当作 AA mode并 fallback，实际 CAS 是 sharpening/upscale stage，可与 TAA/FXAA组合。应从 terminal AA enum中拆出 reconstruction/sharpen chain，避免互斥模型错误。

### P1-23：upscaler provider 不能扩展 FSR2/DLSS/XeSS/STP-like 实现

`RenderUpscalerKind` 只有 Spatial/Temporal，core resolver硬编码 `Dlss`。没有 provider registration、capability query、resource requirements、jitter、reset、quality、exposure/reactive inputs、frame generation或device loss callback。需建立 provider ABI并允许内置与插件实现竞争/选择。

### P1-24：部分 per-frame GPU object 创建仍在热路径

camera velocity每帧新建 bind group，spatial upscale也没有稳定 identity cache；particle velocity每帧建 buffer。TAA cache是正向样例，应统一到 persistent descriptor/buffer arena，并用 stats证明稳定帧创建数为0。

### P1-25：2D、overlay、UI surface、stereo/XR/multiview 没有明确 AA 合同

当前产品测试集中于默认3D offscreen。没有 Core2D/camera stacking、editor gizmo、world-space UI、overlay在AA前后、stereo eye history、multiview array、foveated/dynamic resolution集成。必须定义每个 layer/eye 的 history key与composition phase。

### P1-26：测试只证明静态/重新稳定后的收敛，不证明运动中的时域正确性

static TAA测试使用320x240空场景。dynamic occlusion测试通过添加一个mesh改变完整 validation key，transition帧history实际已失效；随后只比较三个完全相同的occluded frame。它没有验证moving object穿越、continuous camera pan、真实disocclusion、ghost trail或shimmer。fake SMAA/MSAA capability与test-only particle descriptor又绕过标准产品链。需要按第10节重建产品测试。

## 7. P2 差距清单

### P2-1：`Auto` 固定优先 FXAA，不考虑质量、motion 或 provider

Auto应由 profile/platform/content与有效provider选择，至少能区分 editor低延迟、cinematic TAAU、2D pixel-art、XR与fallback，而不是永远先FXAA。

### P2-2：只有 TAA 拥有 quality字段

FXAA sensitivity、SMAA preset、MSAA samples/policy、spatial filter、sharpness、temporal provider quality均缺少统一authoring schema。需要typed per-method settings与migration。

### P2-3：camera-cut/TAA 常量缺少可观测来源

多个magic threshold虽集中在文件顶部，但没有profile来源、runtime tuning、telemetry分布或自动calibration。应纳入quality contract并记录实际触发原因。

### P2-4：TAA bind-group cache 固定最多8项且没有预算信息

cache策略可保留，但需要命中/淘汰、bytes、generation、device reset与多viewport压力统计，避免隐藏 descriptor churn。

### P2-5：缺少 velocity/history/rejection debug views

现有stats以计数为主。需要 current/previous velocity、depth mismatch、reactive、history weight、confidence/lock、sample age、valid rect与provider phase可视化。

### P2-6：AA 文档与当前工作区状态漂移

`anti_alias.md`仍把SMAA描述为“named but unsupported”，当前工作区已有SMAA pipeline/shader；TAA文档记录旧target目录、旧时间点与尚未落库artifact。代码、capability与文档应由同一feature state表生成或至少在promotion gate中交叉校验。

### P2-7：`Dlss` 作为 core AA 枚举会固化vendor边界

应以 generic upscaler provider id + capability表示DLSS/FSR/XeSS/STP-like实现。vendor名称可出现在provider metadata和settings asset，不应迫使core enum每接一个vendor就扩展。

### P2-8：缺少统计显著性的画质与性能判定

单帧changed-pixel或一次submit CPU时间不足以比较AA。需要多帧序列的temporal error、flicker/ghost/shimmer metric、GPU timestamp分位数、warmup、置信区间与机器清单。

## 8. 目标架构

### 8.1 单一 ViewFamily resolution authority

每个 submitted view family 必须携带不可缺省的 `RenderViewFamilyPipeline`。每个 graph pass声明输入phase、输出phase、logical viewport与allocation extent。resource materializer只接受 phase target，不再读取camera上的scalar size。presenter、capture、history与readback均保留viewport origin/valid rect。

### 8.2 typed temporal history registry

建议最小结构包含：

- `HistoryDomainId`：TAA color、depth-motion、moments/lock、HZB、SSR、exposure、volumetric等；
- `HistorySchema`：format、extent space、mips/layers、clear value、version；
- `HistoryGeneration`：camera/view/provider/device/content schema generation；
- `HistoryValidity`：valid rect、last successful frame、reset reason、age；
- `HistoryTransitionPolicy`：reset、resample、retain output-space、last-good；
- `HistoryDiagnostics`：allocated bytes、rejection/coverage、producer/consumer、last error。

正常scene motion不是 generation change。只有 consumer声明的结构条件改变才失效。

### 8.3 unified motion contract

velocity使用一个versioned unit/space约定，例如 current UV minus previous UV，明确是否含jitter、render/output space、viewport origin、y direction、clamp和invalid sentinel。camera/object/skin/morph/WPO/particle/transparent producer都写同一contract，debug/readback用CPU oracle验证。

### 8.4 AA 与 reconstruction/sharpen 分层

建议把当前互斥enum拆成三层：

1. sampling AA：None/MSAA；
2. reconstruction AA/upscale：None/FXAA/SMAA/TAA/TAAU/provider；
3. sharpening/output refinement：None/CAS/RCAS/provider。

resolver根据pipeline、platform、history与provider生成一份runtime plan，明确每一层的phase与资源。DLSS/FSR/XeSS是provider，不是基础枚举分支。

### 8.5 producer-driven reactive/composition inputs

mesh、particle、sprite、transparent compositor、UI surface与custom pass通过registry贡献velocity/reactive/transparency/composition mask。TAA/provider声明必需/可选输入和fallback。缺少关键输入必须产生degraded reason，不得自动绑定black后仍报告exact support。

## 9. 依赖顺序重构里程碑

### M0：冻结feature truth与精确快照

- 将TAA/SMAA/MSAA/CAS/DLSS/DRS状态登记为experimental/unsupported/degraded；
- 记录本报告fingerprint和33个dirty source，完成owner复核；
- 验收：capability、docs、stats、plugin manifest不再给出互相冲突的状态。

### M1：拆分 temporal history domain 与 invalidation

- 删除full-frame equality作为TAA validity条件；
- 引入typed domain、generation、reset reason与success generation；
- 验收：camera/object/light/particle连续运动不reset TAA，resize/cut/provider change准确reset。

### M2：ViewFamily hard cut 成为唯一分辨率权威

- context构造必需ViewFamily，删除optional/builder过渡；
- graph compile/materialization/executor/history/present消费phase target；
- 验收：无production consumer直接二次推导target/render size。

### M3：修复 resource extent 与 viewport contract

- 为TAA、history、velocity、reactive、upscale声明input/output/history space；
- 支持allocation padding、valid rect、non-zero origin、odd size；
- 验收：0.5-1.0 scale、split viewport、ultrawide无WGPU validation error和坐标偏差。

### M4：统一 velocity、jitter 与 camera-cut ABI

- current/previous matrices、origin、viewport、scale、jitter delta进入versioned uniform；
- authored cut、teleport、camera identity与heuristic safety net统一；
- 验收：CPU oracle与GPU readback覆盖camera/object/skin/morph/WPO/particle/transparent。

### M5：补齐 production velocity/reactive producer

- particle velocity descriptor进入标准runtime/plugin；
- transparent/sprite/particle/compositor输出velocity/reactive/transparency；
- 移除test-only fixture作为产品前提；
- 验收：普通scene authoring/cook/runtime可触发全部producer。

### M6：建立正确的 baseline TAA

- previous depth/velocity或depth-motion history、Catmull-Rom、真实disocclusion、exposure/jitter transform；
- history moments/confidence/age与quality tier；
- 验收：continuous pan、occlusion reveal、thin geometry、skinned/morph/particle、HDR exposure sequence通过。

### M7：temporal reconstruction/upscaler provider ABI

- provider声明input/output/history extent、jitter、required resources、reset、quality、async能力；
- 先实现内置TAAU/STP-like baseline，再接FSR2；DLSS/XeSS按可用SDK插件化；
- 验收：provider切换/failure/device loss有结构化fallback，输出space正确。

### M8：真实 FXAA、SMAA 与 spatial sharpen

- FXAA按明确版本实现edge/end search/subpixel/preset；
- SMAA 1x接area/search LUT、stencil、diagonal/corner与quality；
- CAS/RCAS从AA mode拆为可组合stage；
- 验收：算法identity、LUT residency、2D/3D、HDR/SDR placement与artifact通过。

### M9：真实 MSAA render/resolve architecture

- sample count进入attachment、pipeline key、depth/color resolve和camera stack；
- 明确deferred/forward、alpha-to-coverage、velocity、post-process、UI与TAA交互；
- 验收：2x/4x/8x capability实测，所有pipeline/resolve sample count一致。

### M10：GPU-time dynamic resolution runtime owner

- timestamp、delayed readback、scope/generation、hysteresis、budget与decision telemetry接线；
- provider-aware primary/secondary policy与history transition；
- 验收：负载阶跃、timed-out/unavailable sample、multi-viewport、24h稳定性通过。

### M11：Editor、profile、serialization 与 migration

- camera/project/quality profile可author method/provider/quality/scale/sharpness；
- inspector显示requested/effective/provider/reset/degraded/resource/GPU cost；
- 验收：undo/redo、save/reload、cook、旧asset migration与runtime preview同真值。

### M12：debug、artifact、性能与自动回归

- velocity/history/rejection/lock/coverage/DRS heatmap；
- sequence metric、GPU timestamp、VRAM/bandwidth、capture manifest；
- 验收：CI GPU矩阵与本地可复现artifact绑定source fingerprint。

### M13：hard cut 与竞争性产品gate

- 删除full-frame history key、scalar size fallback、test-only particle velocity、custom-SMAA同名路径和descriptor-only MSAA；
- 同场景对照Unreal TSR/TAA、Unity STP/HDRP、Godot FSR2、Bevy/Fyrox轻量实现；
- 只有正确性、画质、GPU时间、VRAM、稳定性和独立review全部通过后才能promotion。

## 10. 验收矩阵

| 域 | 必测场景 | 正确性 gate | 性能/预算 gate | 证据 |
|---|---|---|---|---|
| history validity | static、camera pan、moving light/object/particle、cut、resize、provider switch | 正常motion不reset；结构事件reason精确 | history bytes、reset rate、valid ratio | event log + heatmap + unit oracle |
| camera velocity | pan/orbit/zoom/ortho、subrect、DRS、origin shift | GPU readback与CPU projected UV delta一致 | pass ms、bind creates=0 stable | vector view + readback |
| object velocity | rigid/static violation、skin、morph、WPO、LOD | current/previous identity与deformation正确 | command count、upload bytes、GPU ms | vector overlay + sequence |
| transparent/particle | alpha/additive、flipbook、glass、sprite、OIT、half-res | velocity/reactive/composition producer不缺失 | producer bytes/draws、overdraw | mask atlas + ghost metric |
| baseline TAA | subpixel lines、foliage、specular、occlusion、HDR exposure | 无明显ghost、shimmer、boil；disocclusion及时 | 1080p/4K GPU ms、history bandwidth | sequence metric + PNG/EXR + RDC |
| TAAU/provider | 0.5/0.67/0.77/1.0、odd/aligned、switch/failure | extent/viewport/history正确，fallback结构化 | quality tier GPU/VRAM曲线 | provider manifest + capture |
| FXAA | horizontal/vertical/diagonal/thin edge、HDR/SDR | 与选定FXAA版本oracle一致 | samples/pixel、GPU ms | edge chart + shader hash |
| SMAA | pattern/diagonal/corner、LUT missing、preset | area/search/stencil与preset正确 | pass ms、LUT bytes | three-stage debug + golden |
| MSAA | 2x/4x/8x、forward/deferred、alpha-to-coverage、UI | attachment/pipeline/resolve count一致 | bandwidth、VRAM、GPU ms | validation log + resolve capture |
| DRS | steady、load spike、oscillation、multi-view、timeout | bounded decision、scope/generation正确 | p50/p95/p99 frame time、scale stability | timing trace + decision log |
| Cross-feature | TAAU + GI + fog + transparent + DOF + motion blur + UI | phase/history/velocity不冲突 | graph peak bytes、async overlap、frame budget | full-chain RDC + sequence |

所有产品gate至少包含：feature-off exact baseline、registered-but-empty inert、active visible delta、missing-input degraded path、连续动态sequence、resize/quality/provider切换、device loss与当前source fingerprint。单帧 `changed_pixel_count > N`、pass name存在或compile descriptor sample count都不能独立证明产品正确。

“超过Unreal”需要预先冻结同画质场景、输出分辨率、帧率目标、硬件、driver、warmup与metric。只有Zircon在目标画质指标不低于基线时GPU/VRAM更优，或同预算画质显著更高，且结果可由第三方复现，才可声称某个具体场景/档位超过；不能从功能数量或单机一次测量外推整个引擎。

## 11. 现有测试与 artifact 判定

### 11.1 静态与动态 TAA 产品测试

`render_product_taa_static_empty_scene_history_stays_stable_after_seed_frame`只证明空场景在seed后两个相同帧byte-stable。它不证明AA效果、history贡献或moving reprojection，甚至完全关闭history也可能满足部分稳定性断言。

`render_product_taa_dynamic_occlusion_change_converges_after_history_seed`从无occluder切到新增static occluder。mesh list变化触发full-frame validation invalidation，第一occluded帧history weight为0；后续三个occluded extract完全相同。测试证明重新稳定后的重复帧变化减小，不证明transition时的disocclusion或运动中保留history。修复M1后该测试应增加validity/rejection断言，并用连续移动occluder而不是scene topology开关。

### 11.2 history bridge 测试固化了错误语义

原`render_framework_reports_frame_history_invalidation_when_camera_moves`将camera平移0.25后明确期望 `FrameInputsChanged` 与 `previous_available = false`，曾是P0-1的错误回归锁。2026-08-29源码已将其替换为`render_framework_keeps_frame_history_available_when_camera_moves`，并新增`render_framework_invalidates_frame_history_on_camera_cut`；动态执行证据仍待受管Cargo lane。

dynamic render-size test期望新history handle是合理的当前baseline，但DRS目标需要区分output-space history固定、provider migration与真正schema change，不能永远只测reallocation。

### 11.3 SMAA/MSAA 与 particle 测试绕过生产能力

SMAA测试手工构造 `supports_smaa: true`，实际capability永远false。MSAA compile test只确认graph descriptor count 4，不创建matching WGPU pipeline/resolve。particle velocity product通过test fixture注入带velocity descriptor，默认compile只插color pass。这三类测试应保留为局部component test，但名字/文档不得称product-ready；必须增加standard framework product route。

### 11.4 reactive mask 测试有价值但输出证据缺失

reactive测试覆盖zero/full authored strength、transparent alpha writer、bind-group history slot flip/reuse与stats，这是有价值的基础。唯一artifact exporter被 `#[ignore]`，声明的2026-08-13 PNG不在仓库。即使补出图片，也还需particle/transparency/composition mask与moving sequence，单张有几何的frame不能证明ghost rejection。

### 11.5 artifact 盘点

名称匹配的24个文件总计154,612 bytes：16个velocity相关文件主要是2026-07-03/04图片与command log；2个MSAA文件属于Hybrid GI current-frame post测试；6个temporal文件属于Hybrid GI/Volumetric。没有文件名含TAA、FXAA、SMAA、anti-alias或upscale的直接artifact。

因此现有artifact只能证明早期velocity/其他temporal consumer曾执行，不能验证当前108-file fingerprint下的AA/history/upscale行为，也不含GPU/driver/provider/quality/rejection数据。

### 11.6 本轮验证声明

本轮只做static review与文档校验，未运行Cargo/WGPU/Editor/RenderDoc。9个dedicated focused test文件共4,382行、55个test属性、1个ignored exporter；focused set没有发现以adapter初始化失败后静默return的测试分支，产品测试大多直接unwrap后端初始化。是否在当前机器通过仍未知，不能用源码存在替代执行结果。

## 12. 完成定义与退出条件

09H1只有同时满足以下条件才可从`pending`改为完成：

1. 正常camera/object/skin/morph/particle/light motion不再使TAA全局reset，所有history domain有独立generation/reason；
2. ViewFamily成为context、graph、resource、history、executor与present的唯一resolution authority，scalar fallback完成hard cut；
3. dynamic-resolution TAA/TAAU在所有scale/odd/subrect上attachment与坐标合同正确，无WGPU validation error；
4. camera/object/transparent/particle/procedural velocity和reactive producer在普通scene/cook/runtime产品链可达；
5. baseline TAA具备previous-surface disocclusion、subpixel history filter、exposure/jitter/extent transform与可观测metadata；
6. FXAA/SMAA/MSAA/CAS均以真实算法/phase存在，未实现项保持unsupported且不借同名custom filter；
7. temporal upscaler provider ABI至少接入一个内置高质量实现与一个开放provider路径，failure/device loss/切换有结构化fallback；
8. GPU-time DRS owner真正消费timestamp sample并发布decision/scale/history transition telemetry；
9. Editor/profile/serialization/migration/debug view形成闭环，2D/3D/stereo/XR/camera stack边界明确；
10. 当前源码自动通过第10节矩阵，artifact带fingerprint、GPU/driver、effective provider/settings、GPU timing与资源清单；
11. 与Unreal/Unity/Godot/Bevy/Fyrox同场景的画质/性能差异有可复现记录，未超过项继续作为公开gap；
12. 旧full-frame history equality、optional ViewFamily、test-only particle velocity、custom-SMAA同名路径和descriptor-only MSAA完成hard cut；
13. 独立code review、visual review与performance review均无Critical/Important遗留。

在这些退出条件之前，capability、文档、plugin manifest、release note或Editor不得使用“complete”“production-ready”“TSR-equivalent”或“超过Unreal”描述本组能力。

## 13. 2026-08-29 P0-2 current-source status

M1 的源码/静态部分已进一步收敛：`SceneFrameHistoryTextures` 现在由固定7域状态表拥有generation、valid、last-successful-frame与reset reason；copy只产生write intent，后端返回scene `SubmissionTicket`后才提交TAA/Exposure ping-pong与域状态。TAA、Hybrid GI、AO、SSR、HZB、Volumetric读取已按域分离，Exposure不再跟随camera cut全局reset；无历史帧继续执行seed pass，不再通过clone frame删除全部history资源。

本轮精确`rustfmt --check`、scoped `git diff --check`、`cargo metadata --locked --no-deps`与source guards通过；`copy_history_textures`生产路径的提前flip/set计数为0，whole-frame history strip/clone计数为0，volumetric duplicate-valid计数为0。受管`history_domains_commit`请求在47.5秒内无输出并超时，因此M1只能标记`source_static_complete_dynamic_pending`，不能满足第12节完成条件。per-domain运行时诊断、真实WGPU序列、RenderDoc/PNG、GPU timing、resident bytes、功耗与跨引擎对照仍是公开缺口。结构与测量计划见`docs/plans/performance/01/2026-08-29-temporal-history-domain-architecture-review.md`。

## 14. 2026-08-29 per-domain observability current-source status

M1的诊断基础设施源码已接通：`RenderHistoryDomainsReport`以core只读合同表达history owner是否存在，以及七域各自的valid、generation、last-successful-frame、active reset reason和frame reset reason；scene只在提交票据返回并完成history transaction后发布快照，`RenderGraphExecutionRecord -> RenderStats -> DiagnosticStore`不接触WGPU纹理所有权。每个submitted frame固定产生43个静态路径样本，不做运行时路径字符串构造或动态domain map；active reason表示提交后仍无效的原因，frame reason保留同帧reseed前的reset事件。原因码0表示无reset，1..7依次表示never-produced、previous-frame-unavailable、camera-cut、allocation-changed、feature-disabled、source-unavailable和structural-compatibility-changed。runtime/scene边界通过`RenderFrameHistoryInput`携带上游原因，P0-1相机连续性失败现明确映射为camera-cut，结构兼容键变化单独映射为structural-compatibility-changed。

13个精确Rust文件`rustfmt --check`、scoped `git diff --check`与`cargo metadata --locked --no-deps`通过；focused `history_domains_report`受管请求54.1秒无输出超时。该状态只能标记`observability_source_static_complete_dynamic_pending`：还未形成300帧valid-ratio/reset-rate记录，不能据此宣布结构性瓶颈已经消失，也没有GPU时间、resident bytes、RenderDoc、PNG或功耗证据。
