---
title: Runtime Destruction、Fracture、Geometry Collection、Damage Field 与 Cache 当前工作树复审
category: zircon_runtime
report_id: Runtime181
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zu-runtime-destruction-fracture-geometry-collection-clustering-damage-field-simulation-rendering-cache-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/33-destruction-fracture-geometry-collection-clustering-damage-field-simulation-rendering-cache-scalability-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/241-editor-destruction-fracture-geometry-collection-current-working-tree-review.md
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/core/framework/physics
  - zircon_plugins/physics/runtime/src
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/graphics/scene/scene_renderer
  - zircon_plugins/first_party_runtime_catalog
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/GeometryCollectionEngine/Public/GeometryCollection/GeometryCollectionObject.h
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/GeometryCollectionEngine/Public/GeometryCollection/GeometryCollectionComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/GeometryCollectionEngine/Public/GeometryCollection/GeometryCollectionCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/GeometryCollectionEngine/Public/ChaosBreakingEventFilter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/FieldSystem/Source/FieldSystemEngine/Public/Field/FieldSystemComponent.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/Fracture/Source/FractureEngine
  - dev/UnrealEngine/Engine/Plugins/Experimental/ChaosCaching
  - dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/GeometryCollection/GeometryCollectionTestClustering.cpp
  - dev/godot/modules/jolt_physics/objects/jolt_soft_body_3d.cpp
  - dev/bevy/crates
  - dev/Fyrox/fyrox-impl/src/scene/mesh
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime181 · Destruction/Fracture/Geometry Collection 当前工程化差距

## 1. 结论

当前 Zircon 没有 Destruction、Fracture、Geometry Collection、Clustered Rigid、Damage Field 或 destruction cache runtime owner。`ResourceKind`/`ImportedAsset`/SceneNode 没有 collection、fracture graph、damage profile、field asset 或 cache。production Rust 的 `destruction/fracture/geometrycollection/voronoi/cluster strain/breaking event/debris` 精确扫描为零；少量 `destruction` 仅是 surface/window/session 生命周期文本。

现有 Mesh import/topology/SDF、per-World rigid Physics/Jolt、shape/body/joint/fixed-step、GPU Scene current/previous transform、visibility、LOD、resource cache 与 generic diagnostics 是通用前置，但不具备 stable piece identity、interior faces、hierarchy/connection graph、cluster solver、damage/field ingress、atomic break transaction、piece output、breaking/removal event 或 cache playback。TriangleMesh collider 的逐三角静态 compound 也不能承担大规模碎裂的 collision cook。

历史 Runtime146/33“无生产 owner”结论在当前工作树仍成立。本次刷新登记 **0 项新 P0、30 项 P1、12 项 P2、26 道资格门**；P1 30 Open，P2 12 Open，资格门 22 Fail、4 Partial、0 Pass。目标架构为：

```text
DestructionSource + fracture/field authoring
  -> deterministic FractureCompiler
  -> pieces/interiors/collision/hierarchy/connection/LOD artifact
  -> generation-qualified per-World DestructionRuntimeInstance
  -> ClusteredRigid + Damage/Field/Contact Provider
  -> atomic break/removal/transform/event output
  -> render/nav/audio/VFX/network/query/cache adapters
```

## 2. 当前源码证据

### 2.1 Source、artifact 与 Scene

- `zircon_runtime_interface/src/resource/marker.rs:8-31` 只有普通资源类型，没有 DestructionSource、GeometryCollection、FractureArtifact、DamageField 或 SimulationCache。
- `zircon_runtime/src/asset/assets/imported.rs:21-44,116-147` 无 collection/fracture import dispatch、subasset/piece provenance、interior material 或 build output。
- `zircon_runtime/src/scene/components/scene/node.rs:15-44,47-90` 只承载 mesh/rigid/collider/joint/animation，缺 collection component、initial cluster state、piece policy、damage/cache/network settings。
- Mesh schema 的 indices/attributes/SDF 能提供几何输入，但没有 piece/face/cluster stable IDs、interior surface/material、transform hierarchy、connection graph、collision family 或 break LOD。

### 2.2 Physics、damage 与 events

- neutral physics step result 只有 plan、contacts、triggers；contact event 只有 world/entity/other/point/normal，缺 impulse、relative velocity、subshape/face/material、tick、generation 与 damage provenance。
- Jolt bridge 以独立 rigid body/shape/constraint 为中心，没有 collection particle/cluster、strain、sleep/merge/split、field command、piece transform batch 或 atomic break transaction。
- 没有 FieldSystem-like radial/plane/noise/strain/damage fields、priority/blend/decay、server authority、dedupe 或 replay semantics。

### 2.3 Rendering、cache 与 scalability

- GPU Scene/visibility/mesh renderer 以 entity+mesh+material draw 为单位，没有 piece residency、cluster culling、per-piece bounds、interior material、fracture normals、debris policy 或 instance pool。
- Shadow/velocity/RT/GI 只有 generic adapters；没有 break-time current/previous piece transforms、newly exposed interior history、motion/teleport classification、shadow cache invalidation。
- generic artifact cache 不能替代 chunked simulation cache；没有 frame/timecode、piece state、compression/CRC、seek/checkpoint、provider version 或 deterministic replay。
- scalability 没有 cluster/piece budget、sleep/merge、distance/importance culling、debris cap、collision cook budget、memory residency 或 network bandwidth policy。

## 3. 参考引擎差异

