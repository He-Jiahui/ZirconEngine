# Shader06 M5 Deferred-Lighting DX12 Startup Evidence

Date: 2026-07-28
Status: current-source environment-only PBR performance, Ready-frame visual, and Debug RenderDoc replay evidence. The current Release rebuild and exact viewer CLI suite pass; the retained crate-internal pipeline assertion remains source-owned while its all-lib-test invocation reaches unrelated test-only compile diagnostics before discovery.

## Build

- Coordinator job `d4d8d0e7db4a4d12a9d3cdde0c1fa888`, run `f621b4abe01a40d0a1279b6ef8d2c3f8`, built the deferred-lighting optimization with `cargo build -p zircon_app --bin zircon_shader_pbr_viewer --locked --release` (exit 0 after 29m47s).
- After adding deterministic Ready-frame PNG export, retry job `b84413fc1080450c8a4a99071d5aa320`, run `78c6acce51f249b1a7adfeaf585a68e9`, ran the same release command with exit 0 after 16m42s. The executable returned exit 0 for `--help`, including `--screenshot <path.png>`.
- Job `7cd443a5c1d044d7b1cc701f89cf4142`, run `0a86be3e0c0244c68e4787666e447777`, rebuilt the same release executable after adding startup and Ready-frame timing splits (exit 0 after 14m18s; existing workspace warnings only).
- Job `97b46d95a7894b8e9434727bee4bdafa`, run `e43a0c1754f440f893484c680f4b6622`, rebuilt the split startup-report source with `cargo build -p zircon_app --bin zircon_shader_pbr_viewer --locked --release` (exit 0 after 28m35s). The DX12 runs below use its executable.
- Job `55aabeda3af9469ca3c9e3a1c1a6b6f6`, run `ccf34546597743a386f7fa93d58624bc`, built the explicit Standard-PBR-preview profile with the same command (exit 0 after 15m59s; existing workspace warnings only). The final two DX12 runs below use this executable.
- Debug job `da06618ee2f8493db53ca92b29f60b1d`, run `e7d000f5d85b43e99ac5f7093392be09`, built the current RenderDoc capture bridge with `cargo build -p zircon_app --bin zircon_shader_pbr_viewer --locked` (exit 0 after 47.16s; existing workspace warnings only).
- Final Release attempt job `b51f8156fde443288039efb174f8654c`, run `7b9731e86aad4622a0fcacaca059c45f`, reached `zircon_runtime` but exited 101 on the current Text01-owned change: `TextRasterWorkerPoolDiagnostics` is initialized without its new `cancelled` field at `zircon_runtime/src/text/parallel/raster_pool.rs:201`. This is outside the viewer/Shader06 scope; no Release capture claim is made from the failed run.

## DX12 Measurements

Input was `docs/tests/runtime/shader/assets/polyhaven_lakes_2k.hdr` with source and PMREM face size 256. The first run used a newly created caller-owned IBL cache; the later run reused that cache and did not inject RenderDoc.

| Run | IBL state | Renderer init | Deferred resources | IBL restore/stage | Scene constructed |
| --- | --- | ---: | ---: | ---: | ---: |
| First current-source DX12 run | Written | 35.57s | 27.66s | 4.46s | 40.46s |
| Later current-source DX12 run | Reused | 19.78s | 15.44s | 608.57ms | 20.52s |
| Direct Ready-frame PNG run | Reused | 31.47s | 20.16s | 813.06ms | 32.62s |
| Split-timing Ready-frame PNG run | Reused | 31.09s | 22.74s | 843.51ms | 32.14s |
| Constructor-split Ready-frame PNG run | Reused | 32.54s | 21.82s | 595.29ms | 33.56s |
| Repeated DX12 launch 1 (same released baseline) | Reused | 31.51s | 23.06s | 842.55ms | 32.58s |
| Repeated DX12 launch 2 (same released baseline) | Reused | 32.32s | 23.83s | 845.74ms | 33.38s |
| Standard-PBR-preview launch 1 | Reused | 21.27s | 11.66s | 866.79ms | 22.57s |
| Standard-PBR-preview launch 2 | Reused | 19.62s | 11.18s | 804.01ms | 20.63s |

