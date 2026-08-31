---
title: Editor Procedural Content Generation、Rule Graph、Biome 与 World Generation 当前源码复核
category: zircon_editor
report_id: Editor217
review_date: 2026-08-29
baseline_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
verification_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor40
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md
  - docs/plans/optimize/zircon_editor/114-editor-procedural-content-generation-rule-graph-biome-world-generation-current-source-review.md
  - docs/plans/optimize/zircon_editor/161-editor-procedural-content-generation-rule-graph-biome-world-generation-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_scatter_editor_workspace.zui
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/prefab_and_scatter.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs
  - zircon_plugins/editor_support
  - zircon_plugins/terrain
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/content.rs
  - examples/woc/contracts/m3_terrain_content.json
  - examples/woc/scripts/woc_game/src/generated/contracts.zr
  - examples/woc/scripts/woc_game/src/world
plan_sources:
  - docs/plans/optimize/zircon_editor/40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md
  - docs/plans/optimize/zircon_editor/114-editor-procedural-content-generation-rule-graph-biome-world-generation-current-source-review.md
  - docs/plans/optimize/zircon_editor/161-editor-procedural-content-generation-rule-graph-biome-world-generation-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zv-runtime-vegetation-tree-foliage-grass-species-instancing-wind-animation-billboard-impostor-lod-streaming-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/138-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/149-editor-spawn-rules-encounter-population-world-state-scenario-quest-authority-current-source-review.md
  - docs/plans/optimize/zircon_editor/215-editor-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-current-source-review.md
  - docs/plans/optimize/zircon_editor/216-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGGraph.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGNode.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGPin.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGData.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/Data/PCGPointData.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGComponent.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/Subsystems/PCGSubsystem.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/Subsystems/PCGEngineSubsystem.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/RuntimeGen/PCGRuntimeGenScheduler.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/Grid/PCGPartitionActor.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGManagedResource.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Private/Graph/PCGGraphCompiler.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Private/Graph/PCGGraphExecutor.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Private/PCGComponent.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Private/PCGManagedResource.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG/Public/PCGGraphExecutionInspection.h
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor/Private/Managers/PCGEditorInspectionDataManager.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor/Private/Widgets/SPCGEditorGraphDeterminism.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor/Private/Widgets/SPCGEditorGraphDiff.cpp
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor/Private/Widgets/SPCGEditorGraphProfilingView.cpp
  - dev/godot/modules/noise/fastnoise_lite.cpp
  - dev/godot/modules/noise/fastnoise_lite.h
  - dev/godot/modules/noise/noise_texture_2d.cpp
  - dev/godot/scene/resources/multimesh.cpp
  - dev/godot/scene/resources/multimesh.h
  - dev/Fyrox/fyrox-impl/src/scene/terrain/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain/geometry.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain/quadtree.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain/brushstroke/mod.rs
  - dev/Fyrox/editor/src/interaction/terrain.rs
  - dev/Fyrox/editor/src/scene/commands/terrain.rs
  - dev/bevy/crates/bevy_asset/src/event.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceAllocators.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
finding_status:
  p0_open: 5
  p0_partial: 0
  p0_closed: 0
  p1_open: 70
  p1_partial: 0
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# 217 · Editor PCG / Rule Graph / Biome / World Generation 工程化差距

## 1. 结论

Editor40 的 canonical 结论仍成立：当前 Zircon 没有引擎级 Procedural Content Generation 产品。对 19,672 个 zircon_app、zircon_editor、zircon_hub、zircon_plugins、zircon_reflect_derive、zircon_runtime、zircon_runtime_host 与 zircon_runtime_interface 生产 Rust/TOML/ZUI 文件做一次读取的精确合同扫描，PcgGraph、PCGGraph、ProceduralGraph、RuleGraph、BiomeAsset、BiomeSource、WorldRecipe、CompiledPcg、GenerationOutput、GeneratedObjectId、ManagedGenerated 与 PartitionActor 十二个目标合同全部为零命中。ResourceKind 仍为 26 类，没有 PCG Graph、Biome、World Recipe、Point Data 或 Generated Set；也没有首方 PCG runtime/editor package、compiler、per-World service 或 output consumer。

