---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/background_load.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/camera.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/presenter.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/project_assets.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/tests/runtime_environment_ibl_source_import_staging_contract.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/hdri_metrics.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/scene_fixtures.rs
implementation_files:
  - zircon_app/Cargo.toml
  - zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/background_load.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/camera.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/presenter.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/project_assets.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/hdri_metrics.rs
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
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_8x8_cmft_pmrem_reflection_20260710.png
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_cardinal_120deg_reflection_20260708.png
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_multi_view_reflection_20260707.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_fixed_20260708.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_lifetime_20260708.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_vulkan_20260708_capture.rdc
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_interactive_startup_20260710.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_direct_after_depth_load_20260710.log
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_dx12_20260710.log
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_dx12_20260710_capture.rdc
  - docs/tests/runtime/shader/polyhaven_lakes_2k_staged_ibl_20260710_report.txt
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_staged_ibl_dx12_20260710.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_staged_ibl_dx12_multiview_120_20260710.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_rebuild_live_20260710.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_dx12_rebuild_20260710_capture.rdc
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_rebuild_validation_20260710.txt
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_async_live_20260711.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_async_mouse_drag_yaw_plus120_20260711.png
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_async_dx12_20260711_capture.rdc
  - docs/tests/runtime/shader/zircon_shader_pbr_viewer_async_validation_20260711.txt
doc_type: testing-guide
---

# Runtime Shader PBR HDRI Export Tests

## Purpose

`runtime_shader_pbr_hdri_export.rs` is the focused visual-validation harness for real-HDRI PBR environment lighting. It creates temporary projects, renders standard PBR material spheres under the Poly Haven Lakes HDRI source cubemap, exports accepted screenshots into `docs/tests/runtime/shader`, and then runs saved-PNG assertions against those screenshots.

The harness is intentionally separate from production rendering modules. It exercises the real asset path through `ProjectManager`, `SceneRenderer`, `EnvironmentExtract::source_cubemap(...)`, standard PBR material assets, and generated scene fixtures.

## 8x8 Metallic/Smoothness Matrix

The current standard-PBR matrix is the requested 8x8 grid, not the older 10x10 diagnostic layout. Columns sweep metallic from 0 at the left to 1 at the right. Rows sweep smoothness from 0 at the top to 1 at the bottom, with roughness written as `1 - smoothness`. Both axes include their exact 0 and 1 endpoints, producing 64 independently authored standard materials.

`runtime_shader_pbr_matrix_contract_uses_requested_eight_by_eight_grid` locks the grid size and endpoints. The ignored export test renders the matrix over the staged Poly Haven Lakes 2K HDRI and writes `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_8x8_cmft_pmrem_reflection_20260710.png`. The saved-PNG metric test checks sky/ground variation, metallic response, smoothness response, bounded PMREM energy drift, and increasing high-frequency reflection detail toward the smooth high-metal cells.

The accepted image is 1600x1200, 1,383,647 bytes, SHA256 `AC2DD56DB79C63FC8037D78164D539ECCD42C2FCD8BE2E6E0AFF77D362E1C286`. The export test and saved-PNG metric test pass against the current implementation, and scans found no copy under the repository `target` or the active external Cargo target.

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

`zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs` builds a small manual inspection program for the same source-cubemap PBR path. It loads `docs/tests/runtime/shader/assets/polyhaven_lakes_2k.hdr` by default, generates a temporary project containing one perfect-metal mirror PBR sphere, stages the HDRI as source `.zcube` plus PMREM/SH9/IEM `.zribl`, reloads that pair as the render environment, renders through `SceneRenderer`, and presents the result through a `winit`/`softbuffer` window.

The viewer is split by owner instead of living in one binary file: `app.rs` owns winit event handling, `background_load.rs` owns the panic-safe worker/result channel, `camera.rs` owns orbit camera math and render camera descriptors, `presenter.rs` owns the softbuffer CPU-frame presenter, `scene.rs` owns temporary project loading and `SceneRenderer` calls, `project_assets.rs` owns generated scene/model/material assets, `hdri.rs` owns importer staging plus staged environment restore and display exposure, and `args.rs` owns command-line parsing.

The 2026-07-08 startup fix created the window before scene construction, but it still called `PbrMirrorScene::new(...)` synchronously from `ensure_window`, so Windows reported the visible process as not responding for about 78 seconds. The 2026-07-11 correction starts one named background task after the startup frame is presented. That task owns HDRI staging, temporary-project import, staged `.zcube/.zribl` restore, and `SceneRenderer` creation; it sends the completed scene or error through a channel and calls `EventLoopProxy::wake_up()`. Only the main event loop installs the result and presents frames, so close, resize, pointer, wheel, and repaint events remain responsive throughout loading. Worker panics are converted to the same visible load-failure state instead of silently disconnecting. The default source-cubemap face size remains 256, with `--face-size 64|128|256|512` available for manual quality/performance checks.

