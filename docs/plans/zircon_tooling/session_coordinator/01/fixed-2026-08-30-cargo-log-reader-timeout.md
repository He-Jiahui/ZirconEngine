---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: cargo-log-reader-timeout
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/tests/test_cargo_runner.py
tests:
  - python -m unittest -v tools.session_coordinator.tests.test_cargo_runner.CargoRunnerSourceRootTests.test_collector_bounds_log_reader_join_and_records_timeout
resolved_at: 2026-08-30
---

# Coordinator01: Cargo log reader timeout

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P1-018` terminal collection review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the managed Cargo collector and terminal
  evidence lifecycle.

## 失败现象与复现证据

`CargoJobRunner._finish` called `reader.join()` without a timeout. If an
unmanaged descendant inherited stdout/stderr, the root process could exit while
the pipe remained open forever, blocking terminalization and retaining a Cargo
lane indefinitely.

## 最低共享层根因

The collector bounded log memory but treated EOF as an unconditional lifecycle
barrier. It had no finite reader join budget or durable diagnostic for a reader
that outlived the managed root.

## 架构修复验收

- reader joins use an explicit five-second bound;
- readers still alive after the bound are reported as
  `cargo_run_log_reader_timeout` and owned streams are closed to unblock local
  I/O;
- existing read/write failure reporting, Job Object termination and terminal
  Cargo transitions remain unchanged.

## 禁止临时方案

- Do not wait indefinitely for pipe EOF or increase the join timeout without a
  contract.
- Do not kill an unowned descendant solely because its inherited pipe remains
  open.

## 修复结果与回传

- 根因：Cargo collector joined stdout/stderr readers forever after root exit, allowing inherited pipes to retain a lane.
- 架构修复：Bound reader joins to five seconds, close owned streams after timeout, and persist cargo_run_log_reader_timeout while preserving existing terminal and Job Object handling.
- 验证：RED covered unconditional join; GREEN full Cargo runner mock suite 11/11, py_compile and scoped diff check passed; no Cargo launched.
- 回传：Returned bounded Cargo log reader lifecycle to Coordinator01.
