---
title: Editor Animation Blend Space、Axis、Sample、Triangulation、Interpolation、Filter、Per-Bone、Additive、Sync、Runtime Evaluation、Preview 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor203
review_date: 2026-08-28
baseline_head: 84d8d94e418ba0b1de4f84ca255ba961d5fc52ca
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/composites/animation/workbench_blend_space_details.zui
  - zircon_editor/assets/ui/editor/components/workbench/composites/animation/workbench_sample_weights.zui
  - zircon_editor/assets/ui/editor/components/workbench/composites/feedback/workbench_validation_log.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/blend_space_search.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/blend_space_transport.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/core/editing/animation_document/asset.rs
  - zircon_editor/src/core/editing/animation_document/command.rs
  - zircon_editor/src/core/editing/animation_document/compilation.rs
  - zircon_editor/src/core/editing/animation_document/document.rs
  - zircon_editor/src/core/editing/animation_document/mutation/state_machine.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/core/framework/animation/asset/state_kind.rs
  - zircon_runtime/src/core/framework/animation/asset/state_machine.rs
  - zircon_runtime/src/core/framework/animation/asset/binary.rs
  - zircon_runtime/src/core/framework/animation/asset/graph.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/compile.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/model.rs
  - zircon_runtime/src/core/framework/animation/compiler/schema.rs
  - zircon_plugins/animation/editor/src/plugin.rs
  - zircon_plugins/animation_graph/editor/src/plugin.rs
  - zircon_plugins/animation/runtime/src/state_machine/blend_space/blend_space_1d.rs
  - zircon_plugins/animation/runtime/src/state_machine/blend_space/blend_space_2d.rs
  - zircon_plugins/animation/runtime/src/state_machine/blend_space/geometry.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/compile.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/compiled_state.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/evaluate.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/state_graph_sample.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_cache.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/animation_evaluation_pipeline.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_blend.rs
tests:
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace
  - zircon_editor/src/core/editing/animation_document/tests.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/tests.rs
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
  - docs/plans/optimize/zircon_editor/196-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/197-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/198-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/202-editor-animation-pose-library-pose-asset-pose-name-curve-weight-additive-base-runtime-evaluation-preview-product-integration-current-source-review.md
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
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/82-editor-animation-blend-space-axis-sample-triangulation-interpolation-filter-per-bone-additive-sync-runtime-evaluation-preview-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/82-editor-animation-blend-space-axis-sample-triangulation-interpolation-filter-per-bone-additive-sync-runtime-evaluation-preview-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Animation Blend Space、Axis、Sample、Triangulation、Interpolation、Filter、Per-Bone、Additive、Sync、Runtime Evaluation、Preview 与 Product Integration 当前源码复核

## 1. 结论

当前 Zircon 的 Blend Space 已经不再只是旧报告看到的低质量几何原型。共享 State Machine compiler 会把参数解析成 dense `Scalar`/`Vec2` slot，拒绝空 1D、少于 3 点的 2D、非有限坐标、exact duplicate 与 exact collinear；Animation runtime 使用 `spade::DelaunayTriangulation::bulk_load_stable` 预编译 triangle、adjacency 与 hull，2D 查询保留 per-state triangle hint 并优先邻接游走，越界只扫描预编译 hull。旧 Editor82 指出的 O(n^4) 三元组枚举和逐帧 `BTreeMap` hull 重建已经消失，不能继续作为当前缺陷。

但这还不是工程级 Blend Space 产品。authoritative source 仍内嵌在 `AnimationStateKindAsset`，只含 `parameter + position + graph`；没有独立资源、stable sample/axis ID、axis unit/range/wrap/filter、sample rate/mirror/additive/sync/output policy、dependency build key、qualified handle 或 evaluation receipt。`ResourceKind`没有 Blend Space，Animation Editor 注册的两个 target type 指向不存在的 ZUI；Animation Graph palette声明 1D/2D节点，真实 `AnimationGraphNodeAsset`又没有相应 variant。当前存在 standalone descriptor、Graph palette 与 inline StateKind 三种互相不闭合的产品承诺。

可见 Workbench 仍是静态样板。422 行主 workspace 固定 3 个 `BS_*`条目、8 个 sample、Direction/Speed 轴、`Run_Fwd`、3 秒时间线；details 又把 Horizontal/Vertical 与 X/Y 轴互换，并出现 Speed `0..600`和`0..620`两套范围。27 个 Blend Space route 加 6 个共享 transport route 只做 selection、dropdown、固定时间或固定 feedback；`Preview queued`、`Apply queued`、warning 与 weight heatmap 都不来自 document/compiler/runtime。

