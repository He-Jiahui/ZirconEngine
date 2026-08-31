---
title: Runtime07 Size-hint Feature Capability Batch
category: zircon_runtime
report_id: Runtime07-size-hint-feature-capability-batch-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Size-hint Feature Capability Batch

## Scope

This slice lets the plugin SDK use an input iterator's size hint while appending a batch of
feature capabilities. The old builder re-entered the single-item method for every capability,
forcing the destination vector through its normal geometric growth sequence.

## Change

- Extend the manifest capability vector directly from the converting iterator.
- Preserve existing capabilities, input order, duplicates, and owned `String` output.
- Keep the single-capability builder unchanged for incremental call sites.
- Add a Rust regression for an existing capability followed by an ordered two-item batch.
- Add a Python source performance contract for the size-hint-aware append.

## Deterministic Performance Evidence

The standalone optimized Rust model builds 16,384 capability lists per sample across 31
alternating samples. Each list starts with one existing capability and appends 32 owned strings.
Required string allocations remain in both implementations; the comparison isolates vector
growth. Both implementations produced checksum `15906530942699860029`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 4,096 builds | 159,744 | 143,360 | 10.256% |
| Requested allocation bytes | 19,562,496 | 10,616,832 | 45.729% |
| Run 1 build P50 | 94.9443 ms | 70.8315 ms | 25.397% |
| Run 1 build P95 | 321.2615 ms | 229.1496 ms | 28.672% |
| Run 2 build P50 | 64.8248 ms | 55.4493 ms | 14.463% |
| Run 2 build P95 | 93.8122 ms | 69.9982 ms | 25.385% |

Evidence marker: `RUNTIME07_SIZE_HINT_FEATURE_CAPABILITY_BATCH_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_size_hint_feature_capability_batch_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model preserves required string ownership and equivalent ordered output;
  two runs retained identical allocation profiles, checksums, and positive P50/P95 results.
- The Rust regression locks existing-item retention and batch input order.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot publication.
- Managed plugin SDK compilation and tests remain pending in the next asynchronous Runtime07
  validation batch.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
