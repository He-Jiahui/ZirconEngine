---
title: Editor Authoring Transaction、Command、History、Undo/Redo、Merge Group、Savepoint、Dirty、Document Scope、Object Generation 与 Async Operation 当前工作树复核
category: zircon_editor
report_id: Editor256
review_date: 2026-08-30
baseline_head: cc5cadbd597c3707954ebd6109fad0fd5643a152
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/asset/dirty
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/document
  - zircon_editor/src/core/recovery/document_journal
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/host
  - zircon_editor/src/ui/workbench
  - zircon_plugins/navigation/editor/src/operation_command
  - zircon_plugins/neural/editor/src/plugin.rs
  - zircon_runtime/src/scene/mod.rs
tests:
  - zircon_editor/src/tests/editing
  - zircon_editor/src/tests/ui
  - zircon_plugins/navigation/editor/src/tests/operation_command.rs
  - zircon_plugins/neural/editor/src/tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/225-editor-document-transaction-save-autosave-recovery-authoring-io-current-source-review.md
  - docs/plans/optimize/zircon_editor/254-editor-scene-viewport-input-picking-selection-highlight-gizmo-transaction-current-working-tree-review.md
  - docs/plans/mvp/05-f4-basic-authoring.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/03/2026-08-24-scene-document-history-routing-hardcut.md
  - docs/plans/zircon_editor/editor/03/2026-08-28-transaction-history-data-source.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-22-history-dirty-batch-generation-contract-missing.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-22-transaction-selection-history-wide-snapshot.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-23-pending-edit-retention-contract-missing.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-29-transaction-journal-contract-unimplemented.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-30-fallible-exclusive-transition-context-update.md
  - docs/plans/zircon_editor/editor/03/failure-2026-08-13-detached-entity-batch-editor-inverse-delta.md
  - docs/plans/zircon_editor/editor/03/failure-2026-08-19-gizmo-world-space-interactive-transaction.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ITransaction.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/ScopedTransaction.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/TransBuffer.h
  - dev/UnrealEngine/Engine/Source/Editor/UndoHistoryEditor/Private/Widgets/SUndoHistory.cpp
  - dev/godot/core/object/undo_redo.h
  - dev/godot/core/object/undo_redo.cpp
  - dev/godot/editor/editor_undo_redo_manager.h
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/tests/core/object/test_undo_redo.cpp
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/CoreEditorUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
doc_type: current-working-tree-review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
review_status: current_working_tree_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Authoring Transaction / Command History 当前工作树复核（Editor256）

## 1. 当前裁决

Zircon的事务内核已经不是只有`Vec<Box<dyn Command>>`的临时样例：当前源码具备fallible apply/revert、反向补偿、fault state、Document/Play history route、save token CAS、dirty增量、paged history projection、typed selection snapshot、批量gizmo command、versioned journal codec和durable framing。这些都是应保留的工程底座。

但它仍不能作为“接近或优于Unreal的工程级authoring transaction authority”验收。当前最关键的断点不是缺一个Undo按钮，而是以下系统不变量尚未成立：

1. `history_status/history_details/is_dirty/capture_save_token`等观察API会先`flush_operation_group()`；刷新历史UI、构建普通Editor snapshot或读取dirty，可能提交正在进行的交互。
2. `EngineState`仍只有单`active`栈、单`operation_group`和单`operation`门；`TransactionScope`虽然`!Send`，engine并未记录owner thread/executor/session，另一线程可被误判为同history nested scope。
3. `HistoryContextId::Document(DocumentId)`没有durable document key、document session generation或world generation；文档关闭会detach toolkit/animation state，却没有同步退休该Document history。
4. Runtime `EntityId/NodeId`仍是裸`u64`；多数Scene command没有entity/component/property generation和current-value precondition，旧记录无法严格证明仍指向原对象。
5. UI Asset仍有第二套无界Undo stack，而且先移动undo/redo cursor再执行fallible product replay，失败可分裂history与document状态。
6. Navigation和Neural operation仍把poll、转换、文件I/O及外部副作用放进同步command apply/revert，没有prepare/short commit、cancel/deadline、late-result隔离和输出CAS。
7. durable journal的格式底座存在，但production commit到append的线性化链明确尚未开放；history只按record count限制，未按retained bytes、resource cost、barrier或pin治理。
8. 当前工作树还有直接源码一致性阻塞：`operation_group.rs:79`调用`scope::ensure_single_gateway_history`，本轮全树静态检索只找到这一个调用，未找到定义。

本轮沿用Editor184的finding与gate编号，不重复增加portfolio唯一finding。当前状态保持：P1为 **8 Open / 3 Partial / 0 Closed**，P2为 **2 Open / 2 Partial / 0 Closed**，40项资格门为 **22 Fail / 17 Partial / 1 Pass**。状态未放宽，但当前产品调用链和风险证据比Editor184更明确。

## 2. 审查边界与证据等级

### 2.1 当前性

