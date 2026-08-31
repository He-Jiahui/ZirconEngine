# PlatformBundle template host resolve cache

## Change

Template materialization previously resolved the same host executable once per
template entry. It now lazily resolves the host on the first entry that needs the
comparison and reuses the result for the rest of the manifest. Empty file lists
still do not resolve the host, and a failed host resolution remains fatal.

## Performance evidence

The deterministic contract uses 64 template entries and proves host resolve
calls fall from 64 to 1, a 98.438% reduction. A controlled resolver-cost benchmark
(25 rounds, 2,000 manifests per round) measured:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 2,614,140,800 ns | 1,470,551,700 ns | 43.746% lower, 1.78x |
| p95 | 3,131,954,200 ns | 1,950,857,700 ns | 37.711% lower, 1.61x |
| Host resolves per 64-entry manifest | 64 | 1 | 98.438% lower |

The real filesystem benchmark was discarded after exceeding the 120-second local
budget under shared disk load; it is not used as performance evidence.

## Validation

The cache contract passed 2/2. The expanded PlatformBundle batch passed 72/72,
including resolve-error behavior and the empty-file-list invariant.
