---
title: Editor Authoring Transaction、Command、History、Undo/Redo、Merge、Group、Savepoint、Dirty、Document Scope、Object Generation、Async Operation 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor184
review_date: 2026-08-27
baseline_head: f48ed29a1ff80cf6c35ba747f074532aec48ea6a
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/asset/dirty
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/recovery/document_journal
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
  - zircon_plugins/navigation/editor/src/operation_command
  - zircon_plugins/neural/editor/src/plugin.rs
tests:
  - zircon_editor/src/tests/editing
  - zircon_plugins/navigation/editor/src/tests/operation_command.rs
  - zircon_plugins/neural/editor/src/tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/03/2026-08-24-scene-document-history-routing-hardcut.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-22-history-dirty-batch-generation-contract-missing.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-22-transaction-selection-history-wide-snapshot.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-29-transaction-journal-contract-unimplemented.md
  - docs/plans/zircon_editor/editor/03/failure-2026-08-19-gizmo-world-space-interactive-transaction.md
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
  - dev/Fyrox/editor/src/plugins/animation/command
  - dev/Fyrox/editor/src/plugins/absm/command
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_ecs/src/change_detection
  - dev/bevy/crates/bevy_ecs/src/system/commands/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/CoreEditorUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/InspectorCurveEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentCopyPaste.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Authoring Transaction / Command History 当前源码复核（Editor184）

## 1. 结论

当前事务核心已经具备值得保留的工程底座：RAII scope、嵌套事务、反向rollback、补偿、fault封闭、selection snapshot、history cursor、redo截断、save token、dirty generation、exclusive transition、路由化Edit World、durable journal framing/checksum/compaction/replay以及较密集的局部测试。Editor63之后，主Scene和Animation的产品接入也有实质进展：主Scene已按活动`DocumentId`路由Undo/Redo/savepoint；history record保留具体gateway session route；Animation持久化编辑进入统一`EditorTransactionEngine`，不再维护第二个可写`dirty: bool`；Scene command已有稳定journal type/schema和四类codec。

这些进展修正了Editor63中“主Scene固定Global”“Animation没有统一Undo”“journal只有内存投影”的过时描述，但没有把系统提升为Unreal/Godot级authoring transaction authority。当前核心仍以`Global | Document(DocumentId) | PlaySession`作为history namespace，没有不可变`DocumentKey + session generation + world generation`；整个engine仍只有一个可替换`EditContext`、一个active stack、一个operation group和一个全局operation gate。scope没有owner/thread/session lease，participants只是`BTreeSet<DocumentId>`元数据，command也没有qualified read/write set、precondition、resource/finalize合同。

产品仍然分裂：UI Asset保留私有无界双`Vec` history，并在fallible replay前移动cursor；Navigation和Neural operation仍固定Global并在同步`apply/revert`内轮询Runtime或直接读写文件；operation history query仍查询Global。durable journal模块虽已存在，但Scene codec注册、startup恢复、save checkpoint与产品replayer尚未接线，且`journal_transaction()`仍可在engine mutex内调用任意command的`journal_payload()`。

Editor63继续是本主题11项P1、4项P2与40项Gate的唯一canonical owner；Editor184新增canonical finding为 **0 P0 / 0 P1 / 0 P2**，只刷新状态：**11项P1为8 Open / 3 Partial；4项P2为2 Open / 2 Partial；40项Gate为22 Fail / 17 Partial / 1 Pass**。Editor02的transaction/save父账仍单独拥有对应问题，不在这里重复计数。

本轮只做静态review和重构计划，没有修改production或tests，没有运行Cargo、真实Editor、multi-document、plugin unload、crash recovery、fault/soak/profile或跨引擎同语义benchmark。当前证据不支持“性能或表现优于Unreal”的声明；必须在语义、规模、硬件和capture口径一致后才可裁决。

## 2. 审查边界与currentness

### 2.1 冻结语料

