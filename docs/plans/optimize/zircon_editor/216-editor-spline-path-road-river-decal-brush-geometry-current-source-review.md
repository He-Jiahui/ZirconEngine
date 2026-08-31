---
title: Editor Spline、Path、Road、River、Decal、Brush 与 Geometry 当前源码复核
category: zircon_editor
report_id: Editor216
review_date: 2026-08-29
baseline_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
verification_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor39
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
  - docs/plans/optimize/zircon_editor/113-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
  - docs/plans/optimize/zircon_editor/160-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
related_code:
  - zircon_plugins/rendering/features/decals
  - zircon_plugins/rendering/runtime/src
  - zircon_plugins/rendering/plugin.toml
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/assets/shader
  - zircon_runtime/src/graphics/material
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset
  - zircon_runtime/src/core/framework/scene/component_type_descriptor
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_editor/src/scene
  - zircon_editor/src/ui/binding_dispatch/inspector
  - zircon_editor/src/tests/editing/reflected_command.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui
  - examples/woc/contracts/m3_terrain_content.json
  - examples/woc/scripts/woc_game/src/generated/contracts.zr
  - examples/woc/scripts/woc_game/src/world/decoration_candidate.zr
plan_sources:
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
  - docs/plans/optimize/zircon_editor/113-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
  - docs/plans/optimize/zircon_editor/160-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zw-runtime-decal-projector-material-domain-dbuffer-gbuffer-forward-receiver-culling-batching-atlas-streaming-temporal-rt-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zr-runtime-water-ocean-lake-river-surface-wave-fft-shallow-water-rendering-underwater-buoyancy-query-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/138-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SplineComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SplineMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Editor/ComponentVisualizers/Private/SplineComponentVisualizer.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source/Runtime/Public/WaterSplineMetadata.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source/Runtime/Public/WaterBodyRiverComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Classes/LandscapeSplineControlPoint.h
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Classes/LandscapeSplineSegment.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/DecalComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/CompositionLighting/PostProcessDeferredDecals.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/BrushComponent.h
  - dev/godot/scene/resources/curve.h
  - dev/godot/scene/3d/path_3d.h
  - dev/godot/editor/scene/3d/path_3d_editor_plugin.cpp
  - dev/godot/scene/3d/decal.h
  - dev/godot/modules/csg/csg_shape.h
  - dev/Fyrox/fyrox-math/src/curve.rs
  - dev/Fyrox/editor/src/plugins/curve_editor.rs
  - dev/Fyrox/fyrox-impl/src/scene/decal.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/decal.shader
  - dev/bevy/crates/bevy_math/src/cubic_splines/mod.rs
  - dev/bevy/crates/bevy_pbr/src/decal/clustered.rs
  - dev/bevy/crates/bevy_pbr/src/decal/forward.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal/DecalProjector.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal/DecalSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Material/Decal/DecalProjectorEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Water/WaterDecal/WaterDecal.cs
finding_status:
  p0_open: 5
  p0_partial: 0
  p0_closed: 0
  p1_open: 56
  p1_partial: 14
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 19
  partial: 13
  pass: 0
---

# 216 · Editor Spline / Path / Road / River / Decal / Brush / Geometry 工程化差距

## 1. 结论

Editor39 的 canonical 结论仍成立。当前 Zircon 没有可作为项目真值的空间 Spline、PathFollow、SplineMesh、Road、River、WaterBody、Geometry Brush、CSG 或 Decal Projector 产品链。对 19,662 个 zircon_app、zircon_editor、zircon_hub、zircon_plugins、zircon_reflect_derive、zircon_runtime、zircon_runtime_host 与 zircon_runtime_interface 生产 Rust/TOML/ZUI 文件做精确合同扫描，SpatialSplineSource、SplinePointId、SplineSegmentId、CompiledSplineArtifact、PathFollow、SplineMesh、RoadSource、RiverSource、WaterBodySource、GeometryBrushSource、CsgProgramArtifact 与 DecalProjectorSource 十二个目标合同全部为零命中。

现有 CubicSpline、Hermite 和曲线数学只能证明局部插值能力。它们没有稳定点段身份、弧长重参数化、空间标架、最近点和重叠查询、分段边界与索引、版本化 source、可安装 artifact、Scene persistence 或 Editor 控制点事务，因此不能被升级命名为空间 Spline 产品。DynamicScene 与 World dynamic component 已有 schema、反射、Inspector 编辑和 roundtrip 底座，但 SceneEntityAsset 仍以 camera、mesh、light、post_process、physics、animation、terrain、tilemap、prefab 和 script_bindings 等硬编码 Option 表达项目场景；通用 World 成功不等于 Project Scene、Prefab 与发布包闭环。

