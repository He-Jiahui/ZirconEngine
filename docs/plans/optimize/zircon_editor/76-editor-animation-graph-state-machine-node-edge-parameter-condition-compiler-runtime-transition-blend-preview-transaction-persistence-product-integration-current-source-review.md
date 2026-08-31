---
title: Editor Animation Graph、State Machine、Node-Edge、Parameter、Condition、Compiler、Runtime Transition、Blend、Preview、Transaction、Persistence 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor76
review_date: 2026-08-23
baseline_head: 0d70d1ac6499abcf56c3f6c3ef43cb3a7502a249
baseline_epoch: 351
related_code:
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/binding/animation
  - zircon_editor/src/ui/binding_dispatch/animation
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/animation_graph.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/animation_projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/assets/ui/editor/animation_editor.zui
  - zircon_editor/assets/ui/editor/host/animation_graph_body.zui
  - zircon_plugins/animation_graph/editor
  - zircon_plugins/animation/runtime/src/evaluation/compiled_graph
  - zircon_plugins/animation/runtime/src/evaluation/pipeline
  - zircon_plugins/animation/runtime/src/state_machine
  - zircon_plugins/animation/runtime/src/manager
  - zircon_runtime/src/animation/manager
  - zircon_runtime/src/core/framework/animation
tests:
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/tests/editor_event/animation_runtime
  - zircon_editor/src/tests/editor_event/locator_protocol_hard_cut.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/binding_dispatch/animation.rs
  - zircon_editor/src/tests/host/retained_animation_template_body.rs
  - zircon_plugins/animation_graph/editor/src/tests.rs
  - zircon_plugins/animation/runtime/src/tests.rs
  - zircon_runtime/src/core/framework/tests/framework_surfaces.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Private/AnimBlueprintCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Private/AnimationStateMachineSchema.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Private/AnimBlueprintExtension_StateMachine.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Private/AnimBlueprintPostCompileValidation.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Private/AnimStateTransitionNode.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimNode_StateMachine.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNode_StateMachine.cpp
  - dev/godot/scene/animation/animation_node_state_machine.h
  - dev/godot/scene/animation/animation_node_state_machine.cpp
  - dev/godot/editor/animation/animation_state_machine_editor.cpp
  - dev/godot/editor/animation/animation_blend_tree_editor_plugin.cpp
  - dev/godot/editor/animation/animation_blend_space_1d_editor.cpp
  - dev/godot/editor/animation/animation_blend_space_2d_editor.cpp
  - dev/Fyrox/fyrox-animation/src/machine
  - dev/Fyrox/editor/src/plugins/absm
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/GraphData.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Drawing/Views/MaterialGraphView.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/VFXGraph.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/VFXErrorManager.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/NewCompiler/VfxGraphCompiler.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Animation Graph、State Machine、Node-Edge、Parameter、Condition、Compiler、Runtime Transition、Blend、Preview、Transaction、Persistence 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon并非没有动画图和状态机底座。资产层已经有Clip、Blend、Additive、Mask、Output节点，状态机支持GraphRef、Clip、BlendSpace 1D/2D、SubMachine、transition exit time/interruption、layers与mask；插件Runtime也已经实现dense slot编译、output可达环检测、typed condition evaluator、nested machine lineage、compiled graph/state-machine cache、pose blend和clip event采样。这些代码应保留其算法意图，而不是推倒后再写一套临时解释器。

但Editor到Runtime没有形成一个工程级语义闭环。当前至少存在三套互不等价的“编译/执行”权威：Animation Graph Editor插件的`compile_animation_graph`只返回Output source字符串，状态机compile只返回entry/state/transition计数；`zircon_runtime::animation::DefaultAnimationManager`执行宽松递归解释；`zircon_plugins::animation::runtime`则使用另一套compiled graph/state-machine pipeline。更严重的是，Runtime内置模块和Animation Runtime插件各自注册同名`animation.runtime`模块与同名`animation.runtime.Manager.DefaultAnimationManager`服务，却暴露不同Rust类型和不同状态机能力，最终语义取决于装配拓扑，而不是资产与artifact版本。

Editor命令虽然携带locator，却不是fail-close。`resolve_animation_graph_instance`在locator解析失败或找不到对应实例后静默回退到当前focused `editor.animation_graph`视图；Graph和State Machine共用该view id，因此stale/delayed/durable命令可以落到另一份已聚焦状态机并修改错误资产。这个问题不是父报告中“缺transaction”的重复表述，而是当前可达的跨文档错误写入入口，必须先于任何新节点功能关闭。

真实产品仍只有header、一个Add Node按钮和空Canvas。该按钮固定发送`animation://selected/graph + new_state + State`，而session mutator只接受`output`和`blend`，所以点击是确定性no-op。插件palette又声明Clip、Blend、BlendSpace1D/2D、Output，资产Graph实际枚举为Clip、Blend、Additive、Mask、Output，三者没有共享descriptor/schema。pane payload只发布字符串列表；没有node position、pin、edge、typed parameter、condition tree、compiler diagnostic、runtime state/debug trace或preview subject。

本报告对Editor14既有5项Animation P0与通用graph/state-machine差距不重复计数；新增的是当前源码已证明的错误寻址、Runtime服务身份冲突、三套语义权威的具体断裂、cache currentness、silent failure、dependency/condition/layer/blend-space schema和hot-path成本。新增 **2项P0、20项P1、8项P2与48个资格门**。目标不是扩充现有字符串mutator，而是hard cut到唯一`AnimationSemanticCompiler + CompiledAnimationProgram + AnimationRuntimeService`，Editor只通过qualified document transaction编辑同一份typed schema，并用同一artifact/evaluator做preview与游戏运行。

