---
title: Editor Spline、Path、Road、River、Decal、Brush 与 Geometry 当前源码复核
category: zircon_editor
report_id: Editor160
review_date: 2026-08-27
baseline_head: 2fc6945c5a858b3330a6133019985831e95ae83a
baseline_epoch: 572
verification_head: 5a0a44b7a169e3d03a85b235251f8113802f2ea3
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor39
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
  - docs/plans/optimize/zircon_editor/113-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
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
  - tools/editor-workbench-preview/design.js
  - examples/woc/contracts/m3_terrain_content.json
  - examples/woc/tools/m3_decoration_candidate_source_extract.mjs
  - examples/woc/scripts/woc_game/src/generated/contracts.zr
  - examples/woc/scripts/woc_game/src/world/decoration_candidate.zr
plan_sources:
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
  - docs/plans/optimize/zircon_editor/113-editor-spline-path-road-river-decal-brush-geometry-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zw-runtime-decal-projector-material-domain-dbuffer-gbuffer-forward-receiver-culling-batching-atlas-streaming-temporal-rt-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zr-runtime-water-ocean-lake-river-surface-wave-fft-shallow-water-rendering-underwater-buoyancy-query-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/138-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
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

# 160 · Editor Spline / Path / Road / River / Decal / Brush / Geometry 工程化差距

## 1. 结论

Editor39/113 的产品结论仍成立：当前 Zircon 没有工程级空间 Spline、Path、PathFollow、SplineMesh、Road、River、WaterBody、Geometry Brush 或 CSG 产品。生产源码中的 `CubicSpline`/Hermite 仅服务 glTF 与动画轨道插值，不提供空间点段 identity、弧长参数化、最小扭转标架、最近点查询、分段 bounds、空间索引、Scene persistence 或 Editor control-point authoring。`ResourceKind` 的 26 个枚举也没有上述源资产或编译产物类型。

Decal 仍是最明确的假完成链。可选 feature 注册 `DecalProjector` descriptor、render pass 和 executor，然而 executor 仅返回 `Ok(())`；descriptor 的 `atlas_region` 是 `String`，没有材质/纹理 handle、instance owner、Scene install、extract、cull、batch、shader 或 draw。与此同时 umbrella rendering package 与 builtin classification 仍宣称 Stable/Complete，feature 状态还能显示 available。这个状态必须 fail-close，不能把注册成功当作像素完成。

Editor 侧也存在三个互不相连的 authority：Decal editor crate 只有未被消费的 drawer ID；Material Workbench 暴露 `surface/post_process/decal`，但 runtime `MaterialDomain` 只有 Surface、PostProcess、DebugOverlay、LightFunction；预览工具固定展示 WarningStripe、12 placements 和 Projection 值。控件测试只证明 dropdown 值、paint invalidation 与 journal 记录，没有写回 `.zmaterial`、Scene component 或 render artifact。

本轮确认了一项真实进展：通用 dynamic plugin component 已能注册 schema、附加/删除、进入 Inspector、执行多选原子编辑与 undo/redo，并可在 World TOML/DynamicScene roundtrip；SceneMode registry、插件隔离、Overlay builder、selection 与 transaction 也可复用。因此相关通用基础从 Editor113 的 Open 调整为 Partial。但 `SceneEntityAsset` 仍是 camera/mesh/light/post-process/physics/animation/terrain/tilemap/prefab/script 等硬编码字段，没有通用 plugin payload；Decal 也没有 instance creator。World/DynamicScene 成功不能替代项目 Scene authoring 与发布闭环。

WOC 的道路绕行仍在：14 条道路共 52 个 X/Z 点，长度数组为 `[4,3,4,4,4,4,4,5,3,4,4,3,3,3]`，source SHA-256 为 `fb63b62216ff93c7b4fe7fccfe9bbdc7b5c9bc8a4549ff44dd4390fde78bc0e3`。生成脚本展开 `roadPointX/Z` 分支，`roadDistance` 每次遍历所有 road segment，只做 XZ 距离并使用 `< 5.0` 的项目阈值。它有确定性输入，却没有 road/segment identity、height/frame/arc length、mesh/UV/material/collision/nav/lane/junction/terrain stamp 或 streaming artifact。

