---
related_code:
  - zircon_plugins/ai/editor
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/plugin/extension_materialization.rs
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ai
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_runtime/src/core/framework/ai
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/100-runtime-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-task-navigation-network-save-scalability-editor-product-integration-current-source-review.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_plugins/06/2026-07-28-ai-m5-editor-debug-validation-manifest.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor
  - dev/UnrealEngine/Engine/Plugins/AI/EnvironmentQueryEditor
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/GameplayDebugger
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeEditorModule
  - dev/UnrealEngine/Engine/Plugins/Runtime/SmartObjects/Source/SmartObjectsEditorModule
  - dev/Graphics/Packages/com.unity.shadergraph/Editor
  - dev/Fyrox/editor/src
  - dev/godot/editor
  - dev/bevy/crates/bevy_asset/src
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
review_id: Editor89
---

# 89 · Editor AI / Behavior Tree / Blackboard / Perception / EQS / StateTree / Smart Object / Debug Authoring 当前源码工程化差距

## 1. 当前结论

冻结于 `2026-08-25T16:59:19+08:00` 的当前物理工作树仍不能把 AI Editor 判定为可交付产品。它拥有一批应当保留的局部基础：AI runtime 标准节点目录能够生成 18 项 palette；两类 typed runtime event consumer 带 play-session 与 sequence fence；`AiPieMirror` 按 `(World, Entity)` 隔离快照，`AiBtNodeResultMirror` 当前在 agent 下建立第二层 node map，避免按 node 查询时构造临时 `String`；Perception overlay 对非有限输入做过滤，并按预计 line/pick 数预分配容量；共享 Editor 已有 owner-aware contribution、operation factory、document transaction、job、scene mode factory、viewport overlay provider和runtime event host。

这些基础仍只构成 descriptor、mirror与builder，不构成 authoring/debug 产品链。`zircon_plugins/ai/editor/src/overlay.rs` 继续导入仓内不存在的 `ViewportToolModeDescriptor`，并调用不存在的 `register_viewport_tool_mode`；当前合同已经是 factory-backed `SceneModeRegistration` 与 `register_scene_mode`。AI Editor 仍未进入 `zircon_plugins/first_party_editor_catalog` 的 feature、dependency或registration map，而 `zircon_app` 没有第二条首方Editor装配路径。换言之，当前默认产品不会加载AI Editor；若单独把crate纳入编译，它又会先遇到静态API漂移。

即使绕过上述阻断，Import、Open、Validate、Compile和Toggle Overlay五个可见operation也全部只有descriptor，没有 `OperationCommandFactory`。共享host对这种调用明确返回 `MissingFactory`。Behavior Tree ZUI的palette Table没有provider，graph和inspector仍是业务 `Space`，Blackboard只是无provider的Table；Perception Debug只有无provider的agent Table。`GraphEditorDescriptor`、`GraphNodePaletteDescriptor`、mirror getter和overlay controller在非测试产品代码中都没有AI controller消费。

调试链的语义也不足。`BtNodeResultEvent`由每个agent tick report中唯一的 `active_node` 和最终status生成，每tick至多一条；snapshot prune又只保留snapshot中的单个active node。它不能表达active path、并行附加节点、decorator abort、service/task开始结束、Blackboard delta、EQS item score、program/schema generation或可靠时间轴。mirror虽然有session、sequence和World边界，却仍使用裸 `u64` entity，没有reader lease、bounded history、字段订阅、丢失区间或source revision currentness；catalog调用 `plugin_registration()` 后也不会保留可从产品controller访问的 `AiEditorPlugin`实例。

默认Workbench仍构成第二套静态AI authority。Behavior workspace固定展示 `BT_Enemy`、`BB_Enemy`、`AIController_Enemy`、Selector/Sequence/Decorator和runtime trace；Perception workspace固定展示agent、FOV、hearing和timestamp；Save/Compile/Diff/Validate/Simulate由 `module_command_feedback.rs`写入固定成功结果，包括 `selector branch is reachable` 与 `AI_Guard_01 simulation tick 00:12.4`。当前在途修改只移除了Validate按钮初始selected/checked状态，没有接入真实source、compiler、runtime或mirror。

因此，Editor20登记的5项父P0仍为 **5 Open / 0 Partial / 0 Closed**。本轮不重复增加P0计数；新增60项canonical P1，判定为 **48 Open / 12 Partial**，新增12项P2均为Open；32项验收门为 **26 Fail / 6 Partial**。目标不是继续填空白控件，而是形成：

```text
versioned AI source assets
  -> transactional authoring documents with stable identities
  -> shared deterministic semantic compiler
  -> atomic AI build-set publication + diagnostics/source map
  -> per-World Runtime152 execution and bounded trace
  -> generation-qualified Editor debug session
  -> real graph/blackboard/query/overlay product controllers
```

