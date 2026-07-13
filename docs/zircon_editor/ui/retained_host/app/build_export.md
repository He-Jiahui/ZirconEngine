---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/action_ids.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/error.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary/constructors.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary/status.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary/target.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/jobs.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/jobs/cancellation.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/jobs/enqueue.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/jobs/polling.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/output.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/cancellation.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/enqueue.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/queries.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/snapshot.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/snapshot/progress.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/snapshot/status.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/snapshot/target.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/start.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/state.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/updates.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/worker.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/picker.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/picker/commands.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/picker/selection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/reveal.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/tests.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/profiles.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/diagnostics.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/project.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/rows.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/rows/constructors.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/rows/overlays.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/options.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state/actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state/lookup.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state/polling.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/surface_actions.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/completion.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/poll.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/run.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/cargo_build.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/error.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
  - zircon_editor/src/ui/host/export_process_support/error.rs
  - zircon_editor/src/ui/host/export_process_support/process_tree.rs
  - zircon_editor/src/ui/host/export_process_support/child_guard.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/cargo_build.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/cleanup.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/error.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/native_dynamic_preparation.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/prepare.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/support.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_plan.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/panel_session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/view_model.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/action_ids.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/error.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary/constructors.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary/status.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/execution_summary/target.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/jobs.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/jobs/cancellation.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/jobs/enqueue.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/jobs/polling.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/output.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/cancellation.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/enqueue.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/queries.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/snapshot.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/snapshot/progress.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/snapshot/status.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/snapshot/target.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/start.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/state.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/updates.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/worker.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/picker.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/picker/commands.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/picker/selection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/reveal.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/output_folder/tests.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/profiles.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/diagnostics.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/project.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/rows.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/rows/constructors.rs
  - zircon_editor/src/ui/retained_host/app/build_export_projection/targets/rows/overlays.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/options.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state/actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state/lookup.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state/polling.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/surface_actions.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/completion.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/poll.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/run.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/cargo_build.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/error.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
  - zircon_editor/src/ui/host/export_process_support/error.rs
  - zircon_editor/src/ui/host/export_process_support/process_tree.rs
  - zircon_editor/src/ui/host/export_process_support/child_guard.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/cargo_build.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/cleanup.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/error.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/native_dynamic_preparation.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/prepare.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/support.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_plan.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/panel_session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/view_model.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_plugins/13-standalone-plugin-build.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
  - user: 2026-07-11 Plan 14 M2.1 export wizard job-system hard cutover
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - user: 2026-07-11 Plan 14 M2.2 export worker and pipe-reader hard cutover
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app build-export job-queue ownership scan
  - app build-export job-queue snapshot ownership scan
  - app build-export job-queue snapshot subowner ownership scan
  - app build-export job-queue worker ownership scan
  - app build-export job-queue subowner ownership scan
  - app build-export job-queue state/enqueue/query ownership scan
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/tests.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
  - app build-export action/profile ownership scan
  - app build-export execution-summary ownership scan
  - app build-export execution-summary subowner ownership scan
  - app build-export host-action ownership scan
  - app build-export host-action job/output ownership scan
  - app build-export host-action job subowner ownership scan
  - app build-export output-folder picker/reveal ownership scan
  - app build-export output-folder picker command/selection ownership scan
  - app retained-host owner visibility compile-boundary scan
  - app build-export projection-target ownership scan
  - app build-export projection-target subowner ownership scan
  - app build-export target rows subowner ownership scan
  - app build-export wizard host-actions ownership scan
  - app build-export wizard surface-actions ownership scan
  - app build-export wizard options ownership scan
  - app build-export wizard session-state ownership scan
  - app build-export wizard session-state subowner ownership scan
  - editor UI 10 export wizard test owner split scan
  - cargo test -p zircon_editor --lib export_wizard --locked --target-dir E:\cargo-targets\zircon-editor-export-wizard-0622 --message-format short --color never -- --test-threads=1 (previously blocked before editor tests by runtime GpuMeshResource::indirect_order_signature field visibility; latest rerun timed out after 304s without diagnostics and leftovers stopped)
  - rustfmt --edition 2021 zircon_editor\src\ui\retained_host\app\build_export_wizard_session\session_state.rs zircon_editor\src\ui\retained_host\app\build_export_wizard_session\session_state\actions.rs zircon_editor\src\ui\retained_host\app\build_export_wizard_session\session_state\polling.rs
  - cargo test -p zircon_editor --lib desktop_export_wizard_sessions_project_view_model_after_generate_plan --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-descriptor-rerun-0624 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib export_wizard_ --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-descriptor-rerun-0624 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Build Export Host Actions

