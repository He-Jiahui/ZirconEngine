---
title: Editor Spline、Path、Road、River、Decal、Brush 与 Geometry 当前源码复核
category: zircon_editor
report_id: Editor113
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor39
refreshes:
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
related_code:
  - zircon_plugins/rendering/features/decals/runtime/src
  - zircon_plugins/rendering/features/decals/editor/src
  - zircon_plugins/rendering/plugin.toml
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/graphics/material/mod.rs
  - zircon_runtime/src/graphics/shader/shader_assets.rs
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_editor/src/core/plugin/descriptor.rs
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/selection
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui
  - tools/editor-workbench-preview/design.js
  - examples/woc/contracts/m3_terrain_content.json
  - examples/woc/scripts/woc_game/src/world/terrain_content.zr
  - examples/woc/scripts/woc_game/src/world/decoration_candidate.zr
tests:
  - zircon_plugins/rendering/features/decals/runtime/src
  - zircon_plugins/rendering/features/decals/editor/src
  - zircon_editor/src/scene/modes/tests.rs
  - zircon_editor/src/scene/selection/tests.rs
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - examples/woc/contracts/m3_terrain_content.json
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/27-project-operations-source-control-changelist-diff-automation-report-submit-gates-health-dashboard-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SplineComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SplineMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Editor/ComponentVisualizers/Private/SplineComponentVisualizer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/DecalComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/CompositionLighting/PostProcessDeferredDecals.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source/Runtime/Public/WaterSplineComponent.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source/Runtime/Public/WaterSplineMetadata.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source/Runtime/Public/WaterBodyRiverComponent.h
  - dev/UnrealEngine/Engine/Source/Editor/LandscapeEditor/Private/LandscapeEdModeSplineTools.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/BrushComponent.h
  - dev/godot/scene/resources/curve.h
  - dev/godot/scene/3d/path_3d.h
  - dev/godot/scene/3d/decal.h
  - dev/Fyrox/fyrox-impl/src/scene/decal.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/decal.shader
  - dev/Fyrox/fyrox-math/src/curve.rs
  - dev/bevy/crates/bevy_math/src/cubic_splines/mod.rs
  - dev/bevy/crates/bevy_pbr/src/decal/clustered.rs
  - dev/bevy/crates/bevy_pbr/src/decal/forward.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal/DecalProjector.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal/DecalSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Material/Decal/DecalProjectorEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Water/WaterDecal/WaterDecal.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 113 · Editor Spline / Path / Road / River / Decal / Brush / Geometry 工程化差距

## 1. 结论

当前 Zircon 没有引擎级空间 Spline、Path、PathFollow、SplineMesh、Road、River、WaterBody、Geometry Brush 或 CSG 产品。仓内 `CubicSpline`/Hermite 只服务 glTF 动画或标量 Sound automation，没有空间点/段 identity、弧长参数化、最小旋转标架、最近点查询、分段 bounds、空间索引、Scene persistence 或 Editor control-point tool。

这已经造成项目专用绕行：WOC 先把 14 条道路抽成 JSON，再生成大量 `roadPointX()`/`roadPointZ()` 分支；植被候选对所有 road segment 做二维点到线段距离，得到 deterministic 但不可扩展的折线查询。它不会生成 road mesh/UV/material/collision/nav/lane/junction/terrain stamp/world-partition artifact。另一个示例把 Road 当普通 imported mesh 放入 Scene，说明 Road 是真实产品需求，不能继续留给脚本。

Decal 更严重：可选 `rendering.decals` plugin 注册 `DecalProjector` descriptor、一个 pass 与 `decals.projector-composite` executor，但 executor 只返回 `Ok(())`。Descriptor 没有 instance owner、serialization、extract、shader、texture/material binding、culling、batch 或 GPU draw；启用插件可以“注册成功”却不产生任何像素。Editor crate 只声明 drawer ID，没有 extension registration、Inspector、Scene mode、create/add operation、transaction 或 preview。Material Workbench 的 `decal` dropdown 与 runtime `MaterialDomain` 不一致，是第二 authority。

可保留底座包括 DynamicScene schema/migration/reflection、World plugin component registration、SceneMode/selection/overlay/gizmo、Material graph/pipeline contracts 和 WOC source digest。它们证明基础设施可承载产品，不证明 Spline/Road/Decal/Brush 已完成。

