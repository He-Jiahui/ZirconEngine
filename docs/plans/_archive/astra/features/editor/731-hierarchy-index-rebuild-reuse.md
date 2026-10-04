---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-09-13-hierarchy-index-rebuild-reuse.md
related_records:
  - docs/plans/astra/features/editor/730-hierarchy-projection-single-pass.md
  - docs/plans/astra/features/editor/740-hierarchy-control-id-arc-sharing.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_projection.rs
tests:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_projection.rs
  - tools/tests/test_editor_scene_hierarchy_generation_index_performance_contract.py
---

# Editor hierarchy identity-index rebuild reuse

`SceneHierarchyProjectionState::replace` previously rebuilt a temporary ordered
`BTreeMap` for entity-to-control routing, then traversed that map again to build
the reverse control-to-entity index. The retained state now keeps both indexes
as reusable `HashMap`s, clears and reserves them for the incoming bounds, and
fills row identity plus both control directions in one source-row pass.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 hierarchy identity projection | Reuse row/control index allocations and remove the temporary ordered-map/reverse traversal during full reflow. | RED/GREEN source contract, lower Rust lookup regression, scoped Rustfmt, and the batched Runtime/Editor contract suite; managed Cargo and product allocation/latency evidence remain pending. | implemented_pending_validation |

## Complexity and allocation boundary

The row identity map and the two control lookup maps are now populated in one
`O(N)` pass and retain their hash-table capacity across subsequent reflows.
The old temporary ordered map and reverse traversal imposed ordered `O(N log N)`
work and discarded the previous map allocations. The follow-up
`Editor740-hierarchy-control-id-arc-sharing` record changes the two control
maps to shared `Arc<str>` keys, so distinct control identifiers no longer pay
two string payload allocations. Lookup semantics for duplicate IDs and a
controls slice shorter than the row slice remain last-write/truncated exactly
as before.

## Local evidence

- The new source contract was intentionally RED while the old `BTreeMap` plus
  `rows.iter().zip(controls)` implementation was present, then GREEN after the
  reusable one-pass `HashMap` build landed.
- The in-file regression verifies both-direction lookups, selected-state
  publication, truncated control input behavior, and stale-row removal after a
  replacement reflow.
- The existing hierarchy-native-authority source guard was updated to assert
  the same all-logical-row invariant against the reusable insertion loop; its
  focused repair is covered by the next batched run (test SHA-256
  `A4A76E535BD6F70A2E41574DF3CD198773B86CB6C8FBD109FAD5A72386FC7B09`).
- The source/test fingerprints above describe the pre-Editor740 snapshot;
  Editor740 records the current `Arc<str>` source and contract fingerprints.
- The focused Editor hierarchy-native-authority, hierarchy-generation,
  hierarchy-projection, and Runtime reverse-map contracts pass `19/19`; the
  refreshed single-process Runtime/Editor performance-plus-pressure batch
  covers 351 modules and passes `1346/1346` tests in `47.701s`.
- A subsequent same-source single-process rerun of that exact non-tooling
  batch again covers 351 modules and passes `1346/1346` tests, now in `8.890s`.
- The adjacent Hit-Test/Hierarchy contract batch (including route/query
  scratch and hierarchy pressure modules) passes `129/129` across 25 modules
  in `21.376s`.
- The wider Runtime UI plus Editor projection/pointer/cache batch passes
  `760/760` across 164 modules in `20.998s`.

This is source/contract evidence, not managed Cargo execution or product CPU,
allocator, RSS, or p50/p95/p99 acceptance evidence.

## Managed acceptance gate

The existing owner-attributed Windows Release admission remains deferred by the
external dirty `E:\Git\zr_vm` worktree gate. No new coordinator request or
status query was issued for this slice. Keep the status at
`implemented_pending_validation` until the next batched owner-attributed
validation verifies compile, index parity, allocation, and hierarchy latency.