Decal 继续是最严重的能力失真。zircon_plugins/rendering/features/decals/runtime/src/lib.rs 注册 DecalProjector descriptor、PostProcess pass 与 executor，executor 却只返回 Ok(())；descriptor 的 mode、opacity、normal_blend 和 atlas_region 仍使用 String，没有 typed material/texture/atlas handle、Scene owner、extract、cull、batch、attachment、shader 或 draw。与此同时 zircon_plugins/rendering/plugin.toml 仍把 umbrella package 标为 stable，并把 capability 状态声明为 complete。Editor crate 只有 descriptor/capability 和无人消费的 drawer ID；Material Workbench 暴露 surface、post_process、decal，而 Runtime MaterialDomain 只有 Surface、PostProcess、DebugOverlay 与 LightFunction。注册、下拉项和 pass 名均不能证明像素能力。

WOC 仍以项目脚本绕过引擎级 Road。decoration_candidate.zr 的 roadDistance 对道路与点段做嵌套遍历，以 roadPointX/Z 读取 XZ 点，并用小于 5.0 的裸阈值筛选候选；它没有 Road/Segment identity、弧长、三维高度与标架、宽度和车道、mesh/UV/material、collision/nav、junction、terrain stamp 或 streaming artifact。当前 m3_terrain_content.json 的 SHA-256 是 C481FAAA10CC8B8F136A36DE053015C486A15FC90E95687818906A2537FCC29E；迁移必须以当前 source digest 和可重放编译结果为基线，不能沿用旧报告已经漂移的摘要值。

目标边界保持为：

~~~text
SpatialSplineSource + stable point/segment IDs
  -> deterministic validation/compiler
  -> CompiledSplineArtifact + immutable indexed query view
  -> PathFollow / SplineMesh / Road / River / TerrainStamp adapters

GeometryBrushSource + stable shape graph
  -> deterministic CSG compiler
  -> topology/mesh/collision artifact + provenance

DecalProjectorSource + DecalMaterial source
  -> Scene install + extract/cull/batch
  -> DBuffer/GBuffer/Forward render artifact + pixel evidence
~~~

## 2. 审查范围与方法

### 2.1 当前工作树选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与证据 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin/WOC selected | **891 / 56,334 / 51,373 / 1,973,069 / 355 / 0** | Decal、Material/Scene/Shader、dynamic component、Editor Scene/Inspector 与 WOC；edbecd3d61486a610d29bb593ef7da95b55c8b14c98005c8dfa2f550af7f40cf |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **26 / 18,630 / 15,668 / 742,536 / 12 / 0** | Spline/Path/Road/River/Decal/Brush/CSG 与编辑器；252b069e7c8c21dccdb1e19a563218a18d3694466fc16d4a442e46e9074929cb |
| 全部选择集 | **917 / 74,964 / 67,041 / 2,715,605 / 367 / 0** | 两组按规范化相对路径去重；78f051a4ab63a5ec48b9305997f0336cabbd4fa4c1892bc9ea7a3fd71a4a4ecb |

Zircon 选择集由 examples 3、zircon_editor 160、zircon_plugins 12、zircon_runtime 716 个文件组成。Tooling 按用户要求排除；examples 只纳入 WOC 运行时消费证据，不纳入 examples/woc/tools。统计按当前共享工作树物理内容生成，不能代表整个仓库已经逐文件审完，也不能在未来实施时替代 source recheck。

### 2.2 判定规则

1. Open 表示领域 source、owner、consumer 或资格证据缺失；同名数学函数、descriptor、UI 文本和通用框架不能抵消。
2. Partial 只授予已存在且有明确消费证据的通用底座，例如 DynamicScene roundtrip、Inspector transaction、operation/job、asset/artifact 与 generation。
3. Closed 或 Pass 必须具备 source -> compile -> artifact -> install -> runtime consumer -> Editor transaction -> fault/scale/platform evidence；本轮没有领域项达到该标准。
4. 静态 ZUI、manifest capability、drawer ID、pass registration、空 executor、日志和测试 fixture 均不是产品完成证据。
5. 本轮是静态 review，不修改 production/tests，不运行 Cargo、Editor 或 GPU 动态矩阵。

## 3. 相对 Editor160 的当前重判

