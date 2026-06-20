---
related_code:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/graph_resources.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/color_space.rs
  - zircon_runtime/src/core/framework/render/post_process/exposure_settings.rs
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs
implementation_files:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-17 continue Plan 01 compile.rs modularization
tests:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_describes_hzb_and_ssr_reflection_pyramids_as_mip_chain_transients
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_describes_color_lut_as_rgba16float_3d_transient_when_enabled
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_describes_hzb_as_half_power_of_two_mip_chain
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::dynamic_resolution_keeps_terminal_anti_alias_input_at_viewport_size
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-resource-descriptors-0617 --message-format short --color never
doc_type: module-detail
---

# Pipeline Resource Descriptors

## Purpose

`render_pipeline_asset/resource_descriptors.rs` owns the compile-time conversion from logical render graph resource names to concrete `TextureDesc` and `BufferDesc` values. It keeps WGPU descriptor sizing and format policy out of `compile.rs` and `pass_authoring.rs`, so compile orchestration and RenderGraph pass lowering do not grow backend descriptor policy.

This module is intentionally crate-internal to the render pipeline asset compiler. It does not allocate GPU resources. It only describes the transient texture or buffer that `RenderGraphBuilder` should declare after `graph_resources.rs` has decided whether a resource is transient or external.

## Related Files

- `pass_authoring.rs` calls `texture_desc_for(...)` and `buffer_desc_for(...)` while lowering enabled SRP feature descriptors into a `RenderGraphBuilder`.
- `descriptor_filtering.rs` decides which post-process resources survive before the surviving logical names reach this descriptor-sizing module.
- `graph_resources.rs` decides the resource kind first. Only resources planned as transient texture or transient buffer reach this module.
- `core/framework/render/post_process/stack.rs` and `PostProcessGraphResourceNames` define most of the post-process logical resource names that need special sizing.
- `graphics/visibility/occlusion/hzb_builder.rs` supplies the HZB plan used for half-resolution power-of-two HZB transient sizing.

## Behavior Model

`texture_desc_for(...)` handles special render graph texture shapes:

- Color LUT resources become 3D `Rgba16Float` textures with storage, copy source, and copy destination usage.
- Upscale outputs and terminal AA inputs after output transfer (`FINAL_COMPOSITED`) use the final view size; most other transient render textures use the effective render size.
- HZB and SSR pyramid resources derive half-resolution extents and full mip-chain counts where needed.
- Depth or shadow names use depth format when no post-process format override is present.
- HDR scene-color-like resources use the configured intermediate HDR format; final composited and tonemapped resources use SDR formats.
- Non-depth transient textures receive storage and copy-destination usage so compute and post-process passes can bind them without reopening descriptor policy in executor code.

`buffer_desc_for(...)` handles render graph buffer sizing:

- Light-grid params use the fixed uniform size and uniform/copy-destination usage.
- Light-grid z-bin and tile-mask buffers use the current maximum word counts.
- Exposure histogram and exposure current/previous buffers use fixed word counts from the render framework constants.
- Generic transient buffers scale to one `u32` per effective render pixel and use storage plus copy source/destination usage.

## Design And Rationale

The extraction follows the same split used by SRP-style render pipelines: graph lowering decides which logical resources exist, while descriptor policy decides how those resources should be shaped for the backend. Keeping this as a sibling of `compile.rs` avoids a public API before there is a second compiler consumer, but gives future render plans a clear place to add descriptor rules without expanding the pipeline orchestration file.

The module does not own external resource typing or required/report-only binding contracts. Those stay in `graph_resources.rs` and `RenderGraphBuilder` because they are logical graph ownership questions, not transient descriptor sizing questions.

## Test Coverage

`compile_tests.rs` source-contract tests cover the descriptor policy indirectly through compiled graph lifetimes:

- HZB and SSR reflection pyramid tests assert half-resolution sizes, mip counts, and high-quality HDR formats.
- Color LUT tests assert 3D texture dimensions, format, storage usage, and fixed compute dispatch metadata.
- HZB compile tests assert the half-power-of-two HZB extent and mip-chain shape used by the runtime HZB executor.
- The dynamic-resolution terminal-AA regression asserts that `FINAL_COMPOSITED` remains at viewport/presentation size while scene/postprocess internals use the scaled render size.

The 2026-06-17 resource-descriptor extraction is behavior-preserving. Focused lib-tests remain deferred to the milestone testing stage; the scoped `zircon_runtime --features core-min` check listed in the header is the intended lightweight validation gate for this implementation slice.

## Open Issues Or Follow-Up

`pass_authoring.rs` now owns the descriptor-resource to `RenderGraphBuilder` read/write mapping. External binding generalization and resource lifetime validation closure remain RG-M1 behavior work and should not be hidden inside this descriptor module.