Runtime 热路径有重要进展但输出语义仍不完整。production instance cache按 parameter revision/layout 缓存 dense projection，容量 4096，并在 entity retirement/world replacement清理；compiled machine cache容量64。这修正了旧报告“每帧一定构造 parameter Vec”的笼统判断。然而 public `evaluate()`仍会临时投影，pose混合仍构造 weighted pose `Vec`；所有 sample 使用同一绝对秒，normalized state time取活动 graph 的最大 duration；event路径跳过缺失 graph，pose路径却用`?`整体失败。filter、weight smoothing、marker leader、per-bone、event/root-motion/curve/morph/attribute arbitration和原子 receipt仍为空。

本轮不新增 P0，保留旧 Editor82 的 20 项 P1 和 5 项 P2 ID，状态校正为 **P1：7 Open / 12 Partial / 1 Closed；P2：5 Open**。48 个资格门为 **30 Fail / 18 Partial / 0 Pass**。目标链仍是 `AnimationBlendSpaceSourceDocument -> BlendSpaceCompilePlan -> PreparedBlendSpace -> BlendSpaceRuntimeHandle -> BlendSpaceInstanceState -> BlendSpaceEvaluationRequest -> BlendSpaceEvaluationReceipt`。

本轮只做当前源码静态复核与重构建账，没有修改生产代码，也没有运行 Cargo、真实 Editor、GUI/GPU、cook、runtime preview、reload/fault/soak/profile 或同语义跨引擎 benchmark。Tooling 按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。因此不能宣称 Blend Space 产品可用，更不能宣称性能或表现超过 Unreal。

## 2. 审查边界、owner 与冻结语料

### 2.1 唯一 owner 与去重边界

本报告只拥有 Blend Space 专属纵向链：1D/2D source、axis/sample identity、topology、实例级 filter/weights/time、完整动画输出、Editor toolkit 与 runtime-backed preview。

- Editor04继续拥有通用 asset discovery/import/reimport/catalog/thumbnail/reference repair。
- Editor14继续拥有高级 Animation workspace capability truth 和通用 toolkit/preview/compile 真实性。
- Editor63继续拥有 transaction/history/savepoint/dirty/generation/CAS 基础。
- Editor69继续拥有 PreviewWorld、time domain、pause/step 与 activity scheduling。
- Editor196继续拥有通用 Timeline/transport；本报告只拥有 Blend Space sample phase/time 语义。
- Editor197继续拥有 Graph/State Machine schema/compiler/runtime trace 父账；本报告细化 Blend Space lowering。
- Editor198继续拥有 Clip event/root motion/curve/sync marker prepared contract。
- Editor202继续拥有 dense pose、additive/base、target identity 与 deformation handoff 父账。
- Plugins13与Runtime08C继续拥有 first-party provider、通用 Animation runtime 与 renderer deformation 主干。

旧 Editor82保持 canonical finding owner，Editor203只刷新 currentness 与实施判定，不重复增加全局 finding 总数。实施入口仍为`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`。

### 2.2 Currentness、负证据与工作树状态

- 冻结基线 HEAD 为`84d8d94e418ba0b1de4f84ca255ba961d5fc52ca`，语义以当时磁盘工作树为准。
- 冻结范围存在共享在途改动：Blend Space ZUI/transport、runtime blend/compiled/pipeline、Graph editor plugin和`state_kind.rs`为 modified；shared Animation compiler与Animation document目录为 untracked。报告不回退、不占有这些改动。
- 未跟踪 Rust/ZUI 反查只新增命中 shared compiler 5个文件和1个 visual validation test；没有发现第二套 Blend Space document/controller/runtime handle。
- Animation Editor的`authoring.zui`、`blend_space_1d.zui`与`blend_space_2d.zui`均不存在；descriptor字符串测试只证明注册声明存在。
- 当前优化树没有适用于本轮且未解决的 Blend Space `failure-*.md`；不新增 failure handoff。

### 2.3 冻结语料与可复算 fingerprint

统计口径：路径转为小写正斜杠后排序；每文件取 SHA-256，再拼接`path + NUL + lowercase file hash + LF`计算集合 fingerprint。Zircon 选择集覆盖 Workbench/route、generic document、shared schema/compiler、Animation plugin runtime和focused tests；参考选择集与旧 Editor82 相同，路径已按当前树复核。

