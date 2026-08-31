---
title: Runtime07 Exact Sound Feature Identifiers
category: zircon_runtime
report_id: Runtime07-exact-sound-feature-identifiers-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Sound Feature Identifiers

## Scope

This slice removes formatter growth from the four owned identifiers built for every builtin sound
feature row. It preserves feature IDs, runtime/editor/dist module names, package distribution
metadata, capabilities, target modes, dependency order, and packaging policy.

## Change

- Add a local identifier join helper that sums borrowed part lengths, allocates the final `String`
  once, and appends every part in order.
- Route the sound feature ID and three module IDs through that exact-capacity path.
- Leave direct copies of static distribution metadata unchanged because those already allocate the
  final string exactly once.
- Add a Rust exact-output regression and a Python structure contract for the four identifier paths.

## Deterministic Performance Evidence

The standalone optimized Rust model alternates the two production sound suffixes across 65,536
rows per sample and constructs the same feature/runtime/editor/dist identifiers as the manifest
builder. It alternates legacy and optimized order across 31 samples, counts allocator calls and
requested bytes inside identifier construction, and asserts exact four-string equality for both
production suffixes. Both paths produced checksum `820352090112`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 589,824 | 327,680 | 44.444% |
| Requested allocation bytes | 24,215,552 | 11,960,320 | 50.609% |
| Identifier construction P50 | 71.6638 ms | 29.4520 ms | 58.903% |
| Identifier construction P95 | 100.2437 ms | 55.4128 ms | 44.722% |

Evidence marker: `RUNTIME07_EXACT_SOUND_FEATURE_IDENTIFIERS_MODEL_V1`.

A second complete run remained favorable: P50 improved 59.428% and P95 improved 51.547%.

## Validation

- `python tools/tests/test_runtime07_exact_sound_feature_identifiers_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact identifier arrays for both
  production suffixes, and passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch
  paired with the exact rendering feature identifiers slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
