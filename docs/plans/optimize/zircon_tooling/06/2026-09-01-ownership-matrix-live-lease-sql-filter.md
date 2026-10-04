# Ownership matrix live-lease SQL filter

## Change

Ownership matrix construction now applies the inclusive live-lease expiration
predicate in SQLite before ordering and projecting the lease map. Previously it
loaded and sorted every historical lease, materialized every row in Python, and
parsed every expiration timestamp before discarding expired records. Canonical
UTC text ordering is already the repository's persisted time contract, and the
new `expires_at >= now` boundary matches the prior comparison exactly. SQLite
still scans the unindexed expiration column; the measured savings come from
sorting, returning, parsing, and projecting only live rows.

## Performance evidence

Windows in-memory SQLite benchmark using the production query shape, including
`ORDER BY path_key` and dictionary projection, with 20,000 expired leases and 64
live leases over 31 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 63,193,600 ns | 1,983,000 ns | 96.862% lower, 31.87x |
| p95 | 76,818,800 ns | 3,508,600 ns | 95.433% lower, 21.89x |
| Rows sorted/materialized in Python | 20,064 | 64 | 99.681% lower |
| Python timestamp parses | 20,064 | 0 | eliminated |

## Validation

The SQL-filter performance contracts and the complete focused ownership-matrix
suite passed 5/5 in 14.472 seconds. Broader coordinator validation remains
delegated in the accumulated asynchronous batch.
