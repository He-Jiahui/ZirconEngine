---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-26
summary_slug: failure-lifecycle-history-erasure
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/tests/test_failures.py
  - tools/session_coordinator/tests/test_database.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_failures.FailureGraphTests.test_verified_fix_moves_back_and_updates_both_relative_links tools.session_coordinator.tests.test_database.DatabaseTests.test_schema_68_backfills_immutable_failure_lifecycle_history -v
  - python -B -m unittest tools.session_coordinator.tests.test_failures tools.session_coordinator.tests.test_database -v
resolved_at: 2026-08-26
---

# Coordinator01: Failure graph import erases lifecycle history

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：failure dependency-graph inventory and canonical return review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns both the replaceable current Failure graph and the durable evidence needed to distinguish a newly added failure from a returned fixed lifecycle.

## 失败现象与复现证据

`FailureGraphService.import_repository()` deletes and rebuilds `failure_nodes` from the
current Markdown snapshot. After `failure return` removes the fixing-side `failure-*`
artifact and creates the origin-side `fixed-*` artifact, the next import retains only
the fixed row. The durable database can no longer prove when or where the failure was
originally added.

This prevents the control plane from distinguishing current graph state from immutable
lifecycle history. It also makes cycle cleanup evidence depend on deleted worktree files
instead of durable added/fixed facts.

## 最低共享层根因

`failure_nodes` is intentionally a replaceable projection, but it was also the only
persistent representation of a Failure lifecycle. No append-only table existed behind
the graph import or the low-latency local validation-failure index.

## 架构修复验收

- Persist one immutable `added` event for every lifecycle key and one immutable `fixed`
  event after a canonical return.
- Backfill schema-67 databases from the current failure/fixed projection without
  inventing a second lifecycle on repeated migration or import.
- Keep `failure_nodes` replaceable and keep graph diagnostics based on current state;
  historical events must not re-open returned failures or suppress current cycles.
- Reject update/delete of lifecycle events at the database boundary.

## 禁止临时方案

- Do not retain returned `failure-*` files solely for history.
- Do not stop replacing `failure_nodes`, infer history from Git at query time, or allow
  callers to rewrite event rows.
- Do not claim this persistence layer resolves existing ownership cycles; product owners
  still return those failures individually.

## 修复结果与回传

- 根因：The replaceable failure_nodes projection was also the only durable lifecycle record, so importing a returned fixed artifact erased the original added path and timestamp.
- 架构修复：Schema 68 adds append-only added/fixed lifecycle events, backfills schema-67 projections, records graph imports and low-latency local failures, and rejects update/delete at the SQLite boundary.
- 验证：Focused lifecycle and migration tests passed 2/2; full test_failures plus test_database passed 56/56 in 92.057 seconds; py_compile and scoped diff-check passed; implementation commit b6775b43a53f4878e024f257cab14dc58bd0b769.
- 回传：Failure history now survives current-graph replacement without re-opening fixed nodes or weakening current cycle diagnostics.
