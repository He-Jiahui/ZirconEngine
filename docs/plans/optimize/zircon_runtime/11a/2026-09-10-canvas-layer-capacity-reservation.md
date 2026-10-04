---
title: Runtime11A Canvas Layer Capacity Reservation
category: zircon_runtime
report_id: Runtime11A-canvas-layer-capacity-reservation-2026-09-10
date: 2026-09-10
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A Canvas Layer Capacity Reservation

## Scope

`arranged_canvas_layers` is part of every full arranged-tree projection. Before this slice its
layer output and per-parent child list grew geometrically even though the source tree already
contained the complete Canvas slot catalog. The change only adds bounded capacity reservations;
layer grouping, visibility admission, z-order sorting, and published order remain unchanged.

## Implementation

The grouping pass counts Canvas slots while building `canvas_slots_by_parent`, then reserves that
count for the flattened layer vector. Each Canvas parent also reserves its known slot count before
filtering invalid, detached, or hidden children. The reservations are upper bounds, so malformed
trees cannot cause an unbounded allocation and filtered children do not alter semantics.

## Deterministic work model

For a tree with `S` Canvas slots, the layer vector now starts with capacity `S`; each parent child
projection starts with the exact number of grouped slots for that parent. This removes vector
growth reallocations from the common full rebuild path while retaining the existing O(S log S)
sorting and visibility checks. It is structural allocation evidence, not a product frame or RSS
measurement.

## Validation

- Existing Canvas integration coverage still checks layout, render/hit admission, z-order, hidden
  children, and same-z grouping.
- A source regression requires both bounded capacity reservations and the in-loop slot count.
- Scoped Rustfmt and diff checks pass; the combined Runtime/Editor static contract batch is
  `135/135` with zero failures and errors.
- Managed Cargo and Windows Release p50/p95/p99 allocation/time evidence remain pending.

## Remaining Parent Work

Runtime11A still owns the broader UI driver, window/input lifecycle, retained layout graph,
multi-surface composition, and product/native timing acceptance. This slice does not close those
parent gaps.