fingerprint算法为：相对路径转小写并统一分隔符，追加NUL、原始文件bytes、NUL，对排序集合计算SHA-256。它只冻结本轮读取的当前磁盘，不代表ABI、artifact、编译、行为或性能receipt。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 | fingerprint |
|---|---:|---|---|
| Zircon transaction/editing core | **66 / 9,748 / 8,886 / 328,291 / 28 / 3** | command、history、scope、rollback、routing、save token、dirty、journal、recovery coordinator与Animation document | `503ca3f484c4650b5323ea37497d5d27dbe47592d3cab53ecb6bf0cb6922aa2d` |
| Zircon product integration selected set | **190 / 47,413 / 43,715 / 1,660,931 / 261 / 48** | Scene/menu/operation、DirtyRegistry、UI asset、Animation、Navigation、Neural与host/workbench接线 | `0d26aa1e0f944c9ae52b04af9114afd392c94258cbdd126a50553977e72c4a9d` |
| Zircon focused tests | **68 / 22,831 / 20,836 / 763,560 / 396 / 0** | engine、history、journal、document route、UI asset、Animation与插件operation测试 | `b24bec6677e2bb7acd6ab57e6ece974583a11ff9b4a3339eb407919fb39d0b65` |
| Unreal selected set | **4 / 2,323 / 1,961 / 74,975 / 0 / 0** | transaction context、object lifecycle、RAII、memory budget、barrier与editor lifecycle | `e169d17aa4c18694285812f30d1c51930bf65aba6ca98af80e47ac466dead4b0` |
| Godot selected set | **4 / 1,461 / 1,204 / 49,878 / 0 / 0** | routed histories、merge、reference retention、saved version与version change | `2cc21e2814d4ad8a59065b327798adea9e47d3fdde3c010ac436d1e934beaf30` |
| Fyrox selected set | **16 / 5,404 / 4,767 / 178,555 / 0 / 0** | command stack/group、scene/UI/animation/ABSM domain、reverse/finalize与generational handles | `29d823523fdb449caf0e236d363a0de2cdf7e1611663de05280a867193e84f1e` |
| Bevy selected set | **7 / 7,635 / 6,995 / 279,155 / 36 / 0** | entity generation、change tick/changed-by与deferred command queue | `7f954f70acc76f34037376be6769a246bc2a6de2f1b77ca67dabd6c7815a8d72` |
| Unity Graphics selected set | **4 / 3,444 / 2,934 / 148,888 / 0 / 0** | `SerializedObject`刷新/提交、多对象Undo、created/destroyed object Undo与undo回调消费 | `63925ceef3311d0e39007e8ccda377327f4dc646dad8079f5323f2b17df7c3c2` |

主仓HEAD冻结为`f48ed29a1ff80cf6c35ba747f074532aec48ea6a`。Godot、Fyrox、Bevy和Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像随主工作区冻结。

### 2.2 共享工作树与未验证调用

本轮读取的transaction、journal、UI asset、Animation、Scene route及索引范围存在大量其他Session或用户的modified/untracked文件。本文以fingerprint对应的当前磁盘为事实，不归因、不覆盖、不回退这些变化。

当前`zircon_editor/src/core/editing/engine/transaction/operation_group.rs`调用`super::scope::ensure_single_gateway_history(history)?`，但在当前`zircon_editor/src`中没有找到定义；该调用也是共享工作树中的在途改动。它意味着当前磁盘不能仅凭静态阅读宣称transaction模块可编译。Editor184不把共享在途编译缺口升级为新的canonical架构finding，也不在review阶段越权修复；实施前必须先由owner恢复一致源码并产生新鲜编译receipt。

本轮按用户要求不查询、轮询、等待或实时跟踪协调器；阻塞留在原owner，继续完成独立review。

### 2.3 Owner与去重

| 主题 | 唯一owner | Editor184职责 |
|---|---|---|
| 11项事务架构P1、4项P2、40门 | Editor63 | current-source状态、产品证据、重构顺序 |
| transaction/save/autosave/recovery父账 | Editor02 | 只刷新交叉项，不重复finding |
| DocumentKey/session/world lifecycle | Editor61 | transaction消费qualified session，不另建identity |
| selection/gizmo interaction | Editor59/60 | 消费interaction lease，不在本报告复制交互finding |
| command registry/remote authorization | Editor08 | stable command discovery与principal，不归transaction重建 |
| async job/cancel/shutdown | Editor09 | AsyncCommandCoordinator复用其执行authority |
| Runtime entity/schema/property generation | Runtime object/reflection owner | 提供qualified address与mutation preflight |
| transaction core实施 | Editor03 | 依赖顺序硬切并提交动态receipt |

