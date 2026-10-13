---
title: First-Party Hybrid GI Source、Runtime、Editor、Dist、Catalog、Scene Representation、Surface Cache、Global SDF、Radiance Cache、Probe Trace、Denoise 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins19
review_date: 2026-08-19
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_plugins/hybrid_gi
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_plugins/hybrid_gi/runtime/src
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/shaders
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors
  - zircon_plugins/hybrid_gi/editor/src
  - zircon_plugins/hybrid_gi/dist/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_runtime/runtime-feature-presets.toml
tests:
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_probe_trace_tiles/tests
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_radiance_cache/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors/resolve_trace_handoff/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors/scene_depth_handoff.rs
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors/trace_schedule_handoff.rs
  - zircon_plugins/hybrid_gi/editor/src/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/28-hardware-ray-tracing-blas-tlas-ray-query-pipeline-sbt-denoising-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSceneData.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenMeshCards.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCacheFeedback.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeGather.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeTracing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenRadianceCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenHardwareRayTracingCommon.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeBrickPool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeReferenceVolume.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeReferenceVolume.Streaming.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeReferenceVolume.Debug.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume/ProbeVolumeLightingTab.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/ScreenSpaceLighting/HDRenderPipeline.ScreenSpaceGlobalIllumination.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/UnifiedRayTracing/RayTracingContext.cs
  - dev/godot/servers/rendering/renderer_rd/environment/gi.cpp
  - dev/godot/servers/rendering/renderer_rd/environment/gi.h
  - dev/godot/editor/scene/3d/voxel_gi_editor_plugin.cpp
  - dev/godot/editor/scene/3d/gizmos/voxel_gi_gizmo_plugin.cpp
  - dev/bevy/crates/bevy_pbr/src/light_probe/environment_map.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.rs
  - dev/Fyrox/fyrox-impl/src/renderer/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/gbuffer.rs
  - dev/Fyrox/fyrox-impl/src/renderer/light.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 19 · First-Party Hybrid GI Source、Runtime、Editor、Dist、Catalog、Scene Representation、Surface Cache、Global SDF、Radiance Cache、Probe Trace、Denoise 与 Product Integration 工程化差距

## 1. 结论

`zircon_plugins/hybrid_gi` 不是空壳。它已经有 typed runtime provider、四段 Render Graph feature、单采样/MSAA depth handoff、主场景 HZB trace、Surface Cache/voxel/Global SDF 三种软件后端、Mesh SDF payload、persistent Global SDF page table、Radiance Cache GPU stage、temporal resolve、debug source view、GPU readback ring 与较多结构测试。这些代码证明团队已经建立了一条可运行研究原型，应保留其合同、测试向量和部分 GPU kernel，而不是将整个包重写成另一个 demo。

但当前产品仍不是可与 Lumen、Godot SDFGI 或 Unity APV 比较的工程实现。根 Render Graph 把整个 viewport 压成固定 `8 x 8` trace tile；scene packet 只容纳 16 个 Surface Cache page、4 个 voxel clipmap 和 64 个 voxel cell；场景表示为每 mesh 一个球形 card、每 card 一个所谓 screen probe；Radiance Cache 只有 32 个 slot，每个 probe 是 4 x 4 RGBA8 tile，其 `TRACE` stage 只是把输入的 packed radiance 复制到内部 2 x 2 texel。不存在 BLAS/TLAS、RayQuery 或 ray pipeline 执行路径。

当前 Surface Cache 也没有捕获真实 surface。card bounds 来自 transform translation 与 scale，材质纹理统一采样中心 UV，页面使用均匀 RGBA8/depth sample；CPU voxel light 以手写常数混合 tint/direct light。主 collector 每帧 clone scene/light/中立 DTO，在一个全局 `Mutex` 下完成 Mesh SDF projection、Global SDF residency、GPU dispatch 和 readback enqueue；一次 HGI frame 又请求 7 个固定 buffer、每 atlas/capture/depth slot 一个独立 readback，以及完整 probe tile/indirect args，任一 readback 未完成都会阻塞整帧结果，FIFO 队首还会阻塞后续已完成帧。

包与产品的声明同样不闭合。manifest 将两个能力标为 `partial`，却声明默认 profile 不需要 HGI；App 的 Editor 默认 render profile 和 Editor viewport 实际都会启用 HGI。`dist` 明确承认执行仍由 source runtime module 承载，native carrier 为 stateless metadata/registration shell。Editor 包只注册 view/drawer/template ID，引用的 `plugins://hybrid_gi/editor/authoring.zui` 不存在，没有 settings asset、场景参与规则、card/SDF/probe 可视化、bake/rebuild、undo/save 或诊断操作。

