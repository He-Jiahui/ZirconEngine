---
title: Runtime07 Shared Plugin Module Interner Names
category: zircon_runtime
report_id: Runtime07-shared-plugin-module-interner-names-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Shared Plugin Module Interner Names

## Scope

This slice removes duplicate owned module-name storage from `PluginModuleInterner` and prevents
registry clones from deep-copying every interned module name. The interner previously stored the
same text independently in `Vec<String>` and `HashMap<String, PluginModuleId>`; cloning the runtime
extension registry repeated both allocations for every registered module.

## Change

- Store interned names as `Arc<str>` in both the ordered name table and lookup index.
- Allocate one shared string per newly interned module and share it between both containers.
- Keep duplicate lookup ahead of shared-name construction, preserving the allocation-free
  duplicate path.
- Preserve the existing `Into<String>` registration boundary, name validation, stable module ids,
  and borrowed `name() -> Option<&str>` accessor.
- Add a Rust regression proving the ordered table, lookup index, and cloned interner share the same
  string allocation.
- Add a Python source performance contract for the shared-storage and no-deep-clone shape.

## Deterministic Performance Evidence

The standalone optimized Rust model interns 4,096 distinct module names and separately measures
one fresh interner build and 64 registry-equivalent clones over 31 alternating samples. Build and
clone sampling use independent phases so the large legacy clone heap does not contaminate build
latency. Both implementations produced identical build checksum `7538704738214717990` and clone
checksum `17962333400672977096` in both runs.

### Fresh Interner Build

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 8,215 | 8,215 | 0.000% |
| Requested allocation bytes | 999,388 | 868,380 | 13.109% |
| Run 1 P50 | 3.5578 ms | 3.3058 ms | 7.083% |
| Run 1 P95 | 10.5071 ms | 5.1417 ms | 51.065% |
| Run 2 P50 | 4.7124 ms | 4.9068 ms | -4.125% |
| Run 2 P95 | 40.3832 ms | 18.4710 ms | 54.261% |

The run-2 build P50 moved backward by 0.1944 ms while run 1 improved by 0.2520 ms. This is retained
as observed variance rather than discarded. Deterministic requested bytes improved, and build P95
improved in both runs, so there is no repeated build regression.

### Sixty-four Interner Clones

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 524,416 | 128 | 99.976% |
| Requested allocation bytes | 40,371,200 | 17,302,528 | 57.141% |
| Run 1 P50 | 124.0929 ms | 8.3679 ms | 93.257% |
| Run 1 P95 | 215.1648 ms | 16.9822 ms | 92.107% |
| Run 2 P50 | 118.6402 ms | 7.7166 ms | 93.496% |
| Run 2 P95 | 436.9013 ms | 33.8407 ms | 92.254% |

Evidence marker: `RUNTIME07_SHARED_PLUGIN_MODULE_INTERNER_NAMES_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_shared_plugin_module_interner_names_performance_contract.py`:
  4 passed after all 4 pre-change checks failed.
- `python -m py_compile` passed for the source contract.
- The standalone Rust model retained the real two-container ownership and full interner clone
  shapes; two runs kept identical allocation profiles and checksums.
- The Rust regression locks allocation identity across both indexes and a cloned interner.
- Exact-file Rust formatting, model formatting, and scoped diff checks are required before
  snapshot publication.
- Managed Runtime compilation and tests remain pending in the next asynchronous Runtime07 batch.

Managed batch request: `runtime07-plugin-five-task-batch-20260830-v1`.

Validation attempt: ticket `27e27a159794475b9bd8636cf2859288` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## Remaining Parent-plan Work

This local result does not close Runtime07 product acceptance. The parent plan still requires
deterministic resolver and catalog generations, package trust constraints, transactional lifecycle,
isolation, execution budgets, real editor/app/export/cook traces, and product-scale comparison.
