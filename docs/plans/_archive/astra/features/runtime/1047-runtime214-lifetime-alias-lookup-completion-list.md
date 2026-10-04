---
doc_type: feature-completion
status: source_candidate_pending_validation
validation_status: focused_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
plan_sources:
  - docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/214/2026-09-29-lifetime-alias-index-lookup.md
implementation_files:
  - zircon_runtime/src/render_graph/builder/compile.rs
tests:
  - zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs
---

# Runtime1047 / Runtime214 lifetime alias lookup completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Avoid per-access alias parent resource scans | `resource_lifetimes` reuses its resource declaration index to resolve each live texture-view alias parent and extend its backing lifetime. Expected lookup work changes from O(resources x accesses) repeated scans to O(resources + accesses) including index construction. | Rustfmt and scoped diff checks passed; the existing alias-lifetime regression has not run in the managed Runtime batch. | source_candidate_pending_validation |
| Runtime214 G14 performance | The lookup has a source-level complexity improvement only. | Release p50/p95/p99, allocation/RSS, fixed-scene comparison, hardware profile, and an explicit ceiling remain open. | open |

This is a narrow source candidate. It does not close Runtime214's lifetime architecture or product performance gate.
