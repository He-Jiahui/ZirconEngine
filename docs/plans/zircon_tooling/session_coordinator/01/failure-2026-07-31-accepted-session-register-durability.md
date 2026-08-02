---
handoff_kind: failure
status: open
created_at: 2026-07-31
summary_slug: accepted-session-register-durability
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/client.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/sessions.py
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-16-lifecycle-orphan-recovery-maintenance-hold-integrity-deadlock.md
tests:
  - tools/session_coordinator tests for accepted session.register command durability and terminal request status
  - tools/session_coordinator restart/recovery tests preserving accepted command execution or a terminal failure record
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

Open state: `待修复`; 当前未获得任何可用 Editor05 r2/r3 Session 或 lease，不声明后续源码改动受管。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-07-31 | accepted `session.register` durability | open | request `309202761f554c4e868a8aeb5a6db290` 与 `a8e7dc366f2f4bf8af4ba795a3637d97` 均返回 accepted post-timeout；对应 `session show` 均为 `session_not_found`。 |