## 3. 当前实现拓扑与可保留基础

### 3.1 Core transaction并非临时list

`EditorTransactionEngine`使用scope guard管理commit/cancel/drop；command逐条apply，失败时反向revert，补偿失败会fault engine。history保留cursor、redo分支截断、selection before/after、route和save token；dirty generation基于current/saved position变化。exclusive transition能拒绝dirty替换并清理当前history/context。`TransactionScope`通过`PhantomData<Rc<()>>`成为`!Send`，阻止guard本身跨线程移动。

这些是正确基础，但`!Send`不等于engine拥有线程lease：另一个线程仍可用同一个`Arc<EditorTransactionEngine>`对同一history调用`begin()`，并被当前global active stack解释为嵌套scope。引擎没有记录executor/thread/owner/session token，也没有证明嵌套关系来自同一个交互调用链。

### 3.2 Scene route与replace语义显著进步

`EditWorldRoute`现在捕获具体`GatewaySessionIdentity`，history record保留route，undo/redo会重新激活原route；Play history明确volatile，结束时丢弃并退休route。startup/new/open/AlreadyActive绑定已提交`DocumentId`，scene command、gizmo release、Undo/Redo、snapshot、save saved-top和close dirty prompt都使用`HistoryContextId::Document(document)`。Scene replacement通过exclusive transition清history/context，避免把旧scene record直接留给新World。

但`Document(DocumentId)`仍是session-local整数namespace，不含canonical DocumentKey、document session generation或world generation。route能让旧gateway session失效，但bare `NodeId`、selection snapshot和reflected property command仍不能证明对象generation/value revision；独立document也仍共享单一engine context/active stack/operation gate。

### 3.3 Animation进入统一history，但UI Asset仍是第二权威

Animation持久编辑使用`AnimationEditCommand`和`HistoryContextId::Document`，命令带expected document revision，Workbench Undo/Redo/status也进入统一engine；dirty通过已注册document的history/savepoint投影，不再由animation session维护独立可写bool。这关闭了“Animation完全没有Undo”和“双dirty bool”的旧结论。

Animation当前command仍以整个document replacement为主要粒度，没有interaction merge、selection snapshot或更细qualified read/write set，所以统一接线不等于所有工程门已关闭。

UI Asset仍使用`UiAssetEditorUndoStack`的undo/redo两个无界`Vec`。记录持有完整source/diff、selection和克隆数据；`undo_record()`先从undo pop并push到redo，随后session才执行fallible transition。任一source/document/selection/external-effect阶段失败都可能令cursor与真实文档分裂。它还有独立journal/replay协议，继续阻止scene/UI/Animation形成一个authority。

### 3.4 Journal基础完成了模块化，产品闭环仍缺失

durable journal现有record/file byte限制、length framing、BLAKE3 checksum、sequence、tail fault、atomic compaction、codec registry和replayer；Scene create/delete/update/reflected命令已有stable type/schema与四类codec。这比Editor63时的memory-only projection明显前进。

但`register_scene_command_codecs()`没有production caller，document journal coordinator没有形成startup/open/save/close产品恢复链。codec目前固定schema 1，没有build/schema compatibility window、N/N-1 migration owner或plugin unload retention。更关键的是`journal_transaction()`取得engine state后调用`HistoryStore::journal()`，record又逐command调用`journal_payload()`；payload没有在commit时冻结，任意command仍可在engine mutex内分配、遍历或编码。

### 3.5 Operation group和async command仍是全局同步模型

`EngineState`保留一个`context`、一条`active`栈、一个`operation_group`和一个operation gate。group identity仍是自由`String key`；reservation改进了并发初始化，但token没有owner/session/target/interaction/sequence/deadline，独立document的operation仍可能相互flush或阻塞。`set_merge_mode()`与`add_participant()`返回`()`，busy/fault/not-top时静默no-op。

Navigation factory固定返回Global；command在同步`apply()`中submit Runtime operation，最多poll 16次并`yield_now()`，再harvest；undo/redo重新执行Runtime operation。Neural factory同样固定Global，`apply()`直接读取ONNX、解析转换、保留旧输出并`fs::write`，`revert()`再次写或删文件。两者都没有immutable async prepare、cancel/deadline/progress、generation quarantine、write-time CAS或atomic staged publish。

