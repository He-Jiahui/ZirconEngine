# Shader 04 global shader executor migration acceptance

Plan: docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md
Milestone: M3
Status: completed
Files: ["docs/plans/zircon_runtime/shader/04/2026-07-14-global-shader-executor-migration-acceptance.md", "docs/zircon_runtime/graphics/shader/global-pipeline-layout.md", "zircon_runtime/src/core/framework/render/shader/compute_dispatch.rs", "zircon_runtime/src/core/framework/render/shader/fullscreen_pass.rs", "zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs", "zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/fullscreen_pass.rs", "zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs", "zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mod.rs", "zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs", "zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs", "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/hzb.rs", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/motion_vector_tile_max.rs", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/hzb_pipeline.rs", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/motion_vector_tile_max_pipeline.rs", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/execute_hzb_build.rs", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_motion_vector_tile_max/execute_motion_vector_tile_max.rs", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/hzb_build.wgsl", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/hzb_build_msaa.wgsl", "zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/motion_vector_tile_max.wgsl", "zircon_runtime/src/graphics/shader/builtin_global_shader_contracts.rs", "zircon_runtime/src/graphics/shader/global_pipeline_layout.rs", "zircon_runtime/src/graphics/shader/mod.rs", "zircon_runtime/src/graphics/shader/wgsl/zr_fullscreen_triangle.wgsl", "zircon_runtime/src/render_graph/tests/resources.rs"]

| Milestone | Stage | Status | Evidence |
|---|---|---|---|
| M3 | SH04-M3-T testing | completed | Managed Windows build passed; fresh focused lib-test binary passed 28/28; scoped formatting/diff and handoff validation passed. |

## Scope delivered

- Added one shared built-in global shader contract owner and a typed WGPU layout projection so feature descriptors, layouts, executors, and WGSL no longer maintain separate binding tables.
- Migrated HZB single-sample and 4x MSAA compute paths to generated bindings 0/1/2/3, distinct pipeline identities, named executor lookup, and per-mip parameter updates.
- Migrated motion-vector tile-max to the fullscreen group1 input ABI and the shared `zr_fullscreen_triangle_vs` source, while deleting the feature-local fullscreen contract owner.
- Hard-cut all `ComputeDispatchBuilder` and `FullscreenPassBuilder` fluent methods to consuming `Self -> Self`, matching the June builder convention.
- Strengthened the HZB product regression with nonuniform 4x4 depth, distinct values for every MSAA sample, and exact furthest/closest/span assertions across both mip levels.

## Fresh testing evidence

- Managed Windows `zircon_runtime -SkipTest` build passed with exit code 0 after the Frameworks05 surface repair. The retained build log is `E:/ZirconBuilds/shader04-executor-validation-20260714/zircon-runtime-build-after-frameworks05.log`.
- A fresh managed lib-test binary passed 28/28 focused checks: shader builders 21/21, built-in global contracts 2/2, typed WGPU projection 3/3, real-adapter HZB readback 1/1, and the complete motion-blur product regression 1/1.
- The HZB readback test ran through a real WGPU adapter and passed in 1.42 seconds; the motion-blur product test passed in 48.72 seconds.
- Scoped `rustfmt --check`, scoped `git diff --check`, and the handoff validator all passed; the handoff validator reported 96 artifacts and 0 errors.
- Broad `cargo test -p zircon_runtime --locked` compiled the fresh library test binary, then stopped only in three foreign Runtime04 integration tests that still call retired asset APIs: `virtual_geometry_debug_snapshot_contract.rs`, `runtime_environment_external_cubemap_import_staging_contract.rs`, and `material_shader_redirect_dependency_contract.rs`. This broad package gate is not claimed green by this record.

## Review

- Independent review found no remaining blocker in this migration slice after the HZB regression was strengthened to detect sample-0-only resolves, stale per-mip parameters, and accidental rereads from scene depth.
- The implementation keeps global ABI generation in `core/framework`, backend texture details in `graphics`, and every touched production/test owner below the repository's 800-line review threshold.

## Remaining boundary

- This record completes only the SH04-M3 global executor migration slice. A reusable upload path for non-empty fullscreen group2 parameters and forwarding backend construction errors into `FrameDiagnostics` remain later SH04-M3 work.
- The record does not claim completion of the full Shader04 plan, the broad Runtime04 integration gate, RenderDoc capture, or the remaining product/performance sweep.