## 2. 当前物理范围与证据边界

### 2.1 冻结范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 当前证据 |
|---|---:|---|
| AI Editor package | **12 / 2,396 / 2,223 / 83,869 / 18 / 4** | Cargo、两份ZUI、capability/ID、registration、mirror、overlay及新增allocation tests；fingerprint `4e9ff1e2df2744cca8f9ee8cb64f0794c3644fc2dc45d6584d8375a9f33e1d9f` |
| catalog与产品装配 | **6 / 497 / 435 / 18,194 / 8 / 0** | AI manifest、first-party Editor catalog和App委托；fingerprint `514e8008ff1d92c39293b86e61f33fc817209207bddac337b6fb499d360cdef4` |
| 共享Editor基础 | **25 / 6,563 / 5,979 / 230,667 / 10 / 0** | extension/contribution、factory、asset toolkit、transaction、job、event host、scene mode与overlay provider；fingerprint `8f63690270c94003bbf2377fe4f15cb41382d1a524eb55b6d628860fba4f6c1a` |
| 静态AI Workbench | **4 / 1,212 / 1,134 / 58,308 / 0 / 0** | 两份workspace、generated bottom route与fixed command feedback；fingerprint `868616e63779f7631b44f5d892b87f61ee4aa8b0bf4d80f29317328292369beb` |
| Runtime debug合同 | **6 / 865 / 798 / 34,998 / 0 / 0** | snapshot、tick、event producer与runtime registration；fingerprint `290b986d6efb56ecc79dbbad49997225e23e8db2d45ad551c5d3df80d78ad4c6` |
| Unreal AI Editor参考 | **21 / 13,761 / 11,811 / 472,330 / 0 / 0** | BT/Blackboard资产与图、debugger/diff、EQS、GameplayDebugger、StateTree、SmartObject；fingerprint `de11967eda6fed7f2d8d36fdba6ca784d5ea4e1eb6ac2ffb97a4086e580d0957` |
| Fyrox/Godot/Bevy/Unity Graphics参考 | **13 / 18,632 / 15,909 / 716,665 / 2 / 0** | plugin/command/undo/debug/asset/graph window与Blackboard controller；fingerprint `7a8437c36d7f7f1f5d29b8e6d36bb4747c6b04b09169cea95eb4998385f4ff5a` |

Zircon selected合计53个文件、11,533行、10,569非空行、426,036 bytes、36项test declaration和4项ignored；参考选择集合计34个文件、32,393行、27,720非空行、1,188,995 bytes及2项test declaration。行数和bytes均读取物理文件，不读取Git索引。

### 2.2 Currentness

冻结时selected范围内有17个modified文件和2个untracked allocation test文件，均属于用户或其他Session的在途工作；本轮没有覆盖、格式化或回退这些源码。与Editor20旧快照相比，AI Editor由10文件/1,654行增至12文件/2,396行，测试由9项增至18项，并出现overlay容量预算和mirror lookup allocation证据；这是真实进展，但不关闭任何父P0。

由于共享Editor与AI文件仍在变化，任何实施Session开始前必须重新冻结：catalog、`overlay.rs`、`runtime_mirror.rs`、`plugin.rs`、extension materialization、runtime producer、两份AI Workbench及其指纹。不得以本报告指纹覆盖后续working-tree事实。

### 2.3 动态证据边界

本轮按用户要求只做review，没有运行Cargo、Editor/App、PIE、asset import/cook、save/reopen、runtime trace、viewport overlay、EQS、StateTree、Smart Object、fault/scale/soak/profile或跨引擎benchmark。18项AI Editor test直接构造plugin、registry、mirror或overlay builder；4项release-only allocation test处于ignored。它们不能证明AI Editor crate能通过当前API编译，也不能证明first-party catalog/App会装配、operation会执行或surface会显示真实数据。

## 3. 当前实现的纵向事实

### 3.1 Package、catalog与lifecycle

1. `plugin.rs`登记Behavior Tree asset type、`.btree.toml` importer、toolkit、graph editor、18节点palette、4个Behavior命令和Perception surface/command，并声明两类runtime event consumer；registration信息比空插件完整。
2. `first_party_editor_catalog/Cargo.toml`只有Navigation/Neural optional dependency与feature；`catalog.rs`只有对应两个分支。App只委托该catalog，因此选择Runtime AI不会获得AI Editor registration。
3. retained host直接消费 `EditorPluginRegistrationReport.extensions.into_contribution_batch()`，这条初次注册路径能够保留factory/provider；AI自身没有登记它们。
4. plugin manager的 `build_editor_extensions` 当前重建active snapshot时只复制descriptor、scene mode、graph和pending command，遗漏operation factory、viewport overlay provider、template pane data source和settings page等factory-backed贡献。即使AI日后补齐，manager snapshot/reload路径也会与直接注册路径不一致。
5. manifest和M5 validation文档只能证明意图与声明，不能替代catalog选择、host生命周期、disable/unload、state drain和产品projection证据。

