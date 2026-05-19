---
related_code:
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_node.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/core/framework/render/post_process/validation.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/scene/components/render2d/mod.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_node.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/core/framework/render/post_process/validation.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/core/framework/render/sprite/atlas.rs
  - zircon_runtime/src/core/framework/render/sprite/rect.rs
  - zircon_runtime/src/core/framework/render/sprite/anchor.rs
  - zircon_runtime/src/core/framework/render/sprite/bounds.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/scene/components/render2d/mod.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
plan_sources:
  - user: 2026-05-16 continue Render M4B postprocess pass graph productization
  - user: 2026-05-17 continue M5A PBR material and light runtime baseline
  - user: 2026-05-17 continue M6A sprite/default 2D renderer productization
  - user: 2026-05-18 continue M7A runtime UI placement in the product pipeline
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_runtime/src/graphics/tests/render_product_ui.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - tests/acceptance/render-product-m3a-assets.md
  - tests/acceptance/render-product-m4a-core-pipeline.md
  - tests/acceptance/render-product-m4b-post-process.md
  - tests/acceptance/render-product-m5a-pbr-light.md
  - tests/acceptance/render-product-m6a-sprite-default-2d.md
  - cargo test -p zircon_runtime --locked render_product_post_process
  - cargo test -p zircon_runtime --locked render_graph
  - cargo test -p zircon_runtime --locked render_product_pbr
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo test -p zircon_runtime --locked render_product_sprite
  - cargo test -p zircon_runtime --locked render_product_pipeline
  - cargo test -p zircon_runtime --locked render_product_ui
  - cargo test -p zircon_runtime --locked runtime_ui
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes
  - cargo check -p zircon_runtime --lib --locked
doc_type: milestone-detail
---

# Render M4+ Product Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the M1 render profile vocabulary in `profile.rs` into product-level rendering surfaces for materials, postprocess pass graphs, 2D sprites, anti-aliasing, Solari, VG/HGI integration, and end-to-end frame submit acceptance.

**Architecture:** Neutral contracts stay in `zircon_runtime::core::framework::render`; render-facing assets stay in `zircon_runtime::asset`; runtime-world extraction stays in `zircon_runtime::scene`; concrete WGPU/render-graph execution stays in `zircon_runtime::graphics` and `zircon_runtime::render_graph`; `zircon_app` only selects and stores profiles. Advanced rendering stays profile-gated and plugin/provider-backed, never a dependency for default 2D/3D/UI rendering.

**Tech Stack:** Rust 2021, serde, glam, wgpu 29, Cargo workspace packages `zircon_app`, `zircon_runtime`, `zircon_editor`, `zircon_runtime_interface`, plugin workspace `zircon_plugins`, reference engines `dev/bevy` and `dev/Graphics`.

---

## Current Baseline

- M1 profile contracts exist in `zircon_runtime/src/core/framework/render/profile.rs` and are app-stored through `zircon_app/src/entry/entry_config.rs` and `zircon_app/src/entry/engine_entry.rs`.
- `RenderProductFeature` already names `Material`, `PostProcess`, `Sprite`, `AntiAlias`, `VirtualGeometry`, `HybridGlobalIllumination`, and `Solari`, but most are only profile vocabulary.
- Current asset contracts are `TextureAsset`, `ModelAsset`, `ShaderAsset`, and `MaterialAsset` under `zircon_runtime/src/asset/assets/`; they do not yet expose product-grade render readiness, material dependencies, shader variants, sampler/color-space policy, mesh bounds, or fallback diagnostics.
- `RenderFrameExtract` exists in `zircon_runtime/src/core/framework/render/frame_extract.rs`; submit uses `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/*`; renderer execution still converts through `to_scene_snapshot()` in some paths and built-in rendering remains partly manual rather than graph-authoritative.
- Postprocess has concrete renderer code under `zircon_runtime/src/graphics/scene/scene_renderer/post_process/`, but there is no neutral product pass graph or per-camera effect stack contract.
- Sprite rendering is not a first-class product path; particle billboard sprites exist, but general Bevy-style `Camera2d + Sprite` parity does not.
- Anti-aliasing is a default `Render3d` profile requirement but has no concrete product surface or implementation gate.
- VG/HGI have provider/readback infrastructure, but profile-to-provider wiring, degradation reporting, provider arbitration, and deep submit acceptance are incomplete.
- Solari is profile vocabulary only; there is no plugin, provider, feature descriptor, pass graph node, debug/readback path, or product degradation report.

## Coordination And Safety

- Work stays on `main`; do not create worktrees or branches.
- Re-run cross-session coordination before implementation. Current active overlap areas include ECS/world, reflection registry, plugin profile maturity, taskpool contracts, and color foundation design.
- Do not revert unrelated dirty files. If implementation touches a dirty file, read it first and preserve unrelated changes.
- Use milestone-first cadence: implementation slices may add tests and docs, but build/test execution happens in each milestone testing stage.
- Keep root `mod.rs`, `lib.rs`, and crate entry files structural. New render/product areas must be folder-backed once they own multiple declarations or behavior families.

## File Structure Map

### Shared Render Contracts

Create folder-backed contract modules under `zircon_runtime/src/core/framework/render/`:

- `image/mod.rs`, `image/descriptor.rs`, `image/sampler.rs`, `image/fallback.rs`
- `mesh/mod.rs`, `mesh/topology.rs`, `mesh/bounds.rs`, `mesh/mesh_kind.rs`
- `shader/mod.rs`, `shader/dependency.rs`, `shader/variant_key.rs`, `shader/pipeline_layout.rs`
- `material/mod.rs`, `material/standard_material.rs`, `material/color_material.rs`, `material/dependency_set.rs`, `material/readiness_report.rs`, `material/validation_error.rs`
- `core_pipeline/mod.rs`, `core_pipeline/pipeline_kind.rs`, `core_pipeline/render_phase.rs`, `core_pipeline/phase_item.rs`, `core_pipeline/phase_sort.rs`
- `post_process/mod.rs`, `post_process/effect.rs`, `post_process/stack.rs`, `post_process/pass_graph.rs`, `post_process/validation.rs`
- `sprite/mod.rs`, `sprite/sprite.rs`, `sprite/atlas.rs`, `sprite/bounds.rs`, `sprite/extract.rs`
- `anti_alias/mod.rs`, `anti_alias/mode.rs`, `anti_alias/settings.rs`, `anti_alias/fallback.rs`
- `advanced/mod.rs`, `advanced/degradation.rs`, `advanced/provider_report.rs`, `advanced/profile_runtime_plan.rs`
- `solari/mod.rs`, `solari/settings.rs`, `solari/capability.rs`, `solari/status.rs`

