---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/action_ids.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/profiles.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/options.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/surface_actions.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/action_ids.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/profiles.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/options.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/surface_actions.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app build-export job-queue ownership scan
  - app build-export action/profile ownership scan
  - app build-export execution-summary ownership scan
  - app build-export host-action ownership scan
  - app build-export projection-target ownership scan
  - app build-export wizard host-actions ownership scan
  - app build-export wizard surface-actions ownership scan
  - app build-export wizard options ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Build Export Host Actions

`app/build_export_actions.rs` is the structural Build/Export action family entry. It declares the action id, execution summary, host action, job queue, output folder, and profile modules, then re-exports only the narrow helpers used by sibling app modules.

`app/build_export_actions/host_actions.rs` owns Build/Export callback action routing and synchronous host side effects. It dispatches queue/cancel/output/reveal actions, mutates output-folder overrides, propagates completed execution summaries into host state/status, starts queued jobs, and keeps the lifecycle-facing `RetainedEditorHost` action methods.

`app/build_export_projection.rs` owns pane view-model assembly and wizard view-model attachment. Its child `app/build_export_projection/targets.rs` owns target-row generation from project manifest/export plan data, output-root diagnostics, completed-summary overlays, and queued/running job overlays. `app/build_export_wizard_session.rs` owns wizard-stage retained per-profile session state. `app/build_export_actions/output_folder.rs` owns native output-folder picking and reveal shell integration.

## Job Queue

`app/build_export_actions/job_queue.rs` owns asynchronous desktop export job state. It tracks pending and active jobs, snapshots queue state for pane projection and status tasks, handles pending cancellation and active cancellation requests, polls backend progress/result messages, spawns the export worker thread, converts progress reports to status diagnostics, and converts finished/cancelled results into `DesktopExportExecutionSummary`.

The parent action module re-exports only the queue types and pane/status projection helpers needed by `RetainedEditorHost` and `build_export_projection.rs`.

## Action Ids

`app/build_export_actions/action_ids.rs` owns the stable Build/Export action-id grammar. It maps plan, execute, cancel, output set, output choose, output clear, and output reveal ids into `BuildExportAction`, rejecting empty profile names and empty output roots.

The parent action module re-exports the parser and enum for sibling modules such as `build_export_wizard_session.rs` and `pane_surface_actions.rs`, but the grammar and parser behavior stay in the child module.

## Host Actions

`app/build_export_actions/host_actions.rs` owns retained-host action side effects. It polls desktop export jobs, starts queued jobs, synchronizes status task progress, dispatches parsed `BuildExportAction` values, enqueues profile exports, cancels pending/running exports, chooses output folders, reveals output folders, and resolves effective output roots.

Keeping these effects out of the root action module separates app state mutation from action-id parsing, queue internals, profile declarations, native folder helpers, and completed-summary projection.

## Execution Summary

`app/build_export_actions/execution_summary.rs` owns completed desktop export result DTOs, exported/failed/cancelled result construction, status-line message formatting, pane diagnostic text, fatal-state detection, and target-row application for completed export reports.

The parent action module re-exports only the summary DTO and target-row application helper required by queue polling and Build/Export pane projection. The execution state enum is test-visible only from the parent module so normal library builds do not expose an unused import.

## Profiles

`app/build_export_actions/profiles.rs` owns the desktop/mobile/browser/headless export profile catalog, target platform labels, profile lookup, and project-local default output-root path convention. It keeps profile definitions out of the host action side-effect module while preserving the existing `build_export_actions::*` call surface for projection and wizard code.

## Projection Targets

`app/build_export_projection/targets.rs` owns Build/Export target-row construction for the retained pane. It resolves the active project root and `zircon-project.toml`, generates per-profile native-aware export plans through `EditorManager`, formats fatal/non-fatal diagnostics with the effective output root, and applies completed execution summaries plus queued/running job snapshots to the row data.

The root `app/build_export_projection.rs` stays structural around `RetainedEditorHost::build_export_pane_data(...)`: it gathers diagnostics, delegates target-row construction, attaches the first target's wizard view model, and returns `BuildExportPaneViewData`. This keeps pane assembly distinct from target-row derivation.

## Wizard Surface Actions

`app/build_export_wizard_session/surface_actions.rs` owns desktop export wizard surface action parsing and status text. It maps panel button control ids plus stable `workbench.build_export.*` action ids into `ExportWizardPanelAction`, builds per-profile wizard job ids, validates that plan/start actions receive pipeline options, and formats wizard status-line updates.

`app/build_export_wizard_session.rs` remains the retained app owner for per-profile `ExportWizardPanelSession` state, view-model lookup, plan regeneration, process-runner dispatch, and changed-update polling.

