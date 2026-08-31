---
title: Runtime Environment、Sky、Atmosphere、Cloud、IBL、Reflection Probe、Capture、Convolution、SH、PMREM、Cache、Residency、Submission 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime96
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/graphics/scene/scene_renderer/environment
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/environment_ibl_hydration_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/environment_ibl_compile_options.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/importer/environment_ibl
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/asset/assets/scene/asset.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/scene/components/scene/identity.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime
  - zircon_plugins/rendering/features/reflection_probes/editor
tests:
  - zircon_runtime/src/graphics/tests/project_render/project_scenes/reflection_probe_product.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/render_extract.rs
  - zircon_runtime/src/scene/tests/render_extract/direct_sections.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironment.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentDiffuseIrradiance.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SkyAtmosphereRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricCloudRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/SkyLightComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/ReflectionCaptureComponent.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/SkyManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/PhysicallyBasedSky/PhysicallyBasedSky.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/PhysicallyBasedSky/PhysicallyBasedSkyRenderer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/HDProbe.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/HDProbeSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/ReflectionProbeTextureCache.cs
  - dev/bevy/crates/bevy_light/src/atmosphere.rs
  - dev/bevy/crates/bevy_pbr/src/atmosphere/mod.rs
  - dev/bevy/crates/bevy_pbr/src/atmosphere/node.rs
  - dev/bevy/crates/bevy_pbr/src/atmosphere/environment.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/mod.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/generate.rs
  - dev/godot/scene/3d/reflection_probe.cpp
  - dev/godot/scene/3d/world_environment.cpp
  - dev/godot/scene/resources/environment.cpp
  - dev/godot/scene/resources/sky.cpp
  - dev/godot/servers/rendering/renderer_rd/environment/sky.cpp
  - dev/Fyrox/fyrox-impl/src/scene/probe.rs
  - dev/Fyrox/fyrox-impl/src/scene/skybox.rs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Environment、Sky、Atmosphere、Cloud、IBL、Reflection Probe、Capture、Convolution、SH、PMREM、Cache、Residency、Submission 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon的环境光照底层已经不是占位。当前源码具备HDR/EXR source cubemap导入、canonical cubemap projection、versioned IBL recipe与`.zcube/.zribl`产物、CPU/GPU PMREM和SH9 bake、prepared upload artifact、bounded readback/writeback、全局environment cubemap upload key、reflection probe box/sphere influence与box projection、top-two blending、slot generation，以及实时IBL的generation token、双缓冲、按face/mip切片、完成后原子发布、compiled graph topology cache和GPU timestamp。这些实现应保留为characterization与重构底座。

但是普通项目无法通过场景资产表达任何正式Environment、Sky、Atmosphere、Cloud或Reflection Probe。`SceneAsset`、`SceneEntityAsset`、`SceneNode`和`NodeKind`没有对应字段；`World`构建渲染快照时只把Editor viewport的`preview_skybox`布尔值转换成默认程序渐变。反射探针产品测试在加载项目场景后手工替换`EnvironmentExtract`，反射探针插件的Editor trigger也没有任何`zircon_editor`消费者。因此当前完整能力只对直接构造Rust DTO的测试或调用者可达，不是可保存、可重开、可撤销、可烘焙、可cook的引擎产品。

天空本体仍是horizon/zenith/ground三色插值加解析sun disc。没有atmosphere介质、transmittance/multiple-scattering/sky-view/aerial-perspective LUT，没有cloud/weather/celestial body，也没有方向光、天空、雾、曝光和实时IBL之间的共享sun truth。实时IBL虽然已改成可靠的分片双缓冲状态机，但它只是捕获同一套渐变参数，固定每帧两面，不根据GPU timing调节预算，失败只会无限`Retry`，并在每个capture/downsample/IBL slice创建新的uniform buffer和bind group。

资源生命周期也没有工程闭环。`EnvironmentExtract`仍携带包含source、PMREM、SH和prepared bytes的`Arc`对象；clone是浅拷贝，并非旧报告所描述的逐帧深复制，但把内容资产塞进frame DTO仍然使generation、residency、eviction和device恢复边界错误。冷路径在frame submission中同步`fs::read`、decode、hydrate；hydration与pending bake都固定为4项count cache，没有byte budget、priority或异步I/O。全局cubemap staging upload会在uniform准备期间自行`queue.submit`，reflection probe准备又可能同步load asset，并对每个新probe执行8 mip乘6 face的48次`queue.write_texture`。

探针选择同样只达到了功能可见级别。资源固定分配64个128x128 RGBA16F cubemap槽，连同固定planar资源约占74.67 MiB；CPU每view收集候选并裁到64个，shader再对每个fragment线性扫描全部active probe以选top two。layer mask虽然被打包进GPU结构，却没有被`zr_environment.wgsl`读取。prepare report在正式写scene uniform路径被赋给下划线变量后丢弃，上传、拒绝、失效和降级无法进入frame readiness。

