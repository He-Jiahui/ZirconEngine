---
related_code:
  - zircon_plugins/terrain
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/importer/ingest
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/graphics/feature/builtin_render_feature
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor
  - zircon_runtime/src/core/framework/physics
  - zircon_plugins/physics/runtime/src/backend
  - zircon_plugins/physics/runtime/src/manager
  - zircon_plugins/navigation/runtime/src/manager/bake
tests:
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_runtime/src/asset/tests/assets/authoring.rs
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
  - zircon_runtime/src/asset/tests/project/example_vampire/manifest_scene_imports.rs
  - examples/vampire
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape
  - dev/UnrealEngine/Engine/Source/Runtime/Foliage
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldPartition
  - dev/UnrealEngine/Engine/Plugins/Experimental/VirtualHeightfieldMesh/Source/VirtualHeightfieldMesh
  - dev/Fyrox/fyrox-impl/src/scene/terrain
  - dev/bevy/crates/bevy_render/src
  - dev/bevy/crates/bevy_pbr/src
  - dev/godot/scene/resources/3d/height_map_shape_3d.cpp
  - dev/godot/scene/resources/multimesh.cpp
  - dev/godot/modules/jolt_physics/shapes/jolt_height_map_shape_3d.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/TerrainLit
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/UnifiedRayTracing/Common/TerrainToMesh.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 29 · Terrain/Landscape、Heightfield、Quadtree LOD、Material Layer、Foliage、World Partition、Physics/Navigation、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前没有可执行的地形运行时子系统。仓库已经能解析 `TerrainAsset`，场景文档也能保存一个 `SceneTerrainAsset` 引用，terrain 插件还能注册 component/importer 描述符，图形层则枚举了 `BuiltinRenderFeature::Terrain`。但这四层互不相连：`TerrainAsset` 是内联 `Vec<Real>`；`SceneNode`/`NodeRecord` 没有 terrain component；`World::from_scene_asset` 不消费 `entity.terrain`，`World::to_scene_asset` 又固定写 `terrain: None`；Terrain render feature 明确属于 descriptor-only advanced slot，只产生 extract section 名称，phase 与 pass 都为空。

Vampire 样例没有反证这一结论。其 `Baked Jungle Terrain` 同时绑定普通 `jungle_terrain.model.toml` mesh 与 terrain 引用，产品测试验证的是 mesh 可见、TOML 可解析和高度范围足够大。渲染提取只接收 `meshes`，所以可见地面来自预烘焙普通网格，不是 `TerrainAsset`、terrain feature、运行时 LOD 或地形材质层执行。样例保存为 runtime world 后，terrain 引用还会被 `terrain: None` 丢弃。

物理与导航也没有闭环。Physics 公共合同可表达独立的 `HeightField` asset/collider，但它不从 `TerrainAsset` 派生；Jolt provider 把每个高度格展开为两枚三角形并创建 triangle set，没有使用 Jolt 原生 height-field shape，builtin provider 对 height field 的 validity、AABB、ray cast/contact 均 fail-close。Navigation bake 对 render mesh 只推入按 transform 构造的固定 quad，对 asset-backed triangle mesh/height field 注释为“由 owning asset bake path 收集”，但本轮没有找到该 owner 的实现；因此 terrain、physics 与 nav 的代际一致性不存在。

本篇登记 **0 P0 / 62 P1 / 14 P2**。0 P0 不是完成度认可：Editor16 已登记 Terrain backend 无 consumer、产品 Workbench 伪结果、缺少 partition/streaming authority 等 5 个 P0，本篇不重复计数。Runtime29 负责把这些产品阻断背后的运行时机制收敛为 `TerrainSourceAsset -> TerrainBuildArtifact -> TerrainRuntimeInstance -> Render/Physics/Nav/Foliage/Partition adapters -> typed receipt`。任何产品若把 descriptor-only feature、普通 mesh fallback、可解析 TOML、静态 batch 数量或 capability 名称展示为 Terrain runtime Ready/Executed，应直接沿 Editor16 P0-3/P0-4 升级处理。

## 2. 审查边界、方法与 currentness

### 2.1 冻结输入

本篇冻结 113 个输入、69,648 行、2,712,915 bytes：43 个 Zircon production 输入为 9,529 行、351,833 bytes；10 个 Zircon test/product 输入为 4,938 行、170,484 bytes；60 个参考实现输入为 55,181 行、2,190,598 bytes。组合指纹按相对路径排序，对每个文件计算 SHA-256，再对 `path<TAB>hash` 的 LF 拼接文本计算 SHA-256，结果为 `83ddf41b660dea64ccef6780dc089d7eb2c615478b4be403a8f3d79f599c5f21`。

冻结基线为 `main@25e09a23178000f2e783ce2143cf70a8b118d404`。选定输入中 `frame_extract.rs`、`frame_extract/geometry.rs` 与 `scene_extract.rs` 被 Git 工作树标成 modified，但三者 working blob 分别与 HEAD blob 完全相同，`git diff`/`numstat` 均为空；本篇按当前 bytes 冻结，不把换行或索引状态解释为语义变化。其余选定 Zircon 输入无工作区修改，`dev/` 参考源码不纳入父仓库 dirty 判定。

### 2.2 纵向生产链

