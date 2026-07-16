# M6.8 Managed CPU Reservation Lifecycle Hardening

Plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
Milestone: M6.8
Status: accepted
Files: ["docs/cli-and-tooling/local-session-coordinator.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-16-m6-8-cpu-reservation-lifecycle-hardening.md", "docs/plans/zircon_editor/editor/07/fixed-2026-07-16-stale-session-pending-cpu-reservation-starvation.md", "docs/tools/session_coordinator/workflows.md", "tools/session_coordinator/cargo_jobs.py", "tools/session_coordinator/cargo_reservations.py", "tools/session_coordinator/failures.py", "tools/session_coordinator/git_finalize.py", "tools/session_coordinator/migrations.py", "tools/session_coordinator/sessions.py", "tools/session_coordinator/workflows/milestones.py", "tools/session_coordinator/tests/test_cargo_jobs.py", "tools/session_coordinator/tests/test_cargo_reservations.py", "tools/session_coordinator/tests/test_database.py", "tools/session_coordinator/tests/test_failures.py", "tools/session_coordinator/tests/test_milestone_failure_scope.py", "tools/session_coordinator/tests/test_sessions.py", "tools/session_coordinator/tests/test_workflow_commit.py"]

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与验收证据 |
|---|---|---|
| `ACCEPTED / EXACT COMMIT PENDING` | 2026-07-16 | 将 CPU reservation 状态策略抽取到 `cargo_reservations.py`；reserve/acquire/renew 仅接受可执行 Session；公开 `set_status(STALE)` 与 maintenance `mark_stale` 在同一事务中只终结 pending/no-job reservation；绝对 expiry 跨 restart 不延长；job-bound leased/running 不被 pending TTL 误伤，orphan 终态在同事务内释放 FIFO；schema 41 为新 reservation 持久化 canonical `compatibility_json`，不伪造历史 payload。 |
| `FRESH PYTHON GATES 122/122` | 2026-07-16 | `test_cargo_reservations + test_cargo_jobs + test_sessions` 70/70，`test_database` 15/15，`test_server + test_supervision_service` 37/37；`compileall` 与 exact `git diff --check` 通过。 |
| `PRODUCTION SCHEMA41 REPLAY PASSED` | 2026-07-16 | 旧 reservation `39d9c578...` 在自然 TTL 前由 schema41 策略终结；后继 daemon `5421e008...` 加载 schema 41。reservation `0bbc781e...` 的 canonical payload API/DB 原样往返并随公开 stale 转换置 `expired`；reservation `c692e731...` 绑定短命 supervisor job `c8010778...` 后由 `maintenance.tick` 对账为 orphan/expired；紧随的 Session C job `2853a1a8...` 成功 acquire，证明 FIFO 前进；回放 job 已 release，两个专用 Session 已 archived。 |
| `FAILURE RETURNED / REVIEW CLEAN` | 2026-07-16 | `stale-session-pending-cpu-reservation-starvation` 已通过 lifecycle key 回传为 Editor07 `fixed-*`；handoff validator 161 artifacts / 0 errors，plan-output audit passed，independent final review `P0/P1/P2=0/0/0`。 |
| `NODE-SCOPED FIXED-RETURN SELECTOR / COMMIT PENDING` | 2026-07-16 | 完成 fixed-return manifest 选择器：仅当 immutable manifest 包含当前 fixing plan 的 canonical `fixed-*` 回传工件时，才排除该 fixing plan 的无关 open Failure；同 node/legacy origin Failure 仍阻断，普通 milestone 与 explicit/Goal finalize 继续保留 fixing-plan priority。新增 selector/support 回归 `96/96`、`compileall` 与 scoped diff-check 已通过。首个 managed validation 在客户端轮询 deadline 后仍由服务端完成并 accepted `24/24`，副本随后清理；本记录补写证据改变了 immutable manifest，必须重新受管验证后才可提交。 |

## Scope delivered

- 完成 CPU reservation 可执行状态、绝对过期、orphan/FIFO 与 canonical compatibility payload 的生产硬切。
- 完成 schema 41 迁移、Editor07 failure return、模块文档与 Coordinator01 M6.8 父子计划记录。
- 把 failure selector 绑定到 immutable manifest，并由 gate context、failure-audit refresh 与 Git mutex 内两次 commit guard 复用；不通过全计划 failure_audit 放宽任何普通 milestone。

## Fresh testing evidence

- Python focused/regression gates 共 122/122 通过，`compileall`、exact `git diff --check`、handoff validator 与 plan-output audit 通过。首个 managed `coordinator-actions` validation `24/24` accepted（validation run `9bb1b639...`）；该 gate 对补写 `96/96` 证据前的 manifest 有效，当前 manifest 仍须刷新。
- schema 41 production replay 已验证 stale cleanup、payload 持久化、orphan terminalization 与 FIFO advancement。
- 本轮 fixed-return selector/support 回归 `96/96`：`test_milestone_failure_scope` `13/13`，`test_failures + test_workflow_commit` `40/40`，`test_git_finalize` `43/43`。其中覆盖无关 fixing-plan Failure 放行、同 node/legacy origin Failure 阻断、两次 Git mutex guard、普通 milestone priority 与 ignored session-note 路径拒绝；`python -m compileall -q tools/session_coordinator` 与 scoped `git diff --check` 通过。

## Review

- 当前 exact manifest 的独立复审结论为 `P0/P1/P2=0/0/0`。

## 架构与硬切边界

- 不复活 stale/completed/cancelled/archived Session 的 Cargo admission，不保留旧策略兼容分支。
- 不以 Session stale 为理由终止已绑定或运行的 job；该类 reservation 只随 nominated job 生命周期推进。
- 历史 reservation 的 schema 41 payload 保持 `NULL`，只对新 reservation 写入可审计 canonical JSON。
- 未执行 raw Cargo、手工 SQL、foreign reservation release、手工 Git staging 或 maintenance finalize。
- 未关闭或改写其余 Coordinator01 open Failure；selector 仅允许已完成 fixed-return slice 使用自己的 immutable manifest，不改变任何普通 milestone、explicit finalize 或 Goal closeout 的 Failure priority。

## 当前提交门

- canonical 18-file manifest 由 successor Session
  `coordinator01-m6-8-fixed-return-scope-r2-20260716` 持有 live leases；first-run 的
  19-file binding 仍作为因 protected parent-plan path 被拒绝的不可变审计记录保留。
- 代码、测试、模块文档、本记录与受管 fixed handoff
  形成唯一 exact manifest；Editor07 SelectionModel 业务代码和其 pending 输出记录不在本提交，其他 open Failure 亦不进入本提交。
- 只允许通过 `milestone prepare/validate/review/commit M6.8` 原子提交。
