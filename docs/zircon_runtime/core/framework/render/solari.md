---
related_code:
  - dev/bevy/crates/bevy_solari/src/lib.rs
  - dev/bevy/crates/bevy_solari/src/scene/mod.rs
  - dev/bevy/crates/bevy_solari/src/scene/extract.rs
  - dev/bevy/crates/bevy_solari/src/realtime/mod.rs
  - dev/bevy/crates/bevy_solari/src/realtime/node.rs
  - dev/bevy/crates/bevy_solari/src/pathtracer/mod.rs
  - zircon_runtime/src/core/framework/render/solari/mod.rs
  - zircon_runtime/src/core/framework/render/solari/capability.rs
  - zircon_runtime/src/core/framework/render/solari/settings.rs
  - zircon_runtime/src/core/framework/render/solari/status.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/graphics/solari_runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_plugins/solari/runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/solari/mod.rs
  - zircon_runtime/src/core/framework/render/solari/capability.rs
  - zircon_runtime/src/core/framework/render/solari/settings.rs
  - zircon_runtime/src/core/framework/render/solari/status.rs
  - zircon_runtime/src/graphics/solari_runtime_provider/provider.rs
  - zircon_runtime/src/graphics/solari_runtime_provider/provider_registration.rs
  - zircon_plugins/solari/runtime/src/lib.rs
plan_sources:
  - user: 2026-05-19 continue Render M9B Solari experimental contract
  - user: 2026-05-21 continue Bevy-level Solari experimental gating evidence
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
tests:
  - cargo check -p zircon_runtime --lib --locked --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked render_product_solari --jobs 1 --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_solari_runtime --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Solari Experimental Render Contract

## Purpose

`zircon_runtime::core::framework::render::solari` owns the neutral contract for the Solari experimental render path. It does not implement a raytraced lighting pass. It records whether `RenderProductFeature::Solari` was requested, whether the backend exposes the Bevy Solari capability set, whether a runtime provider was selected, and whether the explicit experimental gate is enabled.

The runtime plugin at `zircon_plugins/solari/runtime` currently registers an honest unavailable provider. This makes product profile wiring testable without pretending that the renderer has a Solari pass executor.

## Bevy Evidence

Bevy keeps Solari outside the default render stack. `dev/bevy/crates/bevy_solari/src/lib.rs:29-39` describes `SolariPlugins` as an experimental plugin group for raytraced lighting, made of `RaytracingScenePlugin` and `SolariLightingPlugin`; the non-realtime `pathtracer::PathtracingPlugin` is explicitly mentioned as validation-only and is not added by default. `lib.rs:49-57` defines the required WGPU feature set: experimental ray query plus buffer/texture binding arrays, non-uniform indexing, and partially-bound binding arrays.

`dev/bevy/crates/bevy_solari/src/scene/mod.rs:39-78` gates scene setup on those features before it mutates render-world state. If the feature check passes, it extends mesh allocator usages for BLAS input/storage, initializes `BlasManager` and `StandardMaterialAssets`, installs `RaytracingSceneBindings`, extracts `RaytracingMesh3d` scene data, prepares/compacts BLAS, and prepares raytracing scene bind groups. `scene/extract.rs:13-53` shows that Bevy's raytracing scene path is tied to `RaytracingMesh3d`, `MeshMaterial3d<StandardMaterial>`, global transforms, previous transforms, and cloned standard-material assets.

`dev/bevy/crates/bevy_solari/src/realtime/mod.rs:35-78` is the realtime lighting side: it loads Solari shader libraries, forces the default opaque renderer method to deferred, checks the same required feature set, initializes compute pipelines during `RenderStartup`, extracts lighting components, prepares per-view resources, and inserts the Solari node into `Core3d` before the normal opaque main pass. `realtime/mod.rs:81-95` also makes the camera/view requirements concrete: `SolariLighting` requires HDR, deferred/depth/motion-vector prepasses, double-buffered prepass resources, storage-binding main textures, and MSAA off.

`dev/bevy/crates/bevy_solari/src/realtime/node.rs:31-55` lists the size of the concrete Solari lighting pipeline family: world-cache decay/compact/update, direct and indirect ReSTIR stages, spatial/temporal passes, specular GI, and optional DLSS ray-reconstruction resolve. `node.rs:78-180` exits early unless all required pipelines, scene bindings, GBuffer/depth/motion-vector textures, current/previous view uniforms, and optional DLSS resources are resident. That is materially larger than Zircon's current status-only provider contract.

