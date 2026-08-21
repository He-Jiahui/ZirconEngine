---
related_code:
  - zircon_plugins/ai/editor
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_runtime/src/core/framework/ai/snapshot.rs
  - zircon_runtime/src/core/framework/ai/tick.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_extension/viewport_overlay_provider.rs
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ai
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_plugins/06/2026-07-28-ai-m5-editor-debug-validation-manifest.md
  - docs/plans/performance/01/2026-07-30-runtime-framework-animation-ai-navigation-tasks-static-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/AssetDefinition_BehaviorTree.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/AssetDefinition_Blackboard.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeGraph.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/EdGraphSchema_BehaviorTree.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/SBehaviorTreeBlackboardEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeDebugger.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/SBehaviorTreeDiff.cpp
  - dev/UnrealEngine/Engine/Plugins/AI/EnvironmentQueryEditor/Source/EnvironmentQueryEditor/Private/EnvironmentQueryEditor.cpp
  - dev/UnrealEngine/Engine/Plugins/AI/EnvironmentQueryEditor/Source/EnvironmentQueryEditor/Private/SEnvQueryProfiler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/GameplayDebugger/GameplayDebuggerCategory_BehaviorTree.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/GameplayDebugger/GameplayDebuggerCategory_Perception.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/GameplayDebugger/GameplayDebuggerCategory_EQS.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeEditorModule/Private/StateTreeCompiler.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeEditorModule/Private/AsyncStateTreeDiff.cpp
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 20 · AI Behavior Tree / Blackboard / Perception / EQS / Debug Authoring 工程化差距

## 1. 结论

Zircon AI Editor 并非完全没有基础。插件已经声明 Behavior Tree asset type、`.btree.toml` importer descriptor、asset toolkit、graph editor、由 runtime 标准节点目录生成的 18 项 palette、Behavior Tree/Perception 两份 ZUI、带 play-session 与 delivery-sequence 防护的 typed runtime event consumer、按 World 区分的 PIE mirror，以及能生成视锥、听觉半径和 stimulus 连线的 `SceneGizmoOverlayExtract`。共享 Editor 侧又具备 owner-aware extension admission、runtime event consumer pump、document transaction、job scheduler、viewport overlay provider和asset toolkit基础。这些合同值得保留，尤其是 typed event schema、session fence、World-qualified lookup和owner-aware provider registration，重构不应退回UI直接锁runtime manager或每帧扫描World。

但当前 AI Editor 不是一个可以进入产品的半成品，而是编译、装配、执行和呈现四层同时断开的 descriptor shell。`overlay.rs`导入已经不存在的 `ViewportToolModeDescriptor`并调用不存在的 `EditorExtensionRegistry::register_viewport_tool_mode`；当前API只有`SceneModeDescriptor`、`SceneModeRegistration`与`register_scene_mode`。同文件声明provider ID却从未注册`ViewportOverlayProviderRegistration`。`plugin.rs`又从`sibling runtime_mirror`导入其中私有`use`的consumer常量，而不是从`extension_ids`导入公开定义。该crate一旦真正链接，就会先在Rust编译边界失败。

产品装配也明确排除了AI Editor。`first_party_runtime_catalog`会为`RuntimePluginId::Ai`返回runtime registration；`first_party_editor_catalog`的feature、dependency和registration分支却只有Navigation与Neural。`zircon_app`只委托这个catalog。因此项目即使选择`ai`并启动Editor Host，也得不到AI asset type、菜单、surface、event consumer或overlay。插件的9个测试全部直接构造`editor_plugin()`或mirror，绕过了真实first-party catalog与产品host；M5 manifest也明确只是`in_progress` validation manifest，不是acceptance record。

即便先修编译和catalog，五个可见operation仍无法执行。Import/Open/Validate/Compile/Toggle仅调用`EditorCommandDescriptor::operation`，AI Editor中operation factory为0；共享host在无factory时明确返回`MissingFactory`。`EditorAuthoringContributionBatch`只能登记descriptor，没有factory字段。更根本的是，`GraphEditorDescriptor`和`GraphNodePaletteDescriptor`在产品UI中没有consumer：全仓非测试产品搜索只找到registry/materialization，没有任何graph document/controller/canvas把它们变成可编辑节点。Behavior Tree ZUI的graph与inspector是两个`Space`，Blackboard是无provider的Table；Perception Debug只有一个无provider的agent Table。五个关键control ID在ZUI/tests之外均无production consumer，两个ZUI的event总数为0。

调试基础同样停在孤立库代码。`AiPieMirror`和`AiBtNodeResultMirror`只被插件自己的getter与tests引用；first-party catalog返回`EditorPluginRegistrationReport`后不会保留可调用的`AiEditorPlugin`实例，也没有panel/controller从host取得consumer state。Perception overlay controller没有product caller或sink adapter。Runtime的`BtNodeResultEvent`又不是节点级trace：它由`AiAgentTickReport::node_result_event()`从整个agent report的唯一`active_node`和最终status转换，每tick至多一个节点，没有active path、parallel additional nodes、search/abort/service/task lifecycle、tree/program generation或timestamp。所谓Behavior Tree高亮目前无法达到M5名称承诺。

仓内另有两份默认Workbench AI surface，但它们是第二套静态authority。`BT_Enemy`、`BB_Enemy`、`AIController_Enemy`、三行节点状态、`AI_Guard_01`、74度视锥、1200cm听觉、时间戳和runtime trace全部写死在ZUI；Validate/Simulate/Save/Compile/Diff只把固定成功字符串写回控件，例如“selector branch is reachable”和“simulation tick 00:12.4”，从不调用AI plugin/runtime/compiler。于是用户能看到比插件surface更完整的AI界面，却得到的是稳定伪造结果。

