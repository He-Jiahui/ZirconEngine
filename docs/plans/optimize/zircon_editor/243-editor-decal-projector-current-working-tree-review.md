---
title: Editor Decal、Projector、DBuffer/GBuffer 与 Receiver 当前工作树复审
category: zircon_editor
report_id: Editor243
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
canonical_owner: Editor243
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/183-runtime-decal-projector-current-working-tree-review.md
related_code:
  - zircon_plugins/rendering/features/decals/editor/src
  - zircon_plugins/rendering/features/decals/runtime/src
  - zircon_plugins/rendering/plugin.toml
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editing
  - zircon_editor/src/scene
  - zircon_editor/src/scene/viewport
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/DecalComponent.h
  - dev/UnrealEngine/Engine/Source/Editor/ComponentVisualizers/Private/SplineComponentVisualizer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/CompositionLighting/PostProcessDeferredDecals.cpp
  - dev/godot/scene/3d/decal.h
  - dev/Fyrox/fyrox-impl/src/scene/decal.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/decal.shader
  - dev/bevy/crates/bevy_pbr/src/decal/clustered.rs
  - dev/bevy/crates/bevy_pbr/src/decal/forward.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Material/Decal/DecalProjectorEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal/DecalSystem.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor243 · Decal/Projector authoring 当前工程化差距

## 1. 结论

当前 Editor 没有 Decal asset/document/factory、projector inspector、scene placement tool、receiver preview 或 DBuffer/GBuffer diagnostics。Decal editor plugin 仅构造 `EditorPluginDescriptor` 并声明 capability；没有 `register_editor_extensions()`、drawer、toolkit、SceneMode、overlay、create/add operation、transaction 或 preview provider。builtin asset registry/toolkit 没有 Decal resource kind。

Material Workbench 可出现 `decal` 相关选择/设计语义，但 runtime MaterialDomain 没有完整 Decal source/compiler/receiver contract；通用 viewport、selection、transform gizmo、PreviewScene 与 capture 不能替代 Decal projector bounds、depth projection、atlas/material residency 或 per-view receipt。由于 Runtime183 的 executor 是 no-op，Editor 不得显示 Decal Ready/Executed。

本报告刷新 Editor39/113/160 的 Decal 边界，登记 **1 项继承 P0（由 Editor39/Plugins04 唯一计数）、28 项 P1、10 项 P2、24 道资格门**；P1 28 Open，P2 10 Open，资格门 21 Fail、3 Partial、0 Pass。

## 2. 当前源码证据

- `zircon_plugins/rendering/features/decals/editor/src/plugin.rs` 只有 descriptor/capability，没有扩展注册或 authoring handlers。
- `zircon_plugins/rendering/features/decals/runtime/src/lib.rs` 的 runtime component descriptor 只有 mode/opacity/normal_blend/atlas_region；executor 明确为 no-op。
- builtin asset registry 没有 Decal type/thumbnail/create/open/reimport；Scene/Inspector 没有 DecalComponent persistence。
- `scene/viewport` 通用 handles 不理解 projector volume、receiver hit、angle/distance fade、atlas region 或 DBuffer channels。
- Material Workbench/fixture routes 没有 document revision、operation receipt、compiler job、runtime generation、pixel evidence 或 failure state。

## 3. 参考引擎差异

Unreal DecalComponent/editor visualizer 和 deferred decal renderer 提供 volume placement、size/fade、material channel、receiver mask、preview and transaction；Godot/Fyrox 将 decal 作为可保存 scene node；Bevy clustered/forward 与 Unity HDRP DecalProjectorEditor/DecalSystem 将 projector、atlas、cluster、draw distance、layer、angle fade 和 material bindings 分层。Zircon 当前是 capability shell。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| ED-DECAL-01 | 无 asset type | DecalMaterial/Projector/Atlas 类型、factory、icons、thumbnail、reimport/open route。 |
| ED-DECAL-02 | editor plugin descriptor-only | 注册 drawer/toolkit/SceneMode/overlay/provider，capability 与 runtime admission 绑定。 |
| ED-DECAL-03 | 无 document | stable projector/material/atlas IDs、schema、revision、dirty/save/reopen/LKG/migration。 |
| ED-DECAL-04 | 无 scene placement | create/add/duplicate/delete/projector transform/size/pivot via typed operation/history。 |
| ED-DECAL-05 | 无 inspector | material, mode, opacity, normal/ORM/emissive, fade, layer, sort, receiver mask fields。 |
| ED-DECAL-06 | 无 volume gizmo | projector box/frustum, inverse transform, bounds, orientation, snap, finite/range validation。 |
| ED-DECAL-07 | 无 receiver preview | depth reconstruction、affected mesh/terrain/transparent receiver、write mask visualization。 |
| ED-DECAL-08 | 无 atlas UI | atlas region picker, UV transform, residency/generation, missing texture failure。 |
| ED-DECAL-09 | 无 material graph | Decal output/input graph, channel/write mask, unsupported node diagnostics and compile. |
| ED-DECAL-10 | 无 compiler job | dependency graph、source span、progress/cancel、artifact generation/install/rollback。 |
| ED-DECAL-11 | 无 preview world | runtime artifact/provider install、real pixel executor、pause/reset/device/world generation。 |
| ED-DECAL-12 | 无 live mirror | projector/entity/world/view/generation、visible/culled/overflow/pipeline status。 |
| ED-DECAL-13 | 无 diagnostics | DBuffer/GBuffer/forward target/channel、blend、depth、atlas、GPU time/memory。 |
| ED-DECAL-14 | 无 temporal debug | projector/camera motion history、TAA reactive mask、teleport/cut reset。 |
| ED-DECAL-15 | 无 shadow/RT debug | shadow/indirect/GI/RT receiver policy and stale resource state。 |
| ED-DECAL-16 | 无 transaction | projector/material/atlas edits all use operation/factory/preflight/undo/dirty participant。 |
| ED-DECAL-17 | 无 roundtrip | source/component/artifact save/reopen/migrate preserve values and stable IDs。 |
| ED-DECAL-18 | no product scene | scene/PIE/standalone/render target/save/reopen receiver scenarios。 |
| ED-DECAL-19 | 静态 fixture 风险 | Workbench labels/counts/status cannot claim Ready without runtime pixel receipt。 |
| ED-DECAL-20 | no fault UI | missing material/atlas, target mismatch, no-op provider, device loss, stale generation。 |
| ED-DECAL-21 | no multi-view | main/reflection/shadow/capture/editor view assignment/history/residency。 |
| ED-DECAL-22 | no batch authoring | multi-select projector edit, deterministic order, partial failure and byte budget。 |
| ED-DECAL-23 | no collaboration | document lease、external change/rebase、conflict/provenance。 |
| ED-DECAL-24 | no performance | compile/preview/GPU time、projector count、overdraw、atlas/memory budgets。 |
| ED-DECAL-25 | no tests | property/roundtrip/operation, projection/blend, visual regression, GPU/fault/scale。 |
| ED-DECAL-26 | runtime/editor ABI | versioned neutral descriptors，UI 不持有 GPU/graph executor。 |
| ED-DECAL-27 | descriptor-only slot | status must be Unavailable until concrete runtime provider is installed。 |
| ED-DECAL-28 | quality truth | asset/provider/artifact/pixel/diagnostic receipts gate Ready/Executed labels。 |

