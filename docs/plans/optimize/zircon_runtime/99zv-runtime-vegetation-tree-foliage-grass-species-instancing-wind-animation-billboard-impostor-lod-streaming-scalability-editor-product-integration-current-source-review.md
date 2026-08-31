---
title: Runtime Vegetation、Tree、Foliage、Grass、Species、Instancing、Wind、Billboard、Impostor、LOD、Streaming、Scalability、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime147
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
historical_refresh_of: Runtime34
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_plugins/gltf_importer/runtime
  - zircon_plugins/terrain
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - examples/vampire/assets/materials/forest_grass_billboard.zmaterial
  - examples/vampire/assets/models/grass_billboard_static_batch.model.toml
plan_sources:
  - docs/plans/optimize/zircon_runtime/34-vegetation-tree-foliage-grass-species-instancing-wind-animation-billboard-impostor-lod-streaming-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Foliage
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/InstancedStaticMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/HierarchicalInstancedStaticMesh.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeGrass.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Classes/LandscapeGrassType.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SpeedTreeWind.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling
  - dev/UnrealEngine/Engine/Source/Editor/FoliageEdit
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Shaders/Nature
  - dev/Graphics/Packages/com.unity.shadergraph
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Nature
  - dev/godot/scene/resources/multimesh.cpp
  - dev/godot/scene/3d/multimesh_instance_3d.cpp
  - dev/godot/servers/rendering
  - dev/bevy/crates/bevy_render
  - dev/bevy/crates/bevy_pbr
  - dev/Fyrox/fyrox-graphics
  - dev/Fyrox/fyrox-impl/src/scene
---

# Runtime Vegetation、Tree、Foliage、Grass 与 Product Integration 当前源码工程化差距

## 1. 结论

当前 Zircon 没有 Vegetation、Tree、Foliage、Grass 或 SpeedTree 运行时产品。对 `zircon_runtime`、`zircon_runtime_interface`、`zircon_editor`、`zircon_app` 与 `zircon_plugins` 中排除 `tests/test_sources/benches/target` 及 test-named 文件后的 **12,065 个 production Rust 文件**执行精确领域扫描，`VegetationSpecies/VegetationAsset/FoliageAsset/FoliagePrototype/FoliageRuntimeCluster/GrassAsset/SpeedTree/Impostor/WindDirectionalSource/WindField/FoliageShading` 为 **0 个命中文件、0 条命中**。少量 `foliage` 只来自普通 shader prewarm 参数，`weather.Component.Wind` 只在 `#[cfg(test)]` 中出现，不能形成领域 owner。

仓内真实存在 typed Mesh/glTF import、普通 Mesh LOD、GPU Scene contiguous span/current-previous transform、frustum/HZB/indirect、alpha mask/double-sided material、shadow/velocity/GI/RT、artifact cache/residency、quality profile和diagnostics等通用前置。但 ordinary mesh consumer 仍对每个 draw 调用 `gpu_scene.register(..., 1)`；primitive bounds以entity translation和最大transform列长近似；`Tree`、`Billboard`、`Terrain`、`MeshLod`仍被明确列在 `DESCRIPTOR_ONLY_ADVANCED_SLOTS`，没有runtime pass。不存在species source/compiler、prototype、instance set、cluster tree、screen-error LOD、card/impostor、WindField、part-aware deformation、thin-leaf shading、跨pass history、streaming bundle或typed adapter receipt。

Foliage Editor静态工作台把 `Forest_A12` 标为 `Ready`，显示 `84K instances`，点击preview/build后硬编码返回 `Preview queued 84K instances` 与 `Build queued 128 clusters`，但Runtime、catalog、artifact、job、cluster或receipt均不存在。该问题已经由Editor16 P0-4精确拥有，本文 **不重复新增Vegetation-owned P0**，将其作为1项继承阻断重新确认；历史72项P1按当前working bytes重判为 **46 Open / 26 Partial / 0 Closed**，16项P2全部Open；40项资格门为 **31 Fail / 9 Partial / 0 Pass**。Partial只表示共享owner可复用，不能解释为植被产品链已启动。目标必须硬切到：

```text
VegetationSpeciesSource + import provenance
  -> deterministic VegetationCompiler
  -> VegetationBuildArtifact
     (parts/attributes/mesh LOD/cards/impostor/wind/collision/bounds)
  -> VegetationPrototype + PlacementArtifact
  -> generation-qualified Cell/Cluster/InstanceSet
  -> WindField + SpeciesResponse + InteractionState
  -> GPU Instance/Cluster/LOD/Visibility/Streaming
  -> Raster/Shadow/Velocity/GI/RT/Physics/Nav typed adapters
  -> runtime-backed Editor authoring、preview、build 与 qualification receipts
```

## 2. 审查边界、方法与 currentness

### 2.1 冻结范围

