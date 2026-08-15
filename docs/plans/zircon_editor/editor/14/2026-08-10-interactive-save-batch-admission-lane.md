Plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
Milestone: M3
Status: source_bound_managed_validation_pending
Files: ["docs/plans/zircon_editor/editor/14/2026-08-10-interactive-save-batch-admission-lane.md", "docs/plans/zircon_editor/editor/14/failure-2026-08-01-interactive-save-batch-admission-lane.md", "tools/tests/test_editor14_interactive_save_job_adapter_contract.py", "zircon_editor/src/core/asset/dirty/mod.rs", "zircon_editor/src/core/asset/dirty/save_batch.rs", "zircon_editor/src/core/asset/dirty/save_job_adapter.rs", "zircon_editor/src/core/asset/dirty/save_job_adapter/tests.rs", "zircon_editor/src/core/jobs/admission.rs", "zircon_editor/src/core/jobs/error.rs", "zircon_editor/src/core/jobs/mod.rs", "zircon_editor/src/core/jobs/system/admission_reservation.rs", "zircon_editor/src/core/jobs/system/mod.rs", "zircon_editor/src/core/jobs/system/pending.rs", "zircon_editor/src/core/jobs/system/pending_task.rs", "zircon_editor/src/core/jobs/system/state.rs", "zircon_editor/src/core/jobs/tests/admission_scaling_contract.rs"]
Depends-On-Snapshots: ["1604"]
---

# Editor14 Interactive Save Batch Admission Lane

## Scope And Pending Validation

The one editor job system now exposes `EditorJobBatchAdmissionReservation`. It claims all requested entry and estimated-byte capacity in the shared pending-state transaction before document save-mutex resolution, executor construction, job objects, or result channels are materialized. A matching commit preserves the admission ids and timestamps; a mutex/factory/commit error drops the reservation, and system shutdown releases every uncommitted claim.

Each admitted item is `InteractiveSave + Interactive`, retains only a light save intent and shared executor, and uses the caller-supplied foreground/autosave mutex owner. Completion polling has an explicit ticket budget with a default of 64. Partial failures remain typed and flow into the existing generation-safe apply/retry contract; adapter shutdown rejects new batches and cancels only its owned tickets while the global job system remains the deadline owner.

This milestone intentionally stops at the shared Editor14 boundary. Editor06 and Editor09 still own the product executor wiring, save-all state machine, close-prompt commit rule, and the full 1/100/10k plus payload/stall performance matrix after the fixed handoff returns. No synchronous fallback, second worker owner, legacy facade, or unconditional mark-clean path was added.

## Validation And Review

- Rust behavior tests cover rejection before mutex/executor materialization with a partially occupied queue, reservation rollback after mutex and commit mismatch failures, entry-and-byte capacity recovery, reuse of the caller-supplied foreground save mutex, typed partial failure with generation-safe retry, cancellation/shutdown, and an explicit bounded completion scan.
- The M4 `system/` hard cut moved the admission-window data owner to `construction.rs` and the submission methods to `submission.rs`. The static contract now checks those leaf owners plus the root's selected re-exports and rejects restoring behavior to `mod.rs`. The pre-hard-cut 16-path snapshot is therefore superseded; the 47-path composite successor is the only current-source input for this failure. Its exact Rust sources pass `rustfmt --edition 2024 --check`; scoped diff checking reports only the repository's LF/CRLF checkout notices.
- Source-bound managed Cargo validation remains required. No direct Cargo command is permitted or claimed.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-10 | Editor14 interactive save batch admission lane | source_bound_managed_validation_pending | Shared `EditorJobBatchAdmissionReservation` now atomically claims `InteractiveSave + Interactive` entry/byte capacity before save-mutex or executor materialization; matching commits preserve admission order and RAII/shutdown release uncommitted claims. Static contract 3/3, scoped rustfmt/diff checks, and the second independent review (0/0/0) pass. Snapshot 1606 seals the current 16-path manifest; managed Cargo terminal evidence is required before returning the failure. Editor06/09 upper wiring stays outside this lower-layer milestone. |
| 2026-08-11 | Editor14 interactive save hard-cut contract successor | open / source-closure-forward-fixed | Reproduced the stale root-location contract failure after M4, then changed the static contract to require `construction.rs`/`submission.rs` ownership and root selected re-exports. The current-source candidate is enlarged to the dedicated M1-M4 composite successor rather than treating snapshot 1606 as valid after the shared module split. | Static contract 3/3 and the exact 47-path Rust `rustfmt --check` pass after repairing two format-only asset-save files; scoped diff checking has only LF/CRLF notices. Managed Cargo and the required independent second review still remain before acceptance. |
