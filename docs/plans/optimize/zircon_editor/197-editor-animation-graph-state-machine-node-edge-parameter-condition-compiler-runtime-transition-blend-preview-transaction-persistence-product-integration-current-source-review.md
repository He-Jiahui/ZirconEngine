---
title: Editor Animation Graph、State Machine、Node-Edge、Parameter、Condition、Compiler、Runtime Transition、Blend、Preview、Transaction、Persistence 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor197
review_date: 2026-08-28
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
related_code:
  - zircon_editor/src/core/editing/animation_document
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/binding/animation
  - zircon_editor/src/ui/binding_dispatch/animation
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/animation_event.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/animation_graph.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/assets/ui/editor/host/animation_graph_body.zui
  - zircon_plugins/animation_graph/editor
  - zircon_plugins/animation/runtime/src
  - zircon_runtime/src/animation
  - zircon_runtime/src/core/framework/animation
tests:
  - zircon_editor/src/core/editing/animation_document/tests.rs
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/tests/editor_event/animation_runtime
  - zircon_editor/src/tests/editor_event/locator_protocol_hard_cut.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/binding_dispatch/animation.rs
  - zircon_editor/src/tests/host/retained_animation_template_body.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents/asset_contracts.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/ui/animation_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/binding/animation.rs
  - zircon_plugins/animation_graph/editor/src/tests.rs
  - zircon_plugins/animation/runtime/tests
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - zircon_runtime/src/core/framework/animation/compiler/tests.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/tests.rs
  - zircon_runtime/src/core/framework/tests/framework_surfaces.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/190-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/196-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
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
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Animation Graph、State Machine、Node-Edge、Parameter、Condition、Compiler、Runtime Transition、Blend、Preview、Transaction、Persistence 与 Product Integration 当前源码复核

## 1. 结论

Editor76之后，Animation Graph/State Machine不再是“完全没有共享compiler与document transaction”的状态。`zircon_runtime::core::framework::animation::compiler`现在拥有统一`AnimationCompileSource/AnimationCompileProduct`入口、Graph/State Machine typed dense IR、稳定诊断码、全图非递归拓扑检查、parameter kind冲突检查、BlendSpace几何准入与layer值验证；插件production runtime的compiled graph/state-machine compiler已经消费这些共享IR。Editor侧新增`AnimationAuthoringDocumentStore`、monotonic revision、whole-source CAS command、`HistoryContextId::Document`、Undo/Redo以及current/last-known-good compilation。旧报告关于“固定Add Node按钮”“save完全不产生共享semantic compilation”“Graph compiler只检查Output可达环”的描述已经失效。

但这些新增底座仍没有形成工程级产品闭环。Graph body现在只剩header和空`animation_graph_canvas_slot`；pane payload仍是`Vec<String>`；没有production canvas/controller、node position、selection、pin/edge、inspector、compiler diagnostics、preview subject或runtime debug projection。capability表明确只允许Output/Blend，Clip/Additive/Mask仍拒绝；插件palette却继续声明另一套节点集合，并把BlendSpace节点放进Graph palette。Editor插件自身还保留一套返回Output字符串/entry与计数的validator/compiler，与共享compiler并存。

两项P0均未关闭。携locator的Graph/State命令在locator解析或查找失败后仍回退执行时focused view，因此仍可写错同kind文档；内置Runtime与Animation Runtime插件仍用相同`animation.runtime`模块名和相同manager stable name注册两个不同Rust类型。源码测试甚至继续断言两种`DefaultAnimationManager`的`TypeId`不同。共享compiler减少了编译语义漂移，但没有消除服务身份与执行authority冲突。

Runtime production pipeline已有真实dense compiled graph/state machine、revision-bounded cache、parameter revision/fingerprint、nested/layer/blend-space执行和较多contract test，这是必须保留的核心资产。不过graph/state/load compile failure仍大量经`.ok()?`变成无输出；frame graph evaluation cache在读取asset/skeleton revision前按`asset id + parameter fingerprint`命中；缺失duration仍被解释为normalized `1.0`；direct Clip state仍固定non-looping；Editor document的LKG从未被pane、preview、save或runtime artifact installer消费。当前“LKG存在”只是内存数据结构，不是热更/预览恢复合同。

本轮刷新Editor76的2项P0为 **2 Open**；20项P1为 **10 Open / 10 Partial / 0 Closed**；8项P2为 **4 Open / 4 Partial / 0 Closed**；48门为 **29 Fail / 19 Partial / 0 Pass**。Partial只表示可复用底座已出现，不表示产品或运行资格通过。

