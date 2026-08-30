---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: command-internal-error-disclosure
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/command_requests.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_command_protocol.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_command_protocol -v
resolved_at: 2026-08-30
---

# Coordinator command failures disclose internal exception strings

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P2-002` in `docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the command journal and HTTP defensive error contract.

## 失败现象与复现证据

Unexpected exceptions in the command journal were converted to `internal_error` while
retaining `str(error)` as the durable message. The HTTP defensive boundary did the same,
returning raw exception text to callers and emitting no correlation identifier. A simulated
SQLite path or SQL detail could therefore become visible through the request journal and API.

## 最低共享层根因

The command transport had no stable unexpected-error contract. Journal persistence, replay,
and HTTP fallback each formatted unhandled exceptions independently, so the most detailed
local exception string became the externally observable error.

## 架构修复验收

- Map unexpected command exceptions to stable `internal_error` text and a request-bound
  `correlationId` in durable journal records and replay.
- Return the same stable shape from the HTTP defensive boundary while logging the full
  exception with correlation ID only on the service side.
- Preserve declared `CoordinatorError` codes/messages and all accepted/deferred/replay
  durability semantics.

## 禁止临时方案

- Do not expose filesystem paths, SQL, process command lines, or exception strings in the
  command response or durable request journal.
- Do not replace durable failure records with an in-memory-only log.
- Do not broaden this fix to unrelated cleanup, Cargo, or Tray transport behavior.

## 修复结果与回传

- 根因：The command transport independently persisted and returned raw unexpected exception strings, with no stable correlation contract.
- 架构修复：Map unexpected command failures to stable internal_error text plus request-bound correlationId in journal and HTTP responses, while logging the full exception only at the service boundary.
- 验证：RED reproduced filesystem and SQL detail leakage in all three journal execution modes and the HTTP fallback; GREEN passed command_protocol 16/16, session_register_durability 15/15, focused py_compile, and diff checks including replay and restricted-log assertions.
- 回传：Returned Coordinator01 command-internal-error-disclosure with durable redaction, correlation, replay, and restricted logging evidence.
