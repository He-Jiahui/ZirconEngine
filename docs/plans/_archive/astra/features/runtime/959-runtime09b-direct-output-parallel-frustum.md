---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-27-single-allocation-parallel-frustum.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
tests:
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
---

# Runtime959 · direct-output parallel frustum

The parallel frustum path now uses the ordered `parallel_map_indices` primitive
to construct `MeshFrustumVisibility` directly at the source index. The serial
fallbacks, output order, stable keys, and visibility results are unchanged, so
the former intermediate work-item projection is not recreated.

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime09B | direct indexed parallel output for frustum visibility | `implemented_pending_validation` | Parallel/serial order-and-result regression and camera-test construction contract are in-file. The independent model reports one physical allocation retained, allocated bytes 4,194,304→2,097,152 (`50%`), and four-run worst-case P50/P95 reductions of `57.689%`/`53.528%`. |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs` | `C044046A1A989CB944AE7811244A5E883223349CD70B9AF6537F0E7E9A40BC6C` |

## Validation handoff

The exact-file formatting and source contracts are included in the grouped
Runtime request. Managed Cargo and current-source Release/product gates remain
pending; no allocation reduction beyond the measured byte reduction is claimed.
