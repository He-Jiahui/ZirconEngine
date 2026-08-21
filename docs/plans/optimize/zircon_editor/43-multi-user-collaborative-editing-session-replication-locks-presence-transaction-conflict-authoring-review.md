---
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/document/lifecycle.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/recovery/session_guard
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_runtime_interface/src/ui/event_ui/control.rs
  - zircon_runtime_interface/src/project/session_lock
  - zircon_hub/src/team/local_git.rs
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/web/src/pages/TeamPage.tsx
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
  - docs/plans/optimize/zircon_editor/27-project-operations-source-control-changelist-diff-automation-report-submit-gates-health-dashboard-review.md
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertMain/Source/Concert/Public/ConcertMessageData.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertMain/Source/Concert/Public/ConcertMessages.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertMain/Source/Concert/Public/IConcertSession.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncCore/Source/ConcertSyncCore/Public/ConcertTransactionEvents.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncCore/Source/ConcertSyncCore/Public/ConcertWorkspaceData.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncCore/Source/ConcertSyncCore/Public/ConcertWorkspaceMessages.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncCore/Source/ConcertSyncCore/Public/ConcertPresenceEvents.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncCore/Source/ConcertSyncCore/Public/ConcertSyncSessionDatabase.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncCore/Source/ConcertSyncCore/Public/Replication/Data/AuthorityConflict.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncCore/Source/ConcertSyncCore/Public/Replication/Data/ObjectReplicationMap.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncCore/Source/ConcertSyncCore/Public/Replication/Data/ReplicationFrequencySettings.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncCore/Source/ConcertSyncCore/Public/Replication/Messages/ChangeAuthority.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncClient/Source/ConcertSyncClient/Public/IConcertClientWorkspace.h
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncClient/Source/ConcertSyncClient/Private/ConcertClientLockManager.cpp
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncClient/Source/ConcertSyncClient/Private/ConcertClientLiveTransactionAuthors.cpp
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncClient/Source/ConcertSyncClient/Private/ConcertClientPresenceManager.cpp
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncClient/Source/ConcertSyncClient/Private/ConcertClientTransactionManager.cpp
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncClient/Source/ConcertSyncClient/Private/ConcertClientWorkspace.cpp
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertSync/ConcertSyncServer/Source/ConcertSyncServer/Private/ConcertServerWorkspace.cpp
  - dev/UnrealEngine/Engine/Plugins/Developer/Concert/ConcertApp/MultiUserClient/Source/MultiUserClient/Private/Replication/Misc/GlobalAuthorityCache.cpp
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/godot/editor/debugger/editor_debugger_tree.cpp
  - dev/godot/editor/debugger/editor_debugger_inspector.cpp
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/Fyrox/editor/src/message.rs
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/builtin_methods.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeProfileEditor.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 43 · Multi-User / Collaborative Editing / Session Replication / Locks / Presence / Transaction Conflict Authoring 工程化差距

## 1. 结论

Zircon当前没有工程级Multi-User或Collaborative Editing产品。production Editor、Runtime、App、Plugin与Runtime Interface中没有协同session、participant、presence、durable activity、server sequencer、distributed lock/authority、remote transaction replay、reconnect/resync或conflict resolution owner。Hub的Team页读取本地Git身份与近期提交者，邀请、权限和Remote Collaboration则明确为disabled/Coming Soon；这是诚实的本地团队概览，不能计为实时协同，也不应被删除。

当前Editor transaction engine不是临时mock。它已有RAII scope、全局/文档history、undo/redo失败恢复、selection回滚、dirty generation、save token、operation gate/group、bounded history detail及事件投影。`EditCommand`还提供`apply/revert/finalize/try_merge/journal_payload`，四类Scene command会输出versioned JSON payload。这个底座值得保留，但其全部身份、顺序、lineage和错误恢复都限定在一个本地engine instance。

`TransactionId(u64)`只是本进程单调序号，`HistorySaveToken`通过`Arc<()>`判断同一engine lineage，`TransactionEvent`没有author、endpoint、session、global sequence、base revision、affected object/property或ack。`participants`实际是受影响的`DocumentId`集合，不是协同参与者；`try_merge`只合并相邻本地`UpdateNodeCommand`，不是并发合并。把这些同名字段直接提升成wire contract会制造错误架构。

`TransactionJournal`只完成编码和top-level schema检查。production没有command decoder、replay registry或`decode`消费者；测试仅验证JSON roundtrip和payload形状。更关键的是`UpdateNodeCommand`与`SetReflectedSceneFieldCommand`执行时不验证当前值仍等于`before`，因此远端重放会退化为静默last-writer-wins。原始`NodeId`、由本机project path派生并碰撞探测的`DocumentId`也不具备跨客户端稳定性。

项目`SessionOwnershipLease`是另一项必须保留但不能误用的基础：Windows named mutex和Unix directory flock保证同一物理project root只有一个Editor修改者，并配合PID/instance/heartbeat处理残留lock。启用Multi-User不能通过删除此lease让多个进程共享同一checkout；正确模型是每个participant拥有独立workspace/sandbox/overlay，再由session authority交换typed activities和asset/package结果。

Editor08已经证明`UiControlRequest::InvokeBinding/InvokeRoute`可绕过`callable_from_remote`并改写provenance。协同transport若复用这条raw control route，会把已知本地控制面缺陷扩大为远端写权限问题。协同命令必须经过统一`InvocationPrincipal + SourceProvenance`、deny-by-default authorization、typed codec、precondition和server acceptance，不能发送trait object、任意UI route或无界JSON。