本文记录读取时 `main@1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8` 的 selected working bytes。共享工作树在最终冻结附近有 **3,298 个 tracked changes、2,115 个 untracked paths**；扫描期间其他Session持续推进。本文不归因、不覆盖、不回退任何既有改动，实施前必须重取指纹并执行source recheck。用户明确暂不优化tooling，本轮没有扫描或规划未来将迁移到Rust的tooling实现。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树指纹 |
|---|---:|---|
| Zircon Source、Scene、Import 与 Editor carrier | **1,745 / 248,118 / 226,299 / 8,821,014 / 2,296 / 77** | `3abb9f9766a96641282e7b4a54bd809de904b553d18f790359a2c9d560680e8a` |
| Zircon Render、GPU Scene 与 Material前置 | **257 / 53,380 / 49,196 / 1,996,412 / 499 / 2** | `ac067b18b5d22e29e90127c95ec61a2b2dc835d163702c4246df0d2e7748b96e` |
| Zircon Catalog、Terrain、App 与 Vampire产品证据 | **227 / 38,101 / 35,075 / 1,375,690 / 436 / 1** | `fd7ce12b5ccf4244ef0aa74c1ce0d14a83932717eaf20e2683060c90f5576abb` |
| Unreal Foliage、ISM/HISM、Landscape Grass、SpeedTree与Editor | **108 / 42,342 / 35,512 / 1,558,207 / 0 / 0** | `de818639c7191823d5017fc1d7e52ed0f21a60217419cd4217a1b7c839333a35` |
| Unity GPUDriven、URP/HDRP Nature与SpeedTree | **82 / 19,949 / 16,650 / 879,294 / 0 / 0** | `146b43d8a0bed4fe67867e2abb4c98d68b6736dabde4f7e9d27d9a779141cd1a` |
| Godot MultiMesh与RenderingServer | **294 / 166,648 / 137,620 / 6,659,080 / 0 / 0** | `324dd750d08ed3a00ff3819c11bc9ab91470bff3d2ff41df53105019e60a1bbc` |
| Bevy/Fyrox通用GPU batching、visibility与LOD对照 | **318 / 143,098 / 129,602 / 5,499,682 / 130 / 0** | `85bf4fd6b08438eaf1f67d058b772a02885ee2aaf581bfd173b10379c20c1618` |

指纹算法为：repository-relative path转`/`并小写排序；每个文件取当前bytes的lowercase SHA-256；聚合输入为按行连接且末尾无LF的`path|file_sha256` UTF-8 payload，再取SHA-256。各选择集是证据边界，不应相加成仓库唯一文件数；tests/ignored由统一静态宏检测统计。

### 2.2 纵向扫描链

本轮逐层核对 ResourceKind -> source/import/schema/provenance -> compiler/artifact/DDC -> prototype/placement -> Scene roundtrip -> instance/cell/cluster identity -> mutation/lifecycle -> GPU Scene/bounds/visibility/LOD/indirect -> wind/deformation/history -> leaf material/pass parity -> streaming/scalability -> physics/nav/query/gameplay adapters -> catalog/App/Editor/tests/product证据。PowerShell扫描覆盖tracked与untracked working bytes；当前 `rg.exe` 受本机Windows Store执行权限阻断，精确复查改用`git grep`与PowerShell `Select-String`，未因此缩小源码边界。

### 2.3 证据等级与执行限制

本文达到E3 source-level review。没有运行Cargo、WGPU、App、Editor、PIE、asset cook、roundtrip、GPU capture、fault/fuzz、scale/soak或竞争benchmark；当前没有Vegetation source/component/compiler/provider/pass可形成有意义的端到端动态证据，运行普通Mesh/GPU Scene测试不能提高领域完成度。实施必须先取得capability truth、source roundtrip、deterministic compiler和CPU oracle的RED证据。

## 3. 当前产品链事实

### 3.1 Resource、Source、Import 与 Scene

1. `ResourceKind`只有Data、Model、Mesh、Material、Texture、Scene、Terrain、NavMesh、Animation等26种既有资源，没有SpeciesSource、BuildArtifact、Prototype、InstanceSet、ImpostorAtlas、WindProfile或InteractionProfile；Editor asset registry也无植被类型。
2. `SceneEntityAsset`能持ordinary mesh、light、physics、animation、terrain、tilemap、prefab和script，没有vegetation component、prototype/placement artifact、cell owner、density或runtime policy。
3. glTF importer真实支持mesh/material/morph/skin和通用PBR扩展，没有branch hierarchy、leaf anchor、frond edge、wind weight/phase、species metadata或SpeedTree extension；普通provenance/validation可复用但不构成植被import。
4. `MeshRendererLodLevel`只保存`min_distance`、model、mesh、material与primitives；`mesh_lod_for_camera`按entity translation到camera的距离选择最大已达门槛，没有projected error、screen coverage、per-instance choice、hysteresis或crossfade。
5. terrain插件中没有scatter、biome、density、slope、foliage、grass或vegetation生产合同；首方runtime/editor catalog和App entry也没有Vegetation插件或capability。

### 3.2 GPU Scene、Bounds、Visibility 与 Representation

