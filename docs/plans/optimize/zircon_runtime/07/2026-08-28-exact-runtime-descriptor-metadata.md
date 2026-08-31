---
title: Runtime07 Exact Runtime Descriptor Metadata
category: zircon_runtime
report_id: Runtime07-exact-runtime-descriptor-metadata-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Runtime Descriptor Metadata

## Scope

This slice removes formatter growth from the shared `RuntimePluginDescriptorBuilder::new` path used
by the built-in runtime plugin catalog. It preserves the `<package>.runtime` module identity,
`Runtime plugin module for <display>` description, descriptor fields, initialization level, and
builder behavior.

## Change

- Build both module metadata strings with exact capacity from borrowed parts.
- Materialize the module ID and description before passing ownership to `ModuleDescriptor`.
- Keep all remaining descriptor construction unchanged.
- Add a Rust exact-output regression plus a Python source contract covering both strings and
  formatter removal.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles five representative package/display-name pairs across
65,536 descriptor metadata constructions per sample. Each row produces and verifies both the
module ID and description. It alternates legacy and optimized order across 31 samples, counts
allocator calls and requested bytes inside construction, and asserts exact output equality. Both
paths produced checksum `131428083284`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 196,608 | 131,072 | 33.333% |
| Requested allocation bytes | 6,317,650 | 4,010,786 | 36.515% |
| Metadata construction P50 | 27.0034 ms | 12.7082 ms | 52.939% |
| Metadata construction P95 | 65.1724 ms | 22.5826 ms | 65.349% |

Evidence marker: `RUNTIME07_EXACT_RUNTIME_DESCRIPTOR_METADATA_MODEL_V1`.

A second complete run remained favorable: P50 improved 53.842% and P95 improved 6.707%.

## Validation

- `python tools/tests/test_runtime07_exact_runtime_descriptor_metadata_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact module IDs and descriptions,
  and passed two complete 31-sample runs.
- Existing Runtime42 borrowed built-in package-ID and descriptor provided-interface source
  contracts remain structurally compatible with this change.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch
  paired with the exact package coordinate ID slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