For RenderDoc evidence, the viewer also supports `--renderdoc-capture-once --exit-after-capture`. The app starts wgpu graphics-debugger capture around the first real `SceneRenderer` PBR/HDRI frame, stops capture after the frame render completes, presents the frame, then exits when requested. This is a development/debug path; normal manual orbit viewing does not trigger capture.

Usage:

```powershell
E:\ZirconBuilds\shader-pbr-viewer-20260711\zircon_shader_pbr_viewer.exe
E:\ZirconBuilds\shader-pbr-viewer-20260711\zircon_shader_pbr_viewer.exe --hdri E:\Git\ZirconEngine\docs\tests\runtime\shader\assets\polyhaven_lakes_2k.hdr
E:\ZirconBuilds\shader-pbr-viewer-20260711\zircon_shader_pbr_viewer.exe --face-size 512
E:\ZirconBuilds\shader-pbr-viewer-20260711\zircon_shader_pbr_viewer.exe --face-size 64 --renderdoc-capture-once --exit-after-capture
```

Left mouse drag orbits the camera. The mouse wheel zooms. The viewer intentionally uses `stage_environment_ibl_source(...)`, `IblSourceCubemapStagingStore::read_source_cubemap_environment(...)`, `EnvironmentExtract::source_cubemap(...)`, the standard PBR material, and the normal camera descriptor path. Manual inspection therefore exercises importer-produced source/derived artifacts and the runtime reflection route rather than a separate debug renderer or a temporary viewer-only PMREM build.

## Test Coverage

Fresh validation for the 2026-07-08 cardinal/viewer slice and owner-split viewer rebuild:

- `rustfmt --edition 2021 zircon_runtime\tests\runtime_shader_pbr_hdri_export.rs zircon_runtime\tests\runtime_shader_pbr_hdri_export\scene_fixtures.rs zircon_app\src\bin\zircon_shader_pbr_viewer\main.rs zircon_app\src\bin\zircon_shader_pbr_viewer\app.rs zircon_app\src\bin\zircon_shader_pbr_viewer\args.rs zircon_app\src\bin\zircon_shader_pbr_viewer\camera.rs zircon_app\src\bin\zircon_shader_pbr_viewer\hdri.rs zircon_app\src\bin\zircon_shader_pbr_viewer\presenter.rs zircon_app\src\bin\zircon_shader_pbr_viewer\project_assets.rs zircon_app\src\bin\zircon_shader_pbr_viewer\scene.rs`
- `rustfmt --edition 2021 --check zircon_app\src\bin\zircon_shader_pbr_viewer\main.rs zircon_app\src\bin\zircon_shader_pbr_viewer\app.rs zircon_app\src\bin\zircon_shader_pbr_viewer\args.rs zircon_app\src\bin\zircon_shader_pbr_viewer\camera.rs zircon_app\src\bin\zircon_shader_pbr_viewer\hdri.rs zircon_app\src\bin\zircon_shader_pbr_viewer\presenter.rs zircon_app\src\bin\zircon_shader_pbr_viewer\project_assets.rs zircon_app\src\bin\zircon_shader_pbr_viewer\scene.rs`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-shader-viewer-0708 CARGO_INCREMENTAL=0 cargo check -p zircon_app --bin zircon_shader_pbr_viewer --no-default-features --features target-client --locked --jobs 1 --color never`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-shader-viewer-0708 CARGO_INCREMENTAL=0 cargo build -p zircon_app --bin zircon_shader_pbr_viewer --no-default-features --features target-client --locked --jobs 1 --color never`
- `E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --help`
- `$env:WGPU_BACKEND='vulkan'; D:\Tools\renderdoc\renderdoccmd.exe capture -w -c E:\Git\ZirconEngine\docs\tests\runtime\shader\zircon_shader_pbr_viewer_renderdoc_vulkan_20260708 -d E:\cargo-targets\zircon-shader-viewer-0708\debug E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --face-size 64 --renderdoc-capture-once --exit-after-capture`
- `$env:WGPU_BACKEND='dx12'; E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --face-size 64 --renderdoc-capture-once --exit-after-capture`
- `$env:WGPU_BACKEND='dx12'; D:\Tools\renderdoc\renderdoccmd.exe capture -w -c E:\Git\ZirconEngine\docs\tests\runtime\shader\zircon_shader_pbr_viewer_renderdoc_dx12_20260710 -d E:\cargo-targets\zircon-shader-viewer-0708\debug E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe --face-size 64 --renderdoc-capture-once --exit-after-capture`
- `Start-Process -FilePath E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe -WorkingDirectory E:\cargo-targets\zircon-shader-viewer-0708\debug -ArgumentList @('--face-size','256') -RedirectStandardOutput docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_fixed_20260708.stdout.log -RedirectStandardError docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_fixed_20260708.stderr.log -PassThru`
- `Start-Process -FilePath E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe -WorkingDirectory E:\cargo-targets\zircon-shader-viewer-0708\debug -ArgumentList @('--face-size','256') -RedirectStandardOutput docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_lifetime_20260708.stdout.log -RedirectStandardError docs/tests/runtime/shader/zircon_shader_pbr_viewer_startup_lifetime_20260708.stderr.log -PassThru`
- `Start-Process -FilePath E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe -ArgumentList @('--hdri','E:\Git\ZirconEngine\docs\tests\runtime\shader\assets\polyhaven_lakes_2k.hdr') -WorkingDirectory E:\Git\ZirconEngine -PassThru`
- `E:\cargo-targets\zircon-shader-120deg-0708\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe export_runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png --ignored --exact --nocapture --test-threads=1`
- `E:\cargo-targets\zircon-shader-120deg-0708\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_environment_metrics --exact --nocapture --test-threads=1`
- `E:\cargo-targets\zircon-shader-120deg-0708\debug\deps\runtime_shader_pbr_hdri_export-810bef67c8d69fb9.exe runtime_shader_pbr_real_hdri_mirror_cardinal_120deg_png_matches_source_reference_metrics --exact --nocapture --test-threads=1`

