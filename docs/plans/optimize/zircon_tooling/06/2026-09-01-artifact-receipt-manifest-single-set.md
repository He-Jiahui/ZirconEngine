# Artifact receipt copy-manifest single set

## Change

Managed artifact receipt validation now builds the normalized copy-manifest
`frozenset` once, uses that same immutable object for duplicate detection, and
returns it to downstream membership checks. The previous path first built a
temporary mutable set to detect duplicates and then hashed every path again into
a second frozenset. Manifest normalization, duplicate rejection, and immutable
return semantics remain unchanged.

## Performance evidence

Controlled Windows Python benchmark with 100,000 normalized paths over 31 rounds:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 35,812,000 ns | 15,894,700 ns | 55.616% lower, 2.25x |
| p95 | 67,391,600 ns | 21,603,900 ns | 67.943% lower, 3.12x |
| Full membership-set builds | 2 | 1 | 50.000% lower |

Tracemalloc peak was flat because the temporary set was released before the
frozenset allocation, so no peak-memory improvement is claimed.

## Validation

The single-projection contract and complete managed artifact receipt test module
passed 14/14 in 23.320 seconds. Broader coordinator validation remains delegated
in the accumulated asynchronous batch.
