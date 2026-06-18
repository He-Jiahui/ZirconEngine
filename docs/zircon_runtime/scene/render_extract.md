---
related_code:
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/render_extract/mod.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/ecs/internal_scene_system.rs
  - zircon_runtime/src/scene/ecs/system_stage.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_profile.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_evaluator.rs
  - zircon_runtime/src/core/framework/render/post_process/exposure_settings.rs
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
implementation_files:
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/render_extract/mod.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_profile.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_evaluator.rs
  - zircon_runtime/src/core/framework/render/post_process/exposure_settings.rs
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
plan_sources:
  - user: 2026-05-08 ECS to render chain milestone execution
  - .codex/plans/ZirconEngine ECS 到渲染链路完善里程碑计划.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m03-canonical-render-extract/plan.md
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - user: 2026-06-10 vampire screen-space HUD, buff particles, shader lighting, and no model health bars
  - user: 2026-06-11 vampire roguelite runtime example and screenshot validation
  - user: 2026-06-11 vampire runtime point light illumination
tests:
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::render_extract_projects_scene_camera_component_product_fields
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_collects_dynamic_particle_sprites_by_camera_layers
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_collects_dynamic_particle_gpu_frames_by_camera_layers
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_carries_scene_camera_order_report_for_scene_camera
  - zircon_runtime/src/scene/tests/render_extract.rs::explicit_camera_render_frame_extract_has_no_scene_camera_order_report
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_keeps_custom_target_layer_geometry_for_visibility_views
  - zircon_runtime/src/core/framework/render/camera_ordering.rs::tests::render_camera_order_report_carries_descriptor_render_type
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/physics_animation_components.rs
  - tests/acceptance/ecs-to-render-chain.md
  - .opencode/workflows/20260531_215744_101_完善ECS到渲染工作流，你可以参照dev 下面graphics的unity的SRP工作流以及unrealEngine虚幻源码渲染能力、bevy fyrox等对w/m03-canonical-render-extract/validation-evidence.md
  - zircon_runtime/src/core/framework/render/light/readiness.rs::light_status_counts_split_ready_and_degraded_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::scene_uniform_packs_authored_point_lights
  - .github/workflows/ci.yml
  - cargo test -p zircon_runtime --lib render_frame_extract_collects_dynamic_particle_sprites_by_camera_layers --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10
  - cargo test -p zircon_runtime --lib render_frame_extract_collects_world_hud_health_bars_as_scene_particles --message-format short --color never: passed 2026-06-11
doc_type: module-detail
---

# Scene Render Extract

The scene render-extract boundary turns authoritative `World` or `LevelSystem` state into `RenderFrameExtract`, the neutral frame DTO consumed by the renderer. In the current M3 canonical render-extract milestone, the important contract is both execution order and DTO authority: native dirty-state systems and plugin hooks must run before render extraction observes world transforms, active state, render-layer masks, and animation pose sidebands, and the scene producer must populate `RenderFrameExtract` sections directly rather than adapting through `SceneViewportRenderPacket`.

## Ownership

`World` remains the runtime scene authority. Public `World` render helpers that take `&self` clone the world and build the extract on the clone, preserving existing callers that expect read-only access while leaving the source world's dirty bits unchanged. The prepared path takes `&mut World` and is used by `LevelSystem` so scheduled render extraction can flush authoritative dirty state instead of producing a stale snapshot.

`LevelSystem` implements `RenderExtractProducer` by calling `with_world_mut(...)`, building the prepared scene extract, and then merging cached animation poses into `RenderFrameExtract::animation_poses`. This keeps animation pose extraction level-owned while scene geometry, camera, lights, active state, and transforms continue to come from `World`.

## Prepared Extract Path

The prepared path is:

