---
title: Runtime hit-grid reverse-map capacity reuse
category: zircon_runtime
report_id: Runtime11A-hit-grid-reverse-map-reuse-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A hit-grid reverse-map capacity reuse

## Scope

`UiHitTestIndex` keeps a reverse `node_id -> cell memberships` authority for
incremental geometry patches. Rebuilding the grid previously dropped that map
and allocated a fresh empty vector for every entry, even when node identities
were unchanged.

## Implementation

`reindex_entry_cells` first refreshes `entry_indices`, then retains matching
`entry_cells` keys and clears their vectors in place. Only IDs absent from the
retained map receive a new empty vector. The existing cell scan, membership
ordering, cold-deserialization repair, and transactional failure restoration
remain unchanged.

## Deterministic work model

After the first build, a stable set of `N` entry IDs performs zero per-entry
reverse-map node/vector allocations on subsequent rebuilds (subject to capacity
growth); the retired path recreated `N` map values and vectors each time. Cell
membership traversal remains `O(M)` for `M` indexed memberships, and changed
IDs retain the existing allocation behavior.

## Validation

- The focused source contract was RED against `entry_cells.clear()` and
  `(entry.node_id, Vec::new())`, then GREEN with retain/clear/or_default.
- The lower Rust regression verifies that a wide entry's cell-vector capacity
  survives a rebuild to a one-cell footprint.
- The focused new contracts pass `4/4`; the single-process non-tooling
  Runtime/Editor performance-plus-pressure batch covers 351 modules and passes
  `1345/1345` tests in `39.080s`.
- Scoped Rustfmt passes. Managed Cargo, allocation, and hit-test latency
  evidence remain pending under the external `E:\Git\zr_vm` dirty-worktree
  admission blocker.

## Acceptance boundary

This is static/source evidence only. Keep the status pending until the next
owner-attributed Windows Release batch verifies compile, membership parity, and
the declared allocation/p50/p95/p99 gates.
