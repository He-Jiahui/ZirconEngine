---
title: Runtime11A Taffy Child Handle Buffer Reuse
category: zircon_runtime
report_id: Runtime11A-taffy-child-handle-buffer-reuse-2026-09-10
date: 2026-09-10
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A Taffy Child Handle Buffer Reuse

## Scope

This slice removes the topology-update temporary allocation used by the retained Taffy
parent product. It preserves node identity, child order, style reconciliation, removal
semantics, and the existing fallback/error boundary. It does not change the retained solve
or exact-output reuse conditions.

## Implementation

`TaffyParentProduct` now owns a `Vec<NodeId>` containing the current ordered child handle
projection. Product creation fills that buffer once. Structural reconciliation clears and
refills the same buffer before `TaffyTree::set_children`, so repeated insert/remove/reorder
updates do not allocate a second `Vec<NodeId>` solely for the Taffy call. The buffer remains
private to the product and is discarded with the product on a failed reconciliation.

## Regression and local evidence

- The retained-product regression grows a product, performs a structural shrink, and verifies
  buffer length, contents, capacity, and pointer identity remain stable.
- `rustfmt --edition 2021 --check` passed for the touched Runtime owner.
- The focused Runtime UI/Taffy/layout contract batch passed `77/77`; the wider combined
  Runtime UI and Editor asset batch passed `123/123`.
- No tooling contract was changed. Managed Runtime Cargo execution and Windows Release
  allocation/time p50/p95/p99 remain pending.

## Acceptance boundary

This record is implementation-complete only. It does not claim managed compilation, product
frame timing, or broad Runtime11A acceptance.
