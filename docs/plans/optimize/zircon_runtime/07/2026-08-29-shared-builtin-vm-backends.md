---
title: Runtime07 Shared Builtin VM Backends
category: zircon_runtime
report_id: Runtime07-shared-builtin-vm-backends-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Shared Builtin VM Backends

## Scope

This slice removes repeated `Arc` control-block allocation when the builtin VM backend family
resolves its stateless mock and unavailable backends. It covers direct backend resolution and the
backend-registry `contains` path, which intentionally resolves a selector to prove availability.

## Change

- Store the two stateless builtin backends in process-wide `LazyLock<Arc<dyn VmBackend>>`
  instances.
- Return an `Arc` reference-count clone for both canonical and alias selectors.
- Preserve unknown-selector error ownership and the existing selector list.
- Add a Rust regression proving canonical and alias selectors share storage while distinct backend
  kinds do not.
- Add a Python source performance contract that rejects per-resolution backend allocation.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles the four canonical/alias selectors through 524,288
resolutions per sample across 31 alternating samples. Lazy initialization is completed before
allocation and timing samples. Both implementations produced checksum `12294445493382947621`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 65,536 resolutions | 65,536 | 0 | 100.000% |
| Requested allocation bytes | 1,048,576 | 0 | 100.000% |
| Run 1 resolution P50 | 49.3691 ms | 5.0277 ms | 89.816% |
| Run 1 resolution P95 | 201.7625 ms | 37.9918 ms | 81.170% |
| Run 2 resolution P50 | 47.4072 ms | 5.0428 ms | 89.363% |
| Run 2 resolution P95 | 86.5483 ms | 7.7300 ms | 91.069% |

Evidence marker: `RUNTIME07_SHARED_BUILTIN_VM_BACKENDS_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_shared_builtin_vm_backends_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model initializes shared instances before measurement, asserts canonical and
  alias pointer identity, and retained identical checksums and positive P50/P95 results across two
  runs.
- The Rust regression covers both backend aliases and distinct-backend storage separation.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot publication.
- Managed Rust compilation and focused backend-family tests remain pending in the next
  asynchronous Runtime07 validation batch.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