`app/build_export_actions.rs` is the structural Build/Export action family entry. It declares the action id, execution summary, host action, job queue, output folder, and profile modules, then re-exports only the narrow helpers used by sibling app modules.

`app/build_export_actions/host_actions.rs` owns Build/Export callback action routing. It parses stable action ids, applies simple output override mutations, and delegates queue/cancel/output-picker/output-reveal side effects to child owners.

`app/build_export_actions/host_actions/jobs.rs` is the structural desktop export job host-action entry. Its children own retained-host job polling/start-next, enqueue preparation, manifest/profile snapshotting, cancellation dispatch, and status task sync.

`app/build_export_actions/host_actions/output.rs` owns output-folder choosing, output-folder reveal, and effective output-root resolution.

`app/build_export_projection.rs` owns pane view-model assembly and wizard view-model attachment. Its child `app/build_export_projection/targets.rs` owns target-row generation from project manifest/export plan data, output-root diagnostics, completed-summary overlays, and queued/running job overlays. `app/build_export_wizard_session.rs` owns wizard-stage retained per-profile session state. `app/build_export_actions/output_folder.rs` is the output-folder platform integration entry.

## Job Queue

`app/build_export_actions/job_queue.rs` is the structural asynchronous desktop export job-queue entry. It re-exports only the app-facing queue and snapshot helpers, while child owners hold state, enqueue behavior, query/projection assembly, cancellation, update polling, startup, worker execution, and snapshot formatting.

`app/build_export_actions/job_queue/state.rs` owns the explicitly injected `EditorJobSystem`, pending/active job structs, progress-only channel, and queued immediate summaries. Pending entries hold a `CancellationToken`; the active entry holds `JobTicket<DesktopExportJobResult>`, the same token, and current UI progress. There is no production `Default`: retained startup constructs the queue from `EditorManager.context().jobs().clone()` and reuses that service for the wizard sessions.

`job_queue/enqueue.rs` owns queued job creation and initial snapshots. `job_queue/queries.rs` owns busy-profile checks and active/pending projections. `job_queue/cancellation.rs` removes and token-cancels pending entries; active cancellation calls both `token.cancel()` and `jobs.cancel(ticket.id())`. `job_queue/start.rs` submits `DesktopExportEditorJob` as `JobCategory::Export`. `job_queue/worker.rs` owns the `EditorJob` adapter, export progress DTO, typed success DTO, and success result-to-summary conversion; it does not create a thread and does not send a finished message. Cancellation and business failure return `JobError::Cancelled`/`JobError::Failed`, keeping generic Job bus terminal events aligned with the retained projection. `job_queue/updates.rs` drains progress first, then calls `ticket.try_take()` and is the sole terminal projection owner for success, failure, panic/channel errors, queued cancellation and active cancellation. `snapshot.rs` and its children continue to own progress conversion, pane/status formatting, target overlay, and status-task projection.

## Structured process concurrency and cancellation

Both export execution paths reuse the same jobs service. `ProcessCommandRunner::new(jobs)` is injected by the wizard panel session, while retained queue workers reach the service through the same `EditorManager.context()`. Each child redirects stdout/stderr to unique temporary files. Every monitor iteration uses nested `EditorJobSystem::join` for two regular-file reads capped at 64 KiB each plus one non-blocking child poll; it then sleeps only after all branches return. This shape stays live with a one-thread Runtime pool, does not submit nested Export tickets, and cannot be backpressured by OS pipe buffers. Wizard incrementally decodes complete lines for streaming UI events and flushes a final partial line after exit; Cargo preserves raw bytes until it constructs the invocation. Final drain loops until both files return EOF. The child guard kills/waits the process tree on unwind; normal completion explicitly disarms it after final drain, then the child handle closes before the output-capture guard removes both temporary files.

The full retained/native/Cargo chain uses `CancellationToken`. Native package loops check between packages and after Cargo, export materialization checks at cleanup boundaries, and process pollers terminate active child trees when the token changes. Native staging is protected by an ownership guard: an early typed failure removes staging/build roots, and a cleanup failure is attached to the primary `NativeDynamicPreparationError`. Wizard and Cargo share the process-tree helper, including Unix process groups, Windows tree termination, and direct-child fallback; failed termination attempts retain command-spawn or fallback-kill `io::Error` sources in addition to display diagnostics. The worker decides `Completed`/`Cancelled`/`Failed` exactly once when it returns. Only a source-free `EditorExportBuildError::Cancelled` becomes `JobError::Cancelled`; cleanup, IO, or termination failures remain typed `JobError::Failed` even when the cancellation token is set. The old `AtomicBool` references, detached worker, pipe-reader threads, and finished-message terminal owner were removed as a hard cutover.

## Typed export error contract

