---
title: Runtime07 Exact Interface Import Key
category: zircon_runtime
report_id: Runtime07-exact-interface-import-key-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Interface Import Key

## Scope

This slice removes formatter growth from the owned extension-registry key created for every plugin
interface import. It preserves the `module_name=>interface_id` identity, duplicate rejection,
owner registration, late bridge binding, iteration order, and error payload.

## Change

- Build the composite import key with exact `module + delimiter + interface` capacity.
- Append the borrowed module name, `=>` delimiter, and borrowed interface ID once before the key is
  transferred to the typed extension point.
- Add a Rust exact-output regression and a Python structure contract for the registration path.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles four representative module names and four interface IDs
across 65,536 key constructions per sample. It alternates legacy and optimized order across 31
samples, counts allocator calls and requested bytes inside key construction, and asserts exact key
equality for the full 4x4 input matrix. Both paths produced checksum `107376361472`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 180,224 | 65,536 | 63.636% |
| Requested allocation bytes | 9,224,192 | 3,276,800 | 64.476% |
| Key construction P50 | 23.3470 ms | 6.0002 ms | 74.300% |
| Key construction P95 | 46.2300 ms | 11.3351 ms | 75.481% |

Evidence marker: `RUNTIME07_EXACT_INTERFACE_IMPORT_KEY_MODEL_V1`.

A second complete run remained favorable: P50 improved 73.235% and P95 improved 72.823%.

## Validation

- `python tools/tests/test_runtime07_exact_interface_import_key_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact keys for every input pair, and
  passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in the next asynchronous Runtime07
  validation batch.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