本轮只做review与文档，不修改生产源码。未运行Cargo、真实Editor、GUI/GPU、native input、save/reopen、plugin reload、fault/soak/profile或同语义跨引擎benchmark，因此不能宣称当前性能、正确性或表现达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | fingerprint |
|---|---:|---|---|
| Editor authoring/product | **32 / 6,707 / 6,339 / 247,486 / 19** | session mutation、binding/event、locator resolution、pane projection、template与ZUI产品 | `a9929d50450afd657685cd06a42175eb96e99071a223466cac60cb164629dd97` |
| Schema/extension/editor compiler | **19 / 2,635 / 2,399 / 87,831 / 12** | graph/state asset、binary upgrade、reference analysis、descriptor/palette与Editor插件validator/compiler | `fa6511fc59e0283db8e874e44e90600e7acbc856ced08b4af87508dac7a7fa84` |
| Runtime compiler/evaluator | **89 / 10,197 / 9,402 / 354,064 / 28** | compiled graph/state machine、condition、blend space、layer、nested transition、pipeline cache与双manager | `48057ff1e842f58ba0ace434b616bd61e699deefedc12819e436ae3c070d94fd` |
| Focused tests | **16 / 4,077 / 3,840 / 150,246 / 58** | session、event、host、plugin runtime/editor与framework surface tests | `1f588a5af39768121e9369ed8ec7192a55ef8bcd25862a0256edfff106b18c3a` |
| Zircon去重合计 | **156 / 23,616 / 21,980 / 839,627 / 117** | 四组按normalized path去重的working-tree逐文件扫描 | `e092a183d6b40cdc209a313c2d42a3a45d9ded358d963bad4a195dc98630f660` |
| Unreal selected set | **10 / 8,499 / 7,177 / 331,747 / 0** | AnimBlueprint compiler/schema/validation/debug maps与runtime state machine | `b3487bed0bbedc1b0df9559c17523ee256aac562e360300db7eac3dd59f71810` |
| Godot selected set | **14 / 11,505 / 9,582 / 441,840 / 0** | state machine/blend tree/blend space runtime资源、编辑器与UndoRedo | `591f543053a3991501cd58dc8386b55f0caafb1b6132ccadf8be7b7715ab53e5` |
| Fyrox selected set | **29 / 11,484 / 10,412 / 427,278 / 11** | ABSM machine/parameter/logic/layer/node与完整command/editor surface | `aa63a08d1462296ab050ef4d93d00a648717daa007d3268423e1633f35d1e2c9` |
| Bevy selected set | **3 / 2,979 / 2,692 / 113,433 / 9** | petgraph asset、threaded evaluation cache、player/transition边界 | `f8835c768eb054f0c2d1d8fbebd4c80e93c23a365f579e02cb636246bec9ee1a` |
| Unity Graphics selected set | **6 / 6,965 / 5,914 / 283,836 / 0** | ShaderGraph undo/delta/validation与VFX graph migration/compiler/error architecture | `18dde5ce743d8f7fee28db72d9ed8d1a4cb1a469d927e501d39b4f1e36e6697b` |
| 五引擎参考合计 | **62 / 41,432 / 35,777 / 1,598,134 / 20** | 五类本地参考按path去重 | `96377ddace595d2c3f4053aee2ae6922c3abf74f6ed9d357677b6a611ea43132` |

fingerprint算法沿用Editor58-75：按normalized lowercase relative path排序，把`path + NUL + lowercase per-file SHA-256 + LF`串联后再取SHA-256。test declarations统计Rust test attribute、C++ `TEST*`与C# `Test/TestCase`声明。指标冻结于完整逐文件扫描时；实现前必须按working tree重算。

聚焦Runtime文件在共享工作树中存在import排序、断行及测试重排等未提交差异，本轮逐行按当前源码审查且不回滚。插件`manager/sampling.rs`的quaternion分支静态漂移在baseline HEAD也存在，不是本轮或其他Session新引入。因为本轮明确不运行Cargo，报告只把它称为源码级编译阻断风险，不声称已经复现编译失败。

### 2.2 当前产品与运行链

```text
Animation Graph / State Machine asset
  -> same editor.animation_graph view id
  -> string-only pane payload
  -> header + Add Node + empty CanvasBox
  -> AddNode(animation://selected/graph, new_state, State)
  -> session accepts only output/blend
  -> deterministic no-op

Durable editor command
  -> locator string
  -> exact open-instance lookup
  -> on parse/miss: focused editor.animation_graph fallback
  -> mutation may target another graph/state-machine document

Editor plugin "compiler"
  -> string diagnostics
  -> graph: output source String
  -> state machine: entry + counts
  x no compiled runtime artifact / dependency stamp / install receipt

Runtime path A
  -> zircon_runtime::animation::DefaultAnimationManager
  -> permissive recursive graph + GraphRef-only state output

Runtime path B
  -> zircon_plugins::animation::runtime::AnimationEvaluationPipeline
  -> compiled graph/state machine/layers/nested machines
  -> compile/load errors collapsed to None
```

### 2.3 可保留底座

1. 保留asset binary envelope与V1/V2/V3 upgrade入口，但升级到stable element identity、typed schema revision与unknown-node preservation。
2. 保留compiled graph的dense slots、output-reachable cycle rejection、skeleton target table和bounded compiled cache。
3. 保留condition evaluator的`Condition / All / Any / Not`、depth limit、parameter table和trigger consumption意图，并让资产直接持久化同一AST。
4. 保留transition runtime的exit time、interruption policy、crossfade state和nested lineage防环意图。
5. 保留blend-space几何权重、layer pose blend、clip revision timing cache与typed layer mismatch diagnostic。
6. 保留Editor event codec携带asset locator的方向，但locator miss必须fail-close并携document/session/revision资格。
7. 保留GraphEditorDescriptor/palette作为扩展发现机制，但descriptor必须来自Runtime唯一node schema registry。
8. 保留reference analysis对graph/state-machine direct references的扫描，并扩展为typed dependency closure与artifact currentness。

