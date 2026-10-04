# Run health log metadata single pass

## Change

Control-plane run health projection now aggregates log output presence and the
latest output timestamp in one pass over stdout/stderr metadata. The previous
path materialized a tuple of all stats, a second tuple of non-empty stats, and a
third generator traversal for the latest timestamp. Log availability, output
state, timestamp, and redaction behavior remain unchanged.

## Performance evidence

Pure projection benchmark for two log metadata records, 100,000 projections per
sample over 41 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 247,462,900 ns | 45,982,800 ns | 81.418% lower, 5.38x |
| p95 | 313,087,000 ns | 81,276,500 ns | 74.040% lower, 3.85x |
| peak traced allocation | 440 bytes | 32 bytes | 92.727% lower |

## Validation

The three focused run-health projection tests passed 3/3 in 13.356 seconds. The
static performance contract, Python compilation, and scoped `git diff --check`
also passed. A broad wildcard snapshot discovery exceeded the 120-second local
budget before producing a result, so it remains in the accumulated asynchronous
coordinator batch and is not reported as green or failed.
