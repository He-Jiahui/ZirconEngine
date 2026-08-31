---
related_code:
  - zircon_runtime/src/ui/surface/render/segmented_controls.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/commands.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/metadata.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/segments.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/state.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/style.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/tabs.rs
  - zircon_runtime/src/ui/tests/render_segmented_controls.rs
implementation_files:
  - zircon_runtime/src/ui/surface/render/segmented_controls.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/commands.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/metadata.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/segments.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/state.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/style.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls/tabs.rs
  - zircon_runtime/src/ui/tests/render_segmented_controls.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
tests:
  - zircon_runtime/src/ui/tests/render_segmented_controls.rs::segmented_rendering_classifies_before_state_and_avoids_selected_lowercase_copy
  - zircon_runtime/src/ui/tests/render_segmented_controls.rs::render_extract_expands_tabs_and_segmented_control_primitives
  - zircon_runtime/src/ui/tests/render_segmented_controls.rs::render_extract_segmented_defaults_to_accent_token_selected_indicator
  - zircon_runtime/src/ui/tests/render_segmented_controls.rs::render_extract_segmented_and_tab_keep_focused_surface_neutral_until_hovered
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 segmented controls render owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M3/M4 | Runtime segmented/tab render folder-backed owner split | `runtime_15_segmented_controls_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 838 -> 57 lines; six production child owners 86/169/211/78/292/77 lines; 42/42 old functions and required hotpath/token anchors retained. |

Completed:

- Kept classification-first extraction, frame validation, visual/state resolution, and concrete dispatch in the root owner.
- Split typed component/options/selection/label/style metadata decoding into `metadata.rs`.
- Split dynamic component and painter-state folding into `state.rs`.
- Split design-token projection and state-dependent visual selection into `style.rs`.
- Split command encoding from segmented and tab command construction.
- Split segmented-group and tab concrete rendering into independent leaf owners.
- Adjusted the existing source-contract regression to inspect the full owner family while retaining the root ordering assertion and adding a 80-line root budget.
- Left the already modified parent render facade and all external call sites untouched.

## Review basis

Local Unreal Slate review separates segmented-control style/state/child presentation and tab responsibilities from the containing route. The existing Zircon retained-host renderer uses the same identity/options/style/segments/tabs/commands direction. This slice applies that common owner model to runtime rendering while preserving the current immediate command API and behavior.

There is no compatibility module, duplicate implementation, generic helper dump, public API expansion, algorithm replacement, or hotpath instrumentation change.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all eight touched Rust files.
- Scoped `git diff --check` passed, apart from LF/CRLF checkout notices.
- Static migration comparison retained all 42 old function definitions and added only three read-only state accessors.
- Root/source contracts confirmed classification before state resolution, no `to_ascii_lowercase` copy, all six child mounts, a 57-line root, and all design-token/style-override hooks.
- Managed Cargo and live rendering tests were not run while bypassing the current validation blocker. They remain required before accepted milestone closeout.
- No CPU, GPU, energy, or power improvement is claimed because this slice does not change the renderer algorithm.

## Open scope

Runtime 15 and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for runtime segmented/tab command construction. Managed compile/test, live render behavior, wider structure/performance guards, milestone commit, coordinator integration receipt, and WeCom publication remain open.