## 3. 父报告校正、唯一owner与不重复计数

| 既有owner | 当前源码重验 | 本报告裁决 |
|---|---|---|
| Editor14 P0-1/P0-2 | 默认toolkit仍不可达，真实Graph body仍为空Canvas，Add Node固定发送不被mutator接受的`State` | 产品不可达/可见假能力继续由Editor14计数；Editor76只登记其具体schema drift与fail-close门 |
| Editor14 P0-3 | graph/state mutation仍通过blanket changed/dirty且无transaction receipt | 通用transaction/history归Editor14与Editor63；本报告不重复P0，只定义typed graph command适配器 |
| Editor14 P0-4 | save不调用共享semantic compiler，Editor插件compile也不产生runtime artifact | “无共享compiler”继续由Editor14；Editor76新增的是同名Runtime服务身份冲突与具体三authority差异 |
| Editor14 P0-5 | Editor preview仍未消费Runtime evaluator | 继续由Editor14；Editor76只规定compiled artifact/debug trace接入条件 |
| Editor14 P1-28至P1-40 | pins/type/cardinality/cycle/layout/node registry/migration/incremental compile/debug普遍缺失 | 不复列相同功能清单；Editor76只登记新证明的descriptor-asset-mutator三方冲突、cache与runtime语义 |
| Editor14 P1-41至P1-52 | entry/transition identity/condition/layer/blend-space/advanced tools通用差距仍成立 | 由Editor14唯一计数；本报告聚焦persisted AST与现有runtime compiler失联、silent failure和dependency currentness |
| Editor63 | transaction、history、savepoint、document scope与async receipt由其拥有 | Graph/State Machine提供domain command、inverse payload和qualified target，不另造history系统 |
| Editor69 | preview clock、cadence、pause/step和runtime-world隔离由其拥有 | Animation只提供artifact install、state trace、pose/debug projection与animation-specific event policy |
| Editor75 | timeline、transport、scrub、track/key/curve/snap/clipboard由其拥有 | Graph/State Machine只消费统一preview session和time authority，不重建timeline |

## 4. P0：当前可达的数据与运行时权威风险

### ED76-P0-01 · locator未命中时回退focused view，可把持久动画命令写入错误资产

`resolve_animation_graph_instance(Some(locator))`先尝试`find_animation_editor_instance("editor.animation_graph", locator)`；任何URI parse失败、路径规范化差异、文档关闭、重开后instance变化或delayed replay miss都会落入focused-view分支。Graph与State Machine刻意共用`editor.animation_graph` descriptor，因此focused view可能是另一份Graph或State Machine。Graph命令可能对错误Graph生效；State命令在focused为另一份State Machine时会正常成功并dirty错误文档，而不是返回target-stale。当前测试覆盖codec和普通focused路径，但没有双文档、stale locator、close/reopen或durable replay错序的fail-close E2E。

必须建立不可伪造的`AnimationDocumentTarget { document_session_id, asset_id, asset_revision, view_instance_id, session_epoch }`。携locator的命令只能exact resolve；locator缺失才允许显式的interactive-focus command，而且在入队时冻结target snapshot。任何revision/session mismatch返回typed stale receipt，不得回退、猜测或静默no-op。

### ED76-P0-02 · 同名Animation module/manager由两套不同实现注册，运行语义取决于装配拓扑

`zircon_runtime/src/animation`和`zircon_plugins/animation/runtime`各自定义`DefaultAnimationManager`与AnimationModule，均使用`animation.runtime`及`animation.runtime.Manager.DefaultAnimationManager`名称；现有测试还明确断言两个manager的`TypeId`不同。内置manager与插件manager复制graph/state/parameter/pose/sampling实现，插件production pipeline又绕过该public manager走compiled evaluator。内置/public state evaluator最终只投影`graph_reference()`，因此Clip、BlendSpace和SubMachine state不会得到等价输出；compiled pipeline却支持这些state、layers、exit/interruption和nested runtime。

这不是单纯“尚无共享compiler”：同一稳定服务名可指向两个类型、两套缓存和不同能力，linked/unlinked plugin装配能改变资产执行语义。必须hard cut为单一owner crate和单一`AnimationRuntimeService`，模块注册时拒绝重复stable identity；legacy manager与插件副本删除，不保留re-export shim或兼容解释器。Editor compiler、preview、game runtime和headless test必须安装同一artifact格式并通过semantic parity测试。

## 5. P1：本轮新增的工程差距

### ED76-P1-01 · palette、资产枚举与session mutator是三套互相矛盾的节点schema

插件palette声明Clip、Blend、BlendSpace1D、BlendSpace2D、Output，却漏掉Graph资产真实支持的Additive与Mask，并把只存在于StateKind的BlendSpace放进Graph palette。session `add_graph_node`又只接受`output`和`blend`；产品按钮发送`State`。需要Runtime-owned `AnimationNodeTypeDescriptor`统一type id、版本、pins、properties、compiler lowering、icon/category和migration，Editor palette与commands只消费descriptor snapshot。

### ED76-P1-02 · connection命令只有from/to字符串，没有pin、edge identity、type与cardinality