另有一个独立于旧09F1十项P0的当前源码阻断：reflection-probes runtime是`zircon_plugins/Cargo.toml`的workspace member，却调用已不存在的`SceneRenderer::render_scene_color_hdr`；当前HDR capture入口属于`RenderFramework::capture_scene_color_hdr(viewport)`，底层renderer capture不是该插件可见的公共方法。静态名称解析已经证明没有同名实现或extension trait。本文没有运行Cargo，因此不伪造编译日志，但在当前源码面上该workspace package无法通过类型检查。

Runtime09F1登记的10项P0仍保持开放，其中“CaptureSky与CaptureCloud重复覆盖”的旧证据已经被当前未提交改动修复，不能继续沿用；非物理天空本身仍开放。本文新增 **1项P0、48项P1、12项P2与48个资格门**。在scene/editor authoring、物理天空、异步产物与resident generation、统一submission、GPU probe assignment、可恢复capture job、fault/scale和同硬件同画质基准闭合前，不得声称Environment/IBL/Reflection Probe达到或优于当前Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 证据等级 | fingerprint |
|---|---:|---|---|
| Zircon environment/IBL/probe产品语料 | **131 / 32,653 / 29,938 / 1,189,658 / 281** | E3逐文件覆盖scene/asset/editor入口、neutral contract、artifact、bake、cache、upload、probe、shader、capture plugin与submission | `da296a02877523a0ce25cd19b749f4519ba03b179269c83d76cdff711fb3b663` |
| focused product tests | **4 / 1,593 / 1,481 / 57,981 / 19** | E3读取真实project render、scene extract与authoring boundary；识别手工environment注入和ignored capture | `ca142fbc5f67202b56c91f40f91f84e1befebb5f64d503c674edc7ac87d48340` |
| 五引擎参考切片 | **27 / 26,640 / 22,700 / 1,192,747 / 0** | E2/E3读取Unreal atmosphere/cloud/skylight/probe、Unity HDRP、Bevy、Godot与Fyrox责任链 | `fe5bfa503c2d873adbe8667d2e3ee4af2a2cc84028f0a5a427d97f390ea812f3` |

Zircon语料由frontmatter中的owner目录和显式integration文件组成；focused tests独立计数。fingerprint算法与Runtime95一致：规范化小写相对路径与每文件SHA-256组成排序manifest，再对manifest执行SHA-256。冻结对象是2026-08-21共享working tree，不是只读HEAD，也不是动态验收receipt。

Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator epoch为336。Bevy、Fyrox、Godot与Unity Graphics revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`和`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal目录不是独立Git checkout，本文用参考切片aggregate fingerprint冻结，禁止把父仓revision伪装成Unreal revision。

冻结时相关路径存在20个其他Session或用户的working-tree修改，共`+831/-683`。其中10个属于realtime IBL graph/scheduler/recorder，9个属于Editor viewport交互，另有`render_packet.rs`交叉修改。本文读取并计入这些版本，不拥有也未修改它们。实施或验证前必须重算fingerprint并重新审计实时IBL切片、compiled topology cache、viewport preview覆盖和capture public API。

### 2.2 Owner边界

| 边界 | Runtime96要求的owner | 禁止的越界 |
|---|---|---|
| Environment source | `zircon_runtime::scene`拥有可序列化Environment/Sky/Atmosphere/Cloud/Probe component、asset schema、migration与World选择规则 | 不把Editor viewport preview状态保存成scene truth；不让graphics反向持有scene对象 |
| Neutral render contract | `core::framework::render`只发布generation-qualified descriptor、stable handle、prepared view和typed readiness | 不携带source/PMREM整份texel链、磁盘路径锁或WGPU对象；不建立兼容双写DTO |
| Graphics execution | `zircon_runtime::graphics`拥有LUT/capture/convolution、resident resource、GPU assignment、submission、fence和device-generation恢复 | 不在scene extract、uniform write或插件中私自submit、同步读盘或构造长期资源 |
| Editor product | `zircon_editor`拥有property panel、gizmo、preview override、capture/bake command、undo/redo、progress、cancel和diagnostic | 不让插件只暴露Rust trigger后宣称Editor已接入 |
| Plugin capability | reflection-probes插件只能通过稳定的runtime service/job contract扩展策略与操作 | 不依赖具体`SceneRenderer`内部方法，不绕过RenderFramework operation lock、viewport与submission owner |
| 历史P0 | Runtime09F1继续唯一计数原10项P0；Runtime96只登记新增API断裂P0 | 不因重复描述把旧P0再次计数，也不因局部算子存在而关闭父P0 |

### 2.3 明确未做

本轮没有修改production code、shader、Cargo、插件或资产，没有运行Cargo、Editor、真实GPU、RenderDoc/PIX、device loss、OOM、large-world、probe pressure、capture cancel、visual golden或性能基准。静态审查能证明schema缺失、调用链断裂、同步I/O、资源分配、shader访问和错误API，不能证明具体硬件的最终GPU时间、画质或稳定性。

## 3. 当前应保留的真实基础

