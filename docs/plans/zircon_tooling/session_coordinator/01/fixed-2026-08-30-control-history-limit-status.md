---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: control-history-limit-status
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/tests/test_control_http.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_history_limit_errors_map_to_bad_request -v
  - python -B -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_history_limit_error_projects_as_400 -v
resolved_at: 2026-08-30
---

# Coordinator01: control history limit HTTP status

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P2-009` malformed query review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns both the typed history query contract and its
  HTTP status projection.

## 失败现象与复现证据

The history query parser correctly raised `history_limit_invalid` for a
non-integer or out-of-range limit, but the central HTTP status map omitted that
code. The stable client-input error was therefore projected as HTTP 409 instead
of 400.

## 最低共享层根因

The query parser and transport status registry evolved independently. The
typed code existed, but was not registered with the other bounded malformed
request errors.

## 架构修复验收

- `history_limit_invalid` remains the router-owned typed error;
- the central HTTP mapping projects it as 400;
- no history query bounds or database behavior change.

## 禁止临时方案

- Do not change the parser to a generic `invalid_request` only to reuse an
  existing status entry.
- Do not special-case either history endpoint at its call site.

## 修复结果与回传

- 根因：The router's typed history_limit_invalid code was omitted from the central HTTP client-error status registry.
- 架构修复：Register history_limit_invalid in the shared bad-request mapping while preserving router parsing, bounds, and database behavior.
- 验证：Focused router/status and real HTTP projections passed 2/2; merged Control HTTP boundary suite passed 18/18; py_compile and scoped diff checks passed.
- 回传：Returned Coordinator01 history limit errors with stable typed HTTP 400 projection.
