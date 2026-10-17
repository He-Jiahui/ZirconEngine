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
  - node docs/ui/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
  - node --check docs/ui/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node --check docs/ui/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
doc_type: module-detail
---

# Workbench Perception Workspace

`workbench_perception_workspace.zui` owns the retained/Taffy declaration for the native-covered AI Perception module workspace. The parent `workbench_additional_module_workspaces.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchPerceptionWorkspaceHost`, keeping the additional host as structure only while preserving the imported root control id.

The split component preserves the original `WorkbenchModulePerceptionWorkspace` root control id and all `WorkbenchModule/Perception*` events. Existing retained navigation and preview actions still refer to the same control and action namespace.

The verifier checks this workspace directly for the shared left/center/right module skeleton, interactive component coverage, Perception route namespace, and native binding coverage.
