---
doc_type: optimization-implementation
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
plan_item: RSH-P1-063
related_code:
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
tests:
  - derived_state_default_component_reads_use_direct_branches
  - retained_node_cache_refresh_updates_one_row_and_preserves_slice_pointer
---

# Runtime62 projected-read test repair

## Failure and repair

In the frozen v4 working-tree candidate of `projected_reads.rs` (SHA-256
`adcba07f82f6ee17d7171526cae3872d23d0695017c6cfa98724067db36df846`),
`derived_state_default_component_reads_use_direct_branches` retained
`world_matrix` in its `targeted` array after the streaming-read refactor
removed the local declaration. This is a statically observed unresolved Rust
identifier in that candidate, not a compiler diagnostic: v4 admission failed
before a validation ticket was created. Git `HEAD` still has the declaration,
but the grouped validation was prepared for the working-tree candidate. The
same test expected the
old `entity` spelling inside `propagate_world_matrix` and inlined name fields
inside `refresh_node_cache`; production now uses `current` while traversing and
delegates node construction to `project_node_for_read`.

The repair removes the dangling local from the target list and checks the
actual helper calls without depending on traversal variable names. It also
replaces the stale source-text cache test with a public `World::nodes()`
regression: rename one of two committed nodes, flush, verify the same retained
slice pointer and unchanged peer row, and assert that the derived-state
diagnostic reports one rebuilt node-cache row. Existing dirty projection and
component-ownership behavior tests in the same module remain intact.

This is a test repair, not a production performance change. RSH-P1-063 is
only partially addressed; other source-text guards in this module remain and
should be converted to observable contracts when their owners change.

## Validation

Pinned Rustfmt 1.94.1 and scoped `git diff --check` passed. A direct static
check of the current `derived_state.rs` verified that the repaired helper
assertions evaluate true, and the dangling identifier is gone from the working
tree. The behavior
test has not run because managed Cargo admission is blocked by the changing
external `zr_vm` worktree. No compile or performance pass is claimed.