正确目标应分两条：`SpatialSplineSource -> CompiledSplineArtifact -> immutable query view -> Road/River/PathFollow/SplineMesh/TerrainStamp adapters`，以及独立的 `DecalProjectorSource + Material/Texture bindings -> extract/cull/batch/render artifact -> Scene/Editor toolkit`；Geometry Brush/CSG 另建 shape/boolean/extrusion authoring domain，不能与 Terrain/Foliage/UI brush 混名。

## 2. 审查范围与证据

### 2.1 当前工作树物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 指纹 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin selected | **692 / 42,479 / 38,674 / 1,607,904 / 90** | Decal package、Scene/DynamicScene/reflection、Material、Scene tools、WOC road source；`cb89f25db5e6217ac8d4aca00bf8f617c77c01b2a7e75b27cc3efba27ba59a0d` |
| Unreal/Godot/Fyrox/Bevy/Unity reference | **23 / 18,208 / 15,335 / 728,727 / 12** | Spline/SplineMesh/visualizer、Water/River、Decal/DBuffer、Curve/Path、Fyrox/Bevy/Unity decal；`b6c57c8c8771d72a34303a1634a76676cc645a50ed4b33eb80aed9baee8218ff` |
| Zircon selected union | **715 / 60,687 / 54,009 / 2,336,631 / 102** | current physical working tree union；`84d4a65332343993ee658dc1af92d70793e7ef1e77c8f51da014928a8d7c11f8` |

范围包含完整 DynamicScene、SceneMode/selection 与 Decal/Material目录，因而显著大于旧报告；统计按 root 去重、排序后以 UTF-8 内容计算 SHA-256。当前 baseline epoch 524，工作树有无关在途修改，实施前必须重新导出 manifest/fingerprint。没有运行 Cargo、spline numeric、road/river compiler、decal pixel、CSG boolean、Editor control-point transaction 或大世界性能验证。

### 2.2 Spline/Path/Road/River/Brush 缺口

1. production 没有 SpatialSpline/Curve3D source、component、asset、artifact、query service 或 Scene node。
2. 没有 PathFollow、SplineMesh、Road、River、WaterBody、GeometryBrush、BrushBuilder 或 CSG kernel。
3. 现有 CubicSpline/Hermite 仅用于 glTF animation/标量 automation，不提供 spatial tangent/normal/roll/arc length。
4. 没有 stable point/segment IDs、control-point transaction、handles、interpolation mode、tangent lock、closed loop 或 versioned migration。
5. 没有 arc-length table、distance->parameter、nearest point、frame orientation、curvature、segment bounds 或 BVH/grid index。
6. Terrain/Foliage Workbench 的 Brush Properties 和 Riverbank labels 是 static template/mock，不是 spatial source。
7. `ResourceKind`/catalog 没有 Spline/Path/Road/River/Water/GeometryBrush，Scene schema 也没有对应 carrier。
8. WOC JSON roads 只有 2D X/Z points/array lengths/hash，没有 Y/tangent/roll/width/profile/junction/segment metadata。
9. codegen 生成 `roadPointX/Z` 大量 branch；查询遍历全部 roads/segments，O(total segments)，没有 local index/batch/SIMD/cache。
10. road query 只返回 XZ 最近距离，不能提供 arc length/lane offset/height/frame/road/segment identity。
11. 没有 road mesh/shoulder/curb/UV/material/collision/nav/traffic lane/junction/terrain conform/stamp/artifact。
12. 没有 river bank/water surface/flow/current/foam/depth/shoreline/physics/audio/weather binding。
13. 没有 geometry brush shape/boolean/extrusion/CSG validation、history、topology artifact、undo 或 bake receipt。

### 2.3 Decal Runtime 缺口

