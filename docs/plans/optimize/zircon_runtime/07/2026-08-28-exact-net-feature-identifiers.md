---
title: Runtime07 Exact Network Feature Identifiers
category: zircon_runtime
report_id: Runtime07-exact-net-feature-identifiers-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Network Feature Identifiers

## Scope

This slice removes formatter growth and an intermediate feature-ID clone from all six built-in
network feature manifests. It preserves the `net.<suffix>` feature identity, corresponding
`.runtime` module identity, dependency declarations, capabilities, target modes, and runtime crate
selection.

## Change

- Build network feature and runtime module IDs with exact capacity from borrowed parts.
- Construct the runtime module ID before transferring feature-ID ownership to the manifest.
- Move the feature ID into the manifest instead of cloning the full string.
- Add a Rust exact-output regression plus a Python contract covering shared construction and the
  clone-free ownership path.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles all six production network feature suffixes across
65,536 manifest-identity constructions per sample. The legacy path models the feature formatter,
manifest-owned feature clone, and runtime-module formatter; the optimized path retains only two
exact-capacity output strings. It alternates legacy and optimized order across 31 samples, counts
allocator calls and requested bytes, and asserts exact equality for both output identities. Both
paths produced checksum `73730970958`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 305,834 | 131,072 | 57.143% |
| Requested allocation bytes | 4,751,309 | 2,250,050 | 52.644% |
| Identifier construction P50 | 34.8850 ms | 12.3969 ms | 64.464% |
| Identifier construction P95 | 74.4482 ms | 26.4213 ms | 64.510% |

Evidence marker: `RUNTIME07_EXACT_NET_FEATURE_IDENTIFIERS_MODEL_V1`.

A second complete run remained favorable: P50 improved 65.489% and P95 improved 32.455%.

## Validation

- `python tools/tests/test_runtime07_exact_net_feature_identifiers_performance_contract.py`:
  3 passed after the pre-change contract failed 2 checks and raised 1 expected missing-helper
  error.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact feature/runtime IDs for all
  production suffixes, and passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch
  paired with the exact dynamic library name slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
