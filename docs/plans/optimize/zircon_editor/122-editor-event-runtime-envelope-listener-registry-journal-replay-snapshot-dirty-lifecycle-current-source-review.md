---
title: Editor Event Runtime、Envelope、Listener Registry、Journal、Replay、Snapshot、Dirty 与 Lifecycle 当前源码复核
category: zircon_editor
report_id: Editor122
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor49
refreshes:
  - docs/plans/optimize/zircon_editor/49-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-product-integration-review.md
related_code:
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/execution_outcome.rs
  - zircon_editor/src/ui/host/editor_event_execution/undo_policy.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/event_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/automation.rs
  - zircon_app/src/entry/entry_runner/editor/composition.rs
  - zircon_app/src/entry/entry_runner/editor/project_automation.rs
tests:
  - zircon_editor/src/tests/editor_event
  - zircon_editor/src/ui/retained_host/app/tests/retained_host_automation.rs
  - zircon_app/tests/editor_mvp_authoring.rs
plan_sources:
  - docs/zircon_editor/core/editor_event.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-editor-event-journal-listener-unbounded-retention.md
  - docs/plans/zircon_editor/editor/02/2026-07-18-editor-event-retention-and-lock-split.md
  - docs/plans/performance/01/2026-08-15-editor-event-retention-routing-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-editor-event-input-transaction-audit-current-architecture-review.md
  - docs/plans/mvp/06-f5-acceptance-wave.md
  - docs/plans/optimize/zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Delegates/MulticastDelegateBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/ScopedTransaction.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorTransaction.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/godot/core/object/object.cpp
  - dev/godot/core/object/message_queue.h
  - dev/godot/core/object/message_queue.cpp
  - dev/godot/core/object/undo_redo.cpp
  - dev/Fyrox/editor/src/message.rs
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Util/MessageManager.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Tests/Editor/UnitTests/MessageManagerTests.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 122 · Editor Event Runtime / Envelope / Listener / Journal / Replay 工程化差距

## 1. 结论

`core::editor_event` 已从早期无界 Vec、全局锁 fanout 演进出真实底座：event ID 与 arrival sequence/revision 分离；journal 与 listener inbox 共享 immutable `Arc<SharedEditorEventRecord>`；DurableReplay/FrameLocal/LatestState 各自有 count/byte/age 预算；latest replacement 有 key index；listener page 有独立 delivery cursor；registry 只在短锁内复制 immutable route snapshot，filter/enqueue 在锁外；drop/coalesce/lag 有诊断。旧 failure 中“无界、深拷贝、registry 持锁回调”的结论已过时。

本轮最严重的问题在 F5 automation：retained-host callback 前后各深拷贝完整 global journal，用前一次 `records().len()` 作为后一次 slice 起点，再把所有记录归因给当前 binding；retention eviction/latest coalesce 可使长度缩短而越界，其他并发或 refresh 记录会被误归因。随后 `normalize_cli_action_records()` 又覆盖 `source=Cli` 与 binding path，测试因此固化伪 provenance。正确做法是 callback 直接返回 qualified `ActionInvocationReceipt`，F5 只消费 receipt，不猜 journal delta。

`EditorEventReplay` 目前可重放所有 record，包括 pointer、transient、save/import/close 和外部副作用；失败 record 也会再次执行。pointer move 在零 listener 时仍支付 command reverse lookup、shell lock、effect/result 分配、record clone、JSON size 编码、journal index 和 fanout，Latest 只限制队列长度不限制输入热路径开销。`begin_event()` 又在执行前推进全局 revision，使失败、no-op 和 presentation 被伪装成 authoring commit。

listener control 仍无 production consumer，delivery DTO 不带完整 event/effect/binding/transaction/save/revision/undo；这套结构可作为本地轮询容器，却不能直接作为 replay、远程控制、恢复或审计协议。Editor48 负责通用 Message Bus，Editor02 负责 transaction/save，Editor08 负责 command/remote admission，Editor14 负责 animation 语义，F5 evidence 归 App owner；本报告只负责 event-specific execution receipt、audit envelope、listener/replay contract。

