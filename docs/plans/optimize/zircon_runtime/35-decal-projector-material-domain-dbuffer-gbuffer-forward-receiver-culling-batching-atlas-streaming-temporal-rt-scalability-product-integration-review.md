---
related_code:
  - zircon_plugins/rendering/features/decals/runtime/src
  - zircon_plugins/rendering/features/decals/editor/src
  - zircon_plugins/rendering/features/decals/runtime/Cargo.toml
  - zircon_plugins/rendering/features/decals/editor/Cargo.toml
  - zircon_plugins/rendering/runtime/src
  - zircon_plugins/rendering/plugin.toml
  - zircon_runtime/src/core/framework/scene/component_type_descriptor
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/scene_asset
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/advanced_slot.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/graphics/material/mod.rs
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/graphics/shader/shader_assets.rs
  - zircon_runtime/src/graphics/shader/template
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred
  - zircon_runtime/src/graphics/visibility/declarations
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registration.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/material/bindless_material_eligibility.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/material/bindless_material_payload_registry.rs
tests:
  - zircon_plugins/rendering/features/decals/runtime/src/lib.rs
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/tests.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/shader/template/tests
  - zircon_runtime/src/graphics/tests/plugin_feature_compile.rs
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/28-hardware-ray-tracing-blas-tlas-ray-query-pipeline-sbt-denoising-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/30-water-ocean-lake-river-surface-wave-fft-shallow-water-rendering-underwater-buoyancy-query-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/34-vegetation-tree-foliage-grass-species-instancing-wind-animation-billboard-impostor-lod-streaming-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
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
  - dev/godot/scene/3d/decal.h
  - dev/godot/servers/rendering/renderer_rd/shaders/decal_data_inc.glsl
  - dev/bevy/crates/bevy_pbr/src/decal
  - dev/Fyrox/fyrox-impl/src/scene/decal.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/decal.shader
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 35 · Decal Projector、Material Domain、DBuffer、GBuffer、Forward、Receiver、Culling、Batching、Atlas、Streaming、Temporal、RT、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon的Decal不是“完全没有代码”，而是一个会返回成功但不产生任何像素的错误完成面。`rendering.decals` runtime插件定义`DecalProjectionMode::{ScreenSpace, Deferred}`和只有mode/opacity/normal_blend/atlas_region四字段的`DecalProjectorDescriptor`，注册一个读取depth/color、写回color的PostProcess pass，再把`decals.projector-composite`绑定到只返回`Ok(())`的`noop_render_executor`。唯一插件test只验证registration成功、component type与pass name；没有执行命令或像素oracle。

进一步纵向检查确认无法通过“替换noop函数”完成：`DecalProjectorDescriptor`没有component/reflection/serde实现或与dynamic JSON互转；`SceneEntityAsset`没有plugin component payload；`RenderFrameExtract`只有geometry/light/environment/post-process/debug/sprite/particle/visibility，没有decal snapshot；GPU execution context虽然可访问通用frame extract、deferred resources和encoder，但没有prepared decal instance/material/batch。`MaterialDomain`只有Surface/PostProcess/DebugOverlay/LightFunction，`.zmaterial`没有Decal domain，`atlas_region`也只是无人消费的String。Pipeline又把Decals插在baked lighting/reflection probes/bloom之后，不能承载pre-lighting DBuffer/GBuffer语义，`normal_blend`在现有attachment合同下不可实现。

Editor39已把空executor、静态Scene断点、Material UI false option和mock authority登记为5项P0，并以P1-50..60建立Decal umbrella要求；Plugins04已拥有metadata-only feature/capability与产品装配真相。本篇不重复这些父finding，登记 **0个新P0 / 72个runtime子P1 / 16个P2**。Runtime35成为Decal可执行runtime分解的唯一owner：`DecalMaterialSource -> qualified DecalMaterialArtifact(domain/channels/blend/stages/variants/dependencies) + DecalProjectorSource -> generation-qualified World Instance -> DecalRenderSnapshot -> per-view visibility/stage work -> DBuffer/GBuffer/Forward/Transparent/Mesh/RT adapters -> typed execution/quality receipt`。Editor39的P1-50..60继续作为父要求，关闭Runtime35子门后才能关闭父项，不能双算为两份实现完成。

## 2. 审查边界、方法与 currentness

### 2.1 冻结输入

冻结语料为286个文件、64,535行、2,492,770 bytes：193个Zircon production文件为29,236行、1,085,634 bytes；24个focused test文件为8,109行、314,912 bytes；4个产品/控制面证据文件为1,430行、61,866 bytes；65个参考文件为25,760行、1,030,358 bytes。指纹算法为按forward-slash相对路径排序，逐文件计算小写SHA-256，形成`path<TAB>file_sha256`行，以单个LF连接且无末尾LF，再对UTF-8 payload计算SHA-256；结果为`2c472f87c0cc2648b6952eac4790500d1ab8dc9254127dba39d84a17c0bb7b35`。

冻结基线为`main@25e09a23178000f2e783ce2143cf70a8b118d404`，按读取时working bytes计算。`frame_extract.rs`、`frame_extract/geometry.rs`和`scene_extract.rs`在Git状态中显示modified，但working blob与HEAD blob逐项相同；Editor39计划源当前为untracked working文件。本篇没有修改或归因这些production/plan输入。实施前必须重导286项manifest、重算指纹并复核在途状态。