The 120-degree cardinal PNG is 1600x1200, 2,099,569 bytes, SHA256 `0E57C496E49B6044F9AA5495BBFE855D5EB575A8D5AFAA8CB55B3DFF1D18AF15`. The same file name was scanned under the repository `target`, `E:\cargo-targets\zircon-shader-120deg-0708`, and `E:\cargo-targets\zircon-cmft-skybox-0707`; no target-copy was found. The owner-split viewer executable was rebuilt on 2026-07-10 at `E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe`, 70,599,168 bytes, SHA256 `D3DCA52330FE5B387341CA6844C5376165857432B14ED529E467E971C233CDE8`. `--help` confirms the default Poly Haven Lakes 2K HDRI path, default face size 256, optional `--face-size`, optional RenderDoc capture flags, left-drag orbit, and wheel zoom controls. A normal DX12 launch created a responsive `Zircon PBR HDRI Mirror Viewer` window; `docs/tests/runtime/shader/zircon_shader_pbr_viewer_dx12_interactive_startup_20260710.png` is 1296x999, 957,068 bytes, SHA256 `24BE09978519130A16495A52A6F8365C9FBD47E565014DEA828813A8FA465241`, and shows the HDRI skybox plus mirror-sphere reflection. Direct DX12 one-shot capture exited 0. RenderDoc DX12 launch also exited 0 and wrote `docs/tests/runtime/shader/zircon_shader_pbr_viewer_renderdoc_dx12_20260710_capture.rdc`, 13,378,644 bytes, SHA256 `3C4332282E5D119BD113E3947BC27982897485C5CE3E48754721F17399EF6D3A`. The prior FXC `X3511` failures were closed by derivative-free shared shadow/depth sampling, not by bypassing deferred lighting or SSR. The split viewer modules stay below the structure-convention ceiling; the current viewer production source stays free of `allow(...)`, `unwrap(...)`, `expect(...)`, and `panic!`.

The staged viewer rebuild on 2026-07-10 is `E:\cargo-targets\zircon-shader-viewer-0708\debug\zircon_shader_pbr_viewer.exe`, 71,040,512 bytes, SHA256 `AEC8F04EEBBE074EF3926165EFC64BBDEC8A8385FF31C691B9AA3CF874A52A1C`. A DX12 launch stayed responsive and logged `status=Written, face_size=256, mip_count=9` with distinct temporary `.zcube/.zribl` paths and no stderr. The default screenshot `zircon_shader_pbr_viewer_staged_ibl_dx12_20260710.png` is 1296x999, 942,532 bytes, SHA256 `49E97541BE3DF071B7D22026FD41FA2178B5C7EF9EBAF951B468DB5CA0695C12`. The four-view contact sheet `zircon_shader_pbr_viewer_staged_ibl_dx12_multiview_120_20260710.png` is 1296x1000, 1,800,034 bytes, SHA256 `6E7969341EE7CFF3E9EE6F2EDC38250FCCC871E640883DFFFD42C24B81F5E486`; its tiles are yaw +120, yaw -120, pitch +120, and pitch -120. Visual inspection confirms matched skybox/reflection orientation and filtered reflection detail without blocky source-mip substitution.

