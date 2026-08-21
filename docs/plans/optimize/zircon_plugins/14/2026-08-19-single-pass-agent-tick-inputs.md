# Plugins14 Single-Pass Agent-Tick Input Optimization Record

- Date: 2026-08-19
- Owner: `plugins14-arc-assets-demand-overlay-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md`, NNAV-P1-037
- Status: implementation complete; batched managed validation pending

## Problem

The native crowd tick projected and sorted every World node once to collect
agents and immediately repeated the same full projection only to determine
whether any runtime obstacle existed. This duplicate work was paid before the
crowd budget and scaled with total scene size rather than active navigation
entities.

## Change

- One stable node projection now collects parsed agent descriptors and detects
  obstacle-component presence together.
- Agent order remains the order supplied by `World::node_records()`.
- The existing fallback decision remains unchanged: any runtime obstacle,
  obstacle world, or off-mesh link still routes the tick to the legacy path.
- No obstacle descriptor is parsed on the native path because this decision
  only requires component presence, matching the previous `is_empty()` use.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| One native tick over 4,096 nodes | 2 World projections | 1 World projection | 50% |
| Scene-node records projected before crowd work | 8,192 | 4,096 | 50% |

## Acceptance

- `single_pass_agent_tick_inputs_preserve_agents_and_obstacle_detection`
  compares the optimized collector with the previous two-pass algorithm.
- `agent_tick_inputs_use_one_world_projection` rejects a restored second
  projection or runtime-obstacle collection call.
- `single_pass_agent_tick_inputs_release_benchmark_evidence` compares 21
  paired, alternating release samples over 4,096 nodes and computes
  nearest-rank P50/P95.
- Timing gate: optimized P95 must be no more than 70% of legacy P95.
- Exact-file Rustfmt, source checks, and `git diff --check`: passed.
- Cargo regression and release P50/P95: pending the next batched Windows
  coordinator validation.

## Remaining Scope

This removes duplicate projection only. The agent system still parses dynamic
JSON and scans the complete World. Typed incremental agent/obstacle ownership,
dirty queues, per-World generations, and item/time budgets remain required to
close NNAV-P1-037.
