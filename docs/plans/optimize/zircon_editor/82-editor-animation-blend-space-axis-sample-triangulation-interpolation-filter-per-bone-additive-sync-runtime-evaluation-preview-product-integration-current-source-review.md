---
title: Editor Animation Blend Space、Axis、Sample、Triangulation、Interpolation、Filter、Per-Bone、Additive、Sync、Runtime Evaluation、Preview 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor82
review_date: 2026-08-23
baseline_head: 68edcd71042de817a74d4ad70efc07cfe2c72bfa
baseline_epoch: 359
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/composites/animation/workbench_blend_space_details.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/composites/animation/workbench_sample_weights.zui
  - zircon_editor/assets/ui/editor/components/workbench/composites/feedback/workbench_validation_log.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/blend_space_search.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/blend_space_transport.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/core/framework/animation/asset/state_kind.rs
  - zircon_plugins/animation/editor/src/plugin.rs
  - zircon_plugins/animation_graph/editor/src/plugin.rs
  - zircon_plugins/animation/runtime/src/state_machine/blend_space/blend_space_1d.rs
  - zircon_plugins/animation/runtime/src/state_machine/blend_space/blend_space_2d.rs
  - zircon_plugins/animation/runtime/src/state_machine/blend_space/geometry.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/compile.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/compiled_state.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/evaluate.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/state_graph_sample.rs
tests:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/blend_space_search.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace
  - zircon_plugins/animation/runtime/tests/animation_blend_space_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_compiled_state_machine_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_state_kind_asset_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/blend_space_state.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/78-editor-control-rig-rig-graph-hierarchy-controls-spaces-constraints-ik-solve-bake-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/81-editor-animation-pose-library-pose-asset-pose-name-curve-weight-additive-base-runtime-evaluation-preview-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/BlendSpace.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/BlendSpace.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Public/AnimNodes/AnimNode_BlendSpacePlayer.h
  - dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Private/AnimNodes/AnimNode_BlendSpacePlayer.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SAnimationBlendSpaceGridWidget.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SAnimationBlendSpace.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/Customization/BlendSpaceDetails.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/PersonaBlendSpaceAnalysis.cpp
  - dev/godot/scene/animation/animation_blend_space_1d.h
  - dev/godot/scene/animation/animation_blend_space_1d.cpp
  - dev/godot/scene/animation/animation_blend_space_2d.h
  - dev/godot/scene/animation/animation_blend_space_2d.cpp
  - dev/godot/editor/animation/animation_blend_space_1d_editor.cpp
  - dev/godot/editor/animation/animation_blend_space_2d_editor.cpp
  - dev/Fyrox/fyrox-animation/src/machine/node/blendspace.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/animation_event.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/GPUResidentDrawer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/GPUDriven/GPUDrivenInstanceDataTests.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Animation Blend Space、Axis、Sample、Triangulation、Interpolation、Filter、Per-Bone、Additive、Sync、Runtime Evaluation、Preview 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon的Blend Space不是“完全没有实现”，但产品表面与运行时事实是两套互不相认的模型。Animation Runtime已经有可保留的1D排序插值、2D重心权重、外壳投影、状态机二进制roundtrip、direct reference收集和1D/2D production pose测试；其中1D查询使用`partition_point`，2D可把最多3个graph pose送入现有混合器。这些代码证明局部算法不应被推倒重写。

但Workbench展示的是独立`BS_Locomotion | 8 samples`资产、Direction/Speed轴、可编辑sample grid、Triangulated/Grid/Weighted下拉、权重热图、Preview/Apply、时间轴和Validation Log。实际代码只过滤3个硬编码名字、切换tab/row/button状态、打开dropdown、把时间设为固定0或3秒，并写入“Preview queued”“Apply queued”字符串；网格、8个sample、权重、warning与preview主体全部来自静态ZUI。`ResourceKind`没有Blend Space，Runtime只把`parameter + position + graph`内嵌在`AnimationStateKindAsset`中，Animation Editor注册的两个Blend Space target type和ZUI又没有真实schema/resource。可见产品没有asset authority、document、transaction、compiler、runtime session或receipt。

运行时底座也只达到局部原型。2D编译枚举所有三元组并对每组扫描全部点，至少为O(n^4)，之后再用非鲁棒浮点测试和greedy overlap筛选；每次越界采样重新构建`BTreeMap`外壳。每个样本共享同一绝对秒时间；没有axis wrap/filter实例状态、phase/marker leader、per-sample rate/mirror、weight smoothing、per-bone policy、notify/root-motion仲裁或partial-failure receipt。缺参数/类型错误返回空sample，pose路径任一graph缺失会整体`None`，event路径却跳过缺失graph继续发事件，pose与事件可产生不一致结果。

本轮不新增P0。Plugins13已经唯一拥有四份Animation ZUI缺失与first-party provider闭环，Editor14已经拥有静态高级动画workspace的capability truth，Editor76已经拥有Graph palette/asset/mutator错位和Blend Space父级schema条目，Runtime08C拥有通用dense pose与动画运行时主干。本报告只展开此前未逐项验收的 **20项Blend Space专属P1、5项P2和48个资格门**，目标链为`AnimationBlendSpaceSourceDocument -> BlendSpaceCompilePlan -> PreparedBlendSpace -> BlendSpaceRuntimeHandle -> BlendSpaceInstanceState -> BlendSpaceEvaluationRequest -> BlendSpaceEvaluationReceipt`。

