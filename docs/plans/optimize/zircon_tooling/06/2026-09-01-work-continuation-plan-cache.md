# Work continuation per-snapshot plan cache

## Change

`WorkContinuationService.project` now parses each distinct `plan_path` once per
database snapshot and reuses the candidate for every waiting Session on that
plan. `None` results are cached too, so fallback projection does not repeatedly
read a plan with no implementation slice.

## Performance evidence

Windows local benchmark, 20 Sessions sharing a 256 KiB plan, 25 rounds of 10
snapshots:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 166,368,300 ns | 4,677,600 ns | 97.188% lower, 35.57x |
| p95 | 352,206,300 ns | 18,156,100 ns | 94.845% lower, 19.40x |
| Plan reads per snapshot | 20 | 1 | 95.000% lower |
| Bytes read per snapshot | 5,243,760 | 262,188 | 95.000% lower |

## Validation

- Cache contracts: 2/2 passed.
- The full control-snapshot module exceeded the 120-second local command budget;
  it is neither green nor failed and remains assigned to the Coordinator lane.
