---
related_code:
  - dev/bevy/Cargo.toml
  - dev/bevy/docs/cargo_features.md
  - dev/bevy/crates/bevy_internal/src/default_plugins.rs
  - zircon_runtime/src/core/framework/render/advanced/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/feature.rs
  - zircon_runtime/src/core/framework/render/advanced/provider_report.rs
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary/capability_summary.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/apply_flagship_profile_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/new.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/types/graphics_error.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_product_advanced.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/advanced/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/feature.rs
  - zircon_runtime/src/core/framework/render/advanced/provider_report.rs
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary/capability_summary.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/apply_flagship_profile_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/new.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/types/graphics_error.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_product_advanced.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
plan_sources:
  - user: 2026-05-22 continue M10 advanced and Solari separation checklist
  - user: 2026-05-21 continue Bevy advanced/default render boundary evidence
  - user: 2026-05-18 continue Render M9A advanced profile integration
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
tests:
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs::default_render_plan_does_not_request_advanced_providers
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs::compile_options_do_not_enable_advanced_capabilities_without_providers
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs::runtime_profile_bundle_for_quality_profile_defaults_without_advanced_flags
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --color never
  - cargo test -p zircon_runtime --lib advanced --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib capability_validation --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib capability_class_report --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib backend_caps_report_queue_classes_and_rt_support_independently --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib advanced_provider_selection --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib compile_options_ --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib advanced_runtime_plan --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_profile_bundle_for_quality_profile --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib virtual_geometry_payload_source --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib hybrid_gi_payload_source --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib payload_source --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib resolve_enabled_features --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib collect_runtime_feedback --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib render_framework_bridge --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked render_product_advanced --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked virtual_geometry --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked hybrid_gi --jobs 1 --message-format short --color never
  - cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-advanced-render-runtime-plugins" render_profile_runtime_plugins --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib advanced_render_plugin_manifests_declare_profile_capabilities --locked --jobs 1 --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib virtual_geometry_registration_contributes_render_feature_descriptor --locked --jobs 1
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib hybrid_gi_registration_contributes_render_feature_descriptor --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Advanced Render Profile Runtime Plan

## Purpose

`zircon_runtime::core::framework::render::advanced` owns the neutral M9A report vocabulary for advanced render products. It is deliberately framework-level: it describes whether `VirtualGeometry` and `HybridGlobalIllumination` are requested, backed by backend capability, backed by a runtime provider, and therefore ready to run.

This module does not instantiate plugin providers, select app features, or touch renderer-private VG/HGI state. Those steps are later M9A slices. The first contract slice gives those later systems one stable DTO family to report through instead of inventing ad hoc booleans.

## M10L Default Render Boundary Evidence

Bevy keeps default rendering and optional renderer breadth separate in source. `dev/bevy/Cargo.toml:134-151` defines the default profile as the union of 2D, 3D, UI, and audio; the rendering plan intentionally excludes audio but still inherits the separate 2D, 3D, and UI obligations. `dev/bevy/Cargo.toml:198-261` then splits `common_api`, `2d_api`, `2d_bevy_render`, `3d_api`, `3d_bevy_render`, `ui_api`, and `ui_bevy_render`, which means API readiness, renderer readiness, and UI rendering cannot substitute for one another.

`dev/bevy/docs/cargo_features.md:10-52` documents that profiles are high-level product groups and collections compose those groups. `dev/bevy/crates/bevy_internal/src/default_plugins.rs:43-77` loads the render-facing default plugins through `RenderPlugin`, image, mesh, camera, light, pipelined rendering, core pipeline, post-process, anti-aliasing, sprite rendering, UI rendering, and PBR before later plugin families. No Bevy source path makes an advanced renderer product a replacement for those default slices.

Zircon deliberately mirrors that separation at runtime. `RenderProfileBundle::default_render()` is `CommonRenderApi + Render2d + Render3d + Ui`; `RenderProfileBundle::advanced_render()` includes `DefaultRender` and then adds `VirtualGeometry` plus `HybridGlobalIllumination`; `RenderProfileBundle::solari_experimental()` includes `AdvancedRender` and then adds Solari. That inheritance is a dependency rule, not completion evidence. A passing `AdvancedRender` submit only proves the advanced provider-gating path, sideband payload handling, and VG/HGI stats for the advanced profile.

The runtime gates enforce the same rule. `AdvancedProfileRuntimePlan::from_profile_bundle(...)` reports VG/HGI as `NotRequested` for `DefaultRender`, even if providers are available. `compile_options_for_profile(...)` only enables advanced compile capabilities when the selected quality profile requests an advanced feature and a provider is present. `runtime_profile_bundle_for_quality_profile(...)` falls back to `DefaultRender` whenever the quality profile does not set VG/HGI/Solari flags.