本轮只做current-source review和文档建账，不修改生产源码。未运行Cargo、真实Editor、GUI/GPU、asset save/load、cook、runtime preview、hot reload、fault/soak/profile或同语义跨引擎benchmark；因此不能宣称Blend Space产品可用、数值稳定或性能达标，更不能宣称性能或表现超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 本轮唯一owner与去重边界

本报告只拥有“1D/2D Blend Space如何成为持久化资产或明确的inline source，如何描述axis/sample/topology/filter/sync/output政策，经唯一Animation compiler生成prepared program，由每实例状态求值，并由Editor事务化编辑和runtime-backed preview”的纵向边界。

- Editor04继续拥有通用asset discovery、import/reimport、catalog、thumbnail与reference repair基础设施。
- Editor14继续拥有高级动画workspace的capability truth、通用toolkit/preview/compile真实性；本轮不重复假UI P0。
- Editor63继续拥有transaction、history、savepoint、dirty document、generation与async CAS。
- Editor69继续拥有PreviewWorld、time domain、pause/step、可见性与throttling调度。
- Editor75继续拥有Timeline/transport通用交互；本轮只拥有Blend Space的phase、sample time和scrub语义。
- Editor76继续拥有唯一Graph/State Machine schema、compiler和runtime trace；其P1-19是本轮的父级rollup，本轮负责细化验收。
- Editor77继续拥有Clip event、root motion、curve、sync marker与prepared clip合同；Blend Space必须消费它们，不能再定义第二套Clip DTO。
- Editor78继续拥有Control Rig、IK与bake；本轮只定义其输出被Blend Space sample消费时的兼容合同。
- Plugins13和Runtime08C继续拥有通用Animation provider、dense pose、scene writeback、GPU deformation与resource residency主干。

### 2.2 Currentness

- 审查HEAD：`68edcd71042de817a74d4ad70efc07cfe2c72bfa`。
- 协作baseline epoch：`359`；session：`optimize-editor82-animation-blend-space-review-r1-20260823`。
- 冻结时`compiled/compile.rs`与Animation Graph editor `plugin.rs`已有其他Session的未提交`use`排序改动；无语义变化。本报告按当前工作树计算fingerprint，不回退或占有这些改动。
- 语义检索确认只有inline `AnimationStateKindAsset::BlendSpace1D/2D`，不存在`ResourceKind::AnimationBlendSpace*`、独立load API、artifact、runtime handle或document session。
- Animation Editor的`animation.Asset.BlendSpace1D/2D`只出现在descriptor和字符串测试中；对应`plugins://animation/editor/blend_space_1d.zui`与`blend_space_2d.zui`不存在。该缺口由Plugins13拥有。
- Animation Graph palette声明`blend_space_1d/2d`，而Graph资产枚举没有这两类节点；该错位由Editor76拥有。
- Visual tests验证ZUI结构、响应式布局、像素可见性和控件状态，不证明asset mutation、compiler、runtime sampling或preview一致性。

### 2.3 冻结语料与可复算fingerprint

统计口径：路径转为小写正斜杠并排序；每个文件取SHA-256，再拼接`path + NUL + lowercase file hash + LF`计算集合fingerprint。declarations使用Rust/C++/C#的`fn/class/struct/enum/trait/interface/record`行首声明正则，仅用于规模定位。

| 范围 | 文件 / 行 / 非空行 / bytes / declarations | fingerprint |
|---|---:|---|
| Zircon selected set | **55 / 12,799 / 12,060 / 533,383 / 325** | `6f4796b8e793bc5fea54895b1046d5d3bf9dc820c82ec38d44cfd018bef99516` |
| Unreal selected set | **8 / 9,713 / 8,324 / 380,098 / 54** | `f81528b326e8cecd8991c6ed9b0234a29aebb69b0b6d40fa8c630b6ac406e02a` |
| Godot selected set | **6 / 4,600 / 3,839 / 178,678 / 11** | `a6220c7a57dc8eb4a76b5ae268cdfbcee29dcd6aac809cdef5f65166039c6e7e` |
| Fyrox selected set | **1 / 538 / 448 / 17,295 / 38** | `7545875d748a9e2e0a1959dd5f4284e50d7bb75d4b31019edfd9ea26c9b98150` |
| Bevy selected set | **4 / 3,041 / 2,750 / 116,334 / 171** | `a762231416f88516e251f6b75cb7c3ea664c4f80a2fda24a98c4cba84d6925c3` |
| Graphics selected set | **3 / 2,728 / 2,249 / 132,901 / 7** | `e3054c02dbc9ba7e04c1582206b01da99d95ef5789b060af503784832519c247` |
| Five-engine deduplicated set | **22 / 20,620 / 17,610 / 825,306 / 281** | `e22e4ebc65546fe0595e75daa08e445af9584d5ca7b80cef773b905a862bbf72` |

## 3. Zircon当前产品链事实

### 3.1 Workbench是高完成度静态样板，不是Blend Space asset editor

`workbench_extension_blend_space_workspace.zui`硬编码3个“资产”、8个sample point、Direction `-180..180`、Speed `0..600`、16x10 heatmap、3秒时间轴和`BS_Locomotion`摘要。右侧details只列4个sample row，并把Horizontal写成Speed、Vertical写成Direction；主网格却把X写成Direction、Y写成Speed，`X Axis`字段又写`Speed 0-620`，source truth在同一页面内已自相矛盾。

