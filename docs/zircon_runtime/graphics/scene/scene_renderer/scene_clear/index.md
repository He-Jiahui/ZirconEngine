---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_color_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_shader.rs
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_color_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_shader.rs
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/index.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs::tests::scene_region_clear_resources_build_for_offscreen_backend
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::base_camera_clear_modes_translate_to_scene_load_ops
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::base_camera_none_clear_with_msaa_clears_scene_color_only
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::overlay_camera_never_clears_scene_color_and_uses_clear_depth_for_depth
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::policy_only_rewrites_scene_first_clear_writes_and_preserves_store
  - cargo test -p zircon_runtime --lib viewport_camera_stack_attachment_policy --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-region-clear-0619 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib scene_region_clear_resources --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-region-clear-0619 --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-region-clear-0619 --message-format short --color never
doc_type: module-detail
---

# Scene Region Clear

## Purpose

`scene_clear` owns the Plan 09 region-scoped clear pass for fixed scene color and depth targets. It exists because WGPU attachment `LoadOp::Clear` clears the whole texture view, while Base/Overlay stacks and split-screen cameras need clear behavior limited to the selected `ViewportRenderRegion`.

`ViewportCameraStackAttachmentPolicy` derives a `ViewportSceneClearPlan` from the selected `CameraRenderDescriptor`. The graph still declares first scene attachment writes as initialization points, but the policy converts first `scene-color` and `scene-depth` graph clears into `Load` so graph execution does not erase sibling camera regions on the shared `OffscreenTarget`.

## Control Flow

`SceneRendererCore::new(...)` constructs one `SceneRegionClearResources` owner with the renderer color format and scene depth format. `SceneRendererCore::render_compiled_scene(...)` records `SceneRegionClearResources::record_frame_clear(...)` after compiled graph resources are materialized and before early graph stages execute.

`record_frame_clear(...)` reads the current frame policy, resolves `Skybox` preview color, explicit `Color(_)`, transparent MSAA `None`, or depth-only clear intent, and returns early when no clear is requested. The internal record path uses load/store attachment ops, applies the selected viewport/scissor rectangle, and issues a fullscreen-triangle draw through one of three pipelines:

- color-only;
- depth-only;
- color plus depth.

The clear shader has no vertex buffers. Its vertex stage emits a fullscreen triangle at `z = 1.0`; the depth pipeline uses `CompareFunction::Always` with depth writes enabled, so depth clear intent writes far depth into the selected region. Color clear uses a small POD uniform generated by `SceneRegionClearColorUniform`.

## Constraints

This pass does not allocate graph resources and does not decide final output ownership. It consumes the fixed renderer-owned `scene_color_view` and `depth_view` already used by frame graph resource binding, then lets graph stages run against those same loaded attachments.

The pass intentionally uses the selected region's physical position and size but a full `0.0..1.0` depth viewport range. That keeps depth clear semantics stable even when a camera descriptor uses a narrowed logical depth range for later draws.

## Validation

The offscreen backend test builds the clear resources, records a color+depth clear against an `OffscreenTarget`, and submits it to WGPU. The same validation lane passed the attachment-policy source-contract tests and a `zircon_runtime` `core-min` cargo check in `D:\cargo-targets\zircon-runtime-scene-region-clear-0619`.

One cold focused lib-test attempt timed out while compiling and then surfaced an unrelated dirty test mismatch against the current `NodeKind` enum. That compile break was corrected from `NodeKind::Light` to `NodeKind::PointLight` before the passing scene-clear tests were rerun.

## Open Issues

This slice is not final Base/Overlay product completion. Remaining Plan 09 work still includes final custom-target composite ownership, UI/scene overlay product order, fully independent per-camera history/light/virtual-geometry/particle state, and pixel/Product/RenderDoc acceptance. Surface present and direct runtime-frame submit now use the selected-camera loop, but they still lack pixel/RenderDoc product acceptance.
