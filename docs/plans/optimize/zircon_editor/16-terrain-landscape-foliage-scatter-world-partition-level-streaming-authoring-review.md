---
related_code:
  - zircon_plugins/terrain
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/core/plugin
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/extensions.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/content.rs
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/zircon_plugins/10/failure-2026-08-01-terrain-tilemap-scene-mode-factories-missing.md
  - docs/plans/zircon_runtime/render/15-terrain-vegetation.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/LandscapeEditor
  - dev/UnrealEngine/Engine/Source/Editor/FoliageEdit
  - dev/UnrealEngine/Engine/Source/Editor/WorldPartitionEditor
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape
  - dev/Fyrox/editor/src/interaction/terrain.rs
  - dev/Fyrox/editor/src/scene/commands/terrain.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain
  - dev/godot/scene/resources/3d/height_map_shape_3d.cpp
  - dev/godot/scene/3d/multimesh_instance_3d.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Material/TerrainLit
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/TerrainLit
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Terrain
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 16 · Terrain/Landscape、Foliage/Scatter、World Partition 与 Level Streaming Authoring 工程化差距

## 1. 结论

Zircon并非完全没有Terrain基础。Runtime已有`TerrainAsset`、`TerrainLayerStackAsset`、`.terrain.toml`/`.terrain_layers.toml`导入、artifact/cache/facade/load接点、`SceneTerrainAsset`引用及示例资产；Terrain插件也有package、editor/runtime/dist三层描述符，Editor注册了asset type、toolkit、operation和menu。这些资产管线与插件装配基础应保留。

但当前产品不能称为工程级Terrain/Landscape、Foliage/Scatter或World Partition authoring。最严重的五个断点是：

1. Terrain Editor插件引用的`editor/authoring.zui`、`editor/terrain_component.zui`和`templates/default_heightfield.toml`三份资源均不存在；builtin registry只识别资源类型而不给toolkit，default linked first-party catalog也不提供Terrain editor registration。动态外部package或许能被发现，但当前仓内默认产品合同不闭合。
2. Import/Create/Open/Sculpt等operation只有descriptor，没有factory；authoring batch把它们注册为command metadata。Terrain scene mode仍没有`EditorSceneMode` factory、input effect、transaction adapter或overlay lifecycle，2026-08-01的open failure handoff经当前源码复核仍成立。
3. Runtime插件明确使用`DiagnosticOnlyAssetImporter`并报告“terrain heightfield importer backend is not installed”；它只注册component descriptor和一个heightfield importer descriptor，没有Terrain renderer/service/system/height query，现有Terrain asset在graphics/scene中没有生产consumer。
4. Terrain、Foliage、Scatter、Level Streaming四个Workbench以硬编码的Summit Valley、84K instances、128 clusters、64K instances、96 cells、HLOD和warning填充界面；callback只修改status/output字符串，没有document、job、runtime streaming或render回执，构成能力过度声明。
5. 没有统一的World Authoring Document、可逆地形编辑delta、foliage/scatter规则资产、partition cell/data layer/HLOD schema、streaming source/runtime authority和durable build artifact。Editor无法证明一次笔刷、散布或cell load来自当前source revision，更无法支持大世界增量、失败恢复与跨平台性能验收。

本报告记录5个P0、60个P1、12个P2，给出M0-M8重构路线与32个验收门。目标是建立`WorldAuthoringDocument + TerrainSource/Artifact/RuntimeInstance + Reversible Region Edit + Deterministic Scatter + Partition Manifest + Budgeted Streaming Authority`，不是继续扩充描述符、固定数字或“queued”文案。本轮只做静态review，没有修改production代码。上一轮同一工作树的`zircon_editor --lib`测试编译在617.2秒后被239个既有test-build错误和122个warning阻断，本轮没有重复相同Cargo lane；44个test attributes只作静态inventory，不得表述为动态通过。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Terrain plugin editor/runtime/dist/package | 16 / 856 / 31,660 | E3逐文件：registration、import plan、runtime diagnostic importer、dist ABI及8个test attributes；fingerprint `5d2d9c02...c30a0d11` |
| Editor world-building Workbench、route、binding与feedback | 14 / 4,821 / 243,025 | E3代表action全链、E2完整ID inventory；fingerprint `f2385462...e1d72a8` |
| Runtime Terrain asset/import/artifact/scene/level接点 | 16 / 5,396 / 196,621 | E3资产链，LevelSystem及consumer absence E3；fingerprint `487b6310...0f125ab` |
| Catalog、authoring batch、plugin materialization与operation dispatch | 9 / 2,216 / 83,245 | E3：默认装配、toolkit与MissingFactory路径；fingerprint `0707a4ef...7e99bca9` |
| selected combined scope | 55 / 13,289 / 554,551 | 当前工作树去重fingerprint `fade7a6d...a5307da`；44个test attributes、0 ignored |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256。它只标识本轮证据集合，不是terrain cook、partition manifest、HLOD、scatter或DDC key。

取证时55个文件中有1个在途source：`world_building.rs`仅存在一行import排序变化，本轮没有触碰它。`source_recheck_required=true`表示实施前必须重取该文件、四份ZUI、feedback/catalog/plugin/runtime consumer和动态测试结果；其余工作树并行变化也不得回滚。

