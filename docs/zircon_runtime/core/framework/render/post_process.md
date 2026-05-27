---
related_code:
  - dev/bevy/crates/bevy_post_process/src/lib.rs
  - dev/bevy/crates/bevy_post_process/src/bloom/mod.rs
  - dev/bevy/crates/bevy_post_process/src/effect_stack/mod.rs
  - dev/bevy/crates/bevy_post_process/src/motion_blur/mod.rs
  - dev/bevy/crates/bevy_post_process/src/dof/mod.rs
  - dev/bevy/crates/bevy_post_process/src/msaa_writeback.rs
  - dev/bevy/examples/3d/post_processing.rs
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
  - user: 2026-05-20 continue Bevy-level render postprocess evidence mapping
  - user: 2026-05-22 continue M10 post-process and anti-alias breadth checklist
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

## Bevy Evidence

The Bevy reference surface is `bevy_post_process`, not a single monolithic effect pass. `dev/bevy/crates/bevy_post_process/src/lib.rs:9-36` splits the crate into `auto_exposure`, `bloom`, `dof`, `effect_stack`, `motion_blur`, and `msaa_writeback`, then wires `MsaaWritebackPlugin`, `BloomPlugin`, `MotionBlurPlugin`, `DepthOfFieldPlugin`, and `EffectStackPlugin` into `PostProcessPlugin`.

`dev/bevy/crates/bevy_post_process/src/bloom/mod.rs:44-83` shows bloom as a real Core2d/Core3d post-process system with extracted component data, prepared textures/bind groups, and scheduling before tonemapping. `dev/bevy/crates/bevy_post_process/src/effect_stack/mod.rs:3-6` names the built-in effect-stack features as chromatic aberration and vignette, while `effect_stack/mod.rs:141-165` extracts those camera components and schedules the combined pass before tonemapping.

The heavier Bevy effects have additional prerequisites. `dev/bevy/crates/bevy_post_process/src/motion_blur/mod.rs:75-173` requires depth and motion-vector prepasses and runs in the Core3d post-process set before bloom. `dev/bevy/crates/bevy_post_process/src/dof/mod.rs:69-241` defines depth-of-field camera state, prepares depth/focus resources, and schedules after bloom and before tonemapping. `dev/bevy/crates/bevy_post_process/src/msaa_writeback.rs:21-33` registers MSAA writeback before the Core2d/Core3d main pass, and `msaa_writeback.rs:110-126` only inserts the blit pipeline when camera writeback policy and MSAA sample count require it.

The user-facing Bevy example `dev/bevy/examples/3d/post_processing.rs` demonstrates the effect-stack authoring model by attaching chromatic aberration and vignette components to a 3D camera. Zircon does not yet expose equivalent camera components for those effects.

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

## Bevy Gap Classification

Zircon currently covers the neutral product graph, bloom/color-grading/history/final-composite node vocabulary, graph validation, renderer execution evidence, and the FXAA node that is shared with the anti-alias surface. That is enough for DefaultRender diagnostics and pass-order accountability.

Zircon is not yet Bevy-complete for post-processing. Motion blur is not implemented because the required motion-vector prepass contract has not been productized. Depth of field is not implemented because the camera focus/lens model and auxiliary blur resources are not exposed through `render::camera` or `render::post_process`. Chromatic aberration and vignette are not implemented as authorable camera components. MSAA writeback is represented only indirectly through camera MSAA settings and anti-alias fallback reporting; there is no Bevy-style sorted-camera MSAA writeback blit path yet.

The next Bevy-parity implementation milestone should add typed post-process authoring descriptors before adding more shader passes. The safe order is: camera-facing post-process settings, neutral graph nodes/resources, validation and stats, then concrete WGPU execution. This keeps advanced effects from bypassing the basic DefaultRender product contract.

## M10.6 Promotion Gate

M10.6 is the post-process side of the post-process/AA breadth gate. It does not treat the existing bloom, color grading, history resolve, final composite, or FXAA nodes as full Bevy post-process parity. Bevy's `PostProcessPlugin` installs MSAA writeback, bloom, motion blur, depth of field, and the chromatic-aberration/vignette effect stack as separate products, so Zircon promotion has to prove each family independently.

| Check | Current evidence | Promotion requirement |
| --- | --- | --- |
| Product graph stays family-aware. | `PostProcessStackDescriptor` and `PostProcessPassGraph` expose bloom, color grading, history resolve, final composite, and FXAA nodes with graph validation and stats. | New effects must first add typed authoring descriptors, stable resource names, validation rules, skipped/executed stats, and pass-order diagnostics before renderer pixels are accepted. |
| Motion blur is not hidden behind post-process success. | Zircon has no productized motion-vector prepass contract for motion blur. | Add camera-facing motion-blur settings, depth/motion-vector prepass ownership, Core3d ordering before bloom, missing-prepass diagnostics, and focused tests. |
| Depth of field is not hidden behind bloom. | Zircon has no camera focus/lens model or auxiliary DoF texture resources in this contract. | Add focal/lens settings, Gaussian/bokeh mode vocabulary or an intentional narrower subset, auxiliary resource validation, and pass ordering after bloom before tonemapping-equivalent output. |
| Effect stack remains explicit. | Chromatic aberration and vignette are documented gaps, not disabled graph nodes today. | Add camera components or descriptors, LUT/resource fallback policy, extract/prepare diagnostics, and graph stats before claiming the effect-stack family. |
| MSAA writeback remains a target/AA boundary. | Camera MSAA can request an AA mode and unsupported sample counts degrade through `AntiAliasFallbackReport`; no sorted-camera MSAA writeback blit exists. | Add multisampled target ownership, sorted-camera writeback policy, resolve/writeback graph node, and target-aware diagnostics before calling MSAA writeback complete. |
| Validation is focused and not full Bevy parity. | Existing graph tests cover disabled effects, missing resources, duplicate outputs, missing dependencies, and cycles. | Current-checkout M10W validation passed the focused post-process graph tests, AA fallback tests, pipeline/pass-order tests, and `cargo check -p zircon_runtime --lib --locked`; this still does not promote missing effect families. |

2026-05-26 M10W validation evidence:

- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked render_product_post_process --jobs 1 --message-format short --color never`: PASS, 9 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked runtime_ui_graph_pass_order --jobs 1 --message-format short --color never`: PASS, 2 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never`: PASS, 39 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`: PASS with 7 existing warnings.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers disabled-effect elision, missing scene color, invalid history input, duplicate output resources, missing effect dependency, and dependency cycles.

`zircon_runtime/src/graphics/tests/render_framework_bridge.rs` covers renderer-facing stats, verifies bloom, color grading, and final composite are recorded as product postprocess nodes without appending synthetic entries to the normal render graph pass list, and checks that history resolve is recorded only after compatible frame history exists.
