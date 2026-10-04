---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-assets-activity-indexed-layout.md
  - docs/plans/optimize/zircon_editor/01/2026-08-26-reflection-builder-dense-nodes.md
  - docs/plans/optimize/zircon_editor/01/2026-08-26-single-pass-surface-metadata-apply-merge.md
  - docs/plans/optimize/zircon_editor/01/2026-08-26-single-pass-surface-metadata-index-merge.md
  - docs/plans/optimize/zircon_editor/01/2026-08-26-sprite-atlas-manifest-hash-index.md
related_records:
  - docs/plans/astra/features/editor/665-activity-registry-hash-index.md
  - docs/plans/astra/features/editor/980-editor01-showcase-action-match.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/layouts/views/assets_activity/content_layout.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/content_layout/indexed_lookup_tests.rs
  - zircon_editor/src/ui/reflection/builder.rs
  - zircon_editor/src/ui/reflection/builder/dense_node_tests.rs
  - zircon_editor/src/ui/template_runtime/model.rs
  - zircon_editor/src/ui/template_runtime/model/optimization_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas/cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas/cache/hash_index_tests.rs
tests:
  - zircon_editor/src/ui/layouts/views/assets_activity/content_layout/indexed_lookup_tests.rs
  - zircon_editor/src/ui/reflection/builder/dense_node_tests.rs
  - zircon_editor/src/ui/template_runtime/model/optimization_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas/cache/hash_index_tests.rs
---

# Editor01 · index/dense/merge completion list

These five implementation-complete Editor01 slices preserve first-match
control lookup, reflection ordering, metadata last-write-wins precedence,
sprite-atlas negative caching and deterministic eviction while eliminating
repeated scans, construction-time ordered-map work, and temporary metadata
maps.

| Plan slice | Optimization boundary | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Assets Activity indexed layout | Build one capacity-sized control-ID index and reuse it for all row/metadata frame writes. | `EDITOR01_ASSETS_ACTIVITY_INDEXED_LAYOUT_BENCH_V1`; indexed P95 ≤60% of linear lookup. | implemented_pending_validation |
| Reflection builder dense nodes | Resolve parent mutation by dense `UiNodeId` slot and project the ordered map once at `finish`. | `EDITOR01_REFLECTION_BUILDER_DENSE_NODES_BENCH_V1`; dense construction P95 ≥30% below the legacy map path. | implemented_pending_validation |
| Surface metadata index merge | Copy source entries directly into index-owned maps without cloning temporary maps per node. | `EDITOR01_SINGLE_PASS_SURFACE_METADATA_INDEX_MERGE_BENCH_V1` plus preorder/last-write tests. | implemented_pending_validation |
| Surface metadata apply merge | Extend caller-owned target maps directly from immutable index entries. | `EDITOR01_SINGLE_PASS_SURFACE_METADATA_APPLY_MERGE_BENCH_V1`; temporary-map count remains zero. | implemented_pending_validation |
| Sprite-atlas manifest hash index | Use a bounded `HashMap` for stable hits while retaining negative caching and lexicographic miss eviction. | `EDITOR01_SPRITE_ATLAS_MANIFEST_HASH_INDEX_BENCH_V1`; hash P95 ≥30% below ordered lookup. | implemented_pending_validation |

## Source snapshots

| Owner | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/views/assets_activity/content_layout.rs` | `3E6CB488338755771B57F10E289F34634EF7BD6FBBC22B2809F0B3CB99246469` |
| `zircon_editor/src/ui/layouts/views/assets_activity/content_layout/indexed_lookup_tests.rs` | `481CCCF5CE64A46E0DAF374589565A1150F9BAA1ED75B1BCDB46343211233D4F` |
| `zircon_editor/src/ui/reflection/builder.rs` | `63877986CDAEC9F0E3DBFD3A76B93F3AA0E78DCFC333FB900D49A079ED3701DD` |
| `zircon_editor/src/ui/reflection/builder/dense_node_tests.rs` | `19312D69D977252DC8DA266F2F7EB31BF3AD1464691C3EC45F959733A0FF7868` |
| `zircon_editor/src/ui/template_runtime/model.rs` | `EFCC714D7DE5CE70DDA30C696BA161AC7F1A1309BD6429911F0CFCF47B96F1EF` |
| `zircon_editor/src/ui/template_runtime/model/optimization_tests.rs` | `C7E4E9C3C946A5D360CB43173FB2002E42543952EF869D57C81F70387D27295` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas/cache.rs` | `EF0B677ADD056D4B6117FF1B787A525FF83D6A35E1DAF7511B7BC636386F404D` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas/cache/hash_index_tests.rs` | `212946C01FE0F5ED82A229A5269CF4553634B5E53DCA434E6C2CE9E88D10830F` |

The source contracts and focused behavior/marker tests are included in the
grouped Editor validation wave. Managed Cargo, Release p50/p95, and product
layout/paint gates remain pending; this record makes no dynamic acceptance
claim.