`workbench_sample_weights.zui`固定显示Direction `0.0`、Speed `600.0`、Run_Fwd权重`1.00`；`workbench_validation_log.zui`固定显示0 error、1 warning、2 info、“1 sample missing”“No duplicate samples found”。这些值不读取Runtime `BlendSpaceWeights3`或compile diagnostic。主网格也没有pointer drag、add/remove、selection-to-inspector、triangle edit或asset dirty绑定。

`blend_space_search.rs`只过滤`BS_Idle_Run / BS_Strafe_Grid / BS_Sprint_Lean`；`blend_space_transport.rs`只修改checked/text/current_time，并把previous/next固定为0/3秒。navigation index把field action标记为布尔值，generic handler最多打开popup；Preview/Apply只选择button并写固定feedback。22个标准化Blend Space action构成的是控件导航合同，不是domain operation。

### 3.2 产品模型与Runtime模型互相矛盾

Runtime schema只有：

```text
AnimationStateMachineAsset
  -> AnimationStateKindAsset::BlendSpace1D { parameter, [(position, graph)] }
  -> AnimationStateKindAsset::BlendSpace2D { parameter, [(Vec2 position, graph)] }
```

它没有独立asset ID、axis name/range/unit/grid/wrap、interpolation/filter、sample ID/rate/mirror/single-frame、additive/base、sync或event policy。2D只绑定一个`Vec2` parameter，Workbench却呈现两条独立轴字段。`ResourceKind`只到AnimationGraph/StateMachine，所谓`BS_Locomotion`无法经Asset Browser、loader或state node引用。

Animation Editor又注册不存在的`animation.Asset.BlendSpace1D/2D` target type和ZUI，Animation Graph palette把Blend Space当Graph node，Runtime却只在StateKind中支持。若不先确定canonical owner，直接补UI会继续制造standalone asset、inline state和graph node三套序列化/编译语义。

### 3.3 1D底座可保留，2D编译与查询不满足工程复杂度

`BlendSpace1D::compile`验证非空/finite、排序、拒绝exact duplicate，查询端点clamp并用`partition_point`做O(log n)区间定位。这是清晰的prepared算法底座，但仍没有axis wrap、epsilon duplicate、sample identity与diagnostic location。

`BlendSpace2D::compile`把少于3点都报为`Empty`，无法表达1点/2点退化空间。三角化枚举全部`a<b<c`，对每个三角形再扫描全部点检查circumcircle，至少O(n^4)；候选随后按顺序用centroid/edge侧测试greedy排重。orientation、incircle、inside和degenerate判定都只依赖`Real::EPSILON`或exact equality，没有scale-aware/adaptive predicate、cocircular tie-break或跨平台拓扑合同。

采样时线性扫描全部triangle；越界后每次重建`BTreeMap<(edge), count>`求hull，再扫描全部边。triangle adjacency、hull edge和spatial acceleration没有进入compiled artifact。当前4项几何测试只覆盖1D简单插值、单三角形外壳投影和正方形两三角形，不覆盖near-collinear、cocircular、large coordinate、duplicate epsilon、稳定拓扑、规模或分配预算。

### 3.4 求值只有权重转发，没有实例级Blend Space语义

State Machine compiler把sample数组下标压成`u32`并保存graph数组，`CompiledGraphSamples`固定为3项。每次求值仍按全部parameter name构造`Vec<Option<&AnimationParameterValue>>`；Blend Space参数缺失、类型错误或non-finite时静默得到空sample。没有typed binding diagnostic、default/fallback policy或上一帧LKG。

所有正权重graph在同一个绝对`time_seconds`采样。`normalized_graph_time`取所有活动graph的最大duration；duration缺失或时间非finite时直接返回1.0。不同长度clip没有normalized phase、marker leader/follower、blend-weighted play rate、loop/seek或sync group语义。

事件路径从所有正权重graph收集clip event，忽略weight，没有All/Highest/None/threshold/dedup政策。pose路径任一graph resolve/evaluate/sample失败会通过`?`使整个pose为`None`，event路径却`continue`保留其他graph事件。单样本pose直接丢弃其weight返回；多样本才进入通用pose blend。Blend Space也没有独立root motion、curve、morph、attribute、additive或per-bone输出合同。

### 3.5 测试证明了局部happy path，没有证明产品闭环

Runtime production测试证明1D两个graph在0.25处得到2.5 translation，2D三点在(0.25, 0.25)得到7.5 translation；binary roundtrip和source mutation后的compiled snapshot也有覆盖。这些是应保留的回归基线。

缺失的是：wrong/missing parameter、non-finite input、axis wrap、filter state、不同clip长度/marker phase、event/root-motion policy、partial graph failure、reload generation、1/2点2D退化、near-degenerate/cocircular topology、跨平台determinism、32/64/256 sample scale、zero-allocation steady state以及Editor add/move/delete/undo/save/reopen/runtime preview。Visual screenshot测试不能替代这些资格证据。

## 4. 参考引擎证据与可迁移原则

### 4.1 Unreal主参考：独立资产、prepared topology和完整实例状态