- Git HEAD记录为`cc5cadbd597c3707954ebd6109fad0fd5643a152`；工作树存在其他会话和用户的在途修改，本报告评的是2026-08-30可见工作树，不把HEAD当成完整源码快照。
- 逐层复核了`core/editing`的command、engine、history、scope、operation group、save token、dirty batch、journal codec/replayer/event；继续追到Scene/Inspector/Hierarchy/Gizmo、Animation、UI Asset、Document Close、History UI、Navigation和Neural生产调用。
- 测试证据覆盖engine history、scope、locking、recovery、operation group、dirty batch、journal codec、scene replay、durable journal及相邻产品测试；测试文件存在不等同于本轮已执行通过。
- 参考源码只用于提取架构不变量，不把不同语言、对象模型或编辑器产品直接照搬成Zircon API。
- Tooling按用户要求排除。本轮未运行Cargo、Editor GUI、真实文件故障、崩溃恢复、并发soak、规模压测或跨引擎benchmark。

### 2.2 证据等级

| 等级 | 本报告含义 |
|---|---|
| `Pass` | 当前生产路径与测试源码共同证明合同闭合；本轮唯一Pass仍是Animation持久authoring mutation进入统一Undo/Redo。 |
| `Partial` | 有可保留typed底座，但identity、lifecycle、失败语义或产品接入仍不完整。 |
| `Fail` | 关键合同缺失、绕过authority，或当前源码没有可验收实现。 |
| `Open` | finding尚未满足关闭条件；不能用“已有某个类型/测试”替代系统验收。 |

## 3. 当前实现拓扑

```text
Scene / Inspector / Hierarchy / Gizmo
  -> EditorState::execute_scene_commands
  -> Document history route
  -> EditorTransactionEngine

Animation document
  -> whole-document CAS command
  -> Document history route
  -> EditorTransactionEngine

UI Asset document
  -> UiAssetEditorUndoStack + UiAssetEditorCommandJournal
  -> private cursor / replay / dirty path                 [second authority]

Navigation / Neural operation
  -> OperationCommand
  -> Global history
  -> synchronous poll / conversion / filesystem effects  [long apply]

History UI / Editor data snapshot / dirty-save queries
  -> history_status / history_details / capture_save_token
  -> flush_operation_group                                [observation mutates]

EditorTransactionEngine
  -> one EngineState mutex
  -> one active stack + one operation group + one operation gate
  -> count-bounded HistoryStore
  -> optional journal serialization
  -> lifecycle event sink

DocumentJournalCoordinator
  -> document/path binding + framed/checksummed file
  -> append_for_test only                                 [production gap]
```

这个拓扑的核心问题是authority没有真正收敛：Scene和Animation进入了统一engine，但UI Asset仍在旁路；独立Document虽有不同history key，却共享一个执行门；观察面与mutation面没有分离；journal有格式但没有production publication owner。

## 4. 可保留的真实进展

### 4.1 History读取与dirty底座

- `HistoryStore`维护cursor、top、saved_top和saved reachability，Undo回到savepoint能够使dirty恢复false。
- save token带engine/history generation并使用`mark_saved_if_unchanged`做CAS，避免后台保存无条件覆盖较新的编辑位置。
- history status已经是O(1)摘要，details有cursor和最大128条分页限制；selection handle使用`Arc`，不再为每次历史查询复制完整selection。
- DirtyRegistry有bounded change journal和generation cursor，可做增量dirty snapshot。

这些改进解决了Editor184之前部分“全history复制”和save race问题，但读取API的flush副作用使“只读projection”合同仍未成立。

### 4.2 Scene与Gizmo

- Scene/Inspector/Hierarchy mutation已集中走`EditorState::execute_scene_commands`并解析Document history，而不是每个入口自行选择Global。
- Scene command保留gateway/world route，world replacement路径已有history清理和generation检查的局部合同。
- Editor254确认Gizmo使用单一批量interaction session：冻结selection roots、world/local transform和parent inverse，全量预检、失败补偿，并在gesture末端提交一个typed batch command。

这些是可保留底座；输入capture/terminal receipt与group token问题继续由Editor254/Editor255拥有，本报告只审其事务接口。

### 4.3 Animation

- Animation持久编辑使用`HistoryContextId::Document(document)`，command以`AnimationDocumentRevision`做whole-document CAS，focused Undo/Redo也走统一transaction engine。
- persistent authoring mutation的Apply/Undo/Redo链已经成立，因此G29保持唯一Pass。

剩余问题是每次编辑clone/swap完整asset并recompile，错误被压成`EditorError::UiAsset(String)`，history status通过`.ok()`静默隐藏busy/fault；文档关闭detach animation store但不退休transaction history。

### 4.4 Journal

- transaction journal有`$zircon`标识、transaction id、command type/schema v1，Scene command具稳定codec。
- durable文件格式有magic、sequence、record length、BLAKE3校验、1 MiB record上限、64 MiB文件上限和65536 record上限。
- `PreparedJournalRecord`先生成有界bytes与digest，replayer在begin前解码命令，避免边解码边部分执行。

