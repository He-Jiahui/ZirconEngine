# Workflow gate narrow rank

## Change

Workflow commit eligibility ranking now carries only `node_id`, `gate_kind`,
`decision`, and `input_fingerprint` through its window-function temporary
result. The previous `source.*`/`evidence.*` projection carried every gate
evidence column, including several potentially large JSON payloads, although the
eligibility calculation never reads them. Latest-row ordering, required gate
kinds, fingerprint consistency, and independent review checks remain unchanged.

## Performance evidence

Real SQLite benchmark with 5,000 gate evidence rows, two unused 4 KiB payload
columns per row, 1,000 latest gate projections, and 21 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 1,381,874,800 ns | 19,016,700 ns | 98.624% lower, 72.67x |
| p95 | 1,989,523,400 ns | 29,734,200 ns | 98.505% lower, 66.91x |

The benchmark asserted identical ordered node/gate/decision/fingerprint rows.

## Validation

The complete workflow projections module passed 2/2 in 13.647 seconds. The
static performance contract, Python compilation, and scoped diff check passed.
Include broader control snapshot/workflow integration in the accumulated
asynchronous validation batch.
