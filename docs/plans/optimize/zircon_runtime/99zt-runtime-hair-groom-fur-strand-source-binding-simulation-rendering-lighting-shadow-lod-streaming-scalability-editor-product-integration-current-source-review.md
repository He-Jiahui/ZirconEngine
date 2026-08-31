---
title: Runtime Hair、Groom、Fur、Strand Source、Binding、Simulation、Rendering、Lighting、Shadow、LOD、Streaming、Scalability、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime145
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting
  - zircon_runtime/src/core/framework/render/material
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature
  - zircon_runtime/src/graphics/material/shading_models
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/oit_buffers
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_runtime/src/graphics/visibility
  - zircon_plugins/asset_importers/model/runtime/src
  - zircon_plugins/gltf_importer/runtime/src
  - zircon_plugins/rendering/features/oit/runtime
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src/entry
  - zircon_editor/src/core/asset/type_registry
tests:
  - zircon_runtime/src/asset/tests/assets/mesh/morph_targets.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/transparent3d.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/shading_model_parity.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests
  - zircon_plugins/gltf_importer/runtime/src/tests.rs
  - zircon_editor/src/tests/editor_asset_type_registry/builtins.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/32-hair-groom-fur-strand-source-binding-simulation-rendering-lighting-shadow-lod-streaming-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99b-runtime-temporal-aa-velocity-history-dynamic-resolution-upscaling-reconstruction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zl-runtime-animation-skeleton-clip-pose-graph-state-machine-layer-mask-blend-ik-root-motion-event-extract-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zm-runtime-physics-world-body-shape-collider-material-joint-query-contact-trigger-fixed-step-jolt-character-controller-vehicle-ragdoll-debug-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99n-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source
  - dev/UnrealEngine/Engine/Plugins/Importers/AlembicHairImporter/Source
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Public/Groom
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Groom
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/HairStrands
  - dev/UnrealEngine/Engine/Shaders/Private/HairStrands
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Hair
  - dev/bevy/crates
  - dev/godot/scene
  - dev/godot/modules
  - dev/godot/servers/rendering
  - dev/Fyrox/fyrox-impl/src
  - dev/Fyrox/fyrox-resource/src
---

# Runtime Hair、Groom、Fur、Strand 与 Product Integration 当前源码工程化差距

## 1. 结论

当前 Zircon 没有 Hair、Groom、Fur 或 Strand 运行时产品。对 `zircon_runtime`、`zircon_runtime_interface`、`zircon_editor`、`zircon_app`、`zircon_plugins` 与 `examples` 中排除 `tests/test_sources/benches/target` 路径及 test-named Rust 文件后的 **12,188 个 production Rust 文件**执行精确词边界扫描，`hair/hairstrands/groom/fur/strand(s)/alembic/fiber/fibre/follicle/melanin/marschner/kajiya-kay/wind field/wind snapshot` 命中为 **0 文件、0 条**。对 Editor/App/catalog/example 的 Rust、TOML、JSON、ZUI 与 WGSL 产品面复核也没有 Hair resource、component、provider、feature、operation、toolkit或 capability。

仓内已有 typed Mesh attribute/topology、skin/Morph、四类 geometry-source descriptor、current/previous deformation、velocity、render graph、generic OIT、shadow、resource residency、Mesh LOD、fixed-step Physics、rigid query/collider和 shading-model plugin registry。这些是真实通用底座，但没有 Hair-owned stable identity、source/artifact、binding、runtime instance、simulation/cache output generation、strand visibility、Hair BSDF、deep transmittance、Editor authoring或产品场景。尤其是当前 OIT 默认平均每像素4层，WGSL 在 `atomicAdd` 后遇到容量溢出直接 `return`，颜色以 `pack4x8unorm` 存储；它无法替代 strand coverage、Hair visibility node、deep shadow或密度守恒 composition。

产品代码没有把 Hair 标为 Ready/Executed，因此本篇不重复创建 false-ready P0，登记 **0 项新的 Hair-owned P0**。历史72项P1按当前 bytes重判为 **56 Open / 16 Partial / 0 Closed**，16项P2全部Open；40项资格门为 **37 Fail / 3 Partial / 0 Pass**。Partial只表示通用owner已经提供同一合同的可复用前置，不表示Hair产品链已启动。目标必须硬切到：

```text
Hair/Groom Source + Import Provenance
  -> deterministic Hair Compiler
  -> Group/Guide/Render-Strand + Root Binding/Interpolation Artifact
  -> Cards/Meshes + Cluster/LOD/Streaming/RT Metadata
  -> generation-qualified per-World HairRuntimeInstance
  -> admitted Deformer / Simulation / Cache Provider
  -> current+previous representation resources + dynamic bounds + fence
  -> Hair Visibility / Lighting / Shadow / RT / GI adapters
  -> runtime-backed Editor authoring、preview 与 qualification receipts
```

## 2. 审查边界、方法与 currentness

### 2.1 冻结范围

