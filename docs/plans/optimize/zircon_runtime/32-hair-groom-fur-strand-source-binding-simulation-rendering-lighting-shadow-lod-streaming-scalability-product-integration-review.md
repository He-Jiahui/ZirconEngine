---
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/importer/ingest
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract
  - zircon_runtime/src/core/framework/render/advanced_lighting
  - zircon_runtime/src/core/framework/render/material
  - zircon_runtime/src/graphics/feature/builtin_render_feature
  - zircon_runtime/src/graphics/material/shading_models
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/oit_buffers
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_runtime/src/graphics/shader
  - zircon_runtime/src/graphics/visibility
  - zircon_plugins/asset_importers/model/runtime/src
  - zircon_plugins/gltf_importer/runtime/src
tests:
  - zircon_runtime/src/asset/tests/assets/mesh/morph_targets.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/transparent3d.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/shading_model_parity.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs
  - zircon_runtime/src/graphics/shader/template/tests/standard_pbr_specialization.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests
  - zircon_plugins/gltf_importer/runtime/src/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/09g2-advanced-surface-lighting-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/28-hardware-ray-tracing-blas-tlas-ray-query-pipeline-sbt-denoising-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/31-cloth-fabric-soft-body-garment-simulation-collision-deformation-rendering-wind-lod-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source
  - dev/UnrealEngine/Engine/Plugins/Importers/AlembicHairImporter/Source
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Public/Groom
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/HairStrands
  - dev/UnrealEngine/Engine/Shaders/Private/HairStrands
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Hair
  - dev/bevy/crates
  - dev/godot/scene
  - dev/godot/modules
  - dev/Fyrox/fyrox-impl/src
  - dev/Fyrox/fyrox-resource/src
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 32 · Hair、Groom、Fur、Strand Source、Binding、Simulation、Rendering、Lighting、Shadow、LOD、Streaming、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前没有 Hair/Groom/Fur/Strand 运行时产品。对 Runtime、Runtime Interface、Plugins、Editor、App 与 Hub production 树执行精确标识搜索，`Hair`、`Groom`、`Fur`、`Strand`、`Alembic`、`Fiber`、`Follicle`、`Melanin`、`Marschner`、`Kajiya`、`WindField` 与 `WindSnapshot` 均为零命中；`ResourceKind`、`SceneEntityAsset`、`RenderFrameExtract`、`BuiltinRenderFeature` 和 importer capability 也没有毛发资源、绑定、实例、变形输出、渲染 feature 或产品状态。仓内少量英文动词 `strand` 只存在于 WOC fixture/comment 与资源事务测试，不是 domain identifier。

现有 Mesh、skinning、Morph、Standard PBR、generic OIT、shadow、velocity 与资源流送都是真实基础，但全部以三角网格或普通透明表面为中心。`MeshAsset` 只提供有限格式的逐顶点 attribute 与 triangle topology；`GpuMeshVertex` 是静态 surface vertex，上传 buffer 只有 `VERTEX`/`INDEX` usage；Scene mesh instance没有 groom/source/binding/guide/cache/representation 字段。OIT 默认平均每像素4层、精确排序最多8层，GPU shader固定最多32层，超出全局 capacity 的 fragment在 `atomicAdd` 后直接丢弃，并以8-bit packed color合并尾部。它不能替代 strand coverage、hair visibility node、deep transmittance或专用 composition。

本篇登记 **0 P0 / 72 P1 / 16 P2**。0 P0 不表示接近完成：当前 catalog 没有 Hair Ready 声明，Runtime09F3 已登记 hybrid GI 缺 hair classification，本篇不重复增加 truthfulness P0。新 owner 必须形成 `HairSourceAsset -> HairBuildArtifact(groups/guides/strands/cards/meshes/binding/LOD) -> HairRuntimeInstance -> Deformation/Cache Provider -> representation-specific resources -> Hair Visibility/Lighting/Shadow/RT adapters -> typed receipt`。在这条链闭合前，禁止把 anisotropic GGX、alpha blend mesh、shell/fins、particle ribbons、Morph、普通 OIT 或 cards-only demo 宣称为工程级 Hair/Groom。

## 2. 审查边界、方法与 currentness

### 2.1 冻结输入

冻结语料为148个文件、68,188行、2,735,778 bytes：63个 Zircon production 文件为12,296行、446,067 bytes；13个 focused test文件为3,934行、136,621 bytes；72个参考文件为51,958行、2,153,090 bytes。指纹算法为按 forward-slash 相对路径排序，逐文件计算小写 SHA-256，形成 `path|file_sha256` 行，以单个 LF 连接且无末尾 LF，再对 UTF-8 payload计算SHA-256；结果为 `8e13c82d4e455f2ebf6f158f6ce3707e5f63c82f94b9d166a35bf9348b016032`。

冻结基线为 `main@25e09a23178000f2e783ce2143cf70a8b118d404`，按读取时 working bytes计算。`frame_extract.rs`、`frame_extract/geometry.rs` 与 `material_features.rs` 虽被状态标记为 modified，但 working blob与HEAD blob相同；`import_gltf.rs` 与 `model_mesh_subassets.rs` 有相邻会话的纯格式变化，本篇没有修改或归因这些文件。实施前必须重导148项manifest、重算指纹并重查所有在途文件，不能把本篇结论当作未来代码的永久快照。

