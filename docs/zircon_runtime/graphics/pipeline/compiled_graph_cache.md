---
related_code:
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
implementation_files:
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
tests:
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::render_graph_compile_frame_fingerprint_tracks_compile_extract_inputs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::render_graph_compile_frame_fingerprint_tracks_camera_target_and_stack_inputs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_key_tracks_texture_target_format_class
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_from_camera_target_retains_resolved_texture_format
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-target-fingerprint-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib compiled_graph_cache --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-target-fingerprint-0618 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib compiled_graph_cache --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-compiled-graph-format-0623 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib viewport_render_output_target --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-output-target-format-0623 --message-format short --color never -- --test-threads=1 --nocapture
doc_type: module-detail
---

# Compiled Graph Cache

## Purpose

`compiled_graph_cache.rs` owns the runtime cache key for compiled render graphs. Plan 09 M1-S2 uses this boundary to keep each selected camera child submit from accidentally reusing a graph compiled for a different camera target or stack role.

## Frame Fingerprint

`RenderGraphCompileFrameFingerprint` records the selected camera inputs that can affect compiled graph resource descriptors, builtin descriptors, or later frame-resource binding assumptions:

- core pipeline kind;
- selected camera target fingerprint;
- selected camera render type;
- whether a viewport rect is present;
- effective view size and dynamic-resolution render size;
- selected camera HDR and MSAA sample count;
- whether particle sprites require the injected core particle pass.

The target fingerprint records `PrimarySurface`, texture `ResourceId` + resolved width/height + compact RGBA8 target format class, or headless width/height. It deliberately lives in `zircon_runtime::graphics`; the neutral `CameraRenderDescriptor` stays a data carrier and does not know about graph cache policy.

`target_resolution.rs` resolves the selected camera output descriptor before compilation and returns a `ResolvedCameraTargetDescriptor`. `compile_pipeline.rs` passes that descriptor's `RenderGraphCompileCameraTargetFingerprint` into `CompiledGraphCacheKey::from_inputs(...)` for both the early feature-discovery compile and the final effective-options compile. The same resolved descriptor now provides the texture format label used by `ViewportRenderOutputTarget::from_camera_target(...)`, so the renderer-bound frame carries the same `rgba8unorm_srgb` versus `rgba8unorm` target class that was used to build the cache key. `extract_compile_fingerprint(...)` still reads the selected `CameraRenderDescriptor` for render type, viewport-rect presence, HDR, MSAA, and size fields, not the transitional `RenderViewExtract.camera` payload.

`ViewportRenderOutputTarget` uses that preflight label as the expected output-target format when planning graph import and post-graph writeback. The planner now blocks prepared-format drift before direct import, copy, or conversion, so a prepared WGPU resource that reports a different descriptor format cannot execute under a compiled graph key that was selected for the original camera-target format class. The neutral render stats still receive the existing blocked-format report row; the narrower distinction remains inside the renderer planner.

## Current Boundaries

Texture target format is now part of the production compile key, output-target frame state, and prepared-resource execution guard for the supported camera texture-target formats. `rgba8unorm_srgb` and `rgba8unorm` map to distinct `RenderGraphCompileTextureTargetFormat` values, so a linear conversion-writeback target cannot reuse a graph key compiled for the same texture id and size as a direct-import sRGB target, and the prepared output target must still report the same format before execution. Missing, unsupported, non-render-target, non-2D, multi-layer, or multi-mip descriptors still fail target preflight before a cache key or output target is built.

This cache slice does not choose Base/Overlay physical attachment ownership, final target ownership, present ownership, or per-camera history. It only prevents graph reuse across selected-camera target/viewport/render-type inputs already visible at compile time.

## Validation

The cache module has unit coverage for existing compile inputs and for camera target/stack inputs: primary versus texture, different texture ids, headless target, viewport rect presence, Base versus Overlay, and identical texture id/size with different sRGB versus linear target format classes. `viewport_render_output_target.rs` has source coverage for retaining the resolved texture format label from `from_camera_target(...)` and for blocking prepared-format drift on both writeback and graph-import plans. The latest source checks passed scoped `rustfmt --edition 2021 --check`, stale-constructor/static-debt scans, and scoped `git diff --check` with only line-ending warnings. The earlier focused locked cache Cargo command was blocked before compilation by current workspace `Cargo.lock` drift; the later focused locked `viewport_render_output_target` Cargo command entered compilation but timed out after 124s without a test result. No new Cargo pass is claimed for the 2026-06-23 format-label slice yet.
