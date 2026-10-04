# Git finalize recovery lock single probe

## Change

Isolated-patch recovery now classifies the proven index lock from one `stat()`
result instead of calling `exists()`, `is_file()`, and `stat()`. Missing locks
still return, regular zero-byte locks are removed, and directories/non-empty
locks remain ambiguous.

## Performance evidence

Windows local benchmark, valid zero-byte lock, 25 rounds of 10,000 checks:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 702,920,500 ns | 234,911,400 ns | 66.581% lower, 2.99x |
| p95 | 945,467,300 ns | 345,414,000 ns | 63.466% lower, 2.74x |
| Metadata probes | 3 | 1 | 66.667% lower |

## Validation

Recovery-lock behavior and delegated-set contracts passed 3/3; module compilation
passed. Full finalize/failure-closeout behavior remains in the Coordinator lane.
