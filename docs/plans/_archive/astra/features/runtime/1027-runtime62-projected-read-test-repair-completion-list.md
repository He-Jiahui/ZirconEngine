---
doc_type: milestone-detail
status: candidate_static_review_complete_managed_validation_pending
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-projected-read-test-repair.md
related_code:
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
tests:
  - derived_state_default_component_reads_use_direct_branches
  - retained_node_cache_refresh_updates_one_row_and_preserves_slice_pointer
---

# Runtime1027: projected-read test repair

| Slice | Work | Status | Evidence |
|---|---|---|---|
| Runtime62 P1-063 test source | Remove a dangling `world_matrix` identifier from the frozen v4 working-tree candidate and update stale source assertions to the current helper boundary. | `candidate_static_review_complete_managed_validation_pending` | V4 preimage SHA-256 `adcba07f82f6ee17d7171526cae3872d23d0695017c6cfa98724067db36df846`; Rustfmt, scoped diff, and direct static assertion checks passed; managed compile/test pending. |
| Retained node cache | Replace a stale implementation-text check with a public two-node rename/flush test for retained slice storage, row values, and one-row diagnostic cost. | `candidate_static_review_complete_managed_validation_pending` | Regression authored; dynamic execution pending. |

- [x] Preserve the existing dirty-read and derived-component behavior regressions.
- [x] Repair the statically observed unresolved identifier in the frozen v4 working-tree test source; no compiler diagnostic or pass is claimed.
- [ ] Run the Runtime lib test batch through managed validation.
- [ ] Convert remaining source-shape guards when their contracts change; RSH-P1-063 is partial.

No performance measurement or Runtime62 G19/G24 acceptance result is claimed.
