---
related_code:
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/model
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/render/frame_extract/geometry.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/advanced_slot.rs
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/shader/template
  - zircon_runtime/src/graphics/shader/wgsl
  - zircon_plugins/terrain/runtime
  - examples/vampire/assets/models/grass_billboard_static_batch.model.toml
  - examples/vampire/assets/models/jungle_broadleaf.model.toml
  - examples/vampire/assets/models/jungle_fern_cluster.model.toml
  - examples/vampire/assets/materials/forest_grass_billboard.zmaterial
tests:
  - zircon_runtime/src/asset/tests/assets/mesh.rs
  - zircon_runtime/src/asset/tests/assets/model.rs
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_runtime/src/asset/tests/project/example_vampire
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/28-hardware-ray-tracing-blas-tlas-ray-query-pipeline-sbt-denoising-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/31-cloth-fabric-soft-body-garment-simulation-collision-deformation-rendering-wind-lod-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Foliage
  - dev/UnrealEngine/Engine/Source/Runtime/Foliage/Public/FoliageType.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/HierarchicalInstancedStaticMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/HierarchicalInstancedStaticMesh.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SpeedTreeWind.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SpeedTreeWind.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Shaders/Nature
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Shaders/Terrain
  - dev/Graphics/Packages/com.unity.shadergraph/ShaderGraphLibrary/Nature
  - dev/godot/scene/resources/multimesh.cpp
  - dev/godot/scene/3d/multimesh_instance_3d.cpp
  - dev/bevy/crates/bevy_render/src/batching
  - dev/bevy/crates/bevy_pbr/src/meshlet
  - dev/Fyrox/fyrox-impl/src/renderer
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 34 · Vegetation、Tree、Foliage、Grass、Species、Instancing、Wind、Billboard、Impostor、LOD、Streaming、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前没有独立的工程级 Vegetation runtime。对 Runtime、Runtime Interface、Plugins、Editor、App 与 Hub production 树执行精确标识搜索，`VegetationSpecies`、`FoliagePrototype`、`FoliageRuntimeCluster`、`SpeedTree`、`Impostor`、`WindDirectionalSource`、`WindField` 与 `FoliageShading` 均为零命中；`Tree` 虽有11个候选命中，但核心只是 `BuiltinRenderFeature::Tree` 及其descriptor/extract名称，和 `Billboard`、`Terrain`、`MeshLod` 一样没有对应pass/executor或产品提交链。它们不是物种资源、树木编译器、植被实例集、风动画或Impostor实现。

现有Mesh/Model/glTF、static batch extract、GPU Scene、visibility、ordinary LOD、Standard PBR与Terrain插件是真实通用基础，但产品链在关键处断开：`GeometryExtract::static_batches`没有graphics consumer；普通mesh构建对每个pending draw固定调用`gpu_scene.register(..., 1)`并只写一个`GpuInstanceData`；LOD只按entity原点到camera的距离选最大`min_distance`；176-byte实例ABI没有species、variation、wind phase、bend、season或interaction payload；shader模板直接fetch position/normal并构建vertex output，没有通用vertex deformation/WPO hook。Vampire的grass所谓static batch是预合并普通mesh，材质为opaque double-sided，forest shader只做fragment surface细节，既没有alpha-cutout、wind、billboard切换，也没有真实draw reduction证据。

Terrain29已拥有terrain-derived foliage/scatter、placement、world cell与相关碰撞/nav/RT集成差距；Editor16已拥有Foliage/Scatter authoring及其产品真实性P0；Runtime09B已拥有通用GPU-driven/instancing P0；App06已拥有Vampire产品证据。本篇不重复这些优先级，而登记 **0 P0 / 72 P1 / 16 P2**，专门拥有 `VegetationSpeciesSource -> deterministic VegetationBuildArtifact(parts/attributes/LODs/cards/impostor/wind/collision) -> VegetationPrototype -> Cell/Cluster/Instance Artifact -> VegetationRuntimeInstance -> Wind/Interaction State -> GPU Instance/Cluster/LOD/Streaming -> Mesh/Billboard/Impostor/Shadow/Velocity/GI/RT adapters -> typed receipt`。0 P0不表示接近完成，只表示catalog没有另行宣称该领域Ready，已有产品谎报由既有owner收口。

## 2. 审查边界、方法与 currentness

### 2.1 冻结输入

冻结语料为342个文件、80,851行、3,017,547 bytes：212个Zircon production文件为28,667行、1,027,774 bytes；15个focused test文件为4,354行、157,165 bytes；11个产品证据文件为3,354行、110,253 bytes；104个参考文件为44,476行、1,722,355 bytes。指纹算法为按forward-slash相对路径排序，逐文件计算小写SHA-256，形成`path<TAB>file_sha256`行，以单个LF连接且无末尾LF，再对UTF-8 payload计算SHA-256；结果为`f568baa7a537597749d612388e2f591d13905a5e796787cd95a49f4a8b66ac3f`。

冻结基线为`main@25e09a23178000f2e783ce2143cf70a8b118d404`，按读取时working bytes计算。Vampire部分model TOML在当前工作树存在但受ignore状态影响，本篇只把它们当当前产品证据，不把本机存在性当clean-clone闭包；该问题的唯一owner仍为App06。实施前必须重导342项manifest、重算指纹、检查working tree/HEAD差异并确认所有在途owner，不能把本篇结论当作未来源码的永久事实。

### 2.2 纵向检查链

