---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
plan_sources:
  - user: 2026-07-07 mirror HDRI grazing asymmetry and right-side over-grazing correction
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png
doc_type: module-detail
---

# Scene Uniform From Frame

## Purpose

`SceneUniform::from_frame(...)` is the CPU-side packer for the scene bind group uniform consumed by skybox, forward, fallback, deferred, and post-process shader paths. It converts a `ViewportRenderFrame` into GPU-ready matrices, camera parameters, ambient lighting, skybox parameters, source-cubemap sampling metadata, environment intensity, and SH9 diffuse coefficients.

This module is a correctness boundary for PBR environment reflection. If it publishes a projection matrix, camera view direction, or cubemap metadata that does not match the active render pass, otherwise correct PMREM and shader code can still produce visibly wrong mirror highlights.

## Related Files

`from_frame.rs` owns the uniform construction. `ViewportRenderFrame` and `ViewportRenderRegion` provide the frame-local render size and region information. `ViewProjectionMatrixPair::from_camera(...)` builds jittered and unjittered camera matrices from a camera snapshot plus an explicit viewport/render size.

The runtime HDRI export tests exercise this path through the real renderer and verify the saved mirror-sphere PNGs with orientation and grazing-balance assertions.

## Behavior Model

The function starts from `frame.effective_camera()` and writes three matrix forms:

- `view_proj`: current jittered clip-from-world matrix.
- `view_proj_unjittered`: current unjittered clip-from-world matrix.
- `inverse_view_proj`: inverse of the unjittered matrix for screen-space reconstruction.

The projection matrix must use `frame.render_region().local_size()`, not `frame.extract.view.effective_render_size()`. Snapshot and export paths can construct a fresh `ViewportRenderFrame` with a 1280x960 render target while the extract still reports a default or stale 1x1 size. Using the extract size makes orthographic and perspective projection disagree with the actual render pass dimensions, which distorts mirror-sphere grazing geometry.

The uniform also publishes:

- camera world position for perspective view-vector reconstruction,
- camera view direction plus an orthographic/perspective flag,
- authored ambient light sum or preview fallback ambient,
- procedural sky colors,
- source cubemap face size and mip count,
- environment enable/intensity/rotation/bake-key flags,
- SH9 diffuse coefficients from the active source-cubemap environment.

## Design And Rationale

The render region is the authoritative projection size because it is derived from the selected camera viewport and any frame-local render target size. This keeps the CPU matrix contract aligned with the render pass viewport and scissor used for the current frame.

The extract remains important for scene data, selected camera descriptors, environment, and post-process settings, but it is not authoritative for the active frame's local render dimensions once a `ViewportRenderFrame` has been built. This distinction matters for screenshot exports, dynamic-resolution paths, and camera stack sub-regions.

The unjittered inverse is intentional. Screen-space reconstruction and post-process consumers must not inherit temporal jitter when rebuilding world-space rays.

## Data Flow

1. `ViewportRenderFrame::from_snapshot(...)` or `from_extract(...)` creates a frame and a `ViewportRenderRegion` for the requested viewport.
2. `SceneUniform::from_frame(...)` reads `frame.render_region().local_size()` and passes it to `ViewProjectionMatrixPair::from_camera(...)`.
3. The resulting matrices, camera vectors, and environment values are copied into the `SceneUniform` buffer.
4. WGSL helpers use these values to reconstruct skybox rays, view vectors, and PBR environment reflection sampling.

For mirror HDRI validation, the same uniform data drives both the visible skybox and the standard PBR mirror sphere, so projection-size mistakes show up as asymmetric edge highlights even when cubemap orientation and PMREM filtering are correct.

## Edge Cases And Constraints

- A stale extract render size must not affect the current frame projection.
- The render region local size is clamped to at least 1x1 by `ViewportRenderRegion`.
- Orthographic cameras publish a fixed view direction with the flag component set to `1.0`; perspective cameras publish camera position and a flag of `0.0`.
- Previous motion-vector matrices still use the motion-vector camera helper path; this document only covers scene uniform matrix construction for current-frame shading.

## Test Coverage

`scene_uniform_uses_frame_render_region_size_for_projection_aspect` locks the regression that motivated the 2026-07-07 fix: an extract that still reports 1x1 must produce a 1280x960 orthographic projection with x/y scale `0.75/1.0`.

The counted validation for the 2026-07-07 HDRI mirror slice is:

- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --color never` passed in `E:\cargo-targets\zircon-cmft-skybox-0707`.
- Direct `runtime_shader_pbr_hdri_export` mirror export passed 1/1 and refreshed the perspective and orthographic PNGs under `docs/tests/runtime/shader`.
- Direct saved-PNG orientation/grazing regression passed 1/1.

The exact Cargo lib-test wrapper for the new unit test did not produce counted pass evidence in this workspace because one run exited rustc without diagnostics and the rerun timed out. The test is compiled by the successful crate check and should be covered by the next long-window lib-test or CI run.

## Plan Sources

This module update belongs to Shader Plan 06 EC-M3p and Render Plan 11 EL-M1/EC-M3p. It follows the user's 2026-07-07 report that the mirror sphere grazing did not look physically plausible, the two sides were inconsistent, and the right side was over-emphasized.

## Open Issues

This change closes the frame projection-size mismatch for the mirror export path. It does not complete the broader EC-M3 acceptance work: strict source-cubemap reference SSIM, automated cube seam gates, RenderDoc/product capture, derived/offline artifact validation, and high-resolution bake acceptance remain tracked in the shader and render plans.
