---
title: Editor Event Runtime、Envelope、Listener Registry、Journal、Replay、Snapshot、Dirty 与 Lifecycle 当前源码复核
category: zircon_editor
report_id: Editor170
review_date: 2026-08-27
baseline_head: 64942164497096a82cbb4a721405d9ffe367bccf
production_baseline: 982baa1ba87bc8c25fe44312507a4af15027e058
canonical_owner: Editor49
refreshes:
  - docs/plans/optimize/zircon_editor/49-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/122-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-current-source-review.md
related_code:
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_editor/src/ui/host/editor_event_runtime_access/event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/input_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs
  - zircon_editor/src/ui/retained_host/event_bridge.rs
  - zircon_editor/src/ui/retained_host/app/automation.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input_outcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/pointer_move_mailbox.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs
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

# 170 · Editor Event Runtime / Listener / Journal / Replay 工程化复核

## 1. 最终结论

Editor Event Runtime 已有必须保留的底座：journal 与所有 listener inbox 共享一个不可变 `Arc<SharedEditorEventRecord>`；DurableReplay、FrameLocal、LatestState 分别受 count、logical bytes、age 三重预算约束；latest-state 有索引合并，listener 使用独立 delivery cursor；registry 在短锁内生成 immutable route snapshot，filter 与 enqueue 在 registry 锁外；page 有 1..256 count 上限，status 提供 pending count/bytes、first/last sequence、drop/coalesce/lag 诊断。当前源码也新增 idle mouse-move mailbox：同 device、非按键、无需 immediate dispatch 的 move 只保留最新值，并在后续非 move、device 切换或 lifecycle 边界前 flush。旧报告中“无界 Vec、每 listener 深拷贝、registry 持锁 fanout、所有 pointer move 一律逐条进入 Host”的表述已不再准确。

五项 P0 仍无一关闭。F5 automation 继续在 callback 前后各复制整个 global journal，再用旧 `records().len()` 对新 snapshot 切片；retention 收缩、latest coalesce、并发无关事件、多 child receipt 均会破坏这项假设。`normalize_cli_action_records()` 随后继续覆盖已提交 record 的 source 与 binding，测试仍把伪 provenance 当成期望。`EditorEventReplay` 仍公开接受任意 `EditorEventRecord` 并重新 dispatch 原始 event，包括失败记录、输入、presentation、save/import/close 和外部副作用。`begin_event()` 仍在执行前对全局 revision 做 `saturating_add`，所以失败、no-op、pointer、hover、selection 与纯 presentation 都会伪增 document revision。

P0-04 因 idle pointer mailbox 从 Open 降为 Partial，但不能关闭：pressed/drag/immediate 路径仍逐 move 同步执行；被保留的最新 move 仍进入 native pointer、workbench callback、event dispatch、command reverse lookup、effect/result 分配、JSON size 计算、journal 与 listener fanout。mailbox 只有 event-loop/device 范围，journal 的 `PointerPosition` latest key 却是全局单键，没有 viewport/document/window scope；也没有 125/500/1,000 Hz qualification artifact。

Listener control 仍只被测试调用，没有 production consumer。它是有界本地轮询容器，不是工程级 subscription protocol：descriptor 只有字符串 ID/name/enabled/filter；没有 owner、principal、capability、generation、affinity 或 lease；unregister 只有立即删除；旧 route snapshot 在 unregister 后仍被测试为可向 orphan inbox 入队；ack 接受任意 stale/future/foreign-shaped cursor，并会删除本 listener 所有不大于该数字的 delivery；page 没有 byte/deadline/gap/resync/final/remaining/oldest-age 合同。cursor 到 `u64::MAX` 后使用 saturation，测试明确固化“同 cursor replacement”，这会丢失单调身份而不是 fail closed。

本轮不新增 finding，继续由 Editor49 拥有 5 个 P0、60 个 P1、15 个 P2。当前状态为：P0 **4 Open / 1 Partial / 0 Closed**；P1 **38 Open / 22 Partial / 0 Closed**；P2 **15 Open**；40 个 canonical gate 为 **25 Fail / 15 Partial / 0 Pass**。没有动态 correctness、race、fault、performance 或跨引擎同场景证据，禁止宣称该控制面达到或超过 Unreal。

## 2. 审查边界与 currentness

### 2.1 Owner 与去重

