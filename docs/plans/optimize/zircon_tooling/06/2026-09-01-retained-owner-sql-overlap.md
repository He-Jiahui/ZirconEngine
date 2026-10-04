# Retained owner SQL overlap

## Change

Cleanup retained-pool ownership checks now filter exact, ancestor, and descendant
target identities in SQLite and return only the newest matching owner. The
previous path loaded every retained job and scanned target keys in Python for
each cleanup safety check. Excluded job IDs, exact-target suppression, release
ordering, Windows-normalized target identity, and overlap semantics remain
unchanged. The SQL uses `substr`/`length` rather than wildcard matching, so path
characters cannot be interpreted as a pattern.

## Performance evidence

Real SQLite benchmark with 10,000 retained jobs, one ancestor match, two excluded
job IDs, and 31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 25,301,000 ns | 5,177,600 ns | 79.536% lower, 4.89x |
| p95 | 65,237,200 ns | 8,038,400 ns | 87.678% lower, 8.12x |
| Returned rows | 10,000 | 1 | 99.990% lower |

The benchmark asserted identical selected owner identity.

## Validation

Five retained-owner tests passed 5/5 in 13.706 seconds, covering exact,
parent/child overlap and pressure eviction. The static performance contract,
Python compilation, and scoped diff check passed. Re-run the complete cleanup
module once in the accumulated asynchronous validation batch with this helper.