本轮只做静态review，不修改production Rust/ZUI，不运行Cargo、Editor、GUI/GPU、native input、save/reopen、plugin reload、fault/soak/profile或同语义跨引擎benchmark。Tooling按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。

## 2. 审查边界与冻结语料

### 2.1 Current working tree

主仓HEAD为`681588f7a1cbfaae3147e8b93e1be6705d810f21`。报告读取2026-08-28 current working tree；animation document/compiler/runtime pipeline含大量共享工作树在途修改与未跟踪文件。本报告以当前磁盘源码为事实，不回退、不归属也不提交这些production变更。

MVP baseline recovery仍为`in_progress`。静态源码中的capability、unit test或注释不能代替动态产品资格；实现前必须重算fingerprint并复核Editor14/63/69/76/184/190/196及Runtime animation owner的最新终态。

### 2.2 冻结物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Editor authoring/product | **50 / 8,188 / 7,680 / 295,119 / 26** | document/session、event/binding、locator、transaction、save、pane/slot与capability | `e3b9cf28289a460109451c32e2788a2faa5bcbc77d115a316cf072d46d3eacfd` |
| Schema/compiler/editor plugin | **35 / 7,712 / 7,028 / 258,418 / 34** | asset/binary/reference、shared compiler/schema及Editor插件第二compiler/palette | `d48e67cf289db70432467da138b76c3f315eae34948bb77222e8c68c0cd8a9c8` |
| Runtime compiler/evaluator | **176 / 18,217 / 16,633 / 624,900 / 78** | 双manager、compiled graph/state/layer/blend/nested、cache、pose/event与diagnostic | `1626b517efec48227991a49b74024bfb11903a302c255432e210225341910f99` |
| Focused tests | **41 / 10,226 / 9,510 / 373,877 / 171** | document LKG/history、event route、product slot、shared/runtime compiler与runtime integration | `8921ae3f43576f1c7f5d103b7d61cf634a815d2234c25e2c5fb7af4ef78edc19` |
| Zircon去重合计 | **301 / 44,129 / 40,658 / 1,544,831 / 307** | 上述四组按normalized path去重 | `f3cddd3ab11a77a6fdd54cb99c2338d85f2d5b575946202d67122a80e85901a0` |
| Unreal selected set | **7 / 6,576 / 5,581 / 261,709 / 0** | compiler/schema/validation/debug map与runtime transition state | `709865614bd17ec62edee6fa39794325791cb85e036f67561550b30b23eeb0e6` |
| Godot selected set | **6 / 8,665 / 7,226 / 337,039 / 0** | state machine/blend tree/blend space runtime资源与真实Editor | `a1579e8950c88b4b3cb7ad23bbc105bee4464dfa8299f4ec079c1d2ba3690231` |
| Fyrox selected set | **29 / 11,484 / 10,412 / 427,278 / 11** | ABSM typed machine/parameter/logic/layer/node与command/editor surface | `aa63a08d1462296ab050ef4d93d00a648717daa007d3268423e1633f35d1e2c9` |
| Bevy selected set | **3 / 2,979 / 2,692 / 113,433 / 9** | petgraph asset、threaded graph、player与transition边界 | `f8835c768eb054f0c2d1d8fbebd4c80e93c23a365f579e02cb636246bec9ee1a` |
| Unity Graphics selected set | **5 / 6,777 / 5,754 / 278,044 / 0** | graph GUID/delta/Undo/validation、migration/compiler/error architecture | `2c0a644e17a12a7a7b96fdbb97c024df1b96a8f3f4f5e277f1f3afa3e3304049` |
| 五引擎参考合计 | **50 / 36,481 / 31,665 / 1,417,503 / 20** | 五组显式路径去重 | `df8c4ae641ee464064336d930ecf21a7477139bbb09acdbde8683dc39461fd23` |

fingerprint按小写规范化相对路径排序，将每个`path + NUL + lowercase file SHA-256 + LF`聚合后再取SHA-256。它是本次review输入receipt，不是compiler artifact key或asset revision。

### 2.3 Owner与去重边界

Editor197只刷新Editor76拥有的Graph/State Machine target、schema、compiler/evaluator authority、product、runtime currentness与专项资格门。Editor14继续拥有动画authoring/compiler/preview总账；Editor63/184拥有通用transaction/history/savepoint；Editor69/190拥有preview world/time；Editor196拥有whole-asset clone、锁内同步全量compile与全局Animation event effect成本。这里引用这些问题，不重复新增owner计数。

## 3. 当前实现拓扑

