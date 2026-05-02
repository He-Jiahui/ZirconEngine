---
related_code:
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/editor/src/lib.rs
  - zircon_plugins/rendering/features/post_process/runtime/src/lib.rs
  - zircon_plugins/rendering/features/post_process/editor/src/lib.rs
  - zircon_plugins/rendering/features/ssao/runtime/src/lib.rs
  - zircon_plugins/rendering/features/ssao/editor/src/lib.rs
  - zircon_plugins/rendering/features/decals/runtime/src/lib.rs
  - zircon_plugins/rendering/features/decals/editor/src/lib.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/lib.rs
  - zircon_plugins/rendering/features/reflection_probes/editor/src/lib.rs
  - zircon_plugins/rendering/features/baked_lighting/runtime/src/lib.rs
  - zircon_plugins/rendering/features/baked_lighting/editor/src/lib.rs
  - zircon_plugins/rendering/features/ray_tracing_policy/runtime/src/lib.rs
  - zircon_plugins/rendering/features/ray_tracing_policy/editor/src/lib.rs
  - zircon_plugins/rendering/features/shader_graph/runtime/src/lib.rs
  - zircon_plugins/rendering/features/shader_graph/editor/src/lib.rs
  - zircon_plugins/rendering/features/vfx_graph/runtime/src/lib.rs
  - zircon_plugins/rendering/features/vfx_graph/editor/src/lib.rs
---

# Rendering Plugin Options

## Owner model

`rendering` is the umbrella plugin package for the Rendering option pool. It owns
eight optional feature bundles:

- `rendering.post_process`
- `rendering.ssao`
- `rendering.decals`
- `rendering.reflection_probes`
- `rendering.baked_lighting`
- `rendering.ray_tracing_policy`
- `rendering.shader_graph`
- `rendering.vfx_graph`

The runtime catalog exposes the package as `RuntimePluginId::Rendering`, with
target modes limited to `client_runtime` and `editor_host`. The package category
is `rendering`, and `PluginPackageManifest` carries that category through TOML
round-trips and descriptor-derived manifests.

## Default policy

The default-enabled feature set is intentionally limited to the options that
preserve the previous frame graph behavior:

- `post_process`
- `ssao`
- `reflection_probes`
- `baked_lighting`

`decals`, `ray_tracing_policy`, `shader_graph`, and `vfx_graph` are opt-in. VFX
Graph depends on `particles` plus the `runtime.feature.rendering.shader_graph`
capability; the catalog reports it as blocked when those dependencies are not
selected, and it does not implicitly enable either dependency.

## Runtime boundary

`zircon_runtime` still owns only neutral contracts: plugin catalog metadata,
`RenderFeatureDescriptor`, render pass executor registration, RHI capability
requirements, and graph compilation. It does not depend on `zircon_plugins`.

Feature implementations live under `zircon_plugins/rendering/features/*`. The
existing-backed features register descriptors matching the old pass contracts:
SSAO keeps the ambient-occlusion history binding, reflection probes and baked
lighting keep their post-process composite slots, and post process keeps
`post.stack`. The default forward/deferred pipeline assets no longer embed those
features directly; applying the default rendering feature descriptors restores
the legacy pass order.

## V1 feature surfaces

`decals` registers a `rendering.Component.DecalProjector` descriptor plus a
screen/deferred-compatible composite pass.

`ray_tracing_policy` provides a policy report over acceleration structure,
inline ray query, and ray pipeline gates. It does not implement a hardware ray
tracer in V1.

`shader_graph` provides a local asset DTO and a minimal WGSL compiler for
constants, texture samples, math nodes, color output, and material output.

`vfx_graph` provides a VFX asset DTO, compile report, emitter component, an
async simulation pass, and a transparent render pass.

## Reference evidence

The module split follows Unreal's separation between `Renderer`, `RenderCore`,
and `RHI`, plus the plugin-shaped examples in
`PostProcessMaterialChainGraph`, `GPULightmass`, and `Niagara`. Unity Graphics
is the secondary reference for SRP `ScriptableRendererFeature`, RenderGraph,
ShaderGraph, VFX Graph, decals, SSAO, and post-process pass organization.

## Validation

Focused checks that passed for this slice:

- `cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1`
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_rendering_runtime --locked --jobs 1`
- `cargo check --manifest-path zircon_plugins/Cargo.toml` for all eight rendering
  feature runtime crates with `--locked --jobs 1`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_rendering_runtime --locked --jobs 1`
- `cargo test --manifest-path zircon_plugins/Cargo.toml` for all eight rendering
  feature runtime crates with `--locked --jobs 1`
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_rendering_editor --locked --jobs 1`
- `cargo check --manifest-path zircon_plugins/Cargo.toml` for all eight rendering
  feature editor crates with `--locked --jobs 1`
- `cargo test -p zircon_runtime --lib --locked rendering_plugin_default_features_restore_legacy --jobs 1`
- `cargo test -p zircon_runtime --lib --locked builtin_rendering_catalog_declares_owner_features_and_defaults --jobs 1`
- `cargo test -p zircon_runtime --lib --locked compile_options_can_disable_clustered_history_and_rendering_plugin_features --jobs 1`

Full `zircon_plugins` workspace tests and broad runtime test filters are still
deferred while parallel sessions are running long editor/workspace Cargo jobs.