Unreal `UBlendSpace`是独立`UAnimationAsset`。`FBlendParameter`保存display name、min/max、grid divisions、snap和wrap；每个`FBlendSample`保存animation、sample value、rate scale、mirror、single-frame与frame index。资产还保存per-axis smoothing type/time/damping/max speed、sample weight smoothing、per-bone override/profile、mesh-space policy、loop、marker sync、notify mode、grid/triangulation选择、preferred triangulation direction、mirror table和additive preview base pose。

authoring sample与runtime `FBlendSpaceData`明确分层，prepared data持有1D segments或2D triangles；查询保存previous triangulation index进行warm start。sample cache属于`FAnimNode_BlendSpacePlayerBase`实例，并保存filter、sample data、cached topology index和previous asset。Runtime对输入先wrap/clamp/filter，再生成并平滑weights；按最高权重marker sample建立leader/follower，同步不同长度sample，显式处理notifies、root motion、rate/mirror、curve/attribute和per-bone/mesh-space pose blend。

Editor `SBlendSpaceGridWidget`直接绘制runtime triangulation和实际preview weights，支持drag/drop animation、add/move/remove/replace/duplicate、snap、manual/auto triangle、preview pin与分析。`SBlendSpaceEditor`用`FScopedTransaction`包围interactive move和结构修改，并把preview position写入真实preview instance。可迁移的是合同分层、prepared query和transaction/preview闭环；不能复制UObject反射、Slate widget结构或当前源码中已标注的cache GUID/镜像性能TODO。

本轮在目标Runtime/Persona test目录未找到可证明Blend Space数值和Editor事务的专用Unreal自动测试，因此报告只采信生产合同，不把Unreal现状当作无缺陷资格标准。Zircon目标还必须补足确定性、fault和性能门。

### 4.2 Godot：持久点/三角形、同步模式与UndoRedo闭环

Godot 1D/2D节点保存blend point name/node/position、min/max、snap、axis label、blend mode和sync mode；2D同时保存triangle并支持auto/manual topology。当前实现区分Interpolated、Discrete、Discrete Carry，以及None、Independent、Cyclic Mutable、Cyclic Constant四种时间政策；cyclic模式缓存sample length并按目标cycle缩放各child delta，invalid child会产生可观察错误。

2D auto topology调用独立Delaunay实现，outside point投影到triangle edge；Editor对add/remove/move/reorder/rename point、add/remove triangle、axis/snap/sync/blend mode和auto-triangle切换逐项建立do/undo method。drag blend position直接驱动AnimationTree runtime参数。Godot没有Unreal的per-bone、marker、notify和root-motion完整合同，不能作为性能上限，但证明“可编辑资源、运行时参数和UndoRedo”必须指向同一对象。

本轮未在`dev/godot/tests/scene`发现Blend Space专项测试，因此同样不把其存在的生产实现误写成充分测试。

### 4.3 Fyrox：Rust成熟库、可复用pose和事件策略

Fyrox `BlendSpace`保存axis name、min/max、snap、sampling parameter、points和triangles；point source是stable pose-node handle，内部pose通过`RefCell`复用。point变更自动重建triangulation，三角化直接使用`spade::DelaunayTriangulation`，并有empty、single point、two point和square topology测试。事件收集显式区分All/MaxWeight/MinWeight。

Fyrox也仍线性扫描triangle/edge，未提供Unreal级filter、sync、per-bone和cook artifact，所以它适合指导Rust ownership、成熟几何库和event policy API，而不是作为最终功能或性能目标。Zircon不应继续维护自制O(n^4)Delaunay近似。

### 4.4 Bevy：没有Blend Space，只提供typed graph与asset lifecycle参考

本地`bevy_animation`源码没有`BlendSpace`/`blend_space`命中。它的可用证据是`AnimationGraph`把Clip/Blend/Add、weight和mask放在同一typed graph，运行时构建threaded graph/computed mask，并响应Asset Added/Modified/LoadedWithDependencies/Removed；`ActiveAnimation`显式保存weight/repeat/speed/seek/last seek，event callback接收实际animation weight。

因此Bevy可约束Zircon的prepared graph、handle currentness、per-instance playback和weighted event上下文，不能被用来证明散点插值、轴语义或Blend Space Editor已经有成熟方案。

### 4.5 Unity Graphics只约束下游deformation/currentness

仓内Graphics是SRP/ShaderGraph/VFX包，不含Mecanim Blend Tree/Blend Space authoring源码。GPU Resident Drawer长期拥有`InstanceDataSystem`、persistent GPU buffer、handle map、update queue、previous transform、dispose/reinitialize、readback和专项测试。它不能作为Blend Space资产主参考，只要求Blend Space求值输出沿Animation/Renderer既有generation与current/previous deformation通道消费，不能由Editor workspace或临时名字Vec直接写GPU。

## 5. P1工程化差距（20项）

### BSPACE-P1-001：缺少canonical `AnimationBlendSpaceSourceDocument`

必须确定独立asset与inline state之间的唯一authority。建议独立source document由Graph/State Machine通过typed reference消费；若允许inline，必须使用同一versioned schema和compiler，而不是另一种struct。文档需有asset ID、schema version、source revision、display metadata和stable serialization。

### BSPACE-P1-002：Sample没有稳定identity、provenance和引用生命周期

