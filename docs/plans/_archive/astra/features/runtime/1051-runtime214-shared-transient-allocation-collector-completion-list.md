---
doc_type: feature-completion
status: source_candidate_pending_validation
validation_status: focused_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
plan_sources:
  - docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/214/2026-09-29-shared-transient-allocation-collector.md
implementation_files:
  - zircon_runtime/src/render_graph/graph/transient_allocation.rs
tests:
  - zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs
---

# Runtime1051 / Runtime214 shared transient collector completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Remove separate buffer allocation collector | Texture and buffer bucket phases extend one result vector, each reserving its exact successful output count first. Final sorting and allocation ID order remain. | Rustfmt and scoped diff checks passed; the existing mixed texture/buffer allocation regression has not run in the grouped Runtime batch. | source_candidate_pending_validation |
| Runtime214 G14 performance | The intermediate buffer collector and its final copy are removed; the shared vector may still relocate texture entries when buffer capacity is added. | Allocator/RSS, Release p50/p95/p99, fixed-scene comparison, hardware profile, and an explicit ceiling remain open. | open |

This is a bounded allocation source candidate, not acceptance of Runtime214's product performance gate.