1. Editor170 只刷新 Editor49/122，不重复登记 Editor02 的 transaction/save authority、Editor08 的 command/remote admission、Editor14 的 animation domain、Editor48/169 的通用 Message Bus、App07/F5 的产品证据或 Runtime Gateway 的跨进程 session owner。
2. 本报告拥有 event-specific intent、execution receipt、audit envelope、listener page/ack/resync、replay admission 与 event lifecycle；领域 owner 必须通过这些合同接入，不能把通用底座存在写成产品闭环。
3. Tooling 按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器。

### 2.2 冻结点

| 项目 | 当前值 |
|---|---|
| 当前磁盘冻结时间 | `2026-08-27T15:05:06.3422050+08:00` |
| Git HEAD | `64942164497096a82cbb4a721405d9ffe367bccf` |
| production baseline | `982baa1ba87bc8c25fe44312507a4af15027e058` |
| working tree | 冻结时 `git status --short` 为 8,101 条；本文以 fingerprint 对应的当前磁盘内容而非 HEAD 内容裁决 |
| 动态证据 | 未运行 Cargo、Editor、F5、replay、race、fault、scale 或 benchmark lane |

### 2.3 可复算 selected set

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | Fingerprint |
|---|---:|---|
| Zircon event/runtime/Host/App/tests | **101 / 19,035 / 17,543 / 689,929 / 243 / 3** | `577f7d1d5ee89f0933238a0457e6ebecabe28cb295a99254d124043781a80489` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **15 / 16,847 / 14,141 / 566,660 / 2 / 0** | `125fd751183323dcb06d65960507a055b54d7001be54ef2d519ed4688ff2e537` |
| 全部选择集 | **116 / 35,882 / 31,684 / 1,256,589 / 245 / 3** | `0c47f8ac75991d760916ae86bbb847ae1487a282b5621336c459f1b8d62b7965` |

Fingerprint 算法为 workspace-relative 小写 `/` 路径与逐文件 SHA-256 组成 `path + NUL + hash + LF` 清单，再对清单做 SHA-256。Zircon scope 递归展开 frontmatter 的 owner/test 目录并按物理路径去重；reference scope 是 15 个明确列出的本地文件。工作树变化不会被误并入这个冻结结果，实施前必须重新复算。

## 3. 当前源码事实

### 3.1 类型层仍把输入、意图、展示、审计和提交混为一个 enum

`types.rs` 只有裸 `EditorEventId(u64)`、`EditorEventSequence(u64)` 与五值 `EditorEventSource`。同一个 `EditorEvent` 同时装 Workbench menu、layout、selection、asset、animation、viewport、operation 和 transient；`EditorEventEnvelope` 只有 source/event。`EditorEventRecord` 又同时保存 raw event、binding path、operation metadata、transaction/save generation、effects、undo policy、before/after revision 和 result。它没有 schema version、codec、owner/document/world、causal parent、request/action identity、transport/executor、before/after digest、redaction 或 terminal disposition。

这导致 `EditorEventRecord` 同时被当作 dispatch return、audit DTO、journal item、listener payload、automation evidence 和 replay program。增加字段不能修复语义冲突；必须硬拆 `RealtimeInput`、`EditorCommandIntent`、`ExecutionReceipt`、`AuditEnvelope` 与 `CommittedOperationEntry`。

### 3.2 revision 在执行前分配，失败与 no-op 也被写成 commit

`EditorEventService::begin_event()` 调用 `allocate_stamp(true)`；event ID、sequence、revision 全部用 `saturating_add`，revision 在 `execute_event()` 之前推进。失败分支仍用 stamp 的 after revision 建 record 并 append；成功但 `changed=false` 也得到同一次 revision 增量。这个全局数既不是 document-scoped revision，也不是 transaction generation。

`authoring_trace()` 在执行后通过当前 history top 或 save generation 猜 transaction/save 关联；它不是与 mutation 同一原子 commit 产生的 receipt。两个并发或嵌套 mutation 可以使查询到的 top/generation 不再属于当前 event。

### 3.3 retention 有真实边界，但不能替代 receipt 与 durable journal

`SharedEditorEventRecord::new()` 对每个 record 做一次完整 JSON 编码以估算 logical bytes；journal 和 listener 共享该 payload，避免 fanout 深拷贝。三类 retention 有独立 count/bytes/age budget、drop/coalesce range 和 order index，这是可保留结构。

