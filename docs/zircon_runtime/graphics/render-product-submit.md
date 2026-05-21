---
related_code:
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/bevy/crates/bevy_render/src/view/window/mod.rs
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/internal.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/render_asset_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/erased_render_asset_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mesh_allocator_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_pbr/src/lib.rs
  - dev/bevy/crates/bevy_pbr/src/pbr_material.rs
  - dev/bevy/crates/bevy_pbr/src/material.rs
  - dev/bevy/crates/bevy_pbr/src/mesh_material.rs
  - dev/bevy/crates/bevy_pbr/src/material_bind_groups.rs
  - dev/bevy/crates/bevy_pbr/src/render/light.rs
  - dev/bevy/crates/bevy_pbr/src/render/pbr.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/pbr_lighting.wgsl
  - dev/bevy/crates/bevy_pbr/src/deferred/deferred_lighting.wgsl
  - dev/bevy/crates/bevy_pbr/src/cluster/cluster.wgsl
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/framework_error.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/config.rs
  - zircon_runtime/src/graphics/backend/render_backend/graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_new_offscreen.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_asset.rs
  - zircon_runtime/src/graphics/extract/history.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/runtime/history/validation_key.rs
  - zircon_runtime/src/graphics/runtime/history/is_compatible.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/graphics_debugger_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/environment.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/request_graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/query_graphics_debugger_status.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/submit_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/create_viewport/create.rs
  - zircon_runtime/src/graphics/runtime/render_framework/destroy_viewport/destroy_viewport.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_pipeline_asset/set_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_quality_profile/set_quality_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/viewport_generation_guard.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/generation.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/config.rs
  - zircon_runtime/src/graphics/backend/render_backend/graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_new_offscreen.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/extract/history.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/runtime/history/validation_key.rs
  - zircon_runtime/src/graphics/runtime/history/is_compatible.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/graphics_debugger_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/environment.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/request_graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/query_graphics_debugger_status.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/submit_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/create_viewport/create.rs
  - zircon_runtime/src/graphics/runtime/render_framework/destroy_viewport/destroy_viewport.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_pipeline_asset/set_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_quality_profile/set_quality_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/viewport_generation_guard.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/generation.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
plan_sources:
  - user: 2026-05-21 continue Bevy PBR material and lighting evidence mapping
  - user: 2026-05-21 continue Bevy presentation surface evidence mapping
  - user: 2026-05-21 continue Bevy render diagnostics evidence mapping
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
tests:
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_debugger_and_history.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::scene_uniform_uses_authored_ambient_light_when_lighting_is_enabled
  - zircon_runtime/src/core/framework/render/light/readiness.rs::light_status_counts_split_ready_and_degraded_slots
  - zircon_runtime/src/graphics/tests/render_product_submit.rs::render_product_pbr_submit_reports_material_fallback_and_light_stats
doc_type: module-detail
---

# Render Product Submit

M4A makes `RenderFrameExtract` the product submit authority. `ViewportRenderFrame::from_extract` keeps a legacy scene snapshot field for compatibility with older public-runtime paths, but renderer internals read camera, mesh, light, overlay, and preview data through accessors backed by `RenderFrameExtract`.

## Bevy Render Diagnostics Evidence

Bevy's render foundation is split between the render sub-app, optional pipelined rendering, and render diagnostics. `dev/bevy/crates/bevy_render/src/lib.rs:120-208` defines `RenderPlugin`, `RenderApp`, `RenderStartup`, and `RenderSystems` for extract, prepare-assets, queue, phase sort, prepare resources, render, cleanup, and post-cleanup. `dev/bevy/crates/bevy_render/src/pipelined_rendering.rs:68-105` moves rendering to a separate thread so frame `N` rendering can overlap frame `N + 1` simulation, with a `RenderExtractApp` handling sync/extract and channels moving the render sub-app between threads.

`dev/bevy/crates/bevy_render/src/diagnostic/mod.rs:37-63` defines `RenderDiagnosticsPlugin` as the owner for render CPU/GPU elapsed time per pass and pipeline statistics. The same docs route consumers through `DiagnosticsStore`, `LogDiagnosticsPlugin`, or Tracy, and state that GPU timestamp and pipeline-statistic support is limited to Vulkan and DX12 while other backends record CPU time only.

