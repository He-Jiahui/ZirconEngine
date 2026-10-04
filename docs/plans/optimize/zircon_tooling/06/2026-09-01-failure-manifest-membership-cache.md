# Failure manifest membership cache

## Change

Failure manifest filtering now builds one immutable membership index after path
normalization and shares it across every candidate Failure node. Child-record
source-slice checks previously rebuilt the same manifest set twice per node,
plus two related-code sets. The optimized path uses direct membership checks
and `frozenset.difference()` without changing path normalization, SQL filtering,
or child-record evidence rules.

## Performance evidence

Windows Python benchmark, 512 candidate nodes and a 4,096-path manifest, 31
rounds per implementation:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 393,539,900 ns | 85,747,600 ns | 78.211% lower, 4.59x |
| p95 | 567,955,400 ns | 130,924,400 ns | 76.949% lower, 4.34x |
| Manifest membership indexes | 2 per candidate node | 1 per request | O(nodes) to O(1) |

## Validation

The focused performance contract and the two manifest/child-record behavior
regressions passed 4/4 in 4.786 seconds. Broader coordinator validation remains
delegated as part of the accumulated asynchronous batch.
