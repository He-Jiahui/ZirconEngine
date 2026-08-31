---
title: Editor Authoring Transaction、Command、History、Undo/Redo、Merge、Group、Savepoint、Dirty、Document Scope、Object Generation、Async Operation 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor63
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/asset/dirty
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/play/pending_edits
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/animation_editor
  - zircon_plugins/navigation/editor/src/operation_command
  - zircon_plugins/neural/editor/src/plugin.rs
tests:
  - zircon_editor/src/tests/editing/transaction_engine
  - zircon_editor/src/tests/editing/context_transactions.rs
  - zircon_editor/src/tests/editing/history.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
  - zircon_editor/src/tests/editing/ui_asset/tree_and_undo.rs
  - zircon_editor/src/core/asset/dirty/tests.rs
  - zircon_plugins/navigation/editor/src/tests/operation_command.rs
  - zircon_plugins/neural/editor/src/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/mvp/05-f4-basic-authoring.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ITransaction.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/ScopedTransaction.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/TransBuffer.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorTransaction.cpp
  - dev/godot/editor/editor_undo_redo_manager.h
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/core/object/undo_redo.h
  - dev/godot/core/object/undo_redo.cpp
  - dev/Fyrox/editor/src/command
  - dev/Fyrox/editor/src/scene/commands
  - dev/Fyrox/editor/src/ui_scene/commands
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_ecs/src/change_detection
  - dev/bevy/crates/bevy_ecs/src/system/commands/mod.rs
  - dev/Graphics/com.unity.postprocessing/PostProcessing/Editor
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Authoring Transaction、Command、History、Undo/Redo、Merge、Group、Savepoint、Dirty、Document Scope、Object Generation、Async Operation 与 Product Integration 当前源码工程化差距

## 1. 结论

当前事务核心已经不是临时的单向command list。`EditorTransactionEngine`具备RAII scope、嵌套事务、反向rollback、失败后的补偿恢复、`CommandEffect`、fault封闭、selection before/after、history cursor、redo截断、save token、dirty generation/journal、history分页、operation group和exclusive transition；23个聚焦测试对这些局部不变量覆盖得明显强于早期版本。这些基础应保留，并作为后续统一authoring transaction authority的内核。

但它还不能作为Unreal/Godot级工程编辑器的产品事务权威。最核心的缺口不是再补一个`undo()`，而是`HistoryContextId`只区分名字，未把某个history绑定到不可变的Document Session、World generation、EditContext、selection domain和object generation。引擎内部只有一个可被替换的`EditContext`；记录中的scene command和selection又持有裸`NodeId(u64)`。因此即使创建`Document(id)` history，也没有合同保证undo/redo一定作用于当初那个document/world，而不是当前被安装进context的World。

产品接入也仍是多权威：scene command、菜单Undo/Redo和Save Project固定使用`Global`；`DirtyRegistry`却查询`Document(document)`；UI asset继续使用私有且无界的双`Vec` undo stack；Animation只有`dirty: bool`而没有history；Navigation和Neural两个真实operation factory虽然进入统一command入口，却仍固定Global，并在同步`apply()`内轮询runtime或转换/覆盖文件。统一引擎存在，不等于产品已统一。

Editor02继续唯一拥有其 **5项P0、30项P1、8项P2**，包括Global路由、双undo authority、无内存预算、journal未持久化、document close不清history、animation无undo、fault无产品恢复等，本报告不得重复相加。Editor59/60/61、Editor08/09、Runtime object identity与各插件报告继续拥有gizmo、selection/world identity、document lifecycle、command discovery、job/cancel和插件域问题。本轮只登记此前未被这些canonical账本覆盖的 **0项P0、11项P1、4项P2**。

目标是建立`AuthoringTransactionService + DocumentTransactionSession + TransactionScopeLease + TypedCommandDescriptor + CommandPrecondition + QualifiedObjectAddress + InteractionGroupToken + HistoryBudget + SavepointCoordinator + AsyncCommandCoordinator + HistoryEventStream + JournalCodecRegistry`。Runtime World继续拥有对象事实；Editor只拥有authoring intent、session/history、selection、dirty/savepoint和恢复策略。任何command在进入apply前必须绑定document/world/object generation与expected revision，任何长耗时或外部副作用必须先prepare、再短时commit，并拥有cancel、deadline、terminal receipt和compensation。

本轮是review-only：未修改production Rust，未运行Cargo、真实Editor、多document save/reopen、plugin reload、外部文件冲突、crash recovery、fault/soak/profile或跨引擎同语义benchmark，因此不能声称性能或表现优于Unreal。tooling按用户要求排除。

