---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_vfx_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - docs/ui-and-layout/ai-workbench-style/component-prototype/verify-web-native-handoff.mjs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_vfx_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
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

# Workbench VFX Workspace

`workbench_vfx_workspace.zui` owns the retained/Taffy declaration for the native-covered VFX module workspace. The parent `workbench_module_workspace.zui` imports it and mounts an `Overlay` wrapper with `WorkbenchVfxWorkspaceHost`, matching the split workspace pattern used by Effect, Material, Behavior, and Assets so the parent file stays focused on overlay composition instead of accumulating every core module body while the imported root control id stays intact.

The split component keeps the original `WorkbenchModuleVfxWorkspace` control id on its root node. Existing retained navigation still toggles that control through `MODULE_WORKSPACE_CONTROLS`; no Rust route, binding, or preview action namespace changes are required for this split.

The native module verifier reads `workbench_vfx_workspace.zui` as a first-class workspace source. It checks that VFX still uses the shared module body skeleton: fixed rail gap, fixed left/right side panels, stretch center panel, interactive tab/list/property/field/button leaves, and `WorkbenchModule/Vfx*` events routed under `WorkbenchModule.Vfx.*`.
