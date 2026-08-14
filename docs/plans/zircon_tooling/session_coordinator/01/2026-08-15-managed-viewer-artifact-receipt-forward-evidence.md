---
record_kind: failure_forward_evidence
status: resolving
recorded_at: 2026-08-15
summary_slug: managed-viewer-artifact-receipt
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
source_artifact: docs/plans/zircon_tooling/session_coordinator/01/failure-2026-08-14-managed-viewer-artifact-receipt.md
---

# managed-viewer-artifact-receipt forward evidence

Coordinator receipt issuance is now bound to the immutable validation copy,
while the lifecycle remains open for the Shader06 origin owner to return:

- A passed ticket from the same Session is no longer sufficient. Its canonical
  source manifest must be a hash-identical or tombstoned subset of the
  materialized copy before a receipt can be issued.
- Source files are verified outside the SQLite write transaction. The copy and
  ticket signatures are then revalidated inside the short receipt transaction,
  preventing stale-snapshot insertion without holding the writer lock during
  file hashing.
- The focused RED proved that an old copy could previously accept a newer
  same-Session ticket and that receipt issuance performed no source hashing.
  Both regressions now pass, and the complete
  `tools.session_coordinator.tests.test_artifact_receipts` suite passes 13/13.
- `python -m compileall -q tools/session_coordinator` and `git diff --check` on
  the scoped Coordinator paths pass. No product Cargo was launched for this
  repair.

The Coordinator failure-return gate correctly refused cross-plan closure
without an active Shader06 origin-plan lease for the fixed destination.
Shader06 retains responsibility for the managed viewer capture and final
lifecycle return; Coordinator must not claim or rewrite its consumer paths.
