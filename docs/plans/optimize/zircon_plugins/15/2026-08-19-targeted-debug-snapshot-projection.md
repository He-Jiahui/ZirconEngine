# Plugins15 Targeted Debug-Snapshot Projection Optimization Record

- Date: 2026-08-19
- Owner: `plugins15-perception-sampling-order-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md`, NAI-P1-047
- Status: implementation complete; combined managed validation pending

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
- Cargo regression and release P50/P95: pending the same batched Windows
  coordinator validation as immutable compiled-tree generation.

## Remaining Scope

Debug snapshots are still emitted every behavior frame and clone complete
blackboard/perception payloads for active agents. Subscription-driven enable,
delta/ring-buffer transport, and items/bytes/time/age budgets remain required
to close NAI-P1-047 rather than merely removing the global projection.
