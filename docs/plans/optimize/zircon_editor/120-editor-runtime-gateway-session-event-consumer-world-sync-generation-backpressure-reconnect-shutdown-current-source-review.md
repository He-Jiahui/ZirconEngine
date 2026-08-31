---
title: Editor Runtime Gateway、Session、Event Consumer、World Sync、Generation、Backpressure、Reconnect 与 Shutdown 当前源码复核
category: zircon_editor
report_id: Editor120
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor47
refreshes:
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
related_code:
  - zircon_editor/src/core/gateway
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/core/sync
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/ui/host/editor_manager_minimal_host.rs
  - zircon_editor/src/ui/host/editor_world_sync.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor/composition.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/world_sync.rs
tests:
  - zircon_editor/src/core/gateway/session/tests.rs
  - zircon_editor/src/core/sync/pump/tests.rs
  - zircon_editor/src/core/sync/watch_map/tests.rs
  - zircon_editor/src/tests/gateway
  - zircon_editor/src/tests/runtime_event_consumer.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump
plan_sources:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/01/failure-2026-07-17-gateway-stable-call-lock-and-clone.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-runtime-event-consumer-unbounded-pump-lock.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-22-world-sync-subscription-invalidation-scaling.md
  - docs/plans/zircon_editor/editor/02/failure-2026-08-01-plugin-registration-runtime-consumer-atomicity.md
  - docs/plans/zircon_editor/editor/01/failure-2026-07-31-highlight-set-gateway-contract.md
  - docs/plans/zircon_editor/editor/01/failure-2026-08-13-editorui10-test-budget-gateway-session.md
  - docs/plans/zircon_editor/editor/02/failure-2026-08-13-editorui10-test-budget-message-runtime-event.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_runtime_interface/04-profiling-plugin-event-script-diagnostic-manifest-crate-ownership-consolidation-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Public/ISessionManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Private/SessionManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Private/SessionManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Private/SessionInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Private/SessionInfo.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bridge/MessageBridge.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bridge/MessageBridge.cpp
  - dev/godot/editor/debugger/editor_debugger_node.h
  - dev/godot/editor/debugger/editor_debugger_node.cpp
  - dev/godot/editor/debugger/script_editor_debugger.h
  - dev/godot/editor/debugger/script_editor_debugger.cpp
  - dev/Fyrox/editor/src/message.rs
  - dev/Fyrox/editor/src/plugin.rs
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/builtin_methods.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 120 · Editor Runtime Gateway / Session / World Sync / Event Consumer 工程化差距

## 1. 结论

Editor 到动态 Runtime 的底层边界已有真实工程基础：`EditorRuntimeGatewayHandle` 用 `ArcSwap` 发布不可变 `GatewayGeneration`，`SessionGateway` 持有 `Arc<RuntimeSession>`，foreign output 有统一预算/release/fuse，plugin event producer 有 64 deliveries / 256 KiB bounded page、sequence、remaining、oldest age，`WorldWatchMap` 使用索引而非全量视图扫描，consumer 有 lifecycle atomic gate、round-robin 和 pending tail 恢复。这些修复必须保留。

阻断在多步协议的代际一致性。`WorldSyncPump::pump()` 分别读取 generation 与 drain invalidation；event consumer 先读 session handle，再按旧 subscription drain/unsubscribe；`ActiveConsumer` 没有保存创建时 gateway generation、Runtime session identity 或 origin lease。Play runtime 在两步之间替换后，新的 session 可以复用 opaque token/subscription 数值，旧 watch map 或 callback 就可能消费新 session 数据。这是跨 PIE/project 的身份污染，不是 Arc 悬空。

World page DTO 没有 page sequence/cursor/remaining/final marker/oldest age；consumer 只拒绝 generation 回退，无法识别同 generation 的缺页、重复页或 gap。它逐 fact 调 Editor bus、忽略 dispatch disposition，却推进 `published_facts` 与 `last_generation`，因此 rejection/backpressure/subscriber fault 会被记录成已提交并永久越过恢复点。gateway replacement 还会清空 watch，未完成 rebind/full-resync 就切换新 session。

callback 虽在 map lock 外执行，仍没有 panic fault domain：`begin_session` panic 可能泄漏 remote subscription，`consume` panic 丢当前 delivery，`end_session` panic 中断本地清理。每 tick 还 clone 完整 capability snapshot 并 diff 全 registration；backlog aggregate 使用全有或全无折叠，一个未 drain consumer 会遮蔽其他 consumer 的真实积压。

退出路径没有统一 terminal coordinator。Host Drop 只注销各自 subscriber/watch，菜单退出还可能等待 remote unsubscribe 成功；transport 丢失时 Editor 可长期停在 Playing。应建立 session-qualified lease、page commit/resync、callback isolation、reconnect 和显式 shutdown choreography。本轮登记真实的 1 个 P0、44 个 P1、12 个 P2、36 个 gate，不重复 Runtime43、Interface02/04/05、Editor07 的父 owner。

