---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
related_records:
  - docs/plans/astra/features/editor/925-20260925-editor-compile-host-file-backed-output-owner.md
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
implementation_files:
  - zircon_editor/src/core/play/process_backend/output.rs
tests:
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs
---

# Editor927 Play Output Runtime Task Owner

## 完成列表

| Scope | Change | Evidence | Status |
| --- | --- | --- | --- |
| Play stdout/stderr reader ownership | Replace the two production `std::thread::Builder` reader owners with a dedicated runtime `TaskPool` I/O owner. The pool is process-scoped, capped at the two Play streams, and does not reuse the persistence I/O lane. | `output.rs` uses `TaskPoolDescriptor::io().with_worker_threads(2)` and the Editor thread-ownership scanner no longer finds a bare thread owner in the Play output module. | implemented |
| Reader completion barrier | Each reader owns a completion guard; terminal `finish` waits for both readers before draining the queue, while normal polling remains bounded and non-blocking. | `ReaderCompletion`, `ReaderCompletionGuard`, and the source contract test cover the two-reader barrier. | implemented |
| Failure/quiescence boundary | A failed process-tree retirement still leaves blocking OS reads pending on the dedicated owner; the existing cleanup state retains the child for retry, while drop only releases the pump consumer. A future ProcessSupervisor must add an explicit pipe-close/quarantine barrier before claiming full shutdown ownership. | `child.rs` keeps the retry owner and does not weaken the ownership scanner; this record does not claim the parent Editor09 shutdown P0. | pending follow-up |

## Validation boundary

Rustfmt and scoped whitespace checks pass for the output owner change. The
post-change Editor package check/test was submitted as one managed batch in
execution session `18198`; its terminal row was wrapper exit `1` without a
retained Cargo diagnostic. A follow-up verbose-output package batch was
submitted in execution session `35163`, but its coordinator row became
`orphaned` before start with no Cargo run or terminal diagnostic. Neither row
provides a test or performance result. Release timing, reader-capacity
behavior under repeated failed tree retirement, and product shutdown evidence
remain pending. No tooling migration was made.

A replacement grouped current-source Editor request was submitted as execution
session `28867` with verbose output and one test thread. It remains
intentionally unpolled, so this record still does not claim a Cargo test,
Release timing, or shutdown-performance result.

The local non-Cargo Editor09 contract batch passed `68/68`; the Play output
drain capacity contract passed `3/3`, and the grouped compile-host owner
contracts passed `26/26`. These are source/ownership checks only and do not
replace the pending managed Rust receipt.

The corresponding grouped Windows Release request was submitted as execution
session `85283` with the ignored `editor09` filter. Its timing, reader-capacity,
and shutdown thresholds remain pending and are not inferred from submission.

The grouped development request later materialized as job
`cd1f8d75566a41448caae9e7b0be3d15` and closed `released` with wrapper exit `1`.
No `cargo_job_runs` row or Cargo diagnostic was retained, so the failure stage
is unknown and this record cannot claim managed test or performance evidence.
The Release request had not materialized a job row in the same bounded ledger
read. No polling loop or isolated retry was launched.

Static review then found a compile-time conversion gap in the TaskPool failure
branch: `TaskPoolBuildError` is now converted with `to_string()` before it is
stored in `PlayOutputCaptureError`. The source contract asserts this boundary;
rustfmt and scoped diff checks pass. The earlier Editor request `75077` was
submitted before this repair and is not attributed to it. Repaired-source
development session `45852` and grouped `editor09` Release session `94946`
were submitted together and remain intentionally unpolled; no Cargo or timing
receipt is claimed yet.

After submission, the reader-owner contract was narrowed to the production
source slice to prevent self-matching fixture text. This is test-only
hardening; the managed requests above still do not provide a receipt for it.

In the later bounded ledger read, repaired-source Editor development job
`6da3d44df7ff4c4686873a43c01a2f2b` and grouped `editor09` Release job
`d67f77792892406cb5a6fc3e06bd4710` were still `running` without a
`cargo_job_runs` row. No managed test, timing, or shutdown-performance result
is inferred, and no polling loop was started.

The next one-time ledger read found no materialized `cargo_jobs` row for the
repaired-source development identifier. The Release job
`d67f77792892406cb5a6fc3e06bd4710` was `released` with wrapper exit `1`, but
the coordinator retained no Cargo run row or diagnostic. The source conversion
repair and reader-owner guard therefore remain pending current-source Cargo,
Release timing, and process-quiescence evidence.

### Fresh grouped validator launch after source-guard stabilization (2026-09-25)

The current Editor source was submitted again through one package-wide managed
library check/test launch (`-LibTests -TestThreads 1 -VerboseOutput`) in the
same wave as Runtime. The wrapper was intentionally left unpolled, so this
entry claims no Cargo compilation, test, Release timing, or process-quiescence
result. A matching `editor09` Release wrapper was then launched separately and
left unpolled; its Release threshold remains pending.

### Bounded terminal reconciliation after the grouped wave (2026-09-25)

The grouped current-source Editor job `10640bced851482884b7c781967043fc`
released with wrapper exit `1`, while the matching `editor09` Release job
`523a586ee22848ecb3e7bc7c61dd1a2c` was stopped by the coordinator's
five-minute health timeout. No `cargo_job_runs` row or compiler diagnostic was
retained for either lane; the Play reader completion barrier therefore remains
source- and static-contract validated only, with managed Rust, Release timing,
and process-quiescence evidence still pending.
