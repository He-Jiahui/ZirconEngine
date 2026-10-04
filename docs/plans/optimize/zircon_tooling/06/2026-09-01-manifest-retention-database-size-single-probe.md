# Manifest retention database-size single probe

## Change

Manifest retention now obtains the SQLite database size through one shared
`stat()` helper at all three capacity/compaction measurement sites. The previous
`exists()` followed by `stat()` performed two path metadata probes and retained
a race where the file could disappear between calls. The helper maps only
`FileNotFoundError` to zero; permission and other I/O failures still propagate.

## Performance evidence

Controlled Windows Python benchmark, 20,000 size reads per sample over 41
samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 9,795,400 ns | 7,741,300 ns | 20.970% lower, 1.27x |
| p95 | 12,111,900 ns | 11,673,000 ns | 3.624% lower, 1.04x |
| Metadata probes per size read | 2 | 1 | 50.000% lower |

The timing benchmark measures controlled call overhead; the deterministic probe
count is the primary evidence because filesystem latency varies by host.

## Validation

The single-probe contracts plus manifest archive/apply and SQLite compaction
behaviors passed 4/4 in 3.607 seconds. Broader coordinator validation remains
delegated in the accumulated asynchronous batch.