`dev/bevy/crates/bevy_solari/src/pathtracer/mod.rs:23-27` states the pathtracer is for reference screenshots rather than games. `pathtracer/mod.rs:34-60` follows the same capability gate, extracts pathtracer components, prepares accumulation textures, and schedules the pathtracer after the Core3d main pass and before tonemapping. Zircon does not claim this validation path yet.

## Capability Contract

`SolariCapabilityRequirement::ALL` mirrors the Bevy Solari feature gate used as evidence for this milestone:

- `InlineRayQuery`
- `AccelerationStructures`
- `BufferBindingArray`
- `TextureBindingArray`
- `NonUniformResourceIndexing`
- `PartiallyBoundBindingArray`

The binding-array entries are first-class `RenderCapabilityKind` variants and are copied from `RenderBackendCaps` through `RenderCapabilitySummary`. `rhi_wgpu::wgpu_backend_caps(...)` derives them from `wgpu::Features` rather than from a product profile flag.

## Runtime Status

`SolariRuntimeReport` is stored in `RenderStats::last_solari_runtime_report`. Submit builds it from four inputs:

- whether the resolved render profile bundle contains `RenderProductFeature::Solari`;
- `SolariSettings`, including `experimental_enabled`;
- backend capability summary;
- selected provider availability.

Status precedence is capability missing, provider missing, experimental disabled, provider unavailable, then ready. Default and advanced profiles therefore report `NotRequested`; `SolariExperimental` can only report `Ready` once backend caps, provider status, and the explicit experimental gate all agree.

## Provider Placement

`SolariRuntimeProviderRegistration` follows the same runtime provider registration pattern as Virtual Geometry and Hybrid GI. `RuntimeExtensionRegistry` collects provider registrations from plugins, `GraphicsModule` carries them into `WgpuRenderFramework`, and framework construction selects one provider by priority.

The selected provider is not a render feature descriptor and does not add graph passes. It only reports availability for the neutral Solari status contract. This keeps M9B separate from any later implementation of a concrete raytraced lighting pass.

## Product Profile Wiring

`RenderQualityProfile::with_solari(true)` resolves to `RenderProductProfile::SolariExperimental` during submit. `zircon_app` maps `RenderProductFeature::Solari` to `RuntimePluginId::Solari` when first-party advanced render plugins are enabled, so app profile selection can collect the provider automatically.

The built-in plugin catalog and runtime profiles list Solari as experimental and optional. Client 3D and Dev profiles can discover it, but DefaultRender and AdvancedRender do not request it unless the product profile explicitly includes `Solari`.

## Bevy Gap Classification

| Bevy Solari area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Experimental plugin grouping | `SolariExperimental` and the first-party Solari runtime plugin/provider registration exist, but the provider deliberately reports `Unavailable`. | Keep Solari opt-in and add a real provider only when the concrete renderer can execute a Solari lighting path. |
| Required GPU capabilities | `SolariCapabilityRequirement::ALL` mirrors Bevy's ray query and binding-array feature gate and reports missing capabilities through `SolariRuntimeReport`. | Keep capability mismatch diagnostics before provider readiness; do not let a profile flag imply backend support. |
| Raytracing scene setup | No BLAS manager, raytracing scene bindings, `RaytracingMesh3d` extraction, or StandardMaterial clone table exists in the Solari path. | Add scene extraction, BLAS build/compact, material binding, and lifetime/resource ownership before claiming scene parity. |
| Realtime Solari lighting | No `SolariLighting` camera component, deferred/prepass/storage-texture/MSAA-off validation, Core3d Solari node, temporal history, or ReSTIR/world-cache compute pipeline family exists. | Add per-view settings, required prepass validation, history resources, compute pipeline cache integration, and Core3d scheduling. |
| Pathtracer validation | Not implemented and not part of the default product path. | Add a separate validation-only path if Solari renderer work needs reference screenshots; keep it out of default runtime profiles. |
| Default render boundary | DefaultRender and AdvancedRender do not request Solari; `SolariExperimental` is the only profile that can request it. | Preserve this boundary so advanced raytracing work does not hide missing baseline camera/light/PBR/sprite/UI render work. |