冻结基线为 `main@94f86015d0da980d6c93ef3cf3fcd9d759d0e477` 的当前 working bytes。初次冻结时共享工作树已有3,253个tracked changes、5,334个含untracked changes；最终复核期间并行工作又增加4个selected产品/测试文件，但Hair命中仍为0，表中记录最终复核bytes。本文不归因、不覆盖、不回退任何既有改动。用户已明确暂不优化 tooling，本轮没有扫描或规划未来将迁移到Rust的 tooling 实现。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树指纹 |
|---|---:|---|
| Source、Scene、Import 与 Editor carrier | **139 / 27,657 / 25,233 / 961,010 / 141 / 37** | `9827386d74d34268ddb60d44463b7b1610f3e0a8d60c73ec9602892580d65192` |
| Animation、Physics、runtime 与 streaming 前置 | **188 / 27,422 / 24,926 / 959,600 / 104 / 1** | `7ae6b2c0fea95bed25ae32c09f652c8507d428c5b5473927803e4bc226b6bf92` |
| Render、Deformation 与 Material 前置 | **338 / 55,194 / 50,122 / 1,992,320 / 374 / 2** | `a71fa9715e769018301a72b4d12de066a6b1f9d934bb1f320143ec60415c6454` |
| Catalog、App、focused tests 与产品证据 | **207 / 32,489 / 29,698 / 1,181,372 / 502 / 3** | `957a3f546dbb5ec508f601435b3769979864440275820c1089c31cf14f05b188` |
| Zircon selected union | **872 / 142,762 / 129,979 / 5,094,302 / 1,121 / 43** | `884a734bf9265f236c8d399ee1b20f6e730452ac87b29f2b3204fa498f4bf651` |
| Unreal Hair Core、Import、Binding、Solver 与 Editor | **271 / 86,786 / 73,374 / 3,427,467 / 0 / 0** | `853afeebbfcbaba21c6bd65d83ce701cb7f44f596ce61444ab41278e26cd035e` |
| Unreal Hair Renderer 与 Shader family | **112 / 46,052 / 38,852 / 1,859,028 / 0 / 0** | `be1f265299078a6db97c25b1886ef5eb536f1347f7dd05c6a2abf91e9489bf4a` |
| Unity HDRP Hair | **10 / 3,159 / 2,565 / 129,833 / 0 / 0** | `f5ba559301237d967e66ca1e6fad26a97c2cd9c34bcf77cebafcaa21d1117945` |
| Bevy、Fyrox、Godot通用基础选择集 | **4 / 6,142 / 5,122 / 224,565 / 0 / 0** | `f6cbab81982f46add6bd296946ce06ec71fdc9012b1e2d098fe20dffc90f7b25` |
| reference selected union | **397 / 142,139 / 119,913 / 5,640,893 / 0 / 0** | `5a55bd3e14303f6e57ad0e5e01db48ba99b5512144916a8286767682e1a6b500` |

指纹算法为：repository-relative path转`/`并小写排序；每个文件取当前bytes的SHA-256；聚合输入为`path + NUL + lowercase(file_sha256) + LF`，再取SHA-256。tests统计Rust `#[test]`/`#[tokio::test]`，参考树按实际文件bytes计算，不假设Git tracking状态。

### 2.2 纵向扫描链

本轮逐层核对 resource kind/source schema -> importer capability/attribute conversion -> stable identity/version -> deterministic compiler/artifact -> root binding/guide interpolation -> Scene persistence/component -> per-World instance/lifecycle -> animation/deformer/cache/simulation -> collision/wind/fixed clock -> current/previous GPU resources/bounds -> macro group/cluster/LOD/streaming -> strand/cards/mesh visibility -> coverage/composition/OIT -> Hair BSDF/environment/multiple scattering -> deep shadow/transmittance/voxel -> velocity/TAA/motion blur -> RT/GI/path tracing -> multi-view/scalability/telemetry/tests/Editor/product evidence。没有发现plugin facade、inline test、feature descriptor、Custom shading字符串或示例之后隐藏的第二套Hair production owner。

### 2.3 证据等级与执行限制

本篇达到E3 source-level review。没有运行Cargo、WGPU、App、Editor、PIE、asset cook、save/reopen、GPU capture、fault/fuzz、scale/soak或竞争benchmark，因为当前没有Hair source/component/provider/pass可执行；运行Mesh、Morph、OIT、shadow或PBR测试不能提高Hair结论。实施前必须重取指纹，从capability truth、source roundtrip、compiler与CPU oracle的RED证据开始。

## 3. 当前产品链事实

### 3.1 Source、Resource、Scene 与 Product Truth

1. `ResourceKind`有26种Data/Model/Mesh/Material/Scene/Physics/Nav/Terrain/Tile/Prefab/Animation/UI资源，没有HairSource、Groom、Binding、Cache、HairMaterial、Profile或BuildArtifact。
2. `SceneEntityAsset`只持有camera、mesh、light、post process、rigid/collider/joint、animation、terrain、tilemap、prefab与scripts；没有Hair component，也没有可无损承载未知typed plugin component的通用槽。
3. `MeshRenderer`只有model/mesh/material、queue、depth bias、Morph weights、primitives、LOD、material overrides、tint与alpha；没有source、binding target、guide/cache、representation、simulation、reset、group override或quality policy。
4. 首方catalog、App entry/composition和Editor内建asset registry均没有Hair provider/package/capability。Editor的26个builtin ResourceKind和现有UI/Animation toolkit不含Groom authoring。
5. catalog/App/Editor没有Hair Ready声明，因此当前是“能力缺失但未虚报”，不是可用产品。

### 3.2 Mesh、Import、Animation 与 Binding 前置

1. `MeshAsset`已有BTreeMap typed vertex attributes、indices、topology、skin、Morph、bounds与validation；它的domain是surface vertex，缺curve offset/count、strand/group cardinality、knots、width、basis/curve type和stable Hair identity。
2. `MeshAsset::from_model_primitive`与glTF importer走triangle surface；glTF明确拒绝非Triangles primitive。Model importer虽真实支持OBJ/PLY/STL/DXF等mesh/CAD格式，但没有Alembic/USD Groom/curve payload。
3. 通用importer contract、typed outcome、artifact/version/dependency和resource revision是可复用前置；没有Hair importer capability、source provenance、unit/width conversion、attribute scope policy或Hair compiler。
4. `AnimationPoseOutput`只表达source、active state和bones；没有root deformation、binding generation、cache time、simulation/reset epoch或Hair phase receipt。
5. Scene mesh/skin和geometry cache目标合同、root barycentric artifact、guide-to-render interpolation与dependency invalidation均不存在。

