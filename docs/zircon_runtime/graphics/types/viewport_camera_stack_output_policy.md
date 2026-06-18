---
related_code:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/submit_capture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
implementation_files:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
tests:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_marks_stack_and_viewport_output_owners
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs::tests::inactive_history_handle_disables_current_previous_and_allocation
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs::tests::build_runtime_frame_carries_prepared_sideband_and_output_target_into_viewport_frame
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib build_runtime_frame --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib resolve_history_handle --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never -- --nocapture
doc_type: module-detail
---

# Viewport Camera Stack Output Policy

## Purpose

`viewport_camera_stack_output_policy.rs` is the Plan 09 graphics-side bridge for final-output ownership inside the current selected-camera child loop. It keeps target write ownership on `ViewportRenderFrame`, not on neutral `RenderFrameExtract` or `CameraRenderDescriptor`, so framework DTOs remain free of renderer resource policy.

## Policy

`camera_loop.rs` emits two booleans for each child submit:

- `stack_terminal`: true for the final camera in that Base stack, meaning the Base itself when no Overlays exist or the last Overlay when they do.
- `viewport_terminal`: true for the terminal child of the last `PrimarySurface` Base stack, or the last Base stack when no primary target exists.

`ViewportCameraStackOutputPolicy::writes_output_target()` follows `stack_terminal`. That lets every completed Base stack write its own resolved texture output while suppressing writes from intermediate Base/Overlay children.

`ViewportCameraStackOutputPolicy::owns_viewport_submission()` follows `viewport_terminal`. Generated offscreen submit uses that bit as the shared viewport owner: only the viewport-terminal child consumes pending graphics-debugger capture requests, resolves or allocates a frame-history handle, updates `ViewportRecord` capture/history/runtime feedback, advances temporal and particle previous-state snapshots, releases superseded renderer history targets, updates `RenderStats`, and publishes the last virtual-geometry debug snapshot. Non-owner children still render and drain renderer readback feedback, but they use an inactive history handle and do not mutate shared viewport record/history/stats. A debug assertion keeps `viewport_terminal` from becoming true without `stack_terminal`.

## Renderer Integration

`build_runtime_frame(...)` attaches the camera-loop policy to `ViewportRenderFrame`. Present submit and direct runtime-frame construction install the default stack-terminal policy, preserving the existing selected-camera behavior outside generated offscreen multi-camera submit.

The renderer consumes the policy in three places:

- `direct_imported_final_target(...)` returns no prepared output target for non-stack-terminal children, preventing graph final aliases from writing directly into the texture target too early.
- `render_frame_with_pipeline_to_target(...)` calls `suppress_output_target_writeback(...)` for non-stack-terminal children, so no post-graph texture copy or linear conversion writes the output target.
- `output_target_capture_resource(...)` returns no prepared texture capture for non-stack-terminal children; those children still finish through the framework offscreen target.
- `submit_selected_camera_frame(...)` gates graphics-debugger capture, `resolve_history_handle(...)`, `record_submission(...)`, temporal and particle previous-state updates, history release, and `update_stats(...)` on `owns_viewport_submission()`.
- `ResolvedHistoryHandle::inactive()` gives non-owner children `None` for both current and allocated history, preventing intermediate child submits from allocating renderer history targets that no viewport record will later own.

## Current Boundaries

This is output and shared-viewport submission ownership, not final Base/Overlay composition. It does not yet make Overlay cameras reuse the Base physical color/depth targets, does not create independent per-camera post-process, lighting, temporal-history, hybrid-GI, virtual-geometry, or particle feedback state, and does not extend the generated camera loop to present submit or direct runtime-frame submit. Non-owner children intentionally render without the shared viewport history handle until a later per-camera history model exists.

## Validation

`camera_loop_marks_stack_and_viewport_output_owners` covers mixed primary and texture Base stacks, proving stack-terminal and viewport-terminal flags are computed separately. `build_runtime_frame_carries_prepared_sideband_and_output_target_into_viewport_frame` proves the policy reaches `ViewportRenderFrame`. `inactive_history_handle_disables_current_previous_and_allocation` covers the non-owner history gate. The output-target slice passed `core-min` `cargo check` plus focused `camera_loop` and `build_runtime_frame` tests in `D:\cargo-targets\zircon-runtime-camera-output-policy-0618`; the viewport-record-owner follow-up uses `D:\cargo-targets\zircon-runtime-camera-record-owner-0618`.
