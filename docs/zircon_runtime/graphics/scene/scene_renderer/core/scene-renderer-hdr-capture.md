---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_hdr_capture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/constants.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/construct.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_region.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_hdr_capture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
plan_sources:
  - user: 2026-07-11 reflection-probe scene capture and unclipped PBR environment reflection
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/constants.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Scene Renderer HDR Capture

## Purpose

The scene renderer keeps lighting and environment reflection in linear HDR until the final output transfer. This is required for mirror metals and captured reflection probes: rendering directly into `Rgba8UnormSrgb` clips radiance above 1.0 before tonemapping and makes bright environment reflections appear white or flat.

## Render Path

The retained offscreen target now has two explicit color roles:

- `scene_color` is `Rgba16Float` and receives skybox, opaque and transparent scene content.
- `final_color` is `Rgba8UnormSrgb` and receives the output-transfer pass, then viewport overlays and screen-space UI.

The direct `SceneRenderer::render(...)` path follows the same split as the compiled graph path. It no longer binds an HDR-created mesh pipeline to an sRGB attachment.

`SceneRenderer::render_scene_color_hdr(...)` renders the normal scene path, retains the pre-output-transfer `scene_color` texture, reads it back through the shared padded-row RGBA16F helper, and decodes the half floats to linear `f32` texels. The method is intended for offline authoring workflows such as reflection-probe capture, not for per-frame gameplay readback.

## Invariants

- Scene geometry, sky and lighting pipelines target `SCENE_COLOR_HDR_FORMAT`.
- Output transfer, overlays and UI target `FINAL_COLOR_FORMAT`.
- The HDR readback occurs from `scene_color`, never from the display-ready final texture.
- Reflection-probe capture clears editor overlays and virtual-geometry debug data before invoking this API.

## Validation

The focused format contracts pass for scene/final separation and HDR SSR/GI resource descriptors. A current `cargo check -p zircon_runtime --lib --locked` completed successfully after the split. Final WGPU viewer and reflection-probe screenshot acceptance remain part of the active Shader 06 / Render 11 testing stage.
