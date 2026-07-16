---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/advanced_materials.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/advanced_materials.rs
plan_sources:
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-14-render-owner-budget-splits.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/advanced_materials.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_draw_command_list.rs
doc_type: module-detail
---

# Mesh Draw Command List

`mesh_draw_command_list.rs` is the narrow owner for sorted mesh command containers. It owns `MeshDrawCommandList`, `MeshPassCommandBuffers`, per-phase count/stat projection, indirect batch summaries, and the shared command ordering helpers.

`mesh_draw_command_list/builder.rs` owns batch expansion into command buffers. It fans `MeshBatchRef` entries through the pass processors, performs static command cache lookup/rebuild, appends dynamic commands, and projects cache stats into `MeshPassCommandBuffers`.

`mesh_draw_command_list/tests.rs` owns the shared fixtures plus command ordering and stats, GPUScene instance source projection, per-phase command buffer construction, indirect batch stats, and pipeline variant assignment. `mesh_draw_command_list/tests/cache.rs` owns static-cache reuse/invalidation, while `mesh_draw_command_list/tests/advanced_materials.rs` owns advanced PBR/transmission phase-order coverage.

The current structure slice preserves `Plan 02 mesh draw command list owner split` with status `render_plan02_mesh_draw_command_list_owner_split_static_passed_cargo_lock_blocked` and adds the Runtime15 follow-up `runtime_15_render_owner_budget_split_current_source_managed_build_passed`. `runtime_15_mesh_draw_command_list_is_folder_backed` guards that builder/cache logic and the cache/advanced-material test families do not return to the parent; the root and both children have independent focused budgets.