但这些只能证明格式/测试底座。`document_journal/coordinator.rs:129-132`明确说明production durable publication在transaction engine拥有commit线性点的immutable capture前不可用，唯一append入口是`#[cfg(test)] append_for_test`。

## 5. 当前源码一致性阻塞

`zircon_editor/src/core/editing/engine/transaction/operation_group.rs:79`调用：

```text
super::scope::ensure_single_gateway_history(history)?
```

对`zircon_editor/src/core/editing`全树静态检索只得到这一处命中，`scope.rs`没有对应定义。只要编译覆盖该模块，名称解析就应失败。因此：

- 本报告不能声称当前工作树可编译；
- 不应在这个状态上解释任何Cargo结果为事务架构验收；
- ED63-M0必须先由代码owner恢复一致源码并提供Windows受管build/check receipt；
- 修复符号本身不能关闭任何P1/P2 finding，它只是恢复可验证基线。

本轮没有修改该生产代码，也没有运行Cargo，以避免把共享工作树的在途断裂与本报告的review结果混在一起。

## 6. 观察API会改变事务状态

### 6.1 已确认的调用链

`transaction/replay.rs`中的`undo`、`redo`、`is_dirty`、`history_status`、`history_details`、`journal_transaction`和`history_generation_snapshot`都会调用`flush_operation_group()`；`save_token.rs`中的`capture_save_token`和`mark_saved_if_unchanged`、`dirty_batch.rs`中的`dirty_states_since`也会flush。

因此以下看似观察/保存准备的行为都有提交副作用：

| 调用者 | 调用 | 风险 |
|---|---|---|
| `TransactionHistorySnapshot::query` | `history_details` | 打开或刷新History视图即可结束当前group。 |
| `EditorStateSnapshot`构建 | `history_status` | 普通UI snapshot/按钮enablement可能改变history。 |
| active scene history projection | shell锁内查询history | flush可能commit并发布生命周期事件，形成reentrancy/锁序风险。 |
| dirty增量轮询 | `dirty_states_since` | dirty观察可成为隐式transaction boundary。 |
| save准备 | `capture_save_token` | 保存动作在真正读取source前可能提交尚未显式End的交互。 |

这不是单纯命名问题。operation group没有显式owner/session/deadline，当前行为把“下一次恰好发生的查询”当作commit trigger，交互结束时刻受UI刷新频率影响。Unreal的Undo History通过transaction/buffer事件刷新projection，Godot也由history changed信号驱动观察；两者都提供了“观察不负责提交当前交互”的明确参考方向。

### 6.2 必须硬切的合同

1. `history_status/history_details/is_dirty/dirty_states_since`必须是严格无副作用snapshot read。
2. group只能由持有`InteractionGroupToken`的owner执行End/Cancel，或由有审计receipt的deadline/capture-loss policy终止。
3. Undo/Redo可以先请求当前interaction终止，但必须返回typed `InteractionTerminationReceipt`，不能把终止隐藏在history query里。
4. History UI只消费immutable snapshot/event cursor；refresh、scroll、selection和窗口重绘不得改变authoring state。

## 7. Scope、并发与History identity

### 7.1 `!Send`没有形成owner验证

`TransactionScope`用`PhantomData<Rc<()>>`阻止guard跨线程移动，这是正确的局部约束；但`EngineState`没有记录scope owner thread/executor/session。nested判断主要依赖active栈顶history相等，所以另一线程在相同history已有active scope时仍可能被当成nested caller。

另外，commit/cancel/Drop在`EngineBusy`时循环等待全局operation；没有deadline、cancel token或owner loss终态。mutex poison通过`into_inner()`继续运行，也没有把poison转换为document fault incident。

### 7.2 participant只是metadata

`participants: BTreeSet<DocumentId>`只记录整数集合：

- 无before revision；
- 无read/write capability；
- 无prepare/admission；
- 无稳定commit/rollback顺序receipt；
- 无participant session generation；
- `add_participant()`失败时静默no-op。

因此它不能证明multi-document transaction原子性，也不能在第二个document失败时安全补偿第一个document。

### 7.3 History key不足

`HistoryContextId`只有`Global | Document(DocumentId) | PlaySession(PlayInstanceId)`。Scene route另有`WorldDomain`和可选gateway session identity，但这些资格没有进入history namespace；`DocumentId`本身是裸`u64`。

Document close路径会unregister dirty state并detach animation document，却没有调用transaction engine清理对应Document history。若逻辑文档重开后复用DocumentId，旧record可面向新session、missing document或已替换world执行。G08因此仍只能是Partial，不能因为某些scene replacement会clear history就视为关闭。

### 7.4 单engine执行门

`EngineState`只有一份context、active stack、operation group、当前operation和fault state。结果是：

- 不同Document的独立编辑无法拥有独立prepare/commit lane；
- 一个长Navigation/Neural operation会阻塞无关Document的Undo、dirty或history query；
- group和query的flush跨Document传播；
- engine fault的隔离域大于单Document session。

目标应是per-document/session lane，只有真正跨participant的短commit才进入显式global coordinator。

