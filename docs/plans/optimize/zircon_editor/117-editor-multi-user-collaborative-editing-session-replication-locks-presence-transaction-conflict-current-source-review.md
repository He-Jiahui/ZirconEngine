---
title: Editor Multi-User、Collaborative Editing、Session Replication、Locks、Presence 与 Transaction Conflict 当前源码复核
category: zircon_editor
report_id: Editor117
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor43
refreshes:
  - docs/plans/optimize/zircon_editor/43-multi-user-collaborative-editing-session-replication-locks-presence-transaction-conflict-authoring-review.md
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

# 117 · Editor Multi-User / Collaborative Editing / Session Replication 工程化差距

## 1. 结论

当前 Zircon 没有工程级 Multi-User 或 Collaborative Editing 产品。Editor、Runtime、Hub、Plugin 和 Runtime Interface 中没有协同 session、participant、presence、durable activity、server sequencer、distributed lock/authority、remote transaction replay、reconnect/resync 或 conflict owner。Hub Team 页读取本地 Git identity 与 contributors，invite/permissions/remote-collaboration 明确为 disabled/Coming Soon；这是诚实的本地团队概览，不能计为实时协作。

本地 transaction engine 不是空壳：RAII scope、apply/revert、undo/redo 失败恢复、selection 回滚、dirty generation、save token、history paging、operation gate/group、bounded bus 和 versioned journal payload 都是真底座。它们的身份和顺序却限定在单一 engine instance：`TransactionId(u64)` 是进程内序号，`HistorySaveToken` 使用 process-local `Arc<()>` lineage，`participants` 实际是 DocumentId 集合，event 没有 author/endpoint/session/global sequence/base revision/affected property/ack。

`TransactionJournal` 只做 schema/JSON 编码，没有 production decoder、command registry、replay、remote apply 或 receipt。`UpdateNodeCommand` 与 `SetReflectedSceneFieldCommand` 执行时不校验当前值等于 `before`，远端重放会退化为静默 last-writer-wins；裸 `NodeId`、leaf field name 和由本地 path 派生的 `DocumentId` 都不能作为跨客户端 wire identity。

`SessionOwnershipLease` 的 named mutex/目录 flock 正确保证同一 physical project root 只有一个本地 writer，不能被删除来“实现协同”。每个 participant 必须使用独立 checkout/sandbox/overlay，由 session authority 交换 typed activities 与 package artifacts。协同 transport 也不能复用 Editor08 已知可绕过 remote gate 的 raw control route。

本轮选定 Zircon scope 为 86 files / 13,789 lines / 12,513 non-empty / 466,134 bytes / 78 test attributes；参考 scope 为 29 / 16,897 / 15,080 / 624,005 / 13；union 为 115 / 30,686 / 27,089 / 1,090,139 / 91。Zircon fingerprint `4a47dd55e44e985293c4d8a312fe6001d9de7f240f7e1fe4084980d799912dd6`，refs `b40cb8f51fadd9a7823c0009f01ac8d0cccc26fbd42b9c7f855d516ed8544b8a`，union `99ffeb15535cc3a04ea3ee34260c06b09f388af5709e37380cfabf198fddc77c`。本报告登记 5 个 P0、70 个 P1、12 个 P2 与 M0-M11；不修改生产代码。

## 2. 证据与边界

### 2.1 当前实现事实

1. `EditCommand` 有 `apply/revert/finalize/try_merge/journal_payload`，rollback 失败会进入 faulted，history 可按 Global/Document scope 分页。
2. `TransactionEventSink` 只投影本地 Editor bus；delivery rejection 会写 warning，而非改变 transaction durability。
3. Journal payload 为 string command type、u16 schema version 和 `serde_json::Value`；decode 仅验证 top-level schema，无 per-command decoder/migration consumer。
4. Create/Delete/Update/reflected-field payload 可描述本地 before/after，但没有跨 session object/property identity 或 precondition hash。
5. `UpdateNodeCommand`/`SetReflectedSceneFieldCommand` 直接写 after，不检查当前值、component generation 或 schema field ID。
6. `DocumentId` 由 project root/scene URI FNV 派生，collision 依赖本机打开文档集合，不同 checkout path 不能得到同一 identity。
7. `ProjectSessionId`、`TransactionId`、history lineage 均 process-local，不能放在 wire protocol 上。
8. OS `SessionOwnershipLease` 记录 PID、instance、heartbeat，保护同一物理 root；它不是用户、resource lock 或 distributed authority。
9. Hub Team 页从 `git config`/`git log` 聚合贡献者；没有 account、role、participant、session、presence 或 activity stream。
10. `UiControlRequest::InvokeBinding/InvokeRoute` 存在 raw control provenance/remote gate 旁路；协同命令必须绕开该路径。
11. Editor26 的 Runtime multiplayer replication 面向游戏玩家，Editor27 的 VCS 面向文件 revision，Editor42 的 snapshot/diff/merge 面向 world artifact；三者不能互相冒充协同 authority。

