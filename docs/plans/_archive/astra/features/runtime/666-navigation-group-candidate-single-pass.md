---
doc_type: milestone-detail
status: implemented_pending_validation
superseded_by:
  - docs/plans/astra/features/runtime/733-navigation-first-group-candidate.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
related_code:
  - zircon_runtime/src/ui/surface/navigation_index.rs
  - zircon_runtime/src/ui/surface/navigation_index/tests.rs
tests:
  - zircon_runtime/src/ui/surface/navigation_index/tests.rs
---

# Navigation Group Candidate Single Pass

The retained navigation-index rebuild now derives each `group_id` first candidate in the same
stream that populates spatial and tab lists. Runtime733 supersedes this record's temporary
candidate-map implementation: the current path compares each node with the retained winner and
does not materialize a per-group vector or perform a second sort/traversal. The merged pass
preserves the existing deterministic first-candidate semantics while removing redundant map and
key-allocation work.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11A/P1-14 | Build group candidate membership during the primary navigation-index stream | superseded | Runtime733 keeps the one-node stream but reduces the first candidate directly into the retained map, removing the temporary group accumulator. The original source snapshot is retained for history; managed Runtime Cargo and release p50/p95/p99 navigation evidence remain pending. |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_runtime/src/ui/surface/navigation_index.rs` | `9C7FD598BD97E7569B30CC7AEB0A858CD68EAD66A23991B1CD494F82871B5FA3` |
| `zircon_runtime/src/ui/surface/navigation_index/tests.rs` | `B4DEC8B58DBC3F28DA1DA0676A256784C0458B4A33BAA3EF25303C91423D3C11` |
