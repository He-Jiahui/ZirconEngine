# Offline command queue scan projection

## Change

Offline command queue scans now call `glob()` directly without a root `exists()`
pre-probe and return the list produced by `sorted(generator)` directly from the
private helper. The previous path copied that sorted list into a tuple for every
replay, capacity snapshot, failed count, and quarantine count. FIFO filename
ordering, missing-directory behavior, and all replay/quarantine semantics remain
unchanged.

## Performance evidence

Controlled Windows Python benchmark sorting 50,000 path objects four times per
sample over 41 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 56,309,400 ns | 46,288,500 ns | 17.796% lower, 1.22x |
| p95 | 87,417,300 ns | 66,171,600 ns | 24.304% lower, 1.32x |
| Sorted-list to tuple copies | 1 per scan | 0 | eliminated |
| Missing-root pre-probes | 1 per scan | 0 | eliminated |

Tracemalloc peak was flat in the preliminary single-pass benchmark, so no peak
memory improvement is claimed.

## Validation

The scan contracts and complete offline command spool module passed 14/14 in
0.906 seconds. Broader coordinator validation remains delegated in the
accumulated asynchronous batch.
