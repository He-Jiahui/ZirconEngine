---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/motion_vector_camera.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/runtime_states.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/product_reports.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/viewport_record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/camera_history_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_present.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/record_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/destroy_viewport/destroy_viewport.rs
  - zircon_runtime/src/graphics/runtime/history/viewport_frame_history.rs
  - zircon_runtime/src/graphics/runtime/history/construct.rs
  - zircon_runtime/src/graphics/runtime/history/update.rs
  - zircon_runtime/src/graphics/runtime/history/access.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/motion_vector_camera.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/runtime_states.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/product_reports.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/viewport_record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/camera_history_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_present.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/record_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/destroy_viewport/destroy_viewport.rs
  - zircon_runtime/src/graphics/runtime/history/viewport_frame_history.rs
  - zircon_runtime/src/graphics/runtime/history/construct.rs
  - zircon_runtime/src/graphics/runtime/history/update.rs
  - zircon_runtime/src/graphics/runtime/history/access.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
tests:
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs::tests::camera_history_key_distinguishes_same_entity_viewport_regions
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs::tests::camera_history_key_distinguishes_base_and_overlay_slots
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs::tests::camera_history_key_distinguishes_culling_layers_without_legacy_loss
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs::tests::camera_history_key_distinguishes_volume_layers_without_legacy_loss
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/history.rs::tests::viewport_record_keeps_histories_per_camera_key
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/motion_vector_camera.rs::tests::viewport_record_keeps_motion_vector_camera_per_camera_key
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs::tests::viewport_record_keeps_particle_previous_sprites_per_camera_key
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/runtime_states.rs::tests::viewport_record_keeps_hybrid_gi_runtime_per_camera_key
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/runtime_states.rs::tests::viewport_record_keeps_virtual_geometry_runtime_per_camera_key
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/product_reports.rs::tests::viewport_record_keeps_product_reports_per_camera_key
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs::tests::successful_non_terminal_submit_records_camera_without_advancing_viewport_index
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-history-owner-0619 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-prev-state-0620 --message-format short --color never
  - cargo test -p zircon_runtime --lib viewport_record_keeps_product_reports_per_camera_key --no-default-features --features core-min --locked --jobs 4 --target-dir target\codex-runtime-camera-products-0620 -- --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 4 --target-dir target\codex-runtime-camera-products-0620 --message-format short --color never
  - D:\cargo-targets\zircon-runtime-camera-history-owner-0619\debug\deps\zircon_runtime-d071a300da0585cb.exe camera_history --test-threads=1 --nocapture
  - D:\cargo-targets\zircon-runtime-camera-history-owner-0619\debug\deps\zircon_runtime-d071a300da0585cb.exe graphics::runtime::render_framework::viewport_record::history::tests::viewport_record_keeps_histories_per_camera_key --exact --test-threads=1 --nocapture
doc_type: module-detail
---

# Viewport Record Per-Camera State

## Purpose

`ViewportRecord` owns renderer-side state that must survive between submissions for a viewport. Plan 09 multi-camera execution means one viewport can submit several selected cameras in one frame: split-screen Base cameras, Base plus Overlay stacks, texture/headless cameras, and the final PrimarySurface camera. A single viewport-wide temporal history slot would let one child camera invalidate or overwrite another child camera's previous frame inputs. The record therefore stores frame history in a `HashMap<ViewportCameraHistoryKey, ViewportFrameHistory>`.

## Camera Key

`ViewportCameraHistoryKey::from_camera(...)` derives the internal map key from the selected `CameraRenderDescriptor`. The key includes the scene entity, `render_order`, Base/Overlay render type, target ordering key, and viewport rect, including depth range bits. It intentionally does not use the transient `ViewportCameraSnapshot` alone because Plan 09 moved target, viewport, render type, and ordering ownership onto `CameraRenderDescriptor`.

The key also includes the selected descriptor's culling and volume layer sets. These fields decide which visibility/static-index, previous motion-vector camera, particle previous sprites, runtime states, product reports, and temporal history slot a child camera consults before the deeper `FrameHistoryValidationKey` compatibility check runs. The private `ViewportCameraHistoryLayerKey` stores the typed layer list from `RenderLayerSet::iter()` rather than a lossy `u32` mask, so layer 32+ cameras do not collapse onto empty or legacy-equivalent keys.

The key is internal to `graphics::runtime::render_framework`. Submit context construction derives it from the selected descriptor through `camera_history_key_for_extract(...)`, and falls back to a synthetic descriptor only for malformed or transitional extracts that lack a selected descriptor. The normal camera loop always projects a single selected descriptor into each child extract first.

## State Flow

`resolve_viewport_record_state(...)` uses the current child key to load that camera's previous visibility history, `VisibilityStaticIndex`, previous motion-vector camera, and particle previous sprites. That keeps static visibility acceleration and temporal previous-state inputs from leaking between split-screen or overlay cameras with different frustums, layers, targets, or viewport regions.

`resolve_history_handle(...)` also queries history by the same key. Compatibility checks still compare target size, render size, pipeline handle, history bindings, and `FrameHistoryValidationKey`; the key decides which camera slot is consulted before those compatibility checks run.

