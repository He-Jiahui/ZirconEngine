---
title: Runtime Hybrid Global Illumination、Scene Representation、Surface Cache、Global SDF、Screen Probe、Radiance Cache 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime98
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors
  - zircon_plugins/hybrid_gi/editor/src
  - zircon_plugins/hybrid_gi/dist/src/lib.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/prepared_runtime_sidebands.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
tests:
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_probe_trace_tiles/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_radiance_cache/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors/resolve_trace_handoff/tests.rs
  - zircon_plugins/hybrid_gi/runtime/tests/hybrid_gi_m4_invalidation_profiles.rs
  - zircon_plugins/hybrid_gi/runtime/tests/hybrid_gi_m4_profile_matrix_wgpu.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_product_advanced.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_editor/src/ui/retained_host/viewport/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/95-runtime-direct-lighting-photometry-light-grid-clustered-forward-plus-shadow-atlas-cascade-point-spot-rect-cookie-ies-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/97-runtime-baked-lighting-lightmap-probe-volume-bake-job-artifact-residency-sampling-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_plugins/19-first-party-hybrid-gi-source-runtime-editor-dist-catalog-scene-representation-surface-cache-global-sdf-radiance-cache-probe-trace-denoise-product-integration-review.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSceneData.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenMeshCards.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSceneCardCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCacheFeedback.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeGather.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeTracing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeFiltering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenRadianceCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenRadianceCacheHardwareRayTracing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenHardwareRayTracingCommon.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/Lumen/LumenScreenProbeTracing.usf
  - dev/UnrealEngine/Engine/Shaders/Private/Lumen/LumenRadianceCacheUpdate.usf
  - dev/UnrealEngine/Engine/Shaders/Private/Lumen/SurfaceCache/LumenSurfaceCacheSampling.ush
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume/ProbeVolumeLightingTab.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/UnifiedRayTracing/RayTracingContext.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/ScreenSpaceGlobalIllumination.cs
  - dev/godot/servers/rendering/renderer_rd/environment/gi.cpp
  - dev/godot/servers/rendering/renderer_rd/environment/gi.h
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/sdfgi_preprocess.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/sdfgi_integrate.glsl
  - dev/godot/editor/scene/3d/voxel_gi_editor_plugin.cpp
  - dev/godot/editor/scene/3d/gizmos/voxel_gi_gizmo_plugin.cpp
  - dev/bevy/crates/bevy_pbr/src/light_probe/environment_map.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.rs
  - dev/Fyrox/fyrox-impl/src/renderer/gbuffer.rs
  - dev/Fyrox/fyrox-impl/src/renderer/light.rs
  - dev/Fyrox/fyrox-impl/src/renderer/mod.rs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Hybrid Global Illumination、Scene Representation、Surface Cache、Global SDF、Screen Probe、Radiance Cache 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon HGI不是空壳。它已经形成独立first-party package、typed provider、四阶段render graph、Mesh SDF artifact、camera-snapped Global SDF clipmap、Surface Cache与voxel scene state、screen probe、Radiance Cache、software trace、temporal resolve、readback sideband和大量CPU/WGPU characterization tests。`HybridGiGpuResources`也持久化了一部分pipeline与Global SDF/Radiance Cache资源。这些实现足以作为研究原型和迁移oracle，不应全部推倒重来。

但产品路径的有效信息仍只有固定8x8、即64个screen/HZB tile。Resolve pass确实写入全viewport纹理，因此旧09F3的“最终输出纹理只有8x8”表述需要收紧；问题在于每个全分辨率像素仍从64个tile的深度、法线、命中、radiance和3x3邻域重建，空间信息量并未随viewport增长。Surface Cache每页只有中心UV生成的`atlas_sample_rgba`、`capture_sample_rgba`和量化depth，不是捕获材质表面；每个mesh只生成一个球形card，每个card最多映射一个screen probe；Visibility函数接收HGI extract后仍明确返回空probe、空update plan、空feedback和空request。

两套GI所有权仍同时存在。插件graph生成current GI texture与history，core post-process又保留16个probe、16个trace region、独立screen-space径向混合、source ledger与history组合。插件prepare完成后把cache、Surface Cache、voxel、Global SDF、Radiance Cache、trace lighting等GPU结果大范围readback到CPU，collector在全局Mutex内同步并重新封装，下一帧再上传。Radiance Cache固定最多32个probe，GPU TRACE阶段把同一个RGB8常量写满每个probe的2x2 interior；Global SDF miss/fallback可制造蓝灰色，不具备可信surface lighting。源码中没有ray query、acceleration structure、BLAS/TLAS、SBT或dispatch rays实现，Hardware RT仍只是能力枚举和数据字段。

