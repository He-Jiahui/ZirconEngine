# Failure delegation origin-lease SQL overlap

## Change

Delegated Failure proof validation now applies exact and separator-bounded
ancestor/descendant lease overlap predicates in SQLite before rows are returned
to Python. The query uses `substr()` equality rather than `LIKE`, so legitimate
path characters such as `%` and `_` cannot become wildcards. Expiration filtering
and the rule that every overlapping live lease must belong to the origin owner
remain unchanged. SQLite still scans live rows for the bidirectional prefix
predicate; the optimization removes unrelated row transfer and Python filtering.

## Performance evidence

Windows in-memory SQLite benchmark with 20,000 unrelated live leases and three
overlapping leases over 41 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 54,227,300 ns | 9,824,600 ns | 81.883% lower, 5.52x |
| p95 | 78,805,900 ns | 19,117,300 ns | 75.741% lower, 4.12x |
| Rows returned to Python | 20,003 | 3 | 99.985% lower |

## Validation

Both delegation performance contracts and the complete failure-return delegation
module passed 6/6 in 46.011 seconds, proving compatibility with the authorization
JSON SQL filter. Broader coordinator validation remains delegated in the
accumulated asynchronous batch.