### 2.2 纵向检查链

本轮逐层检查material source/schema/domain -> compiler/artifact/variant -> projector source/property schema -> Scene/DynamicScene persistence -> stable runtime instance/lifecycle -> frame extract/snapshot -> projector transform/projection/depth reconstruction/clipping -> bounds/visibility/receiver filtering -> sorting/grouping/batching -> DBuffer/GBuffer/forward/transparent/mesh stages -> channel/write-mask/blend/normal -> texture atlas/bindless/residency -> multi-view/MSAA/DRS/temporal -> shadow/GI/RT/virtual geometry -> diagnostics/scalability/tests/product evidence。

24个focused test文件共有173个`#[test]`/`#[ignore]`/`#[tokio::test]`属性，覆盖generic frame extract、pipeline compile、deferred lighting、execution context、shader templates与DynamicScene transaction；另外Decal runtime `lib.rs`内只有一个registration test。测试中`BuiltinRenderFeature::Decal`只被当作descriptor/quality-gate枚举检查，没有projector roundtrip、material compile、extract、projection math、receiver mask、DBuffer blend、command encoding、pixel golden、multi-view、RT或规模测试。

### 2.3 搜索与动态证据边界

精确搜索确认`DecalProjectorDescriptor`、component type和executor ID在Decal package外没有production consumer；Runtime Scene、render extract、visibility与scene renderer没有Decal/Projector数据路径；examples/templates中独立单词`decal`为零命中。WOC的`affixCascadeCalls`只是包含字符序列的噪声，不是Decal产品证据。

本轮是E3 source-level review，没有运行Cargo、WGPU、GPU capture或产品窗口。空executor与零输入链可由源码直接证明；但未来DBuffer精度、像素结果和性能必须由实现后的GPU测试/capture证明。本篇不把普通deferred、material或render graph测试当作Decal通过，也不把“未运行”写成成功。

## 3. 当前可保留的真实基础

1. Runtime plugin可事务注册component descriptor、render feature和executor，可保留为可选Decal provider入口；registration必须从Declared提升到真实Executed/Qualified。
2. World dynamic component registry有type/plugin前缀、schema generation、sparse presence与inspection invalidation，DynamicScene有versioned reflection payload和transactional spawn基础。
3. Render feature/pass descriptor、compiled graph、resource resolver、attachment ops和GPU execution context能承载typed pass资源与命令编码；现有Decal descriptor没有正确使用这些能力。
4. Deferred renderer已有GBuffer albedo/material/emissive资源、lighting pass与shader template，可作为DBuffer/GBuffer composition consumer；Decal必须定义写入时序与编码兼容。
5. visibility、GPU Scene、indirect work和per-view extract已有通用owner方向，可复用bounds/culling primitives；Decal仍需要projector volume、receiver和stage-specific work。
6. Material资产已有texture references、property overrides、readiness/fallback与shader variant key，可扩展为Decal artifact；不能继续用`atlas_region: String`替代资源身份。
7. ResourceStreamer、bindless material payload与texture capability基础可用于atlas/bindless策略；Decal residency、pressure与fallback尚未接入。
8. Runtime09A/09B/09C/09D/09H1和Runtime28已拥有graph、visibility、material、residency、temporal与RT通用合同，本篇只提供Decal domain数据和adapter。

## 4. 当前代码事实与断路

### 4.1 Source、Scene、Instance 与 Extract

1. `DecalProjectorDescriptor`没有material reference、transform/size/pivot、UV、sort、fade、receiver mask、lifetime、color、angle fade或stable identity。
2. component property只有name、字符串value_type和editable，缺stable property ID、default、range、unit、asset kind、validator、schema version和migration。
3. Rust descriptor与registered JSON schema是两份未连接类型；默认值只存在Rust构造，World dynamic component不会由它创建typed instance。
4. `SceneEntityAsset`没有plugin component集合；手工World JSON理论上可进入DynamicScene，不等于项目Scene能author/save/reopen/export。
5. plugin没有runtime system读取dynamic component、建立instance storage、监听transform/property/asset变化或处理unload。
6. `RenderFrameExtract`没有decal数组，`VisibilityInput`只承载mesh renderable，executor无法得到projector volume/material/flags。
7. `BuiltinRenderFeature::Decal`属于descriptor-only advanced slot，produces零pass；可选plugin pass则另建第二个名字相近authority。
8. examples/templates没有可运行Decal场景、材质、projector或accepted image，当前产品路径没有实际caller。

### 4.2 Pass、Material、Projection 与 Capability