1. Canonical cubemap face projection、equirect转换、PMREM layout与SH9/irradiance comparison已有数学和reference tests，应作为所有capture/import路径的唯一oracle。
2. `IblBakeRecipe`、key、versioned artifact descriptor、section decode、atomic staging/writeback和CPU/GPU parity test方向正确，应迁入统一derived-data service，而不是重写一套临时格式。
3. `SourceCubemapUploadArtifact`提供预编码row bytes与upload key，`CubemapUploadStagingArena`可复用host/GPU staging buffer；应由graphics upload scheduler统一消费。
4. 实时IBL已有generation token、stale completion拒绝、double buffer、分片capture/downsample/PMREM/SH9、terminal publish、compiled topology cache和timestamp，可升级为预算驱动job，而不是回退到同步六面捕获。
5. Reflection probe的box/sphere influence、box projection、priority、top-two blend、cubemap/revision slot identity及GPU布局测试可保留为correctness oracle。
6. Environment-only preview在未请求probe provider时只分配1x placeholder，避免无条件74.67 MiB资源，是正确的按能力延迟初始化方向。

## 4. 历史09F1 P0 current-source重验

| 父finding | 当前状态 | current-source证据与关闭要求 |
|---|---|---|
| 09F1-P0-1 scene/World authoring不存在 | 开放 | scene asset/component/NodeKind仍无environment/probe；World只从`preview_skybox`建默认extract；必须以save/reopen/cook/runtime roundtrip关闭 |
| 09F1-P0-2 Editor capture trigger断开 | 开放 | `ReflectionProbeCaptureEditorTrigger`仅在插件内部出现，`zircon_editor`无消费者；必须接入选择、事务、进度、取消和scene dirty |
| 09F1-P0-3 procedural sky非物理且capture重复 | 部分修正后开放 | 当前dirty源码已删除`CaptureCloud`重复覆盖；但正式天空与实时capture仍是三色渐变+sun disc，没有atmosphere/cloud |
| 09F1-P0-4 sun/sky/fog/cloud/exposure/directional无共享truth | 开放 | `ProceduralSkyParams`独立于scene directional light和camera exposure；必须建立generation-qualified environment evaluation |
| 09F1-P0-5 frame payload与同步磁盘I/O | 开放但修正文案 | payload clone主要为`Arc`浅拷贝，不是逐帧深复制；内容仍越过frame边界，cold cache仍在submission同步read/decode |
| 09F1-P0-6 probe prepare同步residency与48次write | 开放 | 当前调用名为`load_texture_asset`，内部可触发resident read；新slot仍逐8 mip×6 face写texture |
| 09F1-P0-7 fragment扫描64 probe且layer无效 | 开放 | shader循环`probe_count`，layer bits只pack不读；必须用per-view/per-cluster assignment和receiver mask关闭 |
| 09F1-P0-8 同步六面capture | 开放 | plugin仍clone六份snapshot、顺序HDR render/readback、CPU bake/stage，且当前调用API已断裂 |
| 09F1-P0-9 import/warm cache全量read/decode | 开放 | artifact section和prepared upload改善了重复转换，但无异步streaming、byte budget、partial residency或cook-time resident policy |
| 09F1-P0-10 无统一budget/readiness/failure terminal | 开放 | realtime retry无terminal，cache/probe/capture report未汇入同一frame/product readiness |

## 5. 新增P0

| ID | 阻断 | 静态证据 | 完成定义 |
|---|---|---|---|
| ENV-P0-001 | reflection-probes workspace package调用不存在的`SceneRenderer::render_scene_color_hdr` | 插件runtime `capture/execute.rs:47`是仓内唯一该符号调用；当前公开HDR入口是`RenderFramework::capture_scene_color_hdr(viewport)`，底层capture受crate/operation lock约束；插件列于`zircon_plugins/Cargo.toml`workspace members | 先建立稳定的capture job/service contract，再让插件通过该contract提交六面任务；plugin workspace type-check、非ignored contract test和真实WGPU product test通过；禁止重新公开内部renderer方法作为临时兼容层 |

## 6. 当前产品链与目标架构

当前链路：

```text
Editor viewport preview_skybox: bool
  -> SceneViewportExtractRequest
     -> World::build_environment_extract
        -> default ProceduralGradient EnvironmentExtract
           -> frame DTO携带source/PMREM/SH/prepared Arc内容
              -> submission同步cache read/decode/hydrate
                 -> scene uniform准备期间upload/私有queue.submit

测试/手写Rust ReflectionProbeData
  -> CPU候选裁到64
     -> sync texture load
        -> 每新slot 48次write_texture
           -> fragment逐像素扫描全部active probe并选top two

插件EditorTrigger（无zircon_editor消费者）
  -> 直接依赖SceneRenderer已删除API
     -> 预期六次snapshot clone + HDR readback
        -> CPU PMREM/SH + 同步stage/register
           -> 返回临时ReflectionProbeData，不写回scene transaction
```

目标链路：

