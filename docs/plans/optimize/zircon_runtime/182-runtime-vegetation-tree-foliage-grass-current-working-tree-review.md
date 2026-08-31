---
title: Runtime Vegetation、Tree、Foliage、Grass 与 Instancing 当前工作树复审
category: zircon_runtime
report_id: Runtime182
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zv-runtime-vegetation-tree-foliage-grass-species-instancing-wind-animation-billboard-impostor-lod-streaming-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/34-vegetation-tree-foliage-grass-species-instancing-wind-animation-billboard-impostor-lod-streaming-scalability-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/242-editor-vegetation-tree-foliage-grass-current-working-tree-review.md
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/scene_renderer
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/core/framework/render
  - zircon_plugins/terrain
  - zircon_plugins/first_party_runtime_catalog
  - examples/vampire/assets/models/grass_billboard_static_batch.model.toml
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Foliage
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/InstancedStaticMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/HierarchicalInstancedStaticMesh.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeGrass.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Classes/LandscapeGrassType.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SpeedTreeWind.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Shaders/Nature
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Nature
  - dev/godot/scene/resources/multimesh.cpp
  - dev/godot/scene/3d/multimesh_instance_3d.cpp
  - dev/bevy/crates/bevy_render
  - dev/bevy/crates/bevy_pbr
  - dev/Fyrox/fyrox-graphics
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime182 · Vegetation/Tree/Foliage/Grass 当前工程化差距

## 1. 结论

当前 Zircon 没有 Vegetation、Tree、Foliage、Grass、SpeedTree 或 foliage cluster runtime 产品。`ResourceKind`/`ImportedAsset`/Scene component 没有 species、prototype、placement、instance-set 或 vegetation artifact；生产路径对 `VegetationSpecies/FoliageAsset/GrassAsset/SpeedTree/Impostor/WindField/FoliageShading` 没有领域 owner。少量 `foliage` 只在 shader prewarm 参数和测试夹具出现，不能形成能力。

`BuiltinRenderFeature::Tree`、`Terrain`、`Billboard` 与 `MeshLod` 被列入 `DESCRIPTOR_ONLY_ADVANCED_SLOTS`；这证明有 descriptor 名称，不证明有 extract、instance buffer、cluster culling、wind deformation 或 draw executor。GPU Scene/visibility、普通 mesh LOD、alpha mask、shadow、velocity、artifact cache 与 quality profile 是可复用基础，但不能替代 species source、placement compiler、HISM/cluster hierarchy、impostor/card、per-instance wind、bounds/LOD/streaming contracts。

Foliage Workbench 仍将 `Forest_A12` 显示为 `Oak 0.72 Ready`、`River_02` 显示为 `Grass 0.81 Queued`，这些是静态 ZUI rows。历史 Runtime147/34 的无 production owner 结论仍成立；本次登记 **0 项新 P0、30 项 P1、12 项 P2、26 道资格门**，P1 30 Open，P2 12 Open，资格门 23 Fail、3 Partial、0 Pass。Editor 静态 Ready 误导由 Editor242/其父 owner 负责。

目标架构：

```text
VegetationSpeciesSource + placement/provenance
  -> deterministic VegetationCompiler
  -> prototype/mesh LOD/card/impostor/wind/collision artifact
  -> generation-qualified cell/cluster/instance-set runtime
  -> GPU instance/culling/LOD/streaming + terrain/nav/physics adapters
```

## 2. 当前源码证据

- `zircon_runtime_interface/src/resource/marker.rs:8-31` 没有 Vegetation/Species/Foliage/Placement/Impostor/InstanceSet 类型。
- `zircon_runtime/src/asset/assets/imported.rs:21-44,116-147` 没有 vegetation importer、prototype subasset、placement artifact 或 source provenance。
- `zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs` 将 Tree/Terrain/Billboard/MeshLod 全部作为 descriptor-only slot；没有对应 production render feature executor。
- Scene carrier 只有 MeshRenderer/Transform/Physics/Animation，不能持久化 species/instance transform、random seed、cell/cluster、wind response、collision/nav policy。
- 例子中的 `grass_billboard_static_batch.model.toml` 和 WOC foliage glTF 是普通 Model/Mesh 资产；没有 runtime instance-set identity、placement generation 或 cluster artifact。

## 3. 参考引擎差异