`ConnectGraphNodes`无法表达连接Additive的base还是additive，也无法表达Blend输入顺序/权重slot；实现只能把Additive永远写入base、Mask写input、Output写source，Blend则append字符串。Disconnect按from/to清除所有匹配，不能选中一条边、重排输入或保留edge metadata。需要stable `NodeId + PinId + EdgeId`、typed pose/value pin、direction、cardinality、ordering和connect-plan diagnostic。

### ED76-P1-03 · 删除节点会把必填引用清成空字符串并保存无效中间态

remove会删除Blend引用，却把Additive base/additive、Mask input和Output source清为空字符串；session立即dirty，save又不要求semantic compile成功。工业级编辑器必须在transaction内执行`DeletePlan`：显式列出被删edges、replacement/rewire、orphan diagnostics和inverse payload；如果允许invalid authoring state，也只能保存为可恢复source并阻止runtime artifact替换，不能让空字符串冒充合法身份。

### ED76-P1-04 · 节点、状态、transition与layer都以可变字符串/数组位置充当身份

Graph Output甚至没有node id，只通过特殊字符串`output`寻址；State、Transition、Layer无stable GUID，布局位置未进入authoring metadata。重命名、复制、merge、undo、diagnostic定位、runtime trace和版本迁移都依赖脆弱字符串。binary node tag采用union式结构并遗留未使用字段，未知新node无法保真往返。需要stable IDs、source span/property path、separate authoring layout、per-kind version与unknown payload preservation。

### ED76-P1-05 · State Machine没有自己的typed parameter schema，条件类型在三层被猜测

Graph参数有name/default value，State Machine却只在condition里存parameter字符串。Editor按literal顺序推断Trigger/Bool/Integer/Scalar/Vec；runtime ParameterTable只intern名字，比较时还把Scalar/Integer/Bool数值化；missing、wrong type和non-finite统一返回false。必须由machine/program声明`ParameterId + type + default + mutability + replication/source policy`，compile静态检查每个condition与graph dependency，runtime只接收layout-compatible parameter block。

### ED76-P1-06 · Runtime拥有All/Any/Not表达式编译器，资产只能持久化flat AND列表

`ConditionExpression`已支持Condition、All、Any、Not并限制深度，但`AnimationStateTransitionAsset`只保存`Vec<Condition>`，machine compiler始终调用`compile_all_conditions`。OR/NOT无法被Editor创建、保存或迁移，runtime能力成为孤岛。应把同一typed AST持久化，提供stable expression-node IDs、short-circuit/trigger consumption规则、schema migration和可定位diagnostics。

### ED76-P1-07 · transition选择依赖Vec顺序，没有stable identity、显式priority或冲突证明

compiled machine把source transition按资产数组顺序组织，evaluate选择第一条满足条件的transition；asset没有TransitionId/priority，Editor又把同一from/to视为唯一并覆盖duration。文件重排、merge或migration即可改变运行结果。需要显式priority/tie-break、stable transition ID、multi-edge支持、ordered compile receipt和ambiguity diagnostic；Editor14继续拥有通用transition authoring总账。

### ED76-P1-08 · Editor validator与Runtime compiler对同一资产给出不同接受/修正规则

Editor Graph validator检查empty Blend和`playback_speed <= 0`，但NaN会穿过比较；Runtime compiler对non-finite playback静默改为1.0，却允许empty Blend产生零clip结果，并增加cycle/skeleton mask检查。State Editor validator只拒绝negative duration，Runtime `TransitionDesc`会把non-finite/非正duration归零并clamp exit time。一个资产可能在Editor显示通过、Runtime静默改义，或Editor拒绝而Runtime接受。必须只有一个semantic compiler和同一diagnostic code set，不允许各层重复手写validator。

### ED76-P1-09 · graph/state/layer/load失败被`.ok()?`折叠为无输出，没有last-good或错误产品

graph cache、state-machine cache、layer compile和多处asset load把typed error转为`None`；调用方通常`continue`，最终角色停止输出pose。Clip evaluator已有`AnimationEvaluationDiagnostic`，graph/state compiler却没有等价事件、asset path、element address、dependency chain、first/last occurrence或recovery状态。需要`CompileResult { artifact?, diagnostics, dependency_stamp }`、last-known-good install policy和Editor/runtime同源Error List。

### ED76-P1-10 · frame graph evaluation cache在读取asset revision之前命中，可在同帧热更后返回旧结果

`evaluate_graph`先按`graph_id + skeleton_id + full parameter map`线性命中frame cache并直接返回，之后才加载graph/skeleton snapshot并验证compiled cache revision。同一个evaluation frame内资源revision变化不会使该entry失效。cache key必须包含artifact ID/revision与skeleton binding revision，asset install还要主动invalidate；frame cache只缓存immutable compiled artifact的evaluation，不缓存未经currentness证明的source身份。

### ED76-P1-11 · 缺失clip/graph依赖时normalized time返回1.0，可意外满足exit time

`normalized_graph_time`在没有可用duration或time非有限时返回1.0；direct Clip state加载失败也`unwrap_or(1.0)`。带exit time的transition因此可能在源动画缺失、编译失败或依赖尚未加载时立即通过，掩盖真实故障并改变游戏状态。缺依赖必须进入明确`Unavailable/Stale/Error`状态，transition policy选择hold/fail/last-good，不能把“未知时长”解释为“播放完成”。

### ED76-P1-12 · direct Clip state把looping固定为false，StateKind没有播放/同步语义

`sample_state_clip_events`固定`looping: false`，Clip state只持有asset reference；没有play rate、loop policy、start offset、sync group/marker、root motion、event policy或state-local time scaling。Graph Clip具备looping/playback speed，两条路径语义不一致。需要共享`AnimationClipPlayerSpec`及同步/marker compiler，而不是按StateKind分叉默认值。

