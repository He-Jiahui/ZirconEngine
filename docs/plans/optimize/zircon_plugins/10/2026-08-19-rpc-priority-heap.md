# Plugins10 RPC Priority-Heap Optimization Record

- Date: 2026-08-19
- Owner: `plugins10-rpc-heap-lazy-snapshot-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md`, NNET-P1-037
- Status: implementation complete; combined managed validation pending

## Problem

Every bounded RPC drain sorted the complete queued-invocation vector, then
removed the admitted prefix and shifted every remaining invocation. A small
frame budget therefore still paid O(N log N) scheduling plus O(N) movement for
an N-entry queue.

## Change

- Queued RPCs now live in a persistent binary max-heap.
- Heap ordering preserves higher priority first and lower sequence first for
  equal priorities, matching the established FIFO tie break.
- Draining K invocations performs K heap pops and never shifts the retained
  queue.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Drain 64 from 100,000 queued RPCs | 1 full 100,000-entry sort | 0 full sorts | 100% |
| Same drain | 99,936 retained-entry shifts | 0 retained-entry shifts | 100% |
| Scheduling complexity | O(N log N + N) | O(K log N) | budget-proportional |

## Acceptance

- Existing priority, quota, timeout, queue-full, and handler regressions remain
  in the combined RPC test batch.
- `rpc_priority_queue_preserves_fifo_order_for_equal_priorities` proves the heap
  retains the prior sequence tie break.
- `rpc_priority_heap_release_benchmark_evidence` compares 21 paired,
  alternating release samples and computes nearest-rank P50/P95; optimized P95
  must be no more than 20% of legacy P95.
- Exact-file Rustfmt, Cargo regression, and release P50/P95: pending one batched
  Windows coordinator validation with the replication payload task.

## Remaining Scope

The queue remains an in-process feature-local owner and the synchronous handler
still cannot be preempted after entry. This record closes drain scheduling
amplification only; canonical transport ownership, byte/age budgets,
deadline/cancel/dedup, terminal responses, and NNET-P1-037/G13/G19/G21 remain
open.
