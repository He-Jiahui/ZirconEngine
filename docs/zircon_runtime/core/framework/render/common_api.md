---
related_code:
  - dev/bevy/docs/cargo_features.md
  - dev/bevy/crates/bevy_internal/src/default_plugins.rs
  - dev/bevy/crates/bevy_image/src/lib.rs
  - dev/bevy/crates/bevy_mesh/src/lib.rs
  - dev/bevy/crates/bevy_camera/src/components.rs
  - dev/bevy/crates/bevy_shader/src/lib.rs
  - dev/bevy/crates/bevy_shader/src/shader.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_material/src/lib.rs
  - dev/bevy/crates/bevy_light/src/lib.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/asset/tests/assets/render_product.rs
  - zircon_runtime/src/core/framework/tests.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
plan_sources:
  - user: 2026-05-21 continue M10 common render API readiness checklist
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_texture_metadata_exposes_image_contract
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_model_metadata_exposes_mesh_bounds_and_vg_presence
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_material_dependencies_validation_and_readiness_are_structured
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_material_readiness_reports_unresolved_dependencies_and_fallbacks
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_material_rejects_invalid_alpha_mask_cutoff
  - zircon_runtime/src/core/framework/tests.rs::render_profile_default_bundle_enables_basic_products_without_advanced_paths
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo test -p zircon_runtime render_profile --locked
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Common Render API Contracts

## Purpose

`CommonRenderApi` is the M10 foundation for render product data, not a renderer. It gives camera, image, mesh, shader, and material a stable descriptor/readiness vocabulary that `Render2d`, `Render3d`, and `Ui` can consume without reaching into importer code or concrete WGPU resources.

This mirrors Bevy's split between `common_api` and renderer collections. Bevy documents `common_api` as scene definition and asset-facing API surface that includes camera, color, image, mesh, shader, material, text, HDR, and PNG support, while explicitly saying that it does not include an actual renderer such as `bevy_render` (`dev/bevy/docs/cargo_features.md:44-52`). Bevy's default plugin order then loads `ImagePlugin`, `MeshPlugin`, `CameraPlugin`, `LightPlugin`, render pipeline, post-process, anti-aliasing, sprite/UI render, and PBR as distinct slices (`dev/bevy/crates/bevy_internal/src/default_plugins.rs:43-77`).

Zircon copies that boundary. The common API layer describes what renderable data means; `zircon_runtime::asset` owns import and serialized asset documents; `zircon_runtime::graphics` owns GPU residency, concrete pipeline objects, and presentation.

## Bevy Source Pressure

| Bevy surface | Source reference | Boundary pressure for Zircon |
| --- | --- | --- |
| Profile and collection split | `dev/bevy/docs/cargo_features.md:22-52` | `CommonRenderApi` must be useful without a renderer and must not imply 2D, 3D, UI, or window presentation by itself. |
| Image API | `dev/bevy/crates/bevy_image/src/lib.rs:19-20` and `lib.rs:36-56` | Image contracts need descriptors, loaders/importers, atlas/fallback vocabulary, and later GPU preparation as separate responsibilities. |
| Mesh API | `dev/bevy/crates/bevy_mesh/src/lib.rs:23-64` and `lib.rs:46-106` | Mesh contracts need asset metadata, topology, bounds, and pipeline-key inputs before renderer-specific buffers and draw phases. |
| Camera API | `dev/bevy/crates/bevy_camera/src/components.rs:16-94` | 2D/3D/HDR/compositing signals are camera-side product data that drive schedules and targets later. |
| Shader API | `dev/bevy/crates/bevy_shader/src/lib.rs:7-8`, `shader.rs:35`, and `shader_cache.rs:66-192` | Shader source, entry points, imports, cache readiness, and pipeline requeue state must remain distinguishable from concrete GPU module creation. |
| Material API | `dev/bevy/crates/bevy_material/src/lib.rs:53-99` | Material readiness needs alpha mode, shader handles, bind-group layout intent, pipeline-key bits, bindless policy, prepass/shadow flags, and fallback diagnostics. |
| 3D light extension | `dev/bevy/crates/bevy_light/src/lib.rs:32-63` and `lib.rs:161-163` | Light belongs to Bevy's 3D API extension rather than `common_api`; Zircon should keep it adjacent but not make it a required common API proof. |

## Zircon Surface