Runtime09F3 已拥有算法和渲染本体的 14 项 P0；Plugins04/06/08、Runtime09A-09F2/09H1/28 与 Editor22 已拥有 renderer composition、catalog/capability、authoring、RHI/GPU lifetime、scene/material/streaming/light/baked/temporal/HRT 的共享硬阻塞。本篇不重复累计 P0，登记 **0 项新增 P0、56 项 P1、14 项 P2**。Plugins19 只拥有 Hybrid GI 从 package、provider、场景表示、GPU resource、editor、native carrier、profile 到产品资格的纵向闭环。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | 冻结事实 |
|---|---:|---|
| tracked package 总体 | 270 / 43,848 / 1,579,121 / 237 | 250 Rust、16 WGSL、4 TOML |
| runtime production Rust/WGSL | 221 / 29,806 / 1,099,143 | provider、scene representation、GPU resources、readback、16 shader |
| runtime tests/fixtures | 39 / 12,241 / 468,626 / 237 | 20 个显式 ignored；21 个 GPU test 无 adapter 时直接成功返回 |
| package/editor/dist | 9 / 317 / 10,843 / 3 | editor 只有注册测试；dist 只有 ABI/manifest 测试 |
| runtime manifest | 1 / 17 / 509 | 直接依赖 `wgpu 29.0.1` |
| ignored shader cache | 42 files | `.zircon-cache` 与 `.zircon/cache` 两套路径，21 份 meta + 21 份 `.wgsl.zst` |
| physical package | 312 files / 约 1,750,590 bytes | tracked source 加 ignored cache，不把 cache 当 source evidence |
| production fingerprint | `f137fd4efad828df2989aa9c138f7c7ae0f0eb46c5929b4a10b91ee09dd0871d` | 221 个 production path 与 file SHA-256 组成排序清单后重算 |

源 revision 为 `bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch 为 335。审查时 Hybrid GI 有 18 个 source/test 文件处于其他 Session 或用户修改状态，集中在 Global SDF、voxel lookup、probe trace、neutral projection、radiance cache 与 source ledger，因此标记 `source_recheck_required: true`。本文没有修改 production 或 tests，也不把 source fingerprint 当作当前 clean 证明。

### 2.2 证据等级

本轮枚举并分类全部 270 个 tracked path，逐一核对 package/editor/dist、runtime module tree、16 份 WGSL、scene representation、GPU resource construction、readback、四个 public executor、测试入口及 App/catalog/profile caller；同时读取 Unreal Lumen、Unity Graphics APV/SSGI/Unified RT、Godot SDFGI/VoxelGI、Bevy Light Probe 与 Fyrox renderer 的对应生产源码。结论属于 E3 静态调用链、数据表示与产品装配审查。

没有运行 Cargo、GPU、Editor、NativeDynamic、像素、跨平台或性能测试。本轮是 review-only，而且共享基线与部分 HGI source 正在变化。237 个 test attribute 是库存，不是通过数；20 个 ignored 和 21 个 adapter 缺失时直接返回的测试尤其不能当发布资格。历史记录中的一次 `209 passed / 20 ignored / 0 failed` 只能证明当时选定命令，不证明 current source 或产品门。

### 2.3 词法与结构扫描

| 项目 | 数量 | 限制 |
|---|---:|---|
| `TODO/FIXME/HACK/XXX` | 0 | 无标记不等于无临时实现 |
| `panic!/todo!/unimplemented!` | 8 | 包含生产文件内测试模块，需实施时按 cfg 重分 |
| `.unwrap()` / `.expect()` | 9 / 33 | 同样需按 production/test AST 复核 |
| `#[allow(...)]` | 19 | 包括 production placeholder 的 dead-code 豁免 |
| `include_str!` | 51 | 多用于 shader 拼接，也用于 source-string 断言 |
| `.contains(...)` | 202 | 大量为源码词面测试，不能代替行为证据 |

### 2.4 与既有报告的边界

- Runtime09F3 是 Hybrid GI 算法、Surface Cache、Global SDF、screen probe、Radiance Cache、trace、resolve 与产品画质的 canonical owner；本篇不得复制其 14 个 P0。
- Plugins04 拥有 Rendering umbrella、feature bundle、native registration replay 和 graph composition；本篇只证明 HGI 具体 package 的 source/native 不等价。
- Plugins06 拥有全体首方 catalog/profile/capability truth；本篇只追踪 HGI 的 manifest、Editor default、App feature 与 provider readiness。
- Plugins08 与 Editor22 拥有通用 authoring/document/operation/render tooling；本篇定义 HGI 需要交付的 domain surfaces。
- Runtime09A-09E/09F1/09F2/09H1/28 分别拥有 RHI、GPU Scene、material/PSO、streaming、direct light、IBL、baked lighting、temporal 与 HRT 基础；HGI 只消费这些公共 owner，不能私建第二套。
- Runtime09D 与 Runtime04 拥有通用 artifact/DDC/residency；Mesh SDF、card capture 与 baked probe 的领域内容在 HGI，存储/流送协议不在 HGI 重造。

## 3. 应保留的真实基础

1. `HybridGiSceneRepresentation`、prepared DTO、provider 与 runtime-prepare collector 已经形成明确 owner 链，可作为重构时的 compatibility oracle。
2. Mesh SDF asset/object、dirty region、Global SDF clipmap/page table/candidate/influence 和 persistent GPU atlas 是真实算法底座。
3. depth handoff 已区分 single-sample/MSAA，并使用 main-scene HZB、inverse view projection、normal code 与 velocity/history输入。
4. probe trace shader 已实现 Surface Cache directional march、Global SDF sphere stepping、voxel fallback、source/lighting diagnostics 和 multi-ray aggregation骨架。
5. Radiance Cache 有 mark/allocate/trace/filter/border/mip/consume stage，已有 generation 与 interpolation corner合同，可迁移到真正的 GPU probe atlas。
6. temporal resolve 已有 velocity reprojection、depth/source/normal/signature rejection、history confidence 与 debug source view，可作为 reference path。
7. Global SDF 与 HGI readback 已接共享 runtime readback admission，至少不会无界提交；当前 backpressure 语义应保留为退化诊断而非主控制流。
8. 测试中已有 Global SDF、voxel lookup、multi-ray、normal rejection、spatial filter、MSAA 和 product PNG 向量，适合升级为 required GPU/pixel corpus。

