---
title: Runtime07 GC Frame Reporting
category: zircon_runtime
report_id: Runtime07-gc-frame-reporting-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 GC Frame Reporting

## Scope

This slice removes repeated allocation and traversal work from the cooperative VM GC frame path.
The pending queue already exposes the maximum number of reports that one frame can produce, while
`VmGcStepReport::from_slots` previously traversed the completed report slice three times to derive
pause, root, and cross-boundary-reference totals.

## Change

- Append due slots to the pending `VecDeque` with `extend`, preserving the existing stable order.
- Read the combined pending length while holding the same lock and allocate `slot_reports` with
  that upper bound instead of geometric growth from `Vec::new()`.
- Aggregate all three saturating counters in one `fold` instead of three complete slice passes.
- Preserve frame budgets, deadline checks, retry ordering, panic/error restoration, slot report
  order, saturating totals, and public diagnostic fields.
- Add a source performance contract covering the queue-derived capacity and single-pass aggregate.

## Deterministic Performance Evidence

The standalone optimized Rust model processes 32,768 frames with 128 reports per frame and 41
alternating baseline/candidate samples after warmup. The baseline uses geometric `Vec` growth and
three folds; the candidate uses one exact allocation and one tuple fold. Both paths produced
checksum `481046445312` in both recorded runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per sample | 196,608 | 32,768 | 83.333% |
| Requested allocation bytes | 198,180,864 | 100,663,296 | 49.206% |
| Run 1 P50 | 69.6081 ms | 38.1958 ms | 45.127% |
| Run 1 P95 | 113.0958 ms | 63.1548 ms | 44.158% |
| Run 2 P50 | 70.0512 ms | 39.4281 ms | 43.715% |
| Run 2 P95 | 109.3172 ms | 60.1386 ms | 44.987% |

Evidence marker: `RUNTIME07_GC_FRAME_REPORTING_MODEL_V1`.

## Validation

- `python -m unittest tools.tests.test_runtime07_gc_frame_reporting_performance_contract`: the
  pre-change implementation produced the expected 2 failures; the optimized implementation passes
  2/2.
- `python -m py_compile tools/tests/test_runtime07_gc_frame_reporting_performance_contract.py`
  passed.
- `python -m unittest discover -s tools/tests -p 'test_runtime07_*'` passed 259/259
  Runtime07 contracts in one batch.
- Exact-file `rustfmt +1.94.1 --edition 2021 --check` passed for both production sources.
- The standalone Rust model compiled with `rustc +1.94.1 -O --edition 2021` and passed twice with
  identical checksums and positive P50/P95 reductions.
- Managed Rust compilation and tests remain pending in the next asynchronous Runtime07 batch.

Managed batch request: `runtime07-vm-gc-six-task-batch-20260830-v1`.

Validation attempt: ticket `a45b8eb5c82d46bab783834a6da58f6a` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
