---
handoff_kind: failure
status: open
created_at: 2026-07-16
summary_slug: lifecycle-orphan-recovery-maintenance-hold-integrity-deadlock
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_workflow_node: M1.1
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/server.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/codex_sync/evidence.py
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/tests/test_supervision_service.py
tests:
  - python -m tools.session_coordinator --repo-root E:/Git/ZirconEngine serve
  - python -m unittest tools.session_coordinator.tests.test_supervision_service tools.session_coordinator.tests.test_supervision_actions
  - python -m unittest tools.session_coordinator.tests.test_supervision_actions.SupervisionActionTests.test_production_lifecycle_rejects_global_shutdown_without_draining
  - python -m unittest tools.session_coordinator.tests.test_cargo_jobs.CargoJobTests.test_reconcile_reports_a_stale_live_job_without_freezing_other_lanes
---

# Coordinator01: lifecycle orphan recovery is blocked by its own maintenance hold

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：Editor05 M1.1 selection and mode-stack foundation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Editor05 已产生可编译的当前源码与聚焦行为证据，但 fresh managed validation、Editor03 M3.2 gate refresh 和 Session 租约续期都依赖 Coordinator 服务；最低共享失败位于 Coordinator 启动期 lifecycle recovery。

## 产出记录与时间

| 状态 | 日期 | 证据 |
| --- | --- | --- |
| `OPEN / JOB CLOSEOUT EXIT + PARTIAL SESSION REGISTER` | 2026-07-16 | schema 37 instance `1957c3ea1a674429998c6be8816129f5` 在 Runtime11 managed job `617c5beba0584fff92db37fc00e579c9` 完成后进入 `busy=false / draining / maintenanceHold=true`，随后 runtime descriptor 消失且 daemon 自行退出。官方 `zircon-session start` 拉起 successor `57ea3c36516241f49ef206bb8c775bc0`，以最近成功 drain `aabe4fa7f47944049aa04c51dbccea6f` 为 proof 的正式 resume `b004932061964be7a71953c3c506adfe` 才恢复 healthy。恢复后，对 stale Editor05 与 Coordinator handoff Session 的 `session.register` 返回 `invalid_status_transition: stale -> resolving_failure`，但响应后的 Session 已部分写入新 display name / plan / write scope；后续显式 `session.set_status active` 才完成恢复。验收需覆盖 managed-job closeout 不退出 daemon，以及 register 状态迁移失败时全事务回滚。 |
| `OPEN / RESUME LEAVES EXPLICIT STOP ACTIVE` | 2026-07-16 | scoped maintenance daemon `5fd6e6e03b0e45b5a63713d47658f8a6` 继承成功 stop `7a54d3026dbe451d87ebd737c4eaa98f` 后，正式 `service.resume` action `4c4f8b9d4c2a4890b2a61b0ddb5b46ff` 返回 `succeeded / state=healthy` 并清除 `maintenanceHold`，但 health 仍为 `explicitStop=true`。随后 Editor05 受管验证在 `session.register@editor05-m1-selection-mode-stack-20260716` 被 `service_explicit_stop_active` 拒绝。该现象证明恢复结果对 mutation admission 作了错误成功声明；未直接修改 recovery state 或绕过 scoped daemon。 |
| `RECOVERED BY TRAY RETRY / ROOT CAUSE STILL OPEN` | 2026-07-16 | 后续 tray retry 拉起 schema 36 instance `01733202997940f3ae958c186268d91a`，supervision 回到 healthy/read-write，但仍报告 `failureCount=1`，且没有新增 orphan-recovery 回归测试或代码修复。该运行态恢复允许其他受管事务继续，不构成 failure return。 |
| `OPEN / STARTUP INTEGRITY DEADLOCK` | 2026-07-16 | 官方 drain 完成并停止 schema 36 实例后，自动恢复与两次 `zircon-session serve` 均未形成在线实例。`.codex/state/session-coordinator/startup-failure.json` 记录 `IntegrityError / migration_or_integrity_failure`；直接运行官方 Python 入口稳定复现下述 traceback。未直接修改 SQLite、未删除 runtime descriptor、未 bypass maintenance hold。 |
| `OPEN / BOUNDED DRAIN BECAME PERSISTENT` | 2026-07-16 | schema 42 instance `2514d14fc3ef4073b478fe782461f2ac` 的 read-only audit 证明普通 `service.drain` 会写 `maintenance_hold=1`，但其 `timeoutSeconds` 仅持久化到 lifecycle intent，未由任何 worker 或 startup recovery 消费；因此一个 30 秒 admission drain 可无限停留在 `draining`。同一 hold 的 release mutation 又被 `service.resume.release@<maintenance-session>` gate 绑定到会过期的 Session heartbeat，形成“Session stale 后合法 release 不可调用”的第二条闭环。当前实例已通过受控 resume 回到 `healthy`；两条外部 CPU job 未被终止。 |
| `IN PROGRESS / GLOBAL DRAIN DISABLED` | 2026-07-16 | 用户要求所有任务始终准入后，生产 `LifecycleService` 禁用 `service.stop`、`service.restart` 与 `service.force_stop`，三者在创建 intent 前即返回 `lifecycle_global_shutdown_disabled`，不会再写入 `draining`、maintenance hold 或 explicit stop。普通 `service.drain` 仅生成 blocker 审计。任务超过 300 秒没有 heartbeat 时只写 `cargo.health_timeout`，实时证据投影仅显示尚未恢复心跳的 active job。 |

## 失败现象与复现证据

```text
RunningCoordinator.start
  -> lifecycle.recover_restart_intents
  -> supervision.fail_lifecycle
  -> supervision._transition
  -> UPDATE service_recovery_state
sqlite3.IntegrityError: maintenance hold prevents mutation window
```