1. `LevelSystem::build_render_frame_extract(...)` enters `World` mutably.
2. `World::build_prepared_render_frame_extract(...)` delegates to `World::build_prepared_render_frame_extract_for_request(...)`.
3. `World::build_prepared_render_frame_extract_for_request(...)` runs the `RenderExtract` built-in systems before reading camera, mesh, light, post-process settings, post-process volume, active, transform, and layer data.
4. The world assembles `RenderViewExtract`, `GeometryExtract`, `LightingExtract`, `PostProcessExtract`, `DebugOverlayExtract`, `ParticleExtract`, and `VisibilityInput` directly. `PostProcessExtract` keeps base defaults, seeds manual exposure from the effective camera `exposure_ev100`, and carries the scene-authored volume DTOs for submit-side resolution.
5. `LevelSystem` appends animation pose sidebands for mesh entities with skeletons.

`SceneViewportRenderPacket` remains available through `to_render_snapshot()` / `to_render_extract()` for preview and roundtrip callers, and `RenderFrameExtract::from_snapshot(...)` remains a framework adapter for tests or legacy snapshot owners. The scene producer no longer uses that adapter for frame extraction.

## Snapshot Contents

`World::build_prepared_render_frame_extract_for_request(...)` emits sorted meshes, directional lights, point lights, rect lights, spot lights, and active ambient light records. Mesh rows include stable node id, world transform, model handle, material handle, tint, mobility, and render-layer mask. Light row types live under `render::light`; `LightingExtract` only aggregates those rows with reflection-probe, baked-lighting, and Hybrid GI sidebands. Rect light rows follow Bevy's orientation contract by deriving the emitted direction from the entity transform's forward vector, while keeping the authored color, intensity, range, and size in `RenderRectLightSnapshot`. Ambient light snapshots are no longer marked renderer-degraded because the basic forward/deferred scene uniform now folds active authored ambient color times intensity into `SceneUniform::ambient_color`; rect lights remain renderer-degraded until a concrete area-light shader path lands. The prepared frame path also builds `GeometryPhaseInput` from the same sorted mesh rows and each `MeshRenderer.material_alpha_mode`, so mesh indices and phase classification stay aligned for opaque, alpha-mask, and transparent queues. Camera rows preserve explicit viewport-request overrides and derive aspect ratio from the request size when present.

The phase inputs now expose the full render-order surface needed by the WGPU main chain: render queue, material queue, depth, depth bias, order in layer, UI z-index, and entity tie-breaker. Current scene components still populate only the stable defaults plus material alpha/depth/z-order, but the DTO and queue builder contract are ready for material cache, sprite atlas, UI graph, and renderer-specific ordering data without adding a second sorting path.

`World::collect_render_particles(...)` is the scene-owned bridge from gameplay-authored dynamic components into `ParticleExtract`. Runtime scripts and fallback project gameplay write transient JSON to `render.particle_sprites` or `gameplay.particle_sprites`; extraction parses those sprite records, filters inactive entities and camera-layer mismatches, records emitter bounds, sorts sprite billboards from the selected camera position, and adds emitters to `VisibilityInput` as dynamic renderables. This keeps attack VFX data in the same frame DTO as meshes, sprites, lights, and post-process state instead of spawning temporary mesh entities for every effect.

The same dynamic particle payloads may also carry a neutral `gpu_frame` object. Scene extraction parses visible `gpu_frame` summaries, aggregates `alive_count`, `spawned_total`, `per_emitter_spawned`, and non-indexed indirect args `[6, alive_count, 0, 0]` into `ParticleExtract.gpu_frame`, and folds optional GPU bounds into the same emitter bounds used by visibility. This remains a scene/framework DTO contract: `World` does not import WGPU, the particles plugin, or the plugin manager. The later particles runtime-prepare collector consumes that neutral frame and creates the `particles.gpu.*` buffer set when no concrete shared-manager backend was executed for the frame.

The same extraction owner now also accepts world-space HUD bar payloads through `render.world_hud_bars` or `gameplay.world_hud_bars`. Vampire gameplay writes compact health bar records for the player and spawned enemies; render extraction converts each bar into a small set of camera-facing particle pips anchored above the entity. These generated HUD sprites use nonzero stable sprite keys and set `RenderParticleSpriteSnapshot.depth_test = false`, so the renderer sends them through the particle overlay color path instead of the depth-tested particle path or the particle velocity writer. This keeps health presentation in the frame DTO and avoids attaching temporary child meshes to every actor. The focused regressions `render_frame_extract_collects_world_hud_health_bars_as_scene_particles`, `world_hud_bar_sprites_use_overlay_depth_path`, and `particle_velocity_vertices_skip_overlay_sprites` cover parsing, camera-layer visibility, particle extraction, overlay depth routing, and velocity exclusion for these HUD bars.