### 3.3 Simulation、Collision、Wind 与 Cache

1. Physics backend只拥有shape/body/constraint、commands、step、queries和events，没有Hair guide state、rod/XPBD solver、root attachment、自碰撞或strand contact。
2. fixed clock、solver groups、raycast/overlap/shape cast、layer/mask/material/contact、skeletal collider/Ragdoll是真实通用底座；它们没有组装成Hair-qualified collider snapshot、continuous collision或deterministic CPU oracle。
3. 没有stretch/bend/twist/root constraint、hair-hair adjacency exclusion、thickness/friction、spatial pair budget、NaN/energy rollback或teleport/reset政策。
4. 没有WindField/WindSnapshot producer，Hair也没有air drag/velocity、sampling space、stale generation或overload degrade。
5. 没有Groom cache schema、record/playback/seek、chunk prefetch/map/unmap、in-flight drain、pressure eviction或half-install rejection。

### 3.4 GPU Resource、Visibility、Composition 与 Temporal

1. `BuiltinRenderFeature`有42种Mesh/SkinnedMesh/MeshLod/Particle/GI/RT/Terrain/Tree/Trail/Billboard/VG等feature，没有HairStrands、HairCards、HairMeshes、HairVisibility或HairShadow。
2. `RenderFrameExtract`只有world/view/geometry/animation poses/lighting/environment/post/debug/sprites/particles/visibility，没有Hair sideband、macro group、deformation resource或binding generation。
3. 四个builtin geometry source只覆盖static/skinned/morphed/skinned+morphed mesh，plugin ID从4开始；这个注册入口可复用，但没有Hair vertex/binding descriptor或runtime provider。
4. `GpuMeshResource`只有vertex/index buffers、count、indirect signature、wire segments与静态bounds；上传以`VERTEX`/`INDEX`为主，没有Hair rest/deformed/current/previous bundle、STORAGE/COPY pool、fence retire或动态bounds。
5. GPU Scene和mesh velocity已有current/previous skin palette与Morph前置，但没有strand/card deformation、reset/cut epoch、reactive mask或representation transition history。
6. generic OIT有真实fragment store/resolve和capability fallback，但每像素容量固定、溢出静默丢弃、颜色压成RGBA8，且完全没有Hair coverage conservation、node capacity receipt、tile classification或deep opacity。

### 3.5 Hair BSDF、Lighting、Shadow、RT 与 GI

1. Material lighting model只有Pbr、BlinnPhong、Unlit和`Custom { name }`；shading registry提供plugin ID入口，但没有typed Hair model、schema或qualification。
2. Standard PBR已有clearcoat、anisotropy、specular/diffuse transmission、thickness、IOR与attenuation；没有strand direction、melanin/absorption/dye、cuticle angle、longitudinal/radial roughness。
3. 没有R/TT/TRT、dual/multiple scattering、Hair energy/IBL、Hair environment AO、deep shadow、voxel/transmittance或density visibility。
4. ordinary mesh shadow、generic RT/GI和path接口没有curve/card/tube geometry、any-hit opacity、motion/refit/rebuild或Hair classification。
5. 没有raster/RT/path与strands/cards/meshes之间的color、energy、shadow、coverage和temporal parity测试。

### 3.6 Editor、Diagnostics 与 Product Qualification

1. 没有Groom asset factory/importer/compiler operation、binding builder、document/toolkit、viewport或transactional editor。
2. 没有curve/group/guide编辑、root binding inspection、cards generation、LOD/cluster可视化、simulation/collision/wind/cache preview或material group authoring。
3. 没有Hair runtime debug snapshot、overflow/memory/CPU/GPU telemetry、fault/fuzz矩阵或provider crash/unload/device-loss证据。
4. 没有short/long/curly hair、fur、beard、braid产品场景，没有source roundtrip、save/reopen/play/export/capture/soak和同口径跨引擎benchmark。

## 4. 必须保留的真实底座

1. 保留asset/importer/artifact/version/dependency框架，把Hair source、binding、cache和representation artifact接入统一publish/install/retire语义。
2. 保留Mesh typed attribute/topology/skin/Morph/bounds作为cards/meshes及scalp target前置；strand source必须独立建模。
3. 保留Animation pose generation、Physics fixed clock/query/collider和current/previous skin/Morph；Hair通过typed adapter消费，不私建第二套skeleton、rigid world或clock。
4. 保留RenderFrameExtract、geometry-source registry、GPU Scene、render graph、velocity和resource streamer；Hair只发布generation-qualified output与bounds。
5. 保留generic OIT作为批准的cards/mesh fallback入口，但必须补overflow/precision receipt；strand主路径使用独立visibility/coverage/composition。
6. 保留shading-model plugin registry作为注册机制，但核心Hair model必须typed、versioned且具CPU/reference numerical oracle。
7. 保留catalog/App/Editor当前诚实不宣称能力的truth；只有source->runtime->render->Editor闭环与qualification通过后才能提升capability。

## 5. 五套参考源码给出的工程边界

### 5.1 Unreal HairStrands 是source/runtime/renderer/editor主参考

Unreal的`FHairDescription`分离vertex、strand和groom identity，并以versioned bulk data保存；`HairAttributes`定义position、width、color、roughness、AO、group、guide、ID、clump、root UV、closest guides/weights、basis/curve type、knots、tool与properties。`FHairStrandsDatas`继续分离raw/built points/curves、packed/transcoded格式、interpolation、root binding、cluster LOD与streaming request。Zircon不能把这些压成Mesh custom字符串attribute。