本轮逐层检查source/import/provenance -> species/part/vertex semantics -> deterministic build/cook -> mesh/card/billboard/impostor representation -> stable prototype/cluster/instance identity -> Scene/world/cell lifecycle -> batch storage/mutation -> hierarchy/bounds -> LOD/transition -> visibility/occlusion/indirect -> wind source/field/deformation/history -> interaction/bending -> leaf/grass optical model -> depth/shadow/velocity/GI/RT -> collision/navigation/query -> streaming/residency -> scalability/diagnostics/tests -> Vampire产品证据。

15个focused test文件只覆盖ordinary mesh/model schema、frame extract、builtin feature枚举、generic GPU Scene、mesh draw、shader template、visibility、terrain和Vampire资产/静态batch DTO。没有species import/cook、branch/leaf semantic、prototype migration、instance add/remove/update、cluster tree、per-instance LOD、billboard/impostor bake/transition、wind history、bending、leaf shading、deformed shadow/velocity、vegetation RT、world-cell streaming或规模基准。

### 2.3 搜索口径与动态验证限制

精确production搜索结果为：`VegetationSpecies=0`、`VegetationAsset=0`、`FoliageAsset=0`、`FoliagePrototype=0`、`FoliageRuntimeCluster=0`、`GrassAsset=0`、`SpeedTree=0`、`Impostor=0`、`WindDirectionalSource=0`、`WindField=0`、`FoliageShading=0`。`TreeAsset`有11个文本候选，逐项复核未形成独立植被产品。仓库级参考树规模很大，本篇只冻结104个直接相关文件；Unreal、Unity Graphics用于专用正参考，Godot、Bevy用于通用批实例/GPU提交底座，Fyrox本地树未找到专用vegetation模块，只作负证据。

本轮是E3 source-level review，没有运行Cargo、WGPU、GPU capture、Vampire executable或植被压力场景。仓内没有可执行的vegetation owner可供动态验收；运行ordinary mesh或terrain测试不能证明专用闭环。所有运行时、图像和性能结论必须由后续里程碑产生raw receipts，本篇不把“未运行”写成通过。

## 3. 当前可保留的真实基础

1. `MeshAsset`已有typed vertex attributes、indices、material sections、bounds与LOD，可作为trunk/branch/frond/leaf/card输入；专用compiler仍须保存part semantics、wind weights、anchor与representation mapping。
2. glTF/Model import、labeled subasset与metadata管线可作为source入口；当前没有SpeedTree或等价植被schema、provenance和validation。
3. Scene `MeshRenderer`、world render snapshot与stable entity identity可承载少量ordinary tree；不能让百万植被实例退化为百万Scene entity。
4. `GeometryExtract::static_batches`已按model/mesh/material/layer聚合静态mesh索引，可作为representation-independent extract DTO的起点；graphics侧必须真正消费并证明draw/work reduction。
5. GPU Scene具备instance span allocator、current/previous transform、primitive/material/lightmap slot和staged upload，可扩展为通用实例底座；当前产品mesh路径把span固定为1。
6. visibility、indirect、HZB、shadow、velocity、RT与residency已有通用owner和接口方向；vegetation必须提供专用cluster bounds、LOD、deformation/history与fallback adapter。
7. Standard PBR已有double-sided、subsurface profile、transmission与alpha模式等材料primitive；叶片需要两面法线、透射/背光、thin surface、alpha coverage和多pass一致性，不能只切`double_sided`。
8. Terrain29、Runtime23/24和Editor38分别提供placement/world partition、space/identity及未来qualified WindField的owner方向；本篇消费这些合同，不再建立第二套坐标、cell或天气真相。

## 4. 当前代码事实与断路

### 4.1 Asset、Scene、LOD 与 Batch

1. Resource/asset schema没有VegetationSpecies、Prototype、BuildArtifact、InstanceSet、ImpostorAtlas、WindProfile或InteractionProfile类型。
2. glTF mesh importer只理解普通POSITION/NORMAL/TANGENT/TEXCOORD/JOINTS/WEIGHTS/COLOR等语义；没有branch hierarchy、leaf anchor、frond edge、wind weight/phase或species metadata。
3. `MeshRendererLodLevel`仅保存`min_distance`及model/mesh/material/primitives；`mesh_lod_for_camera`以entity origin距离选择已达到的最大门槛，没有projected error、screen coverage、hysteresis、crossfade、shadow LOD或per-instance choice。
4. `GeometryExtract::static_batches`排除material override并按普通mesh聚合，但graphics production树没有读取`static_batches`，因此DTO存在不等于提交合批成立。
5. `gpu_scene_sync.rs`对每个pending draw用`register(device, stable_instance_key, 1)`并上传一个实例；GPU Scene虽支持span，但当前mesh产品链没有批量实例输入、mutation或compaction。
6. `GpuInstanceData`为176 bytes，只含current/previous world matrix、primitive、flags、payload/morph slot和lightmap参数；没有species/prototype、random variation、wind phase、bend/interaction、season、LOD fade或representation state。
7. primitive bounds以transform translation作center、最大矩阵列长作radius，没有使用真实mesh local bounds，也没有wind-deformed conservative bounds；对高树、偏心树冠和强风都不可靠。
8. Scene没有实例集/cluster/cell component、稳定instance handle、批量add/remove/update、runtime density scale或save/reload迁移语义。

### 4.2 Shader、Wind、Rendering 与 Product

