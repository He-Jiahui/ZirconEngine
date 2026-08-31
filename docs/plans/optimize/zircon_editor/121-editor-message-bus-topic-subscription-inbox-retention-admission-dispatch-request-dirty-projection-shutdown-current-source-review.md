---
title: Editor Message Bus、Topic、Subscription、Inbox、Retention、Admission、Dispatch、Request、Dirty Projection 与 Shutdown 当前源码复核
category: zircon_editor
report_id: Editor121
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor48
refreshes:
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
related_code:
  - zircon_editor/src/core/editor_message
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/editing/engine/transaction/lifecycle.rs
  - zircon_editor/src/core/i18n/service.rs
  - zircon_editor/src/core/jobs/event.rs
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/core/logging/service.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/plugin/lifecycle_message_bridge.rs
  - zircon_editor/src/core/sync/pump.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
tests:
  - zircon_editor/src/tests/editor_message
plan_sources:
  - docs/zircon_editor/core/editor_message.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/02/2026-07-22-message-inbox-backpressure-and-fanout.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-message-inbox-backpressure-and-fanout.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-17-job-pump-budget-and-pending-scan.md
  - docs/plans/zircon_editor/editor/14/2026-08-10-job-event-delivery-reservation-analysis.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-22-message-subscriber-result-consumer-drift.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/33-localization-string-table-culture-translation-import-export-fallback-pseudo-localization-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageSubscription.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageBus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageReceiver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageSubscription.h
  - dev/bevy/crates/bevy_ecs/src/event/mod.rs
  - dev/bevy/crates/bevy_ecs/src/observer/mod.rs
  - dev/bevy/crates/bevy_ecs/src/observer/centralized_storage.rs
  - dev/godot/core/object/message_queue.h
  - dev/godot/core/object/message_queue.cpp
  - dev/Fyrox/editor/src/message.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Util/MessageManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/FixedBufferStringQueueTests.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 121 · Editor Message Bus / Topic / Subscription / Inbox / Dirty Projection 工程化差距

## 1. 结论

`core::editor_message` 已有真正的工程底座：publication 共享一个 immutable `Arc` payload；每个 subscriber 独立 inbox；Lossless/Latest/Bounded retention、4096/256/256 条目预算、2 MiB 单消息与 16 MiB logical bytes 预算、checked subscriber/delivery sequence、lossless fanout 全有或全无预检、latest key index、全局 surviving sequence 和 request callback 锁外执行均已落地。旧版“无界 Vec、深拷贝 fanout、持 bus 锁回调、ID 饱和复用”结论不再适用。

当前唯一直接产品 P0 在 dirty projection：`EditorHostEventController::publish_view_invalidation()` 发布合法 `view.invalidated` custom message；生产没有该 topic 的 subscriber，bus 仅在至少一个 inbox 接受时调用 `mark_message_dirty()`。topic 解析成功使 fallback `mark_view_dirty()` 不执行，因此 `refresh_view()`、`refresh_workbench()`、资产/状态/扩展注册、组件和 Scene inspection 可以返回空 dirty set，跳过真实刷新。该问题必须与 Editor02/47 的 world generation 合同一起修复，而不是注册假 subscriber。

更高层仍有协议缺口：dispatch report 只返回 subscriber ID 集合，无法区分 NoRoute、Accepted、Rejected、Partial、Closed；lossless backpressure 后原消息 ownership 已被消费；unregister 无条件销毁未读 inbox；shared dispatch 与 unregister 交错可能向 orphan inbox 投递；drain 在业务处理前整箱删除，没有 ack/nack/cursor/page。sequence 被 serde/PartialEq 忽略，不能直接扩展为 replay、持久订阅或跨进程协议。

Document/Play plugin bridge 每 tick 将 bounded inbox 整箱搬到无界 pending，callback 失败只把队首放回，绕过 bus 预算；Job pump 先 pop 再忽略 report；World Sync/Runtime Gateway 的 generation/watermark 由 Editor47 负责，本报告只定义 message commit disposition。Topic adoption、subscriber lifecycle、shutdown 和诊断必须成为唯一 bus authority。

本轮 Zircon scope 为 66 files / 12,147 lines / 10,965 non-empty / 434,869 bytes / 149 test attributes；参考 scope 为 17 / 6,530 / 5,537 / 219,669 / 76；union 为 83 / 18,677 / 16,502 / 654,538 / 225。Zircon fingerprint `4d5ec415735ec7a00af7993701c72df0f2d924fd5d185df28d1cab29d7adb4ce`，refs `7b38cb88a5777a1f81d23d615c49f1d506334f81b20d6eddf0eac50d078b47bf`，union `d919c4f91b41506516ccfcd46a97ba0b93da4744b93578079b37439d53a15b64`。本报告登记 1 个 P0、52 个 P1、15 个 P2 与 40 个 gate；不修改生产代码。

