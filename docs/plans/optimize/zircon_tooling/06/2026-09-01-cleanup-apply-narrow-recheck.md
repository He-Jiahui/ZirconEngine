# Cleanup apply narrow recheck

## Change

Explicit cleanup-plan application now revalidates Cargo jobs from a ten-column
projection and operates directly on rows. The previous path loaded complete job
records and constructed `CargoJob` values, parsing command and process-tree JSON
that cleanup eligibility never uses. Exact-target history, active/live overlap,
retention timestamps, owner selection, and deletion evidence remain unchanged.

## Performance evidence

Real SQLite benchmark with 2,000 jobs, two unused 4 KiB payload columns, command
and process-tree JSON, and 31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 50,538,300 ns | 7,586,000 ns | 84.990% lower, 6.66x |
| p95 | 71,490,600 ns | 16,935,400 ns | 76.311% lower, 4.22x |
| JSON parses | 4,000 | 0 | 100.000% lower |

Both variants preserve all 2,000 job rows; the benchmark includes the JSON
parsing performed by the previous full `CargoJob` construction path.

## Validation

Seven explicit-plan tests passed 7/7 in 25.621 seconds, covering successful
deletion, process and retention revalidation, candidate non-expansion,
validation-copy overlap, untracked targets, and duplicate target attempts. The
static performance contract and Python compilation passed. Validate the complete
cleanup module once from a fresh process for all three cleanup candidates.

The fresh-process complete cleanup module subsequently passed 42/42 in 172.631
seconds with all three cleanup optimizations present.
