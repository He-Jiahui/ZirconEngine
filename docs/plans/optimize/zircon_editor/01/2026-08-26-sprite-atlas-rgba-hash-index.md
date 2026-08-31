---
title: Editor01 Sprite Atlas RGBA Hash Index
category: zircon_editor
report_id: Editor01-sprite-atlas-rgba-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Sprite Atlas RGBA Hash Index

## Scope

This slice replaces both ordered indexes in the sprite-atlas RGBA cache with `HashMap`: texture
path to decoded pixels, and resource key to generation/path metadata. Stable command-stream pixel
lookup now performs two expected constant-time lookups instead of two ordered key traversals.

The 64-entry and 64 MiB bounds, complete resource/path keys, generation validation, resident-byte
accounting, decoded pixel ownership, and invalidation contract are unchanged. Capacity misses still
evict the lexicographically smallest texture path through `keys().min()`, preserving the previous
deterministic policy. The existing `Vec<u8>` copy API remains unchanged because its consumer has
independent in-flight edits; eliminating that deep copy is a separate follow-up.

## Performance Workload

The release workload fills both 64-entry indexes with keys sharing a long prefix and performs 4,096
stable resource-to-pixel lookups.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered-index lookups | 8,192 | 0 |
| Hash-index lookups | 0 | 8,192 |
| Pixel-buffer copies in index benchmark | 0 | 0 |
| Capacity or eviction-policy changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_SPRITE_ATLAS_RGBA_HASH_INDEX_BENCH_V1`. Acceptance requires the two-stage HashMap lookup
P95 to be at least 30% below the legacy BTreeMap path. Exact Windows P50/P95 timings remain pending
the coordinator run.

## Acceptance

- `optimization_batch_20260826br_sprite_atlas_rgba_hash_index_preserves_generation_removal`
  covers synchronized removal, generation metadata, and resident-byte accounting.
- `optimization_batch_20260826br_sprite_atlas_rgba_hash_index_preserves_deterministic_eviction`
  locks lexicographically smallest-path eviction.
- `optimization_batch_20260826br_sprite_atlas_rgba_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges sprite-atlas RGBA
indexing; shared pixel-return ownership remains open.
