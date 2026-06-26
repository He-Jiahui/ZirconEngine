---
related_code:
  - dev/bevy/crates/bevy_sprite/src/lib.rs
  - dev/bevy/crates/bevy_sprite/src/sprite.rs
  - dev/bevy/crates/bevy_sprite/src/texture_slice/mod.rs
  - dev/bevy/crates/bevy_sprite/src/texture_slice/slicer.rs
  - dev/bevy/crates/bevy_sprite_render/src/texture_slice/computed_slices.rs
  - dev/bevy/crates/bevy_sprite_render/src/lib.rs
  - dev/bevy/crates/bevy_sprite_render/src/render/mod.rs
  - dev/bevy/crates/bevy_sprite_render/src/sprite_mesh/sprite_material.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/image_mode.rs
  - zircon_runtime/src/core/framework/render/mod.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/image_mode.rs
  - zircon_runtime/src/core/framework/render/mod.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - user: 2026-06-02 PLEASE IMPLEMENT THIS PLAN - ZirconEngine WGPU 渲染主链闭环计划
  - user: 2026-05-21 continue M10 default 2D and presentation base acceptance checklist
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
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::sprite_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs::sprite_subpasses_apply_graph_attachment_ops_only_to_outer_draws
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs::sprite_batching_preserves_order_and_only_merges_adjacent_matching_textures
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs::sprite_batching_skips_empty_vertex_items
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs::sprite_queue_stats_count_stage_batches_sprites_and_vertices
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs::sprite_queue_stats_report_generated_image_slices_separately_from_sprites
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs::sprite_image_slices_fit_center_preserves_full_uv_and_letterboxes
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs::sprite_image_slices_fit_start_aligns_to_left_top
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs::sprite_image_slices_fill_center_crops_source_rect
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs::sprite_image_slices_fill_end_aligns_source_crop_to_right_or_bottom
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs::sprite_image_vertices_scale_fill_remains_single_quad
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs::build_sprite_vertices_filters_sprites_by_selected_camera_layers
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_sprite_build_vertices_tests.rs::runtime_15_sprite_build_vertices_tests_are_child_owner_split
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs::compiled_scene_outputs_carry_prepared_sprite_queue_stats
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Render Sprite Contracts

## Purpose

`zircon_runtime::core::framework::render::sprite` owns the neutral M6A product sprite contract. It is the non-particle 2D sprite surface required by `RenderProductFeature::Sprite`, and it stays separate from `RenderParticleSpriteSnapshot` and particle plugin billboard ownership.

Concrete rendering remains under `zircon_runtime::graphics`; runtime world authoring and extraction remain under `zircon_runtime::scene`. The framework contract is the shared handoff between those layers.

## Bevy Evidence

Bevy splits sprite authoring and sprite rendering across `bevy_sprite` and `bevy_sprite_render`. `dev/bevy/crates/bevy_sprite/src/lib.rs:68-108` defines `SpritePlugin`: it ensures `TextureAtlasPlugin`, calculates 2D bounds in `PostUpdate`, and optionally installs sprite picking. This is the API/runtime side, not the renderer.

`dev/bevy/crates/bevy_sprite/src/sprite.rs:19-41` defines the authored `Sprite` component fields: image handle, optional texture atlas, tint color, X/Y flipping, custom size, source rect, and image scaling mode. `sprite.rs:168-248` makes the image-mode vocabulary explicit: automatic sizing, `Scale(SpriteScalingMode)`, sliced, and tiled, including Fill/Fit center/start/end modes. `dev/bevy/crates/bevy_sprite/src/texture_slice/mod.rs:10-29` shows Bevy's texture-slice DTO and tiling entry point, including `stretch_value` and per-axis tiling.

`dev/bevy/crates/bevy_sprite_render/src/lib.rs:54-125` defines `SpriteRenderPlugin`: it loads sprite shaders, ensures texture-atlas support, installs mesh2d/sprite-mesh/tilemap render plugins, syncs sprites to the render world, extracts sprites in `ExtractSchedule`, queues sprites in `RenderSystems::Queue`, prepares image/view bind groups, and phase-sorts opaque and alpha-mask 2D bins.