| 范围 | 文件 / 行 / 非空行 / bytes | fingerprint |
|---|---:|---|
| Zircon deduplicated focused set | **71 / 17,855 / 16,689 / 708,781** | `7d23e61ad103c17fdda235720156548bdba800419853a7a2baeb431224d8c061` |
| Unreal selected set | **8 / 9,713 / 8,324 / 380,098** | `f81528b326e8cecd8991c6ed9b0234a29aebb69b0b6d40fa8c630b6ac406e02a` |
| Godot selected set | **6 / 4,600 / 3,839 / 178,678** | `a6220c7a57dc8eb4a76b5ae268cdfbcee29dcd6aac809cdef5f65166039c6e7e` |
| Fyrox selected set | **1 / 538 / 448 / 17,295** | `7545875d748a9e2e0a1959dd5f4284e50d7bb75d4b31019edfd9ea26c9b98150` |
| Bevy selected set | **4 / 3,041 / 2,750 / 116,334** | `a762231416f88516e251f6b75cb7c3ea664c4f80a2fda24a98c4cba84d6925c3` |
| Unity Graphics selected set | **3 / 2,728 / 2,249 / 132,901** | `e3054c02dbc9ba7e04c1582206b01da99d95ef5789b060af503784832519c247` |
| Five-engine reference total | **22 / 20,620 / 17,610 / 825,306** | `e22e4ebc65546fe0595e75daa08e445af9584d5ca7b80cef773b905a862bbf72` |

## 3. 当前真实实现与旧报告校正

### 3.1 产品表面仍是固定 projection

主 workspace固定`BS_Idle_Run / BS_Strafe_Grid / BS_Sprint_Lean`，标题却固定为`BS_Locomotion | 8 samples`。sample grid写 X=`Direction (-180..180)`、Y=`Speed (0..600)`，details写 Horizontal=`Speed 0..620`、Vertical=`Direction -180..180`，`X Axis`字段又显示`Speed 0-620`；UI内部没有单一 axis authority。

search只过滤三个静态名称且不连接 Asset Browser。navigation spec有3个tab、8个row、8个command和8个field action；6个共享 transport action只切换checked状态或把时间设为0/3秒。sample weight固定为`Run_Fwd/Run_Fwd_L/Run_Fwd_R`，validation固定为“1 sample missing”和“No duplicate samples found”。没有 pointer-to-sample hit、drag command、source mutation、transaction、compile job、runtime request或receipt。

### 3.2 canonical source 与产品注册仍分裂

真实 source 只有：

```text
AnimationStateMachineAsset
  -> AnimationStateKindAsset::BlendSpace1D { parameter, [(position, graph)] }
  -> AnimationStateKindAsset::BlendSpace2D { parameter, [(Vec2 position, graph)] }
```

该 inline source有二进制 roundtrip和direct graph reference收集，是可保留的迁移输入，但没有axis/sample stable ID与完整策略。`ResourceKind`只有AnimationGraph/AnimationStateMachine，没有BlendSpace；generic Animation document只覆盖Sequence/Graph/StateMachine。State Machine mutator只有state/transition/condition等通用操作，没有sample/axis命令。

Animation Editor把`animation.Asset.BlendSpace1D/2D`注册到不存在的plugin ZUI；Animation Graph Editor palette声明`blend_space_1d/2d`，但`AnimationGraphNodeAsset`只有Clip/Blend/Additive/Mask/Output。产品必须先冻结“standalone asset + typed player node”或“受控 inline source”的唯一 schema/lowering，不能继续靠三种字符串承诺并存。

### 3.3 shared compiler是真实进展，但还不是完整 artifact compiler

shared compiler把 1D 参数注册为`Scalar`，2D参数注册为`Vec2`，输出dense parameter slot，并产生`ZR-ANIM-COMP-STATE-013..017`诊断。它拒绝空1D、少于3点2D、non-finite、canonical signed-zero duplicate与exact collinear；collinear判定使用 robust orientation。compiled model仍只保存position与graph reference，没有sample ID、axis政策、source location、dependency generation、skeleton/mirror/profile/marker闭包或platform cook key。

Editor generic document具备revision CAS、whole-source swap transaction、同步recompile与last-known-good product。这是可复用底座，但当前没有 Blend Space document kind、细粒度命令、async build cancellation、dependency digest或runtime install CAS。Runtime cache按State Machine source revision懒编译；compile/load失败通过`.ok()?`返回`None`，没有显式 current/LKG receipt，也不把sample graph generation纳入cache key。

### 3.4 2D topology旧性能根因已关闭

`BlendSpace2D`当前使用 Spade稳定 bulk-load Delaunay，预存排序后的triangles、neighbors与hull edges。query先从per-instance triangle hint邻接游走，异常才全triangle回退，outside-hull扫描预编译边界；旧 O(n^4)三元组枚举和每帧构建hull map已经删除。96个cocircular sample、neighbor walk对照exhaustive、outside hull hint与极小/极大坐标均有测试。

剩余问题不能混入旧根因：1/2点2D被直接拒绝；inside epsilon仍使用固定`Real::EPSILON`；没有 insertion-order/cross-platform artifact digest oracle；hull仍线性扫描；没有32/64/256 compile/profile预算。这些归入数值、退化政策和qualification，不重新打开 P1-007。