引入`BlendSampleId`，把position、source graph/clip、source generation、author order、analysis provenance和optional tags绑定到稳定ID。add/remove/move/reorder/duplicate/reimport必须修复selection、diagnostic、transaction和runtime trace引用，不能继续用Vec下标作为跨代identity。

### BSPACE-P1-003：Axis schema只有UI字符串，没有domain合同

引入`BlendAxisDescriptor`，覆盖stable axis ID、parameter binding、display name、unit、min/max、grid/snap、clamp/extrapolate/wrap/circular seam、normalization和precision。1D scalar、2D Vec2与two-scalar binding要有显式且可迁移的选择。

### BSPACE-P1-004：Sample playback/mirror/additive metadata缺失

每个sample至少需要rate scale、loop/hold、mirror recipe/table、single-frame/frame/time、root-motion contribution、base/additive mode、base pose、skeleton/rig compatibility和source capabilities。compiler必须拒绝混合不兼容rig、additive space或缺失mirror映射，而不是等运行时空pose。

### BSPACE-P1-005：Typed parameter binding和错误政策缺失

prepared program应把axis绑定到dense parameter slot和expected type，并定义missing/wrong/non-finite/out-of-range的Reject、Default、Clamp或LastKnownGood政策。结果进入stable diagnostic与evaluation receipt，不能返回`[None; 3]`后假装state正常。

### BSPACE-P1-006：Interpolation、extrapolation和weight threshold不是可编译语义

Workbench的Triangulated/Grid/Weighted没有runtime对应。source schema应明确1D segment、2D triangulation/grid或未来RBF模式、outside-hull policy、duplicate aggregation、zero-weight threshold、normalization和tie-break；unsupported mode必须compile fail-close。

### BSPACE-P1-007：2D topology compiler是O(n^4)自制近似

以经过验证的Delaunay/robust-predicate库或独立geometry service替换全三元组枚举。`BlendSpaceCompilePlan`应产出points、triangles、adjacency、boundary edges、degenerate diagnostics和source mapping；1/2点退化空间必须有明确支持或typed拒绝。

### BSPACE-P1-008：数值鲁棒性与跨平台determinism没有合同

duplicate、orientation、incircle、inside、cocircular和near-collinear都需要scale-aware/adaptive predicate与stable tie-break。Windows/Linux、f32/f64 policy、sample insertion order和serialization roundtrip必须生成同一canonical topology或明确platform artifact key。

### BSPACE-P1-009：缺少BuildSet、dependency digest、cook和LKG

Blend Space编译依赖source schema、sample graph/clip generations、skeleton、mirror、sync marker、blend profile和compiler version。建立async build key、dependency manifest、diagnostic set、immutable artifact、install CAS、last-known-good和platform cook；不得在frame中发现source变更后临时重三角化。

### BSPACE-P1-010：缺少qualified runtime handle与reload生命周期

引入`BlendSpaceRuntimeHandle { asset_id, artifact_generation, provider_generation }`，由Animation provider解析和pin prepared artifact。reload/unload/provider replacement/device/world teardown必须使旧handle可诊断失效，并以generation-safe install迁移或重建instance state。

### BSPACE-P1-011：Axis filter/smoothing没有每实例状态

引入`BlendSpaceInstanceState`保存filtered input、velocity、previous raw input、wrap seam状态与filter generation。至少支持None、linear/exponential、spring-damper、damping和max speed；seek/reset/teleport、pause、fixed-step和replay需要确定性reset policy。

### BSPACE-P1-012：Sample weight smoothing和per-bone policy缺失

区分“平滑输入经过中间sample”与“直接平滑旧/新sample weight”两种语义，并支持ease、global speed、per-bone override/profile及local/mesh-space选择。旧sample淡出、new sample淡入、renormalization和zero threshold必须在dense buffer中无分配执行。

### BSPACE-P1-013：不同长度sample没有phase、marker和leader/follower同步

定义normalized/cyclic time、blend-weighted effective duration、sample rate、loop、seek、marker-compatible leader选择、follower phase和sync group交互。最长duration不能继续作为唯一normalized-state-time；marker缺失/不兼容需有fallback与trace。

### BSPACE-P1-014：Event、root motion、curve、morph和attribute仲裁缺失

至少提供All、HighestWeighted、Thresholded、None事件政策，以及dedup/order/weight context；root motion需定义weighted accumulation或leader authority；curve、morph、custom attribute需与pose使用同一sample/time/weight receipt。pose失败时不得继续发布来源不一致的事件。

### BSPACE-P1-015：Pose blend、additive、mask和rotation语义没有Blend Space合同

消费唯一dense pose evaluator，明确base/additive sample兼容、quaternion hemisphere/normalization、scale policy、missing bone、per-bone mask和reference pose。单sample、双sample、三sample必须走等价验证路径，不能单sample直接丢弃weight/failure元数据。

### BSPACE-P1-016：Partial failure、fallback和evaluation receipt缺失

引入`BlendSpaceEvaluationReceipt`，记录input、filtered input、triangle/segment、sample IDs/weights/times、leader、events/root motion、artifact generation和diagnostics。sample graph缺失时选择FailClosed、RenormalizeRemaining或LastKnownGood，并保证pose/event/root-motion原子一致。

### BSPACE-P1-017：热路径分配与查询复杂度没有预算

