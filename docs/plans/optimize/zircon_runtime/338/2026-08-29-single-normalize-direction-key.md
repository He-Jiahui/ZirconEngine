# Runtime338 Single-Normalize Direction Key

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_queued`
- Session: `root-runtime-interface03-activate-link-failure-20260831` (current-source convergence)
- Validation request: `runtime-six-hotpath-performance-batch-20260831-final`

## Scope

Runtime UI directional-key recognition previously rebuilt the filtered lowercase byte iterator for
each of twelve canonical key candidates. Recognition now normalizes once into a fixed stack buffer
bounded by the longest accepted token, then matches the borrowed byte slice. Separators, ASCII case,
non-ASCII filtering, keyboard-code fallback, and returned navigation kinds remain unchanged.

## Static Evidence

- Normalization passes on an unmatched direction key: `12 -> 1`.
- Heap allocations per lookup remain `0`.
- Maximum normalized stack storage: `16 bytes`.
- At 262,144 checks, normalization passes change from `3,145,728` to `262,144`, a
  91.666667% reduction.

## Performance Gate

The shared `runtime_hotpath_batch_` filter includes this task's three tests. The ignored Windows release
benchmark emits `RUNTIME338_SINGLE_NORMALIZE_DIRECTION_KEY_BENCH_V1`.
It compares the baseline candidate-by-candidate normalization with the fixed-buffer path over 262,144
near-matching direction keys across 31 interleaved sample pairs and requires
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