产品闭环同样未建立。Scene/Prefab/Project没有HGI authoring component或持久化settings；World每帧只注入disabled default，Editor viewport再强制开启并写死32/64/16预算。Editor插件引用不存在的`plugins://hybrid_gi/editor/authoring.zui`，dist carrier明确声明执行仍由source runtime承载。`plugin.toml`诚实标记`experimental/partial`，但Editor默认启用造成能力声明与默认产品行为冲突。239个test attributes中21项ignored；至少26个WGPU测试在adapter不可用时直接`return`，另有2条`Option`路径返回None，103处`.contains(`又表明大量测试只锁定源码字符串。

旧09F3登记的 **14项P0全部保持开放**，本报告不重复新增父P0；新增 **36项P1、8项P2与44个资格门**。本轮同时记录一个窄幅正向变化：当前working tree把Surface Cache assigned-slot membership从线性扫描改为`BTreeSet`并增加性能gate，净增162行、删除4行；它只关闭局部O(n²) membership热点，不关闭整帧clone、CPU权威、伪Surface Cache或residency问题。完成scene truth、单一owner、GPU resident representation、真实surface capture、分层trace、方向Radiance Cache、Editor运维和竞争性验收前，不得声称HGI达到或优于当前Unreal Lumen。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / bytes / test attributes / ignored | 证据等级 | fingerprint |
|---|---:|---|---|
| HGI package全部tracked文件 | **270 / 42,539 / 1,585,115 / 239 / 21** | E3覆盖manifest、runtime、shader、tests、Editor与dist | `a52c3c99e4215c4b680e503c454fe6e511c6d019753238bf4909bbc087780d8a` |
| runtime production-like源码 | **221 / 29,964 / 1,105,137 / 98 / 1** | E3逐文件覆盖provider、representation、GPU lifecycle、16个WGSL shader与四pass executor | `5a52d50e99cf296382c8085b6710fd8f80485b921cabc479620cccb90f039f9b` |
| tests、test_sources、test_support与外部test | **39 / 12,241 / 468,626 / 138 / 20** | E3读取CPU、WGPU、PNG exporter、profile、invalidation与product fixtures | `3e2b35f0129c2b14a82db573303dcdd35ceaec2c5e247d2f26934f2e0674afb7` |
| package manifest、runtime manifest、Editor与dist | **10 / 334 / 11,352 / 3 / 0** | E3读取capability、carrier、extension和默认装配边界 | `0c0d3e7ad7379785937d936afd56ea1ec6ddecb69290be1b85a73ca344ecfa0e` |
| 五引擎参考切片 | **35 / 36,091 / 1,562,537 / 未归一 / 未归一** | E2/E3读取Lumen、Unity APV/SSGI/RT、Godot SDFGI、Bevy probe与Fyrox deferred owner | `89670ea60492310877cc82a0d96b1a127cbcac1a51bd633511eafb3b2b9a2322` |

fingerprint算法为：相对路径与每文件SHA-256组成排序manifest，以TAB分隔字段、LF分隔记录，再对UTF-8 manifest执行SHA-256。行数按PowerShell `Get-Content`逻辑统计。冻结对象是2026-08-21共享working tree，不是只读HEAD；基线HEAD为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator epoch为336。

Bevy、Fyrox、Godot与Unity Graphics revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal不是独立Git checkout，只能用上述参考aggregate fingerprint冻结，禁止把父仓revision伪装成Unreal revision。

冻结时`surface_cache_state.rs`由其他Session持有并存在working-tree修改，当前blob hash为`386db05acfb96e58d010533983df409d2f4254a0`。本文只读取并记录这项局部优化，没有修改或拥有该源码；实现和动态验收前必须重算全部fingerprint。

### 2.2 机械扫描边界

production-like 221文件中有0个TODO/FIXME/HACK/XXX，5个`panic!/todo!/unimplemented!`，9个`.unwrap(`、33个`.expect(`、13个`#[allow(`、25个`include_str!`，以及18个500行以上文件。直接WGPU调用至少包括8处buffer create、8处texture create、9处pipeline create、9处bind-group create、11处queue write、5处GPU-to-buffer copy、2处`map_async`和1处Mutex lock。零TODO不是完整度证据；它只说明临时实现被写成了正式代码形态。

### 2.3 Owner边界

