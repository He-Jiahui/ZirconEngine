---
related_code:
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/drag_resize.rs
  - zircon_editor/src/ui/retained_host/app/tests/child_window_tabs.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60/2026-09-28-native-child-tab-drop-source-routing.md
status: implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
validation_manifest: .codex/state/session-coordinator/async-validation-batches/2026-09-28-editor60-child-drop-static-checks.json
source_manifest: .codex/state/session-coordinator/async-validation-batches/2026-09-28-editor60-child-drop-successor-source-manifest.json
---

# Editor1028 / Editor60 native child tab drop source routing completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| ED60 child drag-state owner | Root callbacks retain the root context; child callbacks use the configured presenter ID to read and update that child's drag state. Missing, hidden, and stale child presenters do not fall back to root state. | Shared source-first callback signature is `Option<&MainPageId>`; real child native-input tests are added. Managed execution is pending. | implemented_pending_validation |
| ED60 target route coordinates | Child-local x/y is translated by the child outer frame from the same floating projection bundle used by the root drag route surface. The translated point is used for both target sync and release resolution. | Projection-origin mapping test and child-to-root/child-to-child route assertions are added. Mixed-DPI and OS window coordinate behavior remain unverified. | implemented_pending_validation |
| ED60 root and child drop outcomes | A child tab dropped on the root document target attaches to the root workspace; a drop on another child attaches to that floating window. | Installed child presenter press/move/release callbacks assert the resulting view host and source/target window lifetime. Managed Editor tests are pending. | implemented_pending_validation |
| ED60 empty-space behavior | A child tab dropped outside all Workbench drag targets follows existing `DetachToWindow` behavior and creates a new native floating presenter. | Installed child callbacks assert the generated floating view host and presenter. Actual OS cross-window Up delivery remains pending. | implemented_pending_validation |
| ED60 validation and product gates | Direct source/manifest assertions and final-newline/trailing-whitespace checks passed. No Cargo command was run for this slice. | Exact checks are recorded in `2026-09-28-editor60-child-drop-static-checks.json`; source digests are recorded in `2026-09-28-editor60-child-drop-successor-source-manifest.json`. OS/winit capture, cross-window release, mixed-DPI behavior, visual acceptance, and measured latency/allocation gates are not verified. | product_gate_pending |

The `source_window_id` is a call-scoped event origin shared with the native resize callback boundary. Programmatic hide must not synchronously invoke this app callback while its `RefCell` host borrow is active; child capture retirement remains a direct local-state cleanup.