`record_history(...)` writes the updated `ViewportFrameHistory` back under `context.camera_history_key()`. The history object now carries both `VisibilityHistorySnapshot` and `VisibilityStaticIndex`, so the old viewport-wide `visibility_static_index` field was removed. When a new `FrameHistoryHandle` is allocated for a camera slot, the previous handle for that slot is released after the record update path finishes.

`prepare_runtime_submission(...)` now ensures Hybrid GI and Virtual Geometry runtime state under the selected camera key. `record_submission(...)` and `record_present_submission(...)` update those keyed runtimes with renderer feedback, then store the viewport-terminal child's capture and stats as the shared viewport summary.

`viewport_record/product_reports.rs` stores the renderer's latest `RenderGraphLightGridReport`
and the child `RenderVirtualGeometryDebugSnapshot` under the selected camera key. Submit, present,
and direct runtime paths record those products after successful child feedback, while the
viewport-terminal owner remains responsible for shared `RenderStats`, graphics capture history, and
the viewport-global last virtual-geometry debug snapshot.

Non-viewport-terminal children call `record_non_viewport_camera_state_after_success(...)`: they
rotate only their selected camera history slot, apply their Hybrid GI and Virtual Geometry feedback
to their keyed runtime states, and store their previous motion-vector camera, particle previous
sprites, light-grid report, and virtual-geometry debug snapshot. They still do not mutate shared
capture, stats, or the viewport-global last virtual-geometry debug snapshot.

`destroy_viewport(...)` now drains all histories from the removed record and releases every stored `FrameHistoryHandle`. This is required because one viewport can now own several history handles at once.

## Current Boundaries

This slice splits temporal color history, the visibility static index, motion-vector camera snapshots,
particle previous sprites, Hybrid GI runtime state, Virtual Geometry runtime state, light-grid reports,
and virtual-geometry debug snapshots by selected camera. Final custom-target composite ownership,
unified sort key coverage, broader UI/scene overlay RenderDoc/product evidence, and editor authoring
controls remain follow-up Plan 09 CO-M2 work.

Plan 06 status `render_plan06_temporal_history_texture_lifetime_owner_suppression_coremin_passed`
keeps the selected-camera temporal color-history slot contract unchanged: the TAA store owns the
underlying `wgpu::Texture` as `_texture` only to keep its paired history `TextureView` alive, while
runtime consumers still sample/write through `TemporalHistoryStore::previous_view()` and
`current_view()`.

## Validation

The module has unit coverage for key separation across same-entity split viewport regions, Base versus Overlay slots, culling layer masks, and volume layer masks, plus record-level tests proving two camera keys keep distinct frame history handles, static-index values, previous motion-vector cameras, particle previous sprites, Hybrid GI runtimes, and Virtual Geometry runtimes. The production crate passed:

```powershell
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-history-owner-0619 --message-format short --color never
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-prev-state-0620 --message-format short --color never
cargo test -p zircon_runtime --lib viewport_record_keeps_product_reports_per_camera_key --no-default-features --features core-min --locked --jobs 4 --target-dir target\codex-runtime-camera-products-0620 -- --test-threads=1 --nocapture
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 4 --target-dir target\codex-runtime-camera-products-0620 --message-format short --color never
```

The filtered lib-test Cargo wrapper for `camera_history` exposed stale test-only helper paths and `ViewportFrameHistory::new(...)` fixtures after the signature change; those helpers were fixed. The wrapper later timed out during the shared lib-test compile, but it produced `D:\cargo-targets\zircon-runtime-camera-history-owner-0619\debug\deps\zircon_runtime-d071a300da0585cb.exe`. Direct binary execution passed the `camera_history` filter with 4 tests and the exact `viewport_record_keeps_histories_per_camera_key` test with 1 test.

For the per-camera previous-state expansion, the filtered lib-test wrapper first exposed and fixed a moved-value issue in the new `motion_vector_camera.rs` test. A subsequent `cargo test -p zircon_runtime --lib viewport_record_keeps --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-prev-state-0620 --message-format short --color never -- --test-threads=1 --nocapture` run exceeded the tool window and did not leave a runnable lib-test binary, so no test pass is claimed for that wrapper yet. `cargo check -p zircon_runtime --lib --tests ...` remains blocked by unrelated stale `RenderMeshSnapshot` fixture fields in `zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs`.

For Plan 09 CO-M4 selected-camera layer-key convergence, `camera_history_key.rs` now carries the culling and volume masks in the history slot key. Scoped `rustfmt --edition 2021`, `rustfmt --edition 2021 --check`, static debt scans, line-count scan, and `git diff --check` passed for the touched file. The focused locked Cargo guard `camera_history_key_distinguishes_culling_layers_without_legacy_loss` was blocked before compilation because the current workspace `Cargo.lock` would need an update while `--locked` was passed, so no new Cargo pass is claimed for this slice. Status anchor: `render_plan09_history_key_layer_masks_static_passed_cargo_lock_blocked`.
