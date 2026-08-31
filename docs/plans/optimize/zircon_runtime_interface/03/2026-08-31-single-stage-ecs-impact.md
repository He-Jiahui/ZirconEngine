# RuntimeInterface03 Single-Stage ECS Impact Projection

## Status

`implementation_complete; managed_validation_pending`

## Scope

- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`
- Parent plan: `03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- Finding: a single schedule-stage query rebuilt impact buckets for all 10 runtime stages.

## Change

Snapshot and delta `schedule_impact(stage)` now call one shared focused aggregator.
It scans the source nodes or changes once, collects only the requested stage's
node IDs and dirty reasons, and preserves the prior sorted/deduplicated owned
receipt. Full-table `schedule_impacts()` and derived-field freshness checks are
unchanged.

The focused aggregators live in `ui/ecs/focused_impact.rs`, keeping the public
ECS contract module from absorbing another implementation domain.

Archived diagnostic stages still return `None`, matching the former lookup in
the runtime-only full impact table.

## Performance Contract

- Stage admission checks per single query: approximately `10 * N -> N`.
- Impact buckets materialized per query: up to `10 -> 1`.
- Benchmark fixture: 4,096 fully dirty nodes, 100 RenderExtract queries, 11
  alternating old/full-table and new/focused samples.
- Acceptance threshold: focused P95 must be at least 20% lower than full-table P95.
- Exact P50/P95: pending managed Windows release benchmark output.

## Verification

- TDD RED: focused static contract failed before the benchmark marker and
  focused helper existed.
- Focused static contract after implementation: `3/3` passed.
- Python compile check and `git diff --check`: passed.
- Managed crate test and ignored release benchmark: pending asynchronous batch.

No commit, push, or WeCom notification is permitted until managed validation is
terminal-successful and the coordinator finalizes the attributed union.
