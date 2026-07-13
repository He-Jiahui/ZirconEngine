---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
plan_sources:
  - user: 2026-06-03 componentized editor UI prototype and native retained/Taffy replication request
  - docs/ui-and-layout/ai-workbench-style/component-prototype/README.md
  - docs/ui-and-layout/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
tests:
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
  - node --check docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node --check docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
doc_type: module-detail
---

# Workbench Ability Workspace

`workbench_ability_workspace.zui` owns the retained/Taffy declaration for the native-covered Gameplay Ability module workspace. The parent `workbench_additional_module_workspaces.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchAbilityWorkspaceHost`, keeping the additional module host focused on overlay composition while preserving the imported root control id.

The split component preserves the original `WorkbenchModuleAbilityWorkspace` root control id and all `WorkbenchModule/Ability*` events. Existing retained navigation still toggles the same workspace control through the module workspace control table; no Rust action or binding namespace changes are required for this split.

The native module verifier reads this file directly. It checks that Ability still follows the shared module body skeleton: fixed rail gap, fixed left/right panels, stretch center panel, interactive tab/list/table/property/field/button leaves, and routes under `WorkbenchModule.Ability.*`.
