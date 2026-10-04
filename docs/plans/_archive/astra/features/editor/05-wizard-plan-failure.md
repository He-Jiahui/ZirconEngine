---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/257-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-working-tree-review.md
---

# Wizard plan failure

## Repairs

Invalid wizard plans can reach the public controller before a stage starts. The
job state returns Failed with no stages, but the adapter previously expected a
last stage and panicked. It now returns WizardPlanFailed with the profile and
complete diagnostics moved from the snapshot. Existing stage failures and typed
runner failures retain their existing propagation.

The regression submits both an invalid previous/delta pack pair and an unavailable
provider plan through the real job service. It checks the downcast error, matching
terminal event diagnostics, zero stage records and zero runner invocations.
The existing backpressure regression also uses the current fixed-slot count type.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M18 | Typed rejection for pre-execution wizard plan failures | implemented_pending_validation | Static review and formatting passed; batch with M19-M20 and earlier regressions |

This is a correctness repair. No performance improvement or test pass is claimed.
M1-M17 stopped before compilation on a manifest/lock mismatch. M20 repaired the
lock and passed locked offline metadata resolution; the combined replacement
run remains required. Its prior receipt is retained in
`.codex/state/astra-m1-m17-validation-20260905.json`.