### 3.6 Event、预算与fault产品化不足

`TransactionEvent`仍只有transaction/history/label/timestamp/kind，没有session/world generation、participants、affected object/property set、dirty/savepoint transition、group、fault或resync cursor。event sink拒绝只记warning，没有强制snapshot invalidation。

history只有默认128 record count cap，没有per-command/record/session/global retained-byte budget、barrier、pin或savepoint-aware eviction；UI Asset甚至没有count cap。fault虽然会封闭engine，但产品没有document-scoped incident、导出副本、reload或safe reset流程，也没有资源/延迟telemetry证明规模可控。

## 4. Editor02父账current-source刷新

下表只刷新与transaction/history直接重叠的父finding，不改变Editor02的总数与owner。

| 父finding | 当前 | current-source裁决 |
|---|---|---|
| ED02-P1-04 双Undo authority | **Open** | core+Animation统一，但UI Asset私有stack/journal仍在 |
| ED02-P1-05 Undo/Redo固定Global | **Partial** | 主Scene和Animation按Document路由；Navigation、Neural、operation query及UI私有stack未收敛 |
| ED02-P1-06 Animation无Undo | **Partial** | persistent mutation已可Undo/Redo，但缺selection restoration、interaction merge和细粒度command |
| ED02-P1-07 UI Asset stack无界 | **Open** | 两个`Vec`、完整source/diff与clone retention仍无预算 |
| ED02-P1-08 core只有count cap | **Open** | 128 records存在，但没有retained-byte/global budget与savepoint-aware eviction |
| ED02-P1-09 close不清history | **Partial** | Scene replace/clear已处理；Animation/toolkit close仍无统一session teardown |
| ED02-P1-10 journal只序列化 | **Partial** | durable framework与codec已存在；production sink/startup recovery仍未接线 |
| ED02-P1-11 journal预算/迁移缺失 | **Partial** | framing/checksum/file+record cap已有；N/N-1 migration、build/schema深度与产品闭环缺失 |
| ED02-P1-12 lifecycle单active | **Open** | transaction engine仍为单context/active stack/operation group |
| ED02-P1-13 Save与document batch分裂 | **Partial** | Scene save token按Document推进；没有aggregate Save All/session protocol |
| ED02-P1-29 Animation双dirty | **Closed** | 可写session dirty已删除，Workbench只读取统一document dirty投影 |
| ED02-P1-30 transaction fault无恢复owner | **Open** | fault封闭存在，产品incident/recovery仍缺失 |
| ED02-P2-01 event rejection只warning | **Open** | 没有per-history resync marker或强制snapshot refresh |
| ED02-P2-02 弱DocumentKey | **Partial** | JournalDocumentKey已canonical project-relative+BLAKE3；lifecycle/history仍用DocumentId |

## 5. Editor63 P1状态刷新

| Finding | 当前 | 证据与关闭条件 |
|---|---|---|
| ED63-P1-01 History namespace未绑定DocumentTransactionSession/EditContext generation | **Partial** | record保留具体gateway route且Scene replacement清history；`HistoryContextId`仍不含DocumentKey/session/world generation。关闭需session registry与旧handle fail-close。 |
| ED63-P1-02 active TransactionScope没有owner/thread/session token | **Open** | guard为`!Send`，但engine active stack无executor/owner lease，另一线程仍可隐式nested。关闭需显式parent lease和thread/executor validation。 |
| ED63-P1-03 participants不是原子参与者租约 | **Open** | 仍是`BTreeSet<DocumentId>`，`add_participant()`只插入元数据且无production caller。关闭需prepare/admit/rollback顺序和before revision lease。 |
| ED63-P1-04 Operation group使用弱字符串身份且全engine单例 | **Open** | 只有`String key`和一个engine-wide group。关闭需typed owner/session/target/interaction/sequence/deadline token。 |
| ED63-P1-05 单operation/context/active stack造成跨Document假串行 | **Open** | `EngineState`仍单context/active/group/gate。关闭需per-session lane与仅短commit的显式multi-session协调。 |
| ED63-P1-06 scope配置失败静默no-op | **Open** | `set_merge_mode/add_participant`返回`()`，不是fallible receipt。关闭需首command前冻结配置并返回typed rejection。 |
| ED63-P1-07 EditCommand缺qualified target/precondition/resource lifecycle | **Partial** | Scene有stable journal type/schema，Animation有expected revision，route保留gateway session；trait仍无read/write set、qualified address、resource/finalize policy。 |
| ED63-P1-08 Merge不能证明同interaction/base revision | **Partial** | UpdateNode按NodeId、PlayTransform按node/world epoch merge；没有interaction id、base value revision、target digest或typed reject reason。 |
| ED63-P1-09 长任务/外部副作用在同步apply/revert | **Open** | Navigation同步poll/harvest，Neural同步转换和文件写删。关闭需async prepare、短commit、cancel/deadline/currentness与CAS/compensation。 |
| ED63-P1-10 engine mutex内调用任意journal serialization | **Open** | payload未在commit冻结，`HistoryStore::journal`仍逐command调用`journal_payload()`。关闭需bounded immutable payload handle和独立journal lane。 |
| ED63-P1-11 UI Asset replay前移动cursor | **Open** | private stack先pop/push再执行fallible replay。关闭需peek/prepare/apply/commit-cursor或直接硬切统一history。 |