本轮选定 Zircon scope 为 140 files / 26,744 lines / 24,362 non-empty / 973,967 bytes / 343 test attributes；参考 scope 为 15 / 9,968 / 8,507 / 347,606 / 13；union 为 155 / 36,712 / 32,869 / 1,321,573 / 356。Zircon fingerprint `bfb39666ac5b56fff636ce6c91cf9cdd40f3a8a8e9bf29fb51f86d9db119caaa`，refs `70e7e85de8165d8c66b5a7b60d182308f301259ac654b4932a8ab2bdea58f55a`，union `30fbf02045a22719b5ffceb2215ff1513165e342669989a47e1723c3319d0d80`。不修改生产代码。

## 2. 当前实现事实

1. ArcSwap generation、RuntimeSession owner、foreign output safety、bounded event page、producer prepare/commit/rollback、indexed watch map 和 round-robin pump 是可保留基础。
2. `ensure_hierarchy_world_watch`、WorldSync pump、runtime consumer reconcile/pump 和 Host Drop 不是一个共同的 session lease/terminal state machine。
3. 同一次逻辑 tick 的多次 ArcSwap load 可以跨越 generation replacement；opaque token 只在本地 map 里解释，不能证明 origin session。
4. World invalidation page 缺 cursor/final/gap，bus dispatch report 不参与 page commit；last generation 可能在 fact 未投影时前进。
5. callback panic 没有 per-consumer quarantine/dead-letter/terminal disposition；slow/poison delivery 的 pending bytes 和 retry 语义不完整。
6. capability snapshot 不变时仍 clone/diff 全 registration；backlog summary 不能表达 lower bound 与 unknown count。
7. replacement 清空 watch，transport loss 要求远端 unsubscribe，Drop 不保证所有 consumer/watch/output/session 按顺序退休。
8. Unreal SessionManager/MessageBridge、Godot debugger disconnect、Fyrox message/plugin、Bevy Remote 均把 session identity、disconnect、message/error 与 teardown 分开建模；Zircon 需吸收边界而非复制 API。

## 3. P0 与 P1/P2 差距

### 3.1 P0

1. **P0-01** 代际污染必须先封口：A generation/session 创建的 watch/subscription 不能被 B session drain/unsubscribe，即使 opaque token 数值相同；replacement 期间必须有 qualified lease、resync 和 old retirement receipt。

### 3.2 P1：44 项

1. **P1-01** 定义 `GatewaySessionIdentity`、`RuntimeSessionIdentity`、`WorldIdentity`、`SubscriptionOrigin`。
2. **P1-02** 定义 generation-qualified `GatewayLease` 与跨步调用边界。
3. **P1-03** watch/subscription token 携带 origin session/generation namespace。
4. **P1-04** 统一 owner/generation/request/sequence/receipt 传播。
5. **P1-05** World page 增加 cursor、page sequence、remaining、final、oldest age。
6. **P1-06** plugin page 增加 producer/session identity 与 schema version。
7. **P1-07** consumer 识别 duplicate、gap、generation gap、stale page。
8. **P1-08** 只有 final page 且 bus commit 成功时推进 watermark。
9. **P1-09** bus rejection/backpressure 进入 page disposition 和重试队列。
10. **P1-10** full baseline/resync 与增量 page 绑定同一 world revision。
11. **P1-11** replacement 实现 quiesce、publish、rebind、resync、retire 状态机。
12. **P1-12** replacement receipt 记录 old/new session、watch、subscription、output 顺序。
13. **P1-13** `ActiveConsumer` 保存 origin generation/session/lease。
14. **P1-14** capability/registration generation 不变时 reconcile 走 O(1) fast path。
15. **P1-15** capability downgrade 只退休受影响 consumer 并给出兼容 receipt。
16. **P1-16** callback begin/consume/end 增加 panic boundary 与 fault classification。
17. **P1-17** begin panic 不泄漏 remote subscription 或 active entry。
18. **P1-18** consume panic 隔离 consumer、保留 tail、生成 poison/dead-letter receipt。
19. **P1-19** end panic 仍完成本地 terminal retirement。
20. **P1-20** retryable/permanent/poison delivery 有不同 policy。
21. **P1-21** slow consumer quarantine 不阻塞公平 consumer。
22. **P1-22** pending delivery 和 dead-letter bytes 受总预算约束。
23. **P1-23** backlog report 表达 lower bound、unknown、oldest age 和 sample。
24. **P1-24** callback deadline、cancel、shutdown disposition typed 化。
25. **P1-25** Host active tick 绑定单一 lease，禁止 generation 混用。
26. **P1-26** hierarchy watch 与 world sync watch 共用 qualified world source。
27. **P1-27** plugin registration prepare/install 与 consumer activation 原子衔接。
28. **P1-28** transport loss 触发本地 Degraded/Stopped，不等待 remote ack。
29. **P1-29** disconnect/reconnect 有 explicit initial sync、gap、retry、backoff。
30. **P1-30** shutdown coordinator 编排 consumer end、watch unregister、output release、gateway detach、session destroy。
31. **P1-31** Drop 只做无阻塞 fallback，不承担未知耗时 cleanup。
32. **P1-32** Host startup 任一阶段失败逆序回滚已提交资源。
33. **P1-33** PlayInstance/project switch/PIE restart 不误伤新 session。
34. **P1-34** output/frame release 在 session destroy 前可证明完成或记录 fuse。
35. **P1-35** V7 ABI、capability limits、page schema mismatch 有 typed unavailable。
36. **P1-36** Editor bus publication 与 projection dirty/refresh 使用 commit receipt。
37. **P1-37** diagnostic 记录 session/page/cursor/backlog/quarantine/resync/shutdown identity。
38. **P1-38** Editor09 jobs 承载 reconcile/resync/cancel/progress/shutdown budget。
39. **P1-39** 接入 Editor25 observation，不复制 diagnostics authority。
40. **P1-40** 建立 A-B-A、replacement 中 detach、callback panic 并发测试。
41. **P1-41** 建立 same-generation multi-page、duplicate、gap、bus reject 测试。
42. **P1-42** 建立 1K/10K subscriptions/watches、slow consumer、backlog、replacement 性能测试。
43. **P1-43** 区分 required correctness、managed performance、managed real-runtime E2E lane。
44. **P1-44** 所有旧 V6/V7、failure handoff、ABI layout 和产品 exit/reconnect 文档完成 currentness recheck。