```text
Asset Browser / OpenAsset
  -> builtin Graph/State Machine toolkit route
  -> AnimationAuthoringDocumentStore(DocumentId, AssetUri, kind, revision)
  -> current AnimationCompileProduct + last-known-good product
  -> AnimationEditorSession(read handle)

Durable Graph/State event
  -> locator exact lookup if currently open
  -> locator miss/parse failure: execution-time focused view fallback  [P0]
  -> whole source clone + string mutation
  -> AnimationEditCommand whole-source CAS swap
  -> HistoryContextId::Document
  -> synchronous shared source compile under document write lock
  -> metadata dirty + string pane projection

Shared compiler
  -> typed Graph/State source diagnostics and dense index IR
  -> plugin runtime adapter resolves skeleton masks/runtime layouts
  -> revision-bounded compiled caches
  x no dependency/install receipt or Editor diagnostic projection

Editor plugin compiler
  -> independent free-string validation
  -> graph returns output source String
  -> state machine returns entry/state/transition counts
  x not the shared compiler product

Runtime authority A
  -> zircon_runtime::animation::DefaultAnimationManager
  -> permissive source evaluator

Runtime authority B
  -> zircon_plugins::animation::runtime::DefaultAnimationManager
  -> plugin AnimationEvaluationPipeline + compiled evaluator
  -> same module/manager stable names as A

Product
  -> AnimationGraphPanePayload(Vec<String>)
  -> header + empty animation_graph_canvas_slot
  x no canvas/controller/preview/diagnostic/debug consumer
```

### 3.1 可保留底座

1. 保留Runtime-owned统一`AnimationCompileSource/AnimationCompileProduct`和stable diagnostic code/element/severity模型。
2. 保留Graph的非递归Kahn拓扑、dense node/parameter slots、unreachable warning和deep graph test。
3. 保留State Machine的dense state/transition IR、parameter kind merge、BlendSpace准入、layer值验证和trigger consumption。
4. 保留plugin runtime由共享IR lowering到skeleton-bound runtime artifact的分层，但必须变成唯一runtime owner。
5. 保留revision-bounded compiled caches、`AnimationParameterSet`的Arc snapshot/revision/fingerprint和state instance parameter projection cache。
6. 保留Editor authoritative document、monotonic revision、CAS command、document-scoped history和current/LKG compilation。
7. 保留Runtime schema registry的owner/version、node kind和typed pin descriptor方向，但扩展为真正动态且被全链消费的schema。
8. 保留binary V1/V2/V3 upgrade入口与direct reference collector意图，升级为stable identity和typed dependency closure。

## 4. 对Editor76旧结论的源码校正

| Editor76旧结论 | 当前源码 | 本轮裁决 |
|---|---|---|
| 默认Animation toolkit不可达 | builtin registry与route-kind测试已覆盖Graph/State Machine open route | 入口问题关闭，由Editor196/当前产品测试承接 |
| Graph body有固定Add Node并发送`State`形成确定性no-op | 当前ZUI已删除按钮，只保留header和空Canvas slot | 旧按钮问题关闭；真实canvas仍Fail |
| mutation只blanket changed/dirty，无transaction | 持久mutation通过Document history、CAS command和Undo/Redo | 通用transaction改为Partial；domain receipt/stable identity仍缺 |
| save不调用共享semantic compiler | 每次document swap会同步调用共享compiler并保存current/LKG | 编译底座Partial；save仍不要求successful/current receipt |
| Graph compiler只验证Output可达环 | 新共享compiler检查全图cycle并警告unreachable node | 该部分已修；unused parameter/dependency pruning仍缺 |
| Runtime compiler与Editor compiler完全各自为政 | plugin compiled runtime已消费共享Graph/State IR | runtime source validation实质收敛；Editor插件第二compiler仍存在 |
| frame cache按full parameter map线性搜索 | 已改为BTreeMap + content fingerprint，并保留碰撞时值相等校验 | 查找底座改善；artifact revision仍不在frame key中 |
| state evaluate每次都重建parameter引用Vec | instance cache按layout pointer + parameter revision缓存dense cloned values | 局部改善；输出字符串与graph evaluation scratch仍分配 |

## 5. P0：当前可达的数据与运行authority风险

### ED76-P0-01 · Open · locator miss仍回退focused view，可写错同kind文档

`resolve_animation_document_instance`仅在`find_animation_editor_instance`命中时按locator返回；URI parse失败、文档关闭、尚未打开、路径不一致或stale replay未命中时，代码直接读取`session.focused_view`。新增`require_animation_document_kind`能阻止Graph命令写入State Machine，修复了cross-kind混淆，但不能阻止Graph A命令写入focused Graph B，也不能阻止State Machine A命令写入focused State Machine B。

