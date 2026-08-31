---
related_code:
  - zircon_runtime/src/ui/surface/render/buttons.rs
  - zircon_runtime/src/ui/surface/render/buttons/button.rs
  - zircon_runtime/src/ui/surface/render/buttons/commands.rs
  - zircon_runtime/src/ui/surface/render/buttons/icon_button.rs
  - zircon_runtime/src/ui/surface/render/buttons/metadata.rs
  - zircon_runtime/src/ui/surface/render/buttons/state.rs
  - zircon_runtime/src/ui/surface/render/buttons/style.rs
  - zircon_runtime/src/ui/tests/render_buttons.rs
implementation_files:
  - zircon_runtime/src/ui/surface/render/buttons.rs
  - zircon_runtime/src/ui/surface/render/buttons/button.rs
  - zircon_runtime/src/ui/surface/render/buttons/commands.rs
  - zircon_runtime/src/ui/surface/render/buttons/icon_button.rs
  - zircon_runtime/src/ui/surface/render/buttons/metadata.rs
  - zircon_runtime/src/ui/surface/render/buttons/state.rs
  - zircon_runtime/src/ui/surface/render/buttons/style.rs
  - zircon_runtime/src/ui/tests/render_buttons.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
tests:
  - zircon_runtime/src/ui/tests/render_buttons.rs::button_rendering_resolves_variant_once_without_joining_lowercase_text
  - zircon_runtime/src/ui/tests/render_buttons.rs::render_extract_expands_button_primitives
  - zircon_runtime/src/ui/tests/render_buttons.rs::render_extract_expands_icon_button_state_surface
  - zircon_runtime/src/ui/tests/render_buttons.rs::render_extract_button_and_icon_button_keep_focused_surface_neutral_until_hovered
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 button render owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M3/M4 | Runtime button render folder-backed owner split | `runtime_15_buttons_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 772 -> 63 lines; six production child owners 75/116/44/160/104/292 lines; 27/27 old free functions and required classification/token/parser anchors retained. |

Completed:

- Kept component admission, frame validation, visual/state resolution, and concrete dispatch in the root owner.
- Split typed component/variant/label/icon/style metadata decoding into `metadata.rs`.
- Split dynamic component and painter-state folding into `state.rs`.
- Split design-token projection and state-dependent visual selection into `style.rs`.
- Split command encoding from ordinary and icon-button command construction.
- Split ordinary button and icon-button geometry into independent leaf owners.
- Adjusted the existing source-contract regression to inspect the full owner family while adding all child mounts and a 80-line root budget.
- Left the already modified parent render facade and extraction consumer untouched.

## Review basis

Local Unreal Slate review separates button style, content/padding, interaction state, and paint responsibilities from the containing route. The existing Zircon retained-host renderer uses the same identity/content/style/surface/command direction and separates icon-button geometry. This slice applies that common owner model to runtime rendering while preserving the current immediate command API and behavior.

There is no compatibility module, duplicate implementation, generic helper dump, public API expansion, algorithm replacement, or hotpath instrumentation change.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all eight touched Rust files.
- Scoped `git diff --check` passed, apart from LF/CRLF checkout notices.
- Static migration comparison retained all 27 old free-function definitions with no missing or duplicate definition; state accessors are methods and do not duplicate policy.
- Root/source contracts confirmed one production variant classification, no joined lowercase copy, all six child mounts, a 63-line root, and all design-token/style-override/parser hooks.
- Production files contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Managed Cargo and live rendering tests were not run while bypassing the current validation blocker. They remain required before accepted milestone closeout.
- No CPU, GPU, energy, or power improvement is claimed because this slice does not change the renderer algorithm.

## Open scope

Runtime 15 and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for runtime button command construction. Managed compile/test, live render behavior, wider structure/performance guards, milestone commit, coordinator integration receipt, and WeCom publication remain open.
