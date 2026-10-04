---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-30-runtime-authored-geometry-delta-publication.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-10-taffy-child-handle-buffer-reuse.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
related_code:
  - zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/product_cache.rs
  - zircon_runtime/src/ui/surface/host_font_assets.rs
  - zircon_runtime/src/ui/surface/ecs_projection.rs
related_records:
  - docs/plans/astra/features/runtime/720-ecs-projection-node-capacity.md
  - docs/plans/astra/features/runtime/721-host-font-borrowed-dedup.md
  - docs/plans/astra/features/runtime/723-runtime-index-output-capacity.md
  - docs/plans/astra/features/runtime/725-runtime-visibility-single-pass-publication.md
  - docs/plans/astra/features/runtime/726-runtime-hit-route-parent-index-reuse.md
tests:
  - tools/tests/test_runtime_ui_architecture_boundary.py
  - tools/tests/test_runtime_ui_authored_geometry_delta_pressure.py
  - tools/tests/test_runtime_ui_legacy_screen_space_analytic_coverage_contract.py
  - tools/tests/test_runtime_ui_responsive_width_gate_contract.py
---

# Runtime UI Static Contract Alignment

The Runtime UI source contracts now match the current retained publication and
layout ownership. The architecture map records 23 top-level UI entries,
including the private `module/` lifecycle/driver owner, and 45 surface entries,
including `host_font_assets.rs` as the host-font admission and resident-lifetime
owner. The authored-geometry checks bind to the persistent hit-grid COW counters,
the dual outer/inner analytic coverage shader result, and clone-free responsive
definition comparison.

The retained Taffy parent-product cache adds private cache and exact-contract
identifiers. Its production-source scan therefore moves from 242 to 254 matching
lines while retaining the same 16-file execution surface; this is a current-source
baseline update, not a claim of additional Taffy execution work.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| RUI-37 | Align Runtime UI source-contract baselines and architecture mirrors with the retained publication/cache owners | implemented_pending_validation | The aggregate `test_runtime_ui_*.py` batch passed 445/445 after repair. Scoped diff checks are clean. Managed Rust compilation and Windows Release allocation/time p50/p95/p99 evidence remain pending; no product performance target is claimed. |

## Coordinator dispatch log

- 2026-09-10: coordinator accepted the sealed Runtime/Editor static-format batch as
  ticket `ccd5411532074737832a87c26bf5414b`. The snapshot includes the Runtime
  architecture test pair; it is intentionally not polled while independent
  repair work continues.
