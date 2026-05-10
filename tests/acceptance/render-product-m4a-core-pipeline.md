# Render Product M4A Core Pipeline

## Scope
- M4A Core Pipeline Phases And Submit Closure from `docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md`.
- Affected layers: neutral Core2d/Core3d pipeline contracts, production world-to-frame phase extraction, render pipeline compile validation, compiled-scene graph-stage execution, and direct `RenderFrameExtract` submit behavior.

## Baseline
- M3A asset/material readiness was accepted before this gate.
- M4A initially had review gaps: production world extraction only built opaque mesh phase inputs, Core2d declared `Transparent2d`/`Ui`/`Overlay` without graph-stage execution coverage, and renderer stage declarations were not forced to match `RenderPipelineAsset.phase_mapping`.
- The checkout remains heavily dirty with unrelated retained UI, AccessKit, reflection, and ECS work; this record covers narrowed M4A runtime/render gates, not full workspace acceptance.

## Test Inventory
- Framework phase cases: `zircon_runtime/src/core/framework/tests.rs` covers Core2d/Core3d phase ordering and camera projection-to-core-pipeline selection.
- Submit cases: `zircon_runtime/src/graphics/tests/render_product_submit.rs` covers direct extract authority, unknown viewport errors without panic, extract-selected Core2d/Core3d default pipeline resolution, and quality-profile override mismatch rejection.
- Pipeline compile cases: `zircon_runtime/src/graphics/tests/pipeline_compile.rs` covers Core2d/Core3d compile behavior, core-pipeline mismatch rejection, AlphaMask3d graph coverage, and missing product phase mapping rejection.
- Production extract case: `zircon_runtime/src/scene/tests/ecs_schedule.rs` covers prepared world extraction queueing alpha-mask and transparent meshes from `MeshRenderer.material_alpha_mode` hints.
- Renderer execution guard: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs` has a focused test proving Core2d `Transparent2d`, `Ui`, and `Overlay` graph stages are in the execution lists while avoiding duplicate early `AlphaMask3d` execution.

## Tooling Evidence
- Tool name: Windows Cargo through PowerShell.
- Why these tools were used: M4A acceptance concerns Rust compile/test behavior in runtime render contracts, pipeline compilation, submit routing, and renderer-stage scheduling.
- WSL-specific tools were not used for this narrowed gate because no platform-specific crash, memory, or thread-safety failure was observed; Linux CI-parity remains a later full-workspace gate.
- Exact commands:
- `cargo test -p zircon_runtime --locked render_product_pipeline --message-format short`
- `cargo test -p zircon_runtime --locked render_product_submit --message-format short`
- `cargo test -p zircon_runtime --locked pipeline_compile --message-format short`
- `cargo test -p zircon_runtime --locked prepared_render_frame_extract_queues_meshes_from_mesh_renderer_alpha_hints --message-format short`
- `cargo test -p zircon_runtime --locked compiled_scene_graph_stage_lists_cover_core2d_product_stages --message-format short`
- `cargo test -p zircon_runtime --locked pipeline_compile_rejects_declared_renderer_stage_missing_product_phase_mapping --message-format short`
- `cargo check -p zircon_runtime --lib --locked --message-format short`
- Final post-review gate:
- `cargo test -p zircon_runtime --locked render_product_pipeline --message-format short && cargo test -p zircon_runtime --locked render_product_submit --message-format short && cargo test -p zircon_runtime --locked pipeline_compile --message-format short && cargo check -p zircon_runtime --lib --locked --message-format short`

## Results
- M4A phase gate passed on 2026-05-09: 2 focused `render_product_pipeline` tests passed, 0 failed.
- M4A submit gate passed on 2026-05-09: 4 focused `render_product_submit` tests passed, 0 failed.
- M4A pipeline compile gate passed on 2026-05-09: 37 focused `pipeline_compile` tests passed, 0 failed.
- Production alpha-hint extract regression passed on 2026-05-09: 1 passed, 0 failed.
- Core2d graph-stage execution guard passed on 2026-05-09: 1 passed, 0 failed.
- Missing phase-mapping regression passed on 2026-05-09: 1 passed, 0 failed.
- Scoped runtime library check passed on 2026-05-09: `cargo check -p zircon_runtime --lib --locked --message-format short` finished successfully.
- Final post-review gate passed on 2026-05-09: `render_product_pipeline`, `render_product_submit`, `pipeline_compile`, and runtime library check all completed successfully.
- Warnings observed during test compilation were unrelated dead-code warnings for `new_with_virtual_geometry_for_test` and `new_for_test`.

## Review Evidence
- Spec review result: passed with no findings.
- Quality review initially found duplicate `AlphaMask3d` graph-stage execution and missing component-source doc frontmatter.
- Quality fixes removed early `AlphaMask3d` execution and added `zircon_runtime/src/scene/components/scene.rs` to scene render-extract docs.
- Quality re-review result: approved with no findings.

## Acceptance Decision
- Accepted for narrowed M4A Core2d/Core3d phases and direct submit closure based on the focused phase, submit, compile, production extract, renderer execution, review, and scoped runtime library evidence listed above.
- Remaining risks: full workspace build/test, plugin workspace validation, WSL/Linux CI parity, direct synthetic `ViewportChanged` race coverage, M4B postprocess graph productization, M5A PBR/light renderer integration, sprite, AA, VG/HGI, and Solari remain open later milestones.