1. Decal plugin 只注册 component/render feature/pass descriptor；`DecalProjectorDescriptor` 没有 component storage/reflection/serialization/instance owner。
2. mode/opacity/normal_blend/atlas_region 无生产消费者，atlas_region 是 String 而不是 Texture/Material/Atlas reference。
3. executor 只返回 `Ok(())`，没有 pipeline/shader/bind group/instance buffer/draw/dispatch；Deferred/ScreenSpace 没有真实分支。
4. pass 读写 scene color，但没有 DBuffer/GBuffer/normal/depth attachment、ping-pong/load-store/alias/hazard 合同。
5. 没有 projector bounds/frustum/cluster culling、visibility mask、distance/angle fade、lifetime、sort、batch 或 budget。
6. 没有 albedo/normal/ORM/emissive channel、receiver policy、forward/mobile/MSAA/ray-tracing/virtual-geometry fallback。
7. 没有 texture streaming/atlas allocation/eviction/bindless capability、stats、quality 或 failure diagnostics。
8. 唯一 runtime test 只验证 registration report/pass name，不执行 GPU 或像素 golden；“registered”不能称为 rendered。

### 2.4 Editor/Material/Scene 边界

1. Decal editor crate 使用默认空 `register_editor_extensions()`；drawer ID、Inspector customization、add/create/open/validate operation 均没有消费者。
2. 没有 projector box visualizer、orientation arrow、pick proxy、SceneMode、overlay、selection、preview world、transaction 或 save/reimport。
3. Material Workbench 固定显示 decal domain，但 runtime `MaterialDomain` 没有 Decal；domain action 只改变 template control/journal。
4. `.zmaterial` schema v2 没有 domain field、decal material validator、shader variant 或 compiled output。
5. 设计工具固定显示 WarningStripe/12 placements/1 sort warning/bounds updated，主 Editor 没有对应业务面。
6. SceneAsset 固定字段不包含 plugin component、Spline、Road、River、Decal/Brush；DynamicScene 可以捕获 reflected dynamic component，但当前没有代码创建或保存 Decal。
7. World `apply_to_world()`/dynamic component/reflection/migration 是可复用基础，不是产品 authority；descriptor 没有 typed defaults、asset refs、schema/migration。
8. SceneMode/selection/overlay/gizmo 基础存在，但没有为这些 domain 建 document/compiler/provider。

## 3. P0：必须先关闭的断路（5 项，全部 Open）

### P0-1：空间 Spline/Road/River/Brush 产品合同缺失

没有 source、stable IDs、compiler、artifact、query view、Scene persistence 或 toolkit。WOC 折线 codegen 是项目绕行，不是引擎能力。必须先建立 versioned SpatialSpline source/artifact 与 typed consumers。

### P0-2：Decal plugin 注册成功但像素执行为空

descriptor/pass/executor 可以通过 registration test，executor 却不提交 GPU 命令，没有 material/texture/instance/cull。必须将 capability 标为 unsupported 或完成真实 extract->cull->batch->render->pixel gate，不能保留“registered=complete”。

### P0-3：Decal Editor/Material/Scene 三 authority 分裂

drawer ID/Material decal dropdown/preview design 不产生 Scene component、asset transaction、shader variant 或 render artifact。必须先统一 source/document/material/Scene owner，再实现 toolkit。

### P0-4：Scene/plugin persistence 没有空间 domain carrier

DynamicScene 能捕获 plugin reflected component 是底座，但 static Scene/Preset/Prefab/World pipeline 没有 Spline/Road/River/Decal/Brush 字段或 install path。必须先定义 versioned domain carrier 与 migration，而不是继续给 SceneEntityAsset 加 Option。

### P0-5：项目专用 road 查询不可扩展且无引擎 consumer

全量线段遍历、magic distance、XZ-only codegen 没有 mesh/nav/collision/lane/terrain/streaming artifact。必须把 source digest 保留迁移到 compiled spatial artifact，之后删除脚本绕行。

## 4. P1：Runtime、Geometry、Decal 与 Editor（70 项，全部 Open）

