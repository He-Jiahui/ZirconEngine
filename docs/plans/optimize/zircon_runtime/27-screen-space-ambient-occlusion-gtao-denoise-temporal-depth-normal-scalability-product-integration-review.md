---
related_code:
  - zircon_plugins/rendering/features/ssao
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_runtime/src/core/framework/render/backend_types/quality.rs
  - zircon_runtime/src/core/framework/render/frame_profile.rs
  - zircon_runtime/src/core/framework/render/post_process/graph_resource_names.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/budget/degrade_ladder.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_history
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_standard_pbr.wgsl
tests:
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/compile_options.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/default_pipelines.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/plugin_features.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/project_render/render_quality.rs
  - zircon_runtime/src/graphics/tests/render_debugger_and_history.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/history.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
reference_engines:
  - dev/Fyrox/fyrox-impl/src/renderer/ssao/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/ssao/blur.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/ssao.shader
  - dev/bevy/crates/bevy_pbr/src/ssao/mod.rs
  - dev/bevy/crates/bevy_pbr/src/ssao/preprocess_depth.wgsl
  - dev/bevy/crates/bevy_pbr/src/ssao/spatial_denoise.wgsl
  - dev/bevy/crates/bevy_pbr/src/ssao/ssao_utils.wgsl
  - dev/bevy/crates/bevy_pbr/src/ssao/ssao.wgsl
  - dev/godot/servers/rendering/renderer_rd/effects/ss_effects.cpp
  - dev/godot/servers/rendering/renderer_rd/effects/ss_effects.h
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/ss_effects_downsample.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/ssao_blur.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/ssao_importance_map.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/ssao_interleave.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/ssao.glsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/ScreenSpaceAmbientOcclusion.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/HDRenderPipeline.AmbientOcclusion.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/GTAO.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/GTAOCommon.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/GTAOSpatialDenoise.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/GTAOTemporalDenoise.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/GTAOBlurAndUpsample.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/GTAOCopyHistory.compute
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/CompositionLighting/PostProcessAmbientOcclusion.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/CompositionLighting/PostProcessAmbientOcclusion.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/PostProcessAmbientOcclusion.usf
  - dev/UnrealEngine/Engine/Shaders/Private/PostProcessAmbientOcclusionCommon.ush
  - dev/LumenInUE5.5.4WithComputeShader/TemporalReprojection.cpp
  - dev/LumenInUE5.5.4WithComputeShader/Res/Shader/ScreenProbeGather/TemporalReprojection.hlsl
---

# 27 · Screen-Space Ambient Occlusion、GTAO、Denoise、Temporal、Depth/Normal Integration、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon 已有一条真实可执行的 SSAO compute 路径：plugin descriptor 能进入 Render Graph，typed 资源声明覆盖 current depth、world normal、furthest HZB 与 current AO，generic compute executor 会按像素 dispatch，offscreen target 也有对应纹理。这些基础应保留。2026-08-30 continuation 已从 evaluate descriptor 删除未资格的 previous AO read，并移除 generic executor 仅凭 AO 输出名称推断 history write 的副作用；temporal 必须等待 motion-qualified 独立合同。

但当前 shader 不是工程级 SSAO/GTAO。它没有重建 view-space position，也没有以投影、世界半径、厚度和衰减估算遮蔽，而是比较中心像素与 8 个相邻像素的 raw device depth，再按 world-space `normal.z` 固定压暗。它本质上是分辨率相关的 depth-edge/orientation darkener。更严重的是，默认 Forward+ 管线没有 normal buffer 写入者却让 SSAO 读取该纹理；最终 post process 又把 `AO^2` 乘到完整已照明 scene color，使直射光、镜面、发光与天空一起变暗。SSAO 还是默认启用 feature，现有产品测试只要求全图平均亮度下降，因此会把错误合成与无效 normal 输入当成成功。

本篇登记 **3 P0 / 48 P1 / 12 P2**。后续不能通过增加几个 sample 或调小常量继续修补；应硬切为 `CompiledAoProfile -> projection-aware depth/normal contract -> GTAO/VBAO evaluate -> edge-aware spatial denoise -> motion/depth/normal-qualified temporal -> bilateral upsample -> indirect-light/specular-occlusion consumers -> per-view history/evidence`。在三个 P0 关闭前，SSAO 不得默认启用或宣称 Complete。

## 2. 审查边界、方法与 currentness

### 2.1 冻结输入

本篇冻结 89 个输入、26,329 行、1,041,326 bytes：62 个 Zircon source/test/product 输入与 27 个参考实现输入。Zircon 输入分为 SSAO plugin/umbrella 10 文件、1,058 行、37,131 bytes，以及 Runtime core/pipeline/shader/test 52 文件、14,166 行、553,205 bytes；参考实现为 11,105 行、450,990 bytes。

组合指纹按相对路径排序，对每个文件计算 SHA-256，再对 `path<TAB>hash` 的 LF 拼接文本计算 SHA-256，结果为 `16c546c2df4ef356374b3e159885a6b7849b1ef9a784f4837cf11417d0c60108`。冻结时为 `main@25e09a23178000f2e783ce2143cf70a8b118d404`，62 个 Zircon 输入均无工作区修改。`dev/` 参考源码不纳入 Git dirty 判定。

### 2.2 纵向生产链

本轮逐层核对：rendering umbrella 默认 feature -> SSAO runtime descriptor -> plugin feature replacement/order -> pipeline compile/runtime flags -> depth/normal/HZB/current/history resource materialization -> params upload -> generic compute dispatch -> AO history copy -> post-process diffuse/specular consumers -> quality/degrade ladder -> Editor descriptor -> compile/fixture/history/product tests。

审查同时反向追踪 normal 的 producer：Deferred Geometry 使用标准 GBuffer encoder 写 world-space normal；默认 Forward+ 的 Mesh pass只写 depth/scene color，不写 `GBUFFER_NORMAL`，但资源绑定仍无条件暴露 retained `target.normal_view`。这不是单个 shader 局部缺陷，而是 pipeline/input qualification 缺失。

### 2.3 证据等级与未覆盖范围

本轮达到 E3 source-level review：Zircon production chain、失败/缺失分支、测试 oracle及五套参考实现均已交叉核对。没有运行 GPU 产品测试或修改 production；已知 workspace 动态验证阻断继续由既有报告拥有，本篇不重复制造测试结果。Ray-traced AO、distance-field AO、baked AO 与通用 contact shadow不在本篇 owner 内；它们只作为未来 typed AO source/fallback 的接口约束。

### 2.4 2026-08-30 M0 产品隔离进度

冻结审查之后已完成第一段 fail-closed 源码收敛，状态为 `ssao_m0_product_containment_source_implemented_static_checks_passed_dynamic_validation_pending`：