## 2. 审查边界、currentness与语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / bytes | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon transaction core | **29 / 4,740 / 155,195** | command、history、scope、rollback、save token、dirty batch、event、journal、routing与operation bridge | `1862f96f09cb49f2c137dfbaae5aad2435371643eb9708cde1c62b7c18e5aebd` |
| Zircon product integration | **52 / 14,627 / 537,866** | Global scene/menu、DirtyRegistry、UI asset、Animation、Navigation、Neural、pending edit与host dispatch | `eb8a4c82c7a1cf4d6d7b98ceff8cb0c3efd2acfd11ce41439bcdb0498b6ccd8b` |
| Zircon focused tests | **23 / 9,220 / 310,855** | scope/history/recovery/locking/group/journal/dirty，以及真实UI asset与插件command测试 | `408a3a6b8f96959b8c7e2d2c969b49b93eaaadf89f4cd5a86cffa3f2710941f6` |
| Unreal selected set | **4 / 2,323 / 74,975** | transaction context、RAII、memory buffer、barrier、object event与editor lifecycle | `aedff26f6976e837c2da41f5ffc45dcf9016397e2643981a3ff1874aa353dc4d` |
| Godot selected set | **5 / 1,660 / 57,699** | per-history manager、object routing、merge、saved version、reference retention与tests | `77f1ae91a52451861241899c9d88558330f9cec744de3b63a5c190b687d4991b` |
| Fyrox selected set | **15 / 5,199 / 171,037** | command stack、scene/UI/animation/ABSM command域、reverse/finalize与generational handle | `a74cd173ae2ab15e112158cd30a113228d03cba09145e62fc723b9399feff687` |
| Bevy selected set | **4 / 6,773 / 250,518** | index+generation entity、change ticks与deferred command queue | `aee21320b695dc0546635c370b8afd664eb568077239c57c61ca47b8546e1244` |
| Unity Graphics selected set | **4 / 2,512 / 103,038** | SerializedObject消费、multi-object undo、created/destroyed object与undo-redo refresh | `55fd07386b6b4d634104de7cb9bb5fe0010db78354ee4a9144648b6194c0efd8` |

fingerprint按规范化小写相对路径、逐文件SHA-256与working-tree内容计算，只证明本轮读取集合；它不是ABI、artifact、性能或动态验收receipt。主仓基线是`bee4c707b714738346b49bba15c59468b8bd9b39`，coordinator baseline epoch为339。Godot、Fyrox、Bevy与Unity Graphics revision分别是`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`和`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像随主仓基线冻结。

### 2.2 在途修改隔离

当前共享checkout存在大量Runtime/UI并行修改。focused corpus中只有`zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs`及其`payload_editing.rs`位于同一宽目录，但它们不属于本报告冻结的52个产品文件，也不改变undo/session/source/save结论。本报告未覆盖、回退或解释其他Session修改；实施前必须重新取得source fingerprint，并重新核对三个共享索引。

coordinator Session为`optimize-editor63-authoring-transaction-history-review-r1-20260822`，模型层声明`5.6-sol / High`，已取得本报告和三个索引的write lease。`docs/plans/zircon_editor/editor/03`仍有transaction selection、gizmo interaction、journal、pending edit retention及detached entity inverse delta等开放failure；本轮只把它们作为实施依赖，不抢占其修复所有权。

### 2.3 canonical owner与去重

| 主题 | 唯一owner | Editor63职责 |
|---|---|---|
| document/save/autosave/recovery及原有transaction finding | Editor02 | current-source刷新，不重复计数5/30/8 |
| scene command、gizmo、selection与mode | Editor03、Editor59、Editor60 | 定义所需transaction lease与qualified address，不复制交互finding |
| document/world lifecycle与multi-document | Editor61 | 消费其DocumentKey/World generation/session terminal receipt |
| command registry、remote与automation | Editor08 | transaction只消费已授权的stable operation identity |
| job、cancel、deadline与shutdown | Editor09 | AsyncCommandCoordinator复用其execution authority |
| plugin lifecycle/provenance | Editor50及各插件报告 | command记录owner generation，不重记插件功能缺口 |
| Runtime entity/world/reflection identity | Runtime24/60/63/109/111 | Editor不得发明第二套对象真值 |
| 新增transaction contract finding | Editor63 | 本报告11项P1、4项P2；实现owner是Editor03计划 |

## 3. 当前实现拓扑与可保留基础

### 3.1 Core transaction路径

```text
begin(label, history)
  -> flush the one global operation group
  -> take the one EditContext
  -> capture the one global SelectionSnapshot
  -> push ActiveTransaction

scope.push(command)
  -> take EditContext
  -> command.apply(context)
  -> retain command or compensate/fault

commit
  -> nested frame folds into parent, or
  -> capture selection_after
  -> merge/push one HistoryStore
  -> update history generation and dirty journal
  -> publish lifecycle event

undo/redo(history)
  -> take the current EditContext
  -> replay retained commands
  -> restore retained bare-node selection snapshot
```

这条链对单context、单World、同步command是完整的。`CommandEffect::Applied/Unchanged`让引擎知道失败command是否需要补偿；undo/redo失败会重做已经反向执行的命令并恢复原selection；补偿再失败则fault engine。scope Drop不会装作成功，而是尝试cancel并保留drop error。`HistorySaveToken`携engine lineage、history、generation和top transaction，能拒绝stale save completion。以上都应保留。

### 3.2 History与dirty基础

`HistoryStore`有redo截断、saved top reachability、count capacity 128、detail paging、generation cursor和finalize淘汰。`history_dirty_changes()`支持generation cursor与full resync；`DirtyRegistry`能把document history与external effect合成批量save候选。问题在于产品scene从未使用该Document history，UI asset与Animation又把自己的dirty真值投影成external effect，所以基础合同没有成为唯一产品事实源。

### 3.3 产品权威实际分裂

```text
Scene authoring
  EditorState::apply_intent / execute_prepared_scene_commands
  -> HistoryContextId::Global
  -> menu Undo/Redo also Global
  -> Save Project marks Global saved

