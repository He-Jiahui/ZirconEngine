---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/typed_api/bundle_transaction/staging.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/change_detection.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/hierarchy_validation.rs
  - zircon_runtime/src/scene/ecs/query/query_data.rs
  - zircon_runtime/src/scene/reflect/derived/component_adapter.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection/hierarchy.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-protected-scene-authored-component-authority.md
tests:
  - zircon_runtime/src/scene/tests/derived_state/protected_authored_authority.rs
  - zircon_runtime/src/scene/tests/derived_state/checked_parent_chain.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api/persistent_entity_core.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api/persistent_scene_render.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/raw_cycle_tests.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_cycle_tests.rs
---

# Runtime1017: protected Scene-authored component authority

Status: `candidate_static_review_complete_managed_validation_pending`.

- [x] Save exact predecessor hashes over frozen Batch S v5 and existing Batch R successors; claim all changed source and documentation paths under the shared Session.
- [x] Close generic World insert/get-mut/remove of `Hierarchy`, `LocalTransform`, and `Mobility`, including direct mutable query and bound-location fetches, bundle staging, and deferred diagnostics.
- [x] Preserve checked Scene setters, default node-record construction, normalization, internal preflight clone, and derived-state rebuild without public raw mutation.
- [x] Route reflected authored writes through checked Scene setters and return a non-removable reflection error for present protected rows.
- [x] Move Q/R raw-cycle fixtures to `#[cfg(test)]` corruption hooks; migrate ECS, graphics, and persistent clone/serde/record fixtures to checked Scene authoring.
- [x] Add behavior regressions for rejection, unchanged state, valid edits, reflection, preflight clone, cycle, and render-product setup; update ECS/query/reflection public contracts.
- [x] Complete scoped static checks and seal exact source/preimage hash manifest and inverse for Batch T; no ignored Release test was added.
- [ ] Obtain grouped managed Cargo compilation and relevant behavior-test acceptance. Pending or rejected tickets do not count as a pass.
- [ ] Measure Release overhead and close Runtime62 product-scale latency and render integration gates; no performance threshold is claimed met.
- [ ] Provide a dedicated checked deferred reparent API if product callers need that operation, then validate its staged topology and failure atomicity.

RSH-P0-001 / G01 has a source candidate, not accepted completion. G02-G05 and wider scene product gates remain open. No direct Cargo, coordinator submission, compiler polling, tooling edit, commit, push, or external notification was performed in this slice.
