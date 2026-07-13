---
related_code:
  - docs/ui-and-layout/ai-workbench-style/component-prototype/src/routing/routes.js
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/error.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/toolbar_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_toolbar_breakpoints.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/error.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/toolbar_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
plan_sources:
  - user: 2026-06-03 Continue the componentized editor workbench; keep layout component-based and make every button visibly responsive
  - docs/ui-and-layout/componentized-workbench-shell.md
  - docs/ui-and-layout/ai-workbench-style/component-prototype/README.md
tests:
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/toolbar_layout.rs zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_toolbar_breakpoints.rs
  - cargo test -p zircon_editor --lib compact_workbench_toolbar_uses_slate_command_density --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0628-thumb-grid-summary --message-format short --color never -- --test-threads=1 --nocapture (2026-06-29: passed)
  - cargo test -p zircon_editor --lib full_workbench_secondary_module_commands_keep_readable_width --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-atomic-20260710-0702 --message-format short --color never -- --test-threads=1 --nocapture (2026-07-10: 1 passed)
  - cargo build -q -p zircon_app --bin zircon_editor --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0628-thumb-grid-summary (2026-06-29: passed)
  - direct zircon_editor test binary capture_m3_gui_acceptance_visual_artifacts --ignored --exact --test-threads=1 --nocapture (2026-06-29: passed, refreshed docs/tests/editor/editor-window-m3-workbench-900x620.png)
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
  - node --check docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-module-contract.mjs
  - cargo test -p zircon_editor --lib workbench_shared_module_commands_route_feedback_to_active_module_output --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-preview-0603 --message-format short --color never -- --nocapture --test-threads=1
  - direct zircon_editor test binary declared_workbench_module_events_dispatch_preview_actions --test-threads=1 --nocapture (2026-06-24: passed)
  - cargo test -p zircon_editor --lib workbench_toolbar --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 7 passed / 1 ignored)
doc_type: module-detail
---

# Module Command Feedback

## Purpose

`module_command_feedback.rs` owns preview-only feedback for Workbench module commands in the retained native editor surface. The module does not execute real gameplay, asset, renderer, or UI-authoring behavior. It updates existing status and output-row component properties so every authored Workbench command gives visible feedback through the same retained projection path used by other componentized controls.

The browser prototype has the same semantic boundary in `routes.js`: shared toolbar labels are component commands first, not hard-coded engine actions. Native feedback mirrors that idea by interpreting a shared command such as Compile, Diff, Save, or Simulate against the currently selected Workbench module.

## Shared Commands

The top module toolbar exposes one set of shared command controls in `workbench_top_toolbar.zui`: Save, Browse, Compile, Diff, and Simulate. `componentized_window.rs` still handles selection state and special Browse navigation. Once the action is recognized as a Workbench module action, it delegates to `apply_workbench_module_command_feedback(...)`.

The same toolbar asset is also the density authority for the compact Workbench command strip. Its command buttons are 30 px high, and `toolbar_layout.rs` mutates the module command group width between compact and full modes so hidden Diff/Simulate controls do not leave visible slack in the 900 px tier. The current readable-label slice reserves 72/92/104 px for Save/Browse/Compile and 54/50 px for full-width Diff/Sim, with compact/full command groups at 276/388 px; compact mode still hides Diff/Simulate before it compresses primary command text.

The feedback module resolves the active module by reading the selected or checked state of the module tabs. That active module chooses the output row family:

- Effect writes to `WorkbenchEffectOutputRow`.
- Ability writes to `WorkbenchAbilityOutputRow`.
- Tags writes to `WorkbenchTagsValidationRow`.
- Perception writes to `WorkbenchPerceptionEventRow`.
- Material writes to `WorkbenchMaterialOutputRow`.
- Behavior writes to `WorkbenchBehaviorOutputRow`.
- Render writes to `WorkbenchRenderCaptureRow`.
- Assets writes to `WorkbenchAssetsOutputRow`.
- VFX writes to `WorkbenchVfxOutputRow`.
- HUD writes to `WorkbenchHudValidationRow`.

Scene mode has no module output surface, so shared command feedback only updates shell status and message count there.

## Panel Commands

Module-local panel buttons such as Material Compile, Behavior Validate, Assets Import, VFX Simulate, Ability Playtest, Render Compile, and HUD Preview keep their existing explicit feedback entries. They are intentionally separate from shared toolbar commands because their labels and output rows are authored inside specific module panels.

## Validation

`workbench_module_commands_update_status_and_module_output_rows` covers the existing explicit command feedback path and Browse's module switch to Assets.

`workbench_shared_module_commands_route_feedback_to_active_module_output` covers the module-scoped shared toolbar semantics. It selects Material, Behavior, Assets, and VFX, dispatches the same shared Compile command, and asserts that feedback lands on each module's own output row. The same test dispatches Diff and Simulate while VFX is active so the active-module route is not limited to Compile.

`verify-native-module-contract.mjs` also reads this Rust module directly. Its static contract requires the ten module tabs to resolve to the matching native output rows, keeps Save/Compile/Diff/Simulate routed through `active_module`, and links browser module-scoped commands such as `material:compile` and `vfx:compile` to the same native shared command action.

The 2026-06-24 architecture closeout also covers `declared_workbench_module_events_dispatch_preview_actions`. That regression dispatches declared module controls through the componentized Workbench bridge and verifies preview actions reach the retained feedback path. The bridge now reports boolean layout mutation failures as `LayoutMutation { node_id, property, source }`, so invalid node/property failures preserve enough context for diagnosis instead of collapsing into a generic dispatch error.

The 2026-06-26 density pass adds `compact_workbench_toolbar_uses_slate_command_density`, which locks the 44 px toolbar band, 34 px module tabs, 30 px command buttons, 4 px command gaps, and compact command-group width. The follow-up no longer treats toolbar popup placement as a static 44 px asset coordinate: `window_menu_state.rs` computes the open menu frame from the trigger control frame, the toolbar frame bottom, the menu node constraints, and the root frame clamp. It writes both the retained node position and `popup_anchor_*` metadata so render state and native popup state agree. `workbench_toolbar_window_menus_anchor_to_toolbar_controls_across_widths` covers Main, More, Run Mode, and Layout menus across 900/1260/1672 widths. `capture_workbench_module_overflow_visual_artifact --ignored` refreshes `docs/tests/editor/editor-window-m3-workbench-module-overflow-900x620.png` without writing screenshots under Cargo `target`.

The 2026-06-29 readable-label follow-up updates the same density regression to lock 276 px compact command group width and 72/92/104 px Save/Browse/Compile command widths. The M3 Workbench screenshot now shows all three primary command labels without ellipsis, while secondary commands remain breakpoint-controlled.
