---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/cache.rs
  - zircon_runtime_interface/src/ui/surface/render/parity.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
implementation_files:
  - zircon_runtime_interface/src/ui/surface/render/batch/plan.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/key.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/clip.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md
tests:
  - zircon_runtime_interface/src/ui/surface/render/batch/tests.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
doc_type: module-detail
---

# UI Batch Ordering and Clipping Contract

## Contract

`UiBatchPlan` first orders paint elements by `(layer, paint_order, source_index)`. A layer is ordering data, not batch-key data: only adjacent elements in the same layer can merge. Each batch retains its exact extraction `source_indices`, so cache, diagnostics, parity snapshots, and visualizers must not infer source ownership from an ordered range.

`UiBatchKey` carries a structured `UiClipState`, rather than a formatted string, plus shader/resource/text backend/draw-effect/opacity state. Equivalent clip states are interned once for a plan; a changed mode or frame is a batch boundary. Nested axis-aligned scissor clips intersect before becoming the active state; no intersection becomes a zero-area scissor, never an unclipped child. Non-scissor clipping remains a distinct state and is not reduced to a scissor approximation.

## Ownership and Constraints

This is the backend-neutral interface contract. Runtime extraction produces the ordered plan; renderer backends may map canonical clip states to native scissor or stencil objects, but no backend object crosses `zircon_runtime_interface`. Atlas allocation, vertex assembly, and persistent GPU resource submission remain later Plan 21 slices.

## Relevant Source and Validation

The Plan 21 S1 tests cover layer ordering, adjacent merge behavior, clip-state separation, and nested scissor intersection. The existing render-contract suite checks that debug, cache, parity, and visualizer projections continue to identify the same source paint elements after ordering.
