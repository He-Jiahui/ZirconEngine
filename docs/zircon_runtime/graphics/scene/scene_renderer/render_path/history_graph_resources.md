---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs
  - zircon_runtime/src/render_graph/types.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-17 implement WGPU-to-render pipeline design from docs/plans/zircon_runtime/render, feature-first with tests deferred
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs::tests::history_binder_imports_enabled_live_history_externals
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs::tests::history_binder_skips_enabled_resources_absent_from_live_graph
doc_type: module-detail
---

# History Graph Resources

`bind_history_graph_resources.rs` owns the built-in scene history actual-binding step before RenderGraph materialization validation. These resources are not transient WGPU resources and are not plugin-owned externals; they are runtime-owned history textures and buffers already allocated by `SceneFrameHistoryTextures`.

The binder takes the compiled graph, the execution resource table, optional history resources, and `HistoryGraphResourceBindingFlags`. It imports only resources that are both enabled for the current frame and present in `CompiledRenderGraph::resource_lifetime_by_name(...)`. This keeps the actual WGPU binding aligned with the compiled graph instead of importing every possible history resource unconditionally.

## Bound Resources

The current binding set covers:

- TAA scene color history: `TAA_HISTORY_PREVIOUS` and `TAA_HISTORY_CURRENT`.
- Screen-space reflection previous history: `HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION`.
- HZB previous furthest history: `HISTORY_PREVIOUS_HZB_FURTHEST`.
- Hybrid GI previous history alias: `history-global-illumination`.
- Exposure history buffers: `EXPOSURE_PREVIOUS` and `EXPOSURE_CURRENT`.

Texture history resources are imported into `RenderGraphExecutionResources` as texture views. Exposure history resources are inserted as buffers. Materialization validation then sees typed report-only external lifetimes as bound when those lifetimes are live in the graph.

## Boundaries

This module is intentionally narrower than the HZB execution-owned external buffer binder. It does not make optional history externals required and does not allocate fallback history resources. Plugin-owned external buffers are handled separately by `bind_plugin_graph_resources.rs`; Hybrid GI's `history-global-illumination` name stays here because its backing is the scene history global-illumination texture, not a plugin buffer.

## Validation State

The source-contract tests build small graphs with typed report-only history externals and assert that enabled live externals are bound before validation, while enabled resources absent from the compiled graph are skipped. Focused lib-test execution remains deferred under the implementation-first milestone cadence; scoped `zircon_runtime --features core-min` cargo checks provide the current compile gate for this slice.
