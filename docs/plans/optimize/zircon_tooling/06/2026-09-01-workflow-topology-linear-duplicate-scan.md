# Workflow topology linear duplicate scan

## Change

`TopologyParser._validate_graph` previously located a duplicate ID with
`ids.count()` inside a generator, making the duplicate path O(n^2). It now builds
the ordered ID list and membership set in one pass and reports the first repeated
ID in input order.

## Performance evidence

Windows Python benchmark, maximum 200 nodes with a duplicate at the end, 25 rounds
of 1,000 validations:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 1,076,283,800 ns | 21,469,000 ns | 98.005% lower, 50.13x |
| p95 | 1,611,331,600 ns | 27,999,700 ns | 98.262% lower, 57.55x |
| Duplicate scan complexity | O(n^2) | O(n) | linearized |

An earlier 20,000-iteration benchmark exceeded 120 seconds and was discarded; it
is not used as evidence.

## Validation

The performance contract and topology behavior batch passed 20/20 in 111.472
seconds, including legacy testing-stage parsing and duplicate diagnostics.
