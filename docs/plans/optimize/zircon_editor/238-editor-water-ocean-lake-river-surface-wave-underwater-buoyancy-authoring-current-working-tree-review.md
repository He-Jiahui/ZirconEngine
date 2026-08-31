---
title: Editor Water、Ocean、Lake、River、Surface、Wave、Underwater 与 Buoyancy Authoring 当前工作树复审
category: zircon_editor
report_id: Editor238
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
canonical_owner: Editor39
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/160-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
  - docs/plans/optimize/zircon_editor/113-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/178-runtime-water-ocean-lake-river-wave-underwater-buoyancy-current-working-tree-review.md
related_code:
  - zircon_editor/src/scene
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editing
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/prefab_tools/editor
  - zircon_plugins/terrain/editor
  - zircon_plugins/navigation/editor
  - examples/woc/scripts/woc_game/src/world
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source/Editor
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source/Runtime/Public/WaterSplineComponent.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source/Runtime/Public/WaterBodyRiverComponent.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source/Runtime/Public/WaterSplineMetadata.h
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Water
  - dev/godot/scene/resources/material.cpp
  - dev/bevy/assets/shaders/water_material.wgsl
  - dev/Fyrox/editor/src/plugins/animation
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor238 · Water/Ocean/Lake/River authoring 当前工程化差距

## 1. 结论

当前 Editor 没有 Water authoring surface、resource factory、scene component、body/zone inspector、wave/simulation preview、query/buoyancy debugger 或 build artifact browser。`ResourceKind`/AssetTypeId 没有 Water 类型，first-party editor catalog 没有 Water provider。现有 Spline/River 相关报告只证明通用 spline authoring 仍未闭合；Terrain、Foliage、Navigation、Weather 和 Physics workbench 也不能提供 Water source。

工作树中唯一出现的水体产品暗示是 Navmesh AI Workbench 的静态 `Water` area 选项，以及 WOC fixture 的私有水位/湖泊判定。它们没有 document id、stable body identity、transaction、compiler job、runtime generation、query result 或 preview world。Foliage 的 `Biome_Riverbank`/`River_02` 是静态 rows，不是 River/Water artifact。不得把这些控件或蓝色材质 preview 显示为 Water Ready。

因此本报告刷新 Editor39/160 的 River/Spline 当前性，**不新增 P0**；新增 **24 项 P1（24 Open）**、**10 项 P2（10 Open）**、**22 道资格门（20 Fail / 2 Partial / 0 Pass）**。Runtime178 拥有运行时 Water resource/service/query/adapter 差异；Editor238 只拥有 authoring、preview、operation、artifact 与调试 UI。

## 2. 当前源码证据

### 2.1 Asset、Document 与 Catalog

- `zircon_editor/src/core/asset/type_registry` 只能映射现有 ResourceKind，没有 water body/material/wave/zone/simulation/record类型。
- `zircon_editor/src/scene` 与 `core/editing` 没有 WaterDocument、body/zone stable id、selection lease、revision、undo transaction、save/reopen 或 source/artifact LKG。
- `first_party_editor_catalog` 没有 Water provider；terrain/navigation/prefab plugin 的 descriptors 不会自动成为 Water owner。

### 2.2 Workbench 与假产品入口

- `workbench_extension_navmesh_ai_workspace.zui` 的 area 下拉包含 `Walkable/Jump/Door/Water`，但 action 仍归 Navigation owner，没有 swim layer 或 Water query receipt。
- `workbench_extension_foliage_editor_workspace.zui` 的 `Biome_Riverbank`、`River_02` rows 是 fixture，不能产生 spline/river/source artifact。
- 没有 Water body placement gizmo、spline control-point editor、shore/bank profile editor、wave spectrum editor、underwater volume view、buoyancy pontoon authoring 或 simulation timeline。
- Editor capture/performance/runtime diagnostics 只显示通用 frame/capture 状态，不能观察 Water tiles/waves/query/physics generation。

### 2.3 参考编辑器差异

