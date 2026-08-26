---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-26
summary_slug: artifact-cleanup-reservation-starvation
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/artifact_governance.py
  - tools/session_coordinator/tests/test_artifact_governance.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_artifact_governance.ArtifactGovernanceTests.test_failed_recovered_reservation_does_not_starve_current_candidate tools.session_coordinator.tests.test_artifact_governance.ArtifactGovernanceTests.test_failed_recovered_reservation_rotates_behind_pending_reservation -v
  - python -B -m unittest tools.session_coordinator.tests.test_artifact_governance -v
---

# Coordinator01: failed artifact cleanup reservation starves independent candidates

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator01 canonical failure cleanup and managed validation recovery
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：生产 cleanup 证据与最低修复都属于同一 Coordinator01 artifact governance owner。

## 失败现象与复现证据

On 2026-08-26, managed Runtime15 and Plugins05 validation copies both failed before Cargo with
`unmanaged_artifacts_detected`. Official `artifact cleanup` request
`84d6239748764cf3ad2b3f697e55bf13` returned `deleted=[]` after the first candidate,
`D:\ZirconBuilds\tooling15-wave100-runtime-20260826-205045`, failed with Windows error 32.
Three independent candidates remained untouched.

The durable event stream showed the same cleanup reservation being retried about every 40 seconds.
The live producer was PID 40928 running
`.codex/sessions/tooling15-integrated-wave100-bootstrap.ps1`; its root was not registered through an
artifact fixture or product-staging lease. The coordinator correctly preserved the locked tree and
continued to block validation admission. It incorrectly let that one retry consume the default
`max_candidates=1` cleanup budget forever.

## 最低共享层根因

`ArtifactGovernanceService.cleanup()` counted both recovered deletions and recovered failures as
processed candidates. It then returned before scanning any current candidate. A failed reservation
also retained its original `reserved_at`, so ordered recovery always selected it first. The same
path could additionally be selected again by `_cleanup()` in the same call.

The producer's missing lease is a separate Tooling15 ownership defect. This failure does not exempt
the producer path, delete a live tree, or weaken `require_clean()`.

## 架构修复验收

- Count only successful recovered deletions against the bounded deletion budget.
- Exclude already-attempted paths from the current cleanup phase so a failed tree is not retried
  twice in one call.
- Preserve the reservation and filesystem identity on failure, but refresh its `reserved_at` so
  later reservations receive a bounded retry opportunity.
- Keep admission fail-closed while any unmanaged path remains and retain the exact failure event.

## 禁止临时方案

- Do not delete or terminate the live Tooling15 producer, bypass `require_clean()`, or exempt a path
  by prefix.
- Do not clear a failed reservation, weaken filesystem-identity comparison, or retry the same path
  twice in one cleanup call.
- Do not increase the global cleanup batch without preserving bounded work and deterministic order.

## 修复结果与回传

The two focused regressions first failed: the independent candidate remained present and the oldest
failed reservation remained first. After the repair they passed `2/2`. The complete artifact
governance suite passed `28/28` in 125.457 seconds; the pre-change baseline was `26/26` in 141.853
seconds. `py_compile` and scoped `git diff --check` also passed.

Keep this failure open until the scoped commit is loaded by a healthy successor and an official
cleanup attempt proves that a locked reservation remains protected while an independent eligible
candidate advances. Tooling15 must separately register or release its bootstrap root before the
artifact audit can become fully clean.