The export pipeline has one typed error chain from Runtime plan selection to the retained job ticket. `zircon_runtime::plugin::ExportBuildPlanError` owns plan-construction failures such as a missing profile. `EditorExportBuildError` owns the editor boundary and preserves plan, materialization IO, Cargo, general child-process, native preparation, source-free cancellation, cancellation-with-cleanup-failure, and wizard business-outcome failures as distinct variants. `NativeDynamicPreparationError` preserves structured paths, staging/build/cleanup IO sources, nested process errors, and preparation-plus-cleanup dual failures. `ExportProcessError` preserves structured stream/path context, spawn/read/poll/wait/cleanup context, typed termination attempts, and the original `std::io::Error`; cleanup failure wraps the primary process failure instead of replacing it.

`manager.rs`, generated Cargo invocation, native preparation, output-folder host actions, and the production wizard command runner return typed errors. A failed wizard stage stores its original error as `JobFailure` beside the cloned display snapshot; the controller returns that same failure to the ticket instead of rebuilding an error from diagnostics. The retained export worker likewise passes the concrete `EditorExportBuildError` directly to `JobError::failed`, so callers can downcast from a completed ticket back to the export-domain error and continue traversing `Error::source()`. `DesktopExportActionError` keeps project-root, manifest, picker/reveal spawn, exit status, unsupported-host, and filesystem failures typed until the status-line projection. `to_string()` is confined to diagnostics/status projection after the typed result has been retained. The former cross-module `Result<_, String>`, `From<String>`, and string-compatible overload paths are not retained.

## Action Ids

`app/build_export_actions/action_ids.rs` owns the stable Build/Export action-id grammar. It maps plan, execute, cancel, output set, output choose, output clear, and output reveal ids into `BuildExportAction`, rejecting empty profile names and empty output roots.

The parent action module re-exports the parser and enum for sibling modules such as `build_export_wizard_session.rs` and `pane_surface_actions.rs`, but the grammar and parser behavior stay in the child module.

## Host Actions

`app/build_export_actions/host_actions.rs` owns retained-host action dispatch. It parses `BuildExportAction` values, applies plan refresh and direct output override mutations, and delegates longer side effects to job/output child owners.

`app/build_export_actions/host_actions/jobs.rs` is the structural desktop export queue side-effect entry. `host_actions/jobs/polling.rs` owns poll/update handling, completed-summary publication, start-next dispatch, queue-to-status-task synchronization, and dirty layout marking. `host_actions/jobs/enqueue.rs` owns busy-profile rejection, active-project/profile/manifest/output-root snapshotting, queued-job creation, and immediate poll kick. `host_actions/jobs/cancellation.rs` owns pending/active cancellation result handling and status text.

`app/build_export_actions/host_actions/output.rs` owns native output-folder side effects and effective output-root lookup.

Keeping these effects out of the root action module separates app state mutation from action-id parsing, queue internals, profile declarations, native folder helpers, and completed-summary projection.

## Execution Summary

`app/build_export_actions/execution_summary.rs` is the structural completed desktop export result entry. It owns the exported/failed/cancelled summary DTOs and declares the child owners that build summaries, format status text, and write completed export reports back to target rows.

`execution_summary/constructors.rs` owns exported/failed/cancelled summary construction from `EditorExportBuildReport`, worker failures, and cancellation reasons. `execution_summary/status.rs` owns fatal-state detection, status-line message formatting, target status labels, and pane diagnostic text. `execution_summary/target.rs` owns completed-summary application to `BuildExportTargetViewData`.

The parent action module re-exports only the summary DTO and target-row application helper required by queue polling and Build/Export pane projection. The execution state enum is test-visible only from the parent module so normal library builds do not expose an unused import.

## Profiles

`app/build_export_actions/profiles.rs` owns the desktop/mobile/browser/headless export profile catalog, target platform labels, profile lookup, and project-local default output-root path convention. It keeps profile definitions out of the host action side-effect module while preserving the existing `build_export_actions::*` call surface for projection and wizard code.

## Output Folder

`app/build_export_actions/output_folder.rs` is the output-folder platform integration entry. `output_folder/picker.rs` owns native folder-picker command execution and process-result handling. `output_folder/picker/commands.rs` owns platform-specific picker command construction. `output_folder/picker/selection.rs` owns selected-folder parsing and stable initial directory resolution. `output_folder/reveal.rs` owns host file-browser reveal command construction and process spawning. `output_folder/tests.rs` keeps the platform command and parsing regressions next to the platform integration owner.

Keeping picker and reveal side effects out of host action routing ensures output overrides stay app-state focused while OS command construction remains isolated and testable.

## Projection Targets