1. GPU Scene可为generic stable key分配contiguous instance span，`GpuInstanceData`固定176 bytes并保存current/previous matrix、primitive、flags、payload/morph slot和lightmap参数；没有prototype/species、stable random、wind phase/weight、bend、LOD或representation state。
2. ordinary mesh draw同步仍以`instance_count = 1`注册并写一个实例，没有instance-set producer、batch mutation、cluster rebuild、visible compaction或vegetation indirect work。
3. 当前primitive sphere center直接取world matrix translation，radius取最大basis列长，不使用真实mesh local bounds，更没有wind-deformed conservative bounds；高树、偏心树冠和nonuniform scale会产生欠界风险。
4. generic frustum、HZB、indirect和residency是真实共享前置，但没有cell/cluster/instance四级visibility、per-instance LOD、representation mapping、overflow receipt或CPU oracle。
5. `Tree`、`Billboard`、`Terrain`、`MeshLod`属于显式descriptor-only advanced slots，测试还断言没有runtime passes。这是诚实Unsupported证据，不是隐藏实现。

### 3.3 Wind、Material、Pass 与 Cross-system

1. production shader没有Vegetation、SpeedTree、wind、branch、leaf、frond或impostor deformation；通用形变只覆盖skin、Morph和Mesh SDF，没有pass-shared vertex deformation/WPO hook。
2. production没有WindSource/WindField/gust/turbulence/per-instance phase/current-history wind table或quality tier。Weather测试字符串不能成为runtime环境风owner。
3. generic alpha mask、double-sided、shadow、velocity、reactive、transmission/subsurface、lightmap/GI/RT可复用；没有thin-leaf optics、mip coverage、billboard facing、LOD fade或同代deformation parity。
4. physics、navigation、query、audio、VFX、network与save没有vegetation adapter，query也不能返回world/cell/prototype/instance/part/material stable identity。

### 3.4 Editor、Examples 与 Product Truth

1. Foliage Editor `.zui`静态列出Paint/Erase/Clusters、`FOL_Forest`、`Oak_Tall`、`Forest_A12 Ready`、`River_02 Grass Queued`和`84K instances 2 warnings`，没有runtime model绑定。
2. callback对preview/build直接返回常量`Preview queued 84K instances density 0.72`和`Build queued 128 clusters 2 warnings`，未创建job、artifact、cluster、diagnostic或terminal receipt。
3. Vampire的`grass_billboard_static_batch.model.toml`是预合并cross-card普通mesh，material为opaque double-sided；没有camera-facing、cutout、wind、LOD、impostor或真实GPU instance product证据。
4. examples/WOC中的bush、fern、oak等普通GLB只证明内容存在，不证明引擎具备species、scatter、runtime instancing或风系统。

## 4. 参考实现给出的工程边界

### 4.1 Unreal：Species policy、Instance storage与Editor transaction必须分层

`UFoliageType`把density/radius、scale、alignment、slope/height/layer filters、collision、cull、lighting/shadow、WPO disable、RVT/HLOD与procedural约束归入type；`FFoliageInfo`/ISM/HISM维护稳定实例、spatial hash、cluster tree、async build、selection、bulk mutation、physics/nav/lightmap和per-instance custom data。FoliageEdMode以真实brush trace、geometry/layer filter、spatial rejection、transaction/undo和add/remove驱动world数据。Zircon应吸收“species policy、placement、runtime storage、render submission、editor transaction各自有owner”，不能复制UObject表面形态。

### 4.2 Unreal/Unity：GPU-driven LOD与风必须保存历史

Unreal Instance Culling把GPU Scene instance runs转为culling、compaction和indirect args；SpeedTree wind覆盖global、branch、leaf、frond、rolling与gust。Unity GPUDriven维护archetype/component、allocation、transform/wind current-history、visible compaction、LODGroup screen-relative transition和occlusion；`SpeedTreeWindGPUDataUpdater`只更新可见树并初始化新可见历史。工程边界是generation-qualified dataflow、history/reset、bounded work和receipt，不是一条sine WPO或一个instance buffer。

### 4.3 Unity Nature：Representation与所有pass必须共享语义

URP/HDRP/ShaderGraph Nature实现branch/frond/leaf/facing-leaf、wind quality、billboard face/crossfade、LOD percentage/crossfade，并覆盖forward、depth、shadow、depth normals与temporal所需数据。Mesh、card、billboard、impostor必须由同一artifact产生并保持instance identity、material、wind phase、bounds和lighting语义。

### 4.4 Godot、Bevy、Fyrox的适用边界

Godot `MultiMesh`提供transform/color/custom data、visible count、bulk buffer、current/previous interpolation和custom AABB，是generic multi-instance最低合同；Editor surface population用面积加权采样、rotation/tilt/scale variation。Bevy提供GPU preprocessing、indirect、late occlusion、current/previous instance input与meshlet visibility；Fyrox提供generic geometry instancing、LOD和visibility。三者本地树都没有完整species/wind/impostor产品，负证据不能降低Zircon目标。

## 5. 目标架构与唯一 Owner

