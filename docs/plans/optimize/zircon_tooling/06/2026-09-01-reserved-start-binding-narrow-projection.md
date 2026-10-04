# Reserved start binding narrow projection

## Change

Reserved Cargo start acknowledgement now loads only the reservation and job
fields used by its durable binding guards. The previous path materialized full
reservation and Cargo job rows, including large compatibility, environment,
metadata, command, and process-tree payloads. Admission, proof, command
fingerprint, request idempotency, and launch scheduling semantics remain
unchanged.

## Performance evidence

Real SQLite benchmark with 5,000 primary-key job lookups, two unused 8 KiB
payload columns, and 31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 76,802,500 ns | 37,350,600 ns | 51.368% lower, 2.06x |
| p95 | 163,396,900 ns | 108,891,100 ns | 33.358% lower, 1.50x |

Both variants returned the same owner, state, PID, and start-time bindings.

## Validation

Four acknowledgement/deadline/rollover tests passed 4/4 in 14.019 seconds. The
static performance contract, Python compilation, and scoped diff check passed.

The complete reserved-start module reported five runner-path failures because
the current local test fixture resolves managed storage to `C:\cargo-targets`,
which the shared current storage-root policy rejects as unsupported. A runtime
wrapper confirmed the persisted launch error exactly; reverting both this
candidate and the prior Cargo registration projection did not change the
failure. Re-run the complete module in the asynchronous batch with its supported
managed storage-root configuration.