Modify:

- `zircon_runtime/src/core/framework/render/mod.rs`
- `zircon_runtime/src/core/framework/render/profile.rs`
- `zircon_runtime/src/core/framework/render/backend_types.rs`
- `zircon_runtime/src/core/framework/render/frame_extract.rs`
- `zircon_runtime/src/core/framework/render/scene_extract.rs`
- `zircon_runtime/src/core/framework/tests.rs`

### Asset Contracts And Importers

Convert asset files that gain multiple behavior families into folder-backed modules:

- Move `zircon_runtime/src/asset/assets/material.rs` to `zircon_runtime/src/asset/assets/material/mod.rs` plus `alpha_mode.rs`, `standard_material_asset.rs`, `dependency_set.rs`, `validation.rs`, and `conversion.rs`.
- Move `zircon_runtime/src/asset/assets/texture.rs` to `zircon_runtime/src/asset/assets/texture/mod.rs` plus `payload.rs`, `metadata.rs`, `sampler.rs`, `color_space.rs`, and `fallback.rs`.
- Move `zircon_runtime/src/asset/assets/model.rs` to `zircon_runtime/src/asset/assets/model/mod.rs` plus `mesh_asset.rs`, `primitive.rs`, `bounds.rs`, `virtual_geometry.rs`, and `conversion.rs`.
- Move `zircon_runtime/src/asset/assets/shader.rs` to `zircon_runtime/src/asset/assets/shader/mod.rs` plus `language.rs`, `entry_point.rs`, `dependency.rs`, `variant.rs`, and `validation.rs`.

Modify:

- `zircon_runtime/src/asset/assets/mod.rs`
- `zircon_runtime/src/asset/assets/imported.rs`
- `zircon_runtime/src/asset/importer/contract.rs`
- `zircon_runtime/src/asset/importer/ingest/import_material.rs`
- `zircon_runtime/src/asset/importer/ingest/import_shader.rs`
- `zircon_runtime/src/asset/importer/ingest/import_texture.rs`
- `zircon_runtime/src/asset/importer/ingest/import_model.rs`
- `zircon_runtime/src/asset/facade/load_state.rs`
- `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs`
- `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs`
- `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs`

### Scene, Extract, And Runtime Rendering

Create:

- `zircon_runtime/src/scene/components/render2d/mod.rs`
- `zircon_runtime/src/scene/components/render2d/sprite.rs`
- `zircon_runtime/src/scene/components/render2d/mesh2d.rs`
- `zircon_runtime/src/scene/render_extract/sprite.rs`
- `zircon_runtime/src/scene/render_extract/core_pipeline.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/sprite/renderer.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/sprite/pipeline.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/sprite/vertices.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/sprite/phase_queue.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/mod.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/build.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/execute.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/mod.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/fxaa.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/resolve.rs`

Modify:

- `zircon_runtime/src/scene/components/mod.rs`
- `zircon_runtime/src/scene/components/scene.rs`
- `zircon_runtime/src/scene/render_extract/mod.rs`
- `zircon_runtime/src/scene/world/render.rs`
- `zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs`
- `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs`
- `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/mod.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs`

### Advanced Plugins And App Wiring

Create:

- `zircon_plugins/solari/plugin.toml`
- `zircon_plugins/solari/runtime/Cargo.toml`
- `zircon_plugins/solari/runtime/src/lib.rs`
- `zircon_plugins/solari/runtime/src/provider.rs`
- `zircon_plugins/solari/runtime/src/capability.rs`
- `zircon_plugins/solari/runtime/src/status.rs`

Modify:

- `zircon_plugins/Cargo.toml`
- `zircon_app/Cargo.toml`
- `zircon_app/src/entry/first_party_runtime_plugins.rs`
- `zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs`
- `zircon_runtime/src/plugin/runtime_profile.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/capability_summary/capability_summary.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs`
- `zircon_plugins/virtual_geometry/plugin.toml`
- `zircon_plugins/virtual_geometry/runtime/src/lib.rs`
- `zircon_plugins/hybrid_gi/plugin.toml`
- `zircon_plugins/hybrid_gi/runtime/src/lib.rs`

### Tests And Docs

Create or update tests:

- Convert `zircon_runtime/src/core/framework/tests.rs` into `zircon_runtime/src/core/framework/tests/mod.rs` when the first render-product test lands, preserving existing test names and adding child modules.
- `zircon_runtime/src/core/framework/tests/render_product_assets.rs`
- `zircon_runtime/src/core/framework/tests/render_product_pipeline.rs`
- `zircon_runtime/src/core/framework/tests/render_product_post_process.rs`
- `zircon_runtime/src/core/framework/tests/render_product_sprite.rs`
- `zircon_runtime/src/core/framework/tests/render_product_anti_alias.rs`
- `zircon_runtime/src/core/framework/tests/render_product_advanced.rs`
- `zircon_runtime/src/core/framework/tests/render_product_solari.rs`
- `zircon_runtime/src/asset/tests/assets/render_product.rs`
- `zircon_runtime/src/scene/tests/render_extract_product.rs`
- `zircon_runtime/src/graphics/tests/render_product_submit.rs`
- `zircon_runtime/src/graphics/tests/render_product_pbr.rs`
- `zircon_runtime/src/graphics/tests/render_product_post_process.rs`
- `zircon_runtime/src/graphics/tests/render_product_sprite.rs`
- `zircon_runtime/src/graphics/tests/render_product_anti_alias.rs`
- `zircon_runtime/src/graphics/tests/render_product_advanced.rs`
- `zircon_runtime/src/graphics/tests/render_product_ui.rs`
- `zircon_app/src/entry/tests/render_profile_runtime_plugins.rs`
- `zircon_plugins/solari/runtime/tests/solari_profile_gate.rs`

