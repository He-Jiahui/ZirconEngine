---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/213-runtime-visibility-gpu-scene-culling-batching-instancing-hzb-virtual-geometry-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/625/2026-09-01-preallocated-frame-batching-collections.md
related_records:
  - docs/plans/astra/features/runtime/21-visibility-shadow-view-admission.md
  - docs/plans/astra/features/runtime/22-visibility-query-normalization.md
  - docs/plans/astra/features/runtime/35-20260901-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/culling/mesh_bounds.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct/tests.rs
tests:
  - zircon_runtime/src/graphics/visibility/culling/mesh_bounds.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct/tests.rs
---

# Runtime961 · prepared local-bounds projection and fail-open visibility

The visibility owner accepts an optional prepared local mesh-bounds slice and
projects a valid off-center local sphere through the full affine transform.
Missing, invalid, non-finite, or overflowed bounds fail open with a finite
center and `f32::MAX` radius so CPU visibility cannot discard a resource while
its bounds generation is incomplete. The batching owner also reserves its
known extract capacity and keeps the prepared-bounds path explicit; the broader
canonical bounds/generation handoff remains parent-plan work.

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime213 / Runtime625 | prepared local bounds projection, fail-open validity policy, and reserved visibility extraction outputs | `implemented_pending_validation` | Off-center affine, missing/invalid/overflow, degenerate-scale, prepared projection, unchanged-bounds, changed-bounds, disabled-shadow, and visibility query regressions are present. This record does not claim closure of the broader VIS213 canonical-bounds/product gate. |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/graphics/visibility/culling/mesh_bounds.rs` | `947E095A6A8F2ED6CB68C97F982261ACCE3722BEB280EBCF4A63B80DECA4A7B7` |
| `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs` | `A831D187ADC842D36CE5FB4E32ED0F1884F6F40A95FC196E33330F29D0B26B54` |
| `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs` | `A17035A96FC562EA6A426A3942B00E8832DE221C5B0065662FD9621B33B07506` |
| `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct/tests.rs` | `672155393CA057F5FD111367E21A7C61E2A732D929398D53178E681F1ACA2FE4` |
| `zircon_runtime/src/graphics/visibility/view_context/build_views.rs` | `BB4990D77FF79010CDD80A07EA4F5DC2515FD54A4668FE7A53EFC0A246A6273` |
| `zircon_runtime/src/graphics/visibility/view_context/build_views/capacity_tests.rs` | `2917BF65E5836A3E369A8FEC5B043E184CD12CEFE854804CB354696A4FED7FEF` |

## Validation handoff

Exact-file formatting, source contracts, and scoped diff checks are included in
the grouped Runtime submission. Managed Cargo, current-source Release, GPU
parity, allocator, and product gates remain pending.
