Plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
Milestone: M1
Status: source_bound_validation_pending
Files: ["docs/plans/zircon_editor/editor/14/2026-08-11-m1-job-system-current-source-manifest.md", "zircon_editor/src/core/jobs/admission.rs", "zircon_editor/src/core/jobs/error.rs", "zircon_editor/src/core/jobs/event.rs", "zircon_editor/src/core/jobs/event_sink.rs", "zircon_editor/src/core/jobs/limits.rs", "zircon_editor/src/core/jobs/mod.rs", "zircon_editor/src/core/jobs/progress.rs", "zircon_editor/src/core/jobs/quota_settings.rs", "zircon_editor/src/core/jobs/system/admission_ledger.rs", "zircon_editor/src/core/jobs/system/admission_reservation.rs", "zircon_editor/src/core/jobs/system/mod.rs", "zircon_editor/src/core/jobs/system/pending.rs", "zircon_editor/src/core/jobs/system/pending_task.rs", "zircon_editor/src/core/jobs/system/state.rs", "zircon_editor/src/core/jobs/tests/admission_scaling_contract.rs", "zircon_editor/src/core/jobs/tests/admission_scaling_contract/indexed.rs", "zircon_editor/src/core/jobs/tests/admission_scaling_contract/keyed.rs", "zircon_editor/src/core/jobs/tests/admission_scaling_contract/reservation.rs", "zircon_editor/src/core/jobs/tests/admission_scaling_contract/support.rs", "zircon_editor/src/core/jobs/tests/background_storm_contract.rs", "zircon_editor/src/core/jobs/tests/quota_settings_contract.rs", "zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs", "zircon_editor/src/core/jobs/tests/thread_ownership_contract/scanner.rs"]
Depends-On-Failures: ["docs/plans/zircon_editor/editor/14/failure-2026-07-17-job-pump-budget-and-pending-scan.md", "docs/plans/zircon_editor/editor/02/failure-2026-07-17-message-inbox-backpressure-and-fanout.md"]
---

# Editor14 M1 Job System Current-Source Manifest

## Scope Delivered

This manifest freezes the current Editor14 JobSystem source set for managed M1 validation:
indexed admission accounting, quota resolution, batch reservation ownership, progress delivery
support, and the folder-backed contracts that protect thread ownership and admission scaling.
It contains no Editor02 message-bus source and no Runtime11 blocking-I/O source.

## Fresh Testing Evidence

- `python -B -m unittest tools.tests.test_check_conventions -v`: 29/29 passed.
- Scoped `rustfmt --edition 2024 --check` and `git diff --check` passed for the extracted
  test owners.
- The static admission-contract inventory retains all 17 test functions; the thread ownership
  guard extraction was source-equivalent apart from module visibility.
- No direct Cargo command was run. Managed M1 validation must supply the current-source compile
  and behavior evidence.

## Review

Independent read-only second review reported `Critical/Important/Minor = 0/0/0`. It confirmed
the Index dependency scenario retains the category slot before runtime scheduling, all moved
test modules resolve through their parent owners, and every affected test file is below the
800-line folder-backed limit.


## Failure State

M1 is not accepted. The JobPump failure remains open because bounded lossless lifecycle delivery
requires Editor02's non-consuming lossless producer admission contract before Editor14 can retain
and retry the original event without a deep clone. The related performance profile also remains a
managed Windows gate.
