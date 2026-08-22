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
  - tools/session_coordinator/tests/test_control_recovery.py
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
- 2026-08-22 live rollover 暴露 hard-cutover 尾部缺口：旧 daemon 的 `/health` 含数百条 lease blocker，身份预检超过 client 3 秒 deadline；successor 正确旋转 bearer 后，已确认 rollover 的状态查询收到 `401 unauthorized`，旧 client 未重新读取 descriptor，因而在 durable action 已成功时仍向调用方报错。
- RED：`test_repository_preflight_uses_bounded_identity_endpoint`、`test_repository_preflight_falls_back_to_health_for_predecessor_daemon`、`test_rollover_status_poll_refreshes_successor_runtime_token`、`test_local_health_identity_and_session_commands_require_runtime_token` 均因缺少生产行为失败。
- GREEN：上述四项精确回归 4/4 通过；随后 client、deferred-action client、runtime descriptor、control HTTP、security matrix、control recovery 与 server 全套 130/130 通过（458.913s）。
- Managed r1 ticket `27660d876ed640d0bffa030e8d8b4fd8` correctly failed because its validation-copy dependency roots omitted the existing plan-handoff validator; no source behavior failed and the copy was not reused.
- Managed r2 ticket `8a4d7238f3174339b77f21dd22f9e6f1` retained source manifest `bc49375c5de599d23ef5df86c467db77a0318de3c2305a7195d7d9b88e09bdaa`, added only the validator dependency root, and passed 130/130 from copy job `f0d134938e3c4227a92291454b795901` with exit 0 in 382.657s.
- Managed r3 ticket `9f27ed02b50540e2a62b4976a438bfa5` exposed one deterministic-test defect: the late-POST reconciliation case asserted the final polling diagnostic even though an earlier status GET had already returned and a last-millisecond retry could time out. The production client is unchanged; the test now uses a direct Handler event to prove the GET overtakes the blocked POST while submission remains unknown, and passed 10/10 independent-process repetitions.

## 最低共享层根因

RunningCoordinator publishes an empty runtime token, CoordinatorClient sends no bearer, CoordinatorRequestHandler._authorized always succeeds, and ControlPlaneHttp marks every loopback request runtime-authorized; this bypasses browser Origin/cookie/CSRF and runtime credential boundaries.

初始 bearer hard-cutover 已进入 HEAD，但 client 仍把含完整 blocker 清单的 `/health` 当每次 control request 的身份探针，并把 bearer rotation 当不可恢复授权失败。认证边界正确后，这两个旧假设分别造成规模相关的 command preflight timeout 与 rollover 成功后的假失败。

## 架构修复验收

- Each daemon instance publishes a non-empty unpredictable runtime token and local clients send its exact Bearer value.
- Legacy command/health and runtime-only control routes reject missing or mismatched bearer credentials.
- Browser control routes require bootstrap cookie, loopback Origin or referrer, CSRF for mutation, and one-time elevation grants without receiving the runtime bearer.
- Runtime descriptor diagnostics, logs, errors, UI payloads, screenshots, and Git never expose the token.
- Focused control HTTP, client, runtime descriptor, server, security matrix, and control recovery suites pass after hard cutover.
- Repository identity preflight has a bearer-authenticated constant-size projection; predecessor compatibility may fall back only on typed endpoint absence and uses the bounded control timeout.
- A confirmed rollover may reload only the successor descriptor after typed `unauthorized`, then query the same action ID; it never repeats preview or confirmation.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: production and focused tests implement per-instance bearer, bounded `/identity`, browser ticket/cookie/Origin/CSRF enforcement and same-action rollover token refresh. Live controlled rollover intent `7d857f28c5964cc8b32be8bd2b4422a5` succeeded from instance `d4a0ceae...` to `335e1fb2...`; successor is schema 65 / `read_write` / healthy, descriptor token length is 43, and the four stale `mvp-test-fixtures-*` cleanup reservations are absent. Local and immutable-copy security/client/server regressions are 130/130 green. A final manifest-bound ticket after this evidence update, scoped commit, WeCom receipt and post-commit rollover remain required before this record can become fixed.
