---
title: Runtime Cloth、Fabric、Soft Body、Garment 与 Deformation 当前工作树复审
category: zircon_runtime
report_id: Runtime179
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zs-runtime-cloth-fabric-soft-body-garment-simulation-collision-deformation-rendering-wind-lod-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/31-cloth-fabric-soft-body-garment-simulation-collision-deformation-rendering-wind-lod-scalability-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/239-editor-cloth-fabric-soft-body-current-working-tree-review.md
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/core/framework/physics
  - zircon_plugins/physics/runtime/src
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow
  - zircon_plugins/first_party_runtime_catalog
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/ChaosCloth/Source/ChaosCloth
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAsset/Source/ChaosClothAssetEngine/Public/ChaosClothAsset/ClothAsset.h
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAsset/Source/ChaosClothAssetEngine/Public/ChaosClothAsset/ClothSimulationModel.h
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAsset/Source/ChaosClothAssetEngine/Public/ChaosClothAsset/ClothComponent.h
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAssetEditorCore/Source/ChaosClothAssetEditor
  - dev/godot/scene/3d/physics/soft_body_3d.cpp
  - dev/godot/modules/godot_physics_3d/godot_soft_body_3d.cpp
  - dev/godot/modules/jolt_physics/objects/jolt_soft_body_3d.cpp
  - dev/bevy/crates/bevy_mesh/src/skinning.rs
  - dev/bevy/crates/bevy_mesh/src/morph.rs
  - dev/Fyrox/fyrox-impl/src/scene/mesh
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Fabric
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime179 · Cloth/Fabric/Soft Body 当前工程化差距

## 1. 结论

当前 Zircon 没有 Cloth、Garment 或通用 Soft Body runtime owner。`ResourceKind` 只有 Mesh/Model/Animation/PhysicsMaterial 等 26 类；`ImportedAsset` 与 `AssetKind` 没有 cloth source 或 cooked simulation artifact；`SceneNode`/`NodeRecord` 只有 mesh、rigid body、collider、joint 和 animation carrier；physics DTO 只描述刚体/形状/约束。对 `zircon_runtime`、`zircon_runtime_interface`、`zircon_plugins` 与 `zircon_editor` 的 production 路径排除 tests、fixtures、target 后，领域词命中仅落在 shader import 测试夹具和无关生命周期文本，没有 solver、component、provider 或 render pass。

Mesh 的 typed attributes、skin/Morph、current/previous deformation、GPU Scene history、Mesh LOD、fixed-step physics、skeletal collider 和 generic material 是可复用底座，不是布料实现。`GpuMeshResource` 只创建 vertex/index usage，不能由 solver 写入动态位置；`PhysicsContactEvent` 只有 entity、point、normal，缺少布料粒子、triangle、impulse 与 generation。普通 skinned mesh、Morph、double-sided material 或 ragdoll 不得被标为 Cloth Ready。

历史 Runtime144/31 的“无生产 owner”判断在当前工作树仍成立。本次刷新登记 **0 项新 P0、30 项 P1、12 项 P2、26 道资格门**；P1 为 30 Open，P2 为 12 Open，资格门为 23 Fail、3 Partial、0 Pass。相邻 Physics/Animation/Renderer 报告的 P0 不重复计数。

目标架构必须是：

```text
ClothSourceAsset + import provenance
  -> deterministic ClothCompiler
  -> simulation/render topology + fabric/constraint/pin/collision/LOD artifact
  -> generation-qualified per-World ClothRuntimeInstance
  -> admitted CPU/GPU SoftBodyProvider
  -> current/previous DeformationOutput + bounds/fence
  -> render, physics, animation, wind, cache, replay adapters
```

## 2. 当前源码证据

### 2.1 Resource、Scene 与生命周期

- `zircon_runtime_interface/src/resource/marker.rs:8-31` 的枚举没有 Cloth、SoftBody、Fabric、Garment、SimulationCache 或 DeformationOutput kind。
- `zircon_runtime/src/asset/assets/imported.rs:21-44,116-147` 的 import dispatch 只覆盖既有资源；不存在 cloth importer、subasset、schema version、source map 或 cook receipt。
- `zircon_runtime/src/scene/components/scene/node.rs:15-44,47-90` 的 Scene carrier 没有 cloth handle、simulation settings、skin binding、collision policy、LOD policy、cache binding 或 rest-pose state。
- 没有 per-World instance table、owner generation、activation/replace/unload、fixed-step admission 或 device-loss retirement。把 `AnimationPlayerComponent.time_seconds` 或 `RigidBodyComponent` 当成软体状态会形成错误 authority。

### 2.2 Mesh、deformation 与 physics 边界