目标架构必须分开建立，不能用一个 `Vec<Vec3>` 或一个 Decal enum 冒充产品：

```text
SpatialSplineSource + stable point/segment IDs
  -> deterministic validator/compiler
  -> CompiledSplineArtifact + immutable indexed query view
  -> PathFollow / SplineMesh / Road / River / TerrainStamp typed adapters

GeometryBrushSource + shape graph + boolean/extrusion policy
  -> deterministic CSG compiler
  -> topology/mesh/collision artifact + provenance

DecalProjectorSource + DecalMaterial domain + typed texture references
  -> Scene install + extract/cull/batch
  -> DBuffer/GBuffer/Forward render artifact + pixel evidence
```

## 2. 审查范围与方法

### 2.1 当前工作树选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与证据 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin/WOC selected | **1,122 / 114,972 / 105,503 / 4,268,190 / 1,133 / 0** | Decal package、Material/Shader/Scene、DynamicScene/plugin component、Editor Scene/Inspector/transaction、preview 与 WOC；`6c4223defde251711c939e137116f8b7c01992bee1bbbd6c9d9e724d1d74e403` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **42 / 32,554 / 27,324 / 1,246,989 / 12 / 0** | Spline/Path/SplineMesh、Road/River/Water metadata、Decal、CSG/Brush 与 Editor visualizer；`a7b14afc381b204b3266b5825c2c95bcb9fc2b9aa32456659dbe8f2244c86bd3` |
| 全部选择集 | **1,164 / 147,526 / 132,827 / 5,515,179 / 1,145 / 0** | 当前共享 dirty working tree 去重物理语料；`a3e79ec22fcf8b2fdd7fea86390d8270619e8d35dc5fbf29b81d459f8861f6c0` |

Zircon 选择集完整纳入 Decal feature、rendering umbrella、Material/Scene/Shader 资产、render pipeline feature、DynamicScene、dynamic component、project I/O、Scene modes/selection/viewport、Inspector、editing tests；另显式纳入 ResourceKind、动画 Hermite、Material Workbench、预览设计、WOC 与 Vampire Scene。参考选择集是 16 个 Unreal、10 个 Godot、5 个 Fyrox、6 个 Bevy、5 个 Unity Graphics 文件。

统计按相对路径去重排序；每个 manifest 行为 `forward/slash/path:lowercase-file-sha256`，行间使用 LF 且末尾无额外 LF，再计算选择集 SHA-256。这里的 tests 只计 Rust `#[test]`，ignored 只计 Rust `#[ignore]`。共享工作树有大量无关在途修改，所以实施前必须重新导出 manifest 与指纹。

### 2.2 判定规则

1. `Open` 表示领域 source、owner、consumer 或资格证据缺失，通用同名基础不能抵消。
2. `Partial` 只授予已存在、可复用且有测试的通用合同，例如 DynamicScene roundtrip、Inspector transaction、SceneMode registry、job/artifact/generation 基础。
3. `Closed/Pass` 必须有 source -> compile -> artifact -> install -> runtime consumer -> Editor transaction -> fault/scale/platform evidence；本轮没有领域项达到这个标准。
4. 静态 ZUI、design.js、test fixture、descriptor、capability、pass name、空 executor 与日志文本均不算产品实现。
5. 本轮是 C2 review，不修改 production/tests，不运行 Cargo 或动态图形验证。Tooling 按用户要求排除。

## 3. 相对 Editor113 的当前变化

