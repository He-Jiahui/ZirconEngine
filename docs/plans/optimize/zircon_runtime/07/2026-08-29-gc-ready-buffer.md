---
title: Runtime07 GC Ready Buffer
category: zircon_runtime
report_id: Runtime07-gc-ready-buffer-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 GC Ready Buffer

## Scope

This slice reduces allocation and tree-work overhead when the VM GC scheduler drains slots
whose due frame has arrived. The scheduler already stores due buckets in a `BTreeMap` and each
slot has one active schedule entry; the temporary `BTreeSet` in `take_due` therefore repeated
ordered tree insertion for a result that is sorted only once at return.

## Change

- Count slot entries in the due buckets with a read-only range pass and allocate one exact
  contiguous result buffer.
- Push each validated slot into that buffer while preserving generation and next-due updates.
- Sort the completed buffer with `sort_unstable` before returning the established stable slot
  order.
- Preserve empty schedules, stale-entry filtering, overflow retirement, and due-frame ordering.
- Add a source performance contract covering exact capacity, contiguous collection, and final
  ordering.

## Deterministic Performance Evidence

The standalone optimized Rust model drains 8,192 scheduled slots across 64 due buckets for 128
projections per sample and 31 alternating samples, after five warmups. The legacy path inserts
each slot into a `BTreeSet`; the optimized path counts the due bucket entries, fills one `Vec`,
and sorts it. Both paths produced checksum `3570126252479944518` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per projection | 1,112 | 1 | 99.910% |
| Requested allocation bytes | 193,656 | 65,536 | 66.159% |
| Run 1 P50 | 173.4855 ms | 19.2474 ms | 88.905% |
| Run 1 P95 | 293.7207 ms | 76.5149 ms | 73.950% |
| Run 2 P50 | 199.1307 ms | 21.0501 ms | 89.429% |
| Run 2 P95 | 340.7860 ms | 110.6797 ms | 67.522% |

Evidence marker: `RUNTIME07_GC_READY_BUFFER_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_gc_ready_buffer_performance_contract.py`: 2 passed after
  the pre-change implementation produced the expected 2 contract failures.
- Exact-file `rustfmt +1.94.1 --check` passed for the production source and model.
- `git diff --check` passed for the production source and contract.
- The standalone Rust model compiled with `rustc +1.94.1 -C opt-level=3` and passed twice with
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
