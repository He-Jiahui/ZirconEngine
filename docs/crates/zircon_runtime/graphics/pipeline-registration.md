---
related_code:
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_asset.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_plugins/rendering/features/ssao/runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs
  - tools/tests/test_frameworks_05_layer_direction.py
---

# Graphics Pipeline Registration

`WgpuRenderFramework::register_pipeline_asset(...)` is the concrete graphics authoring entry point. It validates a `RenderPipelineAsset`, compiles the graph with current backend capabilities and linked executor registrations, and installs the asset under its stable `RenderPipelineHandle`.

This API is intentionally inherent on the WGPU implementation. `RenderPipelineAsset` and its renderer/feature schema remain graphics-owned; moving the concrete schema into the neutral core contract would drag graph execution and asset policy downward. Runtime consumers instead use `RenderFramework::set_pipeline_asset(...)` and `reload_pipeline(...)` with handles.

Default pipelines are created during graphics module construction and receive linked plugin render feature descriptors before the neutral manager service is exposed. Tests that need custom authored pipelines operate on `WgpuRenderFramework` directly. Cross-domain integration tests use the already registered default pipeline and validate observable execution rather than reopening a concrete asset-registration escape hatch on the core trait.

## Built-In Feature Descriptor Ownership

When a first-party plugin exposes a runtime built-in render feature, the runtime descriptor remains the single owner of pass ordering, shader source, bindings, dispatch, history declarations, and resource schemas. The plugin owns package identity, capability checks, and registration only, and obtains the descriptor through a narrow `zircon_runtime::graphics` facade.

SSAO follows this rule through `screen_space_ambient_occlusion_render_feature_descriptor()`. `rendering.ssao` delegates to that function instead of copying compute descriptors or including runtime WGSL by path. The descriptor owns the AO compute family and the crate-private 64-byte `SsaoParams` `UNIFORM | COPY_DST` graph ABI. The frame producer builds that ABI from the generated `CompiledAoProfile`; it carries separate AO work and full input extents, resolution divisor, quality work plan, world-unit radius/thickness/bias/falloff, HZB mip limit, and profile generation. Full-resolution AO executes evaluate then 3x3 joint-bilateral spatial denoise directly into final AO. Half-resolution AO uses ceil-divided relative render extents for raw and spatial products, maps every depth/normal/HZB lookup back into the full SceneLinear coordinate domain, and adds a full-resolution depth/normal bilateral upsample that publishes final AO. The generic compute executor does not infer AO history ownership from an output resource name, and temporal AO remains compile-time unavailable until a motion-qualified pass explicitly declares its history contract. This keeps built-in/default and plugin replacement paths on one physical resource contract. As of 2026-08-30, the manifest and runtime catalog both keep `rendering.ssao` disabled by default. Explicit registration must compile with enabled depth, normal, and HZB writers before evaluate; Deferred currently satisfies the writer-order gate, while Forward+ is rejected because it has no normal producer. A compiled AO profile conditionally adds final AO to exactly one deferred-lighting consumer; that shader applies it only to ambient and environment diffuse, not direct, specular, emissive, or unlit output. Evaluate, spatial, and the conditional upsample use separate logical last-good families at shader interface generation 3. Other compute workloads remain fail-closed by default. A failed candidate may reuse a published pipeline only when entry point, workgroup, complete binding ABI, scene layout, and the Runtime09A device epoch still match; successful WGPU validation is required before publication. The actual dispatch receipt distinguishes candidate and resolved artifact fingerprints and records `Ready` or `UsingLastGood` for AO frame diagnostics. Managed Rust/Naga/WGPU, image, RenderDoc, and performance evidence is still pending.

## Validation

Rustfmt parsing/formatting and `test_render_framework_contract_does_not_accept_graphics_pipeline_assets` passed for the 2026-07-13 cut. The production dependency audit reports 2,144 references / 70 domain edges and 23 remaining forbidden references, with `core -> graphics = 0`. Managed Cargo execution is not claimed while shared Windows validation lanes remain active.