1. `rendering.ssao` 在 umbrella manifest 与 runtime optional-feature catalog 中统一改为默认关闭；默认 Forward+、Deferred 产品图均不再挂载 `ssao-evaluate`。
2. 显式请求 SSAO 时，pipeline compiler 现在要求 `SCENE_DEPTH`、`GBUFFER_NORMAL` 与 `HZB_FURTHEST` 在 `ssao-evaluate` 之前存在启用的写入 pass。Deferred 可通过当前 writer-order 门禁；没有 normal producer 的 Forward+ 会编译失败，而不是读取只因纹理对象存在就被 materialize 的 clear/stale normal。
3. `post.uber` 已移除 `AMBIENT_OCCLUSION` graph read，终端 shader 已移除 `AO^2` 对完整 lit scene color 的乘法。旧的 `ssao_quality_profile_darkens_scene_when_enabled` oracle 被 `ssao_product_default_is_fail_closed_and_preserves_scene_output` 取代，不再把全帧变暗当成 AO 正确性的证据。
4. 本切片只关闭默认暴露、writer availability/order 与错误终端合成。`SSAO-P0-001` 的 projection-aware GTAO/VBAO 算法仍开放；`SSAO-P0-002` 的 typed space/format/sample-count/valid-rect/generation 证明仍归 M1；`SSAO-P0-003` 的 indirect-diffuse 集成和独立 specular-occlusion 合同仍归 M3/P1，当前 SSR specular-occlusion consumer 未在本切片重构。

精确 `rustfmt --check`、源码禁用项扫描、行数预算与 locked Cargo metadata 是本切片可计数的静态证据。受管 validator 仍在进入 Cargo 前被 `cargo_reuse_target_mismatch` 阻断，因此不声明 Rust 编译、WGPU 产品帧、PNG、RenderDoc、GPU frame profile、功耗或性能改善；这些证据必须在通道恢复后写入 `docs/tests/runtime/render`。

### 2.5 2026-08-30 M1 profile、M2 spatial 与 M3 lighting source 续片

M0 之后继续完成了不依赖动态通道的 M1/M2/M3 源码切片，状态更新为 `ssao_m1_profile_output_receipt_m2_gtao_spatial_half_bilateral_m3_indirect_diffuse_source_implemented_static_checks_passed_dynamic_validation_pending`。这不是 accepted milestone；temporal、specular occlusion、WGPU 与产品证据仍开放：

1. `RenderFeatureQualitySettings::default()` 现在与 plugin manifest/runtime catalog 一致地默认关闭 SSAO；只有显式 quality profile 才能重新请求该 feature。
2. 编译器不再只检查同名 writer。显式 SSAO 现在构造并校验 typed qualification receipt，逐项记录 depth、world-normal 与 HZB 的 producer version，并证明 `Depth32Float` standard 0..1 device depth、`Rgba8Unorm` signed-unorm world normal、`Rgba16Float` max-reduction HZB、2D topology、sample count、SceneLinear allocation、HZB mip geometry 与 valid render rect。缺失或歧义 writer、格式/extent/mip/usage 不一致均 fail closed。
3. `GBUFFER_NORMAL` 的 graph physical format 已固定为和 geometry/offscreen owner 相同的 linear `Rgba8Unorm`，不再随 camera HDR 在 sRGB/float format 之间漂移；格式选择与 MSAA 策略分离，避免仅因显式格式就把 GBuffer attachment 私自改成单采样。
4. 当前 AO WGSL 声明单采样 depth/normal 且没有 viewport-origin 参数，因此 MSAA 输入、非零 SceneLinear viewport origin、partial SceneLinear render rect、非 0..1 viewport depth range和 custom/oblique projection 都会在 compile 阶段明确拒绝。它们必须分别由 resolve、render-rect ABI 与 projection reconstruction contract 关闭，不能近似运行。
5. evaluate descriptor 已删除 previous-AO binding/resource，generic compute executor 也不再仅凭 `AMBIENT_OCCLUSION` storage write 发布 history-written side effect。在 M3 接入 motion reprojection、depth/normal/disocclusion rejection 与独立 `AoHistoryKey` 前，temporal authoring 继续 fail closed，当前帧 evaluate 不读取任何旧 AO。
6. 新增 canonical `AoSourceSettings`，以米为单位定义 radius/thickness/depth bias/falloff，并携 intensity、Low/Medium/High/Ultra、half/full resolution 与 temporal authoring request。该值已贯通 camera component、project scene asset/TOML、post-process volume schema/evaluation、frame extract 与 final pipeline compile；`AoSourceSettingsKey` 用 float bits 和 schema version 形成稳定 cache/profile identity。未启用 SSAO 时 AO authoring 不进入 graph cache key，避免无功能变化的 graph cache miss。
7. typed qualification receipt 不再是 compiler 局部临时值。显式且输入合格的 SSAO 会生成持久 `CompiledAoProfile`，记录 artifact/compiler/shader-interface version、目标 `Gtao` method、source key、resolution divisor、projection/depth/render-rect/allocation、三类 typed producer descriptor，并在 `CompiledRenderPipeline::from_parts` 写入非零 validation generation；三个 input receipt 使用同一 generation，避免跨 compiled generation 复用。
8. 新增受约束的 `AmbientOcclusionOutputs` 与 `AoHistoryKey` 合同。输出只能由 typed `ambient-occlusion` texture producer、非零 extent/generation 和 extent内 valid rect构造；compiled pipeline现在要求 graph 中恰好一个未裁剪 AO writer，从其真实 texture lifetime发布 producer pass、format、extent、valid rect，并与 profile 共用 pipeline validation generation。history identity包含 view/world-origin、pipeline/profile、projection/depth、render rect/allocation、depth/normal/motion producer、output format/extent/generation，并只接受 typed `scene-velocity` producer。该 receipt证明compiled product identity，不伪装成逐帧GPU已完成标记。当前 temporal authoring仍在 profile compile阶段 fail closed，因为尚未有 motion/depth/normal逐像素拒绝。
9. 2026-08-30 continuation 已将 `CompiledAoProfile` 接成 shader params 的唯一 authority：64-byte ABI 分离 AO work extent 与 full input extent，并携 resolution divisor、quality-bounded `1x2/2x2/3x3/9x3` slice/sample plan、米制 radius/thickness/bias/falloff、intensity、HZB mip cap 与随工作分辨率缩放的 projected-radius cap；runtime extent/profile generation不匹配会 fail closed。旧 8-neighbor raw-device-depth shader 已替换为使用 unjittered inverse projection、perspective/orthographic view vector、world-position reconstruction、meters-per-work-pixel projected radius、footprint-driven HZB mip 与 horizon bitmask integration 的 GTAO/VBAO evaluate 源码。
10. Full resolution 保留 evaluate -> spatial 两个 compute，spatial 以3x3 joint-bilateral直接写 persistent final AO。Half resolution由 compiled profile选择 divisor 2；共享 `RenderTextureExtentPolicy::Relative` 以 Render reference、`1/2`、`Ceil`分配 raw/spatial中间纹理，保留奇数尺寸末行/末列。Evaluate与spatial每次depth/normal/HZB访问都从work coord显式映射回full SceneLinear coord，随后独立`ssao-bilateral-upsample`读取准确spatial producer version与full depth/normal，以2x2 bilinear + plane-distance + normal rejection写full final AO。compiled `AmbientOcclusionOutputs` 从实际final writer发布，因此full producer为spatial、half producer为upsample。最终 AO 仍只在显式 profile启用时成为唯一 `deferred-lighting` external read，并只调制scene ambient和environment diffuse；direct lighting、environment specular、emissive与unlit不乘AO。motion-qualified temporal和独立specular-occlusion仍开放。
11. `RenderAmbientOcclusionExecutionReport` 现按resolution divisor验证3-pass full或4-pass half chain：除evaluate/raw、spatial/raw与lighting/final边外，half还必须有spatial intermediate write、upsample intermediate read/final write及非零upsample dispatch。22个typed failure bits覆盖三条compute pipeline resolution与资源边，38个固定diagnostic paths记录三条candidate/resolved artifact fingerprint、dispatch/pass/access计数和device identity。通用compute workload仍默认`Reject`；evaluate、spatial、条件upsample分别以显式family和shader interface generation 3选择last-good，所有本帧AO compute必须处于同一Runtime09A device epoch。该receipt证明command recording、materialization前置验证与compiled generation闭合，不是GPU fence/timestamp completion；shader硬失败、OOM/device loss和GPU终态receipt仍开放。
12. Runtime09A新增bounded scene submission terminal journal后，AO不再需要feature私有poller。成功scene ticket在frame receipt完成后进入由RHI `max_unresolved_submissions`限制的日志；唯一frame-begin poll以一次批量status观察推进`Completed/Failed/Cancelled/DeviceLost`，并把最近终态及pending/capacity/observed/terminal backlog计数以独立`RenderSceneSubmissionCompletionReport`发布到`RenderStats`与11个固定diagnostic paths。AO command report与scene terminal report通过frame generation和submission ticket关联，当前frame graph record不会被异步结果覆盖。该源码切片关闭“如何表达GPU终态”的基础设施缺口，但尚未用真实WGPU证明AO pass成功、未把terminal status折叠进AO专用DTO，也不关闭OOM/device loss/视觉验收项。

