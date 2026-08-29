---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: control-json-role-boundary
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/tests/test_control_http.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_runtime_role_endpoints_type_malformed_roles -v
  - python -B -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_runtime_role_endpoint_projects_malformed_role_as_400 -v
  - python -B -m unittest tools.session_coordinator.tests.test_control_security tools.session_coordinator.tests.test_control_security_matrix -v
resolved_at: 2026-08-30
---

# Coordinator01: control JSON role boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P2-009` JSON schema malformed-input review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the runtime-authenticated control router and
  its typed JSON request boundary.

## 失败现象与复现证据

The bootstrap-ticket and elevation-grant endpoints constructed
`WebControlRole` directly from caller JSON. An unknown role string or a
non-string role raised raw `ValueError`; the HTTP adapter treated that as an
unexpected server error and returned 500 instead of a stable client error.

## 最低共享层根因

The two runtime-only endpoints duplicated enum coercion and had no shared typed
schema boundary. Authentication was enforced, but malformed authenticated JSON
could still escape the router's `CoordinatorError` contract.

## 架构修复验收

- both endpoints use one role parser and reject non-string or unknown roles;
- malformed roles return `invalid_request` without raw enum text;
- the HTTP projection is 400 and keeps the sanitized message;
- absent/falsy role values retain the existing endpoint defaults;
- runtime authentication and elevation authorization are unchanged.

## 禁止临时方案

- Do not catch `ValueError` only in the HTTP adapter or turn arbitrary values
  into strings.
- Do not weaken runtime authentication, default roles, or maintainer elevation
  checks to make malformed requests pass.

## 修复结果与回传

- 根因：Bootstrap and elevation endpoints duplicated direct enum coercion, so truthy non-string and unknown role values escaped the typed JSON request boundary.
- 架构修复：Use one router-owned role parser for both runtime-authenticated endpoints, preserving existing defaults while returning sanitized invalid_request for malformed values.
- 验证：Focused router and real HTTP role regressions passed 2/2; control security matrix passed 5/5; full Control HTTP suite passed 36/36; py_compile and scoped diff checks passed.
- 回传：Returned Coordinator01 control role parsing with one typed schema boundary and unchanged runtime authorization semantics.