1. `noop_render_executor`不调用`require_gpu()`、不读frame extract、不解析texture view、不创建pipeline/bind group、不编码draw/dispatch。
2. pass被声明为PostProcess，只读scene-depth/scene-color并原位写scene-color；没有DBuffer/GBuffer normal/ORM/emissive attachment或channel mask。
3. Deferred与ScreenSpace mode没有runtime分支，opacity/normal_blend/atlas_region四字段全部无人读取。
4. Decals排序位于bloom之后、post_process之前，既不是pre-base-pass DBuffer也不是正确deferred-lighting前GBuffer阶段。
5. `MaterialDomain`没有Decal，ShaderProgram/Graph/MaterialGraph/VariantKey无法表达Decal domain；Workbench下拉的`decal`是孤立字符串。
6. `.zmaterial`和Standard PBR projection没有decal channel/write mask、blend policy、receiver response或stage compatibility。
7. 没有depth到world/local projector空间重建、box clipping、inside/outside cull、reverse-Z、camera-relative origin、oblique/stereo处理。
8. 插件manifest disabled-by-default但metadata/capability仍可进入控制面；package registration成功与像素执行之间没有资格receipt。

## 5. 参考实现给出的工程边界

### 5.1 Unreal：材质决定stage/target/blend，proxy决定可见实例

`UDecalComponent`持有material、sort order、screen-size fade、time fade、size、color与scene proxy；`FDeferredDecalProxy`缓存size-inclusive transform/bounds、可见性和fade参数，并有add/update/remove operation。Renderer以`FDecalBlendDesc`把blend mode、render-stage mask与DBuffer write mask压成cook/runtime合同，区分DBuffer、GBuffer、mobile、emissive与AO stage、target mode、write mask、blend/raster state和shader environment。Zircon应吸收material artifact驱动stage资格、generation-qualified proxy和按view work，不复制UObject/RDG具体类型。

### 5.2 Unreal visibility/rendering：volume不是fullscreen贴图

`FVisibleDecal`保存material proxy、sort、blend/fade/color，visibility task按view生成visible/relevant lists并按stage消费；deferred pass处理frustum/size fade、inside volume、reverse culling、stencil、target layout和排序，另有mesh decal、mobile与ray tracing路径。关键边界是projector bounds先裁剪、volume raster只影响被覆盖像素、receiver/material stage一致；Zircon当前单PostProcess token没有这些语义。

### 5.3 Unity HDRP/URP：instance manager、chunk、atlas与technique matrix

HDRP `DecalProjector`暴露material、size/pivot、draw distance/fade、angle fade、UV、layer和scale mode；`DecalSystem`按material/draw order组织set、cull、batch、atlas与GPU data，jobs并行更新。URP提供DBuffer depth copy/clear/render/emissive、screen-space与GBuffer technique，entity manager/chunk/update-cull/draw-call systems分离生命周期和per-view work。Zircon需要显式Technique/SurfaceData capability matrix，不能用一个mode enum和同一executor隐式降级。

### 5.4 Godot、Fyrox、Bevy：较小实现仍有完整数据路径

Godot Decal node把textures、size、modulate、normal fade、distance fade、upper/lower fade、cull mask送到RenderingServer RID并更新AABB；renderer storage/scene cull消费它。Fyrox Decal是可Reflect/Visit的Scene node，持diffuse/normal texture、color、layer与bounds，shader从depth重建投影并写color/normal。它们证明最低产品线也需要持久化实例、资源引用、bounds和真实shader。

Bevy同时提供forward quad decal和clustered forward volume decal；后者extract visible component、准备storage buffer/texture binding array、进入global clusterable object meta，并显式声明bindless/Metal/Web限制和最大texture数量。它不是完整Editor/DBuffer参考，但很好地说明capability失败必须可见，且projector data必须从World贯通extract/prepare/bind/shader。

## 6. 目标架构与唯一 Owner

```text
DecalMaterialSource(domain/channels/textures/blend/receiver/stages)
  -> deterministic DecalMaterialCompiler
  -> DecalMaterialArtifact + Variant/PSO/Dependency/Capability Receipt

DecalProjectorSource(material/transform/size/pivot/UV/sort/fade/mask/lifetime)
  -> Scene/DynamicScene persistence + migration
  -> generation-qualified World DecalInstance/Proxy
  -> DecalRenderSnapshot(bounds/material/flags/current-previous state)
  -> per-view cull + receiver/stage classification + sort/group/batch
  -> DBuffer/GBuffer/Forward/Transparent/Mesh/RT adapters
  -> execution, fallback, overflow, quality and terminal receipts
```

| 领域 | 唯一 owner | Runtime35只消费/提供 |
|---|---|---|
| Decal产品真实性与authoring | Editor39 | P0-01/03/04/05和P1-50..60父要求、Inspector/handles/transaction/preview |
| Package/catalog/capability | Plugins04 | optional feature、source/dist/provider装配与Declared->Qualified truth |
| Decal executable runtime | 新Runtime35 owner | material artifact、projector instance、extract、projection、stage work、receiver、runtime receipts |
| Resource/schema/DDC | Runtime04 | source/artifact identity、dependency、migration、LKG/install/retire |
| Scene/world lifecycle | Runtime05 | plugin component persistence bridge、world generation、transactional spawn/unload |
| Render graph/RHI | Runtime09A | attachment、hazard、load/store、pipeline/resource lifetime与device recovery |
| Visibility/GPU Scene | Runtime09B | generic bounds/culling/indirect primitives；本篇提供projector volume/stage work |
| Material/shader/PSO | Runtime09C + Editor15 | domain/compiler/variant/PSO与graph authoring primitive；本篇定义Decal specialization |
| Texture residency | Runtime09D | atlas/bindless/streaming admission、pressure、fallback与retirement |
| Lighting/GI/temporal/RT | Runtime09E + 09F3 + 09H1 + Runtime28 | 消费同代Decal stage/output/history/fallback，不拥有projector truth |
| Terrain/Water/Vegetation receivers | Runtime29 + Runtime30 + Runtime34 | typed receiver adapter与policy，不复制Decal instance/material |
| Space/identity | Runtime23 + Runtime24 | camera-relative transform、origin generation、stable/live handle primitive |

