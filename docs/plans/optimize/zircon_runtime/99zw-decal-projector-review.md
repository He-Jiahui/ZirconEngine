---
title: Runtime Decal Projector、Material Domain、DBuffer、GBuffer、Forward、Receiver、Culling、Batching、Atlas、Streaming、Temporal、RT、Scalability、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime148
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
historical_refresh_of: Runtime35
related_code:
  - zircon_plugins/rendering/features/decals/runtime/src
  - zircon_plugins/rendering/features/decals/editor/src
  - zircon_plugins/rendering/plugin.toml
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/core/framework/scene/component_type_descriptor
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/material/mod.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/graphics/scene/resources
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui
plan_sources:
  - docs/plans/optimize/zircon_runtime/35-decal-projector-material-domain-dbuffer-gbuffer-forward-receiver-culling-batching-atlas-streaming-temporal-rt-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/DecalComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SceneProxies/DeferredDecalProxy.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DecalRenderingCommon.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DecalRenderingShared.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/CompositionLighting/PostProcessDeferredDecals.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/CompositionLighting/PostProcessMeshDecals.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MobileDecalRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RayTracing/RayTracingDecals.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Decal
  - dev/godot/scene/3d/decal.cpp
  - dev/godot/tests/scene/test_decal.cpp
  - dev/bevy/crates/bevy_pbr/src/decal
  - dev/Fyrox/fyrox-impl/src/scene/decal.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/decal.shader
---

# Runtime Decal Projector、DBuffer/GBuffer 与 Product Integration 当前源码工程化差距

## 1. 结论

当前 Zircon 的 Decal 仍是一个会通过注册、会被图执行记录为已执行、但不会编码任何 GPU work 或产生任何像素的错误完成面。`rendering.decals` 定义 `DecalProjectionMode::{ScreenSpace, Deferred}` 和仅含 `mode/opacity/normal_blend/atlas_region` 的 `DecalProjectorDescriptor`；它注册一个 `PostProcess` pass，读取 `scene-depth/scene-color` 并写回 `scene-color`，却把 `decals.projector-composite` 绑定到只返回 `Ok(())` 的 `noop_render_executor`。唯一 Decal test 只检查 registration、component type 和 pass name。

当前源码没有 Decal material domain、`.zmaterial` domain、channel/write-mask/blend artifact、projector material reference、Scene project payload、World proxy、`RenderFrameExtract` Decal snapshot、bounds/culling/sort/batch、DBuffer/GBuffer attachment、forward/transparent/mesh/RT adapter、atlas/residency、temporal history、diagnostics、pixel test或产品场景。Pipeline 还把 `decals` 排在 baked lighting、reflection probes 与 bloom 之后；这只能表达晚期 scene-color composite，不能表达 pre-lighting DBuffer/GBuffer。`BuiltinRenderFeature::{Decal, Projector}` 被明确列为 descriptor-only slot，这是诚实 Unsupported 基础，不是可执行实现。

排除 tests/test_sources/benches/examples/target 目录与 test-named 文件后的 **12,060 个 production Rust 文件**中，`DecalProjectorDescriptor` 只在 Decal runtime package 的1个文件出现，`decals.projector-composite`只在该文件出现；独立单词 `DBuffer` 为 **0个文件/0条**，独立 `Decal` 和 `Projector` 均仅命中 builtin enum、descriptor-only slot 与 Decal package 3个文件。examples/templates/App 没有独立 Decal 产品消费点。`atlas_region` 的其余命中属于 Sprite atlas，不能成为 Decal atlas consumer。

历史 Runtime35 的 72项P1按当前 working bytes 重判为 **30 Open / 42 Partial / 0 Closed**，16项P2全部Open；40项资格门为 **23 Fail / 17 Partial / 0 Pass**。Partial只表示通用 asset、DynamicScene transaction、render graph、visibility、history、residency、quality或RT owner可复用，不表示 Decal 数据已经贯通。本文不新建 P0 owner：重新确认 Editor39 的4项 Decal P0 与 Plugins04 的 preview/export 装配P0；同时记录 Plugins04 的 metadata-only capability false-publication 已由 concrete-provider admission fail-close，旧结论不能继续写成Open。目标必须硬切到：

```text
DecalMaterialSource(domain/channels/textures/blend/receiver/stages)
  -> deterministic DecalMaterialCompiler
  -> DecalMaterialArtifact + Variant/PSO/Dependency/Capability Receipt

DecalProjectorSource(material/transform/size/pivot/UV/sort/fade/mask/lifetime)
  -> project Scene roundtrip + DynamicScene/World bridge
  -> generation-qualified DecalInstance/Proxy
  -> DecalRenderSnapshot(bounds/material/flags/current-previous)
  -> per-view cull + receiver/stage classification + sort/group/batch
  -> DBuffer/GBuffer/Forward/Transparent/Mesh/RT adapters
  -> execution/fallback/overflow/quality/terminal receipts
```