`dev/bevy/crates/bevy_sprite_render/src/render/mod.rs:49-141` owns the concrete sprite pipeline and `SpritePipelineKey`; `render/mod.rs:141-275` specializes the pipeline with HDR, MSAA, tonemapping, dither, and compositing options. `render/mod.rs:345-573` extracts and queues visible sprites into the 2D render phases, while `render/mod.rs:480-633` manages image bind groups and sprite batches. `render/mod.rs:957-1035` applies `SpriteScalingMode` by either shrinking the quad for Fit modes or cropping UVs for Fill modes, and `dev/bevy/crates/bevy_sprite_render/src/sprite_mesh/sprite_material.rs:198-236` uses the same semantic split for sprite meshes. Zircon's current M6A renderer intentionally implements only a smaller default Core2d subset of that Bevy surface.

## Product Surface

`RenderSpriteSnapshot` carries the render-time sprite payload: entity id, world transform, image handle, optional material handle, atlas UV region, source rect, flip flags, anchor, optional custom size, image mode, color tint, z order, typed `RenderLayerSet`, and material alpha mode. The layer field is a CPU render DTO contract, not the old 32-bit mask; it preserves layer indices above 31 for selected-camera filtering and diagnostics.

`RenderSpriteAtlasRegion`, `RenderSpriteRect`, `RenderSpriteAnchor`, and `RenderSpriteBounds` are neutral DTOs. Atlas regions are normalized UV coordinates, rects describe source-space image sub-rects, anchors use normalized pivot coordinates, and bounds are available for future culling/debug use without coupling the contract to WGPU buffers.

`RenderSpriteImageMode` is the neutral image scaling contract. `Stretch` is the default and preserves the old single-quad behavior. `Scale(RenderSpriteScalingMode)` preserves source aspect ratio with Bevy-style `FillCenter`, `FillStart`, `FillEnd`, `FitCenter`, `FitStart`, and `FitEnd` modes. Fill modes keep the authored draw rectangle and crop the source rect; Fit modes keep the full source rect and shrink/align the drawn quad inside the authored rectangle. `Tiled { tile_x, tile_y, stretch_value }` repeats the source rect along selected axes when the authored draw size exceeds the tile extent. `Sliced(RenderSpriteSlicer)` carries a nine-slice border, independent center/side stretch-or-tile modes, and a maximum corner scale. The DTO mirrors the useful Bevy `SpriteImageMode`, `SpriteScalingMode`, and `TextureSlicer` boundary without importing Bevy asset systems, editor tools, or pipeline specialization.

`SpriteExtract` stores product sprites separately from `ParticleExtract`. `SpriteExtract::from_sprites(...)` derives a `RenderPhaseQueue` from the submitted sprites and the active `CorePipelineKind`, using each sprite's alpha mode, z order, and transform depth.

## Scene Extraction

`Sprite2dComponent` is the runtime scene component that projects into `RenderSpriteSnapshot`. It carries image/material handles plus atlas, rect, flip, anchor, custom size, image mode, tint, z order, and alpha policy. Its default image is `builtin://missing-texture`, and its default image mode is `Stretch`, so missing authored data still produces a debuggable sprite payload and renderer fallback evidence.

`Mesh2dComponent` exists as the parallel 2D mesh authoring shape, but M6A does not treat it as a sprite. This keeps `RenderProductFeature::Sprite` acceptance tied to real sprite payloads instead of considering all 2D renderable components or particle billboard data as equivalent.

`World::to_render_frame_extract(...)` and request-driven world extraction collect active sprites, filter them through the active camera render layers, sort by `(z_order, entity)`, and store them in `RenderFrameExtract.sprites`. Scene authoring still stores entity render layers as the legacy `u32` mask, but `World::render_sprite_snapshot_for_camera(...)` wraps that value with `RenderLayerSet::from_legacy_mask(...)` at the render DTO boundary. Sprite entities are also added to `VisibilityInput` as dynamic renderables; that visibility handoff remains a legacy `u32` boundary and therefore uses `to_legacy_mask_lossy()` explicitly instead of making the sprite DTO lossy.

Inactive cameras produce an empty `SpriteExtract`. Particle billboard snapshots remain under `RenderFrameExtract.particles` and are never copied into `SpriteExtract`.

## Core2d Phase Queue

`build_sprite_phase_queue(...)` classifies sprites into `Opaque2d`, `AlphaMask2d`, or `Transparent2d` for `CorePipelineKind::Core2d` using `RenderMaterialAlphaMode`. The same helper can classify into the 3D phase family if a future product path deliberately submits sprites through `Core3d`, but M6A acceptance is the default Core2d route.