删除每次parameter `Vec`、outside-hull `BTreeMap`和临时weighted pose Vec；prepared artifact预存hull/adjacency并提供warm-start或spatial query。定义1/8/32/64/256 sample、1k/10k角色、cache hit/miss的CPU、allocation、scratch和memory预算，使用profile/benchmark而非代码行推断性能。

### BSPACE-P1-018：Graph/State Machine集成没有唯一schema和lowering

Runtime descriptor必须决定Blend Space是asset-player node、state kind还是两者共享的typed reference。Graph palette、asset serializer、StateKind、Editor mutator和compiler lowering只消费同一`BlendSpaceNodeTypeDescriptor`；unsupported standalone/inline组合在palette和compile阶段fail-close。

### BSPACE-P1-019：缺少真实Editor asset toolkit与可逆authoring操作

实现`BlendSpaceEditorSession`、immutable projection、selection、axis/sample inspector、add/drag/remove/duplicate/replace/mirror、auto/manual topology、diagnostic navigation、clipboard、dirty/save/reopen和multi-document isolation。interactive drag合并transaction，结构变更完整undo/redo，compiler generation以CAS安装。

### BSPACE-P1-020：Preview/debug/qualification仍是静态演示

Preview必须绑定真实subject、skeleton、runtime handle和instance state，网格显示实际filtered/raw input、triangle、weights、phase、leader、events、root motion和diagnostics；transport使用PreviewWorld time domain。建立数值oracle、editor workflow、reload/fault、cross-platform determinism和同语义Unreal/Fyrox/Godot benchmark，固定热图和“queued”文本不能作为验收。

## 6. P2长期能力（5项）

### BSPACE-P2-001：高维/RBF与非规则Blend Field

在1D/2D资格完成后评估3D、RBF、simplex cloud和高维feature field；统一沿用sample ID、prepared artifact与receipt，不新增平行运行时。

### BSPACE-P2-002：Motion analysis与自动sample placement

从clip/root motion/trajectory/pose feature分析轴值、重复点、coverage hole和topology质量，支持可审阅proposal与transaction apply，不允许黑盒直接改source。

### BSPACE-P2-003：Contact/phase-aware blending与motion warp协同

把foot contact、phase marker、stride/turn/root-motion constraint纳入sample选择和输出修正，保持原始Blend Space receipt与后处理receipt可追踪。

### BSPACE-P2-004：大规模角色的LOD、pose sharing与GPU协同

支持按可见性/距离降低sample数量、共享phase/pose page、批量parameter query和deformation upload；降级必须保持事件/root-motion authority与可观察quality tier。

### BSPACE-P2-005：Learned parameter mapping与自适应权重

ML/RBF learned mapper只能作为versioned compiler/evaluator strategy，需训练数据provenance、deterministic fallback、quality metric、平台capability和传统triangulation oracle。

## 7. 目标架构与硬切规则

```text
AnimationBlendSpaceSourceDocument
  +-- BlendAxisDescriptor[1..2]
  +-- BlendSampleDescriptor[stable BlendSampleId]
  +-- interpolation/filter/sync/event/root-motion/per-bone policy
  +-- skeleton/mirror/blend-profile/sample dependency references
          |
          v
BlendSpaceCompilePlan + BuildSet/dependency digest
          |
          v
PreparedBlendSpace
  +-- canonical points/segments/triangles/adjacency/hull/query accelerator
  +-- dense parameter/sample/pose slots
  +-- typed diagnostics + source mapping
          |
          v
BlendSpaceRuntimeHandle(asset/artifact/provider generation)
          |
          v
BlendSpaceInstanceState(filter/weights/phase/leader/cache)
          |
          v
BlendSpaceEvaluationRequest
          |
          v
BlendSpaceEvaluationReceipt
  +-- dense pose/curve/morph/attribute
  +-- event/root-motion contribution
  +-- weights/times/topology/diagnostics trace
          |
          +--> State Machine / Animation Graph / PreviewWorld
          +--> Renderer deformation handoff
```

硬切规则：

1. 删除Workbench固定`BS_*`、sample、weight、validation和queued成功文本的production capability声明；provider/asset缺失时隐藏或Unavailable。
2. 不保留standalone、inline state、Graph palette三套Blend Space schema；全部迁移到唯一descriptor/compiler。
3. 不保留O(n^4)三元组triangulation作为production fallback；test fixture也必须显式标注oracle用途。
4. 不在frame中解析sample graph字符串、重建hull map、分配parameter Vec或重编topology。
5. 不允许pose/event/root-motion在同一evaluation中采用不同partial-failure样本集合。
6. 不使用sample Vec index、display name或ZUI control ID作为持久identity。
7. 不以URI字符串测试、layout screenshot或weight-sum测试关闭asset/compiler/runtime/editor finding。
8. 旧inline payload通过一次性migration转为canonical source/reference；不保留长期双写或compat facade。

## 8. 里程碑

### BSPACE-M0：Capability truth与owner冻结

- ordinary Editor隐藏不可用Blend Space产品入口，移除固定成功feedback。
- 冻结canonical asset/inline policy、owner、typed descriptor和父账依赖。
- 为当前happy-path算法建立回归oracle，记录现有payload migration输入。

### BSPACE-M1：Source schema与migration

