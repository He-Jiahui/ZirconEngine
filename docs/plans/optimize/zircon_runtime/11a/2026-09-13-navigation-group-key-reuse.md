---
title: Runtime navigation group position-map key reuse
category: zircon_runtime
report_id: Runtime11A-navigation-group-key-reuse-2026-09-13
date: 2026-09-13
supersedes: docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-position-map-reuse.md
followed_by: docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-first-group-candidate.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A navigation group position-map key reuse

## Scope

The retained navigation index already kept nested `HashMap` buckets for tab
positions, but every rebuild still passed a cloned `UiNavigationGroupId` to
`HashMap::entry`. `UiNavigationGroupId` owns a `String`, so stable groups paid a
temporary key allocation even when their bucket was already present.

## Implementation

`rebuild_group_position_maps` now probes `positions.get_mut(group_id)` first and
rebuilds the retained bucket in place. Only a newly admitted group constructs a
bucket and clones the owned group key for insertion. Stale-scope pruning,
candidate ordering, inner-map capacity reuse, and deterministic lookup results
are unchanged.

## Complexity and allocation boundary

For `G` retained groups and `F` candidates, rebuild remains `O(G + F)`. Stable
groups no longer allocate or clone their `String` key; allocations are limited
to genuinely new outer keys, new buckets, and inner-map growth. This is a
targeted follow-up to the position-map reuse slice and does not change the
navigation candidate vectors or event-query authority.

## Validation

- TDD RED: the new source contract rejected the unconditional
  `positions.entry(group_id.clone())` path.
- GREEN: the contract passes `8/8`; the existing lower Rust capacity/pruning
  regression remains in place, and `rustfmt --emit stdout` parses the updated
  module.
- The latest same-source batched Runtime/Editor UI performance-contract invocation loaded
  172 modules and passed `772/772` tests in `1.571s`; this is local source
  evidence only.
- Current source hashes:

  | File | SHA-256 |
  | --- | --- |
  | `zircon_runtime/src/ui/surface/navigation_index.rs` | `35FC741B592771C75AFA72B712F582266D24DA5FB66E6B7823D6CF5A58BE93AA` |
  | `tools/tests/test_runtime_ui_navigation_index_performance_contract.py` | `E6869E3D55DC334426A5069E2C395F596F88AB3CB671F09A8F04EBD130FCE966` |

Managed Cargo/Windows Release and product navigation allocation/latency
evidence remain pending under the existing external dirty-worktree gate.

## Acceptance boundary

Keep `validation_status: managed_validation_pending` until the next batched
managed Runtime/Editor run verifies compilation, candidate-order parity,
allocation behavior, and navigation p50/p95/p99. No coordinator request or
status query was made for this follow-up.
