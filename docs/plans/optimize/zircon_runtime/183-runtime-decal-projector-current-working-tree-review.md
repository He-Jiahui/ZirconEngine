---
title: Runtime Decal、Projector、DBuffer/GBuffer 与 Receiver 当前工作树复审
category: zircon_runtime
report_id: Runtime183
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zw-runtime-decal-projector-material-domain-dbuffer-gbuffer-forward-receiver-culling-batching-atlas-streaming-temporal-rt-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/35-decal-projector-material-domain-dbuffer-gbuffer-forward-receiver-culling-batching-atlas-streaming-temporal-rt-scalability-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/243-editor-decal-projector-current-working-tree-review.md
related_code:
  - zircon_plugins/rendering/features/decals/runtime/src/lib.rs
  - zircon_plugins/rendering/features/decals/runtime/src/plugin.rs
  - zircon_plugins/rendering/features/decals/editor/src/plugin.rs
  - zircon_plugins/rendering/plugin.toml
  - zircon_runtime/src/graphics/scene/render_scene/component_projector
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/world/dynamic_components.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/DecalComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/CompositionLighting/PostProcessDeferredDecals.cpp
  - dev/godot/scene/3d/decal.h
  - dev/Fyrox/fyrox-impl/src/scene/decal.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/decal.shader
  - dev/bevy/crates/bevy_pbr/src/decal/clustered.rs
  - dev/bevy/crates/bevy_pbr/src/decal/forward.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal/DecalProjector.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal/DecalSystem.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime183 · Decal/Projector 当前工程化差距

## 1. 结论

Decal 是当前树中最危险的“临时实现”：`zircon_plugins/rendering/features/decals/runtime/src/lib.rs` 注册了 `DecalProjectorDescriptor`、component descriptor、PostProcess pass 和 executor，但 `noop_render_executor` 直接返回 `Ok(())`。因此 feature 可被记录为 registered/executed，却不产生任何 GPU work 或像素。descriptor 只有 mode、opacity、normal_blend、atlas_region，没有 material/texture/sampler、projection transform、bounds、receiver mask、fade、sort、generation、instance owner 或 extraction。

`BuiltinRenderFeature::Decal/Projector` 也仍处于 descriptor-only advanced slot。RenderScene projector component projector 只处理通用 mesh geometry journal，不是 Decal producer；Scene/Material asset 没有 Decal domain/typed source，DynamicScene 的 generic component payload也没有项目保存/渲染消费闭环。历史 Runtime148/35 已指出同一断路；本次刷新登记 **1 项继承 P0（保持由 Editor39/Plugins04 唯一计数）、30 项 P1、12 项 P2、26 道资格门**，P1 30 Open，P2 12 Open，资格门 22 Fail、4 Partial、0 Pass。

目标架构：

```text
DecalSource + material/atlas dependencies
  -> deterministic DecalCompiler
  -> projector/material/receiver/DBuffer artifact
  -> generation-qualified per-World DecalInstanceSet
  -> visible/cull/batch planner
  -> DBuffer/GBuffer/forward/RT receiver passes with real executor
  -> shadow/temporal/streaming/diagnostic/editor receipts
```

## 2. 当前源码证据

- `DecalProjectorDescriptor` 只有 4 个字段，且 `atlas_region: String` 没有 asset handle、validated region or residency lease。
- `render_feature_descriptor()` 声明读写 `scene-depth/scene-color` 的 PostProcess pass，但 `render_pass_executor_registration()` 绑定 `noop_render_executor`；缺 shader bindings、projection math、write masks、blend state、DBuffer targets、receiver classification。
- `decal_projector_component_descriptor()` 仅注册 mode/opacity/normal_blend/atlas_region，缺 stable ID、transform/bounds、material, fade, layer, sorting, lifetime 与 typed finite/range validators。
- plugin manifest/capability 只能说明 feature metadata；concrete provider behavior 为 no-op，registration success 不能当 capability truth。
- Material domain/Scene/FrameExtract/Visibility 没有 Decal payload、projection inverse、receiver channel、atlas generation、current/previous transform 或 per-view culling input。

## 3. 参考引擎差异

