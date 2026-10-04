# Cleanup reservation SQL overlap

## Change

Cleanup reservation conflict checks now filter exact, ancestor, and descendant
target identities in SQLite and return only the earliest matching reservation.
The previous path loaded every active cleanup reservation and scanned target keys
in Python. Reservation ordering, normalized Windows path semantics, and returned
target evidence remain unchanged. The query uses `substr`/`length`, avoiding
wildcard interpretation of path characters.

## Performance evidence

Real SQLite benchmark with 10,000 cleanup reservations, one ancestor match, and
31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 21,778,100 ns | 5,051,100 ns | 76.807% lower, 4.31x |
| p95 | 27,268,600 ns | 9,684,800 ns | 64.484% lower, 2.82x |
| Returned rows | 10,000 | 1 | 99.990% lower |

The benchmark asserted identical earliest overlapping reservation identity.

## Validation

Five reservation lifecycle and parent/child overlap tests passed 5/5 in 17.070
seconds. The static performance contract, Python compilation, and scoped diff
check passed. Include cleanup and Cargo job cross-module coverage in the
accumulated asynchronous validation batch.