| 领域 | 当前源码事实 | 状态 |
|---|---|---|
| Spatial contracts | 十二个目标产品合同在 19,662 个生产文件中零命中；动画/glTF 曲线仍无空间 identity/query/artifact | P0-1 Open |
| Decal runtime | executor 仍为直接 Ok(())，无 extract/cull/batch/draw；manifest 仍 stable/complete | P0-2 Open |
| Material/Editor | Workbench 仍暴露 decal，Runtime MaterialDomain 仍无 Decal；Editor feature 无 extension consumer | P0-3 Open |
| Project Scene | Dynamic component 底座可保留，但 SceneEntityAsset 仍是硬编码字段集合 | P0-4 Open |
| WOC Road | roadDistance、roadPointX/Z、嵌套 segment loop 与 5.0 阈值仍在 | P0-5 Open |
| Tooling | 本轮选择集和产品判定完全排除 tools 与 examples/woc/tools | 不形成 finding |

没有证据支持关闭或降级任何父 finding。Editor216 只刷新当前源码事实，不新增 canonical finding 数量。

## 4. 当前 Zircon 产品链差距

### 4.1 Spatial Spline 与查询合同

1. 缺少 versioned SpatialSplineSource、stable point/segment IDs、schema migration 和 source revision。
2. 缺少空间控制点的 interpolation、tangent、roll、scale、width、up policy、metadata 与 closed-loop seam 合同。
3. 缺少 finite、duplicate、degenerate、自交、坏切线和 seam 早拒绝，以及可定位到点段的 structured diagnostic。
4. 缺少 arc-length table、distance/parameter 双向映射、curvature、minimum-twist frame 与 deterministic nearest query。
5. 缺少 segment bounds、BVH/grid/chunk index、immutable query view、generation fence 和 batch budget。

Bevy 的 cubic spline 模块可作为曲线族、生成器和泛型数学参考，但不提供 Zircon 所需的 Scene/Editor/asset ownership。Unreal SplineComponent 同时维护 curve data、reparam table、closed loop、up direction 与位置/切线/旋转/缩放查询，说明工程级合同不能缩减为一组 Vec3。Godot Curve3D 的 baked distance、tilt、up vector、closest point 与 tessellation，以及 Fyrox curve key identity，进一步证明持久化和查询必须属于同一版本链。

### 4.2 Path、SplineMesh、Road、River 与 Terrain

PathFollow 必须消费 immutable spline query，并定义 distance、offset、frame、loop、speed、authority、network 和 replay，而不是逐帧在 source Vec 上临时插值。SplineMesh 必须编译 profile、deformation、twist/scale、UV、material、LOD 与 collision，并持有 source/artifact provenance。

Road 不是 Spline 的别名。它需要 lane、width/shoulder/curb、bank、junction、surface、traffic metadata，以及 mesh/collision/nav/terrain conform/stamp/decal/streaming artifacts。Unreal LandscapeSpline 的 control point/segment、terrain/navigation 和 mesh 责任展示了这一跨域边界；WOC 当前全量线段查询必须迁入统一索引和 typed result。

River/WaterBody 需要 bank、flow/current/depth、shoreline、foam 和 transition source，由 Runtime99zr 消费。Unreal WaterSplineMetadata 与 WaterBodyRiverComponent 表明宽度、深度、流速等 metadata 不能散落在 Editor 或 renderer；水面、物理、音频、天气、导航、反射与折射必须通过 generation-qualified adapters 和 receipts 接入。

### 4.3 Geometry Brush 与 CSG

当前没有 GeometryBrushSource、stable shape graph、boolean/extrusion program 或 CsgProgramArtifact。Terrain/Foliage paint brush、UI brush 与普通 mesh primitive 都不是 Geometry Brush。

工程级 CSG 至少需要 primitive/union/subtract/intersect/extrusion、deterministic topology、manifold/degenerate/self-intersection validation、UV/material regions、collision bake、provenance、cancel/rollback 与大网格预算。Godot CSG shape 层次和 path extrusion可作语义参考；Unreal BrushComponent 可作 Scene component 与 bounds/collision 生命周期参考。实现不得把同步 boolean 塞进 render thread 或 Editor callback。

### 4.4 Decal Runtime 与能力真相

当前 Decal pass 读取 scene-depth 与 scene-color、写 scene-color，但 executor 不录制 GPU 命令。这不只是缺少优化，而是没有功能。descriptor 的字符串字段也无法表达 typed source、material variant、channel mask、texture residency、atlas allocation、receiver mask、fade、sort、lifetime 和 instance identity。

Unreal deferred decal 路径按 DBuffer/GBuffer stage、blend mode、receiver 与可见性组织工作；Bevy clustered/forward decal 把 extract、GPU buffers、capability 与 binding 分开；Unity HDRP DecalProjector/DecalSystem 维护 material、fade、UV、layer、scale、cull/update/draw，Inspector 还负责兼容性校验；Fyrox Decal 具备 reflected/serialized scene node、bounds 和真实 shader。Zircon 当前只达到注册形状，尚未达到其中任何一套的最小 runtime 闭环。

