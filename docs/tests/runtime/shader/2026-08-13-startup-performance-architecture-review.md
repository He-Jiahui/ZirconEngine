# Shader PBR Startup Performance Architecture Review

Date: 2026-08-13

## Scope And Evidence Boundary

This review covers the Shader PBR viewer's current DX12 startup path before
any new algorithmic optimization. It does not treat the historical 82-second
observation as a current viewer load measurement: the preserved log records
the total duration of `export_procedural_realtime_ibl_mirror_cardinal_120deg_png`.
Separate source-bound DX12 launch measurements diagnose an older 82--100
second startup range, but they remain historical. No current-source viewer
binary is available under an approved E: output root, so a measured startup
claim requires the managed current-source profile workflow.

The executable profile protocol is implemented by:

- `tools/write_zircon_shader_pbr_build_provenance.ps1`
- `tools/zircon_profile_shader_pbr_viewer.ps1`
- `tools/zircon_summarize_shader_pbr_profile.py`

It records five isolated cold runs and five warm runs, validates the Ready
PNG/sidecar and matching GPU timestamp report, captures WPR CPU attribution
and optional energy/RenderDoc evidence, and keeps generated evidence below
this directory. The profile rejects a binary that is not bound to the current
critical-source manifest and a terminal coordinator validation ticket. The
ticket binds the canonical source-manifest hash; the viewer binary is still
locally fingerprinted.

## Findings

1. `runtime_shader_pbr_realtime_ibl_multiview_run_20260712.log` reports the
   one multiview PNG-export test at `82.01s`, not a phase-separated viewer
   startup. Separate `2026-07-11` viewer samples show the process reaching
   roughly `80.875` accumulated CPU seconds after load, but that older binary
   lacks the required timing breakdown. Neither artifact proves that current
   PMREM is the cause of a current loading delay. The test structure iterates
   five camera cases; each creates a temporary project, imports it, creates a
   `SceneRenderer`, renders a frame, and writes an image before the contact
   sheet. Its aggregate duration must therefore not be reported as one load.
2. Separate source-bound DX12 launch diagnostics in
   `zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_startup.md`
   identify the older 82--100 second startup bottleneck as synchronous Standard
   PBR PSO construction: the historical deferred phase was approximately
   91.74 seconds while IBL restore was approximately 1.08 seconds. The later
   cache-reused runs kept IBL at 595--846ms while Standard PSO remained above
   21 seconds. This evidence excludes IBL as the former dominant phase, but it
   must not be used to justify an unmeasured current IBL algorithm rewrite.
3. The current `EnvironmentOnlyPbrPreview` path already avoids immediate
   deferred fullscreen PSO construction and queues the exact Base mesh PSO on
   a background worker. Ready capture deliberately waits for that pipeline.
   Ready schema v12 separates shader/pipeline creation and async queue wait
   from IBL staging timing.
4. Runtime IBL bake PSOs are lazy. `IblBakeWgpuPipelineCache::new` creates
   layouts and a sampler only; shader modules and compute pipelines are
   created when a bake command is actually dispatched. CPU-imported HDRI
   staging therefore must not be charged with GPU bake compilation.
5. The EnvironmentOnly Forward WGSL removes direct-light, shadow, lightmap,
   cookie, irradiance-volume, transmission, and volumetric consumers. The Base
   mesh PSO now selects a matching layout with no group 1 forward receiver.
   `BaseScenePass` creates that receiver only when the EnvironmentOnly profile
   contains a generic command, while the ordinary renderer keeps its prior
   zero-scan eager path.
6. Zircon's persistent `RuntimePipelineCache` is intentionally Vulkan-only.
   It cannot improve DX12 PSO startup. Unreal's relevant architecture is its
   versioned PSO descriptor file cache plus bounded/usage-ranked background
   precompilation, not a backend-specific opaque blob.
7. The checked-in `polyhaven_lakes_2k.hdr` is `2048x1024`. Viewer preflight
   resolves that to a `512` source face and currently defaults PMREM to the
   same `512` result size. That is a 15.17x static PMREM work increase over
   the independent `128x8` baseline, not current-machine timing evidence.
   The profile runner now preserves `automatic` requests and active layout in
   every sidecar/report, so the default policy can be compared directly with
   an explicit `128` result layout without changing source resolution.

## Algorithm-Scale Audit (2026-08-15)

The current CPU PMREM implementation is GGX filtered importance sampling with
the canonical `32/64/128` fast/normal/rough sample budgets. Its parallel
executor is invoked once per mip and receives exactly six full-face outputs;
each mip completes before the next one begins. This preserves deterministic
face-major output, but it caps the largest mip's immediately available CPU
parallelism at six tasks.

