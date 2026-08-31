---
title: Runtime07 Disjoint Capability View Storage
category: zircon_runtime
report_id: Runtime07-disjoint-capability-view-storage-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Disjoint Capability View Storage

## Scope

This slice removes duplicate owned capability keys from the runtime registration capability view.
It preserves direct, module, feature, and status-only capability lookup, first-status-wins manifest
merging, explicit `with_status` replacement, and the existing public `has`/`status` behavior.

## Change

- Treat the plain capability set and status map as disjoint indexes whose union is the complete
  capability view.
- Route `has` through both indexes while leaving `status` as a direct map lookup.
- Index package statuses before direct and module capabilities, remove any earlier plain key, and
  clone a status key only when its first status is accepted.
- Skip plain capability clones when the status map already owns the same key.
- Move the owned `with_status` key directly into the status map instead of cloning it into both
  collections.
- Add a Rust regression for declared-and-status and status-only keys plus a Python source contract
  for the disjoint-storage invariant.

## Deterministic Performance Evidence

The standalone optimized Rust model builds a capability view from 32,768 declarations with matching
status rows across 31 alternating samples. Both implementations assert identical `has` and `status`
results for every capability and produced checksum `7168462637616345106`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 98,334 | 32,783 | 66.662% |
| Requested allocation bytes | 12,910,840 | 6,094,956 | 52.792% |
| Run 1 build P50 | 78.6694 ms | 42.4745 ms | 46.009% |
| Run 1 build P95 | 181.8336 ms | 120.4760 ms | 33.744% |
| Run 2 build P50 | 69.7346 ms | 40.7630 ms | 41.546% |
| Run 2 build P95 | 139.4021 ms | 65.0040 ms | 53.369% |

Evidence marker: `RUNTIME07_DISJOINT_CAPABILITY_VIEW_STORAGE_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_disjoint_capability_view_storage_performance_contract.py`:
  4 passed after the pre-change contract failed 4 of 4 checks.
- The standalone Rust model asserts every public capability/status projection before recording
  metrics and passed twice with identical allocation and checksum results.
- The Rust regression checks that declared-and-status and status-only capabilities remain visible
  while their owned keys reside only in the status index.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required before
  snapshot publication.
- Managed Rust compilation and focused tests remain pending in the asynchronous batch shared with
  the borrowed VM bytecode file-name candidate.

Managed batch request: `runtime07-borrowed-gameplay-seven-task-batch-20260830-v1`.

Validation attempt: ticket `a9dc9a55e9044c239cc7dfda8bbc64b6` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; the 22 local contract
checks remain green while integrated acceptance and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