本轮登记5项P0、60项P1、12项P2。M0必须同时清除compile drift、接入first-party catalog并让所有可见命令失败诚实；M1-M5建立Behavior Tree/Blackboard source asset、transactional document、typed graph schema、真实compiler/cook和runtime-consistent preview；M6-M7以reader-gated delta trace、实例选择、breakpoint、Perception provider和有界overlay完成调试；M8收敛静态Workbench并建立产品/规模/故障资格。Runtime08F的world/agent generation、真实标准节点、latent abort、query service和Perception质量状态是Editor可声明完成的前置，Editor不能用更漂亮的UI掩盖runtime语义仍不成立。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | `#[test]` | 证据等级 |
|---|---:|---:|---|
| AI editor package | 10 / 1,654 / 58,169 | 9 | E3逐文件：Cargo、两份ZUI、capability/ID、plugin、mirror、overlay、tests |
| default Editor assembly | 6 / 490 / 17,285 | 7 | E3：first-party editor catalog feature/dependency/registration与App投影 |
| shared authoring/runtime-event/dispatch | 20 / 5,363 / 196,160 | 8 | E2完整inventory，E3复核graph descriptor、operation failure、provider admission、event session/pump链 |
| static AI Workbench | 8 / 3,232 / 140,084 | 1 | E3：两份ZUI、binding/navigation、preview action、固定feedback与bottom panel |
| runtime debug producer/contract | 5 / 865 / 34,509 | 1 | E3：snapshot、tick event转换、producer、manifest与focused test |
| selected combined scope | 49 / 11,604 / 446,207 | 26 | 当前工作树fingerprint `33a8b8dc4193e1dd9c634cae43a90fdcecf994404efe74c855146f19f640bead`；0 ignored，1个纯import排序在途source |

行数为物理文本行；fingerprint按相对路径排序，为每个当前工作树文件计算SHA-256，再对`path<TAB>hash<LF>`清单计算SHA-256。范围内`zircon_editor/src/ui/retained_host/workbench_preview_actions.rs`存在非本轮产生的纯import排序修改，本轮保持原样；因此`source_recheck_required`为true，实施前必须重算指纹并复核该文件终态。整个仓库另有其他用户/Session修改，本轮不吸收、不回退。

### 2.2 动态证据边界

本轮没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误与122个warning阻断；AI Editor又不在默认Editor catalog，插件workspace lockfile也已在Plugin01审查中确认漂移。本轮没有重复已知不能到达AI产品行为的lane。AI Editor的9个test attributes、default assembly的7个和其他selected scope的10个只作为静态inventory；它们不能证明crate可编译、产品会装配插件、operation可执行、surface有数据或runtime debug能够显示。

### 2.3 参考边界

Unreal是本域主参考。BehaviorTreeEditor源码覆盖Behavior Tree/Blackboard资产定义与factory、graph schema、decorator/service/task子节点、Details customization、事务/undo、编译回写、Find、Revision Diff、PIE instance选择、active path/additional nodes、runtime description、breakpoint和历史步进；EnvironmentQueryEditor覆盖EQS graph、test details与max/average/load/count profiler；GameplayDebugger覆盖远端可复制的Behavior Tree/Blackboard、Perception和EQS分类。StateTree Editor用独立compiler、compile log、diff、binding、simulation和rewind debugger说明复杂AI authoring不能压成一个通用descriptor。

Fyrox、Godot、Bevy和仓内Unity Graphics的Editor/engine源码对Behavior Tree、Blackboard、Perception关键字均无first-party同类authoring命中；Fyrox仅有Runtime08F已登记的轻量behavior utility，Godot本地checkout只有Navigation Agent工具可作空间调试辅助，Bevy task pool不是游戏AI Editor，Unity Graphics不拥有AI域。因此它们用于确认模块边界与“不可降低基线”，不作为缺失企业AI authoring能力的反证。

## 3. 必须保留的基础

1. 保留runtime/editor之间typed event ID、payload schema和decode边界，不允许UI直接依赖`DefaultAiManager`内部锁。
2. 保留`play_session_id`、delivery sequence和World-qualified lookup，并升级为world/agent/program/schema generation，而不是退回裸entity ID。
3. 保留owner-aware extension、operation factory、runtime consumer和viewport overlay provider admission；修复AI插件去dogfood这些合同。
4. 保留runtime标准节点目录驱动palette的方向，但目录必须补齐authoring schema且包含第三方owner/reload generation。
5. 保留`SceneGizmoOverlayExtract`和finite-input过滤，增加candidate/primitive/bytes/time budget、实例过滤与quality state。
6. 保留共享document transaction、background job、asset toolkit、autosave/recovery和diagnostic journal，不在AI插件内另造私有简化栈。
7. 保留M5 manifest的诚实`in_progress`状态，直到产品gate自动生成完成状态。

## 4. 目标架构

```text
ProjectPluginManifest(ai)
  -> first_party_editor_catalog(ai-editor feature)
  -> AiEditorPluginRegistration
       -> BehaviorTree / Blackboard / PerceptionConfig asset definitions
       -> typed operation factories + document toolkit factories
       -> graph schema / palette / property editors
       -> runtime trace reader lease + viewport overlay provider

source assets
  -> immutable AuthoringDocument(revision, stable ids, dependencies)
  -> shared AI semantic compiler
  -> diagnostics + prepared program/schema artifact
  -> atomic publish generation / LKG

AI runtime trace
  -> reader-gated bounded delta stream
  -> world/agent/program/schema generation router
  -> debugger timeline / blackboard watch / perception overlay
  -> no simulation mutation unless explicit debug command authority
```