The old startup report had attributed about 82--100s to loading. Corrected phase accounting identifies `DeferredSceneResources`, not HDR decode, PMREM baking, or a cache hit, as the dominant cost: the historical deferred phase was about 91.74s while IBL cache restore was about 1.08s.

The current implementation prewarms only the standard PBR deferred PSO. The SSS MRT variant is constructed on first use, and the full-screen vertex shader is a small dedicated WGSL module so DX12 does not compile the full deferred-lighting source for that stage. In the latest run, `DeferredSceneResources=22.74s` is entirely `lighting_pipelines=22.74s`; its sampler, shadow fallback, and volumetric fallback resources took only `700.30us`. IBL restore was `843.51ms`. The repeated-run floor is not a cold-start guarantee because DX12/driver compiler cache state changes between processes.

Coordinator GPU job `c30e6c4879974ab6b3b509be799b3c42`, run `0d6a459b5a0c46ec87546d23780e3268`, used `WGPU_BACKEND=dx12`, the Lakes HDRI, source/PMREM face size 256, and the caller-owned cache. It wrote [`zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12.png`](zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12.png), a 1280x960 RGBA PNG (873,222 bytes). Visual inspection confirms the mirror sphere, HDRI horizon, sky, and sun highlight are coherent and the artifact contains no desktop or firewall dialog.

The corresponding current-source split-timing run was GPU job `b9e200859078419da0eb34a161fa98a4`, run `62442e37979b40c0bf0ec75336347963`. It wrote [`zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_timing.png`](zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_timing.png), another 1280x960 RGBA PNG (873,222 bytes), with the same verified PBR reflection. Its first Ready frame was `render=9.14s`, `screenshot_encode=19.48ms`, and `surface_present=1.46ms`; therefore the extra 9.14s is renderer first-frame work, not PNG encoding or desktop presentation.

GPU job `28b65efd6e25444eb34a7048b70bf1c8`, run `8201296a315443188be426cbd1e4b63b`, used the fresh constructor-split executable with `WGPU_BACKEND=dx12`, the same caller-owned cache, and the correct HDRI path under `docs/tests/runtime/shader/assets`. It wrote [`zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_decomposed.png`](zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_decomposed.png), a 1280x960 RGBA PNG (873,222 bytes). Manual inspection again confirms coherent Lakes sky, horizon, mirror-sphere reflection, and sun highlight without desktop or firewall contamination. Its startup report measured `lighting_source_assembly=1.36ms`, `pipeline_foundation=13.91ms`, and `standard_pso=21.80s`; the latter remains the dominant startup cost.

### Cross-process PSO persistence check

The same released baseline was launched twice in immediate succession with `WGPU_BACKEND=dx12`, the same 256-face Lakes input, and the same caller-owned IBL cache. Coordinator GPU job `283b292927104fd3afbb247ee17120b7`, run `7f951197ba22427885c27cabd84d2dac`, reported `standard_pso=23.04s`, `post_process=3.26s`, `backend=3.50s`, `scene_constructed=32.58s`, and `first_frame_presented=9.61s` (`42.19s` Ready total). The immediately following GPU job `ac130a1591e743479c716dd0ca0ac4f8`, run `d2365b5180be4d1c90c3a7a7c6ebe8d6`, reported `standard_pso=23.82s`, `post_process=3.48s`, `backend=3.32s`, `scene_constructed=33.39s`, and `first_frame_presented=9.27s` (`42.65s` Ready total).

Both launches reported `EnvironmentIblSourceStagingStatus::Reused` and restored IBL in less than 846ms, while the standard PSO remained above 23 seconds. This rules out HDR decode, PMREM generation, and the caller-owned IBL cache as the 82--100 second startup cause, and shows that the observed DX12 driver cache does not materially reuse this pipeline across processes. These runs predate the current-worktree timing-boundary fix and fragment-template cleanup, so their frame-subinterval labels are diagnostic only; the whole Ready totals and startup phase values remain valid for the persistence experiment.

