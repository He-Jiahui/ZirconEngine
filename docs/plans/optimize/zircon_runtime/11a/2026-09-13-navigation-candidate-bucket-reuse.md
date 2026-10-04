---
title: Runtime navigation candidate bucket reuse
category: zircon_runtime
report_id: Runtime11A-navigation-candidate-bucket-reuse-2026-09-13
date: 2026-09-13
related_to:
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-position-map-reuse.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-group-key-reuse.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-first-group-candidate.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A navigation candidate bucket reuse

## Scope

The retained navigation index already reused lookup-map buckets, but each full
rebuild still dropped the sorted spatial/tab group vectors with `BTreeMap::clear`.
Stable modal scopes therefore paid outer-node and inner-`Vec` allocation again,
and every streamed candidate entered the group maps through an owned key clone.

## Implementation

Group candidate maps now clear their existing vectors in place and prune only
empty scopes after the node stream. A stable group appends through a borrowed
lookup; only a newly admitted scope clones the `UiNavigationGroupId` and creates
its first vector. MUI-root maps use the same clear/prune lifecycle, while the
sorted vectors remain the sole ordering authority and stale scopes cannot be
queried after a rebuild.

The root remains a cohesive, single-purpose navigation-index owner. Its
candidate-bucket helpers now live in the narrow
`navigation_index/candidate_buckets.rs` child so the design-warning boundary
does not accumulate another responsibility.

## Complexity and allocation boundary

The candidate stream remains `O(F)` before the existing per-scope sort, with
bounded `O(G)` clear/prune work for the retained scope buckets. Stable scope
rebuilds reuse the outer map nodes and vector capacity; allocation is limited
to genuinely new scopes or capacity growth. Stale-scope pruning keeps the
published maps bounded to the current candidate set. No navigation target,
modal filtering, or ordering semantics change.

## Validation

- TDD RED: the new source contracts rejected dropping group maps with
  `clear()` and rejected unconditional `entry(group_id.clone())` in the hot
  stream.
- GREEN: the navigation contract passes `11/11`; the lower Rust regression
  covers clear-in-place capacity retention for group and MUI-root buckets,
  borrowed stable append, and stale empty-scope pruning. Rustfmt parses both
  production and test modules.
- The latest same-source batched Runtime/Editor UI performance-contract
  invocation loaded 172 modules and passed `772/772` tests in `1.571s`; this is
  local source evidence only.
- Current source hashes:

  | File | SHA-256 |
  | --- | --- |
  | `zircon_runtime/src/ui/surface/navigation_index.rs` | `5469B0F3D284B7FEE8189BE5FC38CFDDF73E2DCA7F2E6EEA07549D6B0BDE3EBD` |
  | `zircon_runtime/src/ui/surface/navigation_index/candidate_buckets.rs` | `9E86E277F42E5231ED0FFF577A1F489C1CBD3C2122274817DCAE8A167247CE30` |
  | `zircon_runtime/src/ui/surface/navigation_index/tests.rs` | `6DE097EDB8AF0B1B6215EA8183CAFD90E020A439B0A43344278C1C5CFF2F177D` |
  | `tools/tests/test_runtime_ui_navigation_index_performance_contract.py` | `D6587E1AC21572D0A3077F722E36FF3B6019B881F33B82632A1A97FEEEE5EAED` |

Managed Cargo/Windows Release and product navigation allocation/latency
evidence remain pending under the existing external dirty-worktree gate.

## Acceptance boundary

Keep `validation_status: managed_validation_pending` until the next batched
managed Runtime/Editor run verifies compilation, candidate-order parity,
allocation behavior, and navigation p50/p95/p99. No coordinator request or
status query was made for this follow-up.
