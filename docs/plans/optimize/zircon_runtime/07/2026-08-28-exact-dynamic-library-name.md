---
title: Runtime07 Exact Dynamic Library Name
category: zircon_runtime
report_id: Runtime07-exact-dynamic-library-name-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Dynamic Library Name

## Scope

This slice removes formatter growth from default native plugin library names constructed during
manifest candidate discovery. The shared helper serves both project and dist library paths and
preserves Windows, macOS, and other Unix naming conventions.

## Change

- Allocate each dynamic library name once with exact capacity for the borrowed platform prefix,
  crate name, and platform suffix.
- Route all three target branches through the same exact-capacity helper.
- Add a Rust regression that validates all platform conventions on every host plus a Python source
  contract covering shared construction and formatter removal.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles five crate names, including empty, short, and long
production-shaped identities, across all three platform naming conventions and 65,536 name
constructions per sample. It alternates legacy and optimized order across 31 samples, counts
allocator calls and requested bytes inside construction, and asserts exact output equality for all
15 name/platform combinations. Both paths produced checksum `65570869494`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 135,440 | 65,536 | 51.613% |
| Requested allocation bytes | 5,382,616 | 2,001,006 | 62.825% |
| Library-name construction P50 | 16.0650 ms | 5.8865 ms | 63.358% |
| Library-name construction P95 | 24.4069 ms | 8.4598 ms | 65.338% |

Evidence marker: `RUNTIME07_EXACT_DYNAMIC_LIBRARY_NAME_MODEL_V1`.

A second complete run remained favorable: P50 improved 63.824% and P95 improved 47.950%.

## Validation

- `python tools/tests/test_runtime07_exact_dynamic_library_name_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact names for every platform and
  representative crate identity, and passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07
  batch paired with another completed optimization slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
