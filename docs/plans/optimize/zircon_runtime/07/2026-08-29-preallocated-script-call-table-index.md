---
title: Runtime07 Preallocated Script Call Table Index
category: zircon_runtime
report_id: Runtime07-preallocated-script-call-table-index-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Script Call Table Index

## Scope

This slice removes repeated growth allocations while publishing the immutable two-level name
index for a script call table. The production builder already emits functions in contiguous
module groups, so the index can derive both outer module capacity and inner function capacity
from the entries without allocating a separate counting structure.

## Change

- Count contiguous module groups without heap allocation and preallocate the outer module index.
- Walk each contiguous group once and preallocate its function index to the exact group length.
- Preserve lookup IDs, entry ordering, generation publication, and duplicate-name overwrite
  behavior.
- Preserve correctness when a caller supplies repeated non-contiguous module groups; the later
  group extends the existing module index.
- Add a Rust regression for repeated module groups and a Python source performance contract for
  both preallocation levels.

## Deterministic Performance Evidence

The standalone optimized Rust model builds a two-level index for 64 modules with 24 exports per
module. Each latency sample performs 256 complete index builds; allocation profiles cover 64
builds. Key strings are created before counters start, so the allocation evidence isolates index
construction. Both implementations resolve every original entry and produced checksum
`16413244727001412081`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 64 builds | 16,768 | 4,160 | 75.191% |
| Requested allocation bytes | 7,460,608 | 3,875,840 | 48.049% |
| Run 1 build P50 | 102.4049 ms | 67.6337 ms | 33.955% |
| Run 1 build P95 | 291.1037 ms | 166.4584 ms | 42.818% |
| Run 2 build P50 | 116.0642 ms | 74.3691 ms | 35.924% |
| Run 2 build P95 | 175.3981 ms | 110.6488 ms | 36.916% |

Evidence marker: `RUNTIME07_PREALLOCATED_SCRIPT_CALL_TABLE_INDEX_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_preallocated_script_call_table_index_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts equivalent name resolution before measurement; two runs kept
  identical allocation profiles, checksums, and positive P50/P95 results.
- The Rust regression covers a module that appears in two non-contiguous groups.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot publication.
- Managed Rust compilation and focused call-table tests remain pending in the next asynchronous
  Runtime07 validation batch.

Managed batch request: `runtime07-native-vm-six-task-batch-20260830-v1`.

Validation attempt: ticket `167f127a7c8d48b3a68554a5c4f1d0f7` failed during coordinator
materialization with `unmanaged_artifacts_detected` for
`D:\ZirconBuilds\mvp-test-fixtures-36724`; Cargo did not start, so integrated Rust and performance
acceptance remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