The current-worktree rebuild was repeated in `E:\cargo-targets\zircon-zcube-staged-bundle-0708`: the `target-client` build exited 0, `--help` exposed the HDRI/face-size/RenderDoc/orbit/zoom controls, and the 71,365,120-byte EXE has SHA256 `F465A7A394BD2F017A84BBD7F2B22F718FE09AC72CB29A8122AD7FFD8146A60E`. The normal DX12 process stayed responsive and loaded `status=Written, face_size=256, mip_count=9`; its live 1296x999 screenshot is `zircon_shader_pbr_viewer_dx12_rebuild_live_20260710.png`. The fresh 13,374,191-byte DX12 capture replayed once through `renderdoccmd replay`. The screenshot scan found no shader/PBR/cubemap/HDRI images under the repository `target` or the active external Cargo target.

To keep the manual program available after external Cargo-target cleanup, the same verified executable is staged at `E:\ZirconBuilds\shader-pbr-viewer-20260710\zircon_shader_pbr_viewer.exe`. Its SHA256 is identical to the current-worktree build. The current manual-validation process was launched from that stable path and remained responsive with the title `Zircon PBR HDRI Mirror Viewer`.

The asynchronous-startup rebuild is staged at `E:\ZirconBuilds\shader-pbr-viewer-20260711\zircon_shader_pbr_viewer.exe`, 72,125,440 bytes, SHA256 `BA190E3D2A41E2B9435AB874008CA8744185B22004B37AD320BD7971A529A195`. Its background-task tests pass 2/2 and the `target-client` production build exits 0. A normal DX12 launch had a valid top-level window and `Responding=True` at 1, 3, 5, and 10 seconds, then loaded `status=Written, face_size=256, mip_count=9` while remaining responsive. The 1296x999 live image `zircon_shader_pbr_viewer_async_live_20260711.png` shows the Lakes skybox and detailed mirror reflection; a 343-pixel left drag changed the rendered view by approximately 120 degrees and produced `zircon_shader_pbr_viewer_async_mouse_drag_yaw_plus120_20260711.png`. The new one-shot DX12 RenderDoc capture exited 0, wrote `zircon_shader_pbr_viewer_renderdoc_async_dx12_20260711_capture.rdc` at 14,831,107 bytes with SHA256 `7732F95E73695D5A0101D65692836A2B1ADCFF9F3D5EFA6302DC4FD1C68B19F6`, and replayed once with exit 0. Full process samples and hashes are in `zircon_shader_pbr_viewer_async_validation_20260711.txt`.

Earlier validation for the 2026-07-07 multi-view slice:

- `rustfmt --edition 2021 zircon_runtime\tests\runtime_shader_pbr_hdri_export.rs zircon_runtime\tests\runtime_shader_pbr_hdri_export\frame_assertions.rs`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-cmft-skybox-0707 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export export_runtime_shader_pbr_real_hdri_mirror_multi_view_png --no-default-features --features core-min --locked --jobs 1 --color never -- --ignored --exact --nocapture --test-threads=1`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-cmft-skybox-0707 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_orientation_metrics --no-default-features --features core-min --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1`
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-cmft-skybox-0707 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export runtime_shader_pbr_real_hdri_mirror_reflection_png_matches_source_reference_metrics --no-default-features --features core-min --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1`
- `E:\cargo-targets\zircon-cmft-skybox-0707\debug\deps\runtime_shader_pbr_hdri_export-ddba9a07f2a7f0d8.exe runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_source_reference_metrics --exact --nocapture --test-threads=1`
- `E:\cargo-targets\zircon-cmft-skybox-0707\debug\deps\runtime_shader_pbr_hdri_export-ddba9a07f2a7f0d8.exe runtime_shader_pbr_real_hdri_mirror_multi_view_png_matches_orientation_metrics --exact --nocapture --test-threads=1`

The generated PNG is 1600x1200, 1,857,183 bytes, SHA256 `8E39454F46B8A30530A648FFD5813FEF8307BA1F0CC3C81991E4CF47F9536E2B`. The same file name was scanned under the repository `target` and `E:\cargo-targets\zircon-cmft-skybox-0707`; no target-copy was found.

## Open Issues

The strict source-reference follow-up for yawed multi-view screenshots, the requested 120-degree up/down/left/right saved-PNG gate, responsive asynchronous viewer startup, Vulkan/DX12 viewer RenderDoc capture, importer-driven staged artifact viewer consumption, and six-file/cross authoring contracts are closed. ReflectionProbe GPU resources, top-2 blending, and feature-off sky fallback are also covered by separate product contracts. Remaining EC-M3 follow-ups are outside this saved-PNG harness: reflection-probe scene capture/persistence and dynamic updates, compressed/Basis cubemap transcoding, higher-resolution 4K/16K bake coverage, broader product captures, and full CI once unrelated concurrent workspace changes settle.