## 8. Command、precondition与merge

### 8.1 Object identity仍不工程化

Runtime当前`EntityId = u64`、`NodeId = EntityId`。World allocator可单调生成id，但没有Bevy式index+generation handle，也没有component/schema/value revision地址。对Undo/Redo而言，“id当前存在”不能证明“它仍是原来的对象”。

目标地址至少需要：

```text
QualifiedObjectAddress {
  document_key,
  document_session_generation,
  world_generation,
  entity_index,
  entity_generation,
  component_type_id,
  component_schema_generation,
  property_path,
  value_revision,
}
```

Bevy的`Entity`以index+generation让despawn后的stale handle失效，适合作为Runtime identity primitive参考；它不是Undo系统，也不能替代Document/session/schema/value资格。

### 8.2 Scene command缺少统一CAS

- `UpdateNodeCommand`保存完整before/after，但apply/revert只验证node存在，然后写入字段；不会校验current state等于expected before/after。
- reflected field command以node id、type path字符串、field字符串和值定位，没有schema generation和current-value revision。
- Batch transform在interactive preview/commit入口会验证world generation和after state，但后续Undo/Redo应用targets时没有统一的current before/after CAS。
- Animation whole-document revision CAS更强，但代价是每次编辑clone/swap/recompile完整asset，并未形成细粒度read/write set。

`EditCommand` trait仍缺stable owner/plugin id、qualified target、read/write set、precondition、retained byte cost、external effect、finalize和async policy。`EditCommandError::TargetMissing { target: String }`也不足以表达可恢复的qualified conflict。

### 8.3 Merge和group身份不足

- operation group key是raw `String`，全engine只有一个group；同字符串+同history即可延续，没有owner/session/target/interaction/sequence/deadline。
- `UpdateNode`按node id合并并替换完整after；没有base revision或field-level write set。
- `PlayTransformCommand`结构里已有`interaction_id`，但`try_merge`只比较node id和world replacement epoch，没有比较interaction id。
- 当前产品Play gizmo使用`MergeMode::Disable`，所以后一项是潜伏合同缺口，不应误写成已观察到的生产误合并。
- merge拒绝只有`Reject`，没有wrong owner、stale base、target mismatch、deadline expired等诊断原因。

必须用typed `InteractionGroupToken`取代字符串，并把merge验证放在command descriptor和base revision之上。

## 9. 产品接入完整性

| 产品域 | 当前权威 | 当前优点 | 未闭合风险 |
|---|---|---|---|
| Scene/Inspector/Hierarchy | unified transaction engine + Document history | 入口基本收敛、typed scene command、route保留 | 裸NodeId、弱precondition、单engine lane、query flush。 |
| Gizmo | interactive batch session + one command | multi-root/frozen transform/compensation已有实质进展 | capture/terminal receipt与typed group未统一；详见Editor254/255。 |
| Animation | unified engine + Document history | whole-document revision CAS、focused undo/redo | clone/recompile成本、错误降级、close不退休history。 |
| UI Asset | private undo stack + private journal | entry能记录source/selection/cursor/theme/document effects | 第二authority、无界、replay失败不原子、journal replay非整批原子。 |
| Operation History UI | transaction details projection | paged details和typed selection | query固定Global的入口仍存在，读取会flush group。 |
| Save/Close | dirty registry + save token + document lifecycle | save CAS与dirty delta可保留 | close/toolkit/history session没有统一terminal transaction。 |
| Navigation | Global operation command | before/after snapshot可撤销 | 同步最多16次poll+`yield_now`，无deadline/cancel/late result。 |
| Neural import | Global operation command | 保留旧输出bytes以支持revert | 同步parse/convert/read/write，非atomic publish、无CAS，history retained bytes无界。 |

### 9.1 UI Asset失败原子性

`UiAssetEditorUndoStack::undo_record/redo_record`先从一个vector pop并push到另一个vector，再由session执行fallible replay。`apply_undo_transition`又会先改变selection/source/cursor，再调用可能失败的document apply/revalidate。于是任何中段失败都可能同时造成：

- history cursor已经移动；
- selection/source/cursor只应用了一部分；
- product document没有完成replay；
- dirty/external effect与实际状态不一致；
- 下一次Undo/Redo面对错误的top record。

短期RED门应先要求peek -> prepare -> apply -> commit cursor；最终必须删除私有stack/journal，迁移到统一DocumentTransactionSession。

### 9.2 Focused history没有成为唯一产品规则

Scene和Animation已按focused document路由一部分命令，但Operation History query仍有固定`HistoryContextId::Global`入口；active scene history view不组合Animation；Animation status错误通过`.ok()`变成`None`。产品层无法区分“没有history”和“engine busy/faulted/query failed”。Undo/Redo、label、dirty、savepoint和fault必须从同一个focused session snapshot得出。

## 10. Async与外部副作用

### 10.1 Navigation

Navigation operation factory把bake/clear都固定到Global history。command execute提交runtime operation后在调用线程最多poll 16次并`thread::yield_now()`，随后harvest。它没有：