### Explicit Standard PBR preview profile

`SceneRendererStartupOptions::standard_pbr_preview()` is an explicit tool-only startup configuration. It assembles the deferred fragment source from the Standard PBR material include only, while retaining `FullScene` as the default path for arbitrary scenes and rejecting custom shading-model registrations for the preview profile. This preserves the generic unlit, Blinn-Phong, subsurface, and plugin contracts instead of deleting them globally.

The two current-release DX12 launches reused the same caller-owned IBL artifact and produced [`zircon_shader_pbr_viewer_m5_pbr_preview_20260728_dx12.png`](zircon_shader_pbr_viewer_m5_pbr_preview_20260728_dx12.png) plus a state-only repeat. Manual inspection of the tracked PNG confirms a coherent Lakes HDRI sky, horizon, sun highlight, and mirror-sphere reflection. `standard_pso` fell to `11.65s` then `11.16s`, versus `21.80s` in the constructor-split baseline and `23.04s/23.82s` in its immediate cross-process repetition. The first Ready total was `32.47s`; the repeat was `30.69s`. This establishes the profile as a material reduction in the startup PSO cost, while the remaining 3.3--3.6s post-process setup and about 10s first-frame submission remain separate work.

## Environment-Only PBR Profile

`SceneRendererStartupOptions::environment_only_pbr_preview()` is the viewer-only successor to Standard PBR preview. It keeps GBuffer reconstruction, material metallic/roughness/AO, PMREM, SH9/IEM, reflection probes, ambient, and emissive lighting, while removing unused GPU-scene, direct-light, shadow, volumetric, and custom-shading-model bindings from its deferred-lighting pass layout. The renderer still constructs shared GPU-scene, mesh, and shadow infrastructure; the generic `FullScene` profile remains the default for arbitrary renderer users.

Current-source Release job `a3c7edb2701d40e6932bba67991c32f8`, run `90b77bf258644535b5e88ff475dadcea`, built `zircon_shader_pbr_viewer` with exit 0. The exact CLI job `6c8bb7d4399441e2bfab45ab8d273b6b`, run `37e1f1efd0a3491fa783eca35dac6faf`, completed with 64 passed and 0 failed.

| DX12 Release run | IBL | Renderer init | Deferred / standard PSO | Scene constructed | Ready total |
|---|---:|---:|---:|---:|---:|
| First environment-only PBR run | Written, 3.32s | 12.99s | 1.15s / 1.14s | 16.74s | 26.84s |
| Immediate cache reuse | Reused, 856ms | 7.19s | 877ms / 870.23ms | 8.28s | 19.21s |

The first run's source assembly and pipeline-foundation slices were only `1.43ms` and `4.91ms`. The profile therefore reduces the historical 21--24s DX12 standard PSO compilation to about 0.87--1.14s. The separate first Ready frame still spends about 10s in the diagnostic viewer's offscreen render submission; PNG encoding (19--65ms) and surface present (about 1.5ms) are not the cause. This viewer path synchronously reads back to a CPU `SoftbufferViewportPresenter` and is not the engine's normal GPU texture-present path.

[`zircon_shader_pbr_viewer_m5_environment_only_pbr_20260729_dx12.png`](zircon_shader_pbr_viewer_m5_environment_only_pbr_20260729_dx12.png) and [`zircon_shader_pbr_viewer_m5_environment_only_pbr_cache_reused_20260729_dx12.png`](zircon_shader_pbr_viewer_m5_environment_only_pbr_cache_reused_20260729_dx12.png) are 1280x960 RGBA PNG exports with the same SHA-256 `B9341A9215A1854C7E39A76A0FA077F789BC288B6F73F2954CB7A6A1382EE390`. Both visibly show a coherent Lakes HDRI sun, sky, horizon, road, water, and mirror-sphere reflection.

