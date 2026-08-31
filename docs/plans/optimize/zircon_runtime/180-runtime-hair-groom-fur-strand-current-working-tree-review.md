---
title: Runtime Hair、Groom、Fur、Strand 与 Hair Material 当前工作树复审
category: zircon_runtime
report_id: Runtime180
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zt-runtime-hair-groom-fur-strand-source-binding-simulation-rendering-lighting-shadow-lod-streaming-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/32-hair-groom-fur-strand-source-binding-simulation-rendering-lighting-shadow-lod-streaming-scalability-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/240-editor-hair-groom-fur-strand-current-working-tree-review.md
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/graphics/material
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsCore/Public/GroomAsset.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsCore/Public/GroomBindingAsset.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsCore/Public/GroomCache.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsCore/Public/HairStrandsDatas.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsCore/Public/GroomComponent.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsCore/Public/GroomInstance.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsEditor
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Hair
  - dev/godot/scene/3d/physics/soft_body_3d.cpp
  - dev/bevy/crates/bevy_mesh/src/skinning.rs
  - dev/Fyrox/fyrox-impl/src/scene/mesh
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime180 · Hair/Groom/Fur/Strand 当前工程化差距

## 1. 结论

当前 Zircon 没有 Hair、Groom、Fur、Strand 或 Alembic hair runtime 产品。对 production Rust 路径排除 tests、fixtures、target 后，`hair/groom/fur/strand/follicle/marschner/kajiya-kay/melanin` 领域命中为零。`ResourceKind`、`ImportedAsset`、Scene carrier、importer/catalog、GPU mesh、visibility、shadow、velocity 与 material registry 均无 Groom source、binding asset、strand resource、cluster、cache、deformer 或 Hair BSDF。

现有 Mesh attribute/topology、skinning/Morph、current/previous deformation、OIT、shadow、resource residency 与 generic PBR 是邻接底座。它们无法替代 strand/guide representation、root binding/interpolation、coverage/deep shadow、multiple scattering、hair material parameters、cluster culling 或 cache playback。历史 Runtime145/32 “零生产 Hair owner”结论仍成立。

本次刷新登记 **0 项新 P0、30 项 P1、12 项 P2、26 道资格门**；P1 30 Open，P2 12 Open，资格门 23 Fail、3 Partial、0 Pass。目标架构为：

```text
GroomSource + import provenance
  -> GroomCompiler
  -> guides/render strands/cards/meshes + root binding + clusters/LOD artifact
  -> generation-qualified HairRuntimeInstance
  -> deformer/simulation/cache provider
  -> current/previous strand resources + bounds/fence
  -> visibility/coverage/lighting/shadow/RT/GI/material adapters
```

## 2. 当前源码证据

### 2.1 资源与绑定

- `zircon_runtime_interface/src/resource/marker.rs:8-31` 没有 Groom/Hair/Binding/Cache 类型；`ImportedAsset` 同样没有相应 variant。
- `SceneNode` 只拥有 MeshRenderer、Animation 与 rigid/collider 组件，没有 GroomComponent、binding handle、strand material 或 cache player。
- glTF/model import 只生成普通 mesh/skin/morph，不能产生 curve roots、guide topology、width/color/melanin、cluster hierarchy、card/mesh fallback 或 source-to-artifact provenance。

### 2.2 Simulation/deformation/cache

- Animation skin/morph 的 pose writer 以 skeleton joint 与 morph weight 为输入，没有 strand simulation state、wind/dynamic constraint、root binding、collision proxy 或 per-frame cache ticket。
- Render resource streamer/GPU mesh 只拥有静态 vertex/index residency；没有 strand segment buffer、curve indirection、card fallback、dynamic bounds、velocity resource 或 double-buffered generation。
- 没有 GroomCache chunk/index/seek、record/import、version/CRC、deformer provider、teleport/reset 或 deterministic replay。

### 2.3 Visibility、lighting 与 material

- generic visibility/OIT/shadow/velocity 没有 hair-specific coverage, alpha/depth resolve, strand cluster culling, deep shadow, transmittance, tangent frame 或 multiple scattering。
- material registry 没有 Hair/Marschner/Kajiya-Kay/azimuthal-longitudinal scattering、roughness shift、melanin/IOR/absorption profile；不能把 transparent PBR/OIT 当作 Hair shading。
- 没有 RT/GI hair geometry policy、shadow-caster representation、card/mesh/strand LOD 或 per-view visibility history。

## 3. 参考引擎差异