核心owner划分：

1. `zircon_runtime::core::framework::ai`拥有稳定source/compiled/debug中立合同，不拥有Editor widget或具体plugin manager。
2. AI runtime拥有semantic compiler/executor所需prepared program、world runtime和reader-gated trace producer；Editor不能复制一套验证规则。
3. AI Editor拥有document/session、selection、graph projection、transaction、diagnostic projection和debug presentation，不拥有业务simulation truth。
4. Asset system拥有source revision、import/reimport、dependency、cook/DDC与atomic publish；操作只能通过typed command/result进入。
5. Runtime event host拥有session/subscription/budget/backpressure；view打开关闭只改变reader lease，不改变AI决策结果。
6. Viewport overlay registry拥有provider lifecycle；AI provider只读取选定session/world/agent的bounded debug cache。
7. 静态Workbench只能作为显式sample fixture存在，默认产品入口必须路由到真实AI toolkit，不能保留第二套伪业务authority。

## 5. P0 阻断项

### P0-1：AI Editor crate与当前Editor API静态编译不兼容

`overlay.rs`唯一使用仓内不存在的`ViewportToolModeDescriptor`和`register_viewport_tool_mode`；当前合同是`SceneModeDescriptor`/`SceneModeRegistration`/`register_scene_mode`。`plugin.rs`还通过`runtime_mirror` sibling路径引用其私有consumer常量。默认产品未链接该crate，因而长期掩盖compile drift。M0必须先建立AI Editor required compile lane，再谈功能。

### P0-2：first-party Editor产品catalog没有AI provider

AI runtime位于`base-runtime-plugins`，AI Editor却不在`first_party_editor_catalog`的feature、dependency或registration map中；App没有另一条路径。Project选择AI只会得到runtime，不会得到同包Editor模块。必须新增显式feature与registration快照，并以真实Editor Host启动验证selection、capability、resource解析和disable/unload。

### P0-3：Import/Open/Validate/Compile/Toggle全部是无factory operation

AI Editor登记5个operation descriptor、0个`OperationCommandFactory`。共享host明确把它们终结为`MissingFactory`。Graph/asset toolkit虽引用这些operation，也只能把用户送到失败路径。必须实现typed payload、prepare/apply/undo或read-only command，并在factory不存在时不展示为可用命令。

### P0-4：Behavior Tree、Blackboard、Perception和Overlay surface没有产品controller/provider

Behavior Tree graph与inspector是`Space`，Blackboard/Palette/Perception agent Table均无data provider或event；五个关键control ID无production consumer。Overlay mode声明provider ID但注册provider数为0，controller和mirror也无product caller。必须以真实document/runtime source驱动surface；descriptor、空Table和可单测builder不能计为M5交付。

### P0-5：默认Workbench用固定AI数据与成功反馈构成第二套假产品

Workbench的Behavior/Perception两份workspace把资产、节点、agent、范围、timestamp与结果硬编码；Validate/Simulate及通用Save/Compile/Diff只写固定成功文案。它既不调用AI插件也不消费PIE mirror。M0先显式标为sample/disabled，M8硬切到真实toolkit；不得继续让“selector reachable”或“simulation tick”伪装业务完成。

## 6. P1 核心重构差距

### P1-1：Behavior Tree没有稳定source asset合同

Editor只有`AssetTypeId("ai.behavior_tree")`与扩展名descriptor，没有可持久化asset DTO、source revision、schema version、stable node identity、dependency或migration owner。Runtime `.btree.toml` parser不能自动等价为Editor document。

### P1-2：Blackboard没有独立asset type、factory、importer或toolkit

现有Blackboard只是Behavior Tree ZUI右侧一张Table。无法创建、打开、复用或继承schema，也无法让多棵tree共享同一份key合同。Unreal将Blackboard作为独立资产并与BT编辑器协同打开。

### P1-3：没有Create New产品入口

AI只声明import已有`.btree.toml`，没有Behavior Tree/Blackboard创建factory、template、命名冲突或目录事务。用户无法从Content Browser建立第一份AI资产。

### P1-4：Open operation不读取asset locator或当前source revision

Toolkit descriptor指向`ai.behavior_tree.open`，但没有factory/controller把`AssetToolkitOpenRoute`转换为document。重复打开、focus已有tab、read-only状态、丢失source和外部修改均无语义。

### P1-5：Import没有settings、provenance或reimport合同

Importer只有display name、扩展名与output type，不读取bytes，不绑定runtime compiler，也没有source hash、setting schema、diagnostics、dependency、dry-run、cancel、duplicate或reimport conflict。

### P1-6：没有authoring document IR

Graph descriptor不是图数据模型。当前没有node/edge/auxiliary attachment/Blackboard reference的immutable revision，也没有source DTO与compiled preorder program之间的稳定映射。

### P1-7：AI编辑不进入共享transaction/history

插件没有command、inverse delta、transaction ID、merge policy或history registration。节点移动、连线、参数改变、key重命名和批量粘贴都无法undo/redo。

### P1-8：没有dirty/save/autosave/recovery/lock闭环

AI插件没有document toolkit、dirty generation、save acknowledgement、autosave snapshot、crash recovery或project session lock。Editor02已有的基础没有被消费。

### P1-9：保存没有原子发布与冲突检测

不存在temporary write、flush、rename、source revision compare、external-change prompt或last-good。未来直接写`.btree.toml`会重演其他asset authoring的覆盖与mixed-generation问题。

### P1-10：source、compiled artifact和runtime generation没有绑定

Editor不能回答当前PIE agent运行的是哪个source revision、compiler build或prepared program。Debug高亮即使出现也可能落到已经编辑后的不同图上。

