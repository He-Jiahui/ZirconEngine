---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_behavior_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_behavior_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
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

# Workbench Behavior Workspace

`workbench_behavior_workspace.zui` owns the retained/Taffy declaration for the native-covered Behavior Tree workspace. The parent `workbench_module_workspace.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchBehaviorWorkspaceHost`, leaving behavior tree graph, blackboard, debugger, and detail fields in a focused component file while the imported root control id stays intact.

The split component preserves `WorkbenchModuleBehaviorWorkspace` as the root control id and keeps all `WorkbenchModule/Behavior*` events unchanged. Existing retained navigation and template bindings can therefore continue to toggle and route the workspace without Rust changes.