### 2.2 参考差异

1. Unreal Concert 将 session discovery/admission、endpoint identity、sequenced events、workspace/package activity、transaction、lock、presence 和 replication authority 拆成独立 contracts。
2. Concert transaction event 带 GUID transaction/operation、source endpoint、update index，并区分 snapshot/finalized/rejected；workspace database 保存可恢复 activity。
3. Concert lock manager 和 authority map 处理 package/object/property ownership，presence manager 处理短暂活动；两条流不共享 dirty/history。
4. Godot undo/redo 与 remote debugger 只能证明本地可逆操作和远程观测边界；Fyrox command channel 与 Bevy Remote 不是多用户收敛协议。
5. Unity Graphics 参考只覆盖本地 SerializedObject/Undo 的 Volume authoring，不推断闭源协同实现。

## 3. 差距清单

### 3.1 P0：实施前必须阻断

1. **P0-01** 未认证、无 baseline compatibility 或未完成 initial sync 时，不得公开远程 mutation 或 Live 状态。
2. **P0-02** 不得把 local transaction journal、Hub Git Team、Runtime replication 或 SessionOwnershipLease 宣称为 Multi-User。
3. **P0-03** 禁止把 raw UI binding/route、trait object、local path、PID、DocumentId 或 NodeId 直接放入远程协议。
4. **P0-04** 在 server ordering、typed codec、precondition、authority、ack、rollback、audit 完成前，不得接受远程 World/asset 写入。
5. **P0-05** 必须保留每个 participant 独立 workspace/OS lease；不得通过共享 physical checkout 实现协同。

### 3.2 P1：70 项重构主线