Update test wiring:

- `zircon_runtime/src/asset/tests/assets/mod.rs`
- `zircon_runtime/src/graphics/tests/mod.rs`
- `zircon_runtime/src/core/framework/mod.rs`

Update docs with required YAML frontmatter:

- `docs/assets-and-rendering/bevy-rendering-capability-matrix.md`
- `docs/assets-and-rendering/render-framework-architecture.md`
- `docs/assets-and-rendering/render-product-roadmap.md`
- `docs/zircon_runtime/core/framework/render/profile.md`
- `docs/zircon_runtime/core/framework/render/material.md`
- `docs/zircon_runtime/core/framework/render/post_process.md`
- `docs/zircon_runtime/core/framework/render/sprite.md`
- `docs/zircon_runtime/core/framework/render/anti_alias.md`
- `docs/zircon_runtime/core/framework/render/advanced.md`
- `docs/zircon_runtime/core/framework/render/solari.md`
- `docs/zircon_runtime/asset/render-assets.md`
- `docs/zircon_runtime/graphics/render-product-submit.md`

## Milestone M3A: Product Asset And Material Contracts

**Goal:** Make `RenderProductFeature::{Image, Mesh, Material, Shader}` mean real product-ready contracts instead of raw asset structs.

**In-scope behaviors:**

- Texture assets expose render image metadata: format, color space, sampler, GPU usage, mip count, array layer count, and fallback class.
- Model assets expose mesh asset metadata: topology, bounds, primitive kind, 2D/3D suitability, and optional VG payload presence.
- Shader assets expose dependencies, normalized runtime WGSL source selection, entry point stages, variant keys, and pipeline layout descriptors.
- Material assets expose `StandardMaterial` and `ColorMaterial` product contracts with explicit dependencies, validation, alpha behavior, unlit flag, double-sided flag, and fallback policy.
- Material dependency extraction includes shader and every texture reference used by the material.
- Asset readiness produces structured reports, not silent fallback-only behavior.

**Dependencies:** M1 profile contract and existing asset stack multi-entry importer contract.

**Implementation slices:**

- [x] Convert `material.rs`, `texture.rs`, `model.rs`, and `shader.rs` into folder-backed modules without changing their public module names.
- [x] Add neutral render contracts under `core/framework/render/{image,mesh,shader,material}` and re-export them from `core/framework/render/mod.rs`.
- [x] Add `RenderImageDescriptor`, `RenderImageColorSpace`, `RenderSamplerDescriptor`, `RenderImageFallbackKind`, and `RenderImageUsage`.
- [x] Add `RenderMeshDescriptor`, `RenderMeshTopology`, `RenderMeshBounds`, `RenderMeshKind`, and conversion from `ModelPrimitiveAsset`.
- [x] Add `RenderShaderDependency`, `RenderShaderVariantKey`, and `RenderShaderPipelineLayoutDescriptor`; runtime source selection must prefer non-empty `wgsl_source`, then `source` only for `ShaderSourceLanguage::Wgsl`.
- [x] Add `StandardMaterialDescriptor`, `ColorMaterialDescriptor`, `RenderMaterialDependencySet`, `RenderMaterialReadinessReport`, and `RenderMaterialValidationError`.
- [x] Extend `MaterialAsset` validation so `AlphaMode::Mask { cutoff }` rejects non-finite values and values outside `0.0..=1.0`.
- [x] Extend `ImportedAsset::direct_references()` or its equivalent dependency projection so material shader and texture references become normal asset dependencies.
- [x] Update material, shader, texture, and model import paths to populate the new descriptors and readiness diagnostics.
- [x] Update resource streamer material/shader/texture code so missing references return a readiness report and a declared fallback class.
- [x] Update docs listed in the file map with exact related code, implementation files, plan source, and tests.

**Testing stage:**

- Run `cargo test -p zircon_runtime --locked render_product_assets`.
- Run `cargo test -p zircon_runtime --locked material`.
- Run `cargo check -p zircon_runtime --lib --locked`.
- Debug failures from asset contracts first, then importers, then resource streamer consumers.

**Exit evidence:**

- Tests named with prefix `render_product_assets` pass.
- Material roundtrip, dependency extraction, invalid alpha cutoff, shader runtime source selection, texture metadata, mesh bounds, and fallback readiness assertions pass.
- `docs/zircon_runtime/asset/render-assets.md` and `docs/zircon_runtime/core/framework/render/material.md` describe the accepted product contract.

## Milestone M4A: Core Pipeline Phases And Submit Closure

**Goal:** Make `RenderProductFeature::CorePipeline` map to camera-selected Core2d/Core3d schedules, explicit phases, and a stable direct `RenderFrameExtract` submit path.

**In-scope behaviors:**

- `RenderFrameExtract` carries core pipeline selection and phase-ready payloads directly.
- Core2d and Core3d are neutral render contract names with concrete graph/renderer mapping.
- Render phases include Opaque2d, AlphaMask2d, Transparent2d, Opaque3d, AlphaMask3d, Transparent3d, Prepass, Shadow, Deferred, PostProcess, Ui, Overlay, and Debug.
- Submit path revalidates viewport generation and viewport existence after context build so concurrent viewport destruction or pipeline changes do not panic.
- Legacy `to_scene_snapshot()` is not used by product submit tests as the authority path.

**Dependencies:** M3A render asset readiness; existing M2 camera/layer work if it has landed by implementation time.

**Implementation slices:**