| 领域 | 唯一owner | 本篇只消费/提供 |
|---|---|---|
| Resource/schema/artifact/DDC | Runtime04/Asset owner | 新资源kind、dependency、install/retire receipt |
| Scene/ECS/world lifecycle | Runtime05 | instance-set component、world generation、roundtrip与teardown primitive |
| Vegetation domain | 新Runtime Vegetation owner | species/compiler/prototype、instance/cluster、representation、wind response、interaction truth |
| Terrain placement/world cells | Runtime142/Terrain owner | 只提供placement/cell truth；消费Vegetation prototype，不拥有species/runtime实例 |
| Generic GPU Scene/visibility | Runtime94 | span、bounds、culling、compaction、indirect、overflow primitive |
| Material/shader/PSO | Runtime91 | deformation hook、thin-leaf model、pass variants与qualification primitive |
| Residency/streaming | Runtime09D/Asset owner | artifact/cell bundle admission、atomic install、pressure/retire policy |
| Wind/weather authority | Weather runtime owner | generation-qualified WindField；Vegetation拥有species response和instance history |
| Physics/navigation/query | Runtime Physics/Nav owner | collision/query/nav执行；Vegetation提供stable identity和mutation adapter |
| Foliage authoring | Editor16 | 只消费Runtime compiler/preview/job/receipt，禁止静态伪结果 |
| Vampire产品证据 | App06 | clean-clone运行/capture owner；本篇定义Vegetation资格门 |

## 6. 继承P0：必须先修复的产品真实性

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| Editor16 P0-4（引用） | Open | Foliage Editor在无runtime species/compiler/job/cluster/receipt时硬编码`Ready`、`84K instances`与`128 clusters`成功态；fixing owner保持Editor16，M0前必须隐藏/禁用并显示typed Unsupported reason，或接入真实Runtime job、artifact、diagnostic和唯一终态receipt |

## 7. P1：Species Source、Schema、Import 与 Compiler

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| VEG-P1-001 | Open | ResourceKind无SpeciesSource/BuildArtifact/Prototype/InstanceSet/Impostor/WindProfile；增加独立versioned kind与typed handle |
| VEG-P1-002 | Open | 无独立species source；分离source mesh/metadata/authoring settings与cooked runtime artifact |
| VEG-P1-003 | Open | 无trunk/branch/twig/frond/leaf/grass/card/billboard typed part identity与cardinality |
| VEG-P1-004 | Partial | generic Mesh attribute/reflection可复用；补branch hierarchy、anchor、edge、wind weight/phase、leaf facing和bend mask合同 |
| VEG-P1-005 | Partial | glTF普通import/provenance可复用；补批准extension/sidecar、generator/version/unit/axis/hash及未知关键semantic fail-close |
| VEG-P1-006 | Partial | 通用mesh validation可复用；补topology、pivot、scale、alpha、part cardinality与vegetation结构化diagnostic |
| VEG-P1-007 | Open | 无stable species/part ID；reimport、LOD、material、cell reload和network必须保持semantic identity/generation |
| VEG-P1-008 | Open | 无deterministic compiler；相同source/settings/toolchain/target须稳定生成ordering、digest和artifact |
| VEG-P1-009 | Partial | ordinary distance Mesh LOD可复用为载体；补保护silhouette/leaf density/branch topology的生成与screen error |
| VEG-P1-010 | Open | 无card/billboard/impostor bake；生成多视角color/normal/depth/opacity、basis、padding、bounds与error receipt |
| VEG-P1-011 | Open | 无wind/collision metadata cook；branch hierarchy、stiffness、anchor、interaction envelope和query shape随artifact编译 |
| VEG-P1-012 | Partial | generic artifact hash/version/cache/residency可复用；补独立schema/migration/LKG及dependency/cost/fallback receipt |

## 8. P1：Prototype、Instance Set、Cell、Identity 与 Lifecycle

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| VEG-P1-013 | Open | 无VegetationPrototype；聚合artifact、materials、density/scale/align、LOD、wind、collision、lighting和scalability policy |
| VEG-P1-014 | Open | Scene无runtime instance-set component；持prototype/placement artifact、cell owner和runtime policy，不展开海量ordinary entity |
| VEG-P1-015 | Open | 无stable instance handle；semantic placement ID与slot/generation分离，remove/reuse/rebuild/stale访问可检测 |
| VEG-P1-016 | Open | 无cluster/cell identity；world/cell/prototype/cluster generation进入CPU/GPU/stream/query/event所有句柄 |
| VEG-P1-017 | Open | 无SoA storage；transform、prototype、random、phase、color、season、bend、flags和LOD state按访问模式布局 |
| VEG-P1-018 | Open | 无batch mutation；add/remove/update/range replace/visibility/density change须有transaction、dirty range和receipt |
| VEG-P1-019 | Open | 无cluster tree build/refit；定义层级bounds、leaf capacity、ordering、async build、incremental refit和rebuild阈值 |
| VEG-P1-020 | Partial | generic artifact publication有原子性前置；placement/prototype/buffers/tree/collision完整后才可一次install |
| VEG-P1-021 | Partial | generic lifecycle可复用；补Requested/Building/Resident/Active/Suspended/Retiring/Failed/Cancelled唯一终态 |
| VEG-P1-022 | Partial | generic task generation/cancel可复用；build/upload携world/cell/prototype/device generation并拒绝旧完成 |
| VEG-P1-023 | Open | 无reload/reimport迁移；定义stable placement mapping、bend/season保留条件、LKG和不可迁移理由 |
| VEG-P1-024 | Partial | generic multi-world/drain前置存在；preview/PIE/game隔离mutable state，unload先停work再退GPU/physics/nav |