1. **P1-01** 定义 immutable Project/Scene/Workspace/Participant/Endpoint/Session identity。
2. **P1-02** 定义 server-assigned ActivityId 与 monotonic ActivitySequence。
3. **P1-03** 定义 client operation UUID、causal parent、base revision、input digest。
4. **P1-04** 定义 Object/Component/Property qualified address 与 provenance。
5. **P1-05** 定义 participant principal、role、device、capability 和 auth context。
6. **P1-06** 定义 session admission state、baseline compatibility 与 provider contract。
7. **P1-07** 定义 durable activity、transient presence、package activity 三种 schema。
8. **P1-08** 定义 owner/generation/request/ack/retry/idempotency 传播。
9. **P1-09** 定义 payload limits、depth、collection、rate 与 privacy policy。
10. **P1-10** 禁止 display path、PID、local path 和 process-local IDs 作为 wire authority。
11. **P1-11** 实现 `CollaborativeSessionService` discovery/create/join/leave/close。
12. **P1-12** 实现 server admission 的 project/engine/schema/plugin/source checks。
13. **P1-13** 实现 checkpoint、initial sync、activity gap、Live transition。
14. **P1-14** 实现 reliable request/event envelope、ack、reject、retry、dedup。
15. **P1-15** 实现 ordered durable ActivityLog 与 server sequencer。
16. **P1-16** 实现 activity database、checkpoint、compaction、archive、recovery。
17. **P1-17** 建立 package/source artifact transfer、checksum、chunk、atomic publish。
18. **P1-18** 将 transport/backpressure 接入 Editor09 job、quota、shutdown。
19. **P1-19** 实现 codec/validator/migrator/owner registry。
20. **P1-20** 将 local Scene commands 迁移到 stable address 与 before hash/revision CAS。
21. **P1-21** 让 accepted remote activity 通过现有 transaction engine 原子落地。
22. **P1-22** 为 apply/revert/replay 返回 typed acceptance/rejection receipt。
23. **P1-23** 实现 package/resource lock 与 object/property authority lease。
24. **P1-24** 实现 lock expiry/reclaim/disconnect/server restart 语义。
25. **P1-25** 实现 lock/authority conflict artifact，不静默 last-writer-wins。
26. **P1-26** 将 save/delete/rename/import/reimport 接入 lock、head、disk revision gate。
27. **P1-27** 建立 participant/session/scene/tool/selection/viewport presence schema。
28. **P1-28** 为 presence 增加 TTL、coalescing、interest、rate limit、privacy。
29. **P1-29** 保证 presence 不改变 dirty/history/activity head。
30. **P1-30** 接入 Outliner/Inspector/viewport remote projection 与 follow/jump。
31. **P1-31** 将 InvocationPrincipal/SourceProvenance 接入所有远程 command。
32. **P1-32** deny-by-default 校验 role、capability、object owner、operation policy。
33. **P1-33** 禁止远程执行任意 UI binding/route/menu 字符串。
34. **P1-34** 将 Editor42 snapshot/diff/merge artifact 接入 rebase/conflict flow。
35. **P1-35** 分类 same-property、delete/edit、reparent、component topology 冲突。
36. **P1-36** 实现 optimistic rejection rollback 与 pending UI reconciliation。
37. **P1-37** 实现 collaborative undo 的 compensating activity，不删除共享历史。
38. **P1-38** 实现 offline queue、base pin、bounded queue 和 explicit read-only policy。
39. **P1-39** reconnect 按 checkpoint/head rebase，不按 timestamp 插入旧操作。
40. **P1-40** schema/plugin/codec mismatch 在 apply 前 typed reject。
41. **P1-41** 建立 VCS baseline/branch/revision handoff，不与 live head 混淆。
42. **P1-42** 建立独立 checkout/sandbox/overlay workspace manager。
43. **P1-43** 保留每 workspace OS lease 与残留 owner recovery。
44. **P1-44** package transfer 中断可恢复或原子失败。
45. **P1-45** 维护 session fork/archive、checkpoint retention 与 cleanup。
46. **P1-46** 建立 activity query、stable pagination、cursor gap 与 audit view。
47. **P1-47** 接入 Editor11 durable diagnostic journal，敏感 payload 默认脱敏。
48. **P1-48** 统一 transport/store/codec/lock/presence/package error taxonomy。
49. **P1-49** 建立 server/client shutdown choreography、drain、terminal receipt。
50. **P1-50** Hub 提供 provider/account/session discovery，但不拥有 Editor world state。
51. **P1-51** Editor 提供 active session/participant/activity/authority/conflict workspace。
52. **P1-52** 所有 disabled/unavailable action 按 capability 显示真实原因。
53. **P1-53** 建立 multi-document transaction 与 cross-asset atomic policy。
54. **P1-54** 防止 VCS save、remote activity、local history 三者形成循环写入。
55. **P1-55** 建立 command/event/package fuzz、malformed input 和 plugin fault boundary。
56. **P1-56** 建立 process crash、server restart、client rejoin、partition recovery tests。
57. **P1-57** 建立 duplicate/reorder/delay/loss/backpressure deterministic tests。
58. **P1-58** 建立 same/disjoint property、hierarchy、asset/package conflict matrix。
59. **P1-59** 建立 auth denial、role escalation、privacy leakage、replay attack tests。
60. **P1-60** 建立 package checksum、partial file、atomic rename、disk-full fault tests。
61. **P1-61** 建立 2/4/16/64 client join/edit/accept latency budgets。
62. **P1-62** 建立 presence latency/drop/bandwidth budgets，不污染 durable metrics。
63. **P1-63** 建立 activity store CPU/memory/storage/network/resync telemetry。
64. **P1-64** 建立 lock contention、authority transfer 和 lease expiry telemetry。
65. **P1-65** 建立 baseline drift、source/plugin catalog change 诊断。
66. **P1-66** 建立 bounded message/page/inbox retention 与 backpressure policy。
67. **P1-67** 建立 participant/session permission audit 和 operator controls。
68. **P1-68** 建立 offline intent discard/merge/review workflow。
69. **P1-69** 删除试验 raw transport、process-local identity 与 last-writer-wins 旁路。
70. **P1-70** 端到端证明 provider/session/server/store/codec/authority/presence/workspace/conflict/UX 全闭环。