Scatter Workspace 仍是第二 authority。它保持 233 行、27 个 node、19 条 event route、0 个 document/job/artifact binding，固定展示 SC_Forest、Biome Mask、Slope Filter、Rocks + Ferns、64K instances、18 rules 与 1 conflict。Generate/Validate 最终只进入 preview action；extension_module_feedback.rs 固定返回 Generate queued SC_Forest 64K instances 和 Validation queued 18 rules 1 conflict。规则集、seed 与 density 提交只改变 control-local 字符串，没有 graph source、operation factory、job ticket、request/generation identity、artifact、managed output 或 runtime snapshot。

Terrain 纵链有装配进展，但没有变成 PCG backend。Runtime builtin catalog 已有 Terrain row，package 已有 beta/partial manifest 和 native dist 壳；Editor plugin 也声明五个 operation、两个 importer、toolkit、template 与 Inspector customization。然而 first-party runtime provider switch、first-party editor provider 和 zircon_app feature 均未选择 Terrain；五个 operation 没有 factory，runtime importer 仍使用 DiagnosticOnlyAssetImporter 并明确报告 backend is not installed。native behavior 只有 registration manifest，invoke_command、save_state、restore_state 与 unload 全为 None。Project World 转换仍把 terrain 固定写为 None。故这些变化只能作为插件装配底座，不能降低 PCG/Terrain output finding。

GPU Scene 也不能被误判为 Scatter backend。它已有 stable key、instance span、dirty range、upload 与 previous-transform 底座，但唯一 production mesh caller仍以 instance_count 1 注册。没有 PCG InstanceSet adapter、allocation owner、per-output provenance、culling/LOD receipt、cell unload 或 regeneration diff。WOC 的 deterministic terrain/decorations/collision corpus仍可作为迁移 oracle，当前 m3_terrain_content.json SHA-256 为 C481FAAA10CC8B8F136A36DE053015C486A15FC90E95687818906A2537FCC29E；其 source commit、hash/noise known vectors、seed offsets、candidate lattice、zone/biome/camp/road/lake 与 collision cell replay都不能替代引擎 source/compiler/artifact/install 链。

目标架构必须保持单一真值：

~~~text
PcgGraphSource / BiomeSource / WorldRecipeSource
  -> stable graph/node/pin/edge/parameter identity
  -> deterministic compiler + typed task/data DAG
  -> CompiledPcgProgramArtifact
  -> per-World bounded generation service
  -> immutable GenerationOutput + managed resource diff
  -> Terrain/Foliage/Prefab/Spline/Collision/Nav/Render/Cook adapters
  -> same-request Editor inspection and terminal receipts
~~~

本轮维持 5 项 P0、70 项 P1、12 项 P2 全部 Open，32 项资格门全部 Fail。没有动态证据支持 PCG/Terrain/Scatter 的功能、性能或表现达到 Unreal，更不能声明优于 Unreal。

## 2. 审查范围与方法

### 2.1 当前工作树选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与证据 |
|---|---:|---|
| Zircon Editor/Runtime/Plugin/WOC selected | **1,208 / 303,389 / 282,879 / 11,351,517 / 1,725 / 129** | Scatter、graph/job/transaction、Terrain、asset/World、GPU Scene、catalog与WOC；4263446290ed494f781ae0d7c64fc1844a9f344310a2b9b5fa5a9afead66df14 |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **36 / 34,632 / 29,577 / 1,383,417 / 16 / 0** | PCG全链、noise/MultiMesh、Terrain undo、asset extract与GPU instances；9f15da7ab4a3ea30f2c3af4f72012607168ec074b4d28f456331320ca077f344 |
| 全部选择集 | **1,244 / 338,021 / 312,456 / 12,734,934 / 1,741 / 129** | 两组按规范化相对路径去重；b80715d5ae32f3fcfd50172fe4e6f52af503bb58f04e1657916ea929416f2ae0 |

