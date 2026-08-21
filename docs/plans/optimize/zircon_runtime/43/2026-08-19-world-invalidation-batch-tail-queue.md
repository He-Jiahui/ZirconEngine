# Runtime43 World Invalidation Batch Tail-Queue Optimization Record

- Date: 2026-08-19
- Owner: `runtime43-world-invalidation-tail-queues-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md`, DYN-P1-057 / DYN-GATE-038
- Status: implementation and 21-pair measurement repair complete; combined managed validation pending

## Problem

Every committed invalidation batch used `Vec::remove(0)`. Draining N batches
therefore moved `N * (N - 1) / 2` trailing batch records while holding the
dynamic session owner lock.

## Change

- A newly sealed pending page reverses its batch vector once.
- Page construction iterates the internal vector in reverse, preserving the
  original generation and delivery order at the ABI boundary.
- Commit consumes completed batches with O(1) `pop()` from the vector tail.
- Rollback retains the same internal representation and cached page, so the
  existing two-phase allocation contract is unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Commit 20,000 one-item batches | 199,990,000 batch moves | 0 batch moves after one reversal | 100% |
| Batch commit | O(remaining batches) | O(1) | one complexity class |
| Full drain | O(N^2) moves | O(N) reverse + O(N) pops | one complexity class |

## Acceptance

- `world_invalidation_tail_queues_preserve_batch_and_item_order` proves the
  public page retains original batch order.
- `world_invalidation_tail_queue_source_has_no_front_removal` rejects any
  restored `remove(0)` or prefix drain in the commit owner.
- `world_invalidation_tail_queue_release_benchmark_evidence` emits a dedicated
  21-pair alternating distribution for legacy batch `remove(0)` versus the
  reversed batch vector's `pop()` path. Item commit has a separate distribution.
- Timing gate: tail-queue P95 must be no more than 25% of legacy P95.
- Exact-file Rustfmt and scoped diff checks: passed.
- Cargo regression and release P50/P95: pending a combined Windows coordinator
  batch; no per-task Cargo command is used.

## Remaining Scope

The page DTO still lacks remaining/backlog, cursor, dropped, wake, and resync
receipts. Encoding remains synchronous under the session owner lock. This
record closes only batch-removal complexity and does not close DYN-P1-057 or
DYN-GATE-038 end to end.