### 2.2 纵向检查链

本轮逐层检查 source format/import -> strand/group attribute schema -> stable identity/version -> deterministic build/DDC -> guide/render interpolation -> root-to-surface binding -> Scene persistence -> component/instance lifecycle -> animation/deformer/cache/simulation -> collision/wind -> GPU rest/deformed/current/previous resources -> bounds/cluster/LOD/culling/streaming -> strand/cards/mesh visibility -> coverage/composition/OIT -> hair BSDF/environment/multiple scattering -> deep shadow/transmittance/voxel -> velocity/TAA/motion blur -> RT/GI -> multi-view/scalability/telemetry/tests/product evidence。

13个 focused test文件共有51个 `#[test]`/`#[ignore]`/`#[tokio::test]` 属性，只覆盖 glTF triangle/vertex channel、Mesh Morph、frame extract、generic transparency、skinned velocity、standard shading model与ordinary shadow capture。没有 Hair source roundtrip、binding、guide interpolation、simulation、strand raster、coverage、hair BSDF、deep shadow、LOD streaming或产品场景测试。

### 2.3 参考搜索边界与动态验证限制

Unreal与Unity Graphics提供本域的正参考。对当前 `dev/bevy/crates`、`dev/godot/scene`、`dev/godot/modules`、`dev/Fyrox/fyrox-impl/src`、`dev/Fyrox/fyrox-resource/src` 做文件名精确搜索，没有 hair/groom/fur/strand 模块命中；这只是本地镜像范围内的负证据，不外推为这些引擎或生态绝无相关示例。

本轮是 E3 source-level review，没有运行 Cargo、WGPU、GPU capture 或产品场景。仓内不存在可执行 Hair 类型、shader family 或 scene，运行 generic mesh/OIT/shadow tests不能证明缺失产品。本篇把动态门明确留给实现里程碑，不能把“未运行”写成通过。

## 3. 当前可保留的真实基础

1. `MeshAsset` 的 typed attribute map、indices、bounds、skin与Morph可作为 cards/meshes representation和源导入桥的一部分；strand curve source必须独立建模。
2. 四权重 skinning、current/previous joint palette、Morph weight与velocity路径可作为 scalp/root binding和 cards deformation的基础；它们不表达strand root barycentric、guide interpolation或curve point state。
3. Scene resource reference、asset import outcome、dependency和subasset机制可承载 Hair source/build artifact，但必须新增明确的 asset kind、schema、migration与cook owner。
4. `RenderFrameExtract` 已区分 geometry、poses、lighting、particles与visibility，可增加 typed Hair extract，不应把所有字段塞进 `SceneMeshInstanceAsset`。
5. Renderer已有depth/GBuffer/forward/shadow/velocity、render graph、resource streamer与pipeline cache，可复用调度和资源寿命框架；Hair需要独立representation与pass合同。
6. Generic OIT已有capability fallback、buffer plan与overflow合并，可作为普通transparent cards fallback；strand主路径需要专用coverage/visibility/deep transmittance和可观测overflow。
7. Standard PBR已有anisotropic GGX、transmission、subsurface与custom shading registry，可承载新 Hair shading model的注册入口；现有参数和WGSL不是Hair BSDF。
8. Runtime22/23/24提供clock、space/unit、generation/epoch治理方向，Runtime31提供wind/collision/deformation owner依赖；Hair不得私建第二套时间、天气、刚体world或无代际GPU资源。

## 4. 当前代码事实与断路

### 4.1 Source、Import、Identity 与 Scene

1. `ResourceKind` 没有 HairSource、Groom、GroomBinding、GroomCache、HairMaterial或HairBuildArtifact。
2. `MeshAttributeValues` 只有 `Float32x2/3/4`、`Uint16x4`、`Uint32x4` 等surface vertex数组；没有curve offset/count、per-strand、per-group、knots、width或basis/curve type schema。
3. `MeshAsset::from_model_primitive` 固定生成triangle-list surface，普通custom attribute缺少strand domain、cardinality、units、version与validation。
4. glTF ingest明确拒绝非Triangles primitive；model importer只真实处理OBJ/PLY/STL/DXF或split glTF，FBX/DAE/3DS/USD family是DiagnosticOnly optional native backend。
5. 没有Alembic `.abc`、USD groom、curve cache或typed Groom payload importer，也没有source provenance、coordinate/unit/width conversion和attribute policy。
6. `SceneEntityAsset` 和 `MeshRenderer` 没有groom source、binding target、representation、LOD、simulation/cache、material group或lifecycle字段。

### 4.2 Deformation、GPU 与 Renderer

