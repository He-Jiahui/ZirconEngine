---
title: Runtime Cloth、Fabric、Soft Body、Garment、Simulation、Collision、Deformation、Rendering、Wind、LOD、Scalability、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime144
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/core/framework/physics
  - zircon_plugins/physics/runtime/src
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs
  - zircon_runtime/src/core/framework/render/material/lighting_model.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src/entry
  - zircon_editor/src/core/asset/type_registry
  - examples/woc/native/apps/woc_client/src/shell/class_catalog.rs
tests:
  - zircon_editor/src/tests/editor_asset_type_registry/builtins.rs
  - zircon_runtime/src/asset/tests/assets/mesh/morph_targets.rs
  - zircon_runtime/src/core/framework/physics/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph.rs
  - zircon_plugins/physics/runtime/src/backend/tests/jolt_contract.rs
  - zircon_plugins/physics/runtime/src/skeletal/tests.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/31-cloth-fabric-soft-body-garment-simulation-collision-deformation-rendering-wind-lod-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99b-runtime-temporal-aa-velocity-history-dynamic-resolution-upscaling-reconstruction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zl-runtime-animation-skeleton-clip-pose-graph-state-machine-layer-mask-blend-ik-root-motion-event-extract-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zm-runtime-physics-world-body-shape-collider-material-joint-query-contact-trigger-fixed-step-jolt-character-controller-vehicle-ragdoll-debug-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/ChaosCloth/Source
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAsset/Source
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAssetDataflowNodes/Source
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAssetEditorCore/Source
  - dev/UnrealEngine/Engine/Source/Runtime/ClothingSystemRuntimeInterface
  - dev/UnrealEngine/Engine/Source/Runtime/ClothingSystemRuntimeCommon
  - dev/UnrealEngine/Engine/Source/Editor/ClothPainter
  - dev/godot/scene/3d/physics/soft_body_3d.cpp
  - dev/godot/modules/godot_physics_3d/godot_soft_body_3d.cpp
  - dev/godot/modules/jolt_physics/objects/jolt_soft_body_3d.cpp
  - dev/godot/thirdparty/jolt_physics/Jolt/Physics/SoftBody
  - dev/bevy/crates/bevy_mesh/src/skinning.rs
  - dev/bevy/crates/bevy_mesh/src/morph.rs
  - dev/bevy/crates/bevy_pbr/src/render/morph.rs
  - dev/Fyrox/fyrox-impl/src/resource/gltf/surface.rs
  - dev/Fyrox/fyrox-impl/src/scene/mesh
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Fabric
---

# Runtime Cloth、Fabric、Soft Body、Garment 与 Product Integration 当前源码工程化差距

## 1. 结论

当前 Zircon 仍然没有 Cloth、Garment 或通用 Soft Body 运行时产品。对 `zircon_runtime`、`zircon_runtime_interface`、`zircon_editor`、`zircon_app`、`zircon_plugins` 与 `examples` 中排除 tests/test_sources/benches/target 和测试文件名后的 **12,597 个 production Rust 文件**执行精确词边界扫描，26 条命中分布在 11 个文件，但逐条分类后全部为非领域实现：3 条是 WOC 护甲本地化键，5 条是 shader import path 夹具，其余 18 条是 render、IBL、virtual geometry 或 parity 代码中的一般“seam”措辞。没有 Cloth/SoftBody/Garment/Fabric source、resource kind、component、runtime instance、solver、provider、artifact、deformation output、render feature、Editor toolkit 或产品场景。

仓内已有的 mesh attribute/topology validation、skin/Morph、current/previous deformation、velocity、fixed-step Physics、rigid query/filter/material/contact、skeletal collider/Ragdoll、mesh LOD、resource residency与 Standard PBR 都是真实底座。它们只能证明未来不必从零重写通用支撑，不能把普通 mesh、Morph、Ragdoll、double-sided material、WPO 或 WOC 的 `cloth` 文本升级为 Cloth capability。尤其是当前 Jolt bridge 只拥有 Body/Shape/Constraint，native binding 对 `SoftBody`/Cloth 的精确扫描为零；`GpuMeshResource` 仍是静态 vertex/index buffer，Material lighting model也没有 Fabric/Sheen/Charlie owner。

本篇不重复创建 Physics、Animation、Temporal、Material 与 Editor 上层已经拥有的问题，也没有 catalog/App/Editor false-ready 需要新设产品真值 P0，因此登记 **0 项新的 Cloth-owned P0**。历史 64 项 P1 按当前 bytes 重判为 **51 Open / 13 Partial / 0 Closed**，14 项 P2 全部 Open；36 项资格门为 **33 Fail / 3 Partial / 0 Pass**。Partial 只表示相邻通用 owner 提供了可复用前置，不表示 Cloth 产品链已启动。目标必须硬切到：

```text
Cloth/Garment Source
  -> deterministic Cloth Compiler
  -> Simulation Topology + Fabric/Constraint/Map + Render Mapping Artifact
  -> generation-qualified per-World ClothRuntimeInstance
  -> admitted CPU/GPU Solver Provider
  -> ClothDeformationOutput(current/previous geometry + bounds + fence)
  -> Render / Physics / Animation / Wind / Cache / Replay adapters
  -> runtime-backed Editor authoring、preview 与 qualification receipts
```

## 2. 审查边界、方法与 currentness

### 2.1 冻结范围

