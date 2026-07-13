---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_asset.rs
  - zircon_runtime/src/core/framework/render/framework.rs
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

## Validation

Rustfmt parsing/formatting and `test_render_framework_contract_does_not_accept_graphics_pipeline_assets` passed for the 2026-07-13 cut. The production dependency audit reports 2,144 references / 70 domain edges and 23 remaining forbidden references, with `core -> graphics = 0`. Managed Cargo execution is not claimed while shared Windows validation lanes remain active.
