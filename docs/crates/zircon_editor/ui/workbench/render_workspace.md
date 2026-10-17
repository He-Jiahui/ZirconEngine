---
related_code:
  - zircon_editor/src/ui/retained/dispatch/templates/workbench/module_navigation.rs
implementation_files:
plan_sources:
  - user: 2026-06-03 componentized editor UI prototype and native retained/Taffy replication request
  - docs/ui/ai-workbench-style/component-prototype/README.md
  - docs/ui/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
tests:
  - node docs/ui/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node docs/ui/ai-workbench-style/component-prototype/verify-native-extension-module-contract.mjs
  - node docs/ui/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
  - node --check docs/ui/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node --check docs/ui/ai-workbench-style/component-prototype/verify-native-extension-module-contract.mjs
  - node --check docs/ui/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
doc_type: module-detail
---

# Workbench Render Workspace

`workbench_render_workspace.zui` owns the retained/Taffy declaration for the native-covered Render Pipeline module workspace. The parent `workbench_additional_module_workspaces.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchRenderWorkspaceHost`, so the additional host no longer carries render graph, resource, capture, or settings nodes while the imported root control id stays intact.

The split component preserves the original `WorkbenchModuleRenderWorkspace` root control id and all `WorkbenchModule/Render*` events. Render Pipeline remains preview-only for real render data binding, but its retained workspace skeleton and preview actions are source-level covered.

Render also owns the Shader Editor extension opener. `WorkbenchRenderShaderEditorButton` and `WorkbenchExtension/ShaderEditorOpen` stay in this split component, and `verify-native-extension-module-contract.mjs` reads `workbench_render_workspace.zui` directly to prove that opener remains present.