`UGroomAsset`按group保存guides/strands/cards/meshes、LOD、material、interpolation与physics；`UGroomBindingAsset`支持source/target SkeletalMesh或GeometryCache、async build、LOD引用、derived keys、compatibility、invalidate和resource rebuild。`GroomResources`分离rest/deformed/root/interpolation/raytracing及current/previous资源，`GroomCacheStreamingManager`负责chunk与in-flight生命周期。Hair必须拥有artifact、instance、provider、resource generation边界，不能依赖一个MeshRenderer布尔开关。

Renderer又独立分离visibility/coverage/compaction/tile、macro group、cluster culling、velocity、composition、deep shadow、transmittance、environment、voxelization、forward raster和ray tracing；Shader family与这些pass一一对应。Editor包含factory、reimport、binding、cache track、details、material、viewport、follicle/texture生成和Groom editor mode。这说明“有Hair shader”远低于产品完成线。

### 5.2 Unity HDRP Hair 是光学与跨路径参考

Unity HDRP Hair区分Kajiya-Kay、Marschner与Marschner Cinematic，Surface/BSDF data包含strand direction、transmittance、rim transmission、secondary lobes、melanin/absorption、azimuthal roughness、cuticle angle与strand-count visibility。64立方LUT预积分attenuation、azimuthal与longitudinal scattering；reference实现计算R/TT/TRT attenuation、longitudinal/azimuthal分布并可importance sample，且具raster、ray-tracing和path-tracing接线。

Unity Graphics只证明Hair光学和render-path一致性，不是Groom source、binding或solver参考。Zircon必须分别完成source/runtime与BSDF，不能用任何一侧替代另一侧。

### 5.3 Bevy、Godot、Fyrox 是诚实负证据

当前本地Bevy crates的1,408个源码文件没有Hair/Groom/Fur/Strand模块名，内容仅在PBR anisotropy文档中出现1条一般`hair`示例；Fyrox目标173个源码文件零命中；Godot scene/modules/rendering共1,899个源码文件零命中。它们的Mesh、skinning、Morph、ECS和material实现可作通用基础对照，但不能成为Hair功能完成度基线，也不能把中型引擎缺失变成Zircon降级许可。

## 6. 目标架构与唯一 Owner

| 领域 | 唯一owner | Hair145只消费/提供 |
|---|---|---|
| Resource/schema/artifact/build | Runtime04/85/86 | Hair typed source、dependency、compiler artifact、migration、install/retire receipt |
| Scene/ECS/world lifecycle | Runtime05/99j | component identity、instance generation、world teardown与serialization |
| Skeleton/pose/deformer | Runtime08C/99zl | qualified pose/root target snapshot、teleport/cut epoch |
| Rigid collision/fixed clock/wind | Runtime08A/22/31/99zm | collider/query snapshot、clock、WindField generation；Hair不私建父系统 |
| Hair domain | 新Runtime Hair owner | source、compiler、binding、instance、simulation/cache、representation与diagnostics |
| Visibility/GPU Scene | Runtime09B/current visibility owner | macro group、bounds、cluster/LOD、indirect work与visibility receipt |
| Material/shader/PSO | Runtime09C/91/09G2 | Hair BSDF、variants、LUT与energy qualification |
| Residency/streaming | Runtime09D/99m | chunk/bundle admission、budget、atomic install、retire与pressure policy |
| Shadow/environment/GI/RT | Runtime09E/09F/28 | deep shadow/transmittance、environment、GI/RT update/fallback |
| Editor authoring | Editor32扩展或新Hair toolkit | 只消费Runtime schema/compiler/binding/preview/debug snapshot，不拥有私有Hair truth |

## 7. 状态判定规则

- `Open`：没有Hair-owned类型、producer/consumer或同域artifact；普通Mesh/Render/Physics能力不足以提高状态。
- `Partial`：相邻owner已经提供同一合同可直接复用的前置，但Hair adapter、identity、generation、failure或qualification仍不存在。
- `Closed`：必须有Hair source到产品consumer的静态闭环和对应动态证据；本轮为0。
- P0只登记false-ready、确定性数据损坏或已激活产品正确性阻断；当前Hair完全缺失且未虚报，因此为0。

## 8. P1：Source、Import、Schema、Build 与 Provenance

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| HAIR-P1-001 | Open | `ResourceKind`无Hair/Groom；新增Source、BuildArtifact、Binding、Cache、Material/Profile kind与versioned handle |
| HAIR-P1-002 | Open | 无独立source asset；建立`HairSourceAsset`并硬分离authoring curves与cooked runtime data |
| HAIR-P1-003 | Open | 无group/strand/point stable ID；跨reimport、LOD、merge、diagnostic与cache保持身份，禁止数组下标长期身份 |
| HAIR-P1-004 | Partial | Mesh有typed per-vertex attribute前置，但无curve domain/cardinality/units；Hair定义position、width、color、roughness、AO与missing policy |
| HAIR-P1-005 | Open | 无per-strand/group schema；guide、clump、root UV、group/material、curve/basis/knots及custom namespace必须可验证 |
| HAIR-P1-006 | Open | 无source coordinate/unit合同；显式定义up/forward/handedness、meters、width interpretation、transform bake与precision |
| HAIR-P1-007 | Open | 无Alembic/curve/USD Groom importer；引入provider、format version、attribute map、cancel/budget与typed diagnostics |
| HAIR-P1-008 | Partial | 通用importer已有capability/outcome框架但只处理Model surface；Hair importer独立注册，禁止`.abc`/USD curves路由为triangle Model |
| HAIR-P1-009 | Partial | Mesh已有finite/topology/cardinality validation，但无Hair规则；zero-point、NaN、negative width、bad knots、duplicate ID与超预算必须fail-close |
| HAIR-P1-010 | Open | 无deterministic Hair artifact；编译groups/guides/strands/interpolation/cards/meshes/clusters/LOD及compiler/dependency digest |
| HAIR-P1-011 | Partial | 通用artifact/version/migration/LKG前置存在，但无Hair schema/artifact version；建立incremental rebuild、LKG与retirement |
| HAIR-P1-012 | Partial | import outcome/source revision提供provenance前置，但无Hair build receipt；记录source hash、settings、tool/target、repair、cost与output digest |