## 5. P2 增强任务

| ID | 差异 | 工程化方向 |
|---|---|---|
| ED-DECAL-P2-01 | 缺贴图压缩设置 | BC/ASTC/ETC、normal/ORM packing、mip bias 与平台 artifact。 |
| ED-DECAL-P2-02 | 缺虚拟纹理/streaming UI | atlas/page residency、feedback、priority、eviction、LKG。 |
| ED-DECAL-P2-03 | 缺排序编辑 | deterministic layer/sort key、overlap conflict policy 与 receiver order。 |
| ED-DECAL-P2-04 | 缺特殊 receiver | terrain、water、foliage、particle channel compatibility authoring。 |
| ED-DECAL-P2-05 | 缺静态烘焙 | static decal bake、lightmap/GI invalidation 与 mobile fallback。 |
| ED-DECAL-P2-06 | 缺网络寿命 | authoritative spawn/expiry、prediction、replay/join-in-progress inspector。 |
| ED-DECAL-P2-07 | 缺 capture provenance | projector/material/atlas source 到 GPU capture 的关联。 |
| ED-DECAL-P2-08 | 缺资源回收诊断 | retired material/atlas/pipeline lease、fence reclaim 和 leak report。 |
| ED-DECAL-P2-09 | 缺跨 backend golden | Vulkan/Metal/DX12/WebGPU blend/depth/channel visual parity。 |
| ED-DECAL-P2-10 | 缺 XR/multi-camera | stereo/reflection/capture/editor view history 与 late-latch policy。 |

## 6. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| type/provider/catalog | Fail | asset type、editor provider、runtime capability and unavailable UI。 |
| plugin extension | Fail | drawer/toolkit/SceneMode/overlay/provider registration with lifecycle。 |
| document identity | Fail | projector/material/atlas IDs, schema, revision and migration。 |
| placement operation | Partial | generic Scene operations/transform host exists, but no Decal-specific factory/history。 |
| inspector schema | Partial | generic material/transform inspector host exists, but no Decal channel/receiver fields。 |
| volume gizmo | Fail | projector box/frustum, inverse transform, bounds, snap and validation。 |
| receiver preview | Fail | depth reconstruction, affected receiver and write-mask visualization。 |
| atlas/material UI | Fail | region picker, residency/generation and material graph diagnostics。 |
| compiler job | Fail | dependency graph, source spans, progress/cancel and artifact rollback。 |
| preview world | Fail | runtime artifact/provider install, real pixel executor and generation。 |
| runtime mirror | Fail | entity/world/view/generation, visible/culled/overflow/pipeline status。 |
| diagnostics | Fail | DBuffer/GBuffer/forward target, blend/depth/atlas/GPU/memory metrics。 |
| temporal/RT/shadow | Fail | motion history, reactive mask, receiver policy and reset diagnostics。 |
| transaction | Fail | projector/material/atlas edits use preflight, undo, dirty and savepoint。 |
| round-trip | Fail | source/component/artifact save/reopen/migrate preserve values and IDs。 |
| product scenes | Fail | Scene/PIE/standalone/render target/save/reopen receiver scenarios。 |
| fault UI | Fail | missing material/atlas, target mismatch, no-op/device loss, stale state。 |
| multi-view | Fail | main/reflection/shadow/capture/editor assignment and history policy。 |
| batch authoring | Fail | multi-select edits, deterministic order, partial failure and byte budget。 |
| collaboration | Fail | document lease, external rebase, conflict and provenance. |
| performance | Fail | compile/preview/GPU time, projector count, overdraw and atlas budgets。 |
| test coverage | Fail | property/roundtrip/operation/visual/GPU/fault/scale evidence。 |
| backend host | Partial | generic material/viewport/PreviewScene/capture host exists, Decal provider absent。 |
| quality truth | Fail | asset/provider/artifact/pixel/diagnostic receipts gate Ready/Executed labels。 |

本轮只写审查文档，未修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Editor/WGPU/PIE 动态验证。
