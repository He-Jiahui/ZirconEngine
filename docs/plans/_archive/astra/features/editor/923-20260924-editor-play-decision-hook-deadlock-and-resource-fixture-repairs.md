---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/33-localization-string-table-culture-translation-import-export-fallback-pseudo-localization-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/03/2026-08-24-history-journal-binary-lookup.md
  - docs/plans/optimize/zircon_editor/05/2026-08-26-inspector-customization-hash-admission.md
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
  - docs/plans/optimize/zircon_editor/48/2026-08-26-inbox-eviction-plan-allocation.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
related_records:
  - docs/plans/astra/features/editor/922-20260924-editor-v29-compile-contract-repairs.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Editor923 · Play Decision Hook Deadlock and Resource Fixture Repairs

## 计划完成列表

| Scope | Completed change | Evidence and remaining gate |
| --- | --- | --- |
| Pending Play decision publication | Copy the test-only hook out of its configuration mutex before invoking it; preserve the existing state lock and the two-publisher single-decision assertion. | The sealed v30 Editor library-test output reports `concurrent_prompt_publication_keeps_one_pending_decision` still running after 60 seconds and did not write a batch terminal receipt after more than two hours. Static inspection found the `if let` scrutinee's mutex guard lived into the hook invocation; the first publisher waited on a two-party barrier while holding the mutex needed by the second. A deterministic regression checks `try_lock()` inside the hook. Current-source Rust execution still needs independent evidence. |
| i18n test hook support | Release each configuration mutex after copying its hook and before executing the callback for the event-dispatch, failure-locale-read and locale-capture hooks. | All three independent hooks shared the same `if let` temporary-guard lifetime; a test checks each lock with `try_lock()` inside its own callback. No production locale/event behavior changed; current-source Rust tests pending. |
| Editor asset/resource fixtures | Require both `ResourceManager::register_record` calls to succeed before using a generated management snapshot. | Previously both `Result` values were silently dropped, so invalid fixture records could yield misleading downstream failures. Existing snapshot/resource-access assertions stay unchanged; current-source Rust tests pending. |
| History and operation-group source guards | Limit the operation-group clone prohibition to production source, and select `HistoryStore::journal` after its owning `impl` rather than the earlier `TransactionRecord::journal`. | Both targeted v30 tests reported `FAILED`. The former asserted the absence of a literal that appeared in its own test; the latter selected the first of two same-named methods, missing the actual logarithmic lookup. Existing `transaction_record_by_id`/binary-search/no-clone assertions remain; current-source Rust tests pending. The preexisting foreign edit to the operation-group production method was not modified. |
| Inspector05, Jobs09 and Inbox48 source guards | Read the actual `PendingAdmissionLedger` after the Jobs admission-index extraction; inspect the Inspector vector matching method independent of rustfmt line wrapping; check each Inbox ordered-index `pop_first` inside its own removal method rather than assuming a one-line member call. | All three targeted tests reported `FAILED` in v30 and their original substring checks fail against the corresponding current production sources. Hash-backed IDs, insertion-order matching, category-index projection, count-only eviction and no allocating plan remain asserted; production code and ignored Release thresholds were not changed. Rust execution is still pending. |
| Logging test hook support | Clone the before-emission, after-store and before-dispatch test callbacks out of their individual configuration locks before invoking them. Add a regression that tries each configuration lock from inside its callback. | All three hooks retained the `if let` mutex guard across callback invocation, matching the Play/i18n support defect. The new test was written first and the three locked scrutinees were confirmed still present before repair; Rust execution is pending. The existing foreign `snapshot_tail_if_changed` import/method and log-tail tests were preserved, not replaced. This test-only change does not close Editor11's production logging P0 or its p99 gate. |
| Reentrant logging test fixture | Keep the sink's once-only reentry latch set after its nested log emit, because the nested record is queued and dispatched only after the callback returns. | The broad Editor v33 library run passed the hook-lock regression, then stopped making progress at the reentrant sink test; its verified library-test process consumed over twelve CPU minutes. Source tracing shows the fixture reset the latch before dispatching its queued record, causing each record to enqueue another forever. The existing regression still requires exactly two records; no production sink or dispatch behavior was changed. This post-seal fixture repair needs the next grouped Rust run. |

The v30 managed Editor source was sealed **before** these fixes and cannot
validate them. Its Runtime companion was rejected at source closure with
`compile_input_changed`. Do not claim that the hanging v30 process passes
Editor tests, or interpret local Rust formatting as Release / allocator /
product p50/p95/p99 evidence. Keep Editor207 grid storage and the unrelated
ultra-width layout test open until their conflicting owners converge. The
current focused Editor Jobs check/test batch and subsequent multi-behavior
regression wave are tracked in Runtime696; the later logging hook edit was
made after that Jobs batch sealed its inputs and requires its own later
current-source validation. There is no per-fixture Cargo retry or tooling
implementation in this change.
Exact-file `rustfmt --edition 2021 --check` and scoped `git diff --check`
passed for all ten modified Editor Rust files; these source checks do not
replace the pending managed test or performance runs.

The frozen v30 Editor library test made no progress after the Play decision
hook blocked both publishers. At `2026-09-24T23:30:19+08:00`, after verifying
the exact PID, parent Cargo PID, and managed D-drive executable path, this
session stopped only its own hung Editor library-test process (`30496`) so
the existing Cargo invocation could fail and the managed wrapper could
finalize normally. No other Cargo run or test binary was stopped.
The v30 terminal receipt arrived at `2026-09-24T23:30:43+08:00`:
Editor sealed-source Cargo library check `OK`, library test `FAIL` with
abnormal exit `0xffffffff`, and `compile_workspace_modified` reported by
the managed wrapper for its sealed input copy; the exact writer is unknown.
The output was interrupted before the harness summary;
its 1,286 `... FAILED` lines cannot be attributed or accepted as a complete
test inventory. The newly edited hook code was not in the sealed source.
Future current-source validation must either use a snapshot-isolated
coordinator ticket or wait for direct source-closure sealing before making
further live-input edits. Post-seal edits do not change that sealed copy and
need their own subsequent batch; no current Rust pass is claimed.
The next focused managed Editor attempt selected multiple optimization tests
but exited at `cargo_reuse_pool_busy` before check or test because another
session owned the compatible pool; see Runtime696. This is an admission
failure, not test evidence, and is not a reason to retry in parallel.
An independent, read-only five-module Editor source-contract batch passed
`22/22` (interactive save adapter, i18n fallback, Inspector customization,
history lookup and Play session controller). It does not execute Rust tests,
resolve the conflicting shared-grid contracts, or provide Release timings.

The logging hook repair changed only those three test-only methods and added
the private lock-lifetime regression. Its foreign log-tail projection work
was present before and remains intact. Because the active Jobs snapshot was
sealed before the logging edit, neither its check nor its focused test filter
can qualify this new regression; do not narrow or drop the existing full
concurrency assertions to get a green run.