本轮逐层核对：source/import schema -> artifact/dependency -> scene persistence -> runtime component/identity/lifecycle -> height/normal/weight/hole GPU resource -> patch topology -> quadtree/clipmap LOD -> culling/indirect draw -> terrain material/depth/shadow/motion -> physics height field/query -> navigation bake/dirty tile -> foliage/scatter/instance culling -> world partition/HLOD/residency -> quality profile/diagnostics -> Vampire 产品证据。生产搜索同时覆盖 terrain、landscape、heightfield、foliage、grass、scatter、world partition、runtime cell、streaming source 与 HLOD 同义入口，未发现 facade 后隐藏的第二套执行实现。

### 2.3 证据等级与限制

本轮达到 E3 source-level review。没有运行 Cargo 或 GPU 产品测试：现有 terrain feature 为零 pass、scene world 不保留 terrain component，运行普通 mesh 路径不能提高 Terrain runtime 结论强度；同一工作树已有 `zircon_editor --lib` 被 239 个既有 test-build errors 阻断，本轮不重复该动态 lane。Editor sculpt/import UI、undo/redo 与 World Building Workbench 由 Editor16 拥有；本篇只定义其必须调用的 runtime artifact、preview、streaming 与 receipt 合同。

## 3. 当前可保留的工程基础

1. `TerrainAsset`、`TerrainLayerAsset`、`TerrainLayerStackAsset` 和 `SceneTerrainAsset` 已建立最小序列化名称，可作为硬迁移输入，而不是继续并行新增另一套 Landscape TOML。
2. builtin TOML importer 会校验非空高度数组的 sample count，asset manager 也有 typed `load_terrain_asset` 与 `TerrainMarker`，适合迁移到 source/artifact/runtime 三层。
3. terrain package 标记 `Partial`，runtime importer 使用 `DiagnosticOnlyAssetImporter` 并明确“backend is not installed”；当前诚实 fail-close 应保留。
4. Render feature 系统把 Terrain 标成 explicit opt-in 的 descriptor-only slot，没有伪造 render pass；未来应替换该 slot，而不是保留一个同名平行 provider。
5. Scene mesh extraction、GPU Scene/visibility、render graph、material/PSO 与通用 residency 已有独立 owner，terrain 应以 provider/adaptor 复用这些基础设施。
6. Physics Jolt provider已经有 generational mesh asset registry、static-body restriction 与 descriptor validation；应新增真正 height-field resource path，不必另造第二个 physics world。
7. Navigation 已有 tiled bake、dirty bounds、agent/crowd与 Recast backend入口；terrain 只需提供稳定 geometry generation 与 dirty-region adapter，但不能继续把固定 quad 当真实地形。
8. Vampire 同时保留普通 mesh 与 terrain source，适合改造成迁移期 A/B oracle：同一相机比较预烘焙 mesh 与真实 terrain runtime 的几何、材质、碰撞和 nav 结果。

## 4. 参考实现给出的工程边界

### 4.1 Unreal：Landscape 是跨 Render、Collision、Grass、Nanite、Streaming 与 World Partition 的长期系统

Unreal Runtime Landscape 不把高度图当一个临时 mesh。`LandscapeComponent` 管理 section、height/weight textures、material instance 与 collision component；`LandscapeRender.cpp` 建 scene proxy/render system、按 view 选择 LOD，并将邻接 section LOD 纳入连续性；`LandscapeCulling` 维护 GPU culling 输入；`LandscapeCollision` 直接使用 Chaos height field；`LandscapeGrass` 有 guard band、cache lifetime、异步 builder 与 tick budget；texture streaming manager、Nanite component、HLOD builder 和 World Partition actor descriptor各自拥有明确生命周期。VirtualHeightfieldMesh 再展示 virtual texture、min/max hierarchy、scene proxy 与 vertex factory如何形成独立可选路线。

这套源码的可迁移原则不是复制 UObject 数量，而是：source/component、derived artifact、render proxy、collision representation、grass cache、partition cell 与 HLOD artifact必须拥有不同 identity/generation；它们通过显式 invalidation 和 budget 同步，不能共享一个 `Vec<f32>` 并假定所有消费者同时更新。

### 4.2 Fyrox：较小实现也具有 Chunk、Quadtree、LOD、查询与材质层闭环

Fyrox terrain 只有 6 个核心 Rust 文件，但仍建立 `Terrain -> Chunk -> heightmap texture -> QuadTree -> TerrainGeometry`。Chunk 持有 physical size、height-map size、modification count与 quadtree；修改检测会重建空间层级；modified CDLOD 根据观察者选择 block；API支持高度/法线查询、raycast、bounds、chunk接缝 margin、layer masks与 render data。它证明工程闭环不依赖 Unreal 体量，Zircon 当前缺失的是 owner 和执行路径，而不是文件数量。

### 4.3 Unity Graphics：TerrainLit、GPU driven 与 ray/path integration是不同责任层

Unity Graphics 仓库不拥有 Unity core `TerrainData` authoring，但拥有实际消费合同。HDRP TerrainLit 从 height/normal/hole texture与 per-instance patch数据执行 vertex displacement，支持 splat/height blend、per-pixel normal、decal、depth/GBuffer和 ray-tracing模板；Core `GPUResidentDrawer` 管理 instance data、LOD group、GPU culling、occlusion、indirect batch与debug stats；Unified RT 的 `TerrainToMesh` 有异步 job、hole过滤、法线与明确 UV extent，Path Tracing还有 tile/cell DDA terrain intersection。Zircon不能用一个 Terrain feature enum同时代替这三层。

