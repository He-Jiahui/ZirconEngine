---
title: Runtime07 Preallocated Archive Projection
category: zircon_runtime
report_id: Runtime07-preallocated-archive-projection-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Archive Projection

## Scope

This slice removes avoidable container growth while materializing or previewing export archives.
It contributes export/cook scale evidence for Runtime07 without changing ZIP entry ordering,
compression, path validation, package discovery, or report contents.

## Change

- Preallocate generated-file and copied-package report vectors from their plan upper bounds in
  both materialization and preview paths.
- Preallocate package-directory deduplication sets from the selected native package count.
- Seed archive-entry deduplication with the generated-file plus package-report lower bound using
  saturating arithmetic; package payload entries may still grow the set as needed.
- Repair the stale Rust inventory-reuse guard to follow the current
  `inventory.file_inventory(package_id)` helper instead of requiring the removed direct scan.
- Add Rust and Python source contracts for the capacity and single-inventory behavior.

## Deterministic Performance Evidence

The standalone optimized Rust model projects 16,384 generated files and 2,048 valid native
packages into the same four containers for 17 alternating samples. It isolates report and dedup
container cost; ZIP compression, filesystem reads, and payload allocation are intentionally not
included. Both paths produced checksum `38912`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Container allocation calls | 48 | 4 | 91.667% |
| Requested allocation bytes | 958,728 | 479,264 | 50.010% |
| Projection P50 | 1.4468 ms | 0.8290 ms | 42.701% |
| Projection P95 | 2.3749 ms | 1.3255 ms | 44.187% |

Evidence marker: `RUNTIME07_PREALLOCATED_ARCHIVE_PROJECTION_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_preallocated_archive_projection_performance_contract.py`: 3
  passed after the pre-change contract failed 3 of 3 checks.
- The Rust source regression now guards the current single-inventory lookup and a new Rust
  contract covers all known capacity bounds.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07
  batch; this candidate will not be validated alone.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
