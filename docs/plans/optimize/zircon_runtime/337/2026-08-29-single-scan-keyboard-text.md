# Runtime337 Single-Scan Keyboard Text

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_queued`
- Session: `root-runtime-interface03-activate-link-failure-20260831` (current-source convergence)
- Validation request: `runtime-six-hotpath-performance-batch-20260831-final`

## Scope

Runtime UI keyboard text admission previously scanned characters once for controls and scanned them
again to reject all-whitespace text. Admission now rejects controls and tracks the first non-space
character in one loop. Empty text, Unicode whitespace, control-character rejection, modifier gates,
and the borrowed accepted text remain unchanged.

## Static Evidence

- Character traversals for all-whitespace text: `2 -> 1`.
- Accepted-text allocations remain `0`.
- Unicode `char` classification remains unchanged.
- At 8,192 checks of 4,096 characters, character visits change from `67,108,864` to
  `33,554,432`, exactly 50% fewer.

## Performance Gate

The shared `runtime_hotpath_batch_` filter includes this task's three tests. The ignored Windows release
benchmark emits `RUNTIME337_SINGLE_SCAN_KEYBOARD_TEXT_BENCH_V1`. It
compares the baseline control/whitespace traversals with the combined traversal over 8,192 4-KiB
all-whitespace inputs across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

One combined managed Windows release command must run all six Runtime337/Runtime338 tests. The
coordinator owns exact timing capture, commit, push to `origin/main`, and the one-shot WeCom result
containing exact performance data, test result, commit SHA, and branch.

After shared HEAD advanced, current hashes were re-attested under lease request
`afb668dd6f084a3b9389229ea6085fc6`. Current-source batch ticket
`dd6072a2666c4943bbf2a496d941079c` was queued from snapshot `2510` by request
`runtime-six-hotpath-performance-batch-20260831-current-head-r3` (receipt
`73ee20784cb846e2bcfef6cbddb0f952`). It covers 12 Rust behavior tests, 6 ignored release
benchmarks, and 27 Python contracts; the coordinator owns terminal timing, commit, push, and the
one-shot WeCom report.
