---
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_vertex/particle_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_velocity_vertex/particle_velocity_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle_velocity.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_velocity_vertex/particle_velocity_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle_velocity.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/support.rs
plan_sources:
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - user: 2026-06-14 implement WGPU render pipeline architecture code and update plan progress
tests:
  - rustfmt --edition 2021 --check on TP-M1-S8 particle velocity diagnostic files
  - source scan for last_particle_velocity_missing_sprite_count, particle_velocity_missing_sprite_count, and particle_sprite_count
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never
  - cargo test -p zircon_runtime --lib particle_velocity_gap_counts_sprites_only_when_reconstructed_velocity_is_requested --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never -- --test-threads=1 --nocapture (timed out after 304s without test output; leftover cargo/rustc processes terminated)
  - zircon_runtime/src/core/framework/render/frame_extract.rs::tests::particle_extract_counts_previous_state_by_entity
  - zircon_runtime/src/core/framework/render/frame_extract.rs::tests::particle_extract_consumes_duplicate_entity_previous_state_once_per_row
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::particle_velocity_gap_excludes_sprites_with_previous_state
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_particle_previous_state_suppresses_velocity_gap_stats
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - cargo test -p zircon_runtime particle_velocity_vertices --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - rustfmt --edition 2021 --check on TP-M1-S11 particle stable identity files
  - constructor scan for RenderParticleSpriteSnapshot/RenderParticlePreviousSpriteSnapshot stable_sprite_key coverage
  - cargo test -p zircon_runtime stable_sprite_key --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (blocked before filtered tests by unrelated ui/surface/render/command_palette.rs:311 E0282)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (blocked by unrelated ui/surface/render/command_palette.rs:311 E0282)
  - rustfmt --edition 2021 --check on TP-M1-S12 renderer-owned particle previous-state files
  - cargo test -p zircon_runtime successful_submit_records_particle_previous_state_for_next_frame --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (blocked before filtered tests by unrelated ui/surface/render/command_palette.rs:311 E0282)
  - rustfmt --edition 2021 --check on TP-M1-S13 particle previous billboard-basis files
  - constructor scan for RenderParticlePreviousSpriteSnapshot billboard_basis coverage
  - cargo test -p zircon_runtime previous_billboard_basis --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - D:\cargo-targets\zircon-runtime-temporal-s4d-0614\debug\deps\zircon_runtime-5d2828c2001649f6.exe graphics::runtime::render_framework::submit_frame_extract::submit::update_particle_previous_state::tests::successful_submit_records_particle_previous_state_for_next_frame --exact --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - rustfmt --edition 2021 --check on TP-M1-S14 scene-velocity readback files
  - cargo test -p zircon_runtime execution_record_preserves_scene_velocity_readback_report --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - D:\cargo-targets\zircon-runtime-temporal-s4d-0614\debug\deps\zircon_runtime-5d2828c2001649f6.exe graphics::tests::render_product_particle_velocity::render_product_particle_velocity_writer_writes_nonzero_scene_velocity_pixels --exact --nocapture
  - D:\cargo-targets\zircon-runtime-temporal-s4d-0614\debug\deps\zircon_runtime-5d2828c2001649f6.exe graphics::tests::render_product_particle_velocity --nocapture
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S15 renderer-owned second-frame readback baseline)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S16 same-entity keyed multi-sprite readback baseline)
  - cargo test -p zircon_runtime anonymous_stream --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S17 key=0 anonymous stream diagnostics)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S17 anonymous key product diagnostic)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S18 keyed multi-sprite three-frame dynamic baseline)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs (TP-M1-S20 keyed stress field clean rerun)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S20 production check)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S20 stress product route; passed 8 filtered product tests after ui/surface/input/editable_text/ime_context.rs covered UiInputEvent::ToastTimer)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/frame_extract.rs zircon_runtime/src/core/framework/render/frame_extract/particle_extract_policy.rs zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs zircon_runtime/src/scene/world/render_particles.rs zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs (TP-M1/S22 key=0 hard enforcement)
  - cargo test -p zircon_runtime particle_extract --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 ParticleExtract policy; passed 4 filtered tests)
  - cargo test -p zircon_runtime particle_velocity_vertices --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 velocity builder policy; passed 6 filtered tests)
  - cargo test -p zircon_runtime successful_submit --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 previous cache policy; passed 3 filtered tests)
  - cargo test -p zircon_runtime scene::world::render_particles::tests::world_hud_bar --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 world-HUD producer key migration; passed 2 filtered tests)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 key=0 hard-reject product route; passed 9 filtered product tests)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 production check; passed with 70 existing warnings)
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs::tests::particle_vertices_filter_sprites_by_selected_camera_layers
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs::tests::particle_velocity_vertices_filter_current_sprites_by_selected_camera_layers
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_skips_core_particle_pass_when_particle_sprites_miss_selected_camera_layers
  - cargo test -p zircon_runtime --lib particle_vertices_filter_sprites_by_selected_camera_layers --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan09-particle-typed-layer-0623 --message-format short --color never -- --test-threads=1 --nocapture (blocked before compilation by current Cargo.lock drift)