- [x] Add `core_pipeline` neutral contracts and phase sort keys under `zircon_runtime/src/core/framework/render/core_pipeline/`.
- [x] Extend `RenderViewExtract` with a `CorePipelineKind` selected from camera mode.
- [x] Extend `GeometryExtract` with phase-classification inputs while preserving mesh payload identity.
- [x] Extend `RenderPassStage` with product stages needed by Core2d/Core3d without overloading `Opaque` and `Transparent` for all domains.
- [x] Add phase queue builders for mesh draw classification using material alpha mode and pipeline kind.
- [x] Update `default_forward_plus()` and `default_deferred()` to declare Core3d phase mapping instead of standalone global default behavior.
- [x] Add Core2d pipeline asset declaration with sprite hooks disabled until M6 activates sprite payloads.
- [x] In `submit_frame_extract`, replace the two-lock checked-then-expect pattern with viewport generation validation. If the viewport is gone or generation changed, return a structured `RenderFrameworkError` instead of panicking.
- [x] Add a product submit test that fails if submit requires `RenderFrameExtract::to_scene_snapshot()` to draw the frame.
- [x] Update render architecture docs with the accepted Core2d/Core3d ownership rule.

**Testing stage:**

- Run `cargo test -p zircon_runtime --locked render_product_pipeline`.
- Run `cargo test -p zircon_runtime --locked render_product_submit`.
- Run `cargo check -p zircon_runtime --lib --locked`.
- Debug order: framework contracts, phase classification, pipeline compile, submit generation safety, renderer execution.

**Exit evidence:**

- Phase sort tests prove opaque, alpha mask, and transparent ordering for 2D and 3D.
- Multi-camera or multi-viewport tests prove camera-selected pipeline kind flows into submit.
- Destroyed or changed viewport tests return errors without panic.
- `docs/zircon_runtime/graphics/render-product-submit.md` records the direct submit path.

## Milestone M4B: Postprocess Pass Graph Productization

**Goal:** Make `RenderProductFeature::PostProcess` a per-camera product pass graph instead of renderer-local flags.

**In-scope behaviors:**

- Postprocess stack is a neutral contract on `RenderFrameExtract`.
- Pass graph validates effect order, required inputs, produced outputs, and disabled-effect elision.
- Bloom, color grading, history resolve, and final composite are first-class product nodes.
- Existing SSAO/reflection/HGI-related post inputs remain optional feature inputs and do not become default-profile requirements.
- Concrete renderer execution consumes the validated product graph and records pass execution evidence.

**Dependencies:** M4A core phases and submit closure.

**Implementation slices:**

- [x] Add `PostProcessEffectKind`, `PostProcessEffectSettings`, `PostProcessStackDescriptor`, `PostProcessPassNode`, `PostProcessPassGraph`, and `PostProcessGraphValidationError`.
- [x] Extend `PostProcessExtract` with a validated stack descriptor and graph summary.
- [x] Add postprocess graph build and execute modules under `scene_renderer/post_process/pass_graph/`.
- [x] Register concrete postprocess executors in `render_pass_executor_registry.rs` for bloom, color grading, history resolve, and final composite.
- [x] Update `render_graph_execution_resources.rs` to import scene color, scene depth, history color, and postprocess intermediate resources by declared graph resource name.
- [x] Update renderer stats to report postprocess graph node count, skipped node count, and final composite node.
- [x] Add negative tests for missing scene color, invalid history dependency, duplicate output resource, and postprocess cycles.
- [x] Update `docs/zircon_runtime/core/framework/render/post_process.md` and `docs/assets-and-rendering/render-framework-architecture.md`.

**Testing stage:**

- [x] Run `cargo test -p zircon_runtime --locked render_product_post_process`.
- [x] Run `cargo test -p zircon_runtime --locked render_graph`.
- [x] Run `cargo check -p zircon_runtime --lib --locked`.
- Debug order: neutral graph validation, render graph resource imports, concrete executor registration, renderer stats.

**Exit evidence:**

- Accepted in `tests/acceptance/render-product-m4b-post-process.md` for narrowed `zircon_runtime` scope; no workspace-wide validation was run for this closeout.
- Postprocess pass graph tests cover normal stack, disabled effects, missing inputs, duplicate outputs, and ordering.
- Renderer tests prove enabled bloom/color-grading/final-composite graph nodes are executed through renderer graph records, and `history-resolve` evidence is gated on actual renderer history availability.

## Milestone M5A: PBR Material And Light Product Baseline

**Goal:** Make `RenderProductFeature::{Material, Pbr, Light}` produce a default 3D scene with product-grade StandardMaterial and light extraction.

**In-scope behaviors:**

- `StandardMaterialDescriptor` is used by the mesh pipeline cache and deferred/forward passes.
- Base color, base color texture, normal texture, metallic/roughness, metallic-roughness texture, occlusion, emissive, alpha mode, double-sided, and unlit flag affect phase classification and runtime material keys.
- Directional, point, spot, ambient, and rect-light contracts are extractable. Rect and ambient can start as renderer-degraded if concrete shaders are not ready in this milestone, but the degradation must be explicit.
- Missing material or missing texture uses declared fallback classes and readiness diagnostics.

**Dependencies:** M3A asset contracts and M4A phase classification.

**Implementation slices:**

- [x] Wire `StandardMaterialDescriptor` into `resource_streamer_ensure_material.rs` and mesh pipeline cache keys.
- [x] Extend mesh draw phase classification to use `AlphaMode` and `double_sided` contract data.
- [x] Add ambient and rect-light neutral DTOs under the light portion of render contracts or extend `scene_extract.rs` with narrow declarations.
- [x] Add extraction from runtime scene components into `LightingExtract` for every light kind present in runtime components.
- [x] Ensure missing material, missing shader, and missing texture are reported through `RenderMaterialReadinessReport` and renderer stats.
- [x] Update PBR docs and matrix status.

**Testing stage:**

- [x] Run `cargo test -p zircon_runtime --locked render_product_pbr`.
- [x] Run `cargo test -p zircon_runtime --locked render_product_assets`.
- [x] Run `cargo check -p zircon_runtime --lib --locked`.
- Debug order: material descriptor conversion, phase keys, light extraction, renderer fallback stats.

**Exit evidence:**