复现命令：

```powershell
python -m tools.session_coordinator --repo-root E:/Git/ZirconEngine serve
```

服务在 HTTP 监听和正常 action 处理前终止，`zircon-session status` 只返回 `offline`。失败同时阻断：

- Editor03 63 路径 M3.2 validation 的终态 gate refresh 与后续 review/commit；
- Editor05 当前源码的 fresh managed compile/test；
- Session register、lease heartbeat/current-hash attribution 和 Failure import。

## 最低共享层根因

启动恢复必须把上一实例遗留的 executing lifecycle action 标为
`lifecycle_orphan_recovered`。当前 `recover_restart_intents()` 调用正常
`fail_lifecycle()` 写路径，而 `_transition()` 的数据库完整性约束在
`maintenance_hold` 存在时禁止同一写入。于是服务必须先恢复孤儿 action
才能启动，但恢复写入又要求 maintenance hold 已解除，形成启动闭环。

该问题不同于现有 Cargo PID reuse、milestone manifest 选择和 closeout
checker failure；它发生在服务 bootstrap 的 lifecycle recovery 阶段。

替换 daemon 后还暴露出同一恢复状态机的第二个闭环：成功 stop 将
`explicit_stop=1` 持久化；`LifecycleService._activate_intent(RESUME)` 只在
`releaseMaintenanceHold` 时写 `maintenance_hold=0`，却不收束
`explicit_stop`。因此 action 和 health 可以报告 `healthy`，而
`SupervisionService.require_mutation_allowed()` 仍拒绝全部非维护 Session
mutation。scoped maintenance daemon 也不应接受一个无法真正开放 mutation
window 的 unscoped resume 并返回成功。

普通 admission drain 的状态机也把短时排空与持久维护混为一谈：DRAIN
分支将 `maintenance_hold=1` 写入 durable recovery state，却没有对其
`deadline_at` 安排完成路径。restart 只恢复 stop/restart/force-stop，遗漏
仍活跃的 DRAIN intent；因此服务重启也无法纠正这个泄漏。release gate
额外要求一个 maintenance Session 后缀，但该 Session 可在 hold 存在期间
自然变 stale，导致持有正确 drain action ID 的操作者也无法执行 release。

## 架构修复验收

- 为 daemon bootstrap 定义单一、类型化的 orphan-lifecycle recovery 事务；只允许把上一实例的 executing lifecycle action 终结为明确失败，并同步收束对应 recovery state。
- 该恢复事务必须在 maintenance hold 下合法执行，但不得开放普通 Session、lease、validation、commit 或任意控制 action mutation window。
- `recover_restart_intents()` 可重复执行且幂等；进程在事务中断后再次启动不会重复创建 action、破坏 state fingerprint 或留下 executing action。
- schema 36 现有数据库复放后服务成功启动，`status` 返回新 instance id 与 healthy/read-write 状态；不要求手工 SQL、删除数据库或清空审计历史。
- 明确区分 scoped maintenance resume 与显式 start/resume：前者不得清除用户的 explicit stop，也不得返回“mutation window 已恢复”；后者在权限、实例身份和 stop action 证明匹配后，必须原子清除 `explicit_stop`，且响应状态与 `require_mutation_allowed()` 一致。
- 增加回归覆盖：maintenance hold + orphan executing restart、无 orphan 的普通启动、重复 recovery、事务失败重启、不同 repository identity 拒绝路径。
- 增加回归覆盖：successful stop -> replacement daemon -> explicit resume -> `session.register` accepted；scoped daemon 尝试 unscoped resume 必须拒绝且不得产生成功 action；`healthy + explicitStop=true` 不得被投影成 read-write admission。
- 增加回归覆盖：managed Cargo job 完成后 daemon 不丢 runtime descriptor、不自行退出；stale Session register 若无法进入 `resolving_failure`，则 display/plan/write-scope/status 必须全回滚，禁止错误响应伴随部分写入。
- 修复后重放 Editor03 M3.2 gate refresh 与 Editor05 managed validation，证明不是只让空数据库启动。
- 普通 `service.drain` 不得建立 `maintenance_hold` 或切换为 `draining`；它
  只记录当前 blockers，所有任务继续准入。单个 Cargo 的 timeout、orphan
  reconciliation 与 cleanup 必须独立处理，不能通过全局 admission gate 加速。
  daemon replacement 必须关闭历史遗留的 DRAIN intent，避免旧记录重建永久
  draining。
- live Cargo 连续 300 秒没有 heartbeat 时必须记录一次带 PID tree 的
  `cargo.health_timeout` 审计事件；它不得杀死或复用仍活跃的进程，但也不得
  关闭其他 lane 或其他 Session 的普通准入。
- 生产实例不得通过 stop/restart/force-stop 建立 persistent maintenance hold；
  这三种全局操作必须在 intent 创建前被拒绝。历史 hold 的恢复仍须可审计，
  但不能重新关闭普通任务准入。

## 禁止临时方案

- 禁止直接修改/删除 `coordinator.sqlite3`、runtime descriptor、recovery state 或 action 审计记录。
- 禁止启动时全局关闭 maintenance-hold trigger、跳过 integrity check、把所有 executing action 静默标成功或开放普通 mutation。
- 禁止用 Editor03/Editor05 本地 Cargo、手工 staging 或 direct SQLite 查询代替协调器恢复。

## 修复结果与回传

Open state: `服务已由 tray retry 恢复，但待 Coordinator01 修复并以现有 schema 36 数据库复放回传`; no pass is claimed.
