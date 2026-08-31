---
title: Editor Hair、Groom、Fur、Strand 当前工作树复审
category: zircon_editor
report_id: Editor240
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
canonical_owner: Editor240
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/180-runtime-hair-groom-fur-strand-current-working-tree-review.md
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
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsEditor
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsCore/Public/GroomAsset.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsCore/Public/GroomBindingAsset.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/HairStrands/Source/HairStrandsCore/Public/GroomCache.h
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Hair
  - dev/godot/scene/3d/physics/soft_body_3d.cpp
  - dev/bevy/crates/bevy_mesh/src/skinning.rs
  - dev/Fyrox/fyrox-impl/src/scene/mesh
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor240 · Hair/Groom/Fur/Strand authoring 当前工程化差距

## 1. 结论

当前 Editor 没有 Groom/Hair asset factory、document、binding builder、strand/card/mesh preview、cache editor 或 Hair material inspector。builtin asset registry 仅列出 26 个现有 ResourceKind；toolkit 路由只有 UI 与 Animation。Editor production source 没有 Groom provider、thumbnail renderer、import/reimport path、runtime mirror、operation handler 或 Hair-specific viewport mode。

通用 Asset Browser、Mesh/Animation inspector、PreviewScene、viewport selection/gizmo、capture 与 diagnostics 可以作为宿主，但无法表达 curve/group/root、guide/render strand、binding、cluster、card fallback、cache frame、hair BSDF 或 deep-shadow state。固定 model rows、generic material preview 或 transparent mesh 不得标为 Groom Ready。

本报告登记 **0 项 P0、28 项 P1、10 项 P2、24 道资格门**；P1 28 Open，P2 10 Open，资格门 21 Fail、3 Partial、0 Pass。Runtime180 负责 source/compiler/provider/render/physics owner。

## 2. 当前源码证据

- `zircon_editor/src/core/asset/type_registry/builtin.rs:22-47,78-112` 没有 Groom/Hair/Binding/Cache 类型、creation template、thumbnail 或 open operation。
- `zircon_editor/src/core/asset/type_registry/toolkit.rs` 只有 view id/open operation/capabilities 描述，无 runtime provider lease、document generation 或 preview contract。
- `zircon_editor/src/ui/asset_editor/preview` 仅有通用 preview host/projection/mock；没有 curve viewport、strand diagnostics、binding projection、cache timeline 或 runtime artifact install。
- `zircon_editor/src/scene/viewport` 具备通用 picking/handles/transactions；没有 curve/strand/follicle/cluster selection、density/width brush、root binding paint 或 hair visibility overlays。
- 没有 Hair-specific catalog entry、Scene component inspector、PIE/standalone mirror、per-view cluster/coverage/deep-shadow metrics 或 cache record/seek UI。

## 3. 参考引擎差异

