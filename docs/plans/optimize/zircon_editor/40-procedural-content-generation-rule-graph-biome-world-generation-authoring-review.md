---
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
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
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

# 40 · Procedural Content Generation / Rule Graph / Biome / World Generation Authoring 工程化差距

## 1. 结论

Zircon当前没有引擎级PCG产品。生产代码中不存在`PcgGraph`、`RuleGraph`、typed PCG node/pin、spatial data collection、compiler、executor、graph cache、generation request、partition scheduler、managed generated resource、generation receipt或PCG asset kind。`WorldGeneration`命中只是Scene World的全局revision计数器，不是世界生成系统；`procedural`命中主要是天空、shader效果或UI布局术语，也不能作为PCG证据。

主Editor却已经展示了一个容易被误判为完成的Scatter产品面。`Scatter Rule Graph`页面固定列出`SC_Forest`、Biome Mask、Slope Filter、Rocks + Ferns、64K instances和1 conflict；所谓graph只是四行静态表格，没有node、pin、edge或canvas document。Generate和Validate按钮最终只返回固定的`queued`文字，seed与density只改模板control状态。仓内没有操作executor、job、asset、compiler或receipt能产生这些数量。这是必须先修正的P0 truthfulness问题。

Terrain也不能作为Scatter后端。现有`TerrainAsset`只存一块`width * height`高度数组和layer引用；Scene只保存一个terrain asset reference。Terrain runtime插件将高度图导入器注册成`DiagnosticOnlyAssetImporter`，诊断明确写着`terrain heightfield importer backend is not installed`；Editor的import/create/sculpt均只有descriptor和menu operation path，全仓没有对应operation implementation。没有chunk、LOD、编辑层、流送、生成版本、依赖清单或cook receipt。

`examples/woc`提供了比Editor mock更真实、也更值得保留的确定性基础。工具链从固定source commit抽取zone、biome、lake、67 camps、terrain edit、14 roads、dock和Sowfield数据，校验sentinel与SHA-256，再生成Zr脚本查询。terrain noise、height、ground、slope与decoration candidate有已知向量；`collision_grid.zr`按查询cell重演邻近装饰候选并参与碰撞。本轮`npm run check:m3-terrain`通过，证明这条项目合同当前未漂移。

但WOC仍是游戏专用源码投影，不是引擎PCG。生物群系是沿Z轴的固定zone code，地形层是手写函数顺序，装饰是在10-yard格点上逐候选计算，碰撞查询会重新生成局部候选；Zircon侧没有把结果发布为渲染实例、Terrain artifact、Scene entity、nav/collision bake输入、world partition cell或可编辑生成物。它没有稳定生成物ID、provenance、增量cache、regeneration diff、manual override policy、bake/unbake或跨版本迁移。WOC源项目的renderer会消费`generateDecorations()`，当前Zircon投影只实现规则和局部碰撞重演，不能被Scatter Workbench拿来证明64K实例已生成。

目标不是把WOC函数包进一个`Generate`按钮，也不是建立一个可以任意写World的万能脚本节点。应建立版本化`PcgGraphSource`、typed node/pin/data schema、确定性`CompiledPcgProgramArtifact`、带预算和取消的增量executor、partition-qualified cache、不可变generation output及managed generated resource ownership。Terrain、Foliage/Scatter、Prefab、Spline/Road、Collision、Navigation、Render和Cook只通过typed adapter消费输出；Gameplay Spawn Rules继续由Editor28的权威模拟域拥有，PCG最多产出静态placement definition，不能直接篡改live authoritative population。

本报告登记5个P0、70个P1、12个P2、M0-M11重构路线和32个验收门。它只做review，不修改Runtime、Editor、plugin、interface生产代码或tests。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | test attributes / ignored / 在途 | 审查方式 |
|---|---:|---:|---|
| Scatter Editor false surface | 10 / 4,303 / 220,696 | 2 / 0 / 2 | E3逐ZUI control、route、navigation spec、feedback与template binding |
| Terrain package | 16 / 856 / 31,660 | 8 / 0 / 0 | E3逐manifest、runtime registration、diagnostic importer、Editor contribution与import plan |
| Terrain asset与Scene边界 | 19 / 4,965 / 195,148 | 17 / 0 / 0 | E3逐ResourceKind、authoring asset、artifact payload、load与Scene reference/World projection |
| WOC生成投影 | 38 / 8,904 / 348,107 | 0 / 0 / 0 | E3逐extract/codegen/contract、terrain reducer、decoration candidate、collision replay和test package |
| pinned WOC source model | 7 / 4,792 / 185,602 | 0 / 0 / 0 | E2/E3按固定commit核对world/data/rng/collider/foliage/dock/Sowfield语义 |
| Unreal PCG参考 | 22 / 15,342 / 567,906 | 0 / 0 / 0 | E2/E3逐graph/data/compiler/cache/executor/partition/resource ownership与Editor inspection |
| Godot参考 | 6 / 1,830 / 72,883 | 0 / 0 / 0 | E2逐Noise resource/async texture与MultiMesh实例后端边界 |
| Fyrox参考 | 8 / 6,437 / 257,149 | 28 / 0 / 0 | E2/E3逐Terrain chunk/quadtree/brush thread/undo command，限定其非PCG图边界 |
| Bevy参考 | 3 / 5,184 / 208,438 | 2 / 0 / 0 | E2逐AssetEvent与mesh extract/batching，限定其无Editor PCG产品 |
| Unity Graphics参考 | 3 / 4,442 / 211,923 | 0 / 0 / 0 | E2逐GPU instance allocation/update/culling/batching，限定仓库只覆盖render backend |
| selected combined scope | 132 / 57,055 / 2,299,512 | 57 / 0 / 2 | 当前工作树fingerprint `503bfb6cb55d81e3552585c1960ed884fa16d059e189619e49cce7d2428cba84` |

