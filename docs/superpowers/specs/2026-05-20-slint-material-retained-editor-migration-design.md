# Slint Material Retained Editor Migration Design

## Goal

Migrate the behavior and visual foundations of `dev/material-rust-template/material-1.0` into the Zircon Editor retained UI path without making Slint runtime, generated `.slint`, or `@material` the Editor business UI source.

The accepted target is option 2 from the prior decision: reproduce Slint Material component behavior slice-by-slice inside Zircon's retained `.ui.toml` / `.zui` / runtime UI / editor host architecture. Hub may keep its own Slint Material wrapper work, but this migration is specifically for `zircon_editor` retained UI.

## Architecture

`zircon_editor` owns authoring and host presentation. It should receive Material component semantics through retained assets, style tokens, component descriptors, template projection, and host painter metadata. It must not link Slint, import `material.slint`, or depend on Slint callbacks for editor behavior.

`zircon_runtime` and `zircon_runtime_interface` remain the shared UI contract and behavior owners for lower layers: component descriptors, typed values/events, layout passes, surface frame authority, input routing, and render extract. This M0/M1 slice does not edit those lower layers; it documents the mapping and extends Editor theme tokens so later runtime-support milestones can consume the same Slint Material evidence.

## Source Evidence

Primary reference: `dev/material-rust-template/material-1.0`. The migration treats the following files as foundation evidence:

- `dev/material-rust-template/material-1.0/material.slint`
- `dev/material-rust-template/material-1.0/ui/styling/material_palette.slint`
- `dev/material-rust-template/material-1.0/ui/styling/material_schemes.slint`
- `dev/material-rust-template/material-1.0/ui/styling/material_style_metrics.slint`
- `dev/material-rust-template/material-1.0/ui/styling/material_typography.slint`
- `dev/material-rust-template/material-1.0/ui/styling/material_animations.slint`
- `dev/material-rust-template/material-1.0/ui/components/state_layer.slint`
- `dev/material-rust-template/material-1.0/ui/components/elevation.slint`
- `dev/material-rust-template/material-1.0/ui/components/base_button.slint`
- `dev/material-rust-template/material-1.0/ui/components/text_field.slint`
- `dev/material-rust-template/material-1.0/ui/components/check_box.slint`
- `dev/material-rust-template/material-1.0/ui/components/radio_button.slint`
- `dev/material-rust-template/material-1.0/ui/components/switch.slint`
- `dev/material-rust-template/material-1.0/ui/components/slider.slint`
- `dev/material-rust-template/material-1.0/ui/components/menu.slint`
- `dev/material-rust-template/material-1.0/ui/components/dialog.slint`
- `dev/material-rust-template/material-1.0/ui/components/drawer.slint`
- `dev/material-rust-template/material-1.0/ui/components/navigation_bar.slint`
- `dev/material-rust-template/material-1.0/ui/components/tab_bar.slint`

Stabilizing references are the existing Zircon retained UI docs and plans: `.codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md`, `.codex/plans/ZirconEngine UI 组件化与 Material 样式器重构计划.md`, `.codex/plans/Runtime UI 组件库与 Slint Material Showcase Cutover 计划.md`, and the current Material design matrix/audit docs.

## Retained Mapping Model

Every Slint Material export lands in one of five retained buckets:

- Foundation token: palette, scheme, metric, typography, animation, state layer, elevation.
- Primitive/component descriptor: buttons, checkbox, radio, switch, slider, text field, list item, menu item, icon, progress.
- Composite retained asset: app bar, drawer, dialog, card, navigation, tab bar, search bar, date/time picker, menu, snackbar, bottom sheet.
- Behavior utility: modal, popup, state layer area, focus touch area, extended touch area, grid/horizontal/vertical layout helpers.
- Host/painter support gap: ripple animation, elevation shadows, menu/dialog overlay lifetime, track/thumb primitives, spinner/progress arc, date/time calendar grid, and advanced navigation motion.

The mapping is documented in `docs/ui-and-layout/slint-material-retained-editor-migration.md`. That document is the M0 acceptance artifact and must enumerate every export from `material.slint`, every source path, the retained owner, the milestone, and the intentional divergence.

## Milestones

M0 Evidence and Spec freezes the source inventory, migration rules, no-Slint Editor dependency fence, docs index link, and boundary tests that tie the docs to `material.slint`.

M1 Foundation Tokens brings Slint Material dark palette roles, state-layer opacities, metric ladder, typography scale, animation durations/easings, and elevation-shadow roles into `zircon_editor/assets/ui/theme/editor_material.v2.ui.toml`. Existing compact Editor-specific selectors may remain, but the Slint Material source roles must be present under stable token names.

M2 State Layer, Ripple, Elevation defines retained state-layer behavior from `state_layer.slint`: hover/focus/press/disabled/drag opacity priority, keyboard activation through focus touch areas, tooltip gating, ripple data, and elevation shadow levels. Production implementation belongs in shared runtime UI and retained host painter modules, not in docs-only assets.

M3 Core Controls ports BaseButton/Button variants, IconButton, TextButton, checkbox, radio, switch, slider, text field/search field, list tile, menu item, and progress indicators using retained descriptors, `.zui` prototypes, event routes, and painter coverage.

M4 Surfaces, Navigation, and Overlays ports app bars, cards, drawers, dialogs, menus, navigation bar/drawer/rail, tabs, snackbars, tooltip, date/time picker popups, modal bottom sheets, and layout helper components.

M5 Editor Shell Adoption connects the completed retained components into Editor workbench surfaces where they replace current approximate Material rows without reintroducing Slint business UI.

M6 Validation and Cutover runs the declared focused Cargo gates, static dependency fences, diff checks, docs checks, Material Lab/projection/painter tests, and later visual capture evidence before declaring the migration complete.

## M0 And M1 Acceptance

M0 is complete when the spec, implementation plan, mapping doc, docs index entry, session note, and boundary tests exist, and the tests enforce the full `material.slint` export inventory plus the no direct `zircon_editor` Slint dependency fence.

M1 is complete when `editor_material.v2.ui.toml` exposes Slint Material foundation tokens with stable names and the boundary test proves palette, metric, typography, animation, state-layer, and elevation roles are present with values derived from the local Slint Material template.

This session only claims focused static validation. Full Cargo build/test remains the milestone testing-stage responsibility because the worktree is heavily dirty and unrelated UI/render/session lanes currently own broad compile blockers.

## Deliberate Divergence

Zircon retains its compact dark editor density and existing selector vocabulary where that is already part of Editor UX. Slint Material roles are added as source-aligned foundation tokens and behavior contracts, not blindly applied as a pixel-for-pixel light/dark scheme replacement.

Slint `PopupWindow`, `TouchArea`, and animation behavior is translated into retained surface, event, and painter concepts. No `.slint` component tree becomes the Editor UI source of truth.