M10L therefore treats advanced-render evidence as non-substitutable. It cannot close missing Mesh2d/SpriteMesh rendering, full PBR lighting, UI render targeting, presentation/capture, render diagnostics, shader/material reflection, or Bevy-style render scheduling gaps. Those gaps stay with their default product slices, while this module only owns the advanced profile's capability/provider/runtime-plan truth.

## M10V Advanced And Solari Separation Gate

M10V turns the M10L boundary into the final advanced-render acceptance rule for this render roadmap. Bevy's default render stack is loaded from normal render infrastructure through image, mesh, camera, light, pipelined rendering, core pipeline, post-process, anti-aliasing, sprite/UI render, and PBR in `dev/bevy/crates/bevy_internal/src/default_plugins.rs:43-77`. Bevy Solari is separate: `dev/bevy/crates/bevy_solari/src/lib.rs:29-57` marks it as experimental, puts realtime lighting and raytracing scene setup in a plugin group, and requires ray-query plus binding-array GPU features.

The AdvancedRender side of M10.9 is accepted only when it proves advanced capability honesty: `DefaultRender` reports VG/HGI as `NotRequested`, even with providers linked; `AdvancedRender` reports VG/HGI as `Ready` only when profile request, backend capability, and selected provider all agree; provider absence and backend capability absence remain structured degradation records; submit-side feature enablement, payload source labels, graph pass counts, and stats are cleared when a feature degrades; and Solari remains outside `AdvancedRender` unless the selected product profile is `SolariExperimental`.

This gate also defines what AdvancedRender cannot claim. It cannot substitute for `CommonRenderApi`, Core2d sprite/Mesh2d, presentation targets and screenshots, StandardMaterial/PBR breadth, light families, post-process and anti-alias breadth, render diagnostics, shader/material reflection, or Bevy-style render scheduling. The promotion evidence for this module is provider/status/sideband focused: `render_product_advanced`, provider bridge tests, runtime profile tests, and a scoped `cargo check -p zircon_runtime --lib --locked`.

## Feature Mapping

`AdvancedRenderFeature` currently has two variants:

- `VirtualGeometry`
- `HybridGlobalIllumination`

Each variant maps back to its `RenderProductFeature` and to its required `RenderCapabilityKind` set. The mapping stays in the neutral render framework so profile validation, provider planning, submit preparation, and stats can all talk about the same feature identity.

The current concrete advanced requirement sets are:

- Virtual Geometry: `VirtualGeometry`, `StorageBuffers`, `IndirectDraw`, and `BufferReadback`.
- Hybrid Global Illumination: `HybridGlobalIllumination`, `StorageBuffers`, and `BufferReadback`.

`AsyncCompute` is still modeled as a separate capability, but it is not required by this slice because current render graph queue fallback can run async-declared work on graphics when async compute is unavailable. Ray-query requirements remain reserved for providers that actually require ray-query or for the later Solari milestone.

## Provider Availability

`AdvancedProviderAvailability` is the input side of the runtime plan. It carries optional provider IDs for VG and HGI without owning provider objects. This is intentional: app bootstrap, plugin manifests, and native loading stay outside the neutral runtime plan.

The runtime framework now fills this DTO from selected provider registrations during `WgpuRenderFramework` construction. App bootstrap remains the only layer that may depend on first-party provider implementation crates; once it hands `RuntimePluginRegistrationReport` values to runtime, selected IDs are available through `RenderStats::advanced_provider_availability`.

Graphics bridge tests now keep two separate fixture paths: descriptor-only pluginized render features for missing-provider degradation, and descriptor-plus-no-op-provider fixtures for provider-backed submit acceptance. This prevents runtime graph tests from accidentally treating a descriptor as a runnable advanced feature when provider arbitration has not selected a provider.

## App Provider Collection And Manifests

`zircon_app` exposes `first-party-advanced-render-runtime-plugins` for linking the VG/HGI provider crates. `first_party_runtime_plugin_registrations_for_config(...)` reads `EntryConfig::render_profile` and appends transient target-scoped runtime plugin selections for `RuntimePluginId::VirtualGeometry` and `RuntimePluginId::HybridGi` when the bundle contains the matching advanced product features. `DefaultRender` therefore collects no advanced providers, while `AdvancedRender` collects both providers without requiring every project manifest to list them manually.

The VG/HGI runtime plugin descriptors and `plugin.toml` manifests now declare profile-facing capabilities:

- `runtime.render.advanced.virtual_geometry`
- `runtime.render.advanced.hybrid_gi`

The built-in runtime catalog mirrors those capability declarations and marks both as `Partial`, matching the current provider-backed but still experimental advanced-render maturity. Manifest-contribution tests compare the package manifests, runtime module descriptors, projected descriptor manifests, and catalog descriptors so the capability vocabulary cannot drift silently.