Unreal Foliage/InstancedStaticMesh/HISM 把 prototype、per-instance transform/custom data、cluster tree、instance culling、nanite/LOD、collision、foliage editing 与 landscape grass type 分层，SpeedTree wind 是独立输入。Unity GPUDriven/Nature 管线具有 instance data、LOD group、billboard/impostor、wind/vegetation shaders；Godot MultiMesh 只提供实例 buffer 基础，Bevy/Fyrox 只提供批处理/场景对照。Zircon 目前连普通 instance data 的领域 owner 都没有。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| RT-VEG-01 | 无资源 taxonomy | VegetationSpecies/Prototype/Placement/Cluster/Impostor/WindProfile/CachedCell 类型与 AssetKind 映射。 |
| RT-VEG-02 | 无 source/artifact | source 保存 species/placement/seed；artifact 固化 parts/LOD/cards/impostor/wind/collision/cluster。 |
| RT-VEG-03 | 无 importer/compiler | glTF/FBX/SpeedTree/texture import、units、pivot、normal/alpha validation、deterministic build。 |
| RT-VEG-04 | 无 stable identity | species/prototype/cell/cluster/instance IDs 跨 reimport、streaming、save、network、replay 稳定。 |
| RT-VEG-05 | 无 placement | brush/scatter/biome/spline/terrain projection、density/jitter/slope/altitude/mask/seed 编译。 |
| RT-VEG-06 | 无 cluster hierarchy | HISM/cluster tree、bounds、LOD screen error、frustum/HZB/occlusion 与 multi-view culling。 |
| RT-VEG-07 | 无 instance runtime | per-World cell/cluster/instance-set lifecycle、generation、stream-in/out、retire、capacity。 |
| RT-VEG-08 | 无 GPU instance data | storage/indirect instance buffers、custom data、dirty ranges、compaction、fence/device generation。 |
| RT-VEG-09 | 无 LOD | mesh/card/impostor transition、crossfade、wind continuity、hysteresis、budget/quality tier。 |
| RT-VEG-10 | 无 wind | shared WindField snapshot、species response、gust/turbulence、phase/branch/leaf deformation。 |
| RT-VEG-11 | 无 material | foliage two-sided/alpha/coverage/SSS/transmission/leaf normal/thickness and shadow policy。 |
| RT-VEG-12 | 无 render owner | extract、visibility、depth/GBuffer/forward、shadow、velocity、RT/GI 与 instance receipt。 |
| RT-VEG-13 | 无 collision | per-prototype collision family、instance query/pick、physics/nav obstacle policy。 |
| RT-VEG-14 | 无 terrain binding | terrain cell/height/normal/material dependency、partial rebuild、world partition handoff。 |
| RT-VEG-15 | 无 streaming | cell/chunk residency、priority/importance、memory/bandwidth budget、late result rejection。 |
| RT-VEG-16 | 无 interaction | wind/bend/flatten/harvest/burn/damage state、typed gameplay/VFX/audio events。 |
| RT-VEG-17 | 无 network/save | authoritative placement/state、quantized instance deltas、join-in-progress、save/replay。 |
| RT-VEG-18 | 无 diagnostics | species/cell/cluster/instance counts、culling/LOD/overdraw/GPU time/memory/overflow。 |
| RT-VEG-19 | 无 failure policy | malformed prototype、artifact loss、budget overflow、device/world loss、stale output terminal state。 |
| RT-VEG-20 | 无 tests | compiler/property、placement determinism、cluster culling、LOD/wind parity、stream/fault/scale/soak。 |
| RT-VEG-21 | 普通 Mesh 误充 foliage | capability truth 区分 Mesh/Model 与 Vegetation/InstanceSet。 |
| RT-VEG-22 | descriptor-only slots | Tree/Billboard/Terrain slots 必须绑定 concrete provider 或明确 unavailable。 |
| RT-VEG-23 | 例子绕行 | grass billboard/static batch/WOC foliage 迁移到正式 source/placement/artifact 路径。 |
| RT-VEG-24 | bounds 不真实 | instance/cluster dynamic bounds、origin shift、precision 与 culling history。 |
| RT-VEG-25 | 大世界未定义 | world-partition cell IDs、origin rebasing、streaming generation、replay coordinates。 |
| RT-VEG-26 | 线程 authority 不明 | immutable placement snapshot、GPU lease、render completion 与 simulation ownership。 |
| RT-VEG-27 | editor/runtime 断裂 | Editor source/brush transaction -> runtime compiler/cluster receipt，禁止 fixture。 |
| RT-VEG-28 | quality gate 缺失 | 1/10K/1M instance、memory、CPU/GPU frame、LOD/shadow/streaming budgets。 |
| RT-VEG-29 | visual fallback 未声明 | unsupported backend 只允许 typed billboard/mesh fallback，不能静默成功。 |
| RT-VEG-30 | product integration 缺失 | Scene/PIE/standalone、terrain/nav/physics/audio/VFX/network/save 全链验证。 |

## 5. P2 增强任务