本轮 Zircon scope 为 80 files / 14,775 lines / 13,507 non-empty / 524,029 bytes / 212 test attributes；参考 scope 为 15 / 16,861 / 14,141 / 566,660 / 3；union 为 95 / 31,636 / 27,648 / 1,090,689 / 215。Zircon fingerprint `da786ae4a0ebe851ede0259cc5b9531c8d100a3faa6f1444e17daa48821c8e45`，refs `24611a87e312571057a897729d80b03505f46da35895b351a2a7df6ce0cbc97a`，union `6005d1d0d05894054533ee42fe5554b542302feed4ea5546faad7be10fec8966`。本报告登记 5 个 P0、60 个 P1、15 个 P2 与 40 个 gate；不修改生产代码。

## 2. 当前实现事实与参考差异

1. 事件 retention、listener cursor、shared record 和 route snapshot 是可保留基础；需增加 qualified receipt、generation 和 disposition，而非退回深拷贝。
2. F5 用 journal 长度差，不具备 action identity、causal parent、commit fence 或 retention-safe continuation。
3. 事后 provenance mutation 混淆 initiator、transport、executor、binding 和 observed callback，破坏 audit immutability。
4. raw replay 不区分 committed operation、failed attempt、input、presentation、external request 和 audit observation。
5. `begin_event()` 先推进 revision；失败/no-op/input/presentation 不能获得 DocumentRevision。
6. pointer 高频输入同步进入完整 command/audit/listener 路径，缺少 frame-boundary coalesce 与 edge order 合同。
7. listener 无 production consumer，replay 仅测试调用；公开 method 存在不代表产品采用。
8. Unreal delegates/transactions/message router、Godot MessageQueue/UndoRedo、Bevy message cursor、Fyrox command/message、Unity MessageManager 都把 delivery、transaction、cursor、queue 和 lifecycle 分层；不能以一个 `EditorEventRecord` 覆盖所有语义。

## 3. 差距清单

### 3.1 P0：5 项

1. **P0-01** F5 禁止用 global journal length/slice 生成 action receipt；retention shrink/coalesce、并发无关事件和多 receipt action 必须不 panic、不丢失、不误归因。
2. **P0-02** 禁止 `normalize_cli_action_records()` 改写已提交记录；initiator/transport/executor/binding 必须在 dispatch 时不可变签发。
3. **P0-03** raw `EditorEventRecord` 不得作为可执行 replay；失败、输入、presentation、save/import/close 和外部副作用默认只进 audit。
4. **P0-04** pointer move 不得同步支付完整 command/audit/listener/journal 路径；只在 semantic commit 时产生 receipt。
5. **P0-05** revision 必须从 event order/presentation/document/transaction generation 分离，成功 changed commit 才推进 DocumentRevision。

### 3.2 P1：60 项