44个静态test attributes包含Terrain插件8个注册/最小import plan/dist ABI测试及既有asset/editor SDK测试。它们没有覆盖插件物理资源、默认Editor bootstrap、operation factory dispatch、真实raw/r16/png byte decode、asset create/open/save、sculpt transaction/undo、runtime terrain pixels、foliage/scatter输出、partition/streaming、failure injection或大世界性能。

### 2.2 证据等级与未覆盖边界

- E3：Terrain editor/runtime/dist的每个生产文件和测试逐文件阅读，三份`plugins://terrain/...`资源物理存在性复核。
- E3：authoring batch到command registry和operation dispatch `MissingFactory`分支闭环，scene-mode open handoff按当前源码重新验证。
- E3：四份World Building ZUI到template binding、navigation、reference action和static feedback逐action核对。
- E3：`TerrainAsset`从TOML importer、artifact/cache/facade/load到`SceneTerrainAsset`，再对graphics/scene/plugin consumer作全局搜索。
- E3：Runtime `LevelSystem`逐字段核对，并对partition cell、streaming source、data layer、content bundle及HLOD production authority作全局搜索。
- E3：Unreal LandscapeEditor/FoliageEdit/WorldPartitionEditor与Fyrox terrain interaction/commands作职责对照。
- E2：Unreal Runtime Landscape、Fyrox terrain runtime只抽取Editor所依赖的render/edit/LOD/brush合同；完整算法正确性归Runtime graphics后续实施复核。
- E2：Unity Graphics只作为TerrainLit/ShaderGraph目标与render-path兼容参考；该checkout不包含Unity核心Terrain Tools或World Streaming Editor。
- E2：Godot只提供height-map physics resource与MultiMesh实例消费侧参考；本地源码没有与Unreal等价的通用3D Terrain authoring栈。
- E1：Bevy没有内建Terrain/World Partition Editor，仅可用于asset/task/visibility分层原则，不能作为World Building UX完成度基准。
- 未覆盖：真实GPU/driver像素、R16/PNG corpus、百万实例、数百平方公里world、冷/热streaming、HLOD质量、物理/导航一致性及跨平台内存/IO实测。它们全部进入验收门，不冒充已验证能力。

### 2.3 本轮追踪的生产链

1. Runtime builtin catalog声明Terrain package和`runtime.plugin.terrain`，target为ClientRuntime与EditorHost，说明产品意图是runtime-backed，而非纯Editor mock。
2. Editor builtin plugin descriptor catalog也列出Terrain；但default linked first-party editor catalog只返回Navigation和Neural，不能证明默认发行物会加载Terrain editor crate。
3. builtin asset registry识别Terrain和TerrainLayerStack，却只为UI资产返回builtin toolkit；缺插件时Terrain资产没有可用的详情或只读preview fallback。
4. Terrain editor注册`terrain.authoring` surface、drawer、toolkit、creation template、inspector customization、两个asset importer descriptor和五个operation。
5. `plugins://terrain/editor/authoring.zui`、`plugins://terrain/editor/terrain_component.zui`与`plugins://terrain/templates/default_heightfield.toml`三份目标资源均不存在。
6. `EditorAuthoringContributionBatch`没有operation factory字段，publication调用`register_command`；operation dispatch在无event且找不到factory时返回`MissingFactory`。
7. 五个Terrain operation ID只出现在descriptor和测试中，没有factory、handler、typed payload decoder或document mutation consumer。
8. Terrain插件不再发布descriptor-only scene mode，但也没有补回factory-backed mode；README仍声称Terrain scene mode，当前源码没有对应实现。
9. editor `TerrainHeightfieldImportRequest`只检查非零尺寸、raw/r16/png扩展和可选sample count，返回extension/output kind/expected count计划。
10. 该计划不读取bytes，不处理endianness、bit depth、channel、row order、PNG语义、range、tiling、resample、asset写入或import acknowledgement；LayerStack还复用heightfield sample-count请求。
11. runtime插件注册component descriptor和一个heightfield importer descriptor，但执行者是`DiagnosticOnlyAssetImporter`，并显式报告backend未安装。
12. runtime插件没有TerrainLayerStack importer descriptor、manager/service/system/render feature/height query；dist entry stateless、无command、无bridge method。
13. core builtin TOML importer可以解析`TerrainAsset`与`TerrainLayerStackAsset`；前者只在samples非空时检查`width * height`，后者完全不验证。
14. `TerrainAsset`允许零宽高、空samples、非有限或非正spacing/scale、非有限height/layer strength、重名layer和不匹配weightmap语义。
15. `SceneTerrainAsset`只持一个asset reference，无法表达section/component、transform policy、collision、streaming、material override、runtime generation或edit layer。
16. 示例`jungle_clearing.terrain.toml`是9x9 inline float samples与一层material，证明serialization路径存在，但不是可扩展heightfield artifact或大型world证据。
17. `TerrainAsset`/`TerrainLayerStackAsset`在runtime graphics与scene生产consumer搜索为零；加载成功不等于渲染、碰撞、导航或查询成功。
18. `LevelSystem`包装一个ECS `World`、Loaded/Unloaded lifecycle、metadata和subsystem名称；没有cell manifest、streaming source、data layer、HLOD、resident set、IO ticket或budget authority。
19. Foliage/Scatter没有生产asset schema、plugin、compiler、generator、instance artifact或runtime system；四份World Building ZUI中的相应内容全为静态样例数据。
20. reference action对workbench preview action为空分支；extension action最多切换control/dropdown并应用固定feedback，field edit/commit没有进入typed world transaction。
21. Terrain feedback显示“64 cells / 2 warnings”，Foliage显示“84K instances / 128 clusters”，Scatter显示“64K instances / 18 rules”，Level Streaming显示“96 cells / async / 96 MB”，均没有job ticket或runtime snapshot来源。
22. 旧Runtime Terrain/Vegetation计划已定义quadtree LOD、splat、edit op、foliage和imposter方向，其output record仍标记TV-M1至TV-M4未开始；当前源码的diagnostic-only backend与consumer absence再次印证其未落地。

