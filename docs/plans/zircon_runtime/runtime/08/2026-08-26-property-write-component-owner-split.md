---
related_code:
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/scene/world/property_access/write/animation.rs
  - zircon_runtime/src/scene/world/property_access/write/camera.rs
  - zircon_runtime/src/scene/world/property_access/write/lighting.rs
  - zircon_runtime/src/scene/world/property_access/write/mesh.rs
  - zircon_runtime/src/scene/world/property_access/write/physics.rs
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/property_access.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_world_property_access.rs
implementation_files:
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/scene/world/property_access/write/animation.rs
  - zircon_runtime/src/scene/world/property_access/write/camera.rs
  - zircon_runtime/src/scene/world/property_access/write/lighting.rs
  - zircon_runtime/src/scene/world/property_access/write/mesh.rs
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/property_access.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_world_property_access.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-scene-property-path-compiled-dispatch.md
tests:
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs::world_property_writes_use_direct_optional_state_branches
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs::world_property_writes_pre_size_normalized_segment_vector
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs::world_property_access_moves_into_folder_backed_subtree
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/property_access.rs::review_f5_scene_property_access_uses_scene_error
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_world_property_access.rs::runtime_15_scene_world_property_access_physics_writes_are_child_owner
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 08 property-write component owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M2/M4 | Generic property-write component-domain owner split | `runtime_08_property_write_component_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 610 -> 170 lines; new animation/camera/lighting/mesh owners 194/48/229/127 lines; 91/91 string literals and all measured branch/error/cache counters retained. |

Completed:

- Kept `World::set_property`, missing-entity rejection, one-time segment normalization, and generation publication in the root owner.
- Kept core node, hierarchy, Transform, active, render-layer, and mobility writes in the root route.
- Split camera, mesh, light-family, and animation-family mutation into restricted child owners.
- Retained the existing physics child unchanged and kept dynamic component write fallback last.
- Updated optional-state, typed-error, folder-structure, and production-owner contract tests to inspect the new ownership family.

## Review basis

Unreal separates cached property-path resolution from camera, mesh, and light component mutation owners. Zircon retains its `World` facade and ECS component storage, while matching that responsibility boundary: the path layer resolves and routes; component-domain children own validation and mutation details.

There is no compatibility layer, duplicate implementation, public API expansion, new allocation, dispatch fallback, algorithm replacement, or hotpath instrumentation change.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all nine touched Rust files.
- Static migration comparison retained all 91 string literals with a zero delta.
- `ScenePropertyValue`, `SceneError`, cache-dirty, no-change, resource-construction, and dynamic-fallback occurrence counts match the original owner exactly.
- Root/source contracts confirm all five child mounts, the pre-sized normalization loop, Transform ownership, component-domain routes, typed mesh errors, animation optional-state handling, and dynamic fallback.
- Production files contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Managed Cargo and runtime behavior/performance validation were not run while bypassing the current validation blocker. They remain required before accepted milestone closeout.
- No performance or power improvement is claimed because this slice changes source ownership rather than the property-write algorithm.

## Open scope

Runtime 08 and the full runtime architecture remain `in_progress`. This record closes only generic property-write component ownership. Managed compile/test, property-path behavior suites, representative Inspector/edit profiling, milestone commit, coordinator integration receipt, and WeCom publication remain open.