### 4.4 Godot：没有内建3D terrain renderer，但物理与大规模实例合同是真实的

Godot当前参考树没有通用3D terrain renderer，因此不能作为 Landscape render完成度基线；但 `HeightMapShape3D` 校验 width/depth/data/min/max并提交 PhysicsServer，Jolt provider优先创建原生 `HeightFieldShapeSettings`，只有不满足原生约束时才构建 mesh fallback，还显式处理 hole 与三角剖分差异。`MultiMesh` 提供批量buffer、可见实例数、custom data、AABB和physics interpolation，`MultiMeshInstance3D`还向 navmesh parser暴露每个可见实例。两者直接指出 Zircon 的 height field 三角展开与静态 grass model不是终态。

### 4.5 Bevy：没有内建 Terrain，因此只参考可插拔数据通路

Bevy当前参考树只含 terrain示例模型，没有引擎内建 terrain source/runtime。其适用证据是 `RenderAsset`、mesh extraction、GPU preprocessing、meshlet instance manager和GPU culling：第三方 terrain应编译为稳定 render asset并进入同一可见性/批处理主数据，而不是在渲染器外维护不可观测私有列表。报告不把 Bevy 的通用 meshlet能力误记为 Terrain实现。

## 5. Owner 裁决与非重复边界

| Owner | 本篇拥有 | 本篇不重复拥有 |
|---|---|---|
| Runtime29 | Terrain source/artifact/runtime instance、patch/LOD/culling、terrain surface、physics/nav adapter、foliage runtime、partition runtime adapter、scalability/evidence | sculpt/import UI、通用 RHI/material/residency、通用 world authoring |
| Editor16 | Terrain/Foliage/Scatter/Partition authoring document、operation/toolkit、transaction、preview UI、partition manifest authoring、现有5个产品P0 | runtime terrain算法、GPU resource、collision/nav execution、运行时cell authority |
| Runtime05 | ECS/world生命周期、component registration与scene attach/detach | Terrain component内容与derived resource状态机 |
| Runtime09A | RHI resource/barrier/fence/device loss | terrain patch/height/weight/hole资源配方与代际 |
| Runtime09B | 通用GPU Scene、visibility、instance与LOD主数据 | terrain quadtree/clipmap选择、foliage prototype/scatter语义 |
| Runtime09C | shader/material/PSO compiler与permutation | TerrainLit layer contract、surface inputs与terrain pass资格 |
| Runtime09D | 通用asset streaming/residency/budget/eviction | terrain chunk/page、collision/nav/HLOD协调及artifact schema |
| Runtime23/24 | 坐标/单位/大世界与stable identity/generation | terrain grid origin、cell/patch generation的专属组合规则 |
| Plugins04 | rendering umbrella/package/profile/capability装配真实性 | terrain backend和算法实现 |
| App06 | Vampire产品闭环、截图/帧/交互证据 | 通用Terrain runtime实现；样例只消费其正式接口 |

Virtual Texture的feedback/page table/physical tile pool继续由 Runtime09D 拥有；Runtime29只声明 terrain consumer对tile、fallback mip与residency receipt的要求。通用World Partition manifest authoring由 Editor16拥有；Runtime29拥有manifest进入desired/resident/attached/evicted终态的运行时authority。Terrain插件的package/catalog清理由 Plugins04执行，runtime provider合同由本篇定义。

## 6. P0 裁决与升级条件

本篇没有新增P0。Editor16 P0-3已经拥有“runtime backend为诊断占位且没有render/height query/collision/nav consumer”，P0-5拥有partition/streaming authority缺失；重复登记会扭曲中央账本。以下条件出现时，直接回写既有P0 owner或新增跨owner failure handoff：

1. Runtime/Editor/Hub把 descriptor-only `BuiltinRenderFeature::Terrain` 的 compile success展示为 Terrain rendered/executed。
2. Vampire普通 `jungle_terrain.model.toml` 的mesh draws、pixels或帧率被归因于 `TerrainAsset` runtime。
3. 打开或保存含 `[entities.terrain]` 的scene后仍不提示引用被丢弃，或者产品称 roundtrip成功。
4. Diagnostic-only RAW/R16/PNG importer或空native command manifest被展示为可导入、可烹饪、可发行backend。
5. 固定quad nav bake、builtin physics静默skip或Jolt triangle expansion被标记为terrain collision/nav qualified。

## 7. P1：Source、Artifact、Scene 与 Runtime Instance