本切片新增 source-key roundtrip/default、volume/asset roundtrip、camera/volume extract、cache participation、compiled profile/generation、invalid physical settings、unqualified temporal、history-generation、profile-to-ABI、runtime extent mismatch、partial rect、full/half descriptor topology、ceil-divided extent、evaluate-to-spatial-to-upsample version edge、final output producer、conditional lighting read、default no-AO graph、generic-executor no-implicit-history和AO command-record receipt回归源码。静态量化为full 2个、half 3个AO compute pass，各5个binding；spatial最多9个邻域样本，upsample最多4个depth/normal-aware样本，Ultra最多54个HZB directional samples/work-pixel，full deferred binding 29。AO failure/diagnostic静态计数为22/38；resource descriptor主owner拆为925行，relative schema allocation由146行子owner持有。精确`rustfmt --check`、旧参数/history/错误full-color与direct/specular AO乘法源码扫描、scoped diff check及locked metadata属于本轮静态门；受管 validator 的既知 `cargo_reuse_target_mismatch` 未重复触发。新增测试未执行，没有受管 Rust/Naga/WGPU shader creation、PNG、RDC、GPU profile、功耗或性能数据，不生成纯文本伪截图，也不把本状态计为 accepted milestone。evaluate中的精确`acos`以及half/full实际break-even必须等逐pass GPU profile、带宽计数与解析画质误差语料后决策；当前不声明性能改善。

Half-resolution结构审查先于实现完成：Unity HDRP以`ceil(full * 0.5)`保留奇数边界，half evaluate显式把normal coord乘2，并由独立`GTAOBlurAndUpsample`以full depth执行bilateral upsample；Unreal的SSAO shader同样把downsampled AO与upsample filter作为独立阶段。Zircon据此拒绝“只把raw纹理减半”的方案，选择profile驱动三阶段图与通用relative extent契约。动态恢复后必须分别采集full/half evaluate、spatial、upsample GPU timestamp、transient bytes、dispatch groups、PSO warm/cold、p50/p95/p99和功耗，并以thin-occluder、depth discontinuity、odd extent、camera motion场景比较AO误差；只有总成本下降且漏边/halo容差通过，half模式才可计性能或产品accepted。

### 2.6 2026-08-30 AO temporal 架构重审与旧 history owner hard cut

继续 temporal 之前重新纵向检查了 `CompiledAoProfile`、AO graph、`SceneFrameHistoryTextures`、history binder/epilogue、Zircon motion-vector 语义及参考实现。结论是原共享 history 中无条件创建的 render-sized `Rgba8Unorm` AO 纹理不是可继续扩展的 temporal 基础：profile compiler 当前明确拒绝 temporal，evaluate/spatial/upsample graph 不读取 previous AO，generic compute 也已删除按输出名隐式发布 history write 的副作用。因此该纹理没有合格消费者，却仍随共享 history 分配、初始化、报告绑定资格并保留帧尾 copy 编码路径，混淆了 spatial AO 与 temporal history 的所有权。

本切片执行 hard cut，状态为 `ssao_unqualified_history_owner_hard_cut_static_passed_dynamic_validation_pending`：

1. `SceneFrameHistoryTextures` 删除 AO texture/view/descriptor 与 `zircon-history-ambient-occlusion` 初始化 attachment；当前帧 `OffscreenTarget::ambient_occlusion`、GTAO evaluate/spatial/half bilateral和deferred indirect-diffuse consumer保持不变。
2. shared history binder 不再发布 `HISTORY_PREVIOUS_AMBIENT_OCCLUSION` physical lease；history epilogue不再把current AO复制到共享 history，也不再把spatial AO记录为`SceneHistoryDomain::AmbientOcclusion`写入。公共domain/resource identity暂保留为未来显式迁移合同，但生产路径没有物理owner。
3. `ssao_enabled` 不再单独触发整包`SceneFrameHistoryTextures`创建；AO domain在共享transaction中固定为`FeatureDisabled`，直到独立temporal owner落地。删除的单纹理容量模型为4 bytes/pixel，即1920x1080约7.91 MiB、3840x2160约31.64 MiB；这是静态分配上限差额，不是实测显存、带宽、帧时或功耗改善。
4. Unity HDRP `GTAOTemporalDenoise.compute`以motion执行`previous_uv = current_uv - velocity`，history打包depth/AO/motion magnitude并做3x3 AO bounds clamp、depth与velocity权重。Lumen复刻的`TemporalReprojection.hlsl`进一步明确ViewRect/history UV边界、previous depth gather、disocclusion threshold、自定义bilinear visibility weight、frame-count/confidence和fast-update state；但该复刻中object GBuffer velocity读取被注释为零值，不能直接照搬为动态物体正确性证据。Zircon现有velocity同样采用`current_uv - previous_uv`，未来AO reprojection必须减去velocity，并同时通过camera-cut/FOV/projection/world-origin与object-motion资格。
5. 正确后继只能是独立`AoTemporalHistoryStore`。它仅在`CompiledAoProfile.temporal`、typed motion producer与完整`AoHistoryKey`同时合格时按view创建；previous/current ping-pong和valid rect显式提交，payload至少能证明AO、linear depth、normal与confidence/reject信息，格式由capability/quality compiler裁决，不复用已删除的裸`Rgba8Unorm`。disable/re-enable、cut、teleport、rebase、projection/profile/extent/generation变化均显式invalidate。