命令目标还只有locator字符串，没有document session/generation/revision/view epoch。外层`authoring_trace`在提交后读取focused animation history，因此locator命中非focused文档时还可能把另一个文档的transaction id写进事件记录。必须引入qualified immutable target和直接返回transaction receipt；携locator命令exact miss必须fail-close，interactive focus命令另立事件类型并在入队时冻结target。

### ED76-P0-02 · Open · 同名Animation module/manager仍注册两种类型与两套语义

`zircon_runtime/src/animation/module.rs`和`zircon_plugins/animation/runtime/src/module.rs`继续同时声明`ANIMATION_MODULE_NAME = "animation.runtime"`以及`animation.runtime.Manager.DefaultAnimationManager`。二者各有`DefaultAnimationManager`、graph/state/pose/sampling源码，factory构造方式和能力不同；测试明确断言两种manager `TypeId`不相等。plugin production pipeline又不通过public manager的source evaluator，而是直接使用compiled evaluator。

共享source compiler没有解决stable service identity冲突。必须选定唯一owner crate，将compiled pipeline、manager trait实现和module descriptor合并；注册表必须拒绝重复stable identity。旧manager与复制实现hard cut删除，不能用re-export shim长期维持两种具体类型。

## 6. P1刷新矩阵

| Finding | 状态 | 当前源码证据 | 必须重构 |
|---|---|---|---|
| ED76-P1-01 schema三方漂移 | **Partial** | Runtime已有node/pin schema，Editor command resolver也读取node kind；但插件palette仍独立声明Clip/Blend/BlendSpace/Output，漏Additive/Mask，mutator只允许Output/Blend | serializer、palette、capability、mutator、compiler lowering与inspector只消费同一versioned descriptor snapshot |
| ED76-P1-02 connection只有from/to字符串 | **Open** | event/mutation仍无PinId/EdgeId；Additive只能写base，Blend append字符串，Output用特殊`output` | stable Node/Pin/Edge ID、type/direction/cardinality/order、connect plan和inverse receipt |
| ED76-P1-03 delete写空引用 | **Partial** | remove仍把Additive/Mask/Output必填引用清空并允许保存；共享diagnostic与LKG可识别/保留上一个成功产品，但产品未消费 | transactional DeletePlan、orphan/rewire policy、invalid-source UI与runtime LKG install闭环 |
| ED76-P1-04 可变字符串/Vec位置充当身份 | **Open** | Output无id，state/transition/layer/sample无stable ID，layout也未持久化；未知node tag直接拒绝 | generational IDs、separate layout metadata、unknown payload preservation和vNext migration |
| ED76-P1-05 State Machine无typed parameter source schema | **Partial** | shared compiler会推导dense parameter layout并拒绝kind冲突，runtime parameter set有revision；source仍只在conditions/BlendSpace中写名字且无default/source policy | 持久ParameterId/type/default/mutability/source/replication schema，runtime只接layout-compatible block |
| ED76-P1-06 All/Any/Not AST无法持久化 | **Open** | runtime condition module仍有AST/depth guard，asset仍是flat `Vec<Condition>`且shared compiler按AND lowering | 同一typed expression AST进入asset/editor/compiler/runtime，定义stable expression IDs和trigger消费语义 |
| ED76-P1-07 transition按Vec顺序选首项 | **Open** | asset/compiled IR仍无TransitionId/priority，Editor以from/to唯一定位并覆盖duration | stable transition identity、priority/tie-break/multi-edge和ambiguity diagnostic |
| ED76-P1-08 Editor validator与Runtime compiler分裂 | **Partial** | plugin production runtime已消费shared compiler；但Editor插件仍维护自由字符串validator和伪compile report | 删除Editor第二compiler，所有validate/compile操作调用唯一service并投影同一diagnostic set |
| ED76-P1-09 compile/load failure静默无输出 | **Partial** | Editor document有typed current/LKG；clip/layer有局部diagnostic；graph/state caches仍以`.ok()?`吞错，LKG未安装 | typed compile/install/evaluation failure product、dependency chain、LKG policy和Error List/runtime event |
| ED76-P1-10 frame graph cache可返回旧revision | **Open** | cache先按graph/skeleton id和parameter fingerprint命中，之后才读取resource snapshot/revision | key改为immutable installed artifact identity/revision，replacement主动invalidate frame evaluation |
| ED76-P1-11 缺duration被解释为normalized 1.0 | **Open** | `normalized_graph_time`和direct Clip duration miss仍返回`1.0`，可错误满足exit time | `Available/Stale/Unavailable/Error`时间状态和显式hold/fail/LKG transition policy |
| ED76-P1-12 direct Clip state固定non-looping | **Open** | state kind仍只有clip reference，pose/event sampling均写`looping: false` | 共享ClipPlayerSpec、play rate/loop/start/sync/marker/root motion/event policy |
| ED76-P1-13 N路Blend共用单scalar | **Open** | compiled evaluator仍对首项用`1-scalar`、其余平均分`scalar/(N-1)` | 分离2-way/multi/by-enum/list等schema，编译per-edge weight program与sync/blend profile |
| ED76-P1-14 Layer mask脱离skeleton identity | **Partial** | shared compiler验证weight范围，plugin runtime能编译dense mask并产生局部layer mismatch diagnostic；source仍是position `Vec<Real>`且无skeleton revision | skeleton-qualified mask asset、stable target IDs、binding artifact/dependency stamp及Editor author/preview |
| ED76-P1-15 nested DAG未在compile期封闭 | **Open** | runtime `MachineInstanceKey`仍以depth 8和lineage防环，失败返回None；root compile不解析完整dependency closure | root program dependency DAG、cycle path/depth receipt、nested layout及明确loading/failure state |
| ED76-P1-16 双manager源码复制漂移 | **Open** | 两套manager/module仍存在；plugin compiled pipeline与public source manager又是第三执行入口 | 作为P0-02子项hard cut重复graph/state/pose/sampling和legacy source evaluator |
| ED76-P1-17 graph/state热路径分配与clone | **Partial** | parameter Arc snapshot/revision/fingerprint、state instance dense projection cache和BTreeMap bounded cache是进展；graph evaluate仍分配weights/contributions/clips，state输出clone字符串 | persistent scratch arena、slot output、stable string IDs、allocation budget和profile receipt |
| ED76-P1-18 unreachable cycle/dead analysis缺失 | **Partial** | shared Graph compiler已全图检查cycle、非递归处理deep graph并为unreachable node发warning；unused parameter与dependency pruning/stats仍缺 | 保留全图validator，补unused parameter、dead dependency、pruned set和cook receipt |
| ED76-P1-19 BlendSpace schema不完整 | **Partial** | compiler已验证finite/unique/non-collinear，runtime有1D/2D sampling与triangle hint；仍无axis/range/snap/sample ID/triangulation revision/sync/editor | 独立BlendSpace asset/program/editor和deterministic triangulation/extrapolation receipt |
| ED76-P1-20 缺跨层与规模资格 | **Partial** | 新增shared compiler、deep graph、dense graph/state、blend-space、cache bound、borrowed parameter和pose allocation tests；仍缺locator P0、双module、save/LKG/product/preview和跨引擎E2E | 建立source-to-editor-to-installed-runtime corpus、fault/reload/save及规模profile矩阵 |

