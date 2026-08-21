---
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
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
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

# 39 · Spline / Path / Road / River / Decal / Brush / Geometry Authoring 工程化差距

## 1. 结论

Zircon当前没有引擎级空间Spline、Path、PathFollow、SplineMesh、Road、River、Water Body、Geometry Brush或CSG产品。仓内能搜索到的`CubicSpline`只负责glTF动画插值，Hermite实现只采样动画channel，Sound curve只采样标量自动化；它们都没有空间曲线所需的稳定点/段identity、弧长参数化、旋转最小标架、最近点查询、分段bounds、空间索引、流送、Scene持久化或Editor控制点工具。

项目已经因此产生真实绕行。`examples/woc`把14条道路从外部`BUILTIN_WORLD`抽成JSON，再生成`roadPointX()`和`roadPointZ()`的大量索引分支；植被候选对所有道路和所有线段逐一做二维点到线段距离查询。该实现有确定性数据合同，但它是游戏专用折线表，不会生成道路mesh、UV、材质、碰撞、导航、车道、路口、地形压印或world-partition产物，也没有作者工作流。`examples/vampire`则把Road当普通导入mesh重复放入Scene。两者共同证明Road是现有项目需求，而非可以长期留给示例脚本的可选装饰。

Decal比其余域更危险，因为它不是单纯缺失，而是形成了错误的完成信号。可选`rendering.decals`插件注册`DecalProjector`类型描述、一个PostProcess pass和`decals.projector-composite`执行器；执行器函数只返回`Ok(())`。`DecalProjectorDescriptor`的mode、opacity、normal_blend和atlas_region没有实例owner、extract、shader、texture/material binding、culling、batch或GPU消费。唯一runtime test只验证registration report成功、组件名字和pass名字。当前启用插件可以“成功”却不产生任何像素，这是P0 truthfulness问题。

Editor侧同样只有名义表面。Decal editor crate声明`DECAL_PROJECTOR_DRAWER_ID`，但插件没有覆盖`register_editor_extensions()`，没有注册drawer、Inspector customization、Scene mode、overlay、create/add operation、transaction或preview。`tools/editor-workbench-preview/design.js`存在一张固定显示WarningStripe、12 placements和“Projection bounds updated”的Decal设计稿，但主Editor没有对应业务面。Material Workbench还提供`decal`下拉项，而runtime `MaterialDomain`只有Surface、PostProcess、DebugOverlay和LightFunction；字段事件只更新模板control状态，未产生材质资产事务或compiler receipt。

需要保留并复用的基础也必须准确承认：Runtime plugin component descriptor会通过`apply_to_world()`注册到World；World支持动态JSON组件、稀疏presence、generation和inspection invalidation；`DynamicScene`是schema v2、带迁移、可捕获serializable reflected component和plugin descriptor。Editor有SceneMode registry/factory/stack、selection、overlay provider和transform gizmo基础。这些是实现产品链的底座，但Decal插件没有创建组件实例，静态`SceneAsset`没有通用plugin component字段，Editor没有添加/保存它的operation，Render也没有消费它。不能把“底座允许未来实现”记为“功能已经存在”。

目标不是补一个`Vec<Vec3>`和几个按钮。应建立版本化`SpatialSplineSource`、确定性`CompiledSplineArtifact`、不可变runtime query view与typed consumer adapters；Road、River、SplineMesh、PathFollow、terrain stamp和ribbon decal各自拥有领域schema/compiler。Decal应另建完整projector/material/extract/cull/batch/render/editor闭环。Geometry Brush应作为独立shape/boolean/extrusion authoring域，不得与Terrain画笔、Foliage画笔或UI paint brush混为一谈。

本报告登记5个P0、70个P1、12个P2、M0-M11重构路线和32个验收门。它只做review，不修改Runtime、Editor、plugin、interface、App生产代码或tests。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | test attributes / ignored / 在途 | 审查方式 |
|---|---:|---:|---|
| Decal package、catalog与pipeline truth | 16 / 1,862 / 64,514 | 8 / 0 / 0 | E3逐descriptor、manifest、registration、pass placement和catalog |
| Scene asset、reflection与DynamicScene边界 | 23 / 3,720 / 139,955 | 3 / 0 / 0 | E3逐static Scene、World registration、JSON component、capture/migration |
| Material domain与render contract | 14 / 4,575 / 180,200 | 8 / 0 / 1 | E2/E3逐domain DTO、material document、Workbench route与executor contract |
| Editor Scene tool与plugin authoring底座 | 36 / 4,954 / 162,295 | 31 / 0 / 0 | E3逐plugin hook、SceneMode/selection/overlay及Terrain/Particle对照 |
| mock surfaces与项目专用Road绕行 | 13 / 19,759 / 893,735 | 0 / 0 / 1 | E2/E3逐静态设计稿、Terrain/Foliage模板、WOC生成和查询链 |
| 既有非空间curve碎片 | 7 / 1,532 / 52,383 | 5 / 0 / 0 | E3逐glTF/Hermite/automation sampling，核对无空间API |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics参考 | 36 / 25,382 / 1,003,701 | 12 / 0 / 0 | E2/E3按Spline、Water、Decal、Editor visualizer和CSG职责路由 |
| selected combined scope | 145 / 61,784 / 2,496,783 | 67 / 0 / 2 | 当前工作树fingerprint `6af2678786d08838b1095b802b8cd89cb1a27cacf2a6a94f5f3eff9114524351` |

指纹算法为：对145个选择路径按PowerShell `Sort-Object`排序，逐文件计算小写SHA-256，形成`forward/slash/path|file_sha256`行，以单个LF连接且末尾不追加LF，再对UTF-8无BOM payload计算SHA-256。选择规则包括完整Decals editor/runtime目录、完整`zircon_editor/src/scene/modes`和`scene/selection`目录，以及表中其余显式文件；缺失与重复路径均为0。

