---
related_code:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_color_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_shader.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
implementation_files:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_color_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_shader.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
tests:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::base_camera_clear_modes_translate_to_scene_load_ops
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::base_camera_none_clear_with_msaa_clears_scene_color_only
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::overlay_camera_never_clears_scene_color_and_uses_clear_depth_for_depth
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::policy_only_rewrites_scene_first_clear_writes_and_preserves_store
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs::tests::scene_region_clear_resources_build_for_offscreen_backend
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-attachment-policy-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib viewport_camera_stack_attachment_policy --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-region-clear-0619 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib scene_region_clear_resources --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-region-clear-0619 --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-region-clear-0619 --message-format short --color never
doc_type: module-detail
---

# Viewport Camera Stack Attachment Policy

## Purpose

`viewport_camera_stack_attachment_policy.rs` is the Plan 09 runtime bridge between neutral camera descriptors and renderer clear behavior. It reads the selected `CameraRenderDescriptor` on `ViewportRenderFrame` construction, stores a `ViewportSceneClearPlan`, and keeps the derived policy inside `zircon_runtime::graphics`, so `RenderFrameExtract` and other neutral DTOs still do not carry WGPU-specific state.

The policy deliberately separates clear intent from WGPU attachment load ops. A WGPU `LoadOp::Clear` clears the whole texture view, which is wrong for split-screen and sub-viewport camera stacks sharing a fixed scene target. Scene graph first writes therefore load the existing target, and `SceneRegionClearResources` applies the actual Base/Overlay color/depth clear with viewport and scissor state before graph stages execute.

## Policy Table

Base cameras translate `RenderCameraClear` into a scene clear plan plus first scene graph attachment load behavior:

| Base clear | scene clear plan | first graph `scene-color` | first graph `scene-depth` |
| --- | --- | --- |
| `Skybox` | preview color + depth | Load | Load |
| `Color(_)` | explicit color + depth | Load | Load |
| `DepthOnly` | depth only | Load | Load |
| `None` | no clear | Load | Load |
| `None` with `msaa_samples > 1` | transparent color | Load | Load |

Overlay cameras ignore the color part of `clear`: the scene color plan is empty, and the scene depth plan is set only when `clear_depth` is true. First graph writes still load both fixed scene attachments.

## Graph Integration

`ViewportRenderFrame::from_extract(...)` derives the policy after synchronizing the selected descriptor payload. Public-runtime and snapshot constructors also install an explicit default policy so every renderer-bound frame has deterministic state.

`RenderPassGpuExecutionContext` exposes the frame policy to `RenderPassExecutionContext`. `attachment_ops_for_write(...)` first resolves the compiled graph's declared attachment ops, then applies the camera policy only when all of these are true:

- a GPU frame is present;
- the resource is `scene-color` or `scene-depth`;
- the graph-declared load op is `Clear`.

The policy preserves the graph-declared store op, leaves all non-scene resources unchanged, and leaves later `Load/Store` scene writes unchanged. This keeps the graph compiler's "first write is the initialization boundary" ownership intact while preventing whole-target attachment clears on shared scene targets.

`SceneRendererCore::render_compiled_scene(...)` records `SceneRegionClearResources::record_frame_clear(...)` after graph resources have been materialized and before the first graph stage executes. That pre-graph pass resolves preview/explicit/transparent color clear values, uses the selected `ViewportRenderRegion` viewport and scissor, and writes color and/or far depth into only the selected camera region.

## Current Boundaries

This is the clear-plan and scene-attachment load layer only. Fixed scene target reuse, graph-raster region clipping, terminal post-process region writeback, output-owner gating, surface-present loop ownership, and direct runtime-frame loop ownership now exist in adjacent Plan 09 slices, but the final Base-stack custom-target composite owner, full independent temporal history, lighting, virtual-geometry, particle state, capture, and pixel/RenderDoc acceptance remain open.

## Validation

The module has source-contract unit tests for Base clear modes, the URP-style MSAA `None` color-clear edge case, Overlay color/depth behavior, and the "first clear write only, preserve store op" graph merge rule. The scene-clear module adds an offscreen WGPU backend test that builds the pipelines, records a full-target color+depth clear, and submits it.

The 2026-06-19 scene-clear lane passed `cargo fmt --package zircon_runtime`, the four `viewport_camera_stack_attachment_policy` tests, the `scene_region_clear_resources` offscreen backend test, and `zircon_runtime` `core-min` cargo check in `D:\cargo-targets\zircon-runtime-scene-region-clear-0619`. One focused lib-test attempt timed out during cold compile and then exposed an unrelated dirty test compile break (`NodeKind::Light`), which was fixed to the current `NodeKind::PointLight` variant before rerunning the passing tests.