1. builtin `Tree`、`Billboard`、`Terrain`和`MeshLod`只是advanced feature descriptor/extract slot，pass/executor为空，不能从枚举存在推导执行能力。
2. forward/depth/shadow/velocity WGSL模板都直接fetch undeformed vertex attributes并构建输出；material customization停留在surface函数，没有统一vertex deformation/WPO hook。
3. `ZrVertexInput`没有branch/leaf/frond/anchor/bend语义；把植被数据塞入未声明的color/UV通道会破坏import、压缩、shader reflection和tooling合同。
4. production没有wind source、wind field、gust、turbulence、per-instance phase、current/history table或quality tier；`weather.Component.Wind`只在component registry测试中出现。
5. depth、shadow、velocity和main pass没有共享植被形变函数，无法保证轮廓、阴影和motion vector同代；TAA会把未记录的WPO视为错误运动。
6. Standard material没有thin-leaf/two-sided foliage optical model、back-light transmission、leaf subsurface profile约束或alpha-to-coverage政策；generic `Custom(String)`也不是qualified feature。
7. Vampire grass asset是预合并普通card mesh，`forest_grass_billboard.zmaterial`为opaque double-sided，shader只做fragment detail；不存在camera-facing billboard、wind、cutout、distance transition或impostor。
8. Vampire README声称六个grass static-batch entity折叠成一个extract batch并可供未来GPU draw merging消费；测试只验证DTO/资产，graphics没有consumer，因此不能作为真实draw reduction、性能或植被产品证据。

## 5. 参考实现给出的工程边界

### 5.1 Unreal Foliage：species policy与实例容器必须分层

Unreal `UFoliageType`把density/radius、scale、alignment、slope/height、landscape layer、cull distance、collision/body、mobility、lighting/shadow、custom depth、runtime virtual texture与WPO disable distance等策略归入type；`AInstancedFoliageActor`和`FFoliageInfo`维护level/world中的实例集合、base attachment、selection、move/delete与type映射。Zircon应吸收“species/prototype policy不等于instance storage”、稳定实例身份和world ownership，不复制UObject/Actor的具体组织。

### 5.2 HISM与Instance Culling：批实例需要层级bounds、mutation与GPU work闭环

`UHierarchicalInstancedStaticMeshComponent`维护cluster tree、built/async build state、occlusion layer、density scaling、batch transforms、add/remove/update和bounds；Renderer `InstanceCulling`把instance runs/batches转成visible compaction与indirect arguments。工程边界不是“共享mesh”四个字，而是可变实例集合、层级bounds、异步重建、culling/LOD/occlusion、提交work和receipt保持一致。Zircon现有span allocator/static DTO只覆盖其中一小段。

### 5.3 SpeedTree与Unity GPUDriven：风是有历史和部件语义的数据系统

Unreal `FSpeedTreeWind`维护global、branch、leaf、frond、gust、direction、strength、frequency、LOD与time等参数，并由wind source/component/proxy进入scene。Unity Graphics的`SpeedTreeWindGPUDataUpdater`、`InstanceWindDataUpdateDefs`、compute kernel和HLSL显式保存current/history风数据，覆盖branch twitch/whip/anchor/adherence、leaf ripple/tumble/twitch、frond ripple、turbulence与quality tier。Zircon需要provider-neutral wind field、species response、实例phase和previous deformation；一条正弦WPO不是工程级风系统。

### 5.4 Representation与材质：mesh、card、billboard、impostor必须是同一原型的资格化输出

Unity URP SpeedTree7/8、billboard pass、SpeedTree utility和WavingGrass各自处理顶点形变、camera-facing、alpha、depth normals与pass parity；ShaderGraph Nature库把SpeedTree8/9部件行为暴露给材质图。Unreal foliage还把cull、WPO disable、shadow/lightmap/RT等策略放在type和render proxy链路。Zircon应由同一build artifact生成mesh LOD、cards/impostor、bounds、material variants与transition metadata，并以同一instance identity跨表示切换，禁止由场景脚本手换模型。

### 5.5 Godot、Bevy、Fyrox的适用边界

Godot `MultiMesh`提供instance count/visible count、2D/3D transform、per-instance color/custom data、whole buffer、current/previous interpolated buffer和custom AABB，是通用批实例最低参考，但不是完整vegetation species系统。Bevy的extract/batching/GPU preprocessing、meshlet BVH/LOD/culling可作为可插拔提交底座；本地Fyrox树只找到generic renderer bundle/visibility，没有独立植被系统。负证据只能说明参考范围，不能降低Zircon目标或把ordinary instancing包装成完整植被。

## 6. 目标架构与唯一 Owner

```text
VegetationSpeciesSource + Import Provenance
  -> schema migration + semantic/topology/unit validation
  -> deterministic Vegetation Compiler
       -> Part/Attribute/Material Artifact
       -> Mesh LOD + Card/Billboard/Impostor Artifact
       -> Wind/Collision/Bounds/Streaming Metadata
  -> VegetationPrototype + Placement Artifact(Terrain29/manual/procedural)
  -> generation-qualified Cell/Cluster/Instance Set
  -> Wind Field + Species Response + Interaction State
  -> GPU Instance/Cluster/LOD/Visibility/Streaming
  -> Raster/Shadow/Velocity/GI/RT/Physics/Nav adapters
  -> terminal lifecycle, degradation and qualification receipts
```

