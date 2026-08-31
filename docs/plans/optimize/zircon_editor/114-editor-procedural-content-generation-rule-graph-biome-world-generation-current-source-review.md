---
title: Editor Procedural Content Generation、Rule Graph、Biome 与 World Generation 当前源码复核
category: zircon_editor
report_id: Editor114
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor40
refreshes:
  - docs/plans/optimize/zircon_editor/40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_scatter_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/prefab_and_scatter.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs
  - zircon_plugins/terrain
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/extensions.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - examples/woc/contracts/m3_terrain_content.json
  - examples/woc/tools/m3_terrain_content_source_extract.mjs
  - examples/woc/tools/m3_terrain_content_codegen.mjs
  - examples/woc/tools/m3_decoration_candidate_source_extract.mjs
  - examples/woc/scripts/woc_game/src/world/terrain_content.zr
  - examples/woc/scripts/woc_game/src/world/terrain_noise.zr
  - examples/woc/scripts/woc_game/src/world/terrain_shape.zr
  - examples/woc/scripts/woc_game/src/world/terrain_mountains.zr
  - examples/woc/scripts/woc_game/src/world/terrain_height.zr
  - examples/woc/scripts/woc_game/src/world/terrain_ground.zr
  - examples/woc/scripts/woc_game/src/world/terrain_gradient.zr
  - examples/woc/scripts/woc_game/src/world/decoration_candidate.zr
  - examples/woc/scripts/woc_game/src/world/collision_grid.zr
tests:
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/extensions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - examples/woc/contracts/m3_terrain_content.json
  - examples/woc/scripts/woc_game/src/world/terrain_noise.zr
  - examples/woc/scripts/woc_game/src/world/decoration_candidate.zr
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGComponent.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGGraph.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGNode.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGPin.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGData.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/Data/PCGSpatialData.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/Data/PCGPointData.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGManagedResource.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGSubsystem.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/Grid/PCGPartitionActor.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Private/Graph/PCGGraphCache.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Private/Graph/PCGGraphCompiler.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Private/Graph/PCGGraphExecutor.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Private/Graph/PCGGraphCompiler.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor/Private/Widgets/SPCGEditorGraphDeterminism.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor/Private/Widgets/SPCGEditorGraphDiff.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor/Private/Widgets/SPCGEditorGraphProfilingView.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor/Private/Widgets/SPCGEditorGraphLogView.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor/Private/Widgets/SPCGEditorGraphAttributeListView.cpp
  - dev/godot/modules/noise/fastnoise_lite.h
  - dev/godot/modules/noise/noise_texture_2d.cpp
  - dev/godot/scene/resources/multimesh.h
  - dev/Fyrox/fyrox-impl/src/scene/terrain/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain/quadtree.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain/brushstroke/mod.rs
  - dev/Fyrox/editor/src/interaction/terrain.rs
  - dev/Fyrox/editor/src/scene/commands/terrain.rs
  - dev/bevy/crates/bevy_asset/src/event.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 114 · Editor Procedural Content Generation / Rule Graph / Biome / World Generation 工程化差距

## 1. 结论

当前 Zircon 没有引擎级 PCG 产品。生产代码中不存在 `PcgGraph`、`RuleGraph`、typed node/pin、spatial data、compiler、executor、graph cache、generation request、partition scheduler、managed generated resource、generation receipt 或 PCG asset kind。`WorldGeneration` 命中只是 World revision/cache invalidation，不是世界生成系统。

Editor 却展示了固定的 Scatter Rule Graph：`SC_Forest`、Biome Mask、Slope Filter、Rocks + Ferns、64K instances、1 conflict；Generate/Validate 只返回 queued，seed/density 只是模板字符串。这是 P0 truthfulness 断路，不是“功能尚未丰富”。Terrain 也不能充当 Scatter backend：TerrainAsset 只是 height array/layers，Terrain importer 明确 DiagnosticOnly，缺 chunk/LOD/edit layer/streaming/generation/cook artifact。