Temporal实现前的性能报告门固定如下：在同一adapter/driver/BuildSet、关闭capture干扰并完成warm-up后，分别采集full/half的evaluate、spatial、upsample GPU timestamp和总AO critical-path p50/p95/p99，记录transient/persistent bytes、dispatch groups、PSO warm/cold、RenderDoc pass/resource依赖与整机功耗；运动语料必须覆盖camera/object motion、disocclusion、thin occluder、depth/normal edge、odd extent和history reset。候选temporal应保持每像素有界的`O(P)` reprojection/reject/clamp、热帧CPU allocation 0、无额外整图copy；只有ghosting/convergence容差通过且总成本与功耗相对无temporal基线可解释时，才实现/启用并声明优化。当前动态通道仍在Cargo前被`cargo_reuse_target_mismatch`阻断，所以本切片没有Rust/WGPU、PNG/RDC、GPU timestamp或功耗结论，也不计accepted milestone。

## 3. 当前可保留的工程基础

1. SSAO descriptor 使用真实 per-pixel compute workload，不是固定 `[1,1,1]` no-op；Render Graph 明确声明 depth/normal/HZB/history/AO access。
2. shared HZB 已能被 SSAO 读取，避免为每个 screen-space effect另建一套无所有权金字塔。
3. offscreen target已有current AO物理输出；旧共享AO history已因无合格temporal消费者被hard cut。未来只保留`AoHistoryKey`/domain作为迁移合同，物理history必须由独立`AoTemporalHistoryStore`重新建立。
4. profile、compiled feature flags与 degrade ladder能够禁用 SSAO；插件插入时会移除同名旧 feature，不会在正确注入路径同时执行两遍。
5. 产品测试真实创建 GPU viewport、执行 HZB/SSAO/generic executor并检查 resource materialization；应改造其画质 oracle，而不是删除这条产品 lane。
6. Deferred path 已有 world normal GBuffer，post process也已有 projection/depth参数与 viewport-origin工具；这些能力目前没有被 SSAO 正确消费。

## 4. 参考实现给出的工程边界

### 4.1 Fyrox：轻量实现也先满足几何合同

Fyrox 以半分辨率 R32F target、32 个半球 sample kernel与 4x4 random noise工作；shader用 inverse projection重建 view-space position，把 world normal转到 view space，通过 TBN旋转sample、重新投影并做 range/bias判断，随后执行 blur。其 blur仍是简单 box filter，不能作为 Zircon 最终质量上限，但足以证明“raw depth差 + world normal.z”甚至没有达到轻量 SSAO 的基本几何门槛。

### 4.2 Bevy：VBAO、深度预处理与边缘保持降噪

Bevy 的 camera SSAO settings提供 Low/Medium/High/Ultra/Custom、radius与thickness，并要求 DepthPrepass/NormalPrepass和设备能力。其实现基于 XeGTAO/VBAO：预处理加权 depth mips，用 projection与depth缩放screen radius，以 Hilbert/R2 noise旋转方向，执行 horizon bitmask integration，并把 depth edge数据交给 3x3 edge-preserving spatial denoise。Bevy 当前路径不提供完整 motion-vector temporal reprojection，本篇不会虚构这一能力。

### 4.3 Godot：质量档、adaptive gather与smart blur

Godot 暴露 radius、intensity、power、detail、horizon、sharpness、quality、half-size、adaptive target、blur passes与distance fade；执行链包含 depth downsample、quality-dependent gather、importance map/adaptive阶段、smart/wide blur与interleave。其关键价值是把空间半径、质量成本、边缘信息、降采样/重组和远距fade作为同一可伸缩产品合同，而不是单个固定shader常量。

### 4.4 Unity HDRP：时序拒绝、全/半分辨率与组合策略

HDRP Volume暴露 intensity、direct-lighting strength、radius、temporal accumulation、ghosting reduction、blur sharpness、specular occlusion、step/direction count、full resolution、maximum radius in pixels、bilateral upsample以及occluder/receiver motion rejection。RenderGraph把 evaluate、spatial denoise、temporal denoise、history copy和blur/upsample分开；temporal shader使用motion reprojection、depth similarity、neighborhood clamp与velocity weighting。它还允许在有资格时选择 RTAO，但不会把所有 AO source伪装成同一算法。

### 4.5 Unreal：显式方法、pass family、质量与调度资格

Unreal 将 SSAO/GTAO method、Horizon Search/Integrate/Spatial/Temporal/Upsample pass、downsample factor、normal/history/velocity输入、shader quality和async模式显式化；shader按质量选择 tap数量，并围绕 view position、view normal、falloff、thickness与temporal参数工作。CVar与GPU marker提供调参与诊断入口。Zircon应吸收 typed method/pass/quality/receipt，不照搬 Unreal 的历史兼容开关或以 CVar散布代替 compiled profile。

## 5. Owner 裁决与非重复边界

| Owner | 本篇拥有 | 本篇不重复拥有 |
|---|---|---|
| Runtime27 | AO几何/算法合同、AO输入资格、spatial/temporal denoise、upscale、AO输出与光照合成、quality/scalability、AO产品资格 | 通用RHI资源寿命与队列正确性 |
| Runtime09A | Render Graph、barrier、queue、device generation、async-compute真实性 | AO算法与画质 |
| Runtime09B | HZB生产、visibility/GPU Scene | AO如何采样/线性化/选择mip |
| Runtime09C | shader/PSO artifact与permutation owner | AO source/profile语义 |
| Runtime09E | direct lighting、shadow与contact shadow | SSAO不能压暗direct light的具体合成修复由Runtime27验收 |
| Runtime09H1 | 通用velocity/history generation、viewport/dynamic-resolution坐标域 | AO专属reprojection/rejection/clamp/history payload |
| Runtime09H2 | terminal post-process、SSR主算法与通用Volume | AO evaluate/denoise/composition；SSR只消费typed specular-occlusion结果 |
| Runtime23 | 全局space/unit/projection schema | AO对该schema的具体使用与反向测试 |
| Editor22 | 通用render/post-process authoring framework | SSAO typed settings/schema由Runtime27定义，Editor只消费 |
| Plugins04 | rendering package/capability/default delivery truth | AO算法、输入、质量和产品oracle |

Runtime09H1已拥有“全局 `history_available` 无法表示各history domain有效性”的通用 P0；本篇只登记 AO 的独立 generation、reprojection与reject要求，不重复计同一 P0。Plugins04继续拥有 stable/complete/default capability truth；Runtime27把“当前默认算法产生错误画面”列为产品正确性问题。

### 5.1 2026-08-30 通用 Compute PSO / last-good 结构重审

本轮不以AO专用fallback修补通用shader/PSO所有权。当前 `ComputePipelineCache` 以完整WGSL源文本、entry point和binding layout为候选cache key，仅有`Ready/Failed`条目；`GenericComputeExecutor` 在每个pass的command recording路径持有全局`Mutex`，同步执行Naga parse/validate、bind-group layout、shader module和compute pipeline创建。这些是源码事实，尚未有GPU/CPU profile证明它们是实际帧耗瓶颈。