## 2. 审查边界、方法与 currentness

### 2.1 冻结范围

本文记录读取时 `main@1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8` 的 selected working bytes。写入前共享工作树附近有 **3,314 个 tracked changes、1,234 个 untracked paths**；其他Session持续推进，本文不归因、不覆盖、不回退既有改动。实施前必须重取下列指纹并做source recheck。用户明确暂不优化tooling，本轮未扫描或规划未来将迁移到Rust的tooling实现。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树指纹 |
|---|---:|---|
| Zircon Decal package、catalog 与产品真相 | **294 / 34,582 / 31,620 / 1,282,078 / 477 / 0** | `dd89cb2fc33e5df19b6dd302e93db0d5814ad9dd6e14ff65e9bf9fc28dcab451` |
| Zircon Scene、Material 与 Extract 前置 | **656 / 24,524 / 22,159 / 849,360 / 56 / 0** | `32f2b0e0682a8be2bf8d9401fc0599de138fa812167105c6f9cc93f6de81ec1e` |
| Zircon Render/Visibility/Residency/Temporal 前置 | **415 / 78,755 / 72,655 / 2,924,036 / 743 / 0** | `56660f227a1899e4093bc6194ec2bc8f08fd6654ec3341b78ca5ef87b80c06d9` |
| Unreal Decal runtime、renderer 与 shader | **19 / 7,075 / 6,028 / 294,936 / 0 / 0** | `b6e57885cf8d28143aac4ade641aed45f97f1a3c2b804e10a625d1da716af255` |
| Unity HDRP/URP Decal runtime、editor 与 tests | **59 / 12,250 / 10,534 / 538,645 / 1 / 0** | `12cbd0429d81d2c2b891deddace475cfa5c328a1416d90b836bd7a6e1cd9a1cd` |
| Godot Decal node、renderer、editor 与 tests | **6 / 791 / 650 / 34,581 / 1 / 0** | `7c268a8be91a080049aa3601d9bc8bd114adc4061aeb8a968f20f4e3d009c0a0` |
| Bevy/Fyrox Decal 实现 | **7 / 1,533 / 1,372 / 55,104 / 0 / 0** | `a5aade3808ef66805f2ca67cd5d543060a695fde16a94393786dd66e5b7aa971` |

指纹算法为：repository-relative path转`/`并小写排序；每文件取当前bytes的lowercase SHA-256；聚合输入为`path|file_sha256`以单个LF连接且末尾无LF，再对UTF-8 payload取SHA-256。tests/ignored使用Rust、NUnit/Unity Test与Godot `TEST_CASE`静态声明检测。各选择集是证据边界，不应当作仓库唯一文件总量。

### 2.2 纵向扫描链

本轮逐层核对 package/capability -> material source/domain/schema -> compiler/artifact/variant/PSO -> projector component/property contract -> project Scene/DynamicScene -> World instance/lifecycle -> frame extract -> transform/depth reconstruction/clipping -> bounds/visibility/receiver -> sorting/grouping/batching -> DBuffer/GBuffer/forward/transparent/mesh -> write mask/blend/normal -> atlas/bindless/residency -> multi-view/MSAA/DRS/temporal -> GI/shadow/RT/cross-system -> quality/diagnostics/tests/App/Editor/product evidence。PowerShell覆盖tracked与untracked working bytes；`rg.exe`受本机Windows Store执行权限阻断，精确复查改用`git grep`与`Select-String`，未缩小源码边界。

### 2.3 证据等级与执行限制

本文达到E3 source-level review。没有运行Cargo、WGPU、App/Editor、PIE、asset cook、Scene roundtrip、GPU capture、fault/fuzz、scale/soak或竞争benchmark。源码足以证明 noop、数据断路、stage错误与产品断点；但未来的投影精度、write mask、blend、normal、temporal、跨后端像素与性能必须由实现后的CPU oracle、GPU golden/capture和raw receipt证明，普通render graph/material/deferred测试不能提高Decal完成度。

## 3. 当前可保留的真实基础

1. Runtime plugin能显式注册component descriptor、render feature与executor；catalog现在会阻止缺少concrete feature provider的metadata-only feature发布capability。
2. `DynamicScene`有versioned reflected component payload和事务式preflight/commit spawn；World有dynamic component registry、type generation和inspection基础。
3. Render graph有typed resource access identity、attachment/load-store/lifetime、queue、executor registry、GPU context、profiling与resource resolver。
4. Deferred renderer、material dependency/readiness、shader variant/PSO、visibility/HZB/indirect、ResourceStreamer、bindless、TAA reactive/current-previous、quality/budget和RT均有通用owner方向。
5. `Decal/Projector` descriptor-only slots能诚实表达“只有描述、没有runtime pass”，可作为硬禁用和capability truth的起点。

这些基础均没有 Decal-specific source、artifact、instance、prepared work或consumer，因此只对应下表Partial项。

## 4. 当前代码事实与断路

### 4.1 Package、Capability 与 Product Reachability