| 边界 | Runtime98要求的owner | 禁止的越界 |
|---|---|---|
| Scene truth | `zircon_runtime::scene`拥有可序列化HGI participation、quality/profile、static/dynamic contribution与authoring source | 不由Editor viewport覆写默认值；不把render extract当持久化truth |
| Neutral contract | `core::framework::render`只携typed、versioned、generation-qualified descriptor/handle/readiness | 不携raw page RGB8、voxel cells或WGPU资源；不把readback payload当scene representation |
| Feature implementation | HGI rendering plugin唯一拥有scene representation、trace、cache、resolve与history | core post-process不得再实现第二套probe/trace/history解释 |
| Runtime graphics | graphics拥有GPU resources、render graph execution、residency、fence、device恢复和最终composition attachment | 不让CPU collector成为GPU algorithm authority；不以readback驱动正常下一帧输入 |
| Editor product | `zircon_editor`拥有真实authoring、debug view、capture、预算、operation、diagnostic和save/reopen | 不注册不存在的template；不以字符串详情冒充工具 |
| 历史P0 | 旧09F3继续唯一计数14项父P0；Runtime98只做current-source重验与拆解 | 不因同一根因跨Plugin/Runtime/Editor可见而重复累计P0 |

### 2.4 明确未做

本轮只做静态review，没有修改Rust、WGSL、Cargo、plugin manifest、scene asset或Editor UI，没有运行Cargo、Editor/App、WGPU测试、真实GPU、RenderDoc/PIX、cook/export、device loss、OOM、large-world、visual golden、24小时soak或性能基准。静态源码可证明生产链空桥、固定容量、数据所有权、回读循环和测试跳过，不能替代最终画质和GPU时间证据。

## 3. 当前应保留的真实基础

1. first-party package、typed provider、capability report和四pass graph是正确的feature边界起点，应升级而非改回core硬编码。
2. `RenderHybridGiExtract`的mode/profile/quality/budget/debug view、resolved settings和baked fallback可以迁移为持久化authoring与运行时policy contract。
3. participation epoch、generation、compare-and-commit、last-good prepared frame与bounded pending queue体现了正确生命周期意图。
4. imported Mesh SDF artifact、camera-snapped Global SDF clipmap、page candidate与GPU build是真实进展，可作为software trace底座。
5. Surface Cache stable slot与feedback card ID已有可测试状态机；本轮BTreeSet membership优化可保留，但需要迁移到GPU virtual residency。
6. trace capability graph、source mask、fallback reason与大量runtime statistics适合升级为正式route controller和telemetry schema。
7. resolve已有scene depth、normal、velocity、history signature和debug mode输入，可作为真正full-resolution reconstruction的characterization oracle。
8. CPU/WGPU fixtures覆盖Global SDF、Surface HZB、multi-ray、Radiance Cache与temporal rejection；应保留为低层oracle，并删除“无adapter即成功”的测试语义。

## 4. 历史09F3 P0 current-source重验

| 父finding | 当前状态 | current-source证据与关闭要求 |
|---|---|---|
| 09F3-P0-1 固定8x8 proxy | 开放但修正文案 | 输出attachment是viewport尺寸，但scene/trace packet只有8x8、64 tile，full-res resolve只重建这些tile。关闭需随internal resolution扩展的probe/tile布局和真实画质/耗时曲线 |
| 09F3-P0-2 Surface Cache没有捕获surface | 开放 | 每页只有中心UV RGBA8 atlas/capture样本与量化depth，没有albedo/normal/emissive/opacity/material response atlas。关闭需真实card raster/capture、mip、gutter与lighting update |
| 09F3-P0-3 one mesh/one card/one probe | 开放 | `build_card_descriptors`每mesh一个card；probe按card顺序截取，中心和半径直接复制。关闭需离线多卡参数化、view/depth驱动adaptive placement与稳定identity |
| 09F3-P0-4 Visibility桥为空 | 开放 | `_hybrid_global_illumination`未使用，active probes、update plan、feedback、requested probes全部构造为空。关闭需Visibility/GPU Scene generation到HGI demand的真实consumer |
| 09F3-P0-5 plugin/core双GI与双history | 开放 | plugin四pass写GI/history，core post-process仍上传16 probe/16 region并执行独立径向混合、ledger与history。关闭需单一feature owner和单一composition contract |
| 09F3-P0-6 GPU readback/CPU repack/reupload | 开放 | pending readback仍包含cache、completion、atlas、capture、voxel、RC、trace lighting等；collector在全局Mutex内更新CPU state。关闭需GPU resident authority和只读稀疏诊断回读 |
| 09F3-P0-7 Radiance Cache缓存RGB8常量 | 开放 |最多32 probe；TRACE将同一`radiance_confidence`写入2x2 interior，消费依赖8角全命中。关闭需方向radiance、visibility/occlusion、probe relocation和GPU trace/filter lifecycle |
| 09F3-P0-8 Global SDF无可信surface lighting | 开放 | Global SDF route可从lineage RGB取色，无lineage时制造蓝灰色；不是命中材质/Surface Cache lighting。关闭需hit identity到material/surface radiance的稳定查询 |
| 09F3-P0-9 Hardware RT是假表面 | 开放 | production中没有ray query、acceleration structure、BLAS/TLAS、SBT或dispatch rays。关闭需backend capability、AS build/update/compaction、dispatch、fallback与device matrix |
| 09F3-P0-10 小容量与silent truncation | 开放 | Surface pages、RC probes、core probes/regions、Global SDF pages与trace steps均为小固定上限，多个路径使用`take/min/break`。关闭需预算控制、优先级、overflow receipt、degradation和scale证据 |
| 09F3-P0-11 失效图不精确 | 开放 | 任一相关light变化可标记全部Surface Cache page dirty；geometry/material/light/transform/LOD依赖未形成generation图。关闭需typed dependency key与局部recapture/retrace |
| 09F3-P0-12 baked/static/dynamic能量边界不可信 | 开放 | source ledger有所增强，但依赖Runtime97尚未完成的baked contract；core/plugin仍可分别组合。关闭需唯一light ownership、shadowmask/delta语义与能量golden |
| 09F3-P0-13 Editor产品面缺失 | 开放 | Editor viewport强制开启，插件引用不存在的`authoring.zui`，只有诊断字符串。关闭需持久化authoring、debug overlay、capture、budget、operation与save/reopen |
| 09F3-P0-14 proxy证据冒充产品证明 | 开放 | 21项ignored，至少26个adapter缺失直接返回，2条Option返回None，103处source contains。关闭需required hardware matrix、golden、fault/scale/soak和竞争基准 |

