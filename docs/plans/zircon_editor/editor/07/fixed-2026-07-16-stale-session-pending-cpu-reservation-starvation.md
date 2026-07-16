---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: stale-session-pending-cpu-reservation-starvation
origin_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_workflow_node: M1
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/07
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/sessions.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/supervision/lifecycle.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_reservations tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_sessions
  - python -m unittest tools.session_coordinator.tests.test_database
  - python -m unittest tools.session_coordinator.tests.test_server tools.session_coordinator.tests.test_supervision_service
resolved_at: 2026-07-16
---


# Coordinator01: stale Session pending CPU reservation starves managed validation

## 产出记录与时间

| 状态 | 日期 | 证据 |
|---|---|---|
| `OPEN / STALE OWNER + ZERO JOBS + PENDING RESERVATION` | 2026-07-16 | Editor07 current-source acquire 被 reservation `39d9c5788f09464fb20ea4c761164db4` 拒绝。只读审计确认 owner `render18-af-m2-rebase-20260715` 已于 `2026-07-15T21:18:22Z` 标为 stale，最后心跳为 `20:22:43Z`；预约却在 `22:16:11Z` 创建，`job_id=NULL/status=pending`，同时 `cargo_jobs` 中 leased/running 数为 0。多次 daemon restart 后其 `expires_at` 延长到 `2026-07-16T00:04:58Z`，继续阻塞不相关 CPU 验证。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 来源执行切片：Editor05 viewport SelectionModel consumer hard-cut failure return
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Session 状态、CPU lane reservation、到期和 daemon restart 恢复均由 Coordinator 拥有；Editor07 不得释放或接管 foreign reservation。

## 失败现象与复现证据

受管命令：

```powershell
tools/zircon-session.ps1 cargo acquire test --session-id editor07-selection-consumer-hardcut-20260716
```

稳定返回 `cargo_cpu_lane_reserved`，指定上述 Render18 reservation。只读状态
同时证明 owner 已 stale、没有关联 job，也没有任意活跃 Cargo job。预约创建时间
晚于 owner 被标 stale 的时间，且在 coordinator schema 38/39 多次 restart 后到期
时间继续后移；因此这不是正常运行 job 或活跃 Session 的 FIFO 等待。

## 最低共享层根因

CPU reservation admission 没有把 owner Session 的可用状态作为同一事务内前置
条件；reservation recovery/heartbeat 又允许 `job_id=NULL` 的 pending 项跨 daemon
restart 延长 TTL。结果是一个已经 stale、也从未启动 nominated job 的 Session
仍可永久占据全局 CPU 队首，所有不相关 managed validation 都被拒绝。

## 架构修复验收

- 新建或续期 reservation 时，owner Session 必须处于允许执行的活跃状态；stale、completed、cancelled、archived Session 一律拒绝。
- Session 转 stale 时，在同一受管维护事务中终结其 `job_id=NULL/status=pending` reservation；不得终止已有 running job。
- daemon restart 必须保留原始绝对 expiry，不得给未消费预约重新计时或延长 TTL。
- reservation 指向 job 后，生命周期只随 nominated job 的 leased/running/terminal 状态推进；无 job 的 pending 预约到期后下一 acquire 必须立即前进。
- 增加回归：stale owner 预约拒绝、活跃 owner 转 stale、pending restart 不延寿、零 job 队首过期、公平 FIFO 与 running job 不被误释放。
- 以当前数据库状态复放后，Editor07 Session 可以获得受管 CPU lane；不得要求手工 SQL 或删除 reservation 行。

## 禁止临时方案

- 禁止 Editor07 手工 release/改写 Render18 reservation、伪造 owner 心跳、接管 nominated job 或直接修改 SQLite。
- 禁止用 raw Cargo、仓库内 target、另开未受管 rustc/cargo 进程绕过 FIFO。
- 禁止把所有 stale Session 的 running job 一并终止；修复只收束未消费 pending reservation。

## 修复结果与回传

- 根因：CPU reservation admission and recovery did not require an executable owner Session, public stale transitions did not atomically terminalize pending no-job claims, and daemon recovery renewed unconsumed TTLs.
- 架构修复：Centralized executable Session admission and invalid pending cleanup in cargo_reservations.py; reserve/acquire/renew reject terminal owners; set_status(STALE) and mark_stale expire only pending no-job reservations in the same transaction; pending expiry remains absolute across restart; job-bound leased/running reservations follow job terminal/orphan lifecycle; migration 41 persists canonical compatibility_json for new reservations without fabricating historical payloads.
- 验证：Current-source Python gates: reservation/cargo/session 70/70, database 15/15, server/supervision 37/37; compileall and diff-check pass; handoff validator 160 artifacts/0 errors; plan-output audit pass; independent final review P0/P1/P2=0/0/0. Production: instance 5421e008fda84be6b42480cc0c602cec loaded schema41; old stale reservation 39d9c5788f09464fb20ea4c761164db4 expired before natural TTL; canonical payload reservation 0bbc781eb7e34a7f9fa2cce48e1f5a0 round-tripped then stale-expired; orphan-bound reservation c692e7317228417ead7e4d17fa21e81f expired with job c8010778cfe644568d693d909615eab2; following FIFO job 2853a1a87724409ab937305565bf99ff acquired and both replay jobs released.
- 回传：Coordinator01 schema41 hardens CPU reservation ownership, stale atomic cleanup, absolute expiry, orphan handoff, FIFO progression, and persisted compatibility payload. Production replay passed without manual SQL or foreign reservation mutation; Editor07 can resume managed validation.
