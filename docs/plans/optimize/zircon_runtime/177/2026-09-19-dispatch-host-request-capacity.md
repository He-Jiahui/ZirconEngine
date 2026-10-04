# Runtime820 Dispatch Host-Request Capacity

- Date: 2026-09-19
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Performance status: `deterministic_target_met`
- Owner path: `zircon_runtime/src/ui/dispatch/input_manager/outcome.rs`
- Related baseline: Runtime177 dispatch-outcome single-pass collection

## Problem

The Runtime177 one-pass metadata collector removed the second result scan, but its host-request
output still started at zero capacity. A request-bearing batch therefore paid geometric `Vec`
growth even when the result count supplied a useful lower bound. Empty and request-free batches
must remain allocation-free.

## Optimization

`collect_dispatch_metadata` now tracks the remaining result count and, only when the first host
request is observed, reserves the larger of that lower bound and the current request count. The
existing single result pass, request ordering, dirty-redraw short circuit, inherited redraw state,
and all public outcome fields remain unchanged. The no-request path retains zero capacity, while a
dense one-request-per-result batch reserves its lower bound with one reservation.

## Regression contract

The new lower Rust module
`zircon_runtime/src/ui/dispatch/input_manager/outcome/host_request_capacity_tests.rs` covers
ordered requests, empty-batch zero capacity, sparse suffix reservation, and the ignored managed
benchmark marker `RUNTIME820_DISPATCH_HOST_REQUEST_CAPACITY_BENCH_V1`. The Python source/model
contract is `tools/tests/test_runtime_dispatch_host_request_capacity_performance_contract.py`.
TDD RED was recorded before the production reservation existed; GREEN now passes `3/3`.

## Deterministic evidence

For `4,096` results with the first result carrying one host request, the old zero-capacity model
requires `11` geometric growth events while the lower-bound reservation requires `0`, with one
reservation and a `4,096` lower-bound capacity. Empty batches retain zero reservations. These are
allocation-shape counts, not elapsed-time or product p50/p95/p99 claims.

## Validation boundary

The Runtime820 contract joins the current Runtime/Editor focused batch, which passes `38/38` with
zero failures, errors, or skips. Scoped Rustfmt, Python compilation, and source checks pass. Cargo
was not started locally; managed Windows Release compilation, allocator evidence, and product
percentile gates remain pending because the external `E:\Git\zr_vm` dirty-worktree admission gate
is unresolved. Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/dispatch/input_manager/outcome.rs` | `D44E9709AD9F1FB5C939EF768388904152DE05B36F520DCF6AEAFB9DFC9F061C` |
| `zircon_runtime/src/ui/dispatch/input_manager/outcome/host_request_capacity_tests.rs` | `A3C10D5A5B535764D784A9572A535D66FE2687B50C316581B916E062CB0ECE41` |
| `tools/tests/test_runtime_dispatch_host_request_capacity_performance_contract.py` | `6D8671130C6A947DBDB2D7D51BED7BDD910E34DB646BF8D59CBC9FBA4E3610D5` |
