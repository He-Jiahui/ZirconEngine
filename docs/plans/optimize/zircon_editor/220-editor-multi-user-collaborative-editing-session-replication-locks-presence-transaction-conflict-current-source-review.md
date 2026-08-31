---
title: Editor Multi-User、Collaborative Editing、Session Replication、Locks、Presence 与 Transaction Conflict 当前源码复审
category: zircon_editor
report_id: Editor220
review_date: 2026-08-29
baseline_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
verification_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
canonical_owner: Editor43
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/43-multi-user-collaborative-editing-session-replication-locks-presence-transaction-conflict-authoring-review.md
  - docs/plans/optimize/zircon_editor/117-editor-multi-user-collaborative-editing-session-replication-locks-presence-transaction-conflict-current-source-review.md
  - docs/plans/optimize/zircon_editor/164-editor-multi-user-collaborative-editing-session-replication-locks-presence-transaction-conflict-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/27-project-operations-source-control-changelist-diff-automation-report-submit-gates-health-dashboard-review.md
  - docs/plans/optimize/zircon_editor/219-editor-scene-snapshot-world-diff-merge-restore-conflict-resolution-current-source-review.md
related_hub_owners:
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
related_code:
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/document
  - zircon_editor/src/core/editor_message
  - zircon_editor/src/core/recovery/document_journal
  - zircon_editor/src/core/recovery/session_guard
  - zircon_editor/src/core/project/project_preflight
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_runtime/src/core/framework/net
  - zircon_runtime/src/asset/project/manifest
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/ui/event_ui/control.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_plugins/net/runtime/src
  - zircon_plugins/net/features
  - zircon_hub/src/team
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/web/src/pages/TeamPage.tsx
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
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor220 · Multi-User / Collaborative Editing / Session Replication 当前源码复审

## 1. 结论

Editor164 之后仍没有形成工程级 Multi-User 或 Collaborative Editing 产品。对 zircon_editor、zircon_runtime、zircon_runtime_interface、zircon_hub、zircon_app 与 zircon_plugins 的 17,314 个已跟踪和 2,429 个未跟踪索引内物理 Rust/TOML/ZUI/Zr/TS/TSX 文件执行精确扫描，CollaborativeSessionService、CollaborativeEditingProvider、CollaborativeSession、MultiUser、PresenceService、ActivitySequence、ParticipantId、EndpointId、ResourceAuthorityService、AuthorityConflict、CollaborativeTransaction、RemoteTransaction 十二个合同全部零命中。

本地 transaction、journal、project identity、preflight、session guard 和网络插件都是真实底座，但没有任何 owner 组合出 collaborative provider、session admission、initial sync、server sequence、durable activity store、participant presence、resource authority、accepted remote transaction、conflict artifact 或 reconnect convergence。文件数量、test 数量和 transport 能力不能替代产品纵链。

旧报告的两个关键风险仍原样存在。第一，Scene route 现在会把打开/创建的 document 绑定到 DocumentJournalCoordinator，但 coordinator 的唯一 append API 仍是测试配置下的 append_for_test，注释继续明确说明 production durable publication 要等 transaction engine 在 commit linearization point 拥有 immutable capture。绑定 journal identity 不等于提交 transaction，更不等于 shared activity log。

第二，UiControlRequest 的 InvokeBinding 与 InvokeRoute 仍直接进入 binding/route 执行，只有 CallAction 检查 callable_from_remote；EditorCommandDescriptor::new 与 serde default 仍将 callable_from_remote 设为 true。请求 envelope 没有 authenticated principal、session、device、role、capability 或 replay protection。协作层若复用这条远程 UI control 通道，会形成默认开放、可绕过 action gate且不能审计真实调用者的 mutation 面。

Scene command replay 仍是 silent last-writer-wins 风险：UpdateNodeCommand 只验证 node 存在，按 before/after 差异直接写 after，不验证 current==before；SetReflectedSceneFieldCommand 同样直接写 after。raw NodeId、component type path、field name 和 process-local transaction/history identity 也不能进入 wire。