读取时2个在途文件为`zircon_editor/src/ui/retained_host/workbench_preview_actions.rs`和`zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs`，均非本报告产生。实施前必须重导145文件manifest、重算指纹并复核这两个文件及其后续落地状态。

67个Rust test attributes主要覆盖plugin registration、SceneMode stack、selection、template control和参考引擎自身单元测试。它们不包含Zircon空间Spline数值测试、Road/River compiler golden、Decal pixel test、Editor控制点transaction或大世界性能测试，因此不能用数量代替产品验收。

### 2.2 产品类型与入口缺失

1. Zircon生产Rust代码没有空间`Spline`或`Curve3D`source、component、asset、artifact、query service或scene node。
2. `path follow`、`PathFollow`和`pathfollow`没有生产命中。
3. `Road`和`River`没有production resource/component/plugin；宽泛`road`命中大多来自`broadcast`或示例内容。
4. `WaterBody`、`Water Body`和Water plugin目录不存在；生产`water`命中主要是queue low-water指标，不是水体。
5. `CSG`只有诊断/参考文字，没有boolean geometry kernel或authoring system。
6. `Geometry Brush`和`Brush Builder`没有生产类型或操作。
7. Terrain和Foliage Workbench的“Brush Properties”是静态模板字段；它表示地形/植被画笔UI，不是Geometry Brush。
8. Foliage模板中的`Biome_Riverbank`、`River_02`和Scatter中的`SC_Riverbank`是固定演示label，不是River asset query。
9. `ResourceKind`没有Spline、Path、Road、River、Water、Decal或GeometryBrush；plugin仍可定义custom kind，但当前没有owner这样做。
10. 第一方runtime/editor catalog没有Decals插件实例；`rendering/plugin.toml`只把它声明为disabled-by-default的可选source/library feature。
11. Builtin `Decal`位于`DESCRIPTOR_ONLY_ADVANCED_SLOTS`，只提供名字和extract section token，不提供renderer实现。
12. 主Editor没有Decal Workspace ZUI；唯一完整Decal页面位于独立workbench preview设计工具，是固定mock数据。

### 2.3 Decal Runtime的实际执行路径

1. `RenderingDecalsRuntimeFeature::register()`只注册component descriptor、render feature descriptor和render pass executor。
2. component descriptor的properties只有`name`、字符串`value_type`和`editable`；没有default、range、unit、asset type、stable property ID、schema version、migration或validator。
3. `DecalProjectorDescriptor`是普通Rust struct，却没有实现component storage、reflection、serialization或与descriptor JSON的转换。
4. 该struct在定义文件外没有消费者；mode、opacity、normal_blend和atlas_region也没有消费者。
5. `atlas_region`是`String`，不是Material/Texture/Atlas asset reference，render feature也没有声明texture resource。
6. pass读取scene-depth和scene-color并写scene-color，但executor不绑定pipeline、shader、bind group、vertex/index/instance buffer或draw/dispatch命令。
7. `normal_blend`无法由当前pass合同实现，因为descriptor不读写normal/GBuffer/DBuffer attachment。
8. Deferred和ScreenSpace两个mode没有分支；两者都会进入同一个空executor。
9. feature插入规则把Decals放在baked lighting、reflection probes和bloom之后，再把post_process放在Decals之后。
10. 该位置不具备Deferred DBuffer/GBuffer decal所需的pre-lighting attachment合同，而且Decal在Bloom之后会让颜色/曝光链语义含混。
11. descriptor没有显式定义scene-color read/write的ping-pong、load/store、blend、alias或hazard处理，不能把资源名字当作合法合成合同。
12. 没有projector bounds、view frustum/cluster culling、material grouping、sort order、visibility mask、distance/angle fade、lifetime或budget。
13. 没有albedo/normal/ORM/emissive通道影响mask，也没有transparent receiver、forward、mobile、MSAA、ray tracing或virtual geometry策略。
14. 没有texture streaming、atlas allocation/eviction、bindless capability gate或fallback material。
15. 没有frame stats、visible/dropped count、atlas pressure、pass timing、unsupported material或fallback diagnostics。
16. 唯一test不执行executor并验证像素，只断言registration report成功和pass名称。

### 2.4 Decal Editor与Material false surface

1. Decal editor plugin只返回descriptor/capability，使用`EditorPlugin`默认空`register_editor_extensions()`。
2. `DECAL_PROJECTOR_DRAWER_ID`没有注册点或消费者。
3. 没有Inspector customization、add component、create asset、place projector、open material或validate operation。
4. 没有SceneMode、component visualizer、projector box handle、orientation arrow、pick proxy或selection overlay。
5. 没有document session、transaction、undo/redo、dirty/save、preview world或last-good compile。
6. Terrain和Particles插件至少注册commands、menu、toolkit、creation template和Inspector customization，Decal连这一名义基线也未达到。
7. Particles menu仍被disabled，Terrain authoring也主要是descriptor/import plan；这些局限分别由Editor15/16负责，不能拿来证明Decal完整。
8. Material Workbench的domain选项固定为`surface`、`post_process`、`decal`。
9. runtime `MaterialDomain`只有Surface、PostProcess、DebugOverlay和LightFunction，没有Decal。
10. `ShaderProgramAsset`、`ShaderGraphAsset`、`MaterialGraphAsset`和`ShaderVariantKey`携带该enum，但仓内没有其他生产消费者。
11. Material domain Change/Submit只进入通用panel field action，测试只验证dropdown值与模板journal变化。
12. `.zmaterial` schema v2拥有shader、parent、options、overrides、textures和queue，但没有domain字段，也没有Decal material validation/compiler output。
13. 因而UI中的`decal`是第二套authority；选择它不能形成资产真值、shader variant、render pass或项目roundtrip。
14. 独立preview工具固定显示WarningStripe、12 placements、1 sort warning和Projection bounds updated，源码没有任何对应query或receipt。

### 2.5 Scene、DynamicScene与plugin component边界

