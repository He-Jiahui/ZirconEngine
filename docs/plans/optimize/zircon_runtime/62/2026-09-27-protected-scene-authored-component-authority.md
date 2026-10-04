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
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-protected-derived-component-authority.md
tests:
  - zircon_runtime/src/scene/tests/derived_state/protected_authored_authority.rs
  - zircon_runtime/src/scene/tests/derived_state/checked_parent_chain.rs
  - zircon_runtime/src/scene/tests/derived_state/subtree_cycle_walk.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/raw_cycle_tests.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_cycle_tests.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api/component_mutation.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api/persistent_entity_core.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api/persistent_scene_render.rs
  - zircon_runtime/src/scene/tests/render_dirty_journal/render_component_projection.rs
---

# Runtime62 protected Scene-authored component authority

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| Runtime1017 | Close generic ECS mutation of authored hierarchy, local transform, and mobility | `candidate_static_review_complete_managed_validation_pending` | 2026-09-27 | [Astra completion list](../../../astra/features/runtime/1017-runtime62-protected-scene-authored-component-authority-completion-list.md); Batch T source manifest and exact inverse |

## Contract

`Hierarchy`, `LocalTransform`, and `Mobility` are Scene-authored components. Public generic `World::insert` and `World::remove` return `ProtectedAuthoredComponentMutation`; `get_mut` returns `None`. These checks run before registration, change-tick, lifecycle, storage, or generation changes. Read-only component access remains available. `World::set_parent_checked`, `World::update_transform`, and `World::set_mobility` remain the validated edit routes, including their prior missing-parent, cycle, static transform, and mobility errors.

Both mutable query forms, `&mut T` and `Mut<T>`, reject these components during `QueryState::try_new` with `ProtectedAuthoredComponentWrite`; direct mutable fetches and previously bound storage-location fetches also fail closed. Direct and deferred bundle insert/remove reject before publishing a structural segment. Deferred diagnostics retain the typed `SceneError`. Default `NodeRecord` construction, checked reparenting, project normalization, and internal preflight clones retain narrowly scoped trusted writers. The clone route is restricted to `LocalTransform` and `Mobility`; hierarchy remains owned by node-record topology.

Reflected `LocalTransform` and `Mobility` writes continue to validate field values through `ZrReflect`, then commit through the matching Scene setter. The builtin `Hierarchy` adapter uses `set_parent_checked`. Reflected removal of a present authored component reports `NonRemovableComponent`; missing entities and components keep their own diagnostics. Generic adapter registration cannot gain a mutable bypass through World insertion or removal.

Q/R raw-cycle tests now call a `#[cfg(test)]` hierarchy corruption hook. Its direct and tracked forms retain the prior dirty, tick, and pending-query behavior needed by cycle, detached restore, and normalization regressions. Product code cannot call this hook. Scene, graphics, and ECS fixtures that previously spawned authored rows through generic bundles now construct the default node record and edit through checked Scene APIs; render projection and replay assertions remain in place. Persistent entity-core and scene-render clone/serde/record tests now use the same checked authoring route while keeping their read-only storage assertions.

## Verification and remaining gates

New behavior tests cover all three generic types, query construction and direct fetch, bundle and deferred rejection, checked Scene edits, reflected field/slot/batch edits, reflected removal, and preflight clone. Existing cycle and render-product regressions are retained with migrated setup. The focused grouped Release filter is `scene::tests::derived_state::protected_authored_authority`; relevant neighboring groups are `scene::tests::derived_state`, `scene::tests::ecs_typed_api`, `scene::tests::ecs_commands`, `scene::tests::render_dirty_journal`, and `graphics::scene`. None of these new tests is an ignored Release benchmark; `release_filters` is empty for this slice.

The Batch T source candidate is over frozen Batch S v5 (25-path manifest SHA-256 `fd4f9cb620bbec28fab14c7dd90f8ca4ba8997b70633e853f601497c8ebec787`). Two Batch R Scene predecessors were already superseded by S v5, so their Batch T preimages are S v5 bytes. Exact path leases and preimages were captured before edits. Rustfmt, scoped diff, manifest hash, and Markdown structure checks are static evidence only. Managed Cargo compilation and tests have not run for this candidate; the earlier Batch S admission stopped before compilation because of external `zr_vm` drift.

RSH-P0-001 / G01 source ingress is addressed by this candidate, but acceptance remains open until managed compilation and all relevant behavior tests pass. A deferred *checked reparent* command is still a separate product API; generic deferred `Hierarchy` insertion now reports a protected-authority error and does not substitute for that command. G02-G05 graph, static-policy, derived-row, and product integration gates remain open. The constant-time type classifier has no measured Release cost yet, and no 1K/100K/1M entity latency, render-product p99, or absolute performance target is claimed as passed.
