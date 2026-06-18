---
related_code:
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
implementation_files:
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
tests:
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::render_graph_compile_frame_fingerprint_tracks_compile_extract_inputs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::render_graph_compile_frame_fingerprint_tracks_camera_target_and_stack_inputs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-target-fingerprint-0618 --message-format short --color never
  - cargo test -p zircon_runtime --lib compiled_graph_cache --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-target-fingerprint-0618 --message-format short --color never -- --nocapture
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

The target fingerprint records `PrimarySurface`, texture `ResourceId`, or headless width/height. It deliberately lives in `zircon_runtime::graphics`; the neutral `CameraRenderDescriptor` stays a data carrier and does not know about graph cache policy.

`extract_compile_fingerprint(...)` reads the selected `CameraRenderDescriptor`, not the transitional `RenderViewExtract.camera` payload. This matches the descriptor-owned camera hard cutover and prevents a stale payload from producing a cache key that disagrees with the child submit descriptor.

## Current Boundaries

Texture target format is not yet part of the compile key because the current graph compiler does not receive the prepared output-target descriptor. Texture format is resolved later by the resource streamer for direct-import versus conversion-writeback execution. If a later compositor slice makes final target format a compile-time graph shape input, the target descriptor or a compact format class must be threaded into this key.

This cache slice does not choose Base/Overlay physical attachment ownership, final target ownership, present ownership, or per-camera history. It only prevents graph reuse across selected-camera target/viewport/render-type inputs already visible at compile time.

## Validation

The cache module has unit coverage for existing compile inputs and for camera target/stack inputs: primary versus texture, different texture ids, headless target, viewport rect presence, and Base versus Overlay. The current focused validation passed `cargo check` for `zircon_runtime` with `core-min` and passed the `compiled_graph_cache` lib-test filter.