## 9. P1：Binding、Scene、Instance、Deformer 与 Lifecycle

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| HAIR-P1-013 | Open | 无root-to-surface binding artifact；保存target identity、triangle/root index、barycentric、rest transform与precision |
| HAIR-P1-014 | Partial | skeleton/skin/Morph target前置存在但无Hair binding；类型化target kind、LOD/section、skin/cache generation与missing/reimport policy |
| HAIR-P1-015 | Open | 无guide-to-render interpolation；closest guides、weights、local point mapping与批准算法编译为可验证artifact |
| HAIR-P1-016 | Partial | asset dependency/revision与stale rejection可复用，但无binding owner；source/target topology/LOD/settings变化必须重建且旧代不发布 |
| HAIR-P1-017 | Open | Scene无Hair component；持有source/artifact/binding/material groups、representation、simulation/cache与quality policy |
| HAIR-P1-018 | Open | 无Hair instance；建立world/entity/artifact generation、state、bounds、provider、resource set与terminal receipt |
| HAIR-P1-019 | Open | 无生命周期ticket；Requested/Preparing/Active/Suspended/Retiring/Failed/Cancelled必须唯一终态 |
| HAIR-P1-020 | Open | 无per-group override；material、width/density、LOD、visibility、shadow、simulation按stable group ID覆盖 |
| HAIR-P1-021 | Partial | Animation/Render已有phase前置，但无Hair schedule；固定pose -> root skin -> cache/sim -> interpolation -> extract并可trace |
| HAIR-P1-022 | Open | 无teleport/reset/cut语义；preserve/reset position/velocity、cache seek、origin shift与camera cut分别带epoch |
| HAIR-P1-023 | Open | 无Hair async ownership；task携world/instance/artifact/target/tick generation，cancel/panic后不发布 |
| HAIR-P1-024 | Open | 无two-world/multi-instance隔离；PIE、preview、多world/component不得共享mutable deformation truth |

## 10. P1：Simulation、Collision、Wind 与 Cache

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| HAIR-P1-025 | Open | 无solver/deformer provider；定义CPU/GPU能力、model、limits、determinism、async、memory与fallback admission |
| HAIR-P1-026 | Open | 无deterministic CPU oracle；先交付小规模guide baseline供compiler、cache、GPU differential与headless使用 |
| HAIR-P1-027 | Open | 无guide state；current/previous position、velocity、orientation、rest length与scratch按generation管理 |
| HAIR-P1-028 | Open | 无stretch/bend/twist；选择rod/XPBD等模型，定义compliance、iterations、warm start与误差门 |
| HAIR-P1-029 | Open | 无root/attachment constraint；root固定/滑移、skin motion、break、missing target与LOD迁移显式化 |
| HAIR-P1-030 | Open | 无self collision；邻接排除、thickness、friction、spatial acceleration、pair budget与quality tier |
| HAIR-P1-031 | Partial | Physics已有collider/query前置但无Hair snapshot；消费qualified generation并覆盖continuous/high-speed与initial overlap |
| HAIR-P1-032 | Open | 无wind/aerodynamic输入；消费WindField generation，定义space、sampling、drag、air velocity、stale与fallback |
| HAIR-P1-033 | Partial | Runtime已有fixed-step前置但无Hair policy；substep/iteration/update rate超预算进入typed degrade而非spiral |
| HAIR-P1-034 | Open | 无NaN/energy fail-safe；finite、strain/energy阈值、rollback/LKG、disable reason与fault isolation可观察 |
| HAIR-P1-035 | Open | 无Groom cache；记录group/curve/point、time/sample、compression、artifact/binding generation与seek/reset |
| HAIR-P1-036 | Open | 无cache streaming lifecycle；chunk prefetch/map/unmap、cancel、in-flight drain、pressure eviction及half-install禁止 |

## 11. P1：GPU Resource、Visibility、Composition 与 Temporal

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| HAIR-P1-037 | Open | `GpuMeshResource`是静态surface owner；rest/deformed guides/strands、cards、meshes、roots、interpolation、cluster分别建typed bundle |
| HAIR-P1-038 | Partial | skin/Morph已有current/previous前置但无Hair output；发布artifact/tick/output generation、fence、bounds与reset epoch |
| HAIR-P1-039 | Open | 无dynamic pool/retirement；storage/vertex/indirect/copy用途、suballocation、GPU completion与device loss统一管理 |
| HAIR-P1-040 | Open | bounds只来自静态source；建立CPU/GPU reduction、readback latency、simulation prediction与stale fallback |
| HAIR-P1-041 | Open | VG cluster不是Hair macro group；按group bounds、screen error、curve/point selection、indirect args与overflow receipt实现 |
| HAIR-P1-042 | Open | 无strand raster contract；curve segments、screen-space radius、tangent、pixel coverage、near clip与MSAA policy |
| HAIR-P1-043 | Open | 无Hair visibility buffer；node/coverage/depth/material/velocity、capacity、compaction与tile分类独立实现 |
| HAIR-P1-044 | Partial | generic OIT真实存在但超容量静默drop且RGBA8打包；Hair需coverage-conserving overflow/degrade，cards fallback也报告损失 |
| HAIR-P1-045 | Open | 无Hair depth/composition；固定与opaque depth、scene color、DOF/fog/post的顺序、transmittance与resolve generation |
| HAIR-P1-046 | Open | 无strands/cards/meshes parity；每LOD声明visibility/material/shadow/velocity/RT能力与transition |
| HAIR-P1-047 | Partial | mesh skin/Morph velocity有current/previous前置，但无Hair/reset epoch；接TAA、motion blur与reactive mask |
| HAIR-P1-048 | Open | 无multi-view/stereo/capture政策；simulation共享边界、per-view visibility/history、stereo reuse与offline sampling显式化 |