`RenderPhaseSortKey::for_sprite(...)` orders sprites first by z order, then by phase-specific depth ordering, then by entity tie-breaker. Transparent sprites use reversed depth ordering inside their z bucket, matching the product requirement that transparent 2D sprites can sort back-to-front without losing authored z-order layering.

## Graphics Integration

`RenderPipelineAsset::default_core2d()` now declares the Core2d stage order `Opaque2d -> AlphaMask2d -> Transparent2d -> PostProcess -> Ui -> Overlay -> Debug` and enables `BuiltinRenderFeature::Sprite` alongside the default PostProcess/UI/DebugOverlay tail. PostProcess remains in the default Core2d chain so final composition and UI ordering share the same graph contract; advanced Virtual Geometry, Hybrid GI, and Solari remain absent from default 2D rendering.

The built-in sprite feature descriptor contributes graph passes with executor ids `sprite.opaque`, `sprite.alpha-mask`, and `sprite.transparent`. The descriptor now declares both `scene-color` and `scene-depth`: the opaque sprite pass is the Core2d depth producer, while alpha-mask and transparent sprite passes read the current depth and write the depth attachment back through graph ownership. The executor registry validates those ids, maps them back to `Opaque2d`, `AlphaMask2d`, and `Transparent2d`, and requires renderer GPU context plus the neutral `scene-color` / `scene-depth` graph resources. Missing renderer context is a hard executor error rather than a silent no-op, so product sprite passes are visible in graph execution evidence and remain tied to the SceneRenderer graph path.

The concrete sprite renderer builds texture-tinted quads from `ViewportRenderFrame::sprites()`, consumes `SpriteExtract.phase_queue` when available, and falls back to classifying the sprite vector only when an older caller supplies sprites without a phase queue. `build_sprite_vertices(...)` expands `RenderSpriteImageMode` on the CPU: `Stretch` emits one quad, `Scale` emits one aspect-preserving quad or source crop, `Tiled` emits bounded repeated quads, and `Sliced` emits nine-slice quads with optional center/side tiling. Before expansion, it reads `RenderViewExtract::selected_camera_layers()` through the viewport frame and calls `intersects(&sprite.render_layer_mask)`, so synthetic/direct extracts and scene-backed extracts both keep selected-camera layer filtering on the typed `RenderLayerSet` path. `build_sprite_vertices/tests.rs::build_sprite_vertices_filters_sprites_by_selected_camera_layers` uses a layer-40 sprite and layer-40 camera descriptor to guard this non-lossy behavior. The renderer now receives separate color and depth `RenderGraphAttachmentOps` from the graph executor and maps them through the shared WGPU attachment helpers. When one graph sprite pass emits multiple WGPU subpasses, only the first draw uses the graph load operation and only the final draw uses the graph store operation; intermediate draws always load and store so they cannot clear or discard earlier sprite output or depth state.

The renderer now prepares ordered sprite draw batches before recording WGPU passes. `prepare_sprite_draw_batches(...)` keeps the phase queue order intact and only merges adjacent sprites that already share the same texture id, concatenating their quad vertices into one vertex buffer and one render pass. Non-adjacent matching textures remain separate batches so authored z/order/layer sorting is not changed to chase batching. This is a renderer-side M6 batching contract and not yet Bevy-style binned sprite batching or per-view pipeline specialization.

`prepare_sprite_queue_stats(...)` uses the same ordered batch preparation contract for the active sprite graph stages and returns `PreparedSpriteQueueStats`. `SceneRendererCompiledSceneOutputs` carries that summary back to `SceneRenderer`, which resets it before each render attempt and exposes only the last successful frame through `RenderFramework` stats. The stats are intentionally submit-level diagnostics, not a second renderer path. Since `render_plan14_sprite_queue_stats_suppression_cleanup_static_passed_cargo_deferred_active_lanes`, this DTO no longer carries a non-test dead-code suppression because every field is part of the production `RenderStats.last_sprite_*` projection.

