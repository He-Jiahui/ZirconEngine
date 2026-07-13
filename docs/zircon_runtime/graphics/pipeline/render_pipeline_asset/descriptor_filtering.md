---
related_code:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/fxaa.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/smaa.rs
implementation_files:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-17 continue Plan 01 compile.rs modularization
tests:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_routes_bloom_extract_after_split_scene_color_passes
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_declares_uber_light_list_frame_resource_for_default_stack
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_declares_uber_light_list_as_external_when_clustered_lighting_is_disabled
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_routes_output_transfer_through_fxaa_terminal_input
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_routes_output_transfer_through_smaa_terminal_input
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_preserves_compute_workload_from_feature_descriptor
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-descriptor-filtering-0617 --message-format short --color never
  - cargo test -p zircon_runtime --lib light_list --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-product-light-list-0618 --message-format short --color never -- --test-threads=1 --nocapture
doc_type: module-detail
---

# Pipeline Descriptor Filtering

## Purpose

`render_pipeline_asset/descriptor_filtering.rs` owns compile-time feature descriptor filtering and post-process resource routing. It receives renderer feature descriptors plus `RenderPipelineCompileOptions`, then removes disabled passes, trims inactive resources, and rewrites post-process inputs so the compiled render graph follows the active effect stack.

This keeps `compile.rs` focused on orchestration: validation, feature enablement, metadata collection, and the frame-extract-dependent core particle descriptor insertion. `pass_authoring.rs` receives this module's final descriptor set and performs graph handle creation plus pass/resource lowering.

## Related Files

- `compile.rs` calls `feature_descriptor(...)` for asset validation and `feature_descriptor_for_options(...)` for enabled runtime lowering.
- `pass_authoring.rs` authors graph passes and resource accesses after filtering has decided which logical names remain.
- `resource_descriptors.rs` sizes resources after filtering has decided which logical names remain.
- `core/framework/render/post_process/stack.rs` defines the effect stack and resource name constants consumed by this module.
- `fxaa.rs` and `smaa.rs` provide the terminal anti-alias executor ids used when selecting the active terminal AA pass.
- `compile_tests.rs` keeps the source-contract assertions for bloom routing, terminal AA routing, compute workload preservation, required external binding, and descriptor sizing out of production `compile.rs`.

## Behavior Model

Filtering starts from the descriptor declared by a `RendererFeatureAsset`.

When HZB occlusion is disabled, the HZB occlusion cull pass is removed while HZB build resources remain available for other consumers such as SSR and depth pyramids. When no post-process stack is supplied, optional TAA resolve resources, Color LUT bake, upscale, and terminal SMAA plugin passes are removed so the graph does not declare inactive transient resources.

When a post-process stack is supplied, the module:

- keeps only optional passes whose `PostProcessEffectKind` is enabled;
- keeps resources named by the stack's initial resources or enabled effect inputs/outputs;
- preserves terminal resources such as `final-color`, `final-composited`, `tonemapped`, GI, and contact shadow resources even when they are not produced by a stack entry;
- preserves `LIGHT_LIST` as a basic post-process stack input because `post.uber`'s bind group always needs the cluster buffer; default forward+ reads the graph-owned clustered-lighting buffer, while clustered-lighting-disabled profiles read the renderer-owned frame external buffer;
- rewrites bloom, scene-composite, blur, uber, upscale, and terminal anti-alias inputs so each pass reads the latest scene-color-like output instead of a stale earlier resource;
- routes `post.output-transfer` through `FINAL_COMPOSITED` before terminal FXAA or SMAA writes the real `FINAL_COLOR`.

## Design And Rationale

Descriptor filtering is logical graph policy, not WGPU descriptor sizing. Keeping it separate from `resource_descriptors.rs` avoids mixing "which resources exist" with "how large and what format should they be." Keeping the frame-extract-dependent particle descriptor insertion in `compile.rs` avoids giving this module a hidden dependency on scene extract state.

The module deliberately rewrites feature descriptors before graph resource planning. That means `graph_resources.rs` sees the final logical resource set and can merge external bindings or reject same-name type conflicts without needing to understand post-process stack semantics.

## Edge Cases And Constraints

Only pass/resource declarations are rewritten. The module does not allocate handles, create textures or buffers, author graph passes, or bind WGPU resources.

Terminal anti-alias routing is explicit because FXAA and SMAA are terminal passes that consume the post-process output-transfer result and then write `FINAL_COLOR`. Without the `FINAL_COMPOSITED` rewrite, output-transfer and terminal AA would both compete for the same final write.

Plugin post-process descriptors are filtered only for known terminal SMAA executor id behavior. Other plugin descriptors remain unchanged unless their own feature asset is disabled by compile options.

## Test Coverage

`compile_tests.rs` contains the routing source contracts that exercise this module through public pipeline compilation. The bloom routing test proves bloom reads the latest motion-blurred scene color. The light-list tests prove `post.uber` always declares a `LIGHT_LIST` read and that disabling clustered lighting keeps the resource as a frame external instead of dropping it from the graph. The FXAA and SMAA terminal tests prove output-transfer writes `FINAL_COMPOSITED`, the selected terminal AA reads that transient, and only the selected AA pass remains.

Focused lib-tests are still deferred to the milestone testing stage. The scoped `zircon_runtime --features core-min` check in the header is the lightweight implementation validation gate for this split.

## Open Issues Or Follow-Up

`pass_authoring.rs` now owns the pass-authoring loop that maps pass resource declarations onto `RenderGraphBuilder` calls. Remaining follow-up work in this area is behavioral RG-M1 work, not descriptor filtering: non-HZB/non-shadow-atlas executor-owned External actual binding and resource lifetime validation closure.
