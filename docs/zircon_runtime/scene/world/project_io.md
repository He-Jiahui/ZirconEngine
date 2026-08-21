---
related_code:
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/scene/world/project_io/document.rs
  - zircon_runtime/src/scene/world/project_io/physics.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
  - zircon_runtime/src/scene/world/project_io/transform.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/tests/runtime_camera_core_pipeline_contract.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
implementation_files:
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/scene/world/project_io/document.rs
  - zircon_runtime/src/scene/world/project_io/physics.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
  - zircon_runtime/src/scene/world/project_io/transform.rs
  - zircon_runtime/src/asset/assets/scene/camera.rs
plan_sources:
  - user: 2026-06-14 implement zircon_runtime runtime architecture plan code
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/world/project_io.rs zircon_runtime/src/scene/world/project_io/camera.rs zircon_runtime/src/scene/world/project_io/document.rs zircon_runtime/src/scene/world/project_io/physics.rs zircon_runtime/src/scene/world/project_io/post_process.rs zircon_runtime/src/scene/world/project_io/references.rs zircon_runtime/src/scene/world/project_io/scene_asset.rs zircon_runtime/src/scene/world/project_io/script.rs zircon_runtime/src/scene/world/project_io/transform.rs
  - rustfmt --check zircon_runtime/src/scene/world/project_io.rs zircon_runtime/src/scene/world/render.rs zircon_runtime/src/scene/tests/asset_scene.rs zircon_runtime/src/scene/tests/dynamic_scene_session/capture.rs zircon_runtime/src/scene/tests/dynamic_scene_session/load.rs
  - scene::tests::asset_scene::scene_assets_keep_script_only_entities_as_empty_nodes
  - scene::tests::asset_scene::scene_asset_load_uses_asset_preserving_normalizer_source_guard
  - scene::tests::dynamic_scene_session::capture::runtime_session_archive_capture_level_slot_to_existing_path_upserts_and_preserves_other_slots
  - scene::tests::dynamic_scene_session::capture::runtime_session_archive_previews_capture_to_path_without_writing_archive
  - scene::tests::dynamic_scene_session::load::runtime_session_archive_applies_slot_from_path_to_live_world_and_level
  - runtime_07_project_io_folder_split_keeps_entry_and_converter_owners
  - runtime_camera_core_pipeline_contract (3 passed)
  - performance_hotpath_boundary_audit reports large_file_hotspot_count = 40 and runtime-other = 15 after the render product diagnostics owner split removed one runtime hotspot
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked attempted 2026-06-14; timed out while unrelated active editor/render lanes were compiling
  - cargo test -p zircon_runtime --lib scene_asset --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-closeout-0619 --message-format short --color never -- --test-threads=1 --nocapture attempted 2026-06-20; timed out after 10 minutes with no pass/fail result
  - cargo test -p zircon_runtime --lib scene::tests::asset_scene::scene_asset_load_uses_asset_preserving_normalizer_source_guard --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-closeout-0619 --message-format short --color never -- --exact --test-threads=1 --nocapture attempted 2026-06-20; timed out after 15 minutes with no pass/fail result and residual cargo/rustc processes were stopped
doc_type: module-detail
---

# Scene World Project I/O

## Purpose

`zircon_runtime::scene::world::project_io` owns project-file and scene-asset roundtrip behavior for `World`. It loads authored `SceneAsset` data through `ProjectManager`, reconstructs runtime components and resource handles, serializes a `World` back to project JSON or `SceneAsset`, and repairs derived runtime state after loading.

The 2026-08-11 project I/O owner split keeps the public `World` API unchanged while placing scene-asset conversion and project-document I/O in separate implementation modules. `project_io.rs` is project_io root wiring; `scene_asset.rs` is the scene-asset conversion owner; `document.rs` owns JSON document encoding, decoding, and post-load normalization. Callers continue to use `World::load_scene_from_uri`, `World::from_scene_asset`, `World::to_scene_asset`, `World::save_scene_to_project`, `World::save_project_to_path`, and `World::load_project_from_path`. The existing narrow helpers remain in `project_io/{camera,document,physics,post_process,references,scene_asset,script,transform}.rs`.