- Accepted for runtime-only M5A scope in `tests/acceptance/render-product-m5a-pbr-light.md` using WSL/Linux focused validation.
- StandardMaterial fields roundtrip into renderer keys and phase classification: scalar/color fields, alpha-mask cutoff, double-sided, unlit, and authored PBR texture-slot key bits are covered by `render_product_pbr_streamer_projects_standard_material_into_runtime_key` and existing phase-queue tests.
- Missing dependencies emit readiness diagnostics instead of silent fallback-only behavior: missing material, missing shader, missing texture, unsupported texture upload, and missing runtime WGSL are covered by `render_product_pbr` / `render_product_assets` tests.
- Default 3D fixture submits with lights and PBR material fields without advanced profile features: the M5A submit test records material fallback stats, ambient/rect light counts, and zero VG/HGI executed graph passes.
- Windows-native validation is currently blocked before source by dirty-lockfile dependency skew between `windows 0.61.3` from `gpu-allocator` and `windows 0.62.2` from `wgpu-hal 29.0.3`; do not treat this milestone as full Windows or workspace acceptance.

## Milestone M6A: Sprite And Default 2D Renderer

**Goal:** Make `RenderProductFeature::Sprite` real for default rendering: a first-class 2D sprite component/extract/render path that is independent of particles.

**In-scope behaviors:**

- Runtime scene can express sprite image, atlas region, rect, flip, anchor, custom size, color tint, z order, and render layer.
- `RenderFrameExtract` carries sprite payloads separately from particle sprites.
- Core2d queues sprites into Opaque2d, AlphaMask2d, or Transparent2d according to material/image alpha policy.
- Sprite renderer uses product image/material readiness and fallback reports from M3A.
- Particle billboard sprite code remains particle-owned and does not satisfy `RenderProductFeature::Sprite` acceptance.

**Dependencies:** M3A image/material contracts and M4A Core2d phase support.

**Implementation slices:**

- [x] Add render2d scene component modules for `Sprite2dComponent`, `Mesh2dComponent`, and 2D material handle fields.
- [x] Add `RenderSpriteSnapshot`, `RenderSpriteAtlasRegion`, `RenderSpriteAnchor`, `RenderSpriteBounds`, and `SpriteExtract` under framework render contracts.
- [x] Add scene extract code that projects runtime sprite components into `RenderFrameExtract`.
- [x] Add sprite phase queue builder and sort policy for z order, camera order, and transparent back-to-front ordering.
- [x] Add sprite renderer pipeline, vertex layout, bind groups, and texture fallback integration.
- [x] Add tests proving particle sprites do not count as product sprites.
- [x] Update `docs/zircon_runtime/core/framework/render/sprite.md`.

**Testing stage:**

- [x] Run `cargo test -p zircon_runtime --locked render_product_sprite`.
- [x] Run `cargo test -p zircon_runtime --locked render_product_pipeline`.
- [x] Run `cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes`.
- [x] Run `cargo check -p zircon_runtime --lib --locked`.
- Debug order: scene component extraction, sprite bounds, phase queue, texture/material readiness, WGPU renderer.

**Exit evidence:**

- Accepted for narrowed runtime-only M6A scope in `tests/acceptance/render-product-m6a-sprite-default-2d.md`; no workspace-wide validation or plugin workspace validation was run for this closeout.
- A Bevy-style orthographic `Camera2d + Sprite image` fixture submits through Core2d and records sprite draw stats: `last_sprite_count = 1`, `last_sprite_texture_fallback_count = 1`, `last_sprite_graph_executed_pass_count = 3`, and `last_particle_graph_executed_pass_count = 0`.
- Atlas, rect, flip, anchor, custom size, z order, alpha policy, and visibility-layer extraction tests pass through `render_product_sprite_world_frame_extract_exposes_runtime_sprite_components` and `render_product_sprite_world_frame_extract_filters_by_camera_layers`.
- DefaultRender product acceptance no longer relies on particle billboard rendering for Sprite: product sprite and particle sprite payloads are asserted distinct, and `Mesh2dComponent` does not count as a product sprite.
- Known remaining risks: opaque/alpha-mask/transparent sprite GPU passes currently share one alpha-blended pipeline; alpha-mask cutoff discard, per-phase depth-write/blend specialization, batching, atlas asset import, materialized Mesh2d drawing, AA, UI placement, VG/HGI profile integration, Solari, workspace validation, and plugin workspace validation remain later work.

## Milestone M7A: Runtime UI Placement In Product Pipeline

**Goal:** Keep UI in `DefaultRender` honest by placing runtime UI through the product core pipeline after postprocess and before overlay/final presentation.

**In-scope behaviors:**

- UI render pass placement is explicit for Core2d and Core3d.
- UI uses camera/target selection data instead of being an incidental renderer side path.
- Existing `UiRenderExtract` remains the input contract, while renderer placement and stats become product pipeline-owned.
- World-space UI remains a data-carrying contract; true depth-tested world-space rendering is accepted only when it has a camera-targeted renderer path.

**Dependencies:** M4A core pipeline and M4B postprocess graph.

**Implementation slices:**

- [x] Add a UI phase to `core_pipeline` contracts and graph stage mapping.
- [x] Update submit context to record UI target placement and UI pass order.
- [x] Route `UiRenderExtract` through the product graph record rather than only through legacy command stats.
- [x] Update UI renderer stats with pass order, target size, clipped command count, image payload count, and text payload count.
- [x] Add tests for UI after postprocess and before overlay for Core2d and Core3d.
- [x] Update `docs/assets-and-rendering/runtime-ui-graphics-integration.md` if present and `docs/assets-and-rendering/render-framework-architecture.md`.

**Testing stage:**

- [x] Run `cargo test -p zircon_runtime --locked render_product_ui`.
- [x] Run `cargo test -p zircon_runtime --locked runtime_ui`.
- [x] Run `cargo check -p zircon_editor --lib --locked`.
- Debug order: core phase placement, UI extract target routing, renderer stats, editor compile impact.