Unreal Water Editor 以 WaterBodyOcean/Lake/River/Custom、WaterZone、SplineMetadata、terrain/landscape integration、mesh/HLOD 与 transaction/visualizer 组织 authoring；Unity HDRP Water Editor 以 surface type、geometry, simulation/deformation/foam/waterline/underwater 和 CPU/GPU search 参数驱动 inspector/preview。Godot/Bevy 只提供材质/SSR 示例，Fyrox 的 command/undo 模式可作为 Rust mutation 对照。Zircon 当前只有静态 controls，缺 source-to-artifact authoring contract。

## 3. P1 重构任务

| ID | 当前问题 | 必须完成 |
|---|---|---|
| ED-WATER-01 | 无 Water asset types | 注册 WaterBody/Material/Wave/Mask/Zone/Simulation/QueryPreset 类型、factory、icons、open/reimport/thumbnail。 |
| ED-WATER-02 | 无 Water provider/catalog | provider manifest、first-party catalog/App feature、runtime capability handshake、缺 backend fail-closed。 |
| ED-WATER-03 | 无 source document | WaterDocument、stable body/zone/spline/point ids、revision、dirty/save/reopen/LKG/migration。 |
| ED-WATER-04 | 无 transaction/undo | placement、point edit、split/merge、duplicate/delete、profile edit 进入 typed command/history/byte budget。 |
| ED-WATER-05 | 无 body authoring | Ocean/Lake/River/Pool/Custom body placement、extent、depth、priority、material 与 local frame。 |
| ED-WATER-06 | 无 spline/river authoring | control point tangent/width/depth/bank/flow、closed/open、confluence、cross-section、arc-length preview。 |
| ED-WATER-07 | 无 zone/overlap tools | WaterZone priority、exclusion/island/hole、overlap stack、selection and diagnostics。 |
| ED-WATER-08 | 无 wave editor | analytic/spectral/baked/shallow source、seed/bands/amplitude/choppiness/quality、units/validation。 |
| ED-WATER-09 | 无 terrain/weather bindings | accepted Terrain bottom/River flow/Weather wind snapshots、dependency/rebuild reason、partial invalidation。 |
| ED-WATER-10 | 无 preview world | isolated preview scene、runtime artifact install、fixed-step transport、pause/seek/step/reset/device-loss。 |
| ED-WATER-11 | 无 surface viewport | geometry/LOD/wireframe/normals/velocity/depth/foam/shore/underwater/waterline overlays。 |
| ED-WATER-12 | 无 query debugger | pick point/body、surface/depth/normal/current/immersion flags、error/miss、generation/provenance、batch trace。 |
| ED-WATER-13 | 无 buoyancy debugger | pontoon placement, sampled force/torque/drag/current、Physics command/receipt and history。 |
| ED-WATER-14 | 无 swim/navigation tools | swim volume/cost/entry/exit、agent preview、Navigation artifact integration、unsupported state。 |
| ED-WATER-15 | 静态 Water area fixture | 删除或显式标 unavailable，直到 runtime Water/Nav provider 返回 typed admission。 |
| ED-WATER-16 | 静态 River/Foliage rows | rows 必须来自 asset/query snapshot，显示 stable id/generation/status，不得写 River_02 fixture。 |
| ED-WATER-17 | 无 render/simulation diagnostics | tiles/LOD/budget/GPU time/queue/fallback/deformation/foam/query stats live mirror。 |
| ED-WATER-18 | 无 build artifact | compiler job、dependency graph、warnings/errors/source spans、artifact generation、install/rollback。 |
| ED-WATER-19 | 无 runtime attach | PIE/preview attach、world/body generation admission、stale snapshot/reconnect/shutdown。 |
| ED-WATER-20 | 无 collaboration/source control | body/spline/profile locks、conflict/diff/merge、source authority 与 multi-user transaction。 |
| ED-WATER-21 | 无 WOC migration | 水位/湖泊/游泳/pathfinding fixture 转换工具与 before/after characterization。 |
| ED-WATER-22 | 无 automation | deterministic water fixtures、CPU/GPU query golden、visual capture、physics/network/save tests。 |
| ED-WATER-23 | 无 scale/fault UX | 1K bodies、large ocean、streaming/evict、GPU/device loss、invalid terrain、disk/source failure。 |
| ED-WATER-24 | 无 cross-domain/cinematic integration | Sequencer/Weather/Terrain/Physics/Navigation/Sound/VFX consumer binding 与 receipt inspector。 |