1. `plugin.toml`把Rendering umbrella标记为`stable`并把根capability标为`complete`；Decal optional feature声明runtime/editor capability，但没有Decal-specific maturity/status/qualification receipt。
2. Catalog的metadata-only feature已能因缺少concrete provider而fail-close，这是Plugins04 P0-001的真实修复；但普通Editor bootstrap只提交root plugin registrations，generated export可以提交Decal feature registration，preview/export仍不是同一装配协议。
3. generated export若选择Decal，会加载真实registration，但该registration最终执行noop并被graph记录为executed。concrete registration不等于行为资格。
4. Editor Decal package只提供descriptor与capability常量；没有drawer、toolkit、handle、preview、transaction、diagnostic或runtime receipt consumer。

### 4.2 Source、Scene、Instance 与 Extract

1. `DecalProjectorDescriptor`没有material、transform/size/pivot、UV、sort、distance/angle/lifetime fade、receiver mask、color、mobility或stable identity；`atlas_region`只是未被Decal消费者读取的String。
2. `ComponentPropertyDescriptor`只有`name/value_type/editable`；没有stable property ID、default、range、unit、asset kind、finite validator、schema version或migration。
3. `ZMaterialDocument` v2保存shader/parent/options/overrides/textures/queue/editor，没有domain；唯一`MaterialDomain`只有Surface/PostProcess/DebugOverlay/LightFunction。
4. `SceneEntityAsset`仍是camera/mesh/light/post-process/physics/animation/terrain/tilemap/prefab/script固定字段集合，没有plugin component payload。`DynamicScene`虽有`DynamicComponent`，asset/project bridge没有消费者，不能证明save/reopen/play/export。
5. `RenderFrameExtract`只有view/geometry/animation/lighting/environment/post-process/debug/sprites/particles/visibility，没有projector snapshot或prepared Decal work。

### 4.3 Render、Projection 与 Receiver

1. Decal pass被固定为`PostProcess`，位于baked lighting/reflection probes/bloom之后；没有pre-lighting DBuffer/GBuffer stage。
2. pass只声明scene depth/color，没有MRT layout、write mask、blend state、normal encoding、stencil、projector volume、inside/outside raster、receiver response或forward fallback。
3. executor忽略具有GPU/resource/streamer访问能力的context并直接`Ok(())`；执行框架随后仍提交pass profile和executed pass记录，形成false-success telemetry。
4. visibility、GPU Scene、phase sort、indirect与budget没有Decal producer；没有bounds、frustum/size/distance/angle cull、overlap precedence、stage classification、batch或overflow receipt。

### 4.4 Editor、Tests 与产品证据

1. Material Workbench dropdown继续公开`surface/post_process/decal`，但选择值只是UI状态与journal，不会进入`.zmaterial`、compiler、variant或pass。
2. Decal runtime唯一test只验证registration；catalog/App tests只验证feature ID或crate projection，没有Scene/projector roundtrip、shader compile、projection math、command encoding或pixel output。
3. examples/templates/App没有独立Decal、projector或DBuffer消费点；没有bullet mark、blood、wetness、road marking、graffiti、terrain/skinned receiver场景。

## 5. 参考实现给出的工程边界

### 5.1 Unreal：Proxy、visibility task与render stage是独立责任

`UDecalComponent`持material、size、sort、screen fade、lifetime fade和color，并建立render-thread `FDeferredDecalProxy`，proxy保存transform/bounds/material/fade/sort。Renderer先构建per-view visible/relevant list，再按`EDecalRenderStage`与target mode选择write mask、blend、shader、raster/stencil state；inside volume、inverted transform和view reverse culling有专门决策。另有mesh decal、mobile、ray tracing与path tracing路径。Zircon不能把这些语义压进一个PostProcess token。

### 5.2 Unity HDRP/URP：实例生命周期、technique与资源策略分层

HDRP `DecalProjector`公开material、size/offset、draw distance/fade、angle fade、UV、layer和transparent policy；`DecalSystem`按material/set维护culling、jobs、draw data和atlas。URP以`DecalEntityManager`和entity/cached/culled/draw-call chunks管理add/update/remove/sort，独立systems执行cached update、culling、draw-call generation与draw；renderer feature按DBuffer、ScreenSpace、GBuffer及forward emissive选择真实pass。其runtime test至少覆盖chunk销毁/压缩生命周期。

### 5.3 Godot、Fyrox、Bevy：较小方案仍有闭合数据路径

Godot `Decal`是可持久化Scene node，持四类texture、size、color、normal/upper/lower/distance fade和cull mask，setter直接更新RenderingServer RID与AABB；测试覆盖默认和各属性/bounds。Fyrox `Decal`可Reflect/Visit，持diffuse/normal texture、color、layer与bounds，shader从scene depth重建world position、投影local volume并写color/normal。Bevy forward/clustered decal贯通main World extract、render World prepare、storage buffer、binding array、global clusterable meta和WGSL，并明确bindless、Metal/iOS及最大binding限制。最低可接受线也远高于“注册成功+noop”。

