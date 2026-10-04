# Workflow topology dependency single pass

## Change

Graph validation now checks missing dependencies, self-dependencies, and builds
the reverse dependent index in one dependency traversal. Previously it created a
set, performed a separate membership scan, then traversed dependencies again.
The edge-budget check still runs first and error precedence is preserved.

## Performance evidence

Windows Python benchmark, 200 nodes and 3,790 edges, 25 rounds of 200 validations:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 245,574,200 ns | 163,888,400 ns | 33.263% lower, 1.50x |
| p95 | 362,149,400 ns | 231,550,400 ns | 36.062% lower, 1.56x |
| Dependency traversals per node | 3 | 1 | 66.667% lower |

## Validation

Dependency, duplicate, canonical projection, and testing-stage topology coverage
passed 8/8 in 10.522 seconds.