### P1-11：Subtree依赖与refactor rename不可见

Runtime Subtree依赖只靠字符串参数和注册顺序。Editor没有reference graph、循环预检、rename redirect、find usages或跨asset原子更新。

### P1-12：没有source schema migration

ZUI version 2不是Behavior Tree asset version。节点参数、Blackboard type、extension node generation或compiler规则升级后，没有migrator、compatibility report或LKG reopen策略。

### P1-13：palette只包含进程内标准节点快照

`standard_node_catalog()`绕过runtime实际typed extension catalog，第三方Rust/ZrVM节点、owner、enablement、reload generation和capability均不会出现在palette。

### P1-14：palette节点没有参数schema和默认值

`GraphNodeDescriptor`只携id/display/category与可选pin；AI构造时连pin也为空。Cooldown、Parallel、MoveTo、ScriptTask等需要哪些参数、类型、范围、默认值和资源均不可author。

### P1-15：没有BT结构端口与cardinality规则

Selector/Sequence/Parallel的child顺序、Decorator单child、Service宿主、Task无child、Subtree引用不能由当前空pin descriptor表达。用户界面无法在编辑时阻止非法拓扑。

### P1-16：没有节点创建与放置factory

palette row没有drag/drop、context action、stable ID allocation、default parameter block或transaction。登记18个名字不等于能实例化18个可执行节点。

### P1-17：没有连接、断开、重排和auxiliary attachment语义

仓内没有AI graph schema consumer。连接是否成环、Decorator/Service挂载位置、child优先级、Parallel分支和Subtree边界都没有命令或视觉规则。

### P1-18：Details区域没有selection binder

`AiBehaviorTreeInspector`是`Space`，没有selected node、multi-selection、mixed value、property editor、validation或change command。所有节点参数当前不可编辑。

### P1-19：缺少Blackboard-aware typed property editor

Decorator/Task无法从schema筛选兼容key；enum、entity/class、asset、Vec3、navigation filter、animation state、script function等也没有专用picker或reference validation。

### P1-20：node ID生命周期不成立

Runtime trace以字符串`node_id`关联UI，但Editor没有stable ID创建、duplicate remap、copy/paste namespace、rename或collision检查。显示名与持久身份也未分离。

### P1-21：root、child和Decorator/Service视觉层级没有实现

插件画布是空`Space`；静态Workbench则用三行Table伪图。没有root约束、连线、execution order、auxiliary badge、abort range或Subtree breadcrumb。

### P1-22：缺少图导航与大图可用性

没有pan/zoom/minimap、auto layout、focus selection、bookmark、comment/group、collapse、outline或viewport virtualization。复杂工程树无法仅靠一张Table维护。

### P1-23：图编辑没有键盘与辅助功能合同

不存在node/edge语义树、键盘创建/连接/重排、focus restore、screen reader label或高对比debug状态。ZUI空槽不能作为accessibility证据。

### P1-24：没有Find与reference search

无法按node type、parameter、Blackboard key、asset、script function或diagnostic检索当前tree和项目。Unreal提供`FindInBT`并能从结果跳到节点。

### P1-25：没有revision diff与merge

静态Workbench的Diff只是固定文案。没有graph structural diff、property diff、source-control revision、next/previous difference、conflict或three-way merge。

### P1-26：clipboard没有类型与版本边界

当前没有copy/paste；目标实现还需携source schema、node owner/generation、stable ID remap、Blackboard dependency和size/depth budget，不能复制任意JSON后直接执行。

### P1-27：diagnostic无法定位节点、pin、key或source span

Runtime compiler错误主要是字符串；Editor没有diagnostic code/severity、asset revision、node ID、parameter/key、source span、related asset或fix action，无法形成可跳转问题列表。

### P1-28：Validate与Compile没有调用共享AI compiler

两条operation都无factory。更没有proof表明Editor与runtime `.btree.toml` compiler使用同一semantic IR、节点目录generation和Blackboard layout规则。

### P1-29：authoring metadata与runtime node descriptor均不足

Editor只得到id/display/category；Runtime08F已确认descriptor缺parameter schema、ports、defaults、side effect、lifecycle、thread与debug metadata。不能靠Editor猜测具体节点规则。

### P1-30：没有incremental compile与background job

复杂tree/schema/project reference检查没有job category、dedupe key、priority、cancel、progress、deadline或shutdown fence。未来同步compile会阻塞UI，异步无revision fence又会发布旧结果。

### P1-31：没有prepared artifact、cook与DDC

AI asset没有content digest、compiler/toolchain/schema/dependency key、prepared program blob、cache receipt、target/platform identity或package inclusion。测试中直接register DTO不是shipping asset链。

### P1-32：没有last-good publish与instance migration策略

Compile成功/失败不能原子切换program generation；打开PIE实例在source change后是继续旧program、restart、逐节点迁移还是拒绝，均未定义。

### P1-33：跨tree依赖图与循环诊断缺失

Runtime当前只验证当前target是否已注册，不能发现多asset环。Editor应在prepare阶段建立全局dependency graph并给出cycle path与affected assets。

### P1-34：Blackboard schema变更没有兼容性分析

key add/remove/rename/type/default变更对Decorator、Task、save state与active agents的影响不可见。没有migration preview、reference fixup或breaking-change gate。

### P1-35：Save/Play前没有真实产品validation gate

用户可见Validate是假反馈，Open/Compile又失败；没有dirty document preflight、required errors、waiver、save-with-errors policy或PIE admission。

### P1-36：编译诊断没有稳定code和修复动作