### 3.2 Source、asset、document与graph

1. Behavior Tree只有asset type ID、扩展名和toolkit descriptor，没有Editor-owned versioned source DTO、schema migration、unknown-field policy、source revision或dependency graph。
2. Blackboard没有独立asset type、factory、importer、toolkit、inheritance或key redirect，只是Behavior ZUI中的一张Table。
3. Perception Config、EQS、StateTree、Smart Object没有Editor resource kind、source schema、create/import/reimport、thumbnail、toolkit或cook role。
4. `GraphEditorDescriptor`和`GraphNodePaletteDescriptor`只是注册元数据。仓内没有AI graph document、stable node/edge/auxiliary ID、graph controller、canvas projection、selection、drag/connect、copy/paste、search、diff或source map。
5. Behavior ZUI有86行，palette Table无provider，graph canvas和inspector为 `Space`，Blackboard Table无provider，事件为0；Perception ZUI有30行，只包含无provider的agent Table，事件为0。
6. generic asset type/toolkit、transaction、operation factory和background job是真实可复用底座，但AI没有消费它们形成dirty/save/autosave/recovery/conflict/undo/redo/compile/publish链。

### 3.3 Operation、compiler与publication

1. AI登记Import/Open/Validate/Compile/Toggle Overlay五个operation descriptor，AI package中 `OperationCommandFactory` 和 `ViewportOverlayProviderRegistration` 均为0命中。
2. shared host在找不到factory时明确产生 `OperationCommandFactoryError::MissingFactory`，所以这些菜单/按钮不是“稍后补业务”，而是当前确定失败的产品入口。
3. importer descriptor不读取bytes，不持有settings/provenance/source hash，不调用Runtime152 compiler，也没有reimport、cancel、dependency或diagnostic contract。
4. Editor没有AI build-set、source map、diagnostic identity、prepared publication、last-known-good、atomic multi-asset commit或cook parity。
5. palette取自runtime standard catalog是正确方向，但节点只有展示/类别/ID投影，没有authoring parameter schema、pin/attachment rules、version、owner generation或第三方节点reload语义。

### 3.4 PIE mirror与debugger

1. `AiPieMirror`有play-session和delivery-sequence fence，并以 `(world u64, entity u64)` 保存frame；world snapshot替换时会移除消失agent，raw entity lookup遇到跨World歧义会返回None。这些应保留。
2. identity仍是裸u64，不含World/Entity generation、program build、Blackboard schema、source revision或debug subscription generation；旧对象ID重用与hot reload无法可靠判定currentness。
3. `AiBtNodeResultMirror`当前改为 `(World, Entity) -> BTreeMap<NodeId, Event>`，查询不再为key构造临时String；这是性能局部改进。
4. Runtime producer每agent tick最多发布唯一active node，snapshot也只有唯一active node；prune因此会删除parallel/additional node。当前结构不能成为完整Behavior Tree debugger timeline。
5. mirror只保留latest map，没有bounded delta/history、gap receipt、reader lease、field mask、sampling、retention tier或slow-reader处理。
6. mirror Arc隐藏在registration consumer state中；`AiEditorPlugin::pie_mirror()`与`node_result_mirror()`要求原始plugin实例。catalog只返回registration report后没有产品state access handle，surface/controller也没有订阅者。
7. 没有PIE instance selector、pause/step/step-back、breakpoint、active path、search/abort range、live Blackboard value、EQS detail或远端debug授权。

### 3.5 Perception overlay与高级AI工具

1. overlay builder能生成agent sphere、24段hearing circle、受限sight cone和stimulus line/sphere，并过滤非有限position/radius；当前还会预计算容量。
2. 它为容量计算和实际构建各扫描一次World agent，所有agent共用传入owner；没有selected agent、sense/team/filter、frustum/distance culling、LOD、primitive/bytes/time预算、overflow/quality或缓存复用。
3. `AiPerceptionOverlayController`只有generic sink、enabled/options和publish；没有注册为viewport provider，也没有scene mode factory、session/view lifecycle、capture loss或unload处理。
4. EQS没有graph、generator/context/test details、query preview、score normalization、failed-item解释、profiler或runtime request关联。
5. StateTree没有state hierarchy、transition/condition/binding editor、compiler log、simulation、diff或rewind debugger。
6. Smart Object没有definition/slot source、factory、viewport、component visualizer、view model、validation或World Partition collection workflow。