P1统计为 **8 Open / 3 Partial / 0 Closed**。局部route、codec或expected revision不能单独关闭session identity、command descriptor或merge合同。

## 6. Editor63 P2状态刷新

| Finding | 当前 | 证据与关闭条件 |
|---|---|---|
| ED63-P2-01 TransactionEvent缺可重建typed delta | **Open** | event仍无session generation、affected set、dirty/savepoint、fault与resync cursor。 |
| ED63-P2-02 label/group/journal type缺稳定身份治理 | **Partial** | Scene command type/schema已稳定；label/group仍自由字符串，codec无plugin owner/migration/deprecation治理。 |
| ED63-P2-03 缺history/command资源与延迟遥测 | **Open** | 无retained bytes、scope/group age、merge reason、apply/revert/finalize latency、fault/journal queue metrics。 |
| ED63-P2-04 测试只有局部engine不变量 | **Partial** | focused tests扩展到Scene route、Animation和durable journal，但仍无multi-document、UI replay fault、plugin unload、crash/scale/soak/profile资格。 |

P2统计为 **2 Open / 2 Partial / 0 Closed**。

## 7. 五类参考引擎裁决

### 7.1 Unreal：identity、object participation与memory是事务本体

`ITransaction`把transaction/operation GUID、context、primary object、begin/end/finalize和对象参与作为第一等合同；`FScopedTransaction`提供RAII与Cancel；`UTransBuffer`显式拥有MaxMemory/DataSize、barrier、active count、undo buffer、object reference retention和lifecycle事件。Zircon已有RAII/fault/history cursor，但不能只以record count替代字节预算，也不能用裸NodeId和当前EditContext替代对象/session identity。

### 7.2 Godot：history routing和saved version由manager决定

`EditorUndoRedoManager`按global/remote/scene/resource路由history，`UndoRedo`提供merge mode、reference retention、max steps与version；saved version是history事实而不是面板bool。Zircon主Scene/Animation按Document路由是正确方向，但operation Global、UI私有stack和弱DocumentId说明manager authority尚未统一。

### 7.3 Fyrox：command domain完整性与generational handle应一起迁移

Fyrox command trait的execute/revert/finalize、group逆序行为和Scene/UI/Animation/ABSM各域覆盖说明“有统一trait”必须落到所有authoring产品；其handle/ticket代际也阻止旧对象引用静默命中新对象。Zircon不应复制其具体API，但应吸收domain coverage、finalize和generational address三个合同。

### 7.4 Bevy：不是Undo系统，但能约束Runtime支撑

Bevy entity由index+generation组成，change tick/changed-by帮助验证更新来源，deferred `CommandQueue`把准备与World apply分开。它不能作为Editor history参考，却证明Zircon的qualified object、expected revision和短commit不应由UI字符串补丁临时模拟，而应建立在Runtime代际与mutation preflight上。

### 7.5 Unity Graphics：公开consumer证明编辑、Undo与刷新必须闭环

