---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_material_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_module_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_material_workspace.zui
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

# Workbench Material Workspace

`workbench_material_workspace.zui` owns the retained/Taffy declaration for the native-covered Material workspace. The parent `workbench_module_workspace.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchMaterialWorkspaceHost`, so material-specific graph, compile, dropdown, and preview controls no longer live in the overlay host while the imported root control id stays intact.

The split component keeps `WorkbenchModuleMaterialWorkspace` as its root control id and retains the `WorkbenchModule/Material*` event namespace. The native module verifier reads this file directly to prove the dropdown, table, list, field, and command coverage remains intact after the split.