Debug build job `10e72aaeef7c4c98a69a805e9e090b22`, run `4aef90f1fb984d7b891f9fd2042e5e30`, then performed an application-API RenderDoc DX12 capture in job `87673bedfd3c4d4c8bc065822983d309`, run `2ff035219bb84bb8820ce82352115c24`: `count=1`, `standard_pso=1.09s`, and the retained output is [`zircon_shader_pbr_viewer_m5_environment_only_pbr_20260729_dx12_renderdoc_capture.rdc`](zircon_shader_pbr_viewer_m5_environment_only_pbr_20260729_dx12_renderdoc_capture.rdc), 22,127,749 bytes, SHA-256 `6908A2DD8F916151C1585755F2BDE1E59A469772C97154167E535388427A936B`. Managed replay job `467bb9c94d554616a756b0035b7f0eb1`, run `c03fd12eb1d64ba48bad7d5c62935de0`, ran `D:\Tools\renderdoc\renderdoccmd.exe replay --loops 1` with exit 0.

## Current-Source RenderDoc Capture

The viewer now rejects `--renderdoc-capture-once` in a Release build because wgpu 29 only enables this integration with Rust debug assertions. For a Debug capture, `--renderdoc-dll <renderdoc.dll>` preloads the explicit module before wgpu creates its D3D12 device, and `--renderdoc-capture-path <template>` configures the RenderDoc 1.4.1 capture template. The CLI accepts a DLL and template only with the explicit one-shot capture mode.

The direct DX12 current-source run used `D:\Tools\renderdoc\renderdoc.dll`, the Lakes HDRI, caller-owned 256-face IBL cache, `--renderdoc-capture-once`, and `--exit-after-capture`. Its startup report was `standard_pso=5.97s`, `renderer_init=15.26s`, `ibl_restore=5.37s`, scene construction `22.24s`, and first Ready frame `5.84s`. After `scene.render()` completed, RenderDoc reported `count=1` and its actual output path, which uses the API suffix `_capture.rdc` rather than the `renderdoccmd` frame-number convention.

[`zircon_shader_pbr_viewer_m5_pbr_preview_20260729_dx12_renderdoc_capture.rdc`](zircon_shader_pbr_viewer_m5_pbr_preview_20260729_dx12_renderdoc_capture.rdc) is the resulting 22,129,854-byte capture (SHA-256 `B4D92546E2C14196356184ED82F497CF3885CCD9AAE3020F8D27120FCDA5399B`). `D:\Tools\renderdoc\renderdoccmd.exe replay --loops 1 <capture.rdc>` replayed it locally with exit 0. This is current-source Debug capture/replay evidence; it is not a claim that Release captures are supported.

## Pending Ready-Frame Decomposition

- The viewer's `SoftbufferViewportPresenter` consumes CPU RGBA, so `SceneRenderer::render` synchronously reads its offscreen texture before the presenter copies pixels into the window surface. This diagnostic viewer path is distinct from the engine's `ViewportSurface::present_texture` GPU-present path and must not be treated as a universal runtime-present cost.
- The Standard-PBR-preview screenshot run reports `render=9.88s [extract=501.70us, render_submission=9.87s, readback_and_completion=16.29ms]`; the repeat reports `render=10.04s [extract=318.80us, render_submission=9.99s, readback_and_completion=40.13ms]`. The corrected boundaries no longer overlap. The large first-frame cost is GPU submission/completion in the diagnostic offscreen path, not packet extraction, PNG encoding, or surface presentation.
- The release includes the redundant `vs_main` removal from the assembled deferred fragment template and its separate minimal fullscreen vertex module. The measured profile reduction is an aggregate result; it is not attributed to that one source reduction alone.
- The initial exact viewer job `d542cbca14a74cfb8ac0897063d35780`, run `78ebe34d640040d1b99f5d7b1ce3fc15`, stopped before tests on an unrelated text-cache compile failure. Retry job `d1582806a28242b395c35fb630dc6bcf`, run `ee8bc9b3ac8c415e8b5c0f1028df99dd`, completed with 60 passed / 0 failed. The current source exact CLI job `f2824b427274442c88f80bad8488cfc3`, run `32f0e217ada346a5bfeccacd65ad6f37`, then completed `cargo test -p zircon_app --bin zircon_shader_pbr_viewer --locked -- --test-threads=1` with 64 passed / 0 failed. The four additional checks cover Debug-only capture rejection in Release configuration plus valid and invalid explicit RenderDoc DLL/template combinations.
- The fresh current-source Release gate used reservation `dad3b8f4987445b59c022f728615ffe9`, job `7bf1197f62d942eab53d8d2228c791e6`, and run `aeb263813ed243428a20b30e82a61c28` for `cargo build -p zircon_app --bin zircon_shader_pbr_viewer --locked --release`. It reached `zircon_runtime` but terminated with exit 101 before viewer linking because Text01-owned `scene_renderer/ui/text.rs:357` lost the `GlyphAtlasStorageFormat` import while retaining its native-atlas fallback expression. The proven shared failure is routed as `glyph-atlas-storage-format-import`; this record makes no Release help, capture-flag, or Ready-frame claim from the failed build.
- Native `wgpu` exposes no asynchronous render-pipeline creation path, and wgpu 29.0.3 documents application-managed pipeline-cache persistence only for Vulkan. DX12 therefore has no supported persistent PSO-blob path here. The next optimization must reduce or defer required startup PSOs, or reduce shader source cost, based on the new phase measurements.
- The current generic deferred template resolves all built-in shading-model and environment paths so that an arbitrary scene remains correct. A PBR-viewer-only source profile can only be introduced with an explicit renderer configuration boundary; silently deleting unlit, Blinn-Phong, subsurface, probe, or plugin paths from the generic template would break the engine contract. The immediate MVP route is to retain the already-working loading frame, keep non-critical variants lazy, and establish a configuration-owned minimal PBR pipeline profile before attempting further source pruning.

