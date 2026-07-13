---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/current_structure_owner_inventory.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/geometry.rs
  - zircon_runtime/src/graphics/text/layout/rich.rs
  - zircon_runtime/src/graphics/text/layout/rich/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate/tests/support.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - current-source standalone structure_convention 1304/1304
  - seven historical priority structure gates 7/7
---

# Runtime 15 file-budget convergence

Runtime 15 treats file budgets as ownership boundaries rather than formatting targets. A production owner stays below 800 lines by moving complete responsibilities—tests, report extraction, row groups, or status maps—into folder-backed children. Test and status owners follow narrower per-owner budgets and keep parent files as route, aggregation, or compact status-mirror owners.

The 2026-07-11 convergence closes the prior 54 production and 448 test budget failures. Production behavior was not bypassed: `source_cubemap.rs` delegates its inline tests to `source_cubemap/tests.rs`, while the GPU execution-context root delegates its focused test to `gpu/tests.rs`. Status-output rows were split by coherent topics and reassembled with stable slice ordering; UI checks now read parent routes together with their child row trees.

Plan and documentation evidence follows the numbered-output rule. Parent plans remain current overview/routing authorities. Volatile guard paths and exact result identities live in `docs/plans/zircon_runtime/runtime/15/` numbered records, and the standalone harness resolves those archives without copying concrete output rows back into parent plans.

The 2026-07-12 priority follow-up closes the remaining real over-budget owners without raising a limit. IBL bake planning, rich layout, and compute-workload tests now live in dedicated test children. UI border geometry belongs to `render/geometry.rs`; depth-clear recording belongs to the GPU surface child; froxel integration output/support helpers belong to `integrate/tests/support.rs`. The parent owners keep orchestration and contracts, and no moved behavior is re-exported through compatibility shims.

The current-source standalone harness was rebuilt after those hard splits. All seven historical priority gates passed, followed by the complete structure-convention suite at 1304/1304. This is structural validation only; Cargo package matrices, WGPU, RenderDoc, screenshots, and full CI keep their separate acceptance owners.
