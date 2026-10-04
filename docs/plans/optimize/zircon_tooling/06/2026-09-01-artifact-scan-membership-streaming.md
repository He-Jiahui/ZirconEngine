# Artifact scan membership and streaming directory traversal

## Change

Artifact governance now builds one immutable exact-membership index for managed
paths at the start of a scan and shares it through recursive directory traversal.
Previously every directory child performed a linear `any()` across all managed
paths. Directory children are also consumed directly from `iterdir()` instead
of being copied into a tuple at every recursion level. Containment checks,
reparse-point handling, deterministic result sorting, and the existing behavior
of discarding a partially enumerated directory after `OSError` are preserved.

## Performance evidence

Windows Python controlled benchmarks:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| Exact lookup p50, 8,192 managed paths / 2,048 queries | 1,405,004,200 ns | 233,100 ns | 99.983% lower, 6027.47x |
| Exact lookup p95 | 1,513,876,300 ns | 263,500 ns | 99.983% lower, 5745.26x |
| 65,536-child traversal p50 | 8,539,100 ns | 5,093,000 ns | 40.357% lower, 1.68x |
| 65,536-child peak Python allocation | 3,172,496 bytes | no additional allocation detected | 3.17 MiB removed |

The streaming traversal p95 contained a Windows scheduling spike and is
deliberately excluded from improvement claims.

## Validation

The structural/performance contracts, partial-enumeration error regression, and
three real artifact scan behavior tests passed 7/7 in 31.307 seconds. Broader
coordinator validation remains delegated in the accumulated asynchronous batch.