## 4. 参考引擎给出的最低约束

### 4.1 Unreal Lumen

`FLumenSceneData` 持有 sparse Cards、MeshCards、PrimitiveGroups、增量 upload lists、card page table、last-used buffers 与 Surface Cache feedback；`FLumenSurfaceCacheAllocator` 负责物理页和 sub-page bin allocation，Albedo/Normal/Emissive/Depth/Direct/Indirect/History 是独立 atlas layer。feedback 按屏幕 tile 产生、GPU compact，再以 page hit、分辨率层级和距离驱动 residency，而不是“取前 N 个 card”。

Screen Probe Gather 由屏幕/GBuffer 分配固定与 adaptive probes，支持 importance sampling、tile classification、screen trace、Mesh SDF、Global SDF、HRT 与短程 AO。Radiance Cache 有相机对齐 clipmap、indirection texture、free list、probe atlas、depth/sky visibility、spatial filter、border/mip、indirect args 和 persistent external history。HRT 路径拥有真实 acceleration structure、inline/ray pipeline 和 material/hit-lighting选择。

Zircon 不必复制 UE 类型名或默认参数，但最低门是同等级的多层 scene representation、反馈驱动 virtual residency、GPU 增量更新、可选择 trace backend、持久历史和可分析资源预算。固定 8 x 8、每 mesh 一 card 与 readback 驱动 CPU authority 不满足该门。

### 4.2 Unity Graphics

APV 的 `ProbeBrickPool` 对 SH/validity/sky occlusion/occlusion纹理执行 chunk allocation、deallocation、copy 与 upload；`ProbeReferenceVolume.Streaming` 使用可取消 disk requests、双缓冲 GPU staging、scratch pool、active queue、按相机距离和方向评分的换入换出、worse-loaded cell eviction 与独立内存预算。Debug 有 cell/brick/probe/offset/fragmentation、freeze streaming、score、verbose log 与 sampling debug；Lighting tab 覆盖 baking set、scene membership、bake、warning 和 Debugger 跳转。

HDRP SSGI 在 RenderGraph 中按 full/half resolution trace、reproject、bilateral upscale、temporal filter、diffuse denoiser和双 history validity执行，Ray Traced/Mixed 模式由真实 Unified RayTracing backend 支撑。Zircon 至少要具备同等级的 memory/streaming/bake/debug/denoise lifecycle，不能以一个文本 diagnostics pane 和 3 x 3 的 8 x 8 tile filter 等价替代。

### 4.3 Godot

Godot SDFGI 使用相机滚动 cascade、dirty region、128^3 SDF volume、solid-cell indirect dispatch、R16F light data、有符号 SH history/average、octahedral probe atlas、static/dynamic light阶段、scroll、integrate 和独立 debug pipeline。VoxelGI 有可烘焙 resource、editor plugin、progress、warning、gizmo 与多种 lighting/emission debug。

Godot 的实现规模低于 Lumen，但仍明显高于 Zircon 当前 4 x 4 x 4 CPU voxel代理。它说明“较轻量引擎”也必须让 cascade 以相机为中心、保留浮点/SH radiance、持久化资源，并给 Editor 可操作的 bake/debug surface。

### 4.4 Bevy 与 Fyrox

Bevy 将 baked irradiance volume、environment map、light probe visibility、asset/GPU image、bindless capability与 per-view extraction/binding分开；代码明确记录 ambient-cube的 fetch/quality/memory取舍和 WebGPU binding限制。Fyrox 没有可作为 Lumen 等价物的动态 GI，但其 renderer 仍把 GBuffer、deferred light、shadow renderer、geometry/texture/shader/uniform cache、quality settings 与 statistics 分 owner。

因此 Bevy/Fyrox 只提供最低资源边界和 fallback 诚实性，不是 Zircon 动态 GI 画质目标。缺少成熟 GI 的参考引擎不能被用来降低 HGI 验收线。

## 5. P0 路由，不重复登记

| 已确认最高优先级事实 | Canonical owner | Plugins19 处理方式 |
|---|---|---|
| 固定 8 x 8 tile、每 mesh 一 card、固定容量代理不能构成工程 GI | Runtime09F3 P0 | 不新增 P0；P1 固定 package 迁移与产品 gate |
| Surface Cache 没有真实多视角材质/几何 capture | Runtime09F3、Runtime09B/09C | 不新增 P0；定义 capture artifact 与 residency接口 |
| visibility 到 HGI scene bridge 缺失、core/plugin 双 GI/history owner | Runtime09F3、Plugins04、Runtime09A/09B | 不新增 P0；要求唯一 graph owner |
| 每帧 readback -> CPU repack -> reupload | Runtime09F3、Runtime09A/09D | 不新增 P0；P1 定义 GPU-resident authority |
| Radiance Cache、Global SDF lighting、HRT 与 invalidation 不完整 | Runtime09F3、Runtime09E/28 | 不新增 P0；只细化 package consumer与资格 |
| Editor 无真实 authoring/debug，native dist 不承载执行 | Runtime09F3、Plugins01/04/08、Editor22 | 不新增 P0；要求 source/native/editor产品等价 |
| ignored/source-string tests 可制造“已完成”观感 | Runtime09F3、Tooling10/11/16 | 不新增 P0；升级为 required evidence matrix |

## 6. P1 工程化差距

