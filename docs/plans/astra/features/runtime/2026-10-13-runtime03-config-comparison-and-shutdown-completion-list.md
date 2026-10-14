# Runtime 03 ConfigStore comparison and shutdown completion

**Status:** The bounded source slice is complete and reviewed. Native/type-check, original performance, product, Jenkins, and whole-milestone acceptance remain pending.

- **Task:** optimize-20261013-01a13451
- **Baseline:** 5a547d4e2727d27ddf99578f548efb7e6d19d915
- **Date:** 2026-10-13

## Source complete

- [x] Move deep `Value` comparison outside the map mutex; retain and recheck `Arc<Value>` identity across comparison; retry after concurrent replacement; retire the old owner after unlocking.
- [x] Add three ordinary concurrency regressions for changed and equal comparisons, concurrent raw replacement, and owner retention. Source presence is recorded; these tests have not run.
- [x] Add two shutdown fixtures for cancellation before commit admission and completion of a commit already admitted. While each writer is held, the fixture observes the Drop completion channel for a bounded 100 ms; after release it joins the Drop thread and immediately reads the writer completion flag.
- [x] Add an ignored Release-only raw probe with five equal/changed pairs at 1 byte, 1 MiB, and 100 MiB: 15 pairs total. The changed byte is last; the probe declares no absolute latency budget and has not run.
- [x] Author-reported rustfmt and whitespace checks passed; an independent read-only review found no production correctness blocker. See the [source repair record](../../../optimize/zircon_runtime/03/2026-10-13-config-comparison-and-shutdown-fixture-record.md).

The cumulative [r2 patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-core-r2.patch) has SHA-256 `fd7af6821d14cfe7104966c79ca7f51c6f0ca4415a67e7a862da66eccb71387b`; the [r2 evidence](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-core-r2-evidence.json) has SHA-256 `08b33dff67e66dad0ce2ca423b54ce1c7bc227b38e2a912493d7a748ffd54594`.

## Static guard

- [x] The [r2 runtime change-guard receipt](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-change-guard-r2.json) reports `passed`, 0 errors, and 1 warning. The warning is the newly added ignored Release probe, not an existing ordinary regression being skipped. Root independently verified the five source hashes against the r2 evidence. Include the ignored selector in the initial `#9` validation selection.

This static guard result does not establish Rust compilation, Native test execution, raw probe measurements, or performance acceptance.

## Acceptance pending

- [x] Write the batch08 JSON and Markdown, binding the reviewed source hashes and 15 command arms. Both paths were later absent during one file-evidence reconciliation; the [continuation log](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/continuation-log.md) preserves the original write hashes. The cause is unknown.
- [ ] Reconcile actual queue and native receipts. The later shared-persistence-snapshot repair changes overlapping Runtime source hashes and requires a cumulative follow-up batch; r2 evidence cannot accept those new postimages.
- [ ] Run the focused ordinary Rust regressions and obtain Native/type-check evidence through the authorized validation path.
- [ ] Execute the ignored Release selector and retain all raw samples. Its 100 ms observation window and the 2 s / 30 s wait or shutdown timeouts are not performance budgets.
- [ ] Complete the original Runtime 03 helper thresholds (70% / 75%) and original 45 / 36×3 / 60 matrix. These gates are unchanged and remain pending; the new raw probe does not satisfy them.
- [ ] Complete product, Jenkins, and whole-milestone acceptance.
- [ ] Reconcile Editor 07’s separate asynchronous handoff and receipt under its owner; this completion list does not close that work.

The 100 ms join observation is finite and remains a timing-based check: under extreme scheduler delay beyond the observation window, a skipped join could theoretically escape the in-window signal and race the final direct completion-flag read. The fixture is stronger than an immediate channel peek, but this limitation is recorded rather than presented as a performance threshold.

