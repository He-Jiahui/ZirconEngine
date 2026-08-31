---
title: Editor Cloth、Fabric、Soft Body、Garment 当前工作树复审
category: zircon_editor
report_id: Editor239
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
canonical_owner: Editor239
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/179-runtime-cloth-fabric-soft-body-current-working-tree-review.md
related_code:
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editing
  - zircon_editor/src/scene
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/preview_scene
  - zircon_editor/src/scene/viewport
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_runtime_interface/src/resource/marker.rs
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAssetEditorCore/Source/ChaosClothAssetEditor
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAssetEditorCore/Source/ChaosClothAssetEditorTools
  - dev/UnrealEngine/Engine/Plugins/Experimental/ChaosClothEditor
  - dev/godot/scene/3d/physics/soft_body_3d.cpp
  - dev/godot/modules/jolt_physics/objects/jolt_soft_body_3d.cpp
  - dev/bevy/crates/bevy_mesh/src/skinning.rs
  - dev/Fyrox/fyrox-impl/src/scene/mesh
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Fabric
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor239 · Cloth/Fabric/Soft Body authoring 当前工程化差距

## 1. 结论

当前 Editor 没有 Cloth/Soft Body asset type、factory、document、toolkit、paint mode、simulation preview 或 runtime mirror。`zircon_editor/src/core/asset/type_registry/builtin.rs:22-47` 只注册 26 个既有 `ResourceKind`，其中没有 cloth、fabric、garment、soft-body 或 simulation cache；`toolkit.rs` 只描述 UI/Animation 文档路由。`zircon_editor/src/scene`、`core/editing`、`ui/asset_editor` 与 `ui/preview_scene` 没有对应 provider 或 operation。

现有 Mesh/Animation/Physics inspector、通用 viewport gizmo、preview scene 和 capture/diagnostics 是可复用宿主，但没有 cloth topology/weight-map/pin/constraint 语义，也没有 runtime artifact generation、selection lease、dirty/save/reopen 或 preview-world receipt。测试 fixture 中的 `cloth` 路径和通用 viewport 控件不是功能入口。

本报告为 Editor Cloth 独立 owner，登记 **0 项 P0、28 项 P1、10 项 P2、24 道资格门**；P1 为 28 Open，P2 为 10 Open，资格门为 21 Fail、3 Partial、0 Pass。运行时 solver 与 render/physics P0 由 Runtime179 及其父报告唯一计数。

## 2. 当前源码证据

- builtin registry 没有 Cloth 资源 ID、metadata、thumbnail provider、creation template 或 toolkit；first-party editor catalog 没有 Cloth provider/manifest。
- Scene/Inspector carrier 只暴露 mesh/model/rigid/collider/animation；没有 ClothComponent、source handle、simulation LOD、cache binding、attachment 或 material profile。
- UI asset editor 的 preview host 可承载静态 subject，但没有 runtime artifact install、fixed-step/pause/seek/reset、provider capability、world generation、device loss 或 stale frame rejection。
- viewport interaction 有通用 selection/handle/transaction，却没有 particle/triangle/edge/face selection、weight-map paint、pin brush、seam/tear authoring、collision proxy visualization 或 simulation overlay。
- capture/performance/runtime diagnostics 只报告通用 frame/capture/GPU 数据；没有 particle/constraint/iteration/contact/bounds/fallback/solver fence 指标。

## 3. 参考引擎差异