```text
Versioned Scene Environment / Sky / Atmosphere / Cloud / Probe sources
  -> World authority resolution + volume blend + shared sun/camera generation
     -> neutral EnvironmentFramePacket { ids, generations, handles, view parameters, readiness }
        -> Graphics EnvironmentResourceManager
           -> async derived-data lookup / import / LUT / capture / convolution jobs
           -> byte-budgeted residency + device-generation resources
           -> GPU view/cluster probe assignment
           -> one frame submission owner + fence-qualified publication
              -> forward/deferred/fog/cloud/sky/reflection共用同一generation

Editor command/transaction
  -> job ticket + progress/cancel/diagnostic
     -> atomic artifact commit + scene component update + undo/redo + cook dependency
```

## 7. P1重构项

### 7.1 Scene、World、Editor与产品可达性

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| ENV-P1-001 | `SceneAsset`/`SceneEntityAsset`/`SceneNode`无Environment、Sky、Atmosphere或Cloud source | 定义versioned scene schema、defaults、validation、migration和save/reopen/cook/runtime roundtrip |
| ENV-P1-002 | `NodeKind`与component store无Reflection Probe | box/sphere influence、capture offset、mode、resolution、priority、layers、artifact reference和revision可authoring并进入World |
| ENV-P1-003 | World只把viewport `preview_skybox`变成默认gradient | World从scene source解析正式environment；preview override必须显式分层且不得污染scene truth |
| ENV-P1-004 | Editor trigger没有菜单、inspector、selection或command consumer | inspector/gizmo/context command接入typed job，支持undo/redo、scene dirty、progress、cancel和失败定位 |
| ENV-P1-005 | capture/register返回临时DTO，不原子写回scene与依赖图 | artifact commit、asset registry、component reference、transaction和cook dependency同一原子操作完成或回滚 |
| ENV-P1-006 | 没有environment volume、priority、blend与camera override规则 | World发布确定性authority/volume blend receipt，覆盖多camera、streamed scene、prefab和Editor preview |
| ENV-P1-007 | directional sun、sky、fog、cloud、exposure各自为truth | 建共享`EvaluatedEnvironmentGeneration`，所有consumer读取同一太阳方向/辐亮度、介质与曝光前物理量 |
| ENV-P1-008 | readiness只散落在cache、probe upload和realtime diagnostics | 每frame/job输出source、artifact、resident、capture、convolution、published generation与typed degrade/failure |

### 7.2 物理天空、Atmosphere、Cloud与视觉一致性

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| ENV-P1-009 | 正式天空只有三色插值与sun disc | 至少实现可验证的Rayleigh/Mie/absorption、planet radius、ground albedo与solar irradiance contract |
| ENV-P1-010 | 无transmittance和multiple-scattering LUT | LUT尺寸、格式、sample count、capability fallback、cache key和invalidations由quality/device profile控制 |
| ENV-P1-011 | 无sky-view LUT与camera height/planet transform | sky render、capture和reflection使用同一camera/planet evaluation，地面内外与large-world精度有边界测试 |
| ENV-P1-012 | 无aerial-perspective LUT或scene depth合成 | forward/deferred/fog/translucency/cloud使用统一transmittance/inscattering，depth/reversed-Z/MSAA路径一致 |
| ENV-P1-013 | 无volumetric cloud/weather source | cloud layer、density/weather map、wind、lighting、shadow和ambient occlusion进入scene asset与budgeted render graph |
| ENV-P1-014 | 无celestial body、moon/stars或昼夜变更语义 | 多光源天体、地平线遮挡、星空与时间驱动变化通过共享generation触发必要而非全量更新 |
| ENV-P1-015 | sky/IBL/exposure没有物理单位与reference white关系 | sky radiance、sun illuminance、camera exposure和IBL intensity采用明确单位与color-space conversion |
| ENV-P1-016 | procedural preview、realtime capture和最终sky重复实现函数 | 一个环境evaluation/shader module服务sky、IBL capture、fog/cloud和reference bake，禁止复制近似分叉 |

### 7.3 IBL artifact、cache、residency与submission

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| ENV-P1-017 | `EnvironmentExtract`携带source/PMREM/SH/prepared bytes的内容对象 | frame packet只携带stable handle、artifact/generation identity、sampling transform和readiness，不携带整份texel资产 |
| ENV-P1-018 | `source_cubemap()`在extract期间执行`with_prepared_upload_artifact()` | row encoding/format conversion属于async asset prepare/upload job，extract在steady frame不得做O(texel)工作 |
| ENV-P1-019 | frame submission cold path同步`fs::read`与decode/hydrate | derived-data I/O、decode与validation异步化；render thread只接受ready generation或last-good/typed fallback |
| ENV-P1-020 | hydration与pending bake都固定4项count cache | 统一cache按CPU bytes、GPU bytes、priority、last use、project/device generation和work budget管理 |
| ENV-P1-021 | pending bake无priority、deadline、cancel或dedup receipt | job scheduler支持coalescing、camera/editor/cook priority、cancel、deadline、retry/backoff和terminal status |
| ENV-P1-022 | readback后同步atomic writeback占用提交链 | writeback进入后台artifact commit queue，fence/readback ownership明确，shutdown和失败保持旧产物可读 |
| ENV-P1-023 | cubemap staging在uniform准备中直接`queue.submit` | 所有upload/copy/bake/capture command归入唯一frame submission或显式copy queue ticket，并返回fence receipt |
| ENV-P1-024 | prepared artifact缺失时回退多次`queue.write_texture` | 统一staging/ring uploader按byte/copy count预算批量提交，fallback也产生upload generation与overload receipt |
| ENV-P1-025 | source/PMREM尺寸与mip固定，未连接quality/capability/profile | resolution、format、mips、sample count和fallback由validated profile选择并进入artifact/device compatibility key |
| ENV-P1-026 | artifact兼容只围绕recipe，缺少完整device/runtime generation恢复 | device loss、format capability、shader revision与pipeline layout变化使相关GPU generation失效并可重建 |
| ENV-P1-027 | environment资源未接入统一texture streaming/residency/eviction | Runtime92 resource manager统一拥有CPU/GPU resident bytes、pinning、eviction、reload与last-good policy |
| ENV-P1-028 | cache hit/miss/bake/upload失败未汇入产品状态 | Editor、runtime diagnostics和capture证据可追踪request key、artifact、resident slot、submitted fence和published frame |

