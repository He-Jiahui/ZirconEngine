---
handoff_kind: fixed
status: fixed
created_at: 2026-08-31
summary_slug: cleanup-duplicate-target-deletion-evidence
origin_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/06
fixing_child_dir: docs/plans/optimize/zircon_tooling/06
failure_scope: local
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/tests/test_cleanup.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_cleanup -v
resolved_at: 2026-09-01
---

# Tooling06: preserve per-attempt cleanup evidence for duplicate targets

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 来源执行切片：cleanup plan application and durable deletion evidence
- 修复责任计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 交接原因：The Coordinator cleanup executor owns per-attempt deletion state and durable evidence; downstream cleanup consumers cannot repair a misclassified event after it is committed.

## 失败现象与复现证据

A persisted cleanup plan can contain the same target more than once. The first candidate deletes
the target; the second correctly observes `target_missing`. The pre-fix finalization logic derives
each attempt's durable result from membership in the plan-wide `deleted` list, so both
`cleanup.target_deletion_completed` records and both lane events report `deleted` even though the
second attempt retained the already-missing target.

An in-memory run of the HEAD implementation reproduced the mismatch exactly:
`denied=['target_missing']`, completion results `['deleted', 'deleted']`, expected
`['deleted', 'retained']`.

## 最低共享层根因

`CleanupService.apply` uses aggregate plan state to classify a single candidate attempt. Once any
attempt appends a target to `deleted`, every later duplicate candidate is treated as deleted. The
cleanup result tuple remains superficially correct, but durable evidence no longer describes what
the individual deletion attempt observed or performed.

## 架构修复验收

- Track deletion success as per-candidate state set only after that candidate's `rmtree` succeeds.
- Use that state consistently for cargo-job cleanup status, lane event type, and
  `cleanup.target_deletion_completed.result`.
- Preserve the successful first deletion and the second `target_missing` denial.
- Release the cargo cleanup reservation after both non-exceptional attempts and leave the persisted
  cleanup plan in `applied` state.
- Prove the exact duplicate persisted candidate sequence emits `deleted` then `retained`, with the
  second evidence snapshot recording `before.target_exists=false`.

## 禁止临时方案

- Do not deduplicate or rewrite persisted candidates during application; historical plan input must
  remain auditable.
- Do not suppress the second attempt or its `target_missing` denial.
- Do not infer per-attempt outcome from a plan-wide collection.
- Do not weaken cleanup reservations, validation-copy overlap protection, or unexpected-error
  recovery behavior.

## 修复结果与回传

- 根因：CleanupService.apply classified each candidate attempt from membership in the plan-wide deleted collection, so a later duplicate target inherited the first deletion result even after observing target_missing.
- 架构修复：Track target_deleted as per-candidate state only after that candidate's rmtree succeeds, then use the same state for cargo cleanup status, lane events, and cleanup.target_deletion_completed while preserving the second retained attempt and releasing its reservation.
- 验证：Coordinator validation ticket 2aab9ff49eb34c62a3e340c9053121f8 passed all 42 tools.session_coordinator.tests.test_cleanup tests in 340.807 seconds against source manifest b5964554ba16816bfb96daa19aced9a94c09b019613d61f3a7e0d0043deef091.
- 回传：Duplicate persisted cleanup candidates now keep independent durable outcomes: the first deletion is deleted, the later missing-target attempt is retained with before.target_exists=false, lane events stay ordered, and reservations are released.