指纹算法为：对132个选择路径按PowerShell `Sort-Object`排序，逐文件计算小写SHA-256，形成`forward/slash/path|file_sha256`行，以单个LF连接且末尾不追加LF，再对UTF-8无BOM payload计算SHA-256。选择规则包括完整`zircon_plugins/terrain`目录、完整WOC terrain/decoration测试包、表中其余显式文件；缺失与重复路径均为0。

读取时2个在途文件为`zircon_editor/src/ui/retained_host/workbench_preview_actions.rs`和`zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs`，均非本报告产生。实施前必须重导132文件manifest、重算指纹并复核这两个入口的终态。

57个Rust test attributes主要覆盖Terrain descriptor/import plan、Runtime asset roundtrip、Fyrox terrain/quadtree与Bevy引用代码。它们不包含Zircon PCG graph roundtrip、type-check、determinism、incremental invalidation、managed output、Editor transaction、partition cook或64K实例性能测试，因此不能用数量替代产品验收。

### 2.2 产品类型与入口缺失

1. `ResourceKind`拥有Terrain、TerrainLayerStack、Prefab、MaterialGraph与AnimationGraph，但没有PCGGraph、Biome、WorldRecipe、PointData或GeneratedSet。
2. production Rust没有`PcgGraph`、`ProceduralGraph`、`RuleGraph`或`BiomeAsset`类型。
3. 没有PCG plugin/package/capability/catalog entry；Terrain插件不声明PCG能力。
4. 没有typed node registry、pin schema、edge compatibility或subgraph contract。
5. 没有PCG data type registry；point、surface、volume、spline、attribute set与parameter data均无统一载体。
6. 没有graph compiler、topological schedule、cycle diagnostic或compiled artifact。
7. 没有generation request ID、source revision、input revision、seed stream、bounds/cell或quality tuple。
8. 没有node result cache、dependency key、stale result rejection或last-known-good artifact。
9. 没有PCG scheduler、budget、pause、cancel、retry或shutdown drain。
10. 没有managed generated entity/component/instance/resource ownership。
11. 没有PCG world partition actor/cell manifest或runtime generation radius。
12. 没有Editor graph document、node palette、pin canvas、debug object、attribute viewer、determinism view、diff或profiling view。
13. `WorldGeneration`只是World变更revision，用于cache invalidation，不包含procedural source或generation product。
14. WOC的`worldSeed`是游戏状态字段，不是通用PCG generation request。

### 2.3 Scatter Workbench的实际路径

1. 页面初始隐藏，由通用extension workspace host显示。
2. 左栏固定显示`SC_Forest`、`Rule_Rocks`和`Rule_Ferns`。
3. Rules、Constraints和Output只是tab route，没有不同document model。
4. 标题写`Scatter Rule Graph`，中央内容却只有四个固定table row。
5. Biome Mask固定为`Input Forest Selected`，没有Biome asset reference或sample input。
6. Slope Filter固定为`0-38 deg Ready`，没有Terrain/surface query source、unit schema或compile state。
7. Spawn Rule固定为`Rocks + Ferns 64K`，没有mesh/prefab引用、weight table、point input或output artifact。
8. Collision Test固定为`1 conflict Warning`，没有冲突位置、对象identity、shape或diagnostic ID。
9. Output固定为`18 rules 64K instances 1 conflict`，不读取runtime或job snapshot。
10. rule set下拉固定为Forest、Riverbank、Cliff、Meadow四个字符串，没有资产catalog query。
11. seed字段保存`Seed: 2026`整段字符串，density保存`Density: 0.64`整段字符串，没有typed parsing、range、validation或unit。
12. navigation spec只把control映射到route，未绑定document/session/controller。
13. template binding只注册Click、Change和Submit事件，未创建operation payload。
14. Generate route被重写为`.generate.invoke`后只返回固定`queued`feedback。
15. Validate route同样只返回固定18 rules/1 conflict。
16. 仓内没有该route的operation factory、executor、background job、compiler或durable receipt。
17. 没有测试证明按钮触达任何业务状态；现有2个selected test attributes来自周边host代码。
18. 当前UI把设计fixture呈现为native extension workspace，构成错误能力声明。

### 2.4 Terrain与Scene为什么不能承接PCG