但 retention class 由 raw event variant 推断，不是版本化 schema policy；Latest key 中 pointer、viewport size、timeline、hover/focus/press 大多没有 document/window/viewport scope。`journal()` 仍构造完整 `Vec<EditorEventRecord>` snapshot，并对所有 retained record 深拷贝。它没有 page/cursor/byte/deadline API、segment/checkpoint、append durability 或 archive，因此 F5 读取 snapshot 长度既昂贵又不具因果性。

### 3.4 listener page 是局部进展，lifecycle 仍不成立

每个 listener 有独立 retention store，route snapshot 是 `Arc<[Route]>`，page 以独立 cursor 合并三个 retention lane 并限制最多 256 条。status 可观察 backlog bytes、first/last sequence、drop/coalesce 与 lag，DTO projection 也在 listener registry 锁外。这些均应保留。

仍需硬切的行为如下：

1. register 返回普通 success JSON，没有 generation-qualified lease。
2. enable/filter update 不返回 effective generation/cursor；旧 snapshot 继续按旧 filter 入队。
3. unregister 删除 registry entry 与 public handle，但旧 route 持有 inbox Arc；现有测试明确允许 unregister 后继续 enqueue。
4. page 只有 count、next cursor、has_more；retention gap 只能另查 status，无法将 gap 与 page continuation 原子关联。
5. ack 只返回 removed count；future cursor 会清空全部已有内容，stale cursor 返回成功 0，cursor 不携 listener/generation，无法拒绝 foreign cursor。
6. mutex poison 被 `into_inner()` 静默恢复；没有 consumer callback supervisor、slow deadline、retry、dead-letter、quarantine 或 shutdown disposition。
7. 生产源码没有 register/page/ack/resync/revoke consumer，所有调用均位于 focused tests 或公开转发入口。

### 3.5 raw replay 是危险公开能力

`EditorEventReplay::replay()` 接受 `&[EditorEventRecord]`，逐项取 `record.event.clone()` 并以 `EditorEventSource::Replay` 再次 dispatch。它只在事后比较预期 error 字符串，不做 target identity、schema、precondition、idempotency、side-effect、rollback 或 recursion policy。失败 record 也会先执行，然后才判断“是否同样失败”。

当前产品生产调用为零不等于安全：公开 re-export 允许未来 caller 直接重放 save/import/close、pointer/transient 和外部请求。正确迁移只能让 replay 接受 transaction owner 签发的 versioned `CommittedOperationEntry`；legacy raw record 必须 fail closed 且零 mutation。

### 3.6 pointer mailbox 降低输入量，但没有拆出 realtime path

`UiIdlePointerMoveMailbox` 对 primary mouse、无按键、无需 immediate dispatch 的连续 move 保留最新 metadata/position，并记录 first/last/count；device 切换和后续非 move/lifecycle 事件前会 flush，focused unit test覆盖 latest replacement 与连续 sequence range。Host 也记录 damaged/no-damage/rejected input outcome 与 bounded present batch。

flush 最终仍调用 `handle_pointer_moved()`，继而执行 native hit/routing、workbench input callback 与 event pipeline。pressed/drag/capture/resize 等 immediate 路径不会 coalesce。这个实现降低 idle storm 次数，却没有建立“realtime state update 不访问 command/transaction/journal/listener”的架构边界，也没有 frame budget、deadline、multi-window scope 或高频动态数据。

### 3.7 F5 仍从观察结果反推因果关系并修改历史

`automation.rs` 在 retained callback 前保存完整 journal snapshot 的长度，callback 后再次 snapshot，再用旧长度切新 vector。即使没有越界，这个 slice 也可包含 callback 触发的 refresh、listener side effect 或其他线程记录；zero/multi child receipt 没有策略。随后 adapter clone 每条 record 并改写 `source=Cli` 与 binding path，测试仍断言这种行为。

F5 adapter 必须消费 callback 同步返回的 qualified `ActionInvocationReceipt`；receipt 在 dispatch/admission 时固定 initiator、transport、executor、binding 与 causal identity。evidence adapter 只能 redacted projection，不能读取 journal delta、生成记录或改写来源。

## 4. 本地参考源码对照

### 4.1 Unreal：分别借鉴 lifecycle、transaction 与 input order

Unreal MessageRouter 使用 receiver address、weak recipient、subscription、message scope、router thread 与 tracer；Multicast Delegate 使用 delegate handle、lifetime tracking、remove/compaction；ScopedTransaction 明确 Begin/End/Cancel，EditorTransaction 将 dirty fence 与 object record 纳入 transaction owner；SlateApplication 对 pointer-up、drag/drop 与 synthetic move 有显式顺序规则。Zircon 当前把这些不同职责压进一个 record/listener store。