### 6.1 Package、capability、catalog、carrier 与产品组合

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| HGI-V-P1-01 | manifest 将 runtime 与 advanced capability 标为 partial，却声明 default profile 不需要；Editor default 实际启用 | capability status 由 BuildSet + provider + device + artifact + scene readiness生成，文档与产品选择同源 |
| HGI-V-P1-02 | catalog 仅在 compile feature 下返回 source registration；profile只证明 ID 被选 | composition receipt 逐项证明 linked provider、executor、shader、required backend和运行态健康，否则 fail-close |
| HGI-V-P1-03 | `dist` 为 stateless/schema 0，无 command/state/unload/bridge，并声明 execution 仍在 runtime module | native carrier 真正承载同一 HGI provider/executor，或从 manifest 删除 `native_dynamic` |
| HGI-V-P1-04 | editor 只注册 view/drawer/template ID，模板文件不存在 | 入口加载真实 document/toolkit；缺资源、缺runtime或缺device时显示 unavailable，不发布空能力 |
| HGI-V-P1-05 | plugin runtime 直接依赖具体 `wgpu`，又深度读取 Runtime renderer DTO | 通过稳定 RHI/render extension合同持有 GPU object；package不绑定 host 私有实现布局 |
| HGI-V-P1-06 | plugin graph 与 core post-process 都可拥有 GI/history语义 | 一个 composition owner 决定 pass、history和composite；provider只贡献实现与资源需求 |
| HGI-V-P1-07 | `register()` 成功即可发布 feature，未验证 Mesh SDF/material/HZB/velocity/history/RT backend | 增加 typed prerequisite、degraded reason、quality tier和动态健康状态，任何缺项不可伪装 ready |

### 6.2 Scene identity、card、probe 与 participation

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| HGI-V-P1-08 | provider/collector 每帧 clone prepared frame、extract和三类 light，再在 neutral/internal DTO间重建 | generation-scoped immutable RenderScene snapshot + compact dirty command stream，无全场景往返投影 |
| HGI-V-P1-09 | `build_card_descriptors` 为每 mesh 生成一个 card，中心是 translation，半径只看 scale | 离线/增量 MeshCard representation：多方向 cards、真实 bounds/LOD/material section、stable generation identity |
| HGI-V-P1-10 | production 保留 `placeholder_mesh` 并由 `synchronize_cards` 调用 | placeholder只在 test fixture；production 缺 geometry 必须 typed reject/degrade，不能伪造 builtin model/material |
| HGI-V-P1-11 | `screen_probe_state` 取排序后前 N 张 card，一 card 一 probe，ID 为 enumeration | 从 depth/normal/view tile 分配 screen probe，支持 adaptive placement、jitter、coverage、view identity和稳定 history |
| HGI-V-P1-12 | source ledger 只有 bit mask，dynamic weight 恒为 0/255 | 记录 static/dynamic/emissive/baked/visibility/light-channel provenance、revision与连续混合权重 |
| HGI-V-P1-13 | `Disabled` participation 存在但当前 classifier不产生；baked mode 的 dynamic receiver可缺静态基线 | 由 authored component/material mobility和baked artifact生成统一 participation，显式防 double count 与 missing baseline |
| HGI-V-P1-14 | scene/card/light变化主要触发粗粒度 dirty；无 geometry/material/texture/light/baked依赖图 | dependency-keyed invalidation DAG，按 card page/SDF page/probe/history局部失效并携带 reason/epoch |

### 6.3 Surface Cache、card capture、material 与 feedback

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| HGI-V-P1-15 | Surface Cache 一 card 一 page，packet最多16 page，capture tile固定64，内容为单一 RGBA/depth sample | virtual page table + 多 mip/multi-view card pages + typed albedo/normal/emissive/depth/direct/indirect layers |
| HGI-V-P1-16 | material capture cache 对所有纹理统一采样 `[0.5, 0.5]`，trait请求 UV 也被忽略 | card-space raster capture真实 UV、LOD、sampler、alpha/two-sided/WPO/VT/material graph语义 |
| HGI-V-P1-17 | CPU card shading以简化 PBR/seed 生成均匀 tile | 共用 renderer material/light kernel或专用 capture pass，输出与主材质、shadow、emissive同语义 |
| HGI-V-P1-18 | atlas、voxel、probe irradiance核心表示是 RGB8/RGBA8 | scene radiance使用至少 FP16/RGB9E5/SH/visibility moments的typed format，量化只用于明确的压缩artifact |
| HGI-V-P1-19 | `synchronize` 每次 clone Vec、重建 BTreeMap/BTreeSet、sort并分配 | persistent slot/page indices、dirty ranges、free lists与批量 GPU scatter update，热路径无全量clone/sort |
| HGI-V-P1-20 | feedback只是未驻留 card ID 列表和数量，没有 screen hit、desired mip、priority或GPU compaction | per-view GPU feedback、去重/compact、coverage/距离/命中频率/last-used评分、预算内换入换出 |
| HGI-V-P1-21 | 每帧重建 atlas/capture/depth texture与slot buffer，并逐slot回读4 bytes | persistent atlas generations、batched upload/copy、GPU page commit；readback只用于低频诊断和异步证据 |

