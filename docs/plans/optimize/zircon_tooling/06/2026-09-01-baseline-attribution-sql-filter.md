# Baseline attribution SQL filter

## Change

Baseline reconciliation now filters attribution rows by the active baseline
epoch and executable Session status in SQLite. The previous path fetched every
historical attribution for the changed path set and discarded old-epoch and
terminal-Session rows in Python. Current content-hash matching and unattributed
change behavior remain unchanged.

## Performance evidence

Real SQLite benchmark with 1,000 changed paths, ten historical/stale attribution
rows and one current live row per path, 31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 40,346,800 ns | 8,628,500 ns | 78.614% lower, 4.68x |
| p95 | 56,874,200 ns | 44,979,200 ns | 20.915% lower, 1.26x |
| SQLite rows returned | 11,000 | 1,000 | 90.909% lower |

The benchmark asserted identical current attribution dictionaries before
recording timings.

## Validation

The complete baseline module passed 18/18 in 161.052 seconds, including prior
epoch attribution rejection and reconciliation behavior. The static performance
contract, Python compilation, and scoped `git diff --check` passed. Broader
finalize/server integration remains in the accumulated asynchronous coordinator
validation batch.
