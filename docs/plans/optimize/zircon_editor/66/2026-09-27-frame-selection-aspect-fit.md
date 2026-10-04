---
title: Editor66 Frame Selection Aspect Fit
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: product_gate_pending
---

# Editor66 Frame Selection aspect fit

## Defect and scope

[ED66-P1-14 / G23](../66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md) identifies a concrete clipping defect: Frame Selection sized its camera using vertical FOV or vertical orthographic extent alone. In a 320 × 1280 viewport, two selected node origins at X = ±8 with a 60° vertical perspective lens were projected to approximately X = −322 and 642. Orthographic projection put them at approximately −397 and 717. These are analytical counterexamples from the preimage geometry; managed Rust reproduction is pending.

The production path is retained toolbar FrameSelection → command dispatch → `EditorViewportEvent::FrameSelection` → `EditorState::apply_viewport_command` → `SceneViewportController::apply_command` → the [Frame Selection owner](../../../../../zircon_editor/src/scene/viewport/controller/scene_viewport_controller_frame_selection.rs). The existing [projection context](../../../../../zircon_editor/src/scene/viewport/projection.rs) confirms that perspective FOV is vertical and `ortho_size` is a vertical half-extent.

This Q batch changes that private fit calculation and adds its test child. It continues to use the existing aggregate of selected node origins. True mesh/subtree/component bounds, clipping policy, invalid-lens rejection and typed product receipts remain separate work.

## Geometry and reference evidence

The local Unreal primary reference, [FEditorViewportClient::FocusViewportOnBox](../../../../../dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportClient.cpp), obtains the viewport aspect at lines 817–832, adapts perspective framing to aspect at 842–866, and chooses the smaller viewport axis for orthographic fit at 909–925. Zircon's existing vertical-FOV projection is the authority for the local formulas.

- Apply the current normalized viewport size to the camera before fitting.
- Preserve the existing vertical fit and distance lower bounds. For perspective, also require the padded sphere to fit the horizontal half-angle `atan(tan(fov_y / 2) * aspect)`; taking the larger required distance satisfies both axes. Dividing the old distance by aspect would overestimate the distance for a wide vertical lens.
- For orthographic projection, divide the padded radius by `min(aspect, 1)` when choosing the vertical half-extent. Preserve the existing orthographic camera distance, minimum extent, lens and clip planes.
- Preserve empty/no-scene feedback, selection center, the 6-unit distance floor, zero-offset direction fallback, and viewport dimension normalization.

## Regression coverage

Seven tests in [aspect_fit_tests.rs](../../../../../zircon_editor/src/scene/viewport/controller/scene_viewport_controller_frame_selection/aspect_fit_tests.rs) were authored before the production repair. Fixtures use real scene nodes, the scene camera component, selection APIs and the `FrameSelection` command entry. Visibility assertions use the existing projection context on the resulting selected world positions.

| Test | Observable contract |
| --- | --- |
| `frame_selection_command_fits_perspective_selection_in_portrait_and_landscape` | Eight selected corners, including depth extent, project inside both 320 × 1280 and 1280 × 320 viewports. |
| `frame_selection_command_fits_orthographic_selection_in_portrait_and_landscape` | The same portrait/landscape visibility contract holds in orthographic projection; viewport aspect does not change camera distance. |
| `frame_selection_command_uses_horizontal_fov_for_a_wide_vertical_lens` | A 120° vertical lens yields the expected screen-space golden, detecting the excessive distance from a naive inverse-aspect multiplier. |
| `frame_selection_command_preserves_larger_existing_distance_and_lens` | A camera already 150 units away retains that distance, projection, FOV and clip planes in both modes. |
| `frame_selection_command_preserves_single_point_floors_and_zero_offset_fallback` | Single-point selection retains the existing distance/orthographic floors and handles a camera at the selection center. |
| `frame_selection_command_without_selection_or_scene_preserves_camera_and_orientation` | No camera update or camera/pivot/orientation mutation is reported for absent input. |
| `frame_selection_command_uses_resized_and_normalized_viewport_dimensions` | Landscape-to-portrait resize and zero width/height follow existing normalization and remain visible in both modes. |

The preexisting multiselection-center test remains unchanged. Dynamic red/green and the new screen-space golden are not yet accepted by managed execution.

## Evidence and acceptance

| Gate | Status |
| --- | --- |
| Ownership | Four paths claimed by request `ef0b8da3b3c048dd9b7aa2f53538575e`; all were checked outside sealed O/P source manifests. The existing production owner had no foreign diff. Editor1020 was unoccupied. |
| Record authorization | Optimize maintenance request `4043cc8d5b734d82a038ef1d0c2e1c58`; Astra maintenance request `f8b783ea362a4a8a9cef24ba7e7eabcc`. |
| Static evidence | Exact preimage/diff, tests-first snapshot, scoped Rust formatting, byte-preserving inverse, document structure checks and final source manifest use prefix `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-q-editor66-frame-selection-aspect-fit-*`. |
| Review and dynamic acceptance | Independent source review passed against the final four-path manifest, including all seven command/projection tests and local Unreal reference evidence. Grouped Windows Editor validation remains pending under Q. This implementation agent ran no Cargo or validation-state polling. |
| ED66-G23 | Aspect/FOV fit is implemented for the existing node-origin aggregate; acceptance remains pending managed execution. |
| ED66-G22/G24 and adjacent work | Actual aggregate bounds, near/far and large-world clipping, invalid-lens handling, zoom-in policy, axis identity, truthful product receipts and camera history remain open. The orthographic aspect slice does not close G24. |
| Product and performance | Native product verification, 100K-selection CPU/allocation measurements (G46), large-world tests (G47), and reproducible cross-engine comparisons (G48) remain pending. No timing threshold or performance pass is claimed. |

The change adds constant-size projection math after the existing selection scan. The scan and its complexity are unchanged; actual allocation and latency results require measurement.
