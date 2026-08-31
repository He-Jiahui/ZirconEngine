---
title: Editor Procedural Content Generation、Rule Graph、Biome 与 World Generation 当前源码复审
category: zircon_editor
report_id: Editor161
review_date: 2026-08-27
baseline_head: 5a0a44b7a169e3d03a85b235251f8113802f2ea3
verification_head: e7f88758192bd1e8aa2cb619969825c0c4b152d5
canonical_owner: Editor40
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md
  - docs/plans/optimize/zircon_editor/114-editor-procedural-content-generation-rule-graph-biome-world-generation-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zv-runtime-vegetation-tree-foliage-grass-species-instancing-wind-animation-billboard-impostor-lod-streaming-scalability-editor-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/138-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/149-editor-spawn-rules-encounter-population-world-state-scenario-quest-authority-current-source-review.md
  - docs/plans/optimize/zircon_editor/159-editor-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-current-source-review.md
  - docs/plans/optimize/zircon_editor/160-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
related_handoffs:
  - docs/plans/zircon_plugins/10/failure-2026-08-01-terrain-tilemap-scene-mode-factories-missing.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-27-dynamic-component-property-world-generation.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/terrain
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/graphics/scene/gpu_scene
  - examples/woc/contracts/m3_terrain_content.json
  - examples/woc/scripts/woc_game/src/world
  - examples/woc/tools
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCG
  - dev/UnrealEngine/Engine/Plugins/PCG/Source/PCGEditor
  - dev/godot/modules/noise
  - dev/godot/scene/resources/multimesh.cpp
  - dev/Fyrox/fyrox-impl/src/scene/terrain
  - dev/Fyrox/editor/src/interaction/terrain.rs
  - dev/Fyrox/editor/src/scene/commands/terrain.rs
  - dev/bevy/crates/bevy_asset/src/event.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor161 · PCG / Rule Graph / Biome / World Generation 当前源码复审

## 1. 结论

当前 Zircon 仍没有引擎级 Procedural Content Generation 产品。tracked production roots 与 **1,684 个 untracked Rust/TOML/ZUI/Zr 物理文件**中，对 `PcgGraph | PCGGraph | ProceduralGraph | RuleGraph | BiomeAsset | BiomeSource | WorldRecipe | CompiledPcg | GenerationOutput | GeneratedObjectId | ManagedGenerated | PartitionActor` 的精确组合扫描均为 **0 命中**。`ResourceKind` 仍只有 26 类，没有 PCG Graph、Biome、World Recipe、Point Data 或 Generated Set；`zircon_plugins` 只有 Terrain/Prefab 等相邻包，没有 PCG/Procedural/Biome package。`WorldGeneration` 是运行时 World mutation revision，不参与持久化相等性，不是内容生成系统。

Editor 的产品表面与这一事实相反。Scatter Workspace 是 **233 行、27 个 node、19 条 event route、0 个 provider/controller/document/job/artifact binding**，固定显示 `SC_Forest`、Biome Mask、Slope Filter、Rocks + Ferns、64K instances 与 1 conflict。Generate/Validate 最终只进入 preview-action route，`extension_module_feedback.rs` 固定返回 `Generate queued SC_Forest 64K instances` 与 `Validation queued 18 rules 1 conflict`；seed/density/ruleset 提交也只是 control-local 字符串。它没有 graph source、operation factory、job ticket、generation ID、artifact或 runtime snapshot。这仍是 P0 产品真相断路。

Terrain 不能被当作 PCG backend。当前插件仍只发布 command/importer/toolkit metadata；`EditorAuthoringContributionBatch` 没有 operation factory 字段，Terrain batch 也没有 graph editor、palette 或 scene mode。runtime importer 仍为 `DiagnosticOnlyAssetImporter`，固定诊断 backend 未安装；`TerrainAsset` 仍是 inline height samples/layers carrier，World save 固定写 `terrain: None`。first-party runtime catalog 的 generated-manifest test 已静态包含 Terrain manifest，但 executable registration switch、Editor catalog 和 App feature仍不选择 Terrain。这项静态发现能力不能提升 PCG/Terrain runtime readiness。