### ED76-P1-13 · N路Blend用一个scalar硬编码权重，不能表达工程级blend graph

compiled graph对N个inputs使用`first = 1-scalar`、其余平均分配`scalar/(N-1)`；没有per-edge weight、normalization policy、negative/overdrive、sync、pose cache或blend profile。该算法可作为两路线性blend primitive，但不能作为通用Blend节点契约。descriptor应明确2-way blend、multi-blend、blend-by-enum、blend-list等不同节点，compiler生成每路weight program。

### ED76-P1-14 · Layer mask是与骨架身份脱离的position Vec，compile时没有skeleton binding

asset layer保存name/ref/weight/blend mode和mask权重数组；layer compiler只验证有限值/范围，不能证明mask长度、bone mapping或skeleton revision。真正不匹配到pose blend时才报bone count/name错误，而上游compile错误仍会被吞掉。需要skeleton-qualified mask asset、stable bone/target IDs、binding artifact、dependency revision和per-layer diagnostic；Editor还必须能author/preview solo/mute/order。

### ED76-P1-15 · nested machine的环与深度限制只返回None，dependency graph未在compile期封闭

`MachineInstanceKey`以固定depth 8与lineage contains防环，失败直接`None`；SubMachine reference解析、child compile或owner state失效同样沿Option链消失。应在root program compile时建立完整dependency DAG、报告cycle path/depth budget、生成nested instance layout，并由runtime只执行已验证artifact；动态软依赖必须有明确loading/failure状态。

### ED76-P1-16 · 双manager源码复制已经产生静态编译漂移

内置与插件manager下的graph/parameters/pose/sampling/state_machine近似复制，但已不等价：插件Graph Blend额外为每次递归分配`input_weights Vec`，内置副本已有避免该分配的实现与性能测试；插件`sample_quaternion`第二个match guard绑定`Quaternion(_)`却调用外层`&AnimationChannelValueAsset.iter()`，内置副本正确绑定`Quaternion(value)`。这是重复authority必然漂移的直接证据。应删除副本，而不是补一个同步测试长期维持两套实现。

### ED76-P1-17 · graph/state热路径仍有parameter map复制、线性cache查找和字符串clone

compiled state evaluate每次构造`Vec<Option<&AnimationParameterValue>>`，pipeline随后clone完整parameter map、active state、graph reference和transition strings；frame graph cache最多256项却按完整map线性比较。多实体、多layer、多nested machine会放大分配与比较成本。需要dense parameter buffer、generation-stamped immutable snapshot、slot-indexed state/transition output、arena/scratch reuse和可量化allocation/CPU budget。

### ED76-P1-18 · Graph compiler只验证Output可达子图，unreachable cycle/dead parameter不进入diagnostic

当前cycle DFS从Output source开始，断开的环、死节点、未使用parameter和不可达clip可被保存/编译；runtime不会执行它们，但它们污染依赖、merge、编辑器诊断与cook。compiler应区分fatal structural error与warning：全图duplicate/type/cardinality/cycle必须验证，随后产生reachable/dead sets、unused parameter和dependency pruning receipt。

### ED76-P1-19 · BlendSpace schema缺少axis、range、sample identity、triangulation与同步合同

StateKind已有1D/2D samples和runtime权重几何，但资产没有axis semantic、min/max、snap、normalization/extrapolation、sample IDs、triangulation revision、sync marker或per-sample play-rate。2D runtime返回固定最多3个sample，Editor也没有真实surface。需把BlendSpace做成独立typed asset/program节点，编译确定triangles/edge policy并可视化同一结果。

### ED76-P1-20 · 测试覆盖isolated helper，却没有跨层语义、错误与规模资格

117项聚焦test declarations能证明binary roundtrip、若干session mutation、condition/blend/transition helper和cache上限，属于可保留基础。但没有双文档locator miss、Editor validator与Runtime compiler corpus parity、linked/unlinked manager identity、compile error产品、last-good hot reload、missing dependency exit policy、save/reopen migration、10k node canvas或1k entity allocation profile。现有release benchmark还是ignored/static形态，不能构成性能资格。

## 6. P2：质量、可观测性与维护性债务

### ED76-P2-01 · pane presentation只发布字符串列表

node/state/transition/parameter被预格式化为字符串，UI无法稳定选择、定位diagnostic、增量更新或显示runtime active state。应发布immutable typed projection与revision。

### ED76-P2-02 · mutation只返回`bool changed`，没有domain receipt

调用方无法区分duplicate、invalid kind、missing pin、stale target、validation rejection或true no-op。需要`Applied/NoOp/Rejected/Stale`及affected IDs、inverse payload、diagnostics。

### ED76-P2-03 · transition duration authoring硬编码30 FPS

`duration_frames`通过固定`DEFAULT_STATE_MACHINE_TRANSITION_FPS`换算，未消费project/timeline time domain。应直接编辑duration time value或携显式display rate/tick resolution。

### ED76-P2-04 · cache limit、nested depth与condition depth是无策略的局部常量

64/128/256 cache与8/64 depth没有项目预算、platform profile、telemetry或拒绝receipt。保留防护但纳入central animation budgets。

### ED76-P2-05 · Editor compiler diagnostics是自由字符串

没有stable code、severity、asset/element/property address、dependency chain、fix-it或dedup key，无法接Error List、自动化、suppressions与文档链接。

### ED76-P2-06 · reference analysis只有direct locator集合

它没有边类型、source element、hard/soft/load phase、skeleton context、revision或cycle path；无法直接作为compile dependency stamp和cook graph。