目标架构应由`CollaborativeEditingProvider`、`CollaborativeSessionService`、server-authoritative `ActivityLog`、`CollaborativeTransactionCodecRegistry`、`ResourceAuthorityService`、`PresenceService`和`CollaborationWorkspaceManager`组成。会话负责admission、baseline compatibility、initial sync、ordered durable activity、transient presence、reconnect/resync、checkpoint/archive与audit；Editor42的snapshot/diff/three-way merge负责冲突工件，Editor27的VCS负责repository/file revision，两者都不被协同层复制。

本报告登记5个P0、70个P1、12个P2、M0-M11重构路线和32个验收门。它只做review，不修改Editor、Runtime、Runtime Interface、Hub、Plugin或reference生产代码和tests。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | test attributes / ignored / 在途 | 审查方式 |
|---|---:|---:|---|
| Zircon Editor transaction/document/session foundation | 59 / 11,308 / 383,541 | 52 / 0 / 0 | E3逐command、history、scope、journal、event sink、document identity、session guard与control route |
| Zircon focused transaction/message tests | 25 / 4,575 / 156,052 | 111 / 0 / 0 | E3逐commit/undo/redo/failure recovery、journal roundtrip、paging、bus backpressure |
| Zircon Hub Team与reserved collaboration | 10 / 5,067 / 180,966 | 19 / 0 / 0 | E3逐local Git projection、Team page、Coming Soon和UI contract |
| Unreal Concert / Multi-User参考 | 251 / 37,330 / 1,456,512 | 0 / 0 / 0 | E2/E3逐session、transport、workspace/activity DB、transaction/package/lock/presence/replication authority及UI |
| Godot local undo与remote debugger参考 | 6 / 2,025 / 72,878 | 0 / 0 / 0 | E2/E3确认multi-history undo和remote runtime tree/inspector边界 |
| Fyrox local command参考 | 2 / 815 / 26,174 | 0 / 0 / 0 | E2/E3逐trait-object command stack/group/message channel |
| Bevy Remote参考 | 6 / 5,547 / 196,482 | 17 / 0 / 0 | E2/E3逐JSON-RPC、typed request/error、ECS mutation/watch与transport边界 |
| Unity Graphics local Volume Editor参考 | 2 / 987 / 39,509 | 0 / 0 / 0 | E2确认SerializedObject/Undo局部authoring边界，不推断closed-source协同实现 |
| selected combined scope | 361 / 67,654 / 2,512,114 | 199 / 0 / 0 | 当前工作树fingerprint `43b379c8709c7023f9a83bfe2a3cdd3a702e1f3aae9951001df5e2917afd08eb` |

指纹算法为：对361个选择路径按PowerShell `Sort-Object`排序，逐文件计算小写SHA-256，形成`forward/slash/path|file_sha256`行，以单个LF连接且末尾不追加LF，再对UTF-8无BOM payload计算SHA-256。选择规则覆盖完整Editor editing/document、focused transaction/bus tests、project session guard、Hub Team投影、Concert public contracts与关键client/server/replication/UI实现，以及指定Godot/Fyrox/Bevy/Unity参考；缺失路径0、重复路径0、范围内在途文件0。

199个Rust test attributes主要证明本地transaction、history、journal序列化、message bus、Hub projection及Bevy Remote局部行为。它们不证明两个Editor进程之间的admission、identity、ordering、ack/idempotency、lock race、conflict、reconnect、recovery、authentication或scale，不能替代本报告验收门。

### 2.2 名称检索与产品边界

1. production中没有`MultiUser`、`CollaborativeSession`、`PresenceService`、`ActivitySequence`或等价owner。
2. Hub `TeamPage`展示repository path、本地Git identity、contributors和Hub action history，不展示live participants。
3. Hub `team-invite`、`team-permissions`与`remote-collaboration`均为disabled Coming Soon，文案明确依赖未存在的remote service。
4. `TransactionRecord::participants`保存`BTreeSet<DocumentId>`，语义是一个transaction涉及的文档，不是用户列表。
5. `ProjectSessionId`用于拒绝项目切换后的stale picker/document route，不是网络会话ID。
6. `SessionOwnershipLease`是本机文件所有权互斥，不是resource lock、membership或distributed lease。
7. `MergeMode`与`EditCommand::try_merge`用于本地gesture coalescing，不是多方冲突合并。
8. `TransactionEventSink`投影到本地EditorMessage bus，不是可靠transport、event store或replication stream。
9. `TransactionJournal`是可序列化观察工件，不是可执行wire command或灾难恢复log。
10. Editor26的gameplay replication/online session服务Runtime玩家，不提供Editor authoring convergence。
11. Editor27的VCS changelist/diff处理repository和文件版本，不提供sub-frame live object/property同步。
12. Editor42的snapshot/diff/merge可成为rebase/conflict输入，但没有participant、transport或ordered activity owner。
13. Godot remote debugger与Bevy Remote允许检查/修改运行中world，但不等于协同编辑。
14. Unity Graphics选集只证明本地Volume serialized authoring，不能用仓内无命中推断Unity Editor整体无协同能力。
15. Unreal Concert提供职责参考；Zircon应吸收session/activity/lock/presence/authority边界，而不是复制UObject、Slate或Concert类层次。

### 2.3 本地transaction engine的真实能力

1. `EditCommand`以`apply/revert`表达可逆操作，以`CommandEffect`区分失败前是否已产生效果。
2. undo和redo逐command执行；中途失败会反向恢复已处理命令并恢复原selection。
3. rollback再次失败会令engine进入faulted状态，阻止继续静默操作。
4. transaction scope支持nested同history操作、operation gate、operation group和drop/cancel语义。
5. history按Global或`DocumentId`隔离，拥有capacity、top、saved top、generation、dirty和paged detail。
6. `HistorySaveToken`能检测同一engine内save期间history变化，避免本地保存误标clean。
7. transaction event区分Started/Canceled/Committed/Undo/Redo，产品builder将其投影到bounded local bus。
8. bus会报告Delivered/Backpressured/Rejected；engine当前仅写warning，未把delivery失败变成transaction durability状态。
9. detached sink即使不保留event也返回Delivered，说明event delivery不是commit authority。
10. 这些能力适合作为accepted remote activity落地后的本地transaction执行器，不适合作为session server或durable log。