没有duplicate node/key、unreachable branch、always true/false、unbounded loop、missing provider、latent-without-cancel、query budget等领域diagnostic code，也没有quick fix与suppression provenance。

### P1-37：缺少batch commandlet与CI asset lane

项目无法在headless cook/CI中列举所有BT/Blackboard、编译、输出机器可读结果并绑定source Build Set。单个manager tests不能替代content qualification。

### P1-38：compiler与plugin input没有hard budget/fuzz边界

Editor没有node/edge/depth/parameter bytes/string/clipboard/import size cap，也没有third-party node schema fuzz、migration fuzz或malicious asset隔离。

### P1-39：Blackboard没有inheritance与parent编辑

当前Table不能表示parent schema、inherited key、override/default、shadow conflict或循环。Unreal Blackboard Editor明确区分继承与本地entry并同步BT关联对象。

### P1-40：Blackboard Table没有任何数据provider或mutation event

ZUI只声明`row_identity_field = "key"`，没有rows、selection callback、add/delete/rename/type/default操作或empty/error/loading state；测试只搜索control ID字符串。

### P1-41：缺少key组织与批量编辑

没有category、description、sort/order、search/filter、multi-select、duplicate或CSV/JSON review export。大型schema会退化为不可维护字符串列表。

### P1-42：key重命名/删除没有reference fixup

Decorator、Service、Task、Subtree、script和scene配置引用不会被查找或事务更新；删除无法阻断dangling compiled slot和旧runtime value。

### P1-43：type change没有数据迁移策略

Bool→Integer、Entity→Object、Scalar→Vec3等变更没有compatibility、default conversion、save migration、active instance restart或failure report。

### P1-44：默认值、当前值、继承值和运行时写入未区分

Debug snapshot只有`Vec<AiBlackboardEntry>`，UI没有source default、instance current、writer、last change generation/time或inherited origin，易把观察值误当资产值编辑。

### P1-45：没有Blackboard引用与写入审计

无法显示哪些node读/写key、observer abort依赖、script/event writer或未使用key，也不能检查多writer冲突与authority policy。

### P1-46：缺少per-agent watch/filter与安全导出

没有agent选择、key watch list、changed-only、type/filter/sort、value history、copy/export预算或secret/object reference redaction。

### P1-47：Runtime无debug reader时仍生产全量snapshot

Runtime08F已确认每tick无条件`runtime_snapshot()`并clone全catalog/agent Blackboard/Perception后发送。Editor consumer存在与否不控制采集，违反既有PERF-MVP-584交接。

### P1-48：mirror接收完整owned DTO且没有entry/bytes/age预算

每个snapshot逐World清除再把完整frame插入BTreeMap；没有max agents/keys/stimuli/string bytes、oldest age、drop reason、high-watermark或slow-consumer telemetry。

### P1-49：debug identity缺program/schema/world/entity generation

key只有`(world u64, entity u64, node String)`。World/Entity ID复用、agent换tree、asset reload或同名Subtree node可继承旧高亮和值；snapshot也不携source/compiler generation。

### P1-50：`BtNodeResultEvent`不是真实node trace

事件从agent report唯一`active_node`合成，每tick至多一个overall status。Parallel siblings、active path、Decorator evaluation、Service tick、search failure、abort、latent start/finish和Subtree stack全部丢失。

### P1-51：没有debug instance selector

Mirror可按World查agent，但UI没有session/world/entity/tree/actor选择器、follow selection、pin instance或同entity跨world消歧。Perception builder默认迭代所选World全部agent。

### P1-52：没有timeline、历史步进与pause语义

Mirror只保留每World最新snapshot和每node最新event；没有timestamp/frame/fixed step、ring buffer、step back/forward/into/over/out、pause/resume或history budget。

### P1-53：没有breakpoint、conditional breakpoint或logpoint

Node result mirror只能被动收数据，不能在authoritative runtime设置generation-bound breakpoint、条件、hit count、one-shot、continue或安全失败策略。

### P1-54：debug期间编辑与live reload policy未定义

没有read-only lock、edit-and-restart、pending patch、instance migration、source mismatch banner或old program view。用户可能编辑A revision却观察B generation。

### P1-55：diagnostic不绑定当前source位置

`AiAgentTickReport.diagnostic`和`BtNodeResultEvent.diagnostic`只是字符串，无法证明来自哪个node generation/parameter/provider，也不能跳转到当前revision或区分stale runtime error。

### P1-56：PIE debug没有dedicated server、remote和权限边界

事件consumer只围绕本地play session。没有remote target identity、transport auth、rate/bytes quota、server authority、multi-client comparison或disconnect/reconnect generation。

### P1-57：Perception toggle没有executor与options UI

Toggle operation无factory；`AiPerceptionOverlayOptions`只能由测试/代码mutate，Perception Debug ZUI也没有Sight/Hearing/Stimuli开关、sense filter或overlay state反馈。

### P1-58：overlay默认绘制所选World全部agent，复杂度无界

`build_ai_perception_overlay_with_options`遍历`agents_in_world`，为每agent生成sphere、24段听觉圆、最多24段视锥和每stimulus连线/球。没有selected agent、distance/frustum/LOD、primitive/time/bytes cap或reuse。

### P1-59：Perception debug缺quality、lost reason与空间查询证据

Overlay只知道position/FOV/range和简化stimulus；不显示Physics unavailable/error/deferred、occluder、query latency、pair backlog、last update/lost reason、source generation、team/affiliation或sense provider状态。

### P1-60：现有测试停在registration、mirror和geometry builder层

