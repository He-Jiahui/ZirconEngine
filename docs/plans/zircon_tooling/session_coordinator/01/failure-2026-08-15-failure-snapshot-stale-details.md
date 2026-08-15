---
handoff_kind: failure
status: open
created_at: 2026-08-15
summary_slug: failure-snapshot-stale-details
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/failure_snapshot_drift.py
  - tools/session_coordinator/tests/test_failures.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_failures -v
  - python -m unittest tools.session_coordinator.tests.test_server.ServerTests.test_registration_snapshot_parse_does_not_hold_the_database_writer tools.session_coordinator.tests.test_server.ServerTests.test_database_busy_diagnostic_does_not_terminate_maintenance_loop -v
---

# Coordinator01: stale failure snapshots omit the changed artifacts

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：current-source failure graph refresh after Coordinator schema cleanup
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns immutable failure snapshot preparation, stale comparison, and durable import diagnostics.

## 失败现象与复现证据

Durable request `71cbeec9bb8a4f3c9c2b6fa0361c481b` correctly rejected a
concurrent import with `failure_snapshot_stale`, but its error details were `{}`.
With more than 600 handoff artifacts and continuous owner activity, the caller
cannot identify which path was added, removed, or modified and cannot decide
whether a stable retry window exists.

## 最低共享层根因

`FailureGraphService.import_prepared_snapshot` compares two complete manifest
tuples and raises a generic error on inequality. It discards the already
available repo-relative paths and SHA-256 identities instead of projecting a
bounded deterministic manifest diff.

## 架构修复验收

- Report every changed artifact as `added`, `removed`, or `modified` with the
  exact repository-relative path and available expected/current SHA-256 values.
- Sort changes deterministically and cap the projected list while preserving
  exact total and category counts plus an explicit truncation bit.
- Keep the stale comparison and database replacement fail-closed; diagnostics
  must not weaken the immutable snapshot CAS or hold a writer while parsing.
- Cover modified, added, removed, deterministic ordering, and truncation in
  focused Python regressions.

## 禁止临时方案

- Do not retry imports blindly, increase timeouts, suppress stale detection, or
  publish a partially current graph.
- Do not expose absolute paths or unbounded manifest data in command responses.
- Do not move repository parsing back into the SQLite write transaction.

## 修复结果与回传

Implemented a dedicated manifest-drift projector and connected it to the
fail-closed prepared-snapshot comparison. Durable error details now carry exact
artifact/category counts, a deterministic bounded change list, the available
expected/current SHA-256 identities, and an explicit truncation bit without
exposing absolute paths.

Validation evidence:

- RED: the focused stale-import tests failed because production returned `{}`;
  the bounded helper test initially failed to import because the projector did
  not exist.
- GREEN: all 25 `test_failures` cases pass, including modified, added, removed,
  stable ordering, and truncation coverage.
- The existing session-registration writer-boundary and maintenance DB-busy
  loop regressions both pass, proving the diagnostics did not move parsing into
  a write transaction or weaken recovery behavior.