冻结基线为 `main@94f86015d0da980d6c93ef3cf3fcd9d759d0e477` 的当前 working bytes。冻结时共享工作树已有 3,233 个 tracked changes、4,414 个含 untracked changes（包含本篇新增报告与索引）；本文不归因、不覆盖、不回退任何既有改动。用户已明确暂不优化 tooling，本轮没有扫描或规划未来将迁移到 Rust 的 tooling 实现。基线前进后重算的525个Zircon选择集与522个参考文件指纹和初次冻结完全一致，新增3个production Rust文件均未命中领域词。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树指纹 |
|---|---:|---|
| Source、Scene 与 Editor carrier | **35 / 4,144 / 3,731 / 136,255 / 3 / 0** | `90593998f66398ff542c18d5780d953b0a7be5d22de32961124de8aac49e02a5` |
| Animation、Physics 与 Jolt 底座 | **154 / 18,056 / 16,556 / 611,361 / 76 / 0** | `12ca26f62b9697083b4becf62aaf90a864f4ae8ac1fd05933f034af6129d6e23` |
| Render、Deformation 与 Material 底座 | **153 / 35,486 / 32,561 / 1,322,641 / 341 / 0** | `9e5da5d1f72d9bf273a5076c7efc5b11958e2a18a230e51954538ba6e90b301b` |
| Catalog、App、tests 与产品证据 | **186 / 29,388 / 27,023 / 1,079,569 / 459 / 0** | `7224de8a6696daa0eaa4526fca8b1f44bc0d6d9423aee395171cc8c0b1883d3e` |
| Zircon selected union | **525 / 85,674 / 78,560 / 3,102,643 / 853 / 0** | `d895691b0a9f114d2c8792a712712a470c037c6f72a798a7d19b9afce6441c30` |
| Unreal Cloth family | **488 / 108,366 / 91,887 / 4,516,943 / 0 / 0** | `00d6621caa03b4a7ce29cea6ff864c57c29a97661e586b42c0c4316c595df331` |
| Godot/Jolt SoftBody family | **18 / 8,321 / 6,850 / 328,764 / 0 / 0** | `5b11067b5f7e9d25d0af209e547abbe60c939d04f0bea815f0daf6b2b036ea7a` |
| Bevy/Fyrox skin 与 Morph 对照 | **8 / 9,678 / 8,835 / 369,629 / 5 / 0** | `2c99a3ee9398b522bbc07c82ceef3f8758c56f93c7f5217ba5dc1b39d7e65a0e` |
| Unity HDRP Fabric | **8 / 1,770 / 1,478 / 76,536 / 0 / 0** | `642278d1f94d069c405603d133a67d74503bbb608be71d7011f90402a565860a` |
| reference selected union | **522 / 128,135 / 109,050 / 5,291,872 / 5 / 0** | `f10d51d299a792dad580400d1dfab215c543d5c74da1c5c915b5dd3586e8edac` |
| all selected | **1,047 / 213,809 / 187,610 / 8,394,515 / 858 / 0** | `3b00cadd9c70ab6e1fce7061517e53b32ca8f95ae9538cdd6f453724b04744c8` |

指纹算法为：repository-relative path 转 `/` 并小写排序；每个文件取当前 bytes 的 SHA-256；聚合输入为 `path + NUL + lowercase(file_sha256) + LF`，再取 SHA-256。tests 统计 Rust `#[test]`/`#[tokio::test]`，参考树按实际文件 bytes 计算，不假设 Git tracking 状态。

### 2.2 纵向扫描链

本轮逐层核对：resource kind/source schema -> mesh attributes/topology/import/compiler/artifact -> Scene persistence/prefab/clone -> catalog/App/provider selection -> per-World instance/lifecycle/fixed clock -> solver state/constraints/determinism -> skeleton/kinematic collider/world/self collision -> wind/aerodynamics/gameplay force -> render-to-sim mapping/dynamic output/bounds -> depth/GBuffer/forward/shadow/velocity/RT/GI -> Fabric BRDF/material residency -> LOD/scalability/cache/replay -> Editor registry/toolkit/paint/preview -> product scenario/fault/benchmark。没有发现 plugin facade、inline test、feature descriptor或示例之后隐藏的第二套 Cloth production owner。

### 2.3 证据等级与执行限制

本篇达到 E3 source-level review。没有运行 Cargo、Jolt、WGPU、App、Editor、PIE、asset cook、save/reopen、GPU capture、fault/scale/soak或竞争 benchmark，因为当前没有 Cloth source/component/provider/pass 可执行；运行 skin/Morph、Ragdoll、PBR 或 rigid Physics 测试不能提高 Cloth 结论。实施前必须重取指纹，从 capability truth、source roundtrip、compiler/CPU oracle RED 证据开始。

`model_tier=5.6-sol | thinking_depth=Extra High | selection_reason=cloth simulation rendering physics editor product and five-engine current-source rejudgment | primary_session=self`

## 3. 当前产品链事实

### 3.1 Source、Resource、Scene 与 Product Truth

1. `ResourceKind` 有 Model、Mesh、Material、Terrain、Prefab、Animation 等类型，没有 Cloth、Garment、Fabric、SoftBody、ClothCache 或 ClothProfile。
2. `SceneEntityAsset` 可持久化 mesh、rigid body、collider、joint、animation、terrain、tilemap、prefab 与 scripts，没有 Cloth component，也没有对未知 typed plugin component 的无损 carrier。
3. `MeshRenderer` 只有 model/mesh/material、render queue、Morph weights、primitives、LOD、material overrides、tint/alpha；没有 Cloth artifact、solver profile、collision source、wind binding、teleport/reset、cache或 simulation blend。
4. 首方 runtime catalog、App entry/composition和 Editor built-in asset registry均没有 Cloth provider/package/capability。Editor registry当前列出的 26 个 ResourceKind和 UI/Animation toolkit都不含 Cloth。
5. 产品代码没有把 Cloth 标为 Ready/Executed，因此本篇不新建 false-ready P0。WOC 的三条 `classDetails.armor.cloth` 只是护甲分类本地化键，不是 simulation、material或runtime capability。

