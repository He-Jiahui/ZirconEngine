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
  - zircon_runtime/src/plugin
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

Open state: `待 Editor02/Plugins01 实现锁外分发、有界公平 pump 与 generation-safe sequence commit`。