- owner/task/session lease；
- cancel/deadline/progress receipt；
- prepared artifact和短commit边界；
- operation仍在运行时的late completion quarantine；
- document/world generation currentness复验。

如果submit后poll/harvest失败，transaction可能尝试compensation，但远端或后台operation仍可能继续完成，形成late write。

### 10.2 Neural import

Neural command的apply同步读取ONNX、解析/转换、读取旧output并直接`fs::write`；revert再写回或删除。问题包括：

- 转换和文件I/O占用transaction执行门；
- output没有temp+fsync+atomic rename publication；
- source/output没有digest/mtime/CAS，Undo可能覆盖外部新文件；
- 无cancel/deadline/progress和shutdown owner；
- old output完整bytes被保留在count-bounded history中，没有per-record/session byte budget；
- command没有durable journal codec，崩溃恢复不能重建外部effect。

正确模型是`Prepare immutable artifact -> validate source/output generations -> short atomic publish -> commit bounded record -> terminal receipt`。不可补偿的外部副作用必须成为显式barrier，而不是假装普通可逆command。

## 11. Journal、event与资源治理

### 11.1 Journal仍缺production闭环

当前codec registry按`(String, u16)`精确匹配，没有N/N-1 migration chain、plugin owner lifetime/deprecation/quarantine。replayer把记录解码后放进caller指定history，而record没有DocumentSession identity，无法证明只回放到原session。

`journal_transaction`在engine state lock内遍历动态command并调用`journal_payload()`；任意codec序列化、分配或插件代码都可延长全局锁。production coordinator又没有commit-time immutable payload append。必须在commit线性点冻结bounded handle，锁外编码/校验/持久化，并由save/close/shutdown显式drain。

### 11.2 History只有count cap

默认capacity为128条record，但单条record可以持有大Scene subtree、完整Animation asset、UI source或Neural旧文件bytes。缺失：

- per-command/per-record/session/global retained byte budget；
- oversize admission/rejection；
- savepoint-aware eviction；
- non-evictable barrier/pin；
- finalize失败和resource release receipt；
- evict reason与bytes telemetry。

Unreal `TransBuffer`至少以累计`DataSize`和`MaxMemory`驱动淘汰并保留cancel恢复所需状态；Fyrox command stack在truncate/evict/clear时会调用`finalize`。Zircon需要更强的typed预算和失败合同，而不是只复制某个参考实现。

### 11.3 Event不是可恢复事实流

`TransactionEvent`只有transaction/history/label/frame/kind等摘要；没有Document session generation、affected set、before/after revision、dirty/savepoint、retained cost、fault和resync cursor。event sink对Backpressured/Rejected只记录warning，没有durable delivery receipt。History UI不能依赖这样的event重建权威状态，只能反复query，而query当前又有flush副作用。

## 12. Editor63 finding当前工作树刷新

### 12.1 P1

| Finding | 当前 | 当前工作树证据与关闭条件 |
|---|---|---|
| ED63-P1-01 History namespace未绑定DocumentTransactionSession/EditContext generation | **Partial** | Scene route含gateway/world资格，replace有局部clear；history仍只有DocumentId。关闭需durable key+session/world generation和close/reopen fail-closed。 |
| ED63-P1-02 active TransactionScope没有owner/thread/session token | **Open** | guard为`!Send`，engine active stack无owner/executor；cross-thread同history仍可能隐式nested。关闭需显式lease和parent token。 |
| ED63-P1-03 participants不是原子参与者租约 | **Open** | 仍是`BTreeSet<DocumentId>` metadata，无before revision/admission/commit/rollback receipt。 |
| ED63-P1-04 Operation group使用弱字符串身份且全engine单例 | **Open** | raw String key；更严重的是status/details/dirty/save query会隐式flush。关闭需typed token和owner-only terminal。 |
| ED63-P1-05 单operation/context/active stack造成跨Document假串行 | **Open** | 一份context/active/group/operation/fault。关闭需per-session lane和显式multi-session短commit coordinator。 |
| ED63-P1-06 scope配置失败静默no-op | **Open** | `set_merge_mode/add_participant`返回`()`并在busy/fault/stale时静默不生效。关闭需首command前冻结配置和typed rejection。 |
| ED63-P1-07 EditCommand缺qualified target/precondition/resource lifecycle | **Partial** | Scene codec和Animation revision可保留；trait仍无read/write set、generational address、retained cost、external/finalize/async policy。 |
| ED63-P1-08 Merge不能证明同interaction/base revision | **Partial** | UpdateNode按node，PlayTransform未比较已有interaction_id；当前Play gizmo禁merge。关闭需同session/target/base/interaction/sequence和typed reject。 |
| ED63-P1-09 长任务/外部副作用在同步apply/revert | **Open** | Navigation同步poll，Neural同步转换和直接文件写删。关闭需prepare/short commit/cancel/deadline/currentness/CAS。 |
| ED63-P1-10 engine mutex内调用任意journal serialization | **Open** | `journal_transaction`持engine lock调用动态payload；production append仍test-only。关闭需commit-frozen bounded handle和独立durable lane。 |
| ED63-P1-11 UI Asset replay前移动cursor | **Open** | private stack先pop/push再fallible replay，product mutation也非全程原子。关闭需先修cursor原子性，再硬切统一history。 |