## 5. 当前产品数据链与断点

```text
Scene/Prefab HGI authoring ----X----> RenderHybridGiExtract
                                        ^
World render extract -> disabled default|
Editor viewport -------> force enabled + fixed 32/64/16
                                        |
                                        v
typed provider -> CPU representation -> GPU prepare -> broad readback
                    |                       |               |
                    |                       v               v
                    |                 plugin 4-pass <- CPU repack
                    |                       |
                    |                       v
                    +---------------> viewport GI texture/history
                                            |
                                            v
                              core post-process 16 probe/16 region

Visibility demand --------X--------> HGI probe/page request
real material surface ----X--------> Surface Cache capture
BLAS/TLAS/RT backend ------X--------> hardware trace
Editor authoring.zui -----X--------> usable product tool
```

目标必须是“scene truth -> immutable RenderScene generation -> GPU demand/residency -> material-correct surface representation -> hierarchical trace -> directional radiance cache -> full-resolution reconstruction -> single composition”，不能继续给CPU readback DTO增加字段。

## 6. P1重构项

### 6.1 Contract、Scene truth与identity

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-01 | `experimental/partial` package在Editor默认profile中被强制启用，capability与默认产品行为冲突 | 建立feature admission policy，实验能力默认关闭；required profile必须验证provider/backend/evidence后才能启用 |
| P1-02 | Scene/Prefab/Project没有HGI settings、participation或source persistence，World只创建default extract | 建立可序列化`HybridGiSettingsComponent`、per-object participation、project defaults、migration及save/reopen/cook链 |
| P1-03 | mesh/card/probe/page/trace region identity主要来自序号、card ID或量化payload，缺少asset revision/LOD/material generation | 定义`RenderSceneGeneration + GeometrySubjectId + SurfaceCardId + ProbeId + ResidentGeneration`稳定身份体系 |
| P1-04 | mode/profile预算可被Editor环境变量和硬编码数字覆盖，不进入project/scene truth | 把profile recipe、override层级、platform/backend约束和effective receipt纳入正式配置与资产合同 |
| P1-05 | participation、baked ownership、dynamic contribution和emissive contribution没有统一source ledger | 建立per-light/per-primitive contribution mask与唯一energy owner，贯穿baked、dynamic、surface capture和composition |
| P1-06 | dist carrier无状态、schema0且执行仍由source module托管，source/native/export parity不可证明 | 定义稳定native runtime carrier、versioned state/command/diagnostic ABI与export smoke matrix |

