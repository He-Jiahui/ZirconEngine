---
handoff_kind: fixed
status: fixed
created_at: 2026-07-31
summary_slug: accepted-session-register-durability
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/command_requests.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/sessions.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_session_register_durability
  - python -m unittest tools.session_coordinator.tests.test_command_protocol
  - python -m unittest tools.session_coordinator.tests.test_failures
resolved_at: 2026-08-04
---


# Editor 05 -> Coordinator 01: 已受理 session.register 命令丢失

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：M1 SceneMode input ownership 与 M4.2 HighlightSet gateway handoff 的受管租约登记
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：accepted command 的持久化、终态收敛与 recovery replay 属于 Coordinator01 控制面 owner，Editor05 不能以 caller retry 或绕过 lease 修复。
- 关联既有失败：`docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-16-lifecycle-orphan-recovery-maintenance-hold-integrity-deadlock.md`

## 失败现象与复现证据

两次合法 `session register` 命令均由 coordinator client 报告 `command_post_timeout`，且 details 明确为 `submission: accepted`：

- `309202761f554c4e868a8aeb5a6db290`：`editor05-m4-gateway-handoff-20260731-r2`。
- `a8e7dc366f2f4bf8af4ba795a3637d97`：`editor05-m4-gateway-handoff-20260731-r3`。

两次命令都使用当前 CLI 声明的 `--session-id`、`--plan-path`、`--write-scope` 和 primary role。随后对对应 session id 做一次精确只读 `session show`，均返回 `session_not_found`。这不是参数解析错误、租约冲突或 Cargo 排队；accepted command 未产生 session，也没有可查询的 terminal failure。

## 最低共享层根因

协调器将 command request 投影为 `accepted`，但没有保证其 session.register mutation 可在同一持久化生命周期中完成，或在 daemon/recovery 中将其持久化为 terminal failure。调用方因而不能区分“稍后可安全继续”与“命令已丢失”，会话/lease/failure handoff 无法建立。

## 架构修复验收

- `POST /command` 返回或被 reconciliation 观察到 `accepted` 后，该 request 必须最终落为 completed/failed，并能按 request id 查询；不得静默消失。
- `session.register` 的 session、display name、plan path、write scope、role、status 与 event 必须同一事务提交。任一阶段失败必须留下 terminal failed request 和零 partial session state。
- daemon restart、maintenance recovery 和 descriptor rollover 必须重放已接受但未执行的命令，或显式 terminal-fail；不得让 caller 只得到 `session_not_found`。
- 回归覆盖两个不同 session id 的 accepted registration、立即 query、恢复后 query、失败回滚和后续 lease claim admission。
- 修复应回传到既有 Coordinator01 lifecycle failure；Editor05 取得 terminal session/lease receipt 后才继续触碰 foreign M1/M4 源码。

## 禁止临时方案

- 禁止依赖 sleep、重复盲提交、手改 coordinator SQLite/JSON、把 `command_post_timeout` 当作租约授权，或绕过 lease 修改共享源码。
- 禁止以 maintenance hold、daemon restart 或 caller-local retry 掩盖 accepted-command durability 缺陷。

## 修复结果与回传

- 根因：The command journal could persist session.register as accepted without atomically committing either the Session mutation and terminal response or a durable terminal failure, leaving accepted requests with session_not_found.
- 架构修复：Execute accepted session registration and terminalization transactionally; roll back partial Session, event, workflow, and failure imports on error. Persist second-phase failures, defer only explicitly failed terminal writes, retry them during maintenance, and terminalize interrupted accepted requests during startup reconciliation without replaying a completed mutation.
- 验证：Managed validation ticket f9e868fc90314d77984a08223898147b passed 49/49 tests in validation copy job b9704e25b55940159aa077f3ad1e13e0 (exit 0): 14 session-register durability, 14 command protocol, and 21 failure graph cases. Editor05 origin request f6ce396f91be4bca831daf7586c9c88c terminalized completed, session show returned editor05-accepted-register-origin-primary-r1-20260804, and lease request 71172355a9bb404fa99009963654d73e acquired the fixed destination.
- 回传：Coordinator01 returned transactional accepted-session registration durability to Editor05 with managed 49-test evidence and a fresh terminal register/show/lease origin replay.