1. `TerrainAsset`只有URI、name、width、height、sample spacing、height scale、height samples和layers。
2. 唯一terrain validation只核对非空height sample数量是否等于`width * height`。
3. 没有chunk ID、tile coordinate、mip/LOD、quadtree、edit layer、hole、normal、bounds或revision。
4. `TerrainLayerAsset`只有material、weightmap和strength，不包含blend mode、physical surface、biome tag或generated provenance。
5. `SceneTerrainAsset`只保存terrain asset reference，没有instance settings或partition binding。
6. Scene asset加载链可以反序列化Terrain，但Scene到World构建没有可见Terrain runtime component消费链。
7. 全仓非测试`TerrainAsset`消费者集中在asset parse/cache/load/facade，不进入renderer、physics或navigation。
8. Terrain runtime component descriptor只有`terrain`和`layers`两个asset_ref property。
9. runtime importer是`DiagnosticOnlyAssetImporter`，backend明确未安装。
10. Editor `validate_heightfield_import()`只核对尺寸、扩展名和可选sample count。
11. `TerrainImportPlan`只返回normalized extension、output kind与expected sample count，不产生artifact。
12. import heightfield、weightmap、create、open和sculpt operation path仅在plugin descriptor/tests出现。
13. 没有Terrain operation executor、scene mode factory、brush stroke transaction或save bridge。
14. Terrain/Foliage/Scatter/World Partition的完整产品差距由Editor16负责；本报告只登记PCG对这些消费者的输入合同。

### 2.5 WOC确定性世界生成基础

1. codegen固定source commit `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`并验证commit存在。
2. source manifest必须与固定commit一致，避免当前HEAD悄然替换authority。
3. extractor通过`wocgit:///`加载固定commit的TypeScript模块，而不是复制当前工作树对象。
4. codegen校验3个zone identity/order、biome order、5 lakes、67 camps和307个camp mobs。
5. 它校验terrain edit、2 docks、14 road lengths、road SHA-256、Sowfield bounds/tiers与decoration exclusion。
6. 生成JSON记录逐source file SHA-256和catalog SHA-256。
7. `--check`比较期望文本，不写入漂移内容；本轮该命令通过。
8. `terrain_noise.zr`显式复制JavaScript signed bitwise coercion，并给hash/noise/fbm known vectors。
9. `terrain_shape.zr`组合zone biome hill/base、hub flatten、lake cut和camp flatten。
10. `terrain_mountains.zr`组合ridge、rim、noise和terrace，并保留source layer order。
11. `terrain_height.zr`按Sowfield、mountain、crater和terrain edit顺序求值。
12. `terrain_ground.zr`在terrain之上叠加dungeon floor、Sowfield stand和dock surface。
13. `terrain_gradient.zr`用四次ground height sample求slope/downhill，并复制JavaScript round规则。
14. `decoration_candidate.zr`按10-yard lattice、hash stream、biome density、hub/camp/water/road/slope过滤决定kind/scale/variant。
15. 每个随机量使用不同固定seed offset，局部候选查询是纯函数。
16. `collision_grid.zr`按输入position确定16-yard cell，只重演可能触碰该cell的邻近candidate lattice。
17. procedural decoration在固定collider之后按gx-major、gz-minor顺序参与最多三轮resolve。
18. 该局部重演避免为每个碰撞查询扫描整个world decoration数组，是应保留的确定性优化。

### 2.6 WOC仍不是引擎PCG的原因

1. terrain layer order写死在模块调用链，不能由资产编辑、版本化或type-check。
2. zone biome是固定code和Z区间，不是二维/三维field、mask stack或可混合Biome source。
3. candidate kind概率写死在函数分支，无法复用为规则资产或实例参数。
4. 每个candidate会重复执行terrain、road、camp和slope查询，没有node/result cache。
5. slope查询又触发四次ground height计算，ground height继续遍历camp、ridge、edit和dock数据。
6. road distance对全部road segment线性扫描；该问题和Spline/Road产品归Editor39所有。
7. collision replay只生成radius与center，不形成可查询的generation artifact或stable collider identity。
8. Zircon投影没有Decoration record collection、renderer instance buffer或Scene entity输出。
9. 没有generated object owner，不能安全区分author-created和generator-created对象。
10. 没有generation diff，规则变化只能重新求值，无法按stable ID做add/update/remove。
11. 没有cell artifact、DDC key、streaming manifest或cook receipt。
12. 没有Editor transaction、preview region、selected node execution或before/after diff。
13. 没有manual override、exclude pin、detach/bake/unbake或orphan cleanup policy。
14. custom content、dynamic map和active-run路径在WOC合同中仍明确延期。
15. WOC Tooling05已经登记生成文件/增量/`build.rs`的一般问题，本报告不重复拥有通用codegen治理。
16. 项目级known vectors可迁移为首个PCG golden corpus，但不能让项目脚本反向成为engine API。

### 2.7 可保留基础的上限

1. Runtime asset URI、direct reference、artifact cache和typed load基础可承载PCG source/artifact，但当前没有kind/schema。
2. Editor已有document、transaction、jobs、notification、diagnostic journal和asset toolkit基础，分别由Editor02/09等报告治理。
3. World拥有entity generation与world revision，可作为stale guard输入，但不能替代PCG generation identity。
4. Scene/Prefab、dynamic component和plugin extension registry可作为output adapter，不应让graph直接写内部storage。
5. WOC hash stream、source pin、sentinel、catalog digest和known vectors可用于确定性基线。
6. WOC局部cell候选重演可启发stateless runtime生成模式，但必须挂接统一request key和provenance。
7. TerrainAsset的typed reference和layer direct references可迁移，不应保留单块数组作为大世界runtime格式。
8. Bevy/Unity/Godot的实例后端表明大规模输出应交给renderer-owned批处理，不应展开为每实例Editor entity。

### 2.8 参考引擎差异