Scene-backed frame extracts also attach camera scheduling metadata to `RenderViewExtract`: the selected scene camera entity, the `RenderCameraOrderReport` produced from all active scene cameras, and the Plan 09 `cameras: Vec<CameraRenderDescriptor>` list. This mirrors Bevy's render-app `SortedCameras` resource while preserving Zircon's current single-effective-camera frame shape. The report now carries each active scene camera's descriptor for ordering diagnostics, and the prepared extract path derives layer candidate ownership from the descriptor list. It unions the selected camera descriptor layer set with Texture/Headless descriptor layer sets for mesh and sprite candidates, keeping custom-target visibility from losing layer-isolated geometry before `FrameVisibility` builds `VisibilityViewKey::CustomTarget` rows.

The scene producer builds `RenderViewExtract.cameras` from active scene cameras in deterministic scheduling order. The selected scene camera descriptor is rebuilt from the effective `view.camera` payload so request projection-mode and viewport-size overrides do not leave the descriptor list stale. `RenderViewExtract::selected_camera_descriptor()` is the scene/extract helper for consumers that need selected camera layers or target facts. Explicit `SceneViewportExtractRequest::camera` descriptors do not attach scene camera metadata and keep a single synthetic descriptor with `entity = None`, because their provenance is outside the scene world.

Inactive entities are filtered by `ActiveInHierarchy`. Because `RenderExtractPrepare` runs before the rows are collected, parent active-state propagation, parent reorders, and world transform propagation are current when the renderer sees the prepared extract. Read-only clone-based helpers can also produce a fresh packet or frame extract, but they do not clear dirty bits on the original world.

`PostProcessSettingsComponent` and `PostProcessVolumeComponent` are the runtime scene authoring hooks for post-process extraction. Both are fixed components stored on `World`, skipped by world serialization, and removed with their owning entity. Prepared extraction uses `PostProcessSettingsComponent` only from the selected scene camera entity to seed base bloom, color-grading, and effect-stack settings; explicit `SceneViewportExtractRequest::camera` descriptors keep default base settings because their provenance is outside the scene world. Camera `exposure_ev100` is not part of that component route: it seeds `PostProcessExtract.exposure` as manual `RenderExposureSettings` for the effective camera payload, so submit-time volume resolution can blend or replace it through `post.exposure`. Prepared extraction then reads active volume components whose entities are active in hierarchy and intersect the selected descriptor `RenderLayerSet`, converts the entity `RenderLayerMask` into the neutral volume mask, maps the profile into component overrides, and writes planned DTOs into `PostProcessExtract.volumes`. Global volumes use `VolumeShapeExtract::Global`; local box and sphere colliders carry transformed shape snapshots plus blend distance; capsule/no-collider local volumes are not projected in the current planned shape set. The submit path calls `PostProcessExtract::resolved_settings_for_camera(...)`, so scene extraction snapshots authoring data while `VolumeEvaluator` owns camera-position influence and parameter blending. This slice does not add asset/project schema serialization, editor authoring UI, trigger volumes, capsule/convex projection, or authoring/runtime persistence for volume components.

M3 now fills the non-snapshot frame sections with explicit defaults. `PostProcessExtract` carries preview/display mode plus selected scene-camera base bloom/manual-exposure/color-grading/effect-stack settings and any scene-authored planned post-process volume DTOs. `GeometryExtract` carries the request's virtual-geometry debug override and an empty VG sideband. `LightingExtract` carries an empty disabled Hybrid GI sideband. `VisibilityInput` is derived from the same sorted mesh rows so renderable, static, dynamic, and layer-mask inputs are aligned with geometry. The renderer submit path treats an empty VG sideband as no authored VG payload, preserving automatic provider extraction for advanced profiles while still making the scene-produced frame shape canonical. Render submit statistics also split extracted lights into ready/degraded slots: authored ambient entries, the first directional slot, and the first fixed scene-uniform point-light slots are visible as basic-renderer-ready, while extra directional lights, point lights beyond the fixed uniform cap, spot lights, and rect lights remain explicit degraded slots until their concrete clustered/Forward+/cone/area-light shader paths land.