## 6. 唯一 Owner 与父依赖

| 领域 | 唯一owner | 本篇只消费/提供 |
|---|---|---|
| Decal产品真实性与authoring | Editor39 | Inspector/handles/transaction/preview及P1-50..60父要求 |
| Package/catalog/capability | Plugins04/Runtime catalog owner | provider admission、preview/export装配、Declared->Qualified truth |
| Decal executable runtime | Runtime148（历史Runtime35） | material specialization、projector instance、extract、stage work、receiver与runtime receipts |
| Resource/schema/DDC | Runtime Asset owner | source/artifact identity、dependency、migration、LKG/install/retire |
| Scene/world lifecycle | Runtime Scene owner | project plugin payload bridge、world generation、transactional spawn/unload |
| Render graph/RHI | Runtime Render Graph owner | attachment/hazard/load-store/pipeline/resource/device recovery primitive |
| Visibility/GPU Scene | Runtime94 | generic bounds/cull/indirect primitive；本篇提供projector volume/stage work |
| Material/shader/PSO | Runtime91 + Editor15 | domain/compiler/variant/PSO primitive；本篇定义Decal specialization |
| Residency/streaming | Runtime Render Asset owner | atlas/bindless/streaming admission、pressure、fallback与retirement |
| Temporal/GI/RT/receiver domains | 对应Runtime owner | 消费同代Decal artifact/instance/output，不复制Decal truth |

## 7. 父P0当前复核

本文 **0项新P0**。下表只重新确认父owner状态，不转移fixing owner。

| 父ID | 状态 | 当前复核 |
|---|---|---|
| Editor39 P0-01 | Open | Decal executor仍直接`Ok(())`，且执行框架会记录executed pass；真实像素闭环前必须Disabled/Unsupported或fail-close |
| Editor39 P0-03 | Open | project `SceneEntityAsset`仍无plugin component payload；`DynamicScene`通用payload没有接入project save/reopen/play/export |
| Editor39 P0-04 | Open | Workbench仍公开`decal`，但MaterialDomain和`.zmaterial`没有Decal domain |
| Editor39 P0-05 | Open | 静态UI/descriptor/catalog仍可形成产品外观，尚无runtime-backed operation/artifact/receipt |
| Plugins04 P0-001 | Closed | feature resolution现在检查`has_concrete_feature_provider`，metadata-only feature会provider-missing、阻止capability/extension publication，并有1000-row fail-close test |
| Plugins04 P0-002 | Open | 普通Editor bootstrap仍不提交feature registrations，generated export提交；同一project selection的preview/export执行面不一致 |

## 8. P1：Material Source、Projector Schema 与 Compiler

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEC-P1-001 | Open | `MaterialDomain`无Decal；在唯一domain authority增加Decal并迁移asset/Workbench/graph/variant/pass reader |
| DEC-P1-002 | Open | `.zmaterial` v2无domain；持久化domain/schema/unknown policy/migration，禁止UI字符串旁路 |
| DEC-P1-003 | Open | 无base/normal/ORM/emissive/opacity typed affect/write mask schema |
| DEC-P1-004 | Open | 无blend/stain/normal/DBuffer policy artifact；compile为stage/target/write-mask/blend state |
| DEC-P1-005 | Open | surface material无receive/response channel合同；不兼容组合必须compile fail-close |
| DEC-P1-006 | Open | shader graph无Decal output、inputs、derivative/mip/lifetime与unsupported node诊断 |
| DEC-P1-007 | Partial | generic variant/PSO key可复用；补domain/technique/stage/target/channels/MSAA/view/platform/generation |
| DEC-P1-008 | Partial | generic material dependency/readiness/fallback可复用；补Decal texture/sampler/atlas/bindless/provenance receipt |
| DEC-P1-009 | Open | projector descriptor仍非typed source schema；补material/size/pivot/UV/sort/fade/mask/lifetime/color/mobility |
| DEC-P1-010 | Open | property仅name/type/editable；补stable ID/default/range/unit/asset-kind/finite validator/version/migration |
| DEC-P1-011 | Open | Rust descriptor/default与registered reflection字段仍是两套未连接定义 |
| DEC-P1-012 | Partial | generic artifact hash/cache/publication可复用；补deterministic compiler、target key、diagnostic和LKG |