| 领域 | Editor113 | 当前源码重判 | 结论 |
|---|---|---|---|
| Dynamic plugin component | 注册/反射底座，Inspector/undo 链证据不足 | 有 schema generation、附加/删除、editable gate、finite JSON rejection、World TOML/DynamicScene roundtrip、Inspector 多选原子修改与 undo/redo | 通用基础升级为 Partial；静态 Project Scene carrier 仍 Open |
| Scene authoring infrastructure | SceneMode/selection/overlay/gizmo 基础存在 | registry 支持插件 factory/owner、隔离 panic boundary、overlay checkpoint，builtin 仍仅 Select/Transform | 通用 Scene tool host 为 Partial；Spline/Decal/Brush mode 仍为零 |
| Decal runtime | no-op executor | 仍为同一 `noop_render_executor`，没有实例、shader、draw 或 pixel test | P0-2 保持 Open |
| Decal editor | drawer ID、空 extension | drawer ID 仍只有声明；EditorPlugin 未覆写 extension registration | P0-3 保持 Open |
| Material authority | UI 有 Decal，runtime 无 Decal | `.zmaterial` 与 `MaterialAsset` 仍无 domain，UI dropdown 仍只改 preview state/journal | P0-3 保持 Open |
| Spline/Road/River/Brush | 产品为零 | 精确类型仍为零；Hermite 仍只属于动画 | P0-1 保持 Open |
| WOC road workaround | 14-road XZ codegen | source digest、长度与分支仍在，查询仍全量遍历并用 magic cutoff | P0-5 保持 Open |

## 4. 当前 Zircon 产品链差距

### 4.1 Spatial Spline 与查询合同

1. 没有 `SpatialSplineSource`、`SplinePointId`、`SplineSegmentId`、source revision 或 versioned migration。
2. 没有 position/in-out tangent、roll/tilt、scale/width、interpolation type、closed-loop seam 或 per-point metadata。
3. 没有有限值、重复点、退化段、自交、尖点、闭环连续性与 handedness 校验。
4. 没有 arc-length reparameterization table、distance-to-parameter、curvature、torsion、minimum-twist frame 或 up policy。
5. 没有 segment bounds、BVH/grid/chunk index、nearest/overlap/batch query、稳定 tie-break 与查询预算。
6. 动画 Hermite 接受 scalar/vector/quaternion value 与 tangent，但它没有空间标架、弧长、点段身份或 Scene owner，不能直接升级命名后复用。

### 4.2 Path、SplineMesh、Road 与 WOC

1. 没有 PathFollow 的 progress/distance/offset/loop/tilt/rotation mode、authority、network 或 replay 合同。
2. 没有 SplineMesh 的 profile、start/end tangent、roll/scale/offset、UV、material region、LOD、collision 或 navigation artifact。
3. 没有 Road 的 lane、shoulder、curb、bank、junction、surface、traffic metadata、terrain conform/stamp 与 world-partition owner。
4. WOC JSON 只含 X/Z 点；generated `.zr` 用多层 branch 回放坐标，不可增量编辑、查询或流式装载。
5. Vampire 把道路需求降为普通 imported mesh，进一步证明缺少可编辑 Road source，而不是需求不存在。

### 4.3 River、WaterBody 与 Terrain

1. 没有 River/WaterBody source、bank profile、width/depth/velocity/flow、shoreline、foam 或 transition metadata。
2. 没有 Spline -> water surface/volume/collision/navigation/audio/render adapters，也没有 lake/ocean transition。
3. Terrain 当前 owner 与 Editor138 可承载 height/material/scatter，但没有消费稳定 spline segment 的 stamp/excavation contract。
4. River 的水渲染、物理、浮力与大世界资格由 Runtime99zr 负责；Terrain artifact/clipmap/physics/nav 由 Runtime99zq 与 Editor138 负责。Editor160 不复制这些 renderer finding。

### 4.4 Geometry Brush 与 CSG

1. 生产代码没有 GeometryBrush/BrushBuilder/CSG shape graph，UI 中泛化的 paint/foliage brush 不是 geometry boolean。
2. 没有 box/sphere/cylinder/torus/polygon/path extrusion source，也没有 union/intersection/subtraction 操作。
3. 没有 deterministic boolean kernel、manifold/degenerate/self-intersection/topology repair、material region、UV/tangent 或 collision bake。
4. 没有 shape stable IDs、history、preview generation、cancel、rollback、large-mesh budget、source-to-artifact provenance。