### 4.5 Editor、Material、Scene 与插件边界

Decal editor crate 必须注册真实 extension、factory、AssetType、component creator、Inspector customization、gizmo/visualizer 与 toolkit；drawer 常量本身不是消费链。Material Workbench 的 decal 选项必须由 Runtime domain registry 和 capability admission 派生，不能保留一个 Runtime 无法编译的值。

Godot Path3D editor 的 point/tangent handles、selection 与 undo，以及 Fyrox curve editor 的 document/command/save 生命周期，说明 Spline authoring 必须进入统一 document transaction。Project Scene carrier 必须由 Runtime Scene owner提供 version、schema、unknown-field、dependency 和 migration；不得继续给 SceneEntityAsset 为每个插件领域追加硬编码 Option。

## 5. 五套参考源码对照

| 参考 | 已具备的工程合同 | Zircon 当前缺口 |
|---|---|---|
| Unreal | Spline reparam/query、SplineMesh deformation/collision、Visualizer transaction、Landscape Road、Water metadata/River、deferred Decal、Brush | source/artifact/query/editor/runtime 全链缺失 |
| Godot | Curve3D bake/closest/frame、PathFollow、Path editor undo、typed Decal textures/fades、CSG boolean/path | 空间 persistence、工具和 CSG/Decal consumer 缺失 |
| Fyrox | curve key identity、curve document command/save、serialized Decal node/bounds/shader | stable IDs、document lifecycle与真实 shader 均未接入 |
| Bevy | Bezier/Hermite/Cardinal/B-spline/NURBS math、clustered/forward decal extract/GPU binding | 数学只局部存在，Decal extract/buffer/capability consumer 缺失 |
| Unity Graphics | HDRP DecalProjector material/fade/UV/layer、DecalSystem cull/draw、Inspector validation、WaterDecal | typed projector、system owner、validation和water adapter 缺失 |

参考源码用于抽取 owner、数据流、生命周期、失败语义与资格门，不要求复制 API。Unreal 是本领域重型主参考；Godot/Fyrox 重点验证 Editor 与持久化；Bevy 重点验证数学和 extract；Unity Graphics 重点验证 Decal renderer 与 Inspector。

## 6. Authority 与所有权

1. Editor39 继续是本主题唯一 canonical finding owner；Editor216 只刷新 currentness。
2. Spatial source/compiler/query 应由 Runtime geometry/scene owner持有，Editor只持 document、transaction、tool state 与 projection。
3. Decal GPU 路径由 Runtime99zw 关闭；Editor不得私建 renderer 或用 preview 冒充 draw。
4. River/WaterBody render、physics、buoyancy 与 query 由 Runtime99zr 关闭；Editor只编辑 source 和 adapter projection。
5. Terrain stamp、clipmap、physics/navigation 与 world partition 由 Runtime99zq/Editor138 协同关闭。
6. Material domain/shader artifact 必须并入既有 Runtime Material 与 Editor Material owner，不新增第二套 Decal schema。
7. Scene carrier、Prefab/DynamicScene 与 project I/O 由 Runtime Scene owner定义；Editor只提交可撤销事务。

## 7. P0：必须先关闭的断路（5 Open）

| ID | 状态 | 差距 | 首个关闭条件 |
|---|---|---|---|
| P0-1 | Open | 空间 Spline/Road/River/Brush 没有 source、stable IDs、compiler、artifact、query、Scene carrier 或 toolkit | M0 固化 owner；M1 交付 versioned source、compiled artifact 与 indexed immutable query |
| P0-2 | Open | Decal plugin 注册成功但 executor 不提交 GPU 工作，stable/complete 构成假完成 | 立即 fail-close；Runtime99zw 通过 extract/cull/batch/render/pixel gate 后才恢复 capability |
| P0-3 | Open | Decal drawer、Material dropdown 与 Runtime material/Scene/render authority 分裂 | 删除或禁用第二 authority，建立单一 Decal source/material/component/document/receipt 链 |
| P0-4 | Open | Dynamic World roundtrip 已有基础，但 Project Scene/Prefab 没有通用 versioned plugin/domain carrier | Runtime Scene 提供 schema/version/unknown-field/dependency/migration carrier 和项目文件 roundtrip |
| P0-5 | Open | WOC 使用 XZ codegen、全量 segment loop 与裸阈值，且没有引擎级 Road consumer | 迁移相同 source digest 到 compiled spatial artifact，接通 Road query/mesh/nav 后删除旧分支 |