P1合计：**10 Open / 10 Partial / 0 Closed**。Partial项不得从实施计划删除；它们只是已有底座的精确保留边界。

## 7. P2刷新矩阵

| Finding | 状态 | 当前源码证据与重构要求 |
|---|---|---|
| ED76-P2-01 pane只发布字符串列表 | **Open** | `AnimationGraphPanePayload`仍只有mode/path/status/selection及四个`Vec<String>`；改为revisioned typed immutable projection |
| ED76-P2-02 mutation只有bool | **Partial** | target/capability/compiler已有局部typed diagnostic，transaction内部有CAS error；公开event仍只返回changed bool，需domain `Applied/NoOp/Rejected/Stale` receipt及affected IDs |
| ED76-P2-03 transition authoring硬编码30 FPS | **Open** | mutation仍用`TRANSITION_FRAMES_PER_SECOND = 30.0`；改为显式time value/display rate/tick resolution |
| ED76-P2-04 budgets仍是局部常量 | **Open** | 64/128/256/4096 cache与8/64 depth虽有边界测试，仍无platform/project policy、telemetry和admission receipt |
| ED76-P2-05 Editor compiler自由字符串diagnostic | **Partial** | shared compiler已有code/severity/element；Editor插件和pane仍不消费，需删除自由字符串authority并接Error List |
| ED76-P2-06 reference analysis只有direct locator | **Open** | collector仍没有edge type/source element/load phase/skeleton/revision/cycle path，不能充当artifact dependency stamp |
| ED76-P2-07 stats/trace不完整 | **Partial** | clip evaluator/projection已有stats，pipeline暴露cache len且layer/clip有diagnostic；缺compile phase、rebuild reason、artifact bytes、active path和condition decision trace |
| ED76-P2-08 benchmark/产品证据不足 | **Partial** | 有borrowed parameter与pose buffer allocation局部证据，但无真实canvas、1k entity、多layer/nested、固定硬件跨引擎完整qualification |

