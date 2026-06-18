---
related_code:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
implementation_files:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
tests:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::base_camera_clear_modes_translate_to_scene_load_ops
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::base_camera_none_clear_with_msaa_clears_scene_color_only
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::overlay_camera_never_clears_scene_color_and_uses_clear_depth_for_depth
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::policy_only_rewrites_scene_first_clear_writes_and_preserves_store
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-attachment-policy-0618 --message-format short --color never
doc_type: module-detail
---

# Viewport Camera Stack Attachment Policy

## Purpose

`viewport_camera_stack_attachment_policy.rs` is the Plan 09 runtime bridge between neutral camera descriptors and graph attachment load operations. It reads the selected `CameraRenderDescriptor` on `ViewportRenderFrame` construction and keeps the derived policy inside `zircon_runtime::graphics`, so `RenderFrameExtract` and other neutral DTOs still do not carry WGPU-specific state.

## Policy Table

Base cameras translate `RenderCameraClear` into the first scene color/depth attachment write:

| Base clear | scene-color | scene-depth |
| --- | --- | --- |
| `Skybox` | Clear | Clear |
| `Color(_)` | Clear | Clear |
| `DepthOnly` | Load | Clear |
| `None` | Load | Load |
| `None` with `msaa_samples > 1` | Clear | Load |

Overlay cameras ignore the color part of `clear`: `scene-color` always loads, and `scene-depth` clears only when `clear_depth` is true.

## Graph Integration

`ViewportRenderFrame::from_extract(...)` derives the policy after synchronizing the selected descriptor payload. Public-runtime and snapshot constructors also install an explicit default policy so every renderer-bound frame has deterministic state.

`RenderPassGpuExecutionContext` exposes the frame policy to `RenderPassExecutionContext`. `attachment_ops_for_write(...)` first resolves the compiled graph's declared attachment ops, then applies the camera policy only when all of these are true:

- a GPU frame is present;
- the resource is `scene-color` or `scene-depth`;
- the graph-declared load op is `Clear`.

The policy preserves the graph-declared store op, leaves all non-scene resources unchanged, and leaves later `Load/Store` scene writes unchanged. This keeps the graph compiler's "first write is the clear boundary" ownership intact while letting Plan 09 camera clear semantics alter that first clear into a load for `DepthOnly`, `None`, and Overlay cases.

## Current Boundaries

This is the clear/load translation layer only. It does not yet make Overlay cameras reuse the Base camera's physical color/depth targets, does not choose the final Base-stack composite owner, and does not split temporal history, post-process, lighting, capture, or present ownership by camera.

## Validation

The module has source-contract unit tests for Base clear modes, the URP-style MSAA `None` color-clear edge case, Overlay color/depth behavior, and the "first clear write only, preserve store op" graph merge rule. In the current slice, `cargo check` passed for `zircon_runtime` with `core-min`; the focused `cargo test` filter timed out during shared lib-test compilation and is not counted as passing evidence.