1. Unreal `UPCGGraph`拥有nodes、pins、graph change、parameters和cook compiled data；Zircon只有静态表格行。
2. Unreal `FPCGGraphCompiler`把graph编译为task，`FPCGGraphExecutor`区分execution/data dependencies、预算、active/paused task与cancel；Zircon没有executor。
3. Unreal graph cache按element与input data缓存结果，graph change会通知compiler/cache失效；Zircon没有dependency key。
4. Unreal `UPCGComponent`携带graph instance、seed、generation trigger、partition flag、generation/cleanup radius并公开generate/cancel/cleanup生命周期。
5. Unreal partition actor与subsystem将生成调度接入world streaming和runtime generation，而不是用一个全局按钮处理全世界。
6. Unreal managed resource支持soft release、reuse、unused cleanup、CRC、generated actors/components与transient preview状态；Zircon没有生成物所有权。
7. Unreal PCG Editor有determinism、diff、profiling、log、attribute viewer和debug object tree；Zircon没有graph canvas，更没有inspection。
8. 本地Unreal `PCGBiomeCore` experimental模块本身只有空startup/shutdown，不应单独当作Biome成熟度证据；成熟基线来自PCG core的数据/执行/ownership链。
9. Godot FastNoiseLite是可序列化Noise resource，暴露seed/frequency/fractal/domain warp并驱动异步NoiseTexture更新；它是可复用生成primitive，不是PCG graph。
10. Godot MultiMesh提供instance count、visible count、transform/color/custom data buffer、AABB和physics interpolation，可作为Scatter render output参考，不拥有规则编译。
11. Fyrox Terrain拥有chunk、quadtree、raycast、后台brush stroke和可交换ChunkData的undo command；它证明Terrain consumer必须真实可编辑，但没有通用PCG graph。
12. Bevy AssetEvent区分Added/Modified/Removed/Unused/LoadedWithDependencies，render extract维护changed/removed实例并做CPU/GPU batching；它提供增量consumer语义，不提供Editor PCG产品。
13. Unity Graphics仓内的InstanceDataSystem/InstanceCuller/InstanceCullingBatcher负责GPU instance allocation、transform/probe/wind update、LOD/culling/batch；它是高规模输出后端，不是Unity完整Terrain/PCG源码。
14. 因此参考路由必须以Unreal PCG为产品主参考，以Godot/Fyrox/Bevy/Unity Graphics为primitive、Terrain和实例consumer交叉验证。

### 2.9 动态证据边界

本轮运行`npm run check:m3-terrain`，11.5秒内退出0，固定source extract、sentinel、digest和生成JSON/Zr文本未漂移。该命令只证明WOC terrain content codegen一致性，不证明PCG产品存在，也不证明Scatter UI生成64K实例。

此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断。相关生产输入未修复，本轮未重复同一不可达lane；不能声称Editor动态测试通过。PCG没有可运行的Zircon compiler/executor测试目标。

## 3. 目标架构

### 3.1 Authority分层

```text
PcgGraphSource / BiomeSource / WorldRecipe
    -> semantic validation + deterministic graph compile
    -> CompiledPcgProgramArtifact + dependency manifest
    -> GenerationRequest(source revision, seed, bounds/cell, inputs, quality)
    -> immutable typed data collections + node receipts
    -> managed generation output / partition artifact
    -> typed Terrain / Scatter / Prefab / Spline / Collision / Nav / Render / Cook adapters
```

Source、compiled artifact、execution state、generated output和live consumer instance必须是五层不同identity。Editor只编辑Source；compiler不修改Scene；executor只产生immutable output；adapter在transaction或runtime generation boundary提交；consumer不能把自己的内部对象反写成graph source。

### 3.2 PcgGraphSource schema

1. graph拥有stable asset ID、schema version、source revision、display metadata和parameter schema。
2. node拥有stable UUID、type ID、node schema version、position、settings payload与enabled state。
3. pin拥有stable pin ID、direction、data type、cardinality、required/optional与default policy。
4. edge引用node/pin stable ID，不依赖数组索引或display name。
5. subgraph引用是typed asset reference，必须有递归/cycle规则与parameter mapping。
6. node type由registry贡献，包含owner plugin、capability、schema migration、validator、compiler factory和documentation token。
7. graph parameters区分compile-time、generation-time和per-instance override，不允许任意JSON在各层漂移。
8. source序列化必须canonical，unknown node/pin/settings可roundtrip并进入disabled diagnostic状态。

### 3.3 Typed data model

基础集合至少包含`PointSet`、`SurfaceField`、`VolumeField`、`SplineSet`、`AttributeTable`、`ParameterSet`、`ResourceSet`和`InstanceSet`。每个collection携带schema ID、bounds、coordinate space、element count、attribute domains、source revision和content digest。

Point至少携带stable element key、transform、bounds/extents、density、seed stream key、color/custom attributes与provenance。Surface/Volume必须提供明确sample/filter/project协议、精度与out-of-bounds行为。数据转换必须由typed adapter显式发生，禁止node通过字符串猜测字段类型。

### 3.4 Compiler与artifact

Compiler负责schema migration、node availability、pin compatibility、required input、cycle、subgraph recursion、parameter binding、determinism class、thread affinity、side-effect class、resource/capability和consumer reachability检查。成功输出应包含topological task DAG、typed port layout、constant-folded values、random stream assignments、dependency manifest、node cache policy、estimated cost与diagnostic map。

