---
related_code:
  - zircon_runtime/src/scene/world/property_access/value_conversion.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/compiled.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/domain.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/errors.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/identifiers.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/values.rs
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/property_access.rs
implementation_files:
  - zircon_runtime/src/scene/world/property_access/value_conversion.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/compiled.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/domain.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/errors.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/identifiers.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion/values.rs
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/property_access.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-scene-property-path-compiled-dispatch.md
tests:
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs::world_property_write_segment_expectation_uses_direct_candidate_loop
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs::world_transform_rotation_validation_sums_quaternion_length_directly
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs::world_property_numeric_array_validation_uses_direct_finite_loop
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs::world_property_enum_parsers_match_normalized_values_without_allocation
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs::world_property_write_normalizer_pushes_identifier_characters_directly
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs::world_property_value_conversion_errors_use_direct_result_branches
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs::world_property_value_conversion_facade_keeps_policy_owners_separate
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/property_access.rs::review_f5_scene_property_access_uses_scene_error
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 08 property value-conversion owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M2/M4 | Property value-conversion policy owner split | `runtime_08_property_value_conversion_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 627 -> 21 lines; five child owners 66/208/64/127/200 lines; 43/43 functions and 109/109 string literals retained. |

Completed:

- Replaced the mixed conversion utility with a restricted folder-backed facade.
- Split typed errors, canonical identifiers/axes, compiled-writer adapters, primitive/structured values, and domain conversions into independent owners.
- Preserved all existing `property_access::value_conversion` imports and `World` compiled-adapter visibility.
- Updated conversion behavior, typed-error, and folder-structure contracts to inspect the owning files.
- Added a source contract covering mounts, policy anchors, and per-owner line budgets.

## Review basis

Unreal property-path code separates path data/resolution from typed access, and Bevy separates stable animated-field identity from property evaluation. Zircon now exposes the same lifecycle separation internally while preserving its current ECS/DTO behavior and strict typed errors.

There is no compatibility module, duplicate implementation, public API expansion, algorithm replacement, new allocation, or performance claim.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all nine touched Rust files.
- Static comparison retained all 43 original functions and all 109 original string literals with zero delta.
- Identifier matching, finite validation, typed mismatch, unsupported-value, and resource-construction occurrence counts match the original file.
- Root/source contracts confirm the five private mounts, restricted facade, compiled adapter, domain/error/identifier/value ownership, and a 300-line budget.
- Production files contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Managed Cargo, property behavior, profiling, and power validation were not run while bypassing the current validation blocker.

## Open scope

Runtime 08 and the full runtime architecture remain `in_progress`. This record closes only property value-conversion ownership. Managed compile/test, property-path behavior suites, representative Inspector/edit and compiled-frame profiling, milestone commit, coordinator integration receipt, and WeCom publication remain open.