### 3.5 实例缓存修正了部分热路径，输出原子性仍缺失

production `StateMachineInstanceCache`容量4096，以`AnimationParameterRevision`和compiled layout缓存dense projection与triangle hint；second-chance `VecDeque`有界淘汰，frame transaction按active entity退休，world replacement清空。compiled machine cache容量64。旧“production每帧按名字构造parameter Vec”的描述已经失效，但public `CompiledAnimationStateMachine::evaluate()`仍构造boxed projection，完整零分配合同和benchmark仍不存在。

`state_graph_sample.rs`对所有正权重sample使用同一绝对秒；normalized time取可解析graph的最大duration。event路径对resolve/evaluate失败`continue`，并从所有正权重graph收集事件，不使用weight policy；pose路径对相同失败用`?`整体返回`None`。单sample直接返回pose并丢弃weight元数据，多sample先构造`Vec<(AnimationPoseOutput, Real)>`再交给通用blend。pose、event、root motion、curve、morph与attribute没有共同sample-set receipt。

## 4. P1：Blend Space 工程化差距

### BSPACE-P1-001 · Partial · canonical source只有受限 inline carrier

`AnimationStateKindAsset`、binary roundtrip、direct reference和shared compiler已形成真实source骨架；但没有独立/versioned `AnimationBlendSpaceSourceDocument`或明确的inline descriptor，Workbench/Animation Editor/Graph palette仍各自承诺不同产品形态。应先冻结唯一 schema 与迁移策略，再开放UI。

### BSPACE-P1-002 · Open · sample没有稳定 identity、provenance和引用生命周期

sample身份仍是Vec位置，只有position与graph；reorder/duplicate/merge会改变隐式identity。需引入`BlendSampleId`、source asset/generation、origin/import recipe与reference delta，使diagnostic、selection、trace、undo和reload不依赖index/display name。

### BSPACE-P1-003 · Partial · 有typed parameter/position，没有axis domain合同

1D scalar与2D Vec2 binding、finite position和UI axis presentation已经存在；但name/unit/range/grid/snap/wrap/circular seam/default/filter/normalization不在source。Workbench两套range和X/Y互换证明UI字符串不能作为authority。

### BSPACE-P1-004 · Open · sample playback、mirror和additive metadata缺失

source没有rate scale、mirror table/profile、single-frame/frame index、loop、base/additive mode、sync group或per-sample phase offset。现有graph reference不足以工程化表达aim offset、turn、reverse和不同长度 locomotion。

### BSPACE-P1-005 · Partial · compiler有dense typed slot，runtime错误政策仍静默

compiler能检测空参数和跨用途kind冲突，production cache按revision复用projection；但missing/wrong/non-finite runtime input仍返回空sample，没有default/fallback/diagnostic receipt，也没有2D two-scalar binding与迁移政策。

### BSPACE-P1-006 · Partial · interpolation/extrapolation已有单一路径，但不可配置

1D排序线性插值和端点clamp、2D三角重心与hull projection是真实实现；UI的Triangulated/Grid/Weighted没有runtime对应，outside policy、duplicate aggregation、threshold、normalization与tie-break也不是可编译source语义。

### BSPACE-P1-007 · Closed · 旧 O(n^4) topology compiler与逐帧hull map已删除

当前 topology由Spade稳定Delaunay一次性构建，artifact内保存triangle/adjacency/hull，query使用per-instance hint邻接游走。关闭范围只针对旧算法根因；数值determinism、degenerate policy、artifact mapping和规模预算继续由P1-008/009/017负责。

### BSPACE-P1-008 · Partial · robust topology底座已建立，跨平台determinism未资格化

shared compiler使用robust orientation，Delaunay内部使用f64，triangle排序和极端scale/cocircular测试可保留；inside tolerance仍为固定f32 epsilon，没有near-degenerate矩阵、insertion-order digest、Windows/Linux replay或明确platform artifact key。

### BSPACE-P1-009 · Partial · shared compiler和Editor LKG存在，BuildSet/cook缺失

generic document有current/LKG，runtime cache有source revision与bounded compiled cache；但sample graph/clip、skeleton、mirror、marker、profile和compiler version未进入dependency digest，没有async build/cancel、artifact publication CAS、cook或install receipt。

### BSPACE-P1-010 · Partial · cache生命周期有界，qualified Blend Space handle缺失

compiled/instance cache有容量、revision检查、entity retirement与world replacement reset；但没有`BlendSpaceRuntimeHandle { asset, artifact_generation, provider_generation }`，reload/unload/provider replacement也没有typed invalidation/migration receipt。

### BSPACE-P1-011 · Open · axis filter/smoothing没有每实例状态

