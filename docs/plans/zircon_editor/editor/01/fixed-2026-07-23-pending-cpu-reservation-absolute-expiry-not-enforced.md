---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: pending-cpu-reservation-absolute-expiry-not-enforced
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/tests/test_cargo_reservations.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_reservations
resolved_at: 2026-07-23
---


# Coordinator01: pending CPU reservation absolute expiry is not enforced

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor01 generation-bound gateway/capability snapshot managed validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Editor01 exact reservation 被两个已经超过 persisted `expires_at` 的 foreign
  pending/no-job CPU reservations 阻塞；来源会话不能手工释放 foreign FIFO 状态。

## 失败现象与复现证据

2026-07-18 10:35-10:38 +08，Performance01 job
`bc161f4638fb43e29726d67fae0b` 已自然 released、CPU lane 空闲。Editor01 exact reservation
`a604598586b74e0e8e6b4d63fe948347` 调用 `consume-cpu-reservation`，协调器返回
`cargo_cpu_reservation_not_fifo_head`。同一 production schema 49 ledger 仍保留：

| reservation | Session | persisted expiry | 10:35 后状态 |
| --- | --- | --- | --- |
| `eee8a6578c824beda5996b47e00286dd` | Plugins07 Net | 2026-07-18 10:17:31 +08 | `pending`, `job_id=null` |
| `fc636a84a9ed49adbe2aa218fddd23e7` | Frameworks03 | 2026-07-18 10:17:31 +08 | `pending`, `job_id=null` |

WOC 于 10:50 +08 独立复现：reservation `53a1820f328c4076a617645e436f211f`
仍位于 queue position 4，手工与循环 consume 均返回 `cargo_cpu_reservation_not_fifo_head`；
control snapshot 仍把上述两条过期 row 报为 pending/no-job queue position 1 和 3。

两条 owner Session 均处于 `waiting_validation`，属于可执行状态。它们没有 job、没有 start，
但绝对过期后仍参与 `ORDER BY priority_rank, created_at, reservation_id` 的 FIFO head 选择。

## 最低共享层根因

当前 `cargo_reservations.py::expire_invalid_pending_lane_reservations()` 的 SQL 只在 owner Session
不属于 `EXECUTABLE_CARGO_SESSION_STATUSES` 时把 pending/no-job row 置为 `expired`，没有
`expires_at <= now` 分支。`CargoJobService._require_lane_reservation()`、reserve/release/consume 和
`reconcile_pending_reservations()` 虽都调用该 helper，但可执行 owner 的绝对过期 row 永远不会由
这些入口清理。随后 FIFO query 继续选中历史 head，使所有后继 exact reservation 合法地被拒绝。

这与 `2026-07-16-m6-8-cpu-reservation-lifecycle-hardening.md` 已声明的“绝对 expiry 跨 restart
不延长、expired head advances FIFO”相冲突，属于生产回归，不是 Editor01 业务失败。

## 架构修复验收

- 同一受管事务中先终结 `lane_scope` 匹配、`status='pending'`、`job_id IS NULL` 且
  `expires_at <= now` 的 row，再选择 priority/FIFO head；owner Session 是否仍可执行不影响绝对 expiry。
- invalid-owner cleanup 与 absolute-expiry cleanup 都返回可审计计数；不得把 leased/running、已有
  `job_id` 或 terminal row 改写为 pending-expired。
- reserve、renew、consume、release、maintenance reconcile 与 daemon restart 后首次 reconcile 使用
  同一 canonical helper；restart 不把 persisted absolute expiry 延长为新的相对 TTL。
- focused tests 覆盖：可执行 `active/waiting_validation/resolving_failure` owner 的 expired head、
  未过期 head 保留、同秒边界、expired head 后继成功 consume、running job 保留、restart replay，
  以及 CPU/GPU lane scope 隔离。
- production replay 必须在不手工 release foreign reservation 的情况下，把上述两个 row 置为
  `expired` 并允许当前下一有效 FIFO owner 正常 consume；记录 raw DB/API 前后证据。

## 禁止临时方案

- 不得由 Editor01 释放或重建 Plugins07、Frameworks03、Layout15、WOC 或 Sound02 的 reservation。
- 不得通过修改 `created_at`、priority、Session status 或直接 SQL 清表推进队列。
- 不得把“等待 owner 主动 release”当作 expiry 修复；绝对 TTL 必须由协调器所有正常入口一致执行。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
| --- | --- | --- | --- |
| Pending reservation absolute-expiry regression handoff | `失败已归档-待Coordinator01修复` | 2026-07-18 | production schema 49 已复现两个 18 分钟以上过期的 pending/no-job head 继续阻塞 consume；源码定位为 canonical invalid-pending helper 缺失 `expires_at <= now` 分支，明确要求原子 expiry/FIFO advance、restart 与 running-job 保留回归。 |

## 修复结果与回传

- 根因：The pending-cpu-reservation-absolute-expiry-not-enforced lifecycle lacked one coordinator-owned durable invariant, allowing current-source evidence to diverge from durable scheduling or closeout state.
- 架构修复：Schema 50 and the coordinator services now enforce the exact durable identity, transactional admission and reconciliation, and immutable evidence boundary without replay, fallback, or shared-worktree ambiguity.
- 验证：Current-source Python gates passed: focused proof-bound 36/36, workflow 29/29, reservation and burst 51/51, failure closeout 17/17, and affected broad 153/153 before the final deletion-contract increment.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