## 9. P1：Scene、Runtime Instance、Lifecycle 与 Extract

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEC-P1-013 | Partial | `DynamicScene`有generic payload，但project Scene无bridge；建立versioned plugin component roundtrip，禁止临时Option字段 |
| DEC-P1-014 | Partial | generic DynamicScene preflight/commit可复用；补Decal type/material/dependency原子spawn和零可见失败 |
| DEC-P1-015 | Partial | generic Entity/World generation可复用；补persistent projector ID与slot/generation分离、stale拒绝 |
| DEC-P1-016 | Partial | World dynamic component基础可复用；补Decal instance/proxy、bounds/fade/flags和resource leases |
| DEC-P1-017 | Partial | generic task/lifecycle模式可复用；补Requested/Preparing/Active/Suspended/Retiring/Failed/Cancelled唯一终态 |
| DEC-P1-018 | Partial | generic component/asset change基础可复用；补transform/property/material/visibility/layer dirty propagation |
| DEC-P1-019 | Partial | generic slot/span/batch容器可复用；补generation-qualified add/update/remove、compaction和sort rebuild |
| DEC-P1-020 | Partial | generic asset reload/LKG前置存在；补projector/material schema migration、rebind、fallback和失败原因 |
| DEC-P1-021 | Partial | per-World与Editor/PIE隔离基础可复用；Decal fade/sort/atlas/instance mutable truth仍为空 |
| DEC-P1-022 | Partial | generic device/resource drain前置存在；补先停extract/work、fence retire及旧async不可复活 |
| DEC-P1-023 | Open | `RenderFrameExtract`没有Decal SoA snapshot、inverse/bounds/material/flags/fade/current-previous |
| DEC-P1-024 | Partial | immutable frame publish基础可复用；补Scene/world/material/residency同代冻结合同 |

## 10. P1：Projection、Visibility、Receiver、Sorting 与 Batching

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEC-P1-025 | Open | 无local box、size/pivot、component/world/view/clip/camera-relative及nonuniform scale合同 |
| DEC-P1-026 | Partial | camera/reverse-Z/DRS通用数学可复用；补Decal perspective/ortho/jitter/subrect depth reconstruction oracle |
| DEC-P1-027 | Open | 无world->local volume clipping、epsilon、near-plane、inside-camera和degenerate处理 |
| DEC-P1-028 | Open | 无projection direction、surface normal、upper/lower/angle fade和normal source资格 |
| DEC-P1-029 | Partial | generic bounds类型可复用；补size/pivot/transform生成OBB/AABB/sphere和origin同代更新 |
| DEC-P1-030 | Partial | generic per-view visibility可复用；补screen-size/distance/fade/layer/stereo/capture cull |
| DEC-P1-031 | Partial | render layer/material基础可复用；补receiver response/object category/terrain/water/vegetation/transparent策略 |
| DEC-P1-032 | Partial | generic phase sort有稳定key方向；补Decal sort/material/distance/stable-ID overlap precedence |
| DEC-P1-033 | Open | 无DBuffer/GBuffer/emissive/AO/forward/mobile/RT stage classification artifact |
| DEC-P1-034 | Partial | generic material grouping可复用；补artifact/variant/target/blend/sampler/atlas work grouping |
| DEC-P1-035 | Partial | HZB/GPU Scene/indirect前置存在；补volume cull、visible compaction、draw args、CPU oracle和overflow receipt |
| DEC-P1-036 | Partial | generic frame budget/profiler可复用；补total/visible/culled/dropped/pixels/batches/bytes per-view admission |

## 11. P1：DBuffer、GBuffer、Forward、Shader 与 Pass Correctness

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEC-P1-037 | Open | pass仍在late PostProcess；按依赖插入pre-lighting DBuffer/GBuffer及emissive/AO/forward stages |
| DEC-P1-038 | Partial | render graph typed attachment/lifetime可复用；补DBuffer/GBuffer MRT format/clear/load/store/sample/readers |
| DEC-P1-039 | Partial | generic color target state可复用；补artifact-driven per-target channel write mask |
| DEC-P1-040 | Partial | generic blend state可复用；补base/normal/ORM/emissive/AO预乘/非预乘矩阵与backend parity |
| DEC-P1-041 | Open | 无world/tangent normal、encode/decode、RNM/lerp和zero-strength parity |
| DEC-P1-042 | Open | base material/lighting没有DBuffer decode/apply与baked/dynamic lighting语义 |
| DEC-P1-043 | Partial | graph/pipeline/GPU context可复用；补cube/fullscreen/clustered projector pipeline和depth/cull/stencil state |
| DEC-P1-044 | Open | 无inside/outside、mirrored transform、reverse culling和near-plane raster policy |
| DEC-P1-045 | Open | 无stencil/prepass/cluster overdraw策略，无法限制深重叠成本 |
| DEC-P1-046 | Partial | generic sampler/mip/atlas基础可复用；补projector derivative、mip bias、anisotropy、padding与bleed门 |
| DEC-P1-047 | Open | 无clustered forward、transparent receiver或mesh decal；不支持时也没有compile/activation failure |
| DEC-P1-048 | Open | executor仍noop且返回成功；真executor必须要求prepared input/GPU/resources并编码可观测work |

