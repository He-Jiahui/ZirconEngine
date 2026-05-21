---
related_code:
  - dev/bevy/crates/bevy_shader/src/lib.rs
  - dev/bevy/crates/bevy_shader/src/shader.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/bind_group_layout.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/stage.rs
  - zircon_runtime/src/core/framework/render/shader/entry_point.rs
  - zircon_runtime/src/core/framework/render/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/shader_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_shader.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/stage.rs
  - zircon_runtime/src/core/framework/render/shader/entry_point.rs
  - zircon_runtime/src/core/framework/render/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/shader_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_shader.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
plan_sources:
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Render Shader Contracts

## Purpose

`zircon_runtime::core::framework::render::shader` owns the neutral shader contract that assets, material readiness, renderer preparation, and diagnostics can share without depending on WGPU objects or Bevy's ECS render app. It names shader stages, entry points, serialized dependencies, variant keys, and pipeline layout intent.

This module deliberately does not load files, parse WGSL imports, compile shader modules, allocate bind group layouts, or queue GPU pipelines. Asset import stays under `zircon_runtime::asset`, and concrete shader module or render pipeline creation stays under `zircon_runtime::graphics`.

## Bevy Evidence

Bevy keeps the shader asset surface separate from concrete renderer allocation. `dev/bevy/crates/bevy_shader/src/lib.rs:1-8` exposes `Shader` and `ShaderCache` as the shader crate's public surface. `dev/bevy/crates/bevy_shader/src/shader.rs:33-55` stores raw source, import path, imports, extra imports, shader defs, file dependencies, and validation policy on the shader asset. `shader.rs:85-148` constructs WGSL, GLSL, and SPIR-V shader assets, while `shader.rs:323-382` loads source files and records imported shader file handles.

`dev/bevy/crates/bevy_shader/src/shader_cache.rs:59-66` describes a cache that waits for imports and leaves renderer-specific module compilation to the render device. `shader_cache.rs:182-331` resolves imports, applies shader defs, composes the module, and reports pipelines that must be requeued when a shader changes.

The render-side precedent is `dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs:190-217`, where `PipelineCache` stores queued, creating, ready, and failed pipeline states. `pipeline_cache.rs:438-446` exposes cached bind group layout creation, `pipeline_cache.rs:448-466` requeues dependent pipelines when shader assets change, and `pipeline_cache.rs:468-632` creates render or compute pipelines from shader modules and layout descriptors. `dev/bevy/crates/bevy_render/src/render_resource/bind_group_layout.rs:7-14` describes bind group layouts as the shader resource interface.

Zircon copies the boundary, not the implementation: `render::shader` is the stable DTO layer; `asset::assets::shader` projects authoring data into those DTOs; `graphics` remains the only owner of WGPU shader modules, layouts, and render pipelines.

## Product Surface

`RenderShaderStage` is the common stage vocabulary: vertex, fragment, and compute. The enum is serializable with `snake_case` names so `.zshader`, `.zmeta`, tests, and diagnostics can move stage values across asset and runtime boundaries.

`RenderShaderEntryPointDescriptor` records the public entry-point name plus its `RenderShaderStage`. Asset-side parsing accepts authoring aliases such as `vert`, `vs`, `frag`, `fs`, `comp`, and `cs`, but the framework contract only exposes canonical stage values.

`RenderShaderDependency` records a `ResourceKind` and `AssetReference`. Dependencies are explicit serialized authoring data in the current milestone; they are not inferred from WGSL import syntax by the framework layer.

`RenderShaderVariantKey` records an optional entry point, optional stage, and string define list. It is a neutral key for material or pipeline specialization diagnostics, not a concrete pipeline-cache key. Concrete renderer caches can combine it with target format, material state, mesh layout, and backend limits when needed.

`RenderShaderPipelineLayoutDescriptor` records the intended shader resource interface. Each `RenderShaderBindGroupLayoutDescriptor` stores a group index, optional label, and binding rows. Each `RenderShaderBindingDescriptor` stores binding index, optional label, resource type, and stage visibility. `RenderShaderBindingResourceType` currently names uniform buffers, storage buffers, sampled textures, storage textures, and samplers. `push_constant_ranges` is intentionally a vector of labels or range descriptions rather than a WGPU-native range type because the neutral contract must remain serializable and backend-agnostic.

## Asset Projection

`ShaderAsset::runtime_wgsl_source()` is the runtime source selector. It prefers non-empty emitted `wgsl_source`, then falls back to raw `source` only when `source_language == ShaderSourceLanguage::Wgsl`. Non-WGSL source without emitted WGSL is not render-ready and must fall back or report readiness diagnostics before graphics code attempts to build a shader module.

`ShaderAsset::entry_point_descriptors()` maps serialized `ShaderEntryPointAsset` rows into canonical framework descriptors and filters invalid stage tokens. `ShaderAsset::dependencies()` maps serialized `ShaderDependencyAsset` rows into `RenderShaderDependency`. `ShaderAsset::variant_keys()` derives first-pass keys from entry point names and stage strings. `ShaderAsset::pipeline_layout_descriptor()` clones the serialized layout descriptor so render feature contracts and diagnostics can reason about bind groups without allocating WGPU layouts.

`.zshader` documents are asset-layer authoring documents. They store WGSL file references, entry points, import redirects, material property schema, texture slots, and editor hints. The `.zshader` importer may perform authoring diagnostics such as WGSL capture checks, but `render::shader` stays limited to the product DTOs that the renderer and material readiness layer can consume.

## Graphics Integration

`ResourceStreamer::ensure_shader_source(...)` is the current concrete bridge. It resolves the referenced `ShaderAsset`, requires `runtime_wgsl_source()`, stores the selected WGSL in `ShaderRuntime`, and returns a material readiness fallback report when the shader is missing or cannot provide runtime WGSL. This keeps shader-source failure visible to material diagnostics instead of silently using a fallback.

The mesh renderer cache currently creates WGPU shader modules from the prepared WGSL source and caches modules by shader resource id plus revision. Render pipelines are then keyed by `PipelineKey`, which combines shader identity with material and pipeline state. That is narrower than Bevy's `PipelineCache`, but it preserves the same separation: source and descriptor contracts are asset/framework data, while module and pipeline objects are renderer-owned resources.

## Current Limits

This module is not a full Bevy `ShaderPlugin`, `ShaderCache`, or `PipelineCache`. It does not parse WGSL imports, resolve shader include graphs, apply typed shader-def values, validate Naga modules, track dependent pipelines, deduplicate bind group layouts, or support async pipeline creation states.

The layout descriptor is serialized intent, not reflection. It does not yet derive bind groups from WGSL, validate binding type compatibility, model dynamic offsets, express texture sample types, or map push constants to backend feature gates. Future shader milestones should add those checks below the framework DTO layer so `.zshader` authoring and renderer preparation continue to share one stable contract.

## Test Coverage

`render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts` proves runtime WGSL selection, WGSL fallback source selection, non-WGSL missing-source rejection, entry-point stage projection, dependency projection, variant-key projection, and serialized pipeline layout projection.

The broader `render_product_assets` filter and `cargo check -p zircon_runtime --lib --locked` remain the milestone-level compile/test gates for this surface. The M10D continuation is documentation-only, so no Cargo gate is required unless source changes accompany a later shader contract slice.