### 2.4 Journal、command codec与并发前置条件断点

1. `TransactionJournal` v1保存local transaction/history、label、frame、document participants、selection、significance和command payloads。
2. `CommandJournalPayload`只包含string command type、u16 schema version和`serde_json::Value`。
3. 任一command未实现`journal_payload`时，整个journal查询失败；这对完整性是正确的fail-closed基础。
4. `decode`只反序列化并验证top-level journal schema，不验证每个command type/version/payload。
5. production没有`TransactionJournal::decode`消费者；唯一decode调用位于roundtrip和unsupported schema测试。
6. production没有command decoder registry、migration registry、remote apply、remote revert或replay receipt。
7. Create payload保留intent与retained `NodeRecord`，Delete保留subtree records/camera/selection，Update保留before/after，reflected field保留type path/name/before/after。
8. `UpdateNodeCommand::apply`只检查node存在，再写parent/name/transform；它没有`current == before`的CAS。
9. `SetReflectedSceneFieldCommand::apply`直接写`after`；它同样不校验current value、component generation或schema field ID。
10. `NodeId`和leaf `field_name`不足以在跨scene instance、schema migration、数组/map元素及component replacement后稳定定位。
11. 直接广播当前payload会使两位用户同时修改同一property时按到达顺序覆盖，且被覆盖方没有conflict artifact。
12. 目标codec必须把`decode -> validate identity/schema -> validate preconditions -> authorize -> stage -> apply -> receipt`作为不可拆分协议。

### 2.5 Identity、workspace与本地lease

1. `TransactionId(u64)`从engine state的`next_transaction`分配，重启和多客户端都会重叠。
2. `HistorySaveToken`的lineage是process-local `Arc<()>`，不能序列化或跨连接验证。
3. `ProjectSessionId(u64)`来自static atomic，只表示当前进程中的项目激活generation。
4. `DocumentId(u64)`由project root或`scene:{project_root}:{scene_uri}`做FNV派生。
5. collision处理依赖当前打开文档集合并逐步探测，不同client的占用顺序可生成不同ID。
6. 不同机器checkout path不同，即使同一repository和scene也会派生不同`DocumentId`。
7. `ProjectSceneDocument`保存AssetUri/source path/world，但没有immutable project ID、workspace ID、baseline revision或collaboration revision。
8. session guard把resolved physical project root映射到Windows mutex，Unix则flock `.zircon`目录。
9. persisted lock record拥有PID、instance ID与heartbeat，用于检测active/residual owner。
10. 该lease正确地阻止两个Editor同时直接写同一物理root；协同实现必须把它保留在每个workspace内。
11. 每个participant应使用独立checkout、sandbox或overlay；server activity与package transfer通过显式commit同步。
12. 需要新建`ProjectIdentity/WorkspaceIdentity/SceneIdentity/ObjectIdentity`，不可把本地path、PID或raw entity ID放上wire。

### 2.6 Hub、control boundary与相邻owner

1. `discover_team_overview`读取本地`git config user.name/user.email`及`git log --format=%aN%x1f%aE`。
2. contributor按author identity聚合commit数并排序；这是repository facts，不是account、role或online presence。
3. Team页将reserved collaboration放在Coming Soon panel并保持disabled，没有伪造invite/session成功。
4. 后续Hub可承载provider account、organization和session discovery，但Editor session lifecycle不能由React页面拥有。
5. Editor08已登记`InvokeBinding/InvokeRoute` remote gate与provenance旁路；协同层必须等待统一InvocationGateway封口。
6. participant身份、transport认证、role permission与command capability必须分离，不能复用`callable_from_remote: bool`。
7. VCS baseline由Editor27提供`repository/workspace/revision`；live activity只引用baseline，不替代commit/changelist。
8. snapshot、change set和three-way merge由Editor42提供；collaboration只决定何时生成rebase/conflict plan及如何提交resolution。
9. background initial sync、checkpoint、package transfer与resync走Editor09 admission/quota/cancel/shutdown。
10. durable audit、security denial、transport health和conflict receipt走Editor11 diagnostic journal，不把敏感payload默认写日志。
11. gameplay networking与Editor collaboration可共享底层crypto/transport primitive，但必须保留不同protocol、identity、authority和lifecycle。
12. Hub Cloud/Auth未存在时，provider command应保持Unavailable；不得用anonymous LAN模式伪装企业级安全完成。

### 2.7 focused tests能证明与不能证明的内容

现有111个Editor focused test attributes覆盖nested transaction、rollback、history paging、save token、journal payload、bus capacity/backpressure和部分session guard行为；19个Hub测试覆盖本地Git/Team projection与disabled contract。它们证明基础并非空壳，但全部可在单进程或本地fixture内完成。

缺失的是两进程及以上的确定性资格：同时编辑同property或不同property、duplicate/reordered/delayed message、server reject、lock race、client crash、server crash、network partition、offline queue、baseline drift、schema/plugin mismatch、large package、resync checkpoint、principal denial和audit completeness。协同产品不能用更多单进程mock替代这些lanes。

## 3. 参考实现差异与吸收边界

### 3.1 Unreal Concert session与transport

`IConcertSession`公开session info、working directory、connected endpoint IDs/client info、custom event/request handler及sequenced custom event manager。`ConcertMessages.h`覆盖server/session discovery、create/find/copy/archive/rename、join与administration。这说明Multi-User首先是独立session product和protocol，不是Editor command bus加一个socket。

Zircon应吸收：

- session ID/name/owner/provider、endpoint/user/device identity和显式lifecycle；
- admission中的project、engine、schema、plugin、source baseline与capability negotiation；
- reliable request/event envelope、request ID、ack、ordered durable stream和backpressure；
- live/archived session、checkpoint、working storage、admin与disaster recovery职责分离。