For a `512` face PMREM with ten mips, the recipe-derived source-cubemap
trilinear-call upper bound is `18,417,408`. Mip zero has zero roughness, so
`ggx_prefilter_direction` takes its direct source-sample return before the
GGX loop; its declared Fast count is not executed. The bound consists of
`1,572,864` direct mip-zero calls and `16,844,544` importance or cosine
sample-loop calls. This is not an elapsed-time measurement, an actual texture
read count, or a power measurement: each trilinear call may access multiple
source texels and positive-NdotL rejection reduces sampled work on nonzero
roughness mips.

| PMREM layout | Source-cubemap trilinear-call upper bound | Share of 512 total |
| --- | ---: | ---: |
| 512 mip 0, six 512x512 faces, direct source sample | 1,572,864 | 8.539% |
| 512 mip 1, six 256x256 faces, 32 samples | 12,582,912 | 68.322% |
| 512 mip 2 through 9 | 4,261,632 | 23.139% |
| 512 total | 18,417,408 | 100% |
| 128 total, eight mips | 1,214,208 | 6.593% |

Consequently, `512` versus `128` is a `15.168x` source-sample-call comparison
under the current roughness mapping, while the first two 512 mips account for
`76.861%` of the 512 bound. This corrects the previous record, which multiplied
mip zero by its declared Fast sample count despite the zero-roughness fast
path. Neither static figure establishes a measured bottleneck.

cmft creates `(mip, face)` filter tasks as well, but runs them through its own
CPU worker pool and optional OpenCL worker. cmftStudio publishes the filtered
environment only after its background task reaches `Completed | ExitSuccess`,
then creates GPU buffers on the main thread. Zircon must retain the equivalent
separation: artifact staging/hydration owns IBL timing, while viewer-local
exposure persistence, GPU upload, and UI presentation remain outside it.

No PMREM resolution reduction or row/tile parallelization is authorized by
this audit alone. First capture the two required current-source five-cold and
five-warm matrices and inspect WPR plus v12 staging attribution. If CPU PMREM
is both material and underutilized, evaluate row-bounded tasks for the top
mips while preserving direct final-storage writes, deterministic output order,
serial/parallel byte equality, cache identity, and the existing screenshot
quality gates. Artifact publication is a separate correctness concern: source
`.zcube` and importer-derived `.zribl` now publish as one recovery-backed
bundle transaction, and readers hold the same publication owner at their final
source-miss barrier. Keep that settled-generation contract separate from any
future throughput candidate.

## Current-Source Probe Scaling Audit (2026-08-16)

The retained RenderDoc captures do not measure the current probe path, so this
is a static source-cost audit rather than GPU timing evidence. The full PBR
environment shader calls `zr_environment_select_probes` per shaded fragment.
That function loops from zero to `probe_count`, where the renderer's current
resource capacity is fixed at `MAX_REFLECTION_PROBES = 64`, then ranks the best
two candidates. The packed layer mask is written to `GpuReflectionProbe.misc.w`
but the shader has no object reflection-mask input and does not read it.

| Output size | Full 64-probe selection iterations/frame | At 60 fps |
| --- | ---: | ---: |
| 1920x1080 | 132,710,400 | 7,962,624,000 / second |
| 3840x2160 | 530,841,600 | 31,850,496,000 / second |

Each iteration can perform an enable test, a sphere-distance or rotated-box
test, blend/priority ranking, and later the selected candidates require
projection and cubemap work. The count intentionally excludes those costs.
At 1,000 resident probes, the same 1080p algorithm would rise to
2,073,600,000 iterations per frame, so a higher fixed cap, loop unrolling, or
WGSL branch rearrangement is not a viable scale path.

The current resource reservation has the same fixed-scale problem: 64
`128x128` eight-mip six-face RGBA16F cubemaps reserve 67,107,840 bytes, and the
fixed 1024-square eleven-mip planar RGBA16F chain reserves 11,184,808 bytes,
before source/derived artifacts or upload staging. The replacement is the M9
`ReflectionProbeSpatialAssignment` ABI recorded in the 09F1 plan: a
visibility-owned cluster offset/count table and packed local indices, object
reflection-mask filtering, explicit overflow, and generation-driven rebuilds.
Forward, deferred, and generated PBR must use one lookup module. A cluster
overflow must be observable and choose an explicit fallback, never silently
revert to a global scan.

