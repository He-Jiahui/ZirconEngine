---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_effect_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_module_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_effect_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_module_workspace.zui
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
plan_sources:
  - user: 2026-06-03 componentized editor UI prototype and native retained/Taffy replication request
  - docs/ui-and-layout/ai-workbench-style/component-prototype/README.md
  - docs/ui-and-layout/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
tests:
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
doc_type: module-detail
---

# Workbench Effect Workspace

`workbench_effect_workspace.zui` owns the retained/Taffy declaration for the native-covered Gameplay Effect workspace. The parent `workbench_module_workspace.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchEffectWorkspaceHost`, keeping the parent overlay focused on composition while preserving the imported root control id.

The split component preserves the original `WorkbenchModuleEffectWorkspace` root control id and every `WorkbenchModule/Effect*` event. Existing retained navigation, template bindings, preview actions, and command feedback therefore continue to address the same module workspace.