### 6.4 Mesh SDF、Global SDF 与 voxel clipmap

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| HGI-V-P1-22 | voxel clipmap以全场景 card bounds为中心，非相机中心 | camera-snapped clipmap ring，scroll/dirty slab更新，支持large world origin与multi-view策略 |
| HGI-V-P1-23 | 每层只有 `4 x 4 x 4 = 64` cell，最多8层 | 根据世界尺度/画质/显存的稀疏体素或brick/page结构，容量由budget manager配置且可观测 |
| HGI-V-P1-24 | CPU voxel lighting用 `tint*0.45 + direct*0.9`、abs normal和固定双面下限后tone-map到RGB8 | material/light/shadow一致的radiance injection、方向性/各向异性、visibility、emissive和能量守恒 |
| HGI-V-P1-25 | Global SDF虽有真实page table，但最多128 resident pages、每page 32 candidates、每object 8 payloads | budgeted sparse atlas、分层culling/compaction、overflow queue与质量退化 receipt；禁止静默截断 |
| HGI-V-P1-26 | CPU 构建 page influence/candidate/upload，GPU completion必须readback后才commit | GPU-driven dirty page list、indirect build、generation fence和GPU page table swap；CPU不逐页观察成功 |
| HGI-V-P1-27 | Mesh SDF 来自运行时 geometry seed，未形成可追踪 cook/DDC/stream artifact | versioned Mesh SDF artifact：source/LOD/material flags/build settings/platform/compression key、stream pages和last-good |
| HGI-V-P1-28 | Global SDF trace固定最多16 step，命中后主要沿用lineage/voxel辐射度 | 自适应步进、bias/thickness/normal、near/far backend transition、hit material/surface lighting与错误界可视化 |

### 6.5 Radiance Cache、probe trace、lighting 与 backend routing

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| HGI-V-P1-29 | Radiance Cache 最多32 resident probes、4 clipmap、每轴逻辑分辨率48但物理slot固定32 | camera-centered clipmap indirection、GPU free list、按需求分配数千级 probe、显存预算与稀疏 residency |
| HGI-V-P1-30 | 每 probe atlas 为4 x 4 RGBA8 + 2级极小 mip | octahedral direction/depth/visibility/radiance atlas，quality-scaled resolution、FP/packed HDR和border-safe mip |
| HGI-V-P1-31 | `radiance_cache_update.wgsl` 的 TRACE stage把一个 packed `radiance_confidence` 写入内部2 x 2 | trace stage消费真正的multi-direction ray result、hit distance/visibility，并在GPU上与probe generation原子提交 |
| HGI-V-P1-32 | consume要求8个corner slot/generation全部有效，任一异常即整项return | 稀疏有效角、visibility/normal权重、parent clipmap fallback、confidence normalization和partial residency处理 |
| HGI-V-P1-33 | probe trace最多16 tile/ray组，主要用整数启发式权重、lineage和固定fallback | blue-noise/importance-guided direction、BRDF/solid-angle PDF、temporal reservoir或等价采样理论与误差统计 |
| HGI-V-P1-34 | route/mode包含hardware概念，但包内没有 RayQuery、AS、DispatchRays 或 ray pipeline符号 | 接 Runtime28 的BLAS/TLAS/RayQuery/pipeline/SBT contract，software/hardware混合按device/scene/material逐ray选择 |
| HGI-V-P1-35 | main-scene trace输出固定64 tile radiance，再在全分辨率像素上展开 | viewport/dynamic-resolution驱动的probe grid、adaptive sample density、checkerboard/half/full模式和明确 upscale/filter |

### 6.6 Resolve、GPU lifetime、同步、readback 与性能

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| HGI-V-P1-36 | spatial filter只在8 x 8 tile上做3 x 3 source/depth/signature/normal筛选 | 多阶段 edge-aware spatial/reconstruction filter，使用full-res depth/normal/roughness/hit distance/variance |
| HGI-V-P1-37 | temporal只做单点velocity reprojection、阈值拒绝和固定 `current +/- 0.25` clamp | camera-cut/resize/jitter/dynamic-res aware history、neighborhood statistics、variance/confidence、disocclusion与exposure处理 |
| HGI-V-P1-38 | scene/trace/resolve executor每次执行创建 layout、shader module、pipeline和bind group | device-generation scoped pipeline cache/PSO，异步编译、permutation key、warmup/fallback和device-loss重建 |
| HGI-V-P1-39 | `execute_prepare` 每帧创建大量 buffer/texture/layout/bind group与fallback resource | persistent per-world/per-view pools、transient render-graph aliasing、descriptor cache、retirement fence与memory telemetry |
| HGI-V-P1-40 | 一个全局 `Mutex` 从 scene projection 前一直持有到 GPU encode/readback enqueue后 | per-instance state + 短临界区 + prepared immutable work packet；CPU preparation可并行，GPU提交由render thread owner |
| HGI-V-P1-41 | 每帧至少7个固定readback，加每atlas/capture/depth slot独立readback与完整tile/args | 默认0 readback的GPU-resident loop；诊断按需批量copy到ring，带bytes/request/bandwidth/latency budget |
| HGI-V-P1-42 | future要求所有子readbackready；collector只消费FIFO队首，慢帧阻塞后续帧，满ring则跳过新work | generation-keyed out-of-order completion、过期丢弃、独立GlobalSDF/HGI通道、deadline/cancel与可恢复降级 |

