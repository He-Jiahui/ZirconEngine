# Plugins15 Single-Pass Perception Sampling Optimization Record

- Date: 2026-08-19
- Owner: `plugins15-perception-sampling-order-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md`, NAI-P1-037
- Status: implementation complete; combined managed validation pending

## Problem

Every perception tick called `World::node_records()` once for receivers and a
second time for sources. Each call projected and sorted the complete World,
then both filtered result vectors were sorted again even though
`node_records()` already guarantees entity order. The fixed pair budget did
not bound this pre-scan cost.

## Change

- One stable World projection now feeds receiver and source sampling together.
- Entities carrying both components reuse the same world-transform projection.
- Receiver/source vectors preserve the existing entity order without two
  redundant result sorts.
- Pair cursor, event fairness, LOS behavior, and the 256-pair default budget are
  unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| One tick over 4,096 nodes | 2 full World projections | 1 full World projection | 50% |
| Same tick | 2 redundant sample-vector sorts | 0 redundant sorts | 100% |
| Scene-node projections before filtering | 8,192 | 4,096 | 50% |

## Acceptance

- `single_pass_sampling_preserves_stable_receiver_and_source_order` compares
  the receiver/source entity order with the previous two-pass algorithm.
- `perception_tick_uses_one_world_projection_without_redundant_sample_sorts`
  rejects a restored second projection or result sort.
- `single_pass_perception_sampling_release_benchmark_evidence` compares 21
  paired, alternating release samples over 4,096 nodes and computes
  nearest-rank P50/P95.
- Timing gate: optimized P95 must be no more than 75% of legacy P95.
- Exact-file Rustfmt and scoped diff checks: passed.
- Cargo regression and release P50/P95: pending one batched Windows coordinator
  validation with the ordered-stimulus task.

## Remaining Scope

This is a bounded removal of duplicate work, not completion of NAI-P1-037.
Perception still projects the complete World once per tick and still evaluates
receiver-by-source slots. Incremental listener/source ownership, dirty sets,
spatial indexing, generation fencing, and the G16/G18 steady-state gates remain
open.
