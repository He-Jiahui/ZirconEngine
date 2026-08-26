---
handoff_kind: fixed
status: fixed
created_at: 2026-08-27
summary_slug: control-console-lazy-projection-regression
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/control_plane/history.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_action_execution.py
  - tools/session_coordinator/tests/test_control_history.py
  - tools/session_coordinator/tests/test_control_http.py
  - tools/session_coordinator/tests/test_control_lazy_projections.py
  - tools/session_coordinator/tests/test_control_snapshot.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_control_snapshot tools.session_coordinator.tests.test_control_lazy_projections tools.session_coordinator.tests.test_control_history -v
  - python -B -m unittest tools.session_coordinator.tests.test_action_execution tools.session_coordinator.tests.test_control_http -v
resolved_at: 2026-08-27
---

# Coordinator01: control-console lazy projections leave the backend regression suite inconsistent

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：control snapshot load reduction, history projection and local validation queue controls
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the control snapshot, loopback HTTP facade and validation queue composition changed by this local regression.

## 失败现象与复现证据

The initial control snapshot was split from heavy failure, Git, validation,
continuation and history rows so the browser can load bounded domain projections.
The first focused run exposed an incomplete cutover: `26` snapshot/history tests
reported `4 failures + 8 errors`. Existing tests still asserted that the summary
contained deferred rows or called the removed `_validation` helper. This made the
current backend change impossible to validate or finalize even though the new
lazy-projection tests passed.

The same worktree also introduces loopback read endpoints and closed validation
queue actions. Those routes must be accepted as one backend contract; committing
only the repaired test file would bind tests to production methods absent from
HEAD.

## 最低共享层根因

The summary/detail contract was changed across snapshot, router, HTTP and action
composition, but its existing regression suite was only partially migrated.
The summary and detail projections therefore had no single executable acceptance
boundary, and queue advancement was wired independently from the maintenance
worker without a shared reentrancy gate in the committed baseline.

## 架构修复验收

- Keep the summary bounded and defer heavy rows to typed detail endpoints; do not
  restore the previous large snapshot payload.
- Migrate all existing snapshot assertions to the correct summary or detail
  contract, including live rows, terminal-history bounds and sanitized manifests.
- Preserve strict loopback Host/Origin enforcement. Public reads may omit a
  browser cookie, but ordinary mutation routes remain unauthorized.
- Start validation only from the closed catalog and serialize manual continuation
  with the maintenance worker through the same application gate.
- Persist and project failure/validation history without raw evidence payloads.
- Pass the focused backend suites, Python compilation and diff checks before a
  coordinator finalizer commit and controlled rollover.

## 禁止临时方案

- Do not re-add heavy failure, validation, Git, continuation or audit rows to the
  initial snapshot merely to satisfy legacy assertions.
- Do not expose a caller-controlled command/environment surface for validation.
- Do not bypass loopback Origin checks, accept remote binds, or grant general
  mutation authority to the anonymous observer identity.
- Do not include the concurrently edited Web source or generated `dist` assets in
  this backend failure closeout.

## 修复结果与回传

- 根因：The control summary/detail contract and its regression suite were not cut over atomically.
- 架构修复：Bounded domain projections, sanitized history endpoints, closed loopback validation actions, and one shared non-reentrant queue advancement gate now form the backend contract.
- 验证：Projection/history 26/26; action/HTTP 40/40; Python compilation and scoped diff checks passed; implementation commit a8a66c063afc13a69d5dd9766615ac538091d0a3.
- 回传：The control console backend is copy-stable; Web source and generated assets remain with their existing owner.