`render_product_advanced` is the product-level acceptance surface for this slice. It submits the same AdvancedRender extract with and without runtime providers: provider-backed runs must report both advanced features as effective, execute VG/HGI graph passes, and mark both payload sources as authored; descriptor-only runs must degrade with `ProviderMissing`, execute zero VG/HGI graph passes, and clear stale payload-source stats.

## Provider Arbitration

`VirtualGeometryRuntimeProviderRegistration` and `HybridGiRuntimeProviderRegistration` carry an integer priority. The default priority is `0`, and providers can call `with_priority(...)` when registration order should not decide selection.

`WgpuRenderFramework::new_with_plugin_render_extensions(...)` selects one provider per advanced feature before renderer state is created:

- duplicate provider IDs reject construction with `GraphicsError::AdvancedProviderSelection`;
- a tie at the highest priority rejects construction instead of silently picking by registration order;
- the highest-priority provider is stored in `RenderFrameworkState` and reported through `RenderStats::advanced_provider_availability`.

The arbitration layer deliberately does not collect providers from `zircon_app` or plugin manifests. It only makes framework behavior deterministic once a caller supplies registrations.

## Submit-Time Gating

Runtime submit now consumes provider availability before advanced features reach prepared sidebands:

- `compile_options_for_profile(...)` only opts in `VirtualGeometry` or `HybridGlobalIllumination` capability-gated plugin features when the quality profile requests the feature, the backend reports the capability, and the selected provider ID exists.
- `resolve_viewport_record_state(...)` builds an `AdvancedProfileRuntimePlan` from the active quality profile's advanced flags, backend capabilities, and selected provider IDs.
- `resolve_enabled_features(...)` requires both compiled feature capability metadata and ready `AdvancedProfileRuntimePlan` state before enabling VG/HGI submit sidebands.
- `FrameSubmissionContext` gates `hybrid_gi_enabled` and `virtual_geometry_enabled` through that runtime plan, so degraded features clear stale runtime state and do not carry authored or automatic sidebands forward.
- `RenderStats::last_advanced_provider_reports` records the per-feature reports from the last submit.

This is submit-local profile participation. It does not replace app-level render profile bundle selection or first-party provider collection; it ensures the runtime framework does not silently treat a requested advanced feature as enabled when the required provider is absent.

`render_framework_degrades_requested_advanced_features_without_runtime_providers` covers the negative submit path: requested VG/HGI features without runtime providers do not enter `last_effective_features`, execute zero VG/HGI graph passes, clear payload-source stats to `None`, and report `ProviderMissing` degradations. The provider-backed bridge path separately proves selected provider IDs reach stats and authored VG/HGI payloads execute with ready reports.

## Payload Source

`RenderVirtualGeometryPayloadSource` records the submit-time origin of the effective VG payload:

- `None`: VG did not submit a payload.
- `Authored`: the frame extract supplied an authored `RenderVirtualGeometryExtract`.
- `AutomaticFallback`: runtime provider extraction built the VG payload from regular mesh input because no authored VG payload was present.

`build_frame_submission_context(...)` keeps authored VG authoritative. It only asks the selected VG runtime provider for automatic extraction when the feature is enabled and no authored VG extract exists. `FrameSubmissionContext` then gates both the payload and its source through `AdvancedProfileRuntimePlan`, so a degraded or descriptor-disabled VG feature clears stale source labels as well as stale payloads.

`RenderStats::last_virtual_geometry_payload_source` mirrors the sanitized context value for diagnostics. This lets submit tests distinguish authored payload execution from automatic fallback execution without treating fallback extraction as authored scene data.

`RenderHybridGiPayloadSource` records the submit-time origin of the effective HGI payload. Current HGI submit does not have an automatic fallback extract builder equivalent to VG's mesh-derived extraction path, so the valid runtime states are:

- `None`: HGI did not submit a payload.
- `Authored`: the frame extract supplied an authored `RenderHybridGiExtract`.

`FrameSubmissionContext` gates the HGI payload source through `AdvancedProfileRuntimePlan` in the same way it gates the HGI extract, update plan, and feedback. `RenderStats::last_hybrid_gi_payload_source` therefore clears to `None` when HGI is degraded or descriptor-disabled, and it does not treat visibility-derived update planning as an authored or fallback HGI payload.

## Sideband Readback Feedback

`PreparedRuntimeSubmission` can carry neutral plugin renderer outputs produced by runtime prepare collectors. `collect_runtime_feedback(...)` now merges HGI and VG renderer readback outputs with those prepared sideband outputs in production, not only in test helpers.