P2合计：**4 Open / 4 Partial / 0 Closed**。

## 8. 五套参考源码的可执行差距

| 参考 | 本地源码已证明的工程做法 | Zircon当前差距 | 采用边界 |
|---|---|---|---|
| Unreal | compiler验证visual node/skeleton，修复duplicate NodeGuid，建立NodeGuid到runtime index的debug maps并记录compile phase stats；State Machine schema负责pin/connection/transition conversion，runtime有transition stack/interruption/notify | Zircon shared IR无stable visual identity/debug map/dependency install receipt，Editor无schema-driven canvas，双runtime service仍在 | 采用compiler/schema/debug/runtime分层，不复制UObject/Kismet模型 |
| Godot | Transition Resource包含advance mode/condition/expression/xfade/reset/priority；state与graph保存position/offset；Editor对add/remove/move/connect/rename使用UndoRedo，BlendSpace有真实1D/2D产品 | Zircon transition/state/layout仍是字符串和固定字段，BlendSpace无Editor surface | 采用完整可编辑合同、可逆操作和layout持久化，不照搬动态property系统 |
| Fyrox | machine parameter与LogicNode是typed enum，state/transition使用pool handles；ABSM command具有execute/revert/finalize并恢复ticket/entry，canvas/socket/selection分层 | Zircon无stable handles、condition AST source和domain inverse command，whole-source swap无法支撑大图增量编辑 | 采用typed model、handle和command inverse，结合Zircon generational IDs |
| Bevy | AnimationGraph基于`DiGraph/NodeIndex`，ThreadedAnimationGraphs按AssetEvent currentness重建并预计算postorder/sorted edges/masks，复用容量 | Zircon runtime虽有dense IR/cache，但frame cache仍缺artifact revision，Editor source仍以字符串边表达 | 采用data-oriented immutable artifact和asset-event currentness；不把其临时transition API当成熟状态机基准 |
| Unity Graphics | 本地Graphics不含Animator源码，但ShaderGraph有object GUID、added/removed edge/node delta、Undo和ValidateGraph；VFX有version/sanitize migration、compiler、per-model error manager与profiling | Zircon图source缺stable identity/delta/migration/error projection/compiler phase stats | 仅采用通用graph engineering，不宣称其覆盖Unity Animator语义 |

这些参考也不是无条件性能上限。Zircon只有在同语义correctness、determinism、reload与fault门先通过后，才能用固定硬件/构建的CPU、allocation、memory、compile latency和Editor interaction数据证明更优。

## 9. 目标authority与数据流

```text
Runtime-owned versioned schema
  Node/Pin/Edge/State/Transition/Layer/Parameter IDs
  property + cardinality + migration + dependency rules
                  |
                  v
AnimationSemanticCompiler (single owner)
  source snapshot + skeleton/dependency closure + generation
  -> DiagnosticSet
  -> CompiledAnimationProgram
  -> DependencyStamp
  -> Compile/InstallReceipt
                  |
          +-------+--------+
          |                |
          v                v
Editor Preview       Game/Headless Runtime
same installed       same installed artifact
artifact/evaluator   same AnimationRuntimeService
LKG + debug map      deterministic dense execution

Editor-owned authoring
  qualified document target
  typed delta commands + inverse
  transaction/history/savepoint adapter
  immutable paged graph projection
  canvas/palette/inspector/diagnostic/debug projection
```

Runtime唯一拥有schema语义、compiler、artifact/dependency/install currentness、唯一service、pose/blend/transition/event/root-motion/sync语义及原始debug/stats。Editor唯一拥有document/view/session target、layout/selection/interaction、typed authoring command、compile scheduling、LKG preview选择和可视投影。Editor不得保留第二validator/evaluator，Runtime不得保留同stable name的第二manager。

## 10. 重构里程碑

### ED76-M0：P0 RED与写入封锁

- 增加Graph A/B与State A/B的invalid/stale/closed/reopened locator交错replay测试，先把locator miss改成fail-close。
- command执行直接返回document/transaction/revision receipt，trace不得再读取focused history猜测transaction。
- 增加重复module/manager stable identity注册失败和linked/unlinked semantic parity RED。

### ED76-M1：唯一Runtime service与compiler hard cut

- 选定唯一Animation runtime owner，迁移compiled pipeline并删除内置/插件重复manager/module/source evaluator。
- 删除Animation Graph Editor插件的第二validator和字符串/计数伪compile。
- Editor、preview、game、headless只调用同一compiler/service版本。

### ED76-M2：Schema vNext与stable identity