Zircon不应照搬：

- UObject reflection、Slate widget、module singleton或Concert命名；
- 把参考实现当前身份模型视为足够的企业认证与授权；
- 假设LAN可信、payload可信或plugin catalog天然一致。

### 3.2 Concert transaction、workspace与activity database

`ConcertTransactionEvents.h`的transaction base携GUID transaction/operation ID、source endpoint与update index，并区分snapshot/finalized/rejected事件。`ConcertWorkspaceData.h`和`ConcertSyncSessionDatabase.h`把endpoint、connection、lock、transaction、package及replication event放入带activity ID的持久化数据库；client workspace暴露sync状态、activity stream、package/transaction处理和其他client修改资产查询。

这与Zircon当前local journal的核心差异是：

1. 事件具有跨endpoint身份与更新序列，而不是仅`u64 + frame`。
2. accepted activity有server-owned durable ordering，而不是本地bus delivery。
3. transaction与package activity能查询、恢复和忽略于特定restore流程。
4. workspace知道initial sync、live session和archive，不把join视为一个布尔connected。
5. save前能识别其他client仍有live transaction的asset。

Zircon应采用server-assigned `ActivitySequence`、client operation UUID、author/endpoint、base revision、causal parents、input digest、affected paths、accept/reject outcome和checkpoint lineage。具体数据库技术可独立选择，不需要复制Concert SQLite封装。

### 3.3 Concert locks、packages与presence

Concert lock manager维护resource到endpoint owner映射，通过server request/event异步lock/unlock，并让package save/delete gate检查owner。LiveTransactionAuthors跟踪仍被其他endpoint修改的package。Presence manager则按endpoint投影display/avatar、visibility、world path、transform、play mode及jump/follow。

必须吸收的边界：

- coarse asset/package lock适合不可合并资源和save/delete临界区；
- object/property authority适合高频live edit，不能把所有内容塞进全局文件锁；
- lock/authority由server仲裁，拥有lease/expiry/disconnect/reclaim与owner-visible denial；
- presence是transient、有TTL/频率/interest/privacy的流，不进入dirty、undo或durable transaction history；
- package payload与transaction metadata分离，大文件走checksum、quota和可恢复transfer。

不能把OS project mutex改造成distributed asset lock，也不能把Git file lock当作property authority。

### 3.4 Concert property replication authority

Concert replication以stream和object replication map选择object/property，拥有frequency setting、authority request/release、overlap conflict和global authority cache。不同client可对同一object的不重叠property持有authority，冲突响应返回rejected objects。

Zircon可以吸收property selection、frequency control、disjoint authority和明确拒绝结果，但不应把协同编辑简化为持续复制完整反射对象。authoring transaction仍需要stable property address、before/base precondition、undo semantics、schema migration和durable finalized activity；高频snapshot只作为临时preview，最终commit才进入历史。

### 3.5 Godot与Fyrox本地command边界

Godot `EditorUndoRedoManager`管理多history本地undo/redo，remote debugger tree/inspector服务运行时检查。Fyrox `CommandTrait/CommandStack/CommandGroup`提供execute/revert/finalize与local message sender。它们证明成熟本地command architecture仍不自动形成collaboration：

- 本地history index不等于global activity sequence；
- remote inspector ID不等于authoring stable identity；
- mpsc或UI message不等于authenticated durable transport；
- reversible command不等于可跨版本解码、校验并server reject的wire operation。

Zircon应保留当前优于简单command stack的rollback/save token能力，并在外层增加session协议，而不是重写一套更弱的undo stack。

### 3.6 Bevy Remote与Unity Graphics边界

Bevy Remote Protocol基于JSON-RPC，可经可选HTTP transport执行query/get/spawn/despawn/insert/remove/mutate/reparent/watch，提供typed request/error和registry discovery。它适合参考request envelope、method registry和明确错误，但没有author、base revision、global ordering、undo、conflict、presence或collaborative save。允许raw ECS mutation的API在Zircon中只能是受限diagnostic/automation surface，不能成为协同transaction transport。

Unity Graphics Volume Editor只展示`SerializedObject.Update/ApplyModifiedProperties`等本地authoring调用。本仓不包含Unity Editor完整协同源码，因此只把它作为“局部序列化编辑不等于协同”的边界证据，不作能力强弱推断。

### 3.7 必须吸收与禁止照搬

必须吸收：

1. 独立session/client/server/workspace/activity职责。
2. admission compatibility与initial sync barrier。
3. server ordering、durable activity、checkpoint/archive/recovery。
4. author/endpoint/global sequence/base revision/acceptance identity。
5. package lock、property authority、live author和save coordination。
6. presence的transient、TTL、interest与privacy边界。
7. typed request/error、ack/idempotency、quota与observability。
8. disconnected/reconnect/resync与明确degraded mode。

禁止照搬：

1. raw engine object、trait object、UI route或反射JSON直接远程执行。
2. 单一万能lock、万能CRDT或无冲突last-writer-wins。
3. 共享同一physical checkout以规避workspace设计。
4. 只靠display name、endpoint GUID或LAN发现完成身份安全。
5. 用presence event填充durable history，或用activity log驱动每帧cursor。
6. 把source control、gameplay network、remote debugger和collaboration合成一个protocol。

## 4. 差距登记

### 4.1 P0 阻断项