## 3. 已有工程基础，重构时必须保留

### 3.1 Asset与插件基础

- `TerrainAsset`、`TerrainLayerAsset`、`TerrainLayerStackAsset`、asset reference收集、TOML serialization、artifact payload/store/facade/load链可作为source envelope的迁移起点。
- `SceneTerrainAsset`已经给Scene DTO预留Terrain引用，应演进为versioned instance binding，而不是再造第二个无关Scene字段。
- Terrain package已有editor/runtime/dist边界、capability status、native ABI registration和default packaging，可承接真正的backend与产品装配。
- builtin asset type、toolkit、view/drawer、creation template、inspector customization、operation/menu注册点已经存在；重构应补强factory/resource/readiness硬门。

### 3.2 Editor共享基础

- Editor02的document/transaction/save/recovery方向、Editor03的factory-backed scene mode/effect/overlay合同、Editor04的asset import/reimport与reference graph、Editor09的job authority应被Terrain直接复用。
- operation dispatch会显式产生`MissingFactory`，可升级为bootstrap invariant与capability truth，而不是用no-op factory绕过。
- Workbench已有World Building导航位置和控件词汇；在真实toolkit完成前可作为disabled prototype，但不能继续投影成功、实例数或cell状态。

### 3.3 Runtime与参考实现基础

- Runtime现有asset artifact、render graph、GPU Scene、visibility、material/pipeline、streaming/residency和scene lifecycle是Terrain/foliage必须接入的共享底座，禁止建立平行renderer或私有IO调度器。
- Fyrox已经证明Rust引擎中可实现brush thread、smear/stamp、chunk undo capture及height/hole/layer mask execute/revert；Zircon不能以语言或架构为由停留在descriptor。
- Unreal把Landscape editor mode/tools/import、Foliage edit subsystem及World Partition grid/data layer/HLOD拆成明确owner，说明大世界authoring不能被压成一个静态workspace或一个巨型Terrain组件。

## 4. 目标架构

### 4.1 Source、derived artifact与runtime instance分层

| 层 | 应持有内容 | 不得持有内容 |
|---|---|---|
| `WorldAuthoringDocument` | world/asset ID、source revision、stable element ID、selection、dirty/history、layer/partition/scatter引用 | GPU handle、UI control ID、固定status文本 |
| `TerrainSourceAsset` | versioned height source、component topology、edit layers、weight/hole layers、material references、import provenance | live residency、draw packet、temporary brush state |
| `TerrainEditDelta` | target layer、bounded region、before/after或invertible patch、base revision、merge key、affected dependencies | 直接world/GPU副作用 |
| `TerrainCookArtifact` | immutable tiles/pages、min/max/error/normal/weight/hole数据、dependency/cook key、debug map | mutable Editor draft |
| `TerrainRuntimeInstance` | world transform、resident sections、LOD/visibility/collision/nav handles、artifact generation | authoring history |
| `FoliageScatterSource` | stable rule ID、seed、density/mask/slope/height/collision filters、prototype references、exclusion volumes | 随机临时结果 |
| `FoliageInstanceArtifact` | deterministic cell outputs、prototype ranges、cluster bounds、LOD/imposter references、source key | Editor selection或undo stack |
| `WorldPartitionManifest` | stable cell ID、bounds、actor/content refs、data layer、dependencies、HLOD、priority/cost estimates | live loaded state |
| `WorldStreamingAuthority` | streaming sources、desired/resident/in-flight sets、IO/decompress/upload tickets、budgets、generation与diagnostics | Editor固定96-cell样例 |

权威流水线必须是：

```text
bounded import bytes / versioned authoring source
  -> format dispatch + migration + structural validation
  -> transactional document mutation + affected-region/dependency set
  -> deterministic terrain/scatter/partition compiler
  -> immutable tile/cell/HLOD artifact keyed by source + dependencies + target
  -> budgeted runtime IO/decompress/upload/admission
  -> generation-bound terrain/foliage/streaming snapshot
  -> Editor viewport, diagnostics, statistics and save/build acknowledgement
```

