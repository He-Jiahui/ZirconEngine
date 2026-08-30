---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: failure-return-source-schema-guard
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/tests/test_failures.py
tests:
  - python -u -B -m unittest tools.session_coordinator.tests.test_failures.FailureGraphTests.test_return_rejects_source_schema_errors_without_moving_failure -v
  - python -u -B -m unittest tools.session_coordinator.tests.test_failures.FailureGraphTests.test_verified_fix_moves_back_and_updates_both_relative_links -v
resolved_at: 2026-08-30
---

# Coordinator01: failure return accepted a malformed source artifact

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：failure return lifecycle validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns failure import and return; the source artifact must be canonical before a fixed record is published.

## 失败现象与复现证据

`FailureGraphService.return_fixed` previously called `_fixed_content` and moved the source
without checking source-only validator diagnostics. A failure containing English section
headings could therefore become a `fixed-*` artifact while the repository handoff validator
reported missing canonical headings and source executor fields. The RED regression expected
`invalid_handoff`, but the old implementation moved the malformed source successfully.

## 最低共享层根因

Return validated the resolution fields and result section, but not the source artifact's
canonical handoff schema. Import preserves schema diagnostics for graph visibility, so return
must fail closed on diagnostics attributed to the source path instead of publishing a malformed
fixed artifact.

## 架构修复验收

- Return rejects source-scoped handoff validator errors with `invalid_handoff`.
- A rejected return leaves the open source artifact and destination tree unchanged.
- A canonical source still supports both ordinary and child-record-only returns.
- Diagnostics belonging to unrelated plans do not prevent a valid source return.

## 禁止临时方案

- Do not rewrite malformed source headings during return or silently discard schema errors.
- Do not validate only the result section, disable the repository validator, or add a per-summary allowlist.
- Do not delete or overwrite an open artifact after a rejected return.

## 修复结果与回传

- 根因：failure return did not validate source-scoped canonical handoff diagnostics before moving the artifact
- 架构修复：return_fixed now fails closed on validator errors attributed to the source and preserves the open artifact on rejection
- 验证：RED reproduced a malformed source being moved; GREEN source-schema rejection, ordinary return, child-record-only return, py_compile, and diff checks passed
- 回传：Coordinator failure return now publishes fixed artifacts only from canonical source handoffs.

### Isolated source-validation follow-up

The source-schema preflight now validates an immutable temporary snapshot containing only the
source artifact and its origin/fixing plan files. This removes the redundant full-repository
scan before mutation while retaining the canonical full import after an accepted return.
Focused regressions proved the validator receives the isolated root, malformed sources still
fail closed, and canonical return behavior remains unchanged.