## 12. P1：Hair BSDF、Environment、Shadow、GI 与 RT

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| HAIR-P1-049 | Open | plugin shading registry不是Hair model；建立typed Hair shading model/schema，禁止以`Custom { name }`代替核心合同 |
| HAIR-P1-050 | Open | 无optical parameters；direction、melanin/absorption、dye、IOR、cuticle、longitudinal/radial roughness含units/range |
| HAIR-P1-051 | Open | 无R/TT/TRT；实现批准模型、lobe energy、Fresnel/absorption与reference numerical golden |
| HAIR-P1-052 | Open | 无dual/multiple scattering；density/strand count、LUT或analytic approximation必须有误差/成本门 |
| HAIR-P1-053 | Open | generic PBR energy不证明Hair；direct、environment、emissive、exposure与preintegration在Hair参数域守恒 |
| HAIR-P1-054 | Open | 无deep shadow/opacity；per-light macro group allocation、deep opacity/transmittance与budget |
| HAIR-P1-055 | Open | 无Hair environment/AO；scatter、voxel/deep data和IBL消费同一density/visibility generation |
| HAIR-P1-056 | Open | 无voxel/transmittance；provider定义resolution、memory、update、overflow与stale policy |
| HAIR-P1-057 | Open | ordinary mesh shadow不适用strand；strands/cards/meshes分别定义raster、filter、bias、coverage与fallback |
| HAIR-P1-058 | Open | 无RT geometry/update；curve/tube/card BLAS、refit/rebuild、any-hit、motion、memory与Unsupported类型化 |
| HAIR-P1-059 | Open | 无GI/path tracing语义；接Hair classification、visibility、scattering、denoise与fallback，禁止stale mesh代理 |
| HAIR-P1-060 | Open | 无cross-path parity；raster/RT/path及strands/cards/meshes比较color、energy、shadow与temporal stability |

## 13. P1：LOD、Streaming、Scalability、Diagnostics 与 Product

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| HAIR-P1-061 | Partial | MeshRenderer已有render LOD前置但无Hair representation policy；按screen error、importance、platform、budget与hysteresis选strands/cards/meshes |
| HAIR-P1-062 | Open | 无continuous curve/point LOD；cluster内decimation、radius compensation、stable order与indirect count可验证 |
| HAIR-P1-063 | Open | 无LOD transition；guide/deformation/material/coverage/history切换时禁止pop与energy jump |
| HAIR-P1-064 | Partial | texture/material residency可复用但无Hair bundle；按group/LOD/resource拆chunk并原子install，缺页fallback显式 |
| HAIR-P1-065 | Open | 无Hair global admission；curve/point/segment/node/voxel/bytes/CPU/GPU time决定quality、freeze或representation |
| HAIR-P1-066 | Partial | 通用capability/profile有storage/indirect/RT等前置，但无Hair matrix；限制映射到可解释quality与Unsupported |
| HAIR-P1-067 | Open | 无telemetry/debug snapshot；记录groups、representation、curves/points、LOD、nodes、overflow、memory与CPU/GPU分位 |
| HAIR-P1-068 | Open | 无fault/fuzz矩阵；覆盖malformed source、stale binding、cancel、OOM、device loss、provider crash/unload与cache corruption |
| HAIR-P1-069 | Open | 无asset/compiler tests；source/import/roundtrip/migration/build determinism、binding与LOD artifact golden为空 |
| HAIR-P1-070 | Open | 无render numerical/pixel tests；coverage、BSDF、deep shadow、transmittance、velocity与representation parity为空 |
| HAIR-P1-071 | Open | 无真实产品场景；建short/long/curly hair、fur、beard、braid、wind/collision/cache/LOD及save/export链 |
| HAIR-P1-072 | Open | 无超越基准；同source/镜头/灯光/硬件/画质比较error、CPU/GPU、VRAM、stream stutter与raw receipt |

## 14. P2：完整性与长期竞争力