### 3.6 Workbench与测试真实性

1. 两份默认AI Workbench共414行、48个control和26条event/route，但所有业务数据来自ZUI文字或feedback match分支，不来自AI资产、document、compiler、runtime或mirror。
2. generated bottom panel只路由固定Perception query/validation/compile及Behavior trace/breakpoint/validation条目，没有typed item identity、generation或action receipt。
3. plugin ZUI中的 `AiBehaviorTreePalette`、`AiBehaviorTreeGraphCanvas`、`AiBehaviorTreeInspector`、`AiBehaviorTreeBlackboard`、`AiPerceptionDebugAgents` 在非测试产品代码中没有consumer。
4. `pie_mirror`、`node_result_mirror`、overlay builder/controller在AI package外没有产品caller。
5. focused tests覆盖descriptor、palette、session/stale/world隔离、snapshot replacement、overlay geometry和allocation；缺compile/catalog/App启动、factory dispatch、document transaction、save/reopen、cook parity、debug controller、provider lifecycle、fault/scale/soak。

## 4. 参考引擎差异与采用边界

| 参考 | 已核对的工程事实 | Zircon必须吸收的边界 |
|---|---|---|
| Unreal BehaviorTreeEditor | 独立BT/Blackboard asset definition与factory；graph/schema/subnode；transaction/undo；asset update/compile；Find、revision diff；PIE instance、active path/additional nodes、history与breakpoint | 作为AI authoring/debug主参考；吸收职责拆分、source/compiler映射、调试状态机，不复制Slate/UObject实现 |
| Unreal EnvironmentQueryEditor / GameplayDebugger | EQS graph、test details、run/profiler统计；BT/Perception/EQS远端runtime projection | 建立query identity、item score/失败原因、bounded transport和远端权限；不能用overlay快照冒充query profiler |
| Unreal StateTree Editor | 独立大规模compiler、binding/condition/transition验证、compile log、async diff、timeline/instance debugger | StateTree必须是独立source/compiler/toolkit，不塞入Behavior Tree通用graph的几个node类型 |
| Unreal SmartObjects Editor | definition factory、asset editor/view model、viewport/slot details和visualizer | Smart Object拥有独立definition/slot/World integration；不把它降为Blackboard key或场景标签 |
| Unity Graphics ShaderGraph | persistent `GraphData`、window/save/undo、graph view/controller/view-model、Blackboard controller | 仅作共享graph authoring基础参考；AI语义仍由Runtime152共享compiler定义 |
| Godot | Animation BlendTree用GraphEdit与UndoRedo；EditorDebugger、Inspector和UndoRedo Manager职责分离 | 借鉴轻量graph/transaction/debugger边界，不把Animation BlendTree当AI Behavior Tree替代品 |
| Fyrox | Editor plugin lifecycle与scene graph reversible command | 保留插件隔离和command/inverse边界；其轻量behavior工具不足以降低Unreal级AI产品门槛 |
| Bevy | typed AssetLoader、AssetId、dependency与asset event | 用作typed asset加载/身份下限；Bevy没有同级首方AI Editor，不能作为缺失authoring的合理化依据 |

Unreal在本域是主参考，Unity Graphics/Godot/Fyrox/Bevy只补共享Editor基础。Zircon的Rust ownership、generation identity、bounded transport、prepared publication和跨进程PIE需要按自身Runtime152架构实现，不能机械复制UObject指针或Slate UI。

## 5. 父P0当前重判

| Canonical ID | 状态 | 当前证据与硬切要求 |
|---|---|---|
| `AI-ED-P0-001` | Open | `ViewportToolModeDescriptor`/`register_viewport_tool_mode`在当前仓内无定义；改为`SceneModeRegistration` +真实factory/provider，并建立required compile lane。 |
| `AI-ED-P0-002` | Open | first-party Editor catalog只有Navigation/Neural，App只委托该catalog；同里程碑接入AI feature/dependency/registration/selection并验证disable/unload。 |
| `AI-ED-P0-003` | Open | 5个可见operation均无factory，host确定返回`MissingFactory`；补typed factory/receipt，未实现命令必须Unavailable且不可展示成功。 |
| `AI-ED-P0-004` | Open | graph/Blackboard/Perception/mirror/overlay均无产品controller/provider；descriptor、Table、Space和builder不能计为产品。 |
| `AI-ED-P0-005` | Open | 默认Workbench继续固定AI数据与成功反馈；先标sample/Unavailable，最终硬切到真实toolkit与runtime projection。 |

## 6. Canonical P1 重判

