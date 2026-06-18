---
related_code:
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/scene/world/project_io/physics.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
  - zircon_runtime/src/scene/world/project_io/transform.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
implementation_files:
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/scene/world/project_io/physics.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
  - zircon_runtime/src/scene/world/project_io/transform.rs
plan_sources:
  - user: 2026-06-14 implement zircon_runtime runtime architecture plan code
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/index.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/world/project_io.rs zircon_runtime/src/scene/world/project_io/camera.rs zircon_runtime/src/scene/world/project_io/physics.rs zircon_runtime/src/scene/world/project_io/post_process.rs zircon_runtime/src/scene/world/project_io/references.rs zircon_runtime/src/scene/world/project_io/script.rs zircon_runtime/src/scene/world/project_io/transform.rs
  - runtime_07_project_io_folder_split_keeps_entry_and_converter_owners
  - performance_hotpath_boundary_audit reports large_file_hotspot_count = 40 and runtime-other = 15 after the render product diagnostics owner split removed one runtime hotspot
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked attempted 2026-06-14; timed out while unrelated active editor/render lanes were compiling
doc_type: module-detail
---

# Scene World Project I/O

## Purpose

`zircon_runtime::scene::world::project_io` owns project-file and scene-asset roundtrip behavior for `World`. It loads authored `SceneAsset` data through `ProjectManager`, reconstructs runtime components and resource handles, serializes a `World` back to project JSON or `SceneAsset`, and repairs derived runtime state after loading.

The 2026-06-14 Runtime 07 Project I/O Folder Split keeps the public entry points in `project_io.rs` but moves schema conversion helpers into `project_io/{camera,physics,post_process,references,script,transform}.rs`. This is a behavior-preserving boundary split: callers still use `World::load_scene_from_uri`, `World::from_scene_asset`, `World::save_project`, `World::load_project`, `World::to_project_json`, and `World::to_scene_asset`.

## Related Files

`project_io.rs` is the orchestration file. It owns `SceneProjectError`, `ProjectDocument`, the `World` impl, project file I/O, entity iteration, component assembly, and post-load state repair. After the split, `project_io.rs 772 行` and remains below the repository large-file warning threshold.

The child modules each own one conversion family:

- `references.rs` maps `AssetReference` values to typed `ResourceHandle<T>` values and back, including builtin fallback locators.
- `camera.rs` converts camera targets, viewport rectangles, and `CameraComponent` values.
- `post_process.rs` converts render post-process settings, volumes, profiles, tonemap, vignette, grain, dither, chromatic aberration, and fog DTOs.
- `physics.rs` converts collider shapes.
- `script.rs` decodes the stored `script.bindings` dynamic component payload.
- `transform.rs` converts `TransformAsset` and runtime transforms.

## Behavior Model

Loading starts from either a `ResourceLocator` or a `SceneAsset`. The project manager resolves imported scene artifacts and asset references; builtin locators remain builtin handles, while missing model or material references fall back to explicit missing-resource handles. Component DTOs are translated into runtime scene components before the node record is inserted.

Saving walks runtime node records and component maps, converts runtime components back into scene asset DTOs, serializes script bindings from the dynamic component map, and returns structured `SceneProjectError::SceneAsset` errors when a persistent resource locator is missing. No editor-only authoring state is serialized here.

## Design and Rationale

The old single file mixed project document I/O, resource locator mapping, script binding payload decode, transform conversion, camera conversion, post-process conversion, and collider conversion. That made `project_io.rs` a Runtime 07 `runtime-other` large-file hotspot.

The split follows the current scene/world owner shape rather than introducing a public facade. `project_io.rs` still hides the implementation behind the existing `World` API, and each child module uses `pub(super)` helpers so callers outside project I/O cannot bind to internal conversion details.

## Control Flow

The entry file decides when a scene is loaded, saved, serialized, or rehydrated. It delegates value conversion to child modules at the point where a component field crosses the asset/runtime boundary. The post-load rehydration step stays in the entry file because it mutates multiple `World` maps and rebuilds runtime registries, schedules, derived state, and active camera/light defaults.

## Edge Cases and Constraints

Builtin locators are intentionally preserved by `references.rs`; they must not roundtrip through project UUID lookup. Missing model and material references still produce stable missing handles instead of panicking.

Camera texture targets require a persistent locator on export. When none exists, `reference_for_handle` returns `SceneProjectError::SceneAsset` with the resource label so the authoring layer can surface a concrete project-data problem.

Script bindings remain JSON-decoded from the dynamic component store under `script.bindings`. Invalid binding payloads are reported as scene asset errors tied to the entity id.

## Test Coverage

`runtime_07_project_io_folder_split_keeps_entry_and_converter_owners` locks the project_io folder split. It verifies the entry file declares the six child modules, keeps the `World` project I/O entry points, does not reclaim converter helper definitions, and requires each child module to retain its narrow `pub(super)` conversion owner.

The Runtime 07 owner-budget evidence now reports `large_file_hotspot_count = 40` and `runtime-other = 15` after the render product diagnostics owner split removed one runtime hotspot and added the corresponding Runtime 07 guard. `performance_hotpath_boundary` now has `hotspot_guard_anchor_count = 25` after adding the render product diagnostics owner split guard. Package-level Cargo validation remains pending because the shared Windows workspace had active editor/render compile lanes during this slice.

## Plan Sources

This module split implements the Runtime 07 large-file owner-budget rule from `docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`. The rule says Runtime 07 optimization work must split large runtime hotspots by behavior family before adding more performance logic.

## Open Issues or Follow-up

The split is structural only. It does not change serialization semantics, resource lookup behavior, scene asset schema, or runtime FPS. The remaining Runtime 07 acceptance gates are still extract, ECS query, profiling/FPS, and clean package Cargo validation.
