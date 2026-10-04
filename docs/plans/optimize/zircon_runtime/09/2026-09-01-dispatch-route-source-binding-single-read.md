---
title: Runtime09 dispatch route source binding single read
category: zircon_runtime
report_id: Runtime09
date: 2026-09-01
status: local_candidate
implementation_files:
  - tools/analysis/performance/runtime/runtime_ui_dispatch_route_sharing_pressure.py
tests:
  - tools/tests/test_runtime_ui_dispatch_route_sharing_pressure.py
  - tools/tests/test_runtime_ui_dispatch_route_sharing_performance_contract.py
  - tools/tests/test_runtime_ui_dispatch_route_source_binding_single_read_performance_contract.py
---

# Runtime09 dispatch route source binding single read

## Problem

The dispatch-route evidence source binding read three critical Rust sources to
compute hashes, then `validate_candidate_source_contract` reopened all three as
text. This doubled content I/O and allowed hashes and guard validation to observe
different snapshots.

## Change

`source_binding` now reads each critical source once as bytes, derives uppercase
SHA-256 and strict UTF-8 text from the same payload, and passes the decoded source
map to the candidate contract validator. The validator retains its no-argument
compatibility path for direct callers.

## Performance evidence

Windows benchmark with three 4 MiB UTF-8 sources and 31 alternating samples:

| Measurement | Two-read path | Single-read path | Improvement |
|---|---:|---:|---:|
| p50 | 39,237,100 ns | 26,927,600 ns | 31.372% lower, 1.46x |
| p95 | 62,043,500 ns | 33,542,000 ns | 45.938% lower, 1.85x |
| Reads per source in source binding | 2 | 1 | 50.000% lower |

Every benchmark pair produced the same text-length and digest checksum.

## Validation

- RED: the new contract found file reads in the validator and no preloaded-source
  parameter.
- GREEN: pressure behavior, existing performance contract, and the new single-read
  contract passed 16/16 in 1.290 seconds.
- The candidate joins the accumulated asynchronous batch; coordinator status is
  not polled.
