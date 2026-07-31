---
related_code:
  - dev/bevy/crates/bevy_scene/src/lib.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_light/src/ambient_light.rs
  - dev/bevy/crates/bevy_light/src/rect_light.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/components/scene/hierarchy.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/scene/components/scene/animation.rs
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/scene/tests/support.rs
  - zircon_runtime/src/scene/tests/support/project_fixture.rs
  - zircon_runtime/src/scene/world/project_io.rs
implementation_files:
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/tests/support.rs
  - zircon_runtime/src/scene/tests/support/project_fixture.rs
  - zircon_runtime/src/scene/world/project_io.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs::artifact_store_roundtrips_scene_assets_with_physics_components
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_toml_roundtrip_preserves_ambient_and_rect_lights
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_defaults_new_runtime_foundation_fields_when_omitted
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/asset_scene/product_fields.rs::scene_assets_roundtrip_asset_bound_physics_and_animation_components
  - zircon_runtime/src/scene/tests/asset_scene/product_fields.rs::scene_assets_roundtrip_ambient_and_rect_light_product_fields
doc_type: module-detail
---

# Scene Assets

`zircon_runtime::asset::assets::scene` is the persistent scene document contract. It keeps editor and project files stable while `zircon_runtime::scene::world::project_io` maps saved data into the live `World` component maps.

Bevy's scene system is component oriented: a scene resolves templates into components, registers asset dependencies, and supports field-level patching before spawning. Zircon's TOML scene asset is a simpler persistent form, but it should still carry the same render-facing component families that the runtime can author. That is why camera fields, mesh bindings, and light components are stored beside transform, hierarchy, active state, mobility, physics, and animation data.

## Light Fields

Scene assets now persist the Bevy-aligned light set used by the M5 render product baseline:

- `SceneAmbientLightAsset` stores `color`, `intensity`, and `affects_lightmapped_meshes`. Its defaults mirror Bevy's ambient light defaults: white, brightness-like intensity `80.0`, and lightmapped influence enabled.
- `SceneRectLightAsset` stores `color`, `intensity`, `range`, and `size`. Its defaults mirror Bevy's rect light defaults: white, large cinema-light intensity, range `20.0`, and `1.0 x 1.0` size.
- Directional, point, and spot light assets keep their existing serialized fields.

`World::from_scene_asset(...)` recognizes ambient and rect light records as `NodeKind::AmbientLight` and `NodeKind::RectLight`, then converts array fields into `Vec3`/`Vec2` runtime components. `World::to_scene_asset(...)` converts authored runtime `AmbientLight` and `RectLight` components back into scene asset fields. This closes the persistence gap left after the first M5 authoring/extract slice.

The 2026-06-24 Runtime 15 M3 scene asset integration test folder split keeps this product-field roundtrip coverage under the Runtime 15 test-file budget. `scene/tests/asset_scene.rs` now owns only shared helpers and child mounts, while `scene/tests/asset_scene/product_fields.rs` owns the ambient/rect light product field regression; guard `runtime_15_scene_asset_integration_tests_are_folder_backed` locks that boundary with status `runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred`.

## Test Project Importer Boundary

First-wave image, shader, and model importers are plugin-owned rather than implicit core defaults. Scene integration fixtures therefore open `ProjectManager`, explicitly call `register_first_wave_plugin_fixture_importers_for_test()`, and scan base sources before authoring dependent material and scene documents. Those documents are written through `to_project_toml_string(...)` plus `ProjectManager::persist_runtime_reference(...)`, then scanned in dependency order. This mirrors runtime plugin registration and the current persisted-reference contract while keeping the test project self-contained; failed metadata must not be worked around in `World::load_scene_from_uri(...)`.

The binary artifact cache keeps a separate wire representation for scene rigid bodies. In particular, `ArtifactCachePhysicsMassProperties` represents explicit and auto-from-shape variants without Serde's internally tagged map protocol, which bincode cannot deserialize. Conversion at the artifact boundary preserves the public `PhysicsMassProperties` contract and leaves project TOML unchanged. The artifact-store physics regression uses a non-default auto-from-shape density, while the project fixture exercises the explicit default, so both wire variants remain covered.

## Boundaries

Scene asset persistence does not imply concrete renderer support. Ambient and rect light snapshots still report renderer degradation until PBR shader consumption for those light records is implemented. The asset contract only guarantees that authored light data survives project save/load and reaches the same runtime component maps as editor-created nodes.

Scene light assets have no direct asset references, so `SceneAsset::direct_references()` is unchanged. Camera texture targets, meshes, physics materials, animation resources, terrain, tile maps, and prefabs continue to own reference traversal.
