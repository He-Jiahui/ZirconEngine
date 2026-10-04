# Workflow topology canonical single projection

## Change

`WorkflowTopology.canonical_json` now projects milestone and slice dataclasses
once, hashes the serialized semantic value, then extends that same value with
canonical metadata. Previously `topology_hash` called `semantic_json` first and
canonical serialization projected every node a second time. Public
`semantic_json` and `topology_hash` behavior is unchanged.

## Performance evidence

Windows Python benchmark at the 5,200-node limit, 15 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 641,392,200 ns | 466,552,600 ns | 27.259% lower, 1.37x |
| p95 | 692,266,200 ns | 555,901,200 ns | 19.698% lower, 1.25x |
| Node projections | 10,400 | 5,200 | 50.000% lower |

Peak allocation was effectively unchanged and is not claimed as an improvement.

## Validation

Canonical projection, hash parity, duplicate scan, and testing-stage behavior
passed 7/7 in 7.655 seconds.
