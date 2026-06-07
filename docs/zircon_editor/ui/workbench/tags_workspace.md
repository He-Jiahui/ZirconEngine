---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_tags_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_additional_module_workspaces.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_tags_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_additional_module_workspaces.zui
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

# Workbench Tags Workspace

`workbench_tags_workspace.zui` owns the retained/Taffy declaration for the native-covered Gameplay Tags module workspace. The parent `workbench_additional_module_workspaces.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchTagsWorkspaceHost`, so Tags rows, validation output, and editable fields no longer live in the additional host file while the imported root control id stays intact.

The split component preserves the original `WorkbenchModuleTagsWorkspace` root control id and all `WorkbenchModule/Tags*` events. Existing retained navigation, template bindings, preview actions, and command feedback therefore continue to address the same module workspace.

The native module verifier reads this file as a first-class workspace source and checks the shared non-Scene module skeleton plus the Tags-specific binding namespace.