P1统计：**8 Open / 3 Partial / 0 Closed**。

### 12.2 P2

| Finding | 当前 | 当前工作树证据与关闭条件 |
|---|---|---|
| ED63-P2-01 TransactionEvent缺可重建typed delta | **Open** | event没有session generation、affected set、revision、dirty/savepoint、fault/resync cursor，delivery rejection也无durable receipt。 |
| ED63-P2-02 label/group/journal type缺稳定身份治理 | **Partial** | Scene journal type/schema稳定；label/group仍自由字符串，codec无plugin owner/migration/deprecation治理。 |
| ED63-P2-03 缺history/command资源与延迟遥测 | **Open** | 无retained bytes、scope/group age、merge reason、apply/revert/finalize latency、journal queue与fault指标。 |
| ED63-P2-04 测试只有局部engine不变量 | **Partial** | engine/journal/scene/animation测试明显扩展；仍缺query-nonmutating、document reopen generation、cross-thread nesting、UI replay fault、async late completion、scale/soak/profile。 |

P2统计：**2 Open / 2 Partial / 0 Closed**。

## 13. 五类参考源码差异裁决

| 参考 | 已核对的工程合同 | Zircon当前差异 | 吸收方式 |
|---|---|---|---|
| Unreal | transaction context/session/primary object、object/array participation、selective snapshot、cancel恢复、memory-bounded TransBuffer、buffer/state event驱动Undo History | Zircon弱DocumentId、count cap、query触发commit、无object participation/precondition/owner lease | 吸收identity/participation/memory/event原则，不复制UObject序列化或全局buffer结构。 |
| Godot | manager按scene/resource/global路由history，saved_version和unreachable save state由history owner维护，history_changed驱动产品刷新 | Zircon focused route不全、close未退休session、Operation History仍Global、观察会flush | 建立DocumentSession manager和saved/dirtiness单权威，保留Zircon更强的fallible command与fault语义。 |
| Fyrox | command execute/revert/finalize，group正序execute/逆序revert，stack在truncate/evict/clear finalize | Zircon有更强fallible rollback，但retained resource与eviction finalization无typed receipt | 吸收command domain和terminal resource lifecycle；不要退化到仅count容量。 |
| Bevy | entity index+generation使stale entity handle fail-closed | Zircon Runtime NodeId/EntityId为裸u64，Document/world/schema/value generation分散 | Runtime先提供generational identity；Editor再组成QualifiedObjectAddress。 |
| Unity Graphics公开consumer | `RecordObject(s)`、created/destroy undo、group naming、undo callback refresh；对象/数组引用更新顺序会影响redo正确性 | Zircon产品入口部分统一，但UI Asset/插件仍旁路，mutation与projection刷新没有terminal receipt | 用consumer顺序测试约束产品集成；不声称已审到Unity proprietary Undo core。 |

参考结论不是“选择一个引擎照搬”。Unreal给出transaction identity/participation/memory的成熟基线，Godot给出document history routing/save version，Fyrox给出finalize lifecycle，Bevy给出generational identity，Unity Graphics consumer给出产品mutation/refresh顺序。Zircon要在这些语义之上再用Rust类型系统、per-session并发、typed fault和可测资源预算形成自己的优势。

## 14. 目标架构

```text
Runtime identity authority
  -> WorldGeneration + Entity(index,generation)
  -> ComponentTypeId + SchemaGeneration + ValueRevision

DocumentTransactionRegistry
  -> DocumentSessionKey(durable key, session generation)
  -> per-session HistoryLane / Savepoint / FaultState
  -> focused-session immutable snapshot

AuthoringTransactionService
  -> TransactionLease(owner, executor, session, parent, deadline)
  -> ParticipantLease(before revision, capability, rollback order)
  -> CommandDescriptor(read/write set, preconditions, cost, policies)
  -> InteractionToken(target digest, interaction, sequence, deadline)

AsyncOperationCoordinator
  -> prepare/cancel/progress/deadline/owner
  -> immutable artifact + generation qualification
  -> short atomic commit + late-result quarantine

Durability pipeline
  -> commit-frozen bounded journal handle
  -> lock-free encode/checksum/append queue
  -> save/close/shutdown drain + checkpoint/compaction/recovery

Editor products
  -> Scene / Animation / UI Asset adapters
  -> same focused history/dirty/savepoint/fault snapshot
  -> event-driven History UI with no mutation on read
```

