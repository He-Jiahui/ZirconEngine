---
related_code:
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/framework.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/05/failure-2026-07-13-core-contract-reverse-dependencies.md
tests:
  - tools/tests/test_frameworks_05_layer_direction.py
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
---

# Render Framework Contract

`RenderFramework` is the neutral runtime-facing control surface for viewports and already registered render pipelines. It creates/destroys viewports, submits immutable frame extracts, binds presentation surfaces, selects or reloads pipelines by `RenderPipelineHandle`, changes quality profiles, and reports diagnostics/captures.

The contract intentionally does not accept `graphics::RenderPipelineAsset`. Pipeline assets contain concrete renderer stages, feature descriptors, graph mutations, executor identities, and asset references owned by the graphics implementation. Accepting them in `core/framework` previously created the final `core -> graphics` reverse dependency and made every non-graphics test double implement an upper-layer authoring method.

Pipeline authoring and compilation now stay in the graphics domain. Neutral callers can only select or reload a stable handle that the concrete renderer already owns. Plugin render features enter through runtime extension registration and are applied when the graphics runtime builds its default pipelines; they are not smuggled through the core manager trait.

## Validation

The 2026-07-13 Frameworks05 hard cut removed `register_pipeline_asset(...)` and the graphics import from `RenderFramework`, moved concrete registration to an inherent `WgpuRenderFramework` method, removed the obsolete method from all neutral test doubles, and rewired the plugin-executor integration test to use the already registered forward pipeline. The focused owner guard passed and the production audit reduced forbidden references from 24 to 23 with `core -> graphics = 0`. Cargo validation remains deferred behind active shared Windows lanes.