2026-05-18 M7A evidence was refreshed with `CARGO_TARGET_DIR=target/codex-native-material-painter`. `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never`, `cargo check -p zircon_editor --lib --locked --jobs 1 --color never`, `cargo test -p zircon_runtime --locked render_product_ui --jobs 1 --message-format short --color never` (2 passed), and `cargo test -p zircon_runtime --locked runtime_ui --jobs 1 --message-format short --color never` (23 runtime lib tests plus 6 `runtime_ui_text_render_contract` tests) passed. No workspace-wide validation or plugin workspace validation was run for this milestone closeout.

**Exit evidence:**

- Runtime UI pass ordering is asserted relative to postprocess and overlay.
- Editor still compiles against shared UI/render contracts.
- Accepted for narrowed M7A runtime/editor UI-placement scope; workspace and plugin validation remain later gates.

## Milestone M8A: Anti-Alias Product Surface

**Goal:** Make `RenderProductFeature::AntiAlias` concrete and per-camera instead of a profile-only claim.

**In-scope behaviors:**

- Neutral AA settings support `Off`, `Auto`, `Fxaa`, `Smaa`, `Taa`, `Msaa`, `Cas`, and `Dlss` as product vocabulary.
- First concrete implementation is FXAA as a postprocess graph node.
- Unsupported modes produce structured fallback reports. `Auto` resolves to FXAA for default 3D when postprocess is available.
- MSAA, SMAA, TAA, CAS, and DLSS are not silently claimed as implemented; they remain explicit fallback or unsupported reports until their concrete render targets/history/backend requirements land.

**Dependencies:** M4B postprocess graph and M4A per-camera pipeline selection.

**Implementation slices:**

- [x] Add `anti_alias` neutral contracts and re-export them from render framework.
- [x] Add `AntiAliasSettings` to `RenderViewExtract` or the camera-facing part of `RenderFrameExtract`.
- [x] Add FXAA shader/pipeline resources under `scene_renderer/anti_alias/fxaa.rs`.
- [x] Insert FXAA into `PostProcessPassGraph` when settings resolve to FXAA.
- [x] Add `AntiAliasFallbackReport` to render stats or advanced provider reports.
- [x] Update profile validation so `DefaultRender` can prove `AntiAlias` is backed by Auto-to-FXAA resolution on supported backends.
- [x] Add negative tests for unsupported DLSS, unsupported MSAA sample count, and TAA without history.
- [x] Update `docs/zircon_runtime/core/framework/render/anti_alias.md`.

**2026-05-18 status:** scoped M8A implementation complete and focused validation passed. `RenderViewExtract` carries `AntiAliasSettings`; Default Forward+/Deferred include `BuiltinRenderFeature::AntiAlias`; the `fxaa` graph pass uses executor `post.fxaa`; `RenderStats` reports `last_anti_alias_fallback` and `last_anti_alias_graph_executed_pass_count`; focused tests cover Auto-to-FXAA plus unsupported DLSS, unsupported MSAA sample count, and TAA without history. `pipeline_compile` now asserts the product graph order including `post-process`, `fxaa`, `runtime-ui`, and overlay, so M8A AA placement stays compatible with the M7A runtime UI graph placement.

**Testing stage:**

- [x] Run `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never`.
- [x] Run `cargo test -p zircon_runtime --locked render_product_anti_alias --jobs 1 --message-format short --color never`.
- [x] Run `cargo test -p zircon_runtime --locked render_product_post_process --jobs 1 --message-format short --color never`.
- [x] Run `cargo test -p zircon_runtime --locked render_product_pipeline --jobs 1 --message-format short --color never`.
- [x] Run `cargo test -p zircon_runtime --locked render_product_ui --jobs 1 --message-format short --color never`.
- [x] Run `cargo test -p zircon_runtime --lib builtin_registry_covers_product_postprocess_executor_ids --locked --jobs 1 --message-format short --color never`.
- [x] Run `cargo test -p zircon_runtime --lib capability_validation --locked --jobs 1 --message-format short --color never`.
- [x] Run `cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never`.
- Debug order: settings resolution, pass graph insertion, renderer FXAA execution, fallback reports.

**Exit evidence:**

- Default 3D resolves AA to a concrete implemented path.
- Unsupported AA modes fail or degrade through structured reports, never silent success.

## Milestone M9A: VG/HGI Deep Profile Integration

**Goal:** Make `AdvancedRender` flow from app profile selection through plugin/provider activation, backend capability validation, extract preparation, submit, readback, feedback, and stats.

**In-scope behaviors:**

- `EntryConfig::render_profile` influences runtime plugin/provider selection for VG/HGI.
- First-party linked provider collection includes virtual geometry and hybrid GI when the selected profile requires advanced rendering.
- Backend capability validation distinguishes default render support from advanced render support.
- Provider arbitration is explicit: duplicate provider IDs reject, priority ties reject, and selected provider is reported.
- Authored VG/HGI payloads take precedence over automatic fallback payloads; automatic fallback is reported as fallback, not as authored scene data.
- VG/HGI readback and feedback merge is production code, not test-only helper behavior.
- Missing provider or missing capability degrades with a report or rejects profile activation according to profile policy.

**Dependencies:** M4A submit closure, M5A material/PBR baseline, existing VG/HGI provider crates.

**Implementation slices:**

- [x] Add `AdvancedRenderDegradation`, `AdvancedProviderReport`, and `AdvancedProfileRuntimePlan` under `core/framework/render/advanced/`.
- [x] Extend `RenderCapabilityKind` with concrete advanced requirements that current VG/HGI code actually needs, such as indirect draw, storage buffers, readback, async compute when used, and ray-query when required by a provider.
- [x] Update capability summary and validation to report default, advanced, and experimental capability classes separately.
- [x] Add framework-local provider arbitration tests for duplicate IDs, highest-priority ties, selected provider IDs, and stats availability.
- [x] Gate advanced capability opt-in and submit context VG/HGI sidebands through provider-backed `AdvancedProfileRuntimePlan` reports.
- [x] Report VG submit payload source through `RenderVirtualGeometryPayloadSource`, `FrameSubmissionContext`, and `RenderStats::last_virtual_geometry_payload_source`, with focused tests for authored priority, automatic fallback labeling, and degraded stale-source clearing.
- [x] Report HGI submit payload source through `RenderHybridGiPayloadSource`, `FrameSubmissionContext`, and `RenderStats::last_hybrid_gi_payload_source`; current HGI has no automatic fallback extract path, so the source is `Authored` or `None`.
- [x] Update `zircon_app/src/entry/first_party_runtime_plugins.rs` and `zircon_app/Cargo.toml` so VG/HGI linked providers can be collected behind explicit app features.
- [x] Update plugin manifests and runtime catalog entries for VG/HGI advanced capability declarations.
- [x] Update `resolve_enabled_features.rs` so profile requirements and compiled pipeline descriptors both participate in enabling advanced providers.
- [x] Move VG/HGI prepared sideband merge helpers from test-only availability into production submit feedback collection.
- [x] Add broader runtime-framework submit tests for provider missing, capability missing, and readback/feedback stats.
- [x] Update `docs/zircon_runtime/core/framework/render/advanced.md` and advanced sections in render architecture docs for the completed runtime-local slices.