1. **P0-01** 在authenticated session admission、project/baseline/schema/plugin compatibility、secure transport及initial sync/resync完成前，Multi-User join/create与任何remote authoring command必须保持Unavailable。
2. **P0-02** 禁止远程执行或复制当前`TransactionJournal`、`EditorEvent`、`UiControlRequest`、raw reflection JSON或`Box<dyn EditCommand>`；当前没有decoder/precondition/principal-safe provenance，且Editor08存在已知control旁路。
3. **P0-03** 禁止删除、绕过或弱化`SessionOwnershipLease`来允许多个Editor共享同一physical project root；每个participant必须拥有独立workspace/sandbox/overlay及显式同步协议。
4. **P0-04** 在qualified cross-client identity、base revision/precondition、resource lock/property authority和typed conflict outcome完成前，禁止把当前command apply用于并发远端编辑；不得以silent last-writer-wins作为默认策略。
5. **P0-05** 在server-assigned ordering、durable activity log、ack/idempotency、checkpoint/recovery、accept/reject receipt和rejection rollback完成前，远端activity不得进入本地history、save或package commit。

### 4.2 P1 工程化必做

#### 4.2.1 Identity、session与admission

1. **P1-01** 建立`CollaborationSessionId`、`ParticipantEndpointId`、`UserIdentity`和`DeviceInstanceId`，全部使用不可碰撞wire identity。
2. **P1-02** 建立immutable `ProjectIdentity`、`RepositoryIdentity`、`WorkspaceIdentity`、`SceneIdentity`和`AssetIdentity`。
3. **P1-03** 禁止用local path、PID、`ProjectSessionId`、`DocumentId`、`NodeId`或display name作为跨client authority。
4. **P1-04** 定义session lifecycle：Discovering/Joining/Admitting/InitialSync/Live/Degraded/Reconnecting/Resyncing/Leaving/Archived/Failed。
5. **P1-05** admission baseline包含engine build、protocol、schema catalog、plugin set、project manifest、source branch/revision和content digest。
6. **P1-06** negotiation返回accepted capabilities、disabled features、incompatibility reason和required remediation，不做best-effort静默加入。
7. **P1-07** 分离user account、endpoint、device、workspace和reconnect identity，明确重启/换机/重复登录规则。
8. **P1-08** 建立role/capability matrix，至少区分owner/admin/editor/reviewer/observer及session/package/lock/command权限。
9. **P1-09** session create/join/copy/archive/rename/delete均返回versioned receipt并进入audit。
10. **P1-10** initial sync完成前Editor authoring保持read-only或隔离queue，不允许半基线写入live world。

#### 4.2.2 Transport、server与durable activity

11. **P1-11** 建立provider-neutral `CollaborationTransport`，支持authenticated/encrypted connection、capability negotiation和typed channel。
12. **P1-12** durable activity、request/response、package transfer和presence使用不同QoS/channel，不共享无界队列。
13. **P1-13** 所有envelope携protocol version、session/endpoint、request ID、deadline、idempotency key、payload type/version和byte length。
14. **P1-14** server为accepted durable event分配严格单调`ActivitySequence`，client local sequence只用于重试关联。
15. **P1-15** 建立ack/nack、duplicate suppression、retry window、out-of-order buffer和gap detection。
16. **P1-16** activity record保存client operation UUID、author/endpoint、base revision、causal parents、input digest、affected identities和outcome。
17. **P1-17** 建立append-only durable store、transaction boundary、checksum、schema migration和crash-consistent commit。
18. **P1-18** 建立checkpoint/snapshot、log compaction、retention、archive、restore和disaster recovery，并与Editor42 artifact适配。
19. **P1-19** server拒绝、timeout、disconnect和partial package transfer都有typed terminal state，client不得猜测成功。
20. **P1-20** transport/server实现per-session/client/channel byte、message、CPU、storage和in-flight quota及公平调度。

#### 4.2.3 Transaction codec、replication与local integration

21. **P1-21** 建立`CollaborativeTransactionEnvelope`，分离client intent、server accepted activity和local application receipt。
22. **P1-22** 建立versioned command codec registry，注册type ID、schema、decoder、validator、migrator、apply/revert builder和owner lease。
23. **P1-23** 未知command/plugin/schema必须Rejected或Deferred with reason，禁止忽略后继续提交。
24. **P1-24** 所有object/component/property地址使用Editor41/42共享的qualified stable identity和stable field path。
25. **P1-25** 每个mutation携expected object/component/property revision或before hash，执行前做CAS/precondition。
26. **P1-26** transaction validation区分Decode/Compatibility/Authorization/Identity/Precondition/Invariant/Apply/Commit阶段。
27. **P1-27** client optimistic apply只能在policy允许时发生，并保留完整rollback state与pending UI状态。
28. **P1-28** server reject或canonical result不同时，client通过一项本地reconciliation transaction回滚/重放，不直接改world。
29. **P1-29** remote accepted activity通过现有transaction engine落地，但使用qualified remote provenance且不伪造成local undo owner。
30. **P1-30** 明确local undo语义：生成compensating collaborative transaction，不删除或重排shared activity history。
31. **P1-31** gesture期间发送有界preview snapshot；finalized transaction才进入durable history，snapshot丢失不影响最终一致性。
32. **P1-32** journal/export可从accepted activity生成，但当前`TransactionJournal`不得成为canonical replay格式。

#### 4.2.4 Lock、authority与conflict model

33. **P1-33** 建立`ResourceAuthorityService`，同时支持coarse asset/package lock、object lease和property authority。
34. **P1-34** lock/authority由server仲裁，record包含owner endpoint/user、scope、lease epoch、expiry、reason和activity sequence。
35. **P1-35** acquire/release/renew/reclaim是idempotent request，disconnect、timeout、server restart和admin revoke有确定语义。
36. **P1-36** 同一object的不重叠property可并行持有authority；overlap使用stable property prefix规则检测。
37. **P1-37** save/delete/rename/move/import/reimport前检查package lock及live transaction authors。
38. **P1-38** 定义stale precondition、same-property、delete-vs-edit、create/create identity、reparent/reparent和component topology冲突。
39. **P1-39** 定义asset/package/save、external dependency、source baseline、schema/plugin/codec mismatch冲突。
40. **P1-40** conflict outcome区分AutoRebased/RequiresChoice/Blocked/Unsupported/Rejected/Stale，并保留双方来源。
41. **P1-41** conflict artifact引用base/ours/theirs和activity sequence，交给Editor42产生typed diff/merge/resolution plan。
42. **P1-42** resolution重新提交前复核当前head、authority和input digests；过期resolution必须拒绝。