1. 静态`SceneAsset`只是`Vec<SceneEntityAsset>`；entity以固定字段持久化camera、mesh、lights、post-process、physics、animation、terrain、tilemap、prefab和scripts。
2. 该static schema没有plugin components集合，也没有Spline、Road、River、Decal或Geometry Brush字段。
3. `SceneAsset`的project TOML serialization和World construction只认识这些固定字段，无法从项目文档创建Decal实例。
4. `RuntimeExtensionRegistry::apply_to_world()`确实把每个component descriptor传给`World::register_component_type()`。
5. World把descriptor转为dynamic reflection registration，分配dynamic component ID，并支持稀疏presence与JSON value。
6. `set_dynamic_component()`会验证entity/type，更新inspection cache、component generation和world generation。
7. plugin descriptor生成的非VM dynamic component在field写入时可自动创建JSON object；这仍不是typed domain schema或项目authoring operation。
8. `DynamicScene`的component字段是`type_path`、`plugin_owned`和`ReflectFieldValue`列表。
9. DynamicScene schema版本为2，具备v0到v1、v1到v2迁移，读写使用versioned payload。
10. `DynamicScene::from_world()`会遍历serializable reflected components，并只携带实际被entity使用的plugin descriptors。
11. 因此手工向World挂载Decal JSON后，DynamicScene基础原则上可以捕获它；当前没有代码执行这一步。
12. Editor Play snapshot能捕获运行中的DynamicScene，不等于project Scene document已经能author和reopen Decal。
13. Decal descriptor没有typed default instance、asset reference字段、版本或迁移；即使手工JSON存在也没有render consumer。
14. 正确改造应选择统一的project Scene/plugin component持久化桥，而不是为每个新域继续向`SceneEntityAsset`追加临时Option字段。

### 2.6 WOC道路绕行与性能风险

1. `m3_terrain_content_source_extract.mjs`从外部`BUILTIN_WORLD.roads`复制二维X/Z点。
2. `m3_terrain_content.json`持久化14条折线，但没有稳定road/point/segment ID、Y、tangent、roll、width、profile或junction。
3. codegen以道路长度数组和全体道路JSON SHA-256校验source drift，这一确定性合同值得迁移保留。
4. `renderRoadPointCoordinate()`为每条road、每个point生成嵌套index判断和常量return。
5. 生成的`terrain_content.zr`因此包含大量`roadPointX()`和`roadPointZ()`分支；数据量增加会线性膨胀代码与VM分支。
6. `roadDistance()`对每次查询遍历全部road和全部segment，复杂度为O(total segments)。
7. 查询只在XZ平面做最近折线距离，不能返回arc length、lane offset、height、normal、frame、road ID或segment metadata。
8. 植被剔除直接以`roadDistance < 5.0`判断，半径是游戏脚本magic constant，不来自road profile。
9. 没有segment BVH/grid、batch query、SIMD、cached tile result或world-partition局部索引。
10. 没有道路mesh、shoulder、curb、UV、material layer、decal marking、collision、nav modifier、traffic lane或AI path导出。
11. 没有terrain conform/cut-fill/stamp、bridge/tunnel、slope/bank限制、crossing或junction solver。
12. Vampire示例把导入Road mesh重复放置，只解决可见几何，不解决上述authoring/runtime合同。

### 2.7 可复用curve与Editor底座的上限

1. glTF `CubicSpline`被映射为`AnimationInterpolationAsset::Hermite`，输入是时间key和动画value/tangent。
2. Runtime与Animation plugin各有一套Hermite采样实现，支持Scalar/Vec2/Vec3/Vec4，但没有空间曲线owner或弧长重参数化。
3. Sound automation curve是标量timeline，不提供空间frame或最近点。
4. 这些数学函数可提炼共享多项式/验证工具，但不能直接暴露为SpatialSpline产品。
5. Builtin SceneMode registry只有Select和Transform两个factory。
6. SceneMode context只持有SelectionModel、viewport settings、单个input effect和overlay invalidation。
7. SceneMode本身没有document session、transaction writer、snap service、typed sub-selection或async compile handle。
8. ViewportOverlayBuilder只累积`SceneGizmoOverlayExtract`；它可作为控制点/切线可视化输出，但没有Spline primitive或stable pick identity。
9. SelectionModel当前围绕world/entity domain，未定义Spline point/segment/junction等子对象selection key。
10. Editor extension registry支持overlay provider和authoring contributions，适合扩展，但Decal/Spline/Road/River没有注册任何贡献。

### 2.8 参考引擎差异

| 参考 | 可验证下限 | 对Zircon的约束 |
|---|---|---|
| Unreal | `FSplineCurves`分离position/rotation/scale并维护ReparamTable；SplineComponent有open/closed、UpdateSpline、length/distance/transform查询；SplineMesh有start/end tangent、roll、scale、bounds、collision和nav；Landscape/Water有专用metadata、visualizer与domain tool | 共享Spline kernel与domain-specific metadata/compiler必须分层；不能用一组字符串property覆盖所有消费者 |
| Godot | Curve3D有bake interval/length、up vector、tilt、closest point/offset、tessellation；PathFollow3D有progress/loop/rotation；Decal node真实创建RenderingServer RID并同步texture/size/fade/mask/AABB | 即便较紧凑的引擎也提供持久化资源、查询语义和renderer lifecycle，Zircon空executor低于此基线 |
| Fyrox | Decal是可Reflect/Visit的Scene node，拥有diffuse/normal texture、color、layer、bounds和builder；shader从depth重建投影并输出颜色/normal | 最小Decal也必须有Scene实例、序列化、资源引用、bounds和真实shader；registration test不是renderer test |
| Bevy | cubic_splines提供Bezier/Hermite/Cardinal/B-spline及采样导数；clustered/forward decal有extract、prepare、storage buffer、texture、cluster integration和能力限制 | 可复用数学库与ECS render pipeline都应有明确边界；没有Editor并不降低Zircon产品Editor要求 |
| Unity HDRP | DecalProjector公开material、size/pivot、draw distance/fade、angle fade、UV、layer和scale mode；DecalSystem按material/draw order组织、cull、batch、atlas并影响多材质通道；Editor有handles与migration | Decal需完整source-to-GPU生命周期、稳定handle、迁移和Editor交互；单PostProcess token不构成HDRP级实现 |