## 4. P2 完整度任务

| ID | 必须补齐 |
|---|---|
| ED-WATER-P2-01 | shoreline/harbor/island procedural authoring。 |
| ED-WATER-P2-02 | spectrum import/bake/cache/thumbnail/waveform preview。 |
| ED-WATER-P2-03 | foam/splash/wake emitter graph and VFX preview。 |
| ED-WATER-P2-04 | underwater color/caustics/medium profile and multi-camera preview。 |
| ED-WATER-P2-05 | water material instance/permutation/HDR inspector。 |
| ED-WATER-P2-06 | aquatic navigation/AI/swim behavior authoring。 |
| ED-WATER-P2-07 | network/save/replay Water state diff and migration UI。 |
| ED-WATER-P2-08 | HLOD/world partition/streaming water cell authoring。 |
| ED-WATER-P2-09 | headless/CI validation and reference benchmark dashboard。 |
| ED-WATER-P2-10 | localization/accessibility and collaboration presence for water tools。 |

## 5. 资格门

| Gate | 当前结果 | 通过条件 |
|---|---|---|
| ED-WATER-G01 | Fail | Water asset/document 可创建、保存、重开、迁移、reimport。 |
| ED-WATER-G02 | Fail | provider/catalog/App/runtime capability closure。 |
| ED-WATER-G03 | Fail | body/zone/spline stable identity、transaction/undo/revision。 |
| ED-WATER-G04 | Fail | Ocean/Lake/River source authoring与geometry artifact。 |
| ED-WATER-G05 | Fail | wave/spectrum/shallow source validation and preview。 |
| ED-WATER-G06 | Fail | Terrain/River/Weather dependency graph and invalidation。 |
| ED-WATER-G07 | Fail | PreviewWorld fixed-step/play/seek/reset/device-loss。 |
| ED-WATER-G08 | Fail | surface/underwater/foam/waterline viewport overlays。 |
| ED-WATER-G09 | Fail | query debugger shows typed result/generation/error。 |
| ED-WATER-G10 | Fail | buoyancy/swim/navigation tools consume runtime receipt。 |
| ED-WATER-G11 | Fail | static Water area/River rows removed or capability-gated。 |
| ED-WATER-G12 | Fail | compiler job emits artifact/diagnostics/install/rollback。 |
| ED-WATER-G13 | Fail | PIE/runtime attach and stale/reconnect handling。 |
| ED-WATER-G14 | Fail | WOC migration characterization passes。 |
| ED-WATER-G15 | Fail | source-control/collaboration conflict flows。 |
| ED-WATER-G16 | Partial | existing Spline/Terrain/Navigation/Physics transactions can be reused, but no Water document/handler. |
| ED-WATER-G17 | Partial | existing generic viewport/capture/diagnostics UI can host Water mirror, but no provider snapshot. |
| ED-WATER-G18 | Fail | runtime/editor/render/physics generation alignment。 |
| ED-WATER-G19 | Fail | deterministic visual/query/physics fixtures。 |
| ED-WATER-G20 | Fail | scale/fault/headless/cross-platform evidence。 |
| ED-WATER-G21 | Fail | cinematic/weather/terrain/sound/VFX integration。 |
| ED-WATER-G22 | Fail | static fixture cannot report Water Ready without receipt。 |

## 6. 推荐重构顺序

1. 先建立 Water asset/document/provider/catalog 和 stable command/transaction，再隐藏所有没有 provider 的 Water/Swim UI。
2. 实现 body/spline/zone/wave authoring 与 compiler artifact，接入 PreviewWorld/fixed-step/query/buoyancy debugger。
3. 接入 runtime render/physics/navigation/audio/VFX/weather/terrain generation snapshot，Editor 只消费 receipts。
4. 迁移 WOC/River/Terrain fixtures，补 PIE、save/reopen、collaboration、fault/scale/headless gates。