#### 4.2.5 Presence、interest与live UX

43. **P1-43** 建立独立`PresenceService`，record包含endpoint/user、status、scene/world、tool mode、selection摘要和viewport pose。
44. **P1-44** cursor、camera、transform preview和selection使用transient sequence、TTL、rate limit和last-value coalescing。
45. **P1-45** presence不得改变dirty、undo、activity log、save token或package revision。
46. **P1-46** 建立interest subscription，按project/scene/world/partition/asset/tool过滤，不向所有client广播全部状态。
47. **P1-47** selection/object identity不可见或未加载时返回descriptor/hidden state，不泄露无权限对象。
48. **P1-48** visibility、follow、jump、avatar/profile和location sharing有per-user privacy与session policy。
49. **P1-49** disconnect和stale presence按TTL清除；reconnect以新presence epoch发布，不复用旧cursor sequence。
50. **P1-50** UI展示Live/Degraded/Reconnecting/Out-of-sync/Read-only状态、latency和last accepted activity，不能只显示connected绿点。

#### 4.2.6 Workspace、asset/package、save与offline

51. **P1-51** 建立`CollaborationWorkspaceManager`，每participant使用独立checkout/sandbox/overlay并保留本地OS lease。
52. **P1-52** workspace绑定repository/source baseline、project manifest和session checkpoint，路径只是local projection。
53. **P1-53** package transfer使用content digest、size、chunk/checkpoint、compression policy、quota和atomic publication。
54. **P1-54** nonmergeable binary asset必须lock-first；mergeable text/scene asset仍需revision CAS和semantic merge。
55. **P1-55** save coordinator检查pending remote activity、live authors、lock owner、workspace head和disk source revision。
56. **P1-56** accepted save生成package activity与immutable receipt，失败不得先mark history clean。
57. **P1-57** import/reimport/derived artifact变更携recipe/source/artifact digests，避免不同机器生成结果静默分叉。
58. **P1-58** offline mode明确只读或bounded local queue；queue记录base checkpoint、expiry、bytes和rejoin policy。
59. **P1-59** reconnect先gap detection和resync，再尝试rebase offline intent；不能按本地timestamp直接插回history。
60. **P1-60** VCS submit/pull/branch switch由Editor27协调session freeze、baseline advance和workspace rebase，不在collaboration层直接执行Git。

#### 4.2.7 Product、security、observability与qualification

61. **P1-61** Hub实现provider account/org/session discovery入口；Editor实现session browser、join progress、participant和activity产品面。
62. **P1-62** Outliner/Inspector/asset browser/property row显示remote selection、lock/authority owner、pending/conflict和follow/jump。
63. **P1-63** activity history支持sequence/author/type/object/asset/time/outcome筛选与stable paging，不把presence混入。
64. **P1-64** conflict center展示typed base/ours/theirs、dependency、owner和resolution receipt，并链接Editor42结果。
65. **P1-65** 所有remote mutation经过统一`InvocationPrincipal + SourceProvenance`和deny-by-default policy，不复用raw UI route。
66. **P1-66** untrusted payload在allocation前执行frame/byte/depth/node/string/collection限制，codec/plugin执行有deadline和panic isolation。
67. **P1-67** authentication credential、token、private payload和user PII不进入普通日志；audit保留必要identity、decision和digest。
68. **P1-68** 建立session/transport/store/codec/lock/presence/package指标、bounded trace、health snapshot和operator diagnostic。
69. **P1-69** initial sync/checkpoint/package/resync使用Editor09 job admission、cancel、progress、quota、result retention和shutdown barrier。
70. **P1-70** 建立2/4/16/64 client behavior/fault/compat/performance矩阵，并将核心两进程故障lane设为required。

### 4.3 P2 纵深能力

1. **P2-01** 提供LAN relay、dedicated server与managed service provider，但共享同一session/identity/activity合同。
2. **P2-02** 支持跨region relay、resume token和large-session edge cache，不改变server acceptance authority。
3. **P2-03** 提供review-only branch/session、comment、approval与change proposal工作流。
4. **P2-04** 对适合的数据类型引入经过证明的CRDT/OT adapter；不把万能CRDT作为所有asset默认策略。
5. **P2-05** 支持partition-aware interest、lazy checkpoint和unloaded object descriptor，服务超大世界。
6. **P2-06** 提供remote viewport ghost、laser pointer、camera follow和presentation mode。
7. **P2-07** 支持session fork、checkpoint compare、archive replay和time-travel inspection。
8. **P2-08** 提供admin console、participant moderation、quota policy、audit export和retention governance。
9. **P2-09** 支持package/content dedup、delta transfer、regional cache和bandwidth adaptation。
10. **P2-10** 建立plugin collaborative codec/authority/conflict resolver认证套件与兼容矩阵。
11. **P2-11** 提供自动化client/bot API，但使用独立principal、role、rate limit和visible audit。
12. **P2-12** 在真实大型项目与跨地域场景建立并维护优于参考引擎的join、edit、save、reconnect和scale基线。

## 5. 目标架构

### 5.1 Ownership

