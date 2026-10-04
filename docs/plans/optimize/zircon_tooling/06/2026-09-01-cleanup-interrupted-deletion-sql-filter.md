# Cleanup interrupted deletion SQL filter

## Change

Cleanup restart recovery now extracts deletion identity once in a SQLite CTE,
groups successful terminal completions, and returns only non-terminal start
events whose target keys have active cleanup reservations. The previous path
loaded and JSON-decoded every historical cleanup start/completion event before
discarding completed and unrelated deletions in Python. Failed and
failed-after-restart completions remain retryable, successful completions remain
terminal, reservation ordering remains stable, and the latest start per target
continues to win.

## Performance evidence

Real SQLite recovery benchmark with 10,000 successfully completed deletions and
10 interrupted reserved targets, 31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 152,422,300 ns | 48,976,400 ns | 67.868% lower, 3.11x |
| p95 | 259,226,000 ns | 71,292,300 ns | 72.498% lower, 3.64x |
| Python JSON rows | 20,010 | 10 | 99.950% lower |

The benchmark asserted identical interrupted-deletion payloads before recording
timings. A correlated-subquery prototype reduced Python rows but regressed p50
by 15.482%; it was fully removed before the single-scan CTE implementation.

## Validation

Five focused restart/recovery tests passed 5/5 in 27.124 seconds, including
failed-after-restart retry behavior and validation-copy overlap denial. The
static performance contract, Python compilation, and scoped `git diff --check`
passed. Complete cleanup coverage remains in the accumulated asynchronous
coordinator validation batch.
