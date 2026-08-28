---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: session-register-durability-runtime-auth-drift
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/server.py
tests:
  - tools/session_coordinator/tests/test_command_protocol.py
  - tools/session_coordinator/tests/test_session_register_durability.py
---

# Coordinator01 Session register durability runtime auth drift

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：failure graph and session-registration durability regression sweep
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns both the runtime-authenticated command endpoint and its durability regression transport.

## 失败现象与复现证据

`python -u -B -m unittest tools.session_coordinator.tests.test_session_register_durability.SessionRegisterDurabilityTests.test_two_registration_requests_are_terminal_and_admit_a_lease -v` failed `1/1`: the first `POST /command` returned HTTP `401` instead of `200`. The full module consequently did not reach a terminal result within 120 seconds because each HTTP scenario exercised the same unauthenticated transport before coordinator session logic.

## 最低共享层根因

The production handler correctly requires `Authorization: Bearer <runtime token>`, and current control-plane HTTP tests already send `running.token`. The older `SessionRegisterDurabilityTests._request` helper sent only `Content-Type`, so its registration, replay, accepted-query and restart cases stopped at the HTTP authentication boundary rather than testing command durability.

The same sweep then found the identical stale transport in
`CommandProtocolTests._request`: its durable admission and in-flight query
cases returned `401` before the mocked command could enter. A repository-wide
scan of direct test requests confirmed the other Content-Type-only command
request is the intentional unauthorized server regression.

## 架构修复验收

- Bind every durability-suite HTTP request to the exact token issued by its `RunningCoordinator` instance.
- Keep unauthorized requests rejected; do not weaken or bypass production authentication.
- Re-run the original focused registration/lease reproduction and the complete durability module.
- Re-run the command-protocol module so durable admission and accepted-state queries reach their target logic.
- Preserve command replay, accepted-state visibility, restart terminalization and failed re-registration assertions.

## 禁止临时方案

- Do not make `/command` or `/command/requests/*` anonymously accessible.
- Do not read a global token or reuse a token across isolated coordinator instances.
- Do not replace HTTP coverage with direct application calls merely to avoid authentication.

## 修复结果与回传

`SessionRegisterDurabilityTests._request` now requires the token issued by the
same `RunningCoordinator` instance and sends it as a Bearer authorization
header. Every registration, replay, accepted-query and restart scenario uses
that explicit instance identity; production HTTP authorization is unchanged.

The original dual-registration/lease reproduction passed `1/1`. The complete
durability module passed `15/15` in 130.173 seconds, including duplicate replay,
pre-admission preparation, accepted-state visibility, restart terminalization,
deferred database failure and failed re-registration. The existing server
runtime-token guard passed `1/1`, proving an unauthenticated `/command` request
still returns HTTP `401`.

The adjacent command-protocol RED was `12/14`: durable accept/query returned
HTTP `401`, and the in-flight handler never entered. Its shared request helper
now uses the same explicit per-instance token contract. The complete module
passed `14/14` in 77.000 seconds; no production code or anonymous request
expectation changed.

Open state: `source_validated / local failure record pending managed commit`.