### 4.5 Decal Runtime 与能力真相

1. `DecalProjectorDescriptor` 只有 mode、opacity、normal_blend 与字符串 atlas region；component descriptor 的字段只有 name/type/editable。
2. feature 注册 PostProcess pass，读 scene-depth/scene-color 并写 scene-color，却没有 DBuffer/GBuffer attachment、load/store、stencil、normal reprojection 或 hazard contract。
3. registered executor 是 no-op；唯一局部测试只验证注册报告、默认禁用、component type 与 pass name。
4. 没有 projector bounds/frustum/cluster culling、receiver mask、distance/angle fade、sort、lifetime、visibility 或 streaming budget。
5. 没有 albedo/normal/ORM/emissive typed channels、Decal material domain、texture residency、atlas allocation、GPU instance buffer 或 batch。
6. 没有 forward/mobile/MSAA/ray tracing/virtual geometry fallback、device loss recovery、stats 或 pixel golden。
7. 详细渲染实现与 GPU 门禁归 Runtime99zw；Editor160 只登记 authoring、Scene、Material 和 capability truth 的断路。

### 4.6 Editor、Material、Scene 与插件边界

1. Decal editor plugin 只暴露 descriptor/capability；`DECAL_PROJECTOR_DRAWER_ID` 没有注册或消费者。
2. builtin Scene modes 只有 Select 与 Transform；没有 control-point、road、river、projector 或 brush mode。
3. Overlay builder 可收集 scene gizmo，插件 boundary 可隔离失败，但没有 projector box、方向箭头、point/tangent/width handle 或 CSG handle。
4. `ComponentPropertyDescriptor` 没有 stable field ID、typed default、range、unit、asset kind、schema version 或 migration hook。
5. Dynamic World component 可以进入 Inspector/undo/roundtrip，但 `SceneEntityAsset` 没有通用 plugin payload，Project Scene load/save 不能保存 Decal 或空间 domain source。
6. `.zmaterial` v2 有 shader/parent/options/overrides/textures/queue/editor/diagnostics，却没有 domain；`MaterialAsset` 也没有 Decal domain。
7. Material Workbench 的 `decal` option 只走 shared preview control，测试断言 paint-only invalidation 与 journal 长度，没有 asset document transaction。
8. design.js 的 Decal workspace 固定 WarningStripe、LeakMark、Scratch_A、Signage_01、12 placements 等样例，不是实际 toolkit、catalog 或 preview world。

## 5. 五套参考源码对照

| 引擎 | 可吸收的工程合同 | 对 Zircon 的纠偏 | 不应照搬的部分 |
|---|---|---|---|
| Unreal | `USplineComponent` 的 curve/reparam/closed/up/query；SplineMesh deformation/collision；Visualizer 的事务、点段选择、split/delete/duplicate/snap；Landscape spline 的道路/地形/导航；Water metadata 与 River spline mesh；Decal DBuffer/GBuffer/stencil/cull/batch | authoring source、query artifact、consumer 与 visualizer 必须形成同一 identity/revision 链；Decal pass name 远不足以代表 renderer | 不复制 UObject/Actor 宏体系、全局 editor mode 或未经 Zircon owner 约束的继承层次 |
| Godot | Curve3D baked distance/tilt/up/forward cache、closest/tessellation；PathFollow3D；Path editor handles/undo；Decal typed textures/fades；CSG primitives/boolean/path extrusion/bake | 提供较紧凑的 source/resource/node/editor 分层参考，尤其适合 Rust API 收敛 | 不照搬单线程 SceneTree 假设或将完整 CSG rebuild 放入交互热路径 |
| Fyrox | Curve/key UUID、曲线文档 command stack/save/unsaved lifecycle；Decal scene node、反射/序列化、bounds 与真实 shader | stable identity、文档生命周期和 Rust 反射可直接作为风格参考 | Fyrox curve 主要是标量 curve，不能冒充空间 spline；其 decal 规模也不能替代大场景资格 |
| Bevy | cubic spline 数学覆盖 Bezier/Hermite/Cardinal/B-spline/NURBS/linear；clustered/forward decal 的 extract、GPU buffer、binding limit 与 capability gate | 数学库和 ECS extract 可拆分复用；unsupported 平台必须显式 admission | 数学 spline 不自动提供 Scene persistence/editor/road；示例系统不能直接认定为完整 authoring product |
| Unity Graphics | HDRP DecalProjector 的 material/fade/UV/layer/size lifecycle；DecalSystem 的 culling group、per-material set、GPU data、capacity；Editor handles/Undo；WaterDecal typed region/update/material | 证明 projector source、runtime system、editor、material、culling 与 atlas identity 必须闭合 | 不复制 C# editor serialization 或 HDRP 单管线假设；需抽象 Zircon 的多 backend/feature admission |

