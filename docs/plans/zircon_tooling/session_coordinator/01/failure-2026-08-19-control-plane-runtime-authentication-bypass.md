---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-19
summary_slug: control-plane-runtime-authentication-bypass
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - docs/cli-and-tooling/local-session-coordinator.md
  - docs/plans/zircon_tooling/session_coordinator/01/2026-07-13-simplified-session-management-acceptance.md
  - tools/session_coordinator/client.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/supervision/runtime_descriptor.py
  - tools/session_coordinator/tests/test_client.py
  - tools/session_coordinator/tests/test_control_http.py
  - tools/session_coordinator/tests/test_runtime_descriptor.py
  - tools/session_coordinator/tests/test_server.py
---

# control-plane-runtime-authentication-bypass: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Control-plane focused security suite / optimize P0
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Control-plane focused security suite / optimize P0` — python -m unittest tools.session_coordinator.tests.test_control_http -v

## 最低共享层根因

RunningCoordinator publishes an empty runtime token, CoordinatorClient sends no bearer, CoordinatorRequestHandler._authorized always succeeds, and ControlPlaneHttp marks every loopback request runtime-authorized; this bypasses browser Origin/cookie/CSRF and runtime credential boundaries.

## 架构修复验收

- Each daemon instance publishes a non-empty unpredictable runtime token and local clients send its exact Bearer value.
- Legacy command/health and runtime-only control routes reject missing or mismatched bearer credentials.
- Browser control routes require bootstrap cookie, loopback Origin or referrer, CSRF for mutation, and one-time elevation grants without receiving the runtime bearer.
- Runtime descriptor diagnostics, logs, errors, UI payloads, screenshots, and Git never expose the token.
- Focused control HTTP, client, runtime descriptor, server, security matrix, and control recovery suites pass after hard cutover.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `待修复`; the coordinator must keep the validation ticket and route this Plan to repair work.