## Related Files

`project_io.rs` is a 15-line module declaration and `SceneProjectError` re-export. `scene_asset.rs` owns URI import/export, entity iteration, runtime component assembly, and scene-asset post-load normalization. `document.rs` owns `SceneProjectError`, project document DTOs, bounded JSON I/O, and project-document post-load normalization. Each implementation file remains below the repository production-file soft budget.

The child modules each own one conversion family:

- `references.rs` maps `AssetReference` values to typed `ResourceHandle<T>` values and back, including builtin fallback locators.
- `scene_asset.rs` converts `SceneAsset`/`SceneEntityAsset` values and owns `World::{load_scene_from_uri,from_scene_asset,to_scene_asset,save_scene_to_project}`.
- `document.rs` serializes and reloads project documents and owns `World::{save_project_to_path,load_project_from_path}` plus the shared normalizer.
- `camera.rs` converts camera targets, viewport rectangles, explicit Core2d/Core3d identity, and `CameraComponent` values.
- `post_process.rs` converts render post-process settings, volumes, profiles, tonemap, vignette, grain, dither, chromatic aberration, and fog DTOs.
- `physics.rs` converts collider shapes.
- `script.rs` decodes the stored `script.bindings` dynamic component payload.
- `transform.rs` converts `TransformAsset` and runtime transforms.

## Behavior Model

Loading starts from either a `ResourceLocator` or a `SceneAsset`. The project manager resolves imported scene artifacts and asset references; builtin locators remain builtin handles, while missing model or material references fall back to explicit missing-resource handles. Component DTOs are translated into runtime scene components before the node record is inserted.

`World::from_scene_asset` uses the asset-preserving post-load path: it rebuilds schedules, registries, typed component presence, derived state, and default per-entity maps without injecting a fallback camera or directional light. This keeps sparse assets such as script-only entities and transform-only hierarchies stable across `SceneAsset -> World -> SceneAsset` roundtrips. Project-document loading still uses the default-repair path so older serialized `World` files with no camera or light can regain runtime defaults.

Saving walks runtime node records and component maps, converts runtime components back into scene asset DTOs, serializes script bindings from the dynamic component map, and returns structured `SceneProjectError::SceneAsset` errors when a persistent resource locator is missing. No editor-only authoring state is serialized here.

Camera conversion preserves `core_pipeline` independently from `projection_mode` in both directions. Missing scene fields default to `Core3d`; explicit sprite-camera `Core2d` survives `SceneCameraAsset -> CameraComponent -> SceneCameraAsset`. This prevents orthographic 3D/PBR cameras from silently entering the Core2d schedule after project load or artifact-cache restore.

## Design and Rationale

The old single file mixed project document I/O, scene-asset conversion, resource locator mapping, script binding payload decode, transform conversion, camera conversion, post-process conversion, and collider conversion. The 2026-08-11 split removes the root-file hotspot without adding a facade or a second public API.

The split follows the current scene/world owner shape rather than introducing a public facade. The `World` API remains the only public surface, while conversion helpers continue to use `pub(super)` visibility so callers outside project I/O cannot bind to internal details.

## Control Flow

The scene-asset module decides when a scene is loaded or saved and delegates value conversion at each asset/runtime boundary. The document module owns JSON rehydration because it rebuilds runtime registries, schedules, derived state, and active camera/light defaults. Its shared `normalize_loaded_state` helper takes an explicit default-node policy so project-document recovery can add runtime defaults while scene-asset import preserves the authored entity set exactly.

## Edge Cases and Constraints

Builtin locators are intentionally preserved by `references.rs`; they must not roundtrip through project UUID lookup. Missing model and material references still produce stable missing handles instead of panicking.

Camera texture targets require a persistent locator on export. When none exists, `reference_for_handle` returns `SceneProjectError::SceneAsset` with the resource label so the authoring layer can surface a concrete project-data problem.

Script bindings remain JSON-decoded from the dynamic component store under `script.bindings`. Invalid binding payloads are reported as scene asset errors tied to the entity id.