| Canonical ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| `AI-ED-P1-001` | Open | Behavior Tree无versioned source DTO、schema/unknown policy、stable asset identity和migration。 |
| `AI-ED-P1-002` | Partial | 已有asset type/importer/toolkit descriptor；缺bytes解析、source revision、provenance、create/reimport与真实toolkit factory。 |
| `AI-ED-P1-003` | Open | Blackboard无独立asset type、factory、importer、toolkit、inheritance和redirect。 |
| `AI-ED-P1-004` | Open | Perception Config无source asset、sense config schema、platform override、validation和cook role。 |
| `AI-ED-P1-005` | Open | EQS无source asset、query option/generator/context/test schema、factory和toolkit。 |
| `AI-ED-P1-006` | Open | StateTree无source asset、state/transition/condition/binding schema、factory和toolkit。 |
| `AI-ED-P1-007` | Open | Smart Object Definition/Slot无source asset、scene reference、factory、toolkit和collection artifact。 |
| `AI-ED-P1-008` | Open | AI各source间无typed dependency graph、cycle policy、redirect、missing/LKG诊断。 |
| `AI-ED-P1-009` | Open | 无create template、命名/路径冲突、目录事务与rollback。 |
| `AI-ED-P1-010` | Open | 无import/reimport settings、source hash、cancel、dry-run、dependency与原子提交。 |
| `AI-ED-P1-011` | Partial | GraphEditor/18-node palette descriptor存在；无document/controller/canvas消费，也无authoring parameter schema。 |
| `AI-ED-P1-012` | Open | 无stable node/edge/pin/auxiliary/service/decorator identity和tombstone/redirect。 |
| `AI-ED-P1-013` | Open | 无BT root/composite/task/decorator/service/subtree attachment与连线规则验证。 |
| `AI-ED-P1-014` | Open | palette不含参数/pin/category version、owner generation、capability和第三方reload语义。 |
| `AI-ED-P1-015` | Open | 无graph selection、marquee、move/connect/delete、focus、multi-select和keyboard routing。 |
| `AI-ED-P1-016` | Open | 无copy/paste/duplicate、stable remap、external reference repair与跨文档策略。 |
| `AI-ED-P1-017` | Open | 无Find、semantic search、jump-to-node、reference search和diagnostic navigation。 |
| `AI-ED-P1-018` | Open | 无source/revision diff、node correspondence、conflict projection和merge policy。 |
| `AI-ED-P1-019` | Open | Blackboard无key type/default/category/inheritance/override/rename-impact编辑。 |
| `AI-ED-P1-020` | Open | Inspector仍是`Space`，无typed property、conditional field、multi-edit、unit和validation。 |
| `AI-ED-P1-021` | Partial | shared transaction/history/journal基础存在；AI没有command、inverse delta、merge scope或dirty wiring。 |
| `AI-ED-P1-022` | Open | 无save token、atomic write、source CAS、external-change conflict和last-known-good。 |
| `AI-ED-P1-023` | Open | 无autosave/recovery/session lock/readonly与crash reopen。 |
| `AI-ED-P1-024` | Partial | shared operation factory及`MissingFactory`失败路径真实存在；AI五个operation没有factory/payload/receipt。 |
| `AI-ED-P1-025` | Open | Validate没有调用共享Runtime152 semantic compiler，也没有source-bound diagnostics。 |
| `AI-ED-P1-026` | Open | Compile没有prepared build set、dependency digest、source map或deterministic bytes。 |
| `AI-ED-P1-027` | Open | 多BT/BB/Perception/EQS依赖不能形成单一atomic AI build-set publication。 |
| `AI-ED-P1-028` | Partial | shared background job/admission基础存在；AI无cancel/progress/deadline/owner drain/commit executor。 |
| `AI-ED-P1-029` | Open | Editor preview、PIE、cook/shipping未证明使用同一compiler/artifact。 |
| `AI-ED-P1-030` | Open | 无compile generation/LKG/install acknowledgement/currentness和stale result discard。 |
| `AI-ED-P1-031` | Partial | typed runtime event consumer、manifest与host session基础存在；未接产品reader/controller。 |
| `AI-ED-P1-032` | Partial | mirror具备session/sequence/World隔离与snapshot replacement；identity/generation/currentness仍不足。 |
| `AI-ED-P1-033` | Open | BtNodeResult每tick唯一active node，不能表示active path、parallel nodes和完整node lifecycle。 |
| `AI-ED-P1-034` | Open | 无decorator observer/abort range、service/task begin/end/cancel/fault trace。 |
| `AI-ED-P1-035` | Open | 无Blackboard delta/watch、value provenance、timestamp、schema generation和敏感值策略。 |
| `AI-ED-P1-036` | Open | 无bounded trace journal、gap/overflow receipt、retention tier、reader lease和slow-reader policy。 |
| `AI-ED-P1-037` | Open | 无PIE instance selector、pause/step/step-back、历史游标和session切换状态机。 |
| `AI-ED-P1-038` | Open | 无breakpoint create/remove/enable/disable、condition、hit count和runtime acknowledgement。 |
| `AI-ED-P1-039` | Open | 无program/schema/source revision到graph node的generation-qualified source map。 |
| `AI-ED-P1-040` | Partial | node mirror按World/Entity分层并支持无分配borrowed lookup；producer/prune仍只保留单active node。 |
| `AI-ED-P1-041` | Open | catalog返回registration后无稳定debug state access handle，plugin实例getter无法被产品controller使用。 |
| `AI-ED-P1-042` | Open | 无远端/多进程agent debug、capability授权、privacy/redaction、disconnect/reconnect和clock calibration。 |
| `AI-ED-P1-043` | Partial | overlay有finite过滤、精确预分配和基础FOV/hearing/stimulus几何；仍双扫描且无预算/裁剪/quality。 |
| `AI-ED-P1-044` | Partial | shared scene mode factory与viewport provider lifecycle存在；AI仍调用旧API且未注册provider。 |
| `AI-ED-P1-045` | Open | overlay无selected agent/sense/team/affiliation/filter、show flag和per-view state。 |
| `AI-ED-P1-046` | Open | overlay无frustum/distance/LOD、primitive/bytes/time budget、overflow和cache reuse。 |
| `AI-ED-P1-047` | Open | Perception Debug无agent列表provider、selection、stimulus history、forget/age/quality/failure原因。 |
| `AI-ED-P1-048` | Open | EQS无run request、preview world、item shape/score、failed test解释和query profiler。 |
| `AI-ED-P1-049` | Open | StateTree无hierarchy/transition/condition/binding editor、compiler log、simulation/diff/debugger。 |
| `AI-ED-P1-050` | Open | Smart Object无slot/details/viewport visualizer、claim/use runtime projection和World collection workflow。 |
| `AI-ED-P1-051` | Open | 无AI preview sandbox的isolated World、deterministic seed/time/input、reset与mutation boundary。 |
| `AI-ED-P1-052` | Open | 无Navigation task/query关联、path/cost/result/abort trace和Nav Editor跳转。 |
| `AI-ED-P1-053` | Open | 无Animation/Script/Gameplay task handle状态、跨域diagnostic与owner revoke可视化。 |
| `AI-ED-P1-054` | Open | 无network authority、server/client observation、prediction/replay/save currentness UI。 |
| `AI-ED-P1-055` | Partial | manifest声明capability/event/editor artifact；默认catalog、产品lifecycle和acceptance尚未兑现。 |
| `AI-ED-P1-056` | Open | plugin manager active snapshot遗漏factory/provider等贡献，reload/read-model与直接注册路径不一致。 |
| `AI-ED-P1-057` | Open | Workbench两份AI workspace和generated bottom panel仍是独立静态authority，未硬切真实toolkit。 |
| `AI-ED-P1-058` | Partial | focused tests覆盖descriptor、mirror、world/stale、geometry和allocation；缺产品/故障/规模矩阵。 |
| `AI-ED-P1-059` | Open | 无1/10/100文档、1K/10K node、1K agent、trace/overlay/update延迟与内存预算资格。 |
| `AI-ED-P1-060` | Open | 无编译失败、source损坏、plugin unload、session churn、queue overflow、panic/device loss与长期soak。 |