### 7.4 Realtime IBL状态机与预算

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| ENV-P1-029 | capture固定每帧2 face，与GPU timing和frame pressure无关 | scheduler按GPU timestamp、frame budget、visibility/importance和quality动态选择slice，保持最大延迟上界 |
| ENV-P1-030 | Partial：scheduler已区分recording/submission，按stage执行3次上限与1/2帧wrap-safe backoff，terminal同key抑制并保留last-good；runtime/framework已统一六态status snapshot；managed WGPU fault product证据仍缺 | 补真实record/submit fault injection、公开status query、cancel/supersede交互、PNG/RenderDoc与timing/power证据后关闭 |
| ENV-P1-031 | capture/downsample/IBL每slice创建uniform buffer与bind group | 使用persistent ring/dynamic offsets和generation-keyed bind-group cache，steady update无per-slice heap/GPU对象创建 |
| ENV-P1-032 | 固定双份source+PMREM+SH资源无byte budget | 资源进入environment residency pool，按resolution/format/job count计算预算并对压力给出可解释降级 |
| ENV-P1-033 | 第一次render调用隐式初始化资源并推进内部frame counter | runtime以外部frame/device/view generation驱动，初始化、camera cut、pause和多viewport不会伪造时间 |
| ENV-P1-034 | recorder仍保留“消费compiler顺序但记录culled pass”的未决路径 | graph pass culling、resource lifetime和实际encoder command一致，capture证明被裁pass产生0命令/0资源访问 |
| ENV-P1-035 | GPU timing只能drain，未反馈scheduler | timestamp report带generation和slice identity，被预算控制器消费；unsupported/late/stale report有明确处理 |
| ENV-P1-036 | realtime source只有`ProceduralSkyParams`，不含物理atmosphere/cloud | capture读取与主视图同一evaluated generation，并定义cloud/fog/translucency是否参与及相应cost/profile |

### 7.5 Reflection Probe assignment、upload与shader

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| ENV-P1-037 | CPU收集全场probe后裁64，fragment再线性扫描全部active probe | spatial index/GPU Scene先做view/cluster/tile assignment，fragment只读小型局部list并有global fallback |
| ENV-P1-038 | layer mask打包到`misc.w`但WGSL不读取 | receiver/probe/capture layer在CPU/GPU assignment与最终sample一致生效，64-bit/scene-schema降级有显式规则 |
| ENV-P1-039 | 固定64×128 RGBA16F probe array加planar约74.67 MiB | capacity/resolution/format/compression由device/profile/budget决定，资源按provider实际需求延迟、增长和回收 |
| ENV-P1-040 | `write_scene_uniform`期间`load_texture_asset`可触发同步resident read | probe dependency在prepare generation前异步resolve；uniform write只读ready slot或typed fallback，不阻塞I/O |
| ENV-P1-041 | 每个新probe执行48次`queue.write_texture` | cubemap mip chain进入统一batched uploader/copy encoder，copy count/bytes/fence与slot generation可观测 |
| ENV-P1-042 | 正式路径丢弃`_reflection_probe_upload_report` | active/rejected/overflow/cache hit/upload/failure/last-good逐probe进入frame readiness与Editor diagnostic |
| ENV-P1-043 | slot eviction只围绕cubemap/revision，缺少byte pressure与跨view稳定policy | allocator结合priority、screen influence、last use、capture mode、cost和hysteresis，slot reuse由generation保护 |
| ENV-P1-044 | probe selection缺少large-world、streaming cell和多view复用 | scene spatial owner发布稳定bounds/handles；每view assignment支持stream-in/out、origin shift、XR和camera cut |
| ENV-P1-045 | planar/local/global反射的覆盖顺序是硬编码，缺少统一reflection hierarchy | SSR/planar/local probe/global sky/HGI按confidence、roughness、visibility和blend policy合成并输出debug receipt |
| ENV-P1-046 | Baked/Custom/Realtime/OnDemand模式未持久化为产品contract | scene schema、cook policy、runtime refresh、cross-fade、capture budget和artifact ownership完整可达 |

