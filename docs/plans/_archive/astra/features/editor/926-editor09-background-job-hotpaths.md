---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
related_records:
  - docs/plans/astra/features/editor/923-20260924-editor-play-decision-hook-deadlock-and-resource-fixture-repairs.md
  - docs/plans/astra/features/editor/924-20260925-editor-admission-byte-fixtures-and-commandlet-path-contract.md
  - docs/plans/astra/features/editor/925-20260925-editor-compile-host-file-backed-output-owner.md
  - docs/plans/astra/features/editor/928-20260925-editor09-completion-observer-barrier.md
  - docs/plans/astra/features/editor/929-20260925-editor09-job-join-capability-boundary.md
  - docs/plans/astra/features/editor/930-editor09-promotion-dependency-capacity.md
  - docs/plans/astra/features/editor/931-editor09-reservation-preflight-borrowed-requests.md
  - docs/plans/astra/features/editor/932-editor09-enqueue-dependency-capacity.md
  - docs/plans/astra/features/editor/933-editor09-progress-projection-capacity.md
  - docs/plans/astra/features/editor/934-editor09-event-journal-gap-capacity.md
  - docs/plans/astra/features/editor/935-editor09-scheduling-dependency-capacity.md
  - docs/plans/astra/features/editor/936-editor09-submission-preflight-borrowed-batch.md
  - docs/plans/astra/features/editor/937-editor09-job-record-hash-index.md
  - docs/plans/astra/features/editor/938-editor09-export-wizard-session-hash-index.md
  - docs/plans/astra/features/editor/939-editor09-export-stage-diagnostic-hash-merge.md
  - docs/plans/astra/features/editor/940-editor09-release-all-reservations-capacity.md
  - docs/plans/astra/features/editor/927-20260925-editor09-play-output-task-owner.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/commands/descriptor.rs
tests:
  - zircon_editor/src/core/jobs
---

# Editor926 Editor09 Background Job Hotpaths

## 完成列表

| Scope | Optimization | Evidence and remaining gate |
| --- | --- | --- |
| Bounded job journal | Lifecycle entries are bounded by count/bytes/age; progress updates coalesce by `JobId`, and dropped lifecycle ranges become typed gap records. | Existing Jobs source/tests cover the bounded journal and backpressure restoration; the current grouped Editor batch must still execute the repaired fixture set. |
| Progress projection | Labels/messages are shared and primary-job selection uses a maintained priority index rather than a full scan for each update. | Source and focused tests cover index maintenance and snapshot sharing; release high-water/latency measurements remain pending. |
| Admission and observer paths | Ready-priority accounting skips empty categories, observer callbacks receive transferred batches outside the dispatch lock, and oversized admission identities fail without retaining caller input. | Static contracts and focused tests exist; whole-package managed check/test evidence is pending in Editor execution session `22406`. |
| Wait/dependency hotpaths | Timed ticket waits release the mutex while waiting and restore receiver ownership after timeout; dependency IDs remain sorted and deduplicated by binary search. | Focused tests are included in the package library batch; no isolated retry is recorded. |
| Process-output ownership boundary | CompileHost output is file-backed in Editor925; Editor927 moves Play's two reader owners onto a dedicated runtime task pool with an explicit completion barrier. | The thread-ownership guard remains enabled. Failed process-tree retirement still needs a ProcessSupervisor close/quarantine barrier, so this record does not claim Editor09 shutdown/process ownership closure. |

## Validation boundary

Rustfmt and scoped whitespace checks for the current Editor slices are
available, but the latest grouped development request was reconciled as
coordinator job `8a33ecbbe3ce49e496698f39c45ff019`, `released` with wrapper exit
`1`. Its target was retained but no `cargo_job_runs` diagnostic was retained,
so the exit cannot be classified as a compiler, test, or harness failure. A
fresh grouped Editor request must validate the repaired fixture set.
Development test results, ignored Windows release evidence, and product
p50/p95/p99 gates remain pending; no performance acceptance is inferred from
source structure or dry-run selection.

A fresh post-failure package-wide request was submitted in execution session
`89878` before the Editor927 source edit, so it cannot validate the new Play
owner. A follow-up package-wide request must include Editor927; its managed
terminal receipt remains pending and no per-test retry was launched.