1. `AnimationPoseOutput`只发布bones/source/state，没有hair root/deformer graph、simulation reset、cache time或binding generation。
2. `GpuMeshVertex`固定保存surface position/normal/UV/joints/weights/tangent/color，不能表达curve point、radius、root/curve index、guide weights或packed hair attributes。
3. GPU mesh upload只创建`VERTEX`和`INDEX` buffer，bounds与wire segments在上传时生成；没有storage output、streaming chunk、indirect cluster、current/previous strand或fence-qualified publication。
4. `RenderFrameExtract`没有Hair实例、group/representation、deformed resource、macro group、visibility/deep shadow inputs。
5. `BuiltinRenderFeature`有Mesh/SkinnedMesh/Particle/Trail等，但没有HairStrands、HairCards、HairMeshes、HairVisibility或HairShadow。
6. depth/GBuffer/shadow/velocity pipeline全部从mesh geometry source和standard vertex layout取数，不能消费strand curve topology或guide deformation。
7. generic OIT按固定per-pixel容量分配，写入溢出直接return，resolve最多局部排序32层并压缩颜色；无coverage conservation、node visibility、tile classification、deep opacity或strand overflow receipt。
8. Standard PBR只实现通用anisotropy/transmission；没有strand direction、melanin/absorption、cuticle angle、longitudinal/radial roughness、R/TT/TRT、dual/multiple scattering或Hair IBL。

## 5. 参考实现给出的工程边界

### 5.1 Unreal HairStrands Core：source、build、binding、representation与资源必须分层

`FHairDescription`区分vertex、strand和groom identity，并以versioned bulk data保存；`HairAttributes`定义position、width、color、roughness、AO、group、guide、ID、clump、root UV、closest guides、weights、basis/curve type、knots、tool与properties。`FHairStrandsDatas`进一步区分raw/built data、points/curves、packed/transcoded格式、interpolation、root binding、cluster LOD与streaming chunk。这个边界证明Hair不能被压成Mesh自定义字符串attribute或一组材质参数。

`GroomAsset`按group记录guides/strands/cards/meshes、LOD、material、interpolation与physics，并为rest/interpolation/cluster/raytracing等资源分别持有状态。`GroomBindingAsset`把groom root绑定到skeletal mesh或geometry cache；`GroomResources`区分rest/deformed/current/previous/root/interpolation/cards/guides/strands。Zircon应吸收typed artifact、representation family、binding和generation资源边界，不复制UObject层次或Unreal默认布局。

### 5.2 Unreal Solver/Cache：guide simulation与cache playback是可替换provider

`GroomAssetPhysics`暴露solver selection、Cosserat rods/angular springs/custom路径、substeps、iterations、gravity preloading、air drag/velocity、bend/stretch与collision等约束；`GroomSolverComponent`把solver/deformer挂到Groom component而非改变source truth。`GroomCacheStreamingManager`按component注册、chunk prefetch与map/unmap管理cache，并在删除前等待in-flight read收口。Zircon需要provider admission、cancel/drain、cache generation与terminal receipt，不能让“开启simulation”隐式绑定某个线程或GPU backend。

### 5.3 Unreal Renderer：Hair是独立可见性、光照和时域产品

Renderer明确分离macro group、cluster/LOD、voxelization、deep shadow、visibility、velocity、composition、transmittance、environment与forward raster。Hair visibility包含coverage/node data、PPLL、compute/hardware raster与tile分类；cluster culling生成curve/point count和indirect work；资源与view state还显式处理stereo共享和readback。Shader目录也独立提供coverage、cluster cull、deep shadow/transmittance、guide deformation、interpolation、material、velocity、cards、ray tracing与voxel family。Zircon的generic OIT/mesh shadow无法替代这些pass语义。

### 5.4 Unity HDRP Hair：Hair光学不能退化为普通anisotropic GGX

Unity Graphics的Hair material提供strand direction、melanin/absorption、cuticle angle、longitudinal/radial roughness、Marschner cinematic/non-cinematic、R/TT/TRT lobes、preintegrated azimuthal scattering、energy-conserving longitudinal scattering、dual/multiple scattering，以及ray/path tracing适配。该包主要是光学参考，不是完整Groom source/import/simulation系统；Zircon必须分别交付Hair source/runtime与Hair BSDF，不能用完成一侧证明另一侧。

### 5.5 Bevy、Godot、Fyrox：负参考不是降级许可

当前本地镜像的目标production路径没有独立Hair/Groom模块，报告不虚构比较对象。Bevy的skinning/Morph、Godot/Fyrox的普通mesh/material仍可作为通用基础对照，但不能把中型引擎缺少该域变成Zircon标准。用户目标要求与Unreal同级并争取超越，本域必须采用独立qualification gate。

## 6. 目标架构与唯一 Owner

```text
Hair/Groom Source + Import Provenance
  -> schema migration + source validation
  -> deterministic Hair Compiler
       -> Group/Guide/Render-Strand Artifact
       -> Root Binding/Interpolation Artifact
       -> Cards/Meshes Representation Artifact
       -> Cluster/LOD/Streaming/RT Metadata
  -> generation-qualified HairRuntimeInstance
  -> Deformer / Simulation / Cache Provider
  -> current+previous representation resources and bounds
  -> Hair Visibility / Lighting / Shadow / RT adapters
  -> terminal lifecycle, degradation and execution receipts
```