### ED76-P2-07 · compiler/runtime没有完整stats与trace contract

缺node/edge/state/transition counts、compile phases、cache hit/miss/rebuild reason、artifact bytes、evaluation allocations、active path与transition decision trace，性能回归难以归因。

### ED76-P2-08 · benchmark与产品可用性证据不足

没有可重复的同语义graph/state workload、baseline artifact、CPU/内存阈值、debug/release差异、keyboard/focus/selection/a11y产品验收。不能以单个ignored microbenchmark宣称优于参考引擎。

## 7. 五套参考源码的可执行差距

| 参考 | 本地源码已证明的工程做法 | Zircon当前差距 | 采用边界 |
|---|---|---|---|
| Unreal | AnimBlueprint compiler验证每个visual node、处理duplicate NodeGuid并建立NodeGuid到runtime node的debug maps；StateMachine schema定义pin/connection/transition conversion并在编辑操作中使用transaction；runtime维护transition stack、pose evaluator、notify/interruption与generated-class data | Zircon无稳定visual/runtime映射、无schema-owned connection、无共享artifact/debug map，Editor与Runtime validator分裂 | 参考compiler/toolkit/runtime分层和debug currentness，不复制UObject/Kismet结构 |
| Godot | Transition Resource持有Immediate/Sync/At End、advance mode/condition/expression、xfade curve、reset、priority与break-loop；state保存position/graph offset；编辑器对move/add/remove/reconnect/rename逐项建立UndoRedo；BlendSpace有真实1D/2D编辑表面 | Zircontransition/property surface、layout、UndoRedo和BlendSpace product均未接，state/edge只剩字符串 | 参考完整可编辑合同和可逆操作，不把Godot动态property系统直接搬入Rust |
| Fyrox | Parameter是Weight/Rule/Index/SamplingPoint等typed enum，ParameterContainer维护dirty index；LogicNode持久化And/Or/Xor/Not/animation-ended；Transition使用stable pool handles；ABSM editor commands实现execute/revert/finalize并恢复pool ticket/entry | ZirconState Machine无parameter schema、条件AST不可持久化、transition无stable identity，删除/重建不可逆 | 参考typed machine model、handle与command inverse，结合Zircon generational IDs |
| Bevy | AnimationGraph基于petgraph `DiGraph/NodeIndex`，节点有weight/mask；AssetEvent Added/Modified/LoadedWithDependencies/Removed驱动ThreadedAnimationGraphs重建，预计算postorder、sorted edges与computed masks并复用容量 | Zircon以字符串递归、frame cache全map线性查找且revision检查顺序错误，没有单一threaded artifact | 参考data-oriented artifact/currentness；Bevy transition模块自述为临时API，不把它当成熟状态机基准 |
| Unity Graphics | 本地Graphics checkout不含Animator源码，但ShaderGraph有asset GUID、added/removed/pasted deltas、Undo与ValidateGraph；VFX Graph有CurrentVersion、SanitizeGraph migrations、dirty facets、compiler、per-model error manager与profiling markers | Zircon图资产缺stable identity/delta/migration/error product和compiler phase stats | 只参考通用graph authoring/compiler工程，不宣称它是Unity Animator语义参考 |

参考引擎也不是无条件上限。Bevy `add_edge`明确说明产生环时行为未定义，其state transition API也标记为可能被状态机替代；Unity Graphics仅能证明graph framework，不覆盖Animator；Godot/Fyrox的具体数据布局也不是Zircon性能答案。目标是吸收已验证的compiler、identity、transaction、artifact currentness和debug模式，再以Zircon自己的同语义benchmark证明性能。

## 8. 目标架构与唯一authority

```text
Runtime-owned schema
  AnimationNodeSchemaRegistry
  AnimationStateSchema
  AnimationParameterLayout
  AnimationConditionExpression
  stable Node/Pin/Edge/State/Transition/Layer/Parameter IDs
                |
                v
AnimationSemanticCompiler
  source snapshots + skeleton context + dependency closure
  -> AnimationDiagnosticSet
  -> CompiledAnimationProgram
  -> AnimationDependencyStamp
  -> AnimationCompileReceipt
                |
        +-------+--------+
        |                |
        v                v
Editor Preview      Game/Headless Runtime
same artifact       same AnimationRuntimeService
same evaluator      dense slots/scratch arenas
debug trace         last-good/currentness/install receipt

Editor-owned authoring
  AnimationDocumentTarget (qualified, fail-close)
  AnimationAuthoringDocument + layout metadata
  typed GraphMutationCommand / inverse payload
  transaction/history/savepoint adapter
  immutable AnimationGraphProjection
  compiler diagnostics + runtime debug projection
```

### 8.1 Runtime必须唯一拥有

1. 节点/状态/parameter/condition/layer语义schema和binary version。
2. semantic compiler、dependency closure、artifact format与diagnostic codes。
3. 唯一`AnimationRuntimeService`、artifact cache/install/currentness与evaluation scratch。
4. pose/blend/transition/event/root-motion/sync语义和determinism合同。
5. debug trace原始事实、compile/evaluation stats和预算receipt。

### 8.2 Editor必须唯一拥有

1. qualified document/view/session target、selection、viewport、layout与interaction state。
2. typed graph/state mutation command、transaction grouping、undo/redo与dirty/savepoint适配。
3. palette/search/inspector/canvas/diagnostic navigation和authoring-only metadata。
4. compile scheduling、supersession、last-good preview安装和Error List投影。
5. runtime trace到node/state/transition的visual mapping；不实现第二套evaluator。

### 8.3 必须hard cut删除

