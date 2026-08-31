---
related_code:
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene/entity.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene/physics.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene/rendering.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene/script.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_script.rs
implementation_files:
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene/entity.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene/physics.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene/rendering.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene/script.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
tests:
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs::artifact_store_roundtrips_scene_assets_with_mesh_references
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs::artifact_store_roundtrips_scene_assets_with_camera_targets
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs::artifact_store_roundtrips_scene_assets_with_physics_components
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_script.rs::artifact_store_roundtrips_scene_assets_with_script_binding_json_values
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 04 scene cache payload owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M1/M2 | Scene artifact cache payload folder-backed owner split | `runtime_04_scene_cache_payload_owner_split_implemented_static_passed_managed_validation_deferred` | 2026-08-26 | Root 779 -> 38 lines; four production child owners 136/358/231/44 lines; 14/14 serialized type blocks and 28/28 conversion functions retained. |

Completed:

- Kept only the serialized scene envelope and top-level scene conversion in the root owner.
- Split entity aggregation and component-family composition into `entity.rs`.
- Split mesh/LOD/camera wire projection into `rendering.rs`.
- Split rigid-body/collider/joint wire projection into `physics.rs`.
- Split script binding and cache JSON conversion into `script.rs`.
- Preserved cache-only visibility; no cache DTO became a public Runtime API.
- Left the artifact manifest/version, parent cache payload facade, and currently modified artifact test root untouched.

## Review basis

Unreal keeps object serialization at the object boundary while actor component domains own their concrete data. Zircon's public scene asset model already uses `entity/camera/mesh/physics/extensions` owners. This slice makes the cache wire conversion follow those same domain boundaries while preserving one scene envelope and one bincode representation.

There is no compatibility module, duplicate wire model, schema-version change, generic helper dump, public API expansion, algorithm replacement, or hotpath instrumentation change.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all five touched Rust files.
- Scoped `git diff --check` passed, apart from LF/CRLF checkout notices.
- Static type-block comparison retained all 14 serialized struct/enum definitions with zero field/type/serde/variant mismatch after removing only visibility and whitespace differences.
- Static function comparison retained all 28 conversion definitions and function-name multiplicities.
- All four child mounts exist; the root is 38 lines and the largest child owner is 358 lines.
- Production files contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Existing mesh, camera, physics, and script artifact round-trip tests remain the behavior gate, but managed Cargo was not run while bypassing the current validation blocker.
- No CPU, I/O, allocation, artifact-size, energy, or power improvement is claimed because this slice does not change serialization behavior.

## Open scope

Runtime 04 and the full runtime architecture remain `in_progress`. This record closes only the source ownership implementation for the scene artifact cache payload. Managed compile/test, artifact-byte round-trip evidence, wider structure/performance guards, milestone commit, coordinator integration receipt, and WeCom publication remain open.
