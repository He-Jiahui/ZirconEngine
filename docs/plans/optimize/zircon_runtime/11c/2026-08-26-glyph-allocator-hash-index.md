---
title: Runtime11C Glyph Allocator Hash Index
category: zircon_runtime
report_id: Runtime11C-glyph-allocator-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime11c-three-task-page-hash-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11C Glyph Allocator Hash Index

## Scope

This slice replaces the private glyph-atlas page allocator owner with `HashMap`. Every persistent
glyph allocation resolves one page allocator before its transactional shelf-allocation attempt,
and page invalidation removes that same allocator by key.

Allocator progress, configuration checks, rollback on allocation failure, page invalidation, and
slot identity are unchanged. The separate `slot_rects_by_page` diagnostic projection still builds
and returns a `BTreeMap`, preserving its page-order contract.

## Performance Workload

The release workload fills 16,384 glyph page keys and performs 4,096 stable allocator lookups for
the final page.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered allocator lookups | 4,096 | 0 |
| Hash allocator lookups | 0 | 4,096 |
| Ordered diagnostic projections | unchanged | unchanged |
| Allocator clones/commits | unchanged | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME11C_GLYPH_ALLOCATOR_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at
least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `runtime11c_batch_glyph_allocator_hash_index_isolates_page_lifetimes` covers
  independent page progress and targeted allocator invalidation.
- `runtime11c_batch_glyph_allocator_hash_index_keeps_ordered_diagnostics` locks the
  hash owner while preserving the ordered diagnostic projection.
- `runtime11c_batch_glyph_allocator_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

The managed `runtime11c_batch_` release gate seals this work together with the page-shadow and SDF
page-owner slices in one Cargo invocation: three source contracts, nine Rust tests, and three
performance rows. Dynamic Windows marker values, commit attribution, and WeCom publication remain
pending the coordinator result.

## Remaining Parent-plan Work

Runtime11C still owns unified page/resource contracts, device-generation recovery, quality tiers,
budget pressure, upload scheduling, and product-scale GPU text qualification. This slice only
converges the persistent glyph page allocator index.