所选 Unreal Messaging 也不是 durable ack/replay 的完整上限，不能照抄后宣称 exactly-once。Zircon 应吸收 owner/lifetime/thread/scope/transaction/input boundary，同时自行完成 bounded page、ack/gap/resync 与 receipt。

### 4.2 Bevy：reader cursor 与消息生命周期分离

Bevy Messages 返回 typed `MessageId`，每个 reader 独立持有 `MessageCursor`，并明确双 buffer 更新后何时 missed。它证明 reader progress 应属于 consumer，不应由全局 journal length 推导。Bevy 的 frame-local lifetime 不承担 Zircon 的 durable audit、remote replay 或 listener lease，不能直接作为产品协议。

### 4.3 Godot：bounded deferred queue 与 UndoRedo 分层

Godot CallQueue 有 page/size 边界、thread ownership、flush reentrancy guard 与 clear；UndoRedo 另建 action、do/undo operation、merge 与 commit 语义。它没有把 deferred input/call queue 当成 transaction journal。Zircon 应同样把 realtime/deferred delivery 与 committed operation 分开。

### 4.4 Fyrox：command significance 是最低可接受语义区分

Fyrox CommandTrait 至少区分 significant mutation 与 insignificant selection，提供 execute/revert/finalize；editor message 只是 mpsc sender。其 message channel 不是 listener lifecycle 上限，但 command significance 已比 Zircon“所有 begin_event 都推进 revision”更正确。

### 4.5 Unity Graphics：provider ownership 只提供局部参考

ShaderGraph MessageManager 以 provider/node 归属消息并可按 provider 清理，focused tests 覆盖 dirty/clear/order。它是 diagnostics manager，不是 Unity Editor 全局 event/replay 系统；可借鉴 owner-scoped cleanup，不能用来替代 lease、transaction 或 replay contract。

## 5. Editor49 finding 重判

### 5.1 汇总

| 级别 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 4 | 1 | 0 | 5 |
| P1 | 38 | 22 | 0 | 60 |
| P2 | 15 | 0 | 0 | 15 |

### 5.2 P0

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| P0-01 F5 禁止 global journal length/slice receipt | Open | `automation.rs` 仍两次 snapshot 并按旧长度切片。必须改为 callback 返回 qualified receipt，并覆盖 retention shrink/coalesce/concurrency。 |
| P0-02 provenance 不可事后改写 | Open | `normalize_cli_action_records()` 仍覆盖 source/binding，focused test 固化该行为。必须删除 mutation adapter。 |
| P0-03 raw record 不得执行 replay | Open | `EditorEventReplay` 仍对每个 raw event 重新 dispatch。必须 committed-entry-only，legacy fail closed。 |
| P0-04 pointer move 不走完整审计热路径 | Partial | idle mouse mailbox 合并真实存在；immediate/drag 与每个 flushed move 仍进入完整 callback/event/journal 链，无高频资格。 |
| P0-05 scoped DocumentRevision 只在成功 changed commit 推进 | Open | `begin_event()` 执行前全局 saturating increment，failure/no-op/input/presentation 同样推进。 |