Editor43 保持唯一 canonical owner；状态仍为：**P0：5 Open；P1：64 Open / 6 Partial；P2：12 Open；Gates：29 Fail / 3 Partial / 0 Pass**。没有两个真实 Editor client、authoritative server、durable activity recovery、冲突矩阵、安全或规模测试，因此没有证据证明达到、接近或优于 Unreal Concert。

## 2. 审查范围、统计与 currentness

统计读取当前 working-tree 物理内容，包含相关未跟踪文件。行数为物理行；tests/ignored 统计精确 Rust test/tokio::test/ignore attribute。fingerprint 按 lowercase repository-relative path 排序，将路径与文件 SHA-256 组成清单后再次取 SHA-256。选择集用于复现本报告，不代表参考仓库全部规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Editor authoring / journal / session / remote surface | **138 / 19,713 / 17,853 / 670,007 / 81 / 0** | 05d1bc2efbb7bb7fbf51abaf135c8322f8ddffddbe22e520b642d38ba882d7f2 |
| Runtime / Interface identity / admission / net DTO | **102 / 7,913 / 7,071 / 249,429 / 73 / 0** | a02558e2e20cb2c504541ec09e525651b5768f4a0de82fbf3e4798a5f0692492 |
| Net plugin product boundary | **175 / 16,380 / 14,770 / 575,681 / 147 / 0** | 7ca43b8da86ba490c93f0c3f40155eaafc98d368bcdbad92cc3c8e709aa8474c |
| Hub team / collaboration surface | **4 / 752 / 701 / 26,249 / 6 / 0** | f14d78b15d94ba5656a4b83dc8d5c6c4ae4df8206cb3618527663d7ccbdede0b |
| Zircon selected union | **419 / 44,758 / 40,395 / 1,521,366 / 307 / 0** | cbf3a88953f01b8b5e53159bd73b0f506d00622ef32eb0bee01f686a3d4cfab3 |
| Unreal / Godot / Fyrox / Bevy / Unity Graphics selected | **29 / 16,871 / 14,576 / 624,005 / 11 / 0** | b40cb8f51fadd9a7823c0009f01ac8d0cccc26fbd42b9c7f855d516ed8544b8a |
| All selected | **448 / 61,629 / 54,971 / 2,145,371 / 318 / 0** | c1586ca4d2c3ca95c98d5c0292eb93fa6b18fe198269adf0aad582dda4701da0 |

- baseline 与 verification HEAD 均为 f660cfa9f3f84bff0903e4564ff1af4d065aee73；共享工作树包含大量在途修改，本报告读取物理文件，不回退、不覆盖，也不把在途代码写成已集成产品。
- Editor 集合覆盖 editing、document、editor message、document journal、session guard、project preflight、Scene document route、command descriptor 与 remote control/operation dispatch。
- Runtime/Interface 集合覆盖 framework net、project manifest/identity/session lock 与 UI control contract；Net plugin 只用于判断可复用 transport/gameplay replication 边界，详细裁决仍归 Runtime99zo。
- Hub 集合仍只有本地 Git team projection、Coming Soon model 和 Team page；参考集合是 frontmatter 的精确 29 个文件。
- 按用户要求未查询、轮询、等待或实时跟踪协调器；Tooling 按用户要求排除。
- 本轮只修改 review 与索引，没有修改 production、tests、Cargo、ABI 或参考源码，也没有运行 Cargo、Editor、server、多进程、网络故障、安全、scale、soak 或跨引擎 benchmark。

## 3. 当前源码事实与语义边界

### 3.1 产品 owner、身份和协议仍为零

1. 没有 provider、server/client lifecycle、participant roster、endpoint membership、live/archive session 或 session state machine。
2. 没有 server-assigned ActivityId/ActivitySequence、durable activity store、checkpoint head、gap query、ack/dedup/reorder 或 reconnect resume cursor。
3. 没有 collaborative object/property address、lock/lease、same-property conflict、optimistic reject rollback 或 compensating collaborative undo。
4. 没有 presence schema、TTL、coalescing、interest filtering、privacy、remote selection/viewport projection 或 transient/durable stream separation。
5. 没有 package/source artifact transfer、checksum/chunk protocol、independent workspace manager、session archive/fork 或 activity audit UI。

