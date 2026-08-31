---
related_code:
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/camera_matrices/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_reflection_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/camera_matrices/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_reflection_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
plan_sources:
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - user: 2026-06-14 implement WGPU render pipeline architecture code and update plan progress
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs::tests::post_process_projection_params_ignore_temporal_jitter
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::tests::scene_uniform_inverse_view_projection_is_unjittered
  - cargo fmt --package zircon_runtime -- --check
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s2-0614 --message-format short --color never
doc_type: module-detail
---

# Post-Process Projection

Post-process screen-space effects consume camera projection data differently from raster passes. Raster mesh, sprite, particle, prepass, deferred geometry, overlay, and shadow atlas writers should keep the jittered scene uniform `view_proj` so the current frame is rasterized at the temporal AA sample position. Screen-space reconstruction, reprojection, and history sampling must not inherit that sample offset.

Plan 06 TP-M2-S2 locks that split in two places:

- `SceneUniform::inverse_view_proj` is now the inverse of current `view_proj_unjittered`. Deferred lighting reconstructs world position from depth through this field, so the lighting pass receives stable unjittered positions even when TAA jitter is active.
- Post-process SSR does not read the scene uniform matrix. `PostProcessParams.effect_projection` stores perspective focal scales or orthographic half extents, and `effect_view_x/y/z` stores the camera basis. `post_process_screen_space_reflection.wgsl` reconstructs view-space positions from those unjittered parameters, ray-marches in view space, projects candidates back to screen pixels, and reuses velocity history separately.

Reflection probe screen projection follows the same rule. The post-process `camera_matrices::view_projection(...)` helper now calls `ViewProjectionMatrixPair::from_camera(...).clip_from_world_unjittered` instead of owning duplicate perspective/orthographic projection helpers. The deleted local helpers avoid a second projection formula that could drift from the framework jitter contract.

## Audit Result

The TP-M2-S2 source scan covered `view_proj`, `inverse_view_proj`, `previous_view_proj`, `clip_from_world`, and `world_from_clip` across scene-renderer WGSL/Rust. The resolved ownership is:

- Raster only: deferred geometry, fallback mesh main passes, normal prepass, particle, sprite, overlay, and shadow-map writers keep `scene.view_proj`.
- Velocity/reprojection: velocity object, velocity camera params, HZB occlusion culling, and deferred lighting use unjittered matrices.
- Post-process SSR: uses `effect_projection` and view basis rows that ignore `temporal_jitter`.
- SSAO evaluate: reads the unjittered `SceneUniform::inverse_view_proj`, camera position/orthographic direction, qualified current depth/world normal, and the furthest-depth HZB. It reconstructs world positions, derives meters per pixel from adjacent reconstructed positions, bounds the projected world-unit radius, selects HZB mip from sample footprint, and performs a quality-bounded horizon bitmask search into transient raw AO. A following 3x3 spatial pass reconstructs neighbor positions through the same projection contract and combines plane-distance, normal, and spatial weights before publishing final AO. Neither pass reads previous AO. Non-zero-origin or partial SceneLinear rects remain compile-time rejected until render-rect coordinates enter the kernel ABI.

The 2026-08-30 SSAO continuation passed exact Rust formatting, scoped source-contract scans, and locked Cargo metadata only. The Naga validation test is source-only and has not executed. Managed Rust/WGPU shader creation, analytic projection images, PNG/RenderDoc capture, GPU timing, and power evidence remain pending. The hard TP-M2 product acceptance line also remains: TAA Off must receive a product-level pixel/hash parity baseline before TP-M2 can be closed.
