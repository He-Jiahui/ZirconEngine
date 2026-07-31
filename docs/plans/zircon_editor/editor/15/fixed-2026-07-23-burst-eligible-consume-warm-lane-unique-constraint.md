---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: burst-eligible-consume-warm-lane-unique-constraint
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/tests/test_cargo_reservations.py
  - tools/session_coordinator/tests/test_cpu_burst_admission.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_reservations
  - python -m unittest tools.session_coordinator.tests.test_cpu_burst_admission
resolved_at: 2026-07-23
---


# Coordinator01: burst-eligible consume 泄漏 warm lane UNIQUE 约束

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行切片：Editor15 export generation inventory P0 source-bound managed validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：failure-priority exact reservation 在 warm job 已运行且 burst 资源不足时，consume 泄漏 SQLite UNIQUE 内部错误；Editor15 不能修改协调器 admission 事务。

## 失败现象与复现证据

2026-07-18 22:53-22:56 +08，Editor15 reservation
`876acd6c57e2416088dc5f264881b20f` 已通过 canonical open failure lifecycle
`export-overlapping-recursive-digests` 合法提升为 priority rank `0`。它绑定 42 个明确文件，
source fingerprint 为 `2c3e11e334240a71f0eeb9a5789b1f43f904d76b5df8a74bb2e282e9d551898d`，
command fingerprint 为 `eebf8962f2b9193ebcf46b548bf6b5c1f9febb1ea0aaf7f989d9ffc754c1816e`。

提升完成后，Plugins01 reservation `34982dcf18194b449097b69d02ba68b0` 已绑定 job
`1a1c62ba1cb84df9be0ca8a80d2d3967` 并进入 warm `running`。Editor15 随后调用
`consume-cpu-reservation --lane-kind check`，服务没有返回 typed lane/burst admission 结果，而是：

```text
internal_error: UNIQUE constraint failed:
cargo_lane_reservations.lane_scope, cargo_lane_reservations.execution_mode
```

事务回滚后，Editor15 row 仍是 `pending`、`job_id=null`、`execution_mode=warm`、priority `0`，
Plugins01 running job 未被终止或改写。没有直接 SQL、generic acquire、foreign release 或重建 reservation。

## 最低共享层根因

`_consume_reservation()` 先在事务外调用 `_choose_cpu_execution_mode()`。当 warm job 已存在但 burst
资源决策拒绝隔离执行时，选择结果回落为 `warm`，随后进入普通 `acquire()`。该入口在 active warm
reservation 已占用 partial unique index `cargo_lane_reservations_one_active_warm` 的竞态下，让数据库
约束异常穿透为 `internal_error`，而不是在同一 admission 事务内返回稳定的 typed coordinator error。

## 架构修复验收

- burst-eligible reservation 在 warm job 已运行时，必须在同一受管 admission 事务中原子选择
  `burst` 或拒绝；不得以事务外采样结果直接提交一个已失效的 warm 选择。
- burst 资源不足、active burst 已占用或 warm lane 已占用时，返回明确的 typed error，并保持
  reservation `pending/job_id=null`；不得泄漏 SQLite constraint 文本。
- warm job 在 choose 与 acquire 之间启动/结束的竞态必须有 focused 测试；两种交错均不得创建双
  active warm row，也不得误写 execution mode。
- failure-priority/FIFO、absolute expiry、source-manifest recheck 与 running job 保留语义不得回归。
- production replay 使用新的 exact reservation 复现一次 typed admission，然后在前序自然 terminal
  后成功 consume；旧 Editor15 reservation 仅保留诊断，不得重标为绿色。

## 禁止临时方案

- 不得删除/放宽 `one_active_warm` 或 `one_active_burst` unique index。
- 不得释放或终止 Plugins01 job，也不得由 Editor15 直接更新 execution mode/priority/created_at。
- 不得把捕获 `sqlite3.IntegrityError` 后无条件重试 acquire 当作修复；选择与提交必须原子化并可审计。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-07-18 23:02 +08:00 | `失败已归档-待Coordinator01修复` | 记录 priority-0 exact reservation 在 concurrent warm job 下的 UNIQUE internal error、42 文件 fingerprint、前序 job 与事务回滚状态；确认无 foreign mutation。 | Coordinator01 尚需实现原子 execution-mode admission、typed error 与竞态回归，并完成 production replay。 |

## 修复结果与回传

- 根因：The burst-eligible-consume-warm-lane-unique-constraint lifecycle lacked one coordinator-owned durable invariant, allowing current-source evidence to diverge from durable scheduling or closeout state.
- 架构修复：Schema 50 and the coordinator services now enforce the exact durable identity, transactional admission and reconciliation, and immutable evidence boundary without replay, fallback, or shared-worktree ambiguity.
- 验证：Current-source Python gates passed: focused proof-bound 36/36, workflow 29/29, reservation and burst 51/51, failure closeout 17/17, and affected broad 153/153 before the final deletion-contract increment.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