Editor39的P1-50..60是父requirement，不与本篇逐项一一关闭；Runtime35子finding提供可执行分解和验收。实施manifest应建立parent-child edge，只有相关子finding与Editor authoring门同时关闭，父项才可完成。禁止把父项和子项分别实现两套schema或executor。

## 7. P1：Material Source、Projector Schema 与 Compiler

| ID | 差距 | 必须重构 |
|---|---|---|
| DEC-P1-001 | 无Decal MaterialDomain | 在唯一MaterialDomain/schema中增加Decal并迁移Workbench、asset、graph、variant与pass所有reader |
| DEC-P1-002 | `.zmaterial`无domain | source持久化domain、schema version、unknown policy与migration，禁止UI字符串旁路 |
| DEC-P1-003 | 无Decal channel schema | base color、normal、roughness/metallic/AO、emissive与opacity有typed affect/write mask |
| DEC-P1-004 | 无blend policy artifact | translucent/stain/normal/DBuffer等语义编译为stage、target、write mask和blend state，不运行时猜测 |
| DEC-P1-005 | 无receiver response合同 | surface material声明receive/response channel，compile拒绝不兼容decal/receiver组合 |
| DEC-P1-006 | 无Decal shader graph output | 定义domain-specific inputs/outputs、derivative/mip/opacity/lifetime及unsupported node诊断 |
| DEC-P1-007 | 无variant/PSO identity | key包含domain、technique、stage、target layout、channels、MSAA、view/platform和shader generation |
| DEC-P1-008 | 无material dependency artifact | texture/sampler/atlas/bindless、fallback、residency、license/provenance进入BuildSet和receipt |
| DEC-P1-009 | projector descriptor非typed schema | material asset、size/pivot、UV、sort、fade、mask、lifetime、color与mobility使用stable property IDs |
| DEC-P1-010 | property缺约束/migration | default/range/unit/asset-kind/finite/validator/schema version与migration纳入component contract |
| DEC-P1-011 | Rust default与JSON schema分裂 | source/component/default/reflection/serde使用同一generated或shared typed definition |
| DEC-P1-012 | 无deterministic compiler/DDC/LKG | 相同source/dependency/toolchain/target产生稳定artifact digest、diagnostic与last-good publication |

## 8. P1：Scene、Runtime Instance、Lifecycle 与 Extract

| ID | 差距 | 必须重构 |
|---|---|---|
| DEC-P1-013 | static Scene无plugin payload桥 | Runtime05建立通用versioned plugin component persistence，Decal不得追加临时Option字段 |
| DEC-P1-014 | 无transactional spawn | preflight type/material/schema/dependency后原子创建entity/component/instance，失败零可见变化 |
| DEC-P1-015 | 无stable projector identity | persistent ID与world slot/generation分离，save/reopen/reload/undo/network可追踪，stale访问拒绝 |
| DEC-P1-016 | 无runtime instance/proxy | 建world/entity/component/material/device generations、transform/bounds/fade/flags与resource leases |
| DEC-P1-017 | 无instance lifecycle | Requested/Preparing/Active/Suspended/Retiring/Failed/Cancelled每ticket唯一终态 |
| DEC-P1-018 | 无change propagation | transform/property/material/asset/visibility/layer变化产生dirty range和同代proxy更新 |
| DEC-P1-019 | 无batch add/update/remove | mutation按owner/generation事务发布，slot reuse、compaction和sort重建不破坏identity |
| DEC-P1-020 | 无reload/migration/LKG | material/projector schema变化定义state保留、rebind、fallback、LKG和不可迁移原因 |
| DEC-P1-021 | 无multi-world isolation | Editor preview、PIE、game与多world不共享mutable instance/fade/atlas truth |
| DEC-P1-022 | 无unload/device-loss drain | 停止extract/work后按fence退instance/material/atlas/pipeline，旧async完成不能复活 |
| DEC-P1-023 | RenderFrameExtract无Decal snapshot | 新增generation-qualified SoA snapshot：transform/inverse/bounds/material/flags/fade/current-previous |
| DEC-P1-024 | 无extract一致性 | Scene/world generation、visibility、material readiness和resource residency在同一publish point冻结 |

## 9. P1：Projection、Visibility、Receiver、Sorting 与 Batching

