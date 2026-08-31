---
title: Runtime47 Hover Membership Index Optimization
category: zircon_runtime
report_id: Runtime47-hover-membership-index-2026-08-24
date: 2026-08-24
session_id: root-runtime47-three-task-picking-performance-batch-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime47 Hover Membership Index Optimization

## Scope

This slice removes repeated linear target scans from `PickingHoverMap::is_hovered`. It advances
Runtime47's drag and hover scaling work without changing hit sorting, blocking rules, pointer
capture, event routing, target ownership, or the wider picking architecture.

## Implementation

Each pointer generation now stores its ordered `HitRecord` vector beside a `HashSet<HitTarget>`
membership index. The index is built once when a generation is created or replaced, so repeated
hover diff queries use average O(1) membership checks instead of rescanning the ordered hits.

The ordered vector remains authoritative for `get`, `iter`, and `hit`. Duplicate targets therefore
retain their former first-hit behavior, pointer iteration stays sorted, and the whole generation
continues to share one `Arc` backing store. Replacement and removal update both representations
together.

Regression coverage checks duplicate-target first-hit semantics, replacement, removal, and the
indexed production path. A source contract guards the single-generation state and direct set
membership lookup.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 10,000 alternating tail/missing queries over 2,048 hover hits | 20,480,000 target comparisons | 10,000 hash probes | 99.9512% lookup-work reduction |
| Per-generation hover diff membership | O(hits x queries) | O(hits + queries) average | one index build, then direct membership probes |
| Ordered hit access and duplicate targets | ordered vector scan | ordered vector scan | first-hit semantics retained |
| 2,048 hits / 10,000 queries release p95 | dynamic evidence pending | <= 50 ms and <= 50% of legacy p95 | coordinator release gate |

The ignored Windows-native release evidence alternates 11 legacy/optimized sample pairs and prints
`RUNTIME47_HOVER_MEMBERSHIP_BENCH_V1` with exact p95 nanoseconds, the target, hit/query counts, and
deterministic lookup-work counts. Dynamic elapsed time is accepted only from coordinator terminal
evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, and Runtime47 hover-map source contracts:
  passed.
- `runtime47_batch_hover_membership_tracks_mutation_and_first_hit`,
  `runtime47_batch_hover_membership_uses_generation_index`, and
  `runtime47_batch_hover_membership_evidence` are queued in one Runtime47 three-task asynchronous
  validation batch with the pointer-hit grouping and pointer-location tasks. The batch runs nine
  `runtime47_batch_` Rust tests and three exact performance rows; no local Cargo lane is launched.
- P95 uses the existing nearest-rank helper and the managed ticket must publish the measured
  2,048-hit/10,000-lookup result before this record can be accepted.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

Runtime47 still owns the broader pointer state machine, capture semantics, world ray generation,
drag lifecycle, click/drag classification, and stable event routing. Those milestones remain
separate work and are not claimed complete by this membership optimization.