1. **P1-01** 拆分 RealtimeInput、EditorCommandIntent、PresentationDelta、AuditEnvelope、CommittedOperationEntry。
2. **P1-02** 定义 qualified ActionInvocationId、EventId、OperationId、TransactionId。
3. **P1-03** 定义 source/transport/executor/binding 四个不可变 provenance 维度。
4. **P1-04** 定义 owner、document、world、generation、revision 传播规则。
5. **P1-05** receipt 携带 causal parent、request ID、selection scope、terminal disposition。
6. **P1-06** receipt 记录 effects、before/after digest、dirty/history/save generation。
7. **P1-07** journal record 增加 schema、codec、redaction、retention class。
8. **P1-08** `EditorEventRecord` 与 `CommittedOperationEntry` 采用不同类型。
9. **P1-09** canonical serialization 不允许 evidence adapter 修改记录。
10. **P1-10** F5 直接消费 callback ActionReceipt，删除 journal slice。
11. **P1-11** action 多 child receipt、zero receipt、partial receipt 有明确 policy。
12. **P1-12** retention eviction/coalesce 不改变已签发 receipt digest。
13. **P1-13** event/revision/transaction/save generation 各自定义 overflow 和 stale policy。
14. **P1-14** failure/no-op/input/presentation DocumentRevision advance 为 0。
15. **P1-15** successful changed commit 恰好推进一次 scoped revision。
16. **P1-16** pointer/resize/transient 在 frame boundary coalesce，press/release/cancel 保序。
17. **P1-17** realtime path 不访问 command registry、transaction、journal 或 listener registry。
18. **P1-18** realtime state 按 viewport/document scope 隔离，禁止跨 scope coalesce。
19. **P1-19** event dispatch 只在 semantic command 生成 execution receipt。
20. **P1-20** audit observation 按显式 count/byte/age/deadline budget 派生。
21. **P1-21** replay 输入限定为 versioned CommittedOperationEntry。
22. **P1-22** replay 验证 target identity、schema、precondition、idempotency、side-effect policy。
23. **P1-23** replay precondition 失败在 apply 前拒绝。
24. **P1-24** replay 失败 rollback/compensating/unknown 状态可审计。
25. **P1-25** replay 不递归产生默认可执行 replay record。
26. **P1-26** legacy replay ambiguity fail closed 并保持零 mutation。
27. **P1-27** replay final state/hash、revision、transaction、typed outcome 可验证。
28. **P1-28** journal page 只按 count/bytes/deadline/cursor 输出，不允许 full snapshot。
29. **P1-29** listener subscription 拥有 owner/principal/capability/generation/affinity。
30. **P1-30** listener delivery page 携 first/last sequence、remaining、oldest age、ack。
31. **P1-31** unregister 有 Drain/Reject/DiscardWithReceipt 三态。
32. **P1-32** route unregister 后旧 route 不得投递 orphan inbox。
33. **P1-33** listener disable/filter update 返回 effective generation/cursor。
34. **P1-34** page count/bytes/deadline/gap/resync/final 预算完整。
35. **P1-35** ack 拒绝 foreign/stale/future cursor 且幂等。
36. **P1-36** callback fault/slow/poison 进入 quarantine/dead-letter policy。
37. **P1-37** callback retry 不重复 commit，不丢未处理 tail。
38. **P1-38** listener registry 使用 generation fence 和 snapshot lifetime。
39. **P1-39** listener 真实 production consumer 完成 register/page/ack/resync/revoke。
40. **P1-40** editor event 与 Editor48 Message Bus 只在明确 ABI 边界投影一次。
41. **P1-41** Document/Transaction/Play/Animation/Command topic 建立 producer-consumer matrix。
42. **P1-42** zero listener/zero subscriber 的 event/audit/transaction disposition 不再称 Delivered。
43. **P1-43** custom event namespace/schema/capability/unknown policy 完整。
44. **P1-44** ID/cursor exhaustion 显式拒绝，不 wrap/saturate/reuse。
45. **P1-45** serialized event 不保存 local path、raw UI route 或 process-local object ID。
46. **P1-46** command execution、journal append、listener page、F5 receipt 有 correlation identity。
47. **P1-47** Editor02 transaction/save generation 与 event receipt 原子关联。
48. **P1-48** Editor08 remote principal/capability 在 event dispatch 前验证。
49. **P1-49** Editor14 animation event 只使用真实 sequence/clip domain contract。
50. **P1-50** F5 evidence adapter 只投影 receipt，不生成或改写 event record。
51. **P1-51** 添加 retention shrink/coalesce/concurrency/unrelated record RED tests。
52. **P1-52** 添加 failure/no-op/input/replay external side-effect RED tests。
53. **P1-53** 添加 replay precondition/rollback/unknown/deterministic tests。
54. **P1-54** 添加 pointer 125/500/1,000Hz coalesce/edge-order tests。
55. **P1-55** 添加 listener register/unregister/ack/gap/resync tests。
56. **P1-56** 添加 callback panic/slow/poison/pending-tail tests。
57. **P1-57** 添加 F5 action receipt/provenance immutability integration tests。
58. **P1-58** 添加 1/5/100/10K listener 与 journal page memory/latency tests。
59. **P1-59** 添加 event schema/version/unknown/redaction/fuzz tests。
60. **P1-60** 删除 global journal slice、source mutation、raw replay 和 revision-before-execute 旁路。