`app/build_export_projection/targets.rs` is the structural Build/Export target projection entry. It resolves the active project manifest once, collects current job snapshots, iterates the Build/Export profiles, and delegates per-profile row construction to child owners.

`targets/project.rs` owns active project-root resolution and `zircon-project.toml` loading for the pane. `targets/rows.rs` owns the per-profile target row entry: it clones the manifest for the active profile, requests the native-aware export plan, and delegates target construction plus overlay application to child owners. `targets/rows/constructors.rs` owns ready/blocked target-row construction and diagnostic prefixing. `targets/rows/overlays.rs` owns completed export summary overlays and queued/running job overlays. `targets/diagnostics.rs` owns effective output-root diagnostic prefixing.

The root `app/build_export_projection.rs` stays structural around `RetainedEditorHost::build_export_pane_data(...)`: it gathers diagnostics, delegates target-row construction, attaches the first target's wizard view model, and returns `BuildExportPaneViewData`. This keeps pane assembly distinct from target-row derivation.

## Wizard Surface Actions

`app/build_export_wizard_session/surface_actions.rs` owns desktop export wizard surface action parsing and status text. It maps panel button control ids plus stable `workbench.build_export.*` action ids into `ExportWizardPanelAction`, builds per-profile wizard job ids, validates that plan/start actions receive pipeline options, and formats wizard status-line updates.

`app/build_export_wizard_session.rs` is the structural desktop export wizard session-family entry. `app/build_export_wizard_session/session_state.rs` owns only the retained per-profile session maps and declares the session-state child owners.

`app/build_export_wizard_session/session_state/actions.rs` owns profile action dispatch: generate plan, start with an injectable command runner, cancel, poll request dispatch, plan regeneration, and immediate action update emission.

`app/build_export_wizard_session/session_state/lookup.rs` owns profile view-model lookup and mutable session resolution, including missing-job error construction.

`app/build_export_wizard_session/session_state/polling.rs` owns polling all active wizard sessions, detecting changed snapshots or drained events, and returning only changed profile results. Action identity is not treated as independent polling state; GeneratePlan/Start updates are applied by the dispatch path that created them.

`app/build_export_wizard_session/host_actions.rs` owns the retained-host action entry points for the desktop export wizard. It maps surface action ids to wizard session actions, builds active-project options for plan/start actions, applies wizard updates to layout/status state, and polls all active wizard sessions from host lifecycle.

## Wizard Options

`app/build_export_wizard_session/options.rs` owns host option construction for desktop export wizard plan/start actions. It resolves the active project root and `zircon-project.toml`, applies the effective per-profile output root, looks up desktop export profiles, fills strategy/repo/source-manifest/host-executable/target-platform options, and derives the engine repository root from the editor crate manifest directory.

Keeping wizard option construction outside the session owner separates active-project/profile filesystem policy from per-profile session state and polling.

## Host Wizard Runtime

`ui/host/editor_manager_plugins_export/export_build/wizard/session.rs` owns the shared desktop export wizard panel registration used by retained-host Build/Export sessions. It now registers the v2 panel template together with the `editor_base.zui` import source through `EditorUiHostRuntime::register_v2_template_document_files(...)`, so retained projection can resolve declared v2 imports from a single registered document group.

`wizard/controller.rs` owns the job-system-facing controller declaration, while `wizard/controller/job.rs` is the narrow `EditorJob` execution owner, `wizard/controller/poll.rs` declares event-bearing pending/completed/typed-error ticket polling state, and `wizard/controller/completion.rs` owns direct-finish aggregation. The controller submits `JobCategory::Export` work through the injected `EditorJobSystem`, retains the export-domain event receiver and job-system clone, and maps cancellation to both the shared `CancellationToken` and `EditorJobSystem::cancel(ticket.id())`. Queued work is removed atomically; typed cancellation event/ticket is sent synchronously before its completed handle becomes visible to dependents, without worker submission or waiting for export quota. Scheduled/running work keeps cooperative token semantics. The owner maps business `Finished/Cancelled/Failed` snapshots to matching generic completed/cancelled/failed outcomes, with business diagnostics carried by typed failure. Session polling performs its normal business drain and then calls `JobTicket::try_take`; once ready, controller immediately drains the receiver again and returns those terminal events with the typed result. Session applies the event batch before final/error mapping, preserving final `StageFinished/Cancelled/Failed` stage/output/diagnostics across the two channels. Direct finish returns one `ExportWizardJobCompletion` containing all remaining business events plus the typed result and uses the same ordering. `ExportWizardPanelSession` and `DesktopExportWizardSessions` both require explicit job-system injection; startup assembly supplies `EditorManager.context().jobs().clone()`, while tests construct isolated fixtures.

