---
title: Runtime08D Targeted Navigation Agent Index Optimization
category: zircon_runtime
report_id: Runtime08D-targeted-agent-index-2026-08-24
date: 2026-08-24
session_id: root-runtime08d-agent-index-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime08D Targeted Navigation Agent Index Optimization

## Scope

This slice advances Runtime08D P1-13 for the built-in navigation projection: a targeted
`tick_world_agent` no longer scans every projected agent to locate one entity. It does not claim
the navigation plugin's full scene projection, persistent Detour owner, Crowd/TileCache, bake,
world identity, movement authority, or Editor workflow milestones are complete.

## Implementation

`NavigationWorldProjection` now builds one entity-to-agent-row index with each projection
generation. `RuntimeAgent` retains its optional position row, allowing the same index to serve both
descriptor lookup and incremental position updates without adding a second entity HashMap.

The product targeted-tick route resolves the descriptor through `agent_descriptor` and retains the
existing cloned descriptor boundary before mutating the projection. Missing entities continue to
return an empty report. Regression coverage checks hits, misses, probe accounting, product routing,
and the absence of the former iterator/find scan.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 100,000 tail-entity lookups in a 10,000-agent projection | 1,000,000,000 entity comparisons | 100,000 hash probes; <= 500 ms | 99.99% lookup-work reduction; O(n) -> O(1) average |
| Projection entity indexes | one position-row map | one shared agent-row map | no second entity HashMap |

The ignored Windows-native release evidence prints `RUNTIME08D_AGENT_INDEX_BENCH_V1` with exact
elapsed nanoseconds, target, agent/lookup counts, and deterministic comparison/probe counts. Dynamic
elapsed time is accepted only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, and nine targeted lookup/index/evidence source
  contracts: passed.
- Navigation behavior regressions and ignored release evidence: pending the shared Runtime08D and
  Runtime89 coordinator batch.
- No local Cargo lane is launched, and no coordinator compilation is monitored in real time.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

The first-party navigation plugin still rebuilds native Detour query owners, its manager files are
under another active session, and the plugin projection still has the wider P1-13 scan/allocation
scope. Those changes require their owning session and are not hidden by this built-in improvement.
