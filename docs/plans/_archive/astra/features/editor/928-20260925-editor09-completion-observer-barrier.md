---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/131-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/editor/927-20260925-editor09-play-output-task-owner.md
implementation_files:
  - zircon_editor/src/core/jobs/system/construction.rs
  - zircon_editor/src/core/jobs/system/scheduling.rs
  - zircon_editor/src/core/jobs/system/lifecycle.rs
tests:
  - zircon_editor/src/core/jobs/tests/progress_contract.rs
---

# Editor928 Editor09 Terminal Observer Completion Barrier

## 完成列表

| Scope | Change | Evidence and remaining gate |
| --- | --- | --- |
| Terminal-side admission | Active completion, pending cancellation, and explicit pending-job cancellation increment an instance-owned atomic barrier before removing the progress entry. | `CompletionBarrierGuard` closes the gap between `progress.complete` and observer/promotion side effects; managed Rust validation is pending. |
| Shutdown predicate | `EditorJobSystem::shutdown` waits for both active progress entries and in-flight terminal barriers, with the existing deadline behavior preserved. | A blocking observer regression test proves shutdown cannot return while `job_finished` is still executing; the grouped Editor package receipt is still pending. |
| Scope boundary | The barrier covers job-core terminal observer/promotion quiescence only. | ProcessSupervisor pipe-close/quarantine, runtime task ownership, and dynamic-library unload remain separate parent-plan work; no broader P0 closure is claimed. |

## Validation boundary

Rustfmt and scoped whitespace checks pass for the implementation and focused
test. The new regression has not been run through Cargo directly; it must be
included in the next grouped managed Editor development batch. Existing
coordinator lanes were reconciled as wrapper failures without retained
`cargo_job_runs` diagnostics, so they provide no compiler or test receipt for
this change. Release shutdown/performance thresholds remain pending. No tooling
migration was made.

The test intentionally blocks `job_finished` on a barrier, starts shutdown on
a separate thread, asserts that shutdown remains pending, then releases the
observer and verifies an empty unfinished-job result. This isolates the
lower-layer race without claiming the full process/session shutdown contract.

### Four-lane managed submission after implementation (2026-09-25)

The current source was submitted as a grouped Runtime/Editor wave: Runtime
development PTY `76534`, Editor development PTY `16310`, Runtime02 Release PTY
`34527`, and Editor09 Release PTY `37590`. The wrappers were left unpolled as
requested; launch output only contained terminal-control setup. These entries
are submission receipts, not Cargo or performance results. The record remains
`implemented_pending_validation` until a managed compiler/test receipt and the
Release shutdown/throughput thresholds are available.

The submission predates the follow-up extension that applies the same barrier
to explicit pending-job cancellation. Its receipts therefore cannot be
attributed to the final source; a later grouped current-source batch is still
required.

### Current-source four-lane resubmission (2026-09-25)

The cancellation-path extension was submitted as a fresh grouped wave: Runtime
development PTY `92163`, Editor development PTY `57512`, Runtime02 Release PTY
`65557`, and Editor09 Release PTY `88159`. The wrappers were left unpolled;
their launch output contained only terminal-control setup. No Cargo job,
compiler/test count, or Release threshold is inferred until a managed terminal
receipt is available.

### Local contract evidence (2026-09-25)

The shared non-coordinator contract batch completed 88 Runtime/Editor tests with
`OK`, including the Editor09 suite and the completion-observer source contracts.
Managed Cargo and Release performance receipts remain pending by design.