9项插件测试不通过first-party catalog/Editor Host，不分派operation，不打开asset，不保存document，不绑定control，不安装overlay provider，也不验证runtime reader gate。`plugin_registration().is_success()`在source compile之前没有意义，control ID字符串测试还把空Blackboard Table命名为monitor。

## 7. P2 能力差距

### P2-1：缺少EQS source asset、graph、test/score编辑与profiler

Runtime08F尚无query service，Editor更没有Context/Generator/Test/Score graph、preview result、query history或max/average/load/count profiler。待Runtime M5 query contract稳定后实现。

### P2-2：缺少StateTree、Utility AI和HTN authoring

复杂层级状态、长流程规划与utility score不能都塞入Behavior Tree。应以独立asset/editor/compiler模块接共享Blackboard/query/debug，而非扩大同一张BT graph。

### P2-3：缺少可复制Gameplay Debugger分类与输入配置

没有Behavior/Blackboard/Perception/EQS独立category、按需订阅、远端复制、keyboard mapping、canvas/viewport统一或category enablement持久化。

### P2-4：缺少Brain/Agent/Controller与scene authoring

Runtime08F没有Brain/Agent scene component；Editor也无法给entity选择tree/schema/start policy、controller、team或restart behavior，资产完成后仍无法放入普通scene。

### P2-5：缺少team、squad、cover、reservation与world knowledge工具

Perception/team filter、cover map、Smart Object、reservation冲突、共享knowledge和战术调试没有资产、viewport或审计面。

### P2-6：缺少Mass AI、大世界分片与LOD分析

没有10k/100k agent分层、World Partition cell、spawn/despawn churn、scheduler lane、oldest decision age、pair backlog和memory heatmap工具。

### P2-7：缺少Sense provider与Perception Config Editor

Sight/Hearing之外的Damage/Touch/Custom、affiliation、dominant sense、forget policy、trace channel和provider reload需要独立typed config与extension authoring。

### P2-8：缺少trace/rewind/replay分析

没有durable trace schema、recording session、scrub、decision diff、Blackboard write causality、Perception/query timeline或Rewind Debugger式跨帧关联。

### P2-9：缺少网络权威与多实例差异调试

无法比较server/client/replay decision、prediction/correction、remote agent或determinism divergence，也没有数据敏感性与访问控制。

### P2-10：缺少multi-user协作与semantic merge

大型AI资产需要node/key stable identity、change list、lease、three-way graph merge、conflict和review annotation；当前字符串TOML没有对应Editor协议。

### P2-11：缺少AI scenario、simulation和质量回归工作台

静态Simulate按钮必须由可复现scenario、seed/fixed step、mock/provider policy、expectation、failure minimization、coverage与batch结果替代。

### P2-12：缺少同质量基线的可审计性能/体验比较

没有固定内容、Build Set、hardware、agent/query规模、p50/p95/p99、alloc/RSS/debug overhead、authoring latency与参考引擎同质量配置，不能宣称Editor或AI性能优于Unreal。

## 8. 参考实现差异矩阵

| 能力 | Zircon current source | Unreal reference | 重构结论 |
|---|---|---|---|
| 产品装配 | AI runtime linked，AI Editor不在first-party catalog且source API漂移 | BehaviorTreeEditor/AIGraph作为明确Editor modules注册asset/factory/customization | M0先让同一project selection闭合runtime+editor且required lane编译 |
| 资产 | descriptor-only `.btree.toml` output type，无Blackboard asset | 独立Behavior Tree/Blackboard asset definitions与factories，关联Editor复用 | 建立versioned source asset、create/import/reimport/toolkit |
| 图编辑 | 空Space与无人消费的Graph descriptor | Graph schema、connection rules、Decorator/Service子节点、Details、Find | shared typed graph IR和domain schema，禁止Table伪图 |
| 事务/保存 | 无document/undo/dirty/save | Undo/Redo、editing objects同步、package save、Revision Diff | dogfood Editor02 document transaction与source revision |
| 编译 | Validate/Compile无factory | graph update/asset compile，StateTree另有compiler/log/commandlet | shared semantic compiler、prepared artifact、LKG、headless lane |
| Blackboard | 无provider Table | 独立editor、parent/inherited/local key、runtime value view | standalone asset、typed key mutation、reference fixup与watch |
| BT debugger | 最新snapshot + 单active node latest event | instance选择、active path/additional nodes、runtime description、breakpoint、历史步进 | reader-gated structured trace与generation-bound debugger |
| Perception/EQS | builder可画简化FOV/range，provider未注册；无EQS | GameplayDebugger Perception、EQS query data/render，EQS graph/profiler | bounded provider、quality/lost/query数据和P2 EQS Editor |
| 静态Workbench | 固定BT/agent/结果与成功文案 | 编辑器数据来自asset/runtime instance/profiler | 删除第二authority，所有可见结果绑定source/result receipt |

## 9. 必须硬切的旧实现

1. 删除`ViewportToolModeDescriptor`/`register_viewport_tool_mode`旧API引用；使用当前`SceneModeRegistration`和真实`ViewportOverlayProviderRegistration`，不保留兼容alias。
2. AI Editor必须进入first-party catalog与Editor Host required build；不得继续只在plugin workspace中“存在”。
3. 五个descriptor-only operation要么绑定typed factory，要么从可见菜单/asset toolkit撤下；禁止显示后稳定`MissingFactory`。
4. Behavior Tree graph/inspector、Blackboard和Perception Table在无provider时必须显示typed unavailable，不得把空控件计为能力。
5. Runtime标准节点目录增加唯一authoring metadata；Editor不得维护另一套参数/端口规则或只投影名字。
6. 逐帧full `AiBehaviorDebugSnapshot`退场；仅显式full capture使用owned DTO，常态使用reader-gated bounded delta trace。
7. `BtNodeResultEvent`单active-node兼容流退场；由versioned execution trace覆盖完整lifecycle并携program/node generation。
8. `BT_Enemy`/`AI_Guard_01`静态Workbench默认入口退场；保留sample时必须显式标记fixture且不能发布成功状态。
9. Blackboard不再嵌入为无owner Table；建立独立asset/schema并由Behavior Tree引用。
10. capability状态由compile/product/debug/scale gates生成；registration成功、test attribute数量或历史manifest不能手写提升M5。