| 领域 | 唯一 owner | Vegetation34 只消费/提供 |
|---|---|---|
| Resource/schema/artifact/DDC | Runtime04 | species source/build kind、dependency、install/retire receipt |
| Scene/ECS/world lifecycle | Runtime05 | instance-set component、world generation、teardown与serialization primitive |
| Vegetation domain | 新Runtime Vegetation owner | species/compiler/prototype、专用instance/cluster、representation、wind response、interaction truth |
| Terrain scatter/world cells | Runtime29 | TER-P1-045..052等placement/cell权威；本篇消费placement artifact并提供prototype/runtime adapter |
| Foliage authoring/workbench | Editor16 | 既有5项P0与brush/scatter/preview owner；共享runtime compiler，不能私制预览格式 |
| Generic GPU Scene/visibility | Runtime09B | 既有P0-4拥有span固定1及通用batch/culling；本篇提供vegetation layout、bounds、LOD work |
| Material/shader/PSO | Runtime09C | vertex deformation hook、thin-leaf model、pass variants与qualification primitive |
| Residency/streaming | Runtime09D | artifact/cell bundle admission、atomic install、pressure/retire policy |
| Wind/weather authority | Editor38对应runtime owner | 提供qualified WindField/source snapshot；本篇拥有species response与per-instance history |
| Physics/navigation | Runtime08A + Runtime08D | collision/query/nav owner；本篇提供LOD/instance change adapter与stable hit identity |
| Shadow/GI/velocity/RT | Runtime09E + Runtime09F3 + Runtime09H1 + Runtime28 | 消费同代deformed representation、bounds、history和fallback |
| Space/identity | Runtime23 + Runtime24 | unit/origin/precision与generation-qualified handle primitive |
| Vampire product evidence | App06 | clean-clone资产、真实运行/capture owner；本篇定义vegetation资格门 |

Vegetation domain必须统一species semantic、compiled representations、prototype identity、runtime instance state、wind response和representation transition。Terrain只拥有实例“放在哪里”的placement truth，Weather只拥有环境风场，Renderer只拥有通用提交机制；三者都不能反向拥有species或实例生命周期。

## 7. P1：Species Source、Schema、Import 与 Compiler

| ID | 差距 | 必须重构 |
|---|---|---|
| VEG-P1-001 | 无Vegetation资源身份 | 新增SpeciesSource、BuildArtifact、Prototype、InstanceSet、ImpostorAtlas、Wind/Interaction Profile kind与versioned handle |
| VEG-P1-002 | 无独立species source | 建`VegetationSpeciesSource`，分离source mesh/metadata/authoring settings与cooked runtime artifact |
| VEG-P1-003 | 无part/semantic schema | trunk、branch、twig、frond、leaf、grass blade、card、billboard及material section有typed identity |
| VEG-P1-004 | 无植被vertex attribute合同 | branch hierarchy、anchor、edge、wind weight/phase、leaf facing与bend mask有显式format/reflection |
| VEG-P1-005 | glTF/import缺扩展与provenance | 支持批准extension/sidecar、source generator/version/unit/axis/hash并拒绝未知关键semantic |
| VEG-P1-006 | 无source validation | topology、degenerate、normal/tangent、UV、alpha、pivot、scale、bounds和part cardinality产生结构化diagnostic |
| VEG-P1-007 | 无stable species/part ID | reimport、LOD rebuild、material替换、cell reload与network中保持semantic identity和generation |
| VEG-P1-008 | 无deterministic compiler | 相同source/settings/toolchain/target生成稳定representation ordering、digest与artifact |
| VEG-P1-009 | 无LOD generation/error metric | mesh simplification保护silhouette/leaf density/branch topology并记录screen-space error |
| VEG-P1-010 | 无card/billboard/impostor bake | 生成多视角color/normal/depth/opacity、camera basis、bounds、atlas padding与quality error |
| VEG-P1-011 | 无wind/collision metadata cook | branch hierarchy、stiffness、anchor、interaction envelope、trunk/canopy query shape随artifact编译 |
| VEG-P1-012 | 无migration/DDC/LKG/receipt | schema/artifact独立version，记录dependency/compiler/settings/target/hash/cost/warning/fallback |

## 8. P1：Prototype、Instance Set、Cell、Identity 与 Lifecycle

| ID | 差距 | 必须重构 |
|---|---|---|
| VEG-P1-013 | 无VegetationPrototype | 聚合artifact、materials、density/scale/align、LOD、wind、collision、lighting与scalability policy |
| VEG-P1-014 | 无runtime instance-set component | Scene持prototype/placement artifact、cell owner、runtime policy，不展开为海量ordinary entity |
| VEG-P1-015 | 无stable instance handle | semantic placement ID与slot/generation分离，remove/reuse/rebuild/stale访问可检测 |
| VEG-P1-016 | 无cluster/cell identity | world/cell/prototype/cluster generation进入CPU/GPU/stream/query/event所有句柄 |
| VEG-P1-017 | 无SoA instance storage | transform、prototype、random、phase、color、season、bend、flags、LOD state按访问模式布局 |
| VEG-P1-018 | 无批量mutation合同 | add/remove/update/range replace/visibility/density change有transaction、dirty range和receipt |
| VEG-P1-019 | 无cluster tree build/refit | 层级bounds、leaf capacity、Morton/spatial ordering、async build、incremental refit与rebuild阈值明确 |
| VEG-P1-020 | 无atomic instance publication | placement、prototype、buffers、cluster tree与collision全成后一次install，禁止半cell可见 |
| VEG-P1-021 | 无严格生命周期 | Requested/Building/Resident/Active/Suspended/Retiring/Failed/Cancelled每ticket唯一终态 |
| VEG-P1-022 | 无async generation fencing | build/upload/collision task携world/cell/prototype/device generation，旧完成不能污染新实例 |
| VEG-P1-023 | 无reload/reimport迁移 | stable placement mapping、runtime bend/season保留条件、LKG与不可迁移理由明确 |
| VEG-P1-024 | 无multi-world/unload drain | preview/PIE/game隔离mutable state，world/cell unload先停work再退GPU/physics/nav资源 |