| ID | 当前差距 | 重构要求 |
|---|---|---|
| TER-P1-001 | `TerrainAsset`直接内联全部`Vec<Real>`，source、cooked artifact与live resource同形 | 硬切`TerrainSourceAsset`、`TerrainBuildArtifact`、`TerrainRuntimeInstance`，分别版本化authoring、派生页与live handles |
| TER-P1-002 | 没有stable terrain/chunk/patch/layer ID或generation | 使用Runtime24 identity规则建立`TerrainId/ChunkId/PatchId/LayerId { slot,generation,world }`及stale rejection |
| TER-P1-003 | `validate_dimensions`只在samples非空时比较乘积，零尺寸、空数据、乘法溢出与非finite值可进入artifact | checked dimension/sample/byte upper bound，拒绝零尺寸、NaN/Inf、非法spacing/scale与超预算输入 |
| TER-P1-004 | schema只有width/height/sample_spacing/height_scale，没有origin、axis、quantization、min/max、hole或border | 建versioned grid descriptor，显式unit/axis/origin/extent/sample format/hole/border与coordinate migration |
| TER-P1-005 | `TerrainLayerAsset`只有name/material/weightmap/strength | 层必须含stable ID、blend mode、weight extent/format/channel、UV mapping、physical material、visibility与priority |
| TER-P1-006 | layer stack没有weight sum、dimension、channel alias、dependency cycle与材质资格校验 | build阶段生成canonical layer packing和逐patchdiagnostics，非法组合不得延迟到shader |
| TER-P1-007 | Vampire `.zmeta` 的terrain dependencies为空，尽管source引用ground material | artifact dependency graph必须从typed direct references生成并验证；source/material/weight改变触发精确失效 |
| TER-P1-008 | builtin `.terrain.toml` importer与插件RAW/R16/PNG importer是两套身份，后者只有diagnostic backend | 统一canonical importer/build provider；source format decoder与artifact compiler分层，删除重复truth |
| TER-P1-009 | plugin native runtime是stateless、0 command/event，仅返回registration manifest | native/provider若保留，必须贡献typed build/runtime methods、lifecycle、state schema和unload；否则仅作manifest package不得声称backend |
| TER-P1-010 | 没有artifact version、builder version、target profile、content hash、quality或migration receipt | 建`TerrainArtifactKey`与可复现build receipt，支持旧version拒绝/迁移和target-specific rebuild |
| TER-P1-011 | `SceneTerrainAsset`只持一个terrain引用，缺少layer override、runtime policy、collision/nav/foliage资格 | 定义最小scene component只引用source/artifact与typed policy，避免复制大数组，同时允许显式consumer opt-in |
| TER-P1-012 | 没有Terrain runtime owner、load/prepare/resident/attached/retiring/failed状态机 | 建world-owned service和generation-bound ticket；取消、坏依赖、OOM、device loss、world unload均有终态 |

## 8. P1：Scene Persistence、Render Extraction、Geometry、LOD 与 Culling

| ID | 当前差距 | 重构要求 |
|---|---|---|
| TER-P1-013 | `SceneNode`/`NodeRecord`没有terrain字段，ECS/reflection/query不可见 | 注册typed `TerrainComponent`并进入scene reflection、inspection、clone、prefab与world snapshot合同 |
| TER-P1-014 | `World::from_scene_asset`从不读取`entity.terrain` | load必须解析handle、建立runtime ticket并在失败时产生entity-scoped diagnostic，禁止静默降级 |
| TER-P1-015 | `World::to_scene_asset`固定写`terrain: None` | save/roundtrip必须保留source reference与policy；未迁移前对含terrain的world fail-close而不是数据丢失 |
| TER-P1-016 | `RenderSceneGeometryExtract`只有meshes/lights，`RenderFrameExtract`没有terrain payload | 增加versioned `TerrainRenderExtract`，只携带immutable frame handles/patch decisions，不复制height samples |
| TER-P1-017 | Terrain是descriptor-only slot，descriptor仅有`extract_sections=[terrain]`且phase/pass为空 | 替换为真实provider descriptor、executor registration、pass dependencies和output receipt；删除零pass同名slot |
| TER-P1-018 | Terrain slot没有capability requirement或backend provider gate | 编译profile比较texture/array/storage/indirect/format/limit需求，provider缺失时typed unavailable，不以枚举存在为ready |
| TER-P1-019 | 没有height/normal/weight/hole GPU artifact、usage、layout或upload路径 | build生成target-aware resources和streaming pages，记录format、mip、alignment、bytes、generation与retire fence |
| TER-P1-020 | 没有共享patch vertex/index topology、section/chunk bounds或draw packet | 建immutable patch topology与per-patch data，支持reuse、draw packet cache、debug wireframe和deterministic index winding |
| TER-P1-021 | 没有quadtree、clipmap、CDLOD或screen-error选择 | 建view-dependent LOD selector，输入projection/viewport/error threshold/height bounds并输出stable patch set |
| TER-P1-022 | 没有邻接LOD约束、stitch index、skirt或geomorph | 定义邻居level delta与crack-free策略，跨chunk、负坐标、世界原点迁移和极端scale均有测试 |
| TER-P1-023 | 没有terrain frustum/HZB/occlusion culling与visibility reason | 接入Runtime09B visibility主数据，按patch bounds执行CPU/GPU culling并保留reason counters |
| TER-P1-024 | 没有GPU-driven instance/patch preprocessing、indirect args或compaction | 大地形走GPU patch list/indirect draw，CPU fallback保持同一结果schema并有容量/溢出策略 |
| TER-P1-025 | 没有VirtualHeightfieldMesh/Nanite/meshlet路线或与普通patch路线的eligibility/fallback | 定义可选virtualized provider，不能让Terrain功能强依赖尚未完成的VG；fallback必须可测且视觉等价范围明确 |
| TER-P1-026 | terrain没有depth/prepass/GBuffer/forward/shadow/motion/picking/reflection/GI资格矩阵 | 为每个pipeline声明真实pass、resource access、material permutation与skip reason，禁止只在beauty pass可见 |