`app/host_lifecycle/tick.rs` is the retained main-thread pump owner. Each frame pumps `EditorManager.context().jobs()` before polling export jobs and wizard sessions, so the same Context-owned service injected at construction publishes generic worker events to the editor message bus without a second pool/bus or worker-thread subscriber execution. Its static test scans production source only and proves the single retained tick owner plus ordering; it does not claim to establish OS thread identity.

`wizard/execution.rs` and `wizard/run.rs` own pipeline execution and job event sequencing. Cancellation is classified by the point where the cancel signal is observed: during command execution remains an in-stage cancellation, while a signal observed after stage completion is reported as phase-boundary cancellation by the job runner.

The export wizard test owner tree lives in `wizard/tests/{support,pipeline_plan,pipeline_execution,job,panel_session,view_model}.rs` with `mod.rs` as the local test entry. The former oversized root test file was deleted. 2026-06-22 validation passed `cargo fmt -p zircon_editor --check`, the editor structure audit with `oversized_production_file_count = 8`, old-file existence checks, line-count sampling, and scoped `git diff --check`; focused `export_wizard` Cargo testing previously stopped before editor tests on the active runtime `GpuMeshResource::indirect_order_signature` visibility error, and a later rerun timed out after 304 seconds without diagnostics. Matching cargo/rustc leftovers were stopped, so that historical slice did not claim a pass.

2026-06-24 M6 non-panel/session validation later passed the broader `export_wizard_` filter 58/58. The first full rerun failed only `desktop_export_wizard_sessions_project_view_model_after_generate_plan`: after GeneratePlan dispatch, idle `poll_all()` generated a Poll update with the same snapshot and no drained events, but the old `last_updates` cache compared action labels and treated the idle poll as changed. `DesktopExportWizardSessions` no longer stores action-level last updates; polling now reports changed sessions only when events drain or the projected snapshot changes. This keeps retained-host layout dirtiness tied to wizard state, while action dispatch still immediately applies GeneratePlan/Start updates.

## Boundary Rules

- Keep Build/Export action dispatch, plan refresh status, and direct output override mutations in `app/build_export_actions/host_actions.rs`.
- Keep desktop export job host-action module declarations in `app/build_export_actions/host_actions/jobs.rs`.
- Keep desktop export poll/update handling, completed-summary publication, start-next dispatch, and changed-layout marking in `app/build_export_actions/host_actions/jobs/polling.rs`.
- Keep desktop export enqueue preparation, active-project/profile/manifest/output-root snapshotting, queued-job creation, and immediate poll kick in `app/build_export_actions/host_actions/jobs/enqueue.rs`.
- Keep desktop export pending/active cancellation result handling and cancellation status text in `app/build_export_actions/host_actions/jobs/cancellation.rs`.
- Keep desktop export queue-to-status-task synchronization in `app/build_export_actions/host_actions/jobs/polling.rs`.
- Keep output-folder picker/reveal side effects and effective output-root lookup in `app/build_export_actions/host_actions/output.rs`.
- Keep Build/Export action-id grammar and parser behavior in `app/build_export_actions/action_ids.rs`.
- Keep completed export summary DTOs in `app/build_export_actions/execution_summary.rs`.
- Keep completed export summary constructors in `app/build_export_actions/execution_summary/constructors.rs`.
- Keep completed export summary status messages, diagnostics, labels, and fatal-state detection in `app/build_export_actions/execution_summary/status.rs`.
- Keep completed summary target-row application in `app/build_export_actions/execution_summary/target.rs`.
- Keep export profile catalog, platform labels, profile lookup, and default output-root convention in `app/build_export_actions/profiles.rs`.
- Keep `app/build_export_actions/job_queue.rs` as the structural job-queue entry.
- Keep injected jobs ownership, pending/active typed state, progress channel, and immediate terminal summaries in `app/build_export_actions/job_queue/state.rs`.
- Keep queued job creation in `app/build_export_actions/job_queue/enqueue.rs`.
- Keep busy checks and active/pending queue snapshots in `app/build_export_actions/job_queue/queries.rs`.
- Keep pending/active cancellation state and `DesktopExportCancellation` results in `app/build_export_actions/job_queue/cancellation.rs`.
- Keep progress draining, ticket polling, and unified result-summary collection in `app/build_export_actions/job_queue/updates.rs`.
- Keep start-next active-job transition and `JobCategory::Export` submission in `app/build_export_actions/job_queue/start.rs`.
- Keep job snapshot DTOs and progress snapshot DTOs in `app/build_export_actions/job_queue/snapshot.rs`.
- Keep job progress-report conversion in `app/build_export_actions/job_queue/snapshot/progress.rs`.
- Keep queued/running/cancelled pane and status-task text formatting in `app/build_export_actions/job_queue/snapshot/status.rs`.
- Keep job snapshot target-row overlay application in `app/build_export_actions/job_queue/snapshot/target.rs`.
- Keep desktop export status-task projection with the current status owner in `app/build_export_actions/job_queue/snapshot/status.rs`.
- Keep the desktop export `EditorJob` adapter, progress DTO, typed result DTO, and business result-to-summary conversion in `app/build_export_actions/job_queue/worker.rs`.
- Keep output-folder picker/reveal platform integration in `app/build_export_actions/output_folder.rs` and its `output_folder/` children.
- Keep output-folder picker process execution and failure handling in `app/build_export_actions/output_folder/picker.rs`.
- Keep output-folder picker platform command construction in `app/build_export_actions/output_folder/picker/commands.rs`.
- Keep output-folder picker selected-folder parsing and stable initial-directory selection in `app/build_export_actions/output_folder/picker/selection.rs`.
- Keep `app/build_export_projection.rs` as the structural pane-data entry.
- Keep Build/Export target projection orchestration in `app/build_export_projection/targets.rs`.
- Keep active project-root and manifest loading in `app/build_export_projection/targets/project.rs`.
- Keep per-profile target-row entry orchestration in `app/build_export_projection/targets/rows.rs`.
- Keep ready/blocked Build/Export target-row construction in `app/build_export_projection/targets/rows/constructors.rs`.
- Keep completed-summary and queued/running job target-row overlays in `app/build_export_projection/targets/rows/overlays.rs`.
- Keep output-root diagnostic prefixing in `app/build_export_projection/targets/diagnostics.rs`.
- Do not add target row projection back to action routing or job queue code.
- Keep `app/build_export_wizard_session.rs` as the structural desktop export wizard session-family entry.
- Keep desktop export wizard per-profile session maps in `app/build_export_wizard_session/session_state.rs`.
- Keep profile action dispatch, plan regeneration, and injectable process-runner start dispatch in `app/build_export_wizard_session/session_state/actions.rs`.
- Keep view-model lookup and mutable session resolution in `app/build_export_wizard_session/session_state/lookup.rs`.
- Keep changed-update polling for active wizard sessions in `app/build_export_wizard_session/session_state/polling.rs`.
- Keep desktop export wizard retained-host action dispatch, active-project option lookup for plan/start, poll status propagation, and layout/status mutation in `app/build_export_wizard_session/host_actions.rs`.
- Keep desktop export wizard surface button mapping, wizard job id construction, required-option validation, and wizard status-line message formatting in `app/build_export_wizard_session/surface_actions.rs`.
- Keep active-project export wizard option construction, default source asset manifest path, default host executable path, and engine repo-root derivation in `app/build_export_wizard_session/options.rs`.