- 实现source document、axis/sample stable ID、playback/additive/sync/output policy。
- 注册ResourceKind/asset type/load/save/reference analysis，或以同一schema实现受控inline carrier。
- 迁移现有StateKind payload并验证roundtrip、reference repair和old-version failure。

### BSPACE-M2：Semantic compiler与robust topology

- 引入成熟Delaunay/robust predicate，生成segments/triangles/adjacency/hull。
- 完成1/2点退化、duplicate、near-collinear、cocircular和outside policy。
- 产生source-located diagnostic与deterministic artifact。

### BSPACE-M3：Build/cook/currentness

- 建立BuildSet、dependency digest、async compile、LKG、CAS install与platform cook。
- 实现qualified runtime handle、reload/unload/provider replacement和instance migration。
- 禁止frame-time source load/recompile。

### BSPACE-M4：实例级filter、weight与time sync

- 实现axis filter/wrap、weight smoothing、per-bone policy和zero-allocation scratch。
- 实现normalized/marker leader/follower、sample rate、loop/seek/reset与sync group。
- trace raw/filtered input、topology、weights、phase和leader。

### BSPACE-M5：完整动画输出与Graph/State集成

- 统一pose/curve/morph/attribute/event/root-motion样本集合与failure receipt。
- 完成base/additive/mask/quaternion/per-bone blend。
- Graph/State Machine只通过唯一typed descriptor和runtime handle消费。

### BSPACE-M6：Editor toolkit与事务

- 实现document session、sample canvas、axis/sample inspector、topology、diagnostic navigation。
- add/move/remove/replace/mirror/triangle/axis操作具备transaction、dirty/save/reopen。
- 大sample集使用稳定projection和虚拟化，不让hover/selection改布局。

### BSPACE-M7：Runtime-backed preview与debugger

- PreviewWorld使用真实prepared artifact/instance evaluator/subject/time domain。
- transport、heatmap、timeline、event/root-motion和filtered input来自同一receipt。
- hot reload、compile failure和missing dependency保持上一有效preview并显示generation。

### BSPACE-M8：资格、性能与硬切

- 通过数值、fault、reload、determinism、scale、soak和跨平台门。
- 对同语义Unreal/Fyrox/Godot场景记录功能差异与CPU/memory结果。
- 删除旧inline解释路径、fake workspace data和compat schema，文档/API同步更新。

## 9. 资格门（48项）

### Authority与Schema

- [ ] BSPACE-G-01：ordinary Editor只在canonical provider/asset capability可用时暴露Blend Space入口。
- [ ] BSPACE-G-02：独立asset与inline carrier消费同一versioned source schema和compiler。
- [ ] BSPACE-G-03：axis具有stable ID、binding、unit、range、grid/snap、wrap与normalization合同。
- [ ] BSPACE-G-04：sample具有stable ID、source generation、rate、mirror、single-frame和mode metadata。
- [ ] BSPACE-G-05：add/remove/move/reorder/rename后selection、reference和diagnostic仍按ID稳定。
- [ ] BSPACE-G-06：1D scalar、2D Vec2/two-scalar binding有typed migration与missing/wrong policy。
- [ ] BSPACE-G-07：serialization roundtrip保持axis/sample/topology policy和direct dependencies。
- [ ] BSPACE-G-08：旧StateKind payload一次性迁移，旧写路径和长期双写被删除。

### Compiler、Topology与Cook

- [ ] BSPACE-G-09：compiler拒绝non-finite、incompatible rig/additive/mirror与无效range并定位source。
- [ ] BSPACE-G-10：1点、2点、duplicate、collinear、near-collinear和cocircular有明确结果。
- [ ] BSPACE-G-11：sample insertion/order/serialization变化不改变canonical topology与tie-break。
- [ ] BSPACE-G-12：Windows与Linux对同一source生成相同artifact digest或显式platform key。
- [ ] BSPACE-G-13：prepared artifact包含segments/triangles、adjacency、hull和source mapping。
- [ ] BSPACE-G-14：32/64/256 sample compile满足时间/内存预算，不使用O(n^4)生产算法。
- [ ] BSPACE-G-15：BuildSet覆盖sample graph/clip、skeleton、mirror、marker、profile和compiler version。
- [ ] BSPACE-G-16：compile失败保留LKG；CAS install拒绝过期source/dependency generation。

### Runtime Input、Filter与Weights

- [ ] BSPACE-G-17：prepared parameter slot避免每次求值按名称构造Vec或hash扫描。
- [ ] BSPACE-G-18：missing/wrong/non-finite input产生stable receipt，不静默空pose。
- [ ] BSPACE-G-19：clamp/wrap/circular seam与outside-hull policy在边界连续且可测试。
- [ ] BSPACE-G-20：None/exponential/spring filter在fixed-step、variable-step、seek/reset下确定。
- [ ] BSPACE-G-21：input smoothing与sample-weight smoothing是独立可组合政策。
- [ ] BSPACE-G-22：old/new sample淡入淡出、threshold和renormalization保持权重和为1。
- [ ] BSPACE-G-23：per-bone/profile与mesh/local-space选择在dense pose上无名字查找。
- [ ] BSPACE-G-24：steady-state sample/query/filter路径零堆分配且没有per-frame hull map。

### Time与Animation Output

