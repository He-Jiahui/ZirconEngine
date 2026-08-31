---
title: Editor13 Window Host Hash Reconcile
category: zircon_editor
report_id: Editor13-window-host-hash-reconcile-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Window Host Hash Reconcile

## Scope

This slice replaces the native-window owner with `HashMap` and replaces the layout reconciliation
nested membership scan with one borrowed `HashSet`. Synchronization now retains active windows in
expected linear time and updates each floating window through one entry lookup instead of separate
open and bounds-sync probes.

`states()` sorts the projected rows by `MainPageId`, and Debug builds a borrowed `BTreeMap` only on
the diagnostic path. Window handles, per-window `UiSurface` ownership, layout bounds, close and
reattach behavior, and observable state order are unchanged.

## Performance Workload

The release workload reconciles 1,024 tracked windows against 512 active windows.

| Work per workload | Before | After |
|---|---:|---:|
| Nested membership scans | 1,024 | 0 |
| Hash membership checks | 0 | 1,024 |
| Ordered sync lookups for active windows | 1,024 | 0 |
| Hash sync lookups for active windows | 0 | 512 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR13_WINDOW_HOST_HASH_RECONCILE_BENCH_V1`. Acceptance requires linear hash reconciliation P95
to be at least 70% below the legacy nested scan. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826cb_window_host_hash_reconcile_preserves_state_and_debug_order`
  covers deterministic state and diagnostic order over unordered insertion.
- `optimization_batch_20260826cb_window_host_hash_reconcile_is_linear_and_ordered_at_output`
  locks the hash owner, linear membership index, one-probe sync path, and ordered projection.
- `optimization_batch_20260826cb_window_host_hash_reconcile_p95` reports paired release P50/P95
  samples and enforces the 70% P95 reduction gate.

## Remaining Parent-plan Work

Editor13 still owns platform window creation/destruction, persisted workspace migration, monitor
and DPI recovery, focus restoration, docking transactions, crash-safe restore, and product
integration. This slice only converges retained native-window reconciliation.