### 7.6 Capture、tests、fault与性能资格

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| ENV-P1-047 | 插件计划顺序clone六份snapshot、六次HDR readback、CPU bake并同步stage | capture成为GPU job：复用snapshot generation、分片六面、GPU convolution、最小readback、progress/cancel/atomic commit |
| ENV-P1-048 | 产品测试手工注入EnvironmentExtract，capture test ignored，无scale/fault/benchmark | 真实scene/editor入口覆盖save/reopen/cook/WGPU pixel、64+ probe pressure、cache cold/warm、fault/device-loss与跨引擎基准 |

## 8. P2演进项

| ID | 演进项 | 启动前提 |
|---|---|---|
| ENV-P2-001 | 多行星/轨道尺度atmosphere与从地面到太空连续渲染 | 单行星LUT、large-world transform与精度资格先闭合 |
| ENV-P2-002 | spectral atmosphere、ozone/aerosol SPD与wavelength-aware exposure | RGB物理单位、color management和reference baseline先成立 |
| ENV-P2-003 | 多层体积云、weather simulation、precipitation与云影temporal reconstruction | 基础cloud scene contract、budget/history/disocclusion先合格 |
| ENV-P2-004 | 月相、星表、银河、极光与天文时间系统 | shared celestial generation、曝光与地平线遮挡先稳定 |
| ENV-P2-005 | BC6H/ASTC HDR、octahedral atlas、sparse/bindless probe storage | format capability、residency、gutter和quality parity先闭合 |
| ENV-P2-006 | GPU probe binning、bindless descriptor与indirect update | GPU Scene、cluster list、overflow feedback和device tier先合格 |
| ENV-P2-007 | 动态probe temporal reuse、partial face refresh与importance prediction | 正确六面job、generation/cross-fade和ghosting tests先成立 |
| ENV-P2-008 | hardware ray-traced reflection probe补洞与hybrid confidence | TLAS/SBT/capability/fallback及非RT probe baseline先闭合 |
| ENV-P2-009 | distributed/offline farm IBL与reflection capture bake | deterministic artifact key、transaction、cook provenance和remote failure contract先成立 |
| ENV-P2-010 | probe streaming cells、world partition预取与跨cell共享 | stable handle、spatial owner、byte budget和origin shift先合格 |
| ENV-P2-011 | path-traced sky/IBL/probe reference oracle与自动像素差分 | material/exposure/camera/capture provenance稳定后启动 |
| ENV-P2-012 | learned sky/cloud/probe compression或reconstruction实验 | 正确非神经baseline、模型provenance、fallback、平台成本与许可证据先成立 |

## 9. 实施里程碑

### M96-1：修复P0并冻结产品characterization

- 用failing workspace type-check锁定删除API断裂；
- 定义RenderFramework-owned capture job contract，禁止公开内部SceneRenderer兼容方法；
- 将现有IBL/probe/realtime tests整理为可迁移characterization。

### M96-2：Scene Environment与Probe source

- 增加versioned scene component/asset/property/migration；
- World建立environment authority、volume blend与shared sun generation；
- 真实save/reopen/cook/runtime roundtrip替代测试手工注入。

### M96-3：物理Atmosphere与共享Environment Evaluation

- 实现transmittance、multiple scattering、sky view和aerial perspective LUT；
- sky、fog/cloud、IBL capture与directional sun统一evaluation；
- 建物理单位、reference fixture和quality/capability profile。

### M96-4：Derived Data与Resident Generation

- frame DTO硬切换为handle/generation/readiness；
- 异步read/decode/bake/writeback与byte-budget cache；
- 接入Runtime92 residency、eviction、reload和device-loss重建。

### M96-5：统一Upload与Submission

- 所有cubemap/probe/LUT上传归入batched uploader和唯一submission owner；
- 删除uniform prepare中的私有submit与48次write fallback；
- fence-qualified publication与stale generation拒绝闭合。

### M96-6：预算驱动Realtime IBL

- 在现有双缓冲/分片/compiled cache基础上增加timing feedback、priority、cancel/backoff/terminal；
- persistent ring和bind-group cache消除slice churn；
- physical atmosphere/cloud capture与main view parity。

### M96-7：GPU Probe Assignment与Reflection Hierarchy

- spatial/GPU Scene产生per-view/per-cluster probe list；
- layer、priority、hysteresis、overflow与slot generation闭合；
- SSR/planar/local/global/HGI按confidence统一合成。

### M96-8：Editor Capture/Bake产品闭环

- inspector/gizmo/command、progress/cancel、undo/redo、scene dirty；
- GPU六面capture/convolution job、atomic artifact commit与cook dependency；
- Baked/Custom/Realtime/OnDemand及cross-fade完整可达。

### M96-9：Fault、scale与跨引擎资格

- 完成cache cold/warm、64/256/1K probe、streaming、multi-view、device loss/OOM和soak矩阵；
- 采集CPU/GPU/RAM/VRAM、I/O、upload bytes/copies、capture latency和pixel error；
- 同硬件、同分辨率、同曝光、同sky/probe质量对照Unreal，Unity/Bevy/Godot/Fyrox用于结构与fallback差分。