Unreal Geometry Brush/CSG是独立于Spline/Water/Decal的领域。Zircon应先定义现代用途：blocking volume、greybox、mesh boolean、terrain stamp或procedural extrusion，再决定是否实现兼容BSP工作流。不能为了“像Unreal”而把legacy BSP设为所有路径工具的基础。

### 2.9 动态证据边界

本轮是review-only，没有修改Runtime、Editor、plugin、interface、App生产代码或tests，也没有运行新的动态测试。空executor、descriptor无消费者、Decal editor空hook、MaterialDomain不匹配、static Scene断点和WOC O(total segments)查询均可由当前源码直接证明。

此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断，当前源代码没有解除该阻断条件，本轮没有重复相同lane。后续实施必须先恢复可编译基线，再运行数值、roundtrip、Editor transaction、GPU pixel、frame capture、性能与大世界组合验收。

## 3. 目标架构

### 3.1 Source、Artifact与Instance三层

```text
Project authoring
  -> SpatialSplineSource(schema, stable IDs, points, tangent modes, metadata lanes)
  -> SpatialSplineValidator
  -> SpatialSplineCompiler(source digest, build settings, dependency digest)
  -> CompiledSplineArtifact
       segment coefficients
       arc-length lookup tables
       rotation-minimizing frames / authored roll
       per-segment bounds + spatial index
       deterministic diagnostics + generation
  -> immutable SplineRuntimeView / SplineInstance
       sample by parameter or distance
       closest point / distance / signed lateral offset
       point/segment metadata lookup
       batched and partition-local queries
  -> typed consumers
       PathFollow / SplineMesh / Road / River / Terrain Stamp / Gameplay / Ribbon Decal
```

Source可编辑、可迁移、可diff；Artifact面向性能且不可变；Instance只绑定World transform、visibility、streaming cell和artifact handle。Render、Physics、Navigation、Gameplay线程不得遍历Editor source或任意JSON。

### 3.2 建议核心schema

| 类型 | 必需字段 | 关键不变量 |
|---|---|---|
| `SplineAssetId` / `SplinePointId` / `SplineSegmentId` | stable UUID或稳定整数ID | insert/delete/reorder不改变未受影响对象identity |
| `SpatialSplineSource` | version、coordinate space、closed、points、metadata schema、build settings | 至少2点；finite；ID唯一；闭环规则明确 |
| `SplinePointSource` | id、position、arrive/leave tangent、tangent mode、roll、scale/up override | tangent mode决定可写字段；单位明确 |
| `SplineMetadataLane<T>` | stable lane ID、domain owner、interpolation、per-point/per-segment values | 长度与point/segment topology一致；typed value |
| `CompiledSplineArtifact` | schema、source digest、compiler version、segments、arc table、frames、bounds/index | 同输入byte-identical；无Editor object引用 |
| `SplineSample` | position、tangent、normal、binormal、scale、roll、curvature、segment ID、distance | frame正交、finite、boundary连续性可报告 |
| `SplineClosestResult` | point、distance、squared distance、segment ID、parameter、lateral/vertical offset | 误差界与tie-break确定性明确 |

必须区分linear、constant、auto、clamped auto、broken和user tangent等mode。不要把mode编码为可任意拼写的字符串；不要用point数组索引作为长期identity；不要假设Frenet frame在零曲率和拐点处稳定。

### 3.3 Typed consumer分层

| Consumer | 自己拥有 | 从Spline读取 | 不应回写 |
|---|---|---|---|
| PathFollow | progress/time policy、speed profile、loop/ping-pong、orientation/offset | distance sample与frame | Spline topology |
| SplineMesh/Extrusion | cross-section、mesh/material、UV、tessellation、collision policy | segment curve/frame/scale | Road lane或River flow |
| Road | profile、lane、shoulder、curb、marking、junction、surface、terrain/nav/traffic policy | centerline与typed road metadata | 通用Spline kernel内部缓存 |
| River/Water | width/depth/flow/bank、water material、bed/carve、buoyancy/collision、confluence | centerline/frame与water metadata | Road profile或Decal material |
| Ribbon Decal | width、material、UV tiling、fade/sort、receiver mask | sampled ribbon path | Projector Decal schema |
| Terrain Stamp | height/paint/exclusion falloff、layer mask、build cell | curve footprint | Spline source |
| Gameplay/AI | named lane、event marker、query handle | immutable artifact/query | Editor document |

一条Spline可以被多个consumer引用，但每个consumer必须有独立versioned source和compiler receipt。所谓“万能Spline Component”最多承载曲线和metadata lane，不承载Road、River、Audio、Navigation、Decal的全部业务字段。

### 3.4 Compiler、query与性能合同

1. Compiler先验证finite、ID、topology、tangent、metadata cardinality和coordinate range，再生成artifact。
2. 弧长表使用可配置误差界的自适应细分，不以固定每段10步作为最终高质量策略。
3. Frame采用rotation-minimizing/parallel-transport策略，并把author roll作为显式旋转叠加。
4. 最近点查询先用segment bounds/BVH/grid裁剪，再做curve refinement；提供single、batch和coherent query入口。
5. Artifact按segment/chunk可局部重编译，未改变段复用缓存；source digest、compiler version和settings进入DDC key。
6. Async compile具有generation、cancel、stale reject和last-known-good publish；Editor拖动时可用低质量preview，提交后高质量重编译。
7. World只发布immutable artifact handle；替换以generation原子切换，旧frame可安全完成。
8. World partition按segment bounds映射cell；跨cell曲线有owner/continuation规则，不能复制后产生双重consumer。
9. 大坐标使用local origin/chunk transform，避免把全世界高精度问题塞入单个f32点数组。
10. 性能目标必须以代表性Road/River规模、CPU query吞吐、compile延迟、artifact bytes、streaming churn和GPU cost记录；“优于Unreal”只能由同场景基准证明。

