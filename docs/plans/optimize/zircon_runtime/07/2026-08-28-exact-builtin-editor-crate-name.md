---
title: Runtime07 Exact Built-in Editor Crate Name
category: zircon_runtime
report_id: Runtime07-exact-builtin-editor-crate-name-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Built-in Editor Crate Name

## Scope

This slice removes formatter growth from editor crate names generated for every package in
`PluginPackageManifest::builtin_catalog()`. It preserves descriptor iteration, package manifest
projection, package identity, output order, and the `zircon_plugin_<package>_editor` convention.

## Change

- Allocate each editor crate name once with exact capacity for the static prefix, borrowed package
  ID, and static suffix.
- Route the complete built-in package catalog map through the exact-capacity helper.
- Add a Rust exact-output regression plus a Python source contract covering helper construction and
  formatter removal.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles eight representative package IDs, including short and
long production-shaped identities, across 65,536 editor crate-name constructions per sample. It
alternates legacy and optimized order across 31 samples, counts allocator calls and requested
bytes inside construction, and asserts exact output equality for every identity. Both paths
produced checksum `78385520640`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 81,920 | 65,536 | 20.000% |
| Requested allocation bytes | 4,128,768 | 2,392,064 | 42.063% |
| Crate-name construction P50 | 9.8186 ms | 5.0087 ms | 48.988% |
| Crate-name construction P95 | 24.5228 ms | 10.8826 ms | 55.623% |

Evidence marker: `RUNTIME07_EXACT_BUILTIN_EDITOR_CRATE_NAME_MODEL_V1`.

A second complete run remained favorable: P50 improved 50.125% and P95 improved 66.957%.

## Validation

- `python tools/tests/test_runtime07_exact_builtin_editor_crate_name_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact crate names for every
  representative package ID, and passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch
  paired with the exact particles feature identifier slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