### 5.3 P1

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| P1-01 RealtimeInput/Intent/Presentation/Audit/Committed 分型 | Partial | Host input outcome/mailbox 已独立；`EditorEvent`/record 仍混装全部语义。 |
| P1-02 qualified Action/Event/Operation/Transaction identity | Partial | EventId/Sequence 与 transaction/operation 字段存在；均未 owner/generation qualified，ActionInvocationId 缺失。 |
| P1-03 source/transport/executor/binding immutable provenance | Open | 单 source enum、Replay 映射 UiBinding，F5 仍改写 source/binding。 |
| P1-04 owner/document/world/generation/revision 传播 | Partial | transaction/save generation 局部进入 record；owner/document/world 与 scoped revision 缺失。 |
| P1-05 causal parent/request/selection/terminal disposition | Open | record 无这些字段。 |
| P1-06 effects/digest/dirty/history/save receipt | Partial | effects、revision、transaction/save 局部存在；digest、dirty/history generation 与原子 receipt 缺失。 |
| P1-07 schema/codec/redaction/retention class | Partial | serde 与推断 retention class 存在；无 versioned schema/codec/redaction policy。 |
| P1-08 EventRecord 与 CommittedOperationEntry 分型 | Open | committed type 不存在。 |
| P1-09 canonical serialization 禁止 adapter mutation | Open | F5 mutation 仍在。 |
| P1-10 F5 消费 callback receipt | Open | 仍消费 journal slice。 |
| P1-11 multi/zero/partial child receipt policy | Open | action identity 与 child receipt 均不存在。 |
| P1-12 retention 不改变 receipt digest | Open | receipt/digest 不存在。 |
| P1-13 各 generation overflow/stale policy | Open | event/sequence/revision/cursor 使用 saturation，transaction/save 没有统一 stale policy。 |
| P1-14 failure/no-op/input/presentation revision +0 | Open | begin_event 无条件 +1。 |
| P1-15 successful changed commit scoped revision +1 | Partial | changed outcome 可观察，但 revision 在执行前全局推进。 |
| P1-16 transient frame coalesce 与 edge order | Partial | idle mouse latest mailbox和边界前 flush存在；drag/resize/capture、多scope合同不完整。 |
| P1-17 realtime path 不访问 registry/transaction/journal/listener | Open | flushed move 仍进入完整 event path。 |
| P1-18 realtime state 按 viewport/document 隔离 | Partial | mailbox 位于单 event-loop 且 device 变化会 flush；event retention latest key 仍全局。 |
| P1-19 只在 semantic command 生成 execution receipt | Open | input/transient/presentation 同样返回 `EditorEventRecord`。 |
| P1-20 audit observation 显式预算 | Partial | count/bytes/age 三预算成立；deadline、schema/owner 与 observation policy 不完整。 |
| P1-21 replay 只接受 versioned committed entry | Open | raw record public replay。 |
| P1-22 replay target/schema/precondition/idempotency/effect preflight | Open | 均不存在。 |
| P1-23 replay precondition 在 apply 前拒绝 | Open | 无 precondition。 |
| P1-24 replay rollback/compensating/unknown 可审计 | Open | 只比较 error string。 |
| P1-25 replay 不递归生成可执行记录 | Open | replay dispatch 会正常 record，未分类阻断递归输入。 |
| P1-26 legacy ambiguity fail closed 零 mutation | Open | 任意 raw record 都会先执行。 |
| P1-27 replay final hash/revision/transaction/outcome | Open | replay 返回 `()`，无 final proof。 |
| P1-28 journal count/bytes/deadline/cursor page | Open | public journal 仍是完整 cloned snapshot。 |
| P1-29 listener owner/principal/capability/generation/affinity | Open | descriptor 只有 ID/name/enabled/filter。 |
| P1-30 page first/last/remaining/oldest/ack | Partial | status 另有 first/last/pending，page 有 next/has_more；未形成原子 page receipt。 |
| P1-31 unregister Drain/Reject/Discard receipt | Open | 只有立即删除与普通 success。 |
| P1-32 unregister 后旧 route 不投递 orphan | Open | registry test 明确允许旧 snapshot 在 unregister 后 enqueue。 |
| P1-33 enable/filter effective generation/cursor | Open | update 只返回 listener ID。 |
| P1-34 page count/bytes/deadline/gap/resync/final | Partial | count 1..256 与 has_more 成立；其余缺失。 |
| P1-35 ack 拒绝 foreign/stale/future 且幂等 | Partial | unknown listener 拒绝、重复 ack 返回 0；cursor 未 qualified，future 会清空已有 tail。 |
| P1-36 callback fault/slow/poison quarantine/dead-letter | Open | production callback consumer 不存在，mutex poison 被静默恢复。 |
| P1-37 retry 不重复 commit/不丢 tail | Open | callback/page lease/nack/retry contract 不存在。 |
| P1-38 registry generation fence/snapshot lifetime | Partial | immutable Arc snapshot lifetime 成立；无 generation fence，旧 snapshot 可 orphan enqueue。 |
| P1-39 production consumer register/page/ack/resync/revoke | Open | 精确调用只在 tests 和公开转发入口。 |
| P1-40 event 到 Message Bus 单一 ABI projection | Open | service 持 bus 但不发布明确 event ABI，也无版本化 projection boundary。 |
| P1-41 domain producer-consumer matrix | Partial | typed event variants与部分 transaction/save trace存在；没有显式 adoption/zero-consumer matrix。 |
| P1-42 zero listener/subscriber 不误称 Delivered | Open | event 无 terminal disposition；Editor169 已确认多领域 bus adapter false Delivered。 |
| P1-43 custom namespace/schema/capability/unknown policy | Open | operation/event string可进入记录，无 registry/policy。 |
| P1-44 ID/cursor exhaustion 显式拒绝 | Open | saturation 被生产代码和测试固化。 |
| P1-45 serialized event 禁止 local path/raw route/process ID | Open | binding/node/asset/graph locator 与 transaction u64 直接 serde。 |
| P1-46 command/journal/listener/F5 correlation identity | Partial | event/sequence/operation/transaction 字段局部贯通；ActionInvocationId 缺失且 F5 猜 slice。 |
| P1-47 transaction/save 与 receipt 原子关联 | Partial | dispatch 后查询 history/save generation；不是同一 commit receipt。 |
| P1-48 remote principal/capability preflight | Open | Headless/MCP 只有 source 枚举。 |
| P1-49 animation 使用真实 sequence/clip domain | Partial | `AnimationTrackPath`是真实 typed path；graph/state locator 仍是 string 且 event 无 clip/document identity。 |
| P1-50 F5 adapter 只投影 receipt | Open | adapter 仍读取、clone、改写 event record。 |
| P1-51 retention shrink/coalesce/concurrency/unrelated tests | Partial | bounded/coalesce/out-of-order/1K listener tests存在；F5 shrink/concurrency/unrelated归因缺失。 |
| P1-52 failure/no-op/input/replay side-effect tests | Partial | error、pointer burst/edge与普通 replay integration存在；failure revision、external replay safety缺失。 |
| P1-53 replay precondition/rollback/unknown/deterministic tests | Open | 无 committed replay contract。 |
| P1-54 pointer 125/500/1,000 Hz coalesce/edge tests | Partial | mailbox与edge focused tests存在；无指定频率动态 profile。 |
| P1-55 listener lifecycle/ack/gap/resync tests | Partial | register/filter/unregister/page/ack/status有覆盖；generation/gap/resync/fence缺失。 |
| P1-56 callback panic/slow/poison/tail tests | Open | callback consumer contract 不存在。 |
| P1-57 F5 receipt/provenance immutable integration | Open | 现有测试反而断言 provenance mutation。 |
| P1-58 1/5/100/10K listener 与 page memory/latency | Partial | 1,000/1,024 listener bounded/capacity evidence存在；目标矩阵、10K与page latency缺失。 |
| P1-59 schema/version/unknown/redaction/fuzz tests | Open | serde/filter tests不构成该资格。 |
| P1-60 删除 journal slice/source mutation/raw replay/revision-before-execute | Open | 四条坏路径全部仍在。 |

