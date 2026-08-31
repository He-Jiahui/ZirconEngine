---
title: Runtime07 Borrowed ZrVM Lifecycle Entry Module
category: zircon_runtime
report_id: Runtime07-borrowed-zrvm-lifecycle-entry-module-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed ZrVM Lifecycle Entry Module

## Scope

This slice removes the entry-module `String` allocation performed by every real ZrVM lifecycle
call. The optional-export helper previously borrowed the entire plugin instance mutably, forcing
`call_entry_lifecycle_export` to clone `self.entry_module` before invoking it.

## Change

- Narrow the optional-export helper from `&mut self` to `&mut ZrVmRuntimeOwner`.
- Borrow `runtime_owner` mutably and `entry_module` immutably as disjoint instance fields.
- Route both lifecycle and general export calls through the same field-scoped helper.
- Preserve the process-wide VM lock, optional-export detection, argument forwarding, result
  ownership, and ZrVM error mapping.
- Add a Python source performance contract for the field-scoped borrowed call shape.

## Deterministic Performance Evidence

The standalone optimized Rust model executes 262,144 lifecycle-equivalent calls over 31
alternating samples. Each call retains the runtime-owner mutation and reads the same 21-byte entry
module; instance construction is outside the timed and allocation-profiled region. Both
implementations produced checksum `17121146977712227801` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 262,144 calls | 262,144 | 0 | 100.000% |
| Requested allocation bytes | 5,505,024 | 0 | 100.000% |
| Run 1 P50 | 22.2834 ms | 1.8778 ms | 91.573% |
| Run 1 P95 | 34.2845 ms | 2.5788 ms | 92.478% |
| Run 2 P50 | 23.7268 ms | 1.8852 ms | 92.055% |
| Run 2 P95 | 38.4916 ms | 3.3845 ms | 91.207% |

Evidence marker: `RUNTIME07_BORROWED_ZRVM_LIFECYCLE_ENTRY_MODULE_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_borrowed_zrvm_lifecycle_entry_module_performance_contract.py`:
  4 passed after all 4 pre-change checks failed.
- The standalone Rust model preserves owner mutation and entry-module reads; two runs kept
  identical allocation profiles and checksums with positive P50/P95 results.
- Exact-file Rust/model formatting, Python compilation, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed tests must compile the real `backend-zr-vm` feature; fallback-only compilation does not
  validate this source file.

## Remaining Parent-plan Work

This local allocation removal does not resolve the process-global ZrVM lock, execution budgets,
typed ABI, debugger/profiler surface, or product-scale editor/app/export/cook acceptance owned by
the Runtime07 parent plan.
