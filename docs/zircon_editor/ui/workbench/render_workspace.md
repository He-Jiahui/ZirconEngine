---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-extension-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui
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

# Workbench Render Workspace

`workbench_render_workspace.zui` owns the retained/Taffy declaration for the native-covered Render Pipeline module workspace. The parent `workbench_additional_module_workspaces.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchRenderWorkspaceHost`, so the additional host no longer carries render graph, resource, capture, or settings nodes while the imported root control id stays intact.

The split component preserves the original `WorkbenchModuleRenderWorkspace` root control id and all `WorkbenchModule/Render*` events. Render Pipeline remains preview-only for real render data binding, but its retained workspace skeleton and preview actions are source-level covered.

Render also owns the Shader Editor extension opener. `WorkbenchRenderShaderEditorButton` and `WorkbenchExtension/ShaderEditorOpen` stay in this split component, and `verify-native-extension-module-contract.mjs` reads `workbench_render_workspace.zui` directly to prove that opener remains present.