## 8. P1：Runtime、Geometry、Decal 与 Editor（56 Open / 14 Partial）

| # | 状态 | 需要重构的内容 |
|---:|---|---|
| 1 | Open | 定义 versioned SpatialSplineSource、control point/segment stable IDs 与 migration。 |
| 2 | Open | 支持 Bezier/Hermite/Catmull/linear、tangent/roll/scale/interpolation 与 closed loop。 |
| 3 | Open | 建立 finite/duplicate/degenerate/self-intersection/seam validation 与结构化 diagnostics。 |
| 4 | Open | 编译 arc-length table、parameter/distance mapping、curvature 与 frame orientation。 |
| 5 | Open | 定义 minimum-twist/parallel-transport frame、up policy、banking 与 handedness。 |
| 6 | Open | 编译 segment bounds、BVH/grid/chunk index 与 deterministic nearest query。 |
| 7 | Open | 提供 point/tangent/normal/roll/width/metadata/nearest/overlap batch API。 |
| 8 | Open | 建立 immutable query view、source revision、world generation 与 late-completion fence。 |
| 9 | Open | Scene/Prefab/Instance/World save/load 保留 spline identity/source/artifact dependency。 |
| 10 | Open | Editor control-point/tangent/roll/width handles、snap、multi-select 与 undo/redo。 |
| 11 | Open | 建立 Spline AssetType、document、dirty/save/autosave/recovery/conflict lifecycle。 |
| 12 | Open | PathFollow 消费 spline query，支持 distance/offset/frame/loop/speed/authority/network/replay。 |
| 13 | Open | SplineMesh compiler 支持 profile、twist/scale、UV、material、LOD 与 collision。 |
| 14 | Open | Road source 支持 lane、width/shoulder/curb、bank、junction、surface 与 traffic metadata。 |
| 15 | Open | Road mesh/collision/nav/terrain conform/stamp/decal/stream artifacts 携带 provenance。 |
| 16 | Open | River/WaterBody source 支持 bank、flow/current/depth、shoreline、foam 与 transition。 |
| 17 | Open | Water renderer/physics/audio/weather adapters、LOD、reflection/refraction 有 typed receipt。 |
| 18 | Open | Terrain stamp/extrusion source 使用 spline IDs、falloff、height/material mask。 |
| 19 | Open | Geometry Brush 支持 primitive、union/subtract/intersect、extrusion 与 topology validation。 |
| 20 | Open | CSG kernel deterministic，并诊断 manifold/degenerate/self-intersection/UV/material region。 |
| 21 | Open | Brush history/transaction/preview/bake 支持 cancel、rollback 与 large-mesh budget。 |
| 22 | Open | WOC migration 保留当前 JSON digest determinism 并生成 compiled artifact。 |
| 23 | Open | 删除 roadPointX/Z codegen、O(total segments) 查询与项目裸阈值。 |
| 24 | Open | Road query 返回 arc length/lane offset/height/frame/road/segment/metadata stable result。 |
| 25 | Open | Road index 支持 tile/cell streaming、batch/SIMD/cache/LOD/async query budget。 |
| 26 | Open | Road/River junction graph 接通 intersection、AI/nav 与 traffic adapters。 |
| 27 | Open | DecalProjector 使用 typed component/source/material/texture/atlas reference schema。 |
| 28 | Open | Decal 完成 Project Scene、DynamicScene、Prefab/World/PIE install 与 migration。 |
| 29 | Open | Decal material compiler 增加真实 domain/channel validation、variant 与 artifact。 |
| 30 | Open | Render path 按 capability 选择 DBuffer/GBuffer/forward，并声明 attachment/lifetime/hazard。 |
| 31 | Open | Projector frustum/cluster culling、receiver mask、distance/angle fade、sort 与 lifetime。 |
| 32 | Open | GPU instance/batch/atlas/bindless/streaming/eviction 与 mobile/MSAA fallback。 |
| 33 | Open | Normal/roughness/metallic/emissive/opacity blend 与 depth/receiver policy。 |
| 34 | Open | Lighting/shadow/reflection/temporal/upscale/render-order 产生可查询 receipt。 |
| 35 | Open | Pixel/visual golden、stats、unsupported diagnostics 与 device-loss recovery。 |
| 36 | Open | Decal Editor plugin 注册真实 extension、factory、AssetType、component creation 与 toolkit。 |
| 37 | Open | Projector gizmo/visualizer/pick/selection/overlay/preview world 与 camera preview。 |
| 38 | Open | Decal material editor 把 source/document/compile/preview receipt 接到 Runtime material。 |
| 39 | Open | 用真实 domain registry/capability admission 替换固定 Material decal dropdown。 |
| 40 | Open | Create/Open/Import/Reimport/Validate/Compile/Apply/Playtest 有真实 factory/controller。 |
| 41 | Partial | 通用 plugin/module/catalog/SPI 已存在；仍需为空间与 Decal 闭合 URI、factory、controller、service 与 App。 |
| 42 | Partial | Dynamic component 已有 schema/editable/generation；仍缺 typed default、field ID、range/unit、asset kind、schema version。 |
| 43 | Open | Static Project Scene authoring 必须支持通用 plugin component payload，而非继续添加硬编码 Option。 |
| 44 | Partial | 通用 Prefab/Scene lineage 基础可复用；空间 domain 的 override/remap/merge/unknown field 尚未建立。 |
| 45 | Partial | Editor 已有 selection/document/transaction generation 基础；geometry/decal publish 仍无领域 generation。 |
| 46 | Partial | 通用 operation/job 有 bounded/cancel/shutdown 基础；geometry/decal compiler job 与 atomic publish 尚无。 |
| 47 | Partial | 通用 asset/artifact/cache 基础存在；领域 key、bulk chunk、GC、platform/quality/algorithm 维度尚无。 |
| 48 | Partial | 通用 diagnostic/journal 可复用；缺 point/segment/shape/material/generation/remediation 定位。 |
| 49 | Open | malformed point/curve/boolean/texture/huge extent/path fuzz 与预算拒绝。 |
| 50 | Open | Scene/Prefab/undo/save/reimport/hot-reload/stream-unload roundtrip golden。 |
| 51 | Open | Spline numeric golden 覆盖 arc length/frame/nearest/curvature/twist/degenerate。 |
| 52 | Open | Road/River mesh/collision/nav/terrain/UV/material/flow visual 与 data golden。 |
| 53 | Open | Decal pixel golden 覆盖 DBuffer/GBuffer/forward、channels、MSAA/mobile/device matrix。 |
| 54 | Open | WOC Road 1/1k/100k segment compile/query 记录 p50/p95/p99 与内存界限。 |
| 55 | Open | 大量 Decal、atlas pressure、overdraw、streaming 与 frame hitch benchmark。 |
| 56 | Open | Network/replay/save 对 Path/Road/Decal/Brush 状态、authority 与 determinism 完整。 |
| 57 | Partial | Runtime 不依赖 Tooling，但没有可证明 headless cook 消费真实 geometry/decal artifact。 |
| 58 | Partial | first-party catalog/App 装配框架存在；Decal runtime no-op、Editor empty 使 provider/factory closure 不成立。 |
| 59 | Partial | 通用 release/artifact manifest 可复用；未包含 geometry/decal/material 依赖与平台支持/provenance。 |
| 60 | Open | handedness/float/texture/DBuffer/render graph/nav/physics 跨平台结果无领域证据。 |
| 61 | Partial | plugin unload guard 与 registry generation 存在；领域 upgrade/migration/rollback/stale renderer 未验证。 |
| 62 | Open | Road/River 与 Weather/Region/Navigation/Physics 仍无 typed adapter boundary。 |
| 63 | Partial | 通用 dependency/invalidation 基础存在；空间、Brush、Decal 的精确 artifact invalidation 尚无。 |
| 64 | Open | Editor preview 必须使用真实 artifact/camera，替换固定值与静态反馈。 |
| 65 | Partial | 通用 document/diff 基础可复用；point/segment/material/shape/channel/byte semantic diff 尚无。 |
| 66 | Open | 多用户 field-level merge/lock/presence/review 对空间文档尚无合同。 |
| 67 | Open | self-intersection、bad frame、missing width/material、overdraw/overlap/budget 自动审计尚无。 |
| 68 | Partial | DynamicScene/asset migration 基础存在；领域 canary、old-generation pin、rollback/replay compatibility 尚无。 |
| 69 | Open | Unreal/Godot/Fyrox/Bevy/Unity 的 spline/decal/water 对照仍缺可执行统一 fixtures。 |
| 70 | Open | Stable/Complete 仍可在 no-op provider 下成立，未由 compile/runtime/editor/visual/fault/scale/platform evidence 派生。 |