## Historical Pre-Bridge Capture Diagnosis

- Before the direct API bridge, `D:\Tools\renderdoc\renderdoccmd.exe capture` runs reached the viewer's graphics-debugger start/stop log points but did not emit an `.rdc` at the requested template. This was first reproduced with coordinator job `8c2d0bde473c432faba0679737649230`, run `3eb2d7c5307c493c8f37ed3874ae2fbf`, and again with the split-timing executable in job `c7ab5e9e14bd4045a77d31828f7f0479`, run `21330796957b4c98b5695aa4ac13b50b`. Both used `capture --wait-for-exit --capture-file <state-template>` and exited 0 without a frame-suffixed output.
- The final pre-bridge absolute-path retry, job `dfd65c25913543a09a3a38d101172c0a`, run `88d7e6318b8a40e5925c925871311059`, eliminated working-directory ambiguity: it loaded the HDRI and logged both capture points, but no `.rdc` appeared in the requested evidence directory, release working directory, or `D:\Tools\renderdoc`. The repeated Standard-PBR-preview release command with an absolute cache path had the same result.
- This historical RenderDoc v1.44 output boundary is closed by the Debug-only direct API bridge documented above: it preloads `renderdoc.dll`, applies `SetCaptureFilePathTemplate`, reports `count=1` with the actual `_capture.rdc` path, and the retained capture replays successfully. New source revisions still require their own Debug capture; they must not reuse this artifact as current-source evidence.
- Native window screenshots reached the Ready state but Windows Firewall's first-run authorization dialog obscured the scene. Those images are retained only under `.codex/state` for diagnosis and are not copied into this evidence directory.
- The focused runtime pipeline regression reservation `c24ca7c228f341f7a0f4d5bae4421351` was released without a consumed job so the current-source release build for Ready-frame evidence could use the one-slot CPU lane. That exact runtime test must be recreated after the timing rebuild.
- The first Ready-frame screenshot build failed only on the new code's borrow ordering (`E0499` at `app.rs:288`); the presenter borrow was shortened and retry reservation `ee377bb260a5414a81836759f32c2e0b` completed as the successful job `b84413fc1080450c8a4a99071d5aa320` above. The timing rebuild reservation `10765e224dfa4c58bcad036bcffd0319` completed as job `7cd443a5c1d044d7b1cc701f89cf4142`, and the measured first-frame split above rules out PNG encoding and surface presentation as the source of the 9.17s first-frame delay.