1. 删除内置与插件之间重复的`DefaultAnimationManager`/graph/state/parameters/pose/sampling源码，只保留单一owner。
2. 删除Editor插件的第二套手写validator和只返回字符串/计数的伪compile函数，operation直接调用semantic compiler service。
3. 删除`resolve locator miss -> focused view`回退；interactive focus targeting改为单独command类型。
4. 删除`from/to String`连接协议、特殊`output` identity和空字符串引用；迁移到stable typed element address。
5. 删除Graph palette、asset enum、session mutator各自维护的node kind闭集。

## 9. 重构里程碑

### ED76-M0：P0 RED证据与写入封锁

- 增加双文档/stale locator/durable replay RED E2E并先改为fail-close。
- 增加module/manager duplicate identity检测与linked/unlinked semantic parity RED。
- 为插件quaternion静态漂移建立最小compile/test证据；实现阶段再修，不在review阶段改源码。

### ED76-M1：唯一Runtime service与compiler hard cut

- 选定唯一Animation runtime owner，迁移compiled pipeline并删除两套manager副本。
- 定义`AnimationSemanticCompiler`、artifact/diagnostic/dependency/install receipts。
- Editor、preview、game、headless tests全部调用同一service。

### ED76-M2：Asset Schema vNext与stable identity

- 引入Node/Pin/Edge/State/Transition/Layer/Parameter IDs。
- 分离runtime semantic source与Editor layout metadata。
- 提供V1-V3到vNext migration、unknown node preservation与roundtrip fixtures。

### ED76-M3：Typed Graph与Condition编译

- schema registry定义pins/types/cardinality/properties/lowering。
- 全图结构验证、reachable/dead analysis、dependency closure和skeleton-qualified mask binding。
- 持久化ConditionExpression AST与typed parameter layout。

### ED76-M4：真实Graph/State Machine产品

- typed immutable projection驱动可视canvas、palette、selection、pins、edges和inspector。
- transition/property/layer/blend-space editor消费同一schema和diagnostics。
- Add Node、connect、delete、rename、move全部产生typed command receipt。

### ED76-M5：Transaction、Save与Migration

- 接Editor63 transaction/history/savepoint，drag coalesce但commit为单一domain transaction。
- save先编译source；失败保留source/dirty/last-good，不替换runtime artifact。
- close/reopen、crash recovery、merge与plugin reload保持stable identity。

### ED76-M6：Preview、Debug与Hot Reload

- 接Editor69/75 preview time authority，用同一artifact/evaluator输出pose/events/root motion。
- 安装revision-qualified artifact，处理supersession、dependency invalidation和last-good。
- 发布active state/transition/condition decision/node weights/cache status的typed debug projection。

### ED76-M7：Layer、BlendSpace、Nested与Sync

- 独立BlendSpace schema/compiler/editor，支持axis/range/snap/triangulation/sync。
- layer mask绑定stable skeleton targets，支持order/solo/mute和per-layer diagnostics。
- compile期封闭nested dependency DAG、cycle/depth/error policy。

### ED76-M8：Data-oriented性能收束

- dense parameter blocks、slot outputs、scratch arena和allocation-free steady-state evaluation。
- artifact/revision-keyed O(1) cache与incremental compile invalidation。
- 10k node authoring virtualization、1k entity runtime与deep nested/layer workloads纳入profile artifacts。

### ED76-M9：工程资格与跨引擎同语义基线

- 完成correctness、determinism、save/reopen、fault、reload、accessibility和规模矩阵。
- 为Unreal/Godot/Fyrox/Bevy可共同表达的子集建立同语义资产与硬件/构建配置。
- 只有Zircon在正确性门通过后，CPU/frame time、allocation、memory、compile latency和交互延迟均有证据更优，才允许宣称超过参考引擎。

## 10. 48个资格门

当前状态：**48/48 Fail**。已有unit test不能替代这些跨层终态证据。

### Identity、Target与Authority

- [ ] Gate 01：graph command locator exact命中唯一document/session/revision。
- [ ] Gate 02：invalid、stale、closed或reopened locator返回typed stale/rejected receipt，绝不回退focus。
- [ ] Gate 03：两份Graph与两份State Machine交错durable replay不会跨文档写入。
- [ ] Gate 04：仓库与运行注册表中只有一个Animation module/manager stable identity及一个concrete service type。
- [ ] Gate 05：Editor、preview、game与headless对同一source使用同一artifact/compiler版本。
- [ ] Gate 06：删除双manager后无兼容shim、重复解释器、重复sampling/pose/parameter源码。

### Schema、Graph与Migration

- [ ] Gate 07：Node/Pin/Edge/State/Transition/Layer/Parameter都有stable generational identity。
- [ ] Gate 08：palette、asset serializer、mutator、compiler和inspector消费同一schema registry。
- [ ] Gate 09：connect/disconnect明确pin、edge、type、cardinality、order与inverse payload。
- [ ] Gate 10：duplicate ID、missing pin、type mismatch、cardinality与dangling reference有stable diagnostics。
- [ ] Gate 11：全图cycle、output/reachability和dependency cycle在compile期检测。
- [ ] Gate 12：dead node、unused parameter和pruned dependency进入warning/stats receipt。
- [ ] Gate 13：V1-V3 migration保留语义/layout，未知新node可无损roundtrip或明确拒绝。

### Parameter、Condition与Transition