## 6. Authority 与所有权

| 合同 | 唯一 authority | Editor160 责任 |
|---|---|---|
| Spatial source、stable IDs、compiler、query artifact | `zircon_runtime` 内部 core spine 或经 M0 批准的首方 runtime feature owner | 定义 authoring 所需契约、Scene projection、transaction、gizmo 与 acceptance，不私有复制数学或 artifact |
| Road/River/Terrain consumers | Road owner待 M0 固化；Water 为 Runtime99zr；Terrain 为 Runtime99zq/Editor138 | 只通过 typed adapter 消费 compiled spline，不把水体/地形 renderer 塞回 Editor |
| Geometry Brush/CSG | runtime geometry/compiler owner待 M0 固化 | Editor持有 document/tool state，不持有最终 topology 真值 |
| Decal renderer/material backend | Runtime99zw 与 Runtime material/shader owner | Decal source/document/component creation、Inspector、Scene gizmo、preview 与 receipt projection |
| Project Scene/Prefab/DynamicScene | Runtime Scene authority | Editor只能通过 versioned carrier 与 runtime gateway 修改，不新增 Editor-only payload |
| Catalog/App/capability maturity | first-party runtime/editor catalog 与 App feature composition | Editor验证 factory/provider closure；no-op 或 preview-only 必须 fail-close |

## 7. P0：必须先关闭的断路（5 Open）

| ID | 状态 | 差距 | 首个关闭条件 |
|---|---|---|---|
| P0-1 | Open | 空间 Spline/Road/River/Brush 没有 source、stable IDs、compiler、artifact、query、Scene carrier 或 toolkit | M0 固化 owner；M1 交付 versioned source、compiled artifact 与 indexed immutable query |
| P0-2 | Open | Decal plugin 注册成功但 executor 不提交任何 GPU 工作，Stable/Complete/available 形成假完成 | 立即 fail-close；Runtime99zw 完成 extract/cull/batch/render/pixel gate 后才恢复 capability |
| P0-3 | Open | Decal drawer、Material dropdown、design workspace 与 Runtime material/Scene/render authority 分裂 | 删除或禁用第二 authority，建立单一 Decal source/material/component/document/receipt 链 |
| P0-4 | Open | Dynamic World roundtrip 已有基础，但 Project Scene/Prefab 没有通用 versioned plugin/domain carrier | Runtime Scene 提供 schema/version/unknown-field/dependency/migration carrier，并有项目文件 roundtrip |
| P0-5 | Open | WOC 使用 XZ codegen、全量 segment loop 与 magic cutoff，且没有任何引擎级 Road consumer | 迁移相同 source digest 到 compiled spatial artifact，接通至少 Road query/mesh/nav 后删除旧分支 |

## 8. P1：Runtime、Geometry、Decal 与 Editor（56 Open / 14 Partial）

