# PlatformBundle template root projection cache

## Change

Template materialization now constructs the fixed template-root `Path` once per
manifest instead of once per file entry. Source ordering, validation, resolution,
copying, and diagnostics are unchanged.

## Performance evidence

Windows Python CPU benchmark, 64 entries, 25 rounds of 10,000 manifests:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 2,680,440,900 ns | 1,414,706,400 ns | 47.221% lower, 1.89x |
| p95 | 2,967,343,100 ns | 1,741,443,000 ns | 41.313% lower, 1.70x |
| Root projections per manifest | 64 | 1 | 98.438% lower |

## Validation

The projection contract passed 1/1. The expanded PlatformBundle combined batch
passed 73/73 in 29.294 seconds.
