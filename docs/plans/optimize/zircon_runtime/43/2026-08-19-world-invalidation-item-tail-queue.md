# Runtime43 World Invalidation Item Tail-Queue Optimization Record

- Date: 2026-08-19
- Owner: `runtime43-world-invalidation-tail-queues-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md`, DYN-P1-057 / DYN-GATE-038
- Status: implementation and 21-pair measurement repair complete; combined managed validation pending

## Problem

A partially delivered batch removed dirty tokens and facts with prefix
`Vec::drain`. Repeated small pages over one large batch repeatedly moved the
remaining item suffix, preserving a second O(N^2) path after batch-level
removal was addressed.

## Change

- Sealing reverses each batch's dirty-token and fact vectors once.
- Page construction reads those vectors from the tail and materializes the
  original canonical order in the public page.
- Commit subtracts delivered lengths with tail `truncate`, avoiding any item
  shift or allocation.
- Canonical ascending dirty-token order remains visible in every serialized
  `InvalidationBatch`; only the private pending representation is reversed.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Commit 20,000 items one at a time | 199,990,000 item moves | 0 item moves after one reversal | 100% |
| Partial item commit | O(remaining items) | O(1) truncate | one complexity class |
| Full single-batch drain | O(N^2) moves | O(N) reverse + O(N) reads | one complexity class |

## Acceptance

- `world_invalidation_pages_commit_only_the_delivered_prefix` covers partial
  page commit and retry on the private tail representation.
- `world_invalidation_tail_queues_preserve_batch_and_item_order` validates
  canonical external token order across multiple batches.
- `world_invalidation_tail_queue_source_has_no_front_removal` rejects prefix
  `drain` in the commit owner.
- `world_invalidation_tail_queue_release_benchmark_evidence` provides a
  dedicated 21-pair alternating distribution for repeated prefix `drain(..1)`
  versus reversed-tail `truncate(len - 1)`. It does not reuse the batch-pop
  timing samples.
- Timing gate: tail-queue P95 must be no more than 25% of legacy P95.
- Exact-file Rustfmt and scoped diff checks: passed.
- Cargo regression and release P50/P95: pending a combined Windows coordinator
  batch; no per-task Cargo command is used.

## Remaining Scope

Page sizing still clones and encodes candidates while holding the session
owner lock, and the public page has no remaining/cursor receipt. Those are
separate DYN-P1-057 tasks.
