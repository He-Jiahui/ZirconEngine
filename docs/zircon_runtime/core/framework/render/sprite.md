---
related_code:
  - dev/bevy/crates/bevy_sprite/src/lib.rs
  - dev/bevy/crates/bevy_sprite/src/sprite.rs
  - dev/bevy/crates/bevy_sprite/src/texture_slice/mod.rs
  - dev/bevy/crates/bevy_sprite_render/src/lib.rs
  - dev/bevy/crates/bevy_sprite_render/src/render/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/core/framework/render/sprite/atlas.rs
  - zircon_runtime/src/core/framework/render/sprite/rect.rs
  - zircon_runtime/src/core/framework/render/sprite/anchor.rs
  - zircon_runtime/src/core/framework/render/sprite/bounds.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_item.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/scene/components/render2d/mod.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/core/framework/render/sprite/atlas.rs
  - zircon_runtime/src/core/framework/render/sprite/rect.rs
  - zircon_runtime/src/core/framework/render/sprite/anchor.rs
  - zircon_runtime/src/core/framework/render/sprite/bounds.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/scene/components/render2d/mod.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
plan_sources:
  - user: 2026-05-17 continue M6A sprite/default 2D renderer productization
  - user: 2026-05-21 continue Bevy-level render sprite evidence mapping
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
tests:
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - tests/acceptance/render-product-m6a-sprite-default-2d.md
  - cargo test -p zircon_runtime --locked render_product_sprite
  - cargo test -p zircon_runtime --locked render_product_pipeline
  - cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Render Sprite Contracts

## Purpose

`zircon_runtime::core::framework::render::sprite` owns the neutral M6A product sprite contract. It is the non-particle 2D sprite surface required by `RenderProductFeature::Sprite`, and it stays separate from `RenderParticleSpriteSnapshot` and particle plugin billboard ownership.

Concrete rendering remains under `zircon_runtime::graphics`; runtime world authoring and extraction remain under `zircon_runtime::scene`. The framework contract is the shared handoff between those layers.

## Bevy Evidence

Bevy splits sprite authoring and sprite rendering across `bevy_sprite` and `bevy_sprite_render`. `dev/bevy/crates/bevy_sprite/src/lib.rs:68-108` defines `SpritePlugin`: it ensures `TextureAtlasPlugin`, calculates 2D bounds in `PostUpdate`, and optionally installs sprite picking. This is the API/runtime side, not the renderer.

`dev/bevy/crates/bevy_sprite/src/sprite.rs:19-41` defines the authored `Sprite` component fields: image handle, optional texture atlas, tint color, X/Y flipping, custom size, source rect, and image scaling mode. `sprite.rs:168-195` makes the image-mode vocabulary explicit: automatic sizing, scaled, sliced, and tiled. `dev/bevy/crates/bevy_sprite/src/texture_slice/mod.rs:10-29` shows Bevy's texture-slice DTO and tiling entry point, including `stretch_value` and per-axis tiling.

`dev/bevy/crates/bevy_sprite_render/src/lib.rs:54-125` defines `SpriteRenderPlugin`: it loads sprite shaders, ensures texture-atlas support, installs mesh2d/sprite-mesh/tilemap render plugins, syncs sprites to the render world, extracts sprites in `ExtractSchedule`, queues sprites in `RenderSystems::Queue`, prepares image/view bind groups, and phase-sorts opaque and alpha-mask 2D bins.

`dev/bevy/crates/bevy_sprite_render/src/render/mod.rs:49-141` owns the concrete sprite pipeline and `SpritePipelineKey`; `render/mod.rs:141-275` specializes the pipeline with HDR, MSAA, tonemapping, dither, and compositing options. `render/mod.rs:345-573` extracts and queues visible sprites into the 2D render phases, while `render/mod.rs:480-633` manages image bind groups and sprite batches. Zircon's current M6A renderer intentionally implements only a smaller default Core2d subset of that Bevy surface.

## Product Surface

`RenderSpriteSnapshot` carries the render-time sprite payload: entity id, world transform, image handle, optional material handle, atlas UV region, source rect, flip flags, anchor, optional custom size, color tint, z order, render layer mask, and material alpha mode.

`RenderSpriteAtlasRegion`, `RenderSpriteRect`, `RenderSpriteAnchor`, and `RenderSpriteBounds` are neutral DTOs. Atlas regions are normalized UV coordinates, rects describe source-space image sub-rects, anchors use normalized pivot coordinates, and bounds are available for future culling/debug use without coupling the contract to WGPU buffers.

`SpriteExtract` stores product sprites separately from `ParticleExtract`. `SpriteExtract::from_sprites(...)` derives a `RenderPhaseQueue` from the submitted sprites and the active `CorePipelineKind`, using each sprite's alpha mode, z order, and transform depth.

## Scene Extraction

`Sprite2dComponent` is the runtime scene component that projects into `RenderSpriteSnapshot`. It carries image/material handles plus atlas, rect, flip, anchor, custom size, tint, z order, and alpha policy. Its default image is `builtin://missing-texture` so missing authored data still produces a debuggable sprite payload and renderer fallback evidence.

`Mesh2dComponent` exists as the parallel 2D mesh authoring shape, but M6A does not treat it as a sprite. This keeps `RenderProductFeature::Sprite` acceptance tied to real sprite payloads instead of considering all 2D renderable components or particle billboard data as equivalent.