### 3.2 Mesh、Animation 与 Deformation 前置

1. Mesh asset已具 typed vertex attributes、topology/index validation、skin、Morph、bounds及 source normal/tangent生成，这些可作为 Cloth compiler输入前置。
2. Animation pose output仍只表达 source、active state与 bones；没有 Cloth pre/post pose packet、root/teleport epoch、attachment generation或 simulation phase receipt。
3. GPU Scene和 mesh renderer已经维护 current/previous skin palette与 Morph weights/source，velocity path也有真实 GPU消费。这是比历史基线更强的 previous-deformation前置，不能等同于 Cloth output。
4. CPU/GPU skin和Morph始终从静态 source geometry生成；没有 sim topology、render mapping、dynamic position/normal/tangent owner，也没有 output generation/fence。
5. Mesh LOD存在，但没有 simulation LOD、state transfer、constraint remap、hysteresis或 Cloth budget admission。

### 3.3 Physics、Collision 与 Solver

1. Physics backend contract只抽象 shape/body/constraint、commands、step、active rigid states、queries与events；backend handle也只有 Body/Shape/Constraint。
2. collider只有 Box/Sphere/Capsule/Cylinder/Convex/TriangleMesh/HeightField/Compound，没有 SoftBody shape、particle/face/link、cloth thickness或 deformable contact。
3. Jolt bridge对 `SoftBody`、Cloth、Garment、XPBD/PBD的生产标识扫描为零，没有 soft-body creation settings、motion properties、vertex state、contact listener或 render update。
4. fixed clock、solver groups、raycast/overlap/shape cast、collision layers/masks、physics material/contact和 skeletal collider/Ragdoll是真实通用底座；它们没有被组装为 Cloth solver、kinematic collider snapshot、world collision adapter或 two-way coupling。
5. 没有 stretch/shear/bend/area/tether/seam/volume constraints，没有 self collision、adjacency exclusion、CCD、initial penetration recovery、bounded pair scratch、deterministic CPU oracle或 solver provider admission。

### 3.4 Render、Fabric 与 Temporal

1. `BuiltinRenderFeature`和`RenderFrameExtract`没有 Cloth instance、simulation output、dynamic bounds、deformation sideband或 Cloth feature。
2. `GpuMeshResource`只拥有 vertex/index buffers、count、signature、wire segments与静态 bounds；upload usage仍以 `VERTEX`/`INDEX`为核心，没有 Cloth dynamic surface pool、STORAGE/COPY update、fence retirement或 LKG output。
3. mesh velocity已有 current/previous skin/Morph输入，但 WPO/procedural/Cloth previous deformation统一 ABI仍由 Runtime101-P1-17保持 Open；缺失时无法区分静止、unsupported、reset与 cut。
4. Standard PBR已有 clearcoat、anisotropy、transmission、thickness、IOR与 attenuation；lighting model只有 Pbr、BlinnPhong、Unlit、Custom，没有 Fabric、Sheen/Charlie/Fuzz、Cotton/Wool/Silk或 fabric energy/IBL qualification。
5. 没有 dynamic bounds、normal/tangent重建、depth/shadow/velocity同代消费、thin/double-sided Cloth policy、RT BLAS refit、GI/SDF/card invalidation或 virtual geometry动态形变策略。

### 3.5 Editor、Authoring 与 Qualification

1. 没有 Cloth/Garment/SoftBody asset factory、importer、compiler operation、document、toolkit、Dataflow graph或 simulation preview。
2. 没有 panel/seam authoring、2D/3D pattern inspection、weight-map paint/smooth/erase、pin/backstop/max-distance/tether可视化、collision probe或 LOD mapping调试。
3. 没有 cape/skirt/flag/layered garment产品场景，没有 save/reopen/play/export/cache/replay证据，也没有 Cloth profiler、capture、fault/fuzz、scale/soak和 cross-engine benchmark receipt。
4. 因领域完全缺失，Editor不应先造一套私有 solver或 JSON document；必须消费 Runtime-owned schema/compiler/artifact/preview output。

## 4. 必须保留的真实底座

1. 保留 MeshAsset attribute/topology/skin/Morph/bounds validation，把 Cloth source/compiler接入同一 asset dependency、artifact、residency和 install语义。
2. 保留 Physics fixed clock、generation handle、query/filter/material/contact与 skeletal collider基础；Cloth通过 adapter消费，不私建第二个 rigid world、broad phase或 clock。
3. 保留 Animation pose generation和 current/previous skin/Morph数据，把 Cloth pre-pose/post-pose/root/teleport epoch纳入统一 deformation schedule。
4. 保留 RenderFrameExtract、GPU Scene、mesh pass、velocity与 material/pipeline owner；Cloth只发布 generation-qualified deformation packet和 bounds，不让每个pass各自变形。
5. 保留通用 resource residency与 mesh LOD框架，但新增 Cloth artifact bundle、simulation LOD、state migration、budget和 atomic retire。
6. 保留 catalog/App/Editor当前诚实不宣称能力的 product truth；只有 source->runtime->render->Editor闭环和 qualification gate通过后才能提升 capability。

## 5. 五套参考源码给出的工程边界

### 5.1 Unreal Chaos Cloth 是本域主要架构参考

本轮选择的 488 个 Unreal 文件覆盖 ChaosCloth、ChaosClothAsset、Dataflow nodes、Editor core、Clothing runtime interface/common与 ClothPainter。`ClothCollection`及 facade明确分离 solver config、Fabric、seam stitches、2D/3D sim topology、render topology、bones、tethers、render-to-sim barycentric mapping、skin blend、Morph/accessory数据；`ClothComponent`、simulation model/proxy与 solver又分离 per-LOD artifact、parallel init/tick、suspend/resume、teleport/reset、cached/dynamic bounds、finite fail-safe和 cache state。

