# Codex hook process identity single handle

## Change

The Codex hook coordinator wake-up guard now validates process liveness and
creation time through the shared combined identity helper. On Windows this opens
the coordinator process once and reads both values from the same handle. The
previous path opened the process separately for liveness and creation time and
also allowed a PID-reuse race between those observations. Descriptor, repository
identity, port, schema, and HTTP wake-up behavior remain unchanged.

## Performance evidence

Real Windows current-process benchmark, 500 identity validations per sample over
41 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 16,753,000 ns | 10,524,600 ns | 37.178% lower, 1.59x |
| p95 | 24,190,600 ns | 16,803,100 ns | 30.539% lower, 1.44x |
| Windows process handle opens | 2 | 1 | 50.000% lower |

## Validation

The static performance contract, Python compilation, and scoped `git diff
--check` passed locally. The system Python does not provide pytest, so the
contract module and complete Codex hook suite remain in the accumulated
asynchronous coordinator validation batch.