- Mesh schema 有 positions/normals/tangents/skin/morph 与 topology validation，但没有 simulation topology、seam graph、bending/stretch constraints、mass/area、pin/max-distance、self-collision radius、tether 或 render-to-sim map。
- `zircon_runtime/src/graphics/scene/resources/gpu_mesh` 的资源模块拆分为 vertex/index/bounds/wire segments，未声明 storage vertex output、readback fence、double-buffered dynamic residency 或 generation-qualified deformation packet。
- temporal velocity 能消费普通 current/previous transform/skin/morph，不能证明 cloth solver 产生粒子 velocities、teleport classification、substep history 或 bounds expansion。
- `zircon_runtime/src/core/framework/physics/contact_event.rs` 的 contact DTO 没有 impulse、relative velocity、subshape/triangle、material、tick、solver generation；现有 Jolt bridge 没有 SoftBody/Cloth native binding。

### 2.3 Render、wind 与 scalability

- frame extraction 与 mesh renderer 只发布静态 mesh/model/material/LOD；不存在 cloth deformation producer、skinned collision proxy、wind field snapshot 或 render mapping validation。
- Shadow、visibility、velocity、OIT 与 PBR 可以作为 adapter 目标，但目前没有动态 cloth bounds、backface/thickness、self-shadow、motion vector、transmission/fabric profile 或 failure receipt。
- scalability 只有通用 quality/profile/Lod 设施，没有 cloth particle budget、solver iteration tier、self-collision budget、cloth-to-skin fallback、simulation freeze、distance/importance culling、memory residency或 cache seek budget。

## 3. 参考引擎差异

Unreal `ClothAsset`/`ClothSimulationModel` 把 rest/dynamic mesh、weight map、fabric、config、LOD transition 和 build data 分开，`ClothComponent` 与 simulation proxy 通过 generation/LOD cache 把运行时输出交给 skeletal mesh；Chaos Cloth editor 提供 3D/rest-space viewport、selection、skin-weight transfer 与 weight-map paint。Godot `SoftBody3D` 明确 pin/physics-state 与 backend bridge；Jolt soft-body 维护 particles, faces, links, volume/material and solver state。Bevy/Fyrox 的 skin/morph 只能作为数据布局对照，Unity HDRP Fabric 只覆盖 Charlie/sheens，不是 cloth simulation。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| RT-CLOTH-01 | 无资源类型 | 增加 ClothSource、ClothBuildArtifact、FabricProfile、ClothSimulationCache、ClothMaterialProfile marker/serde/AssetKind 映射。 |
| RT-CLOTH-02 | 无 source/artifact 分层 | source 只含可编辑输入；artifact 固化 topology、map、constraints、LOD、collision、material 与 compiler provenance。 |
| RT-CLOTH-03 | 无确定性 compiler | 固定 units、seed、winding、degenerate/duplicate policy；输出 diagnostics、source spans、hash 与 schema version。 |
| RT-CLOTH-04 | 无 simulation topology | 编译 triangles/particles/edges/faces、stretch/bend/area constraints、mass/damping、pin/max-distance、tether/self-collision data。 |
| RT-CLOTH-05 | 无 render-to-sim map | 为每个 render vertex 生成 barycentric/triangle/skin binding，验证 missing/duplicate/out-of-range，并保留 rest/current mapping。 |
| RT-CLOTH-06 | 无 body/world instance | per-World instance、stable entity/cloth id、generation、activation/retire、capacity、shutdown drain 与 stale-handle rejection。 |
| RT-CLOTH-07 | 无 provider ABI | 定义 neutral SoftBodyProvider，CPU reference 与 GPU provider 共享 step/query/output ABI，backend 缺失必须 fail-closed。 |
| RT-CLOTH-08 | 无 deterministic step | fixed dt、substep、iteration、sleep/wake、teleport/reset、time debt、seed 和 replay checkpoint 必须显式。 |
| RT-CLOTH-09 | 无 collision ingress | skin/rigid/terrain collider snapshot、continuous collision、self collision、friction/compliance 与 collision generation 必须接入。 |
| RT-CLOTH-10 | 无 animation/kinematic binding | bone pose sampling、kinematic particles、attachment/teleport policy、pose generation 与 solve ordering 要有 typed contract。 |
| RT-CLOTH-11 | 无 wind/aerodynamics | wind field snapshot、gust/drag/lift、local velocity、units 与 deterministic sampling 不能通过全局常量注入。 |
| RT-CLOTH-12 | 无 deformation output | current/previous positions/normals/velocities、bounds、motion flags、GPU buffer lease 与 completion fence 必须原子发布。 |
| RT-CLOTH-13 | GPU buffer 不可写 | 扩展 RHI usage、dynamic residency、ring/double buffer、partial dirty range、retirement 与 device generation。 |
| RT-CLOTH-14 | 无 render adapters | visibility、depth/GBuffer/forward、shadow、velocity、OIT、RT/GI、fabric transmission 均消费同一 output generation。 |
| RT-CLOTH-15 | 无 LOD/streaming | simulation/render LOD、transition cache、particle decimation、distance/importance budget、stream in/out 与 hysteresis。 |
| RT-CLOTH-16 | 无 cache/replay | cache chunk、frame index/timecode、compression、seek/checkpoint、provider version、CRC 与 network/save policy。 |
| RT-CLOTH-17 | 无 gameplay/query | particle/triangle ray query、cloth contact event、tear/break/attachment event、damage 与 gameplay adapter。 |
| RT-CLOTH-18 | 无 diagnostics | per-instance timings、iterations、contacts、overflow、fallback、stale generation、memory 与 GPU fence metrics。 |
| RT-CLOTH-19 | 无 failure policy | invalid artifact、provider loss、NaN、budget exhaustion、device loss、world replacement 必须有 typed terminal state，不得静默保持旧帧。 |
| RT-CLOTH-20 | 无 tests | CPU oracle、topology/property tests、collision cases、determinism/replay、LOD transition、GPU parity、fault/soak/scale gates。 |
| RT-CLOTH-21 | 普通 animation 误充 cloth | capability truth 必须区分 Animation/Morph/Skin/Ragdoll 与 Cloth，并在 catalog/API 中 fail-closed。 |
| RT-CLOTH-22 | fabric shader 缺失 | 建立 fabric profile/BRDF/normal/thickness/sheen/transmission 与 lighting model registry，禁止用 double-sided PBR 代替。 |
| RT-CLOTH-23 | 工程性能未知 | 设粒子数、约束数、substep、GPU occupancy、upload bytes、frame budget、memory ceiling 的可复现 benchmark。 |
| RT-CLOTH-24 | 外部 mutation 无门 | 所有 pin/impulse/teleport/material override 走 typed command、world generation、tick 与 receipt。 |
| RT-CLOTH-25 | 多线程所有权不明 | solver、render、physics、animation 之间使用 immutable snapshot/lease，禁止跨线程直接写 Scene 或 GPU resource。 |
| RT-CLOTH-26 | 版本迁移缺失 | source/artifact/component/cache 都要有 schema migration、compatibility matrix、LKG rollback 与 orphan reporting。 |
| RT-CLOTH-27 | 大世界未定义 | origin shift、precision、camera-relative collision/query、partition residency 与 deterministic replay 必须通过同一 world frame。 |
| RT-CLOTH-28 | network/save 未接入 | server authority、quantized state、join-in-progress、prediction/reconciliation、save snapshot 与 cache provenance。 |
| RT-CLOTH-29 | editor/runtime 断裂 | Editor 只能提交 source/operation，runtime 编译并返回 artifact/generation，禁止 preview 私造 solver。 |
| RT-CLOTH-30 | 质量门缺失 | 关闭前必须通过 source roundtrip、CPU oracle、backend parity、render receipt、fault/scale/soak 与 product scenario。 |