### 3.2 Local transaction 与 durable journal 不能冒充协作协议

1. EditorTransactionEngine 已有 RAII/nested transaction、apply/revert/finalize、失败 rollback、selection restoration、dirty generation、history paging、operation gate/group 与 faulted state。
2. EditCommandCodecRegistry 按 command type/schema version 严格注册；Scene create/delete/update/reflected-field codec 与 TransactionJournalReplayer 的 decode-all-before-begin 能避免坏 payload 产生半事务。
3. journal 记录仍是 process-local TransactionId、HistoryContextId、frame、本地 documents/selection 与 command payload；没有 principal、session/endpoint、global sequence、base revision、causal parent、ack、authority 或 audit provenance。
4. DocumentJournalCoordinator 已接入 Scene route 的 bind/unbind 生命周期，磁盘 key 正确地从 project-relative source path 派生，而不是直接使用 session-local DocumentId。
5. coordinator 仍只有测试可见 append_for_test；production commit 没有 immutable capture、append linearization point 或 append failure policy。bind、read、compact、discover、codec、replayer 仍未组成 startup recovery workflow。
6. per-document journal 的 sequence、BLAKE3 checksum、record/file bounds、tail fault classification、sync 与 covered-prefix compaction 是可靠 storage primitive，但不是 server activity database。

### 3.3 Scene replay 仍会静默覆盖并发修改

1. UpdateNodeCommand::apply 只确认 node 存在，再根据 before/after 字段差异写入 after；不会确认 scene 当前 name/parent/transform 仍等于 before。
2. SetReflectedSceneFieldCommand::apply 直接写 after；replay validation 只检查地址非空与 before!=after，没有 current value/hash/revision CAS。
3. payload 使用 raw NodeId、component_type_path String、field_name String 与完整 before/after value，没有 qualified project/scene/source/instance identity、field schema ID 或 provenance。
4. try_merge 只做同一 local NodeId 的 command coalescing，不是 concurrent three-way merge。把当前 replayer 接到网络只会得到 silent LWW。

### 3.4 Project preflight 与 OS lease 的正确归属

1. manifest 已持久化非空 ProjectGuid；ProjectPreflightReceipt 组合 canonical descriptor path、ProjectGuid、manifest digest 与 engine semver compatibility。
2. physical descriptor path 是本机 filesystem authority，不是跨 checkout wire identity；Scene/Workspace/Participant/Endpoint/CollaborativeSession identity 仍缺失。
3. SessionGuard 的 lifecycle、heartbeat、exact-record takeover、Windows named mutex/Unix flock 与 residual recovery 是可靠单工作区保护。
4. ProjectSessionPrincipalV1 是 local desktop request provenance，不是 authentication claim；PID、instance、launch source、operation ID 与 session generation 不能替代 user/device/role/capability。
5. Multi-User 必须给每个 participant 独立 checkout/sandbox/overlay 并保留各自 OS lease，不能让多个进程共同写同一 physical root。

### 3.5 Remote UI control 仍是独立 P0

1. UiControlRequest 提供 InvokeBinding、InvokeRoute、SetProperty、CallAction 等控制请求，但 envelope 没有 principal/session/device/role/capability/auth context。
2. handle_control_request 对 InvokeBinding 与 InvokeRoute 直接执行；只有 CallAction 查询并检查 callable_from_remote。
3. EditorCommandDescriptor::new 与 default_callable_from_remote 仍返回 true，安全默认方向错误；部分 commands 显式覆盖不能修正默认 fail-open。
4. invocation 只标记 UiBinding 或 Headless source，没有真实 authenticated principal/source provenance。协作 activity 必须使用独立 typed protocol。

### 3.6 Runtime Net 与 Hub Team 仍不是 Collaborative Editing

