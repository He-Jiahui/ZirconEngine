---
related_code:
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_node.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/core/framework/render/post_process/validation.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/fxaa.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_node.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/core/framework/render/post_process/validation.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/fxaa.rs
plan_sources:
  - user: 2026-05-16 continue Render M4B postprocess pass graph productization
  - user: 2026-05-18 continue Render M8A anti-alias product surface
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - tests/acceptance/render-product-m4b-post-process.md
  - cargo test -p zircon_runtime --locked render_product_post_process
  - cargo test -p zircon_runtime --locked render_product_anti_alias
  - cargo test -p zircon_runtime --locked render_graph
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Postprocess Pass Graph Contracts

## Purpose

`zircon_runtime::core::framework::render::post_process` owns the neutral M4B postprocess product contract. It describes the per-camera stack as data on `RenderFrameExtract` and keeps effect ordering, resource names, and validation independent from WGPU pipelines.

Concrete rendering stays in `zircon_runtime::graphics`. The renderer consumes the frame's validated graph for execution evidence and treats actual renderer history availability as a final resource gate: when history is not available, it derives a frame-local graph by removing only the history resource/node from the submitted stack. It still uses the existing shader-backed postprocess stack for pixels until later milestones replace individual passes with graph-native implementations.

## Data Model

`PostProcessEffectKind` currently names the first product nodes: bloom, color grading, history resolve, final composite, and FXAA. These names are intentionally product-level, not tied to a specific shader or render-pass asset.

`PostProcessEffectSettings` is the authored node descriptor. It carries the effect kind, enabled flag, required input resource names, produced output resource names, and `after` ordering dependencies.

`PostProcessStackDescriptor` stores the initial resource set and ordered authored effects. `from_extract_settings(...)` derives the default stack from `RenderBloomSettings`, `RenderColorGradingSettings`, history-resolve enablement, and history availability. History resolve requires both an enabled profile feature and a compatible previous history texture; the first compatible frame keeps the history node skipped until renderer history is actually available. Disabled effects remain visible in the descriptor but are elided from executable graph nodes.

`PostProcessPassGraph` is the validated graph summary carried by `PostProcessExtract`. It contains executable nodes, skipped nodes, and the final composite node name for stats and tests.

## Resource Names

`PostProcessGraphResourceNames` defines the stable resource vocabulary used by neutral contracts and concrete renderer resource import:

- `scene-color`
- `scene-depth`
- `history-scene-color`
- `bloom-texture`
- `postprocess.color-graded`
- `postprocess.history-resolved`
- `postprocess.final-composited`
- `final-color`

The compiled-scene renderer imports the physical offscreen target textures under these declared names. `scene-color`, `scene-depth`, `final-color`, and `bloom-texture` map to concrete target textures; `postprocess.color-graded`, `postprocess.history-resolved`, and `postprocess.final-composited` are imported aliases for graph-resource consistency while concrete shader execution still writes through the existing postprocess stack. `history-scene-color` is imported only when `prepare_history_textures(...)` reports a compatible previous history texture, so execution evidence cannot claim history resolve on the first frame after allocation or rotation.

## Validation

`PostProcessPassGraph::validate_stack(...)` enforces the graph invariants before execution evidence is recorded:

- enabled nodes must have all required inputs available before they run,
- produced outputs must not duplicate initial resources or another enabled node output,
- `after` dependencies must target an enabled node in the graph,
- dependency cycles reject the stack before renderer execution.

Disabled effects are not errors. They are converted into `skipped_nodes` so stats and diagnostics can distinguish an authored but disabled effect from a node that never existed.

## Runtime Submit Integration

`build_frame_submission_context(...)` derives the effective stack from the compiled feature set, extract settings, compatible frame history, and resolved anti-alias settings. Profile-disabled bloom or color grading is converted back to default settings before graph validation, history resolve only enters the graph when the compiled pipeline enables the history feature and the viewport already has compatible frame history, and FXAA enters the graph only when `AntiAliasSettings` resolves to `Fxaa`.

`FrameSubmissionContext` carries the effective bloom settings, color-grading settings, `PostProcessStackDescriptor`, and `PostProcessPassGraph`. Extract-submit, present-submit, and direct runtime-frame submit replace the frame's stack and graph with those effective values before calling the renderer, so renderer execution starts from the active pipeline rather than raw authored settings.

The compiled-scene renderer is the final authority for execution evidence. The submitted effective graph remains the source graph for the frame, but after `prepare_history_textures(...)` reports actual renderer history availability, the renderer records a frame-local `PostProcessExtract` graph on `RenderGraphExecutionRecord`. If history is unavailable, that frame-local graph is derived from the submitted validated stack by dropping the history resource and disabling only the history-resolve node. `RenderStats` reads node counts/final-composite metadata from the renderer graph when available. This keeps stats and executed-node evidence aligned when viewport metadata says history is compatible but the concrete history texture was just allocated, released, or resized.

The concrete built-in product executors for `post.bloom`, `post.color-grading`, `post.history-resolve`, `post.final-composite`, and `post.fxaa` require renderer GPU context and validate the graph node's required and produced texture resources through `RenderGraphExecutionResources`. Legacy descriptor IDs such as `post.bloom-extract`, `post.color-grade`, and `post.stack` remain compatibility no-ops for existing compiled pipeline descriptors.

`RenderStats` reports `last_post_process_graph_node_count`, `last_post_process_graph_skipped_node_count`, `last_post_process_final_composite_node`, and `last_post_process_graph_executed_nodes`. The executed-node list is separate from normal render graph passes, so product postprocess evidence does not change existing pass-order expectations such as overlay staying the last compiled graph pass.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers disabled-effect elision, missing scene color, invalid history input, duplicate output resources, missing effect dependency, and dependency cycles.

`zircon_runtime/src/graphics/tests/render_framework_bridge.rs` covers renderer-facing stats, verifies bloom, color grading, and final composite are recorded as product postprocess nodes without appending synthetic entries to the normal render graph pass list, and checks that history resolve is recorded only after compatible frame history exists.