| 核心类型 | 必须承载的合同 |
|---|---|
| `DocumentSessionKey` | canonical durable document key、session generation、kind、owner、terminal state。 |
| `QualifiedObjectAddress` | document/world/entity/component/schema/property/value完整generation/revision。 |
| `TransactionLease` | owner/executor/session/parent、phase、deadline、cancel和terminal receipt。 |
| `ParticipantLease` | role、before revision、write capability、stable commit/rollback order。 |
| `CommandDescriptor` | stable owner/type/schema、read/write set、preconditions、retained cost、journal/finalize/external/async policy。 |
| `InteractionToken` | operation、owner generation、session、target digest、interaction id、sequence、deadline。 |
| `HistoryBudget` | count+bytes、per-record/session/global cap、oversize admission、barrier、pin、eviction reason。 |
| `HistorySavepoint` | session/history generation、record position、source revision/digest、durability stage。 |
| `PreparedOperation` | immutable artifact、input generations、bytes、owner/cancel/deadline、idempotency key。 |
| `HistorySnapshot/Event` | session/currentness、affected set、dirty/savepoint/fault、resync cursor，且读取无副作用。 |

command状态机必须固定为：

```text
Resolve session
  -> Build descriptor
  -> Prepare outside mutation lock
  -> Admit leases and budget
  -> Validate generations/revisions/preconditions
  -> Short apply
  -> Freeze record + journal handle
  -> Commit history/savepoint delta
  -> Publish terminal receipt
```

apply前失败保持Unchanged；partial apply失败按稳定逆序补偿；补偿失败进入document fault；late async result按session/input generation隔离；任何snapshot read不得隐式End/Commit。

## 15. 必须硬切，禁止临时兼容层

1. 不新增第三套Undo stack，也不长期保留UI Asset双写adapter；迁移完成后删除私有stack/journal生产caller。
2. 不以`String`拼接document/group/object identity；所有owner/session/target/interaction使用typed key。
3. 不用“查询前flush”维持旧交互；明确修复所有Begin/Update/End/Cancel producer。
4. 不在transaction mutex内执行任意插件codec、文件I/O、runtime poll、asset compile或callback。
5. 不用裸`u64`存在性检查冒充stale-handle保护；Runtime identity前置完成后再迁移Editor command。
6. 不用record count cap冒充memory budget；所有command必须报告retained cost并接受admission。
7. 不把warning log当作event delivery/fault recovery；terminal、backpressure、quarantine必须有typed receipt。
8. 不用source grep、ignored timing test或单机微基准宣称“性能优于Unreal”；先闭合相同语义，再在同硬件/同数据集测量。

## 16. 依赖有序重构路线

### ED63-M0：恢复一致源码并冻结RED门

1. 修复`ensure_single_gateway_history`缺失并取得Windows受管check/build receipt。
2. 新增read-only query不flush group、cross-thread same-history不能nested、close/reopen generation fail-closed RED tests。
3. 新增UI Asset每个failure point cursor/product原子性、Navigation/Neural late completion与external CAS RED tests。
4. 建立所有Global route、private history、retained bytes、production codec/journal caller census。

### ED63-M1：DocumentSession与qualified identity

1. 复用Editor Document lifecycle的canonical key/session owner，不建立第二registry。
2. Runtime提供world/entity/component/schema/value generation和typed mutation preflight。
3. history、selection、group、savepoint、prepared artifact绑定同一DocumentSessionKey。
4. close/reload/replace原子退休旧scope/history/route/prepared result。

### ED63-M2：Per-session lane与lease

1. `begin()`签发owner/executor/session-qualified lease；nested必须携parent lease。
2. scope配置改为fallible并在首command后冻结。
3. 独立Document并发prepare/read；只有短commit或显式multi-session transaction协调。
4. participant按稳定顺序prepare/admit/commit/rollback并记录before revision。

### ED63-M3：Typed command、precondition与预算

1. 扩展command descriptor、qualified read/write set、CAS precondition和typed conflict。
2. 实现per-command/record/session/global byte accounting、oversize policy、barrier/pin/savepoint-aware eviction。
3. finalize/resource release产生typed terminal receipt。
4. 按Scene/reflection -> Animation -> UI Asset -> plugin operation顺序迁移。

### ED63-M4：Interaction与无副作用观察

1. 统一Begin/Update/End/Cancel，typed group token替代字符串。
2. merge验证owner/session/target/base revision/interaction/sequence/deadline并返回原因。
3. History/dirty/save snapshot完全无副作用，改为event+cursor增量刷新。
4. Gizmo、slider、text composition、curve、drag/drop共享同一interaction协议。

### ED63-M5：Async external operation

1. 复用Editor task authority提供prepare、cancel、deadline、progress、shutdown owner。
2. Navigation产生immutable bake artifact，短commit安装；late result按generation quarantine。
3. Neural使用bounded staging、source/output CAS和atomic publish；不可逆effect成为barrier。
4. plugin unload取消/等待owned artifact、record和codec。

### ED63-M6：产品单权威硬切

1. focused DocumentSession决定Undo/Redo/label/dirty/savepoint/fault，删除固定Global产品查询。
2. UI Asset先通过失败原子性门，再迁移到统一history并删除私有stack/journal。
3. Animation改为可预算的细粒度command，同时保留revision CAS。
4. Save/Save All/Close只消费同一session snapshot和terminal receipt。