`diagnostic/mod.rs:66-94` wires diagnostics in two worlds: the main app stores a `RenderDiagnosticsMutex` and syncs diagnostics in `PreUpdate`, while the render app initializes `DiagnosticsRecorder` and adds begin, resolve, and finish systems to the render graph. `diagnostic/mod.rs:132-192` exposes the pass API through `RecordDiagnostics::time_span`, `pass_span`, `record_f32`, and `record_u32`; the guards require explicit `end(...)`, making missing span closure visible during development.

`dev/bevy/crates/bevy_render/src/diagnostic/internal.rs:23-29` fixes query budgets and buffer sizes for timestamps and pipeline statistics. `internal.rs:83-144` rotates current, submitted, and finished frame diagnostic buffers; `internal.rs:244-285` only creates timestamp and pipeline-statistics query sets when the backend supports them. This is a true GPU diagnostic path, not just a CPU-side render stats snapshot.

Bevy also treats render-asset residency as diagnostics. `render_asset_diagnostic_plugin.rs:31-42` registers a `render_asset/<type>` diagnostic and measures `RenderAssets<A>` during `ExtractSchedule`, then reports the count in `PreUpdate`. `erased_render_asset_diagnostic_plugin.rs:35-46` does the same for erased render assets, while `mesh_allocator_diagnostic_plugin.rs:36-52` registers mesh allocator slab count, slab byte size, and allocation count.

## Zircon Diagnostic State

Zircon currently records render health through submit-owned `RenderStats`, not through a Bevy-style GPU timing recorder. `RenderStats` in `zircon_runtime/src/core/framework/render/backend_types.rs:479-615` carries submitted-frame generation, effective feature names, planned and executed render graph pass/resource/dependency counts, transient graph allocation slots, post-process graph nodes, anti-alias fallback, advanced provider reports, Solari status, queue fallback, async compute pass counts, UI command/payload/order stats, material/sprite readiness, light ready/degraded splits, and VG/HGI runtime counters.

`update_base_stats(...)` in `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs:10-126` writes the baseline stats after a successful submit from the frame submission context, compiled pipeline, renderer execution record, post-process graph, anti-alias fallback, advanced/Solari reports, UI stats, material/sprite renderer stats, and shared `RenderLightReadinessReport`. `query_stats(...)` in `render_framework/query_stats/query_stats.rs:5-8` returns a clone of this stats struct to runtime diagnostics and editor/tooling consumers.

The existing debug marker and RenderDoc path is adjacent but not equivalent to Bevy render diagnostics. `debug_markers.rs` and the compiled-scene renderer label GPU events for external capture; they do not yet feed CPU/GPU pass durations, pipeline statistics, or render-asset diagnostics into `DiagnosticStore`.

## Bevy Gap Classification

| Bevy diagnostics area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Render sub-app diagnostics lifecycle | Zircon has a submit-time `RenderStats` snapshot and `query_stats(...)`; it does not have a separate render app, render graph begin/resolve/finish diagnostics systems, or main/render-world diagnostic mutex. | Add a diagnostics bridge that keeps render submit stats and any future GPU timing data consumable from runtime `DiagnosticStore` without exposing renderer-private state. |
| CPU/GPU pass timing | RenderDoc markers and graph execution records identify passes, but no CPU elapsed-time, GPU timestamp-query, or pipeline-statistics measurements are recorded per pass. | Add per-pass timing/span recording with backend capability fallback: GPU stats on supported backends, CPU-only on unsupported ones. |
| Pipeline statistics | `RenderStats` reports graph counts and runtime product counters, not shader invocations, primitive counts, or pipeline-statistics query results. | Add a backend-gated pipeline-statistics recorder before claiming Bevy `RenderDiagnosticsPlugin` parity. |
| Render asset diagnostics | Material, sprite, texture fallback, VG, and HGI counters are product-specific stats, not a generic `RenderAsset` diagnostic plugin family. | Add typed render-asset residency/count diagnostics and erased-asset count diagnostics if render asset storage becomes generic enough to support it. |
| Mesh allocator diagnostics | Current docs track mesh/material readiness and graph execution; there is no Bevy-style mesh allocator slab/byte/allocation diagnostic. | Add allocator-level mesh memory diagnostics once Zircon's mesh allocator has stable slab/residency ownership. |
| Pipelined rendering visibility | Zircon's runtime framework submit path is synchronous from the caller's perspective; no Bevy-like render thread/sub-app overlap diagnostics exist. | Keep this as a future scheduling milestone; do not conflate current submit stats with pipelined rendering telemetry. |