Unreal对照证据是`RHI/Private/PipelineStateCache.cpp` 4243行附近的`PipelineStateCache::GetAndOrCreateComputePipelineState`：先以完整`FComputePipelineStateInitializer`查找PSO，miss时根据`IsAsyncCompilationAllowed`创建completion event并调用`InternalCreateComputePipelineState`，非file-cache候选作为command-list dispatch prerequisite。`RenderCore/Private/ShaderPipelineCache.cpp`另外拥有compute PSO precompile、batch size/time budget及outstanding/waiting/active/compiled/skipped统计。Zircon应吸收“候选编译、发布、命中与调度回执分层”，不照搬UE的RHI异步任务实现。

结构裁决：

1. `pipeline_label`只是调试名，不能证明旧shader与新params/resource语义兼容；source hash也只能标识candidate artifact，不能充当interface version。
2. 只有显式选择last-good的compute workload才可回退；默认policy仍为`Reject`。
3. 可回退family identity至少包含logical family与caller-owned interface generation；运行时还必须再精确匹配entry point、workgroup size、全量binding layout、scene layout generation和device generation。任一不同都禁止使用旧PSO。
4. candidate cache以artifact fingerprint隔离source revision；family publication独立持有当前last-good。只有Naga验证、WGPU validation error scope和发布条件全部成功才能替换last-good。失败candidate不得污染已发布generation。
5. resolution必须返回typed `Ready/UsingLastGood/Failed`、candidate/resolved fingerprint、interface/device generation和failure reason，并写入`RenderGraphComputeDispatchRecord`。AO只消费该通用receipt，不自行猜测shader是否回退。
6. device loss/OOM属于Runtime09A的device owner；新device generation必须丢弃所有旧WGPU handle，不得跨device使用last-good。本篇只在收到该typed terminal state后发布AO `Recovering/Failed`。

2026-08-30 源码实施已完成上述边界：Render Graph workload/dispatch receipt具有显式fallback policy、family/interface identity和typed resolution schema；通用compute cache分为bounded candidate cache与bounded family publication cache，WGPU validation error scope阻止无效PSO发布；AO evaluate/spatial以及half profile条件启用的bilateral upsample以三个稳定family和interface generation 3显式选入；AO逐帧receipt只消费generic executor写入的`Ready/UsingLastGood`事实。device epoch变化或scene layout变化会清空已发布WGPU handle，接口、entry、workgroup或binding ABI变化均不回退。当前状态为`render_plan07_ssao_half_resolution_bilateral_source_implemented_dynamic_validation_pending`；尚未执行新增Rust/WGPU测试，因此不宣称编译、运行或恢复能力已验收。

性能验证计划（当前只是待测假设，不是优化收益声明）：在validator/WGPU基础设施恢复后，用CPU span分开记录cache lock wait、Naga parse/validate、WGPU module/PSO creation、bind-group creation与command encoding，报告cold miss、warm hit、failed candidate和8/32个concurrent pass的p50/p95/p99；再以RenderDoc/GPU timestamp确认AO evaluate/spatial GPU时间与queue overlap。只在该基线证明cache miss/lock处于critical path后，才引入prewarm/async compilation并用同一corpus复测；需同时报告adapter/driver/backend、CPU帧时、GPU pass时间、pipeline create次数、fallback次数、峰值驻留和功耗采集来源。

## 6. P0：必须先关闭的产品正确性错误

| ID | 当前证据与风险 | 必须达成的修复合同 |
|---|---|---|
| SSAO-P0-001 | `ssao.wgsl`只比较8个相邻像素的raw device depth，按world-space `normal.z`固定压暗；无position reconstruction、projection/world radius、thickness/falloff或far-depth guard。平面可因朝向而变暗，camera/projection/resolution变化会改变结果，却以默认SSAO交付 | 以view-space position/normal和明确depth convention实现GTAO/VBAO基线；world radius、thickness、bias、falloff、max pixel radius进入compiled profile；解析不到合格输入时fail-close为AO=1并发布typed reason |
| SSAO-P0-002 | 默认Forward+没有Deferred Geometry/normal writer，Mesh pass不写`GBUFFER_NORMAL`；资源绑定仍把retained `normal_view`交给SSAO。首次使用只会得到没有场景normal producer资格的初始化/clear值，pipeline切换还可读取stale normal；现有Forward+产品测试正运行此路径 | 每个pipeline在compile时证明depth/normal的producer、space、format、sample count、valid rect与generation；Forward+应增加合格normal prepass/attachment或拒绝SSAO，禁止只因纹理对象存在就materialize成功 |
| SSAO-P0-003 | `post_process.wgsl`读取AO后平方、clamp并乘到完整lit `scene_color`，因此直射diffuse、specular、emissive、sky/background一起被压暗；SSR又把同一标量当specular occlusion | AO只作用于indirect diffuse/ambient irradiance；direct-light influence必须是显式受限policy，emissive/sky不受影响；specular occlusion使用独立、roughness/normal-aware合同，不能复用任意SSAO标量冒充 |

## 7. P1：算法、几何输入与输出语义

| ID | 当前差距 | 重构要求 |
|---|---|---|
| SSAO-P1-001 | raw depth差没有线性化，near/far、reverse-Z与投影会改变阈值含义 | 消费Runtime23 projection/depth descriptor，统一reconstruct/linearize helper并以golden vector验证 |
| SSAO-P1-002 | 没有view-space position重建，sample不是半球或horizon积分 | 选择并version明确GTAO/VBAO算法，建立evaluate pass与numerical reference |
| SSAO-P1-003 | 固定1像素邻域使遮蔽半径随分辨率、FOV和距离变化 | 使用world/view-space radius投影到pixel radius，并设min/max pixel bound |
| SSAO-P1-004 | 没有orthographic、oblique、reversed/standard depth或infinite-far资格 | compiled input contract声明projection class；unsupported必须禁用而非近似运行 |
| SSAO-P1-005 | GBuffer编码world normal，shader却直接以`.z`当几何方向 | 以view matrix转换并验证normal encoding/handedness；不得把world axis当view axis |
| SSAO-P1-006 | zero/clear/non-finite normal经`normalize`后无validity语义 | normal producer提供valid mask或定义clear sentinel；invalid pixel输出AO=1 |
| SSAO-P1-007 | background/far-plane depth没有early-out，sky边界会参与depth edge暗化 | 统一background predicate、valid depth range和MSAA resolve语义 |
| SSAO-P1-008 | 8个固定罗盘offset形成方向偏差与稳定格纹 | 用低差异序列/blue-noise/Hilbert旋转及quality-dependent directions/steps |
| SSAO-P1-009 | 没有radius、thickness、bias、falloff、horizon angle等物理/感知参数 | 定义有单位、范围、default与migration的`AoSourceSettings` |
| SSAO-P1-010 | HZB只固定读取furthest mip 1并比较绝对raw depth差 | 根据sample footprint选择mip，明确closest/furthest语义并以view depth做horizon test |
| SSAO-P1-011 | 已绑定完整HZB mip chain但算法不利用分层搜索或大半径成本控制 | quality compiler选择depth mip策略、search steps与early-out，并报告实际work |
| SSAO-P1-012 | AO最低0.1、`0.24/0.08`和response均硬编码且互相叠乘 | 所有参数进入versioned preset，避免不可解释的多重clamp与double response |
| SSAO-P1-013 | AO没有与material/shading model、two-sided/thin surface语义协商 | 至少定义opaque receiver/occluder资格；subsurface、foliage、unlit与masked策略显式化 |
| SSAO-P1-014 | shader忽略viewport origin/view subrect，只按local coord读physical资源 | 所有depth/normal/HZB/AO读写携render rect/origin，覆盖atlas view、split-screen与capture subrect |

