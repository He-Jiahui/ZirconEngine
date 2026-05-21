---
related_code:
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/render_extract/mod.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/ecs/internal_scene_system.rs
  - zircon_runtime/src/scene/ecs/system_stage.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
implementation_files:
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/render_extract/mod.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
plan_sources:
  - user: 2026-05-08 ECS to render chain milestone execution
  - .codex/plans/ZirconEngine ECS 到渲染链路完善里程碑计划.md
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
tests:
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/physics_animation_components.rs
  - tests/acceptance/ecs-to-render-chain.md
  - zircon_runtime/src/core/framework/render/light/readiness.rs::light_status_counts_split_ready_and_degraded_slots
  - .github/workflows/ci.yml
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
3. `World::build_prepared_render_frame_extract_for_request(...)` runs the `RenderExtract` built-in systems before reading camera, mesh, light, active, transform, and layer data.
4. The world assembles `RenderViewExtract`, `GeometryExtract`, `LightingExtract`, `PostProcessExtract`, `DebugOverlayExtract`, `ParticleExtract`, and `VisibilityInput` directly.
5. `LevelSystem` appends animation pose sidebands for mesh entities with skeletons.

`SceneViewportRenderPacket` remains available through `to_render_snapshot()` / `to_render_extract()` for preview and roundtrip callers, and `RenderFrameExtract::from_snapshot(...)` remains a framework adapter for tests or legacy snapshot owners. The scene producer no longer uses that adapter for frame extraction.

## Snapshot Contents

`World::build_prepared_render_frame_extract_for_request(...)` emits sorted meshes, directional lights, point lights, rect lights, spot lights, and active ambient light records. Mesh rows include stable node id, world transform, model handle, material handle, tint, mobility, and render-layer mask. Light row types live under `render::light`; `LightingExtract` only aggregates those rows with reflection-probe, baked-lighting, and Hybrid GI sidebands. Rect light rows follow Bevy's orientation contract by deriving the emitted direction from the entity transform's forward vector, while keeping the authored color, intensity, range, and size in `RenderRectLightSnapshot`. Ambient light snapshots are no longer marked renderer-degraded because the basic forward/deferred scene uniform now folds active authored ambient color times intensity into `SceneUniform::ambient_color`; rect lights remain renderer-degraded until a concrete area-light shader path lands. The prepared frame path also builds `GeometryPhaseInput` from the same sorted mesh rows and each `MeshRenderer.material_alpha_mode`, so mesh indices and phase classification stay aligned for opaque, alpha-mask, and transparent queues. Camera rows preserve explicit viewport-request overrides and derive aspect ratio from the request size when present.

Inactive entities are filtered by `ActiveInHierarchy`. Because `RenderExtractPrepare` runs before the rows are collected, parent active-state propagation, parent reorders, and world transform propagation are current when the renderer sees the prepared extract. Read-only clone-based helpers can also produce a fresh packet or frame extract, but they do not clear dirty bits on the original world.

M3 now fills the non-snapshot frame sections with explicit defaults. `PostProcessExtract` carries preview/display mode plus default bloom and color grading. `GeometryExtract` carries the request's virtual-geometry debug override and an empty VG sideband. `LightingExtract` carries an empty disabled Hybrid GI sideband. `VisibilityInput` is derived from the same sorted mesh rows so renderable, static, dynamic, and layer-mask inputs are aligned with geometry. The renderer submit path treats an empty VG sideband as no authored VG payload, preserving automatic provider extraction for advanced profiles while still making the scene-produced frame shape canonical. Render submit statistics also split extracted lights into ready/degraded slots: authored ambient entries and the first directional slot are visible as basic-renderer-ready, while extra directional lights plus point, spot, and rect lights remain explicit degraded slots until their concrete PBR/clustered/area-light shader paths land.

## Validation Scope

The focused M1/M2 tests verify that:

- plugin `PostUpdate` hooks can mutate transforms before built-in `PostUpdate` systems propagate world transforms;
- `RenderExtract` built-ins run before `RenderExtract` hooks observe pending dirty state;
- stage completion flushes successful hook mutations before the next stage boundary;
- existing world basics still reflect transform changes in render extracts;
- asset-bound mesh, physics, animation, and graphics render-framework tests still consume the same frame boundary.
- dirty-only parent, active, transform, mobility, and render-layer mutations remain pending until `PostUpdate` or `RenderExtract` systems flush them;
- render extract preparation handles parent reorder plus inactive-parent propagation before collecting mesh rows.
- M3 canonical render-frame extraction populates direct frame sections, including camera aspect, visibility buckets, postprocess defaults, VG debug/default sidebands, and disabled Hybrid GI sidebands.
- M4A prepared render-frame extraction queues alpha-mask and transparent meshes from `MeshRenderer` alpha hints instead of treating production world meshes as all opaque.
- M5 light authoring projects scene-authored `AmbientLight` and `RectLight` into both legacy viewport packets and canonical `LightingExtract`; authored ambient now reaches the basic scene uniform, while rect light snapshots preserve explicit renderer-degraded diagnostics for the unimplemented area-light shading path. The same ambient/rect fields now round-trip through `SceneAsset` before extraction.
- the scene uniform regression `scene_uniform_uses_authored_ambient_light_when_lighting_is_enabled` verifies that active authored ambient entries are accumulated into `SceneUniform::ambient_color` instead of the previous fixed preview ambient fallback.
- the submit-stat regression `light_status_counts_split_ready_and_degraded_slots` verifies that light diagnostics distinguish currently rendered ambient and single-directional slots from extra directional, point, spot, and rect-light slots that are extracted but still degraded in the renderer.
- a structural guard rejects reintroducing `RenderFrameExtract::from_snapshot(...)` inside `zircon_runtime/src/scene/render_extract/mod.rs`.

Fresh focused M2 validation passed on 2026-05-08. The focused render-extract regression passed with `1 passed; 0 failed; 1061 filtered out`, the broader `scene::tests` filter passed with `45 passed; 0 failed; 1018 filtered out`, and the renderer-facing `graphics::tests` filter passed with `107 passed; 0 failed; 956 filtered out`.

Fresh M3 validation also passed on 2026-05-08 using `E:\cargo-targets\zircon-ecs-render-m3` to avoid a repo-local default `target` dep-info write race. The direct `RenderFrameExtract` population test passed with `1 passed; 0 failed; 1070 filtered out`, the structural snapshot-adapter guard passed with `1 passed; 0 failed; 1070 filtered out`, the scene-produced M5 flagship sideband test passed with `1 passed; 0 failed; 1070 filtered out`, the broader `scene::tests` filter passed with `47 passed; 0 failed; 1024 filtered out`, and the renderer-facing `graphics::tests` filter passed with `108 passed; 0 failed; 963 filtered out`.

Acceptance evidence is recorded in `tests/acceptance/ecs-to-render-chain.md`.
