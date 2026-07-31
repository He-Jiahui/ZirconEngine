---
owner_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
milestone: M2
slice: runtime-event-consumer-bounded-fair-pump
status: review_clean_static_green_validation_pending
related_code:
  - zircon_editor/src/core/runtime_event_consumer/error.rs
  - zircon_editor/src/core/runtime_event_consumer/host.rs
  - zircon_editor/src/core/runtime_event_consumer/pump.rs
tests:
  - tools/tests/test_editor02_runtime_event_consumer_bounded_pump_contract.py
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs
failure:
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-runtime-event-consumer-unbounded-pump-lock.md
---

# Editor02 runtime event consumer bounded fair pump

Plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
Milestone: M2
Status: review_clean_static_green_validation_pending
Files: ["docs/plans/zircon_editor/editor/02/failure-2026-07-17-runtime-event-consumer-unbounded-pump-lock.md", "docs/zircon_editor/core/runtime_event_consumer.md", "tools/tests/test_editor02_runtime_event_consumer_bounded_pump_contract.py", "zircon_editor/src/core/runtime_event_consumer/error.rs", "zircon_editor/src/core/runtime_event_consumer/host.rs", "zircon_editor/src/core/runtime_event_consumer/pump.rs", "zircon_editor/src/core/runtime_event_consumer/mod.rs", "zircon_editor/src/tests/mod.rs", "zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs"]

本切片修复 Editor02 runtime event consumer 主线程泵的最低共享层。active registry 只在快照、pending ownership 和 sequence 条件提交时短暂加锁；gateway、JSON decode 与 plugin callback 全部在锁外。transport 当前仍可一次返回完整 `Vec`，但未进入本 tick 预算的 delivery 不再丢失，而是由 Editor02 generation 绑定的 pending queue 保序延后。

## Scope delivered

- `EditorRuntimeEventPumpBudget` 统一控制全局 count、per-consumer count、elapsed time 和 slow callback threshold；默认 retained tick 不再无限处理 backlog。
- round-robin cursor 每 tick 推进 first consumer，小预算下不会固定饿死后序 consumer。
- `ActiveConsumerSnapshot` 固定 consumer id/registration/subscription/generation；sequence commit 必须仍匹配同一 generation，锁外阶段发生 remove/reconcile/end 不会复活旧 consumer。
- `EditorRuntimeEventPumpReport` 暴露 applied、drained、deferred、dropped、slow callbacks、queue depth 与 pending sequence span；失败 delivery 以 typed error + dropped counter 显式退出，其余 backlog 保留。
- 递归/并发 pump 由单 owner atomic guard 拒绝第二 owner；consumer callback 可重入 `active_consumer_count` 等只读观察而不死锁。
- pump 与 lifecycle mutation 通过单一原子 execution owner 互斥；重入或并发 reconcile/end 返回 typed `LifecycleMutationBusy`，关闭 check-then-act 竞态。subscribe/unsubscribe 与 begin/end callback 仍在 `active` map 临界区外。
- gateway、validation 与 payload 错误保留首个 typed error 后继续访问后序 consumer；每个实际执行 tick 都将 next first consumer 向后推进一位，非整除全局/per-consumer 配额下也不会固定偏置。
- delivery payload 直接移动进 typed callback；报告明确使用 pending sequence span，不再把序列范围命名为 wall-clock age。

## Fresh testing evidence

- TDD 第一轮静态合同初始为 `1 failed + 2 errors + 1 passed`，实现基础 pump 后为 `4/4 GREEN`；owner 自审发现 lifecycle reentry 边沿后追加合同先稳定 RED，再实现为 `5/5 GREEN`。
- 新 Rust 回归覆盖 10 条 backlog 按 3 条预算逐 tick 保序排空、两个 consumer 的 1/1 与非整除 3/2 round-robin、gateway-error 后序进展、callback 重入 host observation、并发生命周期 typed-busy 的 2 秒 deadlock guard及 slow callback report。
- ignored managed benchmark gate 以 64 events/tick 预算分别排空 1k/10k delivery，输出 ticks、max applied/tick、max pending sequence span、tick p95、applied/dropped/remaining depth 为 `EDITOR02_RUNTIME_EVENT_PUMP_BENCHMARK`；该 fake gate 不替代 Plugins01 真实 transport encode/decode 门。
- 精确 Rust 文件已通过 `rustfmt +1.94.1 --edition 2021`，scoped `git diff --check` 通过；受管 Cargo 尚未取得终态，因此不把源码存在写成动态 GREEN。

## Review

- 首轮独立复审为 Critical/Important/Minor=`0/3/3`。其中 Editor02 owner 的 2 个 Important 与 3 个 Minor 已进入 TDD 整改；真实 transport 全量 drain 的 1 个 Important 已写入 Plugins01 failure，未冒充本切片已解决。
- 第二轮复审发现非整除 3/2 配额下起点偏置，新增回归与 start rotation 后最终独立复审 Critical/Important/Minor=`0/0/0`。
- Plugins01/runtime transport 未被本会话修改；bounded drain API 必须由 Plugins01 owner 承接，不得让 Editor02 越权吸收 plugin transport。
- milestone commit 前必须取得独立 reviewer `Critical=0 / Important=0`，并完成 failure lifecycle return。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| Runtime event consumer bounded fair pump r3 | `review_clean_static_green_validation_pending` | 2026-07-22 | 首轮 review 0/3/3 后完成 atomic pump/lifecycle owner、并发 busy、error-path 后序公平、payload move 与 pending-sequence-span；复审再发现非整除 3/2 配额偏置，新增合同先 RED 后静态总门 `6/6 GREEN`，实现改为每个执行 tick 推进起始 consumer。最终独立复审 0/0/0；Plugins01 全量 transport drain 已按功能 owner 写入独立 failure。Cargo、failure return 与 managed commit 待完成。 |
