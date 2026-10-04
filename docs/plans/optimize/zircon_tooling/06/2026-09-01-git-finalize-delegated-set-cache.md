# Git finalize delegated membership set cache

## Change

Scoped finalize now builds the normalized delegated-path membership set once and
reuses it for proof binding, material binding, ordinary proof filtering, and
ordinary material filtering. Tuple ordering and diagnostics are unchanged.

## Performance evidence

Windows Python benchmark, 10,000 delegated paths, 25 rounds of 20 checks:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 274,797,400 ns | 213,718,500 ns | 22.227% lower, 1.29x |
| p95 | 422,885,800 ns | 341,241,900 ns | 19.306% lower, 1.24x |
| Delegated set constructions | 4 | 1 | 75.000% lower |

Two earlier benchmark attempts were discarded: one exceeded the command budget,
and one accidentally rebuilt a set per element rather than per filter expression.
Neither is used as evidence.

## Validation

The structural performance contract passed 1/1 and `git_finalize.py` compiled.
The Coordinator must run the full finalize/failure-closeout behavior batch in its
longer combined lane.