## 7. Canonical P2 重判

| Canonical ID | 状态 | 目标 |
|---|---|---|
| `AI-ED-P2-001` | Open | 具备semantic diff/merge、review comment和stable identity的多人AI资产协作。 |
| `AI-ED-P2-002` | Open | 大型BT/StateTree的虚拟化graph、分层LOD、局部布局和增量编译。 |
| `AI-ED-P2-003` | Open | 运行时trace与source revision的时间旅行、fork与deterministic replay。 |
| `AI-ED-P2-004` | Open | 分布式/远端server AI观察、权限、加密、redaction和多session聚合。 |
| `AI-ED-P2-005` | Open | EQS离线数据集、回归比较、统计分布和query optimization advisor。 |
| `AI-ED-P2-006` | Open | Perception热力图、occlusion/visibility解释、历史衰减和跨agent聚合。 |
| `AI-ED-P2-007` | Open | StateTree/BT/EQS/Smart Object跨资产semantic refactor与安全rename。 |
| `AI-ED-P2-008` | Open | 第三方AI node/schema/editor extension的sandbox、version negotiation与hot reload。 |
| `AI-ED-P2-009` | Open | 基于production trace的profile-guided tree/query优化建议，必须可解释且不自动改真值。 |
| `AI-ED-P2-010` | Open | 大世界分区AI authoring、streaming preview、World Partition Smart Object collection和跨cell诊断。 |
| `AI-ED-P2-011` | Open | 同AI source/scenario/seed下与Unreal的行为、debug可见性和性能竞争基准。 |
| `AI-ED-P2-012` | Open | 分布式fault/scale/soak farm，覆盖compile/reload/session/network/save/replay与Editor交互分位。 |