### 5.4 P2

| Finding | 状态 | 说明 |
|---|---|---|
| P2-01 durable audit segment/checkpoint/archive | Open | 当前仅内存 retention store。 |
| P2-02 remote transport/version negotiation/identity | Open | wire contract 未建立。 |
| P2-03 payload dedup/compression/zero-copy/content address | Open | Arc 只解决进程内 fanout。 |
| P2-04 listener QoS/priority/adaptive page | Open | 固定三 lane 和 256 count page。 |
| P2-05 health dashboard/lag heatmap/operator control | Open | 只有 DTO diagnostics，无产品 surface。 |
| P2-06 custom replay/resolution policy marketplace | Open | committed replay 基线未建立。 |
| P2-07 million-record query/retention tier/GC | Open | 无 durable index。 |
| P2-08 deterministic time-travel/event scrubber | Open | raw replay 不安全。 |
| P2-09 privacy/redaction per field/topic/consumer | Open | schema/redaction registry 不存在。 |
| P2-10 event inspector/filter/search/export | Open | listener control 不是产品 inspector。 |
| P2-11 adaptive input budget by frame health | Open | mailbox无frame-health policy。 |
| P2-12 plugin contract certification/revocation/unload | Open | listener没有plugin owner lease。 |
| P2-13 chaos publish/replay/listener/shutdown soak | Open | 无 qualification artifact。 |
| P2-14 cross-platform ordering/memory/latency benchmark | Open | 无同负载artifact。 |
| P2-15 unified provenance browser | Open | Snapshot/Collaboration/Gateway 尚未统一 identity。 |

