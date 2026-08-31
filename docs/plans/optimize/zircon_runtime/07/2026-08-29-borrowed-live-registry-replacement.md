---
title: Runtime07 Borrowed Live Registry Replacement
category: zircon_runtime
report_id: Runtime07-borrowed-live-registry-replacement-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed Live Registry Replacement

## Scope

This slice removes redundant plugin-ID allocation when native live-host registries replace an
already loaded entry. It covers loaded plugin replacement, registration replay revision updates,
bridge generation publication, and runtime bridge binding replacement while preserving the
module-kind partition and `BTreeMap::insert` return contract.

## Change

- Use the caller's borrowed plugin ID to probe the module-kind map before constructing an owned
  key.
- Replace the existing value in place and return the previous value when the plugin ID is already
  present.
- Retain owned key construction for first insertion only.
- Add a Rust regression that proves replacement returns the previous value, updates the mapped
  value, and preserves the original stored-key allocation.
- Add a Python source performance contract for the borrowed replacement path.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles 512 loaded plugin IDs through 524,288 replacements per
sample across 31 alternating samples. Both implementations assert identical previous values,
final key/value maps, and produced checksum `16740854407524512455`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 32,768 replacements | 32,768 | 0 | 100.000% |
| Requested allocation bytes | 851,968 | 0 | 100.000% |
| Run 1 replacement P50 | 172.8583 ms | 120.9445 ms | 30.033% |
| Run 1 replacement P95 | 578.6490 ms | 406.8880 ms | 29.683% |
| Run 2 replacement P50 | 167.2998 ms | 116.0872 ms | 30.611% |
| Run 2 replacement P95 | 339.4086 ms | 250.8568 ms | 26.090% |

Evidence marker: `RUNTIME07_BORROWED_LIVE_REGISTRY_REPLACEMENT_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_borrowed_live_registry_replacement_performance_contract.py`:
  3 passed after the pre-change contract reported 1 error and 2 failures.
- The standalone Rust model asserts complete map and previous-value equality before recording
  metrics; two extended-window runs retained identical allocation profiles, checksums, and
  positive P50/P95 results.
- The Rust regression checks value replacement, previous-value return, and stable stored-key
  allocation identity.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot publication.
- Managed Rust compilation and focused native live-host tests remain pending in the asynchronous
  Runtime07 batch shared with the completed owner feature selection fast path.

Managed batch request: `runtime07-native-vm-six-task-batch-20260830-v1`.

Validation attempt: ticket `167f127a7c8d48b3a68554a5c4f1d0f7` failed during coordinator
materialization with `unmanaged_artifacts_detected` for
`D:\ZirconBuilds\mvp-test-fixtures-36724`; Cargo did not start, so integrated Rust and performance
acceptance remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