Zircon 选择集由 examples 210、zircon_editor 156、zircon_plugins 27、zircon_runtime 814、zircon_runtime_interface 1 个文件组成。Tooling 按用户要求排除；WOC 只纳入运行时 contracts/generated/world consumer，不纳入 examples/woc/tools。统计用于复现本报告，不代表整个工程已审完；共享工作树有并发修改，实施前必须重新冻结语料和指纹。

### 2.2 判定规则

1. Open 表示领域 owner、source、compiler、artifact、consumer 或资格证据缺失；通用框架、同名 revision、静态 UI 与 manifest 不抵消。
2. 共享底座只有在 PCG 产品中有真实 consumer 后才能使 canonical finding 降为 Partial；本轮没有。
3. Closed/Pass 必须具备 source -> compile -> artifact -> request -> bounded execution -> managed output -> consumers -> Editor observation -> fault/scale/platform evidence。
4. fixed feedback、route、descriptor、registration manifest、DiagnosticOnly importer、空 native callbacks 与单实例 GPU caller均不是 PCG 完成证据。
5. 本轮只做静态 review，不修改 production/tests，不运行 Cargo、Editor、PCG、Terrain、WGPU、cook 或 benchmark。

## 3. 相对 Editor161 的当前重判

| 主题 | 当前源码变化 | 状态影响 |
|---|---|---|
| PCG identity | 十二个目标合同在 19,672 个生产文件中仍为零 | P0-02、P1-01至P1-20保持Open |
| Scatter 产品面 | 233行/27 nodes/19 routes/0 binding，固定SC_Forest/64K/18/1与queued反馈 | P0-01、G29保持Open/Fail |
| Terrain catalog | Runtime builtin catalog新增或保留Terrain row；provider switch、Editor catalog、App feature仍无Terrain | 装配底座进展，不改变P0-03 |
| Terrain dist | native dist导出registration manifest；invoke/save/restore/unload仍None | distribution shell，不是runtime backend |
| Terrain authoring | 五个operation与import plan更完整；batch无factory，LayerStack明确拒绝，runtime仍DiagnosticOnly | P1-50/P1-62保持Open |
| Project Scene | Scene schema有terrain字段，但World转Project固定terrain None | G21保持Fail |
| GPU Scene | span/dirty/upload更完整；production mesh仍register count 1 | P1-51、G22保持Open/Fail |
| Generic random | runtime random authority/stream可复用；无graph/node/pin/cell/element派生算法或PCG consumer | P1-21保持Open |
| WOC | 当前digest、source pin、known vectors与cell replay存在；领域逻辑仍硬编码Zr脚本 | P0-04、P1-22/P1-41保持Open |

## 4. 当前产品纵链事实

| 链路 | 当前源码事实 | 判定 |
|---|---|---|
| Asset identity | 26类ResourceKind没有PcgGraph/Biome/WorldRecipe/GeneratedSet | Open |
| Package/catalog/App | 无PCG package/provider/profile；Terrain row不等于PCG owner | Open |
| Source/graph | 无versioned graph、stable node/pin/edge/parameter identity、subgraph或migration | Open |
| Typed data | 无PointSet/SurfaceField/VolumeField/SplineSet/Attribute/InstanceSet | Open |
| Compiler | 无type/cycle/bounds/capability validator、task DAG、artifact或diagnostic map | Open |
| Executor/cache | 无PCG request、task context、stale guard、node cache、DDC或LKG | Open |
| Spatial/Biome | 无cell/halo/dirty propagation/BiomeSource/overlap/blend/WorldRecipe | Open |
| Output ownership | 无GeneratedObjectId、managed resource、diff/reuse/override/orphan cleanup | Open |
| Consumer adapters | Terrain/Foliage/Prefab/Spline/Collision/Nav/Render/Cook均无PCG adapter | Open |
| Editor toolkit | Scatter只有静态表格、route与fixed feedback | Open |
| Terrain | manifest/dist/descriptor存在；import/World/render执行断裂 | Open |
| GPU instances | 通用span存在，production mesh仍单实例 | Open |
| WorldGeneration | runtime-only mutation revision，persistent equality明确忽略 | 非PCG |
| Evidence | 无source-to-output/save-reopen/cancel/stale/fault/scale动态证据 | Open |