## Bevy Presentation Surface Evidence

Bevy keeps camera targets and window presentation explicit instead of treating all rendered output as one swapchain path. `dev/bevy/crates/bevy_camera/src/camera.rs:22-58` defines `Viewport` as a physical rectangle inside a render target and clamps it to the target size. `camera.rs:814-855` defines `RenderTarget::{Window, Image, TextureView, None}` and normalizes them to concrete target keys; the `None { size }` target represents a camera with no color target, useful for prepass-only rendering. `dev/bevy/crates/bevy_render/src/camera.rs:263-300` resolves the normalized target into `RenderTargetInfo` from a window, image, manual texture view, or explicit no-color size, and reports missing window/image/texture-view targets as structured errors at `camera.rs:322-331`.

Window surfaces are render-app resources in Bevy. `dev/bevy/crates/bevy_render/src/view/window/mod.rs:31-45` registers `ExtractedWindows`, `WindowSurfaces`, `extract_windows`, `create_surfaces`, and `prepare_windows`. `mod.rs:49-99` stores the extracted window size, present mode, swapchain texture view, and `SurfaceTexture`, then presents by taking the current surface texture and calling `present()`. `mod.rs:358-458` creates/configures raw-handle WGPU surfaces, chooses a surface format, updates size and present mode, and reconfigures the surface on resize or present-mode changes. `mod.rs:465-508` implements present-mode fallback, ending in FIFO-compatible choices when a requested mode is not advertised.

Bevy screenshots are also target-aware. `dev/bevy/crates/bevy_render/src/view/window/screenshot.rs:49-111` models screenshot requests as a `Screenshot(RenderTarget)` component and returns a `ScreenshotCaptured` image asynchronously. `screenshot.rs:406-439` wires the screenshot plugin into the main app and render app, including `extract_screenshots` and `prepare_screenshots` before view targets are prepared. `screenshot.rs:596-603` copies the prepared screenshot texture into a readback buffer, and `screenshot.rs:647-682` maps the buffer asynchronously, strips row padding, and sends a CPU `Image` result.

## Zircon Presentation State

Zircon's camera target vocabulary is intentionally narrower today. `zircon_runtime/src/core/framework/render/camera.rs:84-94` defines `RenderCameraTarget::{PrimarySurface, Texture, Headless}` with `PrimarySurface` as the default. `submit_frame_extract/build_frame_submission_context/target_resolution.rs:4-38` resolves `PrimarySurface` to the viewport record size, resolves `Headless { size }` to a clamped offscreen size, rejects `Texture(_)` with `UnsupportedCapability("camera texture render target")`, and rejects `Headless` on the surface-present path with `UnsupportedCapability("headless camera surface present")`.

The runtime framework separates offscreen submit/capture from native surface present. `present_frame_extract.rs:24-104` builds the normal submission context, rejects non-primary camera targets for surface present, requires a bound viewport surface, prepares runtime submission, builds the runtime frame, leases the bound surface, and calls `renderer.present_frame_with_pipeline(...)`. The backend surface path in `render_backend/viewport_surface.rs:55-80` acquires the current WGPU surface texture, blits the rendered offscreen source into it, calls `surface_texture.present()`, and treats outdated/lost/timeout/occluded surfaces as nonfatal or reconfigurable. `viewport_surface.rs:270-330` configures the surface with clamped size, an SRGB-preferred format, `AutoVsync` or FIFO-biased present mode, and fixed frame latency.

The focused surface-target tests document this product boundary. `zircon_runtime/src/graphics/tests/surface_targets.rs:111-152` proves offscreen submit/capture survives surface unbind and that `Headless { size }` controls captured frame size. `surface_targets.rs:155-193` proves texture targets and headless surface-present requests return explicit unsupported-capability errors instead of silently falling back to primary output. `surface_targets.rs:196-206` guards the surface present implementation against regressing into readback-based present fallback.

## Presentation Gap Classification

