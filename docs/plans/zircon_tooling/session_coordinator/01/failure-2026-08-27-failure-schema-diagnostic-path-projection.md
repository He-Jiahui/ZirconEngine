---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-27
summary_slug: failure-schema-diagnostic-path-projection
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/tests/test_failures.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_failures.FailureGraphTests.test_invalid_artifact_status_is_diagnostic_not_graph_import_failure tools.session_coordinator.tests.test_failures.FailureGraphTests.test_schema_diagnostics_persist_the_exact_plan_path -v
  - python -B -m unittest tools.session_coordinator.tests.test_failures -v
---

# Coordinator01: failure schema diagnostics omit captured paths

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：schema-68 failure audit actionable-diagnostic review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns immutable plan snapshots, schema validation,
  durable graph diagnostics, and the audit projection used by plan owners.

## 失败现象与复现证据

Schema-68 production audit contains 165 `schema_validation` diagnostics and
reports `paths: []` for all 165. Messages identify either a malformed failure
artifact or a main plan whose relative failure link is stale, but consumers must
parse human text to locate the affected captured file.

The RED suite covers both identities. One invalid failure status must project
the exact failure artifact path; one deliberately removed fixing-plan link must
project the exact fixing plan path. Both imported and persisted diagnostics are
present, but current assertions fail because every path tuple is empty.

## 最低共享层根因

`_parse_immutable_snapshot()` captures every Markdown file below `docs/plans`
and validates only that temporary copy, but returns only the failure-artifact
manifest. `prepare_import_snapshot()` then constructs every schema diagnostic as
`GraphDiagnostic("schema_validation", error)` and discards the captured file
identity even though the validator message starts with that exact snapshot path.

## 架构修复验收

- The immutable snapshot returns a deterministic index of all captured Markdown
  paths without changing the failure-artifact CAS manifest.
- A schema diagnostic is bound only when its message begins with one exact
  captured path followed by `:`; both failure artifact and main plan paths work.
- The structured path survives diagnostic persistence and audit readback.
- Non-prefix mentions and paths absent from the captured snapshot remain empty.
- Focused RED becomes GREEN, the full failure service suite passes, and a
  production import projects paths for current schema diagnostics.

## 禁止临时方案

- Do not scan arbitrary message substrings, accept filesystem paths outside the
  immutable snapshot, or read live files after validation.
- Do not redefine the artifact CAS manifest, suppress schema errors, or rewrite
  foreign failure ownership to reduce the diagnostic count.
- Do not special-case the current 165 messages or their plan names.

## 修复结果与回传

Open state: `implementation GREEN / production successor import pending`.
The immutable snapshot now retains its all-Markdown manifest separately from the
failure-artifact CAS manifest. Schema diagnostics bind only an exact captured
path prefix and preserve that path through database readback. Focused
regressions pass `4/4`, the complete failure service suite passes `31/31`, and
the session-registration slow-parse concurrency regression passes `1/1`.
Python compile, the focused handoff validator, and scoped diff-check also pass.
A successor loaded from the committed source and one production import remain
required before the canonical fixed return. This repair does not claim any
foreign schema diagnostic is resolved.