The historical 2026-07-27 screenshot and RenderDoc capture remain historical evidence only. The tracked Standard-PBR-preview DX12 PNG remains accepted visual and startup-attribution evidence for that historical profile. The current-source environment-only PBR Release build, Debug compile, corrected Ready-frame decomposition, RenderDoc replay, and exact viewer CLI test are complete; its DX12 Ready-frame PNG and capture are recorded above. The earlier Text01 import result is a historical failed Release attempt, not a current Release requirement.

## 2026-08-01 Current-Worktree Follow-up (Not Yet Accepted)

The historical 82--100 second delay remains attributed to synchronous DX12 WGSL/PSO compilation. IBL restore was approximately 1.08 seconds in the original decomposition, while eager `DeferredSceneResources` construction reached approximately 91.74 seconds. Later environment-only measurements reduced scene construction to 8.28 seconds and Ready total to 19.21 seconds; PNG encoding, surface present, and HDR/PMREM cache restore were not dominant. The latest source also uses `PbrMirrorScene::render_to_viewport_surface` for normal interaction, so the CPU `ViewportFrame` readback is limited to explicit screenshot/fallback operation rather than every displayed frame.

The remaining Base material key does not receive shadows. Its Forward template previously still parsed the complete 7,850-character shadow atlas, cascade, point-face, and PCF module even though `ZR_FEATURE_RECEIVE_SHADOWS` was false. The current worktree keeps the canonical `zr_shadow.wgsl` token but selects a 165-character binding-free `zr_gpu_light_shadow_visibility` stub for that variant, a 97.9% module-source reduction. Variants carrying `ShaderFeatureBits::RECEIVE_SHADOWS` still receive the full module. The two specializations deliberately have different include content hashes, so shader prewarm/runtime cache identities cannot reuse the wrong source.

Evidence completed without the occupied Cargo lane: focused `rustfmt --check`, scoped `git diff --check`, 9 environment/lightmap ownership and GPU-binding Python contracts, and 62 shader-prewarm cache/provenance/resource/dimension contracts. The independent second review reported one Important issue in the first draft: shadow support was selected before pass dispatch, so non-Forward pass assembly still copied and hashed an unused full module. The forward fix moves shadow and the pre-existing volumetric selection inside the Forward arm; a post-fix source gate confirms neither is constructed before pass dispatch. No independent `naga` executable is installed. These static results do not replace a fresh managed Rust/WGSL test, DX12 timing run, screenshot, or RenderDoc 1.44 replay; all PNG/RDC artifacts linked earlier in this page predate this worktree specialization and remain historical baselines.

The broader non-Cargo prewarm sweep initially ran 128 tests and found four pre-shader errors: `native_dynamic_fixture` declared `distribution.assets = ["assets/**"]`, the files existed and were unchanged, but Python 3.12 `Path.glob` returned only the terminal directory and the validator rejected it after its file filter. A focused regression reproduced that diagnostic before the fix. The distribution-assets validation owner now normalizes only terminal `**` to recursive contents, leaving other patterns and all path, retired-UI, and `.zui` validation intact. Its focused asset suite passes 7/7 on Python 3.12, the recursive regression passes on Python 3.14, and the original prewarm set now reports 127 passed with one intentional skip. The real Python 3.12 `plugin validate native_dynamic_fixture --json` entry also returns `fatal=false` with no diagnostics, and `zircon_build.py --list-plugins` discovers all 38 plugins including the fixture. This closes the shader-prewarm asset-discovery infrastructure regression; it does not change the not-yet-accepted Cargo/WGSL, quantitative, DX12 screenshot, or RenderDoc gates above.

The expanded source-cost review found a second eager boundary after the pass specialization fix: `assemble_material_shader_template` still called the full builtin registry constructor, so an empty-root Base material and non-Forward passes paid to copy, scan, and hash unrelated shadow/volumetric modules. Runtime material assembly now extracts include roots before registry creation, returns immediately when there are no roots, and asks `ShaderModuleRegistry::with_builtin_modules_for_roots` to materialize only the reachable builtin/custom dependency closure. Deferred-lighting assembly has also moved to this root-scoped API; it supplies only the shading-model includes selected by the active deferred profile, while preserving custom source precedence and the full generic profile. IDE/all-builtin enumeration deliberately retains the complete registry. This is a source-construction optimization with a regression contract, not yet a new DX12 timing claim.