### 6.2 Scene representation、card与visibility

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-07 | 每mesh只产生一个以translation为中心、max scale推导半径的球形card | 建立离线/导入card builder，按surface orientation、coverage、material section和LOD生成多卡及保守bounds |
| P1-08 | production `synchronize_cards`仍可走placeholder mesh，card没有真实triangle/UV/section coverage | hard cutover为typed mesh card artifact，禁止生产fallback伪几何，失败必须有diagnostic与feature degradation |
| P1-09 | screen probe直接按card顺序`take(trace_budget)`，不看view depth、normal discontinuity或screen coverage | 建立screen-space base grid、adaptive probes、importance、disocclusion和稳定temporal identity |
| P1-10 | Visibility接收HGI extract但构造空probe/update/feedback/request，历史也只记录空集合 | 建立Visibility/GPU Scene到HGI的generation-qualified demand、visible card、trace subject与feedback bridge |
| P1-11 | representation每帧clone/sort/比较大量Vec、BTreeMap/BTreeSet，局部BTreeSet优化未改变整体复杂度 | 采用immutable generation、dirty range、arena/slab与增量GPU upload，给10K/100K对象建立CPU预算 |
| P1-12 | deformation、skinning、morph、foliage、heightfield、instancing、translucency和two-sided策略缺失 | 为各geometry class定义representation、update frequency、fallback、memory budget和unsupported receipt |

### 6.3 Surface Cache、capture与feedback

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-13 | 每页仅保存两个RGBA8中心样本和量化depth，无法表示surface variation | 建立多channel physical atlas：base color、normal、emissive、opacity、depth/coverage、lighting及HDR格式策略 |
| P1-14 | card capture没有真实raster/material evaluation、alpha test、two-sided、WPO/deformation或texture residency | 建立capture draw packet、material permutation、resource readiness、partial failure和recapture reason |
| P1-15 | one-card-one-page residency只按card顺序和page budget，overflow仅返回card ID | 建立virtual page table、mip、view/feedback priority、LRU/clock replacement、pinning、overflow receipt和fence |
| P1-16 | feedback不来自最终采样访问，也没有miss frequency、desired mip或screen contribution | 在采样shader产生GPU feedback并compaction/readback摘要，驱动下一代page demand而非全量CPU状态 |
| P1-17 | 任一相关light改变可把全部resident pages标dirty | 建立geometry/material/texture/light/emissive/transform/LOD依赖图和局部capture/lighting invalidation |
| P1-18 | capture/atlas/depth内容经GPU readback进入CPU，再成为后续graph packet | 保持page table、atlas、capture、lighting与completion在GPU；CPU只接收稀疏统计和错误receipt |

### 6.4 Mesh SDF、Global SDF、voxel与trace backend

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-19 | Mesh SDF虽有artifact，但缺少所有geometry class、platform format、streaming、revision和build provenance闭环 | 建立content-addressed SDF build artifact、cook variant、streaming residency和mesh/LOD/material revision依赖 |
| P1-20 | Global SDF虽camera-snapped，但固定4 clipmap、64 cells/edge、128 pages，candidate/overflow策略不足 | 建立设备/quality预算驱动的clipmap/page layout、scroll、clear、priority、overflow和large-world origin支持 |
| P1-21 | Global SDF commit仍由GPU completion readback后CPU `commit_pages`驱动 | 使用GPU page table generation、indirect build、fence与atomic publish；CPU不参与正常page commit |
| P1-22 | SDF trace最多16步、nearest cell/粗normal，hit lighting可来自lineage或伪颜色 | 建立distance-aware stepping、robust normal、thin geometry处理、hit identity和Surface Cache/material radiance lookup |
| P1-23 | voxel fallback以scene bounds为中心、最多8 clipmap且每层只有4³ cells，CPU生成RGB8 radiance | 改为camera/world-partition centered GPU clipmap或删除该fallback，禁止它伪装产品级GI route |
| P1-24 | trace route没有完整screen trace -> Mesh SDF -> Global SDF -> HRT -> sky hierarchy与一致miss contract | 建立统一ray record、route policy、continuation/miss reason、backend capability与跨route数值oracle |

### 6.5 Radiance Cache、lighting与reconstruction

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-25 | RC最多32 probe、4 clipmap，demand从card probe截取；采样要求8角全部存在否则整体miss | 建立camera/consumer-driven sparse probe allocation、partial interpolation、confidence、relocation和scroll |
| P1-26 | 每probe只有RGB8+confidence，GPU TRACE把同一值写满2x2 interior | 存储directional radiance/SH或octahedral map、depth/visibility/variance，并由真实rays写入不同texel |
| P1-27 | RC sample和trace lighting由CPU snapshot构造并回灌prepared frame | 让trace、filter、border、mip、history、allocation与consume全程GPU resident，CPU不拥有radiance真值 |
| P1-28 | emissive propagation、multi-bounce、sky/environment、direct light和baked delta没有统一transport模型 | 定义radiometric units、bounce policy、clamp/firefly、emissive injection、environment miss和energy ledger |
| P1-29 | full-res resolve只从固定64 tile做3x3 filter与单velocity reprojection，固定±0.25 clamp | 建立internal resolution策略、depth/normal/material/roughness rejection、variance-guided filter、disocclusion和history length |
| P1-30 | debug fallback使用硬编码橙/红/蓝灰颜色，并可进入普通输出语义 | debug visualization必须与shipping lighting attachment隔离；fallback输出携typed reason，不得制造可计能量 |