**2026-05-19 status:** M9A neutral contract, capability-report, framework-local provider-arbitration, submit-time provider-gating, explicit profile-plus-descriptor feature resolving, VG/HGI payload-source reporting, production HGI/VG sideband readback merge, runtime-framework broader submit acceptance, app provider collection, plugin manifest/catalog, and product-level `render_product_advanced` slices are in place. `AdvancedProfileRuntimePlan` reports per-feature readiness for VG/HGI from `RenderProfileBundle`, `RenderCapabilitySummary`, and `AdvancedProviderAvailability`; missing backend capability and missing provider are separate degradations. `RenderCapabilityKind`, `RenderCapabilitySummary`, RHI backend caps, WGPU capability projection, quality-profile capability validation, and advanced provider reports include concrete advanced requirements for storage buffers, indirect draw, and buffer readback. `RenderCapabilitySummary::capability_class_report(...)` splits default, advanced, and experimental capability classes for diagnostics. VG/HGI provider registrations carry priority; duplicate IDs and ties at the highest priority reject framework construction; selected provider IDs are reported in `RenderStats::advanced_provider_availability`; compile options only opt in advanced capability-gated plugin features when a selected provider exists; `resolve_enabled_features(...)` requires both compiled capability descriptors and ready `AdvancedProfileRuntimePlan` state; `RenderStats::last_advanced_provider_reports` records last-submit per-feature reports; `RenderStats::last_virtual_geometry_payload_source` distinguishes authored VG extracts from automatic fallback extracts after runtime-plan gating clears degraded stale state; `RenderStats::last_hybrid_gi_payload_source` reports HGI `Authored` or `None` because there is no HGI automatic fallback extract path today; and `collect_runtime_feedback(...)` merges renderer readback with `PreparedRuntimeSubmission` HGI/VG sideband readback in production. `zircon_app` links VG/HGI providers behind `first-party-advanced-render-runtime-plugins`; `first_party_runtime_plugin_registrations_for_config(...)` derives advanced provider selections from `EntryConfig::render_profile`; VG/HGI `plugin.toml`, runtime descriptors, and built-in catalog entries declare the profile-facing advanced capabilities; and `render_product_advanced` now proves provider-backed acceptance plus descriptor-only provider-missing degradation through the planned all-target Cargo filter.

**Testing stage:**

