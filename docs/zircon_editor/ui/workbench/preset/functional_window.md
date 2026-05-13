---
related_code:
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/functional_window.rs
  - zircon_editor/src/ui/workbench/preset/mod.rs
implementation_files:
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/functional_window.rs
  - zircon_editor/src/ui/workbench/preset/mod.rs
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

# Unreal Window Model Preset

`UnrealWindowModelPreset` is the editor-side functional-window contract for the new Workbench direction. It wraps the default `EditorFunctionalWindowPreset` list so Unreal-style feature editors are represented as a window model rather than a loose vector on the design stack.

Each `EditorFunctionalWindowPreset` records:

- `kind`, the stable functional window role;
- `title`, the visible window title;
- `dock_policy`, which determines whether the window is the main Workbench, a docked document, floating-capable, or drawer-backed;
- `primary_views`, the document/tab views that make up the window body;
- `drawer_views`, the tool views owned by that window.

The default model declares the Workbench, Scene/Game, Prefab Editor, Material Editor, UI Asset Editor, Animation Editor, Asset Browser, and Diagnostics windows.

## Window Categories

The Workbench is the single `MainWorkbench` window. It owns Scene/Game as central documents and the first Fyrox-style tool drawers.

Prefab, Material, UI Asset, and Animation are `FloatingAllowed` feature editors. They project to native-window host mode in the default layout and expose their own primary view tabs, while shared drawers such as Inspector still reuse the JetBrains shell slot mapping.

Asset Browser and Diagnostics are `DrawerBacked` utility windows. They can be opened as functional windows while still preserving the tool-window mental model from the main Workbench.

## Projection

`EditorUiDesignStack::default_workbench_layout()` walks `window_model.windows` when creating `ActivityWindowLayout` records. The layout projection maps `dock_policy` to host mode, assigns primary views to `DocumentNode::Tabs`, and maps drawer views through the shell preset plus fallback classifier. The Material foundation catalog also exposes `DocumentNode`, `TabStack`, and `FloatingWindow` as authorable shell components, so v2 demo assets can represent the same split/tab/detach concepts used by the Rust layout model.

`EditorUiDesignStack::default_view_instances()` also walks `window_model.windows`, giving Workbench views the historical `#1` suffix and functional editor views window-scoped suffixes such as `#material_editor`.

## Tests

`default_stack_binds_unreal_window_model_contract` verifies the model has one Workbench, four floating feature editors, and two drawer-backed utility windows. `default_layout` verifies those windows become independent activity-window layouts, and `default_registry` verifies the resulting view instances can sync into `EditorWindowRegistry` without id collisions.
