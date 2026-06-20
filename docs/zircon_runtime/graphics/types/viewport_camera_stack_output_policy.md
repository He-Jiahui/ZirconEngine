---
related_code:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/light_grid_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/shared_product_reports.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/record_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/submit_capture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/final_target_output.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
implementation_files:
  - zircon_runtime/src/graphics/types/viewport_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_camera_stack_output_policy.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/light_grid_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/shared_product_reports.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/record_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/history.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/final_target_output.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
tests:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_marks_stack_and_viewport_output_owners
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::viewport_terminal_camera_target_uses_last_primary_stack_terminal
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::viewport_terminal_camera_target_falls_back_to_last_base_without_primary
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs::tests::camera_loop_frame_submissions_project_selected_children_and_terminal_ui
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/history.rs::tests::viewport_record_keeps_histories_per_camera_key
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs::tests::build_runtime_frame_carries_prepared_sideband_and_output_target_into_viewport_frame
  - zircon_runtime/src/graphics/types/viewport_camera_stack_output_policy.rs::tests::final_target_output_owner_is_stack_terminal_not_viewport_terminal
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/final_target_output.rs::tests::final_target_output_reports_suppressed_texture_children_only
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::suppressed_output_target_writeback_report_is_texture_only
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_camera_stack_suppressed_target_output
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib build_runtime_frame --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib resolve_history_handle --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-present-camera-loop-0619 --message-format short --color never
  - cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-present-camera-loop-0619 --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-present-camera-loop-0619 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-direct-camera-loop-0619 --message-format short --color never
  - cargo test -p zircon_runtime --lib camera_loop_frame_submissions --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-direct-camera-loop-0619 --no-run --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-history-owner-0619 --message-format short --color never
  - D:\cargo-targets\zircon-runtime-camera-history-owner-0619\debug\deps\zircon_runtime-d071a300da0585cb.exe camera_history --test-threads=1 --nocapture
  - D:\cargo-targets\zircon-runtime-camera-history-owner-0619\debug\deps\zircon_runtime-d071a300da0585cb.exe graphics::runtime::render_framework::viewport_record::history::tests::viewport_record_keeps_histories_per_camera_key --exact --test-threads=1 --nocapture
doc_type: module-detail
---

# Viewport Camera Stack Output Policy

## Purpose

`viewport_camera_stack_output_policy.rs` is the Plan 09 graphics-side bridge for final-output ownership inside the current selected-camera child loop. It keeps target write ownership on `ViewportRenderFrame`, not on neutral `RenderFrameExtract` or `CameraRenderDescriptor`, so framework DTOs remain free of renderer resource policy.

## Policy

`camera_loop.rs` emits two booleans for each child submit:

- `stack_terminal`: true for the final camera in that Base stack, meaning the Base itself when no Overlays exist or the last Overlay when they do.
- `viewport_terminal`: true for the terminal child of the last `PrimarySurface` Base stack, or the last Base stack when no primary target exists.

`ViewportCameraStackOutputPolicy::owns_final_target_output()` follows `stack_terminal`. That lets every completed Base stack write its own resolved texture output while suppressing writes from intermediate Base/Overlay children.

`ViewportCameraStackOutputPolicy::owns_viewport_submission()` follows `viewport_terminal`. Generated offscreen submit, native surface present, and direct runtime-frame submit use that bit for the single child that records the shared viewport submission or presents to the native surface. `ViewportCameraStackOutputPolicy::owns_shared_viewport_products()` currently follows the same bit but names the shared product/debug owner explicitly: only that child consumes pending graphics-debugger capture requests, updates `RenderStats`, publishes the last virtual-geometry debug snapshot, and snapshots shared renderer reports such as the light-grid execution report before stats update. Per-camera temporal history and previous/runtime state are deliberately narrower than viewport ownership: every child resolves or allocates its selected-camera slot and non-owner children record that slot without mutating shared capture/stats/debug state. A debug assertion keeps `viewport_terminal` from becoming true without `stack_terminal`.

## Renderer Integration

`build_runtime_frame(...)` attaches the camera-loop policy to `ViewportRenderFrame`. Generated offscreen submit and native surface present receive that policy through `submit_camera_loop(...)`; direct runtime-frame submit receives it through `camera_loop_frame_submissions(...)` after projecting the caller's frame into selected-camera child frames.

The renderer consumes the policy in four places:

- `select_final_target_output(...)` returns no prepared output target for non-stack-terminal children, preventing graph final aliases from writing directly into the texture target too early. Texture targets in that branch publish `SuppressedByCameraStack` graph-import telemetry instead of looking like a not-requested target.
- `render_frame_with_pipeline_to_target(...)` calls `suppress_output_target_writeback(...)` for non-stack-terminal children, so no post-graph texture copy or linear conversion writes the output target. Texture targets in that branch publish `SuppressedByCameraStack` writeback telemetry.
- `output_target_capture_resource(...)` returns no prepared texture capture for non-stack-terminal children; those children still finish through the framework offscreen target.
- `submit_selected_camera_frame(...)` uses `owns_viewport_submission()` for the full shared viewport record path and `owns_shared_viewport_products()` for graphics-debugger capture, `SharedViewportProductReports`, `update_stats(...)`, and the last virtual-geometry debug snapshot.
- `present_selected_camera_frame(...)` uses `owns_viewport_submission()` to render non-owner children offscreen without a surface lease, while only the viewport-terminal child presents and records the present submission; its shared product/debug writes go through `owns_shared_viewport_products()`.
- `submit_selected_runtime_frame(...)` uses the same split, so direct runtime-frame child submits keep viewport-record ownership separate from the named shared product/debug gate.
- `record_non_viewport_camera_state_after_success(...)` is the non-owner state path. It records the selected-camera history slot, applies per-camera HGI/VG runtime feedback, persists motion-vector camera and particle previous sprites for that camera key, and releases a rotated previous handle without updating shared capture, stats, or virtual-geometry debug snapshot state.

## Current Boundaries

This is final-target owner selection, shared-viewport submission ownership, and the named shared product/debug owner gate, not final Base/Overlay product composition. It now includes per-camera temporal color history, visibility static index, motion-vector camera, particle previous sprites, Hybrid GI runtime, Virtual Geometry runtime state, and the shared light-grid stats report boundary. It does not yet create final custom-target composite rules or independent per-camera light/product-debug ownership beyond that shared stats report.

## Validation

`camera_loop_marks_stack_and_viewport_output_owners` covers mixed primary and texture Base stacks, proving stack-terminal, viewport-terminal, final-target, viewport-submission, and shared-product owner bits are computed separately. `final_target_output_owner_is_stack_terminal_not_viewport_terminal` covers the output-policy helpers. The two `viewport_terminal_camera_target_*` tests cover present preflight target selection. `camera_loop_frame_submissions_project_selected_children_and_terminal_ui` covers direct runtime-frame child projection, terminal UI routing, output ownership, render regions, and runtime-frame scene snapshot preservation. `build_runtime_frame_carries_prepared_sideband_and_output_target_into_viewport_frame` proves the policy reaches `ViewportRenderFrame`. `final_target_output_reports_suppressed_texture_children_only`, `suppressed_output_target_writeback_report_is_texture_only`, and `render_product_diagnostics_record_camera_stack_suppressed_target_output` cover the suppressed-by-camera-stack graph-import/writeback diagnostics. `viewport_record_keeps_histories_per_camera_key` covers the per-camera history slot split. The output-target slice passed `core-min` `cargo check` plus focused `camera_loop` and `build_runtime_frame` tests in `D:\cargo-targets\zircon-runtime-camera-output-policy-0618`; the viewport-record-owner follow-up uses `D:\cargo-targets\zircon-runtime-camera-record-owner-0618`. The present-loop follow-up passed `core-min` `cargo check` in `D:\cargo-targets\zircon-runtime-present-camera-loop-0619`; focused `camera_loop` Cargo testing timed out during lib-test compile/link, and `cargo check --lib --tests` is blocked by unrelated stale `RenderMeshSnapshot` fields in `tests/virtual_geometry_debug_snapshot_contract.rs`. The direct-runtime follow-up passed `core-min` `cargo check` in `D:\cargo-targets\zircon-runtime-direct-camera-loop-0619`; focused `camera_loop_frame_submissions` `--no-run` validation did not produce a usable pass result. The final-target owner diagnostics follow-up passed path-scoped rustfmt and `core-min` `cargo check` in `D:\cargo-targets\zircon-runtime-final-target-output-0619`; package-wide fmt is blocked by unrelated dynamic-scene formatting drift. The per-camera history owner slice passed `core-min` `cargo check` in `D:\cargo-targets\zircon-runtime-camera-history-owner-0619`; its Cargo test wrapper timed out during the shared lib-test compile, but direct binary execution passed the `camera_history` filter with 4 tests and the exact record-level history map test with 1 test. The shared-product owner helper slice passed scoped `rustfmt --edition 2021 --check` and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-shared-product-owner-0620 --message-format short --color never`; focused `cargo test -p zircon_runtime --lib viewport_camera_stack_output_policy ...` timed out after 906s without producing a lib-test binary, so no focused test pass is claimed for that wrapper. The shared light-grid product-report boundary passed scoped `rustfmt --edition 2021 --check` and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-light-grid-shared-products-0620 --message-format short --color never` with the repository warning set.

The 2026-06-20 focused validation closeout reused the generated lib-test binary at `D:\cargo-targets\zircon-runtime-texture-linear-product-0620\debug\deps\zircon_runtime-c339c28ec98a5de7.exe`. Direct exact execution passed `graphics::types::viewport_camera_stack_output_policy::tests::final_target_output_owner_is_stack_terminal_not_viewport_terminal` and `graphics::runtime::render_framework::submit_frame_extract::submit::camera_loop::tests::camera_loop_marks_stack_and_viewport_output_owners`, one test each. This closes the previous focused-wrapper timeout gap for output-policy helper semantics and the camera-loop final-target / viewport-submission / shared-product owner mapping.