Required future GPU evidence is the distribution of per-cluster list lengths,
overflow count, fragment visits, upload bytes, VRAM residency, and environment
pass time for 1/64/1k probes at 1080p and 4K. Do not interpret the static
counts as an elapsed time or energy measurement.

## Realtime IBL Bootstrap Scheduling Audit (2026-08-16)

This is a current-source dispatch-shape audit, not a measured GPU-time result.
The scheduler's first request takes a `published_key == None` full-update path
instead of its normal time-sliced state progression. With the default
`128x128`, eight-mip source/PMREM recipe and `8x8` compute workgroups, that
path records the following work in one frame:

| Operation | Compute workgroups |
| --- | ---: |
| `CaptureSky`, six 128x128 faces | 1,536 |
| `CaptureCloud`, six 128x128 faces | 1,536 |
| source mip generation, 64 through 1 | 528 |
| PMREM mip generation, including one terminal all-face average dispatch | 2,059 |
| diffuse SH9 | 96 |
| total bootstrap record | 5,755 |

`CaptureCloud` currently invokes the same gradient capture routine and writes
the same source mip-zero target as `CaptureSky`; it is an overwrite and 1,536
avoidable workgroups, not cloud lighting. The PMREM total differs from the
naive six-face sum because the terminal 1x1 mip dispatch uses `z = 1` and
writes the average to every face.

The fix is architectural: one budgeted generation ticket for first and later
updates, explicit last-good/fallback semantics, no cloud node without a cloud
radiance producer, recipe-keyed compiled dispatch templates, and per-operation
timestamps. Required dynamic evidence is the scheduled/completed work and GPU
time for every operation, first-ready hitch distribution, stale/fallback age,
graph/bind allocation count on unchanged frames, and the effect on
presentation-only dynamic-resolution timing. These static counts do not prove
that IBL caused the historical 82-second export or the older PSO delay.

The same run must report graph-cache hits/misses/evictions and persistent
parameter/bind allocation counts. The current cache is a linear `Vec` with a
34-variant assertion derived only from the old eight-mip/two-face schedule;
the scheduler accepts other valid face quotas and mip counts. Current
timestamps bracket the whole batch with two queries, so they cannot support a
per-operation budget decision. Both are source facts, not timing measurements.

The current double-buffer resource set also has a fixed default-layout cost:
one `128x128`, eight-mip, six-face RGBA16F cubemap is 1,048,560 bytes, so two
slots each holding source plus PMREM allocate 4,194,240 texture bytes. The two
nine-coefficient RGB SH9 buffers add 288 bytes, for a calculable lower bound
of 4,194,528 bytes before texture-view, bind-group, graph and driver allocation
overhead. `RealtimeIblRuntime` creates that bundle only for its first default
request and retains it while resources are present; it carries neither a
quality-profile nor device-generation identity. M7 must replace it with a
recipe/device-keyed resource bundle that retires old generations only after
in-flight work completes. Capture the actual allocated/resident bytes and
retirement count at every quality or device transition; do not represent this
static lower bound as VRAM telemetry.

## Ordered Plan

1. Implement a dedicated EnvironmentOnly Base pipeline layout with group 0
   scene, no group 1 forward receiver, group 2 material, and group 3 GPU scene.
   Lazily create the generic forward receiver bind group only for a command
   whose pipeline variant requires it. Keep generic/local-provider variants on
   the existing ABI.
2. Measure current cold/warm DX12 runs. Attribute the remaining Ready delay
   using v12 creation/queue gauges and WPR stacks before broadening the change.
3. Only if PSO creation remains material, design a versioned backend-neutral
   PSO descriptor manifest: exact pipeline key/source hash/adapter key,
   atomic persistence, invalidation on source or device change, and bounded
   background precompile. Do not persist WGPU handles or claim driver-cache
   reuse on DX12 without a supported backend API.
4. Keep direct IEM unchanged until the M8 CPU/GPU error and throughput data
   exists. Its bounded 37,748,736 candidate iteration shape is a separate
   canonical-quality decision, not evidence for the historical DX12 stall.

## Required Measurements

- Median and range of cold/warm time-to-Ready.
- Base shader module and render pipeline creation count/time.
- Successful async admission queue wait count/time.
- CPU WPR attribution for WGPU, DX12 driver, and shader compilation stacks.
- GPU direct-stage timings from the Ready-matched timestamp report.
- Energy samples only when the platform counter reports an explicit watt unit.
- Two independent five-cold/five-warm matrices for the same HDRI: viewer
  defaults (omit both layout options) and automatic source plus explicit
  `--pmrem-face-size 128`. Compare the active layouts, `pmrem_build`, cache
  output bytes, screenshots, WPR stacks, GPU evidence, and energy samples.
