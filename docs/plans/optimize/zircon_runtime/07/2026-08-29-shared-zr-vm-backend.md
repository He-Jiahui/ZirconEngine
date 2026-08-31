---
title: Runtime07 Shared ZrVM Backend
category: zircon_runtime
report_id: Runtime07-shared-zr-vm-backend-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Shared ZrVM Backend

## Scope

This slice removes repeated `Arc` control-block allocation when the real ZrVM backend family
resolves `zr_vm:project` or its `project` alias. The backend object is stateless; runtime ownership
continues to live in each package instance returned by `load_package`.

## Change

- Store the stateless ZrVM backend in a process-wide `LazyLock<Arc<dyn VmBackend>>`.
- Return an `Arc` reference-count clone for both canonical and alias selectors.
- Preserve package validation, feature-gated real backend loading, unavailable-backend errors, and
  unknown-selector ownership.
- Add a Rust regression proving canonical and alias selectors share the same control block.
- Add a Python source performance contract that rejects per-resolution backend allocation.

## Deterministic Performance Evidence

The standalone optimized Rust model alternates the canonical and alias selectors through 524,288
resolutions per sample across 31 alternating samples. Lazy initialization is completed before
allocation and timing samples. Both implementations produced checksum `1761162118523396901`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 65,536 resolutions | 65,536 | 0 | 100.000% |
| Requested allocation bytes | 1,048,576 | 0 | 100.000% |
| Run 1 resolution P50 | 46.3621 ms | 4.0094 ms | 91.352% |
| Run 1 resolution P95 | 61.7267 ms | 6.9843 ms | 88.685% |
| Run 2 resolution P50 | 44.3250 ms | 4.2201 ms | 90.479% |
| Run 2 resolution P95 | 89.3250 ms | 9.8683 ms | 88.952% |

Evidence marker: `RUNTIME07_SHARED_ZR_VM_BACKEND_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_shared_zr_vm_backend_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model initializes the shared instance before measurement, asserts canonical
  and alias pointer identity, and retained identical checksums and positive P50/P95 results across
  two runs.
- The Rust regression covers canonical/alias storage identity without invoking package loading.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot publication.
- Managed Rust compilation and focused ZrVM backend-family tests remain pending in the
  asynchronous Runtime07 batch shared with builtin VM backend resolution.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