## 5. 五套参考源码对照

| 参考 | 已具备的工程合同 | Zircon 当前缺口 |
|---|---|---|
| Unreal PCG Runtime | Graph/Node/Pin/Data、compiled task DAG、execution/data dependency、World/Engine subsystem、runtime scheduler、partition、managed resource cleanup | 全部PCG owner和数据流缺失 |
| Unreal PCG Editor | execution inspection、determinism、graph diff、profiling与inspection manager | Scatter没有同request真实数据 |
| Godot | versioned FastNoiseLite参数、NoiseTexture异步更新、MultiMesh bulk buffer/visible count/AABB | 只有项目脚本noise，无engine source；无InstanceSet consumer |
| Fyrox | Terrain node/quadtree/raycast、brush stroke与可逆chunk command | Terrain仍DiagnosticOnly且无可逆output |
| Bevy | AssetEvent增删改/依赖加载与render extraction/change tracking/batching | generated diff与精准consumer invalidation缺失 |
| Unity Graphics | Instance allocation/free、GPU data、LOD/culling与visibility output | 大规模实例owner、allocation/cull receipts缺失 |

Unreal 是 PCG 完整产品主参考；其他四套只补充 noise persistence、Terrain 可逆编辑、增量 extract 和大规模实例 consumer。不能因为 Godot/Bevy 没有同等 PCG Editor 就降低 Zircon 的产品门槛。

## 6. Authority 与所有权

1. Editor40 继续是本主题唯一 canonical finding owner；Editor217 只刷新 currentness。
2. Runtime 应新增中立 PCG kernel、compiler、artifact schema 与 per-World service owner，不能放入 Editor、App、WOC 或 Terrain plugin。
3. Editor只拥有 PcgGraph/Biome/WorldRecipe document、transaction、operation、preview与inspection投影。
4. Runtime99zq/Editor138拥有Terrain/Foliage/World Partition；Runtime99zv拥有Vegetation/instance consumer；Editor216拥有Spatial/Road/River；Editor215拥有Weather；Editor149拥有live Spawn/Population authority。
5. PCG只能消费各域 typed adapters，不得复制 Terrain、Spline、Weather、Navigation、Physics、Renderer 或 Gameplay 算法。
6. App只选择project/target/profile/provider并托管生命周期，不持有graph、output、cache或固定反馈。

## 7. P0：必须先阻断的产品与 Authority 断路（5 Open）

| ID | 状态 | 当前问题 | 首个关闭条件 |
|---|---|---|---|
| P0-01 | Open | Scatter Generate/Validate固定返回64K/18/1与queued | 未接真实graph/job/receipt前隐藏或标记Unavailable，只投影typed terminal receipt |
| P0-02 | Open | 无唯一PCG source/artifact/request/output authority，ZUI/WOC/Terrain可能被误当truth | 建PcgGraphSource -> CompiledPcgProgramArtifact -> GenerationRequest -> GenerationOutput单链 |
| P0-03 | Open | Terrain catalog/dist/descriptor存在，但importer DiagnosticOnly、operation无factory、World/render无consumer | Terrain readiness由真实runtime/editor/consumer receipt原子发布 |
| P0-04 | Open | WOC项目脚本可能被直接升级为公共engine API | 只迁移determinism/data/partition/provenance golden，经versioned adapter进入PCG |
| P0-05 | Open | 无managed generated resource/stable ID/diff/cleanup，PCG直写Scene会破坏authoring authority | 先建立owner-qualified immutable output、atomic diff与override/detach/orphan policy |

