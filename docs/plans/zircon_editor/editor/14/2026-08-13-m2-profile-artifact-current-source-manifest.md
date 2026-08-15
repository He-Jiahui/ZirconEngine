---
doc_type: source-manifest
status: source_bound_profile_validation_pending_external_thread_owner_returns
owner_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
milestone: m2-profile-artifact-worker-hard-cut
exact_path_count: 16
Files: ["docs/plans/zircon_editor/editor/14/2026-08-13-m2-profile-artifact-current-source-manifest.md", "docs/plans/zircon_editor/editor/14/failure-2026-08-12-thread-ownership-guard-test-scope.md", "docs/zircon_editor/ui/performance-timeline.md", "docs/zircon_editor/ui/retained_host/host_contract/profiling_artifacts.md", "docs/zircon_editor/ui/retained_host/host_contract/window.md", "zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs", "zircon_editor/src/core/jobs/tests/thread_ownership_contract/scanner.rs", "zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport.rs", "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs", "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs", "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment/tests.rs", "zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/export.rs", "zircon_editor/src/ui/retained_host/host_contract/window.rs", "zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs", "zircon_editor/src/ui/retained_host/host_contract/window/profile_artifact_job_tests.rs", "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs"]
Depends-On-Failures: ["docs/plans/zircon_runtime/runtime/11/failure-2026-08-10-blocking-io-process-output-budget.md", "docs/plans/zircon_editor/editor/12/failure-2026-08-12-native-plugin-development-watcher-job-ownership.md"]
---

# Editor14 M2 Profile Artifact Current-Source Manifest

## Scope

This exact manifest freezes the Editor14-owned profile-artifact repair for later managed validation. It replaces the
retained-host process-lifetime writer channel with an injected `EditorJobSystem` `Export` job, reserves shared
admission before presentation materialization or screenshot painting, returns the canonical `JobId`, reports
admission and explicit invalid-output-root rejection through host diagnostics, and makes the final `UiHostWindow` owner request cooperative
cancellation.

The startup composition leaf injects the same `EditorManager` job system before `RetainedEditorHost` assembly.
`window/profile_artifact_job_tests.rs` owns that host-lifetime contract without absorbing unrelated host-window test
changes. The module documents record the new submission and cancellation ownership; `ZIRCON_PROFILE_OUTPUT_ROOT` is
validated as an absolute non-`C:` artifact destination before any export payload is materialized; UNC roots remain
valid, while a configured `C:`, relative, or device-namespace root is a visible typed submission error.

## Excluded Owners

This manifest deliberately excludes Runtime11's bounded stream implementation and Editor12's native plugin
development watcher implementation. The crate-wide M2 no-bare-thread guard scans the complete production tree, so
its final green result is correctly deferred until both external fixed returns remove their raw process readers and
watcher worker. This snapshot may support the Profile-focused behavior commands below, but must not be presented as
the full M2 guard acceptance or a failure return.

## Static Evidence

- Scoped `rustfmt --edition 2024 --check --config skip_children=true` passed for the seven Profile/window/startup
  Rust owners in this manifest.
- Scoped `git diff --check` passed for all 16 listed paths.
- The old Profile worker symbols have zero production hits in `profiling_artifacts/export.rs`: `OnceLock`,
  `SyncSender`, `sync_channel`, `std::thread::Builder`, and `std::thread::spawn`. The replacement uses
  `EditorJobSystem`, `reserve_batch_admission`, and deferred `materialize_presentation`.
- Independent current-source re-review after the typed output-root repair reported
  `Critical/Important/Minor = 0/0/0`; it covers injection, final-owner cancellation, admission-before-materialization,
  typed host warnings, non-`C:` drive and normal UNC acceptance, and `\\?\`/`\\.\` device-path rejection.
- No direct Cargo command was run.

## Required Managed Validation

- `cargo test -p zircon_editor --lib present_artifact_export_runs_as_an_injected_export_job --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib profile_artifact_admission_reservation_bounds_capture_before_materialization --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib profile_artifact_rejection_precedes_export_materialization --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib rejected_profile_artifact_submission_records_a_host_warning --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib invalid_profile_output_root_precedes_export_materialization --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib invalid_profile_output_root_records_a_host_warning --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib profile_artifact_job_is_cancelled_when_the_final_host_owner_drops --locked --jobs 1 -- --test-threads=1`
- `cargo test -p zircon_editor --lib profile_output_root_requires_an_absolute_non_c_drive_path --locked --jobs 1 -- --test-threads=1`

After the Runtime11 and Editor12 fixed returns, create a fresh successor manifest for the two M2 thread-guard
commands declared in the failure record. Do not reuse a Profile-only receipt to accept the global scanner.
