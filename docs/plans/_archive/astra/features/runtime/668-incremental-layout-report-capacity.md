---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs
tests:
  - zircon_runtime/src/ui/tests/surface_dirty_domains/incremental_layout.rs
---

# Incremental Layout Report Capacity

The incremental layout selection-report fallback now reserves its temporary vectors from the
known report and visited-node bounds. Report merging reserves the sum of the previous and
incremental selection counts, while the in-place patch reserves the visited-node count. Selection
ordering, replacement validation, fallback behavior, and the published report remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime76/RUL-P2-004, RUL-P2-006 | Bound temporary selection-report merge and patch growth by existing pass inputs | implemented_pending_validation | Rust source regression asserts both bounded reservations. Production incremental rebuild Rustfmt and scoped diff checks pass; the test module retains pre-existing formatting drift outside this slice. Managed Runtime Cargo and release layout p50/p95/p99 and allocation evidence remain pending; no coordinator state was polled. |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs` | `0C9A039CA5506CA5D609DE5F58EC7583648BA453A773C588282166889744A0E7` |
| `zircon_runtime/src/ui/tests/surface_dirty_domains/incremental_layout.rs` | `1AE12E820D8591FFE08AFED3C92D0A2B73945D9E9B38753C19F539A994C2601A` |
