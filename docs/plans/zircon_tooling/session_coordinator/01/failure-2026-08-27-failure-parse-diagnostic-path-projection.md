---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-27
summary_slug: failure-parse-diagnostic-path-projection
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/tests/test_failures.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_failures.FailureGraphTests.test_parse_diagnostics_persist_the_exact_artifact_path -v
  - python -B -m unittest tools.session_coordinator.tests.test_failures -v
---

# Coordinator01: failure parse diagnostics omit the artifact path

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：schema-68 failure dependency graph inventory refresh
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns immutable failure parsing, durable graph diagnostics,
  and the control-plane audit projection consumed by failure owners.

## 失败现象与复现证据

Durable `failure.audit` reports seven `parse_error` diagnostics for three exact
failure artifacts, but every diagnostic projects `paths: []`. The artifact path
is present only as a prefix inside the human-readable message, so control-plane
consumers cannot filter or navigate the broken artifact structurally.

The focused RED creates one malformed failure artifact by removing its
`created_at` and `summary_slug`. Both errors are imported and persisted, but the
test fails because their `GraphDiagnostic.paths` values are empty instead of the
same exact repository-relative artifact path.

## 最低共享层根因

`FailureGraphService.prepare_import_snapshot()` receives the immutable artifact
manifest together with validator parse errors, but constructs each diagnostic as
`GraphDiagnostic("parse_error", error)` and discards the artifact identity. The
validator currently exposes error text rather than a structured error object, so
the Coordinator must bind only an exact manifest path prefix rather than guess a
path from arbitrary message content.

## 架构修复验收

- Every parse diagnostic whose message starts with an exact immutable manifest
  path followed by `:` carries that one path in `GraphDiagnostic.paths`.
- The exact path survives the database write and `failure.audit` readback.
- A colon-rich message that does not start with a captured artifact path cannot
  fabricate a path, and no filesystem state outside the immutable snapshot is
  consulted.
- The focused RED becomes GREEN, the complete failure service suite passes, and
  a production import projects exact paths for the current malformed artifacts.

## 禁止临时方案

- Do not parse a Windows drive prefix, scan arbitrary message substrings, or
  accept a path absent from the immutable artifact manifest.
- Do not suppress malformed artifacts, rewrite their foreign ownership, or
  weaken failure schema validation to make the diagnostics disappear.
- Do not change the diagnostic message contract merely to satisfy the test.

## 修复结果与回传

Open state: `implementation GREEN / production successor import pending`.
`FailureGraphService` now binds a parse diagnostic only when its message starts
with an exact path from the same immutable artifact manifest. The focused
regressions pass `2/2`, the complete failure service suite passes `30/30`, and
the database readback test proves the exact path survives persistence. Python
compile and the focused handoff validator also pass. A successor loaded from the
committed source and one production import/audit projection remain required
before the canonical fixed return. No foreign failure artifact is modified or
claimed fixed by this Coordinator repair.
