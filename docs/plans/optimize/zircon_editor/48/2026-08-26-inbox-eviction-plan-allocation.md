# Editor48 Inbox Eviction Plan Allocation Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor48 retention and performance gates, including E-MSG-P1-21,
  E-MSG-P1-22, and E-MSG-P2-15
- Status: implementation and release gate authored; batched managed validation pending

## Problem

The Latest, Latest-replacement, and Bounded inbox admission paths constructed a
heap `Vec` containing every delivery selected for eviction. Admission then
iterated that vector a second time to remove the same oldest entries from
already ordered indexes. A rolling full inbox therefore allocated and copied a
temporary sequence plan on the common one-out/one-in path.

## Change

- Keep the complete no-mutation capacity/byte/sequence preflight, but return
  only the number of oldest entries that must be evicted.
- Commit an accepted plan with `BTreeMap::pop_first` or `BTreeSet::pop_first`.
- Preserve out-of-order rejection, same-key replacement, total byte accounting,
  lane depths, drop/coalesce counters, enqueue dispositions, and delivery order.
- Keep exact evicted-key reporting as separate remaining Editor48 protocol work;
  the current public report did not expose the removed identity vector.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 50,000 full bounded-inbox plans, one eviction each | 50,000 temporary plan allocations | 0 | 100% removed |
| Copied eviction sequence IDs | 50,000 | 0 | 100% removed |
| Planning index probes | first eligible ordered entries | same | unchanged |
| Admission transaction boundary | plan, then mutate | plan, then mutate | unchanged |

The ignored release gate runs 17 alternating allocating/count-only sample pairs
over a full 1,024-entry bounded inbox. Acceptance requires count-only
nearest-rank P95 to be at most 70% of legacy P95, a minimum 30% reduction.
Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826d_editor48_inbox_eviction_plans_store_only_counts`
  locks all three count-only planners and oldest-index commit paths.
- `optimization_batch_20260826d_editor48_inbox_rolling_eviction_preserves_order_and_stats`
  covers 64 rolling deliveries, final order/depth, and exact drop accounting.
- `optimization_batch_20260826d_editor48_inbox_eviction_plan_performance_evidence`
  emits `EDITOR48_INBOX_EVICTION_COUNT_PLAN_BENCH_V1`, raw samples, iteration
  count, retained depth, allocation/copy counts, and the 30% P95 threshold.
- Exact-file Rust 1.94.1 rustfmt, source contracts, and scoped diff checks must
  pass before managed validation submission.

## Remaining Plan Work

This slice does not close Editor48. Typed eviction/resync dispositions, scoped
Latest keys, route generations, page/cursor drain, ack/nack, owner-bound leases,
shutdown fencing, downstream plugin queue budgets, product diagnostics, and
full 1/5/100/10K subscriber qualification remain open.