仓库确有应保留的共享底座：Editor 有 graph/palette descriptor、operation factory contract、transaction/journal 与有界 job admission/cancel/progress/shutdown；Runtime 有 content-addressed asset artifact store、Scene carrier、World generation revision和 GPU Scene stable instance span/dirty upload。它们没有任何 PCG consumer。普通 mesh production sync 仍以 `gpu_scene.register(..., 1)` 注册单实例，因此也不能把潜在 multi-instance span 解释为 64K Scatter output backend。

WOC 继续是有价值但边界清楚的项目 oracle：固定 source commit `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`、source/catalog SHA-256、hash/noise/height known vectors、显式 seed offsets、10-yard candidate lattice、zone/biome/camp/road/lake contract与局部 collision cell replay均可迁移。它的输出仍是项目专用 `.zr` 函数；biome分支、filter顺序和14条道路查询均硬编码，`roadDistance` 对每个 candidate 全量遍历所有 segment，没有 typed graph、node cache、stable generated ID、provenance diff、managed resource、cell artifact或 Render/Scene/Nav/Cook install。不得把 WOC 函数包进 Generate 按钮冒充PCG。

本轮保持 Editor40 的 canonical finding 数不变：**5 项 P0 全部 Open；70 项 P1 全部 Open；12 项 P2 全部 Open；32 项资格门全部 Fail**。没有动态证据支持PCG/Terrain/Scatter性能或表现达到、接近或优于 Unreal；在相同内容、画质、硬件、平台和失败矩阵下取得可复现结果前，禁止作此声明。

## 2. 审查范围、统计与 currentness

统计基于 working-tree 物理文件。行数为物理行；tests/ignored 只统计精确 Rust `#[test]` / `#[ignore]` 声明。fingerprint 按 lowercase repository-relative path 排序，将 `path + NUL + lowercase(file SHA-256) + LF` 拼接后再取 SHA-256。选择集用于复现本报告，不代表参考仓库全部规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Editor 产品面与共享 authoring | **99 / 24,612 / 22,455 / 925,761 / 184 / 8** | `f1994d9845d6c111a1d85d952739a3416cd4a133114b163d99cdd3deea2d8f33` |
| Runtime / Terrain / catalog / output consumer | **144 / 29,915 / 27,353 / 1,073,918 / 168 / 5** | `2c2130d78e58deffeb277fc795210caf73a73f7d49ede193e7dcb1b53ef6ea8b` |
| WOC deterministic corpus | **13 / 6,584 / 6,362 / 192,884 / 0 / 0** | `97e59a4fea7de4d69ceb4c9b0f57f49cb1e9846d3bd9007b79e87217012c2247` |
| Zircon selected union | **256 / 61,111 / 56,170 / 2,192,563 / 352 / 13** | `b5459ae935e320a7bef22890be450b6489f6826259ba4c15e03b765ff71e7fb3` |
| Unreal / Godot / Fyrox / Bevy / Unity Graphics selected | **36 / 28,809 / 24,617 / 1,188,483 / 11 / 0** | `2e9a5085eda0a93aeed8f972cbc3c0e444def9217c7d075982628e3c28543863` |
| All selected | **292 / 89,920 / 80,787 / 3,381,046 / 363 / 13** | `987146fd2ec395949f695ed8041452f348a6ce75bec9ac2811f5938083a0de6b` |

- baseline HEAD 为 `5a0a44b7a169e3d03a85b235251f8113802f2ea3`，commit time 为 `2026-08-27T06:59:02+08:00`；首次选择集冻结时 HEAD 已移动到 verification HEAD。
- 工作树含大量用户与其他 Session 的修改/新增文件；本报告读取物理文件，不回退、不覆盖，也不把在途内容写成已集成能力。
- Unreal `PCG/Public/PCGSubsystem.h` 当前只是5行转发头；本轮已改读 `Public/Subsystems/PCGSubsystem.h`、`PCGEngineSubsystem.h` 与 `RuntimeGen/PCGRuntimeGenScheduler.h`。PCG Runtime/Editor目录规模核对为 **1,442文件、353,676行**，只用于说明审查复杂度，不作为质量或性能结论。
- 按用户要求未查询、轮询、等待或实时跟踪协调器。两个 open handoff 只作为静态边界：Terrain factory handoff直接相关；dynamic property `world generation` handoff是revision一致性问题，不是PCG实现，也不阻塞本轮review。
- Tooling按用户要求排除；WOC extractor/codegen只做静态取证。本轮未运行Cargo、Editor、PCG、Tooling、cook、WGPU、determinism、fault、scale、soak或benchmark。

