# Runtime 03 shutdown-return observation completion

**Status:** One fixture-only source change is frozen, peer-reviewed, and statically guarded. The two Native regressions and every performance acceptance gate remain pending.

- **Task:** optimize-20261013-01a13451
- **Original task base:** `5a547d4e2727d27ddf99578f548efb7e6d19d915`
- **Batch09 source postimage:** `af0b48b1ce84d19b140d2e0aaf744a4999b78f5b542a27c3289d380783d6073c`
- **Incremental basis:** observed HEAD `1de7f1af1f86b124645acdb2743bfe9064f6da8b` (not attributed to this task)
- **After source:** `a524d887dde2ec18d77e66fa39164eccd34692a17ffa1a5c9bf18c13395b1407`
- **Date:** 2026-10-13

## Source complete

- [x] Change only the two shutdown fixtures in [config manager tests](../../../../../zircon_runtime/src/foundation/runtime/tests/config_manager_tests.rs); preserve their ordinary test names and production worker behavior.
- [x] Capture the writer completion `bool` on the Drop thread immediately after `drop(manager)` returns, send and return that same saved value, then let the parent release and join before asserting. Parent cleanup can no longer rewrite the captured result.
- [x] Preserve both contracts: cancellation before admission rejects stale staged JSON and removes staging; an admitted commit remains irrevocable while replacement activation fails fast.
- [x] Preserve the finite 100 ms observation window and document its scheduler-starvation limit. It is not a performance budget and is not deterministic proof against arbitrary starvation.
- [x] Preserve the worker's real join, public APIs, timeouts, performance producers, and all original thresholds.
- [x] Author-reported exact-file rustfmt and whitespace checks passed; independent review found no blocking fixture-ordering or deadlock issue.

The [source record](../../../optimize/zircon_runtime/03/2026-10-13-shutdown-return-observation-record.md) documents the two regression selectors and packet identity. Its patch is SHA-256 `0d10ba69bd8608b75c86c7a5c31ade45faf9961daebaa75bea583bcfe1b77f4c`; its evidence is SHA-256 `039723352892fb5995816e3b1cc9005cf9aa9661f1a970d07e64182b69aca8d5`.

## Static guard

- [x] The [scoped Runtime guard](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-shutdown-observation-guard.json), SHA-256 `9a75e9e96789677d5b4c7b6a3771e33939ee63df71d57e6392e4a86c9bc303e5`, reports `passed`, 0 errors, and 0 warnings for the one-file patch.

Static review and guard results do not establish compilation or Native correctness.

## Validation and acceptance pending

- [ ] Run `foundation::runtime::config_manager_tests::shutdown_waits_for_a_late_writer_before_returning` and `foundation::runtime::config_manager_tests::replacement_activation_fails_fast_while_an_admitted_commit_is_still_running` under the existing `#2` Native arm.
- [ ] Seal the current full source and execution inputs in batch10, including `config_manager_tests.rs` SHA-256 `a524d887dde2ec18d77e66fa39164eccd34692a17ffa1a5c9bf18c13395b1407`. Batch09's seven-file packet and file handoff do not bind or accept this later source.
- [ ] Obtain actual batch10 queue and Native receipts. Historical batch09 writer success and report copies do not establish queueing; a later normal link check found its canonical `.jenkins/records` paths missing again for unknown reasons. Retain that history and reconcile any unstarted prior request before queue submission.
- [ ] Complete the unchanged original 60-sample Config matrix, 70% / 75% helper gates, 45 / 36×3 / 60 matrix, and the user-authorized absolute P95 limits in the [criteria record](../../../optimize/zircon_runtime/03/2026-10-13-config-persistence-absolute-p95-budget-record.md). This fixture-only change changes none of those gates.
- [ ] Complete product, Jenkins, and whole-milestone acceptance.

The saved completion result is captured after `drop(manager)` returns and before the parent performs cleanup, but the finite observation window still has the documented scheduling caveat. The source and guard cover only this fixture slice; they do not alter the batch09 source identities or establish test/performance acceptance.
