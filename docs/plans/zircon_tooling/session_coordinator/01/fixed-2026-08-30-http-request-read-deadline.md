---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: http-request-read-deadline
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/server.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/tests/test_control_http.py
tests:
  - python -m unittest -v tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_legacy_command_sets_request_read_deadline
resolved_at: 2026-08-30
---

# Coordinator01: HTTP request read deadline

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P1-006` HTTP parsing deadline review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the legacy command transport and its request
  parsing boundary.

## 失败现象与复现证据

The control adapter applies a five-second socket timeout for SSE, but the
legacy `/command` path parsed headers and then blocked on `rfile.read()` with
the socket's inherited timeout. A caller could advertise a body length and
send it slowly, occupying one of the coordinator's request workers.

## 最低共享层根因

Request deadline setup lived inside the SSE path instead of at the transport
handler boundary, so ordinary GET/POST header and body parsing had no explicit
read deadline.

## 架构修复验收

- every Coordinator GET/POST handler sets a five-second socket read deadline
  before endpoint parsing;
- control HTTP may keep its explicit SSE timeout behavior;
- existing framing/content-length validation and command routing remain
  unchanged.

## 禁止临时方案

- Do not raise worker limits or rely on executor capacity to mask slow reads.
- Do not add caller-controlled timeout headers or disable body-size checks.

## 修复结果与回传

- 根因：Legacy /command parsed headers and body without setting a socket read deadline; only SSE had a timeout.
- 架构修复：Set the CoordinatorRequestHandler five-second read deadline at every GET/POST boundary while preserving ControlPlaneHttp framing limits and SSE timeout behavior.
- 验证：RED settimeout call count was 0; GREEN deadline and legacy framing regressions passed 6/6; py_compile and scoped diff check passed.
- 回传：Returned the HTTP request read deadline fix to Coordinator01.