## 2. 当前实现事实与参考差异

1. 保留策略、logical bytes、payload sharing、checked sequence、latest index、lossless admission 和 request lock split 均是真实能力；它们需要扩展 receipt/generation，而不是回退。
2. `view.invalidated` 零 target 时返回空成功，dirty authority 与 message side effect 没有分离；该路径可导致最后一次大结构变更永不刷新。
3. Lossless inbox unregister、dispatch/unregister race、poison ownership、pending queue、request timeout/panic 和 shutdown disposition 没有统一生命周期。
4. plugin bridge 的无界 shadow queue 使 producer 看见健康 subscriber，实际 callback backlog 却无界；slow/poison consumer 无 quarantine/dead-letter。
5. World/Scene delta、Transaction、Job、Play、Tool、Focus、Log、I18n topic 没有完整 producer-consumer-resync adoption matrix。
6. Unreal MessageBus/MessageRouter/Subscription 将 receiver identity、subscription lifetime、message context、request/response 和 shutdown 分离；Godot MessageQueue 提供明确 queue/flush 边界；Bevy observer 随 entity 清理，Fyrox mpsc 是低基线；Unity fixed buffer 体现 `TryPush` 失败语义。

## 3. 差距清单

### 3.1 P0

1. **P0-01** `view.invalidated` 无 subscriber 时必须仍提交 authoritative dirty mask 并驱动正确 refresh；不能靠假 subscriber 或无条件成功掩盖 NoRoute。

### 3.2 P1：52 项

1. **P1-01** 将 NoRoute/Accepted/Rejected/Partial/Closed 定义为 typed dispatch disposition。
2. **P1-02** dirty authority 与 inbox acceptance 分离并具备 generation。
3. **P1-03** custom topic 绑定 owner、schema、capability、revision。
4. **P1-04** subscription 携带 owner、scope、generation、lease。
5. **P1-05** delivery 携带 global sequence、topic sequence、request/correlation ID。
6. **P1-06** Lossless reject 保留原消息 ownership 供重试。
7. **P1-07** unregister 返回 pending count/bytes/sequence disposition。
8. **P1-08** publish/unregister race 以 route generation fence 解决 orphan delivery。
9. **P1-09** subscription lease 显式 revoke，Drop 只作无阻塞 fallback。
10. **P1-10** bus closing 后 register/publish/drain 返回 Closed。
11. **P1-11** delivery page 增加 cursor、remaining、oldest age、ack。
12. **P1-12** drain 失败保留未处理 tail，不整箱静默删除。
13. **P1-13** Latest 驱逐返回精确 evicted key 与 resync request。
14. **P1-14** Bounded drop 返回 sequence/range 与 oldest-age 诊断。
15. **P1-15** sequence 在跨进程/serde/replay DTO 中保持一致。
16. **P1-16** single message/inbox/retained bytes 预算绑定 subscriber policy。
17. **P1-17** 实际内存测量与 logical estimate 分开记录。
18. **P1-18** request timeout/cancel/panic/target-retire 有 typed terminal result。
19. **P1-19** request callback lease revalidation 与 deadline enforced。
20. **P1-20** request publish/reentry 不死锁且不会绕过 capability。
21. **P1-21** plugin callback begin/consume/end 增加 panic isolation。
22. **P1-22** poison delivery 进入 dead-letter/quarantine，并保留 receipt。
23. **P1-23** slow subscriber 不阻塞 required consumer，仍保持公平。
24. **P1-24** plugin pending queue 纳入总 count/bytes/age budget。
25. **P1-25** plugin bridge 不得把 bounded bus 转成无界 shadow queue。
26. **P1-26** retryable/permanent/poison callback error policy 分离。
27. **P1-27** Job Started/terminal delivery 使用 reservation/ack，失败可重试。
28. **P1-28** Job progress latest coalesce 不跨越 terminal barrier。
29. **P1-29** Transaction event delivery failure 影响 receipt/diagnostic，而非只写 warning。
30. **P1-30** zero-consumer transaction/log/job 不再误称 Delivered。
31. **P1-31** Scene inspection dirty 提交与 world generation/watermark 原子绑定。
32. **P1-32** structure-only、field delta、compact resync 三种 payload 明确区分。
33. **P1-33** World Sync bus rejection 不推进 generation watermark（Editor47 owner）。
34. **P1-34** Document/Play bridge 按 document/play generation 过滤 stale event。
35. **P1-35** Tool/Focus/Log/I18n/Custom topic 建立唯一 consumer/adoption matrix。
36. **P1-36** topic schema mismatch、unknown namespace、oversize、depth、ID length fail-closed。
37. **P1-37** Custom topic 通过 capability registry，不允许字符串旁路。
38. **P1-38** scene/asset/selection scope 防止跨 document/world/PIE coalesce。
39. **P1-39** callback 失败期间保留 last-known-good projection。
40. **P1-40** startup registration failure 产生 typed degraded state，不 panic 退出。
41. **P1-41** explicit shutdown 编排 producer stop、consumer drain/discard、lease revoke、bus close。
42. **P1-42** diagnostics 记录 owner/topic/generation/depth/bytes/oldest age/pressure。
43. **P1-43** Editor09 jobs 承载 resync/cancel/progress/shutdown。
44. **P1-44** Editor11 logging 不默认记录敏感 payload，并保留 message correlation。
45. **P1-45** 现有 retention、sequence、bytes、request tests 增加 deterministic race。
46. **P1-46** 增加 publish/unregister、request/retire、shutdown/publish barrier tests。
47. **P1-47** 增加 zero-route dirty、bus reject、backpressure、resync integration tests。
48. **P1-48** 增加 plugin panic/slow/poison/partial backlog tests。
49. **P1-49** 增加 1/5/100/10K subscriber与100K delta性能和内存测试。
50. **P1-50** required correctness、managed performance、real-runtime E2E lane 分离。
51. **P1-51** 文档、capability manifest、shutdown runbook、failure handoff保持 currentness。
52. **P1-52** 删除假 subscriber、无界 shadow queue、false Delivered 与不受治理的 raw topic 旁路。