## 9. P2：长期能力（12 Open）

1. GPU spline/road tessellation、meshlet/virtual geometry、bindless material 与 culling。
2. Procedural road/river network、traffic lanes、junction solve 与 terrain excavation。
3. Water simulation、waves/foam/erosion、shoreline climate/weather interaction。
4. SDF/voxel/sparse CSG、incremental boolean、topology repair 与 bake farm。
5. Decal virtual texturing、clustered GPU culling、ray tracing 与 neural material masks。
6. Dynamic destruction/runtime Brush 与 deterministic replicated geometry edits。
7. Remote geometry/decal build farm、quality/RDO/perceptual optimization 与 cache federation。
8. World partition streaming/patch/rollback 支持超大 Road/River/Decal 数据集。
9. 多用户 semantic merge 的 Spline/Brush/Road/Decal collaborative authoring。
10. GIS/CAD/road/river/terrain 数据导入，包含 CRS、coordinate 与 provenance。
11. Procedural animation/path constraints/vehicle/AI/nav 与 replay/time scrubbing。
12. Cross-engine conformance scenes 与公开的 geometry/render/performance benchmark 方法。

## 10. 分层重构顺序

### M0：MVP 基线、Truthfulness 与 Owner 冻结

先服从 docs/plans/mvp/00 的 workspace/build 基线。Decal capability fail-close，Material 的 unsupported domain 不再暴露；冻结 Spatial、Road、Geometry、Decal、Water、Terrain owner 和跨层接口。M0 不实现高级 UI。