## 9. P1：Cluster、LOD、Billboard、Impostor、Visibility 与 Streaming

| ID | 差距 | 必须重构 |
|---|---|---|
| VEG-P1-025 | static batch DTO无提交consumer | 通用问题引用Runtime09B P0-4；vegetation adapter必须证明instance span、draw/work和visible count闭环 |
| VEG-P1-026 | 无per-instance/cluster LOD | 用projected error、screen coverage、importance和cluster coherence选择representation，不按entity origin |
| VEG-P1-027 | 无LOD hysteresis/crossfade | 进入/退出阈值、dither/fade、alpha coverage、history与overdraw预算类型化 |
| VEG-P1-028 | 无mesh/card/billboard/impostor统一映射 | 同一prototype/instance跨表示保持identity、material、lighting、wind phase与query语义 |
| VEG-P1-029 | 无camera-facing billboard合同 | axial/spherical basis、vertical lock、view family/stereo、shadow view和near-pole稳定性明确 |
| VEG-P1-030 | 无impostor parallax/depth | view selection、depth reconstruction、normal/tangent、self-shadow、mip/padding与disocclusion处理缺失 |
| VEG-P1-031 | 无真实mesh bounds | local bounds变换、偏心树冠、nonuniform scale与representation bounds不能用translation+scale球替代 |
| VEG-P1-032 | 无wind-deformed conservative bounds | species envelope、gust/bend极值、dynamic expansion与tightening policy进入visibility/shadow/RT |
| VEG-P1-033 | 无层级visibility/occlusion | cell/cluster/instance/representation四级frustum/HZB，occlusion latency与fast camera fallback明确 |
| VEG-P1-034 | 无visible compaction/indirect work | 按prototype/LOD/material/pass压缩visible instances并生成有界indirect args及overflow receipt |
| VEG-P1-035 | 无cell artifact streaming | prototype dependency、instance pages、impostor atlas、collision/nav按bundle预取/安装/退役 |
| VEG-P1-036 | 无residency与LOD联动 | memory/IO压力选择representation/density且保持关键实例、collision和gameplay边界，禁止静默消失 |

## 10. P1：Wind、Deformation History、Interaction 与 Simulation

| ID | 差距 | 必须重构 |
|---|---|---|
| VEG-P1-037 | 无WindField消费合同 | 接收world/time/generation-qualified direction/strength/gust/turbulence/volume snapshot，不直读Editor状态 |
| VEG-P1-038 | 无species wind response | trunk/branch/frond/leaf stiffness、frequency、anchor、adherence、drag与quality tier成为artifact/profile |
| VEG-P1-039 | 无per-instance variation | stable seed派生phase/amplitude/frequency/orientation，跨reload/network/capture可复算 |
| VEG-P1-040 | 无branch hierarchy deformation | global sway、branch bending/twitch/whip按parent/anchor传播并限制长度/能量/NaN |
| VEG-P1-041 | 无leaf/frond motion | leaf ripple/tumble/twitch、frond ripple与camera-facing在统一space/order下组合 |
| VEG-P1-042 | 无gust/turbulence时间模型 | fixed/variable time domain、spatial sampling、filter、loop/replay和large-world precision明确 |
| VEG-P1-043 | 无vertex deformation hook | Runtime09C提供pass-shared WPO接口；vegetation实现typed attributes、module specialization和qualification |
| VEG-P1-044 | 无current/previous deformation | 保存前后wind table、instance state与representation mapping，velocity/TAA消费同一tick generation |
| VEG-P1-045 | 无history reset语义 | spawn、teleport、cell load、LOD/representation switch、wind source jump与replay seek分别定义reset |
| VEG-P1-046 | 无interaction/bending field | character/projectile/explosion提交typed capsule/impulse/volume，含owner、lifetime、budget与falloff |
| VEG-P1-047 | 无persistent recovery state | bend/flatten/break/recover按species policy更新，gameplay-authoritative与cosmetic边界明确 |
| VEG-P1-048 | 无deformation overload治理 | interaction count、wind quality、update rate、distance和GPU work降级可观察且不破坏collision truth |

## 11. P1：Leaf Material、Pass Parity 与 Cross-System Integration