任何阶段失败都必须保留last-good artifact但标明source revision与artifact generation不一致；不得用旧画面、固定实例数或“queued”字符串表示当前source成功。

### 4.2 Terrain数据与编辑合同

Height source必须定义sample format、endianness、row order、origin、world scale/range、no-data、component/tile尺寸、shared edge、compression与checksum。Weight/hole数据必须有分辨率、channel/packing、normalization、layer identity及迁移规则。导入器先做bounded header/decode，再输出canonical source和stable diagnostics；不能依赖调用者提供的sample count冒充解码。

一次笔刷stroke应经历pointer capture、ray/terrain projection、spacing/smear、bounded affected region、preview overlay、CPU authoritative delta、transaction prepare/commit/cancel、derived dirty marking、async GPU upload与runtime acknowledgement。Height、weight、hole、smooth、flatten、erosion、ramp和spline可以分阶段交付，但所有工具必须共享这一合同。

### 4.3 Foliage/Scatter确定性与增量性

Scatter rule以stable ID、seed、source revision、cell ID和prototype ID构造确定性随机流；filter顺序、采样空间、collision query版本、density normalization和tie-break必须固定。构建输出按cell增量缓存，单规则或单mask修改只能失效受影响cell。手摆实例与生成实例分层，支持exclusion/override，不能在重建时覆盖用户编辑。

### 4.4 Partition、Streaming与HLOD

World partition应从authoring world生成versioned manifest：stable cell、bounds、content ownership、dependency、data layer、always-loaded、runtime grid、cost估计和HLOD引用。Runtime authority按多个streaming source计算desired set，经IO、decompress、asset dependency、GPU upload和world attach状态机进入resident；取消、优先级反转、预算超限、坏cell和shutdown都必须有终态。

Editor预览必须调用同一streaming policy但使用隔离session和显式camera/source；grid selection/load/unload/pin只改变typed state。HLOD build是可取消background job，产物有source key、builder version、target profile、quality metrics和visual compare证据。

## 5. P0：产品不可达、能力过度声明或source到runtime合同断裂的问题

### P0-1 · 默认产品入口与Terrain插件资源包不闭合

三份插件资源物理缺失，builtin Terrain类型无toolkit，default linked editor catalog又不供应Terrain registration。默认产品不能可靠create/open/inspect Terrain；动态package可能性不能替代发行物装配证据。实施时必须把resource resolution、factory readiness和target capability变成publication前原子硬门。

### P0-2 · Terrain operation和scene mode只有声明，没有执行所有权

Import/Create/Open/Sculpt operation没有factory，scene-mode failure handoff仍open。任何重新显示Sculpt工具前，插件必须提供真实factory、typed effect、pointer capture、transaction、overlay、cancel/disable teardown与行为测试；禁止no-op mode、direct world mutation或test-only factory。

### P0-3 · Runtime backend明确为诊断占位，资产没有执行consumer

runtime importer显式报告backend未安装，Terrain component也只是schema；没有renderer、height query、collision/nav consumer或Terrain asset到Scene/Graphics实例化链。Editor在此状态下无法产生runtime一致preview、Play/Cook结果或性能证明。

### P0-4 · World Building Workbench伪造构建、实例和streaming结果

Terrain/Foliage/Scatter/Level Streaming界面把固定数字和成功语义投影为native workspace、preview/build/load queued，而action没有job/runtime调用。产品必须立即改成Prototype/Unavailable与typed原因，或隐藏命令；只有真实ticket与generation-bound result才能显示queued/completed/statistics。

### P0-5 · 缺少统一transactional world document与partition/streaming authority

当前没有可逆region edit、durable source save、deterministic scatter、partition manifest、streaming state machine或HLOD artifact。继续分别补按钮会加深authority分裂。先建立WorldAuthoringDocument和Runtime Streaming Authority，再交付Terrain、Foliage、Scatter与Level Streaming工具。

## 6. P1：工程级完整性差距

### 6.1 Package、资源、capability与产品装配

1. Terrain package manifest、runtime builtin catalog、Editor builtin descriptor与default linked editor catalog四处声明没有单一装配truth或发行profile矩阵。
2. `runtime.plugin.terrain`标记partial，但Editor导航、menu和toolkit不按backend/resource/factory readiness解释具体缺口。
3. 三份`plugins://terrain/...`URI未在registration publication前解析、读取和schema验证，坏package可以先污染registry。
4. builtin asset registry识别Terrain/TerrainLayerStack却无builtin只读details/diagnostic fallback，插件缺失时只能落入“No toolkit”。
5. Terrain editor dist/runtime dist没有authoring command bridge；native行为表stateless且`invoke_command=None`，外部package无法靠ABI补齐现有operation。
6. authoring contribution batch不承载operation factory、document provider、compiler/build provider或preview provider，元数据与执行owner分离。
7. operation、menu、toolkit、view、drawer、importer和inspector customization没有共享owner lease与原子rollback，partial publication可留下残余入口。
8. plugin disable/unload/reload没有定义open Terrain document、active stroke、dirty source、resident preview和unknown layer/rule的保持或终止策略。
9. README声称scene mode与runtime-backed authoring，manifest beta/partial和真实MissingFactory/diagnostic-only状态没有自动一致性检查。
10. tests只验证descriptor/menu/schema/ABI存在，不启动默认产品、解析资源、执行operation或验证capability disable后的入口状态。

