# Runtime41 Ready Apply Index Optimization Record

- Date: 2026-08-19
- Owner: `runtime41-operation-indexed-queues-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md`, P1-12
- Status: implementation and independent 21-pair measurement repair complete; combined managed validation pending

## Problem

`take_prepared_task` searched the complete operation `HashMap` for every owner
apply. Ready selection was coupled to randomized map iteration, and repeated
claims added O(total tasks) work to the owner-frame path.

## Change

- Successful prepare completion appends its handle to a dedicated ready FIFO.
- Owner apply pops the ready FIFO and revalidates phase, prepared payload,
  deadline, and claim state against the canonical task table.
- Cancelled, expired, failed, or removed entries are discarded once without
  scanning unrelated tasks.
- Capacity-triggered compaction bounds stale ready handles when terminal state
  is reclaimed before the next owner apply.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Drain 1,024 ready tasks, worst-case candidate probes | 524,800 | 1,024 | 99.805% |
| Selection per live ready task | O(total tasks) | amortized O(1) | one complexity class |
| Owner apply limit | 8 unchanged | 8 unchanged | scheduling policy preserved |

## Acceptance

- `operation_service_dispatches_phase_work_in_submission_order` validates the
  current one-in-flight prepare policy reaches owner apply in submission order.
- Existing deadline, cancel, panic containment, retained-byte, and harvest
  regressions remain in the same `operation` test batch.
- `operation_service_phase_index_release_benchmark_evidence` emits 21 paired,
  alternating release samples for this ready-apply workload's legacy scan and
  indexed pop algorithms. Its result owns
  `evidence_id=operation_ready_apply_index` and `samples_shared=false`; the
  queued-snapshot workload runs a separate timing distribution.
- Timing gate: indexed P95 must be no more than 25% of legacy P95.
- Exact-file Rustfmt and scoped diff checks: passed.
- Cargo regression and release P50/P95: pending the combined Windows coordinator
  batch; no per-task Cargo command is used.

## Remaining Scope

Completion arrival order is now explicit rather than `HashMap`-random, but
resource conflict keys, ordering groups, generation-conflict checks, and
replay-stable apply scheduling remain open. This record does not claim full
P1-12 closure.