Unreal `DecalComponent` 与 deferred decal renderer 有 size/fade/normal/DBuffer/GBuffer/forward receiver、blend/write mask、view relevance、culling、sort 与 shader permutations；Godot/Fyrox 将 decal 作为持久化 scene node/material/shader；Bevy 同时有 clustered/forward extraction、storage/bindings、cluster assignment 与 WGSL；Unity HDRP 有 DecalProjector/DecalSystem、atlas、draw distance、layer/angle fade 与 editor inspector。Zircon 现在仍是注册+noop。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| RT-DECAL-01 | no material domain | MaterialDomain/asset schema 增加 Decal inputs/outputs、normal/ORM/emissive/opacity、write mask 与 migration。 |
| RT-DECAL-02 | descriptor 太窄 | typed projector source: material, transform, size, pivot, fade, layer, sort, lifetime, receiver mask。 |
| RT-DECAL-03 | no atlas contract | atlas asset/region/sampler residency、generation、UV transform、fallback与leak proof。 |
| RT-DECAL-04 | no compiler | deterministic material/technique/stage/target/permutation artifact、diagnostics、source map。 |
| RT-DECAL-05 | no Scene carrier | versioned Scene/DynamicScene DecalComponent、project save/reopen、clone/prefab and migration。 |
| RT-DECAL-06 | no World instance | per-World instance set、stable ID、generation、activation/retire、capacity、stale rejection。 |
| RT-DECAL-07 | no extract | RenderFrameExtract Decal SoA with inverse transform, bounds, material/atlas generations, flags, current/previous。 |
| RT-DECAL-08 | no culling | view-relative frustum/cluster/HZB/angle/distance fade and receiver classification。 |
| RT-DECAL-09 | no batching | material/atlas/DBuffer compatible batch key, sorting, indirect draw, overflow/compaction receipt。 |
| RT-DECAL-10 | noop executor | implement real DBuffer/GBuffer/forward receiver passes, shader bindings, pipeline and write masks。 |
| RT-DECAL-11 | stage mismatch | resolve geometry/depth/color dependencies and avoid postprocess self-read/write hazards。 |
| RT-DECAL-12 | no receiver contract | opaque/deferred/forward/transparent/terrain/particle receiver policy with typed compatibility errors。 |
| RT-DECAL-13 | no normal/roughness semantics | tangent/normal blend, roughness/metallic/emissive/opacity equations and premultiplied policy。 |
| RT-DECAL-14 | no shadow policy | decal affects shadow/indirect/GI/RT policy, receiver masks and invalidation. |
| RT-DECAL-15 | no temporal | projector motion history, reactive mask, TAA/velocity policy, camera cut/teleport reset。 |
| RT-DECAL-16 | no streaming | material/texture/atlas residency, priority, budget, LKG, late result rejection。 |
| RT-DECAL-17 | no dynamic lifecycle | spawn/update/remove/expiry/fade in typed command and receipt path。 |
| RT-DECAL-18 | no physics/query | pick/raycast receiver, surface projection, collision/terrain attach and query generation。 |
| RT-DECAL-19 | no diagnostics | visible/culled/overflow, target/channel, atlas, pipeline, GPU time, memory/fallback metrics。 |
| RT-DECAL-20 | false capability | no-op provider must be Disabled/Unsupported until pixel receipt proves execution。 |
| RT-DECAL-21 | no tests | projection math, blend/write masks, depth/receiver, atlas, batch, temporal, cross-backend GPU golden。 |
| RT-DECAL-22 | no fault policy | invalid material, missing texture/atlas, target mismatch, device loss and stale generation terminal states。 |
| RT-DECAL-23 | no network/save | persistent/runtime decals, authority, replication, replay, save slot and deterministic expiry。 |
| RT-DECAL-24 | no multi-view | main/reflection/shadow/capture/editor view assignment/history/residency policy。 |
| RT-DECAL-25 | no large world | camera-relative inverse, precision, origin shift and partition residency。 |
| RT-DECAL-26 | no editor bridge | editor source/operation -> runtime compiler/provider/artifact/pixel receipt。 |
| RT-DECAL-27 | descriptor-only advanced slot | concrete provider must own slot or it remains explicitly unavailable。 |
| RT-DECAL-28 | thread ownership unclear | immutable extract, resource lease, graph execution and completion fence。 |
| RT-DECAL-29 | product integration absent | Scene/PIE/standalone/save/reopen plus material/terrain/mesh/transparent receiver scenarios。 |
| RT-DECAL-30 | quality gates absent | CPU oracle, GPU capture, visual diff, fault/scale/soak and performance budgets。 |

## 5. P2 增强任务