`CompiledPcgProgramArtifact`必须内容寻址并记录engine/plugin/compiler/schema版本。编译失败保留last-known-good artifact但标记stale，preview不得把旧结果显示成新revision成功。Cook只能使用与source revision及依赖digest完全匹配的artifact。

### 3.5 确定性与随机流

随机数不能使用共享顺序RNG。稳定stream key至少由graph ID、node stable ID、pin/output、generation seed、partition cell、element stable key和algorithm version组成。添加一个无关node、调整另一branch密度或改变task并行顺序，不应重排未受影响区域的结果。

浮点、hash、排序、空间边界与duplicate tie-break必须平台可复现或明确声明非确定性等级。WOC的独立seed offset和known vectors可作为初始golden，但需升级为版本化algorithm ID和cross-platform corpus。

### 3.6 Executor、cache与取消

Executor必须区分execution dependency与data dependency，支持CPU worker、main-thread consumer、GPU compute和external cook task affinity。每个task有request ID、generation ID、attempt、budget class、progress、cancel token和stale guard。取消后不得发布partial output；旧generation完成也不得覆盖新source revision。

Cache key至少包含compiled node digest、typed input digests、parameters、seed stream、bounds/cell、quality、platform与algorithm versions。cache entry应记录size、cost、last use、dependency reverse index和eviction reason。跨session DDC只能存immutable portable artifact，不能序列化live entity handle或GPU handle。

### 3.7 Spatial partition与增量失效

Generation request必须有明确domain bounds、partition level/cell、halo和consumer purpose。Compiler声明node是local、neighborhood、global reduction还是unbounded；只有local/neighborhood node可安全分区。输入变更通过dependency reverse index计算dirty graph nodes和dirty spatial cells，边界halo确保道路、坡度、碰撞等邻域查询不产生接缝。

World streaming加载cell时请求匹配artifact或按策略runtime generate；卸载时释放managed output；source/biome/terrain/spline变化只失效受影响cell。HLOD、nav、collision和minimap必须消费同一generation revision，禁止各自隐式重跑不同规则。

### 3.8 Generated resource ownership

每个生成元素拥有`GeneratedObjectId(graph, node, output, cell, stable element key)`和完整provenance。managed output记录owner request、source/artifact revision、consumer、resource handles、CRC/content digest、reuse state和cleanup policy。

Regeneration先计算add/update/remove/reuse diff，再原子提交。author-created对象永不被cleanup；manual edit必须按policy变为override、detached authored object或显式禁止。Preview、baked和runtime transient三种lifecycle不可混用。失败/取消只清理本attempt创建且尚未发布的资源。

### 3.9 Typed consumer adapters

1. Terrain adapter消费height/edit/layer field，输出Editor16定义的chunked Terrain source/artifact。
2. Scatter adapter消费InstanceSet，交给renderer-owned batch/GPU scene，不创建64K个Editor document entity。
3. Prefab adapter解析asset dependency、variant/override与stable instance identity。
4. Spline/Road/River adapter消费Editor39的SpatialSpline query/artifact，不复制polyline math。
5. Collision adapter按policy生成简化shape或instance collision set，并带generation ID。
6. Navigation adapter只接收已提交geometry/area/modifier artifact，生成Editor19的nav bake dependency。
7. Render adapter负责LOD、culling、wind、material、instance buffer与streaming residency。
8. Gameplay adapter只能生成静态placement definition/tag/region输入；live spawn、population和authority tick归Editor28。
9. Cook adapter聚合所有consumer receipt并验证同一source/artifact revision。

### 3.10 Biome与WorldRecipe

`BiomeSource`应定义stable biome ID、priority、blend policy、spatial field/mask inputs、climate/surface constraints、material/vegetation/decoration rule references和parameter overrides。Biome选择不能退化成一个字符串dropdown；必须支持overlap、blend、unknown biome、no-data与debug visualization。

`WorldRecipe`编排Terrain foundation、water/spline stamps、biome evaluation、scatter、prefab placement、collision/nav、HLOD/minimap和cook stages，但不吸收各domain compiler。每个stage声明typed inputs/outputs、spatial scope、dependencies、failure policy和receipt。Weather/Climate输入引用Editor38的版本化snapshot/artifact，不直接读取live mutable UI状态。

### 3.11 Editor authoring闭环

PCG toolkit至少包含真实node canvas、palette/search、typed pins、details、parameters、subgraph navigation、find、diagnostics、output/attribute viewer、debug object/cell selector、determinism、diff、profiling和log。所有node/edge/setting编辑进入document transaction，支持undo/redo、dirty/save/reopen、multi-select和copy/paste stable identity policy。

Generate支持selected node、selected bounds/cell、preview world和full bake，返回真实job/receipt。Validate只做compile/contract验证，不伪造instance count。Preview明确显示source revision、artifact generation、stale/LKG状态和consumer readiness。关闭文档、切项目、禁插件或shutdown时必须cancel/drain并释放preview resources。

### 3.12 Diagnostics、Cook与网络边界

Diagnostic至少带stable code、severity、graph/node/pin/cell、source revision、message、related asset和fix action token。运行指标覆盖compile/cache hit、node CPU/GPU time、data bytes、point/instance counts、dirty cells、cancel/stale drop、consumer publish和memory/residency。

