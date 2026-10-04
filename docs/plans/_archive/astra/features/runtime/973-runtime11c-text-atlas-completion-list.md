---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11c/2026-08-26-auto-text-recency-sort-projection.md
  - docs/plans/optimize/zircon_runtime/11c/2026-08-26-glyph-allocator-hash-index.md
  - docs/plans/optimize/zircon_runtime/11c/2026-08-26-glyph-page-single-pass-index.md
  - docs/plans/optimize/zircon_runtime/11c/2026-08-26-icon-request-hash-dedup.md
  - docs/plans/optimize/zircon_runtime/11c/2026-08-26-moved-sdf-async-batches.md
  - docs/plans/optimize/zircon_runtime/11c/2026-08-26-page-shadow-hash-index.md
  - docs/plans/optimize/zircon_runtime/11c/2026-08-26-sdf-atlas-borrowed-cache-accounting.md
  - docs/plans/optimize/zircon_runtime/11c/2026-08-26-sdf-page-hash-owner.md
  - docs/plans/optimize/zircon_runtime/11c/2026-09-09-image-bind-group-idle-retention.md
  - docs/plans/optimize/zircon_runtime/11c/2026-09-20-logical-text-batch-capacity.md
related_records:
  - docs/plans/astra/features/runtime/26-ui-image-bind-group-retention.md
  - docs/plans/astra/features/runtime/661-text-image-owner-and-contract-repair.md
  - docs/plans/astra/features/runtime/669-render-extract-command-capacity.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/859-logical-text-batch-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Runtime11C · text/atlas optimization completion list

This list records the ten implementation-complete Runtime11C text and atlas
optimization slices. They preserve deterministic page/slot ordering, glyph
identity, cache generation semantics, dirty-page reporting, route priority,
image bind-group retention, and logical-text output while reducing scans,
clones, and avoidable allocations. Tooling migration remains deferred.

| Plan slice | Optimization boundary | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Auto-text recency projection | Project recency without identity-cloning/sort-map detours. | `RUNTIME11C_AUTO_TEXT_RECENCY_SORT_PROJECTION_BENCH_V1`, projected P95 ≤60% of legacy. | implemented_pending_validation |
| Glyph allocator hash index | Resolve persistent page allocators through `HashMap`, retaining ordered diagnostics. | `RUNTIME11C_GLYPH_ALLOCATOR_HASH_INDEX_BENCH_V1`, hash P95 ≥30% below ordered lookup. | implemented_pending_validation |
| Glyph page single-pass index | Build one occupancy summary and use inline mask/bitmap fallback instead of repeated scans. | `RUNTIME11C_GLYPH_PAGE_SINGLE_PASS_INDEX_BENCH_V1`, P95 ≥90% reduction. | implemented_pending_validation |
| Icon request hash dedup | Deduplicate icon requests through hash admission while sorting the final plan deterministically. | `RUNTIME11C_ICON_REQUEST_HASH_DEDUP_BENCH_V1`, P95 ≥30% reduction. | implemented_pending_validation |
| Moved SDF async batches | Move owned batches into async dispatch without cloning payload vectors. | Allocation/parity model and managed async-batch tests. | implemented_pending_validation |
| Page-shadow hash index | Use persistent hash indexes for page generation/shadow residency while keeping ordered zero-init output. | `RUNTIME11C_PAGE_SHADOW_HASH_INDEX_BENCH_V1`, hash P95 ≤60% of ordered. | implemented_pending_validation |
| Borrowed SDF cache accounting | Borrow glyph keys in retained/dirty-page accounting and keep deterministic ordered projections. | `RUNTIME11C_SDF_ATLAS_BORROWED_CACHE_ACCOUNTING_BENCH_V1`, borrowed P95 ≤60%. | implemented_pending_validation |
| SDF page hash owner | Hash page ownership lookups while retaining ordered dirty-page projections. | `RUNTIME11C_SDF_PAGE_HASH_OWNER_BENCH_V1`, hash P95 ≥30% reduction. | implemented_pending_validation |
| Image bind-group idle retention | Retain idle UI image bind groups through bounded cache policy. | Existing Runtime26 record and managed WGPU/allocator evidence. | implemented_pending_validation |
| Logical text batch capacity | Reserve logical text batches from authoritative bounds. | `RUNTIME859_LOGICAL_TEXT_BATCH_CAPACITY_BENCH_V1` and capacity tests. | implemented_pending_validation |

Current source snapshots for the seven directly inspected Runtime11C owners are
recorded below. The grouped Runtime package validation covers all ten slices;
no per-plan Cargo invocation is started and no asynchronous result is inferred.

| Owner | SHA-256 |
| --- | --- |
| `zircon_runtime/src/text/atlas/slot_cache.rs` | `DB68F5AC05A9DBC2A4D3DD005337937D62B705F2CA417AD2BDFE6112EB8B94DF` |
| `zircon_runtime/src/text/atlas/page_residency.rs` | `E3392CA2E9C5D38E054E8E737587067C06900E9B467116CD1BD820B97E4B75D4` |
| `zircon_runtime/src/text/atlas/page_shadow/store.rs` | `32A4CB0CF0FD9A0FB07B977B4174E3AD050251B5C42C75FDB2D11DA3CAC200CC` |
| `zircon_runtime/src/ui/icon_atlas/atlas.rs` | `06B1FBC5985B37ACC529E55E3187AE27291C57D0F3F8DB764A9AF500C370A0EB` |
| `zircon_runtime/src/text/sdf/font_bake/atlas_pages.rs` | `12CB7D63AB1456C03AECC4E12CF98B57AD9D0144C8DA0628DD8ACE9B7554AEF0` |
| `zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches/auto_route.rs` | `4053903CD07D424A1680EC714698C25244BA6D4BBBEA67B008B1FB8BF52E3128` |
| `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs` | `33B39FD4CE95A3E4959F4833DD63FE157F0E1EE1CED6F6D647B916423162DE5F` |