SceneRenderer keeps sprite execution at the old visual order point but routes it through `execute_graph_stage(...)`: `Opaque2d` runs after the main opaque scene path, `AlphaMask2d` after the 3D alpha-mask stage, and `Transparent2d` after the 3D transparent stage. This removes the direct `SpriteRenderer::record(...)` bypass from compiled-scene submission while preserving Core2d stage order.

The renderer uses the existing texture streamer fallback path through `ResourceStreamer::texture(Some(sprite.image.id()))` so missing sprite images still draw with the renderer fallback texture.

`ResourceStreamer::ensure_scene_resources(...)` counts sprite texture readiness separately from material readiness. `RenderStats` exposes `last_sprite_count`, `last_sprite_ready_count`, `last_sprite_texture_fallback_count`, `last_sprite_graph_executed_pass_count`, `last_sprite_draw_batch_count`, `last_sprite_batched_sprite_count`, `last_sprite_image_slice_count`, `last_sprite_expanded_image_slice_count`, `last_sprite_vertex_count`, and per-phase sprite batch counts, allowing tests and tools to prove sprite rendering did not go through particle graph passes and to inspect the renderer-side batch profile. The image-slice counters make tiled and nine-slice expansion visible without requiring tooling to infer it from vertex counts.

## Bevy Gap Classification

| Bevy sprite area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Sprite authoring fields | Covered for the core M6A payload: image, optional material, atlas UV, rect, flip flags, anchor, custom size, image mode, tint, z order, alpha mode, and render layers. | Add higher-level authored presets only when editor/importer workflows need them; keep the renderer-facing DTO stable. |
| Texture atlas and rects | Render DTOs support atlas regions and source rects; missing textures degrade through renderer fallback stats. | Add atlas asset import/layout projection and editor-facing atlas tooling before claiming Bevy-level atlas workflow parity. |
| Scale/sliced/tiled sprites | Basic CPU vertex expansion is implemented through `RenderSpriteImageMode::Scale`, `RenderSpriteImageMode::Tiled`, and `RenderSpriteImageMode::Sliced`, reusing the existing WGPU sprite shader and adjacent texture batching. | Add asset-importer metadata, editor scale/nine-slice handles, generated slice diagnostics, and more efficient slice batching before claiming Bevy-level workflow parity. |
| Mesh2d and SpriteMesh | `Mesh2dComponent` exists as scene data but does not count as a product sprite; `SpriteMesh` has no equivalent product path. | Add materialized Mesh2d/SpriteMesh render products and keep them separate from non-particle sprite acceptance. |
| Render phase and queueing | Default Core2d graph passes, `SpriteExtract.phase_queue`, and order-preserving adjacent texture batches are present. | Add Bevy-like binned batching, per-view pipeline specialization, and phase-specific depth/alpha behavior. |
| Pipeline specialization | Current concrete path uses a minimal texture-tinted quad pipeline. | Add HDR/MSAA/tonemapping/dither/compositing keys and separate alpha-mask discard behavior before claiming Bevy pipeline parity. |
| Picking and Text2d | Out of this render contract; not counted as sprite renderer parity. | Route through UI/picking/text milestones so sprite rendering does not absorb unrelated interaction or text layout ownership. |

## Current Limits

M6A intentionally keeps the concrete sprite GPU path minimal. Opaque, alpha-mask, and transparent phase passes share one alpha-blended WGPU pipeline today; per-phase depth-write, alpha-mask cutoff discard, material-specific sprite pipelines, texture-atlas asset import, and GPU culling remain later product work. The M5 render-main-chain cutover only changes ownership and ordering: sprite draw commands now originate from graph executor dispatch. M6 adds adjacent same-texture batching and public queue-preparation stats, and the 2026-06-07 image-mode follow-ups expand stretch/scale/tiled/sliced sprites into CPU quads before that batching step. The image-slice diagnostics follow-up records both total generated image slices and expanded slices beyond the sprite count. `collect_runtime_diagnostics(...)` mirrors sprite readiness plus queue counters into `DiagnosticStore` under `render.sprite.*` and `render.sprite.queue.*`. It still does not reorder sprites into Bevy-like bins, add renderer pipeline specialization, or expose editor/importer image-mode metadata.

The 2026-06-23 Plan 09 CO-M4 status anchor `render_plan09_sprite_render_layer_set_snapshot_static_passed_cargo_lock_blocked` records the typed layer snapshot cutover. Current evidence is scoped formatting, typed-constructor scans, direct sprite-bitwise scans, line counts, and diff checks; the focused locked Cargo run was blocked before compilation by the current `Cargo.lock` drift, so this document does not claim a Cargo, WGPU product, or RenderDoc pass for the slice.

