---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs
plan_sources:
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_draw_command_list.rs
doc_type: module-detail
---

# Mesh Draw Command List

`mesh_draw_command_list.rs` is the narrow owner for sorted mesh command containers. It owns `MeshDrawCommandList`, `MeshPassCommandBuffers`, per-phase count/stat projection, indirect batch summaries, and the shared command ordering helpers.

`mesh_draw_command_list/builder.rs` owns batch expansion into command buffers. It fans `MeshBatchRef` entries through the pass processors, performs static command cache lookup/rebuild, appends dynamic commands, and projects cache stats into `MeshPassCommandBuffers`.

`mesh_draw_command_list/tests.rs` owns the behavior tests formerly inline in the parent file: command ordering and stats, GPUScene instance source projection, per-phase command buffer construction, indirect batch stats, pipeline variant assignment, static cache reuse, and invalidation reason reporting.

The current structure slice is recorded as `Plan 02 mesh draw command list owner split` with status `render_plan02_mesh_draw_command_list_owner_split_static_passed_cargo_lock_blocked`. `runtime_15_mesh_draw_command_list_is_folder_backed` guards that builder/cache logic and tests do not return to the parent, and that all three owners stay within their line budgets.
