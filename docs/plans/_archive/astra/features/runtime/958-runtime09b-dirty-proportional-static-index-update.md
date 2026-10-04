---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-27-dirty-proportional-static-index-update.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs
tests:
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs
---

# Runtime958 · dirty-proportional static-index update

`VisibilityStaticIndex::apply_update_plan` now builds a temporary membership map
only for inserted and updated stable-instance keys, scans the current instance
slice once, and applies removals/last-row semantics without rebuilding a full
scene `BTreeMap`. The zero-change path skips the current-instance scan.

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime09B | changed-key `HashMap` membership replaces the full-scene ordered temporary map while retaining `O(N + K)` behavior and deterministic update semantics | `implemented_pending_validation` | `apply_update_plan` source contract, changed-key update regressions, and the ignored Release benchmark are present; the independent model reports 23,852→4 allocations, 13,158,272→17,472 bytes (`99.87%`), P50 13,400,500→4,304,800ns (`67.9%`), and P95 23,701,700→5,638,900ns (`76.2%`). |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/graphics/visibility/static_index/mod.rs` | `F4DBD37EE81AF6262CC0F558AC8FE5145A88DDED6A25D5E260F8E2F69405638` |

## Validation handoff

Rust formatting, scoped diff checks, source contracts, and the Release model
are admitted through the grouped Runtime validation request. Managed Cargo,
allocator, product, and current-source Release receipts remain pending; no
per-task managed run is started.