doc_type: module-detail
---

# Particle Renderer

The current scene particle path renders transparent camera-facing billboards. `World::collect_render_particles(...)` builds `ParticleExtract` from transient JSON components such as `render.particle_sprites`, `gameplay.particle_sprites`, and world-HUD bars. Each `RenderParticleSpriteSnapshot` contains current-frame state: entity, stable sprite key, position, size, aspect ratio, billboard offset, rotation, sort order, color, intensity, typed `render_layer_mask: RenderLayerSet`, and optional material or texture handles. `RenderParticlePreviousSpriteSnapshot` is the optional previous-state companion keyed by `RenderParticleSpriteIdentity { entity, stable_sprite_key }`. Scene JSON extraction can read `stable_sprite_key`/`sprite_key` for current sprites and wraps the authored legacy entity layer mask into `RenderLayerSet` at the DTO boundary; previous rows are renderer-owned and come from the viewport record after successful submits unless an extract provides explicit previous rows.

`build_particle_vertices(...)` expands those snapshots into six `ParticleVertex` values per selected-camera-visible sprite using the current camera right/up basis, and `build_particle_velocity_vertices(...)` applies the same typed layer intersection before matching previous rows. `RenderPipelineAsset::compile(...)` now auto-inserts `particle-render` only when current particle sprites intersect the selected camera layer set. The compatibility boundary is explicit: `scene/world/render.rs` downgrades particle masks with `to_legacy_mask_lossy()` only when aggregating legacy `VisibilityRenderableInput` data. Status anchor `render_plan09_particle_render_layer_set_snapshot_static_passed_cargo_lock_blocked` records the scoped rustfmt/static pass and the focused locked Cargo blocker caused by current `Cargo.lock` drift. `ParticleRenderer::record(...)` uploads the transient vertex buffer and draws it through `particle.wgsl` into the transparent pass (`particle.transparent`), depth-tested against the current scene depth.

## Temporal Velocity

Plan 06 TP-M1-S8 made the velocity gap explicit rather than writing incorrect velocity. TP-M1-S9 added the first previous-state contract: `ParticleExtract.previous_sprites` can carry previous particle billboard state, and `ParticleExtract::previous_state_sprite_count()` consumes one previous row per matched current sprite. TP-M1-S10 adds the first real WGPU writer: plugin descriptors can declare `particle.velocity`, the built-in executor records through `RenderPassGpuExecutionContext::record_particle_velocity_to_resource(...)`, and `ParticleRenderer::record_velocity(...)` writes matched particle billboards into graph `scene-velocity`. TP-M1-S11 moves the matching key from entity-only to `RenderParticleSpriteIdentity`, so same-entity emitters can opt into deterministic multi-sprite matching by assigning nonzero stable sprite keys. TP-M1-S12 makes previous-state population renderer-owned: `ViewportRecord` stores the current frame's particle sprites as previous rows after successful submit/present/direct runtime-frame paths, and `FrameSubmissionContext` injects those rows when the incoming extract does not provide explicit previous rows. TP-M1-S13 extends those previous rows with optional `RenderParticleBillboardBasisSnapshot`, and the renderer-owned roll records the submitted camera right/up basis for the next frame. TP-M1-S14 adds test-build `scene-velocity` surface readback evidence: `RenderSceneVelocityReadbackReport` summarizes the graph-owned `Rg16Float` target by size, byte length, and nonzero pixel count after graph submission and before transient resource release. TP-M1-S15 extends that readback evidence to the renderer-owned two-frame path: the first submitted particle frame seeds viewport previous rows, and the second moved frame produces nonzero raw `scene-velocity` pixels without author-supplied previous rows. TP-M1-S16 extends the same product path to two same-entity sprites with nonzero stable keys, proving renderer-owned previous rows remain matched and nonzero at the multi-sprite identity boundary. TP-M1-S17 keeps key `0` compatible as a single-sprite anonymous stream but exposes same-entity multi-sprite ambiguity through `RenderStats.last_particle_velocity_anonymous_stream_ambiguity_count` and `render.particle.velocity.anonymous_stream_ambiguity_count`. TP-M1-S18 adds a three-frame product baseline with four same-entity nonzero-key sprites, proving renderer-owned previous rows roll across consecutive successful submits and continue to produce nonzero raw `scene-velocity` pixels beyond the first second-frame handoff. TP-M1-S20 adds and validates a 32-sprite same-entity nonzero-key stress-field product baseline for the same renderer-owned previous-row path. TP-M1/S22 hard-rejects ambiguous key `0` streams in previous-state matching, velocity vertex generation, and renderer-owned previous cache, and migrates generated world-HUD bar sprites to nonzero stable keys.