## 9. P1：Cluster、LOD、Billboard、Impostor、Visibility 与 Streaming

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| VEG-P1-025 | Partial | GPU Scene支持generic span但ordinary consumer固定count=1；建立instance-set producer并以capture证明work reduction |
| VEG-P1-026 | Open | 无per-instance/cluster LOD；用projected error、screen coverage、importance和cluster coherence选择representation |
| VEG-P1-027 | Open | 无hysteresis/crossfade；类型化进入/退出阈值、dither/fade、coverage、history和overdraw预算 |
| VEG-P1-028 | Open | 无mesh/card/billboard/impostor统一映射；跨表示保持identity、material、lighting、wind phase和query语义 |
| VEG-P1-029 | Open | 无camera-facing billboard合同；定义axial/spherical basis、vertical lock、stereo/shadow view和极角稳定性 |
| VEG-P1-030 | Open | 无impostor parallax/depth；补view selection、depth reconstruction、normal/tangent、self-shadow、mip/padding和disocclusion |
| VEG-P1-031 | Open | primitive bounds不是实际mesh bounds；按local bounds、偏心树冠和nonuniform scale计算representation bounds |
| VEG-P1-032 | Open | 无wind-deformed conservative bounds；species envelope、gust/bend极值、dynamic expansion与tightening进入visibility/shadow/RT |
| VEG-P1-033 | Partial | generic frustum/HZB可复用；补cell/cluster/instance/representation四级culling、latency和fast-camera fallback |
| VEG-P1-034 | Partial | generic indirect前置存在；按prototype/LOD/material/pass压缩visible instances并给bounded args/overflow receipt |
| VEG-P1-035 | Partial | generic chunk residency可复用；prototype dependency、instance pages、impostor atlas、collision/nav按bundle安装/退役 |
| VEG-P1-036 | Open | 无residency与LOD联动；memory/IO压力必须联合选择representation/density且保持关键collision/gameplay，禁止静默消失 |

## 10. P1：Wind、Deformation History、Interaction 与 Simulation

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| VEG-P1-037 | Open | 无WindField消费合同；接收world/time/generation-qualified direction/strength/gust/turbulence/volume snapshot |
| VEG-P1-038 | Open | 无species response；trunk/branch/frond/leaf stiffness、frequency、anchor、adherence、drag和quality tier进入artifact/profile |
| VEG-P1-039 | Open | 无per-instance variation；由stable seed派生phase/amplitude/frequency/orientation并可跨reload/network/capture复算 |
| VEG-P1-040 | Open | 无branch hierarchy deformation；global sway、branch bend/twitch/whip按parent/anchor传播并限制长度/能量/NaN |
| VEG-P1-041 | Open | 无leaf/frond motion；leaf ripple/tumble/twitch、frond ripple与camera-facing在统一space/order组合 |
| VEG-P1-042 | Open | 无gust/turbulence时间模型；明确fixed/variable domain、spatial sample、filter、loop/replay和large-world precision |
| VEG-P1-043 | Open | 无pass-shared vertex deformation hook；Material owner提供WPO接口，Vegetation提供typed attributes和qualified module |
| VEG-P1-044 | Partial | GPU Scene有generic current/previous transform；补wind table、instance/representation history与同tick generation |
| VEG-P1-045 | Open | 无history reset；spawn、teleport、cell load、LOD/representation switch、wind jump和replay seek分别定义 |
| VEG-P1-046 | Open | 无interaction field；character/projectile/explosion提交typed capsule/impulse/volume及owner/lifetime/budget/falloff |
| VEG-P1-047 | Open | 无persistent recovery；bend/flatten/break/recover按species policy更新并分离authoritative与cosmetic |
| VEG-P1-048 | Partial | generic quality profile可复用；interaction count、wind quality、update rate、distance和GPU work降级须可观察 |

## 11. P1：Leaf Material、Pass Parity 与 Cross-system Integration

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| VEG-P1-049 | Open | 无thin-leaf/grass shading；定义two-sided normal、transmission/back-light、subsurface、roughness和energy reference |
| VEG-P1-050 | Partial | generic alpha mask/double-sided存在；补A2C、dither、mip coverage preservation、MSAA和platform fallback |
| VEG-P1-051 | Open | 无main/depth/shadow deformation parity；所有pass调用同一qualified deformation/alpha/representation generation |
| VEG-P1-052 | Partial | generic velocity/reactive前置存在；WPO、billboard facing、LOD fade/impostor view change须正确输出motion/disocclusion |
| VEG-P1-053 | Partial | generic shadow可复用；补caster LOD、WPO distance、cutout、cluster bounds、cache invalidation和budget |
| VEG-P1-054 | Partial | generic lightmap/GI可复用；补static/dynamic vegetation、two-sided GI、instance lightmap/probe/RVT和wind边界 |
| VEG-P1-055 | Partial | generic RT/VG前置存在；补alpha/two-sided、instance span、BLAS/TLAS、wind refit、LOD和fallback receipt |
| VEG-P1-056 | Partial | generic physics shapes/lifecycle可复用；补trunk/canopy/query/interaction LOD、mutation和cell unload映射 |
| VEG-P1-057 | Partial | generic navigation可复用；补obstacle/area、canopy policy、batch dirty region、streaming和bend/break更新 |
| VEG-P1-058 | Open | 无picking/query identity；hit返回world/cell/prototype/instance/part/material stable identity与generation |
| VEG-P1-059 | Open | 无season/state adapter；color/leaf density/representation/collision变化原子提交并与save/network分层 |
| VEG-P1-060 | Open | 无audio/VFX/gameplay event；rustle/contact/bend/break以有界typed event输出，consumer不得改写内部状态 |

