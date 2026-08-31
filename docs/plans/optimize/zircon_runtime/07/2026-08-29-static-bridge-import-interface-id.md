---
title: Runtime07 Static Bridge Import Interface ID
category: zircon_runtime
report_id: Runtime07-static-bridge-import-interface-id-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Static Bridge Import Interface ID

## Scope

This slice removes the owned interface-id allocation from typed bridge import construction.
`PluginInterface::INTERFACE_ID` is contractually `&'static str`, but the erased internal import
previously copied it into a new `String` before catalog binding.

## Change

- Store the erased import interface id as `&'static str`.
- Borrow `T::INTERFACE_ID` directly instead of calling `to_string()`.
- Preserve the existing borrowed `interface_id() -> &str` lookup API.
- Preserve binding and update ownership; both `Arc` allocations remain unchanged.
- Add a Rust regression proving the erased import and its clone retain the original static string
  address.
- Add a Python source performance contract for the allocation-free interface-id path.

## Deterministic Performance Evidence

The standalone optimized Rust model creates 131,072 imports for allocation profiling and 65,536
imports per latency sample across 31 alternating samples. It retains the two representative `Arc`
allocations used by the binding and update callback, so the comparison isolates only the owned
interface id and the resulting import storage size. Both implementations produced checksum
`7352184942339731496`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 131,072 imports | 393,217 | 262,145 | 33.333% |
| Requested allocation bytes | 16,908,288 | 10,485,760 | 37.984% |
| Run 1 construction P50 | 36.7519 ms | 20.1057 ms | 45.293% |
| Run 1 construction P95 | 69.2069 ms | 31.6877 ms | 54.213% |
| Run 2 construction P50 | 37.4250 ms | 19.5298 ms | 47.816% |
| Run 2 construction P95 | 76.6574 ms | 40.7346 ms | 46.861% |

Evidence marker: `RUNTIME07_STATIC_BRIDGE_IMPORT_INTERFACE_ID_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_static_bridge_import_interface_id_performance_contract.py`:
  4 passed after 3 of 4 pre-change checks failed and the existing borrowed accessor check passed.
- `python -m py_compile` passed for the source contract.
- The standalone Rust model retained the real two-`Arc` construction shape; two runs kept the
  same allocation profile and checksum, with positive P50/P95 results in both runs.
- The Rust regression locks static-address identity across the erased import and its clone.
- Exact-file Rust formatting, model formatting, and scoped diff checks are required before
  snapshot publication.
- Managed Runtime compilation and tests remain pending in the next asynchronous Runtime07 batch.

Managed batch request: `runtime07-plugin-five-task-batch-20260830-v1`.

Validation attempt: ticket `27e27a159794475b9bd8636cf2859288` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
