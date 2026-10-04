---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/131-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/editor/928-20260925-editor09-completion-observer-barrier.md
implementation_files:
  - zircon_editor/src/core/jobs/system/lifecycle.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
tests:
  - zircon_editor/src/core/jobs/tests/scheduling_contract.rs
---

# Editor929 Editor09 Job Join Capability Boundary

## 完成列表

| Scope | Change | Evidence and remaining gate |
| --- | --- | --- |
| Scheduler escape surface | `EditorJobSystem::join` is now `pub(crate)` rather than an external public API; the existing export-process capture adapter remains in-crate and unchanged. | The scheduling contract asserts the visibility boundary and existing single-worker capture tests retain the borrowing join behavior. Managed Editor compilation/test evidence is pending. |
| Parent-plan boundary | This is only the external-surface hardening step. | A future `JobExecutionContext::parallel_join` still needs to carry parent resource accounting before the broader E-JOB-P1-07 item can be closed. |

## Validation boundary

Rustfmt and scoped diff checks are required after the current source wave. The
previous four-lane managed submission predates this visibility change and is
not attributed to the final source. A later grouped current-source Runtime /
Editor development plus Release batch must provide Cargo and performance
receipts; no tooling migration was made.

The current-source grouped submission uses Runtime development PTY `22890`,
Editor development PTY `85495`, Runtime02 Release PTY `45349`, and Editor09
Release PTY `34355`. The wrappers were left unpolled; launch output is not a
Cargo or performance receipt.

### Local contract evidence (2026-09-25)

The shared non-coordinator Runtime/Editor contract batch completed 88 tests with
`OK`. This verifies the crate-private join boundary and adjacent Editor09 source
contracts locally; managed Cargo compilation and Release performance remain pending.
