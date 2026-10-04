---
related_code:
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/derive_camera.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/oblique_projection.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/probe_data.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/quality.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/reflection_matrix.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/tests.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/update_mode.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/update_state.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter/executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter/shaders/filter.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/gpu_layout.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_plugins/rendering/features/planar_reflections/runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/derive_camera.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/oblique_projection.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/probe_data.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/quality.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/reflection_matrix.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/update_mode.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/update_state.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter/executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter/shaders/filter.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/gpu_layout.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_plugins/rendering/features/planar_reflections/runtime/src/lib.rs
plan_sources:
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/PlanarReflectionProbe.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/PlanarReflectionFiltering.compute
tests:
  - zircon_runtime/src/core/framework/render/advanced_lighting/planar/tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_planar_reflection.rs
  - zircon_plugins/rendering/features/planar_reflections/runtime/src/tests.rs
doc_type: module-detail
---

# Planar Reflection

## Purpose

Planar reflection represents a mirror as a regular render-to-texture camera derived from a main
camera. The framework contract stays independent of WGPU: it describes the mirror plane, update
mode, quality tier, layer mask, reflection transform, capture target, and oblique projection needed
by the camera loop. The graphics layer owns the persistent mip texture, roughness filtering, and
environment sampling.

This follows Plan 18 AF-M4 Slice 2 and the HDRP probe model. It deliberately does not introduce a
second scene-rendering path: the derived camera is submitted through the existing Plan 09 camera
loop before its main camera.

## Contract

`PlanarReflectionProbeData` stores a world transform plus a local reference position. The world
mirror point is `plane_transform * local_reference_position`; transformed local positive Y is the
plane normal. `layer_mask` becomes the derived camera culling mask. `capture_target` is optional:
without a registered render-target texture the probe remains inert, so loading older serialized
data cannot create an implicit camera or GPU allocation.

`PlanarUpdateMode` defaults to `OnDemand`, because every capture is a complete scene render.
`EveryFrame` remains available for editor preview or explicitly dynamic mirrors. Quality tiers map
to square capture sizes of 256, 512, and 1024 pixels; Medium/512 is the default.

`PlanarReflectionUpdateState` is the GPU-independent invalidation owner. A never-captured probe is
due, a successfully captured OnDemand probe remains clean, `mark_dirty` schedules a recapture, and
EveryFrame probes are always due. The renderer calls `mark_captured` only after both capture and
filter succeed, so a failed GPU submission cannot accidentally clear pending work.

V1 resolves at most one active planar source for main-camera shading. Probes with a capture target
are ordered by `probe_id`, and the lowest id wins. This makes selection deterministic while avoiding
an unplanned texture-array ABI; multi-planar blending remains a future contract extension.

## Reflection And Clipping

`planar_reflection_matrix` implements the affine plane reflection `R = I - 2nn^T` plus the
translation needed for a plane that does not pass through the world origin. It rejects non-finite
inputs and zero-length normals. Applying the result twice returns the original point within float
precision.

The reflected camera mirrors eye, forward, and up vectors. It targets the probe texture, clears
camera-stack inheritance, disables temporal jitter, uses a square aspect ratio, and receives a
render order immediately before the main camera. The existing projection path accepts an optional
projection override so every downstream consumer, including light-grid reconstruction, sees the
same oblique matrix.

`planar_oblique_near_clip_projection` replaces row 2 of a right-handed WGPU 0..1 projection. The
replacement maps the mirror plane to clip-space `z = 0`, rejecting geometry on the camera side of
the mirror while retaining geometry behind it. Degenerate planes and singular plane/corner
intersections return `None`; callers must skip that probe instead of rendering with an invalid
projection.

## Frame Flow

1. The camera loop consults `PlanarReflectionUpdateState`, derives due mirror cameras, and submits
   them through the ordinary Plan 09 camera path before the main camera.
2. The `rendering.planar_reflections` descriptor activates `planar.filter` only for a texture target
   owned by an extracted planar probe. The pass reads that camera's HDR scene color.
3. The compute executor writes the runtime-owned 1024x1024 `Rgba16Float` texture. A 256, 512, or
   1024 capture occupies the matching top-left extent; successive dispatches create the complete
   roughness mip chain, and each output mip remains sampleable by the following dispatch.
4. Group 1 binding 29 exposes the persistent filtered texture and binding 30 exposes
   `clip_from_world`, `local_from_world`, bounds, UV scale, mip count, and enabled state.
5. `zr_environment.wgsl` projects the shaded world point into mirror capture UV, rejects points
   outside the local bounds or clip volume, and samples a roughness-selected mip before cubemap
   probes. A mirror capture camera receives disabled planar parameters to prevent recursion.
6. After every submitted capture graph succeeds, the framework marks its probe captured. OnDemand
   probes therefore skip subsequent captures until explicitly dirtied; EveryFrame probes remain due.

Registering the plugin alone is byte-inert. With no extracted probe, no mirror camera or
`planar.filter` node is compiled and the main-camera environment parameters remain disabled.

## Test Coverage

The local contract tests cover reflection involution, point mirroring, oblique near-plane
containment, quality/update defaults, and camera derivation. Camera-loop tests cover first capture,
clean OnDemand skipping, explicit dirty recapture, EveryFrame capture, target ownership, and ordering.
The WGPU filter test validates the `Rgba16Float` mip chain and dispatch dimensions.

`render_product_planar_reflection.rs` owns two product regressions: plugin registration with no probe
must match the baseline frame byte-for-byte, while an active probe must schedule the mirror and main
cameras and visibly alter the mirror-floor region. Its ignored exporter writes the side-by-side PNG
and timing report under `docs/tests/runtime/render` during the AF-M4 testing stage.

## Operational Limits

- V1 shades from one deterministic active planar source; it does not blend multiple planar probes.
- Capture targets must be single-layer 2D render-target assets in a camera-target-supported format.
- The filtered texture is fixed at 1024 square with 11 mips; lower quality tiers reduce dispatch and
  capture work but retain this stable group-1 binding shape.
- The plugin must be registered together with its render-pass executor. A descriptor without the
  executor fails graph execution instead of silently falling back.
