---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_distance_field.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/distance_field_effects.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_distance_field.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/distance_field_effects.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tests::runtime_absorption::structure_convention::production_file_budget::global_budget::runtime_15_no_oversized_production_files (2026-07-13: passed 1/1)
  - tests::runtime_absorption::structure_convention::production_file_budget::render_ui_screen_space_render::runtime_15_screen_space_ui_render_tests_are_child_owner_split (2026-07-13: passed 1/1)
  - tests::runtime_absorption::structure_convention::test_file_budget::global_budget::runtime_15_no_oversized_test_files (2026-07-13: passed 1/1)
  - standalone structure-convention aggregate (2026-07-13: passed 1304/1304)
  - standalone code-review-findings aggregate (2026-07-13: passed 80/80)
status: in_progress
doc_type: module-detail
---

# Screen-Space Text Distance-Field Policy

## Purpose

`render/text_distance_field.rs` owns the conversion from a resolved UI text render request to the glyph raster distance-field mode used by screen-space UI batching. The parent `render.rs` remains the batch planner and delegates this policy decision instead of accumulating glyph-raster implementation details.

The policy preserves the existing behavior:

- explicit `Sdf` requests select the SDF atlas format;
- explicit `Msdf` and `Mtsdf` requests select the MSDF atlas format;
- `Auto` and `Native` begin from the alpha-mask request and allow outline, shadow, glow, or true-distance requirements to promote the request through the shared glyph raster policy;
- requests that the shared policy cannot resolve fall back to `SdfMode::Sdf`, matching the pre-split behavior.

`render/tests/distance_field_effects.rs` owns the focused batch-planning regressions. It verifies that a small native text run with an outline is routed to an SDF batch and that glow on an MSDF request selects the MTSDF true-distance mode.

## Ownership Boundary

The child module does not own UI command traversal, batch placement, glyph shaping, atlas upload, or shader execution. Those remain in their established owners. This split is structural: it reduces `render.rs` and `render/tests.rs` below the repository's 800-line budgets without changing the render contract or adding a compatibility path.

The focused file-budget and child-owner gates plus the full structure and code-review-findings aggregates are complete. Cargo behavior gates remain part of the active Runtime 02 testing stage.