For HGI, renderer and sideband outputs are appended, including scene-prepare atlas/capture/voxel payloads. For VG, page-table/completion/render-path outputs and NodeAndClusterCull page requests are appended before `VirtualGeometryGpuCompletion::from_readback_outputs(...)` and `take_node_and_cluster_cull_page_request_ids()` consume them. Particle feedback keeps its existing renderer-authoritative merge rule.

## Reports And Degradation

`AdvancedProviderReport` is emitted per advanced feature. It records:

- whether the profile requested the feature,
- the provider ID chosen or discovered for that feature,
- a status: `NotRequested`, `Ready`, or `Degraded`,
- structured degradation records.

`AdvancedRenderDegradation` currently reports two concrete reasons:

- `BackendCapabilityMissing`: the profile requested the feature but the backend capability summary does not satisfy one of the feature's required capabilities.
- `ProviderMissing`: the profile requested the feature but no provider ID was available.

If both are true, both degradations are recorded. The report does not hide provider absence behind backend absence, and it does not treat `DefaultRender` as degraded just because advanced features are absent.

## Runtime Plan

`AdvancedProfileRuntimePlan::from_profile_bundle(...)` evaluates a `RenderProfileBundle`, `RenderCapabilitySummary`, and `AdvancedProviderAvailability` into one report per advanced feature.

For `DefaultRender`, the plan reports both advanced features as `NotRequested` and produces no degradation. For `AdvancedRender`, a feature is enabled only when it was requested, all required backend capabilities are present, and a provider ID is present.

This makes the next integration step straightforward: submit-time code can accept only `Ready` reports while stats and profile diagnostics can surface every degraded report without silently pretending VG/HGI ran.

## Backend Capability Vocabulary

M9A extends the neutral backend vocabulary with concrete advanced requirements:

- `RenderCapabilityKind::StorageBuffers`
- `RenderCapabilityKind::IndirectDraw`
- `RenderCapabilityKind::BufferReadback`

`RenderCapabilitySummary` mirrors these fields as `supports_storage_buffers`, `supports_indirect_draw`, and `supports_buffer_readback`. The RHI backend caps expose the same booleans so WGPU capability projection can report them directly. The current WGPU RHI path reports all three as available for the headless baseline used by focused tests.

Capability reporting is also grouped into classes:

- `Default`: current default product capability requirements, currently screen-space anti-aliasing.
- `Advanced`: VG/HGI and their concrete supporting requirements.
- `Experimental`: acceleration structure, inline ray-query, and ray-tracing pipeline capabilities.

`RenderCapabilitySummary::capability_class_report(...)` returns satisfied and missing capabilities for one class. This lets profile diagnostics distinguish default render support from advanced and experimental gaps before later M9A/M9B slices decide whether to degrade, reject, or require explicit provider activation.

## Out Of Scope

This slice still does not implement native dynamic VG/HGI provider loading, Solari, ray-query policy, or deeper VG/HGI visual feature work. It fixes the neutral contract, app-linked first-party provider collection, plugin manifest/catalog capability declarations, framework-local provider arbitration, product-level AdvancedRender acceptance, submit-time provider gating, VG/HGI payload-source reporting, and production HGI/VG sideband readback merge.

## Validation Notes

Fresh M10.9 validation on 2026-05-26 used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate` for the runtime gate:

- `cargo test -p zircon_runtime --locked render_product_advanced --jobs 1 --message-format short --color never` passed: 2 matching tests, 0 failures. This covered provider-backed VG/HGI graph execution with authored payload sources and descriptor-only provider-missing degradation that clears VG/HGI graph pass counts and payload-source stats.

This remains advanced-profile evidence only. It does not promote default render API, Core2d/Mesh2d, PBR/light, presentation, diagnostics, scheduling, or shader/material reflection gaps.

Fresh M9A validation on 2026-05-19 used `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-render-m9a-advanced`. The product and broad runtime filters passed:

- `cargo test -p zircon_runtime --locked render_product_advanced --jobs 1 --message-format short --color never` passed: 2 tests, 0 failures.
- `cargo test -p zircon_runtime --locked virtual_geometry --jobs 1 --message-format short --color never` passed: 47 lib-filtered tests, 2 non-ignored integration tests, 4 ignored legacy automatic-VG tests, and filtered zero-test targets.
- `cargo test -p zircon_runtime --locked hybrid_gi --jobs 1 --message-format short --color never` passed: 19 lib-filtered tests plus filtered zero-test targets.
- Package-level checks passed for the touched provider crates: `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --locked --all-targets --jobs 1` and `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --locked --all-targets --jobs 1`.
- The full plugin workspace check passed after syncing shader importer `ShaderAsset` initializers to the current `texture_slots` field: `cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --locked --all-targets --jobs 1`.
- The schema-sync follow-up was verified with package checks and lib tests for `zircon_plugin_asset_importer_shader_runtime` and `zircon_plugin_shader_wgsl_importer_runtime`.
