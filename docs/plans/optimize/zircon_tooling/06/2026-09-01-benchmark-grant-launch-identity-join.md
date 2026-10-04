# Benchmark grant launch identity join

## Change

Benchmark launch authorization now loads the grant, source and target plan
identity, and workflow owner through one joined query. The previous path loaded
the grant and then issued three dependent point queries before evaluating the
same authorization predicate. The exact workflow binding remains a separate
query, and all launch, copy, manifest, command, plan, workflow, and binding
checks retain their existing error behavior.

## Performance evidence

Real SQLite benchmark, 5,000 valid grant identity loads per sample over 41
samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 133,476,800 ns | 68,026,000 ns | 49.035% lower, 1.96x |
| p95 | 198,985,000 ns | 103,570,700 ns | 47.950% lower, 1.92x |
| SQL statements | 5 | 2 | 60.000% lower |

## Validation

The complete benchmark validation grant module passed 6/6 in 31.861 seconds.
The static performance contract, Python compilation, and scoped `git diff
--check` also passed. Broader server and workspace-copy integration coverage is
retained in the accumulated asynchronous coordinator validation batch.