WOC 是值得保留的项目级确定性基础：固定 source commit、sentinel/catalog/road/camp/lake digest、noise/height/slope known vectors、10-yard candidate lattice 与 collision cell replay。它证明数据 pin、seed stream 和局部重演可工程化，但仍是游戏专用脚本投影：zone/biome/order、candidate branch、terrain layer、road 查询都写死，没有 typed graph、generated output ownership、incremental cache、diff/manual override、cell artifact 或 renderer/Scene/nav/collision install。不能把 WOC 脚本包进 Generate 按钮就宣称 PCG。

目标边界应为：

`PcgGraphSource + typed Node/Pin/Data + seed/parameter/partition -> compiler -> deterministic CompiledPcgProgramArtifact -> bounded incremental executor/cache -> immutable GenerationOutput/ManagedResource -> Terrain/Foliage/Prefab/Spline/Road/Collision/Nav/Render/Cook typed adapters -> Editor graph/document/debug/receipt`

PCG 只能产出静态 placement/definition；Gameplay Spawn Rules 的 live authority 仍归 Editor28/Runtime gameplay 域，不得由 graph 直接写入 live population。

## 2. 审查范围与证据

### 2.1 当前工作树物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 指纹 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin/WOC selected | **38 / 10,833 / 10,298 / 362,683 / 13** | Scatter workspace/binding、Terrain、authoring/scene extensions、WOC source/extract/codegen/terrain/decoration/collision；`6a98b10f607b6dcba75406c37f40f1bbe1503217ffc8f933f3be5637ed164502` |
| Unreal/Godot/Fyrox/Bevy/Unity reference | **31 / 27,167 / 23,345 / 1,109,231 / 11** | Unreal PCG graph/data/compiler/cache/executor/partition/editor、Godot noise/MultiMesh、Fyrox terrain/brush、Bevy assets/mesh、Unity GPU instance/culling；`69b9784d813f443a7e9518d0b27dc33280d306284815f812766997c7dbe7e920` |
| Zircon selected union | **69 / 38,000 / 33,643 / 1,471,914 / 24** | current physical working tree union；`a57ecc275e5df4347037ef264525ce628cacad36e23a630b57100ddd95e40a16` |

统计按 selected root 去重、排序后以 UTF-8 内容计算 SHA-256；test 数只是属性计数。当前 baseline epoch 524，工作树含无关在途修改；实施前必须重导 manifest/fingerprint。WOC/Tooling 生成合同只做静态引用核对，本轮没有运行 Tooling lane、Cargo、PCG graph、determinism、incremental cache、partition cook、Editor transaction 或 64K instance 动态验证。

### 2.2 Product/Scatter/Terrain facts

1. ResourceKind 没有 PCGGraph、RuleGraph、Biome、WorldRecipe、PointData、GeneratedSet；没有 PCG plugin/package/catalog。
2. production 没有 typed node/pin/edge/data registry、type-check、cycle diagnostic、compiler、executor、cache、generation request 或 partition scheduler。
3. Scatter workspace 固定 `SC_Forest`/`Rule_Rocks`/`Rule_Ferns`、4 行表格、64K/1 conflict；没有 canvas node/pin/edge/document。
4. Biome Mask/Slope Filter/Spawn Rule/Collision Test 都是固定字符串；Generate/Validate 只返回 queued。
5. seed/density 是整段字符串，不是 typed parameter、range、seed stream 或 deterministic receipt。
6. 没有 `TerrainAsset` 以外的 PCG input；TerrainAsset 只有 height array/layers，Scene 只保存 reference。
7. Terrain importer 是 DiagnosticOnly，明确 backend 未安装；没有 chunk/LOD/edit layer/brush/cook/stream artifact。
8. 没有 generated entity/instance/resource owner，无法区分 generator output 与 author edits。
9. 没有 first-party Editor/Runtime provider、AssetType、factory、controller、toolkit 或 App feature closure。

### 2.3 WOC facts and limits