## 12. P1：Temporal、Multi-View、Streaming、RT 与 Cross-System

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEC-P1-049 | Partial | generic current/previous transform可复用；补projector/receiver/spawn/remove/teleport/rebase history generation |
| DEC-P1-050 | Partial | TAA reactive/reset前置存在；补opacity/fade/animation/LOD/atlas replacement disposition |
| DEC-P1-051 | Partial | jitter/DRS/view rect基础存在；补projection/depth/UV在render/display坐标域的一致性 |
| DEC-P1-052 | Partial | view family/camera stack基础可复用；补per-eye、single-pass、capture/reflection/portal/overlay资格 |
| DEC-P1-053 | Partial | generic MSAA配置可复用；补depth resolve/per-sample edge/DBuffer sample/A2C pixel门 |
| DEC-P1-054 | Partial | generic atlas/resource identity前置存在；补region generation、mips/padding、format class、fragmentation/compaction/lease |
| DEC-P1-055 | Partial | bindless capability检测可复用；补binding limit、partial binding、Metal/Web/mobile admission fallback |
| DEC-P1-056 | Partial | ResourceStreamer/residency可复用；补importance/mip/prefetch/pressure eviction/fallback/in-flight retirement |
| DEC-P1-057 | Partial | mesh/skinning/virtual geometry前置存在；补deformed receiver depth/normal/response与LOD identity |
| DEC-P1-058 | Open | terrain/water/vegetation没有typed receiver adapter；各域不得复制Decal material/instance truth |
| DEC-P1-059 | Partial | generic lightmap/GI/shadow可复用；补DBuffer/static-baked/emissive GI/capture update语义 |
| DEC-P1-060 | Partial | generic RT pipeline前置存在；补projected/mesh decal BLAS/TLAS/hit/fallback、alpha/channel/sort parity |

## 13. P1：Scalability、Diagnostics、Tests 与 Product

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEC-P1-061 | Partial | generic quality profile可复用；补max decals/distance/channels/technique/atlas/RT/transparent联合策略 |
| DEC-P1-062 | Partial | generic memory/time budget可复用；补instance/visible/overlap/pixels/batches/textures/bytes/CPU/GPU admission |
| DEC-P1-063 | Partial | descriptor-only与backend capability可表达Unsupported；补deferred/forward/mobile/MRT/blend/MSAA/bindless/RT矩阵 |
| DEC-P1-064 | Partial | generic profiler/graph diagnostics可复用；补instance/material/stage/cull/sort/batch/atlas/residency/fallback/timing |
| DEC-P1-065 | Open | 无Decal domain/channel/blend/stage/property/migration/artifact digest tests |
| DEC-P1-066 | Open | 无save/reopen/spawn/reload/unload/multi-world/stale generation/cancel/device-loss tests |
| DEC-P1-067 | Open | 无perspective/ortho/reverse-Z/jitter/DRS/stereo/inside/mirror/rebase projection tests |
| DEC-P1-068 | Open | 无albedo/normal/ORM/emissive/overlap/fade/mip/DBuffer/forward/MSAA/temporal pixel golden |
| DEC-P1-069 | Open | 无malformed source、missing texture、atlas OOM、overflow、shader fail、device loss fuzz/fault矩阵 |
| DEC-P1-070 | Open | 无bullet/blood/rain/wetness/road/graffiti/mesh/terrain/skinned save-play-export-capture场景 |
| DEC-P1-071 | Open | 无1/100/10K/deep-overlap/fast-camera/stream-churn CPU/GPU/memory/overdraw/stutter基准 |
| DEC-P1-072 | Open | 无同资产/视角/receiver/硬件/画质的Unreal/Unity图像与raw performance对照 |

## 14. P2：P1闭环后才能进入的增强

| ID | 状态 | 前置 |
|---|---|---|
| DEC-P2-001 | Open | spline/ribbon decal；先完成Editor39 SpatialSpline artifact、stable segment ID、partition、UV和batch |
| DEC-P2-002 | Open | arbitrary mesh decal；先完成mesh clipping/offset、attachment、skinning、LOD和RT parity |
| DEC-P2-003 | Open | persistent damage/blood；先完成surface identity、save/network、budget/cleanup/material merge |
| DEC-P2-004 | Open | virtual texture decal baking；先完成VT page ownership、transaction、undo/residency/fallback |
| DEC-P2-005 | Open | terrain layer baking；先完成Terrain layer/cell artifact、rebuild/LKG和projector parity |
| DEC-P2-006 | Open | water current/foam；先完成Water surface UV/current/simulation artifact与adapter |
| DEC-P2-007 | Open | vegetation conforming decal；先完成instance/representation/wind deformation与receiver policy |
| DEC-P2-008 | Open | dynamic topology receiver；先完成surface correspondence、history、bounds和RT update |
| DEC-P2-009 | Open | decal lighting/shadow；先定义thickness/opacity/light transport、oracle和cost gate |
| DEC-P2-010 | Open | path-traced decal material；先完成pipeline/SBT/callable、sort/alpha和raster parity |
| DEC-P2-011 | Open | compute raster/tiling；先完成CPU/raster oracle、tile overflow、ordering、MSAA和portable fallback |
| DEC-P2-012 | Open | GPU persistent cluster；先完成stable identity、mutation log、compaction、readback/fault/multi-world |
| DEC-P2-013 | Open | authoritative network decals；先完成persistent ID、interest/late join/rollback/save/entitlement |
| DEC-P2-014 | Open | third-party provider SDK；先完成ABI/version/capability/budget/sandbox/unload/artifact compatibility |
| DEC-P2-015 | Open | collaborative authoring；先完成stable ID、transaction/merge/locking/recovery/provenance |
| DEC-P2-016 | Open | distributed visual qualification farm；先完成frozen BuildSet、GPU/driver capture/raw receipt/promotion |