约束层覆盖 PBD/XPBD stretch、bend、axial、volume、long-range attachment、anim drive、self collision、kinematic triangle和 soft-body collision；Dataflow/Editor提供 self-collision、tether、aerodynamics节点及异步 weight-map paint、dynamic mesh preview、selection、paint/smooth/erase和 node回写。Zircon应吸收 source/artifact/runtime/proxy/Editor分层、generation/task收口和 sim/render mapping，不复制 UObject、宏或默认参数。

### 5.2 Godot/Jolt 给出最小 SoftBody 产品下限

Godot `SoftBody3D`仍有 mesh、collision layer/mask、pin/attachment、precision、mass、stiffness、pressure、damping/drag、server RID、dynamic render mesh update与 custom AABB。Godot Physics backend维护 node/link/face、current/previous/test position、inverse mass、pinned vertex、bending constraint和 bounds；Jolt SoftBody进一步具 vertices/faces/edges/dihedral bend/volume/skinned/max-distance/backstop/LRA、parallel constraint groups、collision、predicted bounds、current/previous skin state与 pressure/iteration。

这说明即使先做 generic SoftBody MVP，也必须有 typed identity、backend lifecycle、topology/constraints、collision、vertex output和 render同步。Zircon当前 rigid-only Jolt bridge低于这个最小闭环。

### 5.3 Bevy 与 Fyrox 是诚实的负证据

当前 Bevy crates 的 1,408 个生产源码文件和 Fyrox相关 654 个源码文件没有 engine-owned Cloth/SoftBody/Garment/Fabric模块。Bevy skin/Morph仍展示 component/extract、storage/uniform fallback、current/previous buffers和 previous joint matrices；Fyrox importer/scene mesh展示 skin/Morph载入与渲染前置。应复用这些基础思想，但不能把参考树的缺失变成 Zircon 降级许可，也不能把普通 deformation命名为 Cloth。

### 5.4 Unity Graphics 只证明 Fabric shading，不证明 Cloth solver

HDRP Fabric的 8 个实质文件区分 Cotton/Wool与 Silk，覆盖 Charlie/sheen、anisotropy、SSS/transmission、preintegrated FGD/IBL、raster、ray tracing与 path tracing。当前 Graphics package没有 Cloth/SoftBody/Garment solver owner，因此它只能约束 Fabric光学与多render-path一致性。Simulation topology、collision、solver和 determinism必须以 Unreal/Godot/Jolt为证据，Fabric BRDF和 Cloth solver必须分别资格化。

## 6. 目标架构与唯一 Owner

```text
ClothSourceAsset / SoftBodySourceAsset
  -> schema migration + diagnostics
  -> ClothCompiler
       -> SimTopologyArtifact
       -> FabricConstraintMapArtifact
       -> RenderDeformerMappingArtifact
       -> CollisionLodCacheMetadata
  -> ClothRuntimeService (per World, accepted generation)
       -> SolverProvider admission
       -> ClothRuntimeInstance lifecycle
       -> Physics/Animation/Wind snapshots
       -> current/previous state + cache/replay
  -> ClothDeformationOutput
       -> RenderScene/GPU Scene/Visibility/Velocity/RT/GI
  -> Editor shared document/compiler/runtime preview
```

| 领域 | 唯一 owner | 本篇边界 |
|---|---|---|
| Resource/artifact/residency | Runtime Resource owner | Cloth source、dependency、artifact bundle、digest、install/retire receipt |
| Scene/World lifecycle | Runtime Scene/World owner | typed component、per-World instance、generation与save/reopen |
| Fixed clock/rigid query | Runtime99zm / Time owner | 提供 qualified tick、collider/query snapshot；不拥有 Cloth solver |
| Skeleton/pose/deformation phase | Runtime99zl | 提供 pose/root/teleport generation；消费 Cloth phase receipt |
| Cloth/SoftBody domain | 新 Runtime Cloth owner | topology/compiler/solver/instance/output/cache与 domain diagnostics |
| Renderer/GPU Scene/visibility | Runtime Render owner | 消费同一 deformation generation、bounds和 fence |
| Fabric material/PSO | Runtime91及 surface-lighting owner | Fabric BRDF、variant、IBL/RT一致性；不拥有 solver |
| Temporal/history | Runtime99b | current/previous、reset/cut epoch、reactive与 velocity policy |
| Wind/weather | Weather runtime owner | 发布 WindField generation；Cloth只做 aerodynamic sampling |
| Editor authoring | 后续 Editor Cloth owner | shared source/compiler、paint、preview、transaction与 diagnostics |

Cloth与 generic SoftBody可以共享 solver kernel、constraint primitive、collision adapter和 provider admission，但必须拥有不同 source profile与产品语义。Garment需要2D/3D pattern、seam、Fabric、skin/render mapping；generic SoftBody可能使用 surface或 tetra topology。禁止用一个巨大 `SoftBodyComponent { json }` 混合所有合同。

## 7. P1：Source、Schema、Compiler 与 Artifact