| ID | 状态 | 后续能力与前置 |
|---|---|---|
| HAIR-P2-001 | Open | procedural groom generation/edit graph；需stable source ID、compiler、transaction与deterministic artifact |
| HAIR-P2-002 | Open | card/texture automatic generation；需strand truth、bake provenance、coverage error与representation parity |
| HAIR-P2-003 | Open | neural/learned deformation；需authoritative fallback、training provenance、error bound与platform admission |
| HAIR-P2-004 | Open | advanced Cosserat rod GPU solver；需CPU oracle、constraint model、fence/generation与fault matrix |
| HAIR-P2-005 | Open | wet/frozen/burning/damaged hair；需Water/Weather/Gameplay/material adapter与identity/state authority |
| HAIR-P2-006 | Open | cutting/growth/dynamic topology；需stable strand/point ID、binding rebuild、cache/network与resource remap |
| HAIR-P2-007 | Open | braids/clumps/inter-strand constraints；需self collision、constraint graph、budget与deterministic tie-break |
| HAIR-P2-008 | Open | contact-aware styling/grooming；需solver、collision、authoring transaction与runtime preview parity |
| HAIR-P2-009 | Open | compressed sparse/deep shadow research；需deep shadow baseline、error metric、memory/quality receipt |
| HAIR-P2-010 | Open | hardware curve primitive backend；需portable strand oracle、capability matrix、RT/raster parity与fallback |
| HAIR-P2-011 | Open | deterministic rollback/network Hair；需fixed tick、state digest、checkpoint、bandwidth与cosmetic authority |
| HAIR-P2-012 | Open | large-world partitioned Groom；需origin/rebase、stream owner、binding continuity与cache seek |
| HAIR-P2-013 | Open | plugin deformer/solver/shading nodes；需ABI/version/capability/budget/unload sandbox与compiler extension |
| HAIR-P2-014 | Open | collaborative groom authoring；需stable semantic ID、merge、locking、transaction/recovery与provenance |
| HAIR-P2-015 | Open | reduced-motion profile；需cosmetic/gameplay区分、quality profile与static/cards fallback |
| HAIR-P2-016 | Open | distributed qualification farm；需deterministic corpus、artifact/build digest、GPU capture与raw receipt归档 |

## 15. 分层重构里程碑

### M0 · Truth、Owner 与 Baseline

- 维持Hair Unsupported，冻结identifier/caller/capability清单；禁止alpha mesh、anisotropy、particle trail、Morph或generic OIT false-ready。
- 批准Hair owner、术语、单位、representation、预算、CPU oracle和reference scene corpus。

### M1 · Source、Importer 与 Compiler

- 建stable group/strand/point/guide ID、typed attributes、coordinate/unit schema与versioned source。
- 完成Alembic/curve provider、migration、validation/fuzz、deterministic compiler、artifact digest与LKG。

### M2 · Binding、Scene 与 Runtime Instance

- 编译root-to-surface和guide interpolation artifact，接skeletal/geometry-cache target generation。
- 接Scene roundtrip、component、per-World instance、lifecycle、group override、cancel/stale与multi-world隔离。

### M3 · Deformer、Cache 与 CPU Simulation

- 接pose/root phase、cache record/playback/streaming与deterministic CPU guide solver。
- 完成constraints、collision、wind、fixed step、NaN/energy、teleport/reset与CPU/GPU differential合同。

### M4 · GPU Resource、Cluster 与 Representation

- 建rest/deformed/current/previous typed bundles、dynamic pool、fence、bounds、macro group、cluster与indirect work。
- 完成strands/cards/meshes LOD artifact、stream chunks、atomic install/retire与device-loss恢复。

### M5 · Visibility、Coverage、Composition 与 Temporal

- 实现strand raster、coverage/node visibility、tile/compaction、overflow/degrade与scene composition。
- depth/velocity/TAA/motion blur/DOF/fog消费同一generation，完成representation transition parity。

### M6 · Hair BSDF、Shadow、Environment 与 RT/GI

- 实现typed Hair material、R/TT/TRT、dual/multiple scattering、energy/IBL与LUT qualification。
- 完成deep shadow/transmittance、environment/voxel、RT geometry、GI/path tracing与cross-path parity。

### M7 · LOD、Streaming 与 Scalability

- 实现per-group representation LOD、continuous curve/point LOD、hysteresis和history migration。
- 完成global budget、platform matrix、residency/pressure、multi-view/XR/capture与telemetry。

### M8 · Editor 与 Product Integration

- Editor只消费Runtime source/compiler/binding/preview/debug/transaction合同，交付Groom、binding、cache、cards与LOD工具链。
- 交付short/long/curly hair、fur、beard、braid场景的create/import/save/reopen/play/export/capture证据链。

### M9 · Reliability 与性能超越门

- 完成fault/fuzz/OOM/device loss/provider crash/unload、长时间soak与规模矩阵。
- 同硬件、同source、同镜头/灯光、同画质与同输入对比Unreal，领先声明由raw receipt复算。

## 16. 资格门当前状态

