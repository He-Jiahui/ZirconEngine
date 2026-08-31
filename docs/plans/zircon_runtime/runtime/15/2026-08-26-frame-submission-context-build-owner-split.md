---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/budget_degrade.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/effective_view_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/ui_submission_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/tests.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/budget_degrade.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/effective_view_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/ui_submission_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/tests.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/tests.rs::frame_submission_context_build_root_remains_a_declarative_orchestration_owner
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/tests.rs::context_build_moves_owned_viewport_and_virtual_geometry_payloads
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/tests.rs::post_process_mutation_reborrows_before_building_the_view_family
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/tests.rs::submission_view_family_preserves_the_selected_camera_viewport_rect
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 frame submission context build owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M3/M4 | Frame submission context build folder-backed owner split | `runtime_15_frame_submission_context_build_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 803 -> 506 lines; budget/view/UI/test owners 51/98/23/193 lines; 25/25 definitions, 8/8 old tests, and 13/13 root anchors retained. |

Completed:

- Kept the ordered `FrameSubmissionContext` construction and VG/HGI product payload selection in `build.rs`.
- Split budget degradation projection into `build/budget_degrade.rs`.
- Split view-family, history, post-process, prior-particle, and graph projection into `build/effective_view_state.rs`.
- Split UI command statistics into `build/ui_submission_stats.rs`.
- Moved all eight inline tests into folder-backed `build/tests.rs` and added a root ownership regression.
- Preserved existing Runtime 07 F3 shared-extract and Plan 08 product anchors without relaxing their guards.
- Left the parent facade, callers, and concurrently modified sibling owners untouched.

## Review basis

The local Unreal view setup and visibility initialization routes separate view-rect policy and begin/end view initialization from the top-level renderer route. This split applies the same ownership principle to Zircon while preserving Zircon's current API and ordered frame semantics.

There is no compatibility module, duplicate implementation, generic helper owner, algorithm replacement, or hotpath instrumentation change.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed across all 17 currently owned Rust files.
- Final scoped `git diff --check` passed, apart from LF/CRLF checkout notices.
- Static migration comparison retained all 25 old definitions and all 8 old tests, then added one structure regression.
- Root product/hotpath contract passed 13/13; four forbidden extract/payload clone anchors remain absent.
- All 29 current goal files exist and contain no trailing whitespace or conflict markers; both new Runtime 15 plan-path contracts have zero missing paths.
- Managed Cargo and live rendering tests were not run while bypassing the current validation blocker. They remain required before accepted milestone closeout.
- No CPU, GPU, energy, or power improvement is claimed because this slice does not change the frame-build algorithm.

## Open scope

Runtime 07, Runtime 15, and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for frame submission context construction. Managed compile/test, live render behavior, wider structure/performance guards, milestone commit, coordinator integration receipt, and WeCom publication remain open.
