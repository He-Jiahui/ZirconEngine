---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-14
summary_slug: session-register-failure-snapshot-long-write-lock
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/command_requests.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_failures.py
  - tools/session_coordinator/tests/test_server.py
  - tools/session_coordinator/tests/test_session_register_durability.py
resolved_at: 2026-08-14
---


# session-register-failure-snapshot-long-write-lock: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Frameworks01 session registration and validation-copy admission concurrency
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Frameworks01 session registration and validation-copy admission concurrency` — Block FailureGraphService immutable snapshot parsing during session.register; validation_copy.materialize currently waits behind the registration BEGIN IMMEDIATE and maintenance recovery logging can exit on a second database-is-locked error.

## 最低共享层根因

session.register parses and validates the full docs/plans snapshot inside execute_accepted_transactionally, while maintenance exception handlers perform an unguarded second write transaction.

## 架构修复验收

- Capture and parse the immutable failure snapshot before the accepted registration write transaction.
- Inside the registration transaction, verify the failure-artifact fingerprint and atomically replace the graph/register the Session.
- A blocked snapshot parse does not delay validation-copy admission, and DB-busy diagnostic persistence does not terminate the maintenance loop.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：session.register parsed and validated the full failure snapshot while its accepted mutation held SQLite BEGIN IMMEDIATE; maintenance recovery then performed an unguarded second diagnostic write after database busy, terminating the long-lived loop.
- 架构修复：Prepare the immutable failure graph after durable request acceptance but outside the admission writer, revalidate the failure-artifact fingerprint and replace the graph inside the registration transaction, and make maintenance failure event persistence best-effort so DB busy cannot terminate the worker.
- 验证：Local: test_failures 23/23, test_session_register_durability 15/15, registration/maintenance focused tests 4/4. Managed validation-copy job 0d64cc0ab0fa4f97999816ed65a1f4a7, input manifest bfbdaf29e4e633e8596876a3729fa47efb3397db717c4f761e65ab3c9dd0b41d, run c83349fdc073475fa05575bc55e92879, command python -m unittest tools.session_coordinator.tests.test_failures, exit 0, 23/23.
- 回传：Session registration no longer holds the database writer during failure snapshot parsing; immutable drift fails closed, duplicate requests skip preparation/import, and maintenance DB-busy diagnostics no longer kill recovery.