### 6.2 Terrain source、import、layer与artifact schema

11. Editor raw/r16/png import plan不读取源bytes，无法验证文件长度、header、endianness、bit depth、channel或row order。
12. PNG没有颜色类型、bit depth、gamma/profile、alpha、interlace、decode limit和height range政策，8-bit彩图也可能被当heightfield。
13. raw/r16没有尺寸推断、显式stride、byte order、signedness、scale/offset、no-data和trailing-byte诊断。
14. `width * height`使用无checked multiplication的usize转换，极端尺寸缺overflow与allocation budget防护。
15. LayerStack复用heightfield request/sample count，不能表达多个weightmap、channel packing、layer identity、normalization或resolution mismatch。
16. runtime插件只声明Terrain heightfield importer，没有TerrainLayerStack runtime importer，Editor两个import入口与runtime package manifest不对称。
17. core TOML Terrain validator允许零维、空samples、非有限或非正spacing/scale、非有限height和尺寸乘法overflow。
18. Terrain layer没有stable ID、schema version、blend mode、physical material、UV/tiling、visibility/lock、edit layer或target channel。
19. layer name/strength/material/weightmap不验证空值、重复、finite/range、reference kind、分辨率、channel或dependency cycle。
20. `TerrainLayerStackAsset`没有validator、version/migration和canonical ordering，任意坏层可进入artifact cache。
21. inline `Vec<Real>` height samples不适合作为大型地形source/artifact，缺tile/page、compression、checksum、min/max/error/normal和partial read合同。
22. Terrain source、derived tile、runtime resident section没有不同类型与generation，asset load成功容易被误判为可运行Terrain。

### 6.3 Terrain document、scene mode、笔刷与保存

23. 没有Terrain-specific document session、source revision、dirty set、history context、autosave payload、close decision或external edit/CAS。
24. Create Heightfield没有真实template、尺寸/component topology/scale/range验证、目标路径占用检查、atomic write和catalog acknowledgement。
25. Open operation没有从Asset Browser selection到toolkit/document/runtime preview的factory链，也没有坏资产的read-only repair模式。
26. Sculpt operation没有scene-mode factory、terrain picking、pointer capture、pressure/tilt、brush spacing/smear、falloff/alpha或overlay。
27. 没有Height/Weight/Hole工具target authority；UI tab、selected layer和runtime edit target可能分裂。
28. stroke没有bounded region delta、before image、inverse、merge key、base revision和cancel rollback，无法可靠undo/redo。
29. CPU source、GPU texture、collision heightfield、navigation dirty tile、LOD error/normal和foliage dependency没有统一affected-region传播。
30. smooth/flatten/ramp/erosion/spline/import-copy-paste/resize等工程工具缺schema、transaction与deterministic算法合同。
31. edit layer/非破坏层、visibility/lock/order/blend、layer merge/bake和跨revision migration完全缺失。
32. multi-selection、multi-terrain seam、shared border samples、world transform和origin rebasing下的笔刷语义未定义。
33. save没有validated snapshot、temp/flush/atomic replace、import acknowledgement、last-known-good、autosave/recovery或partial artifact cleanup。
34. field edit/commit和Workbench feedback不进入typed command/history，固定control字符串也不能作为stable document element identity。

### 6.4 Runtime Terrain、render、physics、navigation与性能

35. `SceneTerrainAsset`只含引用，没有instance transform policy、collision/nav flags、material override、streaming group或runtime generation。
36. scene loading没有把`SceneTerrainAsset`实例化为Terrain runtime entity/component/service；descriptor注册本身不产生行为。
37. graphics没有Terrain extract、renderer、render feature、shared patch geometry、高度采样或material/splat pipeline consumer。
38. 没有component/tile topology、quadtree/clipmap/virtualized geometry LOD、screen error、neighbor constraint、morph/skirt或裂缝验收。
39. Terrain visibility未接view/frustum/occlusion/GPU Scene，section bounds和height min/max也没有增量更新。
40. weight/splat、hole、normal、macro/base map、virtual texture/residency和不同render path/shadow/depth/ray tracing合同缺失。
41. 没有CPU/GPU height query、raycast、collision shape cook与physics generation一致性，Gameplay无法可靠查询地面。
42. navigation没有按dirty terrain tile重建、generation/cancel/commit，也没有hole、slope、layer cost和streaming nav data合同。
43. Terrain upload/build未进入共享asset streaming、resource budget、job cancellation、device loss/recovery和shutdown fence。
44. 没有以Unreal/Fyrox及固定场景作同画质对照的frame time、GPU time、VRAM、IO、LOD pop、edit latency与stutter基线，无法支持“性能优于Unreal”的声明。

### 6.5 Foliage、Scatter、实例与植被authoring