1. 定义 versioned SpatialSplineSource、control point/segment stable IDs 与 migration。
2. 支持 Bezier/Hermite/Catmull/linear、tangent/roll/scale/interpolation、closed loop。
3. 建立 finite/degenerate/self-intersection/duplicate point validation 与 diagnostics。
4. 编译 arc-length table、parameter/distance mapping、curvature、frame orientation。
5. 定义 minimum-twist/parallel-transport frame、up policy、banking、handedness。
6. 编译 segment bounds、BVH/grid/chunk spatial index 与 deterministic nearest query。
7. 提供 point/tangent/normal/roll/width/metadata/nearest/overlap batch APIs。
8. 建立 immutable query view、source revision、world generation、late completion fence。
9. Scene/Prefab/Instance/World save/load 保留 spline identity/source/artifact dependency。
10. Editor control-point handles、tangent/roll/width editing、snap、multi-select、undo/redo。
11. 建立 Spline AssetType、document、transaction、dirty/save/autosave/recovery/conflict。
12. PathFollow 绑定 spline query、distance/offset/frame、loop、speed、authority/network/replay。
13. SplineMesh compiler 支持 profile/cross-section、twist/scale、UV、material、LOD、collision。
14. Road source 支持 lanes、width/shoulder/curb、bank、junction、surface layers、traffic metadata。
15. Road mesh/collision/nav/terrain conform/stamp/decals/streaming artifacts 与 provenance。
16. River/WaterBody source 支持 banks、flow/current/depth、shoreline、foam、surface/volume。
17. Water renderer/physics/audio/weather bindings、LOD、clipmap、reflection/refraction contracts。
18. Terrain stamp/extrude/brush source 使用 spatial IDs、falloff、layer、height/material masks。
19. Geometry Brush domain 支持 primitives、boolean union/subtract/intersect、extrusion、topology validation。
20. CSG kernel deterministic、manifold/degenerate/self-intersection/UV/material region diagnostics。
21. brush history/transaction/preview/bake artifact、cancel、rollback、large mesh budget。
22. WOC road source migration 保留 JSON digest/length determinism并生成 compiled artifact。
23. 删除 `roadPointX/Z` codegen 与 per-query O(total segments) magic constants。
24. road query 支持 arc length/lane offset/height/frame/road/segment/metadata stable result。
25. road index tile/cell streaming、batch/SIMD、cache、LOD、async query budget。
26. road/river junction graph、intersection topology、AI/nav/traffic path adapter。
27. DecalProjector typed component/source/asset reference、mode/opacity/normal/atlas/material schema。
28. Decal Scene persistence、DynamicScene reflection、Prefab/World/PIE install、migration。
29. Decal material compiler、domain/texture channel validation、shader variant and actual artifact。
30. Decal render path choose DBuffer/GBuffer/forward with explicit attachments/lifetime/hazards。
31. Projector frustum/cluster culling、receiver mask、distance/angle fade、sort/priority/lifetime。
32. Decal GPU instance buffer、batch/atlas/bindless/streaming/eviction、mobile/MSAA fallback。
33. Decal normal/roughness/metallic/emissive/opacity blend and depth/receiver policy。
34. Decal lighting/shadow/reflection/temporal/upscale/render graph ordering receipts。
35. Decal pixel/visual golden、stats、unsupported/fallback diagnostics、device loss recovery。
36. Decal Editor plugin register actual extensions, factory, AssetType, component creation and toolkit。
37. Projector gizmo/visualizer/pick/selection/overlay/preview world and camera preview。
38. Decal material editor connects source/document/compile/preview/receipt to Material runtime。
39. Replace fixed Material decal dropdown with actual domain registry/capability admission。
40. Create/Open/Import/Reimport/Validate/Compile/Apply/Playtest operations have real factories。
41. Shared spatial plugin SPI validates module/resource URI/factory/controller/service/catalog/App。
42. DynamicScene plugin components include typed defaults, field IDs, schema version, dependency refs。
43. Static Scene authoring supports plugin component projection without hardcoded options per domain。
44. prefab/instance override/lineage/remap/merge/unknown field preservation for spatial domains。
45. Editor selection/document revision/generation prevents stale geometry/decal publish。
46. background geometry/decal jobs bounded/cancel/retry/shutdown drain/atomic publication。
47. artifact key source/recipe/tool/platform/renderer/quality/algorithm and bulk chunks/GC。
48. diagnostic journal records source span/point/segment/shape/material/asset/generation/remediation。
49. malformed points/curves/mesh/boolean/texture/huge extent/path input fuzz and budget rejection。
50. scene/prefab/undo/save/reimport/hot reload/stream unload roundtrip golden。
51. spline numeric golden arc-length/frame/nearest/curvature/twist across degenerate cases。
52. road/river mesh/collision/nav/terrain/UV/material/flow visual and data golden。
53. decal pixel golden DBuffer/GBuffer/forward, normal/roughness/alpha, MSAA/mobile/device matrix。
54. WOC large road query/compile scales 1/1k/100k segments with p50/p95/p99 and memory bound。
55. large decal count/atlas pressure/overdraw/streaming/GPU frame hitch benchmarks。
56. multiplayer/network/replay/save deterministic path/road/decal/brush state and authority。
57. runtime client/server/editor/headless cook does not depend on Editor design.js or fixture labels。
58. first-party catalog/App feature targets contain actual runtime/editor provider and factory closure。
59. release manifest includes compiled geometry/decal/material dependencies, platform support, provenance。
60. cross-platform handedness/float/texture/DBuffer/render graph/nav/physics outcomes deterministic。
61. plugin enable/disable/upgrade/migration/rollback cannot leave stale descriptor/renderer registrations。
62. road/river water interaction with Weather/Region/Navigation/Physics uses typed adapter boundaries。
63. Decal/Brush/Material changes update dependency graph and invalidate correct artifact only。
64. Editor preview uses real geometry/decal artifact/camera, not static placements/warning text。
65. source/artifact diff displays point/segment/material/shape/channel/byte changes and impact。
66. multi-user field-level merge/lock/presence/review for spatial authoring documents。
67. auto audit for self-intersection, bad frames, missing width/material, overdraw, overlap and budget。
68. algorithm/schema migration supports canary, old generation pin, rollback and replay compatibility。
69. compare Unreal/Godot/Fyrox/Bevy/Unity spline/decal/water quality/perf methods with fixtures。
70. Stable/Complete is derived only from compile, registration, runtime, Editor, visual, fault, scale and platform evidence。