triangle hint不是filtered input。应为每实例保存raw/filtered value、velocity、wrap seam和filter generation，定义None/exponential/spring-damper/max-speed，以及seek/reset/teleport/pause/fixed-step/replay政策。

### BSPACE-P1-012 · Open · sample weight smoothing与per-bone policy缺失

没有区分input smoothing和weight smoothing，没有old/new sample淡入淡出、threshold/ease、per-bone override/profile或mesh/local-space策略。应在dense scratch中归一化并证明steady-state零分配。

### BSPACE-P1-013 · Open · 不同长度sample没有phase/marker leader同步

最大graph duration加共享绝对秒不是sync合同。需要normalized/cyclic time、rate/loop/seek、marker-compatible leader/follower、leader切换、sync-group rejoin与fallback trace。

### BSPACE-P1-014 · Open · event/root motion/curve/morph/attribute仲裁缺失

event当前忽略weight并允许缺graph后继续；root motion、curve、morph、custom attribute没有Blend Space策略。至少定义All/Highest/Threshold/None、dedup/order/weight context与root-motion leader/weighted accumulation，并与pose采用同一原子sample集。

### BSPACE-P1-015 · Partial · 通用pose blend可复用，Blend Space合同仍缺失

多sample可进入现有pose blend，dense target/SoA/quaternion测试由Editor202父账提供；但单sample/multi-sample错误与weight路径不等价，base/additive/mask/reference pose、missing bone、scale和per-bone profile没有Blend Space级oracle。

### BSPACE-P1-016 · Open · partial failure、fallback和evaluation receipt缺失

引入`BlendSpaceEvaluationReceipt`记录raw/filtered input、segment/triangle、sample IDs/weights/times、leader、events/root motion、artifact generation与diagnostics。sample缺失时必须按FailClosed/Renormalize/LKG之一原子处理全部输出。

### BSPACE-P1-017 · Partial · topology/query/cache已收敛，完整热路径预算仍空

O(n^4)、逐帧hull map与production parameter重投影已显著改进；仍有hull线性扫描、public boxed projection、weighted pose Vec/AoS name allocation，没有1/8/32/64/256 sample与1k/10k actor的CPU、memory、scratch和allocation基准。

### BSPACE-P1-018 · Partial · StateKind lowering存在，Graph/asset schema仍分裂

inline StateKind经shared compiler进入prepared runtime，这是可复用主干；Graph palette无asset variant，Animation Editor standalone target无资源/loader，三者没有共同`BlendSpaceNodeTypeDescriptor`。unsupported组合应在palette与compile阶段fail-close。

### BSPACE-P1-019 · Partial · generic document/transaction存在，专用toolkit命令缺失

Animation document已有revision/CAS/undo-compatible whole-source swap，Workbench也有完整视觉组件；但没有Blend Space session、projection、selection、sample hit/drag/add/remove/replace/mirror、axis/topology command、dirty/save/reopen或diagnostic navigation。

### BSPACE-P1-020 · Partial · 局部算法和视觉测试丰富，preview/debug/qualification未闭环

已有1D/2D pose、binary、source snapshot、triangle/hull、responsive layout与action测试；Preview仍固定`Run_Fwd`和queued文本，不读取runtime handle/instance receipt。缺真实subject/skeleton/time domain、reload/fault、determinism、soak/profile与跨引擎同语义对照。

## 5. P2：平台扩展与超越目标

### BSPACE-P2-001 · Open · 高维/RBF与非规则 Blend Field

在1D/2D资格完成后评估3D、RBF、simplex cloud与高维feature field；统一复用sample ID、artifact、handle和receipt，不建立第二运行时。

### BSPACE-P2-002 · Open · motion analysis与自动sample placement

从clip/root motion/trajectory/pose feature分析axis值、coverage hole、duplicate和topology质量，以可审阅proposal + transaction apply交付。

### BSPACE-P2-003 · Open · contact/phase-aware blending与motion warp协同

把foot contact、phase marker、stride/turn/root-motion约束纳入sample选择和输出修正，并保留原始evaluation receipt与后处理receipt。

### BSPACE-P2-004 · Open · 大规模角色LOD、pose sharing与GPU协同

支持按可见性/距离降低sample数量、共享phase/pose page、批量parameter query和deformation upload；降级不能破坏event/root-motion authority。

### BSPACE-P2-005 · Open · learned parameter mapping与自适应权重

learned/RBF mapper只能作为versioned compiler/evaluator策略，必须带训练数据provenance、deterministic fallback、quality metric和traditional topology oracle。

## 6. 五套参考源码裁决

### 6.1 Unreal：主参考定义完整资产、prepared topology与实例状态

