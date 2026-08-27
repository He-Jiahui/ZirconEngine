---
handoff_kind: fixed
status: fixed
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
  - python -B -m unittest tools.session_coordinator.tests.test_artifact_governance.ArtifactGovernanceTests.test_require_clean_recovers_missing_artifact_reservations_online tools.session_coordinator.tests.test_artifact_governance.ArtifactGovernanceTests.test_require_clean_omits_recovered_reservations_from_rejection tools.session_coordinator.tests.test_artifact_governance.ArtifactGovernanceTests.test_require_clean_preserves_existing_artifact_reservation -v
  - python -B -m unittest tools.session_coordinator.tests.test_artifact_governance -v
resolved_at: 2026-08-26
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

- 根因：Artifact cleanup counted retryable reservation failures against the deletion budget, retained their oldest ordering timestamp, and retried the same path twice in one call, allowing one locked producer to starve independent candidates.
- 架构修复：Count only successful recovered deletions, exclude already-attempted paths from the current scan, and rotate failed artifact reservations by refreshing reserved_at while preserving filesystem identity and fail-closed admission.
- 验证：Focused RED/GREEN passed 2/2; full artifact-governance suite passed 28/28 inside maintenance finalizer 369856fcfa654b38888690a4d5d6dd86; commit e82381c81813c6d1947218fe788056e7994dccfc loaded by rollover 11c68c9382424021aa799c7e0442db42 on healthy schema68 successor 5db6f88e3cf540b6ba7f4c10ec5b6fbb. Official cleanup 69dfae39d31140f79fd5661c0a3344b9 preserved locked tooling15-wave101 and deleted independent mvp-resource-management-comparisons. Four stale mvp-test-fixtures reservations are absent.
- 回传：Artifact cleanup now preserves locked producers without allowing them to monopolize bounded cleanup progress; startup recovery has cleared the four missing fixture reservations.

### 2026-08-27 live admission continuation

The startup recovery above did not cover a reservation that became stale while the same daemon
continued serving admissions. A later fixture or product acquire could still observe a missing
`artifact` reservation until a service restart, and an unrelated unmanaged-artifact rejection
continued to project that stale row in `cleanupReservations`.

`require_clean()` now serializes with artifact cleanup and performs a missing-only recovery before
its admission scan. It completes only reservations whose governed target returns an explicit
`FileNotFoundError`; existing directories, reparse points, outside-root paths, and other inspection
errors remain fail-closed and are never deleted by admission. The existing durable terminal/event
path removes the stale row before the subsequent fixture, product-staging, validation-copy, or
managed Cargo overlap check runs.

The focused RED reproduced four missing `mvp-test-fixtures-{11376,29760,10976,16996}` parent rows:
two recovery assertions failed while the live-directory preservation case passed. After the repair,
the focused group passed 3/3 and the complete artifact-governance suite passed 31/31 in 104.092
seconds. This continuation adds no exemption for the two current Tooling15 directories that still
physically exist; those producers remain correctly blocked until their managed lifecycle completes.
