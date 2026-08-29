---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: control-json-number-boundary
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_control_http.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_json_body_types_extremely_large_numbers -v
  - python -B -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_legacy_command_endpoint_types_extremely_large_numbers -v
resolved_at: 2026-08-30
---

# Coordinator01: control JSON number boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P2-009` JSON malformed-input review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the shared JSON decoder boundary for both
  control API and legacy command requests.

## 失败现象与复现证据

Python `json.loads` raises `ValueError` when a JSON integer exceeds the
interpreter's conversion limit. A 5,000-digit numeric token therefore escaped
the control router's JSON error handler, while legacy `/command` projected the
raw conversion failure as `invalid_request` instead of the stable `invalid_json`
contract.

## 最低共享层根因

The decoder catches syntax and UTF-8 errors but not numeric conversion errors;
the two HTTP entry points also classify decoder failures separately.

## 架构修复验收

- oversized numeric JSON tokens are classified as `invalid_json`;
- direct control and legacy command paths expose the same sanitized message;
- no process-wide integer conversion limit is changed;
- valid numeric JSON remains accepted.

## 禁止临时方案

- Do not raise Python's integer conversion limit or catch the error only in an
  outer HTTP handler.
- Do not expose interpreter error text or silently coerce oversized numbers.

## 修复结果与回传

- 根因：Both JSON decoder boundaries omitted Python numeric-conversion ValueError, allowing extreme integer tokens to escape the typed malformed-request contract.
- 架构修复：Classify numeric conversion failures as sanitized invalid_json at the shared control router and legacy command decoder without changing the process-wide integer limit.
- 验证：Direct and legacy 5000-digit regressions passed 2/2; full Control HTTP suite passed 36/36; command protocol passed 16/16; py_compile and scoped diff checks passed.
- 回传：Returned Coordinator01 JSON numeric decoding with uniform sanitized invalid_json behavior for extreme integer tokens.
