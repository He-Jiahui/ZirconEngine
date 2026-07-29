---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: cpu-reservation-ledger-consume-fifo-divergence
origin_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/runtime/11
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/tests/test_cargo_reservations.py
tests:
  - one pending warm CPU reservation that is ledger FIFO head consumes into exactly one leased job
  - cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1
---

# Coordinator01: CPU reservation ledger and consume FIFO divergence

## 来源执行者

- 来源计划: `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 来源执行切片: Runtime11 bounded operation service current-source compile validation
- 修复责任计划: `docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因: CPU reservation persistence, FIFO selection, consume admission, and job binding are Coordinator01 control-plane responsibilities. Runtime11 cannot alter foreign reservations or bypass the managed lane.
- 生命周期键: `cpu-reservation-ledger-consume-fifo-divergence`

## 失败现象与复现证据

Runtime11 created pending warm CPU reservation `a2a7574c4c41465fb4efdff7ea5cd517` for `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1`. The coordinator reported no active Cargo jobs, yet repeated `cargo consume-cpu-reservation` calls returned `cargo_cpu_reservation_not_fifo_head` before creating a job.

The coordinator's durable reservation ledger, queried read-only with the same warm-lane predicate and order used by `CargoJobService._require_lane_reservation` (`priority_rank, created_at, reservation_id`, status in `pending/leased/running/finished`), listed this reservation as the first and only earliest eligible row. The following Shader06, Text09, Runtime08, and Editor03 rows were later. Thus the ledger's FIFO head and the consume admission decision disagree for the same reservation without any live cargo/rustc process.

## 最低共享层根因

The control plane has more than one effective view of warm CPU FIFO eligibility, or mutates/reconciles one view before consume without exposing the resulting head. The exact lower branch between durable ledger selection, reservation reconciliation, supervision proof guard, and job binding remains for Coordinator01 to diagnose. This is distinct from an external owner legitimately holding a prior reservation.

## 架构修复验收

- One canonical FIFO candidate projection is used for status, consume admission, reconciliation, and any supervision guard; a row reported as the durable warm-lane head can be consumed by its owner into one leased, unstarted job.
- Focused Coordinator regression coverage constructs the Runtime11-shaped first pending warm reservation with later pending rows and proves the first reservation binds, later rows remain pending, and no duplicate job is created.
- Reconciliation of expired/finished/leased rows is auditable and cannot make consume reject its own reported head without a typed, actionable reason and the actual predecessor identity.
- After the Coordinator01 fixed return, Runtime11 consumes the same reservation only if its immutable source binding remains valid, otherwise obtains a fresh row; then reruns the declared managed lib check.

## 禁止临时方案

- Do not release, reorder, expire, or edit another session's reservation from Runtime11.
- Do not add a normal Cargo bypass, ignore FIFO, convert the rejection into a false test result, or hide the divergence by expanding retry loops/timeouts.
- Do not repair only a status display while leaving consume admission and durable job binding inconsistent.

## 修复结果与回传

Open state: `待修复`; Runtime11 keeps reservation `a2a7574c4c41465fb4efdff7ea5cd517` pending and does not claim a Cargo result until Coordinator01 returns a verified FIFO admission fix.

2026-07-29 current-source repair: Coordinator01 now owns CPU FIFO eligibility in
`cargo_reservations.py`. Stale/TTL/terminal reconciliation, legacy repeat-priority
yield, warm-lane head selection, warm predecessor selection, and Cargo bind selection
share the canonical active-status and `(priority_rank, created_at, reservation_id)`
projection. Every reconciliation CAS writes one row-level audit event and replay does
not duplicate it. Proof bootstrap accepts only `warm`, freezes `executionMode` in the
immutable binding, and permits an exact consumed retry only when the original leased
job id still matches. A `not_fifo_head` error now includes the actual predecessor's
reservation, Session, status, job, execution mode, priority, and creation identity.

TDD evidence first reproduced all four defects: burst proof acceptance, proof retry
rejection after the first bind, missing execution-mode identity, and absent
reconciliation events. The repaired current source then passed:

- focused proof/FIFO/reconciliation set: `4/4`;
- `tools.session_coordinator.tests.test_cargo_reservations`: `48/48` in `201.362s`;
- `tools.session_coordinator.tests.test_supervision_actions`: `43/43` in `197.448s`;
- focused `test_server` proof-handoff wiring: `4/4` in `22.233s`;
- predecessor-detail RED/green follow-up: `2/2` in `10.869s`;
- exact six Python source/test paths: `py_compile` and `git diff --check` GREEN;
- exact six source/test fingerprint: `d07ca14ba5ca3165a32274549d924b133a6deae04eacbf4ffbdaa101c899621a`.

The original Runtime11 reservation `a2a7574c4c41465fb4efdff7ea5cd517`
had no source manifest and is now expired; Runtime08 reservation
`04ae7a9e103e4582910137fa29c019f9` is also expired. Neither id is reusable.
This handoff remains `open` until exact-scope independent re-review, managed atomic
Coordinator commit, service reload, and the fixed return complete. Runtime11 and
Runtime08 must create fresh source-manifest-bound rows afterward without reordering
the still-valid FIFO.