Cook receipt记录graph source/artifact digest、dependency manifest、algorithm versions、seed policy、partition outputs和consumer receipts。多人协作提交source资产与可重建artifact policy，不提交不可解释的preview transient。若runtime generation参与联网世界，服务器/客户端必须共享算法/version/content digest，或只由authority生成并复制结果；不得假设相同seed自动等于网络确定性。

## 4. 优先级与重构清单

### 4.1 P0：必须先阻断错误完成信号

1. **P0-01** 在真实graph asset、compiler、job和receipt接通前，Scatter Generate/Validate不得返回64K instances、18 rules或1 conflict；改为明确Unavailable/Prototype且不声明queued成功。
2. **P0-02** 建立唯一PCG source/artifact/request/output authority；禁止以ZUI control state、WOC脚本函数或Scene当前状态分别充当第二套graph truth。
3. **P0-03** Terrain插件的DiagnosticOnly importer、无executor operations和无runtime consumer必须继续阻断PCG Terrain output发布，不得把descriptor registration算作consumer readiness。
4. **P0-04** WOC project-specific terrain/decoration port不得直接升级为公共engine API；先抽取确定性、typed data、partition和provenance contract，再通过adapter迁移golden。
5. **P0-05** 在managed generated resource、stable generated ID、regeneration diff和atomic cleanup建立前，禁止PCG直接创建/删除持久Scene entity、asset或collision/nav产物。

### 4.2 P1：工程化主线

#### 4.2.1 Source、schema与registry

1. **P1-01** 新增`PcgGraph`正式ResourceKind/marker、asset URI、artifact kind和catalog type。
2. **P1-02** 定义graph schema version、canonical serialization和unknown-field roundtrip。
3. **P1-03** 定义stable graph/node/pin/edge/parameter identity，禁止数组索引成为公共identity。
4. **P1-04** 建立node type registry，记录plugin owner、capability、schema version与compiler factory。
5. **P1-05** 为node settings建立typed schema、default、range、unit、asset reference和validator。
6. **P1-06** 建立pin direction、data type、cardinality、required/default和dynamic pin contract。
7. **P1-07** 定义subgraph/function引用、parameter mapping、recursion和cycle规则。
8. **P1-08** 区分compile-time、generation-time与instance override参数。
9. **P1-09** 建立graph/source migration registry、version fixture和失败恢复策略。
10. **P1-10** 将asset direct reference/dependency extraction接入node settings与subgraph引用。

#### 4.2.2 Typed data与compiler

11. **P1-11** 定义PointSet、SurfaceField、VolumeField、SplineSet、AttributeTable、ParameterSet、ResourceSet和InstanceSet。
12. **P1-12** 为所有collection定义schema ID、bounds、space、element count、digest和provenance。
13. **P1-13** 为point定义stable element key、transform、density、extents、seed key和attribute domains。
14. **P1-14** 定义surface/volume sample、project、filter、bounds与precision合同。
15. **P1-15** 建立attribute type registry、domain conversion、missing/default和rename/migration规则。
16. **P1-16** 实现pin compatibility、implicit conversion白名单和ambiguity diagnostic。
17. **P1-17** 实现node/subgraph cycle、required input、unreachable output和capability验证。
18. **P1-18** 编译为topological task DAG并区分execution/data dependency。
19. **P1-19** artifact记录compiled layout、constant fold、dependency manifest、cost estimate和diagnostic map。
20. **P1-20** artifact内容寻址并绑定engine/plugin/compiler/schema/algorithm版本。

#### 4.2.3 Determinism、executor与cache

21. **P1-21** 定义graph/node/pin/cell/element派生的稳定random stream算法。
22. **P1-22** 建立algorithm version和cross-platform hash/noise/sort/float golden corpus。
23. **P1-23** 为node声明deterministic、platform-stable、best-effort或non-deterministic等级。
24. **P1-24** 建立generation request ID、source/artifact revision、seed、bounds/cell、quality和purpose。
25. **P1-25** 实现worker/main-thread/GPU/external task affinity与合法切换。
26. **P1-26** 实现budget class、progress、pause、cancel、retry和shutdown drain。
27. **P1-27** 为task和publish建立generation/attempt stale guard，旧结果不得覆盖新revision。
28. **P1-28** 定义node cache key包含compiled digest、typed inputs、parameters、seed、cell、quality和platform。
29. **P1-29** 实现memory cache size/cost/LRU、dependency reverse index与eviction diagnostics。
30. **P1-30** 接入共享DDC保存portable immutable node/output artifact，拒绝live handle。
31. **P1-31** 支持last-known-good compiled artifact并在UI/receipt显式标记stale。
32. **P1-32** 记录cache hit/miss/reject reason、node wall/CPU/GPU time和data bytes。

#### 4.2.4 Spatial、Biome与WorldRecipe