1. WOC 固定 source commit，extractor 校验 zones/biomes/lakes/camps/roads/docks/Sowfield bounds 与 SHA-256 catalog。
2. terrain noise/height/ground/slope/decoration/collision 使用固定 seed offsets、known vectors、10-yard lattice、cell replay。
3. source pin、sentinel、digest、known vector 和局部 collision replay 是可迁移的 deterministic corpus。
4. 但 zone/biome 是固定 Z 区间/code，terrain layer 是手写函数顺序，candidate kind/density 是分支常量。
5. 每次 candidate 会重复执行 terrain/road/camp/slope，road 查询 O(total segments)，无 node/result cache。
6. 输出是 Zr script functions，不是 point data、placement artifact、renderer instance buffer、Scene entity、nav/collision input 或 cell manifest。
7. 没有 stable generated IDs、provenance、add/update/remove diff、manual override/exclude/detach/bake/unbake/orphan policy。
8. 没有 Editor selection/node execution/preview region/before-after diff、job/receipt、incremental invalidation 或 rollback。
9. WOC world renderer 的 generateDecorations 与 Zircon 的局部 collision replay 不能证明 64K render instances 已由 Engine 生成。

### 2.4 Reference routing

1. Unreal PCG graph/node/pin/data/compiler/executor/cache/partition/managed resource 是产品主参考。
2. Godot FastNoiseLite/NoiseTexture 是可序列化生成 primitive；MultiMesh 是高规模实例 consumer，不是 PCG editor。
3. Fyrox Terrain chunk/quadtree/brush/undo 是 Terrain product 参考，不等同于 PCG graph。
4. Bevy AssetEvent 与 mesh extraction/batching 是增量 consumer 参考，无 PCG editor 产品。
5. Unity GPU instance/culling 是输出后端参考，不代表其 Graphics 仓包含完整 PCG/Terrain authoring。

## 3. P0：必须先关闭的断路（5 项，全部 Open）

### P0-1：PCG graph/compiler/executor/product identity 缺失

没有 source、node/pin/data schema、compiler、executor、cache、generation artifact、managed resource 或 AssetType。必须先建立 typed PCG core，不得从 Scatter UI 反推实现存在。

### P0-2：Scatter Workbench 是固定假 authority

SC_Forest、64K instances、1 conflict、seed/density、queued feedback 不读取任何真实 data/graph/job/runtime。必须 fail-close 或接入真实 document/compiler/receipt。

### P0-3：Terrain backend 未安装，不能承接 PCG output

Terrain importer 明确 DiagnosticOnly，Scene/World/render/physics/nav 没有完整 Terrain consumer。必须先有 Terrain artifact/streaming/managed output contract，再设计 scatter adapter。

### P0-4：WOC 确定性脚本不是引擎生成 artifact

WOC 的 hash/known vector/局部 cell replay 可保留，但输出是专用 Zr functions，无 stable output owner/provenance/diff/cache/cook/install。必须迁移 source corpus，不得包裹成按钮。

### P0-5：生成物与 Gameplay/Scene/Render/Collision/Nav authority 未隔离

没有 generated resource lifecycle、manual override、orphan cleanup、partition cell、runtime install；PCG 若直接写 live World 会与 Spawn/Scene/Editor authority 冲突。必须先定义 typed adapters 和 ownership。

## 4. P1：Runtime、Graph、Generation、Editor 与 Release（70 项，全部 Open）