## 6. Canonical 资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 F5 不读取 global journal length | Fail | 仍直接读取两次 snapshot。 |
| G02 ActionInvocationReceipt identity | Fail | 类型不存在。 |
| G03 provenance immutable | Fail | source/binding 仍被覆盖。 |
| G04 retention/coalesce receipt safety | Fail | slice 依赖长度，未覆盖收缩。 |
| G05 concurrency/multi-child 归因 | Fail | 无 causal/action boundary。 |
| G06 realtime path 不进完整 event pipeline | Fail | flushed/immediate move 仍进入。 |
| G07 frame-boundary coalesce | Partial | idle mouse mailbox存在，无完整input种类与frame policy。 |
| G08 press/release/cancel edge order | Partial | 非 move 前 flush结构和focused test存在，本轮未动态运行。 |
| G09 failure/no-op/input revision +0 | Fail | begin_event 无条件推进。 |
| G10 successful scoped commit revision +1 | Fail | revision全局且执行前分配。 |
| G11 committed-only replay input | Fail | raw record replay。 |
| G12 replay precondition/idempotency | Fail | 不存在。 |
| G13 external side-effect policy | Fail | 不存在。 |
| G14 replay rollback/unknown | Fail | 不存在。 |
| G15 deterministic final hash/outcome | Fail | replay返回 `()`。 |
| G16 listener identity/lease/capability | Fail | descriptor不足。 |
| G17 bounded delivery page | Partial | count page存在，bytes/deadline/remaining缺失。 |
| G18 ack/gap/resync | Partial | basic ack/lag diagnostics存在，非原子且无resync。 |
| G19 unregister/fence/disposition | Fail | old snapshot可orphan enqueue。 |
| G20 callback fault/quarantine/dead-letter | Fail | consumer boundary不存在。 |
| G21 Event/Message Bus ABI | Fail | 无版本化单一投影合同。 |
| G22 transaction/save atomic correlation | Partial | after-the-fact trace存在，非原子。 |
| G23 remote principal/capability preflight | Fail | 只有 source 枚举。 |
| G24 animation sequence/clip domain | Partial | typed track path局部存在，clip/document identity缺失。 |
| G25 F5 evidence consumes receipt only | Fail | 仍猜 journal delta。 |
| G26 retention/page budget与drop诊断 | Partial | count/bytes/age与drop/coalesce成立；page deadline/gap缺失。 |
| G27 exhaustion fail-close | Fail | saturation/replacement。 |
| G28 end-to-end correlation | Partial | event/sequence/operation/transaction局部存在。 |
| G29 explicit shutdown/revoke | Fail | event/listener lifecycle无Closing/Closed。 |
| G30 schema/version/unknown/redaction/fuzz | Fail | contract与tests均缺失。 |
| G31 125/500/1,000 Hz input | Partial |静态 mailbox 算法存在，无频率实测。 |
| G32 1/5/100/10K listener | Partial | 1K/1,024局部证据，无完整矩阵。 |
| G33 page memory/latency | Partial | hash/capacity benchmark局部存在，非产品page profile。 |
| G34 cross-platform metrics | Fail | 无 artifact。 |
| G35 race/fault/retry/soak | Fail | 关键生命周期测试缺失。 |
| G36 required correctness lane | Partial | focused静态/单元inventory较丰富，本轮未执行。 |
| G37 managed performance lane | Partial | ignored release benchmark存在，未形成资格矩阵。 |
| G38 F5 product evidence | Fail | evidence provenance仍被改写。 |
| G39 source currentness | Partial | 已冻结current disk/fingerprint；共享工作树继续变化，实施前须复算。 |
| G40 docs/index/link/fingerprint/static quality | Partial | 本轮文档静态校验可完成；实现与动态资格未完成。 |

## 7. 目标架构与 Hard Cutover

```text
Platform Input
  -> RealtimeInputAccumulator(scope, sequence, deadline)
  -> PresentationDelta / semantic Intent

Action Request
  -> Admission(principal, capability, binding, action id)
  -> Execution Preflight
  -> Mutation Transaction
  -> ExecutionReceipt(changed, effects, scoped revision, transaction, digest)
       -> CommittedOperationEntry -> replay/checkpoint
       -> AuditEnvelope -> journal/listener
       -> ActionInvocationReceipt -> F5/product evidence

Listener
  -> OwnerGeneration Lease
  -> Page(cursor, count, bytes, deadline, gap, remaining)
  -> Ack / Nack / Retry / Resync / DeadLetter
  -> Drain / Reject / Discard receipt -> Revoke
```

Hard cutover 要求：

