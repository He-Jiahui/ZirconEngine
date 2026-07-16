---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: terminal-cpu-reservation-owner-release-starvation
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_workflow_node: M3
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_runner.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_reservations tools.session_coordinator.tests.test_cargo_jobs
resolved_at: 2026-07-16
---


# Coordinator01: terminal CPU reservation starves the next managed validation

## 产出记录与时间

| 状态 | 日期 | 完成项目与证据 |
|---|---|---|
| `FIXED / 待受管提交与重载` | 2026-07-16 | job release 同事务释放 bound CPU reservation；历史 `finished` 队首仅在 job 已 `released`、进程树为空且 owner 非可执行时收束。聚焦 Python 回归 `60/60` 通过；canonical P0 Session 已绑定 failure、fixed return、Coordinator01 回传摘要、生产代码、测试与模块文档。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：M3 Text current-source focused 与 graphics-only upward gates
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：CPU reservation、managed runner finish/release 与 stale Session 收束均由 Coordinator01 持有；Frameworks05 不得释放 Runtime11 的 foreign reservation。

## 失败现象与复现证据

Runtime11 预约 `b3c80153012e4b28a145580a7b4fa9a8` 绑定 managed job
`974bc6ebbc95463c82313334d6a24fea`。该 job 于 2026-07-16 08:27 +08:00
自然完成并由 runner 自动 release，结果为 exit 0、24 passed / 0 failed，
`live_process_pids=[]`。owner Session `runtime11-managed-validation-20260716`
最后心跳停在 08:18:54。

Frameworks05 在 job release 后多次执行正式 managed acquire，仍稳定返回
`cargo_cpu_lane_reserved` 并指向同一 Runtime11 reservation。当前实现审计显示
job finish 只把 reservation 改为 `finished`，job release 不推进 reservation；
stale/expiry 收束只处理 `pending + job_id IS NULL`，因此 owner 不再运行
`release-cpu-reservation` 时，全局 CPU FIFO 没有自然前进条件。

## 最低共享层根因

CPU reservation 与 nominated job 的终态不是同一生命周期：runner 已经完成
`finish -> release`，reservation 却仍作为 active `finished` 行参与队首查询。
显式 owner release 是正常的快速路径，但被当成唯一终结路径；Session 丢失时，
已无 live process 的 terminal job reservation 会永久占据全局 CPU 队首。

## 架构修复验收

- nominated job 进入 terminal 且 process tree 为空后，reservation 必须在同一受管生命周期内进入不阻塞队首的 terminal/released 状态；不得依赖客户端后续存活。
- owner 显式 `release-cpu-reservation` 继续保持幂等，但只作为确认/快速路径，不是 terminal job 释放全局 FIFO 的唯一条件。
- stale Session 清理必须区分 running job 与 terminal job：不得终止或接管 running job；可安全收束已 finished/released 且无 live process 的 reservation。
- 新增回归覆盖 runner 自动 finish/release、客户端在 job 启动后退出、owner 变 stale、重复 release，以及下一 Session acquire 立即前进。
- 以 reservation `b3c801...` / job `974bc6...` 的同构状态复放后，Frameworks05 可获得 managed CPU lane；不得要求手工 SQL、foreign owner impersonation 或 raw Cargo。

## 禁止临时方案

- 禁止 Frameworks05 手工释放、续期、改写或冒用 Runtime11 reservation。
- 禁止把 Runtime11 已通过的 job 改写为 orphan/failed，或终止任何仍在运行的 Cargo job。
- 禁止 raw Cargo、额外 target、数据库直改、hook bypass 或绕过 FIFO。

## 修复结果与回传

- 根因：A released CPU job could leave its reservation in finished state when the owner Session became stale, so the FIFO head remained permanently blocking.
- 架构修复：Cargo job release now releases its bound reservation transactionally, and maintenance reconciliation releases only legacy finished CPU reservations whose job is released, process tree is empty, and owner is non-executable.
- 验证：Focused reservation lifecycle suite passed 15/15, Cargo job suite passed 45/45, and scoped maintenance regressions passed for failure-return lease enforcement.
- 回传：The stale Plugins06 FIFO head is non-blocking; a new Render18 reservation may be issued only after this P0 closeout is committed and reloaded.