### 3.5 Road产品链

```text
RoadNetworkAsset
  -> centerline spline references
  -> RoadProfileAsset(lanes, shoulder, curb, camber, materials)
  -> junction graph + crossing/bridge/tunnel annotations
  -> RoadCompiler
       surface mesh / marking decals / collision
       terrain cut-fill/stamp / foliage exclusion
       navigation modifier / traffic lane graph
       HLOD + partition chunks + diagnostics
  -> RoadArtifactSet + per-domain receipts
```

Road point editor只编辑source；它不应同步执行mesh cook、nav bake和terrain重建。Compiler要输出每域dependency和generation，允许Terrain、Navigation和Render异步消费同一accepted build。WOC现有14条折线应迁移为fixture：迁移后距离/排除行为保持golden，同时新增artifact、BVH和可编辑roundtrip。

路口不能靠两条折线视觉相交自动猜测。需要stable junction ID、port/lane connectivity、priority、turn rule和几何生成失败diagnostic。Bridge/Tunnel/Cut/Fill也必须是显式annotation或规则输出，避免terrain stamp破坏桥面和洞口。

### 3.6 River / Water产品链

River不是换材质的Road。它至少需要width、depth、surface elevation/grade、flow direction/speed、bank profile、bed/carve和confluence metadata。Runtime产物要分别服务water surface render、flow/current、terrain carve、collision/query、buoyancy、navigation cost、audio/VFX和streaming。

River compiler必须验证逆坡、零宽、self-intersection、bank inversion、junction discontinuity和cell boundary continuity。Water renderer、simulation和material的主体差距应由后续独立Runtime/Editor Water报告继续展开；本报告只定义Spline/River authoring入口与跨域receipt，不能用一条静态蓝色mesh声称水体完成。

### 3.7 Decal垂直闭环

```text
DecalMaterialSource(domain = Decal, channel mask, textures, blend policy)
  -> DecalMaterialCompiler -> DecalMaterialArtifact

DecalProjectorComponentSource
  -> material reference, transform/size/pivot, UV, sort, fade, receiver mask, lifetime
  -> Scene/DynamicScene persistence + migration
  -> World DecalInstance storage/generation
  -> frame extract(projector bounds, material handle, flags)
  -> view culling / clustering / sorting / batching
  -> DBuffer or GBuffer path + forward/transparent fallback
  -> GPU resources / pass receipts / stats / diagnostics
  -> Editor inspector + projector handles + preview + frame capture
```

当前空executor在真实实现前必须hard-disable或明确报告unsupported，不能继续返回成功。Decal material domain应只有一个source of truth，并贯穿`.zmaterial`、shader graph/compiler、variant key、render pass和Editor dropdown。若某平台不支持normal/ORM或bindless atlas，必须在compile/activation时给出结构化fallback，不得静默丢通道。

### 3.8 Geometry Brush / CSG边界

先定义产品用途再选择kernel。建议最小现代范围为：versioned primitive/profile source、extrusion/revolve/sweep、union/subtract/intersect、transform hierarchy、deterministic mesh build、UV/material surface assignment、collision/nav option和diagnostics。Greybox、blocking volume和terrain stamp可以共享shape source，但输出artifact不同。

Geometry Brush不应成为Spline、Road、River或Decal的基础owner；Sweep/Extrusion可以消费CompiledSplineArtifact。若决定支持Unreal式legacy BSP，应另设兼容层、转换工具和性能预期，不能让BSP topology污染现代mesh/terrain pipeline。

### 3.9 Editor authoring闭环

```text
Asset/Scene document session
  -> typed Spline/Road/River/Decal/Geometry operations
  -> transaction + undo/redo + dirty/save/autosave/recovery
  -> SceneMode factory
       point/segment/junction sub-selection
       add/insert/delete/split/join/reverse/open/close
       tangent/roll/scale/width handles
       snapping + numeric inspector + box/multi-select
       overlay/pick IDs tied to stable source IDs
  -> preview compiler job(generation, cancel, stale reject, LKG)
  -> viewport product + diagnostic anchors + build receipt
  -> save/reopen/play/export parity
```

SceneMode context需要通过受控service访问document operation和transaction，不能直接修改World或template control。拖拽过程要合并为单个transaction，Escape取消并恢复source；selection依赖stable point/segment ID，不依赖数组index。所有按钮和快捷键都路由到同一operation，remote automation只调用显式允许且可审计的operation。

### 3.10 持久化与plugin边界

应优先把static project Scene升级为可版本化plugin component payload或统一authoring Scene schema，再由resolver构建World/DynamicScene。不能继续为每个新component向`SceneEntityAsset`添加Option，否则owner、migration、卸载和第三方扩展都会失控。

Plugin component schema至少需要stable type/property IDs、display metadata、typed value/asset reference、default、range/unit、serialization policy、schema version和migration hook。Runtime卸载必须检查active instances/artifacts/jobs/GPU resources；Editor卸载必须关闭或orphan document、撤销SceneMode、清理overlay/pick和保留可恢复source。

### 3.11 Diagnostics与观测

每次compile/publish至少记录domain、source revision、source/artifact digest、compiler version、generation、duration、cache hit、input/output count、bytes、warnings/errors、cancel/stale/LKG状态。Road/River记录segments、junctions、cells、mesh triangles、collision/nav outputs；Spline记录arc samples、BVH nodes和query误差；Decal记录total/visible/culled/dropped、batches、atlas occupancy、channel/platform fallback和GPU pass timing。

Editor diagnostic必须带stable source object ID和可点击定位；Runtime diagnostic必须带world/view/frame/generation。固定“1 warning”或“Projection bounds updated”字符串不能替代typed diagnostic record。

