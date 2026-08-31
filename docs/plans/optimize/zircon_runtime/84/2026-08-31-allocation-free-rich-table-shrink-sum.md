---
title: Runtime84 Allocation-free Rich-table Shrink Sum
category: zircon_runtime
report_id: Runtime84-allocation-free-rich-table-shrink-sum-2026-08-31
date: 2026-08-31
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: static_passed_managed_cargo_pending
performance_status: deterministic_target_met
---

# Runtime84 Allocation-free Rich-table Shrink Sum

## Scope

The rich-table width solver collected filtered column widths into temporary vectors before every
checked sum. The binary-search shrink path repeated that allocation for all 24 scale probes. This
change keeps the existing geometry budget, minimum-width policy, and search order while folding
admission and checked accumulation directly over borrowed width iterators.

## Change

- Make `checked_sum_accumulated` consume an `Iterator<Item = f32>` and admit each extent in the
  same fold that performs checked accumulation.
- Stream fixed-width, preferred-shrink, and per-probe resolved widths without intermediate vectors.
- Preserve the sorted spanning-cell collections, which are required by the layout algorithm and
  are outside the repeated shrink-sum path.
- Refresh six current-source performance guards discovered by the same Runtime batch without
  changing their production behavior.

## Deterministic Performance Evidence

The pressure model uses 8,192 alternating fixed/shrinkable columns and the production constant of
24 shrink-scale probes. It counts only the temporary collections removed by this change.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Temporary vector allocation sites executed per fit | 26 | 0 | 100.000% |
| Temporary `f32` values written | 106,496 | 0 | 100.000% |
| Minimum temporary payload bytes | 425,984 | 0 | 100.000% |
| Source-column inspections in sum/admission work | 417,792 | 212,992 | 49.019% |

The payload value excludes allocator metadata and spare capacity, so it is a lower bound. These are
deterministic algorithm counts, not CPU, frame-time, RSS, or power measurements. The static target is
zero temporary vector allocation in both `fit_columns_to_available_width` and
`shrink_columns_to_budget`; that target is met.

## Validation

- The corrected rich-table sizing contract failed before the production change because
  `collect::<Vec<_>>()` remained in the shrink solver.
- The six-module focused batch passes 15/15 after the production and current-source guard updates.
- Python bytecode compilation and scoped diff checks pass.
- Managed Windows Cargo compilation and focused Rust behavior tests remain pending in an
  asynchronous multi-task coordinator batch.

## Remaining Work

Integrated Rust acceptance and real product-corpus latency/allocation profiling remain required
before the Runtime84 parent plan can claim end-to-end rich-table performance closure.
