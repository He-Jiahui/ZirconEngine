---
related_code:
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/Cargo.toml
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/capability.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/lib.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/plugin.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests.rs
  - zircon_plugins/rendering/features/volumetric_fog/editor/src/plugin.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/mod.rs
implementation_files:
  - zircon_plugins/rendering/features/volumetric_fog/runtime/Cargo.toml
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/capability.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/lib.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/plugin.rs
plan_sources:
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests.rs
  - tools/audit_plugin_structure.py
doc_type: module-detail
---

# Volumetric Fog Runtime Plugin

## Purpose

`rendering.volumetric_fog` is the optional plugin switch for the AF-M3 froxel chain. It contributes three Lighting-stage async-compute pass descriptors and the matching runtime executor registrations. Production WGPU pipelines remain owned by `zircon_runtime::graphics`.

## Registration Contract

The feature is disabled by default and exposes `runtime.feature.rendering.volumetric_fog`. When enabled with a non-empty volumetric extract, it adds these passes in order:

1. `volumetric.media_inject`
2. `volumetric.light_scatter`
3. `volumetric.integrate`

The passes execute after ShadowAtlas and LightGrid construction and before forward opaque/sky shading or deferred lighting. Scene shading reads `volumetric.integrated`; scatter also consumes the optional external `history.previous.volumetric.scattering` resource.

The runtime package explicitly enables the `zircon_runtime/graphics` dependency feature. The plugin workspace disables runtime default features, so omitting this edge removes render graph types and registration methods at compile time.

## Resource Contract

The plugin declares `volumetric.media`, `volumetric.scattering`, and `volumetric.integrated` as `Rgba16Float` D3 textures. Low, Medium, and High/Ultra graph compilation select depths 48, 64, and 96 respectively. Media and integrated lifetimes alias one physical slot; scattering occupies the second slot.

Compute workload audit uses semantic extents instead of fixed Medium-sized dispatch counts. Media and scatter use `FroxelGrid`; integrate uses `FroxelGridXy`. The executor resolves dimensions from the compiled graph resource before converting them to workgroups.

## Disable Boundary

When the plugin feature is absent or explicitly disabled, none of the three passes or resources enters the compiled graph, and the graph dump matches the baseline pipeline. The canonical volumetric WGSL include remains available to shader assembly, but its runtime apply flag is disabled and no integrated texture is consumed.

## Validation Status

Plugin descriptor structure and capability audit pass, and locked/offline plugin metadata resolves. The focused tests define graph order, feature-off identity, D3 aliasing, semantic dispatch, and four quality tiers. A pre-fix plugin build correctly failed because the package omitted `zircon_runtime/graphics`; the manifest edge is fixed. Post-fix focused compile/test exceeded current shared-machine resource limits and is not recorded as green.