| ID | 差距 | 必须重构 |
|---|---|---|
| VEG-P1-049 | 无thin-leaf/grass shading model | 定义two-sided normal、transmission/back-light、subsurface、roughness与energy conservation参考 |
| VEG-P1-050 | alpha policy不工程化 | cutout、alpha-to-coverage、dither、mip coverage preservation、MSAA与fallback跨平台一致 |
| VEG-P1-051 | 无main/depth/shadow deformation parity | 所有pass调用同一qualified vertex deformation并共享alpha/material/representation generation |
| VEG-P1-052 | 无velocity/reactive-mask parity | WPO、billboard facing、LOD fade和impostor view change产生正确motion/reactive/disocclusion数据 |
| VEG-P1-053 | 无shadow vegetation policy | caster LOD、WPO distance、two-sided/cutout、cluster bounds、cache invalidation与budget明确 |
| VEG-P1-054 | 无GI/lightmap/RTVT adapter | static/dynamic vegetation、two-sided GI、lightmap instance、probe/virtual texture write与wind边界明确 |
| VEG-P1-055 | 无RT vegetation adapter | alpha/two-sided、instance span、BLAS/TLAS、wind deformation/refit、LOD与fallback引用Runtime28闭环 |
| VEG-P1-056 | 无physics collision LOD | trunk/canopy/query/interaction形状、instance mutation、cell unload和render LOD映射有receipt |
| VEG-P1-057 | 无navigation adapter | obstacle/area、walkable canopy policy、batch dirty region、streaming与runtime bend/break更新明确 |
| VEG-P1-058 | 无picking/query identity | hit返回world/cell/prototype/instance/part/material stable identity与generation，不只返回batch entity |
| VEG-P1-059 | 无season/state adapter | color/leaf density/representation/collision变化原子提交并与save/network/content authority分层 |
| VEG-P1-060 | 无audio/VFX/gameplay事件 | rustle/contact/bend/break以有界typed event输出，不能由consumer遍历或改写instance内部状态 |

## 12. P1：Scalability、Diagnostics、Tests 与 Product Qualification

| ID | 差距 | 必须重构 |
|---|---|---|
| VEG-P1-061 | 无联合scalability模型 | density、LOD、wind、shadow、interaction、RT和streaming由同一quality/budget决策且记录结果 |
| VEG-P1-062 | 无global admission budget | instances/clusters/draws/triangles/overdraw/bytes/updates/CPU/GPU/IO决定接受、降级或拒绝 |
| VEG-P1-063 | 无platform capability matrix | storage/indirect/compute/MSAA/RT/texture限制映射Supported/Degraded/Unsupported与approved fallback |
| VEG-P1-064 | 无runtime diagnostics | 展示species/prototype/cell/cluster/instance、LOD、cull reason、wind、memory、draw/work和fallback |
| VEG-P1-065 | 无artifact/compiler tests | semantic import、determinism、LOD error、impostor bake、wind/collision cook、migration golden为空 |
| VEG-P1-066 | 无instance lifecycle tests | batch mutation、stable ID、cluster build/refit、cancel/reload/unload、stale generation和OOM为空 |
| VEG-P1-067 | 无render differential | mesh/card/billboard/impostor、alpha/depth/shadow/velocity/GI/RT与reference image/error门为空 |
| VEG-P1-068 | 无wind/interaction tests | current/history、phase determinism、gust、branch/leaf、bend/recovery、NaN和replay differential为空 |
| VEG-P1-069 | 无规模与故障矩阵 | 1K/100K/1M实例、fast camera、device loss、provider fault、atlas corruption和stream churn为空 |
| VEG-P1-070 | Vampire证据不闭合 | 引用App06修复clean-clone资产与可执行链，再用真实capture证明draw/LOD/wind/shadow而非DTO |
| VEG-P1-071 | 无工程级产品场景 | 建forest/grassland/dense understory/season/storm/interaction/world-cell穿越的save/play/export/capture链 |
| VEG-P1-072 | 无跨引擎超越基准 | 同资产/视角/硬件/画质比较CPU/GPU、memory、IO、stutter、overdraw、LOD error和raw receipts |

## 13. P2：完整性与长期竞争力

| ID | 延后项 | 前置条件 |
|---|---|---|
| VEG-P2-001 | procedural species synthesis | source/compiler schema、stable semantic ID、determinism与quality oracle完成 |
| VEG-P2-002 | runtime growth/aging | topology/identity migration、season/state、save/network与collision/nav事务完成 |
| VEG-P2-003 | branch break/destruction coupling | Destruction33、piece identity、wind/interaction、render/physics output闭环完成 |
| VEG-P2-004 | fire/burn/charring | material state、VFX/weather/gameplay authority、RT/GI与replication完成 |
| VEG-P2-005 | snow/wetness accumulation | Weather owner、surface state、deformation load、material/GI和streaming完成 |
| VEG-P2-006 | biome/ecosystem succession | Terrain29 placement、species lifecycle、deterministic simulation与large-world persistence完成 |
| VEG-P2-007 | GPU procedural scatter | CPU placement oracle、stable IDs、cell transaction、readback/fault和portable fallback完成 |
| VEG-P2-008 | GPU branch/leaf simulation | CPU/reference oracle、history、bounds、collision boundary与device fault isolation完成 |
| VEG-P2-009 | neural impostor/radiance representation | deterministic fallback、training provenance、view/light error bound和cross-platform qualification完成 |
| VEG-P2-010 | virtualized geometry vegetation | Runtime09B/09D、alpha/leaf cluster、wind deformation与representation parity完成 |
| VEG-P2-011 | spectral/translucent foliage optics | thin-leaf baseline、measured material dataset、path/reference render与energy gate完成 |
| VEG-P2-012 | multiplayer authoritative vegetation state | stable IDs、interest management、prediction/rollback、late join与bandwidth receipt完成 |
| VEG-P2-013 | large-world cross-cell organism | cell ownership、root/branch跨界、origin/rebase、stream continuity和atomic migration完成 |
| VEG-P2-014 | third-party species/provider SDK | ABI/version/capability/budget/sandbox/unload与artifact compatibility完成 |
| VEG-P2-015 | collaborative vegetation authoring | stable semantic IDs、transaction/merge/locking/recovery与source provenance完成 |
| VEG-P2-016 | distributed visual/performance farm | frozen BuildSet、asset/license、capture、image/perf raw receipt与promotion governance完成 |