- 引入Node/Pin/Edge/State/Transition/Layer/Parameter/Sample generational IDs。
- runtime semantic source与Editor layout metadata分离但以stable ID关联。
- 完成V1-V3 migration、unknown node preservation、rename/copy/paste/merge roundtrip fixtures。

### ED76-M3：Typed delta authoring与真实产品

- 以schema snapshot驱动palette/search/context menu/inspector和pin/cardinality验证。
- add/remove/connect/disconnect/move/rename/property/condition/layer/sample全部改为delta command + inverse receipt。
- 构建paged immutable projection、selection/hit-test/controller并真正挂载Canvas slot。

### ED76-M4：Compile、Save、Install与Diagnostics

- compile在锁外按source revision/generation执行，支持supersession与dependency stamp。
- save明确区分“允许保存invalid recoverable source”和“允许替换runtime artifact”，返回typed receipt。
- current/LKG/diagnostics进入pane、Error List、autosave/recovery和artifact installer。

### ED76-M5：Preview、Debug与Hot Reload

- 接Editor69/190 time/world authority，preview和game使用同一installed artifact/evaluator。
- 投影active state/transition/condition decision/node weight/clip event/root motion/cache currentness。
- dependency change、compile failure、out-of-order completion和plugin reload有明确stale/LKG/recovery状态。

### ED76-M6：高级状态语义与性能收束

- 持久typed parameter schema和All/Any/Not AST；完成transition priority/sync/marker/interruption/reset/profile。
- 独立BlendSpace与skeleton-qualified layer mask；root compile封闭nested dependency DAG。
- dense parameter blocks、persistent scratch、slot outputs、incremental compile和viewport-culling graph projection。

### ED76-M7：工程资格与跨引擎基线

- 完成correctness/determinism/save-reopen/reload/fault/accessibility和规模矩阵。
- 固定10k node/20k edge Editor、1k entity、多layer/nested/blend runtime workload。
- correctness先通过，再提交固定硬件/build的CPU、allocation、memory、compile latency和interaction对比artifact。

## 11. 48个资格门

当前状态：**29 Fail / 19 Partial / 0 Pass**。Partial只确认局部源码底座，不代表终态资格。

### Identity、Target与Authority

- [Partial] Gate 01：exact locator可命中当前open实例并检查document kind，但没有session/revision target。
- [Fail] Gate 02：invalid/stale/closed/reopened locator仍会回退focus。
- [Fail] Gate 03：两份同kind文档交错durable replay无防串写证明。
- [Fail] Gate 04：仓库仍有两个同stable identity Animation module/manager与不同concrete type。
- [Partial] Gate 05：Editor document和plugin compiled runtime使用shared source compiler；Editor插件、preview和内置manager未统一。
- [Fail] Gate 06：重复manager/source evaluator/sampling/pose/parameter实现仍在。

### Schema、Graph与Migration

- [Fail] Gate 07：Node/Pin/Edge/State/Transition/Layer/Parameter仍无stable generational identity。
- [Partial] Gate 08：compiler与Editor node-kind resolver读取Runtime schema；palette/serializer/mutator/inspector未统一。
- [Fail] Gate 09：connect/disconnect仍无pin/edge/type/cardinality/order/inverse协议。
- [Partial] Gate 10：shared compiler有duplicate/missing/cycle stable diagnostics；无pin/cardinality/property完整定位。
- [Partial] Gate 11：Graph全图cycle/output验证已建立；跨asset dependency cycle未封闭。
- [Partial] Gate 12：unreachable node有warning；unused parameter/pruned dependency/stats receipt缺失。
- [Fail] Gate 13：vNext stable-ID migration和unknown-node preservation缺失。

### Parameter、Condition与Transition

- [Partial] Gate 14：compiler可推导typed dense parameter layout；source没有显式schema/default/source policy。
- [Fail] Gate 15：All/Any/Not AST不能author/save/compile同源往返。
- [Fail] Gate 16：transition无stable ID、priority/tie-break与multi-edge合同。
- [Fail] Gate 17：duration/exit/interruption/reset/sync/marker/blend profile未形成完整authoring surface。
- [Fail] Gate 18：Clip/GraphRef/BlendSpace/SubMachine没有统一playback/time/event/root-motion合同。

### BlendSpace、Layer与Nested Machine

- [Partial] Gate 19：BlendSpace compiler/runtime几何底座存在；axis/range/sample ID/editor/sync缺失。
- [Fail] Gate 20：layer mask未绑定skeleton/target revision。
- [Fail] Gate 21：layer order/weight/mode/solo/mute与per-layer Editor preview缺失。
- [Fail] Gate 22：nested dependency DAG/cycle/depth未进入root compile receipt。

### Transaction、Persistence与Product

