# Action recovery session SQL projection

## Change

Control action restart recovery now projects `sessionId` from action parameter
JSON with SQLite `json_extract`. The previous path fetched every complete
parameter JSON document and decoded it in Python only to read that one field for
the failure audit event. Action selection, state transition, rollover exclusion,
event payload, and transaction behavior remain unchanged.

## Performance evidence

Real SQLite projection benchmark with 10,000 interrupted actions and 31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 68,313,100 ns | 37,697,700 ns | 44.816% lower, 1.81x |
| p95 | 97,393,300 ns | 48,049,200 ns | 50.665% lower, 2.03x |
| Python JSON parses | 10,000 | 0 | 100.000% lower |

The benchmark asserted identical action/kind/session projections before
recording timings.

## Validation

The complete action execution module passed 22/22 in 142.568 seconds, including
previous-daemon-only recovery and immutable audit behavior. The static
performance contract, Python compilation, and scoped `git diff --check` passed.
Broader server integration remains in the accumulated asynchronous coordinator
validation batch.