## 9. P1：Material Layer、Surface Shading 与 Lighting Integration

| ID | 当前差距 | 重构要求 |
|---|---|---|
| TER-P1-027 | strength与可选weightmap没有规范化、default layer、zero-sum或overweight规则 | artifact compiler产生确定性normalized weights，记录clamp/renormalize和缺失层diagnostics |
| TER-P1-028 | 没有splat channel packing、layer count limit、per-patch active layer list | 按target limits编译channel/page layout，仅绑定active layers，并在超过上限时明确bake/fallback/reject |
| TER-P1-029 | 没有height blend、normal blend、mask/AO/roughness与physical material组合合同 | 定义TerrainSurfaceData和layer blend graph，保持能量/法线空间正确并与Runtime09C material ABI一致 |
| TER-P1-030 | 没有holes/visibility mask贯穿depth、shadow、collision、nav和ray路径 | 一个canonical hole artifact驱动所有consumer；边缘过滤和LOD mip不得产生几何/碰撞不一致 |
| TER-P1-031 | 没有macro variation、distance tiling、triplanar/slope/height rule或far-field basemap | 将近景layer与远景baked basemap分开编译，quality profile控制过渡并有reference image gate |
| TER-P1-032 | 没有terrain tangent/normal reconstruction规则，sample spacing/height scale未进入shader contract | 固定cell diagonal、gradient kernel、border sampling与world normal转换，CPU/GPU/physics query使用同一定义 |
| TER-P1-033 | 没有decal、runtime virtual texture、lightmap/probe、shadow bias和Hybrid GI接线 | 通过现有feature gateway消费，不在terrain shader私建平行lighting；每项有eligibility/fallback receipt |
| TER-P1-034 | 没有layer texture residency与LOD选择协同，地形可在geometry resident时采样缺页材质 | geometry/material page共享desired priority但保留独立状态，fallback mip与missing-page可观测 |
| TER-P1-035 | 没有shader permutation预算、PSO warmup或layer组合cache key | Runtime09C生成terrain-specific compatibility key，限制组合爆炸并在cook阶段生成warmup manifest |
| TER-P1-036 | 没有terrain pass的golden、edge/seam、layer blend、hole、shadow与motion vector图像测试 | 建CPU reference和多backend image/metric suite，区分结构正确性、视觉容差与性能回归 |

## 10. P1：Physics、Height Query 与 Navigation

| ID | 当前差距 | 重构要求 |
|---|---|---|
| TER-P1-037 | `TerrainAsset`与`PhysicsMeshAsset::HeightField`完全独立，必须手工复制samples | terrain artifact生成generation-bound physics view，禁止两个可独立编辑的高度真相 |
| TER-P1-038 | Jolt height field被展开为`2*(w-1)*(h-1)`triangle shapes | 使用Jolt原生HeightFieldShape与其block/min-max/holes能力；只有不支持的输入走显式mesh fallback并报告成本 |
| TER-P1-039 | builtin physics把HeightField判invalid，AABB、raycast、contact均返回None | builtin provider要么实现一致query，要么在sync/admission时拒绝backend资格，不能让collider注册后静默无碰撞 |
| TER-P1-040 | 没有统一sample_height/normal/material、raycast、bounds与cell lookup API | 建thread-safe `TerrainQuerySnapshot`，明确边界、hole、插值、triangle diagonal、transform和generation语义 |
| TER-P1-041 | 没有按dirty region增量更新collision tile，任意编辑只能全量重建 | artifact dirty graph生成collision tile jobs，旧generation在新shape commit前保持可用并原子切换 |
| TER-P1-042 | collision没有simple/complex、LOD、material layer、memory budget或cook cache | profile编译collision resolution与physical material，缓存按source/version/backend key管理 |
| TER-P1-043 | Navigation render source把每个mesh节点简化为固定quad，asset-backedheight/triangle collider又被跳过 | terrain adapter直接提交真实patch/collision triangles或heightfield spans，禁止用节点占位quad冒充地形 |
| TER-P1-044 | terrain edit/streaming与dirty nav tile没有generation/attach顺序 | 建Terrain-to-Nav dirty adapter，collision commit、nav rebuild与world cell attach使用同一source generation及取消协议 |

## 11. P1：Foliage、Scatter 与大规模实例运行时

