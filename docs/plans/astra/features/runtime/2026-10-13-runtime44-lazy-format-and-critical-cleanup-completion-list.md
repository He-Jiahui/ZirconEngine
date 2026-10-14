# Runtime 44 lazy-format counter and critical-test cleanup completion

**Status:** Source and scoped static review are complete. Native tests, original performance evidence, product acceptance, and combined milestone acceptance remain pending.

## Source completion

- [x] Correct the log-storm counter predicate to `written <= formatted <= attempted`. Best-effort requests rejected by an already-full queue skip formatting; another request can also be dropped after formatting if a concurrent producer wins the remaining queue slot. The old `formatted == attempted` assertion was false for both valid cases.
- [x] Add two ordinary regressions using the real `DiagnosticLogState::write_lazy` → `SinkRuntime::enqueue_lazy` path and a real one-record queue with `BlockingOutput`.
- [x] Make each regression observe both queue states: 3 attempted / 2 formatted / 2 written / 1 dropped for early formatter skip; 3 attempted / 3 formatted / 2 written / 1 dropped for a concurrent admission loss.
- [x] Save producer and observation results, then release the writer, join producers, and perform real sink shutdown before assertions. Critical companion and 20-sample setup failures follow the same cleanup ordering.
- [x] Drop `BlockingOutput`'s mutex guard before its unchanged two-second timeout panic so cleanup can release the writer after a captured setup error.
- [x] Preserve the 54-row performance matrix, its original resource/caller/RSS/window checks, and the ignored 20-value critical admission probe with its 2 ms configured timeout and 50 ms caller P95 limit. No new aggregate latency cap was introduced.
- [x] Keep the change within four existing tests/fixture files; production logger paths and no-sink behavior were not changed.
- [x] Bind the four frozen postimages and patch to base `1de7f1af1f86b124645acdb2743bfe9064f6da8b`. The original task base `5a547d4e2727d27ddf99578f548efb7e6d19d915` remains context only. Observed HEAD `d588b90c0e3ca36503ac6b4f2c5d2d96ba7acfef` is recorded as concurrent foreign movement, not as this patch's source attribution.
- [x] Pass exact-file rustfmt, scoped diff whitespace validation, and the four-path static change guard. Guard result: passed, 0 errors, 0 warnings, 0 suppressions.

The complete reasoning and hashes are in the [source repair record](../../../optimize/zircon_runtime/44/2026-10-13-lazy-format-counter-and-critical-cleanup-record.md), [patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime44-lazy-counter-and-cleanup.patch), [final evidence](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime44-lazy-counter-and-cleanup-evidence.json), and [scoped guard receipt](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime44-lazy-counter-and-cleanup-guard.json).

## Acceptance still pending

- [ ] Run the ordinary filters `diagnostic_log::sink::tests::backpressure::full_state_queue_skips_best_effort_lazy_formatting` and `diagnostic_log::sink::tests::backpressure::state_lazy_formatting_can_lose_a_concurrent_queue_admission` in the managed batch. Source presence and static review are not test execution.
- [ ] Run the existing ignored `diagnostic_log::sink::tests::performance::perf_mvp_434_bounded_log_storm_matrix` with all 54 rows and its original resource, caller-latency, RSS, and load-window requirements.
- [ ] In batch10, check both RSS gates separately on all 54 raw rows: absolute process `rss_peak <= 134217728` bytes and the existing sampled `peak_growth <= 134217728` bytes. Current source asserts the growth limit but not the absolute peak limit; growth does not satisfy the absolute gate. Preserve baseline/peak/after and `sample_count`, and reject rows missing required samples.
- [ ] Run the existing ignored `diagnostic_log::sink::tests::backpressure::critical_admission_timeout_release_benchmark_evidence` and validate all 20 raw values against its unchanged 2 ms timeout and 50 ms P95 gate. The 20 samples do not create an aggregate latency, maximum, or P99 budget.
- [ ] Complete native/type-check, managed product validation, and the broader runtime acceptance gates.

The combined selection contract retains an ordinal-2 ordinary floor of 66 and ordinal-9 ignored floor of 16. Those are selector minima, not evidence that commands were queued, accepted, or executed. No Cargo/native tests, 54-row measurement, 20-sample measurement, or Jenkins status query was performed for this record.

The finite 2-second fixture wait and 250 ms companion observation windows exist only to report a stuck test path and preserve cleanup. The 1–1.25 second matrix load window, 50 ms caller P95 limit, 2 ms configured critical admission timeout, and original RSS contract remain the applicable performance criteria.
