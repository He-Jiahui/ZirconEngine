# Plugins15 Targeted Debug-Snapshot Projection Optimization Record

- Date: 2026-08-19
- Owner: `plugins15-perception-sampling-order-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md`, NAI-P1-047
- Status: implementation and focused static validation complete; managed release batch queued

## Problem

The behavior-tick system built a complete `AiRuntimeSnapshot` for every World
on every frame, cloned every manager agent and every registered tree
descriptor, then filtered the result back to the current World's active
agents. Work in unrelated Worlds and inactive agents therefore inflated a
debug event generated for a bounded active set.

## Change

- The existing public `runtime_snapshot()` contract remains available for
  explicit whole-manager inspection.
- The behavior-tick system now passes its already computed active entity set to
  an internal targeted projection.
- Targeted projection holds the manager lock once and clones only requested
  agents that still exist in the requested World.
- Debug frame fields, active-agent filtering, event order, and stale-report
  retirement remain unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 8,192 manager agents, 256 active in current World | 8,192 agent projections | 256 agent projections | 96.875% |
| Per-frame global key union | 1 full union | 0 | 100% |
| Per-frame behavior-tree descriptor catalog clone | 1 full clone | 0 | 100% |

## Current Execution Evidence

- Integration Session: `root-runtime-interface03-activate-link-failure-20260831`;
  ownership apply `b684ea3ed9304cf4a9f71e5787befa1a`, fingerprint
  `b9047be13003a7c46040170fe788f53d2c5dcc8722f2488ffecc97d56ebffbbc`.
- Current `manager/snapshot.rs` SHA-256:
  `064822D9DBAEC7B7D2C71B2428103D65DEC5235E172C11A7DE8E817029FC7C00`.
- Unified deterministic model manifest SHA-256:
  `93CF6BD9C2D374D1F4C81CF6776948372611820AAB048DB2EB499977E8493347`.
  It records agent projections `8,192 -> 256` (`96.875%` fewer), global key
  unions `1 -> 0`, and behavior-tree descriptor catalog clones `1 -> 0`.
- Focused source/model/validator contract passed locally `12/12`; managed
  static ticket `049d11366ae94ef38ddc58158d6e6b69` is queued.
- Four-benchmark Windows release batch ticket
  `bf5d08d9143849e189ac6e0fa1bb477c` is queued. Its 21 alternating sample
  pairs are the only accepted P50/P95 source.

## Acceptance

- `targeted_projection_reads_only_requested_world_agents` verifies World and
  entity filtering plus payload preservation.
- `extracted_agent_projection_retains_last_report_only_agents` preserves the
  public full-snapshot union contract and the targeted result for agents whose
  only remaining manager state is a last tick report.
- `behavior_tick_uses_targeted_debug_projection` rejects a restored full
  `runtime_snapshot()` call in the registration system.
- `targeted_debug_snapshot_release_benchmark_evidence` compares 21 paired,
  alternating release samples for 8,192 total agents and 256 active agents,
  then computes nearest-rank P50/P95.
- Timing gate: targeted projection P95 must be no more than 25% of full
  snapshot P95.
- Exact-file Rustfmt, source assertions, and scoped `git diff --check`: passed.
- Cargo regression and release P50/P95: queued in the same batched Windows
  coordinator validation as immutable compiled-tree generation.

## Remaining Scope

Debug snapshots are still emitted every behavior frame and clone complete
blackboard/perception payloads for active agents. Subscription-driven enable,
delta/ring-buffer transport, and items/bytes/time/age budgets remain required
to close NAI-P1-047 rather than merely removing the global projection.