DirtyRegistry
  -> register DocumentId
  -> queries HistoryContextId::Document(document)
  -> scene Global history is therefore outside document dirty projection

UI asset
  -> UiAssetEditorUndoStack { undo: Vec, redo: Vec }
  -> source buffer owns saved_text/revision
  -> host projects external dirty effect

Animation
  -> direct session mutation
  -> dirty: bool
  -> no undo/redo
  -> host projects another external dirty effect
```

这是产品级语义冲突，不是命名问题。一次scene transaction可以让Global history dirty，但registered scene document仍未必在`DirtyRegistry`中变dirty；一次UI asset undo可以返回saved内容，但external effect仍可能保持sticky，直到再次save；Animation保存又直接清自己的bool。Save、Close、窗口标题、Save All和recovery没有共同的savepoint receipt。

### 3.4 Operation command不是异步事务

`EditorHostEventController::invoke_operation`已经会查descriptor/factory、检查remote capability与asset write policy，再把`operation_group`传给`execute_operation()`。这是应保留的统一入口，旧计划中“operation_group完全无消费者”的描述已经过时。

但Navigation factory与Neural factory都固定返回`HistoryContextId::Global`。Navigation command在同步`apply()`里submit runtime request并最多轮询16次；Neural command在`apply()`里读取ONNX、解析转换、生成整份bytes，再直接`fs::write`，同时把原输出完整保留在command中。两者都没有document generation、source digest CAS、deadline/cancel、prepare artifact或异步terminal receipt。统一入口仍在执行一个同步黑盒。

### 3.5 UI asset replay失败原子性

`undo_record()`先从undo pop并push到redo，然后返回transition；`redo_record()`反向执行相同cursor移动。session随后才调用fallible `apply_undo_transition()`。如果transition因source/document/external effect失败，stack cursor已经移动，document却可能仍停在旧状态；下一次Undo会跳过失败记录。这是本报告新增的具体一致性缺口，不能只用“最终迁移统一栈”遮蔽。

## 4. Editor02父账current-source刷新

| 父finding | 当前状态 | 本轮补证 |
|---|---|---|
| P1-04 双undo authority | **Open** | core、UI asset private stack、Animation no-history仍并存 |
| P1-05 Undo/Redo固定Global | **Open** | product intent和remote history query均固定Global；routing helper无product caller |
| P1-06 Animation无undo | **Open** | graph/sequence/state-machine mutation仍只写session并置dirty |
| P1-07 UI asset stack无界 | **Open** | 两个Vec仍无count/byte budget，replay artifact仍clone历史 |
| P1-08 Core只有count cap | **Open** | command contract仍无retained byte estimate；large subtree/value/output均可常驻 |
| P1-09 close不清history | **Open** | toolkit close仍只unregister DirtyRegistry；Document history无teardown owner |
| P1-10/P1-11 journal | **Open** | 仍是内存record到exact-v1 raw JSON，无durable sink/codec registry/replay/budget |
| P1-12/P1-13 lifecycle与save协议 | **Open** | 一套Global scene save与多toolkit external-effect save继续分裂 |
| P1-29 Animation双dirty | **Open** | session bool与host external effect均为可写状态 |
| P1-30 fault恢复 | **Open** | fault/drop error仍没有document-level incident/export/reload/reset产品owner |
| P2-01 event backpressure | **Open** | rejected sink仍只有warning，无generation invalidation/resync receipt |
| P2-02 DocumentKey | **Open** | transaction仍消费session-local DocumentId，command目标仍是裸NodeId/path |

父报告的5项P0、30项P1、8项P2都没有被本轮动态关闭。Editor63不重新登记这些编号，也不把上表数量加入自身11/4统计。

## 5. 本轮新增P1

### ED63-P1-01 · History namespace未绑定DocumentTransactionSession与EditContext generation

证据：`EngineState`只有一个`context: Option<Box<dyn EditContext>>`，同时持有多个`histories: BTreeMap<HistoryContextId, HistoryStore>`。`undo(history)`和`redo(history)`按参数选择store，却从同一个全局slot取当前context执行。`HistoryContextId::Document(id)`只自动增加participant，没有绑定world/context/session generation。

影响：切换World、reload document或未来同时打开多个scene后，旧history可对当前context中的另一World执行裸NodeId command。仅把产品调用从Global改成Document无法修复该问题，反而可能让错误看起来更“per-document”。

关闭：建立`DocumentTransactionSession { document_key, session_generation, world_generation, context_lease, selection_domain, history }`；undo/redo必须通过session handle取得同一generation context，closed/replaced session fail-closed。禁止公开接受脱离session的`HistoryContextId::Document`。

### ED63-P1-02 · 活跃TransactionScope没有owner/thread/session token

证据：`TransactionScope`用`PhantomData<Rc<()>>`阻止scope自身跨线程，但engine是共享对象；`ActiveTransaction`没有owner。另一个线程可直接持engine调用`begin()`，只要history相同，就会被当作合法nested frame，而不是并发owner冲突。

影响：后台callback、plugin或未来async completion可能把不相关command折入前台手势事务；错误label、selection snapshot、cancel/commit边界随后一起被污染。`start_operation`只序列化一次函数调用，不拥有跨多次调用的scope生命周期。

关闭：`begin`签发不可伪造的`TransactionScopeLease { owner, document_session, transaction, thread/executor domain, generation }`；push/configure/commit/cancel都验证lease，嵌套必须由父lease显式创建。跨executor continuation走受控handoff，不能凭history相同自动嵌套。

### ED63-P1-03 · participants只是记录元数据，不是原子参与者租约

证据：`add_participant()`只向`BTreeSet<DocumentId>`插入整数；production没有调用者。commit/undo/redo不验证participant仍open、generation未变、source可写、context属于哪个document，也不取得multi-document ordering或锁。

影响：跨document operation即使在detail/journal里列出participants，也不能保证它们在一个原子边界内被修改或回滚；close/reload后的相同DocumentId亦没有stale拒绝。审计字段被误当成一致性合同。

关闭：把participant改为prepare阶段冻结的`DocumentParticipantLease`，含DocumentKey/session generation、role、before revision、write capability和rollback/compensation policy；按稳定顺序admit并在terminal receipt中逐项结算。无product caller的裸`add_participant(DocumentId)`硬切删除。

### ED63-P1-04 · Operation group使用弱字符串身份且全引擎只有一个

证据：`ActiveOperationGroup`只持`key: String + history + transaction + reservation + phase`，`EngineState`只有一个`Option<ActiveOperationGroup>`。调用方传入任意字符串；没有operation id、source、owner、target property/object、document generation、interaction id、deadline或idle expiry。

影响：两个插件或两个viewport复用同名key可错误续接；owner消失时group可长期占用；另一个document开始操作会先flush当前group。字符串相等无法证明是同一用户交互。

关闭：改为typed `InteractionGroupToken { operation_type, owner_generation, document_session, target_set_digest, interaction_id, sequence, deadline }`，由authoring controller在Begin签发，Update只能续接同token，End/Cancel/owner teardown明确终结。display label与identity分离。

### ED63-P1-05 · 单一operation/context/active stack制造跨Document假串行

证据：engine-wide `operation` gate、`active: Vec<_>`、`operation_group`和context都是单例。`capture_save_token()`、mark saved、undo/redo、begin及dirty batch读取都会先flush全局group；save token又拒绝任何active transaction，即使请求history与active history不同。

影响：未来两个独立document不能并发serialize/save或处理后台prepared result；一个长group会让无关document的savepoint、history query和edit产生延迟或意外commit。它也迫使所有command在一个大互斥域中运行。

关闭：每个`DocumentTransactionSession`拥有独立admission、scope stack、group和context lease；全局事务只通过显式multi-session coordinator按稳定顺序组合。snapshot/status读取使用immutable generation snapshot，不为查询flush另一个session的交互。

### ED63-P1-06 · scope配置API在失败时静默no-op

证据：`set_merge_mode()`与`add_participant()`返回`()`；engine busy、faulted、scope不是stack top或id不匹配时直接什么也不做。调用者随后仍可commit，并认为merge mode/participants已生效。

影响：事务审计、merge和multi-document rollback策略会在最需要fail-closed的并发/故障场景降级，且日志与测试难以证明配置是否真正安装。

关闭：所有scope mutation返回typed result/receipt并验证lease、state与phase；配置只允许在首个command前或明确phase，失败立即阻止后续push/commit。不得用debug assertion或silent branch代替合同。

### ED63-P1-07 · EditCommand没有qualified target、precondition与资源生命周期合同

证据：trait只有label/significant/apply/revert/finalize/try_merge/journal_payload/as_any。scene command持裸`NodeId`与before/after record/value；Runtime的`EntityId`/`NodeId`当前是`u64`。trait没有DocumentKey、World generation、object generation、schema revision、expected value revision、affected object set或fallible finalize receipt。

影响：删除后重用id、world replacement、schema reload、外部文件变化或插件generation切换都只能在黑盒`apply()`里临时发现；某些command可能错误命中新对象。历史内存和外部资源何时释放也无法由engine计划或审计。

关闭：`TypedCommandDescriptor`在admission前声明stable command type/schema、qualified targets、read/write set、preconditions、retained resources和finalization policy。`QualifiedObjectAddress`至少含DocumentKey、World generation、entity(index,generation)、component/type generation与property revision。Runtime提供唯一解析/验证计划，Editor不缓存裸可变指针。

### ED63-P1-08 · Merge合同无法证明同一交互与同一base revision

证据：`try_merge(&dyn EditCommand)`只返回`Reject/Merged`，通过downcast检查具体command；`UpdateNodeCommand`按同一node吸收after状态。record层没有merge key、target property、interaction boundary、before/base revision、最大持续时间、拒绝原因或合并后的precondition复核。

影响：连续但不相关的编辑可能被合并；外部修改插入期间仍可能把旧before与新after拼成一个record；merge失败/成功率也无法诊断。仅比较label或NodeId不构成工程级手势语义。

关闭：merge由`InteractionGroupToken + CommandMergeKey + expected base revision + policy`共同决定，返回typed `Merged/Rejected(reason)/RequiresBoundary`。合并后重新计算affected set、retained bytes和precondition；Begin/Update/End/Cancel与text composition、gizmo capture、slider drag使用同一状态机。

### ED63-P1-09 · 长任务和外部副作用在同步command apply/revert内执行

证据：command接口同步；engine取出唯一context后调用任意command。Navigation在apply内submit/poll/harvest，Neural在apply内读源、解析转换并覆盖文件。没有prepare future、cancel token、deadline、progress、late completion quarantine或post-submit ownership。

影响：UI可冻结，整个transaction engine admission被长任务占用；未知/部分外部效果只能粗略标为`Applied`并触发补偿，无法证明远端任务已停止。undo可能再次执行长I/O，shutdown/plugin unload也没有持有者清单。

关闭：AsyncCommandCoordinator把流程拆成`prepare immutable artifact -> validate currentness -> short atomic commit -> terminal publication`。prepare归Editor09 task authority，携owner/cancel/deadline/budget；commit只安装已验证artifact。外部副作用必须有idempotency key、CAS、compensation或显式不可撤销barrier，late result按generation丢弃。

### ED63-P1-10 · Journal projection在engine互斥域中调用任意command序列化

证据：`HistoryStore::journal()`从record逐command调用`journal_payload()`；transaction plan已记录PERF-MVP-549：该路径在engine mutex内序列化。payload不是commit时冻结的immutable handle，trait实现可以分配、遍历大subtree或执行不可控逻辑。

影响：history导出会阻塞所有edit/undo/savepoint；一个慢或异常plugin command放大整个Editor延迟。相同transaction在不同时间序列化也缺少字节级确定性证明。

关闭：commit时在command owner边界生成bounded `JournalPayloadHandle`或明确`Unavailable(reason)`，record只持immutable bytes/chunk references与codec id；持锁区只复制handle。编码、压缩、checksum和持久化在有预算的journal lane执行，并验证deterministic roundtrip。

### ED63-P1-11 · UI asset undo/redo先移动cursor再执行fallible replay

证据：`undo_record()`/`redo_record()`先在两个Vec间移动entry，再由session执行source/document/selection/external-effect transition。调用链没有在replay失败时把entry和已执行的部分transition共同回滚到原状态。

影响：单次失败可造成“history显示已undo但document未undo”，随后继续编辑会截断错误分支；source文本、document model、selection与external effect还可能各自停在不同阶段。

关闭：迁移统一栈前先增加两阶段`peek -> prepare replay -> apply all participants -> commit cursor`，失败则反向补偿所有已应用参与者且cursor不动。最终删除私有栈，由DocumentTransactionSession持有UI command，并用failure injection覆盖每个transition stage。

## 6. 本轮新增P2

### ED63-P2-01 · TransactionEvent缺少可重建的typed delta

事件只有transaction、history、label、timestamp frame与Started/Committed/Cancelled/Undone/Redone；没有document/world generation、participants、affected object/property set、dirty/savepoint generation、merge/group identity、fault/reset或terminal error。即使sink不丢包，消费者也不能从事件验证当前性或做精确重投影。

关闭：发布versioned `HistoryEventEnvelope`，至少含session identity、history generation、terminal kind、affected-set digest、dirty/savepoint transition和resync cursor；大集合用immutable artifact handle。Editor02 P2-01继续拥有背压后强制resync，本项只拥有事件payload不足。

### ED63-P2-02 · command label、group和journal type缺稳定身份治理

当前label与group主要是自由字符串，journal command type/schema由各实现临时返回；没有统一catalog、localization key、privacy class、plugin owner generation或deprecated/migration状态。UI、诊断、telemetry和recovery容易把显示字符串当协议键。

关闭：建立stable `CommandTypeId/SchemaId/OperationTypeId` catalog；display label走本地化参数，group走typed token，journal codec走版本注册。plugin卸载前必须证明无该generation command/codec holder，或把记录materialize成引擎拥有的兼容形式。

### ED63-P2-03 · 缺少history与command资源/延迟遥测

没有per-history retained bytes、command apply/revert/finalize latency、scope age、merge accepted/rejected reason、group open age、rollback depth、fault stage、journal encode bytes/latency和async prepare/commit queue age。128条上限不能说明性能或内存可控。

关闭：按document/command type/plugin owner输出有基数预算的metrics与trace；设p50/p95/p99、byte cap、max scope/group age与fault rate门槛。telemetry不得泄露路径/属性值，且不能在hot path同步格式化大payload。

### ED63-P2-04 · 测试停留在单engine局部不变量，缺产品矩阵与规模资格

23个focused tests很强地覆盖rollback、fault、save token、dirty cursor、locking和group race，但真实scene仍用Global，Document history主要只在fixture中；没有两document/两World交错、close/reopen、entity id reuse、plugin reload、UI replay每阶段故障、async late completion、1 GB retained history、10万command soak或crash journal replay。

关闭：建立unit/model/product/fault/soak/profile五层矩阵，并与Unreal/Godot相同语义场景比较。每个动态门必须保存输入规模、build/head、hardware、trace、memory peak、latency percentile和terminal receipt；静态source guard不能代替行为资格。

## 7. 五类参考引擎裁决

### 7.1 Unreal：事务上下文、对象生命周期和字节预算是第一等合同

`ITransaction`暴露transaction/operation identity、context、primary object和对象事件；`FScopedTransaction`提供RAII与Cancel；`UTransBuffer`拥有`MaxMemory`、undo/data size、barrier、primary undo object、active record count、redo恢复与lifecycle delegate。对象record与persistent reference/annotation/serialized diff相连，而不是只有匿名command closure。

Zircon应借鉴的是：byte budget、stable operation identity、object participation、barrier、lifecycle event和transaction context；不应照搬UObject序列化或全局Editor singleton。Zircon的typed Rust command与Runtime World边界可以更严格，但前提是补齐qualified identity和资源合同。

### 7.2 Godot：history routing与saved version不能留给UI猜

`EditorUndoRedoManager`为GLOBAL、REMOTE和scene/resource分配history，并按对象归属解析pending action；每个history有`saved_version`，version变化通知能驱动dirty；底层`UndoRedo`支持MergeDisable/MergeEnds/MergeAll、backward undo ops、reference retention、max steps和version events。pending action必须解析到一致history。

Zircon现有`resolve_history_context()`和save token方向正确，但产品不调用routing，Document history也没绑定对象/session。应把Godot“按对象归属路由”的原则升级为“按qualified Document/World/Object generation路由”，而不是复制其ObjectID规则。

### 7.3 Fyrox：command覆盖面与finalize语义优于Zircon产品接入

Fyrox的`CommandTrait`同样使用execute/revert/finalize，`CommandGroup`反序revert/finalize，stack有cursor/capacity；scene、UI scene、Animation和ABSM都有大量真实command。涉及节点移除的command使用generational handle/ticket并在finalize释放保留资源。

Fyrox不是完整的multi-document/savepoint/async authority，不能直接当目标架构；但它证明UI/animation不应保留“以后再接undo”的例外，也证明finalize必须与资源生命周期结合。Zircon应保留更强rollback/fault核心，同时达到同等authoring domain覆盖。

### 7.4 Bevy：代际身份、change tick与deferred apply是Runtime支撑证据

Bevy `Entity`由index+generation组成，despawn后generation变化，旧entity验证失败；change detection用last/this run tick和changed-by信息；`CommandQueue`把deferred mutation与apply时错误处理分开。Bevy没有Editor undo manager，因此不能用来证明transaction产品设计。

Zircon应从Bevy借鉴Runtime侧generational identity、精确change revision和deferred mutation plan，使Editor command可以声明precondition并在短commit时验证。不能把Bevy deferred command直接包装成Editor undo history，也不能让Editor自建第二套entity generation。

### 7.5 Unity Graphics：真实consumer要求SerializedObject与Undo共同工作

本地Graphics仓不含Unity私有transaction core，只能审查消费侧。Volume/Post Processing/Light编辑器使用`SerializedObject.Update/ApplyModifiedProperties`、`Undo.RecordObjects`、created/destroyed object undo和`Undo.undoRedoPerformed` refresh；多对象manipulator会一次记录目标集合。

这证明Zircon的Inspector、curve、volume/light类编辑器不能只改preview或私有session；它们必须通过统一serialized/reflected authoring transaction，并在undo-redo后刷新派生投影。由于核心源码缺失，本报告不对Unity transaction内部的性能、持久化或线程模型作推断。

## 8. 目标架构

### 8.1 权威分层

```text
Runtime World / Asset source authority
  owns object truth, generations, schema revisions, mutation prepare/apply

