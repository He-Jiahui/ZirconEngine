---
related_code:
  - docs/ui-and-layout/ai-workbench-style/component-prototype/routes.js
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_module_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_additional_module_workspaces.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
plan_sources:
  - user: 2026-06-03 Continue the componentized editor workbench; keep layout component-based and make every button visibly responsive
  - docs/ui-and-layout/componentized-workbench-shell.md
  - docs/ui-and-layout/ai-workbench-style/component-prototype/README.md
tests:
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
  - node --check docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - cargo test -p zircon_editor --lib workbench_shared_module_commands_route_feedback_to_active_module_output --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-preview-0603 --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Module Command Feedback

## Purpose

`module_command_feedback.rs` owns preview-only feedback for Workbench module commands in the retained native editor surface. The module does not execute real gameplay, asset, renderer, or UI-authoring behavior. It updates existing status and output-row component properties so every authored Workbench command gives visible feedback through the same retained projection path used by other componentized controls.

The browser prototype has the same semantic boundary in `routes.js`: shared toolbar labels are component commands first, not hard-coded engine actions. Native feedback mirrors that idea by interpreting a shared command such as Compile, Diff, Save, or Simulate against the currently selected Workbench module.

## Shared Commands

The top module toolbar exposes one set of shared command controls in `workbench_top_toolbar.zui`: Save, Browse, Compile, Diff, and Simulate. `componentized_window.rs` still handles selection state and special Browse navigation. Once the action is recognized as a Workbench module action, it delegates to `apply_workbench_module_command_feedback(...)`.

The feedback module resolves the active module by reading the selected or checked state of the module tabs. That active module chooses the output row family:

- Effect writes to `WorkbenchEffectOutputRow`.
- Ability writes to `WorkbenchAbilityOutputRow`.
- Tags writes to `WorkbenchTagsValidationRow`.
- Perception writes to `WorkbenchPerceptionEventRow`.
- Material writes to `WorkbenchMaterialOutputRow`.
- Behavior writes to `WorkbenchBehaviorOutputRow`.
- Render writes to `WorkbenchRenderCaptureRow`.
- Assets writes to `WorkbenchAssetsOutputRow`.
- VFX writes to `WorkbenchVfxOutputRow`.
- HUD writes to `WorkbenchHudValidationRow`.

Scene mode has no module output surface, so shared command feedback only updates shell status and message count there.

## Panel Commands

Module-local panel buttons such as Material Compile, Behavior Validate, Assets Import, VFX Simulate, Ability Playtest, Render Compile, and HUD Preview keep their existing explicit feedback entries. They are intentionally separate from shared toolbar commands because their labels and output rows are authored inside specific module panels.

## Validation

`workbench_module_commands_update_status_and_module_output_rows` covers the existing explicit command feedback path and Browse's module switch to Assets.

`workbench_shared_module_commands_route_feedback_to_active_module_output` covers the module-scoped shared toolbar semantics. It selects Material, Behavior, Assets, and VFX, dispatches the same shared Compile command, and asserts that feedback lands on each module's own output row. The same test dispatches Diff and Simulate while VFX is active so the active-module route is not limited to Compile.

`verify-native-module-contract.mjs` also reads this Rust module directly. Its static contract requires the ten module tabs to resolve to the matching native output rows, keeps Save/Compile/Diff/Simulate routed through `active_module`, and links browser module-scoped commands such as `material:compile` and `vfx:compile` to the same native shared command action.