1. 定义 versioned `PcgGraphSource`、graph/document revision、unknown-field migration。
2. 定义 typed Node/Pin/Data registry、edge compatibility、subgraph/parameter schema。
3. 定义 spatial/point/surface/volume/spline/attribute data layouts 与 ownership。
4. 建立 graph cycle/type/parameter/seed/bounds diagnostics 与 fail-close validation。
5. 编译 deterministic `CompiledPcgProgramArtifact`、topological task schedule 与 dependency graph。
6. 记录 source/input/tool/schema/algorithm/platform/seed key 与 provenance。
7. 建立 generation request：world/cell/bounds/quality/seed/authority/revision。
8. 建立 bounded executor：budget、priority、pause、cancel、retry、shutdown drain。
9. 建立 node/result cache、dependency key、stale reject、last-known-good artifact。
10. 建立 generation state machine、job progress、stage receipt、diagnostic journal。
11. 建立 partition/cell scheduler、interest radius、prefetch、eviction、rollback。
12. 建立 immutable `GenerationOutput`、stable item IDs、add/update/remove diff。
13. 建立 managed generated resource ownership、soft release/reuse/cleanup/orphan policy。
14. 区分 author-created、generator-created、baked/detached/manual override item。
15. Scene/Prefab/World/PIE save/load 保留 graph/output/override/provenance identity。
16. 生成 output 只经 typed Terrain/Foliage/Prefab/Spline/Road adapters。
17. 生成 output 只经 typed Collision/Navigation/Render/Cook adapters。
18. 禁止 PCG 直接写 Gameplay live population，提供 SpawnDefinition adapter。
19. Terrain source/artifact：chunk、LOD/quadtree、height/weight/edit/hole/normal/bounds/revision。
20. Terrain importer 从 DiagnosticOnly 变为唯一 compiler/provider，支持 clean headless cook。
21. Terrain brush stroke/undo/redo/transaction/partial dirty/chunk rebuild。
22. Biome source 支持 field/mask/layer/priority/blend/season/altitude/slope/water queries。
23. Scatter point generation 支持 density/seed/scale/rotation/normal/attribute/constraints。
24. Rule node 支持 sample/filter/transform/merge/partition/mesh/prefab/attribute output。
25. 支持 deterministic random stream per node/item/cell，不共享隐式 global RNG。
26. 支持 deterministic spatial query、broadphase/BVH/grid、batch/SIMD、cache。
27. 支持 spline/road/river/region/weather/nav inputs via Editor39/37/38 typed adapters。
28. 支持 surface/terrain/material/physics/nav queries with generation-qualified snapshot。
29. 输出 prefab/mesh/material/texture references 使用 typed asset handles 与 dependency manifest。
30. 输出 instance buffer/cluster/LOD/culling/bindless/render backend，不展开无界 entities。
31. 输出 collision shapes/compound/filter/authority 与 Physics cook receipt。
32. 输出 nav area/modifier/agent filters/tile invalidation 与 bake receipt。
33. output streaming/cell package/bulk chunks/patch/dedupe/GC/rollback。
34. output quality tiers/LOD/density/budget/GPU/VRAM/CPU/memory policy。
35. output manual override/detach/exclude/pin/seed-lock/orphan reconciliation。
36. graph diff/profiling/log/attribute viewer/determinism/debug object tree。
37. Editor graph canvas/node palette/pin drag/type errors/selection/zoom/pan/clipboard。
38. Editor document transaction/dirty/save/autosave/recovery/conflict/undo/redo。
39. Graph preview selected node/region/cell/seed with real runtime snapshot。
40. Generate/Validate/Cook/Preview/Apply/Reset operations use factory/controller/job/receipt。
41. Replace fixed SC_Forest/64K/1 conflict/queued with catalog/runtime projection。
42. AssetType/ResourceKind/catalog/App feature for PCG/RuleGraph/Biome/WorldRecipe。
43. plugin admission validates module/resource/operation/controller/compiler/executor/service。
44. first-party runtime/editor provider matrix default/client/server/editor/headless。
45. source dependency index invalidates graph on terrain/spline/region/weather/material changes。
46. cache/artifact atomic publication, corruption detection, GC, size/age/platform budgets。
47. worker panic/device loss/disk full/cancel/late completion fault recovery。
48. malformed graph/data/points/mesh/huge extent/seed/path/decompression fuzz。
49. deterministic single/multi-thread, warm/cold cache, machine-to-machine output golden。
50. WOC fixed source/digest/known vectors imported as PCG golden corpus, not runtime API。
51. remove WOC generated function branches after equivalent compiled output/queries pass。
52. WOC road/terrain/collision candidate outputs map to stable generated artifact IDs。
53. PCG output stores source span/node ID/seed/input revision/algorithm/tool provenance。
54. generated output supports incremental add/update/remove and transactional apply/rollback。
55. multi-world/multi-client/server/editor generation isolation and authority fences。
56. large world 1/1k/100k cells/items, streaming and regeneration p50/p95/p99 benchmark。
57. 64K/1M render instance GPU culling/LOD/overdraw/VRAM benchmark。
58. terrain/foliage/physics/nav/render/cook visual/data golden across platforms。
59. Editor disabled runtime consumes cooked output without Editor cache or Tooling script。
60. clean headless package includes graph/artifact/cell/dependency manifest and no fixture data。
61. schema/algorithm/provider/graph migration supports canary, old generation pin, rollback。
62. graph subasset/library/reuse/parameter inheritance and cycle-safe ownership。
63. custom node/plugin SDK includes ABI, determinism, versioning, sandbox, unload lifecycle。
64. runtime procedural producer supports dirty region/double buffer/budgeted update。
65. PCG/Spawn/Scene/Network/Save/Replay boundary has typed handoff and no shared authority。
66. diagnostics filter/export graph/node/cell/item/asset/generation/receipt/remediation。
67. Editor search/catalog/reference/dependency/impact diff includes generated outputs。
68. collaborative graph editing field merge/lock/presence/review annotations。
69. automatic audits for missing dependencies, nondeterminism, overdraw, density/budget hotspots。
70. Stable/Complete derived only from compile, registration, runtime, Editor, fault, visual, scale and platform evidence。

