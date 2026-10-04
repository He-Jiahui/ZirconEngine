# Cargo invalid pending SQL filter

## Change

Cargo lane reconciliation now asks SQLite only for pending reservations whose
owner is missing/non-executable or whose absolute TTL has elapsed. The previous
path materialized every pending reservation in the lane and discarded healthy
rows in Python. Per-row compare-and-set updates, audit events, FIFO ordering,
and the owner-invalid-before-TTL reason priority remain unchanged.

## Performance evidence

Real SQLite benchmark with 10,000 pending reservations, 100 invalid candidates,
and 31 samples:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 40,781,300 ns | 10,964,400 ns | 73.114% lower, 3.72x |
| p95 | 74,267,700 ns | 15,534,400 ns | 79.083% lower, 4.78x |
| Python rows | 10,000 | 100 | 99.000% lower |

The benchmark asserted identical reservation identifiers and reconciliation
reasons before recording timings.

## Validation

The focused reconciliation test passed and covers invalid owners, elapsed TTLs,
healthy pending preservation, one-time terminalization, and audit reasons. The
static performance contract, Python compilation, and scoped diff check passed.
The complete Cargo reservations module remained active beyond the local batch
window and stays assigned to the asynchronous coordinator validation batch.
