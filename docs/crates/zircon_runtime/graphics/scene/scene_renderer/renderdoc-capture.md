---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_runtime/src/graphics/backend/render_backend/graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
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
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_direct_after_depth_load_20260710.log
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_dx12_20260710.log
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_dx12_20260710_capture.rdc
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_interactive_startup_20260710.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_rebuild_live_20260710.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_dx12_rebuild_20260710_capture.rdc
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

The 2026-07-10 DX12 follow-up closed the backend portability failure at the two shared implicit-gradient sites instead of special-casing the viewer. Shadow PCF now uses `textureSampleCompareLevel(...)`, so calls from dynamic light loops do not require fragment derivatives. `load_scene_depth(...)` now uses integer `textureLoad(...)`, so SSR's data-dependent trace/refine loops do not force FXC to unroll an implicit-gradient depth sample. The assembled deferred and fallback shadow-source guards require the explicit-level spelling and reject the old implicit comparison call; the raw-depth post-process guard requires the integer load and rejects `textureSample(scene_depth_tex, ...)`.

After the successful 2026-07-10 viewer build, direct `WGPU_BACKEND=dx12` execution completed capture start/stop and exited 0. `renderdoccmd capture -w` also exited 0 and wrote `docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_dx12_20260710_capture.rdc`, 13,378,644 bytes, SHA256 `3C4332282E5D119BD113E3947BC27982897485C5CE3E48754721F17399EF6D3A`. A normal no-auto-exit DX12 launch became responsive with title `Zircon PBR HDRI Mirror Viewer`; its 1296x999 window capture is `docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_interactive_startup_20260710.png`, SHA256 `24BE09978519130A16495A52A6F8365C9FBD47E565014DEA828813A8FA465241`, and shows the Lakes HDRI skybox plus mirror-sphere reflection. A later documentation/comment-only rebuild attempt was blocked by an unrelated concurrent borrow-check error in `core/framework/animation/animation_target_id.rs`; rerunning the already-built DX12 executable still completed capture start/stop and exited 0, so that unrelated compile signal does not replace the accepted build evidence.

The current-worktree rebuild on 2026-07-10 also passed. `zircon_shader_pbr_viewer.exe` is 71,365,120 bytes with SHA256 `F465A7A394BD2F017A84BBD7F2B22F718FE09AC72CB29A8122AD7FFD8146A60E`. A normal DX12 process remained responsive with the expected window title, loaded a face-size 256 / 9-mip staged `.zcube + .zribl` environment, and produced `docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_rebuild_live_20260710.png` (1296x999, SHA256 `49EA7E954E50541891076C52B217CD05B6E7CFDEA2959D5B38A5EBC2548CD387`). A one-shot DX12 capture exited 0, wrote `zircon_shader_pbr_viewer_renderdoc_dx12_rebuild_20260710_capture.rdc` (13,374,191 bytes, SHA256 `D669EC915B42DB22F21AF1CC0C038A6D2E7FBC6063EB99B98BCCB14964CAB365`), and `renderdoccmd replay --loops 1` replayed it successfully.

## Constraints

RenderDoc thumbnail extraction reported that the capture did not contain an embedded thumbnail, so no thumbnail PNG was generated from the `.rdc`. Screenshots for visual verification remain normal viewer/window captures under `docs/tests/runtime/shader`; capture artifacts also stay under `docs/tests/runtime/shader` and are not written to `target`.