## 5. P2：长期能力（12 项，全部 Open）

1. GPU/compute PCG nodes、sparse spatial data、remote generation farm。
2. multi-resolution/world-partition PCG with streaming partial outputs and shared cache。
3. terrain erosion/hydrology/climate/river/road coupled simulation generators。
4. neural/procedural asset synthesis with deterministic quality/fallback receipt。
5. runtime procedural gameplay-safe producers with rollback/replay and authority leases。
6. hierarchical biome/ecology/population simulation separate from gameplay spawn authority。
7. graph optimizer, common-subexpression cache, task fusion and cost model。
8. collaborative graph/biome authoring, semantic merge, lock and review history。
9. external GIS/CAD/heightfield/point-cloud import with CRS/provenance。
10. generated asset patch/stream/download/resume/canary rollout/rollback。
11. auto content audit for intersections, density, nav/collision conflicts, memory and visual quality。
12. cross-engine PCG/Terrain/instance benchmark with public scenes and methodology。

## 6. 分层重构顺序

### M0：Truthfulness 与 owner 清理

将 Scatter workspace 与 Terrain importer 标为 fixture/unsupported，禁止 fixed 64K/queued 进入 capability；建立 PCG/Terrain/Foliage/Spawn/Scene/Render/Tooling ownership map，不把 Tooling 当 PCG runtime。

### M1：Typed graph/source/data

建立 PcgGraphSource、node/pin/data registry、seed/parameter/bounds/partition/revision schema、type-check/cycle/diagnostic 与 Editor document/transaction。

### M2：Compiler、executor、cache、generation output

实现 compiled program、bounded task executor、dependency cache、immutable GenerationOutput、managed resource ownership、cell scheduler、atomic receipt。

### M3：Terrain/Scatter/Prefab/Geometry adapters

先恢复 Terrain real importer/chunk/LOD/edit/bake，再接 Foliage/Prefab/Spline/Road/Collision/Nav/Render/Cook；迁移 WOC deterministic corpus，删除 generated function branches。

### M4：Editor graph/debug/product closure

装配 graph canvas、node/pin/attribute/determinism/diff/profiling/debug、real preview region/cell/seed、catalog/App/plugin factory、source/artifact projection。

### M5：Runtime/partition/release qualification

完成 multi-world/streaming/network/save/replay、fault/determinism/visual/scale/headless/package/rollback 门禁；未通过前 PCG/Scatter/Terrain capability 不得 Stable。

## 7. 验收门禁（32 门，当前全部 Fail）

