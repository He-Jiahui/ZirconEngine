---
title: Editor06 Borrowed Extension View Validation
category: zircon_editor
report_id: Editor06-borrowed-extension-view-validation-2026-08-27
date: 2026-08-27
session_id: root-editor06-borrowed-extension-view-validation-20260827
implementation_status: implementation_complete
validation_status: local_contract_passed_managed_validation_pending
---

# Editor06 Borrowed Extension View Validation

## Scope

This slice removes owned descriptor-ID copies from extension view batch validation. The candidate
view slice remains alive for the complete validation call, so the private duplicate-admission set
can borrow each `ViewDescriptorId` instead of cloning its String payload.

## Change

- Preallocate the pending-ID `HashSet` for the candidate view count.
- Store `&ViewDescriptorId` borrowed from the candidate slice.
- Preserve the existing registry-first check, batch duplicate order, and error payload.
- Add Rust coverage for unique batches and duplicate IDs within one candidate batch.

## Deterministic Performance Evidence

The standalone Rust model validates 32,768 unique extension view IDs across 15 alternating
legacy/optimized samples and separately verifies a duplicate appended to the batch.

| Extension view batch validation | Before | After | Reduction |
|---|---:|---:|---:|
| Descriptor ID allocations | 32,768 | 0 | 100.000% |
| Descriptor ID bytes | 3,538,944 | 0 | 100.000% |
| P50 | 24,196,900 ns | 6,687,900 ns | 72.361% |
| P95 | 101,465,300 ns | 25,082,200 ns | 75.280% |

Both implementations reject the appended duplicate at index 32,768. The final checksum is
`1,146,880`.

## Validation

- `python -m unittest tools.tests.test_editor06_borrowed_extension_view_validation_performance_contract`
  passes all three source contracts.
- Exact-file `rustfmt --edition 2021` passes.
- The standalone optimized Rust model compiles with `rustc --edition 2021 -C opt-level=3` and
  enforces at least 99% allocation/byte reduction and at least 30% P50/P95 reduction.
- Cargo execution of the two in-source Rust tests remains pending through the session coordinator.

## Remaining Parent-plan Work

Editor06 still owns lifecycle/status authority convergence, durable enablement transactions, native
reload publication, dependency validation, settings contributions, and product-scale qualification.
This slice only removes avoidable ID ownership during extension view batch validation.