| ID | 当前差距 | 重构要求 |
|---|---|---|
| TER-P1-045 | production没有foliage prototype、scatter rule、instance artifact或runtime component | 建`FoliagePrototype/ScatterArtifact/FoliageRuntimeCluster`，与Editor16 authoring IDs一一对应 |
| TER-P1-046 | Vampire foliage是手工摆放GLB和预合并grass billboard model | 样例迁移为真实scatter/cluster consumer，同时保留预烘焙mesh作oracle，不能把static batch称为foliage system |
| TER-P1-047 | 没有seed、cell、source generation、prototype与filter order组成的确定性实例identity | 使用counter-based RNG和stable instance key，跨线程/平台/重载得到相同transform与排除结果 |
| TER-P1-048 | 没有density/slope/height/layer/mask/collision filters或override/exclusion runtime merge | artifact compiler固定filter顺序与tie-break，changed-cell invalidation保留手摆实例ownership |
| TER-P1-049 | 没有HISM/MultiMesh式cluster bounds、GPU instance buffer、LOD/cull/indirect draw | 接入Runtime09B instance主数据，按cluster/instance分层culling并提供CPU fallback与overflow receipt |
| TER-P1-050 | 没有billboard/impostor、LOD crossfade、wind、bending、season与motion vector合同 | prototype声明render route与quality tiers，wind/history进入统一frame data而非material magic values |
| TER-P1-051 | WOC有`foliageDensity`设置但没有引擎runtime consumer，Vampire也无scalability linkage | quality profile把density/distance/LOD/wind映射到typed compiled policy并记录实际instance/draw/bytes |
| TER-P1-052 | foliage collision、nav obstacle、ray tracing、lightmap/GI与world cell ownership不存在 | 各consumer按prototype资格生成派生artifact，随cell/generation原子attach/detach，禁止逐实例临时注册 |

## 12. P1：World Partition、Residency、HLOD 与 Large World

| ID | 当前差距 | 重构要求 |
|---|---|---|
| TER-P1-053 | production没有World Partition manifest、runtime grid、stable cell或streaming source | 消费Editor16 versioned manifest，建立world-owned desired/requested/resident/attached/evicting状态机 |
| TER-P1-054 | terrain chunk、foliage cluster、collision/nav tile与world cell没有ownership/dependency图 | 一个cell bundle记录各artifact generation、dependency与cost，partial attach失败执行原子rollback |
| TER-P1-055 | 没有多个camera/player/portal/source的priority、hysteresis、prefetch与cancel策略 | desired-set compiler合并source shape/velocity/importance，预算变化与priority inversion有确定性终态 |
| TER-P1-056 | 没有CPU/GPU/IO/decompress/collision/nav/instance分域预算与pressure反馈 | 输出per-domain requested/resident/pinned/evictable bytes、latency与denial reason，接Runtime09D全局budget |
| TER-P1-057 | 没有terrain/foliage HLOD、source key、builder version、screen error或替换过渡 | HLOD为可流送派生artifact，近远表示原子切换，source drift、hole/material/foliage变更精确失效 |
| TER-P1-058 | 大世界grid origin、negative coordinate、floating origin与cell/patch identity未定义 | 应用Runtime23/24规则，以integer grid/local coordinates计算，origin shift不改变artifact/instance identity |

## 13. P1：Scalability、Diagnostics、Product Evidence 与 Performance

| ID | 当前差距 | 重构要求 |
|---|---|---|
| TER-P1-059 | 没有Terrain quality profile，LOD、layers、collision、foliage、streaming各域无法联合编译 | 建low/medium/high/ultra及custom compiled policy，约束screen error、active layers、page/cell budgets与degrade order |
| TER-P1-060 | diagnostics没有terrain instance/patch/triangle/page/layer/collision/nav/foliage/cell指标 | 建frame与lifecycle receipt，区分requested/compiled/resident/visible/drawn/queried/baked/attached和fallback原因 |
| TER-P1-061 | 测试只证明registration、TOML roundtrip和普通mesh可见，没有真实terrain output | 增加source-to-pixel、source-to-height query、source-to-collision/nav、save/reopen与stream/evict端到端测试 |
| TER-P1-062 | 没有相同画质/场景/硬件下对Unreal/Fyrox或旧baked mesh的性能与质量基线 | 建固定camera/seed/assets/profile的benchmark corpus，记录median/p95/p99 frame、CPU/GPU、VRAM/RAM、IO、stutter与image metrics；“优于Unreal”只能由同口径receipt支持 |

## 14. P2：成熟度、可维护性与工作流差距

| ID | 当前差距 | 改进要求 |
|---|---|---|
| TER-P2-001 | Terrain/heightfield/landscape命名分散 | 发布术语表与canonical type/path，兼容别名只存在于versioned migration |
| TER-P2-002 | 缺少terrain wireframe、patch/LOD、bounds、normal、layer、hole debug views | 通过Editor22 debug framework注册，不在provider内私建overlay |
| TER-P2-003 | 缺少collision/nav/visual divergence heatmap | 对同一generation采样三类surface并输出位置/法线/material差值 |
| TER-P2-004 | 缺少streaming cell/page/HLOD驻留可视化 | 显示desired/resident/pinned/evicting、cost与source，不用固定统计数字 |
| TER-P2-005 | 缺少terrain artifact inspector与build provenance | 展示source hash、builder/backend/profile、pages、layers、bounds、依赖与diagnostics |
| TER-P2-006 | 缺少确定性scatter replay与instance diff工具 | 按cell/seed/prototype输出added/removed/moved与filter reason |
| TER-P2-007 | 缺少边界语料：1xN、极端高差、holes、负scale、非方形与坏weightmap | 建fuzz/property corpus，所有拒绝路径有typed error且无OOM/panic |
| TER-P2-008 | 缺少跨backend height/normal/diagonal一致性测试 | CPU、WGSL、physics、nav和ray/path使用共享fixtures与误差预算 |
| TER-P2-009 | 缺少terrain shader compile/permutation统计 | cook receipt记录变体来源、命中率、warmup覆盖与剔除理由 |
| TER-P2-010 | 缺少foliage cluster rebuild、wind与LOD transition profiler | 暴露CPU/GPU cost、cluster occupancy、overdraw和crossfade pixels |
| TER-P2-011 | 缺少保存格式兼容矩阵和旧terrain migration fixture | 每个schema version提供forward/backward policy、fixture与loss report |
| TER-P2-012 | 缺少headless/dedicated server terrain资格 | server可加载query/collision/nav artifact而不初始化render resource，并验证内存预算 |
| TER-P2-013 | 缺少文档化支持上限 | 发布dimension/layer/page/cell/instance/backend limits及失败语义，不从实现常量反推合同 |
| TER-P2-014 | 缺少性能改动的质量防退化门 | 任何更快路线必须同时通过geometry seam、material、collision/nav和image quality阈值 |