`build_particle_velocity_vertices(...)` pairs each visible current sprite with one previous row for the same identity, expands both quads into the same triangle topology, and skips sprites without previous state. Key `0` remains the anonymous default stream for single-sprite compatibility, while nonzero keys prevent same-entity rows from cross-matching. When the current frame has multiple key `0` sprites for one entity, `ParticleExtract` marks that entity ambiguous, previous-state matching excludes those rows, the velocity builder skips them, and renderer-owned previous-state roll does not cache them for the next frame. Previous corners use the previous row's stored billboard basis when available; explicit legacy previous rows without that optional basis still fall back to the current camera right/up basis instead of fabricating expanded corners. `particle_velocity.wgsl` projects current corners with `SceneUniform.view_proj_unjittered`, projects previous corners with `SceneUniform.previous_view_proj_unjittered`, and writes clamped screen-space xy velocity to the graph-owned `Rg16Float` target.

`FrameSubmissionContext` carries `particle_sprite_count`, `particle_previous_state_sprite_count`, and `particle_anonymous_stream_ambiguity_sprite_count` from the effective frame extract. `update_base_stats(...)` publishes `RenderStats.last_particle_velocity_missing_sprite_count` as the current sprite count minus the matched previous-state count, and publishes `RenderStats.last_particle_velocity_anonymous_stream_ambiguity_count` as the number of current same-entity key `0` sprites that share an anonymous stream, when all of the following are true:

- reconstructed velocity is requested by motion blur or screen-space reflection;
- the graph executed `particle.transparent`;
- the frame contains at least one particle sprite.

Runtime diagnostics mirror the missing count as `render.particle.velocity.missing_sprite_count` with `particle`, `velocity`, and `missing` tags, and mirror anonymous key ambiguity as `render.particle.velocity.anonymous_stream_ambiguity_count` with `particle`, `velocity`, and `anonymous` tags. Product coverage proves that a motion-blur/TAA particle frame without previous state reports one missing sprite, while the same frame with a previous row reports zero. `render_product_particle_velocity` also proves that `particle.velocity` executes before `particle.transparent` and `temporal.taa-resolve`, that the missing-previous-state path safely no-ops the draw while still preserving graph ordering, that the second frame can use renderer-owned viewport previous-state without author-supplied rows, that both explicit matched rows and renderer-owned second-frame rows produce nonzero raw `scene-velocity` pixels in the test-build readback report, that two same-entity sprites with nonzero stable keys keep missing count at zero and produce nonzero readback on the renderer-owned second frame, that four keyed sprites keep missing count at zero across two consecutive moved frames after the seed frame, and that two same-entity key `0` sprites report anonymous ambiguity and remain missing on the renderer-owned second frame instead of generating unstable velocity. The 32-sprite keyed stress-field product test extends the same assertions to a broader list, and the filtered product group now passes 9 tests.

## Remaining Work

The V1 writer is not the final particle temporal policy. The stable key contract, viewport-record previous-state owner, renderer-owned previous billboard basis, test-build pixel readback, two-frame renderer-owned motion baseline, same-entity keyed multi-sprite product baseline, three-frame keyed dynamic baseline, 32-sprite keyed stress-field product baseline, key `0` ambiguity diagnostics, key `0` hard rejection, and world-HUD producer key migration are present. Key `0` remains a compatibility stream only for single anonymous sprites; producers with more than one sprite per entity must assign nonzero keys or their rows are reported as ambiguous and excluded from velocity. RenderDoc screenshots remain acceptance work before this path can be treated as fully visually validated.