| Owner | 唯一职责 | 禁止拥有 |
|---|---|---|
| `CollaborativeEditingProvider` | provider discovery、account/org/session catalog、create/join/archive admin | world mutation、transaction apply |
| `CollaborativeSessionService` | admission、lifecycle、channel、head/checkpoint、participant membership | Git命令、UI rendering |
| `CollaborationServer` | authentication hook、ordering、authorization、activity/package/authority acceptance | client local history |
| `ActivityLog` | durable accepted activity、checkpoint、query、archive/recovery | presence/cursor |
| `CollaborativeTransactionCodecRegistry` | typed decode/validate/migrate/apply/revert construction | transport、UI |
| `ResourceAuthorityService` | package/object/property lock/lease/authority | OS checkout mutex |
| `PresenceService` | transient participant state、TTL、interest、privacy | dirty、undo、durable history |
| `CollaborationWorkspaceManager` | local checkout/sandbox/overlay、baseline、package publication | session ordering |
| Editor transaction engine | accepted/local intent的可逆本地执行、selection/dirty/history | remote admission/order |
| Editor42 snapshot/diff/merge | base/ours/theirs change/conflict/resolution artifact | participant/session transport |
| Editor27 VCS | repository/workspace/revision/changelist/submit | live object replication |

### 5.2 Canonical identities与revision

```text
CollaborationSessionId
  -> ParticipantEndpointId + UserIdentity + DeviceInstanceId
  -> ProjectIdentity + RepositoryIdentity + WorkspaceIdentity
  -> SceneIdentity / AssetIdentity
  -> ObjectIdentity / ComponentIdentity / StablePropertyAddress

SessionHead
  = ActivitySequence
  + CheckpointId
  + source baseline digest
  + schema/plugin/catalog digest
```

本地`DocumentId`、`NodeId`和filesystem path只在adapter内部解析到qualified identity。所有wire mutation绑定base head和precondition；server acceptance推进head，presence不推进head。

### 5.3 Durable transaction flow

```text
local intent
  -> principal/policy
  -> stable identity + before hash
  -> codec encode + idempotency key
  -> server decode/authorize/precondition/authority
  -> accepted ActivitySequence or typed rejection
  -> client ordered apply/reconcile
  -> local transaction/history/dirty projection
  -> immutable receipt + audit
```

trait object、raw reflection JSON和UI route不跨wire。若使用optimistic apply，pending transaction始终可定位、可回滚，并在accepted canonical activity到达后收敛。

### 5.4 Preview、finalized activity与presence

高频gizmo/property drag拆成三层：

1. local immediate preview，不出进程也能保持输入响应；
2. rate-limited transient snapshot，携gesture ID/epoch/TTL，只影响其他client预览；
3. finalized durable transaction，携base/precondition并进入server activity。

snapshot乱序或丢失只影响短暂视觉；finalized activity必须可靠、有序、可拒绝。cursor、selection、viewport和follow同样属于presence，绝不进入undo或save。

### 5.5 Conflict与save

server检测precondition或authority失败后输出typed conflict descriptor。client以checkpoint/base、本地intent和当前head请求Editor42生成`SceneChangeSet/SceneMergeResult`；用户resolution作为新transaction重新提交。save coordinator同时检查session head、pending local/remote activity、live authors、package lock、workspace baseline和disk revision，成功后才推进saved state并发布package activity。

## 6. 分阶段重构路线

### M0 - 真实性封口

- 保持Hub invite/permissions/remote collaboration disabled。
- 明确`participants`、`ProjectSessionId`、`SessionOwnershipLease`和`try_merge`的本地语义。
- 禁止任何实验transport执行raw UI route、`TransactionJournal`或trait-object command。
- 将Editor08 remote principal/provenance封口列为协同前置。

### M1 - Identity、baseline与provider contract

- 引入session/user/endpoint/device/project/workspace/scene/asset/object/property identities。
- 定义provider/session lifecycle和versioned admission contract。
- 接入Editor27 repository baseline及schema/plugin/catalog digest。
- 为Hub和Editor建立Unavailable/compatibility projection。

### M2 - Secure transport与server skeleton

- 建立authenticated/encrypted provider-neutral transport。
- 实现versioned envelope、request ID、deadline、idempotency、ack/nack和quota。
- 分离durable/request/package/presence channel。
- 建立单节点server acceptance与operator health。

### M3 - Activity store、checkpoint与recovery

- 实现server-assigned `ActivitySequence`和append-only durable store。
- 保存author/base/causal/digest/affected identity/outcome。
- 建立checkpoint、compaction、archive、restore和crash recovery。
- 用gap detection驱动initial sync/reconnect/resync。

### M4 - Transaction codec与precondition

- 建立codec/validator/migrator/owner registry。
- 将Scene commands切换到stable identity与stable field address。
- 增加before hash/revision CAS和typed rejection。
- accepted activity通过现有transaction engine落地并输出receipt。

### M5 - Lock、authority与package save

- 实现asset/package lock、object lease和property authority。
- 接入save/delete/rename/import/reimport gates与live authors。
- 建立lease expiry/reclaim/disconnect/server restart语义。
- 完成package checksum/chunk/atomic publication。

### M6 - Presence与interest

- 实现participant/status/scene/tool/selection/viewport transient schema。
- 增加TTL、rate/coalescing、interest和privacy。
- 完成Outliner/Inspector/viewport remote projection及follow/jump。
- 证明presence不触发dirty/history/save。

### M7 - Conflict、rebase与offline

- 定义全部object/property/hierarchy/package/schema conflict。
- 接入Editor42 base/ours/theirs artifact和resolution plan。
- 实现optimistic rejection rollback、compensating undo和offline queue policy。
- reconnect按checkpoint/head rebase，不按timestamp插入。

### M8 - Workspace与VCS integration

- 实现独立checkout/sandbox/overlay manager并保留OS lease。
- 接入Editor27 branch switch/pull/submit freeze和baseline advance。
- 处理derived artifact digest、project move和workspace cleanup。
- 建立session fork、archive与local residual recovery。

### M9 - Product UX与administration

- 完成Hub provider/session入口及Editor active session/participant/activity/conflict UI。
- 显示authority owner、pending、degraded、out-of-sync和read-only原因。
- 建立role/admin/moderation、audit query与retention controls。
- 所有不可用操作基于capability而非点击后假成功。