| # | 状态 | 需要重构的内容 |
|---:|---|---|
| 1 | Open | 定义 versioned `SpatialSplineSource`、control point/segment stable IDs 与 migration。 |
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
| 22 | Open | WOC migration 保留 JSON digest/length determinism 并生成 compiled artifact。 |
| 23 | Open | 删除 `roadPointX/Z` codegen、O(total segments) 查询与项目 magic constants。 |
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
| 57 | Partial | Runtime 本身不依赖 design.js，但没有可证明 headless cook 消费真实 geometry/decal artifact。 |
| 58 | Partial | first-party catalog/App 装配框架存在；Decal runtime no-op、Editor empty 使 provider/factory closure 不成立。 |
| 59 | Partial | 通用 release/artifact manifest 可复用；未包含 geometry/decal/material 依赖与平台支持/provenance。 |
| 60 | Open | handedness/float/texture/DBuffer/render graph/nav/physics 跨平台结果无领域证据。 |
| 61 | Partial | plugin unload guard 与 registry generation 存在；领域 upgrade/migration/rollback/stale renderer 未验证。 |
| 62 | Open | Road/River 与 Weather/Region/Navigation/Physics 仍无 typed adapter boundary。 |
| 63 | Partial | 通用 dependency/invalidation 基础存在；空间、Brush、Decal 的精确 artifact invalidation 尚无。 |
| 64 | Open | Editor preview 必须使用真实 artifact/camera，替换 placements/warning 固定文本。 |
| 65 | Partial | 通用 document/diff 基础可复用；point/segment/material/shape/channel/byte semantic diff 尚无。 |
| 66 | Open | 多用户 field-level merge/lock/presence/review 对空间文档尚无合同。 |
| 67 | Open | self-intersection、bad frame、missing width/material、overdraw/overlap/budget 自动审计尚无。 |
| 68 | Partial | DynamicScene/asset migration 基础存在；领域 canary、old-generation pin、rollback/replay compatibility 尚无。 |
| 69 | Open | Unreal/Godot/Fyrox/Bevy/Unity 的 spline/decal/water 对照仍缺可执行统一 fixtures。 |
| 70 | Open | Stable/Complete 当前仍可在 no-op provider 下成立，未由 compile/runtime/editor/visual/fault/scale/platform evidence 派生。 |

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

先服从 `docs/plans/mvp/00` 的 workspace/build 基线。Decal capability fail-close，Material/preview 的第二 authority 标记 unsupported；冻结 Spatial、Road、Geometry、Decal、Water、Terrain owners 与跨层接口。M0 不实现高级 UI。

### M1：Spatial Source、Compiler 与 Query

交付 stable IDs、versioned source、validation、arc-length/frame/index compiler、immutable query artifact、generation fence 与 Project Scene/Prefab carrier。先用 numeric/property/roundtrip 证明合同。

### M2：Spline Editor 与 Path Consumer

接入 AssetType/document/transaction、control point/tangent/roll/width handles、selection/snap/undo、real preview；实现 PathFollow 与 SplineMesh 的首个真实 consumer。

### M3：Road、Terrain 与 WOC 迁移

建立 Road source/profile、mesh/UV/material/collision/nav/terrain-stamp artifacts 与 indexed query；以相同 WOC digest 对比迁移结果，最后删除 X/Z 分支和 magic cutoff。

### M4：River 与 WaterBody Adapter

在 Runtime99zr owner 下接通 River bank/flow/depth/shore source、water render/physics/audio/nav/weather adapters，不在 Editor 私建 water truth。

### M5：Geometry Brush 与 CSG

交付 shape graph、boolean/extrusion compiler、topology diagnostics、preview/bake artifact、cancel/rollback/budget，再接入 Editor handles 与 transaction。

### M6：Decal Source、Material 与 Scene

统一 DecalProjector source、Decal material domain、typed texture refs、Scene/Prefab/DynamicScene install、Editor creation/Inspector/gizmo/document 与 compile receipt。

### M7：Decal Render

由 Runtime99zw 完成 DBuffer/GBuffer/forward attachments、extract/cull/batch/atlas/streaming、fallback、device-loss、stats 与 pixel golden。未通过前 feature 保持 unsupported。