## 3. Editor114之后的 current-source 重判

| 主题 | 当前变化 | 对finding的影响 |
|---|---|---|
| PCG identity | tracked production与1,684个untracked source文件的精确领域类型仍为0 | P0-02、P1-01至P1-20保持Open |
| Scatter产品面 | 233行/27 nodes/19 routes继续固定SC_Forest/64K/18/1；route只进入preview registry与fixed feedback | P0-01保持Open，G29继续Fail |
| Terrain发现 | generated-manifest test已包含Terrain manifest；executable catalog/App/Editor provider仍没有Terrain | 只能保留manifest discovery，P0-03与P1-50保持Open |
| Terrain执行 | import plan仍只有格式/尺寸检查；runtime importer仍DiagnosticOnly；World save仍`terrain: None` | PCG Terrain output不能发布，G21继续Fail |
| Graph扩展底座 | `GraphEditorDescriptor`、palette descriptor与extension store存在 | 可供M10接入；没有PCG asset/document/controller，P1-57至P1-66仍Open |
| Job/transaction底座 | job已有admission key、bytes/age预算、dependency、cancel/progress/shutdown；operation factory与durable journal存在 | 可供M4/M10复用；没有PCG task/context/receipt，P1-24至P1-32仍Open |
| Artifact/GPU底座 | asset artifact store与GPU Scene stable span/dirty upload存在 | 可作为consumer primitive；普通mesh仍单实例、PCG adapter为0，P1-30/P1-51仍Open |
| WOC corpus | source pin、digest、known vector、candidate/collision replay继续存在 | 可迁移golden；仍无PCG source/artifact，P0-04与P1-41保持Open |
| Unreal currentness |真实World owner已迁到`Subsystems/PCGSubsystem.h`并新增execution-source/runtime scheduler边界 | 更新目标架构与门禁，不改变Zircon状态 |

## 4. 当前产品纵链事实

| 链路 | 当前源码事实 | 判定 |
|---|---|---|
| Asset identity | 26类ResourceKind没有PcgGraph/Biome/WorldRecipe/GeneratedSet | Open |
| Package / catalog / App | 无PCG package、runtime/editor provider、selection/profile或capability receipt | Open |
| Source / graph | 无versioned graph、stable node/pin/edge/parameter identity、subgraph或migration | Open |
| Typed data | 无PointSet/SurfaceField/VolumeField/SplineSet/Attribute/InstanceSet | Open |
| Compiler | 无type/cycle/bounds/capability validator、task DAG、artifact或diagnostic map | Open |
| Executor / cache | 无PCG request、task context、stale guard、node cache、DDC或LKG | Open |
| Spatial / Biome | 无cell/halo/dirty propagation/BiomeSource/overlap/blend/WorldRecipe | Open |
| Output ownership | 无GeneratedObjectId、managed resource、diff/reuse/override/orphan cleanup | Open |
| Consumer adapters | Terrain/Foliage/Prefab/Spline/Collision/Nav/Render/Cook均无PCG adapter | Open |
| Editor toolkit | Scatter只有静态表格与preview action；无document/canvas/transaction/debug/receipt | Open |
| Terrain | typed carrier与plan存在；runtime/World/render执行断裂 | Open |
| GPU instances |通用GPU Scene可分配span，production mesh consumer仍固定单实例 | Open |
| WOC |项目级deterministic函数与golden存在；无engine artifact/install | Open |
| Evidence |无source-to-output、save/reopen、cancel/stale、fault、scale或cross-platform动态证据 | Open |

## 5. P0：必须先阻断的产品与authority断路