33. **P1-33** 定义world bounds、partition level/cell、halo和coordinate space标准。
34. **P1-34** node声明local、neighborhood、global reduction或unbounded spatial class。
35. **P1-35** compiler验证分区合法性并对global/unbounded node生成明确计划或拒绝。
36. **P1-36** 由input dependency与bounds计算dirty node和dirty cell集合。
37. **P1-37** 实现边界halo、neighbor dependency和seam regression tests。
38. **P1-38** 定义runtime generation radius、cleanup radius、load/unload和priority policy。
39. **P1-39** 定义BiomeSource identity、field/mask、priority、blend与unknown/no-data行为。
40. **P1-40** 建立Biome overlap/blend、surface/climate constraint和debug output。
41. **P1-41** 将WOC zone/biome规则迁移为首个显式Biome/Rule golden，而非硬编码公共API。
42. **P1-42** 定义WorldRecipe stage DAG、typed input/output、failure policy和receipt。
43. **P1-43** WorldRecipe引用Terrain/Spline/Weather/Nav等domain artifact，不复制其算法。
44. **P1-44** 建立selected region/cell、whole world、runtime和cook四种generation profile。

#### 4.2.5 Output、ownership与consumer

45. **P1-45** 定义GeneratedObjectId与graph/node/output/cell/element provenance。
46. **P1-46** 建立managed resource container、CRC、used/reused/unused与preview/baked/runtime lifecycle。
47. **P1-47** regeneration先产出add/update/remove/reuse diff，再原子提交。
48. **P1-48** 定义manual edit为override、detach或forbidden的per-output policy。
49. **P1-49** 失败/取消只清理本attempt transient，不能删除已发布LKG或authored object。
50. **P1-50** Terrain adapter输出Editor16的chunk/edit/layer artifact与真实consumer receipt。
51. **P1-51** Scatter adapter输出batched InstanceSet并接Render/GPU Scene，禁止64K document entity。
52. **P1-52** Prefab adapter解析dependency、variant、override与stable instance identity。
53. **P1-53** Spline/Road/River adapter只消费Editor39的typed SpatialSpline artifact/query。
54. **P1-54** Collision与Navigation adapter绑定同generation revision并支持cell invalidation。
55. **P1-55** Gameplay adapter只发布placement definition，live spawn/population继续归Editor28。
56. **P1-56** HLOD、minimap、render、collision、nav与cook receipt必须聚合成同revision发布门。

#### 4.2.6 Editor authoring

57. **P1-57** 建立PCG asset toolkit和真实graph document/session，而非复用静态Scatter表格。
58. **P1-58** 实现node canvas、typed pins/edges、palette/search、comments、reroute与subgraph navigation。
59. **P1-59** 所有node/edge/settings编辑进入transaction，支持undo/redo、dirty/save/reopen。
60. **P1-60** 实现multi-select、copy/paste、duplicate、delete和stable identity policy。
61. **P1-61** details面板编辑typed parameters/settings并显示validation/migration状态。
62. **P1-62** Generate/Validate绑定真实operation payload、background job、progress、cancel和receipt。
63. **P1-63** 支持selected node、selected bounds/cell、preview world与full bake执行范围。
64. **P1-64** 建立debug object/cell selector、attribute/output viewer和per-node data inspection。
65. **P1-65** 建立determinism、graph diff、profiling、log和cache inspection视图。
66. **P1-66** UI持续显示source revision、artifact generation、stale/LKG和consumer readiness。

#### 4.2.7 Cook、diagnostics与验证

67. **P1-67** 定义stable diagnostic code、graph/node/pin/cell定位、related asset与fix action。
68. **P1-68** 定义cook manifest/receipt、dependency/algorithm digest、partition outputs与consumer receipts。
69. **P1-69** 建立graph roundtrip/migration、compiler golden、determinism、cache invalidation、cancel/stale和ownership test matrix。
70. **P1-70** 建立64K/1M points、partition churn、incremental edit、memory/cache pressure和cross-platform性能资格门。

### 4.3 P2：主线完成后扩展

1. **P2-01** GPU-native point processing与CPU/GPU graph partition optimizer。
2. **P2-02** Custom compute source/HLSL/WGSL node sandbox、resource limits与offline validation。
3. **P2-03** Distributed/remote world generation worker和artifact merge。
4. **P2-04** Hierarchical grammar、shape grammar和assembly generation node library。
5. **P2-05** ML-assisted rule suggestion，只输出可审查source diff而不直接发布世界。
6. **P2-06** Runtime adaptive PCG按player density、platform budget或season切换profile。
7. **P2-07** PCG data CSV/JSON/columnar import/export与external dataset provider。
8. **P2-08** 可视化density/attribute/flow field、heatmap与3D volume inspection。
9. **P2-09** 多用户graph协作、node-level presence和semantic merge。
10. **P2-10** 生成结果时间旅行、request replay和cross-build diff。
11. **P2-11** Biome生态演替、succession和长期world evolution simulation。
12. **P2-12** Marketplace node package签名、compatibility和determinism certification。

## 5. 分层实施路线