- [Partial] Gate 23：现有持久mutation进入Document history；move/rename/layout/typed property命令缺失。
- [Partial] Gate 24：whole-source Undo/Redo可恢复现有source；stable IDs/layout/selection不存在。
- [Fail] Gate 25：save不要求current successful compile/install receipt。
- [Partial] Gate 26：autosave source与in-memory LKG存在；crash/reopen后的diagnostic/LKG关联未建立。
- [Fail] Gate 27：pane payload仍以字符串列表作为唯一产品投影。
- [Fail] Gate 28：真实canvas未显示nodes/pins/edges/selection/diagnostics。
- [Fail] Gate 29：palette与capability/schema仍漂移且无真实产品消费。

### Runtime、Preview与Currentness

- [Fail] Gate 30：Editor preview与game runtime逐帧parity没有产品链。
- [Partial] Gate 31：shared typed compiler、clip/layer局部diagnostic存在；graph/state runtime错误仍吞掉且未接Error List。
- [Partial] Gate 32：Editor document保存LKG；runtime热更失败不继续输出LKG且UI无stale状态。
- [Partial] Gate 33：runtime cache检查source/skeleton revision；无dependency install receipt、generation与out-of-order策略。
- [Fail] Gate 34：frame evaluation cache key不含artifact revision。
- [Partial] Gate 35：graph/skeleton/state source revision可触发局部重编；完整dependency closure精确invalidaton缺失。
- [Fail] Gate 36：missing duration/dependency仍可变成normalized `1.0`。

### Performance、Determinism与Events

- [Fail] Gate 37：graph steady-state evaluate仍分配weights/contributions/clips。
- [Partial] Gate 38：state parameter projection按revision缓存；输出string clone与graph scratch allocation仍在。
- [Partial] Gate 39：compiled/frame caches已bounded且BTreeMap/fingerprint索引；collision equality与artifact currentness仍需预算证明。
- [Fail] Gate 40：10k node canvas不存在，无法证明culling/LOD/incremental projection。
- [Fail] Gate 41：1k entity、多layer/nested/blend完整CPU/allocation/memory artifact缺失。
- [Partial] Gate 42：compiler/evaluator已有source-order与deep graph稳定测试；多线程/reload/save-reopen确定性未证明。
- [Partial] Gate 43：clip/graph/state transition event已有integration tests；loop/trigger/interruption/notify完整matrix缺失。

### Lifecycle、Accessibility与跨引擎资格

- [Fail] Gate 44：canvas keyboard/focus/selection/screen-reader/diagnostic navigation产品不存在。
- [Fail] Gate 45：migration/copy-paste/rename/merge/save-reopen无stable identity完整性证明。
- [Fail] Gate 46：plugin enable/disable/reload仍可能切换semantic authority。
- [Fail] Gate 47：compile crash/cancel/OOM/dependency loss无bounded retry/recovery receipt。
- [Fail] Gate 48：没有同语义跨引擎correctness与固定硬件性能证据。

## 12. 实施顺序与停止条件

1. 先完成M0并关闭Gate 01-06；在错误文档写入和双runtime authority关闭前，不新增节点类型。
2. 再完成M1-M2，建立唯一service/compiler和stable schema；Gate 07-22未收敛前，不把空Canvas包装成“Graph Editor完成”。
3. M3-M5完成真实产品、typed command、save/install/LKG和runtime-backed preview；不得让Editor另写解释器追赶显示效果。
4. M6补齐参数AST、BlendSpace/layer/nested/sync并收敛热路径；所有功能必须穿过同一compile/install/debug链。
5. M7最后执行规模、fault与跨引擎资格。未附正确性结果、硬件、build profile、原始数据和复现命令时，不允许宣称性能或表现超过Unreal等参考引擎。

## 13. 本轮验证边界

- 已逐文件扫描并冻结Editor authoring/product、schema/shared compiler/editor plugin、Runtime compiler/evaluator、focused tests及五套本地参考源码。
- 已静态确认两项P0仍Open，并确认shared compiler、document transaction/LKG、dense runtime compiler、cache与parameter snapshot的真实进展。
- 已纠正Editor76中固定Add Node按钮、无共享compiler、无document transaction、只检查Output可达环和full-map线性frame cache等过时描述。
- 已按Editor14/63/69/76/184/190/196 owner边界去重；whole-source clone/锁内compile和全局event effect继续由Editor196计数。
- 本轮未修改production源码，未运行Cargo或任何动态/GUI/GPU验证；不存在通过动态资格门的依据。
- `current_source_refresh_complete`只表示差距与重构路径按当前源码更新，不表示Animation Graph/State Machine已经完成。
