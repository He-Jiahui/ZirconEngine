# Plugins14 Single-Pass Navigation Stats Optimization Record

- Date: 2026-08-19
- Owner: `plugins14-arc-assets-demand-overlay-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md`, NNAV-P1-037 and NNAV-P1-047
- Status: implementation complete; batched managed validation pending

## Problem

`count_navigation_components()` projected and sorted every World node once for
agents and obstacles, then two helper calls projected the complete World again
for off-mesh links and bridges. A single stats read therefore performed four
full scene projections for four component-presence counters.

## Change

- One node traversal now checks agent, obstacle, off-mesh-link, and
  off-mesh-bridge component presence together.
- All public counter fields and immediate visibility of dynamic component
  writes remain unchanged.
- Bake-specific obstacle counting remains separate because it is a different
  call site and does not request the other counters.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Stats over 4,096 nodes | 4 World projections | 1 World projection | 75% |
| Scene-node records projected | 16,384 | 4,096 | 75% |

## Acceptance

- `single_pass_navigation_stats_preserve_all_component_counts` compares every
  affected counter with the previous four-pass algorithm.
- `navigation_stats_use_one_world_projection_for_all_component_types` rejects
  restored helper scans or additional node projections.
- `single_pass_navigation_stats_release_benchmark_evidence` compares 21 paired,
  alternating release samples over 4,096 nodes and computes nearest-rank
  P50/P95.
- Timing gate: optimized P95 must be no more than 40% of legacy P95.
- Exact-file Rustfmt, source checks, and `git diff --check`: passed.
- Cargo regression and release P50/P95: pending the same batched Windows
  coordinator validation as single-pass agent-tick inputs.

## Remaining Scope

The counters still scan the complete World and use dynamic component lookup.
Incremental lifecycle-owned counters and explicit stats publication cadence
remain necessary for the full NNAV-P1-037/047 acceptance envelope.
