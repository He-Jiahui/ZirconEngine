---
title: Editor Vegetation、Tree、Foliage、Grass 与 Instancing 当前工作树复审
category: zircon_editor
report_id: Editor242
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
canonical_owner: Editor242
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/182-runtime-vegetation-tree-foliage-grass-current-working-tree-review.md
related_code:
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editing
  - zircon_editor/src/scene
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_plugins/terrain/editor
  - examples/vampire/assets/models/grass_billboard_static_batch.model.toml
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/FoliageEdit
  - dev/UnrealEngine/Engine/Source/Runtime/Foliage
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/InstancedStaticMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Classes/LandscapeGrassType.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Nature
  - dev/godot/scene/3d/multimesh_instance_3d.cpp
  - dev/bevy/crates/bevy_pbr
  - dev/Fyrox/fyrox-impl/src/scene
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor242 · Vegetation/Foliage/Grass authoring 当前工程化差距

## 1. 结论

当前 Editor 没有 Vegetation/Species/Prototype/Placement/InstanceSet asset factory、document、scatter brush、cluster inspector、LOD/impostor authoring 或 runtime preview。builtin registry 与 toolkit 没有植被类型；现有 Terrain/Foliage Workbench 只是 UI shell。`workbench_extension_foliage_editor_workspace.zui` 将根节点设为 `collapsed`，并硬编码 `Forest_A12`、`Forest_A13`、`River_02`、`Cliff_01` 与 `Oak/Fern/Grass/Shrub` rows，不能产生 source/artifact/instance receipt。

通用 Mesh/Model inspector、viewport selection/gizmo、Scene transaction、PreviewScene、capture/diagnostics 可复用，但没有 species metadata、placement seed/mask、paint/erase/normalize、cell/cluster identity、wind response、collision/nav policy、LOD crossfade、streaming generation 或 GPU instance metrics。历史 Editor138/16/92 的 Foliage 缺口仍成立；本次登记 **1 项继承 P0（Editor16 唯一计数）、28 项 P1、10 项 P2、24 道资格门**，P1 28 Open，P2 10 Open，资格门 21 Fail、3 Partial、0 Pass。

## 2. 当前源码证据

- builtin asset registry 没有 Species/Prototype/Placement/Cluster/Impostor/WindProfile 类型或 creation template/toolkit。
- Workbench preview/build routes 只投影静态 rows 与 queued/Ready 文案，没有 operation factory、document、compiler job、artifact generation 或 provider receipt。
- Scene/Inspector/viewport 没有 vegetation component、instance selection、brush stroke transaction、terrain projection、cluster/cell overlay 或 collision/nav preview。
- `grass_billboard_static_batch.model.toml` 是普通 Model 资产；不能证明 Foliage asset、instance buffer 或 HISM cluster。

## 3. 参考引擎差异

Unreal FoliageEdit 提供 foliage palette、paint/erase/resize、density/slope/ground alignment、per-instance selection、HISM clusters、transaction 与 landscape grass build；Unity GPUDriven/Nature 关联 LOD/billboard/wind/instance data；Godot MultiMesh 和 Bevy/Fyrox 仅提供批处理/场景编辑对照。Zircon 当前只有 collapsed fixture。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| ED-VEG-01 | 无 asset types | Species/Prototype/Placement/Cluster/Impostor/WindProfile 类型、factory、icons、thumbnail、reimport。 |
| ED-VEG-02 | 无 provider/catalog | editor provider/manifest、runtime capability、unavailable state、receipt-driven badge。 |
| ED-VEG-03 | 无 document | stable species/prototype/cell/cluster/instance IDs、revision、dirty/save/reopen/LKG/migration。 |
| ED-VEG-04 | 无 palette | source species/material/mesh LOD/card/impostor/collision/wind dependencies。 |
| ED-VEG-05 | 无 placement brush | paint/erase/scatter、density/radius/slope/altitude/normal/alignment/jitter/seed。 |
| ED-VEG-06 | 无 biome/mask | terrain/weightmap/spline/biome masks、include/exclude、deterministic sampling。 |
| ED-VEG-07 | 无 instance selection | per-instance/cell/cluster selection、isolate/lock、selection lease、batch edit。 |
| ED-VEG-08 | 无 transform/custom data | rotation/scale/color/phase/custom fields、finite/range/unit validation。 |
| ED-VEG-09 | 无 cluster/LOD view | HISM tree、bounds、screen error、mesh/card/impostor transition、crossfade。 |
| ED-VEG-10 | 无 wind authoring | WindField preview、species response、gust/turbulence and quality tier controls。 |
| ED-VEG-11 | 无 collision/nav | per-prototype collision, nav obstacle, terrain projection, pick/query visualization。 |
| ED-VEG-12 | 无 compiler job | dependency graph、source spans、progress/cancel、artifact generation/install/rollback。 |
| ED-VEG-13 | 无 preview world | runtime artifact/cluster install、fixed update、pause/step/reset、device/world generation。 |
| ED-VEG-14 | 无 live mirror | cell/cluster/instance counts、culling/LOD/streaming/wind/fallback provider status。 |
| ED-VEG-15 | 静态 Ready rows | Forest/River/Cliff sample values must come from runtime snapshot, not ZUI literals。 |
| ED-VEG-16 | 无 transaction | brush strokes、add/remove/move、prototype replace、cluster rebuild、undo/redo/savepoint。 |
| ED-VEG-17 | 无 roundtrip | document/artifact settings save/reopen/migrate preserve IDs and placement hash。 |
| ED-VEG-18 | 无 streaming UI | cell residency/priority/budget/overflow/late result and partition ownership。 |
| ED-VEG-19 | 无 diagnostics | compile/instance/GPU/memory/overdraw/LOD/wind/collision/nav timing。 |
| ED-VEG-20 | 无 product scene | Scene/PIE/standalone/terrain/nav/physics/audio/VFX/network/save end-to-end。 |
| ED-VEG-21 | 无 collaboration | document lease、external change/rebase、conflict and operation provenance。 |
| ED-VEG-22 | 无 fault UI | malformed species、provider/device/world loss、budget overflow、stale generation recovery。 |
| ED-VEG-23 | 无 performance | 1/10K/1M instances、compile time、preview FPS、memory/GPU budget gates。 |
| ED-VEG-24 | 无 tests | brush/property/roundtrip/compiler、cluster/LOD/wind、visual/fault/scale/soak tests。 |
| ED-VEG-25 | Terrain/Foliage 混淆 | Terrain layer/heightfield 与 Vegetation placement/provider 分开。 |
| ED-VEG-26 | fixture routes bypass | Preview/Build/row routes must dispatch typed operations, not only feedback strings。 |
| ED-VEG-27 | editor/runtime ABI | versioned neutral descriptors; editor never mutates runtime instance/GPU buffer directly。 |
| ED-VEG-28 | quality truth | no Ready/Executed until artifact, provider, cluster and render receipts all pass。 |