## 14. 分层重构里程碑

### M0 · Truth、Owner 与 Baseline

冻结BuildSet、owner矩阵、existing P0引用、342项语料、Vampire现状和同资产跨引擎基准；catalog只允许报告普通Mesh/Terrain能力。建立`VegetationCapabilitySnapshot`和unsupported reason，禁止从`Tree`枚举或static-batch DTO推导Ready。

### M1 · Species Source、Schema 与 Compiler

完成VEG-P1-001..012：source/import semantic、stable IDs、deterministic LOD/card/impostor/wind/collision artifact、migration/DDC/LKG与raw build receipt。Compiler作为Runtime/Editor唯一authoritative实现。

### M2 · Prototype、Instance Set 与 Lifecycle

完成VEG-P1-013..024：prototype、SoA instance storage、generation handles、cell/cluster、batch mutation、atomic publication、reload与multi-world/unload。先以CPU/reference backend证明状态机和identity。

### M3 · GPU Instance、Cluster、LOD 与 Streaming

完成VEG-P1-025..036并依赖Runtime09B/09D：真正消费instance span，建立cluster visibility、per-instance LOD、representation transition、visible compaction/indirect和cell residency。每条路径必须有work/count/overflow receipt。

### M4 · Wind、Deformation 与 Interaction

完成VEG-P1-037..048并消费qualified WindField：part-aware deformation、current/previous history、spawn/seek/LOD reset、interaction/bending/recovery和quality tier。CPU小样oracle与GPU differential必须同时存在。

### M5 · Leaf Shading 与所有Pass一致性

完成VEG-P1-049..055：thin-leaf optics、alpha coverage、main/depth/shadow/velocity/reactive/GI/RT parity。Mesh、card、billboard和impostor在批准误差范围内共享外观与风相位。

### M6 · Physics、Navigation 与 Gameplay Adapter

完成VEG-P1-056..060：collision/query/nav、picking identity、season/state和有界audio/VFX/gameplay事件。adapter消费唯一instance snapshot，禁止另建可变副本。

### M7 · Editor、Terrain 与 Product Integration

与Editor16、Terrain29、Editor38和App06闭环species import/authoring、scatter、wind preview、cell streaming、save/reopen/PIE/export。修复Vampire clean-clone资产和真实graphics consumer后再建立accepted captures。

### M8 · Reliability 与 Scalability

完成VEG-P1-061..071：联合预算、platform matrix、diagnostics、artifact/lifecycle/render/wind/fault tests和1M实例压力场景；验证cancel、OOM、device loss、atlas损坏与stream churn。

### M9 · 性能与表现超越门

完成VEG-P1-072：以相同species资产、数量、镜头、风、阴影、RT/GI、硬件、分辨率和warm-up对照Unreal/Unity；归档CPU/GPU timestamps、memory、IO、stutter、draw/work、overdraw、LOD/image error与raw captures。没有可复跑receipt不得声称优于虚幻。

## 15. 验收门

| Gate | 必须证明 |
|---|---|
| VEG-G01 | catalog在M0-M8未闭合前不把Tree/Foliage/Vegetation报告为Ready |
| VEG-G02 | species source/build/prototype/instance-set各有独立schema version与digest |
| VEG-G03 | 相同source/settings/toolchain/target重复build产生相同artifact digest |
| VEG-G04 | malformed semantic/topology/unit/alpha输入fail-close并给出结构化diagnostic |
| VEG-G05 | part/vertex semantic跨import、LOD、card/impostor与shader reflection一致 |
| VEG-G06 | LOD/card/impostor build有silhouette、coverage、normal/depth误差raw receipt |
| VEG-G07 | instance handle在remove/reuse/reload/world replace后拒绝stale generation |
| VEG-G08 | batch add/remove/update与cluster refit/rebuild不会泄漏、重复或错配instance |
| VEG-G09 | async build/upload的旧world/cell/device完成结果不能发布 |
| VEG-G10 | cell/prototype bundle只在所有required资源成功后原子install |
| VEG-G11 | GPU Scene实际注册多实例span，draw/work reduction由capture证明 |
| VEG-G12 | cluster/instance culling与CPU oracle在批准误差内一致 |
| VEG-G13 | LOD基于projected error并有hysteresis，fast camera不持续振荡 |
| VEG-G14 | mesh/card/billboard/impostor切换保持stable identity与批准图像误差 |
| VEG-G15 | billboard在stereo、shadow view、极角与vertical lock场景稳定 |
| VEG-G16 | impostor color/normal/depth/opacity/mip边界无明显atlas bleeding |
| VEG-G17 | 偏心树冠/nonuniform scale/高树bounds不被translation-radius近似错误裁剪 |
| VEG-G18 | 最大批准wind/bend下main/shadow/RT conservative bounds不欠界 |
| VEG-G19 | visible compaction/indirect overflow有fail-close或批准降级receipt |
| VEG-G20 | cell streaming压力下关键collision/gameplay实例不静默消失 |
| VEG-G21 | WindField snapshot绑定world/time/generation且旧weather结果不污染新world |
| VEG-G22 | per-instance wind variation由stable seed复算，reload/replay结果一致 |
| VEG-G23 | branch/leaf/frond deformation不产生NaN、长度爆炸或非法bounds |
| VEG-G24 | current/previous deformation驱动velocity，与reference motion在误差内一致 |
| VEG-G25 | spawn/teleport/LOD switch/cell load/seek分别执行批准history reset |
| VEG-G26 | interaction field有owner/lifetime/budget，过载时降级可观察 |
| VEG-G27 | thin-leaf two-sided/transmission模型通过reference render与energy测试 |
| VEG-G28 | cutout/A2C/dither的mip coverage和MSAA fallback跨平台合格 |
| VEG-G29 | main/depth/shadow使用同一deformation/alpha generation，轮廓一致 |
| VEG-G30 | WPO、billboard和LOD fade产生正确velocity/reactive/disocclusion数据 |
| VEG-G31 | caster LOD/WPO distance变化不会产生未记录shadow popping |
| VEG-G32 | GI/lightmap/RT adapter消费同代representation并报告fallback |
| VEG-G33 | vegetation hit返回world/cell/prototype/instance/part stable identity |
| VEG-G34 | render/collision/nav mutation与cell generation一致且可排空 |
| VEG-G35 | density/LOD/wind/shadow/RT联合预算不破坏关键gameplay policy |
| VEG-G36 | 1K/100K/1M场景记录CPU/GPU/memory/IO/stutter/draw/work raw receipts |
| VEG-G37 | cancel/OOM/device loss/provider fault/atlas corruption/stream churn矩阵通过 |
| VEG-G38 | Vampire clean clone、运行、save/reopen、PIE/export与capture链闭合 |
| VEG-G39 | accepted图像证明grass/tree的LOD、wind、alpha、shadow、velocity而非DTO存在 |
| VEG-G40 | 超越Unreal/Unity的结论绑定同资产/场景/硬件/画质和可复跑raw receipts |

