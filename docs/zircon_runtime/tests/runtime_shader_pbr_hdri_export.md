---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/camera.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/presenter.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/project_assets.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/scene_fixtures.rs
implementation_files:
  - zircon_app/Cargo.toml
  - zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/camera.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/presenter.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/project_assets.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/scene_fixtures.rs
plan_sources:
  - user: 2026-07-08 需要视角上下左右分别旋转120度验证
  - user: 2026-07-08 最好帮我编译一个鼠标可以控制镜头视角的程序让我手动去操纵验证
  - user: 2026-07-07 请你也进行一下多视角验证
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_cardinal_120deg_reflection_20260708.png
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_multi_view_reflection_20260707.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_fixed_20260708.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_lifetime_20260708.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_vulkan_20260708_capture.rdc
doc_type: testing-guide
---

# Runtime Shader PBR HDRI Export Tests

## Purpose

`runtime_shader_pbr_hdri_export.rs` is the focused visual-validation harness for real-HDRI PBR environment lighting. It creates temporary projects, renders standard PBR material spheres under the Poly Haven Lakes HDRI source cubemap, exports accepted screenshots into `docs/tests/runtime/shader`, and then runs saved-PNG assertions against those screenshots.

The harness is intentionally separate from production rendering modules. It exercises the real asset path through `ProjectManager`, `SceneRenderer`, `EnvironmentExtract::source_cubemap(...)`, standard PBR material assets, and generated scene fixtures.

## Multi-View Mirror Validation

The 2026-07-07 multi-view slice adds `SinglePbrSphereCameraView` to `scene_fixtures.rs`. The fixture writes camera eye, target, projection mode, and orthographic size into the generated scene by using `Transform::looking_at(...)`; all views target the same mirror sphere center so the screenshot tests isolate camera direction and reflection response instead of changing material or environment inputs.

`export_runtime_shader_pbr_real_hdri_mirror_multi_view_png` renders four 800x600 tiles:

- orthographic front
- perspective front
- perspective left yaw
- perspective right yaw

The tiles are written as a 1600x1200 contact sheet at `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_multi_view_reflection_20260707.png`.

`runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_orientation_metrics` reads the saved contact sheet, extracts each tile, and applies the existing mirror checks to every view. Those checks require visible real-HDRI upper/lower sky contrast, non-flat mirror reflection detail, correct upper/lower reflection orientation, bounded clipping, and balanced left/right grazing response.

`runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_source_reference_metrics` reads the same contact sheet and compares every tile against a source HDRI reference reconstructed from the matching `SinglePbrSphereCameraView`. The reference path uses the same `Transform::looking_at(...)` basis convention as the generated scene: forward is `target - eye`, right is `forward x world_up`, and up is `right x forward`. This keeps yawed perspective views from accidentally reusing the accepted front-camera reference.

The legacy single-image source-reference test remains separate. It continues to validate the already accepted front orthographic and front perspective screenshots with the historical front-camera projection, while the multi-view contact sheet uses the camera-view-aware reconstruction.

## 120-Degree Cardinal Validation

The 2026-07-08 cardinal slice extends `SinglePbrSphereCameraView` with `perspective_orbit_degrees(...)`. The helper orbits the camera around `SINGLE_PBR_SPHERE_CENTER` while keeping a constant radius, so the four views change only the eye direction and not the mirror material, environment, or target.

`export_runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png` renders four 800x600 tiles:

- up orbit, pitch +120 degrees
- down orbit, pitch -120 degrees
- left orbit, yaw -120 degrees
- right orbit, yaw +120 degrees

The tiles are written as a 1600x1200 contact sheet at `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_cardinal_120deg_reflection_20260708.png`.

`runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_environment_metrics` checks that each tile preserves real HDRI sky/ground contrast, non-flat mirror reflection detail, bounded clipping, and expected orientation. `runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_source_reference_metrics` then compares every tile against a camera-view-aware source HDRI reference, using the same orbit camera descriptors as the export.

## Interactive Viewer

`zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs` builds a small manual inspection program for the same source-cubemap PBR path. It loads `docs/tests/runtime/shader/assets/polyhaven_lakes_2k.hdr` by default, generates a temporary project containing one perfect-metal mirror PBR sphere and an HDRI environment, renders through `SceneRenderer`, and presents the result through a `winit`/`softbuffer` window.

The viewer is split by owner instead of living in one binary file: `app.rs` owns winit event handling, `camera.rs` owns orbit camera math and render camera descriptors, `presenter.rs` owns the softbuffer CPU-frame presenter, `scene.rs` owns temporary project loading and `SceneRenderer` calls, `project_assets.rs` owns generated scene/model/material assets, `hdri.rs` owns HDR equirectangular sampling and source-cubemap environment construction, and `args.rs` owns command-line parsing.