## 5. P2 增强任务

| ID | 差异 | 工程化方向 |
|---|---|---|
| ED-VEG-P2-01 | 缺生态/季节 authoring | seasonal density、growth/death、succession 与可重放生成控制。 |
| ED-VEG-P2-02 | 缺 PCG/biome graph | biome rule graph、spline/weightmap inputs、seed provenance。 |
| ED-VEG-P2-03 | 缺高级树体工具 | branch hierarchy、leaf flutter、interaction bend 与 LOD pose continuity。 |
| ED-VEG-P2-04 | 缺虚拟几何设置 | meshlet/virtual geometry admission、fallback、memory budget。 |
| ED-VEG-P2-05 | 缺 RT/GI 设置 | BLAS/alpha/shadow/GI invalidation、quality tier 与 visual diagnostics。 |
| ED-VEG-P2-06 | 缺 headless authoring | 无渲染 placement validation、collision/nav export 与 server preview。 |
| ED-VEG-P2-07 | 缺扩展协议 | custom species metadata、placement rule、material/wind adapter versioning。 |
| ED-VEG-P2-08 | 缺跨平台矩阵 | WebGPU/Vulkan/Metal/DX12 与低端设备的 feature/fallback UI。 |
| ED-VEG-P2-09 | 缺观测导出 | per-cell/species/LOD/cull/stream capture provenance 与离线报告。 |
| ED-VEG-P2-10 | 缺 canonical scenes | forest/grass/biome/partition scenes、golden hash 与性能历史。 |

## 6. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| asset/provider/catalog | Fail | types/factory/provider/capability and unavailable UI agree。 |
| document identity | Fail | stable species/prototype/cell/cluster/instance IDs and revision。 |
| palette/dependencies | Fail | mesh LOD/card/impostor/material/collision/wind dependency graph。 |
| brush placement | Fail | paint/erase/scatter, terrain/biome masks, deterministic seed and validation。 |
| instance selection | Partial | generic entity/viewport selection host exists, but no per-instance/cell/cluster lease。 |
| transform/custom data | Partial | generic transform editing exists, but no vegetation finite/range/custom-data schema。 |
| cluster/LOD authoring | Fail | HISM tree, bounds, screen error, card/impostor crossfade and hysteresis。 |
| wind/collision/nav | Fail | field preview, prototype collision, terrain projection and nav visualization。 |
| compiler job | Fail | dependency graph, source spans, cancel/progress, artifact install/rollback。 |
| preview world | Fail | runtime artifact install, fixed-step pause/step/reset and generation evidence。 |
| runtime mirror | Fail | live world/cell/cluster/instance/cull/LOD/stream/wind status。 |
| transaction | Fail | brush strokes, add/remove/move, rebuild, undo/redo and savepoint。 |
| round-trip | Fail | source/artifact settings save/reopen/migrate preserve IDs and placement hash。 |
| streaming UI | Fail | cell residency, priority, budgets, overflow and late-result rejection。 |
| diagnostics | Fail | compile/instance/GPU/memory/overdraw/LOD/wind/collision/nav timing。 |
| fault handling | Fail | malformed species/provider/device/world loss and stale generation recovery。 |
| performance | Fail | 1/10K/1M instances, compile time, preview FPS and memory/GPU budgets。 |
| product scenes | Fail | Scene/PIE/standalone/terrain/nav/physics/audio/VFX/network/save end-to-end。 |
| collaboration | Fail | document lease, external rebase, conflict and operation provenance。 |
| backend substrate | Partial | generic mesh/viewport/PreviewScene hosts exist, but no vegetation provider。 |
| fixture truth | Fail | static rows/routes cannot publish Ready/Build without runtime receipts。 |
| editor/runtime ABI | Fail | versioned neutral descriptors; UI cannot mutate runtime/GPU instance state。 |
| test coverage | Fail | brush/property/compiler/cluster/LOD/wind/visual/fault/scale/soak evidence。 |
| quality truth | Fail | artifact/provider/cluster/render receipts gate all status labels。 |

本轮只写审查文档，未修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Editor/PIE/GPU 动态验证。