| ID | 状态 | 当前源码证据与必须重构 |
|---|---|---|
| CLOTH-P1-001 | Open | `ResourceKind`无 Cloth/Garment/SoftBody；新增稳定 kind、marker、schema/version、dependency与 capability owner |
| CLOTH-P1-002 | Open | 无独立 authoring source；建立`ClothSourceAsset`并与 derived runtime artifact硬分离 |
| CLOTH-P1-003 | Open | 只有 render MeshAsset；显式建立2D pattern、3D sim mesh、render mesh三拓扑及映射 |
| CLOTH-P1-004 | Open | 无 panel/pattern/vertex/face/seam身份；使用持久stable ID，数组下标不得成为长期身份 |
| CLOTH-P1-005 | Open | 无 seam/stitch schema；定义端点、方向、rest、weld/stitch与 validation |
| CLOTH-P1-006 | Open | PBR参数不是 Fabric profile；定义密度、stretch/shear/bend/buckle、damping、friction、thickness与单位 |
| CLOTH-P1-007 | Open | 无 typed weight map；max distance/backstop/anim drive/self-collision/tether使用stable map ID和范围 |
| CLOTH-P1-008 | Open | 无 pin/attachment；定义 bone/socket/vertex selection、space、weight、break与 missing-target policy |
| CLOTH-P1-009 | Open | 无 Cloth collision source；类型化 physics asset/entity/layer/mask、primitive filter、thickness与 owner generation |
| CLOTH-P1-010 | Open | Mesh LOD不是 Cloth LOD source；每LOD独立 sim/render topology、transition map、quality与 fallback |
| CLOTH-P1-011 | Partial | Mesh已有 typed attribute/topology/length validation，但无 Cloth domain语义；compiler只导入批准属性并给 stable diagnostic |
| CLOTH-P1-012 | Open | 无 Cloth compiler/DDC；确定性编译 topology、constraints、maps、acceleration、mapping、digest、migration与 LKG |

## 8. P1：Runtime Instance、Solver、Time 与 Determinism

| ID | 状态 | 当前源码证据与必须重构 |
|---|---|---|
| CLOTH-P1-013 | Open | Scene/World无 Cloth component/instance；建立 artifact handle、owner、generation、state、blend、quality与 lifecycle |
| CLOTH-P1-014 | Open | 无 Cloth ticket；Requested/Preparing/Active/Suspended/Retiring/Failed/Cancelled必须唯一终态 |
| CLOTH-P1-015 | Open | Physics provider contract无 soft body能力；定义CPU/GPU limits、determinism、memory、async、fallback admission |
| CLOTH-P1-016 | Open | 无 deterministic CPU oracle；先交付可复算 baseline，供 compiler、GPU differential与 headless使用 |
| CLOTH-P1-017 | Open | 无 particle state；current/previous position、velocity、inverse mass、normal及 scratch按 generation管理 |
| CLOTH-P1-018 | Open | 无 stretch/shear；采用批准的XPBD/等价模型，明确 compliance、iteration、warm start与误差门 |
| CLOTH-P1-019 | Open | 无 bend/buckle/area；编译邻接/rest state，退化三角形必须拒绝或 typed repair |
| CLOTH-P1-020 | Open | 无 tether/LRA；从 kinematic region构建 bounded tethers并可诊断 stretch drift |
| CLOTH-P1-021 | Open | 无 seam执行；seam顺序、重复、断裂、跨LOD映射和 stale topology必须确定 |
| CLOTH-P1-022 | Partial | Physics已有 fixed_hz/max_substeps和 solver groups，但无 Cloth policy；建立 cloth-specific substep/iteration budget与 overload degrade |
| CLOTH-P1-023 | Open | rigid teleport command不定义 Cloth语义；区分 preserve pose/velocity、reset、origin shift、large jump和 cut epoch |
| CLOTH-P1-024 | Open | 无 Cloth pause/suspend/visibility策略；明确 freeze、sleep、offscreen tick、resume warmup与 network authority |
| CLOTH-P1-025 | Open | 无 Cloth async task identity；task必须绑定 world/instance/artifact/tick generation，cancel/stale/panic不发布 |
| CLOTH-P1-026 | Open | 无 particle/constraint scratch budget；池化并在OOM时产生 typed degrade/failure |
| CLOTH-P1-027 | Open | 无 Cloth finite/energy fail-safe；建立 strain/energy阈值、rollback/LKG和 disable reason |
| CLOTH-P1-028 | Open | 无 Cloth cache/replay；记录 artifact、tick、pose、fields、state/output digest，支持 seek/reset/differential |

## 9. P1：Collision、Animation、Wind 与 Gameplay Integration

| ID | 状态 | 当前源码证据与必须重构 |
|---|---|---|
| CLOTH-P1-029 | Partial | skeletal physics已有 collider/Ragdoll前置，但无 Cloth snapshot；从 qualified skeleton/physics generation提取 sphere/capsule/convex |
| CLOTH-P1-030 | Partial | Physics已有 ray/overlap/shape query与 world状态，但无 Cloth broad-phase adapter；禁止每particle全场扫描 |
| CLOTH-P1-031 | Open | 无 self collision；建立 spatial acceleration、adjacency exclusion、thickness/friction与 pair budget |
| CLOTH-P1-032 | Open | 无 deformable CCD；高速骨骼/顶点使用 sweep/TOI或明确 quality fallback |
| CLOTH-P1-033 | Open | 无 initial overlap恢复；提供 pushout、iteration cap、stuck diagnostic与 reset policy |
| CLOTH-P1-034 | Partial | Physics已有 layer/mask/filter，但无 Cloth channels；按 cloth/world/self/character/accessory与 owner generation解析 |
| CLOTH-P1-035 | Partial | Physics已有 material/contact基础，但无 deformable contact persistence；定义摩擦、warm contact、overflow与 event边界 |
| CLOTH-P1-036 | Partial | Animation/Physics/Render已有阶段前置，但没有 Cloth phase；固定 pre-skin -> physics snapshot -> cloth -> render extract顺序 |
| CLOTH-P1-037 | Partial | skin/Morph和 pose generation存在，但无 skin-to-sim/anim drive；reference pose、max-distance与 map必须同artifact计算 |
| CLOTH-P1-038 | Open | 无 Cloth反馈；默认单向，需要反馈时走显式 force/pose adapter与 authority policy |
| CLOTH-P1-039 | Open | 无 WindField输入；消费 Weather owner generation并定义 space、sampling、stale与 fallback |
| CLOTH-P1-040 | Open | 无 aerodynamic模型；drag/lift/pressure按 triangle normal/area/relative velocity并分 quality tier |
| CLOTH-P1-041 | Open | 无 gameplay field ingress；typed force/impulse/explosion必须有 owner、tick、capacity、drop reason与 receipt |
| CLOTH-P1-042 | Open | 无 network policy；区分 server sim、client cosmetic、replicated cache、correction/reset与带宽预算 |

