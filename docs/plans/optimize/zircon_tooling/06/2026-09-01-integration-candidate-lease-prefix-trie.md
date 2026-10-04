# Integration candidate lease prefix trie

## Change

Integration candidate sealing now indexes live lease paths in a temporary prefix
trie and walks each normalized candidate path until the first leased ancestor.
The previous path allocated every ancestor string with repeated split/slice/join
operations and probed a flat dictionary. First-ancestor priority, missing-lease
errors, and emitted lease evidence remain unchanged.

## Performance evidence

Production-equivalent Python benchmark with 1,000 live leases and 10,000 deep
candidate paths, 41 samples:

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| p50 | 16,428,100 ns | 11,619,300 ns | 29.272% lower, 1.41x |
| p95 | 37,005,800 ns | 19,397,700 ns | 47.582% lower, 1.91x |
| peak traced allocation | 111,846 bytes | 328,978 bytes | 217,132 bytes higher |

The temporary trie deliberately trades about 212 KiB of peak allocation at this
scale for lower sealing latency and tail latency. It exists only for one lease
evidence projection and is released afterward; no persistent cache or shared
mutable state was introduced.

## Validation

The complete integration candidate module passed 11/11 in 89.375 seconds. The
static performance contract, Python compilation, and scoped `git diff --check`
passed. Broader server integration remains in the accumulated asynchronous
coordinator validation batch.