| Bevy presentation area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Window surface lifecycle | Zircon has viewport records, raw Win32 surface binding, WGPU surface configuration, present blit, and explicit missing-surface errors. It does not yet have Bevy's render-app `ExtractedWindows` / `WindowSurfaces` resource lifecycle or broad platform surface owner model inside the render plan. | Keep native-window ownership in `zircon_app`/platform, but expose enough render-side diagnostics for bound/unbound/resized/present-mode state without mixing platform input work into the renderer. |
| Image and texture render targets | Bevy supports `RenderTarget::Image` and manual `TextureView`; Zircon has a neutral `Texture(handle)` target but currently rejects it before submit. | Land GPU texture residency/writeback and render-to-texture scheduling before claiming Bevy image/texture-view target parity. |
| No-color / headless target semantics | Bevy `RenderTarget::None { size }` is no-color, useful for prepass-only rendering. Zircon `Headless { size }` renders to an offscreen color target that can be captured. | Record the intentional divergence. Add a true no-color/depth-only target only if depth-prepass or shadow-only camera workflows need it. |
| Screenshot and capture workflow | Bevy has an async screenshot component, per-target preparation, GPU copy-to-buffer, row-padding cleanup, and image callback. Zircon exposes viewport `capture_frame(...)` and RenderDoc capture hooks, but no Bevy-like screenshot request/result pipeline. | Add a screenshot/capture request API that can target primary, headless, and future texture targets, return structured async results, and integrate with dev/CI artifact paths. |
| Present-mode and surface diagnostics | Zircon chooses an advertised present mode internally and records present failures as framework errors; Bevy exposes present-mode fallback decisions in the window render path. | Add surface-format/present-mode/fallback diagnostics to `RenderStats` or `DiagnosticStore` once the platform/window session's surface lifecycle stabilizes. |

## Bevy PBR Material And Lighting Evidence

Bevy's PBR baseline is a product family, not just a shader. `dev/bevy/crates/bevy_pbr/src/lib.rs:130-156` defines `PbrPlugin` with prepass, deferred lighting, GPU instance buffer building, and glTF StandardMaterial defaults. The plugin loads the PBR shader library set at `lib.rs:179-198`, registers `StandardMaterial` and `MaterialPlugin::<StandardMaterial>` at `lib.rs:203-216`, adds SSAO, fog, lightmap, light probes, volumetric fog, SSR, transmission, clustered decals, and contact shadows at `lib.rs:217-230`, syncs directional/point/spot/rect/ambient lights at `lib.rs:232-239`, adds atmosphere and GPU clustering at `lib.rs:240-244`, and conditionally adds deferred PBR lighting at `lib.rs:251-252`.

Bevy's `StandardMaterial` carries a broad authored and GPU-facing contract. `dev/bevy/crates/bevy_pbr/src/pbr_material.rs:26-57` starts the material with base color, UV channel, and texture dependency bindings, while later fields cover emissive, metallic/roughness, normal/occlusion, alpha, transmission, clearcoat, anisotropy, and parallax families. The shader-visible flags at `pbr_material.rs:967-1003` distinguish texture slots, double-sided, unlit, normal-map options, fog, parallax, transmission, clearcoat, anisotropy, specular, and alpha modes. `pbr_material.rs:1010-1056` packs the GPU uniform with base color, emissive, roughness, metallic, transmission, thickness, IOR, clearcoat, anisotropy, flags, alpha cutoff, and parallax parameters.

The Bevy material pipeline is asset-driven and render-phase aware. `dev/bevy/crates/bevy_pbr/src/material.rs:74-144` defines `Material` as an `Asset + AsBindGroup` abstraction used with `Mesh3d` and `MeshMaterial3d`; `material.rs:289-342` initializes specialized material pipeline caches, material instances, bind group allocators, draw commands for shadow/transparent/opaque/alpha-mask phases, material mesh specialization, queueing, bind group preparation, shadow specialization, and shadow queueing. `mesh_material.rs:8-41` makes the mesh-to-material handle component explicit, while `material_bind_groups.rs:36-115` shows the bindless/non-bindless allocator and slab resource tracking surface that backs material bind groups.