| 领域 | 唯一 owner | Hair32 只消费/提供 |
|---|---|---|
| Resource/schema/artifact/DDC | Runtime04 | Hair typed source、dependency、compiler artifact、install/retire receipt |
| Scene/ECS/world lifecycle | Runtime05 | component identity、instance generation、world teardown与serialization |
| Skeleton/pose/deformer | Runtime08C | qualified pose/root surface snapshot、teleport/cut epoch |
| Rigid collision/fixed clock | Runtime08A + Runtime22 | collider/query snapshot与clock admission；Hair不私建刚体world |
| Hair domain | 新 Runtime Hair owner | source schema、compiler、binding、instance、simulation/cache、representation与diagnostics |
| Visibility/GPU Scene | Runtime09B | macro group、bounds、cluster/LOD、indirect work与visibility receipt |
| Material/shader/PSO | Runtime09C + Runtime09G2 | Hair BSDF、variants、LUT与energy qualification |
| Residency/streaming | Runtime09D | chunk/bundle admission、budget、atomic install、retire与pressure policy |
| Direct/shadow/environment/GI | Runtime09E/09F/09G + Runtime28 | deep shadow/transmittance、environment、GI/RT update/fallback |
| Temporal/history | Runtime09H1 | current/previous resource、velocity、reset/cut与reactive policy |
| Wind/weather | Editor38对应runtime owner | qualified field generation；Hair只采样aerodynamic input |
| Authoring/import/preview | Editor32后续Groom authoring owner | import settings、binding、group/LOD/material/sim edit、shared runtime compiler preview |

Hair与Cloth可以共享fixed-step admission、collision snapshot、wind field、async resource和deformer graph primitives，但source topology、constraints、optical model、representation、visibility和qualification必须分别拥有。禁止建立一个`DeformableComponent { json }`把两域的schema与失败语义混在一起。

## 7. P1：Source、Import、Schema 与 Compiler

| ID | 差距 | 必须重构 |
|---|---|---|
| HAIR-P1-001 | 无Hair/Groom资源身份 | 新增Source、BuildArtifact、Binding、Cache、Material/Profile kind与versioned handle |
| HAIR-P1-002 | 无独立source asset | 建`HairSourceAsset`，分离authoring curves与cooked runtime data |
| HAIR-P1-003 | 无stable group/strand/point ID | identity跨reimport、LOD、merge、diagnostic与cache保持稳定，禁止数组下标长期身份 |
| HAIR-P1-004 | 无typed per-point schema | position、width/radius、color、roughness、AO等含cardinality、units、range与missing policy |
| HAIR-P1-005 | 无typed per-strand/group schema | guide、clump、root UV、group/material、curve/basis/knots与custom namespace可验证 |
| HAIR-P1-006 | 无source coordinate/unit合同 | up/forward/handedness、meters、width interpretation、transform bake与precision显式化 |
| HAIR-P1-007 | 无Alembic/curve/USD Groom importer | 引入provider接口、format version、attribute map、cancel/budget与typed diagnostics |
| HAIR-P1-008 | 通用model importer不能承载Groom | Hair importer独立capability，禁止把`.abc`/USD curves路由为triangle Model |
| HAIR-P1-009 | 无source validation与repair policy | zero-point strand、NaN/Inf、negative width、bad knots、duplicate IDs、超预算输入fail-close |
| HAIR-P1-010 | 无deterministic build artifact | 编译groups/guides/strands/interpolation/cards/meshes/clusters/LOD并绑定compiler/dependency digest |
| HAIR-P1-011 | 无cook/DDC/migration/LKG | schema与artifact分别version，支持incremental rebuild、last-known-good与retirement |
| HAIR-P1-012 | 无build provenance/receipt | 记录source hash、settings、tool version、target、warnings、repairs、cost与输出digest |

## 8. P1：Binding、Scene、Instance 与 Deformation

| ID | 差距 | 必须重构 |
|---|---|---|
| HAIR-P1-013 | 无root-to-surface binding artifact | 保存目标geometry identity、triangle/root index、barycentric、rest transform与precision |
| HAIR-P1-014 | 无skeletal/geometry-cache target合同 | target kind、LOD、section、skin/cache generation、missing/reimport policy类型化 |
| HAIR-P1-015 | 无guide-to-render interpolation | closest guides、weights、local point mapping与RBF/批准算法编译为可验证artifact |
| HAIR-P1-016 | 无binding rebuild/invalidation | source、target topology、LOD或settings变化时按dependency digest重建且旧代不可发布 |
| HAIR-P1-017 | 无Hair Scene component | 持有source/artifact/binding/material groups、representation、simulation/cache与quality policy |
| HAIR-P1-018 | 无Hair runtime instance | 建world/entity/artifact generation、state、bounds、provider、resource set与terminal receipt |
| HAIR-P1-019 | 无严格生命周期 | Requested/Preparing/Active/Suspended/Retiring/Failed/Cancelled每ticket唯一终态 |
| HAIR-P1-020 | 无per-group override | material、width/density、LOD、visibility、shadow、simulation按stable group ID覆盖 |
| HAIR-P1-021 | 无animation/deformer phase合同 | 明确pose -> root skin -> guide/cache/sim -> render interpolation -> extract顺序 |
| HAIR-P1-022 | 无teleport/reset/cut语义 | preserve/reset position/velocity、cache seek、origin shift和camera cut分别带epoch |
| HAIR-P1-023 | 无async ownership与stale rejection | task携world/instance/artifact/target/tick generation，cancel/panic后不发布 |
| HAIR-P1-024 | 无two-world/multi-instance isolation | 相同asset在PIE、preview、多个world/component中不得共享mutable deformation truth |