1. Runtime framework net 和 net plugin 提供 TCP/UDP/HTTP/WebSocket、RUDP、RPC、replication/content-download 局部底座；gameplay NetSessionId、player string、client authority 与 RPC target 不能重命名为 Editor participant或 collaborative activity。
2. 当前没有 Editor collaboration provider 消费这些 transport，也没有 admission、server sequence、activity DB、authority/presence schema、package workspace 或 conflict integration。
3. Hub Team 读取本地 Git root、user.name/email 与最近 200 个 commit author，最多投影 8 个 contributor；这只是 repository overview。
4. team invite、permissions、remote collaboration 仍 disabled/Coming Soon。这是诚实状态，应保留；未来 Hub 只拥有 account/provider/session discovery，Editor/Session service 拥有 World activity、authority、presence 与 conflict。

## 4. 参考引擎差异

| 能力 | Zircon 当前 | Unreal Concert 参考 | 必须收敛的边界 |
|---|---|---|---|
| Session / admission | local project preflight 与 single-root lease | live/archived session、endpoint membership、request/response | 分离 local workspace admission 与 collaborative server admission |
| Ordering / durability | process-local transaction/frame，journal production append 断路 | sequenced custom events、workspace activity、session database | server sequence、durable activity、checkpoint/gap/rejoin |
| Transaction | local apply/revert/replay，无 current==before | GUID transaction/operation、source endpoint、update index、snapshot/finalized/rejected | typed operation、precondition、receipt、rollback、compensating undo |
| Lock / authority | 只有 physical root OS lease | package lock、object/property replication authority、conflict query/cache | 保留 OS lease，新增 resource authority service |
| Presence | 无 schema/owner | 独立 presence events/manager 与 live transaction authors | transient TTL stream，绝不污染 dirty/history/activity head |
| Package / workspace | 无 session artifact transfer/workspace manager | client/server workspace、package activity、database persistence | 独立 checkout、checksum/chunk/atomic publish、fork/archive |
| Product UI | Hub local Git + Coming Soon | session browser、participants、activity、locks/presence/workspace | capability-driven Hub discovery 与 Editor collaboration workspace |

Godot EditorUndoRedoManager 与 remote debugger 要求本地历史和远程观测分层；Fyrox command/message 是本地 command stack；Bevy Remote 是 typed JSON-RPC/ECS mutation/watch 边界；Unity Graphics Volume editor 使用 SerializedObject、Undo 与 ApplyModifiedProperties。它们不能替 Zircon 证明多用户收敛，也不允许把 debug control、local command 或 gameplay replication 冒充协作协议。

## 5. 差距清单

### 5.1 P0：5 Open

1. **P0-01 · Open** 未认证、无 baseline compatibility 或未完成 initial sync 时，不得公开远程 mutation 或 Live 状态。
2. **P0-02 · Open** 不得把 local transaction journal、Hub Git Team、Runtime replication 或 SessionOwnershipLease 宣称为 Multi-User。
3. **P0-03 · Open** 禁止把 raw UI binding/route、trait object、local path、PID、DocumentId 或 NodeId 直接放入远程协议。
4. **P0-04 · Open** 在 server ordering、typed codec、precondition、authority、ack、rollback、audit 完成前，不得接受远程 World/asset 写入。
5. **P0-05 · Open** 必须保留每个 participant 独立 workspace/OS lease；不得通过共享 physical checkout 实现协同。

### 5.2 P1：64 Open / 6 Partial