45. 仓内没有Foliage/Scatter source asset、versioned schema、AssetKind、importer、plugin toolkit或runtime service。
46. Foliage Workbench的FOL_Forest/Oak/Fern/Biome、84K instances和128 clusters全是静态文本，不来自asset或runtime snapshot。
47. Scatter Workbench的SC_Forest、18 rules、64K instances和collision conflict同样没有rule document、generator或validator。
48. 没有stable rule/prototype ID、deterministic seed stream、density/mask/slope/height/alignment/scale/rotation/collision/exclusion schema。
49. 没有按partition cell的增量scatter artifact、dependency key、changed-cell invalidation、cache或失败回滚。
50. 手摆实例、生成实例、override/exclusion和重新生成之间没有ownership，重建可能无法保留用户意图。
51. runtime没有cluster bounds、GPU instancing/indirect draw、culling、LOD/imposter、wind、shadow、density scalability和streaming budget接入。
52. 没有百万实例确定性、分布统计、重叠/坡度约束、edit latency、camera traversal stutter和跨平台memory/GPU验收。

### 6.6 World Partition、Level Streaming、Data Layer与HLOD

53. Runtime与插件生产代码没有World Partition asset/manifest、stable cell ID、grid policy、bounds/content ownership或version migration。
54. `LevelSystem`只有整World Loaded/Unloaded，不能表示desired/loading/decompressing/attaching/resident/unloading/failed cell状态。
55. 没有streaming source、distance/shape/priority policy、多个source合并、always-loaded、pin、data layer或runtime condition。
56. 没有cell dependency、cross-cell actor/reference、external actor/content bundle、ownership change和atomic move transaction。
57. 没有IO/decompression/deserialization/GPU upload/world attach的ticket、budget、cancellation acknowledgement、backpressure和shutdown终态。
58. Level Streaming Workbench显示96 cells、Cell_A12/A13/B12、HLOD_04和96 MB，但没有manifest query、resident set或runtime load request。
59. 没有HLOD layer/builder/artifact/source key、incremental rebuild、quality metric、visual compare、runtime transition或last-good策略。
60. 没有World Partition convert/validate/minimap/data layer/content bundle/HLOD build commandlet及大世界cook/package/patch测试。

## 7. P2：不阻断正确性但影响成熟度的差距

1. 缺少per-user terrain brush preset、falloff/alpha library、pressure curve、gizmo color与overlay density偏好。
2. 缺少可命名的Terrain viewport camera、world streaming source、LOD/weight/cell debug bookmark。
3. 缺少最近Terrain/layer/rule/cell与跨document selection/navigation history。
4. 缺少height/weight/hole/edit-layer revision diff、before/after heatmap与side-by-side viewport compare。
5. 缺少笔刷stroke、scatter rule、cell/HLOD的comment、tag、review note与审批状态。
6. 缺少Terrain layer、foliage prototype、scatter rule、data layer和HLOD layer的批量编辑与usage query。
7. 缺少将source manifest、import diagnostics、dirty regions、artifact keys、streaming journal和GPU capture引用导出为bounded support bundle。
8. 缺少opt-in terrain/streaming telemetry摘要，包括edit latency、cache hit、resident cells、IO/GPU upload和LOD transition趋势。
9. 缺少commandlet dry-run、批量reimport、partition validate、scatter rebuild和HLOD stale报告。
10. 缺少Editor scripting/remote automation的typed Terrain/Scatter/Partition command与query surface。
11. 缺少多人协作下的terrain region/cell lock、冲突可视化、ownership transfer和revision annotation。
12. 缺少高对比、色盲安全weight palette、键盘笔刷调整、screen-reader cell table和完整i18n/unit formatting。

## 8. 参考引擎对照

| 参考 | 当前源码证据 | Zircon应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal LandscapeEditor | 88个C++/H文件、40,360行；EdMode、brush/tool、paint/component/erosion/mirror/ramp/spline、raw/png import/export、target/edit layers及automation tests | mode/tool生命周期、target layer、transaction、import诊断、non-destructive edit layer与产品测试分层 | UObject/Slate宏体系和完整历史兼容负担 |
| Unreal FoliageEdit | 48个C++/H文件、12,100行；EdMode/toolkit/subsystem/palette/type details/procedural foliage | 手摆/生成实例ownership、prototype palette、subsystem、paint/reapply/select/erase与大规模实例workflow | 与Unreal Actor/UObject序列化强耦合部分 |
| Unreal WorldPartitionEditor | 95个C++/H文件、17,027行；grid、convert、minimap、data layer、content bundle、HLOD subsystem/compare/build UI | partition manifest、cell owner、data layer/content bundle、HLOD artifact、commandlet和可视化诊断 | 直接复制Unreal World Partition数据格式 |
| Unreal Runtime Landscape | render/culling/edit/readback/grass map/HLOD独立模块 | source-artifact-runtime分层、section LOD/culling、dirty update、grass map与HLOD接点 | 在Editor中直接操作renderer内部资源 |
| Fyrox | `interaction/terrain.rs` 698行、commands 252行；真实brush thread、stamp/smear、UndoData和height/hole/layer execute/revert；runtime含chunk/quadtree/brushstroke | Rust可落地的交互、chunk dirty、可逆命令、quadtree update和线程边界 | 直接共享可变Terrain对象或忽略Zircon job/generation合同 |
| Godot | HeightMapShape3D与MultiMeshInstance3D提供物理height array及实例集挂接 | 物理查询/collision消费与实例资源边界 | 把两个局部类误称为完整Terrain/World Partition Editor基准 |
| Unity Graphics | HDRP/URP TerrainLit GUI、ShaderGraph subtarget、4/8 layer模板、basemap/splat/ray/path tracing接点 | Terrain material target、pass/reflection、layer档位与多render-path验证 | 该checkout不含核心Terrain Tools/streaming authoring，不能据此设计完整UX |
| Bevy | 通用asset/task/visibility/render resource基础，无内建Terrain Editor | 数据导向artifact、并行任务和runtime visibility原则 | 把缺少Editor能力解释为Zircon可以停留在runtime-only |