### 3.3 P2：12 项

1. **P2-01** 多 session checkpoint archive、跨项目 diagnostics 和 replay browser。
2. **P2-02** world page compression、dedup、content-addressed cache。
3. **P2-03** remote viewport/Inspector/Outliner stream 的 interest management。
4. **P2-04** subscription QoS、priority、adaptive page sizing。
5. **P2-05** durable event audit、operator replay 与 privacy policy。
6. **P2-06** cross-platform ABI negotiation 与 mixed-version window。
7. **P2-07** deterministic session replay 与 time-travel diagnostics。
8. **P2-08** GPU/IO completion fence 与 zero-copy output handoff。
9. **P2-09** multi-runtime fan-out、remote worker 和 relay。
10. **P2-10** long-session leak/chaos/network partition soak。
11. **P2-11** host lifecycle visualization、watermark timeline 和 health dashboard。
12. **P2-12** 以相同 page/delivery/diagnostic 完整度建立超过参考实现的 gateway benchmark。

## 4. 目标架构与里程碑

```text
App RuntimeSession -> GatewayGeneration -> Qualified GatewayLease
Lease -> prepare watches/subscriptions -> initial sync
active tick -> page validate(cursor/generation/session) -> bus commit -> watermark
replacement -> quiesce -> publish -> rebind -> resync -> retire
shutdown -> end callbacks -> unregister -> release outputs -> detach -> destroy session
```

| Milestone | 退出条件 |
|---|---|
| M0 | A/B session token collision、generation interleave、transport-loss exit 具备 RED tests 并封口。 |
| M1 | qualified identity/lease、page envelope、cursor/final/gap、commit disposition 冻结。 |
| M2 | callback panic/poison/slow consumer fault domain 与 metrics 完成。 |
| M3 | replacement/reconnect/resync/retire coordinator 完成。 |
| M4 | explicit Host/App shutdown、rollback、output/session ownership receipt 完成。 |
| M5 | diagnostics、jobs、1K/10K scale、required correctness/E2E qualification 完成。 |

## 5. 验收门

1. **G01-G06** origin session/generation/token 不碰撞；replacement lease 不持有全局长锁；old quiesce/new publish/rebind/resync/retire 顺序可观测。
2. **G07-G12** page cursor/final/gap/duplicate、watermark、bus disposition、full baseline 与 incremental revision 通过。
3. **G13-G18** callback panic、poison/dead-letter、slow consumer、retry/cancel/partial backlog 通过 fault matrix。
4. **G19-G24** capability downgrade、transport loss、reconnect、session/project/PIE switch、output release 与 local terminal state 通过。
5. **G25-G30** explicit shutdown、startup rollback、Drop fallback、ABI mismatch、diagnostics/jobs、A-B-A concurrency 通过。
6. **G31-G36** 1K/10K budget、required-vs-managed lane、Windows ABI/product exit/reconnect E2E 和 source currentness 全部达标。

## 6. 本轮验证与限制

本轮只做静态源码、测试 inventory、参考源码与 fingerprint 复核；没有修改 Editor、App、Runtime、Interface 或 tests，也没有运行 Cargo、多进程替换、callback panic、bus fault、reconnect 或 shutdown 动态验证。frontmatter 路径需在实施前重展开；本文不重新计数 Runtime43/Interface02/04/05/Editor07 已拥有的底层 findings。整体 review 仍保持进行中。