## 8. P1：70 项 canonical 工程化主线（70 Open）

### 8.1 Source、Schema 与 Registry

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| P1-01 | Open | 新增正式PcgGraph ResourceKind/marker/asset URI/artifact kind/catalog type。 |
| P1-02 | Open | 定义graph schema version、canonical serialization与unknown-field roundtrip。 |
| P1-03 | Open | 定义stable graph/node/pin/edge/parameter identity，禁止数组索引成为公共identity。 |
| P1-04 | Open | node type registry记录plugin owner、capability、schema与compiler factory。 |
| P1-05 | Open | typed node settings包含default/range/unit/asset reference/validator。 |
| P1-06 | Open | 定义pin direction/data type/cardinality/required/default/dynamic-pin合同。 |
| P1-07 | Open | 定义subgraph/function引用、parameter mapping、recursion与cycle规则。 |
| P1-08 | Open | 分离compile-time/generation-time/instance override参数。 |
| P1-09 | Open | 建立graph/source migration registry、version fixtures与失败恢复。 |
| P1-10 | Open | 将node settings/subgraph接入asset dependency extraction。 |

### 8.2 Typed Data 与 Compiler

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| P1-11 | Open | 定义PointSet、SurfaceField、VolumeField、SplineSet、AttributeTable、ParameterSet、ResourceSet、InstanceSet。 |
| P1-12 | Open | collection携带schema ID、bounds、space、element count、digest与provenance。 |
| P1-13 | Open | point携带stable element key、transform、density、extents、seed key与attribute domain。 |
| P1-14 | Open | 定义surface/volume sample/project/filter/bounds/precision合同。 |
| P1-15 | Open | 建立attribute type registry、domain conversion、missing/default、rename/migration。 |
| P1-16 | Open | 建立pin compatibility、受控implicit conversion和ambiguity diagnostic。 |
| P1-17 | Open | 诊断node/subgraph cycle、required input、unreachable output与capability。 |
| P1-18 | Open | 编译topological task DAG，并分离execution/data dependency。 |
| P1-19 | Open | 编译layout、constant fold、dependency manifest、cost estimate与diagnostic map。 |
| P1-20 | Open | artifact key绑定engine/plugin/compiler/schema/algorithm版本。 |

### 8.3 Determinism、Executor 与 Cache

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| P1-21 | Open | 从graph/node/pin/cell/element派生稳定random stream。 |
| P1-22 | Open | 把WOC known vectors迁入cross-platform hash/noise/sort/float PCG corpus。 |
| P1-23 | Open | node声明deterministic/platform-stable/best-effort/non-deterministic等级。 |
| P1-24 | Open | 定义request ID、source/artifact revision、seed、bounds/cell、quality与purpose。 |
| P1-25 | Open | 定义worker/main-thread/GPU/external affinity与合法切换。 |
| P1-26 | Open | PCG job支持budget class、progress、pause、cancel、retry与shutdown drain。 |
| P1-27 | Open | 建立task/publish generation-attempt stale guard。 |
| P1-28 | Open | cache key覆盖compiled/input/parameter/seed/cell/quality/platform。 |
| P1-29 | Open | 建立memory cache cost/LRU、reverse dependency与eviction diagnostic。 |
| P1-30 | Open | 建立portable immutable node/output DDC artifact。 |
| P1-31 | Open | 建立last-known-good compiled artifact与显式stale UI/receipt。 |
| P1-32 | Open | 记录cache reason、node wall/CPU/GPU time与data bytes。 |