### M8：Catalog、Preview 与产品操作

闭合 first-party runtime/editor provider、App feature、create/open/import/reimport/validate/compile/apply/playtest factories；所有 preview 使用真实 artifact 和 qualified runtime receipt。

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
| 8 | Fail | WOC digest/length 迁入 compiled artifact 且旧 codegen 删除。 |
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
| 29 | Partial | Runtime 不依赖 design.js；但关闭 Editor 后尚无真实 geometry/decal artifact 可消费。 |
| 30 | Fail | cross-engine reference scene 方法公开且质量/性能/内存/overdraw 可比较。 |
| 31 | Fail | unsupported shape/backend/feature 在 admission 早失败；当前 Decal no-op 违反。 |
| 32 | Fail | Stable/Complete 只由 compile/runtime/editor/visual/fault/scale/platform evidence 派生。 |

## 12. 禁止的临时修补

1. 禁止继续用 WOC `roadPointX/Z`、全量 segment loop、magic distance 代替 Spline/Road artifact。
2. 禁止只添加 `Vec<Vec3>`、几个控制点按钮或 Hermite 名称，而没有 stable IDs、frame、arc length、index、transaction 与 query。
3. 禁止将 glTF/动画 Hermite、Terrain/Foliage paint brush 或 UI curve 称为空间 Spline/Geometry Brush。
4. 禁止将 Decal descriptor、pass name、executor `Ok(())`、feature available 或 umbrella Stable 当作像素能力。
5. 禁止只增加 Decal enum、Material dropdown、drawer ID、manifest capability 或 design workspace。
6. 禁止让 Editor 私有 Scene/Material/Water/Geometry 真值；必须通过 Runtime-owned versioned source/artifact/gateway。
7. 禁止继续给 `SceneEntityAsset` 为每个 plugin domain 增加硬编码 `Option`；先设计通用 carrier、unknown-field 与 migration。
8. 禁止在 render thread 同步 tessellate Road、执行 CSG、读文件或提交无界 Decal instances。
9. 禁止用 registration snapshot、静态 preview、手工截图或单元属性测试替代 numeric/data/visual/pixel/fault/scale 门禁。
10. 禁止在未重算 1,164-file 选择集、未复核源漂移前开始实现。

## 13. 跨计划 Owner 与实施边界

1. Editor39 继续作为本主题 canonical finding owner；Editor160 只刷新当前源码事实和状态，不重复增加 canonical finding 总数。
2. Decal GPU 渲染细项由 Runtime99zw 关闭；Editor160 不在 Editor crate 实现 renderer。
3. River/WaterBody 渲染、物理、浮力与查询由 Runtime99zr 关闭；Editor只做 source document 与 adapter projection。
4. Terrain stamp、clipmap、physics/navigation 与 world partition 由 Runtime99zq/Editor138 协同关闭。
5. Material domain/shader artifact 必须与 Runtime09c/Editor15 的 material owner 合并，不能新增第二套 Decal material schema。
6. Scene carrier、Prefab/DynamicScene 与 project I/O 必须由 Runtime Scene owner定义；Editor只提交 transaction。
7. Tooling 本轮及后续当前阶段排除，待用户要求的 Rust 迁移计划单独处理。

## 14. 本轮产出边界

本轮只新增 Editor160 review，并更新分类索引、总索引与 coverage。没有修改 Runtime、Editor、Interface、Plugin、App、examples 或 tests production code；没有运行 Cargo、Editor、WGPU、Spline numeric、Road/River compiler、Decal pixel、CSG boolean、fault/scale/soak、跨平台或跨引擎动态 benchmark。审查过程中未查询、轮询、等待或实时跟踪协调器。

实施必须从 M0 开始，并服从 MVP `00` 基线。首个代码里程碑应先让 Decal capability fail-close、冻结 Spatial/Geometry/Decal owner 与 versioned source/artifact/Scene carrier；在这些合同通过 review 前，不应实现高级 Road/River/Brush UI。