## 16. Finding 到里程碑映射

| Finding | 里程碑 |
|---|---|
| VEG-P1-001..012 | M0-M1 |
| VEG-P1-013..024 | M2 |
| VEG-P1-025..036 | M3 |
| VEG-P1-037..048 | M4 |
| VEG-P1-049..060 | M5-M6 |
| VEG-P1-061..072 | M7-M9 |
| VEG-P2-001..016 | 对应P1与验收门完成后独立立项，不得提前并入MVP |

## 17. 禁止的临时修补

1. 禁止只增加`Vegetation`/`Tree`枚举、空feature或catalog metadata便宣称支持。
2. 禁止把预合并mesh、`static_batches` DTO或共享material数量当作真实GPU instancing证据。
3. 禁止为每棵树/每簇草创建ordinary Scene entity来绕过instance-set、cluster和cell生命周期。
4. 禁止继续固定`register(..., 1)`后在README声称batch/indirect draw已经可用。
5. 禁止用entity-origin distance和单一`min_distance`冒充per-instance screen-error LOD。
6. 禁止运行时由脚本手换mesh/card/billboard并丢失identity、wind phase、shadow或history。
7. 禁止把wind实现为未版本化的global sine或把branch/leaf数据暗塞进任意UV/color通道。
8. 禁止只在main pass形变；depth、shadow、velocity、GI和RT必须共享qualified deformation。
9. 禁止把`double_sided=true`或generic subsurface参数包装成thin-leaf shading完成。
10. 禁止用永久放大的world bounds掩盖wind bounds错误并接受严重occlusion/RT性能退化。
11. 禁止由Terrain、Weather、Renderer、Editor各维护一份prototype/instance/wind mutable truth。
12. 禁止以测试构造DTO、静态shader marker、单帧截图或0退出码替代产品资格。

## 18. 实施前重查清单

1. 重导342项冻结manifest并重算composite SHA-256；记录新增、删除、修改和working-tree来源。
2. 重查Runtime29的TER-P1-045..052、Editor16的5项P0、Runtime09B P0-4及App06 P1-046，禁止重复owner。
3. 重查`GeometryExtract::static_batches`是否已有production consumer及`gpu_scene.register`是否仍固定count=1。
4. 重查`Tree/Billboard/Terrain/MeshLod` feature是否已有真实pass/executor/registration与qualification receipt。
5. 重查shader模板是否新增vertex deformation hook及所有depth/shadow/velocity模板是否同代消费。
6. 重查Weather runtime owner是否已提供generation-qualified WindField；没有时保持显式dependency，不私建global风。
7. 重查Vampire ignored model、material alpha/wind、README与accepted capture状态，按App06 clean-clone边界处理。
8. 锁定M0-M9每阶段BuildSet、target、backend、hardware、quality、warm-up、raw receipt位置与promotion规则。

## 19. 本轮产出边界

本篇只完成静态审查、参考对照、唯一owner划分、差距登记、重构里程碑和验收门，没有修改production代码、Cargo、测试、workflow或产品资产，没有运行构建、测试或GPU capture，也没有证明任何植被功能、性能或表现已完成。后续实施必须先复核source currentness和既有owner，再按M0-M9底层依赖推进；在VEG-G01..G40全部有可复跑证据前，不得把普通Mesh、Terrain scatter配置、static-batch DTO、Tree feature槽位或Vampire预合并草模型宣传为工程级Vegetation runtime。
