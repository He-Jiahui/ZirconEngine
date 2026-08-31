---
title: Runtime11C Moved SDF Async Batches
category: zircon_runtime
report_id: Runtime11C-moved-sdf-async-batches-2026-08-26
date: 2026-08-26
session_id: root-runtime11c-moved-sdf-async-batches-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11C Moved SDF Async Batches

## Scope

This slice removes the transient full-key clone performed while the SDF async generation queue
splits each source/parameter group into scheduler-sized batches. It supports Runtime11C's stable UI
allocation and SDF churn work. It does not change atlas ownership, glyph routing, scheduler budgets,
submission order, fallback behavior, or completion matching.

## Change

- Each completed `AsyncBatchGroup` now consumes its owned entry vector in original insertion order.
- A bounded helper moves at most `max_glyphs_per_batch` entries into each submission vector. The
  submission still owns its entries, but no longer clones `SdfAtlasGlyphKey` or its owned strings.
- Batch size is resolved once per prepare call. A nonzero assertion preserves the failure contract
  previously provided by `slice::chunks(0)`.
- A Rust behavior test fixes the `2 / 2 / 1 / 0` boundaries and glyph order for five entries.

## Deterministic Performance Evidence

The independent release model uses 65,536 entries, four owned strings per key, scheduler batches of
64, and 21 alternating legacy/moved sample pairs. Input construction is outside the allocation and
timing window.

| Evidence | Legacy `chunks().to_vec()` | Moved batches | Result |
|---|---:|---:|---:|
| Allocations per measured run | 263,168 | 1,024 | 99.611% removed |
| Run 1 P50 | 52.643 ms | 18.472 ms | 64.905% faster |
| Run 1 P95 | 110.661 ms | 23.608 ms | 78.667% faster |
| Run 2 P50 | 50.518 ms | 20.594 ms | 59.235% faster |
| Run 2 P95 | 60.566 ms | 58.264 ms | 3.802% faster |
| Run 3 P50 | 52.778 ms | 19.780 ms | 62.523% faster |
| Run 3 P95 | 95.034 ms | 54.355 ms | 42.804% faster |

The managed performance gate requires the exact allocation counts above, at least 99% allocation
reduction, at least 25% P50 improvement, and no more than 5% P95 regression under Windows scheduler
noise.

## Acceptance

- `tools.tests.test_runtime11c_moved_sdf_async_batches_performance_contract` passes 3/3 locally.
- `moved_async_batches_preserve_boundaries_and_entry_order` is submitted through the coordinator in
  the same validation batch as source contracts, exact-file formatting, the performance model, and
  scoped diff checks.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

SDF slots may still relocate during atlas rebuilds; product quality/budget selection remains fixed;
glyph, icon, and image atlases still lack one device owner; and the game/Editor text routes remain
separate.