### 6.7 Editor authoring、debug、operation 与可运维性

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| HGI-V-P1-43 | `plugins://hybrid_gi/editor/authoring.zui` 物理不存在，测试只断言注册ID | 交付真实ZUI/controller/view-model并做启动、布局、交互和缺依赖测试 |
| HGI-V-P1-44 | 无 project/world/volume/mesh participation/settings asset，无save/undo/reopen | versioned HGI settings与volume/component schema，共享Editor document/transaction/save/migration路径 |
| HGI-V-P1-45 | Editor 默认强制enable，custom budget用常量，profile override从env读取并以`OnceLock`永久缓存 | project/profile/viewport layered settings，实时可编辑、可撤销、可序列化，env只作显式开发override |
| HGI-V-P1-46 | Workbench diagnostics主要输出几条格式化字符串和计数 | structured per-view stats：cards/pages/probes/backend/overflow/readback/GPU time/VRAM/history rejection/quality reason |
| HGI-V-P1-47 | shader有Cards/Surface/Voxel/Input Set debug枚举，但Editor无实际控制与overlay | card bounds/pages、SDF clipmap/page、probe rays/radiance/variance、history rejection和backend heatmap可视化 |
| HGI-V-P1-48 | 无card rebuild、Mesh SDF cook、probe bake、cache clear、capture、cancel/progress/failure recovery操作 | typed async operations与receipt，支持cancel、progress、undo/transaction边界、crash recovery和last-good |
| HGI-V-P1-49 | 无设备、平台、场景规模、显存压力与自动scalability UI | capability inspector、budget预估、quality preview、platform override、memory pressure与degraded cause产品面 |

### 6.8 Tests、evidence、release 与竞争性资格

| ID | 当前证据 | 必须重构为 |
|---|---|---|
| HGI-V-P1-50 | 237个test attribute未在本轮执行，历史命令不绑定current fingerprint | required test manifest绑定source/build/adapter/driver/shader hash，partial/omitted不得汇报全绿 |
| HGI-V-P1-51 | 20个高价值GPU/PNG/产品测试显式`#[ignore]` | promotion为分层required GPU/pixel suite；仅人工export的测试必须另标tool，不混入pass统计 |
| HGI-V-P1-52 | 至少21个GPU test在`test_device()`失败时直接`return` | adapter缺失报告skipped/unavailable并使required lane失败，记录backend/device/driver/feature/limit |
| HGI-V-P1-53 | 多个测试只对`include_str!`结果做`contains`或检查stage名字 | shader compile/reflection/layout parity、dispatch结果、invariant/property、negative/fault行为测试 |
| HGI-V-P1-54 | product PNG导出依赖ignore和固定文件名，未绑定scene/camera/exposure/build | versioned scene corpus、linear/HDR golden、perceptual metric、metadata sidecar、artifact store与review receipt |
| HGI-V-P1-55 | 主要依赖WGPU PRIMARY，无DX12/Vulkan/Metal、MSAA矩阵、device loss/OOM/resize/multi-view/large-world soak | 平台与失败矩阵、shader compiler差异、long-run history/residency/streaming稳定性门 |
| HGI-V-P1-56 | 没有同场景同画质同硬件与Unreal/Godot/Unity的帧时间、误差、VRAM证据 | correctness先行的竞争benchmark：CPU/GPU分段、P50/P95/P99、VRAM/RSS/I/O、噪声/泄漏/稳定性和统计置信 |

## 7. P2 长期治理差距

| ID | 当前证据 | 后续治理 |
|---|---|---|
| HGI-V-P2-01 | 多个700行左右Rust/WGSL文件同时含声明、实现和测试 | 按scene/capture/trace/filter/output合同拆模块，并保留generated shader composition manifest |
| HGI-V-P2-02 | shader由5段字符串拼接，接口主要靠词面测试 | 生成式binding/schema/layout校验与source map，编译诊断可定位原fragment |
| HGI-V-P2-03 | Rust/WGSL重复容量、offset、magic和bit layout | 单一versioned schema/codegen，host/shader compile-time size/offset assertion |
| HGI-V-P2-04 | 两套 `.zircon-cache` 与 `.zircon/cache` shader cache目录 | 统一cache root、key、lock、cleanup、quota和source/archive排除策略 |
| HGI-V-P2-05 | stable IDs大量使用裸`u32/u64`与enumeration | typed world/view/card/page/probe/generation IDs，wire/diagnostics显式scope |
| HGI-V-P2-06 | 质量常量分散在Editor、collector、scene packet与shader | versioned quality profile asset + device calibration + budget governor，不允许隐式常量漂移 |
| HGI-V-P2-07 | diagnostics字符串由Editor再次解析和排序 | typed diagnostic schema与stable event/metric names，UI只投影不反向解析字符串 |
| HGI-V-P2-08 | 缺trace/invalidation/readback确定性重放 | capture最小RenderScene generation、dirty commands、RNG和GPU results，支持离线差分重放 |
| HGI-V-P2-09 | 缺Mesh SDF/page/probe格式fuzz与恶意预算测试 | parser/property/fuzz/overflow/OOM corpus，所有分配前执行bytes/items/time admission |
| HGI-V-P2-10 | 缺场景复杂度到预算/质量的校准数据库 | 建立indoor/open-world/foliage/emissive/dynamic/cinematic workload及自动quality recommendation |
| HGI-V-P2-11 | package manifest、profile和Editor默认由人工维护 | 生成式capability/product matrix与currentness validator，矛盾直接阻断发布 |
| HGI-V-P2-12 | reference对照是报告期静态读取 | 记录reference commit、文件digest、适用性和重新审查触发，不把旧快照当当前事实 |
| HGI-V-P2-13 | 无跨版本HGI state/artifact迁移策略 | settings、Mesh SDF、Surface Cache、baked probe与capture证据各自version/migration/retirement |
| HGI-V-P2-14 | 无超越Unreal的公开方法学 | 固定workload、oracle、画质容差、硬件/driver、统计方法与可复现实验包后再允许竞争性声明 |

