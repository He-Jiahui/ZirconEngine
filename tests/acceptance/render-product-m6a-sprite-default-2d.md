# Render Product M6A Sprite Default 2D

## Scope
- M6A sprite and default 2D renderer productization from `docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md`.
- Affected layers: neutral sprite DTOs under `zircon_runtime::core::framework::render`, runtime `render2d` scene components, world-to-`RenderFrameExtract` sprite projection, Core2d sprite phase queues, default Core2d pipeline/feature descriptors, concrete sprite renderer, texture fallback stats, and render stats propagation.
- Explicitly out of scope: `.zmaterial` schema/importer work, material editor UI, atlas asset import, materialized Mesh2d rendering, sprite batching, alpha-mask fragment discard, per-phase GPU pipeline specialization, anti-aliasing, UI pass placement, VG/HGI deep profile integration, Solari, full workspace validation, and plugin workspace validation.

## Baseline
- M3A asset contracts, M4A Core2d/Core3d phase support, M4B postprocess graph productization, and narrowed runtime-only M5A PBR/light baseline were accepted before this gate.
- Before M6A, general runtime sprites were not a first-class product path. Particle billboard sprites existed under `ParticleExtract`, but they did not satisfy `RenderProductFeature::Sprite`.
- The checkout remains heavily dirty with unrelated render, asset, UI, hub, platform, plugin, reflection, and ECS sessions. This record covers narrowed M6A runtime/render gates only.

## Test Inventory
- Contract separation cases: `zircon_runtime/src/graphics/tests/render_product_sprite.rs` verifies `RenderSpriteSnapshot` lives in `RenderFrameExtract.sprites` separately from `RenderParticleSpriteSnapshot` in `RenderFrameExtract.particles`.
- Phase queue cases: `render_product_sprite_phase_queue_uses_core2d_phase_order_and_transparent_depth_sort` verifies Opaque2d, AlphaMask2d, and Transparent2d classification plus transparent back-to-front ordering inside authored z order.
- Submit/stat cases: `render_product_sprite_submit_records_sprite_stats_without_particle_feature` submits a Core2d sprite frame and verifies sprite count, sprite texture fallback count, sprite graph executed pass count, and zero particle graph executed pass count.
- World extraction cases: `zircon_runtime/src/scene/tests/world_basics.rs` covers `Sprite2dComponent` extraction with image, material, atlas region, rect, flip, anchor, custom size, tint, z order, alpha mode, Core2d selection, and phase queue identity.
- Boundary/failure-path cases: `render_product_sprite_world_frame_extract_filters_by_camera_layers` verifies camera render layers filter out hidden sprites and visibility input only includes the visible sprite; `render_product_sprite_mesh2d_component_does_not_count_as_particle_sprite` verifies `Mesh2dComponent` is not accepted as product sprite or particle sprite evidence.
- Pipeline compile cases: `default_core2d_pipeline_compiles_expected_stage_order_and_passes` verifies the default Core2d stage order and `sprite.opaque`, `sprite.alpha-mask`, and `sprite.transparent` graph executor ids.

## Tooling Evidence
- Tool name: Windows-native Cargo through PowerShell.
- Why this tool was used: the focused M6A sprite, pipeline, and runtime library gates compile on the current Windows checkout with a shared explicit target directory, and this gives direct local evidence for the affected crate.
- Exact commands:
- `cargo check -p zircon_runtime --lib --locked`
- `cargo test -p zircon_runtime --locked render_product_sprite`
- `cargo test -p zircon_runtime --locked render_product_pipeline`
- `cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes`
- `cargo test -p zircon_runtime --locked render_product_sprite --jobs 1 --target-dir "D:\cargo-targets\zircon-render-m6a-docs" --message-format short --color never`
- `cargo test -p zircon_runtime --locked render_product_pipeline --jobs 1 --target-dir "D:\cargo-targets\zircon-render-m6a-docs" --message-format short --color never`
- `cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes --jobs 1 --target-dir "D:\cargo-targets\zircon-render-m6a-docs" --message-format short --color never`
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-m6a-docs" --message-format short --color never`

## Results
- Scoped runtime library check passed before this closeout: `cargo check -p zircon_runtime --lib --locked` finished successfully.
- M6A sprite runtime gate passed before this closeout: `cargo test -p zircon_runtime --locked render_product_sprite` passed 5 focused tests.
- M4A/M6A pipeline regression gate passed before this closeout: `cargo test -p zircon_runtime --locked render_product_pipeline` passed 2 focused tests.
- Core2d compile gate passed before this closeout: `cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes` passed 1 focused test.
- After adding the explicit sprite render-layer filtering case, the fresh scoped rerun passed: `cargo test -p zircon_runtime --locked render_product_sprite --jobs 1 --target-dir "D:\cargo-targets\zircon-render-m6a-docs" --message-format short --color never` ran 6 tests and passed all 6.
- Fresh M4A/M6A pipeline regression rerun passed: `cargo test -p zircon_runtime --locked render_product_pipeline --jobs 1 --target-dir "D:\cargo-targets\zircon-render-m6a-docs" --message-format short --color never` ran 2 tests and passed both.
- Fresh Core2d compile rerun passed: `cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes --jobs 1 --target-dir "D:\cargo-targets\zircon-render-m6a-docs" --message-format short --color never` ran 1 test and passed.
- Fresh scoped runtime library rerun passed: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-m6a-docs" --message-format short --color never` finished successfully.
- Validation scope was narrowed to `zircon_runtime`; no workspace-wide root validation or plugin workspace validation was run for this closeout.

## Acceptance Decision
- Accepted for narrowed M6A runtime sprite/default 2D renderer baseline based on focused runtime compile, sprite unit/submit tests, Core2d pipeline compile evidence, and module documentation.
- The acceptance reason is that product sprites are now first-class non-particle data in `RenderFrameExtract`, are populated from runtime world components, are queued through Core2d phases, execute through the default Core2d sprite graph passes, draw through a concrete sprite renderer with texture fallback telemetry, and report stats independently from particle rendering.
- Remaining risks: full workspace build/test, plugin workspace validation, per-alpha-mode sprite GPU pipelines, alpha-mask cutoff discard, batching, atlas asset import, materialized Mesh2d drawing, AA, explicit UI pass placement, VG/HGI deep profile integration, and Solari remain open later milestones.
