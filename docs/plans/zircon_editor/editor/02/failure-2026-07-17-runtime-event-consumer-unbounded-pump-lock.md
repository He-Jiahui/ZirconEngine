---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: runtime-event-consumer-unbounded-pump-lock
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/02
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/runtime_event_consumer/host.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_editor/src/core/gateway/session.rs
tests:
  - slow and reentrant typed-consumer deadlock regression
  - 1000/10000 delivery count/time-budget stress
  - multi-consumer fairness and session/order parity
---

# Editor02：runtime event consumer 无配额锁内 pump

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/core/runtime_event_consumer` 5/5 Rust 文件逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：Editor02 拥有主线程 pump/消息公平性契约；Plugins01/runtime event transport 作为共同实现方提供有界 drain。

## 失败现象与复现证据

`EditorRuntimeEventConsumerHost::pump` 持有整个 `active: Mutex<BTreeMap<...>>`，随后对每个 consumer 跨 gateway/ABI drain 全部 delivery、逐条验证 JSON/schema，并直接调用 typed/plugin consumer。慢回调或重入 reconcile/count/pump 会阻塞或自锁；单次 retained tick 没有 count/time budget，会把所有积压一次性堆到编辑器主线程。

## 最低共享层根因

consumer registry 的 generation/sequence state 与外部 drain、decode、callback 执行共享一个临界区，同时 transport API 只有全量 drain、没有可恢复的预算化 cursor。

## 架构修复验收

- 锁内仅快照稳定 consumer id/subscription/registration owner；gateway drain、decode 与用户回调在锁外执行。
- 每 tick 有全局与 per-consumer count/time budget，consumer 间 round-robin 公平；保序边沿不可静默丢失，可合并类别显式声明。
- sequence commit 使用 generation/session 条件写回，处理 pump 期间 remove/reconcile/end-play，不复活旧 consumer。
- 队列 depth/age/applied/deferred/dropped/slow-callback 指标可见；1k/10k storm 不突破 editor frame budget。

## 禁止临时方案

- 不得在持有 `active` 锁时调用 gateway 或 consumer callback。
- 不得只截断 drain Vec 而丢失剩余 delivery 的 ownership/sequence。
- 不得为避免死锁把 registry 改成无锁但失去 session/generation 一致性。

## 修复结果与回传

Open state: `Editor02 review clean / Cargo pending；Plugins01 bounded transport 依赖仍开放`。

- `pump` 已硬切为 active consumer 快照，gateway drain、decode 与 typed callback 均在 active 锁外执行；递归 pump 通过单 owner guard 返回空报告，不形成第二 delivery owner。
- 全量 transport drain 后的未消费 delivery 由匹配 generation 的 Editor02 pending queue 保序持有；全局 count、per-consumer count、elapsed time 与 slow-callback threshold 统一受 `EditorRuntimeEventPumpBudget` 管理。
- pump/lifecycle 共用原子 execution owner，关闭 check-then-act；并发/重入 lifecycle 返回 `LifecycleMutationBusy`。subscribe/unsubscribe 与 begin/end callback 仍在 active-map 锁外。
- gateway/validation/payload 错误先保留首个 typed error，但继续访问后序 consumer 并推进 cursor；坏 consumer 不再固定饿死其他 consumer。delivery JSON ownership 直接移入 callback，不再逐条深 clone。
- 报告字段已诚实改名为 pending-sequence-span；它不冒充 wall-clock age。1k/10k fake benchmark 仅能证明 Editor02 callback/pending count budget，不能证明生产 transport 帧预算。
- Plugins01 transport blocker 已单独写入 `docs/plans/zircon_plugins/01/failure-2026-07-22-plugin-event-drain-frame-budget.md`；在 count/bytes bounded drain 与真实 encode/decode 动态证据前，本 failure 不得 upward return。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与待办 |
|---|---|---|---|
| 2026-07-22 | `review_clean_static_green_validation_pending` | 首轮独立审查 0/3/3；Editor02 owner 的 atomic lifecycle gate、错误公平性、payload move、指标命名与计划状态已按 TDD 整改；第二轮发现 3/2 配额起点偏置并修复。 | 静态合同 6/6，最终独立复审 0/0/0。Plugins01 全量 drain 根因已转交对应 failure；受管 Cargo、bounded transport、failure return 与 managed commit 待完成。 |
| 2026-07-22 | `transport_evidence_review_clean_validation_pending` | snapshot819 后仅本文新增真实 transport/FakeGateway 证据；exact10 静态合同 `6/6 GREEN`，rustfmt 与 diff-check 已刷新并通过。 | 独立增量复审 `0/0/0`，确认 budget 只约束 apply、完整 Vec drain 与无 byte/deadline pending 仍存在。Plugins01 failure 保持 open；未运行 Cargo，不作 fixed return。 |

2026-07-22逐文件复核确认本failure必须保持open：`max_events/max_elapsed`只约束apply循环，`gateway.drain_plugin_events`仍先返回无上限完整Vec并全部append无界pending；每delivery还有take/commit两次active-map锁。Runtime10/Plugins01需把count+bytes+deadline推入typed producer/ABI，并返回remaining/oldest age；不得把Editor侧budget误写成transport有界。

2026-07-22 external tests补证：`bounded_pump_defers_backlog_without_losing_order`当前明确断言首泵`drained=10/applied=3/deferred=7`，ignored 10k benchmark也让FakeGateway先持有并一次返回完整delivery Vec。因此transport cursor修复必须同步把`drained`改为每tick count+bytes有界，并新增64MiB payload、producer>consumer 60s的pending bytes/oldest-age/RSS门；旧测试不能继续把全量ABI搬运当正确行为。

## 2026-07-30 Performance01 current-source supplement

Performance01按当前SHA重读`runtime_event_consumer/**` 6/6（1,180行）、16个外部tests和完整产品调用链。上面的2026-07-22“production仍全量无界drain”是历史证据，不再描述当前transport：Runtime/Plugins01已实现固定64 events/128KiB payload page、256KiB wire ceiling，以及每subscription 16K events/64MiB queue与typed overflow。Editor02不得继续以旧根因规划重复cursor实现。

Failure仍保持open，但最低当前根因已收窄为Editor host retention和稳态poll：每个visited consumer先drain一页再apply，pending VecDeque无entry/bytes/oldest-age上限，慢callback可逐tick把有界runtime queue迁入无界Editor RSS；pending非空也继续请求新页。每event仍分别锁active map做pop和sequence commit。active play空consumer也逐tick跨ABI执行空JSON batch encode/decode，而controller每tick全量capability snapshot/reconcile归PERF-MVP-565。

Editor02验收更新为：pending非空时不再drain，per-consumer pending至多一页或更严格entry+bytes+age预算；batch/per-consumer owner减少全局锁；保留generation/sequence/fairness。Plugins01/Runtime10负责empty page零编码、remaining/oldest-age和必要的新request-aware ABI。矩阵必须含callback 0/1/4/16ms、stall 60s、payload 128KiB、consumers 64，并记录runtime+editor queue bytes/age/RSS。当前6文件rustfmt GREEN；managed Cargo、ignored真实ABI benchmark与F4 WPR未运行，不能fixed return。