## 8. 目标架构与 owner 边界

| 层 | 唯一职责 | 不得继续承担 |
|---|---|---|
| HGI Product Composition | 选择provider/backend/quality/pass/history并生成readiness receipt | 算法实现、GPU对象、Editor状态 |
| HGI RenderScene Adapter | 消费generation-scoped GPU Scene、material、light、visibility dirty stream | clone全scene、伪造mesh/card |
| Mesh Card / SDF Compiler | source mesh/material -> versioned card/SDF artifact | runtime临时以bounds替代几何 |
| Surface Cache | virtual page、capture layers、feedback、residency与invalidation | 单sample RGBA8 authority |
| Global SDF / Voxel | camera clipmap、sparse page/brick、GPU build/scroll与trace | 全场景CPU 4^3 rebuild |
| Screen Probe Gather | view-driven probe placement、trace schedule、importance sampling | 以card enumeration冒充screen probe |
| Radiance Cache | clipmap indirection、probe atlas、free list、trace/filter/mip/history | 32-slot CPU mirror authority |
| Trace Backend Router | screen/Mesh SDF/Global SDF/HRT选择与统一hit contract | enum/mask无真实backend |
| Resolve / Denoise | reconstruction、temporal/spatial filter、upscale、history validity | 固定8 x 8颜色平铺 |
| Editor HGI Toolkit | settings/volume/participation/bake/debug/capture/diagnostics | 只注册ID或解析字符串 |
| Evidence Pipeline | scene corpus、GPU/pixel/perf/failure/cross-platform receipts | ignored PNG和source contains |

关键数据流必须收敛为：

`World/Assets -> RenderScene generation + dirty stream -> Card/SDF artifact residency -> Surface/Global scene GPU generations -> Screen Probe/Radiance Cache GPU work -> Resolve/Denoise -> Composite/History -> structured diagnostics`。

CPU 只提交generation、dirty ranges和budget policy；正常帧不能依靠GPU readback决定下一帧照明authority。Editor读取同一generation和structured diagnostics，不能维护第二份HGI场景。

## 9. 重构里程碑

### M0 · Truth freeze 与基线资格

- 以 Runtime09F3 为算法 P0 owner，本篇只登记纵向P1/P2；
- 修正 manifest/default Editor/profile 三者的 capability truth；
- 生成 current BuildSet/provider/executor/shader/device/scene readiness receipt；
- 将20 ignored与21 silent skip分类，禁止出现在“全通过”统计中。

### M1 · 唯一 composition 与 RenderScene generation

- 删除core/plugin双GI/history owner；
- 定义HGI prerequisite、backend和degraded reason合同；
- 用immutable RenderScene generation + dirty stream替代clone/neutral/internal往返。

### M2 · Card/SDF artifact pipeline

- 建立multi-card build、真实bounds/material section与Mesh SDF cook/DDC artifact；
- source revision、settings、platform、compression和dependency进入key；
- Runtime只stream/install generation，不临时伪造placeholder。

### M3 · Surface Cache virtual residency

- 分层HDR atlas、virtual page table、GPU feedback/compact、free list和多mip；
- card-space真实材质捕获，接主材质/纹理/灯光/阴影；
- dependency DAG驱动page级局部失效。

### M4 · Camera-centered Global SDF / voxel

- camera-snapped sparse clipmap、scroll slab、GPU page build/commit；
- 容量接统一VRAM budget，overflow可观测且有降级；
- voxel radiance使用方向性HDR和真实lighting injection。

### M5 · Screen Probe 与 trace backend

- 从view depth/normal生成固定+adaptive probe；
- 建立screen/Mesh SDF/Global SDF统一hit contract和importance sampling；
- 接入Runtime28真实HRT backend，支持逐材质/逐ray fallback。

### M6 · Radiance Cache GPU authority

- clipmap indirection、probe free list、HDR radiance/depth/visibility atlas；
- mark/allocate/trace/filter/border/mip/consume全部使用真实GPU数据；
- CPU readback退出正常照明闭环。

### M7 · Resolve、denoise 与 history

- 支持dynamic resolution、half/full/checkerboard和edge-aware reconstruction；
- camera cut/resize/exposure/disocclusion/variance-aware temporal；
- multi-stage spatial/diffuse denoise和可量化误差。

### M8 · GPU lifetime 与热路径

- device-generation pipeline/descriptor/resource cache；
- per-world/per-view pools、render-graph transient aliasing、fence retirement；
- 拆全局mutex，CPU prepare并行，正常帧零readback。

### M9 · Editor product

- settings/volume/participation document、undo/save/reopen/migration；
- card/SDF/page/probe/ray/history/backend可视化；
- bake/rebuild/capture/clear typed operation、progress/cancel/recovery/last-good。

### M10 · Source/native/export parity

- NativeDynamic承载同一provider/executor/lifecycle，或删除声明；
- Client/Editor/export对同一BuildSet、artifact和quality产生一致graph；
- enable/disable/reload/device-loss原子切generation并安全退役。

### M11 · Required evidence matrix

- GPU/pixel tests从ignore晋升，adapter缺失记失败/skip而非pass；
- DX12/Vulkan/Metal、MSAA、resize、multi-view、large-world、device-loss/OOM/soak；
- scene/source/build/adapter/driver/shader hash绑定artifact。

