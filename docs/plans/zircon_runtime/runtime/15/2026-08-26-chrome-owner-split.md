---
related_code:
  - zircon_runtime/src/ui/surface/render/chrome.rs
  - zircon_runtime/src/ui/surface/render/chrome/commands.rs
  - zircon_runtime/src/ui/surface/render/chrome/content.rs
  - zircon_runtime/src/ui/surface/render/chrome/metadata.rs
  - zircon_runtime/src/ui/surface/render/chrome/metrics.rs
  - zircon_runtime/src/ui/surface/render/chrome/state.rs
  - zircon_runtime/src/ui/surface/render/chrome/style.rs
  - zircon_runtime/src/ui/tests/render_chrome.rs
implementation_files:
  - zircon_runtime/src/ui/surface/render/chrome.rs
  - zircon_runtime/src/ui/surface/render/chrome/commands.rs
  - zircon_runtime/src/ui/surface/render/chrome/content.rs
  - zircon_runtime/src/ui/surface/render/chrome/metadata.rs
  - zircon_runtime/src/ui/surface/render/chrome/metrics.rs
  - zircon_runtime/src/ui/surface/render/chrome/state.rs
  - zircon_runtime/src/ui/surface/render/chrome/style.rs
  - zircon_runtime/src/ui/tests/render_chrome.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
tests:
  - zircon_runtime/src/ui/tests/render_chrome.rs::chrome_separator_parsing_does_not_allocate_lowercase_text
  - zircon_runtime/src/ui/tests/render_chrome.rs::chrome_projects_shared_density_control_and_typography_metrics
  - zircon_runtime/src/ui/tests/render_chrome.rs::render_extract_expands_workbench_chrome_surfaces
  - zircon_runtime/src/ui/tests/render_chrome.rs::render_extract_chrome_uses_shared_unavailable_and_active_state_priority
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 chrome render owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M3/M4 | Runtime chrome render folder-backed owner split | `runtime_15_chrome_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 702 -> 62 lines; six production child owners 135/73/103/175/61/212 lines; 32/32 old free functions and required classification/token/geometry anchors retained. |

Completed:

- Kept component classification, frame validation, state/metric resolution, and concrete dispatch in the root owner.
- Split typed kind/label/icon/style metadata decoding into `metadata.rs`.
- Split dynamic painter-state folding into `state.rs`.
- Split design-token projection and state-dependent visual selection into `style.rs`.
- Split shared density/typography projection and separator/content geometry into `metrics.rs`.
- Split command encoding from the concrete chrome command sequence.
- Adjusted the existing source-contract regressions to inspect the full owner family while adding all child mounts, classification-before-state, and an 80-line root budget.
- Left the already modified parent render facade and extraction consumer untouched.

## Review basis

Local Unreal Slate keeps border style/content/padding, dock content, viewport content, and paint responsibilities explicit. The existing Zircon retained-host renderer uses a folder-backed `model/palette/fill/selection/separators` chrome owner family with a separate command stream. This slice applies that common owner model to runtime rendering while preserving the current immediate command API and behavior.

There is no compatibility module, duplicate implementation, generic helper dump, public API expansion, algorithm replacement, or hotpath instrumentation change.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all eight touched Rust files.
- Scoped `git diff --check` passed, apart from LF/CRLF checkout notices.
- Static migration comparison retained all 32 old free-function definitions and added only named concrete-assembly and separator-color functions.
- Root/source contracts confirmed classification before state resolution, no lowercase copy, all six child mounts, a 62-line root, and all design-token/style-override/geometry hooks.
- Production files contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Managed Cargo and live rendering tests were not run while bypassing the current validation blocker. They remain required before accepted milestone closeout.
- No CPU, GPU, energy, or power improvement is claimed because this slice does not change the renderer algorithm.

## Open scope

Runtime 15 and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for runtime chrome command construction. Managed compile/test, live render behavior, wider structure/performance guards, milestone commit, coordinator integration receipt, and WeCom publication remain open.
