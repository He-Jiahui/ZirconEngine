# Runtime 44 stable shutdown terminal-result source repair

**Status:** Source adoption recorded. Native/type-check, performance, product, and Jenkins acceptance remain pending.

## Repair scope

The lifecycle owner now preserves the completed shutdown result after detaching a finished worker. Repeated shutdown calls therefore retain a completed output failure instead of turning the next no-active call into success. The existing initial-idle result remains `true`; a timeout remains retryable while the generation is still completing; a fresh generation resets the prior terminal result only after publication. Joining and unloading the worker remains independent of whether output writes succeeded.

Four ordinary regression names are now present in `lifecycle.rs`:

- `process_shutdown_repeats_completed_worker_output_failures`
- `process_shutdown_repeats_completed_success_without_restarting_worker`
- `process_shutdown_terminal_result_resets_only_after_fresh_generation_publication`
- `process_shutdown_timeout_retains_generation_until_worker_completion`

These names are source additions, not executed test results. The independent review records 16 to 20 Windows ordinary cases and 15 to 19 portable ordinary cases, with the single ignored case unchanged. The previous lifecycle-test prefix remains byte-exact. The new tests use an isolated `ProcessLogController` and real worker paths; they do not invoke the global public wrapper, so this record does not claim a global-wrapper run or reproduction.

## Applied source evidence

The actual canonical apply receipt is [`actual-canonical-apply-v1.json`](../../../../../.codex/outbox/runtime44-stable-shutdown-terminal-result-repair-v1/actual-canonical-apply-v1.json), SHA-256 `23b38dc6efd39788a02a9b8425212fbc5799434efbdf81e024f63905b6f9a784` (11,084 bytes). It records joint strict check and scoped apply exit code 0, four canonical paths across the two adopted packets, unchanged Git index, exact physical postimages, and zero source copies.

For this packet, `sink.rs` is 25,911 bytes (raw SHA-256 `5b034ecd5c36d133cfcc6c212327d53c0986aceb8011b54399702251cc4e418c`; LF SHA-256 `a8b7f20ee50759f7ba5ffd10b627aa34e6b7926cf403d6db2dbf81300a08cbd0`). `lifecycle.rs` is 28,750 bytes (raw and LF SHA-256 `7a355004460c8ee1b1e1d91915059d33ac965cb47822d280508cd0c0366515ac`). The actual receipt records each postimage's exact mtime and no-reparse check.

The author's [source binding](../../../../../.codex/outbox/runtime44-stable-shutdown-terminal-result-repair-v1/source-binding-v1.json) and [contract proof](../../../../../.codex/outbox/runtime44-stable-shutdown-terminal-result-repair-v1/source-contract-proof-v1.json), plus the [independent source review](../../../../../.codex/outbox/runtime44-stable-shutdown-terminal-result-repair-v1/independent-review-v1.json), are retained beside the [candidate patch](../../../../../.codex/outbox/runtime44-stable-shutdown-terminal-result-repair-v1/candidate-v1.patch) and [Native test entrypoint proposal](../../../../../.codex/outbox/runtime44-stable-shutdown-terminal-result-repair-v1/native-test-entrypoints-v1.json). The independent review accepts this bounded Source repair only; type checking, Native execution, and broader contracts remain open.

## Validation and limits

No Rust type-check, Native test, or performance sample ran as part of adoption. Four ordinary cases are present in Source and zero were executed. This repair does not close full durability receipts, process/app/DLL ownership convergence, byte admission, batch hard caps, worker-panic behavior, or the original performance and hardware gates.

The prior 15-command contract remains intact: 14 original producers plus two Runtime 03 matrix producers, 16 total. Conditional future floors are 64 unique ordinary cases for command #2, 193 for command #15, and 12 ignored-library matches for command #9. Those are selection requirements, not run counts. The earlier Native07 recipe remains frozen and refused; a future validation run needs a new coordinator and Source identity. The user-authorized Jenkins handoff is being prepared; it has not been submitted to a queue and is not Jenkins acceptance.

The matching [Runtime 44 completion list](../../../astra/features/runtime/2026-10-12-runtime44-stable-shutdown-terminal-result-source-completion-list.md) tracks this narrow Source scope separately from the pending acceptance gates.