## 10. 分层里程碑

### M0：编译、产品装配与能力真相

- 修复current API drift与私有constant导入，新增AI Editor Windows required check。
- 将AI Editor接入first-party catalog/App selection；补resource、enable/disable/unload与consumer lifecycle product test。
- 撤下所有无factory命令，静态Workbench显式sample/disabled，capability保持Experimental/Partial。

### M1：Behavior Tree / Blackboard source asset与导入创建

- 定义versioned BehaviorTree、Blackboard source DTO、stable IDs、dependency和migration。
- 实现Create/Import/Reimport、provenance/settings、typed diagnostics、locator/revision与asset index。
- 普通项目可从Content Browser创建、打开、关闭和重新打开两类资产。

### M2：transactional document与真实toolkit

- 建立immutable document revision、selection、command/inverse delta、dirty/save/autosave/recovery/lock。
- Open operation factory复用已有tab并绑定asset revision；所有保存atomic且有ack/conflict。
- multi-document、Subtree依赖和Blackboard引用进入project reference graph。

### M3：typed BT graph与Blackboard Editor

- runtime node目录补齐parameter/pin/cardinality/lifecycle/debug metadata并驱动palette/property editor。
- 实现node/edge/Decorator/Service/Subtree创建、连接、重排、copy/paste、find、layout和accessibility。
- 实现Blackboard parent/local key、default/type、reference fixup、compatibility preview和runtime watch。

### M4：semantic compiler、artifact与diagnostics

- Editor/runtime/cook共用pure compiler IR；background job携source/dependency/catalog generation与cancel budget。
- 产出prepared tree/schema artifact、DDC/action digest、LKG与atomic publish/migration policy。
- diagnostics有code/severity/node/key/span/related asset/fix；增加commandlet、CI、fuzz与hard budget。

### M5：runtime-consistent preview与scene activation

- Runtime08F先交付Brain/Agent scene component、asset-to-runtime启动、world/entity cleanup和真实标准节点。
- Editor PreviewWorld只加载当前prepared generation，具备fixed step/seed/provider policy与cancel。
- source/compiled/runtime generation可见，Play admission拒绝stale或invalid required asset。

### M6：reader-gated Behavior Tree debugger

- 建立reader lease、bounded delta trace、active path/additional nodes、search/abort/task/service/subtree lifecycle。
- 支持session/world/instance选择、pause、step/history、breakpoint/logpoint、Blackboard changed slots与diagnostic jump。
- slow/absent Editor不反压runtime；remote/dedicated server走授权、quota与disconnect generation。

### M7：Perception provider、overlay与query debug

- 注册真实viewport provider和Toggle factory，面板驱动sense/agent/filter/options与quality state。
- overlay按selection/frustum/LOD/budget生成，显示occlusion/deferred/error/lost/source generation/team与backlog。
- Runtime query service稳定后增加EQS result/score/trace preview，避免Editor私造查询算法。

### M8：Workbench收敛、产品资格与性能资格

- 默认Workbench AI入口路由真实toolkit/debugger，删除固定数据与固定成功feedback。
- 覆盖project create/open/import/edit/undo/save/reload/compile/cook/PIE/debug/disable/reload/recovery全链。
- 建立10k/100k规模、consumer stall、world churn、asset/plugin reload、fuzz/soak与同质量reference benchmark。

## 11. 验收门

### 11.1 M0-M2：产品与文档

1. `zircon_plugin_ai_editor`在Windows required lane使用current workspace/lock编译；源码不存在旧Viewport Tool Mode API或private sibling import。
2. ProjectPluginManifest选择AI后，真实Editor Host同时装配同generation runtime/editor模块；client/server不装Editor模块。
3. disable/unload AI时asset toolkit、operation、consumer、provider和reader lease按owner撤销，已打开dirty document走统一决策，不悬空callback。
4. 所有可见AI命令均有factory或typed unavailable reason；产品测试不出现`MissingFactory`。
5. Content Browser可创建Behavior Tree与Blackboard、导入`.btree.toml`、显示provenance/settings/diagnostics并安全reimport。
6. 同asset重复Open聚焦同document；source丢失、read-only、external change与revision conflict有typed终态。
7. node/key edit、move、connect、rename、delete、paste均进入共享transaction，undo/redo与dirty token逐步一致。
8. Save/Save All/autosave/recovery写同一canonical source revision，crash/power-loss注入不产生截断或mixed generation。

### 11.2 M3-M5：语义与产物

9. Palette包含当前enabled标准/第三方节点及owner generation；disable/reload后旧node有typed unavailable/migration，不静默换实现。
10. 每个节点的parameter/pin/cardinality/default/range/reference由唯一schema驱动，Editor与compiler不重复硬编码。
11. 非法cycle、child count、Decorator/Service placement、missing Subtree/key/provider在编辑时和headless compile得到相同diagnostic code。
12. Blackboard parent/local/default/type/reference变更有兼容性preview；rename/delete事务更新引用或明确拒绝。
13. source、prepared artifact、compiler Build Set、dependency digest、node catalog generation和runtime program generation可追溯。
14. compile job可cancel、dedupe、限时并拒绝stale result；失败保留LKG且不会把半成品发布给active agent。
15. cook/pack从prepared artifact与dependency graph工作，clean clone和增量DDC命中产生相同digest。
16. 普通scene只配置Brain/Agent与asset即可在client、dedicated server、PIE和headless lane启动AI，无测试手工register/tick。