## 10. 资格门

| Gate | 必须通过的证据 |
|---|---|
| ENV-G01 | Environment/Sky/Atmosphere/Cloud source可save、reopen、prefab、cook并进入World |
| ENV-G02 | Reflection Probe shape/mode/resolution/layers/artifact可roundtrip且migration有版本证据 |
| ENV-G03 | viewport preview override不改变scene序列化或runtime environment truth |
| ENV-G04 | Editor capture command支持selection、transaction、undo/redo、progress、cancel和scene dirty |
| ENV-G05 | artifact/registry/component/cook dependency原子提交，任一步失败可回滚 |
| ENV-G06 | 多environment/volume/multi-camera/streamed scene authority选择确定且可诊断 |
| ENV-G07 | sky、directional sun、fog/cloud、exposure和IBL读取同一generation |
| ENV-G08 | readiness能从source追踪到artifact、resident、submitted fence和published frame |
| ENV-G09 | atmosphere reference参数下transmittance与sky radiance在规定数值/像素容差内 |
| ENV-G10 | multiple-scattering LUT key、size、samples与profile变化只触发必要重建 |
| ENV-G11 | camera height、planet transform、ground/space边界无跳变或NaN |
| ENV-G12 | aerial perspective在forward/deferred/fog/translucency路径数值一致 |
| ENV-G13 | cloud density/weather/wind/lighting/shadow均来自scene generation且受预算控制 |
| ENV-G14 | celestial/time变化触发准确invalidations，不全量重建无关资产 |
| ENV-G15 | sky radiance、sun illuminance、IBL intensity和camera exposure单位闭合 |
| ENV-G16 | 主视图sky与IBL capture在相同generation下满足方向/颜色/亮度parity |
| ENV-G17 | frame packet不再携带source/PMREM整份texel或prepared byte chain |
| ENV-G18 | zero-change extract与frame submit执行0 texel encode、0磁盘I/O和0 artifact decode |
| ENV-G19 | cold cache I/O/decode异步，render thread不阻塞且last-good/fallback可解释 |
| ENV-G20 | cache按CPU/GPU bytes、priority和generation限制，压力下无无界增长 |
| ENV-G21 | bake request dedup/priority/cancel/backoff/terminal及shutdown drain均通过 |
| ENV-G22 | artifact writeback失败保持旧正式产物完整，临时事务可清理 |
| ENV-G23 | environment upload不在uniform prepare私自`queue.submit` |
| ENV-G24 | uploader报告bytes/copies/fence/slot generation并满足固定每帧预算 |
| ENV-G25 | resolution/format/mips/sample count按device/profile选择且进入compatibility key |
| ENV-G26 | device loss/format或shader revision变化只重建相关GPU generation |
| ENV-G27 | environment/probe资源参与统一streaming pin/evict/reload/last-good policy |
| ENV-G28 | cache/bake/upload所有failure与degrade能在Editor/runtime diagnostics定位 |
| ENV-G29 | realtime slice随GPU timing/frame pressure调节且有最大完成延迟上界 |
| ENV-G30 | realtime失败达到terminal或backoff，不永久Retry/占slot；last-good保持可见 |
| ENV-G31 | steady realtime update无per-slice buffer、bind-group或字符串heap churn |
| ENV-G32 | 双缓冲source/PMREM/SH受byte budget管理，压力降级有typed receipt |
| ENV-G33 | pause、多viewport、camera cut和device generation不会误推进内部frame identity |
| ENV-G34 | 被graph裁掉的realtime pass产生0 encoder command和0资源访问 |
| ENV-G35 | timestamp report按slice/generation反馈scheduler，late/stale report被拒绝 |
| ENV-G36 | physical atmosphere/cloud参与或排除capture的policy明确且主视图parity可验收 |
| ENV-G37 | GPU/per-cluster probe list与CPU oracle在随机box/sphere场景等价 |
| ENV-G38 | receiver/probe/capture layer mask在assignment和final sample一致生效 |
| ENV-G39 | probe资源capacity/format/resolution按budget增长回收，feature-off保持placeholder |
| ENV-G40 | uniform write路径执行0同步asset load/read/decode |
| ENV-G41 | 新probe mip chain通过批量copy上传，不再固定48次`queue.write_texture` |
| ENV-G42 | 每probe accepted/rejected/overflow/cache/upload/failure进入frame receipt |
| ENV-G43 | slot pressure下priority/hysteresis稳定，删除/复用后stale generation不可采样 |
| ENV-G44 | large-world origin shift、stream cell、多view/XR下probe bounds与identity稳定 |
| ENV-G45 | SSR/planar/local/global/HGI hierarchy按confidence/roughness连续合成，无硬覆盖跳变 |
| ENV-G46 | Baked/Custom/Realtime/OnDemand capture及cross-fade在scene/cook/runtime均可达 |
| ENV-G47 | capture复用snapshot generation、GPU分片卷积、最小readback、cancel与atomic commit |
| ENV-G48 | 同硬件同画质Unreal基准记录revision、场景、曝光、quality、CPU/GPU/RAM/VRAM和pixel provenance |