The release dry-run selected one `cargo check --release --tests` plus one
ignored `editor09` test launch with one test thread. This is only command
selection. The current development Editor batch must be reconciled first; the
release lane was instead queued in parallel in execution session `74312` to
avoid blocking on the development batch. Its source closure and actual
elapsed/threshold output remain independent pending evidence.

The local non-Cargo Editor09 source-contract batch passes `68/68`; the Play
output capacity contract passes `3/3`, and the grouped compile-host ownership
checks pass `26/26`. These receipts do not establish managed Rust compilation,
test, or Release performance acceptance.

### Bounded grouped ledger reconciliation (2026-09-25)

The fresh grouped Editor development job `cd1f8d75566a41448caae9e7b0be3d15`
closed as `released` with wrapper exit `1`; its target was retained, but the
coordinator retained no `cargo_job_runs` row or Cargo diagnostic. This is a
stage-unknown validator failure, not a compiler/test classification and not a
pass. The submitted Editor Release session `85283` had no materialized
`cargo_jobs` row in the same one-time read, so Release timing and shutdown
thresholds remain pending. No continuous monitoring or per-test retry was
started.

### Post-submission compile repair (2026-09-25)

Static review identified and repaired a type mismatch in Editor927's Play
output capture error path: `TaskPoolBuildError` is now explicitly converted to
the owned diagnostic string expected by `PlayOutputCaptureError`. Rustfmt and
scoped whitespace checks pass. The pre-repair request `75077` is not evidence
for this source. A fresh grouped Editor development request (`45852`) and a
grouped `editor09` Release request (`94946`) were submitted without per-test
retries; their managed results and performance thresholds remain pending.

The reader-owner source guard was subsequently narrowed to the production
slice so fixture strings cannot satisfy the ownership assertion themselves;
this test-only hardening does not replace the pending grouped receipt.

The same one-time ledger read found repaired-source Editor development job
`6da3d44df7ff4c4686873a43c01a2f2b` and grouped Release job
`d67f77792892406cb5a6fc3e06bd4710` still `running`, with no Cargo run receipt
yet. Their status is not treated as a test or performance result.

### One-time terminal reconciliation after independent review (2026-09-25)

The follow-up ledger read found no `cargo_jobs` row for repaired-source
development job `6da3d44df7ff4c4686873a43c01a2f2b` in the queried interval. The
grouped Release job `d67f77792892406cb5a6fc3e06bd4710` had become `released`
with wrapper exit `1`, but still had no `cargo_job_runs` row or Cargo
diagnostic. Both lanes therefore remain without a compiler/test/performance
receipt; no stage is inferred and no isolated retry was launched.

### Fresh grouped validator launch after source stabilization (2026-09-25)

After the `TaskPoolBuildError` conversion and production-only reader-owner
guard were stabilized, a new managed `validate-matrix.ps1 -Package
zircon_editor -LibTests -TestThreads 1 -VerboseOutput` development lane was
launched in the same wave as Runtime. The wrapper was left running without
polling; no Cargo job id, compiler output, test count, or performance result is
claimed from launch. A matching `editor09` Release wrapper was then launched
separately; it was also left unpolled, so no Release timing or threshold receipt
exists yet.

### Bounded terminal reconciliation after the grouped wave (2026-09-25)

The current-source Editor development job `10640bced851482884b7c781967043fc`
and the Editor09 Release job `523a586ee22848ecb3e7bc7c61dd1a2c` both released
with wrapper exit `1`; the Release lane emitted a coordinator
`cargo.health_timeout` before cleanup. Neither lane retained a
`cargo_job_runs` row or Cargo diagnostic, so compiler/test counts, reader-owner
behavior, and Release p50/p95 thresholds remain unverified. This bounded read
does not classify the wrapper exit as a code failure or a pass, and no
continuous monitoring was started.

### Fresh four-lane submission after Editor928 (2026-09-25)

