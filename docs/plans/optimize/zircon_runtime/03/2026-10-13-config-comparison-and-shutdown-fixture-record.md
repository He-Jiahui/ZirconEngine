# Runtime 03 ConfigStore comparison and shutdown fixture source record

**Status:** Source changes are recorded. Rust type-check and Native execution, the original performance matrix, the ignored Release sample probe, product acceptance, Jenkins acceptance, and the Runtime 03 milestone remain pending.

- **Task:** optimize-20261013-01a13451
- **Baseline:** 5a547d4e2727d27ddf99578f548efb7e6d19d915
- **Date:** 2026-10-13

## Source scope

`ConfigStore::store_value_if_changed` now performs deep `Value` equality outside the values-map mutex. It retains the observed `Arc<Value>` while comparing, rechecks owner identity under the mutex before returning unchanged or publishing, and retries the comparison if a concurrent raw replacement changed the owner. Publication retires the replaced `Arc<Value>` after releasing the map guard. The public API, `if_changed` result semantics, and publication barrier remain intact.


The five-file source scope is [ConfigStore implementation](../../../../../zircon_runtime/src/core/runtime/config_store.rs), [ConfigStore tests](../../../../../zircon_runtime/src/core/runtime/tests/config_store.rs), [config manager](../../../../../zircon_runtime/src/foundation/runtime/config_manager.rs), [persistence worker](../../../../../zircon_runtime/src/foundation/runtime/config_manager/worker.rs), and [config manager tests](../../../../../zircon_runtime/src/foundation/runtime/tests/config_manager_tests.rs).

The source adds three ordinary concurrency regressions in [config_store tests](../../../../../zircon_runtime/src/core/runtime/tests/config_store.rs):

- `config_store_changed_comparison_rechecks_a_concurrent_raw_replacement`
- `config_store_equal_comparison_allows_reads_without_replacing_the_owner`
- `config_store_equal_comparison_rechecks_a_concurrent_raw_change`

The Config persistence fixtures cover two shutdown cases in [config manager tests](../../../../../zircon_runtime/src/foundation/runtime/tests/config_manager_tests.rs):

- `shutdown_waits_for_a_late_writer_before_returning` holds the writer after staging but before commit admission. Shutdown must cancel that unadmitted write, remove staging, leave no final file, and wait for the writer to finish.
- `replacement_activation_fails_fast_while_an_admitted_commit_is_still_running` blocks inside an admitted commit. Shutdown cannot revoke it; a new activation fails fast while it remains in progress, and the old write completes after release.

Both fixtures observe the Drop completion channel for 100 ms while the writer is held, then release the writer, join the Drop thread, and read the writer completion flag directly. The author’s rustfmt and whitespace checks passed; the independent read-only review found no production correctness blocker. The finite observation caveat is retained below.

## Release sample probe

The ignored Release-only selector `core::runtime::config_store::tests::runtime03_config_store_equal_and_changed_large_value_samples` is present in [config_store tests](../../../../../zircon_runtime/src/core/runtime/tests/config_store.rs). It emits five equal/changed sample pairs at each payload size: 1 byte, 1 MiB, and 100 MiB, for 15 pairs total. The changed value differs at its last byte. Caller-input allocation is outside the measured interval; retirement of the old value is inside the changed interval. The probe uses the production `store_value_if_changed` path and declares no numeric latency budget.

This is a raw probe definition only; it has not been executed and does not establish the original Runtime 03 matrix or product performance acceptance.

## Evidence and handoff

The cumulative [r2 source patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-core-r2.patch) is 20,982 bytes, SHA-256 `fd7af6821d14cfe7104966c79ca7f51c6f0ca4415a67e7a862da66eccb71387b`. The [r2 evidence](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-core-r2-evidence.json) is 12,589 bytes, SHA-256 `08b33dff67e66dad0ce2ca423b54ce1c7bc227b38e2a912493d7a748ffd54594`. These are the reviewed r2 artifacts; the earlier r1 patch is historical and is not a substitute for this cumulative package.

The [r2 runtime change-guard receipt](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-change-guard-r2.json) reports `passed`, 0 errors, and 1 warning. The warning is the newly added ignored Release probe, not an existing ordinary regression being skipped. Root independently verified the five source hashes against the r2 evidence. Include the ignored selector in the initial `#9` validation selection.

The [Jenkins batch08 file-handoff record](../../../../../.jenkins/records/runtime-editor-optimize-01a13451-20261013-batch08.json) has been written with the nine reviewed Runtime and Editor source hashes and the 15 command arms. Queue and native acceptance still require actual receipts. The subsequent shared-persistence-snapshot repair changes overlapping Runtime source hashes; r2 remains historical evidence for its exact source, and a cumulative follow-up batch must bind the new postimages.

## Validation and acceptance still pending

The 100 ms test observation window and 2 s / 30 s wait or shutdown timeouts are synchronization and lifecycle controls, not performance budgets. The 100 ms check is materially stronger than an immediate peek, but a finite window could theoretically miss a skipped join if the Drop thread were unscheduled for longer than the window and the worker then completed before the direct completion-flag read.

No Rust build, type-check, native test execution, ignored Release probe, original matrix, or product validation is recorded for this packet. The original helper thresholds of 70% / 75% and the original 45 / 36×3 / 60 matrix remain pending unchanged; the raw probe does not replace them. Jenkins and whole-milestone acceptance also remain pending.

Before closeout, reconcile the Jenkins handoff receipt, then record Native and original-matrix results under their own acceptance gates. The r2 guard result above is static-only and does not advance those gates. Editor 07 continues asynchronously under its separate owner and handoff; this Runtime 03 record neither closes nor replaces that work.

The matching [Runtime 03 completion list](../../../astra/features/runtime/2026-10-13-runtime03-config-comparison-and-shutdown-completion-list.md) tracks the source and pending acceptance separately.
