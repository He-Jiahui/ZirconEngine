---
doc_type: optimization-implementation
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
gate: RSH-G24
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/derived_state_scale_profile.rs
  - zircon_runtime/src/scene/tests/derived_state/hierarchy_rebuild.rs
tests:
  - derived_propagation_preserves_nested_branch_values_and_dirty_scope
  - derived_state_full_rebuild_baseline_is_deterministic_for_one_and_one_thousand_nodes
  - derived_state_full_rebuild_baseline_is_deterministic_for_one_hundred_thousand_nodes
  - derived_state_parent_transform_change_rebuilds_the_affected_subtree
  - runtime62_derived_propagation_scale_profile
---

# Runtime62 RSH-G24: propagation stack follows depth

The active-state and world-matrix propagation passes previously copied all
children of a node into a LIFO vector before processing the first child. A
one-million-node star therefore materialized nearly one million pending
`(EntityId, inherited state)` pairs on each pass, including a world matrix
for every child in the matrix pass.

Both passes now keep a child iterator and inherited value only at an ancestor
with more than one child. They process siblings in the same stable order as
the prior reverse-push/LIFO walk. The pending vector's maximum length follows
the number of branching ancestors rather than sibling width or single-child
chain depth. A wide star needs one frame; a chain needs none. The derived
value comparison, publication, render-dirty marking, and visited/written
counter paths remain in the same order for each entity.

The new ignored Release profile samples 1/1K/100K/1M wide-star propagation
and 1/32/1,024 single-child chains, with raw latency and work counters. It
has not run, so the actual allocation-byte and latency changes are
unmeasured. No numeric ceiling or G24 pass is claimed.
The prior source-text test demanded the old reverse-push stack spelling; it
now leaves stack representation open, and a nested-branch behavior regression
checks matrix values, active inheritance, and affected-row counts.

## Candidate source hashes

| Path | SHA-256 |
|---|---|
| `zircon_runtime/src/scene/world/derived_state.rs` | `fe82df8f26f412cf51d728e152486aff5441e907f54ebcbed9929d8b1af8b045` |
| `zircon_runtime/src/scene/world/derived_state_scale_profile.rs` | `fbc55d04ac16e51c4aaf7d5448bbd3f5c0131f9b7e2df11746ca3dfec495f896` |
| `zircon_runtime/src/scene/tests/derived_state/hierarchy_rebuild.rs` | `8e64e0aa64663f236364da6f57f725bd408ca17e56cc3d8a573f852dd2afc119` |

Pinned Rustfmt and scoped `git diff --check` passed. The new and existing
behavior tests and the Release profile remain pending in managed grouped
validation.