### 3.3 P2：主线完成后扩展

1. **P2-01** 多地域 relay、带宽感知与低延迟 edge session。
2. **P2-02** 组织/项目级 RBAC、SCIM/SSO 与审计保留策略。
3. **P2-03** 大型 package dedup、content-addressed cache 与分布式 checkpoint。
4. **P2-04** remote viewport streaming、annotation、review comment 与 approval。
5. **P2-05** domain-specific merge resolver marketplace 与规则版本治理。
6. **P2-06** session recording/replay、activity scrubber 与 deterministic time travel。
7. **P2-07** shared selection/follow、presence heatmap 与 privacy presets。
8. **P2-08** collaborative Sequencer/Variant/PCG/LevelInstance 专用 adapters。
9. **P2-09** headless CI multi-user soak、chaos network 与 long-session leak tests。
10. **P2-10** remote mobile/console client 与 mixed-version compatibility window。
11. **P2-11** offline branch review、cherry-pick 与 conflict explanation UI。
12. **P2-12** 以同规模、同安全与同 durability 条件建立超过参考引擎的协同基线。

## 4. 目标架构与里程碑

```text
Provider -> Session Admission -> Checkpoint/Initial Sync -> Live ActivityLog
Participant -> Typed Request -> Authority/Precondition -> Server Sequence -> Receipt
                 |                                                   |
             Presence (TTL)                                   Package/World Apply
```

Runtime/Scene 提供 stable schema、codec、reference resolve 和 local apply primitive；Session service 拥有 admission、ordered durable activity、checkpoint、reconnect；authority service 拥有 lock/lease/conflict；presence service 只处理 transient projection；Editor transaction 负责本地 atomic apply；Editor42 提供 snapshot/diff/merge；Editor27 提供 VCS baseline；OS lease 继续保护每个 workspace。

| Milestone | 退出条件 |
|---|---|
| M0 | 所有 remote/Live 假入口封口；共享 root 仍被 OS lease 阻断。 |
| M1 | identity、principal、session admission、wire envelope、capability ADR 冻结。 |
| M2 | checkpoint、initial sync、gap、server sequence、activity receipt 完成。 |
| M3 | durable activity store、package transfer、crash recovery 完成。 |
| M4 | typed codec、precondition、local transaction replay、reject rollback 完成。 |
| M5 | lock、object/property authority、save/package gate 完成。 |
| M6 | presence/interest/privacy 与 UI projection 完成。 |
| M7 | conflict/rebase/offline/reconnect 接入 Editor42 artifact。 |
| M8 | workspace/VCS/Hub/provider、session fork/archive 完成。 |
| M9 | product UX、admin、audit、diagnostics、unavailable policy 完成。 |
| M10 | fault/security/compatibility/scale qualification 完成。 |
| M11 | 删除 legacy/raw 旁路，32 门资格与文档/manifest 状态一致。 |

## 5. 验收门

1. **G01-G06** provider、auth、baseline、schema/plugin compatibility、initial sync 和 Live transition 缺失时 mutation disabled 且原因可见。
2. **G07-G12** wire identity、principal、sequence、ack/dedup/reorder、activity durability 和 checkpoint/rejoin 通过。
3. **G13-G18** typed codec、before hash/CAS、same/disjoint/conflict、schema mismatch、rollback、compensating undo 通过。
4. **G19-G24** lock/authority/lease expiry/save gate、package atomic transfer、workspace OS lease 和 reconnect policy 通过。
5. **G25-G30** presence TTL/interest/privacy 与 durable activity、dirty/history/selection 隔离，Outliner/Inspector/viewport projection 通过。
6. **G31-G32** 2/4/16/64 client、crash/partition/security/fuzz/long-session、Hub/Editor UX、docs/manifest/telemetry 全部达标。

## 6. 本轮验证与限制

本轮只做静态源码、测试 inventory、参考源码和物理范围 fingerprint 复核；没有修改 Editor、Runtime、Runtime Interface、Hub、Plugin 或 tests，也没有运行 server、多进程、网络故障、安全或压力验证。frontmatter 路径需在实施前重新展开并重算 manifest；local transaction tests 不能替代两个以上 Editor、server crash、partition/reconnect、lock race、auth 和 scale lane。本文不查询或实时跟踪协调器状态，整体 review 仍保持进行中。
