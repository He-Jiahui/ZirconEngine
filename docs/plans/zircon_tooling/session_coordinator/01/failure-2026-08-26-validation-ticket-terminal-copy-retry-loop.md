---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-26
summary_slug: validation-ticket-terminal-copy-retry-loop
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/tests/test_workspace_copy.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_workspace_copy.WorkspaceCopyTests.test_removed_failed_cargo_copy_projects_materialization_kind tools.session_coordinator.tests.test_workspace_copy.WorkspaceCopyTests.test_ticket_worker_terminalizes_removed_failed_cargo_copy_without_retry -v
---

# validation-ticket-terminal-copy-retry-loop: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：durable FIFO validation ticket recovery after terminal copy cleanup
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the workspace-copy status projection consumed by the validation ticket worker.

## 失败现象与复现证据

Ticket `fa393d064db44d0881175c36a5f2f04d` was claimed at
`2026-08-25T16:05:01Z`. Its Cargo closure failed with
`validation_copy_compile_time_resource_missing`; the durable copy rows preserve the
exact `sourcePath` and `resourcePath`. After each failed copy was cleaned to
`status=removed`, the worker linked a new copy instead of terminalizing the ticket.
At least eleven distinct copy IDs were created, and the global FIFO remained blocked
behind the same `materializing` ticket.

## 最低共享层根因

`validation_ticket_worker.py` already distinguishes a removed Cargo copy from an old
pre-fix wrapper by reading `record.materialization_kind`. Fake worker records expose
that field and the existing regression therefore passes. Production
`WorkspaceCopyService._record_from_row()` did not project the durable
`validation_copies.materialization_kind` column into `WorkspaceCopyRecord`, so
`getattr(..., None)` always selected the legacy rematerialization branch.

## 架构修复验收

- Project the durable materialization kind through `WorkspaceCopyRecord`, including
  async Cargo acknowledgements, status reads and serialized status evidence.
- Preserve `removed` as the filesystem lifecycle status and preserve the typed
  materialization failure fields; do not manufacture a run result.
- A real `ValidationTicketWorker` consuming a removed failed Cargo copy must
  terminalize the ticket once, retain one copy link and create no replacement copy.
- After a committed and loaded fix, the existing FIFO must advance naturally. Do not
  reorder or point-select tickets.

## 禁止临时方案

- Do not update production ticket or copy rows manually.
- Do not delete queued tickets, skip the FIFO head or submit a replacement ticket.
- Do not weaken compile-time resource validation or treat the product resource error
  as a successful Cargo run.

## 修复结果与回传

RED reproduced the missing production field as `AttributeError` in the durable
removed-copy status path. The lower-layer fix adds the optional record field and
projects it from the durable row and Cargo async acknowledgements. Focused GREEN is
`2/2`; managed validation, scoped commit, successor reload and lifecycle return are
pending.