### 3.3 P2：15 项

1. **P2-01** durable message journal、replay cursor 与 checkpoint。
2. **P2-02** cross-process/remote message transport 与 version negotiation。
3. **P2-03** topic QoS、priority、adaptive page sizing。
4. **P2-04** content-addressed payload dedup 与 zero-copy handoff。
5. **P2-05** topic health dashboard、backlog heatmap 与 operator controls。
6. **P2-06** plugin topic marketplace、schema certification 与 revocation。
7. **P2-07** million-message indexed query 与 archive retention。
8. **P2-08** deterministic event time-travel、record/replay。
9. **P2-09** distributed subscriber groups、federated bus relay。
10. **P2-10** privacy/redaction policy per topic and field。
11. **P2-11** UI virtualized message inspector、filter、search、export。
12. **P2-12** adaptive backpressure based on frame budget and health。
13. **P2-13** chaos network/process crash/long-session soak。
14. **P2-14** cross-platform ordering/latency/memory benchmark。
15. **P2-15** 将 message evidence 与 Runtime Gateway/Scene Snapshot/Collaboration 统一 provenance browser。

## 4. 目标架构与里程碑

```text
Typed Producer -> Route Snapshot -> Admission/Retention -> Delivery Page
      |                                          |
  Dirty Authority                         Consumer Lease
      |                                          |
  Commit/Receipt <- Ack/Nack/Retry <- Callback Fault Domain
```

### M0-M4

- **M0**：修复 zero-route dirty P0，所有固定 refresh path 有 authoritative fallback；禁止假 subscriber。
- **M1**：冻结 topic/schema/owner/generation/sequence/dispatch disposition、lease 和 page cursor。
- **M2**：实现 unregister/retry/ack/nack、plugin panic/slow/poison fault domain 和 bounded pending。
- **M3**：实现 World/Scene dirty commit、Job/Transaction/Document/Play adoption matrix、resync/last-good projection。
- **M4**：实现 explicit shutdown、diagnostics、race/fault tests、1/5/100/10K scale 与 required E2E。

## 5. 验收门

1. **G01-G05** zero-route dirty、oversize、sequence exhaustion、NoRoute/Accepted/Rejected/Closed 和 stale scope 通过。
2. **G06-G10** Lossless retry、Latest eviction、Bounded drop、unregister race、lease revoke 通过。
3. **G11-G15** page cursor/ack/remaining、serde sequence、request timeout/panic/reentry 和 topic schema 通过。
4. **G16-G20** plugin panic/slow/poison、pending budget、Job terminal、Transaction receipt、Scene watermark 通过。
5. **G21-G25** Document/Play generation、Custom capability、last-good projection、startup degraded、shutdown order 通过。
6. **G26-G30** diagnostics correlation、Editor09/11 integration、fuzz/malformed payload、cross-scope coalesce 与 deterministic barriers 通过。
7. **G31-G35** 1/5/100/10K subscriber、100K delta、memory/backpressure、managed-vs-required lanes 和 platform metrics 通过。
8. **G36-G40** message docs/manifest/runbook、Editor47/02 owner acceptance、`git diff --check`、Markdown/path/link/fingerprint 全通过。

## 6. 本轮验证与限制

本轮只做静态源码、测试 inventory、参考源码和 fingerprint 复核；没有修改 Editor、Runtime、Interface 或 tests，也没有运行 Cargo、bus race、callback panic、backpressure、shutdown 或大规模动态验证。frontmatter 路径需在实施前重新展开；P0/P1/P2=1/52/15、M0-M4 和 40 门是本报告收尾门。Editor02/09/11/33/47 已拥有的父问题不在本报告重复计数，整体 review 仍保持进行中。