- [ ] Gate 14：State Machine拥有typed parameter layout/default/source policy，无literal inference authority。
- [ ] Gate 15：All/Any/Not/leaf AST可编辑、持久化、编译并保持trigger consumption确定性。
- [ ] Gate 16：transition有stable ID、priority/tie-break和multi-edge合同。
- [ ] Gate 17：duration、exit time、interruption、reset、sync/marker与blend profile均可编辑并验证。
- [ ] Gate 18：Clip/GraphRef/BlendSpace/SubMachine state共享明确playback/time/event/root-motion合同。

### BlendSpace、Layer与Nested Machine

- [ ] Gate 19：BlendSpace 1D/2D有axis/range/snap/sample ID/triangulation/extrapolation与真实Editor surface。
- [ ] Gate 20：layer mask绑定skeleton/target revision，长度或骨名漂移在compile/install期失败。
- [ ] Gate 21：layer order/weight/mode/solo/mute和per-layer diagnostic可author与preview。
- [ ] Gate 22：nested dependency DAG、cycle path和depth budget在root compile receipt中可见。

### Transaction、Persistence与Product

- [ ] Gate 23：add/remove/connect/move/rename/property edit均进入Editor63 transaction/history。
- [ ] Gate 24：undo/redo恢复stable IDs、edges、entry、transition order、layout与selection。
- [ ] Gate 25：save必须取得current successful compile receipt；failure不清dirty也不替换runtime artifact。
- [ ] Gate 26：crash recovery与save/reopen保持source、layout、diagnostics和last-good关联。
- [ ] Gate 27：pane payload为typed immutable projection，不再用字符串列表作为authoritative model。
- [ ] Gate 28：真实产品canvas显示nodes/pins/edges/selection/diagnostics，Add Node不再no-op。
- [ ] Gate 29：palette search/context menu/inspector只暴露当前schema与capability允许的功能。

### Runtime、Preview与Currentness

- [ ] Gate 30：Editor preview与game runtime对pose/events/root motion/transition选择逐帧parity。
- [ ] Gate 31：compile/load/evaluation failures进入统一typed diagnostics与Error List。
- [ ] Gate 32：hot reload失败保持last-good并明确显示stale/error状态。
- [ ] Gate 33：artifact install带source/dependency/skeleton revision并拒绝out-of-order completion。
- [ ] Gate 34：frame evaluation cache key包含artifact revision，mid-frame replacement不返回旧结果。
- [ ] Gate 35：clip/graph/machine/layer/skeleton依赖变化精确invalidate相关artifact/cache。
- [ ] Gate 36：missing duration/dependency不会被解释为normalized 1.0或自动通过exit time。

### Performance、Determinism与Events

- [ ] Gate 37：steady-state graph evaluation每实体零heap allocation或满足明确approved budget。
- [ ] Gate 38：state/layer/nested evaluation复用dense parameter/scratch storage，无full map/string clone。
- [ ] Gate 39：artifact/frame cache lookup为bounded O(1)/O(log n)，不线性比较完整parameter maps。
- [ ] Gate 40：10k nodes/20k edges canvas使用viewport culling、LOD、incremental projection且交互达预算。
- [ ] Gate 41：1k animated entities、多layer/nested/blend workload有CPU、allocation、memory profile artifact。
- [ ] Gate 42：相同source/artifact/parameters在多线程、reload和save/reopen后产生确定性state/pose结果。
- [ ] Gate 43：trigger、notify、loop crossing、interruption和transition event firing policy有完整matrix。

### Lifecycle、Accessibility与跨引擎资格

- [ ] Gate 44：keyboard navigation、focus、selection、screen-reader names与diagnostic navigation通过产品验收。
- [ ] Gate 45：migration、copy/paste、rename、merge、save/reopen保持stable identity与reference integrity。
- [ ] Gate 46：plugin enable/disable/reload不会切换semantic authority，缺capability时明确降级且不丢source。
- [ ] Gate 47：compile crash/cancel/OOM/dependency loss具有bounded retry、recovery receipt和无数据丢失证明。
- [ ] Gate 48：同语义跨引擎correctness先通过，随后Zircon在固定硬件/构建下的runtime与Editor指标有可复验证据优于目标基线。

## 11. 实现顺序与停止条件

1. 先做ED76-M0，关闭错误文档写入和同名service双authority；未通过Gate 01-06前禁止新增节点类型。
2. 再完成M1-M3，建立唯一compiler、schema vNext、stable identity与dependency artifact；未通过Gate 07-18前禁止把现有空Canvas包装成“Graph Editor完成”。
3. M4-M6接真实产品、transaction/save和runtime-backed preview；不得让Editor另写解释器来追赶显示效果。
4. M7补齐BlendSpace/layer/nested/sync，所有功能以compile/install/debug闭环交付。
5. M8-M9最后做规模、fault和同语义跨引擎资格；任何性能宣称必须附artifact、硬件、build profile和正确性结果。

实现阶段若触及`zircon_runtime/src/animation/manager`、`zircon_plugins/animation/runtime/src/manager`或public module contract，必须应用hard-cutover规则并同步公共架构文档；不允许留下`pub use`兼容层来延长双authority。

## 12. 本轮验证边界

- 已逐文件扫描冻结语料，检查asset/editor/event/product/compiler/runtime/cache/manager/tests与五套本地参考源码。
- 已静态确认locator fail-open、同名双manager、三套compiler/evaluator语义、palette/asset/mutator错位、silent `.ok()?`、frame cache revision顺序、normalized-time fallback及quaternion副本漂移。
- 已与Editor14/63/69/75去重，父报告继续拥有通用toolkit可达性、transaction、preview cadence和timeline能力。
- 本轮未修改生产源码，未运行Cargo或任何动态/GUI/GPU验证；所有资格门保持Fail。
- `review complete`只表示当前源码差距已形成可执行重构账，不表示Animation Graph/State Machine功能完成。