Unreal HairStrandsEditor 提供 Groom/Binding/Cache factories、import/reimport options、thumbnail scenes、asset details、material details、custom toolkit、editor mode 与 viewport；Groom core 同时连接 physics/render/cache/track。Unity HDRP Hair 提供 material/Hair shader graph 资产与可调散射参数。Godot/Bevy/Fyrox 只提供通用 physics/mesh/animation 编辑对照，不能替代 Groom authoring。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| ED-HAIR-01 | 无资源类型 | 注册 GroomSource/Artifact/Binding/Cache/HairMaterial/FollicleMask，icons、factory、reimport、thumbnail。 |
| ED-HAIR-02 | 无 provider | editor catalog/manifest、capability handshake、runtime admission、缺 backend fail-closed。 |
| ED-HAIR-03 | 无 GroomDocument | stable groom/group/curve/root/LOD IDs、revision、dirty/save/reopen/LKG/migration。 |
| ED-HAIR-04 | 无 importer UI | Alembic/USD/curve options、units/up-axis、width/color/UV、group filtering 与 provenance。 |
| ED-HAIR-05 | 无 curve viewport | guides/render strands、roots/tangents/width、card/mesh fallback、wireframe/normal/LOD overlay。 |
| ED-HAIR-06 | 无 binding authoring | skeletal mesh projection、root bone/triangle/barycentric、rebind/transfer、missing diagnostics。 |
| ED-HAIR-07 | 无 strand selection | curve/strand/vertex/follicle/cluster selection、isolation、locked selection lease。 |
| ED-HAIR-08 | 无 grooming tools | comb/cut/paint/width/density/clump/noise、symmetry、falloff、typed transaction/undo。 |
| ED-HAIR-09 | 无 material inspector | tangent/roughness/IOR/melanin/absorption/azimuthal scattering 与 texture/LUT dependencies。 |
| ED-HAIR-10 | 无 cache editor | record/import/export、timeline、frame/timecode、seek/checkpoint、version/CRC、underrun。 |
| ED-HAIR-11 | 无 preview world | runtime artifact/binding/provider install、pause/step/reset/seek、world/device generation。 |
| ED-HAIR-12 | 无 simulation controls | wind, deformer, collision, simulation LOD, teleport/reset, deterministic tick controls。 |
| ED-HAIR-13 | 无 visibility diagnostics | cluster culling、coverage/depth、overdraw、card/strand fallback、multi-view status。 |
| ED-HAIR-14 | 无 lighting diagnostics | deep shadow/transmittance、scattering LUT、RT/GI hit data、stale resource state。 |
| ED-HAIR-15 | 无 compiler job | dependency graph、source spans、progress/cancel、artifact generation/install/rollback。 |
| ED-HAIR-16 | 无 runtime mirror | groom/binding/entity/world/tick/generation/provider/output status and terminal reason。 |
| ED-HAIR-17 | 无 scene component | Hair/Groom component inspector、material slot、binding、LOD、cache、render policy persistence。 |
| ED-HAIR-18 | 无 commands | create/bind/rebind/groom/duplicate/delete/cache/bake/assign all use operation factory/history。 |
| ED-HAIR-19 | 无 roundtrip | source/artifact settings save/reopen/migrate preserve curves, roots, maps and unknown fields。 |
| ED-HAIR-20 | 静态 fixture 风险 | sample names/badges/counts must come from provider receipt, never “Groom Ready” text。 |
| ED-HAIR-21 | 无 product scenario | scene save/reopen、PIE、standalone、asset reload、render/shadow/LOD/cache end-to-end。 |
| ED-HAIR-22 | 无 multi-selection | group/curve/material/LOD batch edit with preflight, partial failure and deterministic history。 |
| ED-HAIR-23 | 无 collaboration | document lease、external change/rebase、conflict and operation provenance。 |
| ED-HAIR-24 | 无 performance | import/compile/preview FPS、strand count、GPU memory、coverage/deep-shadow/frame budget。 |
| ED-HAIR-25 | 无 fault UI | malformed groom/binding、provider/cache/device loss、NaN、stale output recovery。 |
| ED-HAIR-26 | 无 tests | import/compiler/binding/roundtrip、preview/cache、visual regression、fault/scale/soak tests。 |
| ED-HAIR-27 | Hair/Fabric 混淆 | Hair material and Cloth/Fabric simulation must be separate type/provider/preview routes。 |
| ED-HAIR-28 | ABI 不稳定 | versioned neutral editor/runtime descriptors，禁止 UI 直接持有 solver/GPU objects。 |

## 5. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| type/catalog/provider | Fail | resource、factory、provider、capability、unavailable state 一致。 |
| document/operations | Fail | typed mutation、undo、dirty/save/reopen、stable IDs 与 conflict handling。 |
| compiler/artifact | Fail | runtime compiler receipt、diagnostics、generation、install/rollback 可观察。 |
| preview/cache | Fail | real provider、fixed-step、seek/reset/underrun/device-loss。 |
| render/material | Fail | strand/card/mesh preview、Hair BSDF/deep-shadow/RT/GI receipts。 |
| product/PIE | Fail | scene/runtime/editor 结果一致。 |
| fault/scale | Fail | invalid assets、provider loss、large groom、多实例长期运行。 |

本轮仅写 review 文档，未修改生产代码、测试、Cargo、ABI、ZUI，也未运行 Editor/Cargo/PIE/codec/GPU 动态验证。
