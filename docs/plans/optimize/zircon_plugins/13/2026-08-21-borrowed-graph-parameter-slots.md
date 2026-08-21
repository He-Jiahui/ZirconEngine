# Plugins13 Borrowed Compiled Graph Parameter Slots

- Date: 2026-08-21
- Owner: `optimize-plugins13-borrowed-graph-parameters-r1-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md`, `NANI-P1-029`
- Status: implementation complete; grouped managed regression and release measurements pending

## Problem

`CompiledAnimationGraph::evaluate` previously traversed every compiled parameter on every
evaluation, looked up every name in the override map, cloned every selected/default
`AnimationParameterValue`, and collected a temporary `Vec`. Graph traversal only consumes scalar
weights for referenced `ParameterSlot` values, so unused parameters paid allocation and clone costs
without affecting output.

## Change

- Graph traversal now carries the immutable compiled parameter table and caller override map by
  reference.
- A referenced slot resolves its finite override or compiled default on demand and borrows the
  resulting value for scalar conversion.
- Clip order, default fallback, non-finite override rejection, integer/bool scalar conversion,
  weight clamping, masks, and additive/base traversal remain unchanged.

## Deterministic Delta

For the release workload of 8,192 compiled parameters and 64 evaluations per sample:

| Metric | Legacy snapshot | Borrowed slot | Delta |
|---|---:|---:|---:|
| parameter visits per evaluation | 8,192 | 1 referenced slot | 99.9878% fewer |
| parameter-value clones per sample | 524,288 | 0 | 100% fewer |
| temporary parameter vectors per sample | 64 | 0 | 100% fewer |

The returned clip vector is still owned by `CompiledAnimationGraphEvaluation`; this milestone does
not claim allocation-free graph evaluation as a whole.

## Acceptance

- Behavior regression proves a non-finite scalar override still falls back to the compiled default.
- Source regression rejects `.cloned()` and parameter snapshot `collect::<Vec<_>>()` in the
  production evaluator.
- The ignored release benchmark runs 21 alternating legacy/borrowed pairs, emits both raw timing
  arrays, and uses nearest-rank P95.
- Borrowed-slot P95 must be no more than 25% of the legacy full-parameter snapshot P95.
- Rust 1.94.1 formatting and scoped diff checks pass.
- Cargo regression counts and measured P50/P95 remain pending the post-Main aggregate batch; no
  timing result is claimed by this record yet.

## Remaining Scope

`NANI-P1-029` remains open for dirty-slot instance state and reusable evaluation stacks. This slice
only removes the unconditional parameter snapshot from compiled graph evaluation.
