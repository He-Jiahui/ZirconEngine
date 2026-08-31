---
title: Editor Destruction、Fracture、Geometry Collection、Damage Field 当前工作树复审
category: zircon_editor
report_id: Editor241
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
canonical_owner: Editor241
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/181-runtime-destruction-fracture-geometry-collection-current-working-tree-review.md
related_code:
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editing
  - zircon_editor/src/scene
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/preview_scene
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_runtime_interface/src/resource/marker.rs
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Experimental/Fracture/Source
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAssetEditorCore/Source/ChaosClothAssetEditorTools
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/GeometryCollectionEngine/Public/GeometryCollection/GeometryCollectionComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/GeometryCollectionEngine/Public/GeometryCollection/GeometryCollectionCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/FieldSystem/Source/FieldSystemEngine/Public/Field/FieldSystemComponent.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/ChaosCaching
  - dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/GeometryCollection/GeometryCollectionTestClustering.cpp
  - dev/godot/scene/3d/physics/soft_body_3d.cpp
  - dev/bevy/crates
  - dev/Fyrox/fyrox-impl/src/scene/mesh
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor241 · Destruction/Fracture/Geometry Collection authoring 当前工程化差距

## 1. 结论

当前 Editor 没有 Geometry Collection/Fracture/Damage Field/Simulation Cache asset type、factory、document、toolkit、fracture tools、cluster editor 或 destruction preview。`builtin.rs` 的 26 个资源种类和 `toolkit.rs` 的 UI/Animation 路由没有 collection/fracture/caching/field；`zircon_editor/src/scene`、`core/editing`、`ui/asset_editor` 与 `ui/preview_scene` 没有相关 provider、operation 或 runtime mirror。

通用 Mesh inspector、viewport selection/gizmo、Scene transaction、PreviewScene、Physics workbench、capture/diagnostics 只能作为宿主。它们没有 piece/face/cluster selection、cutter graph、interior material、connection/strain、field brush、break event timeline、cache seek 或 runtime generation receipt。任何静态 debris/mesh rows 都不能被标为 Destruction Ready。

本报告登记 **0 项 P0、28 项 P1、10 项 P2、24 道资格门**；P1 28 Open，P2 10 Open，资格门 21 Fail、3 Partial、0 Pass。Runtime181 负责 compiler/provider/break/render/cache owner。

## 2. 当前源码证据

- `zircon_editor/src/core/asset/type_registry/builtin.rs:22-47,78-112` 没有 GeometryCollection/Fracture/Field/Cache ID、metadata、factory、thumbnail 或 open operation。
- Scene authoring/Inspector 只有 Mesh/Model/Rigid/Collider/Animation，不能持久化 collection source、piece/cluster state、fracture settings、damage policy 或 cache binding。
- 通用 viewport handle/selection 不理解 piece/face/cluster stable identity，也没有 fracture selection/paint/cluster/connection tool、cutter preview 或 interior visualization。
- PreviewScene 与 play-preview 不能安装 runtime fracture artifact、固定步长运行 cluster solver、注入 field/damage、观察 break transaction 或 seek cache。
- Capture/performance/runtime diagnostics 没有 piece/cluster/contact/strain/break/cache/instance-pool 指标；没有 server/replay/save 状态镜像。

## 3. 参考引擎差异

