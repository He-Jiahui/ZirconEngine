---
title: Editor75 Timeline Static Cache Single-flight
category: zircon_editor
report_id: Editor75-timeline-static-cache-2026-08-24
date: 2026-08-24
session_id: root-editor75-timeline-static-cache-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor75 Timeline Static Cache Single-flight

## Scope

This slice addresses the cache-hit recency scan and same-key concurrent miss duplication identified
by `ED75-P1-09`. It does not claim the parent plan's per-surface cache ownership, collision-safe
descriptor identity, typed Timeline projection, virtualization, render budgets, or product
qualification are complete.

## Implementation

Each static-content cache entry now owns an `Arc<OnceLock<Arc<TimelineStripStaticContent>>>`.
Callers acquire or create the cell while holding the short cache lock, then initialize tick labels
outside that lock. Concurrent callers for the same resident key wait on and reuse the same cell
instead of each constructing a candidate and discarding all but one.

Cache hits update an access epoch directly in the existing `BTreeMap`; the previous
`VecDeque::retain` full-recency scan is removed. Capacity trimming runs only on insertion, chooses
the least-recently-used completed entry, and does not evict an in-flight cell. Epoch exhaustion has
an explicit order-preserving rebase rather than a wrapping counter.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 16-entry cache, 250K repeated hits | 4,000,000 recency-entry visits | 0 recency-entry visits; <= 1 s release | 100% recency scan removal |
| 16 concurrent callers, same cold key | up to 16 tick/label candidate builds | 1 `OnceLock` initialization | up to 93.75% duplicate builds removed |
| Hit-side content generation | double-checked candidate path | initialized cell clone | no candidate allocation on hit |

The ignored Windows-native release evidence prints `EDITOR75_TIMELINE_CACHE_BENCH_V1` with cache
capacity, hit count, legacy and optimized recency visits, and elapsed nanoseconds. Exact elapsed
time and concurrency results are accepted only from coordinator terminal evidence.

## Validation

- Existing static-content reuse, scrub reuse, generation separation, hard tick cap, cache capacity,
  invalid-input behavior, the 16-worker same-key regression, source guard, and release performance
  evidence are prepared for a shared Editor batch.
- Exact `rustfmt --check`, scoped `git diff --check`, and removal of `VecDeque`/`recency.retain` are
  part of the static preflight.
- No local Cargo lane is launched and no compilation is monitored in real time.
- Final validation ticket, terminal marker values, commit integration, and WeCom delivery remain
  pending.

## Remaining Parent-plan Work

The cache is still process-global and keyed by a compact generation digest plus visual budget. A
later milestone must move authority to document/surface render contexts, verify full descriptors,
publish contention/eviction metrics, and integrate the typed paged Timeline projection. The broader
interaction, transaction, preview, persistence, accessibility, render, fault, and cross-engine
gates remain open.