## 15. 分层重构路线

| 里程碑 | 内容 | 退出条件 |
|---|---|---|
| M0 · Truth Cutoff | hard-disable noop、移除/禁用false UI、统一preview/export feature registration与capability状态 | 父P0-01/04/05及Plugins04 P0-002关闭；未实现路径typed Unsupported |
| M1 · Material & Source | Decal domain、source/projector schema、compiler/artifact/dependency/variant/PSO | deterministic roundtrip/migration/bad-input/target-key RED->GREEN |
| M2 · Scene & Instance | project plugin payload bridge、stable ID、transaction spawn、instance/proxy/lifecycle/extract | save/reopen/play/export与stale-generation/cancel/unload门通过 |
| M3 · Projection & Visibility | transform/depth oracle、bounds、per-view cull、receiver、sort/group/indirect | reverse-Z/ortho/jitter/DRS/stereo/inside-volume CPU oracle通过 |
| M4 · Render Correctness | DBuffer/GBuffer/forward passes、attachments、write mask/blend/normal、真executor | nonzero commands与channel/normal/lighting pixel golden通过 |
| M5 · Temporal & Streaming | history/reactive、MSAA、atlas/bindless/residency/fallback | camera/material/atlas churn无ghost/bleed/stale sampling |
| M6 · Cross-System | mesh/VG/terrain/water/vegetation、GI/shadow/RT typed adapters | unsupported fail-close；批准路径同代identity/parity通过 |
| M7 · Scalability & Diagnostics | platform matrix、joint admission、budget/degrade、runtime inspection/fault | 1/100/10K与OOM/device loss/overflow raw receipt通过 |
| M8 · Editor & Product | runtime-backed drawer/handles/preview、六类真实场景、save-play-export-capture | Editor39父要求和产品fixture闭环，无静态成功态 |
| M9 · Qualification | frozen BuildSet、driver/hardware matrix、Unreal/Unity同场景图像与性能对照 | 40门全有可复跑accepted receipt，旧receipt按source/device变化失效 |

## 16. 资格门当前状态

| Gate | 状态 | 当前判定 |
|---|---|---|
| DEC-G01 | Fail | executor仍返回可解释为成功的`Ok(())` |
| DEC-G02 | Partial | metadata-only admission已fail-close；concrete noop provider仍可发布capability且无qualification receipt |
| DEC-G03 | Fail | `.zmaterial`、runtime domain、Workbench和pass没有单一Decal authority |
| DEC-G04 | Fail | 无channel/blend/stage artifact与deterministic digest |
| DEC-G05 | Fail | property schema没有stable ID/default/range/unit/asset kind/version/migration |
| DEC-G06 | Fail | project Scene不能roundtrip plugin Decal payload |
| DEC-G07 | Partial | generic generation/cancel/drain可复用；无Decal instance/task/resource链 |
| DEC-G08 | Partial | generic multi-world前置存在；无Decal mutable state可验证隔离 |
| DEC-G09 | Fail | frame extract没有projector snapshot |
| DEC-G10 | Partial | reverse-Z/相机数学前置存在；无Decal depth oracle |
| DEC-G11 | Partial | jitter/DRS/view rect前置存在；无Decal UV/projection parity |
| DEC-G12 | Fail | inside/near-plane/mirror/nonuniform/degenerate volume未实现 |
| DEC-G13 | Partial | generic visibility可复用；无Decal cull reason/oracle |
| DEC-G14 | Partial | generic stable sort primitive可复用；无Decal overlap key |
| DEC-G15 | Fail | 无stage artifact/classification |
| DEC-G16 | Partial | generic compaction/indirect前置存在；无Decal overflow work/receipt |
| DEC-G17 | Partial | graph attachments可表达；无DBuffer/GBuffer声明或validation |
| DEC-G18 | Fail | 无per-channel write mask/blend像素结果 |
| DEC-G19 | Fail | 无normal encode/blend实现或oracle |
| DEC-G20 | Fail | 无pre-lighting DBuffer apply |
| DEC-G21 | Fail | 无volume cull/stencil/depth state |
| DEC-G22 | Fail | 无projector mip/derivative/atlas padding实现 |
| DEC-G23 | Fail | forward/transparent/mobile既不支持也不显式失败 |
| DEC-G24 | Fail | 真executor、prepared input和nonzero work均不存在 |
| DEC-G25 | Partial | generic TAA reactive/history reset可复用；无Decal producer |
| DEC-G26 | Partial | generic view family可复用；无Decal per-view generation |
| DEC-G27 | Partial | generic MSAA可配置；无Decal edge/DBuffer golden |
| DEC-G28 | Partial | generic resource generation/retire前置存在；无Decal atlas region lifecycle |
| DEC-G29 | Partial | bindless capability检测存在；无Decal admission path |
| DEC-G30 | Partial | ResourceStreamer/fallback前置存在；无Decal pressure behavior |
| DEC-G31 | Fail | 无terrain/water/vegetation/mesh/VG receiver adapter |
| DEC-G32 | Partial | GI/shadow/RT前置存在；无同代Decal consumer |
| DEC-G33 | Partial | generic profiler/graph记录存在；没有Decal totals/cull/batch/atlas/fallback |
| DEC-G34 | Fail | malformed/OOM/shader/device-loss矩阵为空 |
| DEC-G35 | Fail | 当前唯一Decal test仍以registration成功为断言 |
| DEC-G36 | Fail | 六类产品fixture与save/reopen/play/export/capture为空 |
| DEC-G37 | Fail | 1/100/10K与深重叠raw receipts为空 |
| DEC-G38 | Fail | Runtime子门与Editor39 authoring父门均未闭合 |
| DEC-G39 | Fail | 无Decal accepted image/perf receipt可执行currentness失效 |
| DEC-G40 | Fail | 无同资产/receiver/视角/硬件/画质的Unreal/Unity超越证据 |