After the Editor928 completion-barrier change, fresh grouped managed wrappers
were submitted without polling: Editor development (`-Package zircon_editor
-LibTests -TestThreads 1 -VerboseOutput`, PTY `16310`) and Editor09 Release
(`-CargoProfile release -TestFilter editor09 -IgnoredTests`, PTY `37590`).
Runtime development (PTY `76534`) and Runtime02 Release (PTY `34527`) were
submitted in the same batch so the shared source wave can be validated
together. PTY launch output contained only terminal-control setup; no Cargo
job, compiler/test count, shutdown result, or Release threshold is claimed.
The subsequent Editor928 cancellation-barrier extension landed after this
submission, so these receipts are not attributed to that final source state.

The current-source resubmission uses Runtime development PTY `92163`, Editor
development PTY `57512`, Runtime02 Release PTY `65557`, and Editor09 Release
PTY `88159`; all four wrappers remain intentionally unpolled.

After Editor929, the latest grouped current-source submission uses Runtime
development PTY `22890`, Editor development PTY `85495`, Runtime02 Release PTY
`45349`, and Editor09 Release PTY `34355`; these wrappers also remain
intentionally unpolled.

Editor930 adds a source-only promotion capacity optimization after that wave;
the latest four-lane receipts are therefore stale for current-source validation.

The fresh current-source batch submitted after Editor930 uses Runtime development
PTY `2746`, Editor development PTY `6182`, Runtime02 Release PTY `32549`, and
Editor09 Release PTY `34609`; all wrappers remain intentionally unpolled.

Editor931 removes the first-preflight request-reference vector after that
submission, so those PTYs are stale for the current source. A fresh four-lane
batch was submitted after Editor931: Runtime development PTY `81169`, Editor
development PTY `98412`, Runtime02 Release PTY `26311`, and Editor09 Release
PTY `68990`. The wrappers are intentionally left unpolled; no Cargo, test, or
performance receipt is inferred from their launch.

Editor932 then changed `enqueue_pending` to reserve the known dependency upper
bound before filtering. Those Editor931 receipts are stale for the current
source. A new four-lane batch was submitted after Editor932: Runtime development
PTY `81935`, Editor development PTY `42136`, Runtime02 Release PTY `99713`, and
Editor09 Release PTY `4102`. The wrappers remain intentionally unpolled; this
submission supplies no compiler, test-count, or Release performance receipt by
itself.

Editor933 then added known-capacity reservations to the progress projections,
so the Editor933-current batch supersedes the Runtime884 wave for Editor source
validation. Its four PTYs are recorded in the central log and Editor933; all
wrappers remain intentionally unpolled.

Editor934 then reserved the covered event-range capacity during gap merging in
the bounded journal. The Editor933/Runtime885 wave is stale for the current
Editor source; the replacement four-lane PTYs are recorded in the central log
and Editor934. All wrappers remain intentionally unpolled.

Editor935 then reserved the explicit dependency plus optional mutex-tail
capacity in `scheduling_dependencies`. The Editor934 four-lane receipt is stale
for this current Editor source; the next replacement batch is recorded in the
central log and Editor935. All prior wrappers remain intentionally unpolled.

Editor936 then removed the `submit_batch` preflight reference-vector allocation
by forwarding a cloneable exact-size borrowed iterator through the admission
layers. The Editor935 receipt is stale for the current Editor source; the next
replacement batch is recorded in the central log and Editor936. All prior
wrappers remain intentionally unpolled.

Editor940 then changed shutdown reservation release to consume the reservations
map directly, removing the temporary ID vector. The Runtime886 wave is stale
for this current Editor source; the replacement four-lane batch uses Runtime
development PTY `6829`, Editor development PTY `54360`, Runtime02 Release PTY
`97136`, and Editor09 Release PTY `24665`. All wrappers remain intentionally
unpolled, so no compiler, test-count, shutdown, or Release performance receipt
is inferred from launch.

The follow-up Editor940 regression now covers multiple reservation groups and
verifies that shutdown clears pending-entry and pending-byte accounting for every
group. A fresh current-source four-lane batch was submitted with Runtime
development PTY `27946`, Editor development PTY `98284`, Runtime02 Release PTY
`88515`, and Editor09 Release PTY `99265`; all wrappers remain intentionally
unpolled.