## 10. P1：Deformation Output、Rendering、Fabric 与 Temporal

| ID | 状态 | 当前源码证据与必须重构 |
|---|---|---|
| CLOTH-P1-043 | Open | 无 render-to-sim deformer；使用 barycentric/skin blend mapping生成 render position/normal/tangent |
| CLOTH-P1-044 | Partial | renderer已有 current/previous skin/Morph generation前置，但无 Cloth output；发布 buffers、bounds、tick/artifact/output generation与 fence |
| CLOTH-P1-045 | Open | `GpuMeshResource`是静态 vertex/index owner；建立 dynamic surface pool、STORAGE/VERTEX/COPY与 fence retire |
| CLOTH-P1-046 | Open | 无 CPU/GPU solver/output；同artifact/input必须误差等价，unsupported走明确 CPU/skin fallback |
| CLOTH-P1-047 | Open | bounds只来自静态 source；建立 CPU conservative/GPU reduction/readback分层与 stale fallback |
| CLOTH-P1-048 | Partial | source mesh可生成 normal/tangent，但无动态重建；kernel需定义 degenerate policy、cost与 previous一致性 |
| CLOTH-P1-049 | Open | 无 Cloth geometry owner；depth/GBuffer/forward必须消费同一 output generation |
| CLOTH-P1-050 | Partial | skin/Morph velocity已有 current/previous GPU路径，但无 Cloth/reset epoch；统一 shadow、velocity、TAA/motion blur |
| CLOTH-P1-051 | Open | Standard PBR无 Fabric model；建立 cotton/wool/silk批准模型、sheen/fuzz/anisotropy、energy与 IBL一致性 |
| CLOTH-P1-052 | Open | 无 Cloth thin/double-sided policy；定义 backface normal、thickness、shadow、SSS/transmission与 any-hit |
| CLOTH-P1-053 | Open | 无动态 Cloth RT/GI/SDF；BLAS refit/rebuild、card/SDF/lightmap invalidation按 cost显式降级 |
| CLOTH-P1-054 | Open | virtual geometry按静态cluster设计；动态 Cloth必须分离或走批准的 deformation path |
| CLOTH-P1-055 | Partial | 通用 material/texture residency可复用，但无 Cloth atomic bundle；artifact声明依赖，pressure/eviction不得半安装 |
| CLOTH-P1-056 | Open | 无 Cloth multi-view/XR/capture；simulation共享边界与 view-specific cull/history/offline sampling需显式 |

## 11. P1：LOD、Scalability、Diagnostics、Tests 与 Product Qualification

| ID | 状态 | 当前源码证据与必须重构 |
|---|---|---|
| CLOTH-P1-057 | Partial | MeshRenderer已有 render LOD，但无 simulation policy；以screen error、distance、importance、visibility、budget与 hysteresis共同选择 |
| CLOTH-P1-058 | Open | 无 Cloth LOD state；迁移 position/velocity/constraints/render mapping，禁止爆跳或能量注入 |
| CLOTH-P1-059 | Open | 无 Cloth quality tier；substep、iteration、collision、normal、RT按profile分层并输出 degrade reason |
| CLOTH-P1-060 | Open | 无 Cloth global admission；按 particle/constraint/pair/bytes/CPU/GPU time选择 active/frozen/skin fallback |
| CLOTH-P1-061 | Open | 无 Cloth telemetry/debug snapshot；记录 active、iterations、error、contacts、memory与CPU/GPU分位 |
| CLOTH-P1-062 | Open | 无 Cloth fault/fuzz；覆盖 malformed topology、NaN、stale、OOM、device loss、provider crash与 unload |
| CLOTH-P1-063 | Open | 无真实产品测试；建立 cape/skirt/flag/layered garment、collision/wind/teleport/LOD数值与像素golden |
| CLOTH-P1-064 | Open | 无同口径竞争基准；记录误差、穿透、稳定性、CPU/GPU、memory、stutter与画质原始 receipt |

## 12. P2：完整性与长期竞争力

| ID | 状态 | 后续能力与前置 |
|---|---|---|
| CLOTH-P2-001 | Open | tearing/fracture与 dynamic topology；先完成stable topology ID、constraint lifecycle、render remap和 network policy |
| CLOTH-P2-002 | Open | sewing/dressing runtime assembly；先稳定 seam compiler、collision、attachment与 transaction |
| CLOTH-P2-003 | Open | multilayer garment/contact ordering；先稳定self/world collision、layer/filter、budget与 deterministic tie-break |
| CLOTH-P2-004 | Open | volumetric/tetrahedral soft body；先批准 surface cloth kernel与 generic soft-body owner边界 |
| CLOTH-P2-005 | Open | GPU async compute solver；先完成 CPU oracle、barrier、fence/output generation与 device loss |
| CLOTH-P2-006 | Open | reduced-order/ML deformation；先有 authoritative fallback、provenance、error bound与 platform admission |
| CLOTH-P2-007 | Open | cloth cache compression/streaming；先完成 schema、seek、error metric、residency与 atomic publication |
| CLOTH-P2-008 | Open | deterministic rollback cloth；先完成 fixed tick、input/state digest、checkpoint与 float policy |
| CLOTH-P2-009 | Open | wetness/ice/burning/damage coupling；先稳定 Water/Weather/Gameplay adapter与 Fabric authority |
| CLOTH-P2-010 | Open | accessibility motion reduction；先区分 gameplay/cosmetic、quality profile与 render fallback |
| CLOTH-P2-011 | Open | large-world partitioned cloth；先完成 origin/rebase、streaming、跨cell attachment与 cache continuity |
| CLOTH-P2-012 | Open | plugin constraint/deformer nodes；先完成 ABI/version/capability/budget/unload sandbox与 compiler extension |
| CLOTH-P2-013 | Open | collaborative garment authoring；先完成stable panel/seam/map ID、semantic merge与 recovery |
| CLOTH-P2-014 | Open | distributed qualification farm；先完成deterministic fixture、artifact digest、raw receipt与差异定位 |

