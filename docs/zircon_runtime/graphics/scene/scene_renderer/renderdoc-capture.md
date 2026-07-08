---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_runtime/src/graphics/backend/render_backend/graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
plan_sources:
  - user: 2026-07-08 D:\Tools\renderdoc 为 RenderDoc 地址
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
tests:
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_vulkan_20260708_capture.rdc
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_lifetime_20260708.png
doc_type: module-detail
---

# Scene Renderer RenderDoc Capture

## Purpose

`scene_renderer_render_capture.rs` exposes the scene renderer's graphics-debugger capture controls without making RenderDoc a runtime dependency. The implementation calls wgpu's `Device::start_graphics_debugger_capture()` and `Device::stop_graphics_debugger_capture()` through the existing `RenderBackend` owner. When RenderDoc is attached or launches the process, wgpu forwards the capture request; otherwise the calls are no-ops.

## Interactive Viewer Path

`zircon_shader_pbr_viewer` now accepts:

```powershell
--renderdoc-capture-once --exit-after-capture
```

The viewer still creates its window before loading the HDRI scene. Once the first real PBR/HDRI frame is ready to render, the app starts a graphics-debugger capture, renders one frame through `SceneRenderer`, stops the capture, presents the CPU frame, and exits when `--exit-after-capture` is set. This gives `renderdoccmd capture -w` a deterministic process lifetime and keeps manual orbit-camera behavior unchanged when the flags are not used.

## Validation

The 2026-07-08 Vulkan capture command was:

```powershell
$env:WGPU_BACKEND='vulkan'
D:\Tools\renderdoc\renderdoccmd.exe capture -w -c E:\Git\ZirconEngine\docs\tests\runtime\shader\zircon_shader_pbr_viewer_renderdoc_vulkan_20260708 -d E:\cargo-targets\zircon-shader-viewer-0708\debug E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --face-size 64 --renderdoc-capture-once --exit-after-capture
```

RenderDoc exited with code 0. The viewer logged `starting graphics debugger capture on wgpu(vulkan)` and `graphics debugger capture completed`, then exited. RenderDoc wrote `docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_vulkan_20260708_capture.rdc`, 6,557,527 bytes. The visual PNG evidence for the same viewer scene remains `docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_lifetime_20260708.png`.

The first DX12 attempt is intentionally recorded as an open backend portability signal, not as accepted capture evidence. With `WGPU_BACKEND=dx12`, the existing deferred-lighting shader failed FXC loop unrolling in `zircon-deferred-lighting-pipeline` under RenderDoc launch (`error X3511`). The Vulkan capture path is the accepted RenderDoc evidence for this slice.

## Constraints

RenderDoc thumbnail extraction reported that the capture did not contain an embedded thumbnail, so no thumbnail PNG was generated from the `.rdc`. Screenshots for visual verification remain normal viewer/window captures under `docs/tests/runtime/shader`; capture artifacts also stay under `docs/tests/runtime/shader` and are not written to `target`.
