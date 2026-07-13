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

`SceneRenderer::render_scene_color_hdr(...)` renders the normal scene path into HDR scene/final/depth backings acquired from the renderer's `TransientResourcePool`, reads pre-output-transfer `scene_color` through the shared padded-row RGBA16F helper, releases all three backings to the pool, and decodes the half floats to linear `f32` texels. The method is intended for offline authoring workflows such as reflection-probe capture, not for per-frame gameplay readback.

## Invariants

- Scene geometry, sky and lighting pipelines target `SCENE_COLOR_HDR_FORMAT`.
- Output transfer, overlays and UI target `FINAL_COLOR_FORMAT`.
- The HDR readback occurs from `scene_color`, never from the display-ready final texture.
- Reflection-probe capture clears editor overlays and virtual-geometry debug data before invoking this API.
- Every capture brackets transient acquisition with `begin_frame`/`end_frame`, and all success/error paths return acquired textures to the pool.

## Validation

The focused format contracts pass for scene/final separation and HDR SSR/GI resource descriptors. A current locked ReflectionProbe plugin check passes after the transient-pool cutover. The six-face WGPU product test passes 1/1 and asserts the sixth capture reports `texture_created_count=0`, `texture_reused_count=3`, and three retained texture entries. The rebuilt DX12 viewer reached the explicit `Ready` state, presented the Lakes skybox and mirror reflection through the split path, and produced `docs/tests/runtime/shader/zircon_shader_pbr_viewer_hdr_scene_split_ready_20260711.png` (1296x999, SHA256 `D913B0EC12A0769A0F05F3289C445D0DEF85D30A646D133C6A557FEA94C79ED4`). A fresh RenderDoc DX12 capture completed at 14,836,571 bytes, SHA256 `B67EC4138EE2178C04525FC54A0A15A861F343B6F80633ACFE3F067177D7A4AA`; command-line replay remained responsive but did not return within the three-minute acceptance window, so replay is recorded as timed out rather than passed.