## 11. 参考引擎映射与适用性

| 目标能力 | 主参考 | 可迁移原则 | 不应机械复制 |
|---|---|---|---|
| 工程级skylight/probe capture | Unreal `SkyLightComponent`、`ReflectionEnvironmentCapture.cpp` | dirty queue、runtime mode、timeslice、budget/hysteresis、teleport、smooth blend、capture cache与array slot remap | UE UObject/RDG/RHI对象不能直接映射为Rust public API；先固定Zircon owner和generation |
| 物理atmosphere/cloud | Unreal `SkyAtmosphereRendering.cpp`/`VolumetricCloudRendering.cpp`，Unity HDRP PBR Sky | 多级LUT、celestial light hash、aerial perspective、cloud shadow/temporal与quality controls | 不照搬全部CVars；参数必须进入versioned scene/profile和可测试key |
| Sky/IBL manager与cache | Unity HDRP `SkyManager`、`PhysicallyBasedSkyRenderer`、`ReflectionProbeTextureCache` | update mode、static/dynamic sky、precomputation hash/refcount cache、atlas LRU/hash、filter/convolution | Unity native engine不可见部分不能推断；SRP源码只用于可见责任链 |
| Rust/wgpu可移植实现 | Bevy atmosphere与light_probe | ECS source、render-world extract、capability gate、四类LUT render graph、GPU environment generation、per-view probe uniform | Bevy固定数组/系统调度不是性能上限；Zircon仍需byte budget、GPU assignment和fault receipt |
| 场景产品完整性下限 | Godot WorldEnvironment/Environment/Sky/ReflectionProbe | 正式resource/node、property binding、update mode、reflection mask、roughness filter与renderer server边界 | Godot的固定模式与RD细节不替代Zircon generation/submission架构 |
| Rust场景节点下限 | Fyrox ReflectionProbe/SkyBox | serializable node、builder、once/each-frame、force update、render target recreation、resource validation | Fyrox简单point-box单probe选择只作为最低完整性参照，不是规模/表现目标 |

Unreal最适合作为最终工程规模、capture budget、资源缓存和画质上限的主参考；Unity HDRP用于验证sky/probe product manager与atlas cache；Bevy用于验证Rust/wgpu的可实现边界；Godot与Fyrox用于证明场景资源、节点和Editor可达性不能省略。任何“优于Unreal”结论都必须来自同场景、同画质、同硬件的可重复数据，而不是功能名称或单个microbenchmark。

## 12. 禁止的临时修法

1. 禁止只给`SceneEntityAsset`加一个`skybox`或`probe`字段，却不完成schema version、property、save/reopen、World和cook。
2. 禁止为了让插件编译重新公开`SceneRenderer::render_scene_color_hdr`或加compat wrapper；capture必须通过稳定job/service边界。
3. 禁止把viewport preview gradient当正式scene environment，或让Editor状态覆盖runtime truth。
4. 禁止继续把source/PMREM整份内容塞进frame DTO，仅以`Arc`浅拷贝为理由宣称生命周期已正确。
5. 禁止在frame submission、uniform prepare或render thread同步读盘、decode、bake或atomic write。
6. 禁止在resource prepare中私自`queue.submit`，也禁止以提高每帧write次数掩盖统一uploader缺失。
7. 禁止仅提高64 probes、128 face或固定cache 4项常量来掩盖budget、residency和assignment缺失。
8. 禁止保留CPU裁64加fragment扫描64的热路径，再用top-two结果宣称probe clustering已完成。
9. 禁止打包layer mask、upload report或timing report却不让最终consumer和scheduler读取。
10. 禁止用source-string tests或ignored manual capture替代真实scene/editor入口、workspace type-check、WGPU pixel与fault证据。
11. 禁止在失败时无限`Retry`或静默回退默认gradient，同时把readiness标成成功。
12. 禁止在没有同画质同硬件provenance时写“表现或性能优于Unreal”。

## 13. 状态

本文是review与重构计划，不是实现完成声明。Runtime09F1的10项P0继续由原报告唯一计数；其中旧`CaptureCloud`重复证据已废止，但对应物理天空P0未关闭。Runtime96新增1项P0、48项P1、12项P2和48个资格门。ENV-P1-030现为源码实现、六态status硬切与独立production-scheduler行为验证完成，managed fault/status product验证未完成的`Partial`，不能据此前移M96。实施必须按M96-1至M96-9推进，先恢复workspace contract和真实authoring可达性，再迁移资源生命周期、物理天空、GPU assignment和产品证据。

`source_recheck_required: true`在以下条件全部满足前不得改为false：131文件语料fingerprint重算一致或差异已审计；当前20个相关working-tree修改完成归属和合并复查；reflection-probes workspace package有真实type-check证据；Runtime09F1父P0与本文资格门均有failing/passing evidence；Editor、真实GPU、fault、scale和跨引擎基准形成带revision、hardware与capture provenance的验证记录。