`RenderProfileBundle::common_render_api()` exposes `Camera`, `Image`, `Mesh`, `Material`, and `Shader` product features (`zircon_runtime/src/core/framework/render/profile.rs:44-52`, `profile.rs:239-248`). `DefaultRender` includes `CommonRenderApi`, `Render2d`, `Render3d`, and `Ui`, but the common profile itself carries no renderer capability requirement (`profile.rs:78-87`, `profile.rs:301-304`).

`render::mod` re-exports the stable common product DTOs from the owning modules: camera snapshots and targets, image descriptors and sampler/fallback kinds, mesh bounds and topology, material descriptors/readiness reports, and shader entry/dependency/layout descriptors (`zircon_runtime/src/core/framework/render/mod.rs:44-85`, `mod.rs:121-128`).

The module ownership is intentionally narrow:

| Zircon module | Common API responsibility | Not owned here |
| --- | --- | --- |
| `render::camera` | Camera target, viewport, order, active state, clear color, HDR, exposure, MSAA, and render layers. | Multi-camera scheduling, window surface lifecycle, screenshot readback, editor authoring UX. |
| `render::image` | Texture/image descriptor, color space, sampler, usage, asset usage, mip/layer metadata, fallback kind. | File decoding, KTX2/DDS/HDR transcoding, dynamic atlas building, WGPU texture allocation. |
| `render::mesh` | Mesh topology, bounds, primitive kind, 2D/3D suitability, counts, and VG payload presence. | Vertex-buffer residency, skinning/morph GPU data, Mesh2d/SpriteMesh draw execution. |
| `render::shader` | Stage, entry point, dependency, variant-key, and pipeline-layout intent DTOs. | WGSL import graph resolution, shader-def composition, Naga validation, concrete shader modules, pipeline cache states. |
| `render::material` | Standard/color material descriptors, dependency set, alpha/fallback policy, structured readiness report. | Material editor UI, shader reflection, bind-group allocation, full Bevy `StandardMaterial` breadth. |
| `render::light` | Adjacent 3D API snapshots and readiness counts for light families. | CommonRenderApi proof, clustered/Forward+ lighting, shadows, probes/IBL, rectangular area-light shading. |

## M10.3 Readiness Gate

M10.3 can be promoted only when every default-render follow-up slice can explain missing resources with structured data rather than string errors or silent fallback. The gate is source-level and validation-level:

| Check | Current evidence | Promotion requirement |
| --- | --- | --- |
| Texture/image projection is typed. | `render_product_assets_texture_metadata_exposes_image_contract` proves format, color space, usage, asset usage, mip/layer data, and fallback kind. | Keep fallback as `RenderImageFallbackKind` and renderer-specific upload failure as diagnostics outside `render::image`. |
| Mesh metadata is typed. | `render_product_assets_model_metadata_exposes_mesh_bounds_and_vg_presence` proves topology, kind, suitability, counts, bounds, and VG payload presence. | Add vertex attribute layout, skinning, morph, and Mesh2d readiness before those products are accepted. |
| Shader runtime source and layout intent are typed. | `render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts` proves runtime WGSL selection, entry-point descriptors, dependencies, variant keys, and layout descriptors. | Later shader milestones must add import resolution, shader-def composition, reflection, and cache-state diagnostics without changing the DTO boundary. |
| Material readiness is structured. | Material asset tests prove dependency collection, descriptor projection, unresolved shader/texture fallback records, and invalid alpha-mask validation. | Missing material resources must produce `RenderMaterialReadinessReport`, not a hidden fallback-only success. |
| Camera product data is a common input. | Camera docs and tests cover viewport, target, layer intersection, scene projection, inactive cameras, ordering, and headless/texture target errors. | Multi-target scheduling and screenshot readback remain presentation gates, not common API completion. |
| Light remains adjacent 3D API. | Light docs and readiness tests report ready/degraded families for default 3D progress. | Do not require `Light` for `CommonRenderApi`; require it for `Render3d`/PBR acceptance instead. |

This document is a docs-only readiness checklist. It records the current evidence and the next promotion criteria, but it does not claim fresh Cargo validation. The M10.3 promotion command remains `cargo test -p zircon_runtime --locked render_product_assets`, followed by `cargo check -p zircon_runtime --lib --locked` when the shared build queue is quiet.