1. `EditorEventRecord` 不再作为执行返回、audit、listener delivery 与 replay program 的共同公共类型。
2. 删除 F5 global journal length/slice 和 `normalize_cli_action_records()`；任何兼容 adapter 都不得修改 committed provenance。
3. `begin_event()` 只分配 arrival/audit identity；DocumentRevision 由 mutation transaction 在成功 changed commit 时分配一次。
4. pointer/resize/scrub/hover 等 realtime state 在 scope-local accumulator 内合并，只有 semantic intent/commit 进入 command 与 audit。
5. raw replay API 删除或变为只读 audit decode；可执行 replay 只接受 versioned committed entry 并先做完整 preflight。
6. listener ID 迁移为 owner-qualified generation lease；old route、page、ack、unregister 和 shutdown 全受同一 fence 约束。
7. journal/listener page 使用 count+bytes+deadline；gap/resync 与 page continuation 原子返回，禁止 full snapshot 作为消费协议。

## 8. 分层重构计划

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 | 五项 P0 封口 | F5 receipt/provenance、committed-only replay、realtime split、scoped revision 的 RED/Green/cleanup 全完成。 |
| M1 | 类型与 identity | Intent/Receipt/Audit/Committed 分型；Action/Event/Operation/Transaction/Document/World/Generation schema 冻结。 |
| M2 | Commit 与 replay | transaction 原子 receipt、digest/precondition/idempotency、rollback/unknown/final hash 完成。 |
| M3 | Listener lifecycle | owner lease、generation fence、page/ack/gap/resync、Drain/Reject/Discard、fault/dead-letter 完成。 |
| M4 | Owner 接入 | Editor02/08/14/48、Runtime Gateway 与 App/F5 producer-consumer matrix 全部接入并删除旁路。 |
| M5 | Storage/diagnostics/shutdown | bounded journal page、schema/redaction、correlation、Closing/Closed、operator diagnostics 完成。 |
| M6 | 资格与超越性基准 | correctness/race/fault、125/500/1K Hz、1/5/100/10K listener、soak、跨平台和同场景跨引擎 artifact 完成。 |

M0 必须先于 M1-M6；不能用扩大 retention、增加假 listener、把 snapshot 改成另一种 index guess 或仅隐藏 raw replay re-export 来伪装封口。

## 9. 逐 owner 检查台账

| Owner/文件簇 | 已检查的真实实现 | 仍需重构 |
|---|---|---|
| `types.rs` / dispatcher | event/source/envelope/effect/record 与 dispatch trait | 分型、qualified identity、schema、provenance、terminal disposition |
| service/state/stamp | sequence owner、journal/listener lock split、record fanout | checked allocation、commit-time scoped revision、atomic receipt |
| retention/journal | Arc payload、三lane预算、indexes、drop/coalesce diagnostics | scoped latest key、checked cursor、journal page/durability、避免full clone |
| listener/filter/route/registry/control | filter compile、immutable route snapshot、count page/basic ack/status | lease/generation/fence、page budgets、strict ack、unregister/fault/shutdown |
| replay | 公开 raw replay 与 error matching | committed-only preflight/rollback/idempotency/final proof |
| Host event dispatch/execution | typed per-domain execution、changed/effects、局部transaction/save trace | receipt owner、执行前后原子性、失败/no-op revision |
| retained pointer/event loop | idle move mailbox、input sequence/outcome、present batch、edge前flush | realtime boundary、multi-scope、immediate path、deadline与高频资格 |
| retained automation/App F5 | 真实callback route与产品evidence装配 | 删除journal slice/provenance mutation，消费typed action receipt |
| focused tests | retention/order/page/ack/listener/filter/error/pointer/1K局部证据 | P0 RED、race/fault/replay safety、10K、product E2E |
| Unreal/Bevy/Godot/Fyrox/Unity refs | lifecycle/transaction/cursor/bounds/command significance/provider scope | 只吸收边界；不把任一局部参考冒充完整可靠协议 |

## 10. 本轮 closeout 与限制

本轮只完成静态 review、参考源码对照、finding/gate 重判、refactor plan、selected-set fingerprint 与索引记录；没有修改 Editor、Runtime、App、Interface、Cargo 或 tests，也没有运行动态命令。共享 working tree 在冻结后仍可能继续变化，Editor170 只对 frontmatter 与 fingerprint 所列快照负责。

Editor49 只有在 5 个 P0 全部关闭、60 个 P1 逐项有实现与真实 production consumer 证据、40 门全部 Pass 后才可完成。后续实现必须从 M0 开始，首个补丁应同时删除 journal-delta receipt 与 provenance mutation，并建立失败/no-op/input revision +0 的 RED tests；否则继续扩展 listener/filter/replay 表面只会放大错误合同。