| ID | 状态 | 当前问题 | 必须重构为 |
|---|---|---|---|
| P0-01 | Open | Scatter Generate/Validate仍固定返回64K/18/1与queued | 未接真实graph/job/receipt前隐藏或标记Prototype/Unavailable；只投影typed terminal receipt |
| P0-02 | Open | 无唯一PCG source/artifact/request/output authority，ZUI/WOC/Scene可被误当三套truth | 建`PcgGraphSource -> CompiledPcgProgramArtifact -> GenerationRequest -> GenerationOutput`单链 |
| P0-03 | Open | Terrain DiagnosticOnly、operation无factory、World/render无consumer，却可被UI暗示为output backend | Terrain provider/consumer readiness必须由Runtime142与Editor138真实receipt原子发布 |
| P0-04 | Open | WOC项目脚本可能被直接升级为公共engine API | 只迁移determinism/data/partition/provenance golden，经versioned adapter进入PCG |
| P0-05 | Open | 无managed generated resource/stable ID/diff/cleanup，PCG若直写Scene会破坏authoring authority | 先建立owner-qualified immutable output、atomic diff与override/detach/orphan policy |

## 6. P1：70项 canonical 工程化主线

以下ID与Editor40一一对应；共享底座只记为“可复用”，没有PCG consumer时不把finding降为Partial。

### 6.1 Source、schema与registry

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| P1-01 | Open | 无PcgGraph ResourceKind/marker/asset URI/artifact kind/catalog type；按版本化正式资源新增 |
| P1-02 | Open | 无graph schema version、canonical serialization与unknown-field roundtrip |
| P1-03 | Open | 无stable graph/node/pin/edge/parameter identity；禁止数组索引成为公共identity |
| P1-04 | Open | 无node type registry；registry必须记录plugin owner、capability、schema与compiler factory |
| P1-05 | Open | 无typed node settings/default/range/unit/asset reference/validator |
| P1-06 | Open | 无pin direction/data type/cardinality/required/default/dynamic-pin合同 |
| P1-07 | Open | 无subgraph/function引用、parameter mapping、recursion与cycle规则 |
| P1-08 | Open | compile-time/generation-time/instance override参数未分层 |
| P1-09 | Open | 无graph/source migration registry、version fixtures与失败恢复 |
| P1-10 | Open | 通用asset dependency extraction没有node settings/subgraph consumer |

### 6.2 Typed data与compiler

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| P1-11 | Open | 定义PointSet、SurfaceField、VolumeField、SplineSet、AttributeTable、ParameterSet、ResourceSet、InstanceSet |
| P1-12 | Open | collection缺schema ID、bounds、space、element count、digest与provenance |
| P1-13 | Open | point缺stable element key、transform、density、extents、seed key与attribute domain |
| P1-14 | Open | surface/volume sample/project/filter/bounds/precision合同不存在 |
| P1-15 | Open | 无attribute type registry、domain conversion、missing/default、rename/migration |
| P1-16 | Open | 无pin compatibility、受控implicit conversion和ambiguity diagnostic |
| P1-17 | Open | 无node/subgraph cycle、required input、unreachable output与capability验证 |
| P1-18 | Open | 无topological task DAG及execution/data dependency分离 |
| P1-19 | Open | 无compiled layout、constant fold、dependency manifest、cost estimate与diagnostic map |
| P1-20 | Open | 无绑定engine/plugin/compiler/schema/algorithm版本的content-addressed artifact |

### 6.3 Determinism、executor与cache

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| P1-21 | Open | 无graph/node/pin/cell/element派生的稳定random stream算法 |
| P1-22 | Open | WOC known vectors未迁入cross-platform hash/noise/sort/float PCG corpus |
| P1-23 | Open | node没有deterministic/platform-stable/best-effort/non-deterministic等级 |
| P1-24 | Open | 无request ID、source/artifact revision、seed、bounds/cell、quality与purpose |
| P1-25 | Open | 无PCG worker/main-thread/GPU/external affinity与合法切换 |
| P1-26 | Open | Editor job底座存在；PCG仍无budget class、progress、pause、cancel、retry与shutdown drain |
| P1-27 | Open | 无task/publish generation-attempt stale guard |
| P1-28 | Open | 无覆盖compiled/input/parameter/seed/cell/quality/platform的node cache key |
| P1-29 | Open | 无PCG memory cache cost/LRU、reverse dependency与eviction diagnostic |
| P1-30 | Open | 通用artifact store不是PCG DDC；无portable immutable node/output artifact |
| P1-31 | Open | 无last-known-good compiled artifact与显式stale UI/receipt |
| P1-32 | Open | 无PCG cache reason、node wall/CPU/GPU time与data-byte指标 |