Unreal Fracture/Dataflow/GeometryCollection Editor 提供 collection factory、geometry selection、fracture modes、cluster/connection tools、interior material、field/debug visualization、cache/preview 与 transaction；GeometryCollection runtime 还有 breaking/collision/removal/trailing filters，FieldSystem 有可组合 field assets/components。Godot/Bevy/Fyrox/Unity 可作为通用 mesh/physics/authoring 对照，但没有同等 collection editor 闭环。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| ED-DESTR-01 | 无 asset types | 注册 DestructionSource/GeometryCollection/FractureArtifact/DamageField/Cache/Profile 与 factory/reimport/thumbnail。 |
| ED-DESTR-02 | 无 provider/catalog | editor provider、runtime capability handshake、feature manifest、缺 backend fail-closed。 |
| ED-DESTR-03 | 无 document | stable collection/piece/face/cluster/connection IDs、revision、dirty/save/reopen/LKG/migration。 |
| ED-DESTR-04 | 无 source inspector | source mesh、seed、cutter、material、units、fracture/cluster/collision settings。 |
| ED-DESTR-05 | 无 geometry viewport | intact/piece/interior/normals/UV/collision/wireframe/LOD visualization。 |
| ED-DESTR-06 | 无 selection model | piece/face/cluster/connection selection、isolate/lock、generation-qualified selection lease。 |
| ED-DESTR-07 | 无 fracture tools | plane/radial/voronoi/brick/cutter graph、preview/apply、deterministic seed、undo。 |
| ED-DESTR-08 | 无 cluster tools | hierarchy levels、merge/split/connection graph、strain threshold、propagation preview。 |
| ED-DESTR-09 | 无 interior tools | interior material/UV/normal generation、seam/duplicate diagnostics。 |
| ED-DESTR-10 | 无 collision authoring | per-piece/cluster collision family、convex/implicit/triangle policy、filter/material。 |
| ED-DESTR-11 | 无 field authoring | radial/plane/box/noise/strain/damage field gizmo、falloff/priority/blend/decay。 |
| ED-DESTR-12 | 无 preview world | runtime compiler/artifact install、fixed-step provider、pause/step/reset/seek/device-loss。 |
| ED-DESTR-13 | 无 break visualization | particles/bodies, cluster state, strain/contact, break/removal/trailing events and receipts。 |
| ED-DESTR-14 | 无 cache editor | record/import/export、timeline、frame/timecode、chunk/CRC/version、seek/checkpoint。 |
| ED-DESTR-15 | 无 compiler job | dependency graph、source spans、progress/cancel、artifact generation/install/rollback。 |
| ED-DESTR-16 | 无 runtime mirror | world/entity/collection/tick/generation/provider、piece count、active/sleep/broken status。 |
| ED-DESTR-17 | 无 commands | create/fracture/cluster/field/collision/cache/duplicate/delete all through operation factory/history。 |
| ED-DESTR-18 | 无 roundtrip | source/artifact settings save/reopen/migrate preserve IDs, unknown fields and hashes。 |
| ED-DESTR-19 | 静态 fixture 风险 | sample debris/cluster/count/break text must come from runtime receipts, never static success. |
| ED-DESTR-20 | 无 product scene | Scene/PIE/standalone add, save/reopen, simulate, break, render, nav/audio/VFX/network/replay。 |
| ED-DESTR-21 | 无 diagnostics | compile/cook/cluster/field/solver/break/cache/GPU/memory/bandwidth/fallback metrics。 |
| ED-DESTR-22 | 无 multi-selection | piece/cluster batch mutation with preflight, partial failure, deterministic history and byte budget。 |
| ED-DESTR-23 | 无 collaboration | document lease、external change/rebase、conflict and operation provenance。 |
| ED-DESTR-24 | 无 performance | fracture compile, collision cook, piece count, preview FPS, instance pool and memory budget。 |
| ED-DESTR-25 | 无 fault UI | malformed artifact, compiler cancel, provider/device loss, NaN, stale generation, rollback。 |
| ED-DESTR-26 | 无 tests | geometry/property/roundtrip、fracture determinism、preview/cache、visual/fault/scale/soak。 |
| ED-DESTR-27 | rigid/destruction 混淆 | ordinary rigid/compound physics must remain distinct from collection/cluster capability。 |
| ED-DESTR-28 | ABI 不稳定 | versioned editor/runtime descriptors，禁止 editor 私造 piece body 或直接写 physics world。 |

## 5. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| type/catalog/provider | Fail | collection/field/cache types、factory、provider、capability 与 unavailable UI。 |
| document/authoring | Fail | cutter/cluster/field/collision commands、undo、roundtrip、stable IDs。 |
| compiler/artifact | Fail | runtime build receipt、diagnostics、generation、install/rollback。 |
| preview/break/cache | Fail | real provider、fixed-step、break events、cache seek/replay。 |
| product integration | Fail | Scene/PIE/standalone 与 nav/audio/VFX/network/save/replay 一致。 |
| scale/fault | Fail | large piece counts、cook/memory/frame budgets、malformed/provider/device failures。 |

本轮只写审查文档，未修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Fracture/Physics/GPU/PIE 动态验证。