## 8. P1：Spatial Denoise、Temporal 与 History

| ID | 当前差距 | 重构要求 |
|---|---|---|
| SSAO-P1-015 | evaluate后没有spatial denoise，噪声/边缘只能靠减少几何信息掩盖 | 独立edge-aware spatial pass，消费depth/normal/packed edge并有sharpness/quality档 |
| SSAO-P1-016 | temporal只把当前像素与previous同坐标固定`mix(...,0.18)` | 使用motion vector或可证明的camera reprojection映射history坐标 |
| SSAO-P1-017 | 不消费object/camera velocity，moving occluder/receiver留下拖影 | 分别支持occluder/receiver motion rejection并记录motion availability |
| SSAO-P1-018 | 无depth/normal similarity test，遮挡边界直接混旧AO | history sample必须通过linear depth、normal cone与surface identity gate |
| SSAO-P1-019 | 无disocclusion/viewport reveal判定 | 新暴露、out-of-bounds、camera cut、teleport、rebase像素立即拒绝history |
| SSAO-P1-020 | 无neighborhood min/max/moment clamp，旧暗值可跨表面传播 | temporal resolve使用current neighborhood bounds或moments限制反馈 |
| SSAO-P1-021 | AO只继承全局`history_available`，没有独立generation/valid rect | 建`AoHistoryKey`与`AoHistoryState`，包含view、projection、extent、profile、input generations与valid rect |
| SSAO-P1-022 | `temporal_history=false`只关闭TAA，SSAO仍在全局history存在时混旧AO | AO temporal成为独立typed setting；禁用时不读/写旧history并发布原因 |
| SSAO-P1-023 | 当前采样完全固定，没有temporal jitter/noise，却仍累积history造成lag | 只有存在跨帧互补sample sequence和reject/clamp时才启用temporal accumulation |
| SSAO-P1-024 | disable/re-enable、quality/method/parameter变化可复用stale AO | profile/method/parameter hash进入history key，任何不兼容变化显式invalidate |
| SSAO-P1-025 | current AO按render size，history AO按output size，copy只覆盖左上render rect | 统一history coordinate domain，或记录scaled valid rect并执行明确resample/clear |
| SSAO-P1-026 | RGBA8 history只存单标量，无depth/motion/confidence或precision合同 | 选择可解释的AO/history格式与aux payload；格式由quality/capability编译且有memory budget |

## 9. P1：Pipeline Integration、Scalability、性能与 Authoring

| ID | 当前差距 | 重构要求 |
|---|---|---|
| SSAO-P1-027 | `write_ssao_compute_params`传`target.size`，AO/depth/normal/HZB却按`target.render_size`分配 | params、dispatch、UV、resource extent统一来自render rect；display extent只用于最终upscale |
| SSAO-P1-028 | degrade ladder先降到0.85/0.7 render scale，正好进入错误AO坐标域，较晚才关闭SSAO | 每个degrade step先通过组合资格；scale变化必须重编profile/history或立即禁用 |
| SSAO-P1-029 | 只有full-resolution evaluate，没有half/quarter-res或bilateral upsample | quality档选择full/half resolution、denoise与upsample pipeline，并限制最大pixel radius |
| SSAO-P1-030 | `RenderQualityProfile`只有布尔开关，无法表达成本/质量 | 增加typed `AoQualityTier/CustomAoQuality`，编译为directions、steps、mips、resolution、denoise、temporal参数 |
| SSAO-P1-031 | `[4.6,0.0015,0.18,0.88]`在资源写入函数硬编码 | settings -> validation -> compiled profile -> uniform单向生成，禁止runtime magic tuning authority |
| SSAO-P1-032 | 没有project/camera/volume override、blend、priority或serialization | Runtime定义稳定schema与merge规则，Editor22只通过共享compiler预览/创作 |
| SSAO-P1-033 | current/history均为RGBA8而只消费red，至少占用约4倍标量带宽/驻留 | 依据backend storage支持选择R8/R16F/packed edge格式并测量质量、带宽与兼容性 |
| SSAO-P1-034 | 输出只有AO标量，没有bent normal/confidence/validity，消费者只能猜 | 定义`AmbientOcclusionOutputs`；MVP可只生产diffuse AO，但identity/validity必须typed，可扩展bent normal |
| SSAO-P1-035 | SSR直接用SSAO标量生成specular occlusion response | 建独立specular-occlusion contract，结合roughness/view normal/bent normal或明确禁用 |
| SSAO-P1-036 | builtin与plugin复制同名descriptor和shader authority，未来参数/资源易漂移 | 保留一个canonical descriptor/compiler/shader artifact owner；plugin只注册provider/capability |
| SSAO-P1-037 | Forward/Deferred只按feature name启用，没有input compatibility matrix | pipeline compiler输出每个AO input producer与conversion plan，缺失时给typed compile diagnostic |
| SSAO-P1-038 | 声明`AsyncCompute`不等于实际重叠，AO也没有独立GPU timing/work estimate | 接Runtime09A queue资格，记录evaluate/denoise/temporal/upscale GPU时间、pixels、steps与overlap receipt |

## 10. P1：产品测试、诊断与资格

| ID | 当前差距 | 重构要求 |
|---|---|---|
| SSAO-P1-039 | Editor feature只有名称、crate与capability descriptor | 在Editor22框架中提供真实settings/volume/camera inspector、preview generation与runtime diagnostic，不建私有算法 |
| SSAO-P1-040 | 唯一产品oracle只要求AO开启后平均亮度降低5，错误全局压暗反而更容易通过 | 改为区域/语义oracle：contact darkening、open surface neutrality、background/emissive/direct preservation同时成立 |
| SSAO-P1-041 | 无解析几何ground truth或CPU reference | 建sphere-on-plane、corner、parallel planes、thin occluder、open plane、sky edge corpus和容差mask |
| SSAO-P1-042 | 无camera rotation/world orientation不变性测试 | 同一view-space构型旋转world/camera后AO在容差内一致，专门阻止`normal.z`回归 |
| SSAO-P1-043 | 无direct light、specular、emissive、unlit、sky分量隔离测试 | capture分量buffer或构造受控scene，证明AO只影响许可的indirect consumers |
| SSAO-P1-044 | 无moving occluder/receiver、disocclusion、cut、teleport、rebase temporal corpus | 输出逐帧raw/filtered/history/reject mask并检查ghosting、convergence和reset |
| SSAO-P1-045 | 无0.7/0.85/full DRS、viewport origin、split-screen、orthographic测试 | 每种合格projection/extent/subrect跑像素与resource-coordinate oracle |
| SSAO-P1-046 | 无quality tier成本/画质曲线、GPU vendor/backend或memory基线 | 固定corpus报告GPU p50/p95/p99、bandwidth/bytes、quality error和adapter identity |
| SSAO-P1-047 | 无unsupported format/capability、OOM/device loss/shader failure/last-good策略 | compile/runtime返回Ready/Disabled/Unsupported/UsingLastGood/Recovering/Failed及终态receipt |
| SSAO-P1-048 | 产品测试不固化shader/profile/build/currentness，截图/统计不能证明执行了哪代AO | evidence记录BuildSet、shader/artifact/profile/input/history/device generations、executed pass与golden hash |