## 8. 验收门当前状态

| Gate | 状态 | 当前判定 |
|---|---|---|
| `AI-ED-G01` | Fail | AI Editor当前静态引用已删除Viewport Tool API，没有required compile证据。 |
| `AI-ED-G02` | Fail | Project选择AI时first-party Editor catalog/App不返回AI registration。 |
| `AI-ED-G03` | Partial | asset/import/toolkit descriptor存在；create/import/open/reimport/save/reopen产品链未达标。 |
| `AI-ED-G04` | Fail | 五个operation均无factory，真实host会返回`MissingFactory`。 |
| `AI-ED-G05` | Fail | BT/Blackboard/Perception/EQS/StateTree/Smart Object source schema与migration未建立。 |
| `AI-ED-G06` | Fail | AI document dirty/transaction/undo/redo/autosave/recovery未建立。 |
| `AI-ED-G07` | Partial | graph/palette descriptor存在；真实graph controller/canvas/selection/edit/save不存在。 |
| `AI-ED-G08` | Fail | Blackboard独立资产、inheritance、key rename影响与live value未闭合。 |
| `AI-ED-G09` | Fail | Validate/Compile未调用共享semantic compiler，也无deterministic artifact/source map。 |
| `AI-ED-G10` | Fail | 多资产build set不能prepared/atomic publish或回退LKG。 |
| `AI-ED-G11` | Fail | Editor preview/PIE/cook/shipping compiler与artifact parity无证据。 |
| `AI-ED-G12` | Partial | shared command/factory/transaction/job基础存在；AI未消费。 |
| `AI-ED-G13` | Fail | plugin manager snapshot与direct registration对factory/provider贡献不一致。 |
| `AI-ED-G14` | Fail | AI plugin disable/unload/reload没有document/reader/job/provider drain与terminal receipt。 |
| `AI-ED-G15` | Fail | Behavior Tree surface仍含业务`Space`和无provider Table。 |
| `AI-ED-G16` | Fail | Perception surface agent Table无provider/selection/history。 |
| `AI-ED-G17` | Partial | typed event、session/sequence/World mirror真实存在；generation/source currentness不足。 |
| `AI-ED-G18` | Fail | trace无法表达active path、parallel nodes、abort、service/task和Blackboard delta。 |
| `AI-ED-G19` | Fail | breakpoint、step、step-back、instance selector和timeline未建立。 |
| `AI-ED-G20` | Fail | runtime/editor source-map及hot reload generation handoff不存在。 |
| `AI-ED-G21` | Fail | trace无bounded retention、gap/overflow、reader lease和slow-reader资格。 |
| `AI-ED-G22` | Fail | 远端/多进程debug授权、clock、disconnect/reconnect未建立。 |
| `AI-ED-G23` | Partial | overlay geometry、finite过滤、预分配和shared provider substrate存在；AI provider/lifecycle/budget缺失。 |
| `AI-ED-G24` | Fail | EQS graph/run/item score/profiler不存在。 |
| `AI-ED-G25` | Fail | StateTree compiler/editor/diff/simulation/debugger不存在。 |
| `AI-ED-G26` | Fail | Smart Object definition/slot/editor/visualizer/collection不存在。 |
| `AI-ED-G27` | Fail | static Workbench仍显示固定AI资产、trace与成功反馈。 |
| `AI-ED-G28` | Fail | App/Editor/PIE端到端产品lane不存在。 |
| `AI-ED-G29` | Fail | compile/source/plugin/session/overflow/device fault matrix不存在。 |
| `AI-ED-G30` | Partial | 18项focused test和4项ignored allocation test存在；均未覆盖产品装配与主要生命周期。 |
| `AI-ED-G31` | Fail | 1K/10K node、1K agent、trace/overlay P50/P95/P99及内存预算无证据。 |
| `AI-ED-G32` | Fail | 长时间PIE/reload/open-close/network/save/replay/overlay soak与Unreal竞争基准不存在。 |

## 9. 分层重构顺序

### M0：编译、catalog与truthfulness硬切

用当前 `SceneModeRegistration`/provider合同替换旧Viewport Tool API；修复公开ID导入；加入AI Editor required compile lane和first-party catalog feature/dependency/registration/App测试。五个无factory operation及两份静态Workbench先统一投影Unavailable/sample，禁止继续显示成功结果。同步修复plugin manager materialization遗漏factory/provider的问题，保证初次注册、snapshot、reload只有一套贡献语义。