### M12 · 竞争性性能与画质门

- 与Unreal Lumen、Godot SDFGI、Unity SSGI/APV在同场景同画质同硬件比较；
- 报告CPU/GPU分段、P50/P95/P99、VRAM/RSS/I/O、噪声、漏光、temporal stability；
- correctness、failure、soak和统计置信全部达标后，才允许“优于Unreal”的结论。

## 10. 验收矩阵

| Gate | 验收内容 |
|---|---|
| HGI-G01 | manifest、profile、App、catalog与Editor default来自同一生成式产品定义 |
| HGI-G02 | source/library/native三种packaging注册并执行同一feature/backend语义 |
| HGI-G03 | 缺provider/executor/shader/device feature/scene prerequisite时fail-close |
| HGI-G04 | 普通帧不clone完整scene/light/neutral DTO，不持有全局collector长锁 |
| HGI-G05 | card来自真实geometry/material section并支持multi-card/mip/LOD |
| HGI-G06 | production无placeholder mesh/material作为成功路径 |
| HGI-G07 | Screen Probe由view/GBuffer分配，非card顺序枚举 |
| HGI-G08 | participation与baked/dynamic source provenance可持久化且无double count |
| HGI-G09 | Surface Cache有多层HDR atlas、virtual page、feedback、priority和eviction |
| HGI-G10 | material capture保留UV、alpha、two-sided、WPO、emissive与texture LOD |
| HGI-G11 | Mesh SDF/card artifact有versioned key、DDC、stream、last-good和retirement |
| HGI-G12 | Global SDF/voxel以camera clipmap滚动，dirty slab局部更新 |
| HGI-G13 | 所有容量/overflow进入budget与structured diagnostics，无静默truncate |
| HGI-G14 | Radiance Cache拥有GPU indirection/free list/HDR depth/visibility/radiance atlas |
| HGI-G15 | probe trace使用真实multi-ray hit与PDF，不复制单一packed RGB |
| HGI-G16 | hardware模式实际构建/消费AS并执行RayQuery或ray pipeline |
| HGI-G17 | software/hardware backend输出统一hit/material/lighting contract |
| HGI-G18 | resolve支持dynamic resolution、camera cut、resize、exposure和disocclusion |
| HGI-G19 | denoise使用depth/normal/roughness/hit distance/variance并通过edge/leak golden |
| HGI-G20 | pipeline/layout/shader/descriptor按device generation缓存与重建 |
| HGI-G21 | transient/persistent resource有明确pool、alias、fence和VRAM telemetry |
| HGI-G22 | 正常照明闭环零GPU readback；诊断readback有批量/频率/带宽预算 |
| HGI-G23 | readback可out-of-order完成、取消/丢弃过期generation且无FIFO队首阻塞 |
| HGI-G24 | Editor模板物理存在并通过真实window/controller交互测试 |
| HGI-G25 | settings/volume/participation支持undo/save/reopen/migration和external conflict |
| HGI-G26 | bake/rebuild/capture/clear提供typed progress/cancel/result/recovery receipt |
| HGI-G27 | card/page/SDF/probe/ray/history/backend debug view消费同一runtime generation |
| HGI-G28 | diagnostics为typed schema，不靠Editor解析自由文本 |
| HGI-G29 | required test不存在adapter时报告unavailable并使资格lane非绿 |
| HGI-G30 | ignored product GPU/pixel tests进入machine-readable required matrix |
| HGI-G31 | shader binding/layout由生成式schema和真实compile/reflection验证 |
| HGI-G32 | DX12/Vulkan/Metal、MSAA、resize、multi-view、large-world、device-loss/OOM通过 |
| HGI-G33 | 1000+动态物体/灯光/材质更新soak无历史污染、页泄漏或无界分配 |
| HGI-G34 | golden绑定scene/camera/exposure/source/build/shader/driver并保存linear HDR artifact |
| HGI-G35 | benchmark报告CPU/GPU/VRAM/RSS/I/O/质量误差/统计置信且可复现 |
| HGI-G36 | 在全部correctness/failure/soak gate前不宣称达到或超过Unreal |

## 11. 验证边界与状态

| 项目 | 状态 | 证据 |
|---|---|---|
| 全tracked package inventory | review_complete | 270 files、43,848 lines、1,579,121 bytes |
| production path inventory | review_complete | 221 files、29,806 lines、1,099,143 bytes；fingerprint见2.1 |
| WGSL语义审查 | review_complete | 16 shaders；固定8 x 8、32-slot RC、4^3 voxel、16 page packet等均由current source确认 |
| package/editor/dist/catalog/profile审查 | review_complete | 缺失ZUI、metadata-only carrier、Editor default矛盾均由调用链确认 |
| reference engine对照 | review_complete | Unreal、Unity Graphics、Godot、Bevy、Fyrox production source |
| Cargo/GPU/Editor/native/跨平台/性能验证 | not_run | review-only；不能据此宣称current tests passing |
| Production重构 | pending | 本篇只写review与重构计划 |

本报告完成的是 Hybrid GI package/product vertical 的首轮工程审查，不是 Hybrid GI 实现完成。下一步实施必须从 Runtime09F3 的 P0 和本篇 M0/M1 开始，先恢复 capability truth、唯一 graph owner 与 RenderScene generation；不得继续在固定容量 packet、CPU mirror和ignored PNG之上叠加更多“已实现”名称。