| 里程碑 | 内容 | 退出条件 |
|---|---|---|
| M0 | Capability truth与静态UI降级 | Scatter不再伪造queued/count/conflict，Terrain diagnostic-only状态可见 |
| M1 | PcgGraph source/schema/registry | graph/node/pin/edge/parameter可canonical roundtrip并迁移 |
| M2 | Typed data与compiler | typed pin验证、subgraph/cycle诊断和compiled artifact golden通过 |
| M3 | Determinism与request identity | 稳定random stream、cross-platform known vectors和stale guard通过 |
| M4 | Executor/jobs/cache/DDC | budget/cancel/pause、cache key/eviction和portable artifact闭环 |
| M5 | Spatial partition/invalidation | cell/halo、dirty propagation、streaming load/unload和seam tests通过 |
| M6 | Managed generated resources | stable ID、provenance、diff、reuse、cleanup和manual override安全 |
| M7 | Terrain与Scatter垂直切片 | 一个真实graph输出chunked Terrain输入及batched instances，receipt同revision |
| M8 | Prefab/Spline/Collision/Nav adapters | typed跨域输出、cell invalidation和consumer readiness闭环 |
| M9 | Biome与WorldRecipe | overlap/blend、stage receipt和WOC golden迁移完成 |
| M10 | Editor toolkit与inspection | canvas/transaction/preview/diff/determinism/profiling/log可用 |
| M11 | Cook、规模、故障与发布 | partition cook、64K/1M性能、取消恢复、跨平台/联网政策通过 |

M0-M4不得等待完整Terrain renderer；它们建立独立可测的PCG kernel。M5-M9不得绕开Editor16/19/28/38/39定义的domain authority。M10不允许使用固定count补齐视觉页面，所有数字必须来自generation/consumer receipt。

## 6. 验收门

1. **G01** production catalog存在唯一PcgGraph asset kind、schema和owner plugin。
2. **G02** graph/node/pin/edge stable ID在save/reopen与无关排序变化后不漂移。
3. **G03** unknown/newer node settings可以roundtrip并给出禁用诊断。
4. **G04** pin type/cardinality/cycle/subgraph recursion错误均有stable diagnostic定位。
5. **G05** compiled artifact绑定source、dependency、engine/plugin/compiler/schema digest。
6. **G06** 同source/input/seed/cell跨线程顺序生成相同element IDs与content digest。
7. **G07** 插入无关node不重排未受影响branch/cell的随机结果。
8. **G08** supported Windows/Linux平台通过hash/noise/sort/float known vectors。
9. **G09** cancel后无partial output发布，旧generation完成不能覆盖新revision。
10. **G10** last-known-good preview明确标记stale且cook拒绝stale artifact。
11. **G11** cache key变化与dependency reverse invalidation覆盖所有typed inputs。
12. **G12** DDC artifact不包含entity、pointer、thread、GPU或process-local handle。
13. **G13** local/neighborhood/global/unbounded node的partition合法性可验证。
14. **G14** halo和相邻cell seam golden无缺口、重复或随机跳变。
15. **G15** source局部编辑只重跑受影响node/cell，指标可证明增量范围。
16. **G16** streaming cell load/unload正确创建、复用和释放managed output。
17. **G17** 每个generated object可追溯graph/node/output/cell/element/source revision。
18. **G18** regeneration diff准确区分add/update/remove/reuse并原子提交。
19. **G19** cleanup不会删除authored、detached、override或其他generator拥有的对象。
20. **G20** preview/baked/runtime transient资源生命周期和保存策略互不混淆。
21. **G21** Terrain adapter只有真实backend/artifact/consumer ready时才能发布成功。
22. **G22** 64K Scatter输出走batched instance backend，Editor document entity数量不随实例线性增长。
23. **G23** Collision、Nav、Render、HLOD/minimap/cook receipt引用同generation revision。
24. **G24** Gameplay live spawn/population没有被PCG graph直接修改。
25. **G25** Biome overlap、blend、unknown与no-data行为有golden和viewport debug。
26. **G26** WOC terrain/decoration known vectors迁移后与固定source contract一致。
27. **G27** `npm run check:m3-terrain`继续通过，且项目codegen不成为engine runtime依赖。
28. **G28** PCG canvas所有编辑支持transaction、undo/redo、dirty/save/reopen。
29. **G29** Generate/Validate显示真实job ID、progress、cancel、diagnostic和receipt，不出现固定64K/18/1数据。
30. **G30** debug object/cell、attribute viewer、determinism、diff、profiling和cache inspection可追踪同request。
31. **G31** 64K/1M点、partition churn、cancel storm、cache pressure和shutdown有预算/内存资格证据。
32. **G32** clean checkout的compile/test/cook/pack与支持平台矩阵通过，故障可复现并有artifact bundle。

## 7. 实施纪律

1. 先完成P0 truth和M1-M4 kernel，再恢复Scatter产品声明；不能先美化静态页面。
2. PCG core不依赖Terrain、Foliage、Prefab或WOC具体类型，跨域只走typed adapter。
3. 不复制Editor16的Terrain/World Partition、Editor28的Spawn Rules、Editor38的Weather或Editor39的Spline算法。
4. WOC只作为第一个determinism/biome/scatter golden corpus，项目source pin与engine schema分别版本化。
5. 所有side effect在managed output commit阶段发生；普通transform/filter node必须纯函数化。
6. 任一阶段若只能返回descriptor、route或固定feedback，capability必须保持Unavailable/Partial。
7. 每个里程碑同时交付schema fixture、failure test、observability和migration，不接受仅happy path。
8. 实施前重算132文件manifest和fingerprint，复核2个在途Editor文件，禁止覆盖用户/其他会话改动。

## 8. 本轮输出

本轮新增本review文档并登记到`zircon_editor/index.md`、根`index.md`与`coverage.md`。未修改production Runtime、Editor、plugin、interface代码或tests。WOC terrain codegen check通过；Editor动态测试仍受既有编译错误阻断，未重复相同lane。