### M10 - Security、observability与scale

- 完成credential/PII/logging policy、payload budget、plugin isolation和abuse limits。
- 建立transport/store/codec/lock/presence/package指标及bounded trace。
- 优化interest、coalescing、frequency、package dedup和checkpoint transfer。
- 建立4/16/64 client及跨地域基线。

### M11 - Qualification与硬切

- required两进程behavior/fault/compat/security lanes全部通过。
- crash/reconnect/partition/duplicate/reorder/schema mismatch/package failure均有确定receipt。
- 删除实验raw transport、legacy identity和任何last-writer-wins旁路。
- 只有完整provider/session/server/workspace产品达到资格后才启用Multi-User命令。

## 7. 验收门

1. 无provider、未认证、baseline不兼容或initial sync未完成时，所有remote mutation不可执行且原因可见。
2. remote principal经CallAction/InvokeRoute/InvokeBinding/Menu/Operation任一路径都不能绕过deny policy。
3. wire payload不包含trait object、raw UI route、本地path、PID、`DocumentId`或raw `NodeId` authority。
4. 两client checkout path不同仍解析到相同project/scene/object/property identity。
5. 两client同时加入后只在完整checkpoint和activity gap补齐后进入Live。
6. duplicate request只产生一项accepted activity和一个package effect。
7. reordered/delayed durable message被buffer或resync，不以到达顺序静默改写head。
8. activity store commit后server crash/restart不丢accepted activity，也不重复sequence。
9. client crash/rejoin从checkpoint和gap恢复到与server相同head/digest。
10. simultaneous same-property edits产生一个accept和一个typed conflict/rebase outcome，不silent overwrite。
11. simultaneous disjoint-property edits可按policy并行且最终所有client一致。
12. delete-vs-edit、reparent-vs-reparent和component topology冲突都有typed artifact。
13. stale schema/plugin/codec command在apply前被拒绝，world/history/dirty不改变。
14. optimistic command被server拒绝后完整rollback，selection与pending UI也收敛。
15. collaborative undo生成compensating activity，不删除共享历史且所有client一致。
16. package lock race只有一个owner；disconnect/expiry/reclaim后无双owner窗口。
17. save检查pending activity、live authors、lock、session head、workspace和disk revision。
18. save失败不先mark clean，不发布成功package activity，也不丢本地history。
19. package transfer中断可恢复或原子失败，目标不会出现半文件。
20. 两Editor不会共享同一physical root；每个workspace的`SessionOwnershipLease`仍有效。
21. presence packet丢失、乱序、过期只影响短暂显示，不改变activity head、dirty或undo。
22. privacy/permission关闭时remote selection/location不可见且不从其他API泄露。
23. network partition进入明确Degraded/Read-only或bounded queue状态，不显示假Live。
24. reconnect先resync再rebase offline intent，过期base不会直接写入head。
25. source branch/revision advance由VCS/session协调，未同步client不能继续写旧baseline。
26. activity query按sequence稳定分页，presence不进入结果，stale cursor有typed结果。
27. payload byte/depth/node/string/collection与rate quota在业务codec执行前生效。
28. unknown/malicious plugin codec不会令server或Editor进程崩溃，也不会留下partial activity。
29. audit能关联principal/endpoint/request/client operation/activity/command/affected identity/outcome，且不记录敏感payload。
30. 2/4/16/64 client基线记录join time、accepted edit latency、presence latency、server CPU/memory/storage/network和resync time。
31. core两进程behavior、message fault、process crash、compatibility与security lanes均为required且无ignored。
32. Multi-User启用清单逐项证明provider、admission、transport、store、codec、authority、presence、workspace、conflict、UX与qualification全部达标。

## 8. 风险、依赖与实施顺序

1. 第一依赖是Editor08统一InvocationGateway；在principal/provenance旁路存在时接remote transport会扩大安全面。
2. 第二依赖是Editor41/42 stable identity、snapshot/diff/merge；没有它们只能依赖raw entity和leaf field name。
3. 第三依赖是Editor27 repository/workspace/baseline及Editor02 save/recovery hardening。
4. server activity和client local history必须分层；强行共用`TransactionId`会在undo、reconnect和archive时形成歧义。
5. property authority不能替代transaction precondition；持有authority期间仍可能发生schema、object lifetime或baseline变化。
6. coarse lock不能作为全部并发模型；否则大型团队会被单资产/单scene串行化。
7. 也不能把全部数据宣称为CRDT；scene hierarchy、binary assets、plugin-defined state和save side effects需要不同策略。
8. presence与durable activity必须从schema、queue、storage到UI完全分流，否则每帧状态会污染history和带宽。
9. 本地OS lease不是障碍而是数据安全底线；workspace architecture完成前不得开展共享root试验。
10. 性能目标应按join、edit acceptance、preview、save、reconnect和scale分别定义，不以单个ping值代替产品性能。

建议实施顺序严格为`M0 -> M1 -> M2 -> M3 -> M4 -> M5 -> M6 -> M7 -> M8 -> M9 -> M10 -> M11`。M4之前不允许remote mutation，M5之前不允许协同save，M7之前不允许offline write，M11之前不公开宣称工程级Multi-User完成。

## 9. 验证说明

本轮只做静态review和文档变更，没有修改production Editor、Runtime、Runtime Interface、Hub、Plugin、App代码或tests，也没有启动network/server/多进程动态验证。选集361文件、67,654行、2,512,114 bytes，fingerprint为`43b379c8709c7023f9a83bfe2a3cdd3a702e1f3aae9951001df5e2917afd08eb`，范围内在途文件0。

此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断；本轮没有重复无法抵达协同产品行为的同一lane。实施阶段必须先重导selection manifest并重算fingerprint，再补齐真正的2+ Editor process、server crash、message fault、partition/reconnect、compatibility/security和scale验证，不能把当前199个单进程/局部test attributes解释为协同资格。