## 9. P1：Guide Simulation、Collision、Wind 与 Cache

| ID | 差距 | 必须重构 |
|---|---|---|
| HAIR-P1-025 | 无simulation/deformer provider合同 | 定义CPU/GPU能力、solver model、limits、determinism、async、memory与fallback admission |
| HAIR-P1-026 | 无deterministic CPU oracle | 建小规模guide baseline用于compiler、cache、GPU differential与headless qualification |
| HAIR-P1-027 | 无guide dynamic state | current/previous position、velocity、orientation、rest length与scratch按generation管理 |
| HAIR-P1-028 | 无stretch/bend/twist constraints | 选定rod/XPBD等模型，定义compliance、iterations、warm start与数值误差门 |
| HAIR-P1-029 | 无root/attachment constraint | root固定/滑移、skin motion、break、missing target与LOD迁移必须明确 |
| HAIR-P1-030 | 无hair-hair/self collision策略 | 邻接排除、thickness、friction、spatial acceleration、pair budget与quality tier |
| HAIR-P1-031 | 无body/world collision | 消费qualified collider/query snapshot，支持continuous/high-speed与initial overlap恢复 |
| HAIR-P1-032 | 无wind/aerodynamic输入 | 消费WindField generation，定义space、sampling、drag、air velocity、stale与fallback |
| HAIR-P1-033 | 无fixed-step/substep budget | Hair solver接Runtime22 clock，substep/iteration/update rate有overload degrade而非spiral |
| HAIR-P1-034 | 无NaN/energy/failure保护 | finite、strain/energy阈值、rollback/LKG、disable reason与fault isolation可观察 |
| HAIR-P1-035 | 无Groom cache schema/playback | 记录group/curve/point、time/sample、compression、artifact/binding generation与seek/reset |
| HAIR-P1-036 | 无cache streaming lifecycle | chunk prefetch/map/unmap、cancel、in-flight drain、pressure eviction和half-install禁止 |

## 10. P1：GPU Resource、Visibility、Composition 与 Temporal

| ID | 差距 | 必须重构 |
|---|---|---|
| HAIR-P1-037 | 无representation-specific GPU resource | rest/deformed guides/strands、cards、meshes、roots、interpolation、cluster分别建typed bundle |
| HAIR-P1-038 | 无current/previous publication | output带artifact/tick/output generation、fence、bounds和reset epoch，consumer拒绝错代 |
| HAIR-P1-039 | 无dynamic buffer pool/retirement | storage/vertex/indirect/copy用途、suballocation、GPU completion与device loss统一管理 |
| HAIR-P1-040 | 无dynamic/conservative bounds | CPU/GPU reduction、readback latency、simulation prediction和stale fallback分层 |
| HAIR-P1-041 | 无macro group与cluster culling | group bounds、screen error、curve/point selection、indirect args与overflow receipt |
| HAIR-P1-042 | 无strand raster geometry contract | curve segments、screen-space radius、tangent、pixel coverage、near clip与MSAA policy |
| HAIR-P1-043 | 无Hair visibility buffer | node/coverage/depth/material/velocity数据、capacity、compaction与tile分类独立实现 |
| HAIR-P1-044 | generic OIT对dense hair静默丢层 | Hair path需coverage-conserving overflow/degrade；cards fallback也必须报告损失 |
| HAIR-P1-045 | 无Hair depth/composition合同 | 与opaque depth、scene color、DOF/fog/post的顺序、transmittance和resolve generation一致 |
| HAIR-P1-046 | 无strand/card/mesh representation parity | 每LOD明确支持的visibility/material/shadow/velocity/RT能力与transition |
| HAIR-P1-047 | 无Hair velocity/history | current/previous strand或card deformation进入velocity、TAA、motion blur与reactive mask |
| HAIR-P1-048 | 无multi-view/stereo/capture政策 | simulation共享边界、per-view visibility/history、stereo reuse和offline sampling显式化 |

## 11. P1：Hair BSDF、Environment、Shadow、GI 与 RT

| ID | 差距 | 必须重构 |
|---|---|---|
| HAIR-P1-049 | 无typed Hair lighting model | 建Hair shading model和material schema，禁止以`Custom { name }`字符串代替核心合同 |
| HAIR-P1-050 | 无strand optical parameters | direction、melanin/absorption、dye、IOR、cuticle、longitudinal/radial roughness有units/range |
| HAIR-P1-051 | 无R/TT/TRT single scattering | 实现批准模型、lobe energy、Fresnel/absorption与reference numerical golden |
| HAIR-P1-052 | 无dual/multiple scattering | 局部/全局散射、density/strand count、LUT或analytic approximation有误差/成本门 |
| HAIR-P1-053 | 无Hair energy/IBL一致性 | direct、environment、emissive、exposure与preintegration在roughness/absorption域守恒 |
| HAIR-P1-054 | 无deep shadow/opacity | 为密集strand建立per-light macro group allocation、deep opacity/transmittance与budget |
| HAIR-P1-055 | 无Hair environment lighting/AO | 遮蔽、scatter、voxel/deep data和IBL按同一density/visibility代消费 |
| HAIR-P1-056 | 无voxel/transmittance representation | voxel page/DOM等provider有resolution、memory、update、overflow和stale政策 |
| HAIR-P1-057 | ordinary mesh shadow不适用strand | strand/cards/meshes分别定义shadow raster、filter、bias、alpha/coverage与fallback |
| HAIR-P1-058 | 无RT geometry/update策略 | curve/tube/card BLAS、refit/rebuild、any-hit、motion、memory和unsupported平台类型化 |
| HAIR-P1-059 | 无GI/path tracing语义 | Runtime09F3/28接Hair classification、visibility、scattering、denoise和fallback，禁止stale mesh代理 |
| HAIR-P1-060 | 无cross-path visual parity | raster/RT/path、strand/cards/meshes在批准误差范围内比较color、energy、shadow与temporal稳定性 |

