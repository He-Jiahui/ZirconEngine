---
title: Runtime77 Pointer Hover Borrowed Route
category: zircon_runtime
report_id: Runtime77-pointer-hover-borrowed-route-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Pointer Hover Borrowed Route

## Scope

This slice removes unconditional route allocation and replacement from active-pointer hover-state
updates. Bubble-route priority, direct-target fallback, route-target fallback, per-pointer state,
press/capture behavior, and returned dispatch diagnostics remain unchanged.

## Change

- Borrow the bubble/direct/route-target slice directly from `UiInputDispatchResult` instead of
  cloning it into a temporary `Vec`.
- Accept borrowed hover paths in `UiActivePointerTable` and compare them with retained state before
  copying.
- On a changed route, clear and extend the retained buffer so an adequate existing capacity is
  reused; on an unchanged route, perform no allocation or node copy.

## Deterministic Performance Evidence

| 512-node unchanged route, 16,384 pointer events | Before | After |
|---|---:|---:|
| Temporary route clones per sample | 16,384 | 0 |
| Route nodes copied per sample | 8,388,608 | 0 |
| Route node comparisons per sample | 0 | 8,388,608 |
| Retained hover-buffer replacements | 16,384 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME77_POINTER_HOVER_BORROWED_ROUTE_BENCH_V1`. Acceptance requires retained-route reuse P95 to
be at least 25% below unconditional route-clone P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ap_pointer_hover_path_reuses_retained_buffer` covers unchanged and
  changed routes while checking retained allocation identity and capacity.
- `optimization_batch_20260826ap_pointer_hover_path_borrows_dispatch_route` requires borrowed
  manager slices, a change guard, and in-place retained-buffer extension.
- `optimization_batch_20260826ap_pointer_hover_path_reuse_p95` reports paired P50/P95 samples and
  enforces the 25% P95 reduction gate.

## Remaining Parent-plan Work

Runtime77 still owns navigation indexing, reusable route scratch, qualified pointer identity,
transactional effects, queue backpressure, device coverage, and product-scale performance
receipts. This slice only converges unchanged active-pointer hover-route publication.