### 6.6 Composition、GPU lifetime与调度

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-31 | plugin graph与core post-process共同解释probe、trace、source ledger、GI texture和history | hard cutover为HGI plugin唯一owner；core只消费一个generation-qualified indirect-light attachment |
| P1-32 | execute prepare仍逐帧创建大量buffer/texture/bind group，多个Vec全量清零 | 建立persistent resource pool、transient graph alias、descriptor cache、capacity growth和retirement fence |
| P1-33 | pending readback由一个全局Mutex collector管理，front-only ready造成head-of-line blocking | 按renderer/device/view实例隔离queue，completion按ticket收割；诊断回读独立限流且不阻塞frame pipeline |
| P1-34 | `AsyncCompute`只是标签，没有queue overlap、barrier、timestamp或收益证明 | 由render graph compiler产生真实queue schedule、ownership transfer和timestamp evidence，不满足门则走graphics queue |
| P1-35 | 小容量路径使用`take/min/break`静默降级，预算名与实际GPU内存/工作量不对应 | 建立统一budget controller、admission、priority、overflow/degradation receipt和显存/带宽/trace work telemetry |
| P1-36 | pipeline/profile名如Cinematic/OpenWorld与固定8x8、32 probe实际能力不匹配 | profile由versioned recipe生成，声明明确的internal resolution、rays、memory、latency和backend requirement |

## 7. P2治理项

| ID | 当前差距 | 治理要求 |
|---|---|---|
| P2-01 | `ScenePrepare`、`SurfaceCache`、`RadianceCache`等成熟术语让proxy获得错误完成度 | 文档和diagnostic明确标注prototype限制，直到资格门通过才升级maturity |
| P2-02 | 18个production-like文件超过500行，GPU execute、representation与test source owner过大 | 按scene representation、residency、trace backend、reconstruction、readback拆folder-backed模块和接口 |
| P2-03 | 103处`.contains(`大量锁定源码文本、shader token或实现细节 | 迁移为typed reflection、compiled graph、buffer layout和pixel/numeric behavior断言 |
| P2-04 | visual exporter文件名携日期，report不绑定source/ref/device/driver/profile identity | 采用content-addressed evidence manifest，记录source SHA、adapter、driver、settings、golden和容差 |
| P2-05 | settings虽有serde，但没有独立schema version、project migration和unknown-field运营策略 | 建立versioned settings document、migration fixture、forward/backward兼容范围与拒绝diagnostic |
| P2-06 | manifest声明Windows/Linux/macOS，缺少真实backend/adapter/driver capability矩阵 | release manifest只声明已验证组合，unsupported backend必须fail closed或明确fallback |
| P2-07 | 历史计划、插件文档、capability和代码完成度不同步 | 建立自动source currentness、owner link、maturity/evidence gate和stale report检查 |
| P2-08 | 没有公开限制清单、可复现场景包和竞争性benchmark方法 | 发布固定scene/camera/profile、画质metric、GPU时间、显存、CPU时间、warmup与采样规范 |

## 8. 参考引擎给出的最低约束

### 8.1 Unreal Lumen

Unreal切片不是要求复制类名，而是证明产品HGI至少需要：独立`LumenSceneData`和primitive/card/page identity；多卡mesh representation与正式card capture；physical atlas、page table、mip、feedback、allocation/eviction/recapture；screen probe base/adaptive placement；screen/mesh SDF/global SDF/hardware RT分层trace；persistent directional Radiance Cache、free list、clipmap propagation、adaptive probe与GPU indirect args；temporal/spatial filtering和统一composition。Zircon当前每mesh一card、每页两个RGBA8样本、32常量probe和CPU readback循环不满足这些最低约束。

### 8.2 Unity Graphics

Unity APV显示probe system必须由memory budget推导brick pool尺寸，拥有chunk allocator/free list、cell streaming、scenario/blending、validity/sky/probe occlusion数据和Editor Lighting Tab；HDRP SSGI与Unified RT context显示screen-space history/denoise和AS build/dispatch是正式backend合同。Zircon可借鉴其资源与产品生命周期，不应把APV baked probe或SSGI单独等同于完整Lumen替代品。

