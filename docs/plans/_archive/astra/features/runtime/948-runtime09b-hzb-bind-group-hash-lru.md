---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-hzb-bind-group-hash-lru.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/bind_group_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/bind_group_cache/hash_lru_tests.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/bind_group_cache/hash_lru_tests.rs
---

# Runtime948 · HZB bind-group hash LRU hit path

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09b HZB occlusion bind-group cache | Stable hits remain indexed by the sampled-HZB plus indirect-resource key and retain monotonic-generation LRU behavior. The hit path now uses one `HashMap::get_mut` probe instead of `contains_key` followed by `get_mut`; miss creation, 64-entry eviction, revision identity, and overflow rebasing remain unchanged. | The release marker model reports `262,144` legacy key comparisons versus `4,096` optimized hash lookups (`98.4375%` deterministic reduction), P50 `137,100ns→56,100ns`, and P95 `216,200ns→75,900ns` (`64.89%` reduction, above the plan's 50% gate). The current-source contract now rejects the redundant `contains_key` probe; grouped managed Release evidence remains pending. | implemented_pending_validation |

## Deterministic boundary

The cache still creates no bind group on a stable hit. `last_used` advances for
every hit, and full-cache eviction still removes the oldest generation before a
new bind group is inserted. Sampled-resource and indirect-resource revisions
remain part of the key, and overflow rebasing preserves oldest-first order.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/bind_group_cache.rs` | `A8050FEC983186062EFCCB0228816CA8A79C9944E5728EE01982ABECBAF2BDF9` |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/bind_group_cache/hash_lru_tests.rs` | `56066C33B82740E8E92D210B2E7BA5BEE73D7805B7F428509C89E846981058DD` |

## Validation handoff

The existing Release test executable supplied the independent HZB marker
receipt, but its source-contract test was compiled before the current
single-probe repair and therefore is not treated as a current-source package
receipt. The production contract was tightened and repaired in the worktree;
the final-source grouped Runtime/Editor managed batch (Runtime PTY `54276`,
Editor PTY `15395`) will compile the current source. No per-task Cargo run is
started. Managed Release allocation/P50/P95 and renderer-product gates remain
pending.
