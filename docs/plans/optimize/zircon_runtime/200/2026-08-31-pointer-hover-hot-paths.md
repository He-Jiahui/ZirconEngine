---
title: Runtime200 Pointer Hover Hot Paths
category: zircon_runtime
report_id: Runtime200-pointer-hover-hot-paths-2026-08-31
date: 2026-08-31
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: managed_validation_queued
---

# Runtime200 Pointer Hover Hot Paths

## Scope

This batch closes two independent pointer-hover costs without changing node order or hover
semantics. The active pointer table now compares a borrowed node iterator before mutating its
retained route buffer. Pointer hover diffing keeps the allocation-free nested scan for tiny paths
and switches dense large paths to one membership table reused across the entered and left phases.

The current shared source also projects `UiPointerRoutingReceipt::physical_bubble_route()` directly
into the iterator setter. That projection lives in a larger input-manager text/IME/diagnostic
lifecycle union and is intentionally not absorbed into this focused snapshot. Its integration and
the product input-latency profile remain an explicit external acceptance dependency.

## Implementation

- `set_hovered_path_iter` accepts a cloneable borrowed iterator. Equal routes leave the existing
  allocation, pointer, length, and capacity untouched. Changed routes clear and extend the retained
  buffer, reusing capacity when the new route fits.
- The slice adapter forwards `hovered.iter().copied()` without collecting or cloning a vector.
- `hover_diff` returns immediately for equal paths. A comparison budget of 64 keeps normal tiny
  paths on the allocation-free linear branch. Larger paths allocate one capacity-sized `HashSet`,
  clear it between phases, and preserve current/previous source order in the output vectors.

## Deterministic Performance Evidence

The model is frozen to `HEAD 5ffc4945095a6fc734bcbb2e632958026350b760` and exact HEAD Git
blobs `a042804a567e258151a66c6635abd2a52c20e0ba` (pointer table) and
`9315b899d3f7cd79e2f6c0b2604e634f0332092b` (event routing). It models 1,000,000
events, a 512-node retained route, disjoint 512-node large hover paths, and disjoint 8-node small
paths.

| Metric | HEAD behavior | Current algorithm | Change |
|---|---:|---:|---:|
| Stable-route Vec clones | 1,000,000 | 0 | -100.000% |
| Stable-route node copies | 512,000,000 | 0 | -100.000% |
| Stable-route Vec allocations, lower bound | 1,000,000 | 0 | -100.000% |
| Stable-route copied node payload, lower bound | 4,096,000,000 bytes | 0 | -100.000% |
| Stable-route node comparisons | 0 | 512,000,000 | explicit equality gate |
| Large hover-diff membership work | 524,288,000,000 equality comparisons | 2,048,000,000 hash inserts/lookups | -99.609375% operations |
| Large hover-diff membership allocations | 0 | 1,000,000 | one table per changed event |
| Small hover-diff membership allocations | 0 | 0 | unchanged |

These are deterministic copy and operation counts, not elapsed-time measurements. Hash operations
and equality comparisons have different costs; the 99.609375% figure is a work-count comparison,
not a latency claim. Allocator metadata, cache locality, output vectors, branch prediction, RSS,
and product P50/P95/P99 are excluded.

## Dynamic Acceptance

The ignored Rust release benchmark uses 17 alternating sample pairs, 16,384 stable-route updates
per sample, and route depth 512. It records exact sample arrays plus P50/P95 and requires retained
buffer P95 to be at least 25% below repeated `to_vec` replacement. One managed Windows batch must
run the focused Rust behavior/benchmark tests and both Python task contracts. Commit, push, record
finalization, and WeCom publication remain gated on terminal managed evidence.

This dynamic gate is grouped with Runtime200 borrowed dispatch route sharing under one managed
Windows release script. Cargo is filtered by the shared `runtime200_` prefix rather than submitted
per test; Python source contracts and deterministic pressure models run in the same ticket. The
queued ticket is `776ac58291114580ab254879d2f7fea4`, submitted by request
`runtime200-ui-route-performance-batch-20260831-r1` (receipt
`01dd978ba6064af3963ee5126cae0822`) from snapshot `2491`; its manifest is superseded by the
current record hash after this reconciliation. A final current-source refresh will be submitted
after all record text is stable.
