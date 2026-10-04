# Cleanup active job SQL filter

## Change

Immediate Cargo cleanup now reloads only active jobs for its transaction-time
safety gate, rather than materializing every historical Cargo job and filtering
statuses in Python. If deletion remains eligible, a separate six-column scan
supplies the complete target-overlap audit evidence without loading large unused
environment or metadata payloads. Process checks, target ancestor overlap,
retained-pool protection, deletion evidence, and compare-and-set behavior remain
unchanged.

## Performance evidence

Real SQLite benchmark with 2,000 jobs, 20 active jobs, two unused 4 KiB payload
columns, and 31 samples. The after path includes both the active full-row query
and the six-column evidence scan used by the final implementation:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 348,861,300 ns | 6,741,600 ns | 98.068% lower, 51.75x |
| p95 | 675,775,000 ns | 9,332,000 ns | 98.619% lower, 72.41x |
| Active full rows | 2,000 | 20 | 99.000% lower |
| Audit rows | same full rows | 2,000 narrow rows | large payloads omitted |

The benchmark asserted identical active job identifiers.

## Validation

Four focused cleanup tests passed in 40.155 seconds, covering immediate delete,
active-job denial, and retained parent/child overlap protection. The static
performance contract, Python compilation, and scoped diff check passed.

An earlier full-module process imported the first implementation before its
audit-row fix. It later reported ten identical `all_rows` `NameError` failures;
the missing audit source was then replaced by the explicit narrow evidence scan,
and all focused paths passed after re-import. The asynchronous batch must run the
complete cleanup module from a fresh process against the current source.

The requested fresh-process rerun subsequently passed the complete cleanup
module 42/42 in 172.631 seconds with all three cleanup optimizations present.