### 8.4 Spatial、Biome 与 WorldRecipe

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| P1-33 | Open | 定义world bounds、partition level/cell、halo与coordinate space。 |
| P1-34 | Open | node声明local/neighborhood/global reduction/unbounded spatial class。 |
| P1-35 | Open | compiler验证partition合法性并拒绝global/unbounded node。 |
| P1-36 | Open | 从input dependency与bounds推导dirty node/cell。 |
| P1-37 | Open | 建立halo、neighbor dependency与seam regression。 |
| P1-38 | Open | 定义runtime generation/cleanup radius、load/unload与priority policy。 |
| P1-39 | Open | 定义BiomeSource identity、field/mask、priority、blend、unknown/no-data行为。 |
| P1-40 | Open | 建立Biome overlap/blend、surface/climate constraint与debug output。 |
| P1-41 | Open | 将WOC硬编码zone/biome迁移为显式Biome/Rule golden。 |
| P1-42 | Open | 建立WorldRecipe stage DAG、typed I/O、failure policy与receipt。 |
| P1-43 | Open | 对Terrain/Spline/Weather/Nav domain artifact使用typed引用。 |
| P1-44 | Open | 定义selected region/cell、whole world、runtime与cook generation profiles。 |

### 8.5 Output、Ownership 与 Consumer

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| P1-45 | Open | 定义GeneratedObjectId及graph/node/output/cell/element provenance。 |
| P1-46 | Open | 建立managed resource container、CRC、used/reused/unused与preview/baked/runtime lifecycle。 |
| P1-47 | Open | 建立add/update/remove/reuse regeneration diff与atomic commit。 |
| P1-48 | Open | 定义per-output manual override/detach/forbidden policy。 |
| P1-49 | Open | 隔离attempt transient、LKG与authored object并完成失败/取消清理。 |
| P1-50 | Open | Terrain adapter提供chunk/edit/layer artifact和真实consumer receipt。 |
| P1-51 | Open | 将InstanceSet接入batched GPU Scene adapter，而非单实例register。 |
| P1-52 | Open | 建立Prefab dependency/variant/override/stable instance adapter。 |
| P1-53 | Open | 消费Editor39 SpatialSpline artifact/query建立Road/River adapter。 |
| P1-54 | Open | Collision/Nav使用同generation revision与cell invalidation adapter。 |
| P1-55 | Open | 只提供placement Gameplay adapter，不接管live spawn authority。 |
| P1-56 | Open | 聚合HLOD/minimap/render/collision/nav/cook同revision receipts。 |

### 8.6 Editor Authoring

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| P1-57 | Open | 基于GraphEditorDescriptor建立PCG toolkit和真实document/session。 |
| P1-58 | Open | 建立typed pins/edges、palette/search/comment/reroute/subgraph canvas。 |
| P1-59 | Open | 建立node/edge/settings transaction、undo/redo、dirty/save/reopen。 |
| P1-60 | Open | 建立multi-select/copy-paste/duplicate/delete与stable identity policy。 |
| P1-61 | Open | Details消费typed setting validation/migration，而非字符串字段。 |
| P1-62 | Open | Generate/Validate接operation factory/job/progress/cancel/terminal receipt。 |
| P1-63 | Open | 建立selected node/bounds/cell、PreviewWorld与full bake scope。 |
| P1-64 | Open | 建立debug object/cell selector、attribute/output viewer与per-node inspection。 |
| P1-65 | Open | 建立determinism/diff/profiling/log/cache inspection产品视图。 |
| P1-66 | Open | UI显示source revision、artifact generation、stale/LKG与consumer readiness。 |

### 8.7 Cook、Diagnostics 与验证

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| P1-67 | Open | stable diagnostic code定位graph/node/pin/cell/asset/fix action。 |
| P1-68 | Open | cook manifest/receipt携带dependency/algorithm digest、partition与consumer receipts。 |
| P1-69 | Open | 建立roundtrip/migration/compiler/determinism/cache/cancel/stale/ownership矩阵。 |
| P1-70 | Open | 建立64K/1M point、partition churn、incremental edit、memory/cache与cross-platform证据。 |

