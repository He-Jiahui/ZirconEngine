---
title: Runtime render-graph transient allocation output capacity
category: zircon_runtime
report_id: Runtime840-transient-allocation-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime840 · render-graph transient allocation output capacity

## Scope

`allocate_transient_lifetimes` filters and sorts the complete lifetime slice
before emitting exactly one compiled allocation per retained lifetime. The
output vector previously started at zero capacity even though the filtered
input length is an exact upper bound.

## Implementation

- Reserve `lifetimes.len()` after the existing import/persistent filters and
  stable interval sort.
- Preserve slot reuse, allocation IDs, interval ordering, resource-size
  validation, and all bucket/materialization semantics.
- Add a folder-backed lower source regression and the ignored
  `RUNTIME840_TRANSIENT_ALLOCATION_CAPACITY_BENCH_V1` Release marker.

This is a narrow compiler-local allocation change. It does not alter render
graph authority, transient aliasing proof, bucket identity, or backend/RHI
materialization. Tooling production remains out of scope.

## Deterministic work model

For 4,096 retained lifetimes, the old zero-capacity collector models 11
geometric growth events; the exact filtered bound changes that to `11→0`.
Empty input remains zero-capacity. This is allocation-shape evidence only, not
allocator, CPU, RSS, GPU, or product p50/p95/p99 evidence.

## TDD and local evidence

- The Python source/model contract was intentionally RED against the original
  `Vec::new()` output collector and GREEN after the bound and lower-test wiring
  were added (`2/2`).
- The lower Rust source/order regression and ignored Release marker are wired
  in `transient_allocation/capacity_tests.rs`.
- Exact-file Rustfmt and Python compilation pass. The focused Runtime/Editor
  batch containing this slice and the adjacent recent slices passes `46/46`;
  the one-process broad non-tooling loader covers `933` modules and passes
  `3907/3907` tests in `62.574s`, with zero failures, errors, load errors, or
  skips.
- Managed Cargo/Windows Release, allocator, and render-graph product
  percentile evidence remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/render_graph/graph/transient_allocation.rs` | `D4BDBF1BA9E9B43382C90FB1144739CBB20FF61B0D7AEE3337C92947499C6015` |
| `zircon_runtime/src/render_graph/graph/transient_allocation/capacity_tests.rs` | `12C81B13BC8FA1BCC8C31FC2F52605481D43F7A4006E704B228BC0D54DCEB79F` |
| `tools/tests/test_runtime_transient_allocation_capacity_performance_contract.py` | `7311AEF4259E0AA8A56C421B656038F6FFEF9A2254ACAE74E2BAE8B3D1CB59DA` |

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, allocation
parity, and the declared render-graph product p50/p95/p99 gates.
