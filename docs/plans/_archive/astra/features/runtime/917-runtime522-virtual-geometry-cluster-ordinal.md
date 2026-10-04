---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime522-virtual-geometry-cluster-ordinal.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_ordinal.rs
tests:
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_ordinal.rs
---

# Runtime917 Runtime522 virtual-geometry cluster ordinal lookup

The sorted, deduplicated cluster-ID invariant is now used through
`binary_search` instead of a linear position scan. Missing clusters preserve
the existing ordinal-zero fallback.

Marker `RUNTIME522_VIRTUAL_GEOMETRY_CLUSTER_ORDINAL_BENCH_V1` reports the
32,768-look-up comparison model (4,096 clusters). Managed Release validation
and the coordinator performance receipt remain pending.