| ID | 差异 | 工程化方向 |
|---|---|---|
| RT-VEG-P2-01 | 缺生态演替 | species succession、seasonal density、growth/death 与确定性事件源。 |
| RT-VEG-P2-02 | 缺程序化生成扩展 | biome rule graph、PCG adapter、seed provenance 与可重放增量生成。 |
| RT-VEG-P2-03 | 缺高级树体变形 | branch hierarchy、leaf flutter、interaction bend 与跨 LOD pose continuity。 |
| RT-VEG-P2-04 | 缺虚拟几何策略 | dense foliage 的 meshlet/virtual geometry admission、fallback 与内存预算。 |
| RT-VEG-P2-05 | 缺 RT/GI 专项 | BLAS instance policy、alpha test、wind update、GI invalidation 与质量分级。 |
| RT-VEG-P2-06 | 缺 server/headless 表达 | 无渲染实例 query、碰撞/nav/采集状态与低成本 authoritative snapshot。 |
| RT-VEG-P2-07 | 缺扩展协议 | 自定义 species metadata、placement rule、material/wind adapter 的版本化插件接口。 |
| RT-VEG-P2-08 | 缺跨平台基线 | WGPU backend、integrated/discrete GPU 与低端设备的 feature/fallback matrix。 |
| RT-VEG-P2-09 | 缺观测导出 | per-cell/species/LOD/cull/stream 数据导出、capture provenance 与离线分析。 |
| RT-VEG-P2-10 | 缺差分构建 | source dependency change 到局部 cell/cluster artifact rebuild 的最小失效集。 |
| RT-VEG-P2-11 | 缺大规模基准语料 | forest/grass/biome/streaming canonical scenes、golden hash 与性能历史。 |
| RT-VEG-P2-12 | 缺可访问调试视图 | 色盲安全 LOD/cell/cluster overlays、文本统计与自动化可读快照。 |

## 6. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| resource taxonomy | Fail | typed Species/Prototype/Placement/Cluster/Impostor/WindProfile resource kinds。 |
| source schema | Fail | versioned source、unknown-field preservation、units、seed and dependency provenance。 |
| compiler artifact | Fail | deterministic build、hash、diagnostics、source map and target-qualified artifact。 |
| stable identity | Fail | species/prototype/cell/cluster/instance IDs survive rebuild、stream and save。 |
| per-World authority | Fail | one generation-qualified lifecycle owner with replace/unload/drain semantics。 |
| placement semantics | Fail | brush/biome/spline/terrain rules compile deterministically。 |
| generic mesh substrate | Partial | ordinary mesh/model assets exist, but no vegetation-specific source/artifact contract。 |
| GPU instance substrate | Partial | GPU Scene/buffer infrastructure exists, but no foliage instance-set owner or receipt。 |
| visibility substrate | Partial | generic visibility/LOD exists, but no cluster hierarchy/card/impostor transition。 |
| cluster culling | Fail | HISM-style hierarchy、multi-view frustum/HZB/occlusion and overflow evidence。 |
| LOD/card/impostor | Fail | stable thresholds、crossfade、hysteresis、wind continuity and quality tiers。 |
| wind deformation | Fail | shared WindField generation and branch/leaf response reach render execution。 |
| foliage material | Fail | two-sided/alpha/coverage/SSS/transmission/thickness semantics and validation。 |
| shadow/velocity/GI | Fail | animated bounds、velocity、shadow、GI/RT update and visual parity receipts。 |
| terrain binding | Fail | height/normal/layer dependencies and partial rebuild use typed generations。 |
| streaming/residency | Fail | cell admission、priority、memory/bandwidth budgets and late-result rejection。 |
| physics/query | Fail | prototype collision、instance pick/query and lifecycle synchronization。 |
| navigation | Fail | obstacle/export policy、dirty regions and rebuild receipts。 |
| interaction/gameplay | Fail | bend/flatten/harvest/burn/damage state and typed events。 |
| network authority | Fail | placement/state replication、join-in-progress and deterministic deltas。 |
| save/replay | Fail | source/runtime state、generation、seed and instance deltas round-trip。 |
| editor bridge | Fail | document/brush transaction compiles and installs through runtime receipts。 |
| diagnostics | Fail | counts、culling、LOD、overdraw、GPU time、memory and overflow are observable。 |
| failure/device loss | Fail | malformed source、budget/provider/device/world loss terminate explicitly。 |
| scalability/performance | Fail | 1/10K/1M instance frame/memory/stream budgets with regression baselines。 |
| product integration | Fail | Scene/PIE/standalone/terrain/nav/physics/audio/VFX/network/save end-to-end。 |

本轮仅写审查文档，未修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 vegetation/GPU/PIE 动态验证。