Unreal GeometryCollectionEngine 提供 `GeometryCollectionObject/Component/Cache`、piece transforms、hierarchy、simulation particles、ISMPool 与 breaking/collision/removal/trailing event filters；FieldSystem 提供 asset/component/object 与 field commands；Fracture plugin 提供 authoring、Dataflow、cluster/connection/interior material、cache 与 editor tools，HeadlessChaos 覆盖 deterministic geometry/physics cases。Godot/Jolt SoftBody 只能证明粒子/链接 provider 的边界，Bevy/Fyrox/Unity Graphics 不能替代 collection pipeline。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| RT-DESTR-01 | 无资源 taxonomy | 注册 DestructionSource/GeometryCollection/FractureArtifact/DamageField/SimulationCache/Profile 类型。 |
| RT-DESTR-02 | 无 source/artifact | source 保留可编辑 mesh/cutter/seed/material/field；artifact 固化 pieces/interiors/collision/hierarchy/connection/LOD。 |
| RT-DESTR-03 | 无 fracture compiler | deterministic Voronoi/plane/cluster/cutter pipeline、units/seed、diagnostics、source map、hash/version。 |
| RT-DESTR-04 | 无 piece identity | stable collection/piece/face/cluster/connection IDs，跨 reimport/LOD/cache/network 保持映射。 |
| RT-DESTR-05 | 无 interior geometry | 编译断面、法线、UV/material、seam/duplicate policy 与 exposed-interior metadata。 |
| RT-DESTR-06 | 无 hierarchy/connection | parent/children, cluster levels, connection graph, strain threshold, propagation and rebuild artifact。 |
| RT-DESTR-07 | 无 collision cook | piece/cluster collision families、convex/implicit/triangle policy、material/filter、cook validation。 |
| RT-DESTR-08 | 无 World instance | per-World component/instance、generation、activation/replace/unload、sleep/retire、capacity。 |
| RT-DESTR-09 | 无 provider ABI | neutral ClusteredRigid/DamageField provider，CPU oracle 与 Chaos-like backend 同一 step/output。 |
| RT-DESTR-10 | 无 field ingress | radial/plane/box/noise/strain/damage fields、falloff/priority/blend/decay、tick/generation。 |
| RT-DESTR-11 | 无 contact damage | impulse/velocity/subshape/face/material contact schema、damage aggregation、dedupe/authority。 |
| RT-DESTR-12 | 无 break transaction | preflight、threshold/propagation、atomic split/removal/rigid spawn、failure rollback、receipt。 |
| RT-DESTR-13 | 无 piece output | batch transforms/velocities/sleep/active flags、current/previous, bounds, generation, fence。 |
| RT-DESTR-14 | 无 render integration | piece/cluster draw packet、interior material、shadow/velocity/RT/GI/visibility/cache invalidation。 |
| RT-DESTR-15 | 无 instancing pool | ISM/mesh pool、stable slot, retire fence, compaction, multi-view culling and overflow policy。 |
| RT-DESTR-16 | 无 LOD/streaming | cluster/piece LOD、distance/importance, sleep/merge, debris cap, partition residency。 |
| RT-DESTR-17 | 无 simulation cache | frame/timecode chunks、piece/cluster state、compression/CRC、seek/checkpoint、version/provider。 |
| RT-DESTR-18 | 无 gameplay adapters | nav blockers, audio/VFX/decal, query/picking, damage events, quest/gameplay tags, replay。 |
| RT-DESTR-19 | 无 network/save | server authority, quantized break events, join-in-progress, prediction/reconciliation, save snapshot。 |
| RT-DESTR-20 | 无 diagnostics | piece/cluster counts, break/contact/field stats, solver/cook/cache/GPU time, memory/bandwidth。 |
| RT-DESTR-21 | 无 failure policy | invalid artifact, provider loss, budget overflow, NaN, device loss, stale generation and rollback states。 |
| RT-DESTR-22 | 无 tests | fracture determinism, topology/interior, collision, damage/field, atomic break, cache/replay, render parity。 |
| RT-DESTR-23 | 普通 rigid 误充 destruction | capability truth 区分 rigid/compound collider 与 collection/cluster owner。 |
| RT-DESTR-24 | generic cache 误充 Chaos cache | dedicated cache schema、artifact lineage、seek/CRC/compatibility receipt。 |
| RT-DESTR-25 | 大世界未定义 | origin shift、partition/streaming、precision、cross-world IDs and replay coordinates。 |
| RT-DESTR-26 | 线程/所有权不明 | immutable field/contact snapshot、solver output lease、render/physics/event publication order。 |
| RT-DESTR-27 | editor/runtime 断裂 | editor source/operation -> runtime compiler/provider/artifact/generation，禁止 editor 私造 pieces。 |
| RT-DESTR-28 | 事件过滤器缺失 | typed breaking/collision/removal/trailing events、subscription generation、bounded retention。 |
| RT-DESTR-29 | 产品集成缺失 | Scene/PIE/standalone save/reopen、audio/VFX/nav/network/replay end-to-end。 |
| RT-DESTR-30 | 质量门缺失 | CPU oracle、provider parity、fault/scale/soak、GPU capture 与 performance budget。 |

## 5. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| source/fracture artifact | Fail | deterministic pieces/interiors/hierarchy/connection/collision artifact。 |
| world/provider | Fail | generation-qualified collection instance、cluster solver与field ingress。 |
| break/events | Fail | atomic split/removal、typed events、dedupe、rollback。 |
| render/cache | Fail | piece output、interior/shadow/velocity/RT、cache seek/replay。 |
| gameplay/network/save | Fail | nav/audio/VFX/query、server authority、save/join/replay。 |
| scalability | Fail | 1/100/1000 collections、piece budget、memory/bandwidth/frame budget。 |
| fault/soak | Fail | malformed artifact、provider/device loss、NaN、long-running stress。 |

本轮只写 review 文档，未修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Fracture/Jolt/GPU/PIE 动态验证。
