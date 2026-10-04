---
related_code:
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/mod.rs
  - zircon_editor/src/ui/workbench/preset/shell_preset.rs
implementation_files:
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/mod.rs
  - zircon_editor/src/ui/workbench/preset/shell_preset.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - cargo test -p zircon_editor --lib default_stack --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 8 passed)
  - cargo test -p zircon_editor --lib default_layout --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 5 passed)
  - cargo test -p zircon_editor --lib default_registry --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 2 passed)
doc_type: module-detail
---

# JetBrains Shell Preset

`JetBrainsShellPreset` is the editor-side shell contract for side drawers, document tabs, and detachable tool windows. It sits beside the Material visual skin, Fyrox panel contracts, and Unreal functional window model so shell behavior can be restyled later without rewriting panel composition.

## Drawer Model

Each `JetBrainsDrawerPreset` is keyed by canonical `ActivityDrawerSlot` and records:

- `label`, the product-facing drawer group name;
- `default_mode`, the startup drawer state;
- `visible_views`, the view ids that can appear in that drawer group;
- detach and attach permissions;
- whether the drawer can collapse to an activity bar entry;
- whether drawer extent and selected view should persist.

The default stack declares four drawer groups. `LeftTop` exposes Hierarchy and Asset Browser as project tools. `LeftBottom` exposes Module Plugins and starts collapsed. `RightTop` exposes Inspector. `Bottom` exposes Console, Runtime Diagnostics, and Desktop Export.

The preset intentionally stores view ids instead of `ViewInstance` ids. Window-scoped instance suffixes still belong to the default registry projection, while the shell preset describes the stable product drawer contract. `drawer_for_view`, `drawer_slot_for_view`, and `default_mode_for_slot` are the query helpers used by the default layout and view-instance projection so shell routing has one source of truth.

## Tab And Floating Behavior

`JetBrainsTabBehavior` captures the workbench tab rules expected from the shell: tabs are reorderable, dropped tabs activate, middle-click can close document tabs, and tool tabs remain close-guarded.

`JetBrainsFloatingWindowBehavior` captures the detachable-window contract: tool views detach into native windows, retain a path back to their original drawer, take focus when detached, persist floating geometry, and restore hidden drawer state when attached again.

These flags are behavioral requirements for later retained-host and window-manager integration. The preset itself does not open windows, mutate layout state, or render a tab strip.

## Default Layout Use

`EditorUiDesignStack::default_workbench_layout()` uses the shell preset to route Workbench drawer views and to apply drawer startup mode. The default Modules drawer therefore keeps the Plugin Manager tab but starts collapsed, matching the JetBrains-style lower tool window behavior. Shared feature-editor views such as Inspector reuse the same shell slot; feature-editor-only views such as Asset Preview and Material Graph fall back to the view-id classifier until they receive their own shell preset.

## Integration Guard

`EditorUiDesignStack::material_fyrox_jetbrains_unreal()` builds the default shell through `default_jetbrains_shell_preset()`. The `default_stack_binds_jetbrains_shell_drawers_and_detach_contracts` test verifies the first drawer and detach defaults. The `default_stack_shell_contract_covers_workbench_drawer_views` test requires every drawer view on the main Workbench functional window to have a shell drawer assignment. The `default_layout` tests then prove the projected Workbench layout preserves those shell slots and the collapsed Modules drawer mode.

This guard keeps the new default UI path aligned with the user's requested direction: Fyrox-like panel contents, JetBrains-like drawers and tabs, and Unreal-like feature windows, all layered as replaceable presets rather than hard-coded layout branches.
