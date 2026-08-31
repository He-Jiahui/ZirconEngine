---
title: Runtime07 Indexed Unique Refresh Merge
category: zircon_runtime
report_id: Runtime07-indexed-unique-refresh-merge-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Indexed Unique Refresh Merge

## Scope

This slice removes the quadratic notification-order scan when a native-plugin discovery refresh
batch appends previously unseen manifest paths. It preserves latest-event-wins behavior, duplicate
path movement to the newest position, directory-removal invalidation, parent/child notification
order, and full-root-scan dominance.

## Change

- Use the existing `current_actions` map as the exact-path membership index before scanning the
  notification-order vector.
- Run the order-vector `retain` only when `remove(&path)` proves that a Refresh path is already in
  the current batch.
- Append new Refresh paths directly while retaining the one cloned path for order storage and the
  moved path for action ownership.
- Keep directory Remove handling unchanged because it must still invalidate descendant paths.
- Add Rust regressions for unique append order and duplicate Refresh reordering plus a Python source
  performance contract for the indexed fast path.

## Deterministic Performance Evidence

The standalone optimized Rust model merges 2,048 existing paths with 2,048 distinct later Refresh
paths across 31 alternating samples. Both implementations assert identical action maps and
notification order and produced checksum `5956641763639077669`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Notification-order path comparisons | 6,290,432 | 0 | 100.000% |
| Allocation calls | 2,388 | 2,388 | 0.000% |
| Requested allocation bytes | 288,416 | 288,416 | 0.000% |
| Run 1 merge P50 | 47.0450 ms | 2.0720 ms | 95.596% |
| Run 1 merge P95 | 74.5333 ms | 4.3406 ms | 94.176% |
| Run 2 merge P50 | 53.3786 ms | 2.1579 ms | 95.957% |
| Run 2 merge P95 | 94.2258 ms | 6.9271 ms | 92.648% |

Evidence marker: `RUNTIME07_INDEXED_UNIQUE_REFRESH_MERGE_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_indexed_unique_refresh_merge_performance_contract.py`:
  3 passed after the pre-change contract failed 2 of 3 checks.
- The standalone Rust model asserts complete action-map and notification-order equality before
  recording metrics; two runs retained identical allocation profiles and positive P50/P95 results.
- Rust regressions cover new unique paths and duplicate paths while the existing tests retain
  directory removal and full-root invalidation behavior.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required before
  snapshot publication.
- Managed Rust compilation and focused discovery-refresh tests remain pending in the asynchronous
  Runtime07 batch shared with borrowed reflection owner validation.

Managed batch request: `runtime07-native-vm-six-task-batch-20260830-v1`.

Validation attempt: ticket `167f127a7c8d48b3a68554a5c4f1d0f7` failed during coordinator
materialization with `unmanaged_artifacts_detected` for
`D:\ZirconBuilds\mvp-test-fixtures-36724`; Cargo did not start, so integrated Rust and performance
acceptance remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
