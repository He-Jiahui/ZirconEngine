---
related_code:
  - zircon_runtime/src/ui/surface/render/selection_controls.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/checkbox.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/commands.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/geometry.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/metadata.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/radio.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/state.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/style.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/toggle.rs
  - zircon_runtime/src/ui/tests/render_selection_controls.rs
implementation_files:
  - zircon_runtime/src/ui/surface/render/selection_controls.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/checkbox.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/commands.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/geometry.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/metadata.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/radio.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/state.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/style.rs
  - zircon_runtime/src/ui/surface/render/selection_controls/toggle.rs
  - zircon_runtime/src/ui/tests/render_selection_controls.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
tests:
  - zircon_runtime/src/ui/tests/render_selection_controls.rs::selection_control_rendering_uses_central_tokens_and_validated_overrides
  - zircon_runtime/src/ui/tests/render_selection_controls.rs::render_extract_expands_selection_control_indicators
  - zircon_runtime/src/ui/tests/render_selection_controls.rs::render_extract_checked_controls_keep_active_visuals_when_hot
  - zircon_runtime/src/ui/tests/render_selection_controls.rs::render_extract_selection_controls_keep_focused_surface_neutral_until_hovered
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 selection controls render owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M3/M4 | Runtime checkbox/radio/toggle render folder-backed owner split | `runtime_15_selection_controls_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 829 -> 63 lines; eight production child owners 97/123/83/120/62/92/268/62 lines; 43/43 old functions and required state/token/parser anchors retained. |

Completed:

- Kept classification-first extraction, frame validation, visual/state resolution, and concrete dispatch in the root owner.
- Split component identity, painter family, label, metric, boolean, and color parsing into `metadata.rs`.
- Split checked/selected/disabled and painter-state folding into `state.rs`.
- Split design-token projection and state-dependent visual selection into `style.rs`.
- Split shared mark/label/track/thumb/dot frame calculations into `geometry.rs`.
- Split label emission and render-command encoding into `commands.rs`.
- Split checkbox, radio, and toggle concrete command construction into independent leaf owners.
- Adjusted the existing source-contract regression to inspect the full owner family while adding classification ordering, child-mount, and root-budget guards.
- Left the already modified parent render facade, extraction route, and external call sites untouched.

## Review basis

Local Unreal Slate review separates checkbox type, checked state, style/content layout, and concrete visual rebuilding. The existing Zircon retained-host renderer uses the same identity/style/commands and concrete-control direction. This slice applies that owner model to runtime rendering while preserving the current immediate command API and behavior.

There is no compatibility module, duplicate implementation, generic helper dump, public API expansion, algorithm replacement, or hotpath instrumentation change.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all ten touched Rust files.
- Scoped `git diff --check` passed, apart from LF/CRLF checkout notices.
- Static migration comparison retained all 43 old function definitions and added only two read-only state accessors.
- Root/source contracts confirmed classification before state folding, all eight child mounts, a 63-line root, required design-token/parser hooks, and absence of the three legacy visual constants.
- Production `panic!`, `unwrap`, `expect`, and `allow(dead_code)` anchors remain absent.
- Managed Cargo and live rendering tests were not run while bypassing the current validation blocker. They remain required before accepted milestone closeout.
- No CPU, GPU, energy, or power improvement is claimed because this slice does not change the renderer algorithm.

## Open scope

Runtime 15 and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for runtime checkbox/radio/toggle command construction. Managed compile/test, live render behavior, wider structure/performance guards, milestone commit, coordinator integration receipt, and WeCom publication remain open.