| ID | 差距 | 必须重构 |
|---|---|---|
| DEC-P1-025 | 无projector transform合同 | local box、size/pivot、component/world/view/clip和camera-relative space及nonuniform scale明确 |
| DEC-P1-026 | 无depth reconstruction | reverse-Z、perspective/ortho、jittered/non-jittered matrix、viewport rect与DRS坐标通过oracle |
| DEC-P1-027 | 无volume clipping | world position投影到local box并处理epsilon、near-plane、inside-camera和degenerate size |
| DEC-P1-028 | 无face/angle rejection | projection direction、surface normal阈值、upper/lower/angle fade及normal source资格化 |
| DEC-P1-029 | 无真实bounds | size/pivot/transform生成OBB/AABB/sphere，origin rebase和动态transform同代更新 |
| DEC-P1-030 | 无per-view frustum/size/distance cull | screen-size/draw-distance/fade、layer/view family/stereo/shadow/scene capture分别计算 |
| DEC-P1-031 | 无receiver filter | render layer、material response、object category、terrain/water/vegetation/transparent策略类型化 |
| DEC-P1-032 | 无overlap precedence | sort order、material/state、distance与stable ID tie-break确定，禁止unstable iteration决定像素 |
| DEC-P1-033 | 无stage classification | material artifact将每instance分到DBuffer/GBuffer/emissive/AO/forward/mobile/RT批准stage |
| DEC-P1-034 | 无material grouping/batching | 按artifact/variant/target/blend/sampler/atlas组织work，sort自由度与state reduction显式权衡 |
| DEC-P1-035 | 无GPU cull/indirect路径 | large-count volume cull、visible compaction、draw args与CPU oracle/overflow receipt闭环 |
| DEC-P1-036 | 无view/work budget | total/visible/culled/dropped、pixels/volumes/batches/bytes按view限额并给出批准degrade |

## 10. P1：DBuffer、GBuffer、Forward、Shader 与 Pass Correctness

| ID | 差距 | 必须重构 |
|---|---|---|
| DEC-P1-037 | pass stage位置错误 | DBuffer在base/lighting前，GBuffer在lighting前，emissive/AO/forward按依赖插入而非统一PostProcess |
| DEC-P1-038 | 无typed attachment layouts | DBuffer/GBuffer各MRT format、clear/load/store、sample count、lifetime与readers由graph声明 |
| DEC-P1-039 | 无channel write mask | pipeline/color targets按artifact只写批准channel，未影响通道保持receiver原值 |
| DEC-P1-040 | 无blend-state matrix | base/normal/ORM/emissive/AO分别定义预乘/非预乘、alpha与target blend，跨backend验证 |
| DEC-P1-041 | 无normal encode/blend parity | world/tangent normal、oct/packed encoding、RNM/lerp policy和zero-strength在receiver编码域一致 |
| DEC-P1-042 | 无DBuffer decode/apply | base material/lighting显式采样DBuffer、应用response并保留static/baked lighting语义 |
| DEC-P1-043 | 无projector volume pipeline | cube geometry/fullscreen/clustered technique各有qualified vertex/fragment、cull/stencil/depth state |
| DEC-P1-044 | 无inside/outside raster policy | camera进入volume、mirrored transform、reverse culling和near-plane crossing不漏画/全屏爆画 |
| DEC-P1-045 | 无stencil/overdraw策略 | volume stencil/prepass/cluster选择受overlap和GPU成本驱动，不能每decal无界fullscreen |
| DEC-P1-046 | 无mip/derivative policy | projector UV、screen-space derivative、mip bias、anisotropy和atlas padding避免shimmer/bleeding |
| DEC-P1-047 | 无forward/transparent路径 | clustered forward、transparent receiver和mesh decal要么真实支持，要么compile/activation显式unsupported |
| DEC-P1-048 | noop executor仍是成功 | 真executor要求prepared input/GPU/resources并编码可观测work；零输入与unsupported使用typed disposition |

## 11. P1：Temporal、Multi-View、Streaming、RT 与 Cross-System Integration

| ID | 差距 | 必须重构 |
|---|---|---|
| DEC-P1-049 | 无current/previous projector状态 | moving projector/receiver、spawn/remove、teleport和origin rebase定义transform/history generation |
| DEC-P1-050 | 无TAA/reactive策略 | opacity/fade/material animation/LOD/atlas replacement产生正确reactive/disocclusion或history reset |
| DEC-P1-051 | 无jitter/DRS一致性 | depth reconstruction、projector clip和UV在render/display尺寸、viewport subrect与jitter域一致 |
| DEC-P1-052 | 无stereo/multi-view parity | 每eye matrix/cull、single-pass layout、scene capture/reflection/portal/overlay view有明确资格 |
| DEC-P1-053 | 无MSAA/sample策略 | depth resolve/per-sample edge、DBuffer sample count、A2C与forward fallback有像素门 |
| DEC-P1-054 | 无atlas allocator | region identity、padding/mips、format/channel classes、generation、fragmentation、compaction和lease缺失 |
| DEC-P1-055 | 无bindless capability/fallback | binding array limit、partial binding、Metal/Web/mobile限制在admission时选择atlas/array/unsupported |
| DEC-P1-056 | 无streaming/residency | visible importance、mip request、prefetch、pressure eviction、fallback和in-flight retirement有receipt |
| DEC-P1-057 | 无mesh/virtual-geometry receiver | skinned/deformed/Nanite-like/meshlet geometry的depth/normal/material response与LOD切换明确 |
| DEC-P1-058 | 无terrain/water/vegetation adapter | 各域只提供receiver data/policy；Decal owner维护material/instance/stage和stable hit identity |
| DEC-P1-059 | 无GI/lightmap/shadow语义 | DBuffer与baked/dynamic lighting、emissive GI、shadowed surface和capture update边界明确 |
| DEC-P1-060 | 无RT/path tracing路径 | projected/mesh decal在BLAS/TLAS/material hit或raster fallback中的alpha/channel/sort/parity受Runtime28治理 |

