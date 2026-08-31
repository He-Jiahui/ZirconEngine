---
title: Runtime07 In-place Behavior Callback Cache
category: zircon_runtime
report_id: Runtime07-in-place-behavior-callback-cache-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 In-place Behavior Callback Cache

## Scope

This slice removes redundant callback-key cloning when a cached script behavior callback receives a
new generation or VM handle. Cache misses still own the provider-qualified callback identity; cache
hits now replace only the Copy handle.

## Change

- Centralize callback cache writes in one helper.
- Update an existing `BTreeMap` entry through borrowed lookup without cloning its package and node
  strings.
- Clone the two-string callback key only for a genuine cache miss.
- Route both resolver generation refresh and post-invocation handle refresh through the helper.
- Preserve cache miss insertion, manager rebinding invalidation, generation checks, and callback
  resolution behavior.
- Add a Rust existing/missing-key regression plus a Python source contract.

## Deterministic Performance Evidence

The standalone optimized Rust model isolates the callback cache-write layer with 128 cached
provider-qualified keys, a representative 70-byte two-string key, and a Copy handle. Each sample
performs 131,072 generation refreshes, alternates legacy and optimized order across 31 samples,
counts allocations for one refresh, and verifies identical final handles and rolling checksums. It
deliberately excludes shared manager resolution and VM invocation costs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Existing-key refresh allocation calls | 2 | 0 | 100% |
| Existing-key refresh requested bytes | 70 | 0 | 100% |
| Cache refresh P50 | 51.2882 ms | 24.3306 ms | 52.561% |
| Cache refresh P95 | 79.3864 ms | 76.3196 ms | 3.863% |

Evidence marker: `RUNTIME07_IN_PLACE_BEHAVIOR_CALLBACK_CACHE_MODEL_V1`.

Two additional complete runs remained favorable. The second improved P50/P95 by
64.886%/44.195%; the third improved them by 55.604%/68.383%. All paths produced checksum
`15229812157611835392`.

## Validation

- `python tools/tests/test_runtime07_in_place_behavior_callback_cache_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust 1.94.1 model compiled and passed three complete 31-sample runs with identical
  final handles and checksums.
- The Rust guard verifies that an existing key reports an in-place update and a missing key reports
  insertion while preserving both cached handles.
- Existing `test_plugins08_vm_active_interface_snapshot.py` completed 8 of 9 checks; its sole
  failure is an unrelated stale `JobScheduler::process_io()` expectation against the already-dirty
  `vm_plugin_package_discovery/io.rs`, which this slice does not edit.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch.

Managed batch request: `runtime07-borrowed-gameplay-seven-task-batch-20260830-v1`.

Validation attempt: ticket `a9dc9a55e9044c239cc7dfda8bbc64b6` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; the 22 local contract
checks remain green while integrated acceptance and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
