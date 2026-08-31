---
title: Runtime07 Exact Particles Feature Identifiers
category: zircon_runtime
report_id: Runtime07-exact-particles-feature-identifiers-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Particles Feature Identifiers

## Scope

This slice removes formatter growth from all three built-in particles feature identities. It
preserves the `particles.<suffix>` identity, dependency declarations, capabilities, display names,
and optional-feature catalog order.

## Change

- Build particles feature IDs with exact capacity from the borrowed namespace and row suffix.
- Keep manifest ownership transfer and dependency construction unchanged.
- Add a Rust exact-output regression for the longest production suffix plus a Python source
  contract covering exact construction and formatter removal.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles all three production particles feature suffixes across
65,536 identity constructions per sample. It alternates legacy and optimized order across 31
samples, counts allocator calls and requested bytes inside construction, and asserts exact output
equality for every feature ID. Both paths produced checksum `48676820307`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 109,226 | 65,536 | 40.000% |
| Requested allocation bytes | 3,058,320 | 1,485,477 | 51.428% |
| Identifier construction P50 | 12.7335 ms | 5.1154 ms | 59.827% |
| Identifier construction P95 | 28.9775 ms | 7.4401 ms | 74.325% |

Evidence marker: `RUNTIME07_EXACT_PARTICLES_FEATURE_IDENTIFIERS_MODEL_V1`.

A second complete run remained favorable: P50 improved 56.624% and P95 improved 67.362%.

## Validation

- `python tools/tests/test_runtime07_exact_particles_feature_identifiers_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact feature IDs for all
  production suffixes, and passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07
  batch paired with another completed optimization slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