### M1：Spatial Source、Compiler 与 Query

交付 stable IDs、versioned source、validation、arc-length/frame/index compiler、immutable query artifact、generation fence 与 Project Scene/Prefab carrier。先以 numeric/property/roundtrip 证明合同。

### M2：Spline Editor 与 Path Consumer

接入 AssetType/document/transaction、control point/tangent/roll/width handles、selection/snap/undo 与真实 preview；实现 PathFollow 和 SplineMesh 的首批真实 consumer。

### M3：Road、Terrain 与 WOC 迁移

建立 Road source/profile、mesh/UV/material/collision/nav/terrain-stamp artifacts 与 indexed query；以当前 WOC digest 对比迁移结果，最后删除 X/Z 分支和裸阈值。

### M4：River 与 WaterBody Adapter

在 Runtime99zr owner 下接通 River bank/flow/depth/shore source、water render/physics/audio/nav/weather adapters，不在 Editor 私建 water truth。

### M5：Geometry Brush 与 CSG

交付 shape graph、boolean/extrusion compiler、topology diagnostics、preview/bake artifact、cancel/rollback/budget，再接入 Editor handles 与 transaction。

### M6：Decal Source、Material 与 Scene

统一 DecalProjector source、Decal material domain、typed texture refs、Scene/Prefab/DynamicScene install、Editor creation/Inspector/gizmo/document 与 compile receipt。

### M7：Decal Render

由 Runtime99zw 完成 DBuffer/GBuffer/forward attachments、extract/cull/batch/atlas/streaming、fallback、device-loss、stats 与 pixel golden。未通过前 capability 保持 unsupported。

### M8：Catalog、Preview 与产品操作

闭合 first-party runtime/editor provider、App feature、create/open/import/reimport/validate/compile/apply/playtest factories；所有 preview 使用真实 artifact 与 qualified runtime receipt。

### M9：Fault、Scale、Release 与竞争基线

完成 malformed/fuzz、roundtrip、fault、1/1k/100k scale、large decal、cross-platform/headless package、rollback、visual/data/pixel golden 与公开 cross-engine benchmark。

## 11. 验收门禁（19 Fail / 13 Partial / 0 Pass）