| Gate | 状态 | 当前判定 |
|---:|---|---|
| 1 | Fail | 无Hair source，无法对NaN、空curve、bad knots/width/group/ID给typed diagnostic |
| 2 | Fail | 无group/strand/point/guide/material/LOD stable ID与roundtrip |
| 3 | Fail | 无coordinate、unit、width、basis/curve type与attribute cardinality合同 |
| 4 | Fail | 无Hair compiler/artifact/digest |
| 5 | Fail | 无Hair importer，因此没有cancel/OOM/malformed/provider-unload半成品门 |
| 6 | Fail | 通用version存在，但无source/binding/cache/representation独立version与migration矩阵 |
| 7 | Fail | 无binding target generation、compatibility与stale rejection |
| 8 | Fail | 无root barycentric、guide weight或render interpolation validator |
| 9 | Partial | catalog/App未虚报Hair，但Scene不能保留Hair source，也无统一Unsupported receipt |
| 10 | Fail | 无instance ticket、world generation与唯一终态 |
| 11 | Fail | 无Hair mutable state，无法证明world/preview/PIE隔离 |
| 12 | Partial | Animation与Render phase底座存在，但Hair root/cache/sim/interpolation phase为空 |
| 13 | Fail | 无teleport/reset/origin/cache-seek/cut epoch与previous/history政策 |
| 14 | Fail | 无deterministic CPU oracle与state/output digest |
| 15 | Fail | 无stretch/bend/twist/root analytic/golden收敛门 |
| 16 | Fail | rigid query存在，但无Hair self/body/world/high-speed/overlap collision |
| 17 | Fail | 无WindField generation、sampling与stale policy |
| 18 | Fail | fixed clock存在，但无Hair substep/iteration overload degrade |
| 19 | Fail | 无Groom cache录制/seek/loop/reset/version/error门 |
| 20 | Fail | 无cache chunk cancel/unmap/eviction与in-flight drain |
| 21 | Fail | 无Hair rest/deformed/current/previous resource、generation与fence |
| 22 | Fail | 只有静态source bounds，无动态/reduction/readback fallback |
| 23 | Fail | 无Hair macro group/cluster curve-point count/indirect overflow receipt |
| 24 | Fail | 无strand raster subpixel/near-clip/tangent/MSAA coverage测试 |
| 25 | Fail | generic OIT容量不构成Hair visibility node capacity政策 |
| 26 | Fail | OIT不是批准的Hair cards fallback，fragment loss与precision无产品门 |
| 27 | Fail | 无Hair depth/composition与opaque/fog/DOF/post顺序合同 |
| 28 | Partial | skin/Morph velocity已有previous前置，但无Hair LOD/reset/missing-previous语义 |
| 29 | Fail | 无strands/cards/meshes silhouette/coverage/energy/shadow parity |
| 30 | Fail | 无Hair BSDF finite/energy/reference numerical golden |
| 31 | Fail | 无melanin/absorption/dye/cuticle/two-roughness单位与texture override |
| 32 | Fail | 无R/TT/TRT和multiple scattering跨direct/IBL/raster/RT能量门 |
| 33 | Fail | 无deep shadow/transmittance light/macro-group budget与stale/overflow政策 |
| 34 | Fail | 无environment/voxel data、generation、resolution与pressure degrade |
| 35 | Fail | 无curve/card/tube BLAS、any-hit、refit/rebuild与fallback |
| 36 | Fail | 无Hair multi-view/stereo/capture simulation/visibility/history政策 |
| 37 | Fail | 无1/10/100/1000 component与批准curve规模global budget证据 |
| 38 | Fail | 无Hair debug snapshot与零reader trace成本证明 |
| 39 | Fail | 无short/long/curly/fur/beard/braid source、oracle、pixel capture与soak |
| 40 | Fail | 无同口径visual/error/CPU/GPU/RSS/VRAM/I/O/stutter raw benchmark |

## 17. Finding 到里程碑映射

| Finding | 主里程碑 |
|---|---|
| HAIR-P1-001..012 | M0-M1 |
| HAIR-P1-013..024 | M2-M3 |
| HAIR-P1-025..036 | M3 |
| HAIR-P1-037..048 | M4-M5 |
| HAIR-P1-049..060 | M6 |
| HAIR-P1-061..072 | M7-M9 |
| HAIR-P2-001..016 | 对应P1与资格门完成后独立立项，不得提前并入MVP |

## 18. 禁止的临时修补

1. 禁止给`MeshRenderer`增加`hair: bool`和几个density/roughness字段后宣称Hair完成。
2. 禁止把Mesh custom attribute、line list、particle trail、shell/fins或Morph当作Groom source长期schema。
3. 禁止把generic anisotropic GGX称为Marschner/Hair BSDF，或把普通alpha blend/OIT称为strand visibility。
4. 禁止用固定每像素4/8/32层且静默drop overflow的OIT作为dense Hair主路径。
5. 禁止source、binding、simulation/cache、cards、strands与meshes各自维护不同group/strand identity。
6. 禁止CPU、GPU、Editor preview与cache playback各自实现不同root/interpolation/constraint语义。
7. 禁止Hair owner私建第二套skeleton truth、rigid world、clock、weather、material compiler或streamer。
8. 禁止每帧重建/上传完整curve/cards buffer、同步readback bounds或无fence销毁in-flight资源。
9. 禁止depth、visibility、shadow、velocity、RT和GI读取不同代deformation/binding。
10. 禁止cards fallback无条件覆盖Unsupported；fallback必须带画质、成本、原因和终态receipt。
11. 禁止把Unity HDRP Hair shader当成完整Groom系统参考，也禁止逐字复制Unreal类型/默认值。
12. 禁止在没有同source、镜头/灯光、画质、硬件和raw receipt时宣称表现或性能超过Unreal。

## 19. 实施前重查清单

1. 重导selected manifest并重算本篇指纹；任何变化先标记报告stale再评估finding。
2. 重跑production exact identifier、ResourceKind、Scene、RenderFeature、importer capability与tests查询，确认没有新Hair owner并入。
3. 取得Runtime04/05/08A/08C/09B/09C/09D/09E/09F3/09G2/09H1/22/23/24/28/31/85/86/91与Editor32/38 owner确认。
4. 先批准source/binding/compiler/CPU oracle，再选择GPU solver、visibility和BSDF实现；不得由现成shader或依赖便利性倒推架构。
5. 首个RED切片覆盖capability truth、source roundtrip、malformed validation、deterministic artifact与CPU guide oracle。
6. 动态lane按Windows优先，先core/compiler/headless，再WGPU产品场景、GPU capture、fault/soak与跨引擎benchmark。

## 20. 本轮产出边界

本轮只新增静态review与分层重构计划，没有修改production Runtime、Editor、Interface、Plugin、App代码或tests，没有运行Cargo或WGPU。报告不表示Hair/Groom已经可用，也不授权从P2高级能力开工；实现必须从M0 truth/owner与M1 source/compiler开始，以stable source/binding identity、deterministic CPU oracle、generation-qualified GPU output和真实产品证据逐层收敛。