## 12. P1：Scalability、Diagnostics、Tests 与 Product Qualification

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| VEG-P1-061 | Partial | generic quality profile存在；density、LOD、wind、shadow、interaction、RT和streaming须联合决策并记录 |
| VEG-P1-062 | Partial | 多处局部容量/预算可复用；建立instances/clusters/draws/triangles/overdraw/bytes/CPU/GPU/IO统一admission |
| VEG-P1-063 | Partial | descriptor-only slots诚实表达Unsupported；补storage/indirect/compute/MSAA/RT/texture capability matrix和approved fallback |
| VEG-P1-064 | Partial | generic diagnostics可复用；补species/prototype/cell/cluster/instance、LOD/cull/wind/memory/work/fallback snapshot |
| VEG-P1-065 | Open | 无artifact/compiler tests；semantic import、determinism、LOD error、impostor、wind/collision和migration golden为空 |
| VEG-P1-066 | Open | 无instance lifecycle tests；mutation、stable ID、tree refit、cancel/reload/unload、stale generation和OOM为空 |
| VEG-P1-067 | Open | 无render differential；mesh/card/billboard/impostor、alpha/depth/shadow/velocity/GI/RT图像误差门为空 |
| VEG-P1-068 | Open | 无wind/interaction tests；history、phase、gust、branch/leaf、bend/recovery、NaN和replay differential为空 |
| VEG-P1-069 | Open | 无规模与故障矩阵；1K/100K/1M、fast camera、device loss、provider fault、atlas corruption和stream churn为空 |
| VEG-P1-070 | Open | Vampire不是闭合证据；须以clean-clone真实运行/capture证明draw、LOD、wind、alpha和shadow |
| VEG-P1-071 | Open | 无工程级产品场景；交付forest/grassland/understory/season/storm/interaction/world-cell save/play/export链 |
| VEG-P1-072 | Open | 无跨引擎超越基准；同asset/view/hardware/quality比较CPU/GPU、memory、IO、stutter、overdraw、LOD/image error |

## 13. P2：基础资格门之后的高级能力

| ID | 状态 | 延后条件 |
|---|---|---|
| VEG-P2-001 | Open | procedural species synthesis；先完成source/compiler schema、stable semantic ID、determinism和quality oracle |
| VEG-P2-002 | Open | runtime growth/aging；先完成topology/identity migration、season/save/network和collision/nav transaction |
| VEG-P2-003 | Open | branch break/destruction coupling；先完成Destruction owner、piece identity、interaction和render/physics output |
| VEG-P2-004 | Open | fire/burn/charring；先完成material state、VFX/weather/gameplay authority、RT/GI和replication |
| VEG-P2-005 | Open | snow/wetness accumulation；先完成Weather surface state、deformation load、material/GI和streaming |
| VEG-P2-006 | Open | biome/ecosystem succession；先完成Terrain placement、species lifecycle、determinism和large-world persistence |
| VEG-P2-007 | Open | GPU procedural scatter；先完成CPU placement oracle、stable IDs、cell transaction、readback/fault和fallback |
| VEG-P2-008 | Open | GPU branch/leaf simulation；先完成CPU oracle、history、bounds、collision boundary和device fault isolation |
| VEG-P2-009 | Open | neural impostor/radiance representation；先完成deterministic fallback、provenance和view/light error bound |
| VEG-P2-010 | Open | virtualized geometry vegetation；先完成GPU Scene/residency、alpha leaf cluster、wind和representation parity |
| VEG-P2-011 | Open | spectral/translucent foliage optics；先完成thin-leaf baseline、measured dataset和path/reference energy gate |
| VEG-P2-012 | Open | multiplayer authoritative vegetation state；先完成stable IDs、interest、rollback、late join和bandwidth receipt |
| VEG-P2-013 | Open | large-world cross-cell organism；先完成cell ownership、origin/rebase、stream continuity和atomic migration |
| VEG-P2-014 | Open | third-party species/provider SDK；先完成ABI/version/capability/budget/sandbox/unload和artifact compatibility |
| VEG-P2-015 | Open | collaborative vegetation authoring；先完成stable semantic ID、transaction/merge/locking/recovery和provenance |
| VEG-P2-016 | Open | distributed visual/performance farm；先完成frozen BuildSet、capture、raw receipt和promotion governance |

## 14. 分层重构里程碑

### M0 · Truth、Owner 与 Baseline