`UBlendSpace`是独立`UAnimationAsset`。`FBlendParameter`包含display name、min/max、grid divisions、snap与wrap；`FBlendSample`包含animation、sample value、rate scale、mirror、single-frame与frame index。资产还覆盖axis interpolation、sample weight smoothing、per-bone/profile、notify trigger、marker leader和prepared grid/triangulation；player node保存输入、play rate、loop、reset/teleport等实例语义。

Persona grid/editor/details/analysis使用真实asset mutation、`FScopedTransaction`、add/move/delete/replace与analysis proposal。应迁移的是source/prepared/instance/transaction/receipt分层，不是UObject或Slate形态。

### 6.2 Godot：显式点/triangle、sync mode与UndoRedo闭环

Godot 1D/2D资源提供blend point add/remove/move/name、interpolated/discrete/discrete-carry、None/Independent/Cyclic Mutable/Cyclic Constant sync；2D持久化triangle并支持auto triangle。Editor对move/config/label/add/remove/reorder/name调用`EditorUndoRedoManager`。其动态属性形式不必照搬，但“资源操作即运行时source操作”的闭环值得采用。

### 6.3 Fyrox：Rust pose node与event collection策略

Fyrox `BlendSpace`把point关联到PoseNode，计算三点权重并按All/MaxWeight/MinWeight收集事件，说明事件政策必须显式而不能忽略weight。其当前线性triangle查询和较轻source模型只作语义次参考，不作为Zircon最终规模上限。

### 6.4 Bevy：没有首方 Blend Space，只提供prepared graph边界

Bevy的AnimationGraph是可序列化asset，运行时构建threaded/prepared traversal，node有weight与mask，event/transition遵循typed player生命周期。它不能证明Blend Space特性，但可约束Zircon不重复发明graph、mask、asset generation与event traversal。

### 6.5 Unity Graphics：只约束下游实例/deformation currentness

Unity Graphics选择集展示GPU resident drawer、instance data system及editor tests中的bounded instance lifecycle和generation-sensitive更新。它不拥有Blend Space source/editor/evaluator，只用于检查最终pose/deformation consumer不会接受stale或半提交结果。

## 7. 目标架构与硬切规则

```text
AnimationBlendSpaceSourceDocument
  +-- BlendAxisDescriptor[1..2]
  +-- BlendSampleDescriptor[stable BlendSampleId]
  +-- interpolation/filter/sync/event/root-motion/per-bone policy
  +-- skeleton/mirror/profile/sample dependency references
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
BlendSpaceEvaluationRequest -> BlendSpaceEvaluationReceipt
          |
          +--> State Machine / Animation Graph / PreviewWorld
          +--> pose/curve/morph/attribute + event/root motion
          +--> renderer deformation handoff
```

硬切规则：

1. provider/source不可用时，普通Editor入口隐藏或显示Unavailable；删除固定`BS_*`、weight、validation与queued成功文案。
2. standalone asset、inline state和Graph node只能共享一个versioned schema/compiler；旧inline payload仅作为迁移输入。
3. 保留Spade prepared topology与hint walk，不恢复自制O(n^4)或frame-time hull rebuild。
4. frame内禁止解析locator字符串、重编topology、按名字重投影参数或无界分配scratch。
5. pose/event/root motion/curve/morph/attribute必须消费同一sample/time/weight receipt。
6. sample Vec index、display name与ZUI control ID不得作为持久identity。
7. URI字符串测试、layout screenshot、weight-sum test和代码存在性不能关闭产品gate。
8. 不建立Editor私有Blend Space DTO或第二套pose buffer；消费shared Animation schema/compiler与dense pose主干。

## 8. 重构里程碑

### BSPACE-M0 · Capability truth、owner与回归基线

- 隐藏不可用入口，移除固定成功feedback；冻结standalone/inline/player node唯一政策。
- 固化现有1D、Spade 2D、hint/cache和binary happy-path oracle，保留旧payload迁移fixture。

### BSPACE-M1 · Source schema、identity与migration

- 建立axis/sample stable ID、metadata、output policy和dependency reference。
- 注册ResourceKind/asset/document或明确受控inline carrier；完成旧StateKind单向migration。

### BSPACE-M2 · Semantic compiler与deterministic prepared artifact

- 在现有Spade基础上完成1/2点退化、near-degenerate/cocircular、source mapping与canonical digest。
- 编译axis/filter/sync/event/root-motion/per-bone政策，unsupported mode fail-close。

### BSPACE-M3 · Build/cook/currentness与qualified handle

- 建立BuildSet、async job、cancel/LKG、CAS publication、platform cook和dependency invalidation。
- 实现qualified runtime handle、reload/unload/provider replacement/world teardown与instance migration。

### BSPACE-M4 · 实例级filter、weight与time sync