Unreal HairStrandsCore 明确分离 GroomAsset、GroomBindingAsset/Builder、GroomResources、GroomComponent/Instance、GroomCache、cluster/LOD/deformer 与 rendering data，并有 HairStrandsRuntime/Solver/Dataflow 和 editor factory/thumbnail/viewport/toolkit。Unity HDRP `Runtime/Material/Hair` 同时提供 Hair BSDF、multiple scattering、ray/path tracing 与 Hair shader graph。Godot/Bevy/Fyrox 可作为 physics/mesh 数据布局对照，但没有可直接借用的完整 strand pipeline。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| RT-HAIR-01 | 无资源 taxonomy | 增加 GroomSource、GroomArtifact、GroomBinding、HairMaterial、GroomCache、FollicleMask marker/AssetKind。 |
| RT-HAIR-02 | 无 source/artifact 分层 | source 保存 curves/guides/import options；artifact 保存 strands/cards/meshes、binding、clusters、LOD、hash。 |
| RT-HAIR-03 | 无 importer/provenance | Alembic/USD/curve importer、units/up-axis、root/width/color validation、subasset与source span。 |
| RT-HAIR-04 | 无 compiler | deterministic guide interpolation、strand generation、card/mesh fallback、cluster build、artifact diagnostics。 |
| RT-HAIR-05 | 无 binding | skeletal mesh projection、root bone/triangle/barycentric、transfer、rebind、missing root error。 |
| RT-HAIR-06 | 无 representation | guide/render strand/cluster/card/mesh schema、width/tangent/color/melanin/UV/LOD metadata。 |
| RT-HAIR-07 | 无 World instance | per-World HairRuntimeInstance、entity/groom/binding generation、attach/replace/unload/retire。 |
| RT-HAIR-08 | 无 provider ABI | deformer/simulation/cache provider registry，CPU oracle 与 GPU provider 同一 output contract。 |
| RT-HAIR-09 | 无 dynamic state | root pose、wind, inertia, constraints, collision, teleport/reset, sleep/wake and deterministic tick。 |
| RT-HAIR-10 | 无 cache | chunked cache、timecode/frame、compression、seek/checkpoint、version/provider compatibility、CRC。 |
| RT-HAIR-11 | 无 current/previous output | strand/card/mesh buffers、bounds、velocity、motion flag、fence、generation-qualified lease。 |
| RT-HAIR-12 | 静态 GPU mesh | storage/indirect/curve resources、ring residency、partial updates、retirement/device generation。 |
| RT-HAIR-13 | 无 cluster culling | per-view cluster hierarchy、screen error、frustum/HZB/coverage、multi-view and overflow receipt。 |
| RT-HAIR-14 | 无 strand coverage | rasterization/visibility buffer、alpha/depth/moment handling、order-independent coverage contract。 |
| RT-HAIR-15 | 无 Hair lighting | Hair BSDF、tangent frame、azimuthal/longitudinal scattering、roughness/IOR/melanin/absorption。 |
| RT-HAIR-16 | 无 shadow/transmittance | deep shadow/voxel or equivalent, self-shadow, opaque caster fallback, stale generation rejection。 |
| RT-HAIR-17 | 无 RT/GI | BLAS/curve/card policy、ray hit attributes、GI/transmission/multiple scattering integration。 |
| RT-HAIR-18 | 无 LOD/streaming | strand/card/mesh LOD, cluster transition, density/width preservation, residency/importance budget。 |
| RT-HAIR-19 | 无 animation contract | pose generation, interpolation, root binding and cloth/deformer ordering with typed snapshot。 |
| RT-HAIR-20 | 无 gameplay/query | hair hit/query, attachment, grooming events, damage/wetness/wind input adapters。 |
| RT-HAIR-21 | 无 diagnostics | strand/cluster counts, culling, coverage overflow, cache status, CPU/GPU time, memory/fence。 |
| RT-HAIR-22 | 无 failure policy | invalid groom/binding, provider loss, cache underrun, NaN, device loss and stale output terminal states。 |
| RT-HAIR-23 | 无 network/save | server authority, quantized cache/state, join-in-progress, save provenance and replay. |
| RT-HAIR-24 | 无 material residency | Hair LUT/texture/profile residency, permutation key, prewarm and fallback capability truth。 |
| RT-HAIR-25 | 无 tests | import/property/compiler, binding, CPU/GPU parity, cache seek, visibility/shadow, fault/soak/scale。 |
| RT-HAIR-26 | 无 large-world policy | camera-relative strands, precision, origin shift, partition streaming and deterministic bounds。 |
| RT-HAIR-27 | 普通 PBR 误充 Hair | capability/catalog/API 必须区分 Hair material 与 transparent/double-sided PBR。 |
| RT-HAIR-28 | 跨线程 authority 不明 | immutable groom/deformer snapshot、resource lease、render completion、simulation ownership。 |
| RT-HAIR-29 | 编辑器桥缺失 | editor 只能提交 source/operation，runtime 返回 artifact/binding/cache receipts。 |
| RT-HAIR-30 | 质量门缺失 | end-to-end product scenario、roundtrip、provider parity、visual/fault/perf acceptance。 |

## 5. 资格门

| 门 | 结果 | 证据要求 |
|---|---|---|
| resource/import/artifact | Fail | source/import/build/hash/schema/subasset 全部可重现。 |
| binding/instance | Fail | skeletal binding、generation、replace/unload 与 stale rejection。 |
| provider/simulation/cache | Fail | CPU oracle、GPU parity、cache seek/underrun/replay。 |
| visibility/lighting | Fail | strand coverage、shadow、RT/GI、Hair BSDF 与 fallback receipts。 |
| LOD/streaming | Fail | strand/card/mesh transition、residency、memory/frame budget。 |
| editor/product | Fail | editor preview、scene save/reopen、PIE/standalone 与 runtime 一致。 |
| fault/scale/soak | Fail | malformed input、device/provider loss、large groom、多实例长期运行。 |

本轮仅新增审查文档；没有修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Hair solver/GPU/PIE 动态验证。