## 9. P2：主线闭合后的长期能力（12 Open）

1. GPU-native point processing与CPU/GPU graph partition optimizer。
2. Custom compute/HLSL/WGSL node sandbox、resource limits与offline validation。
3. Distributed/remote generation worker与artifact merge。
4. Hierarchical/shape grammar与assembly node library。
5. ML-assisted rule suggestion只输出可审查source diff。
6. Runtime adaptive PCG profile切换与可重演policy。
7. CSV/JSON/columnar/external dataset provider。
8. Density/attribute/flow/heatmap/3D volume inspection。
9. 多用户graph协作、node presence与semantic merge。
10. Generation request replay、time travel与cross-build diff。
11. Biome生态演替与长期world evolution simulation。
12. Marketplace node签名、compatibility与determinism certification。

## 10. 分层重构顺序

| 里程碑 | 必须交付 | 退出条件 |
|---|---|---|
| M0 Truth Freeze | 隐藏/禁用Scatter固定结果；建立缺PCG/Terrain provider和fixed feedback RED证据 | UI只显示typed unavailable，不再出现伪queued/64K/18/1 |
| M1 Source/Schema/Registry | PcgGraph/Biome/WorldRecipe资源、stable IDs、typed settings/pins、migration | canonical roundtrip与newer schema fail-close |
| M2 Typed Data/Compiler | Point/Surface/Volume/Spline/Attribute/Instance data、validator与artifact | deterministic compile golden和diagnostic定位 |
| M3 Request/Determinism | request identity、random stream、algorithm version、WOC-derived vectors | thread order和无关node变更不扰动未影响output |
| M4 Executor/Cache/DDC | per-World bounded executor、affinity、cancel/retry/shutdown、cache/LKG | cancel/stale/cache/fault/budget矩阵 |
| M5 Spatial/Biome/Partition | bounds/cell/halo/dirty propagation、Biome overlap/blend | seam与局部失效证据 |
| M6 Managed Output | GeneratedObjectId、provenance、atomic diff、override/detach/orphan cleanup | 失败不损坏authored/LKG |
| M7 Terrain/Scatter Slice | 真实Terrain adapter与batched InstanceSet consumer | 一个graph source-to-render/query/receipt纵链 |
| M8 Cross-Domain Adapters | Prefab/Spline/Collision/Nav/HLOD/Cook/SpawnDefinition receipts | 无跨域第二authority |
| M9 WorldRecipe/WOC Migration | stage DAG、profile、WOC zone/biome/decoration golden | 项目codegen不成为engine runtime依赖 |
| M10 Editor Product | toolkit/canvas/transaction/preview/attribute/diff/profile/cache inspection | 所有显示数据来自同request/runtime snapshot |
| M11 Release Qualification | cook/headless/network/save/replay、64K/1M、fault/scale/soak/platform | 32门全部通过后才提升maturity |

M0-M4 建立独立 PCG kernel，不等待完整 Terrain renderer。M5-M9 必须等待各 domain typed adapter，不能复制简化版。MVP docs/plans/mvp/00 仍为 In Progress；当前只允许 review 和计划，不开始 advanced PCG 实现。