## 15. 目标架构与硬切合同

```text
TerrainSourceAsset
  -> TerrainArtifactCompiler
       -> TerrainBuildArtifact
            - height/normal/hole pages
            - layer packing + material dependencies
            - patch bounds + min/max hierarchy
            - collision/nav/foliage/HLOD recipes
  -> TerrainRuntimeService (world owner)
       -> TerrainRuntimeInstance { id, generation, state, resident chunks }
            -> TerrainRenderProvider -> TerrainRenderExtract -> render graph passes
            -> TerrainPhysicsAdapter -> native height-field resources
            -> TerrainNavigationAdapter -> dirty tiled bake inputs
            -> FoliageRuntimeProvider -> clusters/instances/indirect draws
            -> WorldPartitionAdapter -> desired/resident/attached cell bundle
       -> TerrainRuntimeReceipt
```

核心硬切规则：

1. Source、artifact、runtime instance不再共用同一序列化struct；source可编辑，artifact不可变且可重建，runtime只持generation-bound handles。
2. Height、normal、hole、layer、collision、nav和foliage都必须声明source generation；任何消费者不得以“最近加载的同名资源”猜测版本。
3. Terrain render provider使用现有render graph/GPU Scene/material/residency合同，不创建第二套device、visibility或shader truth。
4. Scene persistence在migration完成前fail-close；禁止继续把未知terrain component静默写成None。
5. 普通mesh fallback保留明确`FallbackMesh` route与原因，不能计入Terrain provider execution receipt。

## 16. 依赖有序重构里程碑

### M0 · Truth Hard Cut

- 隐藏或标记descriptor-only Terrain为Unavailable，阻止零pass feature进入Ready/Executed。
- 对scene terrain load/save数据丢失增加typed hard failure，并引用Editor16现有P0。
- 建立canonical Terrain owner、schema、artifact与receipt名称。

### M1 · Source/Artifact/Identity

- 完成versioned source/grid/layer/hole schema、严格validation与dependency graph。
- 实现deterministic artifact compiler、stable chunk/patch/layer identity和migration fixtures。

### M2 · Runtime Instance 与 CPU Reference

- 建TerrainRuntimeService、lifecycle/ticket/generation与Scene/ECS roundtrip。
- 实现CPU sample/normal/raycast/bounds、reference patch topology与seam oracle。

### M3 · GPU Patch Render MVP

- 上传height/normal/hole/layer resources，完成patch draw、depth/GBuffer/forward/shadow/picking。
- 接入render graph、material compiler、GPU lifetime和真实execution receipt。

### M4 · Quadtree LOD、Culling 与 GPU Driven

- 实现screen-error LOD、neighbor constraint、crack-free transition与HZB/occlusion。
- 建GPU patch preprocessing、indirect draw、overflow/degrade与性能计数。

### M5 · Terrain Surface 与 Lighting

- 完成layer packing/blend、normal/height/hole、far basemap、decal与lighting integration。
- 建shader permutation、PSO warmup、golden image与quality profile。

### M6 · Physics、Query 与 Navigation

- 使用native height-field backend、增量collision tile与material mapping。
- 将真实terrain geometry接入dirty tiled nav bake，并建立generation/commit顺序。

### M7 · Foliage Runtime

- 编译deterministic scatter artifact，建立prototype/cluster/instance identity。
- 完成GPU culling/LOD/billboard/wind、density scalability与consumer adapters。

### M8 · Partition、Residency 与 HLOD

- 消费versioned partition manifest，完成多source desired set、预算、attach/evict/rollback。
- 生成terrain/foliage HLOD并验证large-world/origin shift稳定性。

### M9 · Product Qualification 与性能超越门

- 将Vampire从baked mesh主路径迁移到真实terrain，保留A/B oracle和失败回退。
- 在同硬件、同画质、同相机路径下与固定Unreal/Fyrox基线比较；没有完整receipt不得宣称性能或表现领先。

## 17. 验收门