Project-document and scene-asset normalization rebuild the next entity allocation cursor with checked arithmetic before mutating schedules, derived registries, or default nodes. Project normalization precomputes whether a default camera and directional light are missing, reserves both IDs, and requires the post-default cursor to remain allocatable before it creates either node. A persisted `u64::MAX` entity, or a state whose restored/default-node allocation would reach the reserved maximum, returns `SceneProjectError::ProjectNormalization { path, source: SceneError::EntityIdExhausted }` for path-based project loads instead of panicking in debug builds, wrapping the cursor to zero in release builds, or returning a world with a reserved next cursor. Scene-asset normalization retains the typed `SceneError::EntityIdExhausted` source at its non-document boundary.

Project-document loading also preflights every persisted component map before rebuilding typed component projections. A local transform map is checked first so invalid transform data returns its typed `SceneError` (for example, `ZeroScaleTransform`) rather than reaching an internal projection panic. A structurally valid component whose entity is absent from the serialized entity list returns `SceneError::MissingEntity { operation: "load persisted component", .. }`. Direct `World` deserialization rejects the same orphaned-component state as a parse error, so no public deserialization route can construct an invalid projection.

## Test Coverage

`runtime_07_project_io_folder_split_keeps_entry_and_converter_owners` locks the project_io folder split. It verifies root wiring, scene-asset conversion ownership, document I/O ownership, the existing helper modules, and the absence of conversion or document behavior from the root file.

Runtime 05 scene-asset closeout now pins the asset-preserving normalizer through `scene_assets_keep_script_only_entities_as_empty_nodes`: script-only entities remain `NodeKind::Empty`, keep `script.bindings`, and no longer gain default camera/light records during `SceneAsset` roundtrip. The same test also calls `World::to_render_extract()` so sparse asset worlds keep a safe render-extract path without persisting fallback camera/light nodes. Dynamic session single-entity fixtures use explicit `World::empty()` levels when they are testing remap collisions rather than default-level bootstrap contents.

`scene_asset_load_uses_asset_preserving_normalizer_source_guard` adds source-level coverage for the same contract: `World::from_scene_asset` must call `normalize_scene_asset_after_load`, scene-asset normalization must pass `ensure_default_nodes = false`, project-document normalization must pass `true`, and default camera/light spawning must stay gated behind `ensure_default_nodes`.

`project_load_rejects_exhausted_entity_ids_without_panicking` writes an exhausted ID into a serialized project document and requires both the exact document path and typed exhaustion source. `project_load_rejects_default_node_allocation_exhaustion_without_panicking` additionally covers `u64::MAX - 2`, where the old second default spawn overflowed, and `u64::MAX - 3`, where the old load returned a reserved final cursor. The tests guard both build profiles because the old unchecked addition had different debug and release failure modes.

`project_load_rejects_invalid_orphan_local_transform` proves an orphaned transform with a zero scale fails with `SceneError::ZeroScaleTransform` before projection rebuild. `project_load_rejects_valid_orphan_component_without_panicking` covers the structurally valid counterpart and requires the typed missing-entity result, preventing the prior `component projection value must belong to a registered world entity` panic from returning.

The 2026-06-20 Runtime 05 Cargo verification window did not produce a usable result: the broader `scene_asset` filter timed out after 10 minutes, then the exact source-guard test timed out after 15 minutes and its residual cargo/rustc processes were stopped. No Cargo pass or failure is claimed from those attempts.

The Runtime 07 owner-budget evidence now reports `large_file_hotspot_count = 40` and `runtime-other = 15` after the render product diagnostics owner split removed one runtime hotspot and added the corresponding Runtime 07 guard. `performance_hotpath_boundary` now has `hotspot_guard_anchor_count = 25` after adding the render product diagnostics owner split guard. Package-level Cargo validation remains pending because the shared Windows workspace had active editor/render compile lanes during this slice.

## Plan Sources

This module split implements the Runtime 07 large-file owner-budget rule from `docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`. The rule says Runtime 07 optimization work must split large runtime hotspots by behavior family before adding more performance logic.

## Open Issues or Follow-up

The split is structural only. It does not change serialization semantics, resource lookup behavior, scene asset schema, or runtime FPS. The remaining Runtime 07 acceptance gates are still extract, ECS query, profiling/FPS, and clean package Cargo validation.