## 5. P2：长期能力（12 项，全部 Open）

1. GPU spline/road tessellation、meshlet/virtual geometry、bindless material and culling。
2. procedural road/river network generation、traffic lanes、junction solve、terrain excavation。
3. water simulation、waves/foam/erosion、shoreline climate/weather interaction。
4. SDF/voxel/CSG sparse geometry、incremental boolean、topology repair and bake farm。
5. decal virtual texturing、clustered GPU culling、ray tracing、neural material masks。
6. dynamic destruction/runtime brush and deterministic replicated geometry edits。
7. remote geometry/decal build farm、quality/RDO/perceptual optimization and cache federation。
8. world partition streaming/patching/rollback for huge road/river/decal datasets。
9. multi-user collaborative spline/brush/road/decal authoring with semantic merge。
10. external GIS/CAD/road/river/terrain data import with coordinate/CRS/provenance.
11. procedural animation/path constraints/vehicle/AI/nav integration with replay/time scrubbing。
12. cross-engine conformance scenes and public geometry/render/performance benchmark methodology。

## 6. 分层重构顺序

### M0：Truthfulness 与 owner 清理

将 Decal capability、Material decal domain、设计工具、WOC road shortcut 明确标为 unsupported/fixture；删除 registration-success-as-rendered 结论，盘点 Spline/Road/River/Brush/CSG/Decal owners。

### M1：Spatial Spline source/compiler/query

建立 typed source、stable IDs、frame/arc-length/index artifact、immutable query view、Scene/Prefab/World persistence 与 Editor control-point transaction。

### M2：Road/River/PathFollow/SplineMesh/Brush consumers

以同一 spline artifact 实现 typed road/river/path/spline-mesh/terrain-stamp、水体与 navigation/collision/streaming adapters，迁移并删除 WOC codegen。

### M3：Decal source/material/render

建立 projector/source/material schema、Scene install、DBuffer/GBuffer/forward render path、culling/batch/atlas/streaming、pixel golden 与 diagnostics。

### M4：Editor toolkits与plugin/catalog闭环

接入 AssetType、document/transaction、gizmo/selection/overlay、real preview、operations/factory/controller、first-party catalogs/App feature。

### M5：Fault、Scale、Release

完成 malformed/fault/determinism/roundtrip/visual/large-world/cross-platform/headless package/rollback/benchmark 门禁；未通过前 capability 不得 Stable。

## 7. 验收门禁（32 门，当前全部 Fail）