1. **P1-01 · Partial** 已有持久 ProjectGuid、manifest digest 与 local project identity；仍需 immutable Scene/Workspace/Participant/Endpoint/Session identity，且 physical path 不得上 wire。
2. **P1-02 · Open** 定义 server-assigned ActivityId 与 monotonic ActivitySequence。
3. **P1-03 · Open** 定义 client operation UUID、causal parent、base revision、input digest。
4. **P1-04 · Open** 定义 Object/Component/Property qualified address 与 provenance。
5. **P1-05 · Open** 定义 participant principal、role、device、capability 和 auth context。
6. **P1-06 · Partial** 已有 local project preflight、manifest/engine compatibility 与 session-guard lifecycle；仍需 collaborative provider、source/plugin/schema baseline admission 与 server receipt。
7. **P1-07 · Open** 定义 durable activity、transient presence、package activity 三种 schema。
8. **P1-08 · Open** 定义 owner/generation/request/ack/retry/idempotency 的端到端传播。
9. **P1-09 · Open** 定义 payload limits、depth、collection、rate 与 privacy policy。
10. **P1-10 · Open** 禁止 display path、PID、local path 和 process-local IDs 作为 wire authority。
11. **P1-11 · Open** 实现 CollaborativeSessionService discovery/create/join/leave/close。
12. **P1-12 · Open** 实现 server admission 的 project/engine/schema/plugin/source checks。
13. **P1-13 · Open** 实现 checkpoint、initial sync、activity gap、Live transition。
14. **P1-14 · Open** 实现 reliable request/event envelope、ack、reject、retry、dedup。
15. **P1-15 · Open** 实现 ordered durable ActivityLog 与 server sequencer。
16. **P1-16 · Open** 实现 activity database、checkpoint、compaction、archive、recovery；local document journal 不得冒充 activity DB。
17. **P1-17 · Open** 建立 package/source artifact transfer、checksum、chunk、atomic publish。
18. **P1-18 · Open** 将 transport/backpressure 接入 Editor09 job、quota、shutdown。
19. **P1-19 · Partial** 已有 strict local command codec registry、schema lookup 与 decode-all-before-apply；仍需 collaborative owner registry、migration、wire validation 与 plugin/schema negotiation。
20. **P1-20 · Open** 将 local Scene commands 迁移到 stable address 与 before hash/revision CAS。
21. **P1-21 · Partial** replayer 已通过现有 transaction engine 原子 commit local decoded commands；仍无 server-accepted remote activity、authority/precondition、receipt 与 production assembly。
22. **P1-22 · Open** 为 apply/revert/replay 返回 typed acceptance/rejection receipt；当前 Result<TransactionId, JournalReplayError> 不是协作 receipt。
23. **P1-23 · Open** 实现 package/resource lock 与 object/property authority lease。
24. **P1-24 · Open** 实现 lock expiry/reclaim/disconnect/server restart 语义。
25. **P1-25 · Open** 实现 lock/authority conflict artifact，不静默 last-writer-wins。
26. **P1-26 · Open** 将 save/delete/rename/import/reimport 接入 lock、head、disk revision gate。
27. **P1-27 · Open** 建立 participant/session/scene/tool/selection/viewport presence schema。
28. **P1-28 · Open** 为 presence 增加 TTL、coalescing、interest、rate limit、privacy。
29. **P1-29 · Open** 保证 presence 不改变 dirty/history/activity head。
30. **P1-30 · Open** 接入 Outliner/Inspector/viewport remote projection 与 follow/jump。
31. **P1-31 · Open** 将 authenticated InvocationPrincipal/SourceProvenance 接入所有远程 command。
32. **P1-32 · Open** deny-by-default 校验 role、capability、object owner、operation policy。
33. **P1-33 · Open** 禁止远程执行任意 UI binding/route/menu 字符串，并修正默认 remote-callable 策略。
34. **P1-34 · Open** 将 Editor42/219 snapshot/diff/merge artifact 接入 rebase/conflict flow。
35. **P1-35 · Open** 分类 same-property、delete/edit、reparent、component topology 冲突。
36. **P1-36 · Open** 实现 optimistic rejection rollback 与 pending UI reconciliation。
37. **P1-37 · Open** 实现 collaborative undo 的 compensating activity，不删除共享历史。
38. **P1-38 · Open** 实现 offline queue、base pin、bounded queue 和 explicit read-only policy。
39. **P1-39 · Open** reconnect 按 checkpoint/head rebase，不按 timestamp 插入旧操作。
40. **P1-40 · Open** schema/plugin/codec mismatch 在 apply 前 typed reject。
41. **P1-41 · Open** 建立 VCS baseline/branch/revision handoff，不与 live head 混淆。
42. **P1-42 · Open** 建立独立 checkout/sandbox/overlay workspace manager。
43. **P1-43 · Partial** 已有 robust single-workspace OS lease、heartbeat、residual exact-record recovery；仍需 per-participant workspace manager 与 session/workspace mapping。
44. **P1-44 · Open** package transfer 中断可恢复或原子失败。
45. **P1-45 · Open** 维护 session fork/archive、checkpoint retention 与 cleanup。
46. **P1-46 · Open** 建立 activity query、stable pagination、cursor gap 与 audit view。
47. **P1-47 · Open** 接入 Editor11 durable diagnostic journal，敏感 payload 默认脱敏。
48. **P1-48 · Open** 统一 transport/store/codec/lock/presence/package error taxonomy。
49. **P1-49 · Open** 建立 server/client shutdown choreography、drain、terminal receipt。
50. **P1-50 · Open** Hub 提供 provider/account/session discovery，但不拥有 Editor world state。
51. **P1-51 · Open** Editor 提供 active session/participant/activity/authority/conflict workspace。
52. **P1-52 · Open** 所有 disabled/unavailable action 按实时 capability 显示真实原因；当前 static Coming Soon 只满足诚实占位。
53. **P1-53 · Open** 建立 multi-document transaction 与 cross-asset atomic policy。
54. **P1-54 · Open** 防止 VCS save、remote activity、local history 三者形成循环写入。
55. **P1-55 · Open** 建立 command/event/package fuzz、malformed input 和 plugin fault boundary。
56. **P1-56 · Open** 建立 process crash、server restart、client rejoin、partition recovery tests。
57. **P1-57 · Open** 建立 duplicate/reorder/delay/loss/backpressure deterministic tests。
58. **P1-58 · Open** 建立 same/disjoint property、hierarchy、asset/package conflict matrix。
59. **P1-59 · Open** 建立 auth denial、role escalation、privacy leakage、replay attack tests。
60. **P1-60 · Open** 建立 package checksum、partial file、atomic rename、disk-full fault tests。
61. **P1-61 · Open** 建立 2/4/16/64 client join/edit/accept latency budgets。
62. **P1-62 · Open** 建立 presence latency/drop/bandwidth budgets，不污染 durable metrics。
63. **P1-63 · Open** 建立 activity store CPU/memory/storage/network/resync telemetry。
64. **P1-64 · Open** 建立 lock contention、authority transfer 和 lease expiry telemetry。
65. **P1-65 · Open** 建立 baseline drift、source/plugin catalog change 诊断。
66. **P1-66 · Partial** local Editor bus、history 与 journal 已有消息数/字节/record bounds 和 backpressure reporting；仍需 collaborative request/page/inbox/peer quota 与 slow-client policy。
67. **P1-67 · Open** 建立 participant/session permission audit 和 operator controls。
68. **P1-68 · Open** 建立 offline intent discard/merge/review workflow。
69. **P1-69 · Open** 删除试验 raw transport、process-local identity 与 last-writer-wins 旁路。
70. **P1-70 · Open** 端到端证明 provider/session/server/store/codec/authority/presence/workspace/conflict/UX 全闭环。

