---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/177/2026-08-26-dispatch-outcome-single-pass.md
  - docs/plans/optimize/zircon_runtime/177/2026-09-19-dispatch-host-request-capacity.md
related_code:
  - zircon_runtime/src/ui/dispatch/input_manager/outcome.rs
  - zircon_runtime/src/ui/dispatch/input_manager/outcome/single_pass_metadata_tests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/outcome/host_request_capacity_tests.rs
tests:
  - zircon_runtime/src/ui/dispatch/input_manager/outcome/host_request_capacity_tests.rs
  - tools/tests/test_runtime_dispatch_host_request_capacity_performance_contract.py
---

# Runtime820 Dispatch Host-Request Capacity

The Runtime177 single-pass dispatch metadata collector now admits host-request capacity lazily
from the remaining result-count lower bound. Request-free batches stay zero-capacity; dense and
sparse request-bearing batches avoid avoidable geometric growth while preserving request order,
redraw detection, and the existing outcome contract.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime820 | Reserve the remaining-result lower bound on the first host request without adding a second result pass | implemented_pending_validation | TDD source/model contract `3/3`; lower empty/order/sparse regression and ignored `RUNTIME820_DISPATCH_HOST_REQUEST_CAPACITY_BENCH_V1` marker are wired; `4,096`-result deterministic model changes `11→0` growth events with one reservation; managed Cargo/Release and dispatch product p50/p95/p99 evidence remain pending. |

## Complexity boundary

Only the local output-vector capacity policy changes. Host-request cloning, result traversal,
dirty-redraw short-circuiting, inherited redraw state, ordering, and public DTO semantics are
unchanged. The reservation is a lower bound rather than a speculative exact total, so unusually
dense later results may still grow normally.

## Managed gate

The earlier focused Runtime/Editor source/model batch passes `38/38`; a refreshed ten-contract
Runtime/Editor source/model recheck passes `42/42` with zero failures, errors, or skips; no
standalone Cargo command was started. Managed Windows Release compilation and product allocation/latency evidence remain
pending behind the external dirty `E:\Git\zr_vm` admission gate. No coordinator status was
polled.