The shader side confirms why PBR parity must include both forward and deferred lighting. `dev/bevy/crates/bevy_pbr/src/render/pbr.wgsl:65-89` creates `PbrInput` from StandardMaterial bindings, handles alpha discard, writes deferred/prepass output when configured, or applies PBR lighting plus in-shader post-lighting in forward mode. `deferred/deferred_lighting.wgsl:59-86` reconstructs `PbrInput` from the deferred G-buffer, folds in SSAO when available, and applies the same PBR lighting path. `render/pbr_lighting.wgsl:34-122` defines the BRDF and lighting input model for point, spot, and directional lights, including clearcoat and anisotropy variants. Cluster data is its own GPU surface: `cluster/cluster.wgsl:4-24` defines point/spot/probe/decal clusterable object kinds and cluster metadata, and `render/light.rs:1316-1343` writes point/spot lights into `GpuClusteredLight`; `render/light.rs:1519-1624` builds per-view `GpuLights` with ambient, directional, clusters, and rect-light storage.

## Zircon PBR Material And Lighting State

Zircon's current baseline intentionally lands below Bevy's full PBR plugin breadth. The neutral material descriptor in `zircon_runtime/src/core/framework/render/material/standard_material.rs:8-23` carries name, dependency set, base color, base-color/normal/metallic-roughness/occlusion/emissive textures, metallic, roughness, emissive, alpha mode, unlit, double-sided, and fallback policy. `render/material/readiness_report.rs:31-58` records validation errors and fallback usage so runtime stats can distinguish usable material rows from fallback-dependent rows.

Concrete GPU preparation currently projects that descriptor into a smaller runtime material. `resource_streamer_ensure_material.rs:18-84` loads the material, optional shader contract, and readiness report; `resource_streamer_ensure_material.rs:89-119` resolves standard texture slots; `resource_streamer_ensure_material.rs:150-195` merges shader readiness, handles blocking validation, and constructs `MaterialRuntime`. `material_runtime.rs:27-42` stores the runtime scalar/texture fields, `PipelineKey`, and readiness report; `pipeline_key.rs:4-17` keys shader revision, double-sided, alpha blend/mask/cutoff, unlit, and standard texture-slot presence.

Zircon has real renderer hooks, but they are still narrower than Bevy's PBR path. `deferred_scene_resources/record_gbuffer_geometry.rs:5-45` records the deferred geometry pass by binding scene/model/texture/geometry data for each mesh draw. `deferred_scene_resources/execute_lighting.rs:4-52` runs a fullscreen deferred lighting pass over albedo, normal, background, and scene-color targets. `execute_clustered_lighting.rs:14-87` writes directional-light data into a fixed clustered-light buffer and dispatches a compute culling pass, but it currently operates on `RenderDirectionalLightSnapshot` only; Bevy-style point/spot clustered light shading, rect area-light shading, shadow maps, probes, transmission, and advanced material lobes remain outside the accepted baseline.

## PBR Material And Lighting Gap Classification

| Bevy PBR area | Zircon product state | Completion requirement |
| --- | --- | --- |
| StandardMaterial surface | Zircon covers the core base-color, normal, metallic/roughness, occlusion, emissive, alpha, unlit, and double-sided descriptor surface. | Add the missing Bevy StandardMaterial families deliberately: reflectance/specular, transmission/thickness/IOR/attenuation, clearcoat, anisotropy, parallax/depth maps, UV transforms/channels, lightmap interaction, and debug/shader-def controls. |
| Material bind groups | Zircon prepares runtime textures and pipeline keys, but material binding is not Bevy's `AsBindGroup`/bindless allocator model. | Land explicit bind-group layout reflection, slot validation, fallback resource residency, material cache invalidation, and bindless/non-bindless policy before claiming Bevy-like material plugin parity. |
| Phase-specialized material pipeline | Zircon has Core3d phases plus forward/deferred pipeline assets and alpha-derived queues. | Add material-specialized pipeline cache states, per-material shader defs, shadow/deferred/prepass variants, OIT/transmission phases, and structured pipeline-error diagnostics. |
| Physically based lighting | Zircon consumes authored ambient light and one basic directional slot, plus a limited directional clustered-light compute path. | Implement point/spot clustered lighting, rect/area lights, shadows, contact shadows, lightmaps, probes/IBL, SSAO/SSR coupling, clearcoat/anisotropy/transmission lighting, and per-view light visibility before marking full PBR lighting complete. |
| Deferred parity | Zircon records a G-buffer geometry pass and fullscreen lighting pass. | Align the G-buffer contract with material flags, normal/motion/depth prepasses, deferred lighting pass IDs, SSAO/specular occlusion, and fallback handling for unlit and unsupported material modes. |
| Authoring and assets | Zircon has runtime descriptors and asset-side material files, but this docs slice does not enter `.zmaterial` or material editor implementation. | Sequence `.zmaterial`, material editor projection, shader-contract authoring, and asset hot-reload with the active asset/material lane rather than folding it into render submit docs. |