### 5.3 P2：12 Open

1. **P2-01 · Open** 多地域 relay、带宽感知与低延迟 edge session。
2. **P2-02 · Open** 组织/项目级 RBAC、SCIM/SSO 与审计保留策略。
3. **P2-03 · Open** 大型 package dedup、content-addressed cache 与分布式 checkpoint。
4. **P2-04 · Open** remote viewport streaming、annotation、review comment 与 approval。
5. **P2-05 · Open** domain-specific merge resolver marketplace 与规则版本治理。
6. **P2-06 · Open** session recording/replay、activity scrubber 与 deterministic time travel。
7. **P2-07 · Open** shared selection/follow、presence heatmap 与 privacy presets。
8. **P2-08 · Open** collaborative Sequencer/Variant/PCG/LevelInstance 专用 adapters。
9. **P2-09 · Open** headless CI multi-user soak、chaos network 与 long-session leak tests。
10. **P2-10 · Open** remote mobile/console client 与 mixed-version compatibility window。
11. **P2-11 · Open** offline branch review、cherry-pick 与 conflict explanation UI。
12. **P2-12 · Open** 以同规模、同安全、同 durability、同平台和同故障条件建立超过参考引擎的协同基线。

## 6. 目标架构与 ownership

~~~mermaid
flowchart LR
    Hub["Hub provider/account/session discovery"] --> Admission["Session admission and baseline checks"]
    Workspace["Independent workspace + OS lease"] --> Admission
    Admission --> Sync["Checkpoint + initial sync + gap closure"]
    Sync --> Live["Live session"]
    LocalTx["Editor local transaction capture"] --> Request["Typed operation + principal + base"]
    Live --> Request
    Request --> Authority["Authority and precondition"]
    Authority --> Sequencer["Server sequencer"]
    Sequencer --> Store["Durable activity store"]
    Store --> Apply["Editor transaction apply + typed receipt"]
    Presence["Transient presence TTL stream"] --> Projection["Outliner / Inspector / viewport projection"]
    Store --> Conflict["Rebase / conflict artifact / compensating undo"]
    Conflict --> Apply
