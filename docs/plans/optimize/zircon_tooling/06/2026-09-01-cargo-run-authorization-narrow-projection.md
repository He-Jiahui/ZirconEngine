# Cargo run authorization narrow projection

## Change

Cargo process registration and authorization rollback now query only the job
fields consumed by each guard. The previous paths materialized the complete
Cargo job row, including potentially large environment and metadata payloads,
before checking owner, state, command, and process identity. Authorization and
rollback predicates, error codes, and state transitions remain unchanged.

## Performance evidence

Real SQLite point-query benchmark with 2,000 jobs, three 8 KiB unused payload
columns per row, and 31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 52,195,200 ns | 23,595,600 ns | 54.794% lower, 2.21x |
| p95 | 101,320,500 ns | 44,673,800 ns | 55.908% lower, 2.27x |

Both variants performed the same primary-key lookups and decoded the same
persisted command JSON. The improvement isolates avoided unused-column
materialization.

## Validation

The complete Cargo runner module passed 16/16 in 2.012 seconds. The static
performance contract, Python compilation, and scoped `git diff --check` passed.
Broader Cargo scheduling and server integration remain in the accumulated
asynchronous coordinator validation batch.
