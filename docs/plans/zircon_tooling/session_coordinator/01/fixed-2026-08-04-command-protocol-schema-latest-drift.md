---
handoff_kind: fixed
status: fixed
created_at: 2026-08-03
summary_slug: command-protocol-schema-latest-drift
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/tests/test_command_protocol.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_command_protocol.CommandProtocolTests.test_schema_49_upgrade_preserves_terminal_incident_rows -v
resolved_at: 2026-08-04
---


# Coordinator01: command-protocol migration test pins a stale latest schema

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：accepted `session.register` durability 相邻 command-protocol 回归
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：schema migration 与 command journal 历史 incident replay 均由 Coordinator01 持有，必须在同一计划内前向修复测试真值漂移。

## 失败现象与复现证据

The accepted-session-register durability adjacent suite ran 37 tests and found one
Coordinator01-owned failure. The schema-49 incident replay migrated successfully to
the repository's current schema 58, but the test still asserted that migration must
stop at schema 50. The terminal incident rows were not reported as lost; the stale
test constant rejected the valid current migration result before checking them.

## 最低共享层根因

`test_schema_49_upgrade_preserves_terminal_incident_rows` duplicated the migration
module's latest-version constant. Schema 50 was correct when the command journal was
introduced, but later monotonic migrations advanced the authoritative version to 58.

## 架构修复验收

- The test imports `LATEST_SCHEMA_VERSION` from the migration owner and requires the
  schema-49 fixture to reach that exact version.
- The replay still proves command-request and Cargo-start incident rows survive the
  complete current migration chain.
- Focused command-protocol, session, and accepted-registration suites pass.

## 禁止临时方案

- Do not lower the production schema version, stop migration at 50, skip later
  migrations, or weaken the historical-row assertions.

## 修复结果与回传

- 根因：The schema-49 replay test duplicated schema 50 instead of reading the migration owner latest-version authority.
- 架构修复：The test imports LATEST_SCHEMA_VERSION from migrations and verifies the fixture reaches the complete current migration chain while preserving terminal incident rows.
- 验证：Fresh Windows Python regression: exact schema replay 1/1 passed; command protocol, accepted-registration durability, and Failure graph suites 49/49 passed.
- 回传：Coordinator01 command-protocol migration gate can resume; stale schema pin is removed.
