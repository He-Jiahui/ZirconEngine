---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/types/viewport_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/types/viewport_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_flattens_base_then_overlays_for_submit_order
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_extracts_select_each_sequence_descriptor
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_routes_ui_to_last_primary_stack_terminal_only
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_routes_ui_to_last_base_when_no_primary_base_exists
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_marks_stack_and_viewport_output_owners
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs::tests::inactive_history_handle_disables_current_previous_and_allocation
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::base_camera_clear_modes_translate_to_scene_load_ops
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::base_camera_none_clear_with_msaa_clears_scene_color_only
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::overlay_camera_never_clears_scene_color_and_uses_clear_depth_for_depth
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs::tests::policy_only_rewrites_scene_first_clear_writes_and_preserves_store
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_defaults_to_full_target_without_camera_rect
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_clamps_camera_rect_to_target
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_clamps_fully_outside_rect_to_last_in_bounds_pixel
  - zircon_runtime/src/core/framework/render/frame_extract.rs::tests::render_frame_extract_selected_camera_descriptor_replaces_active_selection_only
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-loop-0618 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-ui-terminal-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-loop-0618 --no-run --message-format short --color never
  - cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-loop-0618 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-ui-terminal-0618 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib render_frame_extract_selected_camera_descriptor_replaces_active_selection_only --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-loop-0618 --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-attachment-policy-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib compiled_graph_cache --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-target-fingerprint-0618 --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib build_runtime_frame --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib resolve_history_handle --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never -- --nocapture
doc_type: module-detail
---

# Submit Camera Loop

## Purpose

`camera_loop.rs` is the Plan 09 M1-S2 offscreen submit scaffold. It turns the neutral multi-camera descriptor list carried by `RenderFrameExtract.view.cameras` into ordered selected-camera child submits without moving WGPU objects, viewport records, or renderer internals into the extract DTO.

## Submit Flow

`submit_frame_extract_with_ui(...)` acquires the render-framework operation lock once, keeps the profiling scope named `submit_frame_extract`, and calls `submit_camera_loop(...)`. The loop resolves `extract.view.cameras` with `resolve_camera_sequence(...)`; if no active Base camera remains it returns `UnsupportedCapability { capability: "active camera sequence" }`.

The resolved sequence is flattened as each Base descriptor followed by that Base camera's Overlay descriptors. Each descriptor is projected into a child extract through `RenderFrameExtract::with_selected_camera_descriptor(...)`, which replaces the child `view.cameras` list with that one descriptor, aligns `view.scene_camera_entity`, updates the transitional `view.camera` payload, selects the core pipeline kind, applies descriptor MSAA, and preserves the descriptor target/viewport sizing rules. The existing single-camera render body then handles context build, runtime preparation, and graph execution for each child; viewport-terminal ownership decides which child commits shared capture, history, record, previous-state, and stats state afterward.

The loop routes the shared `UiRenderExtract` only to the terminal child in the selected Base stack. It chooses the last `PrimarySurface` Base stack when present, otherwise the last Base stack for texture/headless-only sequences. The terminal child is that stack's final Overlay descriptor, or the Base descriptor when the stack has no overlays. Intermediate child submits receive no shared UI, so the current scaffold does not draw the same screen-space UI into every selected-camera submit.

The loop also computes per-child output ownership. `stack_terminal` is true only for the final child in that Base stack, while `viewport_terminal` is true only for the terminal child selected for viewport-level ownership. `ViewportRenderFrame` carries that `ViewportCameraStackOutputPolicy`; the renderer uses `stack_terminal` to suppress direct final-target import, texture writeback, and prepared-texture capture for non-stack-terminal children, and generated offscreen submit uses `viewport_terminal` to choose the single shared viewport record/history/stats owner.

Each child submit also produces a target-sensitive compiled graph cache key. `CompiledGraphCacheKey` now includes the selected camera target fingerprint, Base/Overlay render type, and viewport-rect presence beside the existing core pipeline, size, HDR, MSAA, particle, compile-option, capability, and shader-quality inputs. That keeps texture/headless/overlay child submits from reusing a graph compiled for a different selected-camera shape.

## Current Boundaries

This module is a loop boundary, not the final Base/Overlay compositor. It does not directly own graph attachment ops. The selected child descriptor now flows into `ViewportCameraStackAttachmentPolicy`, and `RenderPassExecutionContext::attachment_ops_for_write(...)` uses that policy to translate `RenderCameraClear` and Overlay `clear_depth` into the first `scene-color` / `scene-depth` graph clear write. The merge preserves graph store ops and leaves later load-store writes unchanged. The output policy is similarly scoped: non-stack-terminal children do not write the resolved texture output target, and non-viewport-terminal children render with inactive shared history, drain renderer runtime feedback, and then skip viewport record/history/stats/debug-capture ownership.

Base/Overlay physical target reuse is currently provided by the fixed renderer-owned `OffscreenTarget`: `bind_frame_graph_resources(...)` binds live `SCENE_COLOR` and `SCENE_DEPTH` graph resources to the same frame target views for each selected-camera child, while the camera stack attachment policy decides clear versus load on the first scene write. Each selected child frame also carries a `ViewportRenderRegion` derived from the selected descriptor's clamped `viewport_rect`; graph-raster passes apply it as WGPU viewport plus scissor state, so split-screen Base cameras no longer draw outside their assigned physical rect in those passes. The renderer still does not split fullscreen post-process, light, temporal history, hybrid-GI, virtual-geometry, or particle runtime state by camera, and final custom-target composite semantics are still follow-up work. Present submit and direct runtime-frame submit still follow the selected-camera path with the default terminal output policy.

Those omissions are intentional Plan 09 follow-up work. The current value of the module is to remove the last offscreen single-camera submit assumption and give later slices a single place to add load/store, target reuse, history, and composite policy.

## Validation

The module has focused unit coverage for Base-then-Overlay flattening, selected-descriptor child extract projection, terminal UI routing for both PrimarySurface and texture/headless-only camera sequences, and stack/viewport output owner selection. `RenderFrameExtract` has companion coverage proving that selecting a descriptor replaces only the active child selection and carries the selected camera target, layer, pipeline, MSAA, and entity payload into the child frame. `build_runtime_frame` coverage proves the output policy reaches renderer-bound frames.
