# Plugins15 Single-Pass Perception Sampling Optimization Record

- Date: 2026-08-19
- Owner: `plugins15-perception-sampling-order-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md`, NAI-P1-037
- Status: implementation and focused static validation complete; managed release batch queued

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

## Current Execution Evidence

- Integration Session: `root-runtime-interface03-activate-link-failure-20260831`;
  ownership apply `b684ea3ed9304cf4a9f71e5787befa1a`, fingerprint
  `b9047be13003a7c46040170fe788f53d2c5dcc8722f2488ffecc97d56ebffbbc`.
- Current `perception/scan.rs` SHA-256:
  `AB2133C3ACF11A4FB841EA5BE52995608839B0516DEF2F4A31FE4D5D394CF768`.
- Unified deterministic model manifest SHA-256:
  `93CF6BD9C2D374D1F4C81CF6776948372611820AAB048DB2EB499977E8493347`.
  It records World projections `2 -> 1`, projected node records
  `8,192 -> 4,096`, and redundant sample sorts `2 -> 0`.
- Focused source/model/validator contract passed locally `12/12`; managed
  static ticket `049d11366ae94ef38ddc58158d6e6b69` is queued.
- Four-benchmark Windows release batch ticket
  `bf5d08d9143849e189ac6e0fa1bb477c` is queued. Its 21 alternating sample
  pairs are the only accepted P50/P95 source.

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
- Cargo regression and release P50/P95: queued in one batched Windows
  coordinator validation with the ordered-stimulus task.

## Remaining Scope

This is a bounded removal of duplicate work, not completion of NAI-P1-037.
Perception still projects the complete World once per tick and still evaluates
receiver-by-source slots. Incremental listener/source ownership, dirty sets,
spatial indexing, generation fencing, and the G16/G18 steady-state gates remain
open.