The 2026-07-08 startup fix creates the window and presents a startup frame before HDRI loading and temporary project rendering begin. This avoids the previous double-click failure mode where the process was alive but no top-level window appeared while the HDRI cubemap/PMREM path was still loading. The default source-cubemap face size is capped to 256 for interactive startup, with `--face-size 64|128|256|512` available for manual quality/performance checks.

For RenderDoc evidence, the viewer also supports `--renderdoc-capture-once --exit-after-capture`. The app starts wgpu graphics-debugger capture around the first real `SceneRenderer` PBR/HDRI frame, stops capture after the frame render completes, presents the frame, then exits when requested. This is a development/debug path; normal manual orbit viewing does not trigger capture.

Usage:

```powershell
E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe
E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --hdri E:\Git\ZirconEngine\docs\tests\runtime\shader\assets\polyhaven_lakes_2k.hdr
E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --face-size 512
E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --face-size 64 --renderdoc-capture-once --exit-after-capture
```

Left mouse drag orbits the camera. The mouse wheel zooms. The viewer intentionally uses the same `EnvironmentExtract::source_cubemap(...)`, `build_source_cubemap_from_equirect(...)`, `build_source_cubemap_irradiance_cube(...)`, standard PBR material, and camera descriptor path as the saved-PNG harness, so manual inspection exercises the same runtime reflection route rather than a separate debug renderer.

## Test Coverage

Fresh validation for the 2026-07-08 cardinal/viewer slice and owner-split viewer rebuild:

- `rustfmt --edition 2021 zircon_runtime\tests\runtime_shader_pbr_hdri_export.rs zircon_runtime\tests\runtime_shader_pbr_hdri_export\scene_fixtures.rs zircon_app\src\bin\zircon_shader_pbr_viewer\main.rs zircon_app\src\bin\zircon_shader_pbr_viewer\app.rs zircon_app\src\bin\zircon_shader_pbr_viewer\args.rs zircon_app\src\bin\zircon_shader_pbr_viewer\camera.rs zircon_app\src\bin\zircon_shader_pbr_viewer\hdri.rs zircon_app\src\bin\zircon_shader_pbr_viewer\presenter.rs zircon_app\src\bin\zircon_shader_pbr_viewer\project_assets.rs zircon_app\src\bin\zircon_shader_pbr_viewer\scene.rs`
- `rustfmt --edition 2021 --check zircon_app\src\bin\zircon_shader_pbr_viewer\main.rs zircon_app\src\bin\zircon_shader_pbr_viewer\app.rs zircon_app\src\bin\zircon_shader_pbr_viewer\args.rs zircon_app\src\bin\zircon_shader_pbr_viewer\camera.rs zircon_app\src\bin\zircon_shader_pbr_viewer\hdri.rs zircon_app\src\bin\zircon_shader_pbr_viewer\presenter.rs zircon_app\src\bin\zircon_shader_pbr_viewer\project_assets.rs zircon_app\src\bin\zircon_shader_pbr_viewer\scene.rs`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-shader-viewer-0708 CARGO_INCREMENTAL=0 cargo check -p zircon_app --bin zircon_shader_pbr_viewer --no-default-features --features target-client --locked --jobs 1 --color never`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-shader-viewer-0708 CARGO_INCREMENTAL=0 cargo build -p zircon_app --bin zircon_shader_pbr_viewer --no-default-features --features target-client --locked --jobs 1 --color never`
- `E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --help`
- `$env:WGPU_BACKEND='vulkan'; D:\Tools\renderdoc\renderdoccmd.exe capture -w -c E:\Git\ZirconEngine\docs\tests\runtime\shader\zircon_shader_pbr_viewer_renderdoc_vulkan_20260708 -d E:\cargo-targets\zircon-shader-viewer-0708\debug E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --face-size 64 --renderdoc-capture-once --exit-after-capture`
- `Start-Process -FilePath E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe -WorkingDirectory E:\cargo-targets\zircon-shader-viewer-0708\debug -ArgumentList @('--face-size','256') -RedirectStandardOutput docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_fixed_20260708.stdout.log -RedirectStandardError docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_fixed_20260708.stderr.log -PassThru`
- `Start-Process -FilePath E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe -WorkingDirectory E:\cargo-targets\zircon-shader-viewer-0708\debug -ArgumentList @('--face-size','256') -RedirectStandardOutput docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_lifetime_20260708.stdout.log -RedirectStandardError docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_lifetime_20260708.stderr.log -PassThru`
- `Start-Process -FilePath E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe -ArgumentList @('--hdri','E:\Git\ZirconEngine\docs\tests\runtime\shader\assets\polyhaven_lakes_2k.hdr') -WorkingDirectory E:\Git\ZirconEngine -PassThru`
- `E:\cargo-targets\zircon-shader-120deg-0708\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe export_runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png --ignored --exact --nocapture --test-threads=1`
- `E:\cargo-targets\zircon-shader-120deg-0708\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_environment_metrics --exact --nocapture --test-threads=1`
- `E:\cargo-targets\zircon-shader-120deg-0708\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_source_reference_metrics --exact --nocapture --test-threads=1`