- 实现axis filter/wrap、sample weight smoothing、threshold与per-bone/profile。
- 实现normalized/marker leader/follower、rate/loop/seek/reset/sync group并trace全过程。

### BSPACE-M5 · 完整动画输出与Graph/State集成

- 统一pose/curve/morph/attribute/event/root-motion sample集合与failure receipt。
- 完成base/additive/mask/quaternion/per-bone blend，Graph/State只消费同一typed descriptor。

### BSPACE-M6 · 真实Editor toolkit与transaction

- 实现document session、sample canvas、axis/sample inspector、topology与diagnostic navigation。
- add/move/remove/replace/mirror/triangle/axis操作可undo/redo、dirty/save/reopen，interactive drag合并transaction。

### BSPACE-M7 · Runtime-backed PreviewWorld与debugger

- Preview使用真实subject、skeleton、prepared handle、instance state和time domain。
- grid/heatmap/timeline/filter/phase/event/root motion全部来自同一receipt，compile失败显示LKG generation。

### BSPACE-M8 · Fault、scale、performance与硬切

- 通过malformed/reload/unload/provider replacement、determinism、soak与scale矩阵。
- 建立同语义Unreal/Fyrox/Godot功能/质量/CPU/memory对照，删除fake workspace与parallel schema。

## 9. 48 个资格门

状态说明：`Partial`只承认可复用共享基础或局部算法，不表示Blend Space vertical slice已通过；`Pass`需要真实产品动态证据。

### 9.1 Authority与Schema

| Gate | 当前 | 资格条件 |
|---|---|---|
| BSPACE-G-01 | Fail | ordinary Editor只在canonical provider/source可用时暴露Blend Space入口。 |
| BSPACE-G-02 | Partial | inline source/binary/compiler已存在；standalone/Graph仍未消费同一versioned schema。 |
| BSPACE-G-03 | Fail | axis具有stable ID、binding、unit、range、grid/snap、wrap与normalization合同。 |
| BSPACE-G-04 | Fail | sample具有stable ID、source generation、rate、mirror、single-frame和mode metadata。 |
| BSPACE-G-05 | Fail | add/remove/move/reorder/rename后selection/reference/diagnostic按ID稳定。 |
| BSPACE-G-06 | Partial | 1D Scalar/2D Vec2 dense binding已编译；two-scalar migration与runtime错误政策缺失。 |
| BSPACE-G-07 | Partial | 旧inline source可binary roundtrip/direct reference；完整axis/sample/topology policy不在payload。 |
| BSPACE-G-08 | Fail | 旧StateKind payload完成单向迁移，parallel write path被删除。 |

### 9.2 Compiler、Topology与Cook

| Gate | 当前 | 资格条件 |
|---|---|---|
| BSPACE-G-09 | Partial | compiler拒绝non-finite/empty/duplicate/collinear；rig/additive/mirror/range/source定位缺失。 |
| BSPACE-G-10 | Partial | duplicate/collinear/cocircular/extreme scale有局部测试；1/2点与near-degenerate政策不全。 |
| BSPACE-G-11 | Partial | stable bulk-load与sorted topology存在；insertion/order/cross-platform digest未证明。 |
| BSPACE-G-12 | Fail | Windows/Linux对同source生成相同artifact digest或显式platform key。 |
| BSPACE-G-13 | Partial | prepared runtime保存triangles/adjacency/hull；缺segment/source mapping与自包含dependency。 |
| BSPACE-G-14 | Partial | O(n^4)已删除且96点测试存在；32/64/256 compile时间/内存预算未建立。 |
| BSPACE-G-15 | Fail | BuildSet覆盖sample graph/clip、skeleton、mirror、marker、profile和compiler version。 |
| BSPACE-G-16 | Partial | Editor generic LKG与runtime revision cache存在；dependency CAS/cook/install receipt缺失。 |

### 9.3 Runtime Input、Filter与Weights

| Gate | 当前 | 资格条件 |
|---|---|---|
| BSPACE-G-17 | Partial | production instance按revision缓存dense projection；public evaluate仍分配且无完整预算。 |
| BSPACE-G-18 | Fail | missing/wrong/non-finite input产生stable receipt，不静默空pose。 |
| BSPACE-G-19 | Partial | 1D clamp和2D hull projection可测；wrap/circular seam与可配置outside policy缺失。 |
| BSPACE-G-20 | Fail | None/exponential/spring filter在fixed/variable/seek/reset下确定。 |
| BSPACE-G-21 | Fail | input smoothing与sample-weight smoothing是独立可组合政策。 |
| BSPACE-G-22 | Fail | old/new sample淡入淡出、threshold与renormalization保持权重和为1。 |
| BSPACE-G-23 | Partial | dense pose/per-bone共享底座存在；Blend Space profile/space政策未接入。 |
| BSPACE-G-24 | Partial | topology/query/parameter cache已去除主要重复工作；完整sample/filter/pose路径仍非零分配。 |

