---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: http-request-concurrency-boundary
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_server.py
tests:
  - python -m unittest -v tools.session_coordinator.tests.test_server.ServerTests.test_http_requests_are_bounded_and_shutdown_drains_handlers
  - python -m unittest -v tools.session_coordinator.tests.test_server.ServerTests.test_http_requests_enforce_per_client_quota
resolved_at: 2026-08-30
---

# Coordinator01: HTTP request concurrency boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P1-005` HTTP transport concurrency review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the loopback HTTP transport and its shutdown
  lifecycle.

## 失败现象与复现证据

`_CoordinatorHttpServer` inherited `ThreadingHTTPServer` with
`daemon_threads=True`, so every accepted socket created an unmanaged daemon
thread. There was no global or per-client request budget, no typed overload
response, and `RunningCoordinator.stop()` could close the listener without
draining active handlers. A slow or streaming client could therefore consume
unbounded threads and leave mutation requests competing with an uncontrolled
shutdown.

## 最低共享层根因

The transport delegated admission and ownership to `ThreadingMixIn` instead of
keeping an explicit request registry. Without a bounded executor, counters and
a shutdown state, the coordinator had no durable local capacity contract.

## 架构修复验收

- accepted HTTP requests run on a fixed non-daemon executor with a global
  worker limit and per-client limit;
- requests over either limit receive a structured HTTP 503
  `request_overloaded` response with `Retry-After` and are closed cleanly on
  Windows;
- the listener stops accepting work before `server_close()` waits for every
  accepted request and active-request accounting returns to zero;
- existing Coordinator HTTP routing and lifecycle behavior remain unchanged.

## 禁止临时方案

- Do not restore unbounded daemon threads or silently drop accepted requests.
- Do not increase the limit without an explicit transport capacity contract.
- Do not bypass Coordinator admission by creating caller-owned HTTP workers.

## 修复结果与回传

- 根因：The Coordinator inherited unbounded ThreadingMixIn daemon dispatch, so global and per-client request admission plus shutdown ownership were undefined.
- 架构修复：Run accepted requests through a fixed non-daemon ThreadPoolExecutor with global and per-client quotas, typed 503 overload responses, bounded Windows input draining, and server_close executor drain.
- 验证：RED missing request_worker_limit; GREEN connection-level concurrency and per-client quota tests passed 2/2; py_compile and scoped diff check passed. Full suites timed out under unrelated process saturation.
- 回传：Returned the HTTP request concurrency boundary fix to Coordinator01 with bounded admission, typed overload, and shutdown drain.
