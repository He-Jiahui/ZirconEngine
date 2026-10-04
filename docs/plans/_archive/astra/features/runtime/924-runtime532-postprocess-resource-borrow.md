---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime532-postprocess-resource-borrow.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs
---

# Runtime924 Runtime532 post-process resource borrowing

Post-process graph execution now traverses compiled input/output resource names
by reference through the read-only GPU resolver instead of cloning vectors and
strings before validation. Resource-kind routing remains unchanged.

Marker `RUNTIME532_POSTPROCESS_RESOURCE_BORROW_BENCH_V1` reports the removed
temporary owned-allocation model. Managed Release validation remains pending.