The basic forward and deferred mesh shaders share `SceneUniform`. When preview lighting is enabled, `SceneUniform::from_frame(...)` now reads active authored ambient lights from `RenderFrameExtract::lighting.ambient_lights`, accumulates `color * intensity`, and writes that value to `ambient_color`. If no ambient light is authored, the renderer keeps the existing preview fallback ambient value. This closes the first concrete ambient-light consumption step without changing `.zshader` / `.zmaterial` ownership or adding a new material pipeline.

Render submit stats now split light slots by renderer readiness as well as by total count. `RenderLightReadinessReport` in `render::light` owns the rule: `last_ambient_light_ready_count` / `last_ambient_light_degraded_count` report whether authored ambient slots are usable by the current `SceneUniform` path; `last_directional_light_ready_count` is capped to the single directional slot currently consumed by the basic `SceneUniform` path; point/spot ready counts remain zero because those lights are extracted but not shaded by the current default PBR path; rect lights remain degraded until the PBR/area-light shader path lands. This mirrors the Bevy distinction between ambient/direct/clusterable/rect light GPU representations without letting the advanced lighting gap hide inside a single total count.

Core pipeline selection is neutral framework data. Cameras select `CorePipelineKind::Core2d` for orthographic projections and `CorePipelineKind::Core3d` for perspective projections. Unset viewport submit uses that extract-owned pipeline kind to choose the built-in Core2d or Forward+ Core3d pipeline, while explicit viewport pipelines and quality-profile overrides remain authoritative and are rejected at compile time if their `core_pipeline` does not match the submitted extract.

`HistoryResolve` is no longer part of the default effective pipeline. `RenderFeatureQualitySettings::default()` leaves `history_resolve` disabled, `BuiltinRenderFeature::HistoryResolve` requires explicit opt-in, and profile compilation enables it only when a profile calls `with_history_resolve(true)`. This keeps default Core3d rendering free of scene-color temporal blending until motion vectors, reprojection, camera-cut detection, and disocclusion checks exist.

`GeometryExtract` carries phase queues derived from material alpha mode plus the selected pipeline. Production world extraction reads the alpha hint stored on each `MeshRenderer` and creates phase inputs from the sorted mesh rows, so mesh draw construction can consume aligned opaque, alpha-mask, and transparent queues instead of falling back to raw mesh-vector order.

Pipeline compile validates that declared renderer stages with product phases have matching `RenderPipelineAsset.phase_mapping` entries. The enforced stage-to-phase mapping covers 2D mesh stages, 3D mesh stages, depth prepass, shadow, deferred, postprocess, UI, overlay, and debug; lighting and ambient occlusion remain product-phase-neutral until a dedicated phase exists. Runtime graph execution now calls declared graph stages through `execute_graph_stage` while retaining the concrete post-process stack, overlay, and screen-space UI renderer calls. The post-process graph recorder writes its pass nodes into the same `RenderGraphExecutionRecord` via a short mutable reborrow of `RenderGraphStageExecution::record`, so the compiled-scene path keeps one execution-record owner for both staged graph passes and generated post-process graph nodes.

Submit safety is guarded by viewport generations. Context building captures the viewport record generation while resolving size, effective pipeline, quality profile, and history state. Before runtime prepare mutates viewport runtime state, and again before recording the rendered frame back into the viewport, submit revalidates that the viewport still exists and that its generation matches. Missing viewports return `RenderFrameworkError::UnknownViewport`; changed viewports return `RenderFrameworkError::ViewportChanged` instead of relying on checked-then-`expect` panics.

Frame history reuse now includes an extract validation key. `build_frame_submission_context(...)` records world id, camera snapshot, mesh identity/transform/model/material/tint/mobility/layer mask, lighting extract, animation pose extract, post-process settings, particle extract, and the compiled effective feature names. `resolve_history_handle(...)` reuses the previous `FrameHistoryHandle` only when size, pipeline, history bindings, and this validation key all match. Camera motion, mesh motion, material/tint/layer changes, light changes, pose changes, bloom/color-grading/preview changes, particle changes, world changes, and feature toggles therefore allocate a new history handle before renderer history textures are reused. Renderer history copy also preserves slot semantics: `FrameHistorySlot::SceneColor` is copied from `OffscreenTarget.scene_color`, not from post-processed `final_color`, so bloom/color grading and later overlay/UI composition do not feed back as scene-color history.