### 9.4 Time与Animation Output

| Gate | 当前 | 资格条件 |
|---|---|---|
| BSPACE-G-25 | Fail | 不同duration/rate sample按明确normalized/cyclic政策推进。 |
| BSPACE-G-26 | Fail | marker leader/follower、leader变化与不兼容fallback可追踪。 |
| BSPACE-G-27 | Fail | loop/reverse/seek/teleport/pause/step/sync-group rejoin行为确定。 |
| BSPACE-G-28 | Fail | event支持All/Highest/Threshold/None并具有order/dedup/weight context。 |
| BSPACE-G-29 | Fail | root motion使用明确leader或weighted accumulation且与pose sample集一致。 |
| BSPACE-G-30 | Fail | curve/morph/custom attribute使用同一sample/time/weight receipt。 |
| BSPACE-G-31 | Partial | 通用pose blend与quaternion oracle可复用；Blend Space additive/mask/per-bone全矩阵缺失。 |
| BSPACE-G-32 | Fail | sample缺失按FailClosed/Renormalize/LKG原子影响全部输出。 |

### 9.5 Editor与Preview

| Gate | 当前 | 资格条件 |
|---|---|---|
| BSPACE-G-33 | Fail | Asset Browser能Create/Open/Save/Reopen/Rename/Delete Blend Space。 |
| BSPACE-G-34 | Fail | axis/sample inspector编辑typed source projection，不保存ZUI字符串。 |
| BSPACE-G-35 | Fail | add/drag/remove/duplicate/replace/mirror完整undo/redo并合并interactive move。 |
| BSPACE-G-36 | Fail | auto/manual triangle、snap、boundary和compile diagnostic在canvas可视化。 |
| BSPACE-G-37 | Partial | generic Animation document有revision/CAS/dirty/LKG；Blend Space session/command/reimport冲突缺失。 |
| BSPACE-G-38 | Fail | PreviewWorld subject、skeleton、asset handle和instance generation可检查。 |
| BSPACE-G-39 | Fail | grid/heatmap/timeline/filter/phase/event/root motion来自真实receipt。 |
| BSPACE-G-40 | Fail | missing provider/source/compile failure显示Unavailable/LKG，不显示fixed queued success。 |

### 9.6 Qualification与Hard Cut

| Gate | 当前 | 资格条件 |
|---|---|---|
| BSPACE-G-41 | Partial | inside/outside/vertex/edge/cocircular/extreme scale有局部oracle；完整duplicate/degenerate矩阵不足。 |
| BSPACE-G-42 | Fail | wrong parameter、missing graph、reload/unload/provider replacement与fault injection通过。 |
| BSPACE-G-43 | Partial | source binary与旧payload fallback测试存在；artifact migration/reference repair/old-version矩阵不足。 |
| BSPACE-G-44 | Fail | 1k/10k actor与1/8/32/64/256 sample满足CPU/memory/allocation预算。 |
| BSPACE-G-45 | Fail | 长时loop/marker/filter/weight smoothing soak无phase drift、NaN或generation泄漏。 |
| BSPACE-G-46 | Fail | Windows/Linux deterministic replay产生相同topology/weights/events/receipt digest。 |
| BSPACE-G-47 | Fail | 同语义Unreal/Fyrox/Godot对照记录功能、质量、CPU和memory。 |
| BSPACE-G-48 | Partial | O(n^4)与frame hull map已硬删；fake workspace、parallel schema与compat write path仍在。 |

## 10. 实施顺序、停止条件与复核边界

1. 先执行M0能力诚实化和M1唯一source/schema；不得先把静态workspace接到另一套Editor私有DTO。
2. 保留现有Spade topology、1D binary search、dense parameter projection、triangle hint和bounded cache，围绕它们补合同与资格。
3. 在Editor202 dense pose/atomic consumer主干未闭合前，不允许Blend Space旁路建立第二套AoS pose或renderer upload。
4. M2后必须先通过deterministic artifact与degenerate RED矩阵；否则不得进入大规模Editor canvas。
5. M4/M5必须用同一request/receipt同时驱动pose、event、root motion、curve、morph与debug trace。
6. M6/M7必须验证真实Create/Edit/Undo/Save/Compile/Preview/Reload/Reopen用户流；视觉截图只作布局回归。
7. M8没有profile与同语义基准前，文档和UI不得出现“优于Unreal”的性能/质量结论。
8. 实施前重新冻结所有modified/untracked owner文件；本报告的fingerprint不是未来实现的替代验收。

本报告完成的是Editor82 Blend Space链的当前源码复核和重构规格，不代表任何production重构已经完成。
