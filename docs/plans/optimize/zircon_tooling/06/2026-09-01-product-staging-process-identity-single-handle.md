# Product staging process identity single handle

## Change

Artifact product staging owner validation now uses a combined process liveness
and creation-time identity check. On Windows the helper opens the process once,
reads exit state and creation time from that same handle, then closes it. The
previous path opened and closed one handle for liveness and a second handle for
creation time, adding syscall cost and a PID-reuse race between observations.
Non-Windows behavior retains the existing liveness-then-creation checks behind
the same API.

## Performance evidence

Real Windows current-process benchmark, 500 identity validations per sample over
41 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 16,405,100 ns | 9,888,000 ns | 39.726% lower, 1.66x |
| p95 | 35,580,800 ns | 14,550,200 ns | 59.107% lower, 2.45x |
| Windows process handle opens | 2 | 1 | 50.000% lower |

## Validation

The single-handle contracts and complete artifact product staging module passed
6/6 in 6.768 seconds. A broader process-module run had one unrelated Windows
temporary-directory cleanup lock failure; the process module itself passed
13/13 for candidate 35 and broader validation remains delegated in the
accumulated asynchronous batch.