## 13. 分层重构里程碑

| 里程碑 | 交付 | 退出门 |
|---|---|---|
| M0 Truth/Owner | 术语、owner、Unsupported truth、budgets、CPU oracle与 fixtures | 普通Mesh/Morph/Ragdoll/Fabric不再可能被误报为Cloth；capability/catalog/App/Editor一致 |
| M1 Source/Compiler | source schema、stable IDs、patterns/topologies、seam/fabric/maps、deterministic artifact | malformed corpus、migration、byte-identical digest、LKG与 roundtrip通过 |
| M2 Runtime/CPU Solver | Scene component、per-World lifecycle、particle state、XPBD baseline、bounded memory | fixed-input digest可复算；cancel/stale/unload不发布 |
| M3 Animation/Collision/Wind | pose/attachment、kinematic/world/self/CCD、WindField/aerodynamics、force ingress | phase、generation、filter、pair budget和 fault receipts通过 |
| M4 Dynamic Deformation | sim-to-render mapping、current/previous output、bounds、normal/tangent、surface pool | depth/GBuffer/forward/shadow/velocity同代，CPU/GPU parity合格 |
| M5 Fabric/Temporal/RT | Fabric BRDF、thin surface、TAA/motion、BLAS/GI、atomic residency | raster/RT/path visual golden、reset/cut与 device-loss fallback合格 |
| M6 LOD/Scalability/Cache | sim/render LOD、state transfer、quality/update rate、global budget、cache/replay | 1/10/100/1000 instance与seek/replay误差、memory/stutter门通过 |
| M7 Editor/Product | shared document/compiler、paint/preview/debug、cape/skirt/flag/layered场景 | save/reopen/play/export、PIE/runtime parity和 actionable diagnostics通过 |
| M8 Reliability/超越 | fuzz/OOM/crash/unload/soak与同口径竞争 benchmark | 原始receipt可复算；无证据不得宣称超过 Unreal |

## 14. 资格门重判

| Gate | 状态 | 当前判定 |
|---:|---|---|
| 1 | Fail | 没有 Cloth source，无法诊断 NaN/退化/non-manifold/seam/map/fabric/预算 |
| 2 | Fail | 没有 panel/pattern/vertex/face/seam/map/LOD stable ID与 roundtrip |
| 3 | Fail | 没有 Cloth compiler/artifact/digest |
| 4 | Fail | 没有 sim/render topology与 mapping validator |
| 5 | Partial | catalog/App未虚报 provider，但没有可保留的 Cloth source、统一 Unsupported receipt或 Editor truth |
| 6 | Fail | 没有 instance ticket、world generation或 stale-output rejection |
| 7 | Fail | 没有 deterministic CPU oracle |
| 8 | Fail | 没有任何 Cloth constraint与 analytic/golden误差门 |
| 9 | Fail | fixed clock存在，但没有 Cloth overload/degrade/freeze语义 |
| 10 | Fail | rigid pause/teleport不能证明 Cloth pose/velocity/reset/origin policy |
| 11 | Fail | skeletal collider前置存在，但无 Cloth-qualified bone generation snapshot |
| 12 | Fail | query/filter存在，但无 deformable world/self/CCD/overlap recovery |
| 13 | Fail | 无 Cloth broad phase、pair budget、分位与 drop receipt |
| 14 | Partial | Animation/Physics/Render阶段底座存在，但 Cloth phase与跨World隔离缺失 |
| 15 | Fail | 无 WindField generation或 stale policy |
| 16 | Fail | 无 aerodynamic模型与数值门 |
| 17 | Fail | 无 Cloth gameplay force ingress与 teardown receipt |
| 18 | Fail | 无 render mapping、LOD/topology/skin blend连续性 |
| 19 | Fail | 无 Cloth current/previous output、generation、bounds与 fence |
| 20 | Fail | 无 Cloth pass，因此无法证明五类pass同代 |
| 21 | Fail | 只有静态 source bounds，无动态/预测/reduction策略 |
| 22 | Partial | skin/Morph velocity已有 previous状态，但 Cloth/reset/cut/unsupported previous ABI缺失 |
| 23 | Fail | 无 Fabric BRDF与 cotton/wool/silk energy/IBL visual golden |
| 24 | Fail | Fabric与 solver均不存在，无法证明独立启用和 fallback合同 |
| 25 | Fail | 无 deformed Cloth RT/GI/shadow更新或 fallback |
| 26 | Fail | 通用 residency存在，但无 Cloth material/output atomic bundle |
| 27 | Fail | Mesh LOD存在，但无 Cloth state/constraint/mapping迁移 |
| 28 | Fail | 无 Cloth quality profile与成本差异 receipt |
| 29 | Fail | 无 Cloth global budget或 1/10/100/1000 instance证明 |
| 30 | Fail | 无 hidden/offscreen/server/XR/capture simulation/render policy |
| 31 | Fail | 无 Cloth cache record/seek/loop/version/error门 |
| 32 | Fail | 无 authoritative/cosmetic replay/network policy |
| 33 | Fail | 无 Cloth malformed/fuzz/cancel/OOM/crash/device-loss/unload矩阵 |
| 34 | Fail | 无 Cloth debug snapshot与零reader成本证明 |
| 35 | Fail | 无 cape/skirt/flag/layered source、oracle、pixel capture或 soak |
| 36 | Fail | 无同资产/画质/硬件 benchmark与原始 receipt |