## Validation Notes

The 2026-07-12 typed export-error hard cutover adds actual ticket-level regression coverage for both export adapters. Wizard tests submit the real controller and prove that ordinary process IO failure plus a process failure observed after an in-flight cancellation request remain downcastable `EditorExportBuildError` values. The retained queue test submits `DesktopExportEditorJob` through `EditorJobSystem` and recovers the same typed error from `JobTicket`. Native preparation owns staging cleanup for its full lifetime, attempts both temporary roots, and retains materialization-plus-cleanup or process-plus-termination dual failures as structured values. Windows validation passed `cargo check -p zircon_editor --lib --locked --jobs 1` and the focused wizard 3/3 plus retained worker 1/1 tests; string conversion remains confined to UI diagnostics after typed retention.

The 2026-06-18 job-queue split reduced `build_export_actions.rs` from 896 lines to 498 lines. `build_export_actions/job_queue.rs` is 413 lines and owns the desktop export queue, progress snapshots, cancellation result, status task projection, pane job overlay helper, worker thread execution, and result-to-summary conversion. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export job-queue ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 job-queue snapshot split reduced `build_export_actions/job_queue.rs` from 413 lines to 285 lines. `build_export_actions/job_queue/snapshot.rs` is 106 lines and owns job/progress snapshot DTOs, pane diagnostics, Build/Export target overlay application, and status task projection. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export job-queue snapshot ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 job-queue operation subowner split reduced `build_export_actions/job_queue.rs` from 225 lines to 118 lines. At that historical point `updates.rs` polled worker messages and `start.rs` spawned a worker; Plan 14 M2.2 later replaced those mechanics with progress draining plus typed ticket polling and `EditorJobSystem` submission while retaining the same subowner boundaries.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export job-queue subowner ownership scan, and scoped `git diff --check`, all of which passed except for existing CRLF conversion warnings in the dirty worktree. Focused `cargo check` was not rerun for this slice because independent `zircon_runtime` Cargo test/check processes were still active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 action/profile split reduced `build_export_actions.rs` to 360 lines. `build_export_actions/action_ids.rs` is 78 lines and owns `BuildExportAction` plus action-id parsing. `build_export_actions/profiles.rs` is 74 lines and owns export profile definitions, profile lookup, platform labels, and default output-root construction. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export action/profile ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed after widening child item visibility to the app boundary. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 execution-summary split reduced `build_export_actions.rs` to 218 lines. `build_export_actions/execution_summary.rs` is 130 lines and owns `DesktopExportExecutionSummary`, completed export state construction, status messages, pane diagnostics, fatal-state detection, and completed-summary target row projection. `build_export_actions/job_queue.rs` remains responsible for queue/progress/cancellation snapshots and imports only the summary DTO constructors. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export execution-summary ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 wizard surface-actions split reduced `build_export_wizard_session.rs` from 307 lines to 242 lines. `build_export_wizard_session/surface_actions.rs` is 79 lines and owns wizard surface action mapping, wizard job id construction, required-option validation, and wizard status messages. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export wizard surface-actions ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 wizard options split reduced `build_export_wizard_session.rs` from 261 lines to 211 lines. `build_export_wizard_session/options.rs` is 59 lines and owns active-project wizard option construction, source asset manifest defaulting, host executable path construction, export profile lookup, and engine repository root derivation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export wizard options ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 wizard host-actions split reduced `build_export_wizard_session.rs` from 194 lines to 130 lines. `build_export_wizard_session/host_actions.rs` is 65 lines and owns retained-host wizard action dispatch, active-project options lookup for plan/start actions, update application, poll status propagation, and layout/status mutation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export wizard host-actions ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 host-action split reduced `build_export_actions.rs` from 218 lines to 21 lines. `build_export_actions/host_actions.rs` is 209 lines and owns retained-host desktop export job polling/start-next/status-task sync, parsed Build/Export action dispatch, enqueue/cancel/output override mutation, folder picker/reveal side effects, and effective output-root resolution. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export host-action ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 wizard session-state split reduced `build_export_wizard_session.rs` from 140 lines to 8 lines. `build_export_wizard_session/session_state.rs` is 133 lines and owns `DesktopExportWizardSessions`, stored per-profile sessions, view-model lookup, action dispatch with injectable runner, plan regeneration, changed-snapshot polling, and missing-job errors. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export wizard session-state ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. The follow-up owner-split batch cargo check caught the moved session store visibility, so `DesktopExportWizardSessions` plus `view_model(...)`, `dispatch_profile_action(...)`, and `poll_all(...)` were widened to `pub(in crate::ui::retained_host::app)` while test-only internals remained parent-module visible. `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` then passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remained deferred for that historical slice.