## 12. P1：Scalability、Diagnostics、Tests 与 Product Qualification

| ID | 差距 | 必须重构 |
|---|---|---|
| DEC-P1-061 | 无联合scalability | max decals/view、draw distance、channels、technique、atlas、RT和transparent由同一quality/budget决定 |
| DEC-P1-062 | 无global admission | instances/visible/overlap/pixels/batches/textures/bytes/CPU/GPU time决定接受、降级或拒绝 |
| DEC-P1-063 | 无platform matrix | deferred/forward/mobile、MRT/blend/MSAA/bindless/RT能力映射Supported/Degraded/Unsupported |
| DEC-P1-064 | 无runtime diagnostics | 展示instance/material/stage/cull reason/sort/batch/atlas/residency/fallback/CPU/GPU timing |
| DEC-P1-065 | 无schema/compiler tests | domain、channel/blend/stage、property validation、migration、artifact digest与bad input golden为空 |
| DEC-P1-066 | 无Scene/lifecycle tests | save/reopen/spawn/reload/unload/multi-world/stale generation/async cancel/device loss为空 |
| DEC-P1-067 | 无projection math tests | perspective/ortho/reverse-Z/jitter/DRS/stereo/inside volume/nonuniform/mirror/origin rebase为空 |
| DEC-P1-068 | 无GPU pixel golden | albedo/normal/ORM/emissive、overlap sort、fade、mip、DBuffer apply、forward/MSAA/temporal为空 |
| DEC-P1-069 | 无failure/fuzz矩阵 | malformed material/property、missing texture、atlas OOM、overflow、shader fail、device loss/plugin unload为空 |
| DEC-P1-070 | 无真实产品场景 | 建bullet marks/blood/rain/wetness/road marking/graffiti/mesh/terrain/skinned多场景save-play-export-capture链 |
| DEC-P1-071 | 无规模基准 | 1/100/10K decals、深重叠、fast camera、stream churn记录CPU/GPU/memory/overdraw/draw/batch/stutter |
| DEC-P1-072 | 无跨引擎超越门 | 同资产/视角/receiver/画质/硬件对照Unreal/Unity并归档图像误差和raw performance receipts |

## 13. P2：完整性与长期竞争力

| ID | 延后项 | 前置条件 |
|---|---|---|
| DEC-P2-001 | spline/ribbon decal | Editor39 SpatialSpline artifact、stable segment ID、partition、UV和batch闭环完成 |
| DEC-P2-002 | arbitrary mesh decal | mesh clipping/offset、receiver attachment、skinning、LOD和RT parity完成 |
| DEC-P2-003 | persistent damage/blood accumulation | surface-space identity、save/network、budget/cleanup与material merge完成 |
| DEC-P2-004 | virtual texture decal baking | VT page ownership、transactional update、residency、undo/save和cross-platform fallback完成 |
| DEC-P2-005 | terrain layer decal baking | Runtime29 material layer/cell artifact、rebuild/LKG与runtime projector parity完成 |
| DEC-P2-006 | water current/foam decal | Runtime30 water surface UV/current/simulation artifact和qualified adapter完成 |
| DEC-P2-007 | vegetation conforming decal | Runtime34 instance/representation/wind deformation与receiver policy完成 |
| DEC-P2-008 | dynamic topology receiver | deformation history、surface correspondence、bounds和RT update完成 |
| DEC-P2-009 | decal lighting/shadow casting | physically defined thickness/opacity/light transport、reference oracle和cost gate完成 |
| DEC-P2-010 | path-traced decal material | Runtime28 pipeline/SBT/material callable、sort/alpha和raster parity完成 |
| DEC-P2-011 | decal compute raster/tiling | CPU/raster oracle、tile overflow、ordering、MSAA和portable fallback完成 |
| DEC-P2-012 | GPU-driven persistent cluster | stable identity、mutation log、GPU compaction、readback/fault和multi-world isolation完成 |
| DEC-P2-013 | authoritative network decals | persistent ID、interest/late join/rollback/save、asset entitlement和bandwidth receipt完成 |
| DEC-P2-014 | third-party decal provider SDK | ABI/version/capability/resource budget/sandbox/unload与artifact compatibility完成 |
| DEC-P2-015 | collaborative decal authoring | stable IDs、transaction/merge/locking/recovery和source provenance完成 |
| DEC-P2-016 | distributed visual qualification farm | frozen BuildSet、GPU/driver matrix、capture/image diff/perf raw receipt与promotion完成 |

## 14. 分层重构里程碑

### M0 · Truth、Parent Owner 与 Baseline

冻结Editor39 P0/P1父边、Plugins04装配边、286项语料和跨引擎场景。空executor保持unsupported或hard-disabled；catalog不得从registration/quality-gate枚举推导Ready。建立DecalCapabilitySnapshot与parent-child finding manifest。