~~~

Runtime/Scene 只提供 stable schema、codec、reference resolve 与 deterministic local apply primitive；Session service 拥有 admission、ordered durable activity、checkpoint 与 reconnect；Authority service 拥有 lock/lease/conflict；Presence service 只处理 transient projection；Editor transaction engine 是唯一 local atomic apply authority；Editor219 提供 snapshot/diff/merge artifact；Editor27 提供 VCS baseline；Hub 只提供 provider/account/session discovery；OS lease 继续保护每个 participant 的独立 workspace。

## 7. 依赖顺序与里程碑

| Milestone | 退出条件 |
|---|---|
| M0 | 封口默认 remote-callable、raw binding/route 与伪 Live；共享 physical root 仍被 OS lease 阻断。 |
| M1 | identity、principal、session admission、wire envelope、capability ADR 冻结。 |
| M2 | checkpoint、initial sync、gap、server sequence、activity receipt 完成。 |
| M3 | durable activity store、package transfer、crash recovery 完成。 |
| M4 | typed codec、precondition、local transaction replay、reject rollback 完成。 |
| M5 | lock、object/property authority、save/package gate 完成。 |
| M6 | presence/interest/privacy 与 UI projection 完成。 |
| M7 | conflict/rebase/offline/reconnect 接入 Editor219 artifact。 |
| M8 | workspace/VCS/Hub/provider、session fork/archive 完成。 |
| M9 | product UX、admin、audit、diagnostics、unavailable policy 完成。 |
| M10 | fault/security/compatibility/scale qualification 完成。 |
| M11 | 删除 legacy/raw 旁路，32 门资格与文档/manifest 状态一致。 |

不得从“先能连上两个 Editor”开始倒推协议。M0/M1 必须先冻结安全默认、身份、baseline、wire 与 ownership；M2/M3 建立可恢复 authoritative head 后，M4 才允许把 accepted activity 接入 transaction replayer；M5/M7 依赖 Editor219 stable change/conflict artifact；M6 presence 必须始终与 durable activity 分流。

## 8. 验收门

