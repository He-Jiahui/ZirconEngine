---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/frame_hit_test/tests.rs
tests:
  - zircon_runtime/src/ui/surface/frame_hit_test/tests.rs
---

# Popup Projection Stack Capacity

`UiSurface::popup_hit_test_projections` now reserves its projection vector from the retained
popup-stack length before walking the stack. The explicit loop preserves stack order, skips popup
entries without a node or arranged geometry, and computes the same anchored/non-anchored target
frames as before. The change removes avoidable vector growth during projected hit-index rebuilds
without changing popup geometry, ordering, or filtering semantics.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11A / popup hit authority | Reserve projected popup entries from the popup-stack bound | implemented_pending_validation | Source regression asserts bounded reservation, stack-order iteration, invalid-entry filtering, and explicit projection insertion. Scoped Rustfmt, diff check, and the batched Runtime/Editor static contracts pass `82/82`. Managed Runtime Cargo and release p50/p95/p99 allocation evidence remain pending; no coordinator state was polled. |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/ui/surface/frame_hit_test.rs` | `0372EA1F9F9CD5F572CFD5007F34AC8EA8A2F058315704EF568C1419F00C19F2` |
| `zircon_runtime/src/ui/surface/frame_hit_test/tests.rs` | `55E7E2D7C17B24587EED2859642489ADD7440EF89B52882C4DBB3A97675AC8F1` |
