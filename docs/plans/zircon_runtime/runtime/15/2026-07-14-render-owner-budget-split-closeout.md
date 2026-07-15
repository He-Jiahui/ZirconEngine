# Runtime 15 render owner budget split closeout

Plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
Milestone: M3
Status: completed
Files: ["docs/assets-and-rendering/render-framework-architecture.md", "docs/plans/zircon_runtime/runtime/15/2026-07-14-render-owner-budget-split-closeout.md", "docs/plans/zircon_runtime/runtime/15/2026-07-14-render-owner-budget-splits.md", "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.md", "docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.md", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/pipeline_resource_usage.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/sprite_stage_selection.rs", "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs", "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/advanced_materials.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/cache.rs", "zircon_runtime/src/tests/runtime_absorption/code_review_findings/render_structure.rs", "zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pass_gpu_context_mesh_command_lists.rs", "zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_draw_command_list.rs"]

## Scope

- Split mesh recording out of the GPU render-pass context owner.
- Split cache and advanced-material coverage out of the mesh command-list test owner.
- Split sprite-stage selection and shared graph-resource usage out of compiled-scene orchestration.
- Updated the focused structure guards and module documentation without compatibility shims or budget relaxation.

## Verification

- Managed Windows `cargo build -p zircon_runtime --locked`: job `9dac70c034fb4aa18155d370f77073e1`, exit 0.
- Coordinator ephemeral test lane `6ee2d5f2a9c34e7e80c1338b5887dd3a`: three exact structure guards passed 3/3, exit 0.
- Scoped `rustfmt --edition 2021 --check` and `git diff --check` are required by the final immutable-manifest gate.

## Review

- The slice changes ownership and source layout only; rendering behavior remains in the moved implementation.
- Runtime15 remains `in_progress`; this closeout covers only the Render18-triggered owner-budget repair.
