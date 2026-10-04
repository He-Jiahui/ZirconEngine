# AI effort SQL aggregate

## Change

AI effort reporting now lets SQLite aggregate milestone count and hours by
outcome/cost class. Python only reads milestone rows whose blocker JSON is
non-empty and expands those rows for the blocker projection. The previous path
read, parsed, and accumulated every ledger row in Python even when the report
only needed grouped totals. Baseline, ledger, blocker, and forecast response
fields remain unchanged.

## Performance evidence

Real SQLite report benchmark with 10,000 milestones and 1% blocker-bearing rows,
31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 75,114,800 ns | 15,323,800 ns | 79.599% lower, 4.90x |
| p95 | 103,892,100 ns | 32,045,300 ns | 69.155% lower, 3.24x |
| Python rows processed | 10,000 | 106 | 98.940% lower |

The 106 after rows comprise 100 blocker-bearing milestones and six grouped
outcome/cost-class aggregates. The benchmark asserted complete aggregate result
equality before recording timings.

## Validation

The complete AI effort API module passed 5/5 in 43.488 seconds. The static
performance contract, Python compilation, and scoped `git diff --check` passed.
Broader control-server coverage remains in the accumulated asynchronous
coordinator validation batch.