### 6.4 Spatial、Biome与WorldRecipe

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| P1-33 | Open | 无world bounds、partition level/cell、halo与coordinate-space标准 |
| P1-34 | Open | node没有local/neighborhood/global reduction/unbounded spatial class |
| P1-35 | Open | compiler不能验证partition合法性或拒绝global/unbounded node |
| P1-36 | Open | 无由input dependency与bounds推导dirty node/cell的机制 |
| P1-37 | Open | 无halo、neighbor dependency与seam regression |
| P1-38 | Open | 无runtime generation/cleanup radius、load/unload与priority policy |
| P1-39 | Open | 无BiomeSource identity、field/mask、priority、blend、unknown/no-data行为 |
| P1-40 | Open | 无Biome overlap/blend、surface/climate constraint与debug output |
| P1-41 | Open | WOC zone/biome仍为硬编码函数，尚未迁移为显式Biome/Rule golden |
| P1-42 | Open | 无WorldRecipe stage DAG、typed I/O、failure policy与receipt |
| P1-43 | Open | 无对Terrain/Spline/Weather/Nav domain artifact的typed引用 |
| P1-44 | Open | 无selected region/cell、whole world、runtime与cook generation profiles |

### 6.5 Output、ownership与consumer

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| P1-45 | Open | 无GeneratedObjectId及graph/node/output/cell/element provenance |
| P1-46 | Open | 无managed resource container、CRC、used/reused/unused与preview/baked/runtime lifecycle |
| P1-47 | Open | 无add/update/remove/reuse regeneration diff与atomic commit |
| P1-48 | Open | 无per-output manual override/detach/forbidden policy |
| P1-49 | Open | 无attempt transient/LKG/authored object隔离的失败与取消清理 |
| P1-50 | Open | Terrain adapter没有chunk/edit/layer artifact和真实consumer receipt |
| P1-51 | Open | GPU Scene只提供通用span且production mesh为单实例；无batched PCG InstanceSet adapter |
| P1-52 | Open | 无Prefab dependency/variant/override/stable instance adapter |
| P1-53 | Open | 无对Editor39 SpatialSpline artifact/query的Road/River adapter |
| P1-54 | Open | Collision/Nav没有同generation revision与cell invalidation adapter |
| P1-55 | Open | 无placement-only Gameplay adapter；PCG不得接管live spawn authority |
| P1-56 | Open | HLOD/minimap/render/collision/nav/cook receipts没有同revision聚合门 |

### 6.6 Editor authoring

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| P1-57 | Open | GraphEditorDescriptor可复用；无PCG toolkit和真实document/session |
| P1-58 | Open | Scatter表格不是canvas；无typed pins/edges、palette/search/comment/reroute/subgraph navigation |
| P1-59 | Open | 无PCG node/edge/settings transaction、undo/redo、dirty/save/reopen |
| P1-60 | Open | 无multi-select/copy-paste/duplicate/delete与stable identity policy |
| P1-61 | Open | details只改字符串；无typed setting validation/migration projection |
| P1-62 | Open | Generate/Validate仍走preview action；无operation factory/job/progress/cancel/receipt |
| P1-63 | Open | 无selected node/bounds/cell、PreviewWorld与full bake scope |
| P1-64 | Open | 无debug object/cell selector、attribute/output viewer与per-node inspection |
| P1-65 | Open | 无PCG determinism/diff/profiling/log/cache inspection产品视图 |
| P1-66 | Open | UI不显示source revision、artifact generation、stale/LKG或consumer readiness |

### 6.7 Cook、diagnostics与验证

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| P1-67 | Open | 无stable diagnostic code及graph/node/pin/cell/asset/fix-action定位 |
| P1-68 | Open | 无cook manifest/receipt、dependency/algorithm digest、partition与consumer receipts |
| P1-69 | Open | 无roundtrip/migration/compiler/determinism/cache/cancel/stale/ownership矩阵 |
| P1-70 | Open | 无64K/1M point、partition churn、incremental edit、memory/cache与cross-platform资格证据 |