- Run `cargo test -p zircon_runtime --locked render_product_advanced`. Passed on 2026-05-19: 2 tests, 0 failures.
- Run `cargo test -p zircon_runtime --locked virtual_geometry`. Passed on 2026-05-19: 47 lib-filtered tests plus filtered integration targets; 4 ignored legacy automatic-VG tests remained ignored by design.
- Run `cargo test -p zircon_runtime --locked hybrid_gi`. Passed on 2026-05-19: 19 lib-filtered tests plus filtered integration targets.
- Run `cargo test -p zircon_app --locked render_profile_runtime_plugins`. Focused 2026-05-19 app-provider variants passed with `first-party-advanced-render-runtime-plugins` alone and with `first-party-runtime-plugins` combined.
- Run `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets`. Passed on 2026-05-19 with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-render-m9a-advanced` after syncing the shader importer and WGSL importer `ShaderAsset` initializers to the current `texture_slots` field. Focused VG/HGI registration tests and package-level `--all-targets` checks for `zircon_plugin_virtual_geometry_runtime`, `zircon_plugin_hybrid_gi_runtime`, `zircon_plugin_asset_importer_shader_runtime`, and `zircon_plugin_shader_wgsl_importer_runtime` also passed.
- Debug order: app profile provider collection, runtime profile catalog, capability validation, submit preparation, provider readback/feedback, stats.

**Exit evidence:**

- `AdvancedRender` cannot silently pass without required providers or backend support.
- VG/HGI submit tests prove profile-selected deep integration through readback and feedback stats.
- `DefaultRender` tests prove VG/HGI remain disabled by default.
- Product-level `render_product_advanced` tests prove provider-backed AdvancedRender acceptance and descriptor-only provider-missing degradation.

## Milestone M9B: Solari Experimental Contract And Gated Runtime Path

**Goal:** Make `SolariExperimental` an explicit experimental product with plugin/provider/capability/degradation semantics instead of profile vocabulary only.

**In-scope behaviors:**

- Solari has neutral contracts, plugin manifest, runtime provider trait, capability requirements, and status/degradation reports.
- Solari profile validation requires ray-query and acceleration-structure capabilities plus any binding-array/bindless capabilities added to `RenderCapabilityKind`.
- Solari runtime provider can report `Unavailable`, `CapabilityMissing`, `ProviderMissing`, `ExperimentalDisabled`, or `Ready`.
- DefaultRender and AdvancedRender do not enable Solari.
- First Solari runtime path is allowed to be a gated unavailable provider with full reports; it must not claim visual raytraced lighting until an actual pass executor exists.

**Dependencies:** M9A advanced provider and capability reporting.

**Implementation slices:**

- [x] Add `solari` neutral contract module with `SolariSettings`, `SolariCapabilityRequirement`, `SolariRuntimeStatus`, and `SolariDegradationReason`.
- [x] Extend `RenderCapabilityKind` with binding-array or bindless resource capabilities required by the local Bevy Solari evidence.
- [x] Create `zircon_plugins/solari/runtime` with provider registration and unavailable-provider behavior.
- [x] Add `zircon_plugins/solari/plugin.toml` and workspace membership in `zircon_plugins/Cargo.toml`.
- [x] Add catalog/runtime-profile entries so `SolariExperimental` can request the Solari plugin explicitly.
- [x] Update profile capability validation for `RenderProductFeature::Solari` to require all Solari experimental capabilities.
- [x] Add tests proving Solari is disabled in DefaultRender and AdvancedRender, rejects missing capabilities, reports provider missing, and reports experimental disabled.
- [x] Update `docs/zircon_runtime/core/framework/render/solari.md`.

**Testing stage:**

- Run `cargo test -p zircon_runtime --locked render_product_solari`.
- Run `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked solari`.
- Run `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets`.
- Run `cargo check -p zircon_app --locked --all-targets`.
- Debug order: capability vocabulary, profile validation, plugin manifest/catalog, app provider collection, provider status.

**Exit evidence:**

- SolariExperimental has a real gated runtime path and clear status reports.
- No default or advanced profile test requires Solari.

## Milestone M10A: End-To-End Product Submit Acceptance

**Goal:** Prove that render profiles correspond to working product paths from app profile selection through asset readiness, scene extract, graph execution, submit, stats, docs, and example fixtures.

**In-scope behaviors:**

- DefaultRender accepts and submits a frame containing 3D mesh/PBR material/lights, 2D sprite, runtime UI, postprocess, and concrete AA without advanced features.
- AdvancedRender accepts and submits VG/HGI only when providers and backend capabilities satisfy requirements.
- SolariExperimental reports a gated experimental status and does not change default acceptance.
- Headless profile does not activate render products.
- Acceptance evidence records commands, output status, skipped unsupported capability cases, and remaining intentional divergences from Bevy.

**Dependencies:** M3A through M9B.

**Implementation slices:**

- [x] Add `tests/acceptance/render-product-default-profile.md` with exact fixture description and commands.
- [x] Add `tests/acceptance/render-product-advanced-profile.md` with VG/HGI provider and fallback cases.
- [x] Add runtime graphics tests under `render_product_submit.rs` for DefaultRender, Headless, AdvancedRender, and SolariExperimental.
- [x] Add app bootstrap tests proving profile bundle selection flows into plugin/provider planning.
- [x] Add docs index links from `docs/assets-and-rendering/index.md` to every render product module doc.
- [x] Update `docs/assets-and-rendering/bevy-rendering-capability-matrix.md` with accepted, degraded, experimental, and intentionally divergent statuses.
- [x] Remove or quarantine product acceptance that still depends on legacy snapshot authority if all product paths have direct extract coverage.

**2026-05-19 status:** M10A focused acceptance is implemented and locally validated on Windows with a shared target dir. `cargo test -p zircon_runtime --locked render_product --jobs 1 --message-format short --color never` passed 57 focused product tests; `cargo test -p zircon_app --locked render_profile --jobs 1 --message-format short --color never` passed the default-feature app profile gate; `cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-advanced-render-runtime-plugins" render_profile --jobs 1 --message-format short --color never` passed the advanced-plugin app profile gate; `cargo check -p zircon_editor --lib --locked --message-format short --color never` passed; and `cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --locked --all-targets --message-format short --color never` passed. Full workspace CI-equivalent build/test and the optional validation-matrix script remain separate final-promotion gates for a cleaner worktree window.

**Testing stage:**

- Run `cargo test -p zircon_runtime --locked render_product`.
- Run `cargo test -p zircon_app --locked render_profile`.
- Run `cargo check -p zircon_editor --lib --locked`.
- Run `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets`.
- Run `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1` when active dirty sessions have settled enough for workspace validation.
- CI-equivalent final gate for clean milestone promotion is `cargo build --workspace --locked --verbose`, `cargo test --workspace --locked --verbose`, `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose`, `cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose`, and `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose`.
- Debug order: runtime product tests, app profile tests, plugin workspace checks, editor compile, full workspace validation.

**Exit evidence:**

- DefaultRender product submit test proves material, postprocess, sprite/2D, AA, UI, and 3D path execute without VG/HGI/Solari.
- AdvancedRender product submit test proves VG/HGI provider path, readback, feedback, and stats are connected.
- SolariExperimental reports explicit experimental status and capability gating.
- Docs and acceptance files contain current related-code headers and command evidence.

## Execution Notes

- Implement milestones in order. Do not start M6 sprite renderer before M3A asset readiness and M4A Core2d phases pass their testing stages.
- Do not treat smoke tests or empty tests as promotion evidence. Each milestone needs positive, negative, boundary, and degradation coverage listed above.
- If an upper-layer failure appears, diagnose lower shared support first: asset readiness, render contracts, extract payloads, phase graph, submit context, renderer execution, then app/plugin wiring.
- Keep profile claims honest. If a feature cannot be backed by a product path in its milestone, adjust the profile or add a structured degradation report before accepting the milestone.
- Keep module docs synchronized with implementation. Every changed code module listed in this plan needs either a mirrored module doc or an update to the existing owning functional doc.

## Self-Review Checklist For Implementers

- Every `RenderProductFeature` named in `profile.rs` has either a product implementation path, a structured degradation path, or an explicit experimental status.
- `DefaultRender` does not enable VG, HGI, or Solari.
- `RenderProductFeature::Sprite` is backed by a non-particle 2D sprite path.
- `RenderProductFeature::AntiAlias` is backed by a concrete default mode or rejected with structured validation.
- Postprocess execution is visible through graph records, not only renderer-local flags.
- Submit tests prove direct `RenderFrameExtract` authority and no viewport race panic.
- VG/HGI tests prove provider selection, missing-provider behavior, readback, feedback, and stats.
- Solari tests prove experimental gating and no default activation.
- Docs frontmatter lists current `related_code`, `implementation_files`, `plan_sources`, `tests`, and `doc_type`.