- 先关闭继承的Editor16 P0-4：Foliage Editor只显示typed Unsupported，或接真实Runtime job/receipt；冻结owner、术语、identity、units、预算、corpus和capability matrix。
- `Tree/Billboard` descriptor、普通Mesh、GLB与cross-card资产均不得提高Vegetation能力等级。

### M1 · Species Source、Schema 与 Compiler

- 完成VEG-P1-001..012：source/import semantic、stable IDs、deterministic LOD/card/impostor/wind/collision artifact、migration/DDC/LKG和raw receipt。
- Runtime compiler成为Editor与cook唯一authoritative实现。

### M2 · Prototype、Instance Set 与 Lifecycle

- 完成VEG-P1-013..024：prototype、SoA、generation handle、cell/cluster、batch mutation、atomic install、reload和multi-world drain。
- 先以CPU/reference backend证明状态机、identity和determinism。

### M3 · GPU Instance、Cluster、LOD 与 Streaming

- 完成VEG-P1-025..036：真实消费multi-instance span，建立cluster visibility、screen-error LOD、representation transition、compaction/indirect和cell residency。
- 每条路径必须输出visible/work/overflow/degrade receipt。

### M4 · Wind、Deformation 与 Interaction

- 完成VEG-P1-037..048：消费qualified WindField，建立part-aware deformation、history/reset、interaction/recovery和quality tier。
- CPU oracle与GPU differential并存，禁止无历史的单pass sine WPO。

### M5 · Leaf Shading 与 Pass一致性

- 完成VEG-P1-049..055：thin-leaf optics、coverage、main/depth/shadow/velocity/reactive/GI/RT parity。
- Mesh/card/billboard/impostor在批准误差内共享外观、风相位、bounds和identity。

### M6 · Physics、Navigation 与 Gameplay Adapter

- 完成VEG-P1-056..060：collision/query/nav、picking、season/state和bounded audio/VFX/gameplay event。
- adapter消费唯一instance snapshot，不维护第二份mutable truth。

### M7 · Editor、Terrain 与 Product Integration

- 与Editor16、Runtime142、Weather和App06闭环import/authoring/scatter/wind preview/cell streaming/save/reopen/PIE/export。
- UI列表、进度、warning、count、build和preview全部来自Runtime typed snapshot/receipt。

### M8 · Reliability 与 Scalability

- 完成VEG-P1-061..071：联合预算、platform matrix、diagnostics、compiler/lifecycle/render/wind/fault tests和1M实例压力门。
- 覆盖cancel、OOM、device loss、atlas损坏、fast camera和stream churn。

### M9 · 性能与表现超越门

- 完成VEG-P1-072：同species资产、数量、镜头、风、阴影、RT/GI、硬件、分辨率和warm-up对照Unreal/Unity。
- 归档CPU/GPU timestamp、memory、IO、stutter、draw/work、overdraw、LOD/image error和raw capture；无可复跑receipt不得声称超越。

## 15. 资格门当前状态

| Gate | 状态 | 当前判定 |
|---:|---|---|
| 1 | Fail | Editor硬编码Ready/queued/count，capability truth不成立 |
| 2 | Fail | species source/build/prototype/instance-set均无独立schema/version/digest |
| 3 | Fail | 无deterministic compiler或artifact digest |
| 4 | Fail | 无vegetation semantic/topology/unit/alpha fail-close diagnostic |
| 5 | Fail | 无part/vertex semantic跨import、LOD、representation和shader reflection合同 |
| 6 | Fail | 无LOD/card/impostor build及silhouette/coverage/normal/depth error receipt |
| 7 | Fail | 无instance handle，不能拒绝remove/reuse/reload后的stale generation |
| 8 | Fail | 无batch mutation和cluster refit/rebuild |
| 9 | Fail | 无build/upload generation fencing |
| 10 | Partial | generic artifact原子publication可复用；Vegetation bundle install仍为空 |
| 11 | Partial | GPU Scene支持span；ordinary mesh consumer固定count=1且无植被capture |
| 12 | Partial | generic frustum/HZB可复用；无cluster/instance CPU oracle和误差门 |
| 13 | Fail | ordinary Mesh LOD不是projected-error per-instance LOD且无hysteresis |
| 14 | Fail | 无mesh/card/billboard/impostor统一identity和图像误差 |
| 15 | Fail | 无stereo/shadow/极角billboard稳定性 |
| 16 | Fail | 无impostor color/normal/depth/opacity/mip atlas |
| 17 | Fail | translation-radius近似不能证明偏心树冠/nonuniform scale bounds |
| 18 | Fail | 无最大wind/bend conservative bounds |
| 19 | Partial | generic indirect命令前置存在；无vegetation compaction/overflow receipt |
| 20 | Partial | generic chunk residency可复用；无cell/prototype atomic dependency bundle |
| 21 | Fail | 无world/time/generation-qualified WindField snapshot |
| 22 | Fail | 无stable-seed per-instance wind variation |
| 23 | Fail | 无branch/leaf/frond deformation和NaN/energy/bounds门 |
| 24 | Partial | GPU Scene有current/previous transform；无deformation/representation history |
| 25 | Fail | 无spawn/teleport/LOD/cell/seek history reset |
| 26 | Fail | 无interaction owner/lifetime/budget和observable degrade |
| 27 | Fail | 无thin-leaf reference render与energy test |
| 28 | Partial | generic cutout/double-sided存在；无A2C/mip coverage/platform资格 |
| 29 | Fail | main/depth/shadow无共享vegetation deformation/alpha generation |
| 30 | Fail | WPO/billboard/LOD无正确velocity/reactive/disocclusion数据 |
| 31 | Partial | generic shadow存在；无caster LOD/WPO/cluster bounds和popping receipt |
| 32 | Partial | generic GI/RT前置存在；无同代vegetation representation/fallback |
| 33 | Fail | query不能返回world/cell/prototype/instance/part stable identity |
| 34 | Fail | render/collision/nav mutation无cell generation和drain |
| 35 | Fail | generic quality配置未形成density/LOD/wind/shadow/RT联合预算 |
| 36 | Fail | 无1K/100K/1M CPU/GPU/memory/IO/stutter raw receipts |
| 37 | Fail | 无cancel/OOM/device loss/provider fault/atlas corruption/stream churn矩阵 |
| 38 | Fail | Vampire没有Vegetation clean-clone运行/save/reopen/PIE/export/capture闭环 |
| 39 | Fail | 没有证明grass/tree LOD、wind、alpha、shadow、velocity的accepted图像 |
| 40 | Fail | 无同资产/场景/硬件/画质的Unreal/Unity可复跑对照 |