The 2026-06-24 Sprite build vertices test owner split keeps `graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs` as the 473-line production owner for phase extraction, selected-camera layer filtering, image-mode CPU slicing, and nine-slice/tile helpers, while `graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs` owns the former inline tests and fixtures. Guard `runtime_15_sprite_build_vertices_tests_are_child_owner_split` and status anchor `render_plan14_sprite_build_vertices_test_owner_split_static_passed_cargo_deferred_active_compile_lane` lock this parent/test-child boundary; the slice has scoped static evidence only while Cargo/WGPU/RenderDoc remain deferred behind active compile lanes.

The accepted M6A scope also does not add `.zmaterial`, shader/material importer schema, material editor UI, anti-aliasing, UI pass placement, advanced VG/HGI integration, or Solari.

## M10R Default 2D Promotion Gate

M10R uses this document as the default 2D side of the M10.4/M10.7 gate. The Bevy baseline is broader than Zircon's current M6A implementation: `2d_bevy_render` includes the render backend, core pipeline, post-process, sprite renderer, and gizmos render collection, while `SpriteRenderPlugin` also installs Mesh2d, ColorMaterial, SpriteMesh, tilemap, extract, queue, bind-group preparation, and phase-sort systems.

For Zircon, the promotable scope is narrower and explicit. `Render2d` may advance to the default Core2d sprite base when product sprite extraction, Core2d phase queues, default Core2d pipeline compilation, sprite graph pass execution, texture fallback stats, image-mode vertex expansion, and non-particle separation are all covered by focused tests. Mesh2d/SpriteMesh drawing, atlas asset import/layout projection, Bevy-style binned batching, per-view pipeline specialization, Text2d, and picking stay open requirements rather than hidden inside the sprite pass.

This gate is paired with the presentation target gate in [Render Product Submit](../../graphics/render-product-submit.md). A screenshot or headless capture is only valid as combined smoke evidence after the sprite product path and camera target error model are both clear; it cannot replace the default 2D renderer evidence above.

## Test Coverage

`zircon_runtime/src/graphics/tests/render_product_sprite.rs` proves the product sprite contract is distinct from particle billboard sprites, verifies Core2d sprite phase ordering, and submits a Core2d sprite frame that records sprite stats while leaving particle graph execution at zero.

`zircon_runtime/src/scene/tests/world_basics.rs` proves world extraction preserves sprite image, material, atlas, rect, flip, anchor, custom size, image mode, tint, z order, Core2d selection, phase queue identity, and camera-layer filtering. It also proves `Mesh2dComponent` does not count as a product sprite or particle sprite.

`zircon_runtime/src/graphics/tests/pipeline_compile.rs` proves the default Core2d pipeline compiles with sprite graph passes and the expected required extract sections.

`zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs` proves `Stretch` remains one quad, `Scale` preserves aspect through Fit quad alignment and Fill source cropping, `Tiled` creates repeated quads, `Sliced` creates nine regions, and excessive tiling is capped. `zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs` proves adjacent texture batching preserves order, skips empty vertex payloads, and counts per-stage batches, sprites, generated image slices, expanded image slices, and vertices. `compiled_scene_outputs_carry_prepared_sprite_queue_stats` proves the compiled-scene handoff carries sprite queue stats into the renderer output path. `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` proves those sprite readiness and queue counters are also surfaced through runtime `DiagnosticStore` paths.

2026-06-02 render-main-chain validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`. `cargo test -p zircon_runtime --lib --locked graph_execution --jobs 1 --message-format short --color never` passed 23 graph-execution tests, including `sprite_executor_requires_renderer_context_instead_of_nooping`. `cargo test -p zircon_runtime --lib --locked render_product_sprite --jobs 1 --message-format short --color never` passed 7 focused sprite/product tests, including the Core2d submit path. `cargo test -p zircon_runtime --lib --locked sprite_subpasses_apply_graph_attachment_ops_only_to_outer_draws --jobs 1 --message-format short --color never` passed the sprite attachment-op subpass regression. All three commands emitted only pre-existing UI/accessibility/text warnings outside this sprite lane.
