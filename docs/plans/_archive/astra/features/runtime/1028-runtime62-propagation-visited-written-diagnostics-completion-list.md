---
doc_type: milestone-detail
status: candidate_static_review_complete_managed_validation_pending
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-derived-propagation-written-row-diagnostics.md
related_code:
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/performance_diagnostics.rs
  - zircon_runtime/src/scene/tests/derived_state/work_counters.rs
  - docs/crates/zircon_runtime/scene/ecs.md
tests:
  - derived_state_full_rebuild_baseline_is_deterministic_for_one_and_one_thousand_nodes
  - derived_state_full_rebuild_baseline_is_deterministic_for_one_hundred_thousand_nodes
  - derived_state_leaf_transform_change_rebuilds_only_affected_rows
  - derived_state_parent_transform_change_rebuilds_the_affected_subtree
  - derived_state_leaf_active_change_rebuilds_only_the_affected_row
  - derived_state_structured_reparent_avoids_global_hierarchy_work_at_one_thousand_nodes
  - derived_state_structured_reparent_avoids_global_hierarchy_work_at_one_hundred_thousand_nodes
  - ecs_frame_performance_diagnostics_exports_propagation_visit_and_write_counts
---

# Runtime1028: derived propagation visited and written rows

| Slice | Work | Status | Evidence |
|---|---|---|---|
| Runtime62 derived-state counters | Preserve existing propagation entity counts as visited rows and add per-component written-row counts through the existing diagnostics export path. | `candidate_static_review_complete_managed_validation_pending` | Full rebuild counts every newly published row; leaf transform/active behavior counts one write; equal checked reparent visits two rows with zero active or matrix writes. |
| Diagnostic export | Publish active and world-matrix write counts as stable `scene.ecs.derived_state.*_written_entities` series. | `candidate_static_review_complete_managed_validation_pending` | Direct-store regression asserts existing visit values `7/11` and write values `3/5`; series count adjusted from 58 to 60. |

- [x] Count writes locally in each propagation traversal and record totals once per pass.
- [x] Keep visit and write meanings separate in fields and exported metric keys.
- [x] Run pinned edition-2021 rustfmt check and static byte/whitespace checks.
- [ ] Run the grouped managed Runtime behavior tests.
- [ ] Add/run truthful 1K/100K/1M visited, written, allocated-byte, and latency evidence for G24.

G24 is only partially addressed by clarifying and testing visited versus
written row counts. Its 1K/100K/1M allocation-byte and latency measurements,
raw samples, and numeric acceptance ceiling remain outstanding. No ceiling has
been invented, and no performance pass is claimed.