Graphics源码中的公开Editor consumer在修改前使用多对象Undo记录，在`SerializedObject.Update/ApplyModifiedProperties`之间编辑，并在undo/redo回调后刷新派生视图；created/destroyed object也使用对应Undo API。本文只据此要求Zircon的Inspector/Curve/Volume等consumer走统一transaction并重投影，不推断Unity私有transaction核心、性能或持久化实现。

## 8. 目标架构

```text
Runtime World / Asset source authority
  -> generational object/schema/value identity
  -> prepare/validate/apply short mutation plans

AuthoringTransactionService
  -> DocumentTransactionSession registry
  -> TransactionScopeLease + DocumentParticipantLease
  -> TypedCommandDescriptor + CommandPrecondition
  -> InteractionGroupToken + HistoryBudget
  -> SavepointCoordinator + HistoryEventStream

AsyncCommandCoordinator
  -> immutable prepare artifact, cancel/deadline/progress/owner
  -> currentness validation, short commit, late-result quarantine

JournalCodecRegistry
  -> commit-frozen bounded payload
  -> framing/checksum/migration/replay/checkpoint/compaction

Editor products
  -> focused session routing
  -> immutable history/dirty/fault snapshots
  -> typed Begin/Update/End/Cancel intents and terminal receipts
```

| 核心类型 | 必须承载的合同 |
|---|---|
| `DocumentTransactionSessionId` | DocumentKey、session generation、document kind、owner、world route与terminal state |
| `QualifiedObjectAddress` | document/world generation、entity index+generation、component/schema generation、property path/value revision |
| `TransactionScopeLease` | owner/executor、session、transaction、parent、phase、deadline、cancel state |
| `DocumentParticipantLease` | participant role、before revision、write capability、commit order、rollback/compensation |
| `TypedCommandDescriptor` | stable type/schema、read/write set、preconditions、retained resources、journal/finalize/undo policy |
| `InteractionGroupToken` | operation type、owner generation、target digest、interaction id、sequence、deadline |
| `HistoryBudget` | count、retained bytes、per-record/session/global cap、barrier、pin与eviction reason |
| `HistorySavepoint` | session/history generation、record top、source revision/digest、durability stage |
| `AsyncPreparedCommand` | immutable artifact、input generations、bytes、owner/cancel/deadline与idempotency key |
| `HistoryEventEnvelope` | session/currentness、terminal transition、affected set、dirty/savepoint、fault与resync cursor |
| `JournalCodecRegistry` | command type/schema、N/N-1 migration、size/depth budget、plugin owner retention与quarantine |

command状态机必须是：`Resolve session -> Build descriptor -> Prepare -> Admit leases -> Validate generations/revisions -> Short apply -> Commit immutable record/payload -> Publish terminal receipt`。apply前失败保持Unchanged；partial apply失败反向补偿；补偿失败形成document fault；late async completion按session/input generation隔离。

## 9. 依赖有序重构路线

### ED63-M0 · 恢复一致current source并建立RED门

1. 先解决`ensure_single_gateway_history`未定义等在途源码一致性，由owner提交Windows受管编译receipt。
2. 增加two-document/two-world、cross-thread implicit nest、ID reuse、UI replay各阶段failure、async late result和Global/Document dirty mismatch RED tests。
3. 冻结所有product Global route、private history、retained bytes和codec production caller census。
4. 保留现有route/Animation/journal tests，不以source grep替代行为门。

### ED63-M1 · DocumentTransactionSession与qualified identity

1. 复用Editor61 canonical DocumentKey/session lifecycle，不再创建第二套document registry。
2. Runtime提供world/entity/component/property generation与typed mutation preflight。
3. history/context/selection/group/savepoint绑定同一session generation。
4. close/reload/replace原子退休旧route、history、scope和prepared result。

### ED63-M2 · Scope lease、participant与并发边界

1. `begin()`签发owner/executor/session-qualified lease，nested必须携parent lease。
2. scope配置改为fallible receipt，首command后冻结。
3. multi-document participant按稳定顺序prepare/admit，记录before revision与rollback order。
4. 独立document可并发prepare/status/save；只有短World commit或显式multi-session transaction串行。

### ED63-M3 · Typed command、预算与资源终结

1. `EditCommand`迁移到descriptor/read-write/precondition/qualified target合同。
2. 引入per-command/record/session/global byte accounting与barrier/pin/savepoint-aware eviction。
3. finalize发布typed terminal receipt；资源释放失败进入可诊断policy。
4. 先迁移Scene/reflection/Animation command，再迁移插件operation和UI Asset。