对照原则是吸收职责边界、失败语义、性能证据和可测试合同。Zircon目标可以在artifact格式、job system和GPU架构上不同，但不能省略同等级产品必须承担的import、transaction、render、streaming、diagnostic和recovery责任。

## 9. 重构里程碑

### M0 · Capability Truth、资源与factory硬门

- 补齐或撤销三份Terrain resource URI；默认catalog、dynamic package、builtin fallback形成明确发行矩阵。
- publication前原子验证resource、operation factory、scene-mode factory、runtime backend与target capability。
- 四份World Building prototype停止显示伪成功、固定实例/cell/warning；未实现项投影typed unavailable reason。

### M1 · Canonical Terrain Source、Importer 与 Artifact

- 定义versioned Terrain source、height/weight/hole/edit-layer schema、migration、bounded validator和import provenance。
- 实现raw/r16/png真实byte decode、endianness/channel/range/size policy、atomic asset creation与import acknowledgement。
- 编译immutable tiled artifact，包含checksum、min/max/error/normal、dependency/cook key与partial read。

### M2 · Transactional Terrain Document 与 Scene Mode

- 建立World/Terrain document session、stable element ID、dirty/history/autosave/close/recovery。
- 提供factory-backed Terrain mode、picking/pointer capture/brush overlay、region delta、commit/cancel/undo/redo。
- Height/Weight/Hole先形成纵向闭环，再增加smooth/flatten/ramp/spline/erosion与edit layers。

### M3 · Runtime Terrain Instance、Renderer 与 Query

- 从Scene引用实例化generation-bound Terrain runtime instance，接asset residency、GPU Scene、visibility和render graph。
- 交付section topology、LOD、seam、splat/hole/normal、shadow/depth及material pipeline。
- 交付height/raycast/collision/nav dirty接口、device loss/recovery和runtime diagnostics。

### M4 · Incremental Build、Jobs 与 Durable Preview

- Terrain cook/upload/nav/collision/preview进入共享job admission、dedup、cancel acknowledgement、budget和shutdown fence。
- affected region只失效必要tile/section/dependency；Editor显示source/artifact/runtime generation和last-good状态。
- 保存、build和preview采用durable acknowledgement与故障注入门。

### M5 · Foliage/Scatter Source、Compiler 与 Runtime

- 定义prototype/rule/exclusion/override schema与deterministic random合同。
- 生成按cell缓存的instance artifact，支持changed-cell invalidation、手摆/生成ownership和冲突诊断。
- 接GPU instancing/culling/LOD/imposter/wind/shadow/scalability与streaming budget。

### M6 · World Partition Manifest 与 Runtime Streaming

- 建立stable cell、content ownership、dependency、data layer、always-loaded、streaming source和cost schema。
- 实现desired/in-flight/resident状态机、IO/decompress/upload/attach pipeline、budget/backpressure/cancel/recovery。
- Level Streaming workspace改为真实manifest/runtime projection与typed load/unload/pin command。

### M7 · HLOD、Minimap、Commandlet 与 Cook

- 建立HLOD layer/builder/artifact/source key、incremental stale detection、visual compare和runtime transition。
- 交付partition convert/validate、minimap、data layer/content bundle、scatter/HLOD build commandlets。
- 验证cook/package/patch对cell、Terrain tile、foliage artifact和HLOD dependency的完整性。

### M8 · 性能、故障、跨平台与发布门

- 建立与Unreal/Fyrox同内容同画质场景的CPU/GPU/VRAM/IO/edit/streaming基线，未测不得宣称领先。
- 覆盖corrupt/huge import、Nth-step write/build/load failure、plugin unload、device loss、cancel race与shutdown。
- Windows主lane通过后按真实需求补Linux/目标平台，发布profile必须列出backend、quality、budget与已知限制。

## 10. 验收门