The distribution validator follow-up now also rejects Windows drive-relative (`C:outside/**`) and root-relative (`\\outside\\**`) forms using native and `PureWindowsPath` anchors and parent components before invoking the host glob implementation. This prevents Python 3.12 `NotImplementedError` from escaping and preserves root confinement on POSIX validation hosts. The focused asset tests pass 7/7 and the distribution owner set passes 31/31 on bundled Python 3.12; the checked-in fixture validates with `fatal=false`, and the build entry discovers 38 plugins. The prewarm aggregate remains 127 passed with one intentional skip. Fresh managed Cargo/WGSL, the four quantitative tests, DX12 screenshot/timing, and RenderDoc replay remain open; all earlier PNG/RDC artifacts on this page remain historical for the current worktree.

The final incremental independent review reports `Critical 0 / Important 0 / Minor 0`. It confirmed the environment-only profile does not construct the disabled-volumetric override, the roots condition is covered by a source contract, and this page matches the 7/7 and 31/31 test counts for that candidate. The subsequent environment PBR audit separates global and local reflection semantics: sky/source, SH/IEM, and PMREM use the global environment intensity, while reflection probes and planar reflections remain independently enabled. The initial unconditional non-positive-intensity return incorrectly skipped those local providers and received `Critical 0 / Important 1 / Minor 0`; the provider-aware fix returns before normalization and texture access only when global intensity is non-positive, the probe set is empty, and planar reflection is disabled. When a local provider keeps the path active, diffuse SH/IEM and the sky/PMREM remainder are independently skipped at non-positive global intensity, while planar/probe sampling, probe weights, and nonzero BRDF behavior remain unchanged. Exact-zero diffuse input now skips SH/IEM access, and exact-zero reflection radiance skips the BRDF LUT lookup; nonzero, negative, and NaN reflection values continue through the original split-sum path. The seven assembled-source environment contracts live in `template/tests/environment.rs`, keeping the root test owner below its 1,000-line ceiling; the new contract proves each texture operation is uniquely owned by its zero-contribution branch. Focused rustfmt, branch-ownership/source guards, scoped diff integrity, 9 environment/lightmap contracts, and the 127-passed/1-intentional-skip prewarm set pass. The post-fix independent review reports `Critical 0 / Important 0 / Minor 0`. The candidate remains not accepted until fresh managed Rust/WGSL and the product gates above complete.

## Ready-Frame Evidence Gate

The next current-source DX12 screenshot must be emitted with its matching
`.png.txt` v2 sidecar and checked before it is copied into this directory:

```powershell
python tools/zircon_validate_shader_pbr_viewer_evidence.py <ready.png> --expected-backend Dx12 --require-direct-present
```

The gate validates the bounded encoded PNG checksum/RGBA dimensions/visible-pixel
non-blankness and the sidecar's complete active-cubemap face-size/mip layout,
phase-duration hierarchy, environment-only PBR profile, and process-local
`MeshPipelineCache` interpretation of the Base-prewarm cache hit. It does not
start the viewer or measure GPU time. The historical PNGs in this record predate
the v2 sidecar and must remain historical baselines rather than being backfilled
or accepted by this new current-source gate.

## RenderDoc Replay Evidence Gate

Every new Debug capture must retain the viewer-reported lowercase `.rdc` artifact
and pass the time-bounded replay command before it is cited as current-source evidence:

```powershell
python tools/zircon_validate_shader_pbr_renderdoc_replay.py <capture.rdc>
```

The command invokes `D:\Tools\renderdoc\renderdoccmd.exe replay --loops 1` against a
verified temporary snapshot, rejects missing, empty, non-regular, and
non-lowercase-`.rdc` inputs, and prints the capture size plus SHA-256 alongside the
replay result. This establishes capture identity and replayability only; it does not
substitute for the matching v2 Ready-frame PNG gate or a fresh managed DX12 run.
