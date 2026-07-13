---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-extension-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-extension-module-contract.mjs
plan_sources:
  - user: 2026-06-03 componentized editor UI prototype and native retained/Taffy replication request
  - docs/ui-and-layout/ai-workbench-style/component-prototype/README.md
  - docs/ui-and-layout/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
tests:
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-extension-module-contract.mjs
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
  - node --check docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node --check docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-extension-module-contract.mjs
  - node --check docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
doc_type: module-detail
---

# Workbench Assets Workspace

`workbench_assets_workspace.zui` owns the retained/Taffy declaration for the native-covered Asset Browser module workspace. The parent `workbench_module_workspace.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchAssetsWorkspaceHost`, matching the split component pattern used by Effect, Material, Behavior, and VFX so the parent overlay stays focused on workspace composition while the imported root control id stays intact.

The split component preserves the original `WorkbenchModuleAssetsWorkspace` root control id and all `WorkbenchModule/Assets*` events. Existing retained navigation therefore continues to toggle the same root workspace control through `MODULE_WORKSPACE_CONTROLS`.

Assets also owns the native opener buttons for recorded More Editors evidence workspaces, including Data Table, Source Control, Build Export, Automation Report, Project Overview, and Save Data. Those buttons keep their `WorkbenchExtension/*Open` events inside the Assets component, while `verify-native-extension-module-contract.mjs` reads this split file directly to prove the opener bindings remain present.
