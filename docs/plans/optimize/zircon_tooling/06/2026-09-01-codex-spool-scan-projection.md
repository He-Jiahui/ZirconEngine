# Codex spool scan projection

## Change

Pending spool scans no longer probe the pending directory with `exists()` before
calling `glob()`, which already returns an empty iterator for a missing root.
Ordered pending paths are now produced directly by `sorted(generator)` instead
of first materializing a tuple and then copying it into a sorted list. The
acknowledgement, quarantine, ordering, and missing-directory behaviors remain
unchanged.

## Performance evidence

Controlled Windows Python benchmark projecting and sorting 100,000 path objects
over 21 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 26,861,900 ns | 20,202,100 ns | 24.793% lower, 1.33x |
| p95 | 33,734,000 ns | 25,730,500 ns | 23.725% lower, 1.31x |
| Peak Python allocation | 2,400,216 bytes | 1,601,296 bytes | 33.285% lower, 1.50x |
| Missing-root metadata probes | 1 pre-probe | 0 pre-probes | eliminated |

## Validation

The structural contracts and complete Codex spool test module passed 13/13 in
0.970 seconds. Broader coordinator validation remains delegated in the
accumulated asynchronous batch.