### M1 · Material/Projector Schema 与 Compiler

完成DEC-P1-001..012：唯一Decal MaterialDomain、`.zmaterial`/graph/channel/blend/receiver/stage schema、typed projector properties、deterministic artifact/variant/DDC/LKG。Editor15/39只能消费shared compiler。

### M2 · Scene、Instance、Lifecycle 与 Extract

完成DEC-P1-013..024：通用plugin component persistence、transactional spawn、stable instance/proxy、batch mutation、multi-world/unload和`DecalRenderSnapshot`。先用CPU inspection oracle验证generation/lifecycle。

### M3 · Projection、Visibility 与 Work Compilation

完成DEC-P1-025..036：space/depth/clip/angle、bounds/cull/receiver/sort、stage classification、batch与GPU work。CPU oracle覆盖每view visible/stage/order，GPU overflow必须可观察。

### M4 · DBuffer/GBuffer 与真实Executor

完成DEC-P1-037..048：修正graph阶段与attachments，实现channel/blend/normal/DBuffer apply、volume/stencil/mip、forward/transparent disposition，并删除成功型noop。首次GPU pixel golden在此成为required。

### M5 · Temporal、Multi-View 与 Residency

完成DEC-P1-049..056：current/previous、TAA/reactive、jitter/DRS、stereo/MSAA、atlas/bindless/streaming。所有view、quality与platform fallback绑定source/build/device receipt。

### M6 · Cross-System 与 RT

完成DEC-P1-057..060：mesh/virtual geometry、terrain/water/vegetation、GI/lightmap/shadow和RT/path tracing adapter。consumer只读取同代Decal snapshot/artifact，不反向拥有实例。

### M7 · Editor/Product Integration

与Editor39关闭Inspector/projector handles、transaction/save/reopen/preview；建立至少六种真实Decal product fixture、Play/export/capture与frame diagnostics。只有Runtime子门和Editor父门同时完成才关闭P1-50..60。

### M8 · Reliability 与 Scalability

完成DEC-P1-061..071：联合预算、platform matrix、diagnostics、schema/lifecycle/math/pixel/fuzz和10K decal压力场景；覆盖OOM、overflow、shader fail、device loss与plugin unload。

### M9 · 性能与表现超越门

完成DEC-P1-072：以相同projector/material/receiver/overlap/view/hardware/quality对照Unreal/Unity，归档CPU/GPU timestamps、memory/atlas、overdraw、draw/batch、stutter与image error。没有raw可复跑receipt不得声称优于虚幻。

## 15. 验收门

| Gate | 必须证明 |
|---|---|
| DEC-G01 | 空executor在真实闭环前报告Unsupported/Disabled，不再返回可解释为成功的结果 |
| DEC-G02 | Decal capability只能由同代provider、artifact、instance、execution与qualification receipt升级 |
| DEC-G03 | `.zmaterial`、shader graph、variant、pass与Editor只有一个Decal domain authority |
| DEC-G04 | channel/blend/stage artifact重复build digest稳定且不兼容组合fail-close |
| DEC-G05 | projector schema有stable property ID/default/range/unit/asset kind/version/migration |
| DEC-G06 | project Scene save/reopen/play/export保留plugin Decal payload、stable ID和material reference |
| DEC-G07 | spawn/reload/unload/cancel/device loss后旧instance/task/resource generation不能发布 |
| DEC-G08 | multi-world/PIE/preview的fade、sort、atlas和instance state隔离 |
| DEC-G09 | frame extract携同代projector inverse/bounds/material/flags/current-previous state |
| DEC-G10 | reverse-Z perspective/ortho depth reconstruction与CPU oracle在误差内一致 |
| DEC-G11 | jitter/DRS/viewport subrect/stereo下local projection和UV没有坐标漂移 |
| DEC-G12 | inside-camera、near-plane、mirror/nonuniform/degenerate volume不漏画或全屏爆画 |
| DEC-G13 | frustum/screen-size/distance/angle/receiver cull reason可解释且与oracle一致 |
| DEC-G14 | overlap排序有stable tie-break，重启/线程调度不改变结果 |
| DEC-G15 | stage classification与material artifact一致，错误stage不能静默执行 |
| DEC-G16 | visible compaction/indirect overflow有fail-close或批准degrade receipt |
| DEC-G17 | DBuffer/GBuffer attachments、load/store/sample/lifetime通过graph validation |
| DEC-G18 | 未影响channel保持receiver原值，write mask与blend state逐target正确 |
| DEC-G19 | normal encode/blend在零/半/全强度与不同receiver normal下通过pixel oracle |
| DEC-G20 | DBuffer apply发生在lighting正确阶段并保持baked/dynamic lighting语义 |
| DEC-G21 | volume cull/stencil/depth state在inside/outside/reverse culling场景正确 |
| DEC-G22 | mip/derivative/atlas padding在斜视、缩小和边缘没有批准阈值外bleeding/shimmer |
| DEC-G23 | forward/transparent/mobile不支持时compile/activation明确失败，不静默丢channel |
| DEC-G24 | 真executor要求prepared input与GPU context并编码非零可观测work |
| DEC-G25 | spawn/remove/move/fade/material animation产生正确TAA reactive/history disposition |
| DEC-G26 | multi-view/scene capture/reflection/portal只消费对应view generation |
| DEC-G27 | MSAA depth/edge/DBuffer sample策略通过像素golden和backend matrix |
| DEC-G28 | atlas region generation、compaction和eviction不会让in-flight draw采样错误内容 |
| DEC-G29 | bindless限制在admission时选择qualified atlas/array/unsupported路径 |
| DEC-G30 | streaming pressure下fallback可见且关键persistent decal不静默消失 |
| DEC-G31 | terrain/water/vegetation/mesh/virtual geometry receiver policy由typed adapter提供 |
| DEC-G32 | GI/lightmap/shadow/RT消费同代artifact/instance并报告不支持channel/fallback |
| DEC-G33 | runtime diagnostics报告total/visible/culled/dropped/batches/atlas/CPU/GPU/fallback |
| DEC-G34 | malformed schema/material、missing texture、atlas OOM、shader fail、device loss矩阵通过 |
| DEC-G35 | registration/compile/unit测试不再作为pixel或产品资格替代品 |
| DEC-G36 | 六类产品fixture均通过save/reopen/play/export/capture与frame inspection |
| DEC-G37 | 1/100/10K及深重叠场景有CPU/GPU/memory/overdraw/draw/batch/stutter raw receipts |
| DEC-G38 | Editor39 P1-50..60只有在对应Runtime35子门和authoring门均关闭后才完成 |
| DEC-G39 | changed source/build/device/driver使旧accepted image/perf receipt自动过期 |
| DEC-G40 | 超越Unreal/Unity的结论绑定同资产/receiver/视角/硬件/画质和可复跑raw receipt |

