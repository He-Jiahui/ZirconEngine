---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-26-native-wgpu-present-million-edit-diagnostic.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/graphics/tests/mod.rs
tests:
  - zircon_runtime/src/graphics/tests/runtime_ui_edit_native_present_profile.rs
---

# Runtime82 native WGPU present million character edit diagnostic completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Retained text edit to native WGPU present evidence | Added an ignored Windows Release test using real keyboard dispatch, a live Win32 WGPU surface, a native present ticket, advancing generation, same-generation retained output readback, and changed text-region pixels. The test uses winit's Windows worker-thread builder, a ten-second window-event deadline, a sixty-second libtest-thread receive deadline around the full GPU diagnostic, and the public capture API to finish submission before querying its receipt. | A separate managed Runtime Release test ticket is pending. A process-level timeout is not verified in the runner. This does not cover App/Dynamic Session ingress or establish the million-character product latency, allocation, RSS, and Unreal comparison thresholds. | implemented_pending_validation |

The fixture records one diagnostic sample per stage and keeps
`RTE-GATE-016` and `RTE-GATE-047` open. The captured texture is the retained
renderer output for the presented generation, not a direct swapchain read.
`present_enqueue_*_ns` and `finish_and_readback_*_ns` describe distinct API
wall times; neither alone is a native display-latency measurement.