- Per-run IBL parallel chunk submissions for equirectangular projection,
  source mip generation, PMREM, and irradiance-cube build. The profile rejects
  an aggregate that does not equal the sum of those phases; these are dispatch
  shapes, not worker utilization or elapsed-time measurements.

An independent PMREM default is a candidate to reduce CPU bake work, not an
implemented conclusion or a claim that it removes a driver PSO stall. Both
decisions require the measurements above.

## Historical Baseline Sanity Check (2026-08-15)

The local `D:\Tools\renderdoc\renderdoccmd.exe` v1.44 replayed
`zircon_shader_pbr_viewer_m5_environment_only_pbr_20260729_dx12_renderdoc_capture.rdc`
with `--loops 1` and exit code zero. Its paired PNG SHA-256 is
`B9341A9215A1854C7E39A76A0FA077F789BC288B6F73F2954CB7A6A1382EE390`.
Visual inspection shows the Lakes sky, horizon, and the expected mirror-sphere
reflection without desktop contamination. This checks the local RenderDoc
installation and retained historical baseline only; it is not current-source
timing, screenshot, or replay acceptance evidence.

## Remaining Infrastructure Requirement

The coordinator validation-ticket schema records source manifests and terminal
command evidence, but does not publish a built viewer-artifact fingerprint.
The profile therefore requires a passed ticket and verifies the current source
set before capture, while retaining the viewer's local SHA-256 in the capture
manifest. A future managed viewer-build action must emit binary path, SHA-256,
byte length, source-manifest hash, and ticket ID as one coordinator-owned
artifact before cross-machine profiling acceptance is enabled. Until then,
this profile is source-bound local diagnostic evidence, not a substitute for
an integrated managed build receipt.

For the current local diagnostic workflow, first obtain a terminal, passed
Shader06 validation ticket and its matching Tooling01 managed viewer-artifact
receipt through the coordinator. Then create the capture provenance explicitly,
followed by the five-cold/five-warm profiler run:

```powershell
pwsh .\tools\write_zircon_shader_pbr_build_provenance.ps1 `
  -ViewerExe E:\ZirconBuilds\shader-pbr\zircon_shader_pbr_viewer.exe `
  -OutputPath E:\ZirconBuilds\shader-pbr\viewer-capture-provenance.json `
  -ValidationTicketId <passed-ticket-id> `
  -ArtifactReceiptId <passed-artifact-receipt-id>

pwsh .\tools\zircon_profile_shader_pbr_viewer.ps1 `
  -ViewerExe E:\ZirconBuilds\shader-pbr\zircon_shader_pbr_viewer.exe `
  -HdriPath E:\ZirconBuilds\shader-pbr\studio.hdr `
  -BuildProvenance E:\ZirconBuilds\shader-pbr\viewer-capture-provenance.json

# Same source resolution, independent 128 PMREM comparison matrix.
pwsh .\tools\zircon_profile_shader_pbr_viewer.ps1 `
  -ViewerExe E:\ZirconBuilds\shader-pbr\zircon_shader_pbr_viewer.exe `
  -HdriPath E:\ZirconBuilds\shader-pbr\studio.hdr `
  -BuildProvenance E:\ZirconBuilds\shader-pbr\viewer-capture-provenance.json `
  -PmremFaceSize 128
```

The profiler queries the supplied ticket once to reject a stale, absent, or
nonterminal source validation record. It never polls queued/running work. An
omitted layout parameter is bound as `automatic`, while the active layout must
remain consistent across the measured matrix; an explicit value must also match
the active layout exactly.

## Static Verification

2026-08-13 static verification completed:

- `python -m unittest tools.tests.test_zircon_summarize_shader_pbr_profile`
  passed 17 tests, including source-ticket hash/status, Ready evidence
  replay rejection cases, layout-manifest binding, automatic-layout handling,
  and rejection of a mismatched IBL phase-submission total.
- `Invoke-Pester -Path tools/tests/zircon_profile_shader_pbr_viewer.Tests.ps1`
  passed 10 tests.
- PowerShell parser checks, Rust `rustfmt --emit stdout` parsing for the
  modified pipeline sources, and `git diff --check` for this scope passed.

No Cargo/WGPU execution, current-source viewer capture, WPR trace, RenderDoc
capture, or screenshot was run in this session. Those remain measurement
work, not accepted rendering evidence.
