# Runtime177 Dispatch Outcome Single Pass

- Date: 2026-08-26
- Session: `root-runtime-interface03-activate-link-failure-20260831` (current-source convergence)
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_queued`
- Validation request: `runtime-six-hotpath-performance-batch-20260831-final`

## Problem

Runtime input outcome construction traversed every dispatch result once to clone host requests and
again to detect dirty-redraw effects. Batched keyboard, pointer, and accessibility dispatch paid two
top-level result scans even though both metadata outputs are finalized together.

## Optimization

- Collect host requests and detect dirty-redraw effects in one result loop.
- Stop inspecting effects after redraw becomes known while continuing ordered request collection.
- Preserve host-request order and the window's pre-existing redraw request.

## Regression Contract

The shared `runtime_hotpath_batch_` filter owns three Runtime177 tests: request/redraw behavior,
single-pass source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME177_DISPATCH_OUTCOME_SINGLE_PASS_BENCH_V1`, processes 2,048 batches of 256 results per
sample, reduces top-level result passes from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Deterministic Evidence

The source-bound model uses the benchmark workload of 2,048 batches with 256 results each. It
reduces top-level result passes from 4,096 to 2,048 and top-level result visits from 1,048,576 to
524,288, exactly 50%. Host-request order, pre-existing redraw state, and dirty-redraw detection are
preserved. These counts are scan work, not elapsed-time claims; the paired release P50/P95 samples
remain managed acceptance evidence.

## Validation Ownership

One combined managed Windows release command must run all six Runtime177/Runtime182 tests under the
shared filter. The coordinator owns exact P50/P95 backfill, commit, push to `origin/main`, and the
one-shot WeCom report after a pushed SHA exists.

After shared HEAD advanced, current hashes were re-attested under lease request
`afb668dd6f084a3b9389229ea6085fc6`. Current-source batch ticket
`dd6072a2666c4943bbf2a496d941079c` was queued from snapshot `2510` by request
`runtime-six-hotpath-performance-batch-20260831-current-head-r3` (receipt
`73ee20784cb846e2bcfef6cbddb0f952`). It covers 12 Rust behavior tests, 6 ignored release
benchmarks, and 27 Python contracts; the coordinator owns terminal timing, commit, push, and the
one-shot WeCom report.