### M1：Source Asset与依赖合同

定义Behavior Tree、Blackboard、Perception Config、EQS、StateTree、Smart Object Definition/Slot的versioned source、stable identity、unknown policy、migration、typed dependency、create/import/reimport、thumbnail/toolkit/cook role。与Runtime152共同确定共享schema，禁止Editor和Runtime各自解析一套近似TOML。

### M2：Transactional Document与Graph基础

建立immutable document revision、stable node/edge/pin/auxiliary identity、selection/controller/canvas/Inspector/Blackboard、typed command/inverse delta、dirty/save/autosave/recovery/conflict、copy/paste/search/diff。复用共享Editor transaction和graph基础，不在AI插件中再造简化undo栈。

### M3：Shared Compiler与Atomic Build Set

Editor Validate/Compile、PIE、cook和shipping统一调用Runtime152 semantic compiler；产出deterministic AI build set、dependency digest、source map和diagnostics。job只负责prepare，owner thread执行CAS/atomic publication并回传generation-qualified receipt；失败保留LKG。

### M4：真实AI Toolkit与产品controller

为BT/Blackboard/Perception/EQS建立factory-backed toolkit、pane data source和command controller，把ZUI的Table/Space替换为真实projection。first-party catalog返回可访问的typed service/state handle，Workbench路由到同一toolkit，不保留第二套AI状态。

### M5：Bounded Runtime Debug Transport

Runtime152发布active path、parallel nodes、node/service/task/abort、Blackboard delta、Perception/EQS事件的bounded journal；identity含session/world/entity/program/schema/source generation。实现reader lease、field mask、gap/overflow/quality、retention tier、disconnect和hot reload fence。

### M6：Behavior Debugger与Perception Overlay

完成instance selector、pause/step/history、breakpoint、search/source jump、live Blackboard和runtime description；AI overlay通过真实viewport provider注册，支持selected agent/sense/team/filter、frustum/distance LOD、primitive/bytes/time budget、cache与quality state。

### M7：EQS、StateTree与Smart Object产品

分别建立EQS graph/run/profiler、StateTree hierarchy/binding/compiler/diff/simulation/debugger、Smart Object definition/slot/view model/viewport/visualizer/World collection。它们复用source/compiler/document/debug基础，但保持独立领域owner，不合并成万能AI graph。

### M8：故障、规模、soak与竞争资格

覆盖source损坏、migration、compile cancel、publication失败、plugin reload、session churn、queue overflow、remote disconnect和device loss；建立1/10/100文档、1K/10K node、1K agent、trace/overlay P50/P95/P99与长期soak。最后以同source/scenario/seed与Unreal对比authoring完整性、debug可见性和性能，未达到门槛不得标Stable/Complete。

## 10. 禁止的临时修补

1. 禁止只把 `ViewportToolModeDescriptor` 改名为 `SceneModeDescriptor`，却不提供scene mode factory和overlay provider。
2. 禁止只给first-party catalog加一个AI分支，而不验证compile、selection、disable/unload和产品controller。
3. 禁止给operation注册空factory或固定success receipt来消除 `MissingFactory`。
4. 禁止把descriptor、palette条目数、空Table或`Space`替换成几行静态row后宣称graph已完成。
5. 禁止Editor复制Runtime152验证/编译逻辑，或让Preview与shipping消费不同artifact。
6. 禁止以裸World/Entity u64和单active node作为长期debug协议。
7. 禁止无限trace、每帧全量snapshot、无reader lease缓存或UI直接锁Runtime manager。
8. 禁止以allocation下降替代candidate、primitive、bytes、time与slow-reader预算。
9. 禁止把EQS、StateTree、Smart Object压成Behavior Tree palette中的几个伪节点。
10. 禁止保留固定 `BT_Enemy`、agent、trace、Validate/Simulate/Compile成功文案作为默认产品。
11. 禁止用M5 manifest、unit test数量或直接构造plugin的测试替代App/Editor/PIE产品证据。
12. 禁止在hard cutover后保留旧Workbench、旧API、compat module、fallback compiler或双写authority。

## 11. 本轮产出与后续验证

本轮仅新增current-source review并更新索引/覆盖矩阵，没有修改production Runtime、Editor、App、plugin或tests。tooling按用户当前目标排除；没有实时跟踪或等待协调器状态。

实施必须从M0开始，并依赖Runtime152对AI source/compiler/world runtime/task/query/debug contract的同步收敛。每个里程碑完成前重新冻结physical files、dirty owners、fingerprint与动态证据；Closed/Pass必须同时具备source-to-product、failure、scale、currentness和删除旧authority的证据。