## 7. P2：主线闭合后能力

| ID | 状态 | 当前差距 |
|---|---|---|
| P2-01 | Open | GPU-native point processing与CPU/GPU graph partition optimizer |
| P2-02 | Open | custom compute/HLSL/WGSL node sandbox、resource limits与offline validation |
| P2-03 | Open | distributed/remote generation worker与artifact merge |
| P2-04 | Open | hierarchical/shape grammar与assembly node library |
| P2-05 | Open | ML-assisted rule suggestion只输出可审查source diff |
| P2-06 | Open | runtime adaptive PCG profile切换与可重演policy |
| P2-07 | Open | CSV/JSON/columnar/external dataset provider |
| P2-08 | Open | density/attribute/flow/heatmap/3D volume inspection |
| P2-09 | Open | 多用户graph协作、node presence与semantic merge |
| P2-10 | Open | generation request replay、time travel与cross-build diff |
| P2-11 | Open | Biome生态演替与长期world evolution simulation |
| P2-12 | Open | Marketplace node签名、compatibility与determinism certification |

## 8. 五套参考源码映射

| 参考 | 本轮确认的工程合同 | Zircon应采用 | 不应误用 |
|---|---|---|---|
| Unreal PCG Runtime | Graph/Node/Pin/Data、compiled task DAG、execution/data dependency、cache memory budget/LRU、World/Engine subsystem、RuntimeGen scheduler、PartitionActor、managed resource soft/hard release、inspection generation | 作为PCG完整产品主参考，吸收typed source/data、per-World owner、bounded task/cache、partition与managed lifecycle | 不复制UObject/Actor布局；过期5行转发头不能当当前subsystem证据 |
| Unreal PCG Editor | Determinism、graph diff、profiling、log、attribute list与node execution inspection均是独立产品视图 | M10必须从同request/stack/artifact读取真实数据 | 不只复制窗口外观或固定行数据 |
| Godot | FastNoiseLite将seed/noise参数序列化，NoiseTexture有异步更新边界，MultiMesh通过RenderingServer维护bulk buffer/visible count/AABB | noise primitive必须版本化；InstanceSet通过批量GPU consumer而非大量Scene entity | Godot本地源码没有完整PCG authoring，不作为graph parity证据 |
| Fyrox | Terrain node、quadtree、raycast、brush thread与可逆chunk swap构成小而真实的source-to-runtime闭环 | Terrain adapter最低必须达到真实artifact/query/edit/undo，不接受DiagnosticOnly | 不把较小Terrain实现当大世界PCG终点 |
| Bevy | AssetEvent Added/Modified/Removed/LoadedWithDependencies及render-world extraction/change tracking/batching | generated diff与asset dependency事件应支持精确增量consumer | 没有内建PCG editor，不能替代source/compiler/managed output |
| Unity Graphics | InstanceDataSystem与InstanceCuller拥有分配/free-list、GPU data、LOD/culling与visibility输出 | 作为64K/1M InstanceSet下游数据布局、allocation与culling参考 | Graphics包不拥有PCG World/Editor authority |

## 9. 目标架构与owner边界

```text
Editor-owned authoring truth
  PcgGraphSource / BiomeSource / WorldRecipeSource
    -> stable graph/node/pin/edge/parameter identity
    -> transactional document + revision + atomic save
        |
        v
Runtime-owned neutral compile truth
  PcgCompiler
    -> CompiledPcgProgramArtifact
       {typed DAG, dependency manifest, algorithm versions, diagnostics, cost}
        |
        v
Runtime-owned per-World generation truth
  PcgWorldService
    -> GenerationRequest(request, revision, seed, cell, quality, purpose)
    -> bounded executor + node cache + stale guard + partition scheduler
    -> immutable GenerationOutput + ManagedGeneratedResource + atomic diff
        |
        +--> Terrain / Foliage / Prefab / SpatialSpline typed adapters
        +--> Render / Collision / Navigation / HLOD / Cook receipts
        +--> SpawnDefinition only; never direct live population mutation
        +--> Editor observation: snapshot, progress, diagnostics, inspection
```