The 2026-06-24 export wizard session polling fix removes action-level `last_updates` from `DesktopExportWizardSessions`. `session_state/actions.rs` still returns action updates for immediate host application, while `session_state/polling.rs` now treats `events_drained > 0` or snapshot inequality as the only non-error changed states. Validation passed `rustfmt --edition 2021` for the three touched session-state files, the focused retained-host session test 1/1, and `cargo test -p zircon_editor --lib export_wizard_ --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-descriptor-rerun-0624 --message-format short --color never -- --test-threads=1 --nocapture` 58/58 with existing warning noise only.

The 2026-06-19 wizard session-state subowner split reduced `build_export_wizard_session/session_state.rs` from 136 lines to a 15-line retained map owner. `session_state/actions.rs` is 70 lines and owns action dispatch plus plan regeneration. `session_state/lookup.rs` is 28 lines and owns view-model/session lookup. `session_state/polling.rs` is 45 lines and owns changed-update polling. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export wizard session-state subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 host-action job/output split reduced `build_export_actions/host_actions.rs` from 209 lines to 52 lines. `host_actions/jobs.rs` is 98 lines and owns desktop export polling, start-next, enqueue, cancellation, and status-task sync. `host_actions/output.rs` is 69 lines and owns output picker/reveal side effects plus effective output-root lookup.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, and an app build-export host-action job/output ownership scan. A fresh `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` was attempted but failed before reaching editor code on current `zircon_runtime` worktree errors in `graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs`: missing `ViewportCameraStackAttachmentPolicy` export and a borrow after moving `submission.frame`. A narrower no-default editor check was attempted and timed out, so this slice does not claim a fresh compile pass. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 host-action job subowner split originally created a dedicated `status_task.rs`; the later job hard cutover removed that file and folded queue-to-status-task synchronization into current `host_actions/jobs/polling.rs`. `host_actions/jobs/enqueue.rs` continues to own busy-profile rejection, active-project/profile/manifest/output-root snapshotting, queued-job creation, and immediate poll kick; `host_actions/jobs/cancellation.rs` owns pending/active cancellation result handling and status text.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export host-action job subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 output-folder visibility correction widened `output_folder/picker.rs` picker helpers and `output_folder/reveal.rs` reveal helper to `pub(in crate::ui::retained_host::app::build_export_actions)`, preserving the sibling `host_actions/output.rs` import path without exposing OS integration outside the Build/Export action family. Validation used `cargo fmt -p zircon_editor`, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 projection-target subowner split reduced `build_export_projection/targets.rs` from 164 lines to 33 lines. New child owners are `targets/project.rs` (12 lines), `targets/rows.rs` (116 lines), and `targets/diagnostics.rs` (12 lines). `build_export_actions.rs` now re-exports `DesktopExportJobSnapshot` at the app boundary so the target row child can accept job snapshots explicitly while the job queue internals stay under the Build/Export action family.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export projection-target subowner ownership scan, and scoped `git diff --check` (only the existing CRLF conversion warning appeared). A fresh `cargo check` was deferred for this slice because separate runtime Cargo checks are currently active in this workspace; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 owner-split compile correction established that `build_export_actions/job_queue/worker.rs` imports `DesktopExportExecutionSummary` from the Build/Export action parent rather than the job-queue child. Plan 14 M2.2 preserves that summary ownership but now feeds it from a typed `DesktopExportJobResult` consumed by `updates.rs`, not a finished worker message. The operation child boundaries remain `cancellation.rs`, `updates.rs`, and `start.rs`; current line counts are intentionally not frozen in this document.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export job-queue ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 execution-summary subowner split reduced `build_export_actions/execution_summary.rs` from 130 lines to a 26-line structural entry. `execution_summary/constructors.rs` is 55 lines and owns exported/failed/cancelled summary construction. `execution_summary/status.rs` is 60 lines and owns fatal detection, status-line messages, target labels, and pane diagnostics. `execution_summary/target.rs` is 13 lines and owns completed-summary application to `BuildExportTargetViewData`.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export execution-summary subowner ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. A fresh `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never` was attempted, but it failed before reaching editor code on current `zircon_runtime` duplicate method definitions in `scene/dynamic_scene/session/path_capture.rs` (`capture_*_to_path*` and `preview_*_to_path*` methods). This slice does not claim a fresh compile pass; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The follow-up 2026-06-19 owner-split batch compile validation reran `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 output-folder picker command/selection split reduced `build_export_actions/output_folder/picker.rs` from 131 lines to a 45-line process execution owner. `picker/commands.rs` is 78 lines and owns platform-specific picker command construction. `picker/selection.rs` is 17 lines and owns stable initial-directory selection plus selected-folder parsing.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export output-folder picker command/selection ownership scan, and scoped `git diff --check`. Focused `cargo check` was not rerun for this slice because an independent `zircon_runtime` Cargo test process was active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 retained-host owner visibility compile-boundary correction widened `output_folder/picker/commands.rs::folder_picker_commands(...)` and `output_folder/picker/selection.rs::parse_selected_folder(...)` only to `pub(in crate::ui::retained_host::app::build_export_actions::output_folder)`. This preserves the `picker.rs` re-export and `output_folder/tests.rs` coverage after the picker subowner split without exposing OS command construction outside the output-folder integration family. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app retained-host owner visibility compile-boundary scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 job snapshot projection subowner split originally created a dedicated `snapshot/status_task.rs`; the later job hard cutover removed that file and folded status-task projection into current `snapshot/status.rs`, alongside queued/running/cancelled status text and pane diagnostics. `snapshot/progress.rs` retains progress-report conversion and `snapshot/target.rs` retains Build/Export target-row overlay application.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export job-queue snapshot subowner ownership scan, and scoped `git diff --check`. Focused `cargo check` was not rerun for this slice because independent Cargo/rustc processes were active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 target rows subowner split reduced `build_export_projection/targets/rows.rs` from 116 lines to a 36-line per-profile target-row entry. `targets/rows/constructors.rs` is 72 lines and owns ready/blocked target construction. `targets/rows/overlays.rs` is 18 lines and owns completed-summary plus queued/running job overlays.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export target rows subowner ownership scan, and scoped `git diff --check`. Focused `cargo check` was not rerun for this slice because independent Cargo/rustc processes were active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 job-queue state/enqueue/query split reduced `build_export_actions/job_queue.rs` from 118 lines to a 16-line structural entry. `job_queue/state.rs` is 45 lines and owns pending/active job structs plus queue channels, `job_queue/enqueue.rs` is 35 lines and owns queued job creation, and `job_queue/queries.rs` is 37 lines and owns busy-profile checks plus active/pending snapshots.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app build-export job-queue state/enqueue/query ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