`app/build_export_wizard_session/host_actions.rs` owns the retained-host action entry points for the desktop export wizard. It maps surface action ids to wizard session actions, builds active-project options for plan/start actions, applies wizard updates to layout/status state, and polls all active wizard sessions from host lifecycle.

## Wizard Options

`app/build_export_wizard_session/options.rs` owns host option construction for desktop export wizard plan/start actions. It resolves the active project root and `zircon-project.toml`, applies the effective per-profile output root, looks up desktop export profiles, fills strategy/repo/source-manifest/host-executable/target-platform options, and derives the engine repository root from the editor crate manifest directory.

Keeping wizard option construction outside the session owner separates active-project/profile filesystem policy from per-profile session state and polling.

## Boundary Rules

- Keep output-folder overrides, status-line mutations, export queue dispatch, cancellation dispatch, and `RetainedEditorHost` action methods in `app/build_export_actions/host_actions.rs`.
- Keep Build/Export action-id grammar and parser behavior in `app/build_export_actions/action_ids.rs`.
- Keep completed export summary DTOs, summary status messages, summary diagnostics, fatal-state detection, and completed summary target-row application in `app/build_export_actions/execution_summary.rs`.
- Keep export profile catalog, platform labels, profile lookup, and default output-root convention in `app/build_export_actions/profiles.rs`.
- Keep pending/active job DTOs, worker thread spawning, progress polling, cancellation state, queue snapshots, and job status projection helpers in `app/build_export_actions/job_queue.rs`.
- Keep output-folder picker/reveal platform integration in `app/build_export_actions/output_folder.rs`.
- Keep pane row construction and Build/Export panel diagnostics in `app/build_export_projection.rs`; do not add target row projection back to action routing or job queue code.
- Keep desktop export wizard per-profile session state, view-model lookup, plan regeneration, changed-update polling, and process runner dispatch in `app/build_export_wizard_session.rs`.
- Keep desktop export wizard retained-host action dispatch, active-project option lookup for plan/start, poll status propagation, and layout/status mutation in `app/build_export_wizard_session/host_actions.rs`.
- Keep desktop export wizard surface button mapping, wizard job id construction, required-option validation, and wizard status-line message formatting in `app/build_export_wizard_session/surface_actions.rs`.
- Keep active-project export wizard option construction, default source asset manifest path, default host executable path, and engine repo-root derivation in `app/build_export_wizard_session/options.rs`.

## Validation Notes

The 2026-06-18 job-queue split reduced `build_export_actions.rs` from 896 lines to 498 lines. `build_export_actions/job_queue.rs` is 413 lines and owns the desktop export queue, progress snapshots, cancellation result, status task projection, pane job overlay helper, worker thread execution, and result-to-summary conversion. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export job-queue ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 action/profile split reduced `build_export_actions.rs` to 360 lines. `build_export_actions/action_ids.rs` is 78 lines and owns `BuildExportAction` plus action-id parsing. `build_export_actions/profiles.rs` is 74 lines and owns export profile definitions, profile lookup, platform labels, and default output-root construction. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export action/profile ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed after widening child item visibility to the app boundary. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 execution-summary split reduced `build_export_actions.rs` to 218 lines. `build_export_actions/execution_summary.rs` is 130 lines and owns `DesktopExportExecutionSummary`, completed export state construction, status messages, pane diagnostics, fatal-state detection, and completed-summary target row projection. `build_export_actions/job_queue.rs` remains responsible for queue/progress/cancellation snapshots and imports only the summary DTO constructors. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export execution-summary ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 wizard surface-actions split reduced `build_export_wizard_session.rs` from 307 lines to 242 lines. `build_export_wizard_session/surface_actions.rs` is 79 lines and owns wizard surface action mapping, wizard job id construction, required-option validation, and wizard status messages. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export wizard surface-actions ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 wizard options split reduced `build_export_wizard_session.rs` from 261 lines to 211 lines. `build_export_wizard_session/options.rs` is 59 lines and owns active-project wizard option construction, source asset manifest defaulting, host executable path construction, export profile lookup, and engine repository root derivation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export wizard options ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 wizard host-actions split reduced `build_export_wizard_session.rs` from 194 lines to 130 lines. `build_export_wizard_session/host_actions.rs` is 65 lines and owns retained-host wizard action dispatch, active-project options lookup for plan/start actions, update application, poll status propagation, and layout/status mutation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export wizard host-actions ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 host-action split reduced `build_export_actions.rs` from 218 lines to 21 lines. `build_export_actions/host_actions.rs` is 209 lines and owns retained-host desktop export job polling/start-next/status-task sync, parsed Build/Export action dispatch, enqueue/cancel/output override mutation, folder picker/reveal side effects, and effective output-root resolution. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export host-action ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