Editor AuthoringTransactionService
  owns DocumentTransactionSession registry
  owns interaction/transaction/history/savepoint state
  validates qualified command descriptors and participants

Editor AsyncCommandCoordinator
  owns prepare tasks, cancel/deadline/budget and late-result quarantine
  never owns a second copy of World truth

Editor presentation
  consumes immutable HistorySnapshot / DirtySnapshot / terminal receipts
  sends typed Begin/Update/End/Cancel intents
```

### 8.2 核心类型

| 类型 | 必须承载的合同 |
|---|---|
| `DocumentTransactionSessionId` | DocumentKey、session generation、document kind、owner、terminal state |
| `QualifiedObjectAddress` | document/world generation、entity index+generation、component/type generation、property path/revision |
| `TransactionScopeLease` | owner/executor、session、transaction、parent、phase、deadline、cancel state |
| `TypedCommandDescriptor` | stable type/schema、read/write set、precondition、retained resources、journal codec、undo policy |
| `DocumentParticipantLease` | participant role、before revision、write capability、rollback/compensation与commit order |
| `InteractionGroupToken` | operation type、owner generation、target digest、interaction id、sequence、deadline |
| `HistoryBudget` | count、retained bytes、per-record bytes、pinned barrier、global/session budgets |
| `HistorySavepoint` | session/history generation、top record、source revision/digest、durability stage |
| `AsyncPreparedCommand` | immutable artifact、input generations、bytes、deadline、cancel/owner token |
| `HistoryEventEnvelope` | session/currentness、terminal transition、affected set、dirty/savepoint、resync cursor |
| `JournalCodecRegistry` | command type/schema、N/N-1 migration、size/depth budget、owner retention与quarantine |

### 8.3 command执行状态机

```text
Intent
  -> Resolve DocumentTransactionSession
  -> Build typed command descriptor
  -> Prepare read/write set and preconditions
  -> [optional async prepare with cancel/deadline]
  -> Admit participant leases in stable order
  -> Validate generations/revisions
  -> Apply short Runtime mutation plan
  -> Commit immutable history record + journal handle
  -> Publish terminal receipt + dirty transition