### 3.3 P2：15 项

1. **P2-01** durable audit segments、checkpoint、replay cursor 与 archive。
2. **P2-02** remote event transport、mixed-version negotiation 与 cross-process identity。
3. **P2-03** payload dedup、compression、zero-copy page 与 content address。
4. **P2-04** listener QoS、priority、adaptive page sizing。
5. **P2-05** event health dashboard、lag heatmap、operator controls。
6. **P2-06** domain custom replay/resolution policy marketplace。
7. **P2-07** million-record indexed query、retention tier、GC。
8. **P2-08** deterministic time-travel debugging 与 event scrubber。
9. **P2-09** privacy/redaction policy per field/topic/consumer。
10. **P2-10** UI event inspector、filter、search、export。
11. **P2-11** adaptive input budget based on frame health。
12. **P2-12** plugin event contract certification、revocation 与 unload safety。
13. **P2-13** chaos publish/replay/listener/shutdown long-session soak。
14. **P2-14** cross-platform event ordering、memory、latency benchmark。
15. **P2-15** 与 Snapshot/Merge、Collaboration、Runtime Gateway 统一 provenance browser。

## 4. 目标架构与里程碑

```text
Raw/Input -> typed Intent -> execution preflight -> CommittedOperationEntry
                                      |                 |
                             ActionInvocationReceipt  AuditEnvelope
                                      |                 |
                         bounded Listener Page/Ack/Resync
```

### M0-M6

- **M0**：F5 receipt hard cut、provenance immutability、pointer/revision P0 封口。
- **M1**：event/intent/audit/committed types、identity、generation、schema、retention policy 冻结。
- **M2**：replay 只接受 committed entry，precondition/side-effect/rollback 完成。
- **M3**：listener lease、page/ack/gap/resync、fault/quarantine/shutdown 完成。
- **M4**：Editor02/08/14/48/Runtime Gateway integration matrix 完成。
- **M5**：F5/App evidence、diagnostics、redaction、bounded storage 完成。
- **M6**：1/5/100/10K listener、high-frequency input、fault、cross-platform benchmark 与 required E2E 完成。

## 5. 验收门

1. **G01-G05** F5 不读取 global journal 长度；receipt/provenance immutable；retention/concurrency 不 panic/误归因。
2. **G06-G10** pointer 热路径无 command/audit/listener/journal；edge order/coalesce；revision 只在成功 changed commit 推进。
3. **G11-G15** replay committed-only、precondition、side-effect、rollback、deterministic hash 通过。
4. **G16-G20** listener identity/lease/page/ack/gap/resync/unregister/fault 通过。
5. **G21-G25** bus/transaction/remote/animation/F5 integration、zero-route、custom schema、redaction 通过。
6. **G26-G30** retention/page budget、correlation、shutdown、fuzz、unknown policy 通过。
7. **G31-G35** high-frequency input、1/5/100/10K listener、memory/latency、cross-platform metrics 通过。
8. **G36-G40** required correctness、managed performance、F5 product evidence、source currentness、docs/index/link/fingerprint 全部通过。

## 6. 本轮验证与限制

本轮只做静态源码、测试 inventory、参考源码与 fingerprint 复核；没有修改 Editor、Runtime、App、Interface 或 tests，也没有运行 Cargo、F5、replay、retention、pointer、listener fault 或性能动态验证。frontmatter 路径需在实施前重新展开；P0/P1/P2=5/60/15、M0-M6 和 40 门是本报告收尾门。Editor02/08/14/48、App07 和 Runtime/Interface parent owner 的既有 findings 不在本报告重复计数；整体 review 仍保持进行中。