## 12. P1：LOD、Streaming、Scalability、Diagnostics 与 Product Qualification

| ID | 差距 | 必须重构 |
|---|---|---|
| HAIR-P1-061 | 无per-group representation LOD | screen error、distance、importance、platform、budget与hysteresis选择strands/cards/meshes |
| HAIR-P1-062 | 无continuous curve/point LOD | cluster内curve/point decimation、radius compensation、stable order与indirect count可验证 |
| HAIR-P1-063 | 无LOD transition state | guide/deformation/material/coverage/history在representation与density切换时无pop/energy jump |
| HAIR-P1-064 | 无streaming chunk/residency合同 | source artifact按group/LOD/resource拆chunk，依赖bundle原子install且缺页有明确fallback |
| HAIR-P1-065 | 无global budget/admission | curve/point/segment/node/voxel/bytes/CPU/GPU time限制决定quality、freeze或representation |
| HAIR-P1-066 | 无platform capability matrix | storage/atomics/indirect/RT/MSAA/precision限制映射到可解释quality与Unsupported |
| HAIR-P1-067 | 无telemetry/debug snapshot | 记录groups、representation、curves/points、LOD、nodes、overflow、memory、CPU/GPU分位 |
| HAIR-P1-068 | 无fault/fuzz矩阵 | malformed source、stale binding、cancel、OOM、device loss、provider crash/unload与cache corruption |
| HAIR-P1-069 | 无asset/compiler tests | source/import/roundtrip/migration/build determinism、binding与LOD artifact golden为空 |
| HAIR-P1-070 | 无render numerical/pixel tests | coverage、BSDF、deep shadow、transmittance、velocity和representation parity为空 |
| HAIR-P1-071 | 无真实产品场景 | 建short/long/curly hair、fur、beard、braid、wind/collision/cache/LOD场景与save/export链 |
| HAIR-P1-072 | 无跨引擎超越基准 | 同资产/镜头/灯光/硬件/画质比较error、CPU/GPU、VRAM、stream stutter与raw receipt |

## 13. P2：完整性与长期竞争力

| ID | 后续能力 | 前置条件 |
|---|---|---|
| HAIR-P2-001 | procedural groom generation/edit graph | stable source IDs、compiler、transaction和deterministic artifact已完成 |
| HAIR-P2-002 | card/texture automatic generation | strand truth、bake provenance、coverage error和representation parity已完成 |
| HAIR-P2-003 | neural/learned deformation | authoritative fallback、training provenance、error bound与platform admission完成 |
| HAIR-P2-004 | advanced Cosserat rod GPU solver | CPU oracle、constraint model、fence/generation与fault matrix完成 |
| HAIR-P2-005 | wet/frozen/burning/damaged hair | Water/Weather/Gameplay/material adapter与identity/state authority稳定 |
| HAIR-P2-006 | cutting/growth/dynamic topology | stable strand/point ID、binding rebuild、cache/network与resource remap完成 |
| HAIR-P2-007 | braids/clumps/inter-strand constraints | self collision、constraint graph、budget和deterministic tie-break稳定 |
| HAIR-P2-008 | contact-aware styling/grooming | solver、collision、authoring transaction与runtime preview parity完成 |
| HAIR-P2-009 | compressed sparse/deep shadow research | deep shadow baseline、error metric、memory/quality receipt完成 |
| HAIR-P2-010 | hardware curve primitive backend | portable strand oracle、capability matrix、RT/raster parity与fallback完成 |
| HAIR-P2-011 | deterministic rollback/network hair | fixed tick、state digest、checkpoint、bandwidth与cosmetic authority完成 |
| HAIR-P2-012 | large-world partitioned Groom | origin/rebase、streaming owner、binding target continuity与cache seek完成 |
| HAIR-P2-013 | plugin deformer/solver/shading nodes | ABI/version/capability/budget/unload sandbox与compiler extension完成 |
| HAIR-P2-014 | collaborative groom authoring | stable semantic IDs、merge、locking、transaction/recovery与source provenance完成 |
| HAIR-P2-015 | accessibility motion-reduction profile | cosmetic/gameplay区分、quality profile与static/cards fallback完成 |
| HAIR-P2-016 | distributed qualification farm | deterministic corpus、artifact/build digest、GPU capture与raw receipt自动归档完成 |

## 14. 分层重构里程碑

### M0 · Truth、Owner 与 Baseline