### ED63-M4 · Interaction、merge与group

1. 统一Begin/Update/End/Cancel controller和typed group token。
2. merge验证owner/session/target/base revision/interaction/sequence/deadline，返回typed rejection reason。
3. gizmo、text composition、slider、curve和drag/drop共享协议。
4. preview使用revisioned lease；Cancel恢复before，End只做短commit。

### ED63-M5 · Async external operation

1. 复用Editor09 task authority提供prepare、cancel、deadline、progress、shutdown owner。
2. Navigation bake输出immutable artifact，不在command apply中poll。
3. Neural转换使用bounded staging、source/output CAS、atomic publish与compensation/barrier。
4. plugin unload取消或等待其owned artifacts/records/codecs，late result按generation quarantine。

### ED63-M6 · 产品单权威硬切

1. focused DocumentTransactionSession决定Undo/Redo/label/dirty/savepoint；删除operation query Global默认。
2. UI Asset先修cursor原子性，再把command/journal吸收到统一history并删除私有stack。
3. 保留Animation统一route，补selection restoration、细粒度command和interaction merge。
4. Save/Save All/Close只消费同一session snapshot和terminal receipt。
5. 删除所有双写dirty、compat adapter和旧route，要求零production caller gate。

### ED63-M7 · Production journal、recovery与observability

1. commit冻结bounded payload handle，持锁区只复制handle；后台编码/checksum/persist。
2. 把codec registry接到startup/open/save/checkpoint/replay/compaction，验证truncated tail和N/N-1 migration。
3. 发布history bytes/latency/group age/merge reason/rollback/fault/async queue metrics。
4. 执行1/10/100 documents、10万commands、large subtree/value、plugin reload、fault/crash/soak/profile与同硬件跨引擎同语义benchmark。

依赖顺序为`M0 -> M1 -> M2 -> M3 -> M4/M5 -> M6 -> M7`。M4与M5可在typed descriptor稳定后并行实现；在identity/session/lease未完成前，不得用compat adapter宣称产品已经统一。

## 10. 验收门

### 10.1 Identity、Session与Scope（G01-G08）

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

### 10.2 Command、Rollback与History（G09-G16）

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

### 10.3 Interaction、Merge与Routing（G17-G24）

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

### 10.4 Savepoint、Dirty与产品统一（G25-G32）

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

### 10.5 Async、Journal、Event与资格（G33-G40）

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

40项门统计为 **22 Fail / 17 Partial / 1 Pass**。唯一Pass是Animation持久authoring mutation已经进入统一Undo/Redo；scrub/range/playback等非持久交互不在该门范围。Partial只表示当前源码已有可保留底座，不能作为工程系统验收。

## 11. 实施所有权、非目标与当前裁决

Editor03拥有transaction core与统一command/history硬切；Editor61提供DocumentKey/session/world lifecycle；Runtime object/reflection owner提供generational identity和mutation preflight；Editor09提供async execution；Editor59/60迁移gizmo与selection；Editor02负责save/autosave/recovery和父finding关闭。各owner必须通过typed contract连接，不能复制registry、task runtime或dirty authority。

本报告不要求把Runtime gameplay mutation全部变成Editor Undo，不复制Unreal UObject transaction serialization，不把Bevy deferred queue误称为Undo，不推断Unity私有核心，不重新审查Tooling。目标是在Editor authoring边界形成可证明的单一transaction/history/savepoint authority。

当前裁决：

1. **保留**：RAII scope、rollback/compensation/fault、history cursor/save token、route retention、exclusive transition、Animation统一history、durable journal framing/checksum/replayer基础。
2. **停止扩展为默认方案**：弱DocumentId history、单EditContext/active/group/gate、裸NodeId command、String group、同步外部I/O command、UI Asset私有stack与未接线journal。
3. **实施第一步**：先恢复一致current source并建立M0动态RED证据，再交付DocumentTransactionSession和qualified identity；不能继续通过增加command类型扩大错误World重放面。
4. **最终资格**：必须同时证明identity、ownership、rollback、memory、savepoint、async cancellation、journal recovery、plugin teardown、multi-document并发、fault与规模性能，而不是只证明一次Undo成功。