1. 默认Editor发行物可以create/open/inspect一份Terrain；缺插件或backend时入口disabled并给typed原因，不出现“No toolkit”或点击后`MissingFactory`。
2. Terrain plugin全部resource URI在publication前解析并通过schema验证；任一缺失时整批注册回滚且无残余menu/toolkit/view。
3. default linked、dynamic package与builtin fallback矩阵有自动化bootstrap测试，明确每个profile的Terrain editor/runtime状态。
4. Import/Create/Open/Sculpt每个可见operation都有生产factory、owner lease、typed payload和成功/失败/disabled行为测试。
5. Terrain scene mode覆盖activate、input、pointer capture、overlay、commit、cancel、undo/redo、capability disable和document close teardown。
6. raw/r16/png corpus覆盖大小、endianness、stride、bit depth、channel、row order、range、truncation、trailing bytes、decode limit与malformed输入。
7. 极端width/height、`width * height` overflow和allocation bomb在分配前被bounded diagnostic拒绝。
8. Terrain/LayerStack schema验证finite/range、stable layer ID、重复、reference kind、weight resolution/channel和migration。
9. Create/Import采用temp、flush、atomic replace、catalog/import acknowledgement；Nth-step failure不留下半资产或假成功。
10. 一次Height/Weight/Hole stroke形成一个可逆transaction；commit、undo、redo后source hash、采样值和dirty region精确往返。
11. active stroke cancel、模式切换、插件disable、project close和crash recovery不会留下半写source、悬挂pointer capture或后台写入。
12. 共享边界跨component编辑无裂缝，CPU source、GPU tile、collision、nav、LOD error和normal的generation最终一致。
13. Terrain asset从Scene load到runtime instance、extract、render pass和可见像素有端到端产品测试，不以descriptor注册替代。
14. LOD相邻级差、morph/skirt/seam在固定camera path与reference image中无可见裂缝，结果跨run稳定。
15. splat/weight/hole/normal、shadow/depth及至少两种render path通过GPU像素和capture资源绑定验证。
16. height query、raycast、collision和render surface在固定采样点与编辑前后误差阈值内一致。
17. navigation只重建dirty tile，cancel/late result不能覆盖新generation；hole和slope语义有路径查询测试。
18. Terrain cook/upload/preview进入共享job authority，具备dedup、progress、cancel acknowledgement、resource budget和bounded shutdown。
19. source编译失败时保留last-good artifact，但Editor明确显示source revision、artifact generation、错误位置和preview状态。
20. Foliage/Scatter相同source/rule/cell/seed跨run、线程数和增量重建产生bit-stable或定义容差内稳定结果。
21. 修改一个scatter rule/mask只失效相交cell；未受影响artifact key、instance hash和resident state保持不变。
22. 手摆实例、generated实例、override和exclusion在重建/undo/redo/save/reopen后ownership与结果一致。
23. 百万实例场景通过实例数、draw/dispatch、CPU/GPU时间、VRAM和camera traversal stutter预算；无固定文本冒充统计。
24. World Partition manifest在相同world/source版本下稳定，cell ID、bounds、content owner、dependency、data layer和cost可验证。
25. 多streaming source、priority inversion、pin/always-loaded和data layer切换产生确定desired set与可解释决策日志。
26. cell状态覆盖desired、queued、IO、decompress、attach、resident、unload、failed与cancelled；每个ticket恰有一个终态。
27. IO/decompress/upload/attach任意第N步失败不会发布半cell，retry或rollback后resident world与manifest generation一致。
28. Level Streaming workspace的cell、memory、HLOD和warning全部来自manifest/runtime snapshot；不存在硬编码96 cells/96 MB成功语义。
29. HLOD build key包含source/dependency/builder/target/profile，changed cell只重建必要HLOD，失败保留last-good并标stale。
30. partition convert/validate/minimap/scatter/HLOD commandlet支持headless、dry-run、structured report、cancel和非零失败码。
31. 同内容同画质benchmark记录Zircon与指定Unreal/Fyrox版本、硬件、驱动、设置、warmup、采样和raw artifacts；只有统计显著时才允许“领先”声明。
32. Windows required lanes、真实GPU product lane、failure injection、large-world soak、plugin lifecycle和至少一个目标平台lane全部GREEN后，才可把Terrain/Foliage/World Partition capability从partial提升为complete。

## 11. 非目标与边界

- 本报告不要求第一阶段复制Unreal全部Landscape、Foliage或World Partition功能；要求先建立不会被后续规模和失败语义推翻的authority与纵向闭环。
- Runtime Terrain renderer、visibility、asset residency和material细节仍受Runtime09系列约束；本报告只定义Editor必须消费的合同和产品门。
- 物理与导航算法本身分别归Runtime08A/08D；Terrain必须提供generation-bound height/collision/nav dirty接点，不得各自私读source文件。
- Unity Graphics与Godot本地checkout不是完整World Building Editor参考；缺失部分由Unreal/Fyrox和Zircon共享架构原则补足。
- 本轮不修改production源码、不关闭已有failure handoff、不声称Cargo或GPU测试通过，也不基于静态代码宣称性能优于Unreal。

## 12. 完成定义

Editor16只有在M0-M8全部完成、32个验收门有可重放证据、Terrain/Foliage/Scatter/Partition各自拥有source到runtime纵向产品闭环、四份Workbench不再投影任何无来源成功语义，并且dynamic/static package、save/recovery、job/shutdown、large-world和真实GPU lane均通过后，才可从`implementation_status: pending`改为完成。

在此之前，最准确的产品描述是：Zircon拥有Terrain资产与插件描述符基础，但Terrain backend、工程级authoring、Foliage/Scatter和World Partition/Level Streaming均未完成；当前World Building Workbench是静态prototype，不是可用于生产的大世界编辑器。