The 120-degree cardinal PNG is 1600x1200, 2,099,569 bytes, SHA256 `0E57C496E49B6044F9AA5495BBFE855D5EB575A8D5AFAA8CB55B3DFF1D18AF15`. The same file name was scanned under the repository `target`, `E:\cargo-targets\zircon-shader-120deg-0708`, and `E:\cargo-targets\zircon-cmft-skybox-0707`; no target-copy was found. The owner-split viewer executable was recompiled on 2026-07-08 at `E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe`; `--help` confirms the default Poly Haven Lakes 2K HDRI path, default face size 256, optional `--face-size`, optional RenderDoc capture flags, left-drag orbit, and wheel zoom controls. Manual launch from the generated executable directory with `--face-size 256` created process id `26996`; the 80-second lifetime monitor recorded a top-level window by 3s, `Responding=True` by 40s, and the same `Zircon PBR HDRI Mirror Viewer` window still alive and responding at 80s with stdout/stderr logs empty. Startup screenshots `docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_fixed_20260708.png` and `docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_lifetime_20260708.png` are both 1296x999 and show the HDRI skybox plus mirror-sphere reflection; the lifetime screenshot was captured from the still-running process id `26996`. Vulkan RenderDoc capture through `D:\Tools\renderdoc\renderdoccmd.exe` wrote `docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_vulkan_20260708_capture.rdc`, 6,557,527 bytes. The first DX12 RenderDoc attempt did not produce accepted evidence because the existing deferred-lighting shader hit an FXC loop-unroll validation failure under `zircon-deferred-lighting-pipeline`; this remains a backend portability follow-up. The split viewer modules stay below the structure-convention ceiling; the current viewer production source stays free of `allow(...)`, `unwrap(...)`, `expect(...)`, and `panic!`.

Earlier validation for the 2026-07-07 multi-view slice:

- `rustfmt --edition 2021 zircon_runtime\tests\runtime_shader_pbr_hdri_export.rs zircon_runtime\tests\runtime_shader_pbr_hdri_export\frame_assertions.rs`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-cmft-skybox-0707 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export export_runtime_shader_pbr_real_hdri_mirror_multi_view_png --no-default-features --features core-min --locked --jobs 1 --color never -- --ignored --exact --nocapture --test-threads=1`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-cmft-skybox-0707 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_orientation_metrics --no-default-features --features core-min --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-cmft-skybox-0707 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export runtime_shader_pbr_real_hdri_mirror_reflection_png_matches_source_reference_metrics --no-default-features --features core-min --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1`
- `E:\cargo-targets\zircon-cmft-skybox-0707\debug\deps\runtime_shader_pbr_hdri_export-ddba9a07f2a7f0d8.exe runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_source_reference_metrics --exact --nocapture --test-threads=1`
- `E:\cargo-targets\zircon-cmft-skybox-0707\debug\deps\runtime_shader_pbr_hdri_export-ddba9a07f2a7f0d8.exe runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_orientation_metrics --exact --nocapture --test-threads=1`

The generated PNG is 1600x1200, 1,857,183 bytes, SHA256 `8E39454F46B8A30530A648FFD5813FEF8307BA1F0CC3C81991E4CF47F9536E2B`. The same file name was scanned under the repository `target` and `E:\cargo-targets\zircon-cmft-skybox-0707`; no target-copy was found.

## Open Issues

The strict source-reference follow-up for yawed multi-view screenshots and the requested 120-degree up/down/left/right saved-PNG gate are closed by the 2026-07-08 updates. Remaining EC-M3 follow-ups are outside this saved-PNG harness: GPU/offline PMREM seam comparisons, RenderDoc/product captures, derived/offline artifacts, higher-resolution bake coverage, and full CI once the unrelated UI compile errors in the current workspace are resolved.
