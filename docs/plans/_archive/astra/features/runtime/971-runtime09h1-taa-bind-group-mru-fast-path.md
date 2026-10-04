---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09h1/2026-08-26-taa-bind-group-mru-fast-path.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/972-runtime09d-to-09h1-completion-list.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_bind_group_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_bind_group_cache/mru_tests.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_bind_group_cache/mru_tests.rs
---

# Runtime971 · TAA bind-group MRU fast path

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09H1 TAA resolve cache | Stable frames check the LRU tail directly and return the cloned WGPU bind group without scanning or relocating the eight-entry deque. Non-MRU hits retain linear lookup and move-to-tail behavior; invalidation and miss-only creation remain unchanged. | The focused tests compare stable and non-MRU recency order and lock the direct tail branch. Stable modeled comparisons fall from `32,768` to `4,096` and relocations from `4,096` to `0`; the ignored Release gate requires at least 50% P95 reduction. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_bind_group_cache.rs` | `DF432F554F3EF82EAF6D0ED7AB58D2C29B8E3C8CAA22A17185BF9119431A14F8` |
| `zircon_runtime/src/graphics/scene/scene_renderer/temporal/taa/taa_resolve_bind_group_cache/mru_tests.rs` | `2C3F79B8DE0F76F2C98A60C20FDCD3560F3F6C22D9435D01B628C13B0FFE4188` |

## Validation handoff

The TAA cache tests and `RUNTIME09H1_TAA_BIND_GROUP_MRU_FAST_PATH_BENCH_V1`
marker are included in the grouped Runtime package validation. Managed Release
P50/P95 and product temporal-reconstruction evidence remain pending.
