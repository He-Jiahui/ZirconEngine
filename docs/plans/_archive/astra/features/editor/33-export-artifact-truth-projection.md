---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/101-editor-project-operations-source-control-changelist-diff-automation-validation-submission-health-current-source-review.md
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
related_code:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection/astra_artifact_truth_tests.rs
tests:
  - tools/tests/test_editor_build_export_projection_performance_contract.py
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection/astra_artifact_truth_tests.rs
---

# Export Artifact Truth Projection

The export-wizard panel now projects only execution-announced artifact paths.
It no longer clones and merges planned output descriptors into the published
artifact list, so a plan cannot appear as a completed result after cancellation
or failure. JSON in stdout remains log data until the Report stage has actually
announced a report artifact.

This also removes the per-projection planned-artifact vector clone and avoids
parsing stdout JSON on runs that produced no report artifact. The optimization
is intentionally bounded to panel projection; it does not claim end-to-end
export latency or allocation percentiles.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| E33 | Publish only execution-confirmed export artifacts and remove planned-artifact merge allocation | implemented_pending_validation | Module tests cover pending, running, cancelling, cancelled, failed, and finished projections. The local source-contract test additionally locks actual-only artifact projection and report-artifact-gated stdout parsing. The combined R38/E33 contract batch passed 24/24, and the latest cross-slice Runtime manifest/Editor export batch passed `42/42`; Python compilation, E33 rustfmt, and scoped diff checks passed. Managed Rust compilation and Windows Release p50/p95/p99 evidence remain pending. |

## Coordinator dispatch log

- 2026-09-11: the combined `zircon_app`/`zircon_runtime`/`zircon_editor`
  Cargo batch was submitted with request id
  `astra-runtime-editor-compile-batch-20260911-r1`, but immutable admission
  rejected it before a ticket was created because external worktree
  `E:\\Git\\zr_vm` is dirty. No Cargo command ran and this record does not infer
  a compile result from the rejection.
