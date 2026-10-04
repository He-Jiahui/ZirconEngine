---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/127-editor-workbench-shell-autolayout-constraint-language-responsive-region-binding-geometry-current-source-review.md
  - docs/plans/optimize/zircon_editor/09/2026-08-26-export-wizard-session-hash-index.md
  - docs/plans/optimize/zircon_editor/160/2026-08-26-feature-dependency-message-direct-render.md
  - docs/plans/optimize/zircon_editor/122-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/101-editor-project-operations-source-control-changelist-diff-automation-validation-submission-health-current-source-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
  - docs/plans/optimize/zircon_editor/145-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/96-editor-render-pipeline-render-graph-frame-debugger-capture-lighting-bake-reflection-probe-post-process-debug-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/entry/tests.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state/hash_index_tests.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/dependencies/capacity_tests.rs
  - zircon_editor/src/tests/editor_event/runtime/stack_play.rs
  - zircon_editor/src/ui/host/project_recovery_decision/tests.rs
  - zircon_editor/src/tests/host/retained_activity_rail_pointer/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/asset_browser.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/interaction_policy.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs
  - zircon_editor/src/core/editing/animation_document/tests.rs
  - zircon_editor/src/tests/editor_event/animation_runtime/graph.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/support.rs
---

# Editor921 Window, Animation, Event, and GPU Test Contract Repairs

## 计划完成列表

| Scope | Completed source change | Evidence | Acceptance |
| --- | --- | --- | --- |
| Console pane pointer | Hold the constructed pane across route borrowing rather than borrowing a temporary. | One v27 temporary-lifetime error; hit, local-frame, and scroll assertions unchanged. | implemented_pending_validation |
| Export session index | Supply the fourth, typed window ID when generating each isolated profile plan. | One v27 missing argument; current session method and neighboring window fixture inspected. | implemented_pending_validation |
| Plugin dependency message | Distinguish the local report from the helper function so the second fixture still calls `report(0)`. | One v27 local shadowing error; expected message/capacity assertions unchanged. | implemented_pending_validation |
| Event journal and recovery | Keep the journal owner alive during its record borrow; parse the dereferenced document string in the recovery candidate iterator. | Two v27 temporary/reference diagnostics. | implemented_pending_validation |
| Activity rail and asset drag | Compare the workbench's typed drawer slot to its own slot enum, preserving the distinct event-domain slot; use the current five-argument click callback while retaining the separate nine-argument pointer-event route. | Two v27 signature/type diagnostics; both callback schemas and slot owners inspected. | implemented_pending_validation |
| ZUI command governance | Parse each alias target into `EditorOperationPath` before the typed registry lookup; preserve the assertion that every removed alias still resolves. | One v27 `Borrow<&str>` trait-bound error; registry key type verified. | implemented_pending_validation |
| GPU stats counter | Construct the external non-exhaustive stats type using `Default` and assign its public ledger fields, leaving six recorded metric assertions intact. | One v27 forbidden external struct initializer. | implemented_pending_validation |
| Animation graph fixtures | Clone the fixture's asset through its accessor, add the dangling edge before building a new document with a synchronized compilation snapshot; bind the typed node-command error to the actual failed dispatch rather than the successful open event. | Two v27 privacy/error-type diagnostics; constructor and test expectation inspected. | implemented_pending_validation |
| Workbench slot padding | Read the surface's layout slots through `UiTree::layout_slots()` while retaining the child-ID selection and padding assertion. | One v27 access to a now-private slots field; current runtime-interface accessor inspected. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/entry/tests.rs` | `E5A4BBCDF017950E307DEE9697094973FF978634D28EC89E2623855DCA4D4D79` |
| `zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state/hash_index_tests.rs` | `C3E8E0668057FB6FEB6B1CA55215A02C5B6759F7D7FA9E741F4827E7547A4306` |
| `zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/dependencies/capacity_tests.rs` | `F42A6520839BD7974A6F2785789041F00009B5B56FF61492C3D1A4DB8CAA9033` |
| `zircon_editor/src/tests/editor_event/runtime/stack_play.rs` | `BFDDB236658FEF0FD1FEF35FC8911D91967C35B34B161D6BC98013EA4A62D6F1` |
| `zircon_editor/src/ui/host/project_recovery_decision/tests.rs` | `8269F21D4C8C0FD73464FE5DFDD06F05CEF93E51A39D9BF0521C0C0D36F46B8D` |
| `zircon_editor/src/tests/host/retained_activity_rail_pointer/dispatch.rs` | `08C536C5BD858BF3A9ABFAB29E0466D08253848DD618B25E2CC2AAB44D4AAC6A` |
| `zircon_editor/src/ui/retained_host/app/tests/drag_sources/asset_browser.rs` | `A3D2B5CD54C2A458AF72D28D75E77234CBCC1A31BE893BB1DFBE64D1B226C18A` |
| `zircon_editor/src/tests/ui/boundary/zui_asset_governance/interaction_policy.rs` | `3F1E9CA9A62BC545EDF1E6968C5C964ADC56C82064CAD4056F498CB82BDD64E9` |
| `zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs` | `6601A3C81A4597802625943ABAB62E0F2B76D5E3A3D66F14662081D2E3DB4A7B` |
| `zircon_editor/src/core/editing/animation_document/tests.rs` | `8C64819792612BAE4484A107F903B6BFAE52D581401D4871BBA58767C82E215D` |
| `zircon_editor/src/tests/editor_event/animation_runtime/graph.rs` | `CECAE665096F1D5C94875B12DA14CA72782296FFB51E1D9DC591D05743F4BEA9` |
| `zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/support.rs` | `238D67DED0BCD9F9C7DDD732EFE3077F2260681D8A64CF0C9DA38737FCCAB125` |

## Managed gate

These twelve clean test owners postdate v28's metadata rejection. The single
later coordinator admission check found an unrelated blocking Cargo lease;
there was no second status check, per-fixture Cargo run, or synchronous
wait. Grouped current-source Rust tests and ignored Release, allocator,
and product percentile qualification remain required.
