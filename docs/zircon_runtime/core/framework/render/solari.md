---
related_code:
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
