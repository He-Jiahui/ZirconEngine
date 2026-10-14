# Runtime 03 shutdown-return observation fixture source record

**Status:** The fixture-only source change is frozen, independently reviewed, and passed its scoped static guard. Native execution is pending. The change does not alter production shutdown behavior, performance producers, or acceptance thresholds.

- **Task:** optimize-20261013-01a13451
- **Original task base:** `5a547d4e2727d27ddf99578f548efb7e6d19d915`
- **Batch09 source postimage:** `af0b48b1ce84d19b140d2e0aaf744a4999b78f5b542a27c3289d380783d6073c`
- **Incremental patch basis:** observed checkout HEAD `1de7f1af1f86b124645acdb2743bfe9064f6da8b`; this is distinct from the original task base and is not attributed to this task.
- **After source:** `a524d887dde2ec18d77e66fa39164eccd34692a17ffa1a5c9bf18c13395b1407` (31,935 bytes)
- **Date:** 2026-10-13

## Source and behavior

Only [config manager tests](../../../../../zircon_runtime/src/foundation/runtime/tests/config_manager_tests.rs) changed. The two affected ordinary tests remain:

- `foundation::runtime::config_manager_tests::shutdown_waits_for_a_late_writer_before_returning`
- `foundation::runtime::config_manager_tests::replacement_activation_fails_fast_while_an_admitted_commit_is_still_running`

Each Drop thread now saves `writer.is_completed()` immediately after `drop(manager)` returns, sends that saved `bool` over the completion channel, and returns the same value as its thread result. The parent observes the channel while the writer remains held, releases the writer, joins the Drop thread, and only then checks the saved result. Parent cleanup can no longer change a previously captured `false` into `true`, which was possible when the parent read the live writer flag after release and join.

In the first fixture, the parent releases and joins before checking cancellation, captured completion, and rejected staging; it removes the temporary directory before those assertions. In the admitted-commit fixture, the parent releases and joins before checking the captured completion, preserves the already-admitted stale write, verifies fast-fail activation, and lets a replacement manager publish fresh data before the final saved-result assertion. Failed held-writer observations do not leave a worker blocked: both fixtures release the writer and join before assertions.

The test writer sets its `completed` flag after the commit operation returns and before its `write` method returns. The production worker's Drop path is unchanged and still joins its worker thread unconditionally at `worker.rs:333`. No production source path, public API, timeout, ordinary test name, or `config_original_matrix.rs` producer changed.

## Observation limit

The fixtures retain `TEST_JOIN_OBSERVATION_TIMEOUT = 100 ms` while the writer is held. This is a finite scheduling observation window, not a performance budget and not a proof under arbitrary scheduler starvation. The Drop thread can be preempted after `drop(manager)` returns but before it reads `is_completed()`. If it remains unscheduled past the observation window, the parent may release the writer, the worker may complete, and the later getter could capture `true` even if a hypothetical Drop omitted the join. This limitation is explicit in the evidence; the test does not claim a deterministic skip-join proof for that extreme schedule.

The previous 2 s / 30 s waits, the existing 100 ms window, and the separate activation fast-fail check are lifecycle/test controls. They do not define or replace a performance gate.

## Review and static checks

The [incremental patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shutdown-observation.patch) is 4,069 bytes, SHA-256 `0d10ba69bd8608b75c86c7a5c31ade45faf9961daebaa75bea583bcfe1b77f4c`. The [source evidence](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shutdown-observation-evidence.json) is 9,714 bytes, SHA-256 `039723352892fb5995816e3b1cc9005cf9aa9661f1a970d07e64182b69aca8d5`. The [scoped guard receipt](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shutdown-observation-guard.json) is 705 bytes, SHA-256 `9a75e9e96789677d5b4c7b6a3771e33939ee63df71d57e6392e4a86c9bc303e5`; it reports `passed`, 0 errors, and 0 warnings, bound to the one-file patch.

The author-reported exact-file `rustfmt --check` and `git diff --check` passed. Independent read-only review found no blocking fixture-ordering, saved-value, or deadlock issue. No Rust build, type-check, or Native test was run locally.

## Validation and acceptance

The two selectors above remain pending under the existing `#2` Native test arm. Batch09's source hash `af0b...6073c`, seven-file patch, evidence, and guard do not cover or accept this postimage. The next batch10 must seal `config_manager_tests.rs` at `a524...b1407` with its full current execution inputs; the batch10 handoff and queue/native receipts are pending.

Batch09's report copies remain available as historical file-handoff evidence: [JSON](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch09-handoff.json) and [Markdown](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch09-handoff.md). A later normal link check found the canonical `.jenkins/records` paths missing again for unknown reasons. Historical writer success and matching copies do not establish current queueing or validation. Batch10 should retain the older history, record its current source seals, and reconcile any unstarted prior request before queue submission.

The original 60-sample Config producer, the 70% / 75% helper gates, the 45 / 36×3 / 60 matrix, and the user-authorized absolute P95 caps are unchanged. Their measurements and all Native, product, Jenkins, and milestone acceptance remain pending. See the [absolute P95 criteria](2026-10-13-config-persistence-absolute-p95-budget-record.md) and the [Runtime 03 batch09 source record](2026-10-13-shared-config-persistence-snapshot-record.md).

The matching [shutdown-return completion list](../../../astra/features/runtime/2026-10-13-runtime03-shutdown-return-observation-completion-list.md) tracks this source slice and its pending acceptance separately.
