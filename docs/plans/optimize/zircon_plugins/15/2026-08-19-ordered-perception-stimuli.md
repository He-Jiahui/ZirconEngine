# Plugins15 Ordered Perception Stimuli Optimization Record

- Date: 2026-08-19
- Owner: `plugins15-perception-sampling-order-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md`, perception snapshot cost adjacent to NAI-P1-037 and NAI-P1-047
- Status: implementation complete; combined managed validation pending

## Problem

Each receiver stored perceived stimuli in a `HashMap`. Every `snapshot()` and
every entry in `snapshots()` cloned the complete map into a vector and sorted it
by sense rank and source. Repeated debug/runtime reads therefore paid a fresh
`O(N log N)` sort even when the perceived set had not changed.

## Change

- Each receiver now stores stimuli in a `BTreeMap` ordered by the existing
  `(sense rank, source entity)` contract.
- Snapshot reads clone values directly in deterministic order and perform no
  sort.
- Refresh replacement, age comparison, forgetting, receiver ordering, and
  scan cursor semantics remain unchanged.
- The tradeoff is explicit: stimulus lookup/refresh changes from expected
  `O(1)` hash access to `O(log N)` tree access in exchange for linear ordered
  snapshots.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Snapshot 8,192 stimuli | 1 full sort | 0 sorts | 100% |
| Snapshot complexity | O(N log N) plus clone | O(N) ordered clone | one complexity class |
| Refresh lookup | expected O(1) | O(log N) | accepted only if read-path P95 gate passes |

## Acceptance

- `ordered_stimulus_storage_preserves_snapshot_contract_after_out_of_order_refresh`
  proves sense/source ordering remains unchanged.
- `snapshot_reads_do_not_sort_ordered_stimulus_storage` rejects a restored
  snapshot sort or unordered inner store.
- `ordered_perception_stimuli_release_benchmark_evidence` compares 21 paired,
  alternating release samples over 8,192 stimuli and computes nearest-rank
  P50/P95.
- Timing gate: optimized snapshot P95 must be no more than 75% of legacy P95.
- Exact-file Rustfmt and scoped diff checks: passed.
- Cargo regression and release P50/P95: pending the same batched Windows
  coordinator validation as single-pass sampling.

## Remaining Scope

This record removes repeated snapshot sorting only. Runtime debug publication
is still unconditional, snapshot payloads are still fully cloned, and no
delta/ring-buffer subscription gate exists. Those broader NAI-P1-047 and G16
requirements remain open.