| ID | 差异 | 工程化方向 |
|---|---|---|
| RT-DECAL-P2-01 | 缺投影纹理压缩策略 | BC/ASTC/ETC、normal/ORM packing、mip bias 与跨平台 artifact。 |
| RT-DECAL-P2-02 | 缺虚拟纹理/streaming | atlas/virtual page residency、feedback、priority、eviction 与 LKG。 |
| RT-DECAL-P2-03 | 缺贴花排序 | deterministic layer/sort key、overlap policy 与 receiver conflict resolution。 |
| RT-DECAL-P2-04 | 缺地形/水体 receiver 扩展 | terrain、water、foliage、particle 等特殊 receiver 的 channel contract。 |
| RT-DECAL-P2-05 | 缺烘焙/静态路径 | static decal bake、lightmap/GI invalidation 与 runtime/mobile fallback。 |
| RT-DECAL-P2-06 | 缺网络寿命策略 | authoritative spawn/expiry、prediction、replay and join-in-progress codec。 |
| RT-DECAL-P2-07 | 缺 editor capture | projector/material/atlas provenance 与 GPU capture 的可追踪关联。 |
| RT-DECAL-P2-08 | 缺动态资源回收 | retired material/atlas/pipeline leases、fence reclamation 与 leak test。 |
| RT-DECAL-P2-09 | 缺跨 backend shader golden | Vulkan/Metal/DX12/WebGPU blend、depth and channel visual parity。 |
| RT-DECAL-P2-10 | 缺 XR/multi-camera policy | stereo/late-latch/reflection/capture view history and culling rules。 |
| RT-DECAL-P2-11 | 缺安全输入治理 | material graph recursion、texture dimensions、projector extent and budget limits。 |
| RT-DECAL-P2-12 | 缺 benchmark corpus | dense projected decals、overlap/overdraw、stream/device-loss regression scenes。 |

## 6. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| plugin capability registration | Partial | descriptor/pass admission exists, but capability cannot publish Ready before pixels。 |
| component descriptor | Partial | four fields are exposed, but no typed material/transform/identity schema。 |
| graph pass declaration | Partial | named PostProcess pass exists, but dependencies and target hazards are incomplete。 |
| projector scene adjacency | Partial | generic projector geometry carrier exists, but no Decal producer/extract owner。 |
| resource taxonomy | Fail | DecalMaterial/Projector/Atlas resource kinds and AssetKind mapping。 |
| source schema | Fail | transform/size/fade/layer/receiver/lifetime/material dependencies round-trip。 |
| compiler artifact | Fail | deterministic technique/permutation/pipeline artifact, source map and hash。 |
| stable identity | Fail | projector/material/atlas IDs, generation, activation and retire semantics。 |
| per-World authority | Fail | one DecalInstanceSet owner with stale-result rejection and capacity policy。 |
| extraction | Fail | RenderFrameExtract SoA with inverse transform, bounds and resource generations。 |
| culling | Fail | frustum/cluster/HZB/angle/distance receiver-aware visibility evidence。 |
| batching | Fail | material/atlas/DBuffer-compatible deterministic batch and overflow receipt。 |
| render executor | Fail | real DBuffer/GBuffer/forward pass writes pixels; no `noop_render_executor`。 |
| receiver policy | Fail | opaque/deferred/forward/transparent/terrain/particle compatibility errors。 |
| blend semantics | Fail | normal/roughness/metallic/emissive/opacity equations and write masks。 |
| shadow/GI/RT | Fail | receiver policy, invalidation and alpha/depth behavior across views。 |
| temporal history | Fail | projector/camera motion, reactive mask, cut/teleport reset and velocity。 |
| streaming/residency | Fail | texture/atlas/pipeline admission, LKG, priority and late rejection。 |
| dynamic lifecycle | Fail | typed spawn/update/remove/expiry/fade commands and receipts。 |
| query/physics | Fail | surface pick/raycast/projection and generation-synchronized attachment。 |
| diagnostics | Fail | visible/culled/overflow/channel/atlas/pipeline/GPU/memory metrics。 |
| fault/device loss | Fail | invalid source/target, missing resources and device loss terminal states。 |
| network/save/replay | Fail | persistent/runtime state, authority, deterministic expiry and round-trip。 |
| editor bridge | Fail | editor source/operation compiles, installs and reports pixel receipt。 |
| performance/product | Fail | multi-view, thousands of projectors, PIE/standalone/save and frame budgets。 |
| benchmark corpus | Fail | canonical overlap/overdraw/stream/device-loss scenes with stable GPU baselines。 |

本轮仅写审查文档，没有修改 Decal 生产代码、测试、Cargo、ABI 或 ZUI，也未运行 WGPU/GPU capture/PIE 动态验证。
