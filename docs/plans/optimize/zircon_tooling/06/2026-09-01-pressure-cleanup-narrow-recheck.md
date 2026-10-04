# Pressure cleanup narrow recheck

## Change

Pressure-driven Cargo pool eviction now performs its transaction-time safety
recheck with four-column rows and loads a separate six-column projection only
after the target is eligible for deletion evidence. The previous path loaded
every Cargo job column, including large environment and metadata payloads, to
inspect only job identity, status, PID, target identity, and audit state. LRU
ordering, ancestor overlap, process checks, retained-pool protection, validation
copy protection, and deletion evidence remain unchanged.

## Performance evidence

Real SQLite benchmark with 2,000 job rows, two unused 4 KiB payload columns, and
31 samples. The after measurement includes both final narrow scans:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 384,165,200 ns | 11,624,900 ns | 96.974% lower, 33.05x |
| p95 | 595,333,400 ns | 39,081,800 ns | 93.435% lower, 15.23x |

Both variants preserve all 2,000 job identities and the same ordering; the
improvement isolates unused-column materialization.

## Validation

Seven pressure-eviction tests passed 7/7 in 35.130 seconds, covering successful
LRU eviction, active lease, acquire/start revalidation, overlapping running and
recorded live processes, validation-copy overlap, and unexpected deletion
failure. The static performance contract, Python compilation, and scoped diff
check passed. Run the complete cleanup module from a fresh process in the shared
asynchronous validation batch together with the immediate-cleanup optimization.

The fresh-process complete cleanup module subsequently passed 42/42 in 172.631
seconds with all three cleanup optimizations present.
