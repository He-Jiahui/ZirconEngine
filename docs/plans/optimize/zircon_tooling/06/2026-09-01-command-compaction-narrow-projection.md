# Command compaction narrow projection

## Change

Durable command-request compaction now projects only `request_id`, `status`,
`response_json`, and `error_json`. The previous queries selected every column,
including fields the compaction loop never consumes. Candidate ordering,
retention limits, tombstone generation, and update predicates remain unchanged.

## Performance evidence

Real SQLite benchmark with 500 durable terminal requests, a 16 KiB response
payload per request, and 31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 305,517,500 ns | 111,750,600 ns | 63.423% lower, 2.73x |
| p95 | 359,106,800 ns | 168,257,600 ns | 53.146% lower, 2.13x |

Both variants consumed the same 500 response payloads. The reduction comes from
avoiding materialization of unused command metadata and an additional large
payload column in the benchmark schema.

## Validation

The two directly related command-protocol compaction and retention tests passed
2/2 in 2.776 seconds. The static performance contract, Python compilation, and
scoped `git diff --check` passed. The complete command-protocol module exceeded
the local 184-second validation window without a result and remains assigned to
the accumulated asynchronous coordinator validation batch.
