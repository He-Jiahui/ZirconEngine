---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-27-indexed-virtual-geometry-execution-projection.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs
tests:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs
---

# Runtime960 · indexed virtual-geometry execution projection

`ExecutionLookup` now builds frame-local cluster-to-instance, stable-key-to-
sorted-unique-cluster, and composite cluster-key indexes once. Execution
segments and selected-cluster expansion reuse those indexes instead of scanning
all clusters and instances for each segment, while preserving first-match,
legacy stable-key, ordinal, and missing-row behavior.

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime09B | shared indexed projection for virtual-geometry execution and selected-cluster expansion | `implemented_pending_validation` | Overlap, duplicate first-match, sorted/deduplicated ordinal, and legacy-key regressions are in-file. The independent model reports allocations 2,561→73 (`97.15%`), P50 2,713,400→356,200ns (`86.87%`), and P95 6,414,900→1,238,600ns (`80.69%`); the in-repository ignored benchmark retains the 60% P95 gate. |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs` | `F552EEE90C30E151B557567FA84C67251A2B75D3065C4FEB6DBFFA671AD4BC31` |

## Validation handoff

Formatting, source contracts, focused regressions, and the Release model are
part of the grouped Runtime admission. Managed Cargo, allocator, product, and
current-source Release gates remain pending.
