---
related_code:
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/mod.rs
  - zircon_editor/src/ui/workbench/preset/panel_preset.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation.rs
implementation_files:
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/mod.rs
  - zircon_editor/src/ui/workbench/preset/panel_preset.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - cargo test -p zircon_editor --lib default_stack --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 8 passed)
doc_type: module-detail
---

# Fyrox Panel Preset

`FyroxPanelPreset` is the editor-side panel composition contract for the new UI direction. It keeps the Fyrox-inspired panel contents separate from the Material visual skin, the JetBrains shell, and the Unreal-style functional window model.

Each preset is keyed by `view_id` and records:

- `title`, the product-facing panel name;
- `components`, the required logical component roles inside the panel;
- `interactions`, the behavior categories the panel must support.

The component roles intentionally name domain pieces, not concrete renderer widgets. For example, `editor.inspector` declares `PropertyGrid`, `InspectorSection`, and `FieldEditor`; `editor.hierarchy` declares `SearchField`, `TreeView`, and `ContextMenu`; `editor.assets` and `editor.asset_browser` declare folder tree, grid/list, preview, and metadata roles. A future Unreal or JetBrains style can remap those roles to a different visual composition while preserving panel ownership and behavior.

`FyroxPanelComponentRole::component_id()` maps each role to the current Material foundation component descriptor id. This is a deliberately small bridge: the panel preset still speaks in editor-domain roles, while v2 assets and renderer-facing catalogs can resolve those roles to concrete component contracts.

`EditorUiDesignStack::material_fyrox_jetbrains_unreal()` populates panel presets for every primary view and drawer view declared by the default functional windows. The `default_stack_has_panel_contracts_for_every_declared_view` test is the guard: adding a new preset window or view now requires explicitly choosing its panel contract instead of leaving it as an untyped placeholder.

The `default_stack_fyrox_panel_roles_resolve_to_material_components` test is the second guard: every role used by those panel presets must resolve to `UiComponentDescriptorRegistry::material_editor_foundation()`. This prevents Fyrox-style panel design from drifting away from the component catalog that Material, Unreal, and JetBrains skins will reuse.

This file does not render UI and does not depend on retained host or runtime scene state. It is safe to use for menu, descriptor, snapshot, or asset-generation stages that need to understand what a panel is before picking a concrete `.ui.toml` body.
