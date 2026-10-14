# Runtime 44 lazy-format counter and critical-test cleanup record

**Status:** Four-file test/fixture source repair is complete and the scoped static guard passed. Native tests, the original performance evidence, product acceptance, and combined milestone acceptance remain pending.

## Defect and correction

The log-storm report required `formatted == attempted`. That rejected valid best-effort drops: `SinkRuntime::enqueue_lazy` checks whether a best-effort queue is already full before invoking the formatter, so such attempts are attempted but not formatted. A second valid case occurs when a formatter starts while capacity remains, then another producer fills the queue before `try_send`; the first formatted record is then dropped.

The accounting predicate is now `written <= formatted <= attempted`. For accepted records `A`, formatted records dropped after a race `B`, and requests rejected before formatting `C`, `attempted = A + B + C`, `formatted = A + B`, and `written = A`. The old equality was therefore incorrect. The rest of the conservation checks remain exact: `dequeued + dropped_debug == attempted`, `written == dequeued`, validated unique records equal written records, and the queue-drain, output-thread, duplicate, malformed, output-error, and write/flush/sync checks remain in place. This correction changes an assertion; it is not accurate to say every assertion stayed unchanged.

The real call path exercised by the added ordinary regressions is `DiagnosticLogState::write_lazy` → open/filter check → `SinkRuntime::enqueue_lazy` → bounded queue admission. No production logger, state, worker, setting, or no-sink behavior was changed.

## Ordinary regressions and cleanup

Two ordinary cases were added in `zircon_runtime/src/diagnostic_log/sink/tests/backpressure.rs`:

- `diagnostic_log::sink::tests::backpressure::full_state_queue_skips_best_effort_lazy_formatting` holds the real writer on the first record, fills the one-record queue with the second, then submits a third Debug record. It observes 3 attempted, 2 formatted, 2 written, and 1 dropped. It captures the held snapshot, releases the writer, performs real sink shutdown/join, then checks the counts and exact output lines.
- `diagnostic_log::sink::tests::backpressure::state_lazy_formatting_can_lose_a_concurrent_queue_admission` holds the first output, starts a real State formatter while the queue is empty, pauses that formatter on a channel, fills the queue with a winning producer, then resumes the paused formatter. It observes 3 attempted, 3 formatted, 2 written, and 1 dropped. The test saves the producer result while the writer remains held, then releases the writer, joins the producer, performs real sink shutdown/join, and checks the saved results, counts, and exact output lines.

The critical companion and the ignored 20-sample probe now defer setup, receive, admission, and join assertions until after their cleanup path. They release the real `BlockingOutput`, join producer threads, and call the real sink shutdown before evaluating saved results. `BlockingOutput::wait_until_blocked` retains its two-second timeout and panic message but drops its mutex guard before panicking. Callers capture that wait result and rethrow it only after cleanup; they call `release()` directly, so catching a setup panic does not stand in for unblocking the writer. Cleanup does not claim recovery from process abort, arbitrary worker deadlock, or unrelated mutex poisoning.

The two-second fixture wait and 250 ms companion receive windows are bounded observations for safe test cleanup. They are not performance budgets.

## Preserved performance contract

The ignored `PERF-MVP-434` matrix still has the original 54 rows: 3 requested rates × 2 caller counts × 3 scoped-rule counts × 3 sink delays. Its row fields, one critical companion, queue capacity 4096, batch size 256, 25 ms flush interval, 1–1.25 second load window, caller P95 limits, and Windows RSS requirements remain unchanged. The existing 128 MiB sampled peak-growth limit and the requirement for Windows baseline/peak/after RSS samples also remain unchanged.

The ignored `critical_admission_timeout_release_benchmark_evidence` still emits 20 alternating Warn/Error samples. The configured admission timeout remains 2 ms and its original caller P95 limit remains 50 ms. The probe does not establish a new aggregate latency, maximum, or P99 budget. The historical 10 ms critical companion observation and marker are also unchanged. No-sink semantics are outside this patch.

## Source identity and retained evidence

The source packet is based on `1de7f1af1f86b124645acdb2743bfe9064f6da8b`; the original task base `5a547d4e2727d27ddf99578f548efb7e6d19d915` is background context only. During this work HEAD was observed at `d588b90c0e3ca36503ac6b4f2c5d2d96ba7acfef`. The packet records that concurrent HEAD movement separately; it does not attribute that commit to this repair.

The frozen source postimages are:

- `tests/performance/critical.rs` — `d31efc454b3af5a785fa7632e17f3c73ff1753b9f85546d48502b4b3fb012a52`
- `tests/performance/report.rs` — `812ba409dc201f6de9f647c05ee4f40cf6c3175e3fe8930faca33ca539f3de25`
- `tests/backpressure.rs` — `d79ee277cd3dab361cc309f93f84ec2c94d8ad8e4f051bb7d170db85c0874d20`
- `tests/fixtures.rs` — `33f7cd8cd8d1deb51369ce4c6d751a19e20e16eb1a2c71994b182dc96dad1561`

The [four-file patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime44-lazy-counter-and-cleanup.patch) is 14,260 bytes, SHA-256 `9977c2205ca1f3033ff7e4e5d0159e08e6606fa8fc432c7b18766e13048ce416`. The [final evidence packet](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime44-lazy-counter-and-cleanup-evidence.json) is 17,171 bytes, SHA-256 `4a61730cf351829b81eb58380f003136968c3f47cc01877f993dcd18c6451ac7`. Its pre-handoff metadata-only correction changed the combined ordinal-9 minimum from an earlier-phase 15 to the frozen current floor of 16; it did not change source, patch, or test selectors. The independent [scoped change guard](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime44-lazy-counter-and-cleanup-guard.json) is 1,131 bytes, SHA-256 `d70261584269547b588d2962b44bb5b72f97e909e303aa4b40625548d376569a`; it passed with zero errors, warnings, and suppressions.

## Validation still pending

Exact-file `rustfmt --check`, scoped `git diff --check`, source-hash binding, and the four-path static guard passed. No Cargo or native test ran for this packet, and no performance or RSS result was measured. The two new ordinary filters and the existing 54-row and 20-sample ignored gates remain to be run through the managed validation batch. The combined ordinal-2 ordinary selector floor is 66; the ordinal-9 ignored selector floor is 16. These are selection floors, not run results. Product and broader runtime acceptance remain open.

- [ ] In batch10, validate both original RSS gates independently for every one of the 54 raw matrix rows: absolute process `rss_peak <= 134217728` bytes and the existing sampled `peak_growth <= 134217728` bytes. The current `report.rs` checks peak growth but does not enforce the absolute peak cap, so growth cannot substitute for the absolute check. Preserve each row's baseline/peak/after values and `sample_count`; missing required samples do not pass.

The matching [Runtime 44 completion list](../../../astra/features/runtime/2026-10-13-runtime44-lazy-format-and-critical-cleanup-completion-list.md) tracks the source repair separately from those pending gates.