## 11. 验收门禁（32 Fail / 0 Partial / 0 Pass）

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 | Fail | production catalog无唯一PcgGraph kind/schema/owner plugin。 |
| G02 | Fail | graph/node/pin/edge stable ID不存在。 |
| G03 | Fail | unknown/newer node settings无roundtrip与禁用诊断。 |
| G04 | Fail | type/cardinality/cycle/subgraph recursion无compiler诊断。 |
| G05 | Fail | 无绑定source/dependency/tool/schema/algorithm的compiled artifact。 |
| G06 | Fail | 无跨线程稳定element ID/content digest。 |
| G07 | Fail | 无branch/cell random stream隔离。 |
| G08 | Fail | WOC known vectors尚未成为engine PCG跨平台门。 |
| G09 | Fail | 无PCG cancel与stale publish guard。 |
| G10 | Fail | 无LKG/stale preview/cook拒绝合同。 |
| G11 | Fail | 无PCG cache key与reverse invalidation。 |
| G12 | Fail | 无portable DDC artifact与live-handle拒绝。 |
| G13 | Fail | 无spatial class与partition legality。 |
| G14 | Fail | 无halo/seam golden。 |
| G15 | Fail | 无局部dirty node/cell执行证据。 |
| G16 | Fail | 无streaming cell managed output lifecycle。 |
| G17 | Fail | generated object无完整provenance。 |
| G18 | Fail | 无atomic add/update/remove/reuse diff。 |
| G19 | Fail | cleanup无法证明保护authored/detached/override/foreign object。 |
| G20 | Fail | preview/baked/runtime lifecycle未分层。 |
| G21 | Fail | Terrain importer仍DiagnosticOnly且World/render断裂。 |
| G22 | Fail | 64K仍为固定文本，production mesh GPU Scene仍单实例。 |
| G23 | Fail | Collision/Nav/Render/HLOD/Cook无同generation receipt。 |
| G24 | Fail | 无placement-only Gameplay adapter与authority test。 |
| G25 | Fail | 无Biome overlap/blend/unknown/no-data golden与debug。 |
| G26 | Fail | WOC terrain/decoration尚未迁入PCG artifact corpus。 |
| G27 | Fail | Tooling按用户要求排除，且codegen仍是项目依赖证据。 |
| G28 | Fail | Scatter无graph document/canvas/transaction/save/reopen。 |
| G29 | Fail | Generate/Validate仍发布固定queued/64K/18/1。 |
| G30 | Fail | 无同request attribute/determinism/diff/profile/cache inspection。 |
| G31 | Fail | 无64K/1M、partition churn、cancel storm、cache pressure与shutdown证据。 |
| G32 | Fail | clean compile/test/cook/pack、平台与fault artifact矩阵未通过。 |

## 12. 禁止的临时修补

1. 禁止新增几个PCG/Node/Pin/Biome enum、ZUI canvas或manifest capability后宣称完成。
2. 禁止继续用SC_Forest、64K、18 rules、1 conflict、seed/density字符串或queued feedback冒充结果。
3. 禁止把WorldGeneration revision、WOC world seed、Terrain height array或Render Graph称为PCG系统。
4. 禁止把WOC Zr函数包装成Generate按钮而没有typed source/compiler/artifact/managed output。
5. 禁止让PCG直接写live World、Gameplay population、renderer buffer、Physics world或NavMesh owner。
6. 禁止把DiagnosticOnly Terrain、普通mesh单实例GPU Scene或Godot MultiMesh当成PCG产品。
7. 禁止在render/physics/runtime thread同步生成全世界、读文件、展开无界point data或创建64K document entities。
8. 禁止使用global RNG、数组索引、pointer、thread/GPU handle或unstable iteration order进入artifact key。
9. 禁止失败/取消先删除旧generation，或late result覆盖新source revision。
10. 禁止Editor、Terrain、Foliage、Spline、Weather、Spawn分别复制私有executor/cache/partition truth。
11. 禁止保留旧路径shim、双写source/artifact/runtime state或以compat layer掩盖owner迁移。
12. 禁止在32门、同画质benchmark与跨平台故障矩阵通过前声明达到或优于Unreal。

## 13. 本轮产出边界

本轮只完成 current-source review、参考引擎对照、canonical 状态重判与依赖有序重构计划。没有修改 Zircon production/tests，没有运行 Cargo、Editor、PCG、Terrain、WGPU、cook、determinism、fault、scale、soak 或 benchmark。Editor40 仍是 canonical owner；5 项 P0、70 项 P1、12 项 P2 与 32 项门禁不会因本报告自动关闭。
