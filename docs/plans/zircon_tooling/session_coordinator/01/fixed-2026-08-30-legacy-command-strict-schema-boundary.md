---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: legacy-command-strict-schema-boundary
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_command_protocol.py
tests:
  - python -m unittest -v tools.session_coordinator.tests.test_command_protocol.CommandProtocolTests.test_handler_rejects_malformed_command_envelopes
  - python -m unittest -v tools.session_coordinator.tests.test_command_protocol
resolved_at: 2026-08-30
---

# Coordinator01: legacy command strict schema boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P1-004` legacy command schema review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the authenticated legacy command transport and
  its durable request journal boundary.

## 失败现象与复现证据

The legacy `/command` handler coerces a non-string `command` with `str()`,
accepts some falsey non-object `arguments` as an empty object, emits raw Python
type messages for other malformed fields, and ignores unknown top-level fields.
The same envelope can therefore be accepted or rejected differently based on
Python truthiness rather than one versioned request schema.

## 最低共享层根因

JSON decoding and command dispatch are adjacent in the handler, but there is no
explicit envelope validator between them. Each field performs its own partial
coercion, while the outer exception boundary supplies the observable error.

## 架构修复验收

- the decoded envelope must be an object with only `command`, `arguments`, and
  optional `request_id`;
- `command` and present `request_id` must be strings, and `arguments` must be an
  object when present;
- malformed envelopes return HTTP 400 `invalid_request` with one sanitized
  stable message;
- valid existing command envelopes preserve journal, replay, and admission
  behavior.

## 禁止临时方案

- Do not stringify caller values, depend on truthiness, or leak Python parser
  and type errors.
- Do not weaken bearer authentication or bypass the durable command journal.

## 修复结果与回传

- 根因：Legacy /command performed partial coercion and truthiness-based defaults instead of validating one strict command envelope, so malformed caller shapes diverged in status and leaked Python type errors.
- 架构修复：Validate the decoded envelope explicitly: object-only allowed fields, string command/request_id, object arguments, no coercion, and one sanitized invalid_request response while preserving journal/replay semantics.
- 验证：Malformed envelope RED covered five variants; GREEN focused regression passed 1/1, command protocol passed 17/17, legacy framing/JSON regressions passed 5/5, py_compile and scoped diff checks passed.
- 回传：Returned Coordinator01 legacy command strict schema boundary with typed sanitized malformed-envelope handling.