Unreal Cloth Editor 同时提供 rest-space 与 3D viewport、mesh selection、weight-map paint、skin-weight transfer、simulation visualization、preview scene 与 advanced preview details，并把 editor transaction 与 ClothAsset/SimulationModel build artifact 连接。Godot SoftBody inspector 至少表达 pinned points、simulation/physics state 与 backend availability；Bevy/Fyrox 的网格/骨骼工具只能作为数据编辑参考，Unity Fabric 只覆盖材质模型，不能替代 cloth authoring。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| ED-CLOTH-01 | 无 asset type | 注册 ClothSource/BuildArtifact/FabricProfile/Cache/Mask 类型、icons、create/open/reimport/thumbnail。 |
| ED-CLOTH-02 | 无 provider/catalog | editor provider manifest、capability handshake、runtime admission 与缺 backend 的不可用状态。 |
| ED-CLOTH-03 | 无 ClothDocument | stable cloth/LOD/mesh/vertex-map IDs、schema、revision、dirty/save/reopen/LKG/migration。 |
| ED-CLOTH-04 | 无 source editing | rest mesh、sections、material slots、fabric configs、units 与 import provenance 编辑器。 |
| ED-CLOTH-05 | 无 topology view | particles/triangles/edges/faces、seam、winding、degenerate、constraint diagnostics 可视化。 |
| ED-CLOTH-06 | 无 pin/weight map | paint/erase/smooth/normalize、named maps、symmetry、brush falloff、transaction/undo。 |
| ED-CLOTH-07 | 无 collision authoring | body/skin/terrain proxy 选择、self-collision radius、friction/compliance、collision layer。 |
| ED-CLOTH-08 | 无 attachment authoring | bone/socket/kinematic points、max distance、tether、teleport/reset policy。 |
| ED-CLOTH-09 | 无 simulation controls | fixed dt、substeps、iterations、gravity/wind、pause/step/reset/seek、deterministic seed。 |
| ED-CLOTH-10 | 无 preview world | runtime-produced artifact 安装、isolated world、provider generation、shutdown/drain、device-loss recovery。 |
| ED-CLOTH-11 | 无 simulation visualization | particles、constraints、normals、velocity、collision contacts、bounds、LOD、sleep/wake overlays。 |
| ED-CLOTH-12 | 无 render mapping view | render-to-sim barycentric map、skin binding、missing/duplicate/out-of-range errors。 |
| ED-CLOTH-13 | 无 cache UI | record/import/export、chunk index、frame/timecode、seek/checkpoint、version/CRC、budget。 |
| ED-CLOTH-14 | 无 compiler job | source spans、dependency graph、progress/cancel、artifact generation、install/rollback。 |
| ED-CLOTH-15 | 无 runtime mirror | entity/cloth id、world/tick/generation、provider status、current/previous output 与 terminal reason。 |
| ED-CLOTH-16 | 无 inspector | simulation/render LOD、mass/damping, wind, collision, solver tier, material profile 的 typed fields。 |
| ED-CLOTH-17 | 无 commands | placement、duplicate、delete、bind、paint、pin、cache、bake 全部进入 operation/factory/undo。 |
| ED-CLOTH-18 | 无 source roundtrip | save/reopen/upgrade 后 semantic equality、stable IDs、unknown-field preservation 与 LKG。 |
| ED-CLOTH-19 | 无 diagnostics | CPU/GPU time、particles/constraints、contact/overflow、memory/upload/fence、fallback、stale。 |
| ED-CLOTH-20 | 静态 fixture 风险 | 任何 Cloth 文案、preview badge、sample rows 必须绑定 provider receipt，失败时明确 unavailable。 |
| ED-CLOTH-21 | 无 product scene | Scene/PIE/standalone 中添加 Cloth 后能加载 source、build、instantiate、simulate、render、save。 |
| ED-CLOTH-22 | 无 multi-selection | cloth/LOD/map 多选修改需 typed batch、preflight、partial failure 和 deterministic history。 |
| ED-CLOTH-23 | 无 collaboration boundary | document lock/lease、external change/rebase、operation provenance、conflict UI。 |
| ED-CLOTH-24 | 无 performance gates | authoring topology size、preview FPS、compile time、memory、GPU/CPU solver budget 测量。 |
| ED-CLOTH-25 | 无 fault handling | malformed source、compiler cancel、provider loss、device loss、world replace 有可恢复 terminal state。 |
| ED-CLOTH-26 | 无 test matrix | authoring/property/roundtrip、compiler errors、preview fixed-step、cache seek、visual regression。 |
| ED-CLOTH-27 | fabric/cloth 混淆 | Fabric material 与 Cloth simulation 分离展示，capability 与 dependency 不得互相伪装。 |
| ED-CLOTH-28 | editor/runtime ABI 未锁 | 使用版本化 neutral descriptors，禁止 UI 直接构造 solver 或写 Scene runtime state。 |

## 5. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| asset/catalog/provider | Fail | type、factory、provider、runtime capability 和 unavailable UI 一致。 |
| document roundtrip | Fail | source 保存重开、迁移、冲突恢复保持语义与 stable IDs。 |
| operation/undo | Fail | 所有 authoring mutation 有 preflight、receipt、undo/redo、dirty participant。 |
| compiler/artifact | Fail | editor job 只能消费 runtime compiler receipt，显示 generation/errors。 |
| preview-world | Fail | preview 运行真实 runtime provider，支持 step/seek/reset/cancel/device loss。 |
| diagnostics | Fail | 可追踪到 world/tick/cloth generation 的动态指标。 |
| product/PIE | Fail | scene save/reopen、PIE、standalone 与 runtime 输出一致。 |
| fault/scale | Fail | malformed source、provider failure、100+ instances、内存与帧预算可验。 |

本轮只写审查文档，没有修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Editor/Cargo/PIE/solver 动态验证；实施前必须重新取源指纹。
