---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_hud_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_additional_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\extensions\workbench_extension_performance_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\extensions\workbench_extension_telemetry_dashboard_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-extension-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_hud_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_additional_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\extensions\workbench_extension_performance_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\extensions\workbench_extension_telemetry_dashboard_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics.rs
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

# Workbench HUD Workspace

`workbench_hud_workspace.zui` owns the retained/Taffy declaration for the native-covered HUD Editor module workspace. The parent `workbench_additional_module_workspaces.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchHudWorkspaceHost`, leaving HUD widget hierarchy, preview, validation, and details controls in a focused module file while the imported root control id stays intact.

The split component preserves the original `WorkbenchModuleHudWorkspace` root control id and all `WorkbenchModule/Hud*` events. Existing retained navigation, template bindings, and preview action dispatch still use the same workspace control and route namespace.

The native module verifier reads this file directly and checks the standard non-Scene module layout skeleton plus HUD-specific interactive coverage. The native-extension verifier also checks HUD-hosted extension openers; Performance and Telemetry Dashboard are intentionally only prototype-only More Editors evidence workspaces opened from `WorkbenchHudPerformanceButton` and `WorkbenchHudTelemetryDashboardButton`.