- [ ] BSPACE-G-25：不同duration/rate sample按明确normalized/cyclic policy推进。
- [ ] BSPACE-G-26：marker leader/follower、leader变化和incompatible marker fallback可追踪。
- [ ] BSPACE-G-27：loop、reverse、seek、teleport、pause、step和sync-group rejoin行为确定。
- [ ] BSPACE-G-28：event支持All/Highest/Threshold/None并具有order/dedup/weight context。
- [ ] BSPACE-G-29：root motion使用明确leader或weighted accumulation且与pose sample集一致。
- [ ] BSPACE-G-30：curve、morph和custom attribute使用同一sample/time/weight receipt。
- [ ] BSPACE-G-31：base/additive/mask/quaternion/scale/per-bone blend有数值oracle。
- [ ] BSPACE-G-32：sample缺失/失败按FailClosed/Renormalize/LKG政策原子影响全部输出。

### Editor与Preview

- [ ] BSPACE-G-33：asset browser能创建、打开、保存、重开、rename和删除Blend Space。
- [ ] BSPACE-G-34：axis/sample inspector直接编辑source projection，不保存ZUI字符串状态。
- [ ] BSPACE-G-35：add/drag/remove/duplicate/replace/mirror操作完整undo/redo并合并interactive move。
- [ ] BSPACE-G-36：auto/manual triangle、snap、boundary和compile diagnostic在canvas可视化。
- [ ] BSPACE-G-37：dirty/savepoint、autosave、external change、reimport和merge conflict按generation处理。
- [ ] BSPACE-G-38：PreviewWorld subject、skeleton、asset handle和instance generation可检查。
- [ ] BSPACE-G-39：grid、heatmap、timeline、filtered input、phase、events和root motion来自真实receipt。
- [ ] BSPACE-G-40：missing provider/resource/compile failure显示Unavailable/LKG，不显示固定queued成功文本。

### Qualification与Hard Cut

- [ ] BSPACE-G-41：1D/2D inside/outside/vertex/edge/duplicate/degenerate数值oracle覆盖充分。
- [ ] BSPACE-G-42：wrong parameter、missing graph、reload/unload、provider replacement和fault injection通过。
- [ ] BSPACE-G-43：binary/source/artifact migration、reference repair和old-version rejection通过。
- [ ] BSPACE-G-44：1k/10k角色和1/8/32/64/256 sample场景满足CPU/memory/allocation预算。
- [ ] BSPACE-G-45：长时loop/marker/filter/weight smoothing soak无phase drift、NaN或generation泄漏。
- [ ] BSPACE-G-46：Windows/Linux deterministic replay产生相同topology、weights、events和receipt digest。
- [ ] BSPACE-G-47：同语义Unreal/Fyrox/Godot对照记录功能、quality、CPU和memory，禁止异义benchmark宣传。
- [ ] BSPACE-G-48：旧fake workspace data、parallel schema、O(n^4)compiler和compat写路径完成硬删除。

## 10. 实施约束与非目标

1. 先收敛MVP的1D/2D locomotion/aim空间，不以3D/RBF/ML延迟基础资产和运行时合同。
2. 保留现有1D排序插值、2D重心/外壳测试意图和production pose happy path，但替换不合格的topology compiler与查询准备。
3. 不通过新增另一套`zircon_editor`私有Blend Space DTO修补插件资源；Runtime schema是唯一authority。
4. 不把Unreal UObject/Slate直接搬入Rust；提取axis/sample/prepared/instance/receipt合同。
5. 不把Godot的dynamic property或Fyrox当前线性triangle扫描当最终性能方案。
6. Bevy只作为typed graph、asset lifecycle、mask和weighted event参考，不宣称其实现Blend Space。
7. Graphics只约束下游deformation generation/currentness，不拥有Blend Space authoring或采样。
8. 在Runtime08C/Plugins13的dense pose与renderer handoff完成前，Blend Space不得旁路建立第二套pose buffer。

## 11. Review closeout

| 项目 | 状态 | 证据 |
|---|---|---|
| Workbench产品与action追踪 | review_complete | 静态8 sample/grid/heatmap/log、3-name search、0/3秒transport和fixed feedback已定位 |
| Asset/schema/plugin追踪 | review_complete | inline StateKind、无ResourceKind、缺失ZUI target与Graph palette错位已定位并去重父账 |
| Geometry/compiler追踪 | review_complete | 1D O(log n)可保留；2D O(n^4)+greedy overlap、per-sample hull map和测试缺口已定位 |
| Runtime evaluation追踪 | review_complete | silent parameter failure、absolute seconds、max duration、all-positive events和pose/event failure分叉已定位 |
| 五套本地参考 | review_complete | Unreal/Godot/Fyrox/Bevy/Graphics职责边界、生产合同与专项测试缺口已记录 |
| P0 | 0 new | 缺失ZUI、假workspace和palette错位继续由Plugins13、Editor14、Editor76拥有 |
| P1 / P2 / gates | 20 / 5 / 48 | 已给出目标类型、里程碑、硬切规则和可验收门 |
| 生产实现 | not_started | 本轮没有修改生产源码 |
| 动态验证 | not_run | 未运行Cargo、Editor、GUI/GPU、cook、runtime preview、fault/soak/profile或benchmark |

后续实现前必须重新读取最新source、MVP/owner文档与本报告fingerprint；任一相关schema、provider、compiler、evaluator或Editor product发生变化，都应先刷新current-source事实，再执行BSPACE-M0。