### ED63-M7：Durability、观测与资格

1. commit冻结bounded journal handle；独立lane编码/checksum/append。
2. startup/open/save/close/shutdown接入checkpoint/replay/drain/compaction和N/N-1 migration。
3. 发布history bytes、scope/group age、merge reject、apply/revert/finalize latency、rollback/fault/queue metrics。
4. 完成1/10/100 documents、10万commands、large subtree/value、plugin reload、fault/crash/soak/profile。
5. 只有语义、数据集、硬件、编译选项和观测窗口对齐后，才比较Unreal/Godot/Fyrox并讨论性能领先。

依赖顺序：`M0 -> M1 -> M2 -> M3 -> M4/M5 -> M6 -> M7`。M4与M5只可在typed identity/descriptor稳定后并行。

## 17. 验收门

### 17.1 Identity、Session与Scope（G01-G08）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G01 | 每条history绑定DocumentKey/session/world generation | **Partial** |
| G02 | 旧session/world handle在replace后fail-closed | **Partial** |
| G03 | entity/component/property使用generational address | **Fail** |
| G04 | selection snapshot按document/world generation限定 | **Partial** |
| G05 | scope push/commit/cancel验证owner lease | **Fail** |
| G06 | cross-thread同history不能隐式nested | **Fail** |
| G07 | participant是validated lease而非整数metadata | **Fail** |
| G08 | close/reload原子清理或迁移session history | **Partial** |

### 17.2 Command、Rollback与History（G09-G16）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G09 | command声明stable type/schema与read/write set | **Partial** |
| G10 | apply前验证object/schema/value revision | **Partial** |
| G11 | partial apply对所有participant可补偿 | **Fail** |
| G12 | compensation failure形成document fault incident | **Fail** |
| G13 | per-command/record/session/global byte budget | **Fail** |
| G14 | barrier、pin与淘汰不会静默破坏savepoint | **Fail** |
| G15 | finalize资源释放有typed terminal receipt | **Fail** |
| G16 | undo/redo只在原session context执行 | **Partial** |

### 17.3 Interaction、Merge与Routing（G17-G24）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G17 | Begin/Update/End/Cancel状态机统一 | **Fail** |
| G18 | group token含owner/session/target/sequence/deadline | **Fail** |
| G19 | merge校验base revision与同一interaction | **Partial** |
| G20 | merge拒绝原因可诊断 | **Fail** |
| G21 | gizmo preview受revisioned transaction lease保护 | **Fail** |
| G22 | focused document决定Undo/Redo/history label | **Partial** |
| G23 | 无production scene command固定Global | **Partial** |
| G24 | 独立document操作不会互相flush/block | **Fail** |

### 17.4 Savepoint、Dirty与产品统一（G25-G32）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G25 | dirty由history position与durable savepoint派生 | **Partial** |
| G26 | Undo回savepoint自动清dirty | **Partial** |
| G27 | scene/UI/Animation共享同一history authority | **Partial** |
| G28 | UI asset replay失败不移动cursor或分裂状态 | **Fail** |
| G29 | Animation所有持久authoring mutation可undo/redo | **Pass** |
| G30 | Save terminal receipt以CAS推进savepoint | **Partial** |
| G31 | Save All/Close消费同一session snapshot | **Partial** |
| G32 | document fault可导出副本、reload或安全reset | **Fail** |

### 17.5 Async、Journal、Event与资格（G33-G40）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G33 | 长任务prepare与短commit分离 | **Fail** |
| G34 | async command有cancel/deadline/progress/owner | **Fail** |
| G35 | late completion按generation隔离 | **Fail** |
| G36 | 外部副作用有CAS、compensation或barrier | **Partial** |
| G37 | journal payload在commit冻结且持锁区不编码 | **Fail** |
| G38 | journal有framing/checksum/budget/codec migration/replay | **Partial** |
| G39 | event可按generation与affected set重建状态 | **Fail** |
| G40 | product/fault/soak/profile与同语义benchmark有receipt | **Fail** |

资格门统计：**22 Fail / 17 Partial / 1 Pass**。唯一Pass只覆盖Animation持久authoring mutation；不代表Animation性能、document lifecycle、journal或产品History UI已验收。

## 18. 本轮验证边界与下一步

- 已完成静态源码、生产调用、测试源码、owner plan/failure handoff和五类仓内参考实现交叉复核。
- 已确认缺失符号、query flush链、Document close未清history、UI Asset私有cursor顺序、Navigation/Neural同步effect及production journal append缺口。
- 未修改Rust/Cargo/ABI/ZUI，也未运行Cargo或性能测试；因此没有任何实现项可标记Closed。
- 下一实现阶段必须从ED63-M0 RED门和一致源码开始，而不是继续为单个菜单/按钮添加临时Undo逻辑。

要达到并最终在可测指标上超过Unreal，首先必须让identity、transaction terminal、resource budget、failure atomicity和durability成为不可绕过的系统合同；在这些前提未闭合前，吞吐或延迟数字没有可比语义。
