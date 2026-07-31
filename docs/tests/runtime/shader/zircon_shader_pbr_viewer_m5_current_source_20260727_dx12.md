# Shader06 M5 Current-Source DX12 Viewer Evidence

Date: 2026-07-27
Status: DX12 evidence from the pre-review source is visually verified. The post-review release build now succeeds after unrelated UI/plugin fixture repairs, but the current startup-timing, IBL-accounting, reusable-project-cache, renderer-startup-report, and first-ready-frame-presentation source changes were made afterward. The retained image/RDC are therefore historical visual evidence only; a fresh release run, replacement DX12/RenderDoc evidence, plan-output, and failure closure remain before milestone closeout.

## Current Executable

- Coordinator job `5fa4839c0a5644f0bd359fe917e04942` / run `b453e1db550c47cfb32e9216ea78ded3` completed `cargo build -p zircon_app --locked` with exit 0.
- Executable: `D:\cargo-targets\zircon-engine\pool\841a130ffbd3fd2e938e76b488988119044b676acced751dae7166d95d7f1025\debug\zircon_shader_pbr_viewer.exe`.
- `--help` succeeded and listed HDRI, IBL cache, face-size, PMREM face-size, RenderDoc, orbit, and zoom controls.
- Coordinator job `9c8e527cec8b4d02adf5cb26a5965ce6` / run `c2d9622d1aac444fb6a81669bf810587` completed `cargo build -p zircon_app --locked --release` with exit 0 in 21 m 41 s.
- Coordinator job `bbcebec1362a47e9b5abe7621b4b97af` / run `0713c696b1d94a6e81854bf8415489ad` subsequently completed `cargo build -p zircon_app --locked --release` with exit 0 in 15 m 54 s; its release viewer executable also accepted `--help`.
- The release executable at the same pool's `release\zircon_shader_pbr_viewer.exe` returned exit 0 for `--help` and exposed the same runtime controls.

## DX12 Runtime

- Backend: `WGPU_BACKEND=dx12`.
- Input: `assets/polyhaven_lakes_2k.hdr`.
- Bake configuration: source face size 256, PMREM face size 256, 9 source mips, 9 PMREM mips.
- First load reached `Ready - IBL Written`; the runtime reported 11.98 s staging and 65.93 s total elapsed time. In that historical executable, the displayed IBL total incorrectly included unrelated HDRI/project/runtime/renderer startup, so it is not an IBL-bake measurement.
- The loading title continued to report that the window was responsive. In Ready state, a real left drag and wheel changed yaw/pitch from `20/-15` to `49/-7`; the captured window remained responsive.

## Artifacts

- Screenshot: `zircon_shader_pbr_viewer_m5_current_source_20260727_dx12.png`.
- RenderDoc capture: `zircon_shader_pbr_viewer_m5_current_source_20260727_dx12_capture.rdc`.
- Capture command used `D:\Tools\renderdoc\renderdoccmd.exe capture` with the same DX12 executable and `--renderdoc-capture-once --exit-after-capture`.
- Runtime capture log recorded `starting graphics debugger capture on wgpu(dx12)` followed by successful completion.
- `D:\Tools\renderdoc\renderdoccmd.exe replay --loops 1` completed successfully for the recorded `.rdc`.

The pre-review release gate and DX12 artifacts are complete and retained as historical visual evidence only. The post-review release build now succeeds, but the newer startup-performance source requires a fresh replacement build, DX12 run, and RenderDoc capture before coordinator validation, final independent review, and managed milestone commit can close M5.

## Current-Source Rebuild Status

- Coordinator job `4e4620a1530040578143a2a2898e723a`, run `f714cc862ded43119948cadbe60f3ec2`, initially exited 101 at external UI/plugin fixture errors. The shared-source repairs were then compiled successfully by job `bbcebec1362a47e9b5abe7621b4b97af` / run `0713c696b1d94a6e81854bf8415489ad`.
- Coordinator job `c70c8bd9c92e4aaebcc7d47375b83afa` / run `df50cb1dee1f4b5bbde525c726452e8d` exited 101 while compiling the new startup report. The failure was limited to missing report re-exports, one relative module path, and the `EmptyViewportIconSource` import. Follow-up job `7aac95d05c034515bf83fe7dbf62b19f` / run `132cf210838a4d9cb30b99b280e06a0a` then isolated the last missed `graphics::scene` facade re-export. All paths were repaired before the current FIFO reservation was created; neither failed run produced an executable, image, or RDC evidence.
- The current viewer source now times HDR decode, project assets, runtime bootstrap, project open, world load, renderer initialization, and IBL restoration separately; it reports whether the versioned project cache was reused or regenerated. It also uses a 16,384-triangle generated mirror asset rather than the prior 36,864-triangle asset.
- Initial DX12 diagnosis on the preceding release establishes the correct baseline: cold start was 95.54 s total with renderer initialization at 90.55 s and IBL write at 3.68 s; warm start was 87.60 s total with renderer initialization at 86.48 s and IBL cache restore at 0.89 s. The previous title attributed the whole startup interval to IBL, which was an accounting error rather than an IBL processing cost.
- The newest source now splits renderer initialization into backend, core, and resource-streamer phases, then splits the core phase into setup, mesh/environment, shadows, deferred, scene effects, and overlay/UI groups. It separately records scene construction, first-frame presentation, and user-visible readiness. It also synchronously presents the first completed PBR frame before exposing the Ready title, because the diagnostic run showed an otherwise-ready title over the startup checkerboard until an input-triggered redraw. These changes require the fresh release run below before they become accepted evidence.
- No viewer image or RenderDoc capture is claimed for those newest changes. The screenshot and RDC above remain retained, manually reviewed DX12 evidence for the preceding release; Shader06 must rerun the managed release build and replace these artifacts before M5 commit.
- Current workflow evidence: M5 v7 isolated validation job `eec4621a646b4af1a43972b7715660bd`, run `e82ec2e18c2a4d38bf53cf31c33530ee`, completed exit 0 with `managed_validation_succeeded`; the final independent re-review accepted the candidate with 0 critical and 0 important findings. Neither result substitutes for the required post-review runtime rebuild and replacement visual artifacts.