## 16. Finding 到里程碑映射

| Finding | 里程碑 |
|---|---|
| Editor16 P0-4（引用） | M0，先于所有产品展示继续扩展，fixing owner不变 |
| VEG-P1-001..012 | M0-M1 |
| VEG-P1-013..024 | M2 |
| VEG-P1-025..036 | M3 |
| VEG-P1-037..048 | M4 |
| VEG-P1-049..060 | M5-M6 |
| VEG-P1-061..072 | M7-M9 |
| VEG-P2-001..016 | 对应P1与资格门完成后独立立项，不得提前并入MVP |

## 17. 禁止的临时修补

1. 禁止只增加`Vegetation`/`Tree`枚举、空feature、catalog metadata或静态workspace便宣称支持。
2. 禁止把普通Mesh、预合并cross-card、GLB数量、GPU span allocator或descriptor-only slot当作GPU vegetation证据。
3. 禁止为每棵树/每簇草创建ordinary Scene entity绕过instance-set、cluster和cell lifecycle。
4. 禁止继续`register(..., 1)`后以README、DTO或UI count声称batch/indirect已经执行。
5. 禁止用entity-origin distance和单一`min_distance`冒充screen-error per-instance LOD。
6. 禁止脚本手换mesh/card/billboard并丢失identity、wind phase、shadow或history。
7. 禁止把wind实现为未版本化global sine或把branch/leaf数据暗塞任意UV/color通道。
8. 禁止只在main pass形变；depth、shadow、velocity、GI和RT必须共享qualified deformation。
9. 禁止把`double_sided=true`、generic subsurface或opaque cross-card包装成thin-leaf shading。
10. 禁止用永久放大的world bounds掩盖wind bounds错误并接受严重visibility/RT退化。
11. 禁止由Terrain、Weather、Renderer和Editor各维护一份prototype/instance/wind mutable truth。
12. 禁止硬编码Ready、queued、progress、warning、instance/cluster count或成功toast；必须来自typed runtime receipt。

## 18. 实施前重查清单

1. 重算本文七组fingerprint，记录新增、删除、修改及working-tree来源。
2. 重查Runtime142 Terrain placement、Editor16、Weather、Runtime94和App06 owner，禁止重复实现。
3. 重查ordinary mesh consumer是否仍`register(..., 1)`，GPU Scene是否已有真实multi-instance producer。
4. 重查`Tree/Billboard/Terrain/MeshLod`是否仍descriptor-only，若新增pass必须验证registration、executor和receipt。
5. 重查shader是否新增pass-shared vertex deformation及depth/shadow/velocity同代消费。
6. 重查Weather是否提供generation-qualified WindField；没有时保持显式dependency，不私建global风。
7. 重查Foliage Editor是否已移除硬编码Ready/84K/128或接真实job/diagnostic/terminal receipt。
8. 重查Vampire material alpha/wind、model、runtime consumer和accepted capture，普通cross-card仍不得提高资格。
9. 锁定M0-M9每阶段BuildSet、target/backend、hardware、quality、warm-up、raw receipt和promotion规则。

## 19. 本轮产出边界

本篇只完成静态源码审查、参考对照、唯一owner划分、继承P0/P1/P2登记、分层重构路线和资格门，没有修改production代码、Cargo、测试、workflow、UI或产品资产，没有运行构建、测试或GPU capture，也没有证明任何Vegetation功能、性能或表现已完成。后续实施必须先复核source currentness并由Editor16关闭P0-4，再按M0-M9底层依赖推进；在本文40项资格门全部有可复跑证据前，不得把普通Mesh、Terrain名称、Tree/Billboard descriptor、GPU span、静态Foliage Editor或Vampire cross-card宣传为工程级Vegetation runtime。
