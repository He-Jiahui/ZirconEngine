---
related_code:
  - zircon_runtime_interface/src/ui/skin/mod.rs
  - zircon_runtime_interface/src/ui/skin/preset.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/tests/ui_skin_contracts.rs
implementation_files:
  - zircon_runtime_interface/src/ui/skin/mod.rs
  - zircon_runtime_interface/src/ui/skin/preset.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
  - .codex/plans/ZirconEngine UI 组件化与 Material 样式器重构计划.md
tests:
  - zircon_runtime_interface/src/tests/ui_skin_contracts.rs
  - cargo check -p zircon_runtime_interface --lib --locked --target-dir target/codex-shared-b (2026-05-11: passed)
  - cargo test -p zircon_runtime_interface --lib ui_skin_contracts --locked --target-dir target/codex-shared-b (2026-05-11: passed, 2 passed)
doc_type: module-detail
---

# UI Skin Presets

`zircon_runtime_interface::ui::skin` owns the shared identity layer for editor UI style and product-model presets. It deliberately stores stable IDs and neutral DTOs in the interface crate so runtime, editor, and future tooling can agree on the active skin and authoring model without copying string constants.

## Preset IDs

The first combined editor UI direction is split into four independent presets:

- `material_dark` is the default visual skin. It owns Material Dark semantic tokens and visual component states.
- `fyrox_panel` is the panel-content preset. It names the editor panels whose content and interaction patterns should follow Fyrox: scene viewer, hierarchy, inspector, asset browser, console, and plugin manager.
- `jetbrains_shell` is the window-shell preset. It names the shell roles for side drawers, tool window tabs, document tabs, and detached floating windows.
- `unreal_window_model` is the functional-window preset. It names Unreal-style workbench/asset-editor windows as the top-level editing units.

These IDs are not rendering code. They are durable contract names consumed by component catalogs, editor workbench presets, asset fixtures, and later user-selectable skin switching.

## Material Dark Tokens

`UiDesignPresetDescriptor::material_dark()` exposes semantic token families instead of hard-coded widget colors:

- palette: primary, secondary, status colors, and mode;
- text: primary, secondary, disabled;
- action: active, hover, pressed, selected, disabled;
- surface/divider/focus/elevation/radius/spacing/typography/icon-size.

This mirrors Material UI and Slint Material enough for shared component styling while keeping engine editor components free to translate the values into retained UI paint, surface render commands, or native host state.

## Visual States

The Material skin advertises the full first-stage primitive state set: normal, hover, pressed, selected, disabled, focused, error, and warning. Runtime component descriptors can require the same state names without depending on a concrete renderer.
