# RuntimeInterface03 Single-Domain ECS Impact Projection

## Status

`implementation_complete; managed_validation_pending`

## Scope

- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`
- Parent plan: `03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- Finding: a single dirty-domain query rebuilt impact buckets for all 8 domains.

## Change

Snapshot and delta `dirty_domain_impact(domain)` now call one shared focused
aggregator. It scans the source nodes or changes once and collects only the
requested domain's node IDs, preserving the prior sorted/deduplicated receipt.
Full-table `dirty_domain_impacts()` and derived-field freshness checks remain
unchanged.

The focused aggregators live in `ui/ecs/focused_impact.rs`, keeping the public
ECS contract module from absorbing another implementation domain.

## Performance Contract

- Domain membership checks per single query: approximately `8 * N -> N`.
- Domain buckets materialized per query: up to `8 -> 1`.
- Benchmark fixture: 4,096 fully dirty nodes, 100 Render-domain queries, 11
  alternating old/full-table and new/focused samples.
- Acceptance threshold: focused P95 must be at least 20% lower than full-table P95.
- Exact P50/P95: pending managed Windows release benchmark output.

## Verification

- TDD RED: focused static contract failed before the benchmark marker and
  focused helper existed.
- Focused static contract after implementation: `3/3` passed; combined
  accessibility and ECS performance contracts: `12/12` passed.
- Python compile check and `git diff --check`: passed.
- Managed crate test and ignored release benchmark: pending asynchronous batch.

No commit, push, or WeCom notification is permitted until managed validation is
terminal-successful and the coordinator finalizes the attributed union.