`World::to_render_frame_extract(...)` and request-driven world extraction collect active sprites, filter them through the active camera render layers, sort by `(z_order, entity)`, and store them in `RenderFrameExtract.sprites`. Sprite entities are also added to `VisibilityInput` as dynamic renderables so visibility/debug consumers see the same scene renderable set as the renderer.

Inactive cameras produce an empty `SpriteExtract`. Particle billboard snapshots remain under `RenderFrameExtract.particles` and are never copied into `SpriteExtract`.

## Core2d Phase Queue

`build_sprite_phase_queue(...)` classifies sprites into `Opaque2d`, `AlphaMask2d`, or `Transparent2d` for `CorePipelineKind::Core2d` using `RenderMaterialAlphaMode`. The same helper can classify into the 3D phase family if a future product path deliberately submits sprites through `Core3d`, but M6A acceptance is the default Core2d route.

`RenderPhaseSortKey::for_sprite(...)` orders sprites first by z order, then by phase-specific depth ordering, then by entity tie-breaker. Transparent sprites use reversed depth ordering inside their z bucket, matching the product requirement that transparent 2D sprites can sort back-to-front without losing authored z-order layering.

## Graphics Integration

`RenderPipelineAsset::default_core2d()` now declares the Core2d stage order `Opaque2d -> AlphaMask2d -> Transparent2d -> PostProcess -> Ui -> Overlay -> Debug` and enables `BuiltinRenderFeature::Sprite`. `PostProcess` stays disabled by default in this Core2d asset, and advanced Virtual Geometry, Hybrid GI, and Solari remain absent from default 2D rendering.

The built-in sprite feature descriptor contributes graph passes with executor ids `sprite.opaque`, `sprite.alpha-mask`, and `sprite.transparent`. The executor registry validates those ids and requires the neutral `scene-color` resource to exist, so product sprite passes are visible in graph execution evidence instead of being purely renderer-local draws.

The concrete sprite renderer builds texture-tinted quads from `ViewportRenderFrame::sprites()`, consumes `SpriteExtract.phase_queue` when available, and falls back to classifying the sprite vector only when an older caller supplies sprites without a phase queue. It uses the existing texture streamer fallback path through `ResourceStreamer::texture(Some(sprite.image.id()))` so missing sprite images still draw with the renderer fallback texture.

`ResourceStreamer::ensure_scene_resources(...)` counts sprite texture readiness separately from material readiness. `RenderStats` exposes `last_sprite_count`, `last_sprite_ready_count`, `last_sprite_texture_fallback_count`, and `last_sprite_graph_executed_pass_count`, allowing tests and tools to prove sprite rendering did not go through particle graph passes.

## Bevy Gap Classification

| Bevy sprite area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Sprite authoring fields | Covered for the core M6A payload: image, optional material, atlas UV, rect, flip flags, anchor, custom size, tint, z order, alpha mode, and render layers. | Add Bevy-style image scaling mode as first-class contract instead of treating sliced/tiled/scaled modes as future renderer work. |
| Texture atlas and rects | Render DTOs support atlas regions and source rects; missing textures degrade through renderer fallback stats. | Add atlas asset import/layout projection and editor-facing atlas tooling before claiming Bevy-level atlas workflow parity. |
| Sliced/tiled sprites | Not implemented beyond neutral rect/atlas vocabulary. | Add texture-slice descriptors, stretch/tile policy, generated slice stats, and renderer-side slice batching. |
| Mesh2d and SpriteMesh | `Mesh2dComponent` exists as scene data but does not count as a product sprite; `SpriteMesh` has no equivalent product path. | Add materialized Mesh2d/SpriteMesh render products and keep them separate from non-particle sprite acceptance. |
| Render phase and queueing | Default Core2d graph passes and `SpriteExtract.phase_queue` are present. | Add Bevy-like binned batching, per-view pipeline specialization, and phase-specific depth/alpha behavior. |
| Pipeline specialization | Current concrete path uses a minimal texture-tinted quad pipeline. | Add HDR/MSAA/tonemapping/dither/compositing keys and separate alpha-mask discard behavior before claiming Bevy pipeline parity. |
| Picking and Text2d | Out of this render contract; not counted as sprite renderer parity. | Route through UI/picking/text milestones so sprite rendering does not absorb unrelated interaction or text layout ownership. |

## Current Limits

M6A intentionally keeps the concrete sprite GPU path minimal. Opaque, alpha-mask, and transparent phase passes share one alpha-blended WGPU pipeline today; per-phase depth-write, alpha-mask cutoff discard, batching, material-specific sprite pipelines, texture-atlas asset import, and GPU culling remain later product work.

The accepted M6A scope also does not add `.zmaterial`, shader/material importer schema, material editor UI, anti-aliasing, UI pass placement, advanced VG/HGI integration, or Solari.

## Test Coverage

`zircon_runtime/src/graphics/tests/render_product_sprite.rs` proves the product sprite contract is distinct from particle billboard sprites, verifies Core2d sprite phase ordering, and submits a Core2d sprite frame that records sprite stats while leaving particle graph execution at zero.

`zircon_runtime/src/scene/tests/world_basics.rs` proves world extraction preserves sprite image, material, atlas, rect, flip, anchor, custom size, tint, z order, Core2d selection, phase queue identity, and camera-layer filtering. It also proves `Mesh2dComponent` does not count as a product sprite or particle sprite.

`zircon_runtime/src/graphics/tests/pipeline_compile.rs` proves the default Core2d pipeline compiles with sprite graph passes and the expected required extract sections.