## 5. 资格门与执行顺序

| 门 | 结果 | 证据要求 |
|---|---|---|
| capability truth | Fail | catalog、plugin manifest、API 与 runtime provider 同时报告 Cloth availability。 |
| source roundtrip | Fail | source 保存、重开、迁移后 stable IDs 与 diagnostics 不变。 |
| compiler determinism | Fail | 相同输入/seed/compiler version 得到相同 artifact/hash。 |
| CPU oracle | Fail | pinned cloth、collision、wind、sleep/wake、teleport 的 golden frames。 |
| provider parity | Fail | CPU/GPU 输出误差、bounds、velocity、events 和 terminal states 可比较。 |
| render integration | Fail | depth/GBuffer/shadow/velocity/RT/GI 使用同 generation 且无 stale frame。 |
| cache/replay | Fail | seek、rewind、network resim、save/load 与 version mismatch 有结果。 |
| editor preview | Fail | runtime artifact 驱动 preview，失败/取消/设备丢失可恢复。 |
| scalability | Fail | 1/100/1000 instance、LOD、memory、GPU/CPU frame budget 与 overflow receipt。 |
| fault/soak | Fail | invalid topology、NaN、provider loss、world replace、device loss、long run 不崩溃不静默成功。 |

## 6. 结论性边界

本报告只记录 runtime Cloth/Fabric/Soft Body owner。Mesh、Animation、Physics、Wind、Material、Visibility、Network 与 Editor 的共性问题继续由其 canonical 报告负责；实现时必须先完成跨 owner ABI 和 capability handoff，再开始 solver 或 shader。当前没有可运行的 Cloth production path，因此本轮不运行 Cargo、GPU、PIE、solver benchmark 或动态测试，只做源码审查与重构计划。
