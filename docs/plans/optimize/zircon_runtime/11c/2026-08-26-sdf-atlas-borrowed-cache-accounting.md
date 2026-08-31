---
title: Runtime11C SDF Atlas Borrowed Cache Accounting
category: zircon_runtime
report_id: Runtime11C-sdf-atlas-borrowed-cache-accounting-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11C SDF Atlas Borrowed Cache Accounting

## Scope

This slice removes transient glyph-key ownership from SDF atlas cache membership, eviction, and
transition reporting. It supports Runtime11C P1-13/P1-14 and the P2-5 long-running atlas churn
qualification. It does not claim a shared device atlas, generational slot handles, compaction,
product quality tiers, or unified game/Editor text routing.

## Change

- Cache membership indexes borrow `SdfAtlasGlyphKey` values from the retained slot list. New keys
  are discovered in a short read-only phase, then cloned only when a new owned cache slot is
  actually appended.
- Inactive eviction plans store slot indices. Their comparator reads generation and key from the
  cache, avoiding one owned key clone per candidate while preserving generation/key/index order.
- Previous/current transition sets and lookup maps borrow keys from their source plans. Dirty-page,
  stable/relocated, added, and evicted accounting remains unchanged.
- The final atlas plan still owns every live key; this slice does not weaken plan lifetime or
  serialization boundaries.

## Deterministic Performance Evidence

| 4,096 stable slots | Before | After | Reduction |
|---|---:|---:|---:|
| Transition-report key clones | 16,384 | 0 | 100% removed |
| Owned strings cloned inside report keys | 32,768 | 0 | 100% removed |
| Cached membership key clones | 4,096 | 0 | 100% removed |
| Inactive eviction candidate key clones | one per candidate | 0 | 100% removed |

The ignored release gate alternates 17 owned-key and borrowed transition-accounting samples. It
emits `RUNTIME11C_SDF_ATLAS_BORROWED_CACHE_ACCOUNTING_BENCH_V1`; acceptance requires borrowed P95
to be at most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826g_runtime11c_borrowed_cache_accounting_preserves_eviction_order`
  covers generation/key tie-breaking and retained current glyphs.
- `optimization_batch_20260826g_runtime11c_cache_accounting_borrows_glyph_keys` rejects owned key
  clones in membership, eviction, and transition reporting.
- `optimization_batch_20260826g_runtime11c_borrowed_cache_accounting_performance_evidence` emits
  both P95 values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting with child traversal disabled, scoped diff checks, and source
  contracts must pass before managed validation submission.

## Remaining Parent-plan Work

SDF slots may still relocate when the plan is rebuilt, product quality/budget selection remains
fixed, glyph/icon/image atlases do not share a device owner, and long-running CJK/emoji/theme/DPI
churn evidence remains open.