failure before apply       -> Unchanged
failure after partial apply -> compensate all applied participants
compensation failure        -> document-scoped fault incident
late async completion       -> quarantine by session/input generation
```

### 8.4 savepoint与dirty

dirty必须是`history current position != durable savepoint`加上typed external effects的只读投影。Save只能消费generation-bound immutable snapshot，磁盘/remote terminal receipt成功后用CAS推进savepoint；Undo回到savepoint自然清dirty，Redo离开自然置dirty。UI asset `saved_text`与Animation `dirty`不得继续作为独立可写真值。

### 8.5 不可保留的兼容层

硬切目标包括：产品`HistoryContextId::Global` scene路由、`UiAssetEditorUndoStack`、Animation可写`dirty: bool`、裸`add_participant(DocumentId)`、自由字符串group identity、未绑定session的Document history、长I/O command apply，以及以raw NodeId/path作为持久history目标。迁移期adapter必须有删除里程碑和零caller gate，不能留下双写或`pub use`兼容门面。

## 9. 依赖有序重构路线

### ED63-M0 · RED资格与父P0封闭前置

1. 将Editor02五项P0、Editor59 gizmo interaction、Editor60 stale selection/world和Editor61 document lifecycle作为entry gates。
2. 增加两Document/两World、cross-thread owner、id reuse、UI replay stage failure、async late result与Global/Document dirty mismatch RED tests。
3. 冻结当前API caller census、history memory census和所有product Global route。
4. 不先写adapter掩盖测试；每个RED test必须证明旧行为确实失败。

### ED63-M1 · Qualified identity与DocumentTransactionSession

1. Runtime先提供World/entity/component/property generation与typed mutation preflight。
2. Editor61提供DocumentKey/session lifecycle terminal state。
3. 新建session registry，把context、selection、history、group、savepoint绑定到同一session generation。
4. close/reload/replace执行exclusive teardown并使旧handle fail-closed。

### ED63-M2 · Scope lease、participant与并发边界

1. `begin`签发owner-qualified lease，嵌套必须携parent lease。
2. scope configuration改为fallible receipt，首command后冻结。
3. multi-document participant在prepare阶段按稳定顺序admit。
4. 独立document可并发prepare/save/status，只有短commit或显式multi-session transaction串行。

### ED63-M3 · Typed command、预算与资源终结

1. command声明stable type/schema、qualified target、read/write set、expected revision与undo policy。
2. 引入per-command/record/session/global byte accounting和barrier/pinning。
3. finalize变成可审计资源终结receipt；失败进入diagnostic/fault policy。
4. scene/reflection command先迁移，使用generational target与Runtime mutation plan。

### ED63-M4 · Interaction、merge与group

1. 建立Begin/Update/End/Cancel interaction controller和typed group token。
2. merge验证target set、base revision、owner与时间/sequence边界。
3. gizmo preview改为revisioned interaction lease；cancel恢复before，end短commit。
4. text composition、slider、curve、drag/drop共享同一协议，不各造coalescing。

### ED63-M5 · Async external operation

1. 与Editor09合并prepare task、cancel、deadline、progress和shutdown owner。
2. Navigation bake返回prepared artifact/terminal receipt，不在command apply里轮询。
3. Neural import使用bounded conversion、staged atomic file、source/output CAS和compensation/barrier。
4. plugin unload等待或取消所有owned prepared/committed holders，并隔离late result。

### ED63-M6 · 产品单权威硬切

1. scene/menu/history query按focused DocumentTransactionSession路由，删除Global默认。
2. UI asset command吸收到统一history，先修cursor原子性，再删除私有stack/journal。
3. Animation所有mutation改为typed reversible command，删除可写dirty bool。
4. DirtyRegistry只消费统一history savepoint与typed external effect receipt；Undo回savepoint必须清dirty。
5. Save/Close/Save All只消费同一session snapshot与terminal receipt。

### ED63-M7 · Journal、observability与规模资格

1. commit冻结bounded journal payload handle，后台framing/checksum/codec/migration/persistence。
2. crash injection验证checkpoint+journal replay、truncation quarantine与N/N-1 schema。
3. 增加history bytes/latency/group age/rollback/fault/async queue telemetry。
4. 执行1/10/100 document、10万command、large subtree/value、plugin reload、fault/soak/profile和跨引擎同语义benchmark。

依赖顺序是`M0 -> M1 -> M2 -> M3 -> M4/M5 -> M6 -> M7`。M4和M5可在M3 typed command稳定后并行设计，但M6不得在identity/session/lease/command contract未完成时用compat adapter提前宣称统一。

## 10. 验收门

### 10.1 Identity、Session与Scope（G01-G08）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G01 | 每条history绑定DocumentKey/session/world generation | **Fail** |
| G02 | 旧session/world handle在replace后fail-closed | **Fail** |
| G03 | entity/component/property使用generational address | **Fail** |
| G04 | selection snapshot按document/world generation限定 | **Fail** |
| G05 | scope push/commit/cancel验证owner lease | **Fail** |
| G06 | cross-thread同history不能隐式nested | **Fail** |
| G07 | participant是validated lease而非整数metadata | **Fail** |
| G08 | close/reload原子清理或迁移session history | **Fail** |

### 10.2 Command、Rollback与History（G09-G16）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G09 | command声明stable type/schema与read/write set | **Fail** |
| G10 | apply前验证object/schema/value revision | **Fail** |
| G11 | partial apply对所有participant可补偿 | **Fail** |
| G12 | compensation failure形成document fault incident | **Fail** |
| G13 | per-command/record/session/global byte budget | **Fail** |
| G14 | barrier、pin与淘汰不会静默破坏savepoint | **Fail** |
| G15 | finalize资源释放有typed terminal receipt | **Fail** |
| G16 | undo/redo只在原session context执行 | **Fail** |

### 10.3 Interaction、Merge与Routing（G17-G24）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G17 | Begin/Update/End/Cancel状态机统一 | **Fail** |
| G18 | group token含owner/session/target/sequence/deadline | **Fail** |
| G19 | merge校验base revision与同一interaction | **Fail** |
| G20 | merge拒绝原因可诊断 | **Fail** |
| G21 | gizmo preview受revisioned transaction lease保护 | **Fail** |
| G22 | focused document决定Undo/Redo/history label | **Fail** |
| G23 | 无production scene command固定Global | **Fail** |
| G24 | 独立document操作不会互相flush/block | **Fail** |

### 10.4 Savepoint、Dirty与产品统一（G25-G32）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G25 | dirty由history position与durable savepoint派生 | **Fail** |
| G26 | Undo回savepoint自动清dirty | **Fail** |
| G27 | scene/UI/Animation共享同一history authority | **Fail** |
| G28 | UI asset replay失败不移动cursor或分裂状态 | **Fail** |
| G29 | Animation所有authoring mutation可undo/redo | **Fail** |
| G30 | Save terminal receipt以CAS推进savepoint | **Fail** |
| G31 | Save All/Close消费同一session snapshot | **Fail** |
| G32 | document fault可导出副本、reload或安全reset | **Fail** |

### 10.5 Async、Journal、Event与资格（G33-G40）

| Gate | 通过条件 | 当前 |
|---|---|---|
| G33 | 长任务prepare与短commit分离 | **Fail** |
| G34 | async command有cancel/deadline/progress/owner | **Fail** |
| G35 | late completion按generation隔离 | **Fail** |
| G36 | 外部副作用有CAS、compensation或barrier | **Fail** |
| G37 | journal payload在commit冻结且持锁区不编码 | **Fail** |
| G38 | journal有framing/checksum/budget/codec migration/replay | **Fail** |
| G39 | event可按generation与affected set重建状态 | **Fail** |
| G40 | product/fault/soak/profile与同语义benchmark有receipt | **Fail** |

40项门当前全部Fail。局部unit test通过、类型存在、菜单能Undo或command可序列化都不能单独关闭任何一门。

## 11. 实施所有权与非目标

Editor03实施计划拥有transaction core与统一command/history硬切；Editor61先交付DocumentKey/session/world lifecycle；Runtime object/reflection报告交付generational identity和typed mutation preflight；Editor09拥有async execution；Editor08拥有command discovery/remote authorization；Editor59/60拥有gizmo与selection consumer迁移；Editor02拥有save/autosave/recovery和其父finding关闭。各owner必须通过typed contract连接，不能复制registry、DocumentKey或task runtime。

本报告不要求把所有runtime gameplay mutation都变成Editor undo；不要求复制Unreal UObject transaction serialization；不把Bevy deferred queue称为undo系统；不推断Unity私有核心；不把autosave、source control或plugin packaging重新归到Editor63；不审查tooling。目标只是在Editor authoring边界建立可证明的单一transaction/history/savepoint authority。

## 12. 当前裁决

1. **保留并继续强化**：RAII scope、CommandEffect、rollback recovery、fault封闭、history cursor、save token、dirty generation cursor、exclusive transition、event入口和operation factory dispatch。
2. **不能继续扩展为默认方案**：Global scene history、单EditContext多history、裸NodeId command、自由字符串group、同步外部I/O command、UI asset私有栈、Animation dirty bool。
3. **实施第一步不是继续加command类型**：先关闭M0 RED gates，建立DocumentTransactionSession与qualified identity；否则更多command只会扩大可重放到错误World的历史表面积。
4. **工程级目标不是“能撤销一次”**：必须同时证明identity、ownership、rollback、memory、savepoint、async cancellation、journal recovery、plugin teardown、multi-document concurrency与规模性能。

在这些门关闭并取得动态receipt前，Zircon可以说“已有较强的单context同步事务内核”，不能说“已有Unreal/Godot级工程authoring transaction系统”，更不能以静态结构声称性能或表现优于Unreal。