## Validation Scope

Fresh workflow M03 validation is recorded separately from the older May M3 history. The current workflow uses `m03-canonical-render-extract/validation-evidence.md` and the focused `zircon_runtime/src/scene/tests/render_extract.rs` module to lock the canonical producer contract: `World` and `LevelSystem` populate `RenderFrameExtract` directly, `LevelSystem` keeps animation pose ownership, inactive cameras preserve deterministic default sideband shape while removing scene payload rows, camera-layer filtering applies uniformly to meshes, sprites, and visibility inputs, request camera layers override scene-camera layers, and production scene/submit paths do not route through snapshot adapters. This evidence remains focused `zircon_runtime` scene/graphics validation only and is not a root workspace, plugin workspace, export, or final green claim.

The 2026-06-15 Plan 07 PP-M2-S3 hard cut updates the scene post-process coverage to planned DTOs and evaluator resolution. Focused coverage includes `render_frame_extract_carries_scene_post_process_volumes_for_camera_layers`, `inactive_post_process_volume_hierarchy_is_excluded_from_frame_extract`, `scene_camera_post_process_settings_seed_frame_extract_before_volume_resolution`, `explicit_request_camera_ignores_scene_camera_post_process_settings`, `local_sphere_post_process_volume_uses_camera_distance_for_full_influence`, `local_sphere_post_process_volume_fades_in_blend_band`, `local_sphere_post_process_volume_outside_blend_band_has_zero_influence`, `local_box_post_process_volume_uses_camera_distance_for_blend`, `local_capsule_post_process_volume_is_not_projected_to_planned_extract`, and `local_post_process_volume_without_collider_is_excluded`. `render_post_process_extract` passed 9 filtered tests; the two scene extract filters passed individually; `render_volume_evaluator` passed 6 focused evaluator tests. The PP-M3-S1a exposure contract follow-up extends `render_extract_projects_scene_camera_component_product_fields` so camera `exposure_ev100` is also asserted on `PostProcessExtract.exposure.manual_ev100`.

Fresh workflow M03 named validation on 2026-06-02 used `D:\cargo-targets\zircon-m03-canonical-render-extract`. The focused `scene::tests::render_extract` module passed with `6 passed`, the structural no-snapshot-adapter guard passed with `1 passed`, exact carry-forward `ecs_schedule` guards each passed with `1 passed`, `world_basics` passed with `14 passed`, the M5 sideband smoke passed with `1 passed`, `graphics::tests::visibility` passed with `23 passed`, and aggregate `scene::tests` passed with `205 passed; 2272 filtered out`. `cargo check -p zircon_runtime --lib --locked --jobs 1` passed with existing warning-only output after rerunning through transient concurrent renderer/submit edit and plugin feature-shape visibility mismatches. The required root `cargo fmt --all --check` passed after `cargo fmt --all` formatted unrelated active-lane files; this root formatting gate is recorded as a validation fix/rerun, not as a workspace build/test claim.

The focused M1/M2 tests verify that:

- plugin `PostUpdate` hooks can mutate transforms before built-in `PostUpdate` systems propagate world transforms;
- `RenderExtract` built-ins run before `RenderExtract` hooks observe pending dirty state;
- stage completion flushes successful hook mutations before the next stage boundary;
- existing world basics still reflect transform changes in render extracts;
- asset-bound mesh, physics, animation, and graphics render-framework tests still consume the same frame boundary.
- dirty-only parent, active, transform, mobility, and render-layer mutations remain pending until `PostUpdate` or `RenderExtract` systems flush them;
- render extract preparation handles parent reorder plus inactive-parent propagation before collecting mesh rows.
- M02 derived-state regressions prove both clone-based legacy viewport packets and mutable prepared `RenderFrameExtract` flush pending parent/active/transform/node-cache work before reading render rows; clone-based helpers leave the source world's dirty flags pending, while the prepared mutable path clears the live world.
- M02 also locks active-camera selection and property-path product-field edits as node-cache/render-extract freshness inputs without changing scheduler or main-loop ordering.
- M3 canonical render-frame extraction populates direct frame sections, including camera aspect, visibility buckets, postprocess defaults, VG debug/default sidebands, and disabled Hybrid GI sidebands.
- M4A prepared render-frame extraction queues alpha-mask and transparent meshes from `MeshRenderer` alpha hints instead of treating production world meshes as all opaque.
- M5 light authoring projects scene-authored `AmbientLight` and `RectLight` into both legacy viewport packets and canonical `LightingExtract`; authored ambient now reaches the basic scene uniform, while rect light snapshots preserve explicit renderer-degraded diagnostics for the unimplemented area-light shading path. The same ambient/rect fields now round-trip through `SceneAsset` before extraction.
- the scene uniform regression `scene_uniform_uses_authored_ambient_light_when_lighting_is_enabled` verifies that active authored ambient entries are accumulated into `SceneUniform::ambient_color` instead of the previous fixed preview ambient fallback.
- the submit-stat regression `light_status_counts_split_ready_and_degraded_slots` verifies that light diagnostics distinguish currently rendered ambient, single-directional, and fixed point-light uniform slots from extra directional, over-cap point, spot, and rect-light slots that are extracted but still degraded in the renderer.
- the dynamic particle regression `render_frame_extract_collects_dynamic_particle_sprites_by_camera_layers` verifies that only active, camera-layer-visible emitters are collected, that parsed sprite fields and bounds survive into `ParticleExtract`, and that the emitter also appears in the visibility input used by the renderer.
- the dynamic particle GPU-frame regression `render_frame_extract_collects_dynamic_particle_gpu_frames_by_camera_layers` verifies that only active, camera-layer-visible neutral GPU frames are aggregated into `ParticleExtract.gpu_frame`, that indirect args word1 follows visible alive count, and that GPU bounds expand emitter visibility bounds without adding sprites.
- the world HUD bar regression `render_frame_extract_collects_world_hud_health_bars_as_scene_particles` verifies that health bar payloads authored through `render.world_hud_bars` are projected into camera-facing particle pips for frame extraction.
- the custom-target visibility regression `render_frame_extract_keeps_custom_target_layer_geometry_for_visibility_views` verifies that a Texture-target scene camera contributes its layer to mesh candidate extraction while the selected primary camera keeps its own camera layer set.
- a structural guard rejects reintroducing `RenderFrameExtract::from_snapshot(...)` inside `zircon_runtime/src/scene/render_extract/mod.rs`.

2026-06-18 particle GPU-frame scene auto-collection validation used `D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618`. `cargo check -q -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618` passed with the existing warning set. `cargo test -p zircon_runtime --lib render_frame_extract_collects_dynamic_particle_gpu_frames_by_camera_layers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618 --message-format short --color never -- --nocapture` passed 1 focused test after the lib-test target finished compiling.

Fresh focused M2 validation passed on 2026-05-08. The focused render-extract regression passed with `1 passed; 0 failed; 1061 filtered out`, the broader `scene::tests` filter passed with `45 passed; 0 failed; 1018 filtered out`, and the renderer-facing `graphics::tests` filter passed with `107 passed; 0 failed; 956 filtered out`.

Fresh M3 validation also passed on 2026-05-08 using `E:\cargo-targets\zircon-ecs-render-m3` to avoid a repo-local default `target` dep-info write race. The direct `RenderFrameExtract` population test passed with `1 passed; 0 failed; 1070 filtered out`, the structural snapshot-adapter guard passed with `1 passed; 0 failed; 1070 filtered out`, the scene-produced M5 flagship sideband test passed with `1 passed; 0 failed; 1070 filtered out`, the broader `scene::tests` filter passed with `47 passed; 0 failed; 1024 filtered out`, and the renderer-facing `graphics::tests` filter passed with `108 passed; 0 failed; 963 filtered out`.

Acceptance evidence is recorded in `tests/acceptance/ecs-to-render-chain.md`.