- 冻结identifier/caller/capability清单，维持Hair Unsupported；禁止alpha mesh、anisotropy、particle trail、Morph或generic OIT false-ready。
- 批准Hair owner、术语、单位、representation、性能预算、CPU oracle和reference scene corpus。

### M1 · Source Schema、Importer 与 Compiler

- 建Hair source、stable IDs、groups/points/strands/guides/attributes与coordinate/unit schema。
- 完成Alembic/curve provider接口、migration、validation、deterministic compiler、DDC/LKG和malformed fuzz。

### M2 · Binding、Scene 与 Runtime Instance

- 编译root-to-surface与guide interpolation artifact，接skeletal/geometry-cache target generation。
- 接Scene persistence、component、instance lifecycle、per-group override、cancel/stale和multi-world isolation。

### M3 · Deformer、Cache 与 CPU Simulation

- 接animation/root deformation、cache record/playback/streaming与deterministic CPU guide solver。
- 完成stretch/bend/twist、attachment、collision、wind、fixed step、NaN/energy与teleport/reset。

### M4 · GPU Resource、Cluster 与 Representation

- 建rest/deformed/current/previous typed resource bundle、pool、fence、bounds、cluster和indirect work。
- 完成strands/cards/meshes LOD artifact、stream chunk、atomic install/retire和device loss恢复。

### M5 · Visibility、Coverage、Composition 与 Temporal

- 实现strand raster、coverage/node visibility、tile/compaction、overflow/degrade与scene composition。
- depth/velocity/TAA/motion blur/DOF/fog消费同一output generation，完成representation transition parity。

### M6 · Hair BSDF、Shadow、Environment 与 RT/GI

- 实现typed Hair material、R/TT/TRT、dual/multiple scattering、energy/IBL和LUT qualification。
- 完成deep shadow/transmittance、environment/voxel、RT geometry、GI/path tracing和cross-path parity。

### M7 · LOD、Streaming 与 Scalability

- 实现per-group representation LOD、continuous curve/point LOD、hysteresis和history migration。
- 完成global budget、platform matrix、residency/pressure、multi-view/XR/capture和telemetry。

### M8 · Editor/Product Integration

- 为后续Editor Groom authoring提供共享import/compiler/binding/preview/debug snapshot与transaction合同。
- 交付short/long/curly hair、fur、beard、braid场景的create/import/save/reopen/play/export/capture证据链。

### M9 · Reliability 与性能超越门

- 完成fault/fuzz/OOM/device loss/provider crash/unload、长时间soak和规模矩阵。
- 在同硬件、同source、同镜头/灯光、同画质与同输入下对比Unreal，领先声明由原始receipt复算。

## 15. 验收门

1. source对NaN/Inf、空/单点curve、非法knots/width/group/ID、重复属性和超预算输入给typed diagnostic。
2. group/strand/point/guide/material/LOD stable ID经save/reopen/reimport/migration不漂移。
3. coordinate、unit、width、basis/curve type和attribute cardinality在import后可复算，不靠隐式默认猜测。
4. 相同source/dependency/compiler/settings/target生成byte-identical artifact与相同digest。
5. importer cancel、OOM、malformed block、provider unload不会留下可被catalog接受的半成品。
6. source、binding、cache与representation artifact独立version，并有reader/writer/migration/LKG矩阵。
7. binding target topology/LOD/generation变化会拒绝旧root/interpolation数据并产生可解释rebuild。
8. root barycentric、guide weights与render interpolation对退化triangle、orphan root和zero weight fail-close。
9. provider缺失时Scene/catalog/Editor/App统一报告Unsupported且保留source，不创建空instance。
10. instance每个ticket唯一终态，world replace/unload后stale task/output不能发布。
11. 相同asset在两个world、preview与PIE的mutable deformation/cache/quality完全隔离。
12. animation -> root skin -> cache/simulation -> interpolation -> render extract phase固定且可trace。
13. teleport/reset/origin shift/cache seek分别处理current/previous/velocity/history并带typed epoch。
14. CPU oracle在固定artifact/input/tick政策下state/output digest可复算。
15. stretch/bend/twist/root constraint各有analytic/golden收敛和energy误差门。
16. self/body/world collision覆盖高速、initial overlap、thickness/friction、pair overflow与layer/filter。
17. WindField相同generation在不同update rate下结果满足误差门，stale field不混入新tick。
18. solver substep/iteration/update rate超预算进入可解释degrade/freeze/cache/cards fallback，无spiral或NaN扩散。
19. cache录制/seek/loop/reset与live result在批准误差内，损坏/缺帧/版本不匹配明确失败。
20. cache chunk cancel/unmap/eviction等待in-flight consumer收口，半安装bundle不可见。
21. GPU resource携rest/deformed/current/previous、artifact/tick/output generation和fence，consumer拒绝错代。
22. dynamic bounds覆盖真实strand/cards形变，GPU readback延迟有保守fallback且不过度膨胀超预算。
23. cluster/LOD culling输出curve/point/segment/indirect count可复算，overflow有drop/degrade receipt。
24. strand raster在subpixel width、near clip、extreme tangent、MSAA与高overdraw下coverage稳定。
25. visibility node capacity耗尽不会静默丢发；画质降级、重分配或fail状态可观察。
26. generic OIT只作为批准的cards/mesh fallback，其fragment loss与color precision误差有门。
27. Hair depth/composition与opaque、fog、DOF、post、exposure顺序固定，无halo、leak或generation混用。
28. velocity/TAA/motion blur区分连续变形、LOD/representation切换、teleport/reset和missing previous state。
29. strands/cards/meshes在同asset/material/pose下满足coverage、silhouette、energy和shadow parity门。
30. Hair BSDF在批准参数域满足finite、reciprocity/适用能量约束和reference numerical golden。
31. melanin/absorption/dye、cuticle和两类roughness的单位、范围、texture override与fallback一致。
32. R/TT/TRT、dual/multiple scattering在direct/IBL/raster/RT路径不重复计能量。
33. deep shadow/transmittance按light/macro group预算，stale或overflow不会静默退成opaque mesh shadow。
34. environment/voxel数据与visibility generation一致，resolution/pressure degrade记录画质与成本。
35. RT BLAS与any-hit对curve/card/tube表示执行批准的refit/rebuild/fallback，无stale geometry。
36. multi-view/stereo共享只发生在明确view-invariant资源，visibility/history/cut保持per-view正确。
37. global budget在1/10/100/1000 components与批准curve规模下无无界allocation、queue或readback。
38. debug snapshot能解释source/artifact/binding/provider/representation/LOD/overflow/memory/CPU/GPU time，关闭读者时无全量trace。
39. short/long/curly hair、fur、beard、braid场景通过source roundtrip、CPU oracle、WGPU像素/帧capture与soak。
40. 同口径benchmark同时记录visual/numerical error、CPU/GPU、RSS/VRAM、I/O、stream stutter与统计分布；领先声明可由raw receipt复算。