Graphics debugger capture is a submit-scoped request, not a persistent rendering mode. The only live triggers are `RenderFramework::request_graphics_debugger_capture(viewport)` and `ZR_RENDERDOC_CAPTURE_NEXT=1`; editor UI and dynamic API commands do not currently expose a separate RenderDoc button. The trait method stores a pending viewport in `WgpuRenderFramework`; non-matching viewport submits leave it pending. The environment variable arms the first viewport created by the framework so desktop debug launches can capture the first rendered frame without editor code calling the trait method. On the matching submit, capture begins before runtime prepare/render command recording and finishes after the frame is produced. The blocking wgpu stop/poll step runs after the framework state mutex is released, while an operation lock remains held so no second frame or viewport/pipeline mutation can enter the active capture window. Destroying a viewport with pending or queued debugger capture clears that debugger state and records a destroyed-viewport error instead of leaving `capture_pending` true forever. The status query reports wgpu capture-hook availability, the selected wgpu backend as `wgpu(dx12)` / `wgpu(vulkan)` / equivalent, pending/active flags, the last captured frame generation, and any submit or stop error. `available` means the backend exposes the wgpu debugger capture hook; it does not prove RenderDoc is attached. If the matching submit fails during preflight before capture starts, the pending request is consumed and `last_error` records the preflight error. If a submit fails while capture is active, cleanup still stops the capture and clears active/pending state before returning the original error.

RenderDoc-readable markers are centralized in `zircon_runtime/src/graphics/debug_markers.rs`. The compiled-scene command encoder emits markers for `FrameExtract`, `Clear`, `Prepass`, `MainScene`, `Lighting`, `DeferredLighting`, `PostProcess`, `HistoryCopy`, `Overlay`, and `UI`; the readback path emits `Readback` before the GPU-to-CPU copy. Graph-stage execution maps `RenderPassStage::Lighting` to the generic `zircon::Lighting` marker so Forward+ lighting stages are not mislabeled as deferred, while the fixed deferred lighting pass still uses `zircon::DeferredLighting`.

For Windows RenderDoc capture, launch Zircon from RenderDoc with environment variables set before process start: use `WGPU_BACKEND=dx12` for Direct3D 12 or `WGPU_BACKEND=vulkan` for Vulkan, set `WGPU_DEBUG=1` and `WGPU_VALIDATION=1` when validation output is needed, and set `ZR_RENDERDOC_CAPTURE_NEXT=1` to capture the first created viewport's next submit. After capture, inspect the event browser for `zircon::FrameExtract`, `zircon::MainScene`, `zircon::Lighting`, `zircon::PostProcess`, `zircon::HistoryCopy`, and `zircon::UI`; history textures are labeled `zircon-history-scene-color`, `zircon-history-global-illumination`, and `zircon-history-ambient-occlusion`. The CPU fallback path still marks final readback as `zircon::Readback`, while the app-host window path now binds a native surface and finishes redraw through a wgpu swapchain `SurfaceTexture::present()` after a `zircon-present-blit-pass`. Keep `HistoryResolve` explicitly disabled unless the test scenario intentionally opts into temporal scene-color blending.

Validation coverage lives in `render_product_pipeline`, `render_product_submit`, `pipeline_compile`, `project_render`, `render_framework_bridge`, and `render_debugger_and_history` tests. The submit test intentionally verifies that direct extract frames can diverge from the legacy scene snapshot, proving product rendering must not use `to_scene_snapshot()` as the draw authority. The render debugger/history tests cover idle debugger status, exact DX12/Vulkan backend status under `WGPU_BACKEND`, backend env parsing, first-created-viewport capture arming, marker registry coverage, matching-viewport request consumption, unknown viewport rejection, destroyed pending-capture cleanup, history validation-key invalidation, and explicit history-resolve opt-in. Manual `.rdc` acceptance remains a desktop RenderDoc step because this automated gate cannot launch the GUI capture workflow.
