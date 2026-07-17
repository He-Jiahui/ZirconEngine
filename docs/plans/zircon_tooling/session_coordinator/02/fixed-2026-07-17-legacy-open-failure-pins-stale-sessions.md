---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: legacy-open-failure-pins-stale-sessions
origin_plan: docs/plans/zircon_tooling/session_coordinator/02-codex-session-hook-sync.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/legacy.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_legacy_migration.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_legacy_migration tools.session_coordinator.tests.test_server
resolved_at: 2026-07-17
---


# Coordinator02 → Coordinator01: open failure pins stale legacy Sessions forever

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/02-codex-session-hook-sync.md`
- 来源执行切片：H3 single-flight evidence reconciliation 与跨 Session 状态投影
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：legacy root Session 的 liveness、stale/归档与 Failure graph 映射由 Coordinator01 管理；Coordinator02 只能观察并投影其结果。

## 失败现象与复现证据

Coordinator 配置的 Session TTL 为 600 秒，maintenance interval 为 900 秒。最近三次
maintenance tick 均为 `succeeded`，但只读聚合仍显示 9 条超过一小时无 heartbeat 的业务
Session 处于 `active` 或 `resolving_failure`。

这些记录没有 live PID、live lease 或 fresh heartbeat；其共同点是关联计划存在 open Failure。
legacy report 将它们的 `activity_reasons` 标为 `open_failure`，并映射为 active。部分根 note
甚至声明 `complete` 或 `archived`，但仍被长期留在 `.codex/sessions/` 活动根目录。

这会把 durable Failure priority 与 Session liveness 混为一谈，污染 owner、活动列表、归档
统计和后续调度判断；不得由运维直接改 Session 状态或移动文件掩盖问题。

## 最低共享层根因

`LegacyMigrationService._database_activity_reasons()` 将 fixing plan 的任意 `open_failure`
加入 `activity_reasons`。`_read_note()` 只要存在任一 activity reason 就映射为 `active`；
`CoordinatorApplication._maintenance_tick_unlocked()` 又把所有有 activity reason 的 Session
列入 `legacy_active_sessions`，从 `mark_stale()` 与 `archive_stale()` 中排除。

因此 open Failure（应由 Failure graph 持久管理）错误地成为了 legacy Session 的永久 liveness
租约。maintenance tick 正常运行也无法收束这些历史 root note。

## 架构修复验收

- 明确拆分 Session liveness reason 与 Failure priority reason。仅 live PID、recent note、
  fresh service heartbeat、live lease 或 pending patch 可阻止 stale/归档；open Failure 不可。
- open Failure 必须继续留在 Failure graph、保持 fixing-plan priority，并在 Session 变 stale 或
  根 note 归档后仍可 `failure open` 查询；不得自动关闭、降级或删除 Failure。
- legacy note 在没有 liveness reason 时按现有 TTL/归档窗口收束，即使源 frontmatter 含过期的
  active/resolving_failure/complete/archived 状态；有真实 liveness 的相同场景必须保持 active。
- 增加 focused 回归：old note + open Failure -> stale/eligible archive；归档后 failure remains
  open；live PID/lease/recent heartbeat + open Failure -> active；重复 maintenance tick 幂等。
- 修复加载后用受管 maintenance tick 收束当前过期根 note，保留审计记录、Failure artifacts、
  运行中 Cargo 和真实 live Session，不使用人工 SQLite 或批量移动作为替代。

## 禁止临时方案

- 禁止把全部 resolving_failure Session 强制 archive，或为了清理活动列表关闭 open Failure。
- 禁止删除 `.codex/sessions/` root note、直接修改 SQLite，或把 open Failure 从 priority graph
  中移除。
- 禁止用扩大 TTL、关闭定时清理或把所有 legacy note 视为 archived 来掩盖 liveness 分类错误。

## 修复结果与回传

- 根因：LegacyMigrationService treated a fixing plan open Failure as a Session liveness reason.
- 架构修复：Removed Failure graph lookup from legacy liveness classification; only PID, freshness, heartbeat, lease, and pending patch signals retain a note.
- 验证：python -m compileall -q tools/session_coordinator; python -m unittest tools.session_coordinator.tests.test_legacy_migration tools.session_coordinator.tests.test_server -v (40 passed).
- 回传：Coordinator02 H3 may resume with stale/archived root notes independent of durable Failure priority.