## 11. P2：P0/P1 闭合后的高级能力

| ID | 能力 | 前置条件 |
|---|---|---|
| SSAO-P2-001 | bent normal输出与environment/IBL方向性遮蔽 | typed AO outputs、environment lighting consumer与质量oracle |
| SSAO-P2-002 | multi-bounce AO或albedo-aware energy compensation | indirect-light能量合同与material input已稳定 |
| SSAO-P2-003 | GTAO与RTAO/DF-AO按view/platform切换或融合 | typed AO source、fallback资格、history transition与噪声一致性 |
| SSAO-P2-004 | foveated/VRS-aware AO与perceptual sample allocation | 基础quality error metric、VRS contract与edge safety |
| SSAO-P2-005 | temporal supersampling与adaptive sample budget | motion/rejection/clamp、deterministic sample sequence和GPU timings |
| SSAO-P2-006 | subgroup/shared-memory depth/normal tile优化 | canonical算法、backend feature gate与跨vendor数值容差 |
| SSAO-P2-007 | async-compute overlap自动调度与critical-path预算 | Runtime09A queue/barrier/timeline evidence和AO per-pass workload |
| SSAO-P2-008 | AO debug view：raw、horizon、edge、spatial、history、reject、upsample | 每个中间资源有stable identity、bounded lifetime和capture schema |
| SSAO-P2-009 | pixel inspector显示sample radius、mip、directions、confidence与reject reason | shader debug payload、readback budget与Editor22 inspector |
| SSAO-P2-010 | 基于scene统计的quality tier建议与自动降级 | stable workload corpus、telemetry retention与不可自认证quality gate |
| SSAO-P2-011 | offline AO replay/capture用于跨GPU differential | deterministic input capture、shader artifact与reference evaluator |
| SSAO-P2-012 | 同硬件同画质的Unreal/Unity/Godot竞争基准与自动回归二分 | 正确性、视觉容差、BuildSet、driver和统计协议全部冻结 |

## 12. 目标架构

```text
Project / Volume / Camera AoSourceSettings
  -> AoProfileCompiler
  -> CompiledAoProfile
       method + quality + units + resolution + sample sequence
       input requirements + formats + budgets + history compatibility
  -> PipelineInputQualification
       linear depth / view normal / motion / material class / render rect
  -> AoDepthPreprocess + depth hierarchy
  -> AoEvaluate(GTAO/VBAO)
  -> AoSpatialDenoise(edge-aware)
  -> AoTemporalResolve(reproject / reject / clamp / confidence)
  -> AoBilateralUpsample
  -> AmbientOcclusionOutputs
       indirect_diffuse_ao / optional specular_ao / bent_normal / validity
  -> Lighting consumers before terminal composition
  -> per-view AoHistoryState + bounded telemetry + qualification receipt
```

`CompiledAoProfile`必须是运行时唯一参数authority，携source/profile/compiler/shader/backend generation和可估算work。Render Graph只能在input qualification成功后实例化pass；纹理对象存在不代表内容有效。AO output在lighting composition边界被消费，terminal post process不得再次解释或平方同一标量。

history identity至少包含view family/view、pipeline、projection/depth convention、render rect、method/profile、depth/normal/motion generations和world-origin epoch。history reuse是逐像素资格，不是单个全局布尔值。dynamic resolution变化要么使用明确scaled history/resample contract，要么invalidate，不能部分copy后继续声称完整有效。

## 13. 分层实施计划

### M0 · Product Truth 与 P0 Containment

- 默认关闭当前SSAO或标记Unavailable，替换“更暗即成功”oracle；
- pipeline compile验证normal/depth producer，Forward+无合格normal时拒绝feature；
- 移除terminal全scene AO乘法，先保证禁用AO时所有lighting分量不变。

### M1 · Schema、Input Qualification 与 Canonical Owner

- 定义`AoSourceSettings/CompiledAoProfile/AmbientOcclusionOutputs/AoHistoryKey`；
- 合并builtin/plugin descriptor/shader authority，建立versioned artifact与last-good；
- 接入projection/depth/normal/render-rect/motion producer contract。

### M2 · GTAO/VBAO Evaluate 与 Spatial Pipeline

- 实现view-space reconstruction、depth hierarchy、horizon integration、quality sample sequence；
- 建half/full resolution、edge data、spatial denoise与bilateral upsample；
- 以CPU/numerical reference和analytic scene关闭几何正确性门。

### M3 · AO History 与 Lighting Composition

- 实现motion reprojection、depth/normal/disocclusion rejection、neighborhood clamp与confidence；
- 把AO接入indirect diffuse，建立独立specular-occlusion策略；
- 覆盖cut/teleport/rebase/DRS/subrect/multi-view history transition。

### M4 · Authoring、Scalability 与 Failure Recovery

- 接Project/Volume/Camera settings、preset migration与Editor22 preview/debug；
- quality/budget compiler选择resolution/directions/steps/denoise/temporal，并验证degrade ladder；
- 建capability/device-loss/OOM/shader-failure/last-good状态与receipt。

### M5 · Product 与 Competitive Qualification

- 执行analytic、component isolation、motion、DRS、多相机、多backend与长期soak矩阵；
- 固化GPU timing、memory/bandwidth、quality error、artifact/device identity；
- 只有同硬件、同分辨率、同画质容差与同统计协议下才能比较Unreal/Unity等参考实现。

## 14. 验收门