### 8.3 Godot

Godot SDFGI拥有camera-relative cascade、scroll、voxel preprocess、direct light、probe integrate、history/occlusion和debug shader；VoxelGI又有真实Bake命令、进度与gizmo。它的算法目标低于Lumen，但仍证明“runtime cascade + editor bake/debug + persistent resource”必须同时存在，不能只有render DTO和诊断字符串。

### 8.4 Bevy与Fyrox

Bevy irradiance volume明确是外部baker消费侧，却仍提供正式asset/component、RenderAssets、view extraction、bind group与capability fallback；Fyrox展示GBuffer、deferred lighting、environment convolution与quality settings的明确owner。二者适合校准Rust模块边界和资产生命周期，不作为Zircon动态GI达到Unreal水平的证明。

## 9. 目标架构

```text
Scene HybridGiSettings / Participation / Build References
                         |
                         v
              HybridGiSceneCompiler
       card artifact / mesh SDF artifact / dependency graph
                         |
                         v
              RenderSceneGeneration (immutable)
                         |
          +--------------+----------------+
          |                               |
          v                               v
 SurfaceCacheResidencyService      GlobalSdfResidencyService
 capture/page table/feedback       clipmap/page/build/scroll
          |                               |
          +---------------+---------------+
                          v
                 HybridGiTraceService
       screen -> mesh SDF -> global SDF -> HRT -> sky
                          |
                          v
                 RadianceCacheService
 allocation/directional radiance/history/filter/relocation
                          |
                          v
              HybridGiReconstructionService
      full-res resolve/disocclusion/denoise/history/debug
                          |
                          v
         one IndirectLightingAttachment + GenerationReceipt
                          |
                          v
                  Core final composition
```

推荐owner为：`HybridGiSceneCompiler + HybridGiArtifactStore + HybridGiRenderSceneService + SurfaceCacheResidencyService + GlobalSdfResidencyService + HybridGiTraceBackendRegistry + RadianceCacheService + HybridGiReconstructionService + HybridGiBudgetController + HybridGiDiagnosticsService + HybridGiAuthoringService`。neutral core只定义contract，实际资源和算法由graphics/plugin拥有，Editor通过稳定operation/service调用，不直接改render extract。

## 10. 硬切换与里程碑

1. M0 Truth freeze：保留当前CPU/WGPU oracle，能力保持experimental，禁止默认Editor启用和产品级宣传。
2. M1 Scene contract：加入可持久化settings、participation、identity、profile recipe与migration，删除viewport truth覆写。
3. M2 Single owner：移除core 16 probe/16 region GI解释，建立唯一indirect-light attachment和history owner。
4. M3 Artifact pipeline：导入/离线生成多卡与Mesh SDF artifact，建立dependency key、cook和streaming。
5. M4 Surface capture：实现material-correct card raster、HDR多channel atlas、mip/gutter与capture readiness。
6. M5 Virtual residency：实现GPU page table、feedback、priority、eviction、dirty range、generation与fence。
7. M6 Global SDF：实现预算化camera/world-partition clipmap、GPU commit、overflow、scroll与large-world。
8. M7 Trace backends：完成screen/Mesh SDF/Global SDF hierarchy，再实现真实HRT AS build/dispatch/fallback。
9. M8 Screen probe/RC：实现adaptive placement、方向radiance、relocation、occlusion、GPU allocation/filter/history。
10. M9 Reconstruction：实现resolution policy、variance/filter、disocclusion、history和debug/shipping隔离。
11. M10 GPU lifetime：删除正常帧广泛readback，建立persistent/transient resources、queue schedule和device恢复。
12. M11 Editor product：完成authoring、debug overlay、capture、budget、operation、save/reopen/cook与native/export parity。
13. M12 Acceptance：通过required GPU matrix、golden、scale、fault、soak以及同硬件同画质Unreal对照后再hard cutover maturity。

旧CPU representation、core GI composition、RGB8 cache、placeholder mesh、source-string proof与compatibility shim不得长期并存。每个里程碑完成后删除旧owner，不使用`pub use`、wrapper或双写桥保活旧路径。

## 11. 资格门

