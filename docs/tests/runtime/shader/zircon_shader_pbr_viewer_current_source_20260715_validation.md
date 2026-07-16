# Zircon Shader06 current-source interactive viewer validation

Date: 2026-07-15 Asia/Shanghai
Source HEAD before milestone commit: `8360d307ac805883b1e0febd1b0b7cc0a72112a1`

## Architecture gate

- `scene::tests::viewer_uses_real_runtime_module_and_asset_manager_lifecycle` searches only the source before `#[cfg(test)]`, then locks the real `TasksModule.descriptor()`, Foundation -> Tasks -> Asset registration/activation order, `ProjectAssetManagerAccess`, and production `SceneRenderer::new` path. It rejects a direct default asset manager and test-only renderer construction.
- `scene::tests::scene_renderer_drops_before_its_runtime_services` locks the declaration order that drops the renderer before its owning `CoreRuntime` services.
- `camera::tests::mouse_wheel_zoom_changes_radius_and_clamps_to_orbit_limits` locks wheel-driven radius changes and both orbit limits.
- Managed job `1b86e228d43f4fad9edd9f0ef69d48df` passed the viewer test binary 18/18 before the final production-only search-scope correction. A read-only production-anchor-removal simulation is RED as expected after that correction. Managed job `03c5072d8e5b4214b146fd620027324d` then executed the corrected current source and passed the viewer binary 18/18, including the production-only lifecycle guard.

## Managed Windows production build and test

- Coordinator job: `1b86e228d43f4fad9edd9f0ef69d48df`.
- Command: `validate-matrix.ps1 -RepoRoot E:\Git\ZirconEngine -Package zircon_app -VerboseOutput`.
- Configuration: `CARGO_INCREMENTAL=0`; `CARGO_PROFILE_DEV_DEBUG=0`.
- Result: Cargo build OK and Cargo test OK; exit 0.
- Managed target: `D:\cargo-targets\zircon-engine\pool\c07cadc864b35086ee68c4f87411d5a2a854b0e5f37ed02c5b10c87e4873aca6`.
- EXE: `D:\cargo-targets\zircon-engine\pool\c07cadc864b35086ee68c4f87411d5a2a854b0e5f37ed02c5b10c87e4873aca6\debug\zircon_shader_pbr_viewer.exe`.
- EXE size: 72,369,664 bytes.
- EXE SHA256: `F8EEAD721B125E9D4CAEF374E9A532F07EDE0909B4DCC724B173F3508994233A`.
- `--help` exit: 0; output names left-drag orbit, mouse-wheel zoom, source face size, PMREM face size, yaw/pitch, and one-shot RenderDoc options.

## Real Lakes HDRI DX12 interactive run

- HDRI: `assets/polyhaven_lakes_2k.hdr`, 5,918,432 bytes, SHA256 `B2506E0EE912C4C599FF013566FBD3ECAAC2F4B176319D450CCE0DE5758FED98`.
- Arguments: `--face-size 512 --pmrem-face-size 512`.
- The window stayed responsive in all 26 five-second samples and reached `Ready - yaw 0 pitch 0` at the 131-second sample. The runtime log reported scene preparation in 127.41 seconds.
- Runtime log: source face 512/mip 10, staged and active PMREM face 512/mip 10, derived `face_0512_mips_10.zribl`, and empty stderr.
- A targeted native `WM_MOUSEWHEEL` event changed 446,585 of 1,294,704 screenshot pixels (34.4932%) with mean absolute RGB delta 8.4627. This proves the real `WindowEvent::MouseWheel -> handle_mouse_wheel -> OrbitCamera::zoom -> request_redraw` path changed the presented frame.
- A targeted 343-pixel native left-button drag changed the title from yaw 0 to yaw 120 while the process remained responsive.
- A preliminary global-input attempt was rejected by Windows foreground-stealing policy (`SetForegroundWindow=false`) and correctly produced no image change. Direct window-targeted native messages isolated automation focus from the viewer event path; no production input bypass was added.
- Screenshot: `runtime_shader_pbr_interactive_viewer_current_source_20260715.png`, 1296x999, 1,184,470 bytes, SHA256 `90D45BD3256C323275BCF112551264DD26DBFEA7D5550F24C7685C7B1D3A1354`.
- Original-resolution inspection: the Lakes road, shoreline, trees, clouds, and horizon are continuous in the skybox and clearly recognizable across the enlarged mirror sphere. The sphere is not white, the reflection orientation matches the environment, and the previous low-resolution block mosaic is absent.

## RenderDoc DX12

- Tool: `D:\Tools\renderdoc\renderdoccmd.exe`.
- Capture arguments: Lakes 2K, source 512, PMREM 512, `--renderdoc-capture-once`, `--exit-after-capture`.
- Capture log: source and active PMREM are both 512x512 with 10 mip levels; scene preparation took 167.96 seconds; graphics debugger capture completed; process exit 0.
- Capture: `zircon_shader_pbr_viewer_current_source_dx12_renderdoc_20260715_capture.rdc`.
- Capture size: 48,462,499 bytes.
- Capture SHA256: `1F12B9B03C0E3C2B8D1ED5068868C3FD589DB76578F639E06E277C11ABDBD0BC`.
- `renderdoccmd replay --loops 1` exit: 0.

## Artifact placement

- The screenshot, capture, and this record are under `docs/tests/runtime/shader`.
- Exact-name recursive scans found zero screenshot or RDC copies under `E:\Git\ZirconEngine\target` and the managed Cargo target.
