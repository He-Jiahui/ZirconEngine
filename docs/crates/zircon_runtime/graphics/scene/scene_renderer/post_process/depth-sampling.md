---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
plan_sources:
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - user: 2026-07-12 continue shader and material rendering verification
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs::tests::viewport_depth_fallback_shader_removes_raw_depth_texture_sampling
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_viewport_depth_fallback_shader_parses_for_gl_backends
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_scene_depth_texture
  - cargo test -p zircon_runtime --lib --locked --offline viewport_depth_fallback_shader
  - zircon_runtime lib-test binary filter: shader
doc_type: module-detail
---

# Post-Process Depth Sampling

Post-process scene-depth reads have two backend modes. `RawDepthTexture` binds a `texture_depth_2d` and reads the physical scene-depth pixel. `ViewportDepthFallback` binds a non-filterable `texture_2d<f32>` for GL/ANGLE compatibility and replaces the raw read with a deterministic normalized logical-pixel Y value.

The common `load_scene_depth` helper must remain derivative-free. Screen-space reflection tracing can call it from data-dependent loops, where implicit-derivative texture sampling is invalid or backend-dependent. The raw shader therefore uses `textureLoad(scene_depth_tex, physical_coord, 0)`, and static tests reject `textureSample(scene_depth_tex)` for this resource.

The fallback rewrite is scoped to the exact raw-depth return expression. Its replacement uses `clamped` and `viewport_size`, which are local to `load_scene_depth`:

```wgsl
return clamp(
    (vec2<f32>(clamped) + vec2<f32>(0.5, 0.5)).y / f32(viewport_size.y),
    0.0,
    1.0,
);
```

Using the logical coordinate avoids applying the physical viewport origin twice. The half-pixel offset samples the pixel center, and the clamp keeps the synthetic depth in the same normalized range as the raw path. The former replacement referenced an undefined `uv`, so generated GL/ANGLE WGSL failed parsing before pipeline creation.

## Validation

On 2026-07-12, the current-source lib-test binary passed both `viewport_depth_fallback_shader` tests and the dedicated `post_process_shader_samples_bound_scene_depth_texture` contract. The broader `shader` filter reported 383 passed, 15 failed, and 4 ignored; none of the remaining failures belongs to this post-process depth fallback slice.
