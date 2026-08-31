---
title: Runtime07 Exact Plugin Module Builder Metadata
category: zircon_runtime
report_id: Runtime07-exact-plugin-module-builder-metadata-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Plugin Module Builder Metadata

## Scope

This slice removes formatting machinery and geometric string growth from the plugin SDK module
manifest constructors. Runtime, editor, native, and VM module names all append fixed suffixes,
and their default description prepends one fixed phrase, so the exact output length is known.

## Change

- Add one private metadata joiner that sums part lengths and allocates the final string once.
- Route runtime, editor, native, and VM module-name construction through that joiner.
- Use the same joiner for the default `Plugin module <name>` description.
- Preserve module names, descriptions, kinds, crate names, target defaults, and builder behavior.
- Add a Rust regression covering the names and descriptions of all four built-in module kinds.
- Add a Python source performance contract for exact capacity and the absence of production
  `format!` paths.

## Deterministic Performance Evidence

The standalone optimized Rust model builds all four module metadata variants 131,072 times per
sample across 31 alternating samples. The package id is intentionally long enough to exercise
the old formatting growth path. Both implementations produced checksum
`17149695307903222528`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 65,536 four-kind builds | 1,048,576 | 524,288 | 50.000% |
| Requested allocation bytes | 68,222,976 | 34,734,080 | 49.087% |
| Run 1 build P50 | 306.2564 ms | 111.4695 ms | 63.603% |
| Run 1 build P95 | 457.7750 ms | 261.3060 ms | 42.918% |
| Run 2 build P50 | 351.1075 ms | 137.3248 ms | 60.888% |
| Run 2 build P95 | 437.8178 ms | 172.8641 ms | 60.517% |

Evidence marker: `RUNTIME07_EXACT_PLUGIN_MODULE_BUILDER_METADATA_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_exact_plugin_module_builder_metadata_performance_contract.py`:
  4 passed after the pre-change contract rejected the missing exact-capacity joiner.
- `python -m py_compile` passed for the source contract.
- The standalone Rust model retained identical output for runtime, editor, native, and VM
  metadata; two runs kept the same allocation profile, checksum, and positive P50/P95 results.
- The Rust regression locks all four generated names and default descriptions.
- Exact-file Rust formatting, model formatting, and scoped diff checks passed before snapshot
  publication.
- Managed plugin SDK compilation and tests remain pending in the next asynchronous Runtime07
  validation batch.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
