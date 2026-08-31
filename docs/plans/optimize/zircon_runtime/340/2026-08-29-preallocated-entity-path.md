# Runtime340 Preallocated Entity Path

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_queued`
- Session: `root-runtime-interface03-activate-link-failure-20260831` (current-source convergence)
- Validation request: `runtime-six-hotpath-performance-batch-20260831-final`

## Scope

Runtime entity-path parsing previously grew the segment vector from its default capacity while
splitting, trimming, filtering, and allocating each retained segment. Parsing now reserves a bounded
capacity derived from input length before the same iterator pipeline. Empty segments, whitespace
normalization, path errors, and canonical raw output remain unchanged.

## Static Evidence

- Full path scans: `1 -> 1`.
- Segment-vector reallocations: `variable -> 0` for ordinary long paths.
- Segment contents and ordering remain unchanged.
- At 8,192 parses of 1,024 seven-byte segments, the implementation performs 8,192 explicit
  preallocations totaling 33,546,240 planned slots while preserving 8,388,608 owned segments.

## Performance Gate

The shared `runtime_hotpath_batch_` filter includes this task's three tests. The ignored Windows release
benchmark emits `RUNTIME340_SINGLE_SCAN_ENTITY_PATH_BENCH_V1`. It
compares the baseline unreserved segment vector with length-based preallocation over 8,192 paths of
1,024 segments across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

One combined managed Windows release command must run all six Runtime340/Runtime341 tests. The
coordinator owns exact timing capture, commit, push to `origin/main`, and the one-shot WeCom result.

After shared HEAD advanced, current hashes were re-attested under lease request
`afb668dd6f084a3b9389229ea6085fc6`. Current-source batch ticket
`dd6072a2666c4943bbf2a496d941079c` was queued from snapshot `2510` by request
`runtime-six-hotpath-performance-batch-20260831-current-head-r3` (receipt
`73ee20784cb846e2bcfef6cbddb0f952`). It covers 12 Rust behavior tests, 6 ignored release
benchmarks, and 27 Python contracts; the coordinator owns terminal timing, commit, push, and the
one-shot WeCom report.