## 16. Finding 到里程碑映射

| Finding | 里程碑 |
|---|---|
| DEC-P1-001..012 | M0-M1 |
| DEC-P1-013..024 | M2 |
| DEC-P1-025..036 | M3 |
| DEC-P1-037..048 | M4 |
| DEC-P1-049..060 | M5-M6 |
| DEC-P1-061..072 | M7-M9 |
| DEC-P2-001..016 | 对应P1与验收门完成后独立立项，不得提前并入MVP |

## 17. 禁止的临时修补

1. 禁止只删除`noop`名字、仍返回`Ok(())`或编码空pass便宣称Decal已实现。
2. 禁止从component/pass registration、quality-gate enum、manifest capability或disabled-by-default推导Ready。
3. 禁止把Decal继续实现为bloom之后的通用scene-color PostProcess来冒充DBuffer/GBuffer。
4. 禁止在Editor保留`decal`字符串而runtime/schema/compiler继续没有MaterialDomain。
5. 禁止用`atlas_region: String`、文件路径或slot index替代typed asset/generation/lease。
6. 禁止为Decal向`SceneEntityAsset`再加临时Option字段，绕过通用plugin payload持久化桥。
7. 禁止executor每帧扫描World/dynamic JSON或自行解析Editor数据；必须消费冻结extract/prepared work。
8. 禁止无bounds/fullscreen逐Decal绘制、永久放大bounds或只靠draw distance掩盖culling缺失。
9. 禁止把normal/ORM/emissive静默丢弃并仍报告Deferred mode成功。
10. 禁止用不稳定容器迭代决定overlap顺序，或为每个sort order牺牲无限state batching而无预算。
11. 禁止只实现main deferred view而忽略forward/mobile/stereo/MSAA/scene capture/RT disposition。
12. 禁止以单张截图、测试shader、固定mock、0退出码或平均变色替代channel/pixel/performance raw receipts。

## 18. 实施前重查清单

1. 重导286项冻结manifest并重算composite SHA-256，记录working/HEAD、新增、删除和依赖变化。
2. 重查Editor39 P0-01/03/04/05及P1-50..60、Plugins04 capability P0，建立parent-child而非重复owner。
3. 重查Decal package executor、component descriptor、MaterialDomain、Scene plugin payload和RenderFrameExtract是否已变化。
4. 重查pipeline placement、deferred attachment layout、shader templates、visibility/GPU Scene和texture residency当前owner。
5. 重查examples/templates是否出现真实Decal caller、资产、accepted image或performance receipt。
6. 重查Unreal/Unity/Godot/Bevy/Fyrox本地reference snapshot identity与引用行，不能从路径存在推导currentness。
7. 锁定M0-M9每阶段BuildSet、backend、GPU/driver、resolution/MSAA/DRS、view、warm-up与raw evidence位置。
8. 保持review-only边界；实施前先关闭P0 false success面，再按schema->instance->extract->render顺序推进。

## 19. 本轮产出边界

本篇只完成静态审查、参考对照、跨报告owner拆分、runtime子finding、里程碑和验收门，没有修改production代码、Cargo、manifest、tests、workflow或产品资产，没有运行构建、测试、GPU capture或产品窗口，也没有证明任何Decal功能、性能或表现已经完成。后续必须先复核source currentness并关闭Editor39/Plugins04的false-success父问题；在DEC-G01..G40全部具有可复跑证据前，不得把Decal枚举、component descriptor、pass token、quality gate、Material UI下拉或成功registration报告宣传为工程级Decal runtime。