## 16. Finding 到里程碑映射

| Finding | 主里程碑 |
|---|---|
| HAIR-P1-001..012 | M0-M1 |
| HAIR-P1-013..024 | M2-M3 |
| HAIR-P1-025..036 | M3 |
| HAIR-P1-037..048 | M4-M5 |
| HAIR-P1-049..060 | M6 |
| HAIR-P1-061..072 | M7-M9 |
| HAIR-P2-001..016 | 对应P1与验收门完成后独立立项，不得提前并入MVP |

## 17. 禁止的临时修补

1. 禁止给`MeshRenderer`增加`hair: bool`、几个density/roughness字段和alpha texture后宣称Hair完成。
2. 禁止把Mesh custom attribute字符串、line list、particle trail、shell/fins或Morph target当作Groom source长期schema。
3. 禁止把generic anisotropic GGX称为Marschner/Hair BSDF，或把普通alpha blend/OIT称为strand visibility。
4. 禁止用固定每像素4/8/32层且静默丢overflow的OIT作为dense hair主路径。
5. 禁止让source、binding、simulation/cache、cards、strands和meshes各自维护不同group/strand identity。
6. 禁止CPU、GPU、Editor preview、cache playback各自实现不同root/interpolation/constraint语义。
7. 禁止在Hair owner内私建第二个skeleton truth、rigid world、clock、weather、material compiler或streamer。
8. 禁止每帧重建/上传完整curve或cards buffer、同步readback bounds，或无fence销毁in-flight资源。
9. 禁止depth、visibility、shadow、velocity、RT和GI读取不同代deformation或binding。
10. 禁止以cards fallback无条件覆盖unsupported路径；fallback必须带画质、成本、原因和终态receipt。
11. 禁止把Unity HDRP Hair shader当成完整Groom系统参考，也禁止把Unreal类型/默认参数逐字复制成Zircon架构。
12. 禁止在没有同source、同镜头/灯光、同画质、同硬件和原始receipt时宣称表现或性能超过Unreal。

## 18. 实施前重查清单

1. 重导148个输入manifest并重算`8e13c82d...`指纹；任何变化先标记本篇stale再评估finding。
2. 复核`frame_extract.rs`、`frame_extract/geometry.rs`、`material_features.rs`的blob状态，以及`import_gltf.rs`、`model_mesh_subassets.rs`相邻会话修改是否仍为纯格式。
3. 重跑production exact identifier、ResourceKind、Scene、RenderFeature、importer capability与tests查询，确认没有新Hair owner并入。
4. 取得Runtime04/05、08A/08C、09B/09C/09D/09E/09F3/09G2/09H1、22/23/24/28/31与Editor32/38 owner确认。
5. 先批准source/binding/compiler/CPU oracle，再选择GPU solver、visibility与BSDF实现；不得由某个现成shader或依赖便利性倒推产品架构。
6. 动态lane按Windows优先，先core/compiler/headless，再WGPU产品场景、GPU capture、fault/soak和跨引擎benchmark。

## 19. 本轮产出边界

本轮只新增静态review与分层重构计划，没有修改production Runtime、Editor、Interface、Plugin、App代码或tests，没有运行Cargo或WGPU。报告不表示Hair/Groom已经可用，也不授权从P2高级能力开工；实现必须从M0 truth/owner与M1 source/compiler开始，以stable source/binding identity、deterministic CPU oracle、generation-qualified GPU output和真实产品证据逐层收敛。
