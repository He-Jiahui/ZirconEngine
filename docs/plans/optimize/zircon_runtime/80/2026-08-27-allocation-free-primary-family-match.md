---
title: Runtime80 Allocation-free Primary Family Match
category: zircon_runtime
report_id: Runtime80-allocation-free-primary-family-match-2026-08-27
date: 2026-08-27
session_id: root-runtime80-allocation-free-primary-family-match-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime80 Allocation-free Primary Family Match

## Scope

This slice removes temporary normalized `String` allocations while selecting the primary member
of a font family asset. It preserves the previous `trim().to_ascii_lowercase()` comparison
semantics: surrounding Unicode whitespace is ignored, ASCII case is folded, and non-ASCII bytes
remain case-sensitive. It does not change face-index precedence, fallback selection, descriptor
normalization, or persistent font family keys.

## Change

- Compare borrowed trimmed family names with `eq_ignore_ascii_case` during the member scan.
- Keep `normalized_family_key` for `FontAssetSourceKey`, where an owned canonical key is required.
- Cover whitespace and mixed ASCII case matching plus strict non-ASCII case behavior with Rust
  tests and a source contract.

## Deterministic Performance Evidence

The standalone optimized Rust model scans 32,768 face-index candidates with the matching family at
the final position and alternates legacy/optimized order for 17 samples. A counting allocator
measures only the scan, and both implementations return the same member index across ASCII,
Unicode-whitespace, and non-ASCII case fixtures.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Family normalization allocations | 65,536 | 0 | 100% |
| Family normalization bytes | 1,343,473 | 0 | 100% |
| Primary member scan P50 | 5.668 ms | 1.168 ms | 79.387% |
| Primary member scan P95 | 6.626 ms | 1.567 ms | 76.357% |

Evidence marker: `RUNTIME80_ALLOCATION_FREE_PRIMARY_FAMILY_MATCH_MODEL_V1`.

## Validation

- `python -m unittest tools.tests.test_runtime80_allocation_free_primary_family_match_performance_contract -v`: 3 passed.
- Exact-file Rust formatting and scoped diff checks passed.
- This task is queued in the Runtime80 three-task asynchronous validation batch with borrowed
  protected-glyph set construction and single-probe glyph byte accounting. The batch runs the three
  Python source contracts, four `runtime80_batch_` behavior tests, two ignored release benchmarks,
  and the
  standalone family-match model. Dynamic evidence, integration SHA, and automatic WeCom delivery
  remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime80 still owns cooked font bytes, project/session collection authority, fallback determinism,
malformed-font budgets, platform coverage, and product-scale glyph/cache validation gaps recorded
in the canonical review.
