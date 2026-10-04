# Require-owned-live lease index

## Change

Exact ownership validation now asks SQLite only for the requested Session's
unexpired leases, builds one immutable key index, and checks each requested path
through its separator-bounded ancestor keys. Previously every requested path
rescanned every Session's lease row and repeatedly parsed expiration timestamps,
making validation O(requested paths x all leases). Directory leases continue to
satisfy child-path checks; child leases do not satisfy parent checks; the error
code and missing-path detail remain unchanged.

## Performance evidence

Windows in-memory SQLite benchmark with 4,000 other-owner live leases, 4,000
target-owner expired leases, one target-owner live directory lease, and 256
requested child paths:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 877,010,600 ns | 1,212,400 ns | 99.862% lower, 723.37x |
| p95 | 939,344,100 ns | 1,659,100 ns | 99.823% lower, 566.18x |
| Search shape | paths x all lease rows | filtered rows + path depth | indexed ancestor membership |

## Validation

The performance contracts and three core lease behaviors passed 5/5 in 25.704
seconds. The broader IntegrationCandidate consumer suite was submitted as part
of the accumulated asynchronous validation and is not claimed green here.