1. Terrain source零尺寸、溢出、NaN/Inf、非法spacing/scale和超预算输入全部typed拒绝。
2. 非空height samples严格等于checked width*height，空source只允许显式procedural/provider类型。
3. Grid axis/origin/unit/extent/diagonal/border/hole规则有versioned schema与migration fixture。
4. Layer weights维度、channel、normalization、default/zero-sum和material dependency可复验。
5. 相同source/profile/compiler version生成byte-identical artifact与相同composite key。
6. source/material/weight/hole局部变化只失效受影响artifact与consumer generation。
7. Scene load把terrain reference变为typed component和runtime ticket，不再静默忽略。
8. Scene save/reopen保留terrain source、policy与identity；未迁移输入fail-close且不改写文件。
9. world clone/prefab/inspection/reflection/snapshot均能保留或明确拒绝Terrain component。
10. runtime lifecycle覆盖requested/preparing/resident/attached/retiring/failed/cancelled终态。
11. stale Terrain/Chunk/Patch handles在world reload、device loss和slot reuse后被拒绝。
12. headless world能加载query/collision/nav artifact且不创建GPU resource。
13. Terrain feature不再是零phase/零pass descriptor-only slot。
14. provider缺失或capability不足时profile返回typed unavailable/fallback，不返回Ready。
15. render extract只携带immutable handles与patch decisions，不复制完整height/weight arrays。
16. height/normal/hole/layer GPU resources拥有format/layout/bytes/generation/fence receipt。
17. patch topology在非方形、chunk边界、负坐标和极端scale下winding/bounds正确。
18. quadtree/clipmap选择满足screen-error预算，并对多view输出稳定结果。
19. 任意相邻LOD组合无裂缝、T-junction可见缝或未定义geomorph。
20. frustum/HZB/occlusion结果有reason计数，camera cut与history invalidation正确。
21. GPU-driven overflow有确定性fallback，不丢patch、不越界且有diagnostic。
22. depth/GBuffer/forward/shadow/motion/picking各pass与beauty geometry一致。
23. virtualized route不可用时普通patch route保持可见并报告fallback原因。
24. layer blend在0/1/多层、hole边缘、mip过渡和far basemap过渡下通过golden。
25. CPU/GPU terrain normal与tangent在约定误差内一致。
26. decal/lightmap/probe/GI/shadow资格有明确pass/fallback receipt。
27. material page缺失时只采样合法fallback mip，不读取未驻留资源。
28. terrain shader permutation与PSO warmup受预算约束且cook receipt完整。
29. reference images覆盖seam、layer、hole、shadow、motion与LOD transition。
30. TerrainAsset到Physics height-field无手工sample复制和独立可编辑真相。
31. Jolt首选native HeightFieldShape；mesh fallback只在明确不兼容输入触发并报告triangle/bytes/cook time。
32. builtin physics不支持height field时在admission阶段拒绝，不让ray/contact静默返回None。
33. visual/query/physics对height、normal、hole与triangle diagonal使用同一fixture。
34. terrain dirty region只重建相交collision/nav tiles，并按generation原子commit。
35. 相同scatter source/seed/cell/prototype跨线程和平台产生相同stable instances。
36. foliage density、slope/height/layer/mask/collision filter与override/exclusion结果可解释。
37. foliage cluster GPU culling、LOD、billboard/wind/motion在overflow和camera cut下稳定。
38. `foliageDensity`设置改变真实instance/draw/bytes receipt，不只是保存配置值。
39. 多streaming source合并desired set，priority/hysteresis/cancel与budget pressure结果确定。
40. terrain/foliage/collision/nav/HLOD cell bundle partial failure原子rollback。
41. cell/HLOD attach/evict不产生一帧视觉、碰撞或导航代际混合。
42. origin shift、负world grid和大坐标不改变cell/patch/instance stable identity。
43. Vampire首帧terrain pixels来自Terrain provider receipt；普通mesh fallback被单独标记。
44. 同口径benchmark同时记录CPU/GPU/内存/IO/stutter与image quality，领先声明可由原始receipt复算。

## 18. Finding 到里程碑映射

| Finding | 主里程碑 |
|---|---|
| TER-P1-001..012 | M0-M2 |
| TER-P1-013..018 | M0-M3 |
| TER-P1-019..026 | M3-M4 |
| TER-P1-027..036 | M5 |
| TER-P1-037..044 | M6 |
| TER-P1-045..052 | M7 |
| TER-P1-053..058 | M8 |
| TER-P1-059..062 | M5、M8、M9 |
| TER-P2-001..014 | 对应主里程碑完成后收口，不得替代P1执行 |

## 19. 本轮验证与未执行项

- 已验证113个冻结输入全部存在，分组行数/bytes与组合SHA可复算。
- 已核对Terrain production consumer：asset/import/load存在；Scene world、render extract、render executor、terrain query、foliage/partition production consumer缺失。
- 已核对`BuiltinRenderFeature::Terrain`走descriptor-only目录，生成extract name但0 phase/0 pass。
- 已核对Vampire terrain entity同时携带普通mesh与terrain reference，现有pixel/draw evidence只能证明mesh路径。
- 已核对Jolt height field展开triangle set、builtin provider fail-close、Navigation fixed-quad/asset-backed skip边界。
- 已核对Unreal/Fyrox/Bevy/Godot/Unity Graphics适用性；没有把不存在于参考树的Bevy/Godot/Unity core Terrain实现计入证据。
- 本轮未修改production/test/Cargo/workflow，未运行无法增加Terrain执行证据的Cargo/GPU lane；实施阶段必须从M0真相硬门开始。
