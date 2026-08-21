# Runtime41 Queued Snapshot Index Optimization Record

- Date: 2026-08-19
- Owner: `runtime41-operation-indexed-queues-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md`, P1-11
- Status: implementation and independent 21-pair measurement repair complete; combined managed validation pending

## Problem

`take_queued_snapshot_task` searched the complete operation `HashMap` for every
owner snapshot claim. Selection order depended on the map seed, and draining N
queued tasks could inspect `N * (N + 1) / 2` task entries.

## Change

- Admission appends the monotonic operation handle to a dedicated FIFO.
- Snapshot selection pops the FIFO and performs one authoritative task-table
  lookup, skipping cancelled, expired, or otherwise stale handles once.
- A live unarmed FIFO head remains at the front and blocks later claims until
  asynchronous deadline arming completes, preserving strict submission order.
  Stale handles are removed without rotating the live head.
- Admission compacts stale phase handles at the configured task-capacity
  boundary, keeping index residency O(max tasks) even when queued tombstones
  are repeatedly evicted without a runtime tick.
- The task `HashMap` remains the canonical state and accounting owner.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Drain 1,024 queued tasks, worst-case candidate probes | 524,800 | 1,024 | 99.805% |
| Selection per live queued task | O(total tasks) | amortized O(1) | one complexity class |
| Phase-index residency | implicit in task table | O(max tasks), compacted at admission | bounded |

## Acceptance

- `operation_service_dispatches_phase_work_in_submission_order` validates 128
  snapshot claims in submission order.
- `operation_service_phase_selection_uses_indexed_queues` rejects the former
  `HashMap::iter().find_map` selection and locks the bounded FIFO ownership.
- `queued_phase_index_retains_live_admissions_until_deadline_arming_completes`
  locks the concurrent admission/owner-tick repair: unarmed live handles are
  retained at the head and block later claims, while stale or expired handles
  remain removable.
- `operation_service_does_not_bypass_an_unarmed_fifo_head` constructs the
  admission window directly and proves that a later task cannot overtake it.
- `operation_service_phase_index_release_benchmark_evidence` emits 21 paired,
  alternating release samples for this queued-snapshot workload's legacy scan
  and indexed pop algorithms. Its result owns
  `evidence_id=operation_queued_snapshot_index` and `samples_shared=false`;
  the ready-apply workload runs a separate timing distribution.
- Timing gate: indexed P95 must be no more than 25% of legacy P95.
- Exact-file Rustfmt and scoped diff checks: passed.
- Cargo regression and release P50/P95: pending the combined Windows coordinator
  batch; no per-task Cargo command is used.

## Remaining Scope

Priority classes, principal/handler quotas, stable request policy, queue-position
telemetry, cancellation propagation, and replay receipts remain open. This
record closes only deterministic bounded selection for the existing FIFO class.