| Gate | 验收内容 |
|---|---|
| SSAO-G01 | 当前depth-edge/orientation darkener不再以默认SSAO产品路径启用 |
| SSAO-G02 | Forward与Deferred均在compile receipt中证明depth/normal producer、space、format、extent和generation |
| SSAO-G03 | invalid/missing depth或normal fail-close为AO=1并发布typed diagnostic |
| SSAO-G04 | view-space position reconstruction通过perspective、orthographic与reverse-Z golden vectors |
| SSAO-G05 | world/view normal转换与encoding在world/camera旋转后保持AO不变 |
| SSAO-G06 | radius/thickness/bias/falloff有单位、范围、serialization和migration |
| SSAO-G07 | open plane、sky/background和far depth在容差内保持无遮蔽 |
| SSAO-G08 | corner、sphere-on-plane、thin occluder和parallel-plane结果匹配reference tolerance |
| SSAO-G09 | sample sequence无固定罗盘方向偏差，quality档可复现且有version |
| SSAO-G10 | depth hierarchy mip选择随sample footprint变化且closest/furthest语义有测试 |
| SSAO-G11 | spatial denoise保持depth/normal边缘，不把AO跨surface扩散 |
| SSAO-G12 | temporal使用motion reprojection，不再固定读取同坐标previous AO |
| SSAO-G13 | depth、normal、motion、disocclusion与out-of-bounds均可拒绝history |
| SSAO-G14 | temporal feedback经neighborhood/moment clamp，移动边界无持久暗拖影 |
| SSAO-G15 | AO history有独立key、generation、valid rect和reset reason |
| SSAO-G16 | 禁用AO temporal时不读取旧AO，重新启用不会复用不兼容history |
| SSAO-G17 | camera cut、teleport、world rebase、pipeline/method/profile变化显式invalidate |
| SSAO-G18 | render-size、display-size、resource extent、dispatch和UV坐标域一致 |
| SSAO-G19 | 1.0/0.85/0.7 DRS及resize转换无越界、stale top-left history或画面漂移 |
| SSAO-G20 | viewport origin、subrect、split-screen与多view资源不串读 |
| SSAO-G21 | Low/Medium/High/Ultra/Custom编译到有界directions/steps/mips/resolution/denoise策略 |
| SSAO-G22 | half/full resolution结果满足各自质量容差且bilateral upsample不漏边 |
| SSAO-G23 | max pixel radius和work estimate阻止近相机/高分辨率成本无界增长 |
| SSAO-G24 | current/history格式按能力选择并进入显存/带宽预算，不再无理由使用双RGBA8标量纹理 |
| SSAO-G25 | AO只影响许可的indirect diffuse/ambient lighting分量 |
| SSAO-G26 | emissive、unlit、sky/background、direct specular在AO开关前后保持容差 |
| SSAO-G27 | specular occlusion有独立roughness/normal-aware合同或明确禁用 |
| SSAO-G28 | builtin/plugin只有一个descriptor/compiler/shader artifact authority |
| SSAO-G29 | plugin enable/default/capability与Runtime输入/算法资格一致，不能只凭registration成功 |
| SSAO-G30 | Editor settings经共享schema/compiler进入runtime，preview显示实际generation/quality/disposition |
| SSAO-G31 | 产品oracle同时检查contact darkening与open/background/emissive/direct preservation |
| SSAO-G32 | analytic CPU/reference corpus在required GPU lane至少执行一个adapter，0 case不得计pass |
| SSAO-G33 | moving occluder/receiver、disocclusion、cut与teleport逐帧artifact无超限ghosting |
| SSAO-G34 | evaluate/spatial/temporal/upscale分别记录GPU timestamp、pixels、steps和bytes |
| SSAO-G35 | quality tier报告同场景quality error、GPU p50/p95/p99、bandwidth与memory峰值 |
| SSAO-G36 | unsupported capability/format返回Disabled或Unsupported，不创建假成功pass |
| SSAO-G37 | shader compile失败、OOM pressure与device loss使用last-good或明确终态并隔离旧generation |
| SSAO-G38 | capture包含BuildSet、shader/profile/input/history/device generation与executed-pass receipt |
| SSAO-G39 | 24h camera/motion/resize/quality-switch soak无资源增长、stale history或非有限像素 |
| SSAO-G40 | “优于Unreal”只在同硬件、同场景、同画质容差与公开统计协议下表述 |

## 15. 风险、依赖与硬切约束

1. 先关闭P0，再调画质。当前产品oracle奖励全局变暗，继续调sample/strength只会让错误更难识别。
2. 先证明input producer，再创建pass。不能用clear texture、fallback white/black或retained view掩盖pipeline缺输入。
3. 先拆lighting composition，再增加bent normal、multi-bounce或RTAO；错误消费位置会污染所有高级AO source。
4. Runtime09A/09B/09C/09H1与Runtime23是底层依赖；本篇不得在AO模块内再造RHI、HZB、shader cache、velocity或projection authority。
5. Plugins04负责包与capability truth，Editor22负责通用authoring surface；二者不能复制AO settings merge/compiler或渲染算法。
6. Fyrox box blur只证明轻量基线，不是最终denoise目标；Bevy没有完整temporal reprojection，不能用其缺席降低Zircon history标准。
7. Unreal/Unity/Godot的参数数量不是目标本身；Zircon要先冻结参数单位、作用域、编译结果、预算和可验证失败语义。
8. 本轮遵守用户要求暂停新增tooling专题；未来验证复用既有evidence/BuildSet控制面，不新开tooling owner。

## 16. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Zircon source/test/product inventory | review_complete | 2026-08-16 | 62输入；plugin、pipeline、resources、history、shader、composition与product test纵向闭环 |
| 五套参考实现核对 | review_complete | 2026-08-16 | 27输入；Fyrox 3、Bevy 5、Godot 7、Unity HDRP 8、Unreal 4 |
| Currentness fingerprint | review_complete | 2026-08-16 | 89输入、26,329行、1,041,326 bytes；SHA-256 `16c546c2...c60108` |
| Algorithm/input/composition深读 | review_complete | 2026-08-16 | raw depth 8-neighbor、world normal.z、Forward normal无writer、full-lit-color乘AO |
| Finding与owner裁决 | review_complete | 2026-08-16 | 3 P0 / 48 P1 / 12 P2；通用history、RHI、plugin delivery与Editor framework不重复计数 |
| M0 product containment | source_implemented_dynamic_validation_pending | 2026-08-30 | 默认入口 fail closed；Forward+ 缺 normal producer 拒绝；terminal full-lit AO 乘法删除；旧“更暗即成功” oracle 删除 |
| M1 typed profile/output/execution receipt | source_implemented_dynamic_validation_pending | 2026-08-30 | producer version + depth/normal/HZB format/sample/extent/mip/render-rect 静态资格；canonical settings/profile/output/history identity与逐帧command-record receipt已落源码；兼容shader candidate的同device-epoch last-good已落源码，GPU completion、未发布硬失败、OOM/device-loss终态与动态证据仍开放 |
| Generic Compute PSO compatible last-good | source_implemented_dynamic_validation_pending | 2026-08-30 | 显式family/interface generation + 完整binding/workgroup ABI + device epoch约束；Naga/WGPU validation成功后才发布；Reject默认，AO evaluate/spatial/条件upsample显式启用；22-bit AO failure receipt与38条诊断路径已静态核对，动态WGPU证据仍开放 |
| M2 GTAO evaluate + spatial + half bilateral source | source_implemented_dynamic_validation_pending | 2026-08-30 | world-unit horizon evaluate写raw AO；full为evaluate->spatial，half以ceil-divided relative extent执行evaluate->spatial->full bilateral upsample；full depth/normal坐标映射、typed version edge与最终输出writer已落源码，画质/性能证据仍开放 |
| AO unqualified shared-history owner hard cut | source_implemented_static_checks_passed_dynamic_validation_pending | 2026-08-30 | 删除共享history中的AO texture/view/init/bind/copy/write-intent；SSAO不再触发共享history创建。静态容量模型减少4 bytes/render-pixel的无消费者持久纹理；未来独立`AoTemporalHistoryStore`、动态WGPU和性能/功耗证据仍开放 |
| M3 indirect-diffuse composition source | source_in_progress_dynamic_validation_pending | 2026-08-30 | final AO条件接入唯一deferred-lighting consumer；只调制ambient/environment diffuse，direct/specular/emissive/unlit保持独立；temporal、独立specular-occlusion与动态证据仍开放 |

完成标准不是“开启后画面更暗”或“SSAO pass已执行”，而是每个合格view都从有producer证明的depth/normal/motion输入生成几何正确、边缘稳定、时序可拒绝、可伸缩且只作用于许可光照分量的AO，并能用current artifact、GPU成本与反例scene证明结果。达到这些门之前，当前SSAO应视为不具默认产品资格。