## 4. 优先级与重构清单

### 4.1 P0：必须先阻断错误完成信号

| ID | 差距 | 必须动作 | 完成证据 |
|---|---|---|---|
| P0-01 | Decal executor空实现却返回成功 | 在真实render闭环前hard-disable/unsupported；禁止registration success被解释为feature ready | 启用路径明确失败或真实产生验证像素，无静默`Ok(())` |
| P0-02 | 没有空间Spline source/artifact/query authority | 定义版本化schema、compiler、artifact与immutable query API | 数值/roundtrip/golden/perf门通过，Road/River不再自带曲线数学 |
| P0-03 | static Scene无法author/persist plugin Decal/Spline实例 | 建立统一plugin component project persistence、migration和World bridge | save/reopen/play/export保留typed实例和稳定ID |
| P0-04 | Material UI提供Decal但runtime/schema/compiler没有该domain | 建立单一MaterialDomain authority，未完成前移除或禁用false option | `.zmaterial`、compiler、variant、pass和Editor roundtrip一致 |
| P0-05 | WOC Road与Editor mock把项目需求伪装成完成 | 把WOC道路列为迁移fixture；静态Decal/River/Brush反馈不得进入产品catalog | product inventory只显示有owner、operation、artifact和receipt的能力 |

### 4.2 P1：工程化主线

1. **P1-01**：批准`SpatialSplineSource`、`CompiledSplineArtifact`和`SplineRuntimeView`的crate/module owner，禁止Editor/plugin各自复制kernel。
2. **P1-02**：定义Spline source schema ID/version及向前迁移链。
3. **P1-03**：为Spline、point、segment和metadata lane定义stable identity。
4. **P1-04**：定义local/world coordinate、origin rebasing与大坐标策略。
5. **P1-05**：实现open/closed topology和最小point count验证。
6. **P1-06**：实现linear/constant/auto/clamped/broken/user tangent typed enum。
7. **P1-07**：定义position、tangent、roll、scale/up的单位、默认值和finite约束。
8. **P1-08**：实现typed point/segment metadata lane及cardinality验证。
9. **P1-09**：定义source digest、compiler version、settings与dependency digest合同。
10. **P1-10**：定义结构化diagnostic code、severity、stable object anchor和fix hint。
11. **P1-11**：实现确定性segment coefficient生成并覆盖boundary continuity。
12. **P1-12**：实现自适应弧长LUT、误差界和distance-to-parameter反查。
13. **P1-13**：实现rotation-minimizing frame并正确叠加authored roll/up override。
14. **P1-14**：实现position/tangent/frame/curvature按parameter与distance采样。
15. **P1-15**：实现closest point/offset/segment查询和确定性tie-break。
16. **P1-16**：生成per-segment AABB、aggregate bounds与BVH/grid artifact。
17. **P1-17**：提供batch/coherent closest query，避免WOC逐查询全段遍历。
18. **P1-18**：实现局部重编译与未变segment artifact复用。
19. **P1-19**：接入DDC key、cache hit/miss和artifact byte accounting。
20. **P1-20**：建立async compile generation、cancel、stale reject和last-known-good publish。
21. **P1-21**：World以immutable handle绑定artifact，替换不阻塞读线程。
22. **P1-22**：制定CPU scalar/SIMD与可选GPU tessellation职责，避免每帧重复烘焙。
23. **P1-23**：扩展component property descriptor为stable typed schema，而不是`value_type: String`。
24. **P1-24**：为property增加default、range、unit、asset-reference kind、validator和serialization flags。
25. **P1-25**：为plugin component descriptor增加schema version和migration hook。
26. **P1-26**：建立static project Scene到plugin component payload的统一序列化桥。
27. **P1-27**：建立plugin component payload到World/DynamicScene的preflight/transactional spawn。
28. **P1-28**：保证plugin-owned component save/reopen/play snapshot/export parity。
29. **P1-29**：定义plugin卸载时active source/instance/artifact/job/GPU资源的veto与清理。
30. **P1-30**：为Spline/Road/River/Decal/Geometry定义asset/resource kind或可验证custom kind owner。
31. **P1-31**：让asset registry/reference extractor理解所有typed dependency。
32. **P1-32**：实现schema migration golden与unknown-field/unknown-version明确失败。
33. **P1-33**：禁止通过raw JSON绕过domain validator；JSON只作为序列化载体。
34. **P1-34**：定义`RoadNetworkAsset`、`RoadProfileAsset`和stable junction/port/lane IDs。
35. **P1-35**：实现lane、shoulder、curb、camber、surface和marking typed profile。
36. **P1-36**：实现road surface/side/marking geometry与UV/material artifact。
37. **P1-37**：实现road collision cook、nav modifier和traffic lane graph receipts。
38. **P1-38**：实现terrain cut/fill/stamp、foliage exclusion及显式bridge/tunnel规则。
39. **P1-39**：实现junction connectivity validator和deterministic junction geometry。
40. **P1-40**：实现slope、curvature、bank、clearance与self-intersection diagnostics。
41. **P1-41**：实现Road partition/HLOD chunks与cross-cell ownership。
42. **P1-42**：迁移WOC 14条道路为versioned fixture并保留source digest/golden。
43. **P1-43**：用Spline query/BVH替代WOC生成X/Z分支和O(total segments)脚本扫描。
44. **P1-44**：定义`RiverAsset`、width/depth/flow/bank/bed与confluence metadata。
45. **P1-45**：实现river surface/bank/bed geometry和terrain carve receipts。
46. **P1-46**：实现flow/current artifact以及water renderer/simulation typed adapter。
47. **P1-47**：实现River collision/query、buoyancy、navigation、audio/VFX适配边界。
48. **P1-48**：实现逆坡、零宽、bank inversion、self-intersection和confluence诊断。
49. **P1-49**：River跨partition cell保持高度、流量、材质和simulation continuity。
50. **P1-50**：建立Decal material domain的单一enum/schema/compiler authority。
51. **P1-51**：`.zmaterial`持久化Decal domain并提供version migration。
52. **P1-52**：定义DecalProjector typed component：material、size/pivot、UV、sort、fade、mask、lifetime。
53. **P1-53**：实现Decal instance storage、generation、bounds和Scene/DynamicScene roundtrip。
54. **P1-54**：实现Decal frame extract、view culling、sorting、material grouping和batching。
55. **P1-55**：实现DBuffer/GBuffer通道写入及正确的lighting前pass位置。
56. **P1-56**：实现forward/transparent/mobile/MSAA/ray-tracing/virtual-geometry策略或显式unsupported。
57. **P1-57**：实现albedo/normal/ORM/emissive channel mask与receiver layer/mask。
58. **P1-58**：实现texture streaming、atlas/bindless allocation、eviction、pressure与fallback。
59. **P1-59**：移除空executor；registration test升级为executor command和GPU pixel golden。
60. **P1-60**：修正plugin feature placement/resource hazard合同，并以render graph validation覆盖。
61. **P1-61**：定义Geometry Shape/Brush source、primitive/profile和stable face/surface IDs。
62. **P1-62**：选择并封装robust boolean/extrusion/sweep kernel，定义失败diagnostic与确定性。
63. **P1-63**：Geometry输出mesh、UV/material surface、collision/nav和conversion artifact。
64. **P1-64**：Editor建立Spline point/segment/junction typed sub-selection identity。
65. **P1-65**：实现add/insert/delete/split/join/reverse/open/close及多选operation。
66. **P1-66**：实现tangent/roll/scale/width/projector box handles、snapping与numeric inspector。
67. **P1-67**：所有drag合并为transaction，支持Escape rollback、undo/redo、dirty/save/recovery。
68. **P1-68**：SceneMode通过受控document/transaction service工作，禁止直接改World/template。
69. **P1-69**：实现async preview、generation/stale/cancel/LKG、diagnostic anchor和progress receipt。
70. **P1-70**：建立save/reopen/play/export、remote operation audit、viewport screenshot/frame-capture和性能acceptance矩阵。

