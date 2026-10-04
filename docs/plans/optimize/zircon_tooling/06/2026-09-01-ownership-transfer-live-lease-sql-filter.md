# Ownership transfer live-lease SQL filter

## Change

Ownership transfer preview and apply validation now filter expired leases in the
SQLite query using the repository's canonical UTC text representation. The old
path fetched every historical lease into Python and parsed every expiration
timestamp before retaining live rows. The inclusive `expires_at >= now` boundary
matches the previous `now <= parse_utc(expires_at)` rule exactly. This change
reduces row materialization and Python timestamp parsing; it does not claim to
remove SQLite's table scan because the lease table has no expiration index.

## Performance evidence

Windows in-memory SQLite benchmark with 20,000 expired leases and 64 live
leases, 31 rounds per implementation:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 49,237,600 ns | 1,885,800 ns | 96.170% lower, 26.11x |
| p95 | 61,997,900 ns | 3,201,700 ns | 94.836% lower, 19.36x |
| Rows materialized in Python | 20,064 | 64 | 99.681% lower |
| Python timestamp parses | 20,064 | 0 | eliminated |

## Validation

The SQL-filter performance contracts plus live-foreign-lease, successful apply,
and stale-preview behavior regressions passed 5/5 in 26.642 seconds. Broader
coordinator validation remains delegated in the accumulated asynchronous batch.