| # | 状态 | 门禁 |
|---:|---|---|
| 1 | Fail | source/control/segment/artifact/query/instance IDs 与 revision/generation 完整。 |
| 2 | Fail | curve finite/degenerate/self-intersection/frame/arc-length 早拒绝并诊断。 |
| 3 | Fail | nearest/distance/parameter/frame/curvature query deterministic、indexed、batch、bounded。 |
| 4 | Partial | World/DynamicScene/Inspector roundtrip 已有基础；Project Scene/Prefab spatial identity 仍缺。 |
| 5 | Fail | PathFollow/SplineMesh/Road/River/Water/TerrainStamp 消费真实 query/artifact。 |
| 6 | Fail | Road mesh/UV/material/collision/nav/lane/junction/terrain/stream artifacts 完整。 |
| 7 | Fail | River flow/depth/shore/foam/reflection/physics/audio/weather adapters 有 receipt。 |
| 8 | Fail | WOC digest 迁入 compiled artifact 且旧 codegen 删除。 |
| 9 | Fail | Decal descriptor/component/source/material/texture/atlas 与 Scene/Prefab install 闭合。 |
| 10 | Fail | Decal DBuffer/GBuffer/forward attachment/order/lifetime/hazard 正确。 |
| 11 | Fail | Decal cull/batch/instance/atlas/streaming/receiver/blend/device fallback 正确。 |
| 12 | Fail | Decal runtime pixel/visual golden 能证明真实 draw，而非 registration success。 |
| 13 | Fail | Material Decal domain、variant、compiler/artifact、texture ref 与 renderer 一致。 |
| 14 | Partial | Dynamic component 已有 schema/reflection/World roundtrip；default/version/dependency/unknown field 不完整。 |
| 15 | Partial | Editor document/transaction/selection/SceneMode/overlay 基础存在；领域 toolkit/preview/diagnostic 未闭环。 |
| 16 | Fail | 领域 create/open/import/reimport/validate/compile/apply/playtest factory/controller 可执行。 |
| 17 | Partial | 通用 operation/job 有界与取消基础可复用；领域 job/atomic publish 未验证。 |
| 18 | Partial | 通用 JSON finite rejection 与资源预算存在；空间/CSG/Decal malformed fuzz 尚无。 |
| 19 | Partial | 通用 asset/artifact/cache/rollback 基础可复用；领域 key/chunk/provenance 未定义。 |
| 20 | Fail | WOC/Road/Decal/Brush 1/1k/100k 与 CPU/GPU/memory/hitch 达标。 |
| 21 | Partial | 多 World、inspection、resource generation 基础存在；领域 late publish fence 尚无。 |
| 22 | Partial | plugin/catalog/App admission 框架存在；Decal no-op 与 empty Editor 使产品 closure 失败。 |
| 23 | Partial | 通用 undo/autosave/recovery/conflict 基础存在；空间文档 roundtrip/merge/reimport 尚无。 |
| 24 | Partial | 通用 dependency graph 可复用；领域依赖与精确 invalidation 尚无。 |
| 25 | Fail | visual/data/GPU golden 覆盖 frame、curve、Road、Water、Decal channel、MSAA/mobile。 |
| 26 | Partial | 通用 diagnostic journal 可复用；point/segment/shape/material/generation 维度尚无。 |
| 27 | Fail | GIS/CAD/terrain coordinate/provenance、handedness 与 platform precision 通过。 |
| 28 | Partial | 通用 schema/plugin migration 基础存在；领域 canary/pin/rollback/replay 尚无。 |
| 29 | Partial | Runtime 不依赖 Tooling；但关闭 Editor 后尚无真实 geometry/decal artifact 可消费。 |
| 30 | Fail | cross-engine reference scene 方法公开且质量/性能/内存/overdraw 可比较。 |
| 31 | Fail | unsupported shape/backend/feature 在 admission 早失败；当前 Decal no-op 违反。 |
| 32 | Fail | Stable/Complete 只由 compile/runtime/editor/visual/fault/scale/platform evidence 派生。 |

## 12. 禁止的临时修补

1. 禁止继续用 WOC roadPointX/Z、全量 segment loop、裸距离阈值代替 Spline/Road artifact。
2. 禁止只添加 Vec<Vec3>、控制点按钮或 Hermite 名称，而没有 stable IDs、frame、arc length、index、transaction 与 query。
3. 禁止将 glTF/动画 Hermite、Terrain/Foliage paint brush 或 UI curve 称为空间 Spline/Geometry Brush。
4. 禁止将 Decal descriptor、pass name、executor Ok(())、feature available 或 umbrella stable 当作像素能力。
5. 禁止只增加 Decal enum、Material dropdown、drawer ID 或 manifest capability。
6. 禁止让 Editor 私有 Scene/Material/Water/Geometry 真值；必须通过 Runtime-owned versioned source/artifact/gateway。
7. 禁止继续给 SceneEntityAsset 为每个 plugin domain 增加硬编码 Option；先设计通用 carrier、unknown-field 与 migration。
8. 禁止在 render thread 同步 tessellate Road、执行 CSG、读文件或提交无界 Decal instances。
9. 禁止用 registration snapshot、静态 preview 或单元属性测试替代 numeric/data/visual/pixel/fault/scale 门禁。
10. 禁止在未重算 917-file 选择集和未复核源漂移前开始实现。

## 13. 本轮产出边界

本轮只完成 current-source review、参考引擎对照、差距分类与重构顺序，不修改 Zircon production/test，不运行 Cargo、Editor、GUI、GPU、physics、navigation、water、cook 或动态 benchmark。Editor39 仍是 canonical owner；5 项 P0、70 项 P1、12 项 P2 和 32 项门禁均未因本报告自动关闭。