统计：**33 Fail / 3 Partial / 0 Pass**。Partial gate不允许提升 capability；必须补齐该 gate的 Cloth-specific全部条件才可转 Pass。

## 15. Finding 到 Owner 与里程碑映射

| Finding | 主里程碑 | 相邻 owner依赖 |
|---|---|---|
| CLOTH-P1-001..012 | M0-M1 | Resource、Scene、Editor asset/import |
| CLOTH-P1-013..028 | M2、M6 | World、Time、Physics provider |
| CLOTH-P1-029..042 | M3 | Animation、Physics、Weather、Network |
| CLOTH-P1-043..056 | M4-M5 | Render/GPU Scene、Material、Temporal、RT/GI |
| CLOTH-P1-057..064 | M6-M8 | Scalability、Diagnostics、Editor/Product |
| CLOTH-P2-001..014 | P1门完成后独立立项 | 不得提前并入MVP或借P2绕过CPU oracle |

上层现行 owner对齐如下：Runtime99zm 的 `PH-P1-041`只负责为 vehicle/soft body/cloth/rope/destruction建立分包 provider与 coupling边界；Runtime99zl把 cloth/deformation graph保持为 advanced animation Open；Runtime99b 的 Runtime101-P1-17负责 WPO/procedural deformation统一 previous-state ABI；Runtime91与 surface-lighting owner负责 material/compiler/BRDF。Cloth source/compiler/solver/runtime/output/Editor纵切面只在本篇计数，避免跨报告重复P0。

## 16. 禁止的临时修补

1. 禁止给`MeshRenderer`加`cloth: bool`和几个 stiffness字段后逐顶点积分。
2. 禁止把 render mesh直接当 sim mesh，或用 Morph target数组保存每帧 Cloth state。
3. 禁止用 Ragdoll、double-sided material、vertex animation、WPO或摇摆 shader冒充 Cloth。
4. 禁止 CPU、GPU、Editor preview和 cache playback各自实现不同 constraint/solver语义。
5. 禁止把任意 Mesh custom attribute字符串当 seam/fabric/weight-map长期schema。
6. 禁止在 Cloth里私建 rigid world、broad phase、fixed clock、weather或 material compiler authority。
7. 禁止每帧重建完整 GPU mesh、同步 GPU readback后才提交，或无 fence复用动态buffer。
8. 禁止 depth、GBuffer、forward、shadow、velocity、RT分别读取不同代 geometry。
9. 禁止无界 self-collision pair、scratch、task、debug trace或 gameplay force队列。
10. 禁止碰撞失败、NaN、OOM、device loss或 provider缺失时静默回到“仍在摆动”的假成功。
11. 禁止把 Unity Fabric shader、Bevy/Fyrox skin/Morph或 Godot demo当成完整 solver完成度。
12. 禁止只做 flag demo、一个 cape或 Editor paint外观就关闭 source/compiler/lifecycle/collision/qualification条目。
13. 禁止保留旧路径 facade、compat module、re-export或 JSON bridge；实施采用明确 hard cutover。
14. 禁止在没有同资产、同画质、同硬件和原始 receipt时宣称性能或表现超过 Unreal。

## 17. 首个允许的实施切片

当前工程仍受 MVP priority gate约束，本篇是 C3 review-only，不授权立即实现 advanced Cloth。前置允许后，首切片只做 M0-M1 的 truth与CPU compiler/oracle RED门：

1. 定义 Cloth/SoftBody术语、唯一 owner、ResourceKind/source/component/capability的 Unsupported truth测试。
2. 建最小但非临时的 versioned source schema：stable topology IDs、sim/render topology、seam、Fabric、maps、attachments与 LOD。
3. 建 deterministic compiler artifact/digest、malformed corpus、migration/LKG和 save/reopen roundtrip。
4. 建无渲染依赖的 deterministic CPU oracle fixture，先覆盖 particle state、stretch/bend/tether/seam和 finite failure。
5. 只有上述门闭合后，才选择 Jolt SoftBody扩展、独立XPBD provider或 GPU provider；不得由依赖便利性倒推架构。

## 18. 实施前重查清单

1. 重导本篇 525 个 Zircon与522个参考文件选择集，重算11组指纹。
2. 重跑12,597个production Rust文件的精确领域扫描，逐条人工分类新增命中。
3. 复核`ResourceKind`、`SceneEntityAsset`、`MeshRenderer`、Physics handle/shape/provider、`RenderFrameExtract`、`BuiltinRenderFeature`、Material lighting model、catalog/App/Editor registry。
4. 取得 Resource、Scene/World、Runtime99zm、Runtime99zl、Runtime99b、Runtime91、Weather、Editor32 owner确认。
5. 先写 capability/source/compiler/CPU oracle RED测试，再写production实现；不得从 demo或 GPU kernel起步。
6. 动态验证按 Windows优先：core/compiler/headless -> Jolt/WGPU -> App/Editor/PIE -> fault/scale/soak -> cross-engine benchmark。

## 19. 本轮产出边界

本轮只新增静态 current-source review与索引记录，没有修改 Runtime、Interface、Plugin、App、Editor production代码或 tests，也没有运行 Cargo、Jolt SoftBody或 WGPU。报告不表示 Cloth已经可用，不授权从 P2高级能力开工；实现必须从 M0 truth/owner与 M1 source/compiler开始，以 deterministic CPU oracle、generation-qualified output、runtime-backed Editor和真实产品证据逐层收敛。