1. spline/source/control/segment/artifact/query/instance IDs、revision/generation 完整。
2. curve finite/degenerate/self-intersection/frame/arc-length validation 早拒绝并诊断。
3. nearest/distance/parameter/frame/curvature query deterministic、batch、indexed、bounded。
4. Scene/Prefab/World/PIE/save/undo/reimport/hot reload 保持 spatial identity/source。
5. PathFollow/SplineMesh/road/river/water/terrain stamp consumer 使用真实 query/artifact。
6. road mesh/UV/material/collision/nav/lane/junction/terrain/stream artifacts 完整。
7. river flow/depth/shore/foam/reflection/physics/audio/weather adapters 有 receipt。
8. WOC source digest/length determinism 迁移到 compiled artifact，旧 codegen 删除。
9. Decal descriptor/component/source/material/texture/atlas schema 与 Scene/Prefab install 闭合。
10. Decal DBuffer/GBuffer/forward render graph attachments/order/lifetime/hazards 正确。
11. Decal cull/batch/instance/atlas/streaming/receiver/normal/blend/device fallback 正确。
12. Decal runtime pixel/visual golden 不因 registration success 通过而缺 draw。
13. Material decal domain、shader variant、compiler/artifact、texture refs 与 render consumer 一致。
14. DynamicScene/plugin component default/reflection/migration/unknown field roundtrip 正确。
15. Editor AssetType/document/transaction/gizmo/selection/preview/diagnostic 真实闭环。
16. create/open/import/reimport/validate/compile/apply/playtest factories/controller/receipt 可执行。
17. background geometry/decal jobs bounded/cancel/retry/shutdown/atomic publication。
18. malformed/NaN/overflow/huge mesh/boolean/texture/path fuzz 无 panic/OOM。
19. source/artifact/tool/platform/algorithm key、bulk chunks、GC、rollback/provenance 正确。
20. WOC/road/decal/brush 1/1k/100k scale、GPU/CPU/memory/frame hitch 达标。
21. multi-world/viewport/client/server/stream cell generation fence 无 stale publish。
22. plugin/catalog/App/headless client/server/editor capability admission 闭合。
23. scene/prefab/undo/autosave/recovery/conflict/merge/reimport 不覆盖未提交 edits。
24. road/river/geometry/decal material/texture dependency invalidation 精确且可追踪。
25. visual/data/gpu golden 覆盖 frames、curves、roads、water、decal channels、MSAA/mobile。
26. diagnostics 可按 source/point/segment/shape/material/asset/generation/receipt 筛选导出。
27. external GIS/CAD/terrain coordinate/provenance、handedness、platform precision 通过。
28. algorithm/schema/plugin upgrade 支持 canary、old generation pin、rollback、replay。
29. Editor 关闭后 Runtime 仍消费实际 geometry/decal artifacts，不依赖 design.js/fixtures。
30. cross-engine reference scene 方法公开，质量/性能/内存/overdraw 可比较。
31. unsupported feature/shape/backend 在 admission 早失败，禁止静默 no-op。
32. Stable/Complete 只由 compile、registration、runtime、Editor、visual、fault、scale、platform evidence 派生。

## 8. 禁止的临时修补

1. 禁止继续用 WOC `roadPointX/Z`、全量 segment loop 和 magic distance 代替 spline/road artifact。
2. 禁止添加 `Vec<Vec3>` 或几个控制点按钮而没有 frame/arc-length/index/transaction/query contract。
3. 禁止 Decal registration/pass/executor `Ok(())` 作为像素完成证明。
4. 禁止只增加 Decal/Material domain enum、manifest capability、drawer ID 或设计稿。
5. 禁止把 Terrain/Foliage/Sound/glTF animation curve 混称为 spatial spline/geometry brush。
6. 禁止把 DynamicScene reflection 或 SceneMode stack 的存在称为产品 authoring 完成。
7. 禁止在 render thread 同步生成 mesh、执行 CSG、读文件或提交无界 decal instances。
8. 禁止用 test attribute、registration snapshot、static preview、手工截图替代 32 门资格。
9. 禁止在重新导出 692-file manifest/fingerprint 前实施本报告假设，或通过 lockfile drift 绕过 `--locked`。

## 9. 本轮产出边界

本轮只新增 Editor113 review、索引与分层计划，没有修改 Runtime、Editor、Interface、Plugin、App 或 tests production code，也没有运行 Cargo、spline/road/river/decal/CSG 动态验证；未查询或实时跟踪协调器。实施必须从 M0 开始，先恢复编译基线、建立空间 owner inventory 和 Decal fail-close，再实现任何路径/道路/水体/刷子 UI。
