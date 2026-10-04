# Failure delegation authorization SQL filter

## Change

Failure-return delegation lookup now filters authorization event JSON in SQLite
to the exact requested lifecycle keys before rows cross into Python. A guarded
`CASE WHEN json_valid(...) THEN json_extract(...) END` preserves the prior rule
that malformed historical payloads are ignored. Descending event order,
newest-per-lifecycle selection, and early completion remain unchanged. The old
path materialized every authorization event for the Session and decoded JSON in
Python until all requested keys were found.

## Performance evidence

Conservative Windows in-memory SQLite benchmark with 10,000 unrelated valid
events, 10,000 malformed events, and 32 requested authorization events inserted
latest, over 31 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 26,901,000 ns | 16,074,000 ns | 40.248% lower, 1.67x |
| p95 | 62,071,800 ns | 22,515,800 ns | 63.726% lower, 2.76x |
| Rows returned to Python | up to 20,032 | 32 | 99.840% lower |

The requested events were deliberately newest, which favors the previous early
break behavior; older requested authorizations benefit more from SQL filtering.

## Validation

The SQL-filter contract and complete failure-return delegation test module
passed 5/5 in 49.345 seconds. Broader coordinator validation remains delegated
in the accumulated asynchronous batch.
