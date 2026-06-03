---
related_code:
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
plan_sources:
  - user: 2026-06-02 implement ZirconEngine WGPU render main-chain closure plan
  - .codex/plans/ZirconEngine ECS 到渲染链路完善里程碑计划.md
  - .codex/plans/Zircon SRPRHI 渲染管线补全计划.md
tests:
  - zircon_runtime/src/core/framework/tests.rs::render_phase_sort_key_uses_unified_queue_layer_depth_order
  - zircon_runtime/src/core/framework/tests.rs::geometry_phase_inputs_feed_unified_sort_components_into_queue
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs::render_product_sprite_phase_queue_honors_material_queue_and_ui_z_index
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors
  - cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked unified_sort_components --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked render_product_sprite_phase_queue_honors --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Render Frame Extract

## Purpose

`RenderFrameExtract` is the neutral frame DTO submitted through `RenderFramework`. Scene and runtime producers fill it directly so graphics code can compile and execute render graph work without reading editor state or concrete world internals.

## View Size Contract

`RenderViewExtract` records an optional `target_size` alongside the camera snapshot. The size is derived from an explicit viewport rectangle or headless camera target when the extract is created, and `RenderFrameExtract::apply_viewport_size(...)` updates both the camera aspect ratio and the stored target size before submission.

`RenderViewExtract::effective_view_size()` is the canonical read path for SRP and RenderGraph descriptor derivation. It clamps through the camera viewport when present and falls back to `1 x 1` only when the extract does not yet know a surface or headless target size.

## Sort Key Contract

`RenderPhaseSortKey` now exposes `RenderPhaseSortComponents` as the shared ordering input for 3D, 2D, UI, overlay, and debug draw records. The packed order is render queue, material queue, order in layer, UI z-index, depth or reverse depth for transparent phases, then entity tie-breaker.

`GeometryPhaseInput`, `SpritePhaseExtractInput`, `MeshPhaseInput`, and `SpritePhaseInput` carry the same queue fields with defaulting constructors. Meshes use depth plus entity tie-breaker by default; sprites map `z_order` to order in layer and can now add material queue, render queue, depth bias, and UI z-index without changing the queue builder contract.

## Design And Rationale

The size belongs on the extract, not in the SRP asset, because the same pipeline asset can be used for multiple viewports, headless targets, editor previews, and camera stacks. The compiler therefore receives the product pipeline and per-frame view data separately and derives graph resource descriptors from both.

This is still neutral data. No WGPU surface, texture, or swapchain object is stored in the framework DTO.

## Test Coverage

The focused pipeline compile test verifies that a headless HDR camera with 4x MSAA produces `scene-color` and `scene-depth` graph lifetimes with the expected extent, format, and sample count. Broader scene extract and renderer execution validation remains part of the milestone testing stage.

Focused validation on 2026-06-02 passed for `pipeline_compile` with 42 tests, plus the two direct phase-order filters for mesh unified sort components and sprite material queue/UI z-index ordering. These runs used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain` and emitted only pre-existing warning classes outside this change.
