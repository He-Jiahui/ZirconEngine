---
title: Runtime07 Exact Feature Definition Key
category: zircon_runtime
report_id: Runtime07-exact-feature-definition-key-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Feature Definition Key

## Scope

This slice removes formatter growth from the runtime plugin catalog's shared
`feature_id@provider_package_id` definition key. The helper is used both when definitions enter the
catalog and when a project selection resolves an explicit or owner-default provider. Key identity,
lookup behavior, and provider selection semantics remain unchanged.

## Change

- Allocate the composite key once with exact capacity for both borrowed identities and the `@`
  separator.
- Append the feature ID, separator, and provider package ID without formatter growth.
- Add a Rust exact-output regression for normal and empty identity parts plus a Python source
  contract for the shared helper.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles five feature IDs and five provider package IDs,
including empty, short, long, and multi-segment identities, across 65,536 key constructions per
sample. It alternates legacy and optimized order across 31 samples, counts allocator calls and
requested bytes inside key construction, and asserts exact output equality for all 25 identity
combinations. Both paths produced checksum `89335739161`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 144,178 | 65,536 | 54.545% |
| Requested allocation bytes | 6,645,257 | 2,726,257 | 58.974% |
| Definition-key construction P50 | 19.2119 ms | 5.7570 ms | 70.034% |
| Definition-key construction P95 | 25.4247 ms | 8.1892 ms | 67.790% |

Evidence marker: `RUNTIME07_EXACT_FEATURE_DEFINITION_KEY_MODEL_V1`.

A second complete run remained favorable: P50 improved 69.645% and P95 improved 65.423%.

## Validation

- `python tools/tests/test_runtime07_exact_feature_definition_key_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact keys for all representative
  identity combinations, and passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch
  paired with the exact event catalog namespace slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
