---
title: Runtime417 Direct Bitmap Shelf Allocation
category: zircon_runtime
report_id: Runtime417-direct-bitmap-shelf-allocation-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime417 Direct Bitmap Shelf Allocation

Bitmap glyph allocation now mutates the selected shelf allocator directly through the existing
`BTreeMap` entry. `GlyphAtlasShelfAllocator::allocate` computes candidate cursors in locals and
writes them back only after the glyph fits, so failed allocations preserve the same state without
copying the allocator.

The previous defensive path performed a map lookup, cloned the allocator, allocated against the
trial, then performed a second tree operation to replace the original on every successful glyph.
The new path performs one mutable lookup and one allocation while retaining the exact page rollover
and failure behavior.

The ignored Windows Release benchmark emits
`RUNTIME417_DIRECT_BITMAP_SHELF_ALLOCATION_BENCH_V1` over 17 alternating paired samples, each
performing 65,536 successful allocations through a one-page `BTreeMap`, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime417 is prepared with Editor345 under request
`runtime417-editor345-performance-batch-20260830di-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
