---
title: Runtime57 Bounded Monitor Selection
category: zircon_runtime
report_id: Runtime57-bounded-monitor-selection-2026-08-27
date: 2026-08-27
session_id: root-runtime57-bounded-monitor-selection-20260827
implementation_status: implementation_complete
validation_status: local_contract_passed_managed_validation_pending
---

# Runtime57 Bounded Monitor Selection

## Scope

This slice addresses `PLH-P1-036` in the current window-creation path. Monitor handles remain
transient until the planned `DisplayTopologySnapshot` owner exists, but creating one window no
longer collects every available monitor into a temporary `Vec`. The descriptor can request at most
one monitor for placement and one for fullscreen, so the creation context retains only those two
indices in fixed storage.

## Change

- Pass the descriptor's position and mode demand into `WindowMonitorContext`.
- Extract, deduplicate, and retain at most two explicit monitor indices.
- Enumerate only through the largest requested index and store matching handles in a fixed array.
- Preserve the primary-monitor lookup and the existing `Current`, `Primary`, and missing-index
  fallback behavior.
- Add Rust coverage for two distinct indices, duplicate indices, and non-index selections.

## Deterministic Performance Evidence

The standalone optimized Rust model uses 65,536 synthetic monitor handles, requests indices 7 and
511, and alternates the legacy and bounded paths for 21 samples with 64 selections per sample. It
asserts identical selected handles, zero optimized-path allocations, at least 99% fewer enumerated
items, and at least 80% P50/P95 reduction.

| Monitor selection during window creation | Before | After | Reduction |
|---|---:|---:|---:|
| Allocations per selection | 1 | 0 | 100.000% |
| Allocated bytes per selection | 524,288 | 0 | 100.000% |
| Enumerated monitor handles | 65,536 | 512 | 99.219% |
| P50 for 64 selections | 977,300 ns | 42,800 ns | 95.621% |
| P95 for 64 selections | 2,123,900 ns | 47,600 ns | 97.759% |

Evidence checksum: `1,392,384`.

## Validation

- `python -m unittest tools.tests.test_runtime57_bounded_monitor_selection_performance_contract -v`:
  3 passed.
- Exact-file `rustfmt --edition 2021 --check` and scoped `git diff --check` pass.
- The standalone Rust model compiles with `rustc --edition 2021 -O` and passes all equivalence and
  performance gates.
- Cargo compilation and focused Rust tests remain pending in the asynchronous coordinator batch.
- The foreign-modified runtime-entry source guard still matches the former one-argument constructor
  and full-`Vec` implementation. Its owner must update that stale source-shape expectation before
  the managed test suite can accept the new contract; this slice does not alter that leased path.

## Remaining Parent-plan Work

Runtime57 still owns the stable display identity, topology generation, hotplug event, window
registry, surface lease, lifecycle, and command-broker gaps recorded in the canonical review. This
slice only bounds transient monitor selection during current window creation.