| Gate | 状态 | 当前证据与退出条件 |
|---|---|---|
| G01 | Fail | 无 collaborative provider/discovery/create/join/leave owner。 |
| G02 | Fail | local launch provenance 明确不是 auth；远程请求无 principal/role/device/capability。 |
| G03 | Partial | 有 ProjectGuid、manifest digest、engine semver preflight；缺 server baseline receipt、source/plugin catalog。 |
| G04 | Fail | 无 collaborative plugin/schema negotiation、migration policy 与 typed reject。 |
| G05 | Fail | 无 checkpoint fetch、snapshot install、gap closure。 |
| G06 | Fail | 无只有 initial sync 完成后才能进入的 Live state。 |
| G07 | Fail | Scene/Workspace/Participant/Endpoint/Session 与 qualified object/property identity 缺失。 |
| G08 | Fail | UI/command/activity/store/audit 没有端到端 authenticated provenance。 |
| G09 | Fail | 没有 ActivityId/ActivitySequence 或 authoritative head。 |
| G10 | Fail | 没有 request ID、ack/reject、retry、dedup、reorder buffer。 |
| G11 | Fail | local document journal production append 断路，也不是 server activity store。 |
| G12 | Fail | 无 checkpoint generation、resume cursor、gap query、server crash recovery。 |
| G13 | Partial | local registry、strict decode、decode-all-before-apply 已有；缺 wire/migration/plugin owner 与 production assembly。 |
| G14 | Fail | Scene update/reflected field 不比较 current 与 before。 |
| G15 | Fail | 当前远端重放会 silent LWW，无 conflict artifact。 |
| G16 | Fail | 无 property-level concurrent merge 或 dependency-aware apply。 |
| G17 | Fail | 无协作 apply 前的 plugin/schema/codec negotiation 与 reject receipt。 |
| G18 | Fail | local rollback 可复用；无 optimistic reject reconciliation 或 shared-history compensating activity。 |
| G19 | Fail | 只有 physical workspace lease，没有 package/object/property lock。 |
| G20 | Fail | 无 server lease、disconnect reclaim、transfer、restart recovery。 |
| G21 | Fail | save/delete/rename/import 未绑定 collaborative head/authority/disk revision。 |
| G22 | Fail | 无 collaborative chunk/checksum/resume/atomic publish protocol。 |
| G23 | Partial | robust single-root OS lease 和 residual recovery 已有；缺独立 checkout/sandbox/overlay manager。 |
| G24 | Fail | 无 base-pinned bounded offline queue、read-only policy、rebase/review。 |
| G25 | Fail | 无 participant/scene/tool/selection/viewport presence owner。 |
| G26 | Fail | 无 TTL、interest、coalescing、rate、bandwidth、privacy policy。 |
| G27 | Fail | 没有实际 presence stream 可验证 non-dirty separation。 |
| G28 | Fail | 无 participant/activity/lock/conflict workspace 或 Outliner/Inspector/viewport projection。 |
| G29 | Fail | 无 collaborative multi-document atomic policy、dependency order 或 partial-failure receipt。 |
| G30 | Fail | 无 session audit、redaction、operator control、drain 与 terminal receipt。 |
| G31 | Fail | 无 crash/partition/replay/fuzz/privacy/2-64 client/long-session qualification。 |
| G32 | Fail | Hub 诚实 Coming Soon，但 provider/server/store/Editor UX/telemetry/docs 全链未装配。 |

## 9. 验证说明与风险

1. 本轮完成 448 个 selected 物理文件的统计/fingerprint、19,743 个产品文件的十二合同扫描、当前源码语义复核与 29 个参考文件复核；没有执行会改变产品状态的操作。
2. 未运行 Cargo、Editor、server 或动态多进程测试，因为本轮为 review-only；静态 test inventory 不能把任何产品门推断为 Pass。
3. 共享 working tree 正在变化，报告冻结的是验证时物理内容；实施前必须重新计算 manifest/fingerprint 并复核接口终态。
4. journal bind、strict codec、local replay、ProjectGuid、preflight、OS lease、net transport 与 Hub Git projection 是正确底座；把底座命名为 Multi-User 会掩盖安全、顺序、durability 与 convergence 缺口。
5. 当前 remote UI mutation 面必须在协作实施前先 fail closed；不能等 provider/server 完成后再补身份和授权。
6. “优于 Unreal”必须在相同 participant 规模、安全、数据完整度、durability、故障注入、网络条件、硬件与平台下由端到端 benchmark/profile 证明；当前没有这类证据。

文档终验必须确认：frontmatter 路径存在；P0/P1/P2 ID 唯一连续为 5/70/12，状态为 5 Open、64 Open/6 Partial、12 Open；M0-M11 完整；G01-G32 唯一且状态为 29 Fail/3 Partial；Editor/根索引与 coverage 452 同步；Markdown 无 trailing whitespace 或断链。整体引擎 review 继续进行，本报告不把 review 完成解释为实现完成。