### 4.3 P2：完成主线后再扩展

1. **P2-01**：NURBS、rational weights与CAD-style import。
2. **P2-02**：Spline fitting、simplification、resampling与GIS polyline import/export。
3. **P2-03**：多人协作下point/segment stable-ID merge与冲突可视化。
4. **P2-04**：Road ruleset/grammar、批量生成、procedural placement与PCG adapter。
5. **P2-05**：lane marking library、traffic signal、intersection template和roundabout工具。
6. **P2-06**：bridge/tunnel specialist generator及structural clearance分析。
7. **P2-07**：River erosion、sediment、floodplain、seasonal flow和network simulation。
8. **P2-08**：Spline ribbon Decal、mesh Decal和terrain-only Decal specialized consumers。
9. **P2-09**：Decal virtual texturing、mega-atlas与GPU-driven indirect submission。
10. **P2-10**：Geometry Brush bevel、inset、face-level modeling和non-destructive modifier stack。
11. **P2-11**：legacy BSP import/conversion兼容层，前提是有真实项目需求。
12. **P2-12**：ML-assisted road/river fitting只能输出可审查source与deterministic compiler输入。

## 5. 分层实施路线

| Milestone | 交付内容 | 依赖 | 退出条件 |
|---|---|---|---|
| M0 · Truth Cutoff | hard-disable空Decal、移除/禁用false UI、产品catalog capability审计 | 无 | P0-01/P0-04/P0-05闭环 |
| M1 · Architecture & Baseline | owner/RFC、benchmark corpus、WOC migration fixture、reference parity matrix | M0 | 核心类型和performance budget批准 |
| M2 · Source Schema | SpatialSpline source、stable IDs、metadata、migration、property schema升级 | M1、Runtime04/05 | roundtrip/migration/fuzz通过 |
| M3 · Compiler & Query | coefficients、arc LUT、frames、bounds/BVH、batch query、DDC、LKG | M2 | 数值、determinism、query性能门通过 |
| M4 · Scene & Plugin Persistence | project Scene plugin payload、World/DynamicScene bridge、unload/reload | M2-M3、Plugin01 | save/reopen/play/export parity |
| M5 · Spline Editor | sub-selection、operations、handles、transaction、preview/compiler diagnostics | M3-M4、Editor02/03/05/09 | Editor端到端交互门通过 |
| M6 · Road & SplineMesh | profile、geometry、collision/nav/terrain、junction、partition、WOC迁移 | M3-M5、Editor16/19 | Road artifact与项目fixture门通过 |
| M7 · River / Water Authoring | river metadata、surface/bed/bank、flow adapters、partition continuity | M3-M5及独立Water runtime owner | River source-to-domain receipts通过 |
| M8 · Decal Vertical Slice | material domain、component、extract/cull/batch/render、Editor visualizer | M0、M2、M4、Runtime09D | GPU pixel/frame capture/Editor门通过 |
| M9 · Geometry Authoring | shape/boolean/extrusion/sweep、mesh/collision/nav、Editor operations | M3-M5 | deterministic geometry与failure diagnostics通过 |
| M10 · Scale & Optimization | partition/HLOD、SIMD/batch、GPU-driven paths、telemetry、stress | M6-M9 | budget和soak无回退 |
| M11 · Product Cutover | 删除WOC/generated road绕行和mock truth、docs/SDK/export/upgrade | M6-M10、Editor27 | 全矩阵、迁移、回滚、release gate通过 |

不得并行跳过M0、M2、M3或M4。Road、River和Geometry可以在共享Spline artifact稳定后由不同owner并行；Decal不依赖Spline，可以在Material/Scene persistence合同稳定后并行，但不能继续复用当前空executor作为“骨架完成”。

## 6. 验收门