### 11.3 M6-M7：调试与可视化

17. 无AI debug reader时runtime不构造full snapshot、不clone catalog/Blackboard/Perception；telemetry证明debug bytes和alloc为零或固定常数。
18. trace携session/world/agent/program/schema/node generation与frame/timestamp，ID复用、tree switch和reload不继承旧高亮。
19. Parallel、Decorator、Service、Subtree和latent Task的active/search/abort/start/finish全部可视，不能由单`active_node`冒充。
20. instance selector可区分多PIE world、相同entity ID、dedicated server和remote target；断线/重连不混session。
21. pause/step/history/breakpoint/logpoint与runtime scheduler协作，取消或continue获得exactly-one terminal且不改变非调试agent。
22. Blackboard monitor支持changed-only、writer/generation/time、default/current/inherited区分和有界watch/export。
23. Perception Toggle通过真实factory与provider生效；关闭view释放reader，关闭overlay清除所有owned extract。
24. overlay只画选定/可见agent并受candidate/primitive/bytes/time budget；10k agent场景不会生成无界line/pick Vec。
25. Physics unavailable/error/deferred、occluder、pair age/backlog、lost reason、team/affiliation和source generation在debug中可区分。
26. slow consumer、payload decode error、budget drop和provider revoke有telemetry/diagnostic，不反压AI tick或无限保留mirror state。

### 11.4 M8：资格与诚实完成

27. 默认Workbench不含`BT_Enemy`/`AI_Guard_01`固定业务结果；所有状态绑定asset/runtime/job receipt或明确sample标识。
28. create→edit→undo→save→reload→compile→cook→PIE→debug→stop→reopen真实产品lane在clean workspace通过。
29. compile error、source conflict、world unload、entity ID复用、asset/plugin reload、remote disconnect和Editor crash恢复均有产品故障lane。
30. 10k active/100k LOD agent记录Editor打开/关闭时runtime p50/p95/p99、alloc、RSS、trace bytes、oldest age和UI frame cost。
31. graph/import/compiler/event/clipboard/plugin schema有size/depth/count/time hard budget、fuzz与长时间soak，RSS/state/queue不持续增长。
32. capability/M5状态由current-source Build Set与上述gate生成；registration test、空Table、固定feedback、零测试或历史绿色不得满足完成或“优于Unreal”声明。

## 12. 既有计划与handoff处理

- `docs/plans/zircon_plugins/06-ai.md`继续拥有业务实现，但M5必须按本报告重开：T1不能用Graph descriptor/palette数量验收，T2不能用最新full snapshot和单active node验收，T3不能用geometry builder单测验收。
- `2026-07-28-ai-m5-editor-debug-validation-manifest.md`保持`in_progress`；它已经声明“not an acceptance record”，不得被索引或manifest投影升级。
- Runtime前置由[Runtime08F](../zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md)拥有；Brain/Agent scene启动、world/entity generation、真实节点、latent cancel、query service、Perception quality和reader-gated trace先于Editor完成。
- Document/asset/job/PIE通用能力分别复用Editor02/04/09/07，不在AI插件内创建第二套history、asset index、thread或session协议。
- Workbench固定AI surface由本计划M0/M8拥有收敛；任何暂留必须显式sample并在产品capability中不可达。
- 当前in-flight import排序不改变本轮业务结论，但实施开始前需重算49文件fingerprint并确认first-party catalog、AI Editor和Workbench终态。

## 13. 完成定义

本报告仅在以下条件同时满足时转为implemented：AI Editor current source可编译并进入产品catalog；普通项目可创建/导入/事务编辑/保存/编译/cook Behavior Tree与Blackboard；scene可通过asset启动真实AI；Editor和runtime共享semantic compiler与generation；Behavior debugger提供有界、按需、完整lifecycle trace；Perception provider真实安装并有质量/预算状态；静态Workbench不再伪造结果；32项gate均绑定current Build Set、原始结果和规模数据。此前保持`review_complete / implementation_pending`。

## 14. 本轮验证状态

- 已逐文件读取AI Editor package 10文件与9个test attributes；确认5个operation descriptor、0 factory、2个业务`Space`、0 ZUI event、0 overlay provider registration。
- 已从Project selection追到first-party runtime/editor catalog与App投影；确认runtime有AI分支、Editor无AI feature/dependency/registration。
- 已从asset toolkit/graph descriptor追到operation dispatch；确认无AI product graph consumer且所有operation会落到`MissingFactory`。
- 已从runtime event producer追到mirror/overlay/control ID；确认typed host基础存在，但UI/provider无consumer，node event只由单active-node report合成。
- 已逐项读取两份AI Workbench ZUI、binding、navigation和feedback；确认所有业务数据与Validate/Simulate/Save/Compile/Diff终态固定。
- 已对照Unreal BehaviorTreeEditor/AIGraph/Blackboard/Diff/Debugger、EnvironmentQueryEditor/Profiler、GameplayDebugger与StateTree Editor关键实现。
- 已验证selected scope统计为49 / 11,604 / 446,207、26 test attributes、0 ignored，fingerprint为`33a8b8dc4193e1dd9c634cae43a90fdcecf994404efe74c855146f19f640bead`。
- 本轮没有修改production code/tests，没有运行动态测试，也没有把既有编译阻断、静态test inventory或M5 manifest当作通过证据。