- Editor161/Editor40只拥有PCG graph/Biome/WorldRecipe authoring、document、operation、preview和inspection产品面。
- Runtime应新增中立PCG kernel与per-World service owner；它不能放在Editor、App、WOC或Terrain plugin内部。
- Runtime142/Editor138拥有Terrain/Foliage/World Partition；Runtime Vegetation owner拥有species/cluster/wind/LOD；Editor160拥有SpatialSpline/Road/River；Editor159拥有Weather；Editor149拥有live Spawn/Population authority。PCG只能消费typed adapter，不能复制各域算法。
- App只选择project/target/profile/provider并托管生命周期；不得持有graph data、generated output、cache或固定反馈。
- source、compiled artifact、generation output、runtime installed resource和Editor projection必须使用不同类型与generation，不得双写。

## 10. 依赖有序重构里程碑

| 里程碑 | 必须交付 | 退出条件 |
|---|---|---|
| M0 Truth Freeze | 隐藏/禁用Scatter固定结果；建立缺PCG/Terrain provider、fixed feedback与WorldGeneration误认的RED证据 | UI只显示typed unavailable，不再出现伪queued/64K/18/1 |
| M1 Source/Schema/Registry | PcgGraph/Biome/WorldRecipe资源、stable IDs、typed settings/pins、migration、dependency extraction | canonical roundtrip/unknown/newer schema fail-close |
| M2 Typed Data/Compiler | Point/Surface/Volume/Spline/Attribute/Instance data与type/cycle/partition validator、compiled artifact | deterministic compile golden与diagnostic定位通过 |
| M3 Request/Determinism | request identity、random stream、algorithm version、cross-platform WOC-derived known vectors | thread order与无关node变更不扰动未影响output |
| M4 Executor/Cache/DDC | per-World bounded task executor、affinity、cancel/retry/shutdown、node cache/LKG/portable artifact | cancel/stale/cache/fault/budget矩阵通过 |
| M5 Spatial/Biome/Partition | bounds/cell/halo/dirty propagation、Biome overlap/blend、generation/cleanup radius | seam与局部失效证据通过 |
| M6 Managed Output | GeneratedObjectId、provenance、add/update/remove/reuse、override/detach/orphan cleanup | 失败不损坏authored/LKG，atomic diff通过 |
| M7 Terrain/Scatter Slice | 真实Terrain adapter与batched InstanceSet接Runtime consumer | 一个graph source-to-render/query/receipt纵链通过 |
| M8 Cross-Domain Adapters | Prefab/Spline/Collision/Nav/HLOD/Cook/SpawnDefinition同revision receipts | 无跨域第二authority，cell invalidation闭合 |
| M9 WorldRecipe/WOC Migration | stage DAG、profile、WOC zone/biome/decoration golden迁移 | 项目codegen不成为engine runtime依赖 |
| M10 Editor Product | toolkit/canvas/transaction/preview/attribute/determinism/diff/profile/log/cache inspection | 所有显示数据来自同request/runtime snapshot |
| M11 Release Qualification | cook/headless/network/save/replay、64K/1M、fault/scale/soak/platform/benchmark | 32门全部通过后才可提升maturity |

M0-M4建立独立PCG kernel，不等待完整Terrain renderer。M5-M9必须等待各domain typed adapter，不能复制简化版。MVP `docs/plans/mvp/00-engine-grade-mvp-baseline.md`仍为In Progress；当前只允许review与计划，不开始advanced PCG实现。