| Gate | 必须证明的证据 |
|---:|---|
| 01 | Scene/Prefab/Project可save/reopen HGI settings、profile与per-object participation |
| 02 | project override、scene override、platform/backend override有确定优先级和migration |
| 03 | feature admission在provider/backend/evidence不足时fail closed，Editor不默认启用experimental |
| 04 | source/native/export三种装配产生同一capability、settings与visual receipt |
| 05 | RenderScene/Card/Probe/Page/Trace identity跨增删、LOD、streaming和reload稳定 |
| 06 | baked/dynamic/emissive/environment contribution只有一个energy owner且无double count |
| 07 | 多卡artifact覆盖凹体、薄体、非均匀缩放、负缩放和多material section |
| 08 | card artifact携source revision、builder version、cook target、checksum与dependency manifest |
| 09 | Visibility实际产生visible card/probe/page/trace demand并进入HGI consumer |
| 10 | 10K/100K对象下CPU representation增量成本满足预算，无整帧全量clone/sort |
| 11 | skinned/morph/foliage/heightfield/instancing/two-sided/translucency策略有明确实现或receipt |
| 12 | camera cut、world origin shift、scene streaming和device restore保持identity/history正确 |
| 13 | capture对PBR material、texture、alpha test、emissive、normal和LOD产生可验证差异 |
| 14 | Surface Cache拥有HDR多channel atlas、mip、gutter、dilation与物理格式说明 |
| 15 | GPU feedback记录miss、desired mip、frequency和screen contribution |
| 16 | residency具有allocation、eviction、pinning、priority、overflow和fence receipt |
| 17 | material/texture/light/transform变化只失效相关page并有dependency proof |
| 18 | page table/atlas generation原子发布，不能混用新映射与旧内容 |
| 19 | capture失败、texture未resident和pipeline缺失有可操作diagnostic与fallback |
| 20 | Surface Cache正常帧不依赖GPU全量readback到CPU再上传 |
| 21 | Mesh SDF artifact覆盖版本、压缩、streaming、LOD与platform capability |
| 22 | Global SDF clipmap随camera/world partition滚动，page commit全程GPU resident |
| 23 | page/candidate overflow不会静默丢失，有优先级、退化和telemetry |
| 24 | thin geometry、inside geometry、overlap、large distance与normal重建有数值oracle |
| 25 | hit identity可查询可信surface/material radiance，shipping miss不制造伪颜色 |
| 26 | screen -> Mesh SDF -> Global SDF -> HRT -> sky route有统一ray/miss contract |
| 27 | HRT拥有BLAS/TLAS build/update/compaction、dispatch、barrier、memory和fallback矩阵 |
| 28 | screen probe基于depth/normal/coverage自适应放置并有稳定temporal identity |
| 29 | Radiance Cache存储方向radiance、depth/visibility/confidence而非RGB8常量 |
| 30 | RC allocation、trace、filter、border、mip、history和consume全程GPU resident |
| 31 | probe relocation、partial interpolation、scroll、occlusion和stale policy有oracle |
| 32 | emissive、multi-bounce、sky、direct light与baked delta满足统一radiometric contract |
| 33 | reconstruction信息量随internal resolution扩展，不再固定64 tile |
| 34 | temporal rejection覆盖depth、normal、material、motion、exposure、disocclusion和history length |
| 35 | core只消费一个HGI output generation，不再上传第二套probe/trace region |
| 36 | steady-state无per-frame pipeline/layout创建，大资源由pool/graph生命周期管理 |
| 37 | 正常帧无广泛cache/atlas/voxel/trace readback，诊断回读独立且限流 |
| 38 | async compute有queue overlap、barrier和timestamp收益；无收益时不宣称async |
| 39 | profile明确GPU时间、CPU时间、显存、带宽、rays、resolution和degradation预算 |
| 40 | Editor有真实HGI面板、debug overlay、capture、budget与operation，不引用缺失template |
| 41 | Editor操作支持undo/redo、scene dirty、save/reopen、失败、重试、取消和diagnostic跳转 |
| 42 | cook/export/native runtime不依赖source-only执行模块或Editor环境变量 |
| 43 | required GPU测试在无adapter/feature时失败或明确skip receipt，不得直接成功返回 |
| 44 | 固定竞争场景在同硬件同画质下记录quality metric、GPU/CPU time、显存、warmup与Unreal对照 |

## 12. 完成定义

Runtime98只有在44个资格门全部有current、可复现证据，旧09F3的14项父P0由原owner逐项关闭，Plugins19的package差距同步清零，并且Runtime89/94/95/97的render graph、visibility、direct/baked lighting依赖满足时，才能把`review_status`之后的实现状态推进为accepted。当前状态只能是 **review complete / implementation not started**。

本报告是Runtime HGI架构与产品闭环owner；Plugins19继续拥有package/catalog/source/native/editor/dist物理装配细节，Runtime97拥有baked-lighting producer/consumer与能量边界，Runtime89拥有render graph调度，Runtime94拥有Visibility/GPU Scene，Runtime95拥有direct-light truth。后续修复必须按这些owner分层回写，禁止在多个报告重复宣称关闭同一父P0。
