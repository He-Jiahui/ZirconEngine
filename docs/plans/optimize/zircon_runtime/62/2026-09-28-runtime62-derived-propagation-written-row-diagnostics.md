---
doc_type: optimization-implementation
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
gate: RSH-G24
related_code:
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/performance_diagnostics.rs
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

# Runtime62 RSH-G24: propagation visit and write counters

## Scope

Keep `active_propagation_entities` and `world_matrix_propagation_entities` as
visited-row counts, and add separate
`active_propagation_written_entities` and
`world_matrix_propagation_written_entities` counters. Each propagation pass
accumulates visits and writes locally. A write is counted only when a derived
value differs or is missing and `replace_derived_component` updates the
`ActiveInHierarchy` or `WorldMatrix` row. The pass records both totals once
after its frontier traversal; it does not touch frame diagnostics per node.

The existing diagnostic export path now publishes both written-row counts as
`scene.ecs.derived_state.active_propagation_written_entities` and
`scene.ecs.derived_state.world_matrix_propagation_written_entities`.
Behavior coverage includes initial rebuilds, actual transform and active
changes, plus equal-value checked reparenting where two rows are visited but
neither derived component is rewritten.

This completes only the visited-versus-written counter semantics subset of
G24, not scale acceptance. It does not provide allocated-byte or latency
measurements, raw samples, or the 1K/100K/1M profile. The numeric acceptance
ceiling remains outstanding; none is defined or claimed here.

## Candidate source hashes

These hashes identify the current shared working-tree candidate bytes; files
with pre-existing edits retain those edits.

| Path | SHA-256 |
|---|---|
| `zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs` | `e147be608fd4031dbb3fbdffe41c23237529cb46cf8479d49a77fd9391f1f1f8` |
| `zircon_runtime/src/scene/ecs/mod.rs` | `9dd843e39db16aa00bbe4657589e94908f45247e82ba9ec32c78856439668a1b` |
| `zircon_runtime/src/scene/world/derived_state.rs` | `17a097e4195a5ee534033b506cb3b942ff0962771093c93b659c0904544851e3` |
| `zircon_runtime/src/scene/world/performance_diagnostics.rs` | `147d8372c23996e8752a111d97ef5178eed9082a2462aad1032b3a5bd16eb086` |
| `zircon_runtime/src/scene/tests/derived_state/work_counters.rs` | `61ff68f939a34d5d2b9ec02c2c6393bb0b1175429177aeb1164a2074841c77be` |
| `docs/crates/zircon_runtime/scene/ecs.md` | `2bd2d8f1ee324d8a4c01d49098555fac18b37d56390cd6192d6cd8822bf49a22` |

## Static review and open gate

Pinned `rustfmt +1.94.1 --check --edition 2021 --config skip_children=true`
passed for the five Rust paths. `git diff --check` passed for the four tracked
Rust paths and the edited ECS module documentation; direct final-LF and
trailing-whitespace checks passed for those six paths, including the untracked
test file. No Cargo or compiler command was run.

G24 remains open. Managed Runtime behavior tests and raw 1K/100K/1M visited,
written, allocation-byte, and latency evidence are still required, together
with a defined numeric acceptance ceiling. No G24 completion, measured pass,
or threshold pass is claimed.
