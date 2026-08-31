---
title: Runtime11C SDF Page Hash Owner
category: zircon_runtime
report_id: Runtime11C-sdf-page-hash-owner-2026-08-26
date: 2026-08-26
session_id: root-runtime11c-three-task-page-hash-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11C SDF Page Hash Owner

## Scope

This slice replaces the persistent SDF atlas page owner with `HashMap`. Reset checks, glyph
replacement, stale-placement clearing, and nonzero-pixel accounting all resolve pages by stable
`GlyphAtlasPageKey` during each atlas update.

The bake output still has deterministic page order and stable `source_offset`: one explicit sorted
page projection is created after all page mutations. Dirty-page accumulation remains a `BTreeMap`,
so upload region order is unchanged.

## Performance Workload

The release workload fills 16,384 page keys and performs 4,096 stable persistent-page lookups for
the final key.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered page lookups | 4,096 | 0 |
| Hash page lookups | 0 | 4,096 |
| Ordered bake projections per update | implicit tree traversal | 1 explicit projection |
| Dirty-page order changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME11C_SDF_PAGE_HASH_OWNER_BENCH_V1`. Acceptance requires hash lookup P95 to be at least 30%
below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator run.

## Acceptance

- `runtime11c_batch_sdf_page_hash_owner_preserves_bake_order` covers intentionally
  shuffled page ownership and ascending bake projection.
- `runtime11c_batch_sdf_page_hash_owner_keeps_explicit_projection` locks the hash
  owner, sorted bake boundary, and ordered dirty-page owner.
- `runtime11c_batch_sdf_page_hash_owner_p95` reports paired release P50/P95 samples
  and enforces the 30% P95 reduction gate.

The managed `runtime11c_batch_` release gate seals this work together with the glyph allocator and
page-shadow slices in one Cargo invocation: three source contracts, nine Rust tests, and three
performance rows. Dynamic Windows marker values, commit attribution, and WeCom publication remain
pending the coordinator result.

## Remaining Parent-plan Work

Runtime11C still owns unified atlas resource contracts, device-generation recovery, quality tiers,
budget pressure, upload scheduling, and product-scale GPU qualification. This slice only converges
the persistent SDF page lookup owner.