1. graph/source/node/pin/data/parameter/seed/revision identity stable and versioned。
2. type-check/cycle/finite/unsupported input failures are early and diagnostic。
3. compiler/task schedule/dependency/cache/generation receipt deterministic。
4. bounded executor/cancel/retry/shutdown/worker panic/device loss safe。
5. cell partition/interest/prefetch/eviction/rollback generation-safe。
6. output stable IDs/provenance/add-update-remove/manual override/orphan cleanup correct。
7. Terrain artifact/chunk/LOD/edit layer/streaming/import/cook/roundtrip valid。
8. Biome/Noise/PointData/Surface/Spline/Region/Weather typed adapter inputs valid。
9. Scatter constraints/density/random stream/scale/rotation/normal/attributes golden。
10. Prefab/mesh/material/texture output dependencies and render instance backend correct。
11. collision/nav outputs and bake receipts match generated geometry.
12. WOC digest/known vectors/source pin migrate to PCG corpus without behavior drift。
13. no generated script branch/runtime helper remains as undisclosed second authority。
14. graph Editor document/canvas/node/pin/type error/selection/undo/save/recovery works。
15. Generate/Validate/Cook/Preview/Apply/Reset returns real job/generation/artifact/diagnostic receipt。
16. fixed Scatter facts removed; empty/error/missing provider state truthful。
17. plugin/catalog/App/factory/controller/compiler/executor/service admission closed。
18. source/artifact/tool/platform/algorithm key, atomic publication, GC, rollback correct。
19. malformed graph/data/mesh/extent/seed/path fuzz no panic/OOM/unbounded work。
20. single/multi-thread/warm/cold/machine deterministic output hashes and visual/data golden。
21. multi-world/client/server/editor/stream generation isolation and authority correct。
22. render/physics/nav/cook/terrain/foliage outputs consume actual artifacts not fixture counts。
23. 1/1k/100k cells/items and 64K/1M instances meet CPU/GPU/VRAM/memory/hitch budgets。
24. clean headless package/runtime works with Editor and Tooling disabled。
25. save/prefab/PIE/reimport/hot reload/undo/conflict preserve graph/output/override identity。
26. diagnostics filter/export graph/node/cell/item/asset/generation/receipt/remediation。
27. schema/node/plugin/algorithm upgrade supports canary, old pin, migration, rollback, replay。
28. source dependency changes invalidate only affected graph/cell/output artifacts。
29. custom node unload/sandbox/ABI/determinism and missing provider fail-close。
30. visual/quality/overdraw/terrain/nav/collision/gpu culling golden across platform tiers。
31. external GIS/heightfield/point data coordinate/provenance and package manifest correct。
32. Stable/Complete derived only from compile, registration, runtime, Editor, fault, visual, scale and platform evidence。

## 8. 禁止的临时修补

1. 禁止用固定 Scatter Rule Graph、64K instances、1 conflict、seed/density 字符串或 queued feedback 冒充 PCG。
2. 禁止把 `WorldGeneration` revision、WOC worldSeed 或 Terrain height array 命名为 PCG generation system。
3. 禁止把 WOC Zr functions 包进 Generate 按钮而不生成 typed graph/artifact/managed output。
4. 禁止让 PCG graph 直接写 Gameplay live population、Scene storage、renderer internal buffers 或 Physics world。
5. 禁止只加 graph/node/pin/biome enum、manifest capability 或 ZUI canvas而无 compiler/executor/cache。
6. 禁止把 Terrain DiagnosticOnly importer、Foliage/Scatter fixture或MultiMesh backend当作 PCG product。
7. 禁止在 render/physics/runtime thread 同步运行全世界 generation、读文件或复制无界 point data。
8. 禁止用 test attribute、known vector、static screenshot 或 WOC `--check` 替代 32 门 PCG 资格。
9. 禁止在重新导出 38-file manifest/fingerprint 前实施本报告假设，或通过 lockfile drift 绕过 `--locked`。

## 9. 本轮产出边界

本轮只新增 Editor114 review、索引与分层计划，没有修改 Runtime、Editor、Interface、Plugin、App 或 tests production code，也没有运行 Tooling lane、Cargo、PCG graph/determinism/cache/partition/cook/Editor 动态验证；未查询或实时跟踪协调器。实施必须从 M0 开始，先恢复编译基线、关闭 Scatter 假面并建立 typed PCG/Terrain owner inventory，再接入任何 graph UI。