## 11. G01-G32 资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 | Fail | production catalog无唯一PcgGraph kind/schema/owner plugin |
| G02 | Fail | graph/node/pin/edge stable ID不存在 |
| G03 | Fail | unknown/newer node settings无roundtrip与禁用诊断 |
| G04 | Fail | type/cardinality/cycle/subgraph recursion无compiler诊断 |
| G05 | Fail | 无绑定source/dependency/tool/schema/algorithm的compiled artifact |
| G06 | Fail | 无跨线程稳定element ID/content digest |
| G07 | Fail | 无branch/cell random stream隔离 |
| G08 | Fail | WOC known vectors尚未成为engine PCG跨平台门 |
| G09 | Fail | 无PCG cancel与stale publish guard |
| G10 | Fail | 无LKG/stale preview/cook拒绝合同 |
| G11 | Fail | 无PCG cache key与reverse invalidation |
| G12 | Fail | 无portable DDC artifact与live-handle拒绝 |
| G13 | Fail | 无spatial class与partition legality |
| G14 | Fail | 无halo/seam golden |
| G15 | Fail | 无局部dirty node/cell执行证据 |
| G16 | Fail | 无streaming cell managed output lifecycle |
| G17 | Fail | generated object无完整provenance |
| G18 | Fail | 无atomic add/update/remove/reuse diff |
| G19 | Fail | cleanup无法证明保护authored/detached/override/foreign object |
| G20 | Fail | preview/baked/runtime lifecycle未分层 |
| G21 | Fail | Terrain backend仍DiagnosticOnly且World/render断裂 |
| G22 | Fail | 64K仍为固定文本，production mesh GPU Scene仍单实例 |
| G23 | Fail | Collision/Nav/Render/HLOD/Cook无同generation receipt |
| G24 | Fail | 无placement-only Gameplay adapter与authority test |
| G25 | Fail | 无Biome overlap/blend/unknown/no-data golden与debug |
| G26 | Fail | WOC terrain/decoration尚未迁入PCG artifact corpus |
| G27 | Fail | 本轮按用户要求未运行Tooling，且codegen仍是项目依赖证据 |
| G28 | Fail | Scatter无graph document/canvas/transaction/save/reopen |
| G29 | Fail | Generate/Validate仍发布固定queued/64K/18/1 |
| G30 | Fail | 无同request attribute/determinism/diff/profile/cache inspection |
| G31 | Fail | 无64K/1M、partition churn、cancel storm、cache pressure与shutdown资格证据 |
| G32 | Fail | clean compile/test/cook/pack、平台与fault artifact矩阵未通过 |

## 12. 禁止的临时修补

1. 禁止新增几个PCG/Node/Pin/Biome enum、ZUI canvas或manifest capability后宣称功能完成。
2. 禁止继续用`SC_Forest`、64K、18 rules、1 conflict、seed/density字符串或queued feedback冒充执行结果。
3. 禁止把`WorldGeneration` revision、WOC world seed、Terrain height array或Render Graph称为PCG系统。
4. 禁止把WOC `.zr`函数包装成Generate按钮而没有typed source/compiler/artifact/managed output。
5. 禁止让PCG直接写live World、Gameplay population、renderer内部buffer、Physics world或NavMesh owner。
6. 禁止把DiagnosticOnly Terrain、普通mesh单实例GPU Scene、Vampire baked grass或Godot MultiMesh当成PCG产品。
7. 禁止在render/physics/runtime thread同步生成全世界、读文件、展开无界point data或创建64K document entities。
8. 禁止使用global RNG、数组索引identity、process pointer、thread/GPU handle或unstable iteration order进入artifact key。
9. 禁止失败/取消先删除旧generation，或late result覆盖新source revision。
10. 禁止Editor、Terrain、Foliage、Spline、Weather、Spawn分别复制私有PCG executor/cache/partition truth。
11. 禁止保留旧路径shim、双写source/artifact/runtime state或以compat layer掩盖owner迁移。
12. 禁止在G01-G32、同画质benchmark与跨平台故障矩阵通过前声称达到或优于Unreal。

## 13. 本轮完成定义

本轮只完成current-source review：逐项复核Editor40/114、Scatter产品面、Terrain与catalog/App纵链、共享graph/job/transaction/artifact/GPU基础、WOC deterministic corpus、两个静态failure边界和五套本地参考源码；按canonical ID重判5项P0、70项P1、12项P2与32门；给出owner边界、M0-M11依赖序与禁止临时实现清单。

本轮没有修改Runtime、Editor、Interface、Plugin、App、WOC或tests production code，没有关闭implementation milestone，也没有用文件数、test属性、静态UI、known vector或参考引擎规模替代产品证据。实施必须从M0 truth freeze与RED tests开始；在MVP基线允许advanced工作且M0-M4闭合前，保持`implementation_status: pending`。
