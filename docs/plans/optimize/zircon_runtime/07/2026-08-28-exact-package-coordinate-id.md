---
title: Runtime07 Exact Package Coordinate ID
category: zircon_runtime
report_id: Runtime07-exact-package-coordinate-id-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Package Coordinate ID

## Scope

This slice removes formatter growth from `PluginPackageManifest::package_id()` when all package
coordinate fields are present. The method has 33 production call sites across catalog,
registration, validation, and reporting paths. It preserves the qualified
`prefix.company.name` identity and the existing legacy-ID clone fallback when any coordinate field
is empty.

## Change

- Allocate a complete package coordinate once with exact capacity for all three borrowed segments
  and two dot separators.
- Append the prefix, company, and name directly without formatter growth.
- Keep incomplete-coordinate detection and `self.id.clone()` fallback unchanged.
- Add a Rust regression for both qualified and fallback identities plus a Python source contract.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles five representative complete coordinate tuples,
including short and long multi-segment identities, across 65,536 package-ID constructions per
sample. It alternates legacy and optimized order across 31 samples, counts allocator calls and
requested bytes inside construction, and asserts exact output equality for every tuple. Both paths
produced checksum `68720577722`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 209,714 | 65,536 | 68.750% |
| Requested allocation bytes | 6,514,203 | 2,097,134 | 67.807% |
| Package-ID construction P50 | 30.9305 ms | 6.5882 ms | 78.700% |
| Package-ID construction P95 | 53.1715 ms | 14.6110 ms | 72.521% |

Evidence marker: `RUNTIME07_EXACT_PACKAGE_COORDINATE_ID_MODEL_V1`.

A second complete run remained favorable: P50 improved 79.237% and P95 improved 80.469%.

## Validation

- `python tools/tests/test_runtime07_exact_package_coordinate_id_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact complete-coordinate IDs, and
  passed two complete 31-sample runs; the Rust regression separately covers the unchanged fallback.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07
  batch paired with another completed optimization slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
