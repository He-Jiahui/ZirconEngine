---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-activity-registry-hash-index.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_records:
  - docs/plans/astra/features/editor/665-activity-registry-hash-index.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/control/service.rs
tests:
  - zircon_editor/src/ui/control/service/activity_registry_hash_tests.rs
  - tools/tests/test_editor_activity_registry_snapshot_capacity_performance_contract.py
---

# Editor activity-registry snapshot capacity

The activity view/window registries already use `HashMap` for direct lookup,
but their deterministic snapshot methods still collected cloned descriptors
into geometrically growing vectors. The snapshot builders now reserve the
authoritative registry lengths before extending and sorting. Descriptor
ownership, duplicate rejection, ordering, and lookup semantics are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 activity registry | Reserve view/window registry counts before deterministic snapshot sorting. | RED/GREEN source probes, a production capacity/order regression, and the batched Runtime/Editor contract suite pass; managed Editor Cargo and Release timing/allocation evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

Lookup remains expected `O(1)` through the existing hash maps. Snapshot
publication remains `O(V log V + W log W)` for deterministic sorting, while the
destination vectors no longer grow geometrically for `V` views or `W` windows.
No second registry or alternate ordering authority is introduced.

## Local evidence

- The RED probe confirmed both snapshot methods still used bare `collect`; the
  GREEN probe and source contract assert the exact registry-length reservations
  and explicit `extend` paths.
- The production regression preserves sorted view/window output and checks both
  returned vector lower bounds.
- Current source snapshot SHA-256: `1EF8E50F4A4DA930A1712B2FC471E5D0C3791AAF031010C4819E5A2C802E0B62`
  (`service.rs`) and `B8180388F3D28D3098E989EB3353D715DB2C2E1629223D1777BC541338826184`
  (`activity_registry_hash_tests.rs`).
- The final one-process non-tooling Runtime/Editor performance-plus-pressure
  batch was refreshed for this slice: 347 modules and `1331/1331` tests passed
  in `6.361s`, including the activity-capacity source contract.
- Scoped Rustfmt, `git diff --check`, and record whitespace/reference checks
  pass. No tooling production implementation was changed.

This is source/contract evidence, not managed Cargo execution or product CPU,
allocator, RSS, or p50/p95/p99 acceptance evidence.

## Managed acceptance gate

The previous owner-attributed admission in `696` was rejected before ticket
creation by the external dirty worktree gate. No new coordinator request or
status query is issued for this slice; include it in the next batched Windows
Release validation and keep the row at `implemented_pending_validation` until
the snapshot allocation and latency gates are measured.
