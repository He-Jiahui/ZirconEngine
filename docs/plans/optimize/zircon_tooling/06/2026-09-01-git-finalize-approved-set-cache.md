# Git finalize approved membership set cache

## Change

Owned-scope validation now builds the approved-path set once and reuses it for
both omitted-owned and unchanged-approved differences. Ordering, error precedence,
and the returned approved tuple remain unchanged.

## Performance evidence

Windows Python benchmark, 10,000 approved paths, 25 rounds of 50 checks:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 153,837,000 ns | 121,201,800 ns | 21.214% lower, 1.27x |
| p95 | 213,857,800 ns | 176,707,000 ns | 17.372% lower, 1.21x |
| Approved set constructions | 2 | 1 | 50.000% lower |

## Validation

The three Git-finalize performance contracts passed 4/4 and module compilation
passed. Full finalize behavior remains assigned to the longer combined lane.
