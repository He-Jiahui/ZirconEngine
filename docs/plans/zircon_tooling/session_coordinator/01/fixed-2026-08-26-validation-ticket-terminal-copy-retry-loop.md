---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-26
resolved_at: 2026-08-26
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
The ticket accumulated 43 copy links before a timing window exposed its failed state;
the next FIFO ticket accumulated 175 links under the same retry loop.

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

- 根因：`WorkspaceCopyService._record_from_row()` omitted the durable Cargo
  `materialization_kind`, so the validation ticket worker misclassified a removed
  failed Cargo copy as a legacy wrapper and rematerialized it indefinitely.
- 架构修复：The workspace-copy record and async acknowledgement now preserve the
  durable materialization kind while retaining the typed copy failure and removed
  filesystem lifecycle status.
- 验证：Focused tests passed `2/2`; workspace-copy `75/75`, validation-ticket
  `29/29` and terminal-status `8/8` suites passed; production FIFO ticket
  `3da70e951586482ebee8ac807a507e85` terminalized without a successor copy.
- 回传：Implementation commit `3282dfad2a3a0dce246dfa8f300d7d30d70ed9a9`
  was loaded by healthy schema-67 successor `ad937947ed424268b598c4aaffcdf10b`.

RED reproduced the missing production field as `AttributeError` in the durable
removed-copy status path. The lower-layer fix adds the optional record field and
projects it from the durable row and Cargo async acknowledgements. Focused tests
passed `2/2`; complete workspace-copy tests passed `75/75`; validation-ticket tests
passed `29/29`; terminal-status tests passed `8/8`; `py_compile` and `diff --check`
passed. Independent review reported Critical 0, Important 0, Minor 1 and Ready. The
non-blocking minor is that ticket terminal evidence remains generic while the linked
copy row retains the exact typed materialization details.

Maintenance finalizer request `ca05e528aebb46e3967fce517894350f` committed the
exact three paths as `3282dfad2a3a0dce246dfa8f300d7d30d70ed9a9`. Controlled
rollover action `453a42d960f44f7e80a3ea25077a862b`, intent
`5e2e8e288d00456e8fa7a0350ea7b093`, loaded healthy read-write schema-67 successor
`ad937947ed424268b598c4aaffcdf10b`. The unrelated 19-path staged projection retained
fingerprint `55b653e4d4d32e130c08024c68f7b78f0e81a925b90afeb10f8833ca75ec9ae2`.

Production FIFO proof used the already-active ticket
`3da70e951586482ebee8ac807a507e85`. Its final old-daemon copy
`2b4944c5fff14ac88fa58120d6a86623` was cleaned to `removed` at
`2026-08-25T21:36:15Z` with the typed resource failure intact. The successor
terminalized the ticket at `2026-08-25T21:37:56Z`; its copy-link count remained 175
and no successor link was created. No ticket row, copy row or FIFO order was manually
changed.
