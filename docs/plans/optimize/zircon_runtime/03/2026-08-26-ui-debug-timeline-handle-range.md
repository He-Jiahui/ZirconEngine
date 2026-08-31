---
title: Runtime03 UI Debug Timeline Handle Range
category: zircon_runtime
report_id: Runtime03-ui-debug-timeline-handle-range-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime03 UI Debug Timeline Handle Range

## Scope

This slice removes linear handle-membership scans from the retained UI debug timeline. Frame
capture order, retention eviction, selected-frame behavior, snapshots, and saturation behavior
remain unchanged.

## Change

- Use the first and last retained handles as an inclusive membership range. Handles are allocated
  monotonically and every capture appends exactly one frame, so the retained `VecDeque` has no
  handle gaps.
- Remove the post-capture membership scan: capacity is clamped to at least one and the newly
  appended frame cannot be evicted by the same retention loop.
- Keep snapshot lookup unchanged; this slice only eliminates repeated membership scans.

## Deterministic Performance Evidence

| 16,384 retained frames, 512 missing-handle probes | Before | After |
|---|---:|---:|
| Frame-entry visits per sample | 8,388,608 | 0 |
| Boundary reads per sample | 0 | 1,024 |
| Membership complexity | `O(N)` | `O(1)` |
| Capture self-check scan | up to retained capacity | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME03_UI_DEBUG_TIMELINE_HANDLE_RANGE_BENCH_V1`. Acceptance requires range-membership P95 to
be at least 95% below linear-scan P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826an_timeline_handle_range_preserves_retention_membership` covers
  retained, evicted, future, and selected handles.
- `optimization_batch_20260826an_timeline_membership_uses_contiguous_handle_range` requires the
  first/last range boundary and rejects both linear membership and capture self-check scans.
- `optimization_batch_20260826an_timeline_handle_range_p95` reports paired P50/P95 samples and
  enforces the 95% P95 reduction gate.

## Remaining Parent-plan Work

Runtime03 still owns the full runtime diagnostics, profiling, configuration, capture, export, and
product-observability surface. This slice only converges retained UI timeline membership.