1. **G01 · Truth Inventory**：所有Spline/Road/River/Decal/Geometry capability都有source owner、runtime owner、editor owner、artifact和receipt；否则catalog明确unsupported。
2. **G02 · No Silent No-op**：启用feature后不存在返回成功但不产生声明副作用的executor/operation。
3. **G03 · Source Recheck**：实施前重导145文件scope、重算fingerprint并审查全部在途差异。
4. **G04 · Schema Roundtrip**：各source在save/reopen后stable IDs、typed references和语义完全一致。
5. **G05 · Migration**：每个历史schema fixture可迁移或以明确版本错误拒绝；无silent default data loss。
6. **G06 · Determinism**：相同source/dependencies/settings/compiler生成byte-identical artifact和diagnostics顺序。
7. **G07 · Numeric Finite**：NaN/Inf、零长段、重复ID、非法closed topology在compile前被定位拒绝。
8. **G08 · Arc Length**：distance/parameter往返、总长和分段长在公布误差界内覆盖直线、曲线、闭环和极端scale。
9. **G09 · Frame Stability**：零曲率、拐点、近180度变化和闭环seam不产生NaN、随机翻转或不可解释twist。
10. **G10 · Closest Query**：BVH/batch结果与高精度oracle一致，tie-break跨平台稳定。
11. **G11 · Query Performance**：代表性10k/100k segment场景满足批准CPU latency/throughput和allocation budget。
12. **G12 · Incremental Compile**：单点编辑只重编译受影响segments/chunks，未变artifact cache命中可观测。
13. **G13 · Async Safety**：新generation完成后旧job不得覆盖；cancel/stale/LKG都有receipt和测试。
14. **G14 · Scene Persistence**：plugin component在project Scene、World、DynamicScene、Play snapshot与export之间保持一致。
15. **G15 · Plugin Lifecycle**：active instance/artifact/job/GPU resource阻止不安全卸载；安全卸载无悬挂ID或callback。
16. **G16 · WOC Migration**：14条道路从versioned asset加载，原距离/排除golden保持，生成X/Z分支和全段脚本扫描删除。
17. **G17 · Road Product**：surface、UV/material、collision、nav、terrain和lane graph共享同一accepted generation。
18. **G18 · Junction**：cross/T/Y/merge/roundabout fixture具有确定connectivity、geometry和失败diagnostic。
19. **G19 · Road Partition**：跨cell加载/卸载不丢段、不双生成、不裂缝，HLOD与collision/nav generation匹配。
20. **G20 · River Product**：width/depth/flow/bank/bed/confluence roundtrip并产生独立renderer/simulation/terrain/query receipts。
21. **G21 · River Continuity**：跨segment/cell/confluence的surface height、flow和bank continuity满足误差门。
22. **G22 · Material Authority**：Decal domain只由一个versioned enum/schema定义，Editor/UI/compiler/variant/pass完全一致。
23. **G23 · Decal Pixel**：albedo、normal、ORM和emissive channel golden在支持路径产生预期GPU像素。
24. **G24 · Decal Visibility**：frustum/layer/receiver/distance/angle/lifetime culling和sort order有CPU与frame-capture证据。
25. **G25 · Decal Scale**：1k/10k projector场景记录visible、culled、batch、atlas、CPU/GPU时间并满足批准budget。
26. **G26 · Decal Fallback**：forward/mobile/unsupported channel/atlas pressure均有结构化fallback或明确拒绝，无静默丢效果。
27. **G27 · Geometry Robustness**：boolean/extrusion/sweep对退化、自交、共面和大坐标fixture确定成功或给出stable diagnostic。
28. **G28 · Editor Transaction**：每次drag是一条可undo transaction；Escape恢复；save/reopen保留source；redo不换stable ID。
29. **G29 · Editor Selection**：point/segment/junction/projector handle的pick、box select、多选、隐藏/锁定和删除行为确定。
30. **G30 · Editor Async Preview**：拖动preview、commit build、cancel、stale reject、LKG和diagnostic click-through由真实job receipt驱动。
31. **G31 · Visual Evidence**：Windows真实Editor窗口截图、viewport capture和GPU frame capture验证非空、无mock数据、无overlay重叠。
32. **G32 · Release Matrix**：恢复编译基线后，Windows主lane、必要Linux lane、serialization/fuzz/GPU/perf/soak/export/upgrade矩阵全部归档。

## 7. 实施纪律

1. 本报告不授权在现有Decal文件里继续增加descriptor字段并保留空executor；M0必须先处理truthfulness。
2. 不接受`Vec<Vec3>`加线性采样作为正式Spline完成定义；可作为M1 benchmark prototype，但不得进入public product contract。
3. 不接受每个domain复制一套Hermite、arc length和closest-point数学；shared compiler必须单一owner。
4. 不接受用dynamic JSON property bag替代Road/River/Decal typed source；reflection是桥，不是domain schema。
5. 不接受把WOC道路生成脚本继续扩展成mesh/collision/nav owner；它只能作为迁移输入和回归fixture。
6. 不接受把`Biome_Riverbank`、`River_02`、Decal preview图或固定feedback作为功能证据。
7. 不接受在主线程拖动每个point时同步重建全路网、全NavMesh、全Terrain或全GPU atlas。
8. 不接受以Unreal API数量为成功标准；成功标准是本报告的source/artifact/runtime/editor和量化性能门。
9. 每个milestone实施后更新相关Runtime/Editor/Plugin/Tooling报告和coverage，不在本文件隐式吸收其他owner的失败。
10. 在真实基准证明前，不声称性能超过Unreal；目标可以更高，证据门不能更低。

## 8. 本轮输出

本轮只新增本review/refactor plan并更新优化索引与coverage，不修改生产代码或tests。当前最先应执行M0：阻断Decal空成功与Material false option；随后以WOC 14条Road作为M1/M2/M3的现实fixture，先建立共享SpatialSpline source/artifact/query，再让Road、River、Editor和Geometry消费者进入分层实现。