## 17. Finding 到里程碑映射

| Finding | 里程碑 |
|---|---|
| Editor39 P0-01/03/04/05、Plugins04 P0-002 | M0；fixing owner不变 |
| DEC-P1-001..012 | M0-M1 |
| DEC-P1-013..024 | M2 |
| DEC-P1-025..036 | M3 |
| DEC-P1-037..048 | M4 |
| DEC-P1-049..060 | M5-M6 |
| DEC-P1-061..072 | M7-M9 |
| DEC-P2-001..016 | 对应P1与资格门完成后独立立项，不得提前并入MVP |

## 18. 禁止的临时修补

1. 禁止只把noop改名、登记更多enum/property/pass或在test里继续断言registration success。
2. 禁止把late scene-color fullscreen composite包装成DBuffer/GBuffer或Deferred支持。
3. 禁止在`SceneEntityAsset`追加Decal专用`Option`绕过通用plugin component project payload。
4. 禁止继续使用`atlas_region: String`代替material/texture/region generation、lease和residency。
5. 禁止由Editor、Scene、Renderer、Terrain/Water/Vegetation各保存一份Decal mutable truth。
6. 禁止无bounds/receiver/sort/overflow时逐Decal fullscreen，或以永久放大bounds掩盖culling错误。
7. 禁止只写scene color却宣称normal/ORM/emissive channel已支持。
8. 禁止只修main view；stereo、capture、reflection、portal、MSAA、DRS与TAA必须使用同代数据。
9. 禁止把generic graph、HZB、bindless、RT或descriptor-only slot计为Decal功能完成。
10. 禁止硬编码Ready、executed、queued、count、timing或成功toast；必须来自typed runtime receipt和可复跑像素证据。

## 19. 实施前重查清单

1. 重算本文七组fingerprint，记录新增/删除/修改及working-tree来源。
2. 重查Decal executor是否仍noop、pass是否仍PostProcess且位于bloom后；任何移动必须验证真实资源/命令而非名称。
3. 重查Plugins04 concrete-provider admission与preview/export装配，禁止重新开放metadata capability。
4. 重查MaterialDomain、`.zmaterial`、Workbench、shader graph、variant/PSO是否已收敛为一个Decal authority。
5. 重查project Scene plugin payload bridge与DynamicScene transaction，要求save/reopen/play/export和migration。
6. 重查`RenderFrameExtract`、visibility、prepared work、stage classification、DBuffer/GBuffer attachments和executor输入generation。
7. 重查atlas/bindless/residency、temporal/MSAA/multi-view、GI/shadow/RT及receiver adapter的fail-close。
8. 重查tests是否从registration/enum升级到roundtrip、CPU oracle、nonzero command、pixel golden、fault/scale/raw receipt。
9. 锁定M0-M9每阶段BuildSet、target/backend、hardware/driver、quality、warm-up和promotion规则。

## 20. 本轮产出边界

本篇只完成静态源码审查、五引擎参考对照、唯一owner划分、父P0 currentness复核、历史P1/P2逐项状态重判、分层重构路线与40项资格门，没有修改production代码、Cargo、测试、workflow、UI或产品资产，也没有证明任何Decal像素、性能或表现已完成。MVP主线仍处于00基础阶段，本轮只做允许的C3只读高级审查；后续实现必须先关闭false-success与preview/export真相，再从唯一material/projector source、project Scene roundtrip、deterministic compiler和CPU projection oracle的RED证据开始。
