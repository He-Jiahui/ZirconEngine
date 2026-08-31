# Shader06 Realtime SH9 Dispatch Parity Performance Plan

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Scope: terminal `ProjectDiffuseSh9` graph, shader command plan, WGPU recorder parity
Status: implementation_complete_pending_managed_validation
Date: 2026-08-22
Last updated: 2026-08-25

## Decision

The terminal realtime SH9 operation is a single 8x8x1 workgroup reduction.
Its dispatch shape is now owned by
`IBL_BAKE_IRRADIANCE_SH9_DISPATCH_GROUPS` in `ibl_bake_graph_plan`, the shared
graph/execution contract. The offline graph, shader command plan, and realtime
graph all consume that constant. The WGPU recorder already encodes the shader
command plan, and a regression test compares that encoded command with the
compiled realtime graph.

## 2026-08-23 cmft Architecture Review

`dev/cmft/src/cmft/cubemapfilter.cpp` is a useful offline scheduling reference,
not a material-integrator replacement. Its `imageRadianceFilter` exposes a
Blinn/Phong lighting model, gloss scale/bias, optional base-mip exclusion, edge
fixup, and a CPU/OpenCL task list per mip and cube face. `dev/cmftStudio` keeps
that filtering in a background job with separate input/output images and only
publishes the completed result. Zircon retains the reusable engineering parts:
immutable source/output ownership, explicit mip/face work, and publication
after a complete result. Its corresponding owners are the canonical GGX/FIS
recipe, `RealtimeIblTimeSliceScheduler`, and the non-sampled A/B resource slot.

Copying cmft's Blinn/Phong or gloss controls into the standard PBR path would
break the established GGX PMREM, BRDF-LUT, and roughness contract. No such
algorithm substitution is justified. The structural P1-2 direction is instead
to keep each slot's physical views and per-command immutable parameters alive
across sky-content revisions; the current third-generation B-slot evidence is
the acceptance target for that decision.

Current-source recheck explicitly rejects the stale `CaptureCloud` duplicate
work hypothesis in the broader optimization inventory. The scheduler has one
`CaptureSky` stage only; the default two-face budget produces three capture
operations for the six cube faces, and the compiled graph test asserts that no
cloud capture pass exists without a distinct cloud producer. This is not an
optimization candidate until a real cloud-radiance owner is introduced.

## 2026-08-23 Unreal GGX Sampling Review

`dev/UnrealEngine/Engine/Shaders/Private/MonteCarlo.ush` expresses the generic
GGX light-direction PDF as `D * NoH / (4 * VoH)`. Zircon's cubemap prefilter
uses the documented `V = N` assumption, so `VoH = NoH` and this reduces to the
existing `D * 0.25` in both the CPU reference and WGSL recorder path. Its
filtered-importance source LOD remains `0.5 * log2(omega_sample /
omega_pixel)`, using the same solid-angle factor and the canonical
roughness-to-mip inverse mapping described by the parent plan. The deferred
material path samples that PMREM with `BRDF LUT(NoV, roughness)`; it therefore
retains the same split-sum contract as Unreal's `PreIntegratedGF` path.

Zircon's centred deterministic Hammersley sequence and `E.y *= 0.995` tail
guard are intentionally not copied from Unreal's generic sampling call. CPU
and GPU share those choices today, and the current PBR matrix has shown no
observable discontinuity that would justify a content-changing PMREM revision.
Any future change to either choice requires a fixed-HDRI CPU/GPU/reference
error comparison (including outlier and seam metrics), a fresh RenderDoc
capture, and an `IBL_BAKE_ALGORITHM_VERSION` increase in the same change. No
unmeasured PMREM-math optimization follows from this review.

A companion regression walks every scheduled PMREM slice and compares its
compiled graph dispatch with the WGPU command plan. It covers the three
face-chunked base-mip tickets, four full-face higher-mip tickets, and the
single terminal-average `[1,1,1]` dispatch.

The constant belongs to the graph/execution contract rather than the shader
plan. This preserves the existing one-way dependency from shader command
planning to graph planning and does not add a graph-to-shader dependency.

`realtime_sh9_kernel_command_matches_the_full_request_without_readback` also
compares the complete runtime SH9 command to the full-request command after
clearing only offline readback descriptions. This locks the shader source,
parameters, output binding, pipeline key, and dispatch as one execution
contract while preserving the runtime's no-readback lifecycle.

## Pre-Implementation Review

`dev/cmft/src/cmft/cubemapfilter.cpp` schedules radiance-filter work as
resumable task lists, so partial progress and completed work remain distinct.
Unreal's `ReflectionEnvironmentRealTimeCapture.cpp` similarly publishes a
double-buffered realtime capture only after its time-sliced state completes.
Zircon therefore keeps its existing ticket scheduler and fixes the shared
dispatch description; it does not copy Unreal's state count or add a second
scheduler.

The current default Zircon ticket has 21 physical batches: three sky-capture
batches, seven source-mip batches, ten PMREM batches, and one terminal SH9
batch. `ProjectDiffuseSh9` is the final batch and its WGSL reduction uses one
64-lane workgroup. The former realtime graph independently reported `[4,4,6]`
(96 groups), while the encoded command actually used `[1,1,1]`.

## Static Cost Finding

The previous graph budget overstated a full default ticket by 95 workgroups:

| Component | Workgroups |
| --- | ---: |
| Capture | 1536 |
| Source mips | 528 |
| PMREM | 2059 |
| Terminal SH9 | 1 |
| Correct total | 4124 |
| Previous reported total | 4219 |

The correction is `95 / 4124 = 2.30%` of the corrected static ticket budget.
It fixes graph metadata, scheduling visibility, and profiling attribution. It
does not remove GPU work because the WGPU command was already dispatching one
group. No GPU-time, energy, or power improvement is claimed from this change.

## 2026-08-25 Execution-Resource Cache Architecture Review

### CPU Observation Plan

The execution-resource cache already owns cumulative test-only hit, miss,
validation, and entry statistics, but the accepted runtime CPU sidecar only
exposes zero binding counts on a reuse. That is insufficient to distinguish a
cache hit from a cold path which happens to bind no optional resources, and it
does not let an exported capture establish whether a topology was materialized
or reused.

Before changing the cache, the reviewed report path is:
`RealtimeIblExecutionResourceCache::resolve` ->
`RealtimeIblGraphPreparationReport` ->
`RealtimeIblCpuTimingReport` -> the CPU command-recording sidecar. The GPU
timestamp metadata is intentionally outside this path and must remain free of
CPU cache fields.

The minimal observation contract is one hit-or-miss count plus the current
entry-count and topology-capacity snapshots for every recorded batch. The cache
resolves those values from its existing branch, `HashMap::len`, and the already
configured scheduler capacity; the aggregate report sums hits and misses and
keeps the maximum entry count and capacity. This adds fixed-size scalar copies
only, starts no clocks, allocates no memory, and does not alter graph
compilation, WGPU command encoding, A/B slot selection, or shader dispatch.

Managed Windows profiling must later show exactly one execution-resource cache
outcome per accepted CPU sample, at least one miss for a cold topology, reuse
after the topology repeats, and an entry peak no greater than the scheduler's
topology capacity. Until that capture exists, this is observability work only;
it makes no CPU-time, GPU-time, energy, or power-improvement claim.

The compiled-graph cache retained the immutable `CompiledRenderGraph`, but the
runtime still created a new `RenderGraphExecutionResources`, copied stable
WGPU view handles into `BTreeMap` entries, and re-ran materialization validation
for every active realtime-IBL batch. Those are CPU control-plane operations;
they do not create a WGPU texture, change a bind group, or observe sky pixels.
The resource names, slot ownership, and physical views are fixed for a
`(ready_slot, work_slot, operation)` topology under one source/PMREM layout.

`RealtimeIblGpuResources` now owns an execution-resource cache for the same
device-owned A/B allocations. A miss binds the live compiled resource names and
validates the graph exactly once; a hit returns that immutable lookup table with
zero binding count, binding time, and validation time. A source/PMREM layout or
scheduler-capacity change clears the entries. The cache stores neither radiance
data nor command parameters, so it cannot retain an old sky revision;
recorder-side parameter and bind-group caching remains a separate owner.

The scheduler's default shape is 21 batches per generation and two slot
topologies, hence at most 42 execution-resource entries. For 256 same-layout
generations, the prior upper bound was `256 * 21 = 5,376` resource-table
materializations and validations. The new bound is 42, removing 5,334 such
operations (`99.21875%`) after the two-slot warmup. This is a static count, not
a wall-time, GPU-time, power, or cross-engine efficiency claim. The execution
resource cache regression checks identity reuse after a sky-key change and
records one miss, one validation, and one hit; it requires managed Rust/WGPU
execution before it can become runtime evidence.

## 2026-08-25 Shared Realtime Graph Topology Identity Review

The compiled-graph cache and the execution-resource cache both select entries
by ready slot, work slot, and one time-sliced operation, while separately
resetting on the request layout. The execution-resource cache is not a second
WGPU command path: it replaces the prior per-batch construction, physical-view
binding, and validation of `RenderGraphExecutionResources`; the recorder still
uses the same device-owned views and command plan. Comparing the pre-cache
path confirms that no materialized-resource ownership is transferred to a sky
generation or across device allocations.

Before this review those two caches declared structurally identical private
keys. That duplication recreates the same design failure that produced the
SH9 dispatch mismatch: a future field can be added to one cache and omitted
from the other. The graph-plan owner must therefore publish one
`RealtimeIblGraphTopologyKey`, and both cache owners must consume it. The
key includes both A/B slots even for a repeated logical operation, because a
completed generation swaps source/PMREM/SH9 physical allocations before the
next capture begins.

This is an identity/convergence repair, not a change to the 4,124-workgroup
GPU schedule. Its static cost is one shared small value key per cache lookup;
it removes no GPU command, does not change bind-group reuse, and has no
measured wall-time, energy, or power result. The existing managed five-cold /
five-warm profile and GPU timestamp evidence remain the required performance
proof. A source regression must show that the same operation reuses its key
within one ticket but differs after the A/B ownership swap.

## 2026-08-25 Environment-Only Forward Normal Contract

The optimized environment-only Forward specialization removes direct-lighting
modules, but it cannot waive the normalized-input ABI of its IBL helper. Vertex
normals are normalized before interpolation, so the fragment input must be
normalized again before using it as a reflection normal or in `N dot V`.
`shade_forward` now computes one zero-safe `world_normal`, passes it to
`zr_environment_pbr_indirect`, and reuses it for the ambient diffuse Fresnel
term. The environment-only helper continues to consume the caller-normalized
value directly, avoiding a second per-pixel normalization.

This restores equivalence with Standard PBR's Forward normalization boundary
without adding texture sampling, bindings, or a runtime branch. The specialized
source-contract regression rejects a raw `surface.normal_ws` IBL call and
requires the one prepared normal before that call. Rust formatting, scoped diff,
and source-contract checks pass; managed Rust/WGPU compilation plus current
screenshot and timing validation remain pending, so the plan status is
unchanged.

The same ABI review leaves the environment-only helper's `vec3<f32>(0.04)`
dielectric F0 as an intentional baseline-only contract. The material streamer
sets `PipelineKey::pbr_ior_override` whenever a material derives a non-default
dielectric F0, and the variant registry then keeps that material on the generic
Forward PBR path, which carries `surface.dielectric_f0`. Custom surfaces also
retain the generic closure. The deferred environment-only GBuffer has no F0
channel, so widening this specialization would require an explicit deferred ABI
project rather than a local shader edit.

## 2026-08-24 M6 Shader/PSO Metric Boundary Recheck

The current-source audit enumerated all seven production mesh-cache
`ensure_*` shader-module creation paths and the runtime prewarm-manifest path.
Each records a shader-module metric immediately after its actual WGPU creation
call, and records a render-pipeline metric immediately after its actual WGPU
pipeline creation call; the async Base worker records queue wait when its
admitted job begins, before it performs the same creation accounting. The
Ready sidecar reads only cached-map lengths and a locked cumulative snapshot;
it does not rescan the variant registry or add work to cache-hit draws.

`prewarm_pipeline_validation` deliberately remains outside this counter: its
temporary module and pipeline are used only by its test callers and are not a
Viewer production-startup path. The reported scope is therefore explicitly
`MeshPipelineCache` live objects and creation calls, not every WGPU object in
the renderer. This provides a sound CPU-side attribution boundary for a
future adapter-specific baseline, but it is not a WPR measurement and does
not justify a cache, scheduling, shader-variant, PMREM, or power optimization
without the managed cold/warm product captures below.

## Shared PMREM Dispatch Contract

PMREM face-range dispatches now use the backend-neutral
`ibl_bake_pmrem_dispatch_groups_for_face_range` helper in
`ibl_bake_graph_plan`. The realtime graph and WGPU slice builder both consume
it, while the WGPU layer retains only its shader-parameter adjustment for a
partial-face or non-terminal write. This removes the second `z`-dimension
derivation without making the render graph depend on the WGPU command module.

The same source audit found CaptureSky and source-mip formulas presently match
their WGPU recording formulas: 8x8 groups over the destination face size, with
the ticket face count for capture and six layers for downsampling. Their
recorder report currently echoes graph metadata, however, so it is not a
second execution-path proof. The product RenderDoc gate must inspect all four
compute labels before treating those two stages as measured parity.

## Measurement Protocol

After the shared `zircon_runtime` compile baseline is restored:

Before changing recorder allocation or caching, capture the separate CPU
recording baseline with the ignored
`profile_realtime_ibl_capture_and_source_mip_binding_encoding` test through the
managed validation wrapper. Its one cold ticket plus 255 warm tickets must emit
the concrete WGPU adapter name, backend, vendor ID, device ID, and adapter type,
together with separate capture, source-mip, and PMREM/SH9 parameter-buffer,
bind-group, cache-hit/miss, and binding-creation-time counters. The current
ticket topology is fixed at 3 capture + 7 source-mip + 10 PMREM + 1 SH9
operations. This profile finishes encoders without submitting them, so it
measures CPU command/binding recording only; it must not be used as GPU time,
throughput, power, or rendered-image evidence. A cache proposal is eligible
only when this named-adapter baseline identifies avoidable stable-slot work and
the dispatch/resource identity review proves that reuse cannot retain stale
sky radiance.

1. Run the managed `realtime_sh9_graph_workload_matches_the_encoded_sh9_command`
   and `realtime_pmrem_graph_workloads_match_encoded_commands_for_every_scheduled_slice`
   library tests. They must observe the terminal SH9 operation, every scheduled
   PMREM slice, their fixed graph dispatches, and equality with the encoded
   commands.
2. Run the ignored product export with the `profiling` feature,
    no pre-existing live core capture, and
    `ZR_RENDERDOC_CAPTURE_REALTIME_IBL_FINAL_SH9=1` through the managed
    validation wrapper. It must write the current PNG,
    `runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_reflection_20260823_p1_2.png`,
    the frame-submission and GPU timestamp sidecars, the CPU command-recording
    sidecar `runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_cpu_timing_20260824.txt`,
    and the requested RenderDoc capture under `docs/tests/runtime/shader`.
3. Inspect the PNG, record file sizes and SHA-256 hashes, and replay the newly
   generated RDC with `D:\Tools\renderdoc\renderdoccmd.exe replay`.
4. Record all 63 GPU timestamp samples from the three 21-slice generations.
    Run five independent three-generation captures before comparing median and
    p95 values. Report only measured values, the adapter/backend, and the
    capture configuration.
5. Measure power only with an external, device-specific counter available for
   the same capture. In its absence, report power as not measured rather than
   extrapolating it from GPU timestamps.

## Historical Blockers

Managed Windows validation job `35a4d3987f3f4588b06d7f76213c9468` used an
ephemeral target under `F:\cargo-targets`. Its production `cargo build -p
zircon_runtime --locked` stage passed in 29m12s. The selected PMREM library
test did not run because the crate-wide `cfg(test)` build stopped on 26 external
UI/Text test errors, including missing `TARGET_BINDING_COUNT`, missing
`measure_line_width`, obsolete `UiAssetLoader::load_str` calls, stale
`UiCompiledDocument::root` accesses, and an obsolete `BindingTransaction::commit`
call. These are outside Shader06 ownership; the prior Runtime07 initializer
error is no longer in this validation output.

Product validation job `5dbd283533c647bdab2678c7bf7faccc` started the ignored
`runtime_shader_pbr_realtime_ibl_export` target with
`ZR_RENDERDOC_CAPTURE_REALTIME_IBL_FINAL_SH9=1` and RenderDoc 1.44 on `PATH`.
It made no progress past `Updating crates.io index`. A direct request to
`https://index.crates.io/config.json` later returned HTTP 200 in 1.312 seconds,
but Cargo registry dependency-index requests remained transport-blocked. The
controlled retry `9d7e80c4f7894b8988035174099ecd35` forced the sparse registry
protocol, disabled HTTP multiplexing, limited HTTP timeout to 30 seconds, and
limited network retry to one. It then reported repeated `[28] Timeout was
reached` failures after 30 seconds with zero bytes received. The product jobs
were stopped to release their ephemeral lanes. No product image, CPU or GPU
timing report, or RenderDoc capture was generated. Retry the same managed
product command only after Cargo can fetch its registry dependency index, then
inspect the generated PNG and replay the RDC before closing this failure.

The later managed retry used a fresh ephemeral target and Cargo Home under
`F:\cargo-targets` (`7e0ae6ed29c441fda32fd0cfa7cc2c57`), so it did not create
an artifact on `C:`. It also stopped before product execution: Cargo timed out
after 30 seconds with zero bytes while downloading the sparse index entry for
`crossbeam-utils`. This independently reproduces the registry transport block;
no PNG, CPU/GPU timing report, or RenderDoc capture was emitted.

A subsequent managed reusable-pool validation compiled the ignored product
target in 13m57s under
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`.
The test process then failed after 31.93s. Re-running the already-built test
binary with `--nocapture` reproduced the product assertion at
`runtime_shader_pbr_realtime_ibl_export.rs:753`: the final capture reported
zero GPU timestamp reports where the two 21-slice generations require 42.

Further review on 2026-08-23 invalidated the initial queue-drain hypothesis.
`capture_frame` already calls `wait_for_readback_completions`, which waits for
the device and drains the shared queue before the product test extracts timing
reports. The zero reports instead come from the compiled-scene render path:
it passed `gpu_pass_timer.is_some()` into realtime IBL recording. The product
export does not enable the unrelated scene pass timer, so it never encoded the
two timestamp writes or their 16-byte resolve/readback despite the realtime
IBL collector reporting device support.

The compiled MVP path now enables timestamp recording from
`RealtimeIblRuntime::gpu_timestamps_supported()` itself. This preserves the
bounded three-slot queue and adds the existing timestamp design's two query
writes plus one 16-byte resolve/readback only for a realtime IBL batch. The
legacy direct-scene path has the same old generic-timer gate, but its source is
currently owned by an active external session and is not modified here; it
requires a follow-up parity repair before declaring the two renderer paths
semantically identical. Product validation must prove the compiled path yields
all 42 reports and enables the image, timing, and RenderDoc gates.

On 2026-08-23, managed Windows product validation retried through the reusable
pool at
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`.
Its `cargo build -p zircon_runtime --locked` stage stopped after 8m15s before
the export test could start. The first reported hard error is external to
Shader06: `ui/v2/component_instancer.rs` imports the private
`ui::template::asset` module after `ui/template/mod.rs` declares it as
`mod asset`. Rust reported three errors in total (`E0603` and `E0282`) and 215
warnings; no diagnostic identified the realtime-IBL timing-drain boundary.
The subsequent selected test command could not compile the same crate, so it
did not create a current PNG, CPU/GPU timing report, or RenderDoc capture.
This remains a compilation-bound validation blocker, not evidence that the
queue-drain repair has passed or failed at runtime.

## 2026-08-23 Product Validation Result

The later external UI failure was repaired outside this Shader06 scope. The
managed Windows product command then completed successfully with the
`D:\\cargo-targets\\zircon-engine\\pool\\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`
target lane:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 `
  -Package zircon_runtime `
  -TestTarget runtime_shader_pbr_realtime_ibl_export `
  -TestFilter export_procedural_realtime_ibl_pbr_matrix_png `
  -IgnoredTests
```

`Cargo build` and the selected ignored product test both passed. The test
published the required 8x8 PBR matrix PNG and reported GPU timestamp support
with exactly 42 operation samples: two complete 21-batch realtime IBL ticket
generations. The final report contains 8248 scheduled workgroups, equal to two
times the corrected 4124-workgroup ticket budget. In particular, frames 21
and 42 are the two terminal `diffuse_sh9` batches and each records one
scheduled workgroup. This is the runtime counterpart to the graph/command
contract repair; the prior product failure reported zero samples.

The P1-1 RenderDoc evidence files are:

| Artifact | SHA-256 | Notes |
| --- | --- | --- |
| `docs/tests/runtime/shader/runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_reflection_20260819.png` | `4F3967C321222618C60C7A9E2BDAB8B92E522FB37118B18BE3690C5810F5D473` | Visually inspected: complete 8x8 metallic/roughness sphere matrix, continuous sky/ground reflection, no blank or overlapping tiles. |
| `docs/tests/runtime/shader/runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_timing_20260819.txt` | `EF8E2CFA1468259BD38DF80C12E89B7F7E3526B7E6142ED424D7E61F65198BDC` | Current RenderDoc-injected revalidation CPU sidecar; not a baseline performance sample. |
| `docs/tests/runtime/shader/runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_gpu_timing_20260819.txt` | `A80BE411498DF37DCEB4204B87C5153C2BB254DC44622ADC6EFFAF78BF8F4521` | Current RenderDoc-injected 42-sample GPU sidecar; not a baseline performance sample. |

RenderDoc 1.44 was invoked with an explicit E: capture template and the
product test's `ZR_RENDERDOC_CAPTURE_REALTIME_IBL_FINAL_SH9=1` final-SH9
capture request:

```powershell
D:\\Tools\\renderdoc\\renderdoccmd.exe capture `
  --working-dir E:\\Git\\ZirconEngine `
  --capture-file E:\\Git\\ZirconEngine\\docs\\tests\\runtime\\shader\\runtime_shader_pbr_realtime_ibl_final_sh9_20260823_r1 `
  --wait-for-exit <product-test-binary> export_procedural_realtime_ibl_pbr_matrix_png --exact --ignored --nocapture

D:\\Tools\\renderdoc\\renderdoccmd.exe replay --loops 1 `
  E:\\Git\\ZirconEngine\\docs\\tests\\runtime\\shader\\runtime_shader_pbr_realtime_ibl_final_sh9_20260823_r1_capture.rdc
```

The P1-1 capture test passed in 65.87s. The produced RDC is
`docs/tests/runtime/shader/runtime_shader_pbr_realtime_ibl_final_sh9_20260823_r1_capture.rdc`
(23,661,068 bytes, SHA-256
`969F967A99F43AAE34D0D22738DE159E4983F39FEB7163A0FEBEAA9E206E0AFB`).
`renderdoccmd replay --loops 1` returned exit code 0 after local replay. No
new Cargo or RenderDoc artifact was written under C:.

## Five-Run Baseline Measurements

Five independent, non-RenderDoc-injected executions of the same already-built
product test each passed and each produced all 42 timestamp samples. The table
records the test's per-ticket CPU scheduling average and its GPU-operation
summary; `wall_s` is only the process duration, not a GPU performance claim.

| Run | wall_s | Initial CPU avg ms | Updated CPU avg ms | GPU samples | GPU operation avg ms | GPU operation max ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 39.440 | 604.305 | 540.965 | 42 | 0.043106 | 0.291840 |
| 2 | 40.916 | 651.987 | 588.628 | 42 | 0.068876 | 1.112064 |
| 3 | 39.035 | 534.580 | 547.942 | 42 | 0.043398 | 0.293888 |
| 4 | 41.012 | 629.962 | 658.499 | 42 | 0.043422 | 0.291840 |
| 5 | 44.003 | 636.189 | 587.167 | 42 | 0.043179 | 0.292864 |

Using nearest-rank percentile selection over those five independent ticket
samples gives an initial-ticket CPU average p50/p95 of 629.962/651.987ms and
an updated-ticket CPU average p50/p95 of 587.167/658.499ms. GPU operation
average p50/p95 is 0.043398/0.068876ms; GPU operation maximum p50/p95 is
0.292864/1.112064ms. The second run's 1.112064ms peak is retained as an
observed outlier rather than discarded. It is not evidence of a corrected
algorithmic bottleneck because four other maximum samples were 0.291840 to
0.293888ms, but it prevents any claim of a stable sub-0.3ms p95 maximum.

For the final run's 42 individual operations, timestamp p50/p95/max is
0.013312/0.141312/0.292864ms. Per-operation totals are: CaptureSky 6 samples,
3072 workgroups, 0.010923ms mean; source mip 14 samples, 1056 workgroups,
0.012142ms mean; GGX PMREM 20 samples, 4118 workgroups, 0.065075ms mean;
and terminal diffuse SH9 2 samples, 2 workgroups, 0.138240ms mean. The
terminal reduction is latency-dominated but is one workgroup as designed; the
static dispatch repair did not change its GPU execution cost and makes no
energy-saving claim.

The test report does not emit an adapter identifier or explicit WGPU backend,
so neither is inferred from a file name. No external device-specific power
counter was available for this run; power and energy are therefore not
measured. The recorded measurements establish runtime timing visibility and
the absence of the former zero-report bottleneck, not a cross-engine power or
hardware-normalized performance comparison.

## 2026-08-23 PMREM Command-Planning Review

The product CPU figures above are end-to-end ticket measurements. They include
frame submission and readback synchronization, so they cannot responsibly be
used to attribute a 600ms ticket cost to Rust command planning. The existing
compiled graph cache already removes graph compilation on a topology hit, and
GPU timestamp samples show the measured GPU operation times independently.
This review therefore targets only an allocation/algorithm defect proven by
the recorder source, without making an unmeasured frame-time claim.

`RealtimeIblWgpuRecorder::record_graph_plan` derives a `PMREM_SH9` request for
each realtime batch. Before this correction, every `Prefilter` pass called
`ibl_bake_wgpu_prefilter_command_for_slice`, which built the complete request
plan, linearly searched it for one PMREM mip, edited the result, and cleared
its readback copies. At the default eight PMREM mips, that request plan has
eight PMREM commands plus one SH9 command. Each PMREM command initially owns
six readback-copy descriptions and the SH9 command owns one, so a single
prefilter slice constructed nine commands and 49 readback-copy descriptions
only to record one command with no readback. The default 21-batch ticket has
ten PMREM slices; two observed ticket generations therefore exercise this
path 20 times.

The correction keeps the shared PMREM face-range dispatch contract and creates
only the selected PMREM kernel/command for a realtime slice. The command
planner now has an explicit runtime path that omits readback construction at
the source. Relative to the previous default path it removes eight unrelated
commands, nine readback-copy vectors, and all 49 readback-copy descriptions
per PMREM slice. The remaining per-dispatch parameter buffer and bind-group
allocation belongs to the larger immutable graph-template/resource-generation
change described by the optimization review; it is deliberately not claimed
as complete here.

At the default ticket scale, the former ten PMREM slices plus one SH9 pass
constructed 99 command records and 539 readback-copy descriptions before
recording the 11 actual compute commands. The runtime path now constructs the
11 required command records and zero readback-copy descriptions. This is a
static allocation/algorithm count, not an elapsed-time result: the product
test's CPU timer brackets submission and readback synchronization and cannot
separate these removed host allocations from GPU or driver latency.

The characterization regression compares the optimized slice command with the
selected command from the old full-request construction for a base-mip face
chunk, an intermediate chunk, and the terminal all-face average. A second
regression exercises the runtime kernel constructor directly and locks it to
the offline PMREM command after removing its readback payload. Together they
lock kernel kind, pipeline metadata, parameter words, shared dispatch extent,
and the empty realtime readback contract. `rustfmt --check` and `git diff
--check` passed for the edited files. The 2026-08-23 fresh library attempt
compiled the production crate but then stopped on 30 external UI test API
drift errors (the Runtime02 handoff covers them), before any `realtime` test
could execute. No post-change runtime timing, power, or cross-engine
comparison is reported.

The post-change managed production gate completed successfully:

```powershell
$env:CODEX_THREAD_ID='shader06-realtime-sh9-dispatch-parity-r1-01a019a5-20260822'
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 `
  -Package zircon_runtime -SkipTest
```

It ran `cargo build -p zircon_runtime --locked` in the existing managed D:
pool and passed in 6m21s. This confirms the runtime command-planning code and
its visibility boundary; it does not execute the `cfg(test)` regressions.
The corresponding fresh library attempt used `-LibTests -TestFilter realtime`.
Its production build stage also passed, but the lib-test compilation stopped
before enumeration on the 30 external UI test errors listed in the Runtime02
handoff. No target directory or generated artifact was created on C:.

The PMREM/SH9 command-planning change was also revalidated through the ignored
product export after the runtime-only command path and the coordinated SH9
recorder patch were present. The managed command's build and selected test
both passed. Its non-RenderDoc execution produced the unchanged 1600x1200 PNG
hash above, 42 operation samples, and two terminal `diffuse_sh9` operations
with one scheduled/completed workgroup each. The report recorded 0.044032ms
average and 0.299008ms maximum GPU operation duration; its full-ticket CPU
averages were 740.878ms (initial) and 687.776ms (updated). These are one
end-to-end sample with submission/readback synchronization, not a comparison
against the five-run baseline and not evidence of an overall CPU improvement.

RenderDoc 1.44 then captured the freshly built product test with the same
final-SH9 request. The replacement capture passed in 65.87s and
`renderdoccmd replay --loops 1` exited successfully. It is the current
23,661,068-byte RDC and SHA-256 listed above. RenderDoc injection rewrites the
same timing sidecars, so their current hashes belong to capture validation;
they must not be used for the non-injected runtime comparison. The image hash
did not change, and visual inspection again found all 64 material samples,
continuous sky/ground reflections, and no blank or overlapping tile.

## 2026-08-23 P1-2 Binding Lifetime Design (Implemented, Validation Pending)

The next recorder cost is structurally different from the P1-1 command-plan
allocation defect. `RealtimeIblGpuResources` owns two persistent physical
slots, each with a stable source cubemap view, PMREM storage view per mip, and
SH9 output buffer. In contrast, `record_ibl_command` currently creates a new
uniform buffer and bind group for every PMREM or SH9 dispatch. The default
ticket has ten PMREM slices and one SH9 dispatch, so it performs eleven of each
creation per ticket even though the relevant resource views are stable for a
work slot. The scheduler's sky revision must *not* be the cache key: it changes
captured texel content, not source/output resource identity or IBL parameters.

The adopted direction is a slot-owned immutable binding-template cache. Its
key must cover physical source/PMREM dimensions and mip counts, the command
kind, the command parameter words (including PMREM face range and terminal
average flag), and the selected work slot. It must intentionally exclude the
environment bake-key revision. On a cache miss it creates the command's
immutable params buffer and bind group against that slot's persistent views;
on a hit it only encodes the cached pipeline/bind-group dispatch. A physical
layout or quality change invalidates the complete two-slot template set. The
first default configuration therefore creates at most 22 params buffers and
22 bind groups (11 commands x 2 slots); steady-state tickets create zero of
either resource.

A dynamic-offset uniform ring is rejected for this MVP: all dispatches are
recorded into one command encoder, and overwriting a shared uniform range
before submission would make earlier dispatches observe the last parameter
write. A per-slot immutable buffer per scheduled command avoids that data
hazard and matches the existing double-buffer ownership boundary. This follows
Unreal's realtime capture structure: external cube resources persist while RDG
builds pass parameters and SRV/UAV bindings for the selected mip/face pass;
Zircon should similarly make stable physical resources explicit and rebuild
only when topology changes, rather than tie their lifetime to changing sky
content.

The post-implementation structural review also checked the cache identity
against resource lifetime, rather than assuming dimensions alone were enough.
`RealtimeIblRuntime::ensure_gpu_resources` creates its recorder and the two
physical `RealtimeIblGpuResources` slots together, once per runtime/device;
there is no same-layout resource replacement while that recorder survives.
The runtime request always requires `PMREM_SH9`, and its bake key changes sky
texel content only. Consequently slot plus layout plus command slice identifies
both the immutable parameter words and the actual bound views in this MVP. If
future device recovery or resource reallocation permits replacing resources
without replacing the recorder, that path must explicitly reset this cache;
layout equality alone must never be used as a cross-resource lifetime token.

Current source implements this cache and reports template cache hits/misses,
params-buffer and bind-group creations, resets, and a narrow `Instant` around
template construction. The focused recorder test covers a cold record followed
by the same stable work slot, which must create all bindings once and create
none on the warm record. The product export now executes three 21-frame
generations: the initial and updated generations initialize the B and A slots,
then an otherwise equivalent revision returns to B. It must report 63 timestamp
operations, preserve all three terminal SH9 publications, and show exactly ten
PMREM plus one SH9 B-slot cache hit with zero params-buffer or bind-group
creation in the third generation. This remains a design and static-count
result, not an elapsed-time or power claim: P1-1 already reduced default
runtime command construction from 99 records and 539 readback descriptions to
11 records and zero readback descriptions, while P1-2 has an expected
steady-state reduction of 11 params-buffer and 11 bind-group creations per
ticket. Acceptance still requires the current library binary to run the focused
test, a managed product export to report the counters, unchanged GPU timestamp
topology, and a fresh RenderDoc capture with the same pass topology. Only then
compare warm-cache CPU construction time across repeated settled runs; no
cross-engine power or energy conclusion is permitted without adapter and
external power telemetry.

### 2026-08-23 Pre-Implementation Profile

The current-source realtime-IBL product executable was rebuilt in the managed
D: target pool and its two 21-frame tickets completed in 44.93s. Its own
per-frame `Instant` report measured 689.647ms initial-ticket CPU average
(1314.764ms maximum) and 563.475ms update-ticket CPU average (737.367ms
maximum). WGPU timestamp queries were available for all 42 operations and
reported 0.296741ms average GPU operation time with a 3.343360ms maximum.
These are end-to-end debug-profile submission samples, including driver work
and synchronization; they do not identify bind-group construction as the
dominant wall-clock cost and must not be used to claim a total CPU or GPU
speedup.

Windows Performance Recorder was also attempted with `wpr -start CPU
-filemode`, but the host policy rejected `CPU.Verbose.File` with
`0xC5585011`; no ETL was generated and `wpr -status` confirmed recording was
not active. The focused PMREM encoder micro-profile test therefore records the
remaining measurable CPU hot path without submission or image I/O, but cannot
be compiled until the external Runtime02 library-test errors are resolved.

The optimization decision remains evidence-based despite that tool blocker:
the default scheduler deterministically emits ten PMREM and one SH9 command
per ticket, and the current recorder creates one params buffer plus one bind
group for every such command. P1-2 is scoped only to removing that 11+11
steady-state object churn. It has no justified target for overall ticket wall
time, GPU execution time, energy, or power. After implementation the recorder
counters must prove zero warm-cache params/bind-group creations; the same
product timestamp topology and a RenderDoc capture must remain unchanged.

## 2026-08-23 Real HDRI Textured-Material Product Evidence

The compiled framework export of the Poly Haven Lakes HDRI with the AmbientCG
Metal009 color, metallic-roughness, and normal maps passed in the managed D:
test lane. Its frame assertions require the single sphere to reflect the
environment and the textured material to retain surface variation. The
non-injected export produced the following immutable evidence:

| Artifact | SHA-256 | Result |
| --- | --- | --- |
| `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_20260823_r3.png` | `62BEC47E3C5911BDC7F7A4A36AB67942265FCB7A73B50131EF91CA9059044500` | 1280x960 frame; visually inspected as a textured metallic sphere with continuous lake/sky reflection and no blank frame or tiled-grid fallback. |
| `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_20260823_renderdoc_r2.png` | `62BEC47E3C5911BDC7F7A4A36AB67942265FCB7A73B50131EF91CA9059044500` | RenderDoc-injected frame; byte-identical to the non-injected PNG. |
| `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_20260823_r3_renderdoc_r2_capture.rdc` | `449923FEB035FFD3F824C9E47D320FA36935E321C3B41478E2DD7E0336BDC5D5` | 33,603,176 bytes; `renderdoccmd replay --loops 1` returned 0. |

The first CLI injection proved that the product test passed but did not create
an RDC because the test never requested a graphics-debugger frame. Current
source now has the opt-in `ZR_RENDERDOC_CAPTURE_PBR_HDRI_EXPORT` request and
completion/error assertions. Its focused source-contract test passed through
the managed validator. With that variable enabled, RenderDoc captured the
same ignored product test successfully in 138.49s and produced the replayable
RDC above. All temporary and target directories were under D: and all evidence
was written under E:; no new capture or Cargo artifact was created on C:.

## Remaining Gate

The compiled-scene MVP and the real HDRI textured-material product path are
verified. Current source also restricts direct-scene realtime-IBL timestamps
to actual WGPU timestamp support, matching the compiled-scene gate; a focused
direct-scene product regression remains to be run after its owner exposes the
lane.

P1-1's historical compiled-scene product evidence remains valid. A fresh
Windows product export for P1-2 completed on 2026-08-23 using an F: ephemeral
target: the package build and selected ignored test both passed. The non-RenderDoc
artifacts are retained as `*_20260823_p1_2_non_renderdoc.*`: the 1600x1200 PNG
has SHA-256 `4F3967C321222618C60C7A9E2BDAB8B92E522FB37118B18BE3690C5810F5D473`,
the CPU report has SHA-256
`78143892C3B419E438031579503AD2342FB11E2F354855FEE0FE5C2E1A2AFCB8`, and the
GPU report has SHA-256
`A36DD630F96CA644B0376BD106DD2ECF605D80B6E0DDA3CD21F53451DF5549A0`. The GPU
report records three 21-operation generations with 4124 workgroups each. The
first two generations create 11 parameter buffers and 11 bind groups each;
the third B-slot generation records 11 template-cache hits and zero new
parameter-buffer or bind-group creation. Visual inspection found all 64
material samples, continuous sky/ground reflections, and no blank or
overlapping tile.

P1-2 remains unaccepted until the focused recorder regression and a fresh
RenderDoc final-SH9 capture/replay complete. A persistent-target request for
the latter encountered an already-owned compatible Cargo pool before Cargo
started, so no source, Cargo, PNG, timing, or RDC artifact was created by that
attempt. This is an availability constraint, not Rust, WGPU, or algorithm
evidence; independent implementation and architecture work continues rather
than waiting on the validation lane. The accepted evidence must still preserve
the GPU timestamp topology under debugger injection. No overall elapsed-time,
GPU-time, energy, or power claim is accepted until those current-source gates
complete.

The non-RenderDoc product report measured per-frame submission intervals of
780.610ms for the initial ticket, 871.090ms for the updated ticket, and
905.681ms for the warm ticket. These values include submission and readback
synchronization during a shared-host debug run and are not a command-recording
microbenchmark. They neither establish a regression nor invalidate the
structural result: the only supported P1-2 performance conclusion is the
observed warm-slot elimination of 11 parameter-buffer and 11 bind-group
creations. Adapter-specific power, energy, and cross-engine timing remain
unmeasured.

On 2026-08-24, a second managed request for the focused PMREM-and-SH9 recorder
regression was accepted by `cargo.acquire`, but the coordinator returned a
post-response timeout after 15 seconds without a terminal reconciliation.
Cargo did not start and no managed target was created, so this is neither a
test pass nor a source failure. Do not poll or automatically retry the
coordinator; retain the focused regression as an explicit acceptance gate and
continue independently verifiable source and documentation work.

The focused regression now uses the production 128-face-size, 8-mip request
and the same two-face scheduler budget as `RealtimeIblRuntime`. It must record
exactly ten PMREM command slices plus one terminal SH9 slice on the selected
physical slot; first use creates all eleven binding templates, while the
second use must hit all eleven with zero params-buffer, bind-group, or cache
reset events. This replaces the earlier reduced 16/5 fixture, which could
prove cache mechanics but not the product ticket topology.

The same non-injected GPU sidecar reports timestamp-query support for all 63
single-pass, single-dispatch operations: the operation mean is 0.043414ms and
the maximum is 0.292864ms. Those values are per scheduled operation, not a
continuously bracketed ticket-total GPU measurement. That capture's narrow
`binding_creation_micros` accumulator, which includes command construction,
pipeline lookup, parameter-buffer creation, and bind-group creation, sums to
60.428ms across the 11 cold B-slot binding misses and 7.502ms across the 11
cold A-slot misses. The third B-slot ticket records 11 hits and 0.000ms in
that accumulator. This quantifies the eliminated steady-state construction
work without claiming a total-frame improvement or treating first-use pipeline
work as bind-group cost.

The current report schema preserves those historical aggregate observations but
does not reuse that ambiguous metric for new samples. Active profiling now
reports three distinct PMREM/SH9 cold-miss CPU windows:
`command_plan_creation_micros` covers only creation of the backend-neutral
command plan, `pipeline_ensure_micros` covers only the pipeline-cache ensure
call, and `binding_creation_micros` covers only parameter-buffer and bind-group
creation. The latter now has the same object-creation boundary as the capture
and source-mip metrics. All three remain zero when core profiling is inactive,
which means unsampled rather than zero cost. This is an attribution repair, not
a cache, dispatch, rendering, energy, or power claim; the next named-adapter
managed profile must generate the first comparable current-schema data.

`RealtimeIblGpuTimingReport` remains a timestamp-readback-backed public
diagnostic, so its `elapsed_gpu_nanoseconds` remains a GPU-clock value only.
The former limitation that its attached CPU fields were inaccessible on
adapters without timestamp queries is now removed by the separate public
`RealtimeIblCpuTimingReport` drain. That report is created only while core
profiling has a live capture epoch, enters its bounded 256-entry ring only
after the associated submission is accepted. Stopping seals that epoch so an
already submitted pipeline tail can still drain; reset or restart advances the
epoch and rejects older pending work. It carries its capture epoch and an
overwrite count, so a long capture cannot silently mix sessions or grow
unbounded. The CPU drain is deliberately independent of timestamp support;
the GPU drain remains the only source for GPU duration. The no-submit encoder
micro-profiles deliberately bypass this drain because they are recording-cost
measurements, not accepted WGPU submissions.

The ignored compiled-scene product export now starts this capture only when
built with the `profiling` feature and no capture is already live. It otherwise
leaves the active external epoch entirely untouched and skips the destructive
CPU drain/sidecar, preserving that session's reports. A capture it owns is
stopped before the CPU public drain. It writes
`runtime_shader_pbr_realtime_ibl_generation_ticket_8x8_cpu_timing_20260824.txt`
beside the existing Shader06 evidence. The sidecar labels its clock domain as
`cpu_command_recording_only` and emits the separate command-plan,
pipeline-ensure, binding-creation, graph-resource-binding, and validation
windows for accepted submissions. A non-profiling build asserts that the drain
is empty and writes no CPU sidecar. The product test removes that exact current
sidecar before every run, so a disabled or externally owned capture cannot
present stale CPU data as current evidence. This is source wiring only: no current
sidecar, adapter measurement, full-frame time, GPU duration, energy, or power
claim is inferred until the managed profile completes.

**Status: measurement-attribution, cross-adapter CPU-report, and product-sidecar wiring implementation complete; runtime validation pending.**

## 2026-08-24 Post-P1-2 Capture and Source-Mip Re-review

The next source review inspected the complete `RealtimeIblRuntime` record path,
`RealtimeIblCaptureWgpuPipelines`, the compiled graph cache, and the matching
cmft/cmftStudio and Unreal capture structures before proposing another
optimization. The default scheduler ticket contains three `CaptureSky`
operations (two faces per slice), seven source-mip operations, ten PMREM
slices, and one SH9 slice. P1-2 now covers only the last eleven compute
commands.

`record_capture` currently creates one 112-byte uniform buffer and one bind
group for each of the three capture operations. Its uniform includes the
procedural sky colors, resolved sun, intensity, face size, and first face, so
the value must change whenever the sky revision changes. Caching that buffer or
bind group by work slot alone would replay obsolete sky radiance into a new
generation, violating the source-revision contract. A revision-keyed cache
would retain an unbounded stream of sky revisions and is rejected for this MVP.

`record_downsample_mip` likewise creates one 16-byte uniform buffer and one
bind group for each of the seven source-mip operations. Unlike capture, its
parameter words are fixed by the 128/8 topology and its source/destination
views are stable per physical A/B slot. It is therefore the only plausible
next binding-template candidate, but it is not yet an implementation decision:
the current recorder report counts PMREM/SH9 template work only, while the
existing GPU timestamp report measures dispatch duration rather than CPU
object construction. The runtime now separately records the active-batch
`RenderGraphExecutionResources::new()` plus `bind_graph_plan` window, its
materialization-validation window, and the actual bound texture-view/buffer
counts. The new report is measurement scaffolding only: treating either
resource-preparation window as the bottleneck before the named-adapter profile
would still be speculative.

cmftStudio performs radiance/irradiance filtering in a background job and
publishes a separate completed image, while Unreal's reflection capture uses
persistent external resources with per-pass RenderGraph setup. Zircon's A/B
slot and publish-after-SH9 policy already follows that ownership direction.
The next implementation must preserve it: add narrowly scoped capture and
source-mip creation counters plus a no-submit encoder micro-profile, compare
cold and same-slot warm samples on one named adapter, then decide whether a
slot-owned downsample template removes measurable host work. No dynamic
uniform-ring rewrite is authorized: the current single command encoder would
need non-overlapping offsets and a bounded lifetime proof before sharing
mutable parameter storage. No elapsed-frame, GPU-time, energy, or power claim
is made from this audit.

## 2026-08-24 Capture/Source-Mip Measurement Infrastructure

The recorder now reports the CPU-side object creation of the two remaining
uncached stages without changing their binding lifetime or their encoded
commands. `record_capture` and `record_downsample_mip` each return the exact
one parameter-buffer and one bind-group creation they performed, plus a narrow
`Instant` window that covers those two WGPU object-creation calls only. Active
profiling places those CPU counters and graph-preparation windows in the
separate `RealtimeIblCpuTimingReport`; GPU timestamps remain in
`RealtimeIblGpuTimingReport`, rather than treating two clock domains as one
sample. The CPU clocks are gated by `profiling::capture_epoch()`; ordinary
realtime IBL recording retains the creation counters but leaves every
CPU-microsecond field at zero and does not call `Instant`. Zero therefore means
"not sampled", not a zero-cost WGPU operation.

The default 128/8, two-face ticket regression now records all 21 passes and
locks the intended topology: three CaptureSky binding pairs, seven source-mip
binding pairs, ten PMREM template pairs and one SH9 template pair. Repeating
the same physical slot must retain the three plus seven dynamic-stage pairs,
while the PMREM/SH9 template counters change from eleven misses/creations to
eleven hits/zero creations. The product export additionally rejects a timing
sidecar that misattributes CaptureSky work to source-mip counters or vice
versa.

The binding-cache reset metric was also corrected: the first layout assignment
to an empty recorder is initialization, not a reset. A reset is now counted
only when an existing layout changes and entries are actually cleared. This
makes cold and warm samples comparable without changing cache contents,
dispatches, resource views, or render output.

These counters are measurement scaffolding, not an optimization result. The
new ignored `profile_realtime_ibl_capture_and_source_mip_binding_encoding`
test now records a complete ticket 256 times without queue submission. It
emits the three CaptureSky and seven source-mip creation counters and windows
separately from the PMREM/SH9 cold-miss and warm-hit counters. The next
profiling pass must run it on one named adapter for cold and same-slot warm
samples with the runtime `profiling` feature and an active
`ProfileCaptureConfig`, then inspect whether source-mip creation materially accounts for the
encoder interval before any slot-owned downsample template is considered. The
manual profile requires an explicit absolute non-C
`ZIRCON_PROFILE_OUTPUT_ROOT`; after its measured assertions it exports the
native timeline, hotspot reports, and summary there, then prints the export
directory beside the adapter-identified console metrics. The trace records a
frame plus distinct cold-ticket and per-iteration warm-ticket encoding spans,
while the creation microseconds remain in their separate CPU timing report.
No current
elapsed-time, GPU-time, energy, power, or cross-engine comparison is claimed.

The adjacent ignored `profile_realtime_ibl_graph_resource_preparation` test
drives the production `RealtimeIblRuntime` through 256 distinct source
revisions without queue submission. It reports the same adapter identity and
separates the 5,376 active-batch execution-resource binding windows from
materialization validation, while recording their live texture-view and buffer
binding counts. The ignored profile starts its own `ProfileCaptureConfig`,
requires an explicit absolute non-C `ZIRCON_PROFILE_OUTPUT_ROOT`, and exports
its capture trace after its measured assertions; it fails clearly when the
runtime was not built with `profiling`. Its trace contains one frame and one
span for every prepare-and-record batch, so profile retention is inspectable.
Run it beside the recorder profile before proposing a
resource-template or graph-instance cache: it measures host preparation only,
does not create a steady-frame cost, and cannot establish GPU duration, total
frame time, energy, or power.

## 2026-08-24 Environment Provider Boundary Re-review

The regular Standard-PBR environment module has three independently valid
reflection providers: the global PMREM/SH9 environment, local reflection
probes, and planar reflection. Its prepared-input gate returns early only when
all three are unavailable. Planar sampling is attempted first because a valid
planar result replaces the cubemap path; otherwise the selected local probes
are blended with the global sky fallback. Local probe intensity is clamped and
applied at the probe sample rather than being multiplied by global environment
intensity. This is the required provider contract: disabling a global sky must
not disable a valid local probe or planar reflection.

`ENVIRONMENT_ONLY_PBR` forward specialization deliberately uses
`zr_environment_only_pbr.wgsl`, which excludes the five local-provider
bindings and their helper functions. It is a constrained Standard-PBR preview
profile whose source-size regression requires at least a 25 percent reduction
from the generic Forward assembly. The deferred environment-only preview uses
the complete generic environment module and keeps local providers, so the two
profiles must not be treated as interchangeable. The current template tests
assert both sides of this boundary; folding the full provider module into the
forward specialization would add bindings and compilation work without fixing
the normal scene PBR path.

This review rejects two unmeasured changes: a global shader-template cache in
the material-surface builder, and a provider-unification change to the
environment-only Forward profile. The former must be designed in Shader03's
source-variant/artifact lifecycle because pipeline lookup currently assembles
source before its module-cache lookup, and its key must include pass, geometry,
static material features, alpha-cutoff bits, and source generation. The latter
would invalidate an intentional specialization. The next performance step is
therefore to run the existing no-submit realtime-IBL micro-profile and the
Shader03 source-assembly measurement on a named adapter, then compare their
CPU intervals and allocation counts before selecting one owner-scoped change.
No current wall-time, GPU-time, energy, power, or cross-engine equivalence
claim follows from this source review.

## 2026-08-24 Generic EnvBRDF Approximation Conformance

The final material/PBR audit compared Zircon's split-sum helpers with Unreal
`BRDF.ush`. The active LUT path already follows Unreal's `EnvBRDF` contract:
it samples `AB`, uses `F90 = saturate(50 * F0.g)`, and returns `F0 * A + F90
* B`. Zircon's GGX PMREM path remains the documented `V = N` reduction of
Unreal's generic `D * NoH / (4 * VoH)` PDF, and cmft/cmftStudio remain
offline-filter scheduling references rather than material-integrator sources.

The inactive generic helper `zr_environment_env_brdf_approx` still used the
older fixed-`F90 = 1` expression. It is retained in generic assembled source
for custom/future fallback consumers, so leaving it divergent would make a
future call disagree with the active LUT helper. The helper now uses the same
clamped green-channel F90 rule. A focused source regression locks the F90
definition, the `F0 * A + F90 * B` term, and rejects the old fixed-F90 term.

## 2026-08-24 glTF Default-PBR Compound Asset Identity

The material-input audit found a correctness boundary that must be fixed before
the external zero-roughness product scene can be treated as representative.
The stable `gltf_importer` previously emitted
`res://shaders/default_pbr.zshader` for both generated `DefaultMaterial` and
explicit glTF materials. This is a historical single-file locator, while the
runtime built-in importer, `ProjectManager` compound-source resolver, minimal
project fixture, and `examples/vampire` all identify the standard PBR shader
package by its logical root `res://shaders/default_pbr`, persisted through
`default_pbr.zmeta`. The resolver deliberately treats these as distinct: a
single-file locator is not rewritten to a compound root.

The working comparison therefore establishes an actual source incompatibility,
not a naming preference. A test first required both material kinds to contain
and reference the canonical URI; the old importer source supplied the red
baseline. The material asset source owner `asset/assets/material/default_pbr.rs`
now owns the compound root and a cached parsed `AssetReference`; both the
built-in and stable plugin importers call the `asset::assets` accessor. For a
document with N explicit materials, that reduces default-URI parsing from N+1
calls to one process-first-use parse, while still cloning one value reference
per material. Static cross-owner, formatter, and scoped diff checks pass. This
does not change PMREM/SH9 dispatch or frame rendering; it has no measured CPU
interval, GPU-time, energy, or power result and restores material dependency
resolution only. The next managed product run must execute this regression plus
the external-glTF zero-roughness mirror export, then retain its current PNG and
terminal-SH9 RenderDoc capture before any M4/M5 acceptance statement.

This is a semantic consistency repair only. The active Standard-PBR path
continues to call the LUT helper, so this edit has no claimed current image,
CPU, GPU-time, energy, or power effect. It requires the next managed Rust/WGSL
run together with the existing external-glTF product export and RenderDoc
capture before being counted as acceptance evidence.

## 2026-08-24 Prepared Static-Cubemap Submission Boundary

**Status: implementation complete; runtime acceptance pending.** The direct
and compiled render paths now record the prepared static-cubemap copy into the
caller-owned frame encoder, and the compiled path retains serial-prefix order
before graph-parallel command buffers. Focused source regressions cover the
deferred upload key, caller encoder ownership, and post-submit commit order;
`rustfmt --check` and scoped `git diff --check` complete without errors. This
is not a validated rendering or performance result: managed Windows Rust/WGPU
execution, a new HDRI screenshot, a RenderDoc capture, and cold/warm timing
remain pending. No milestone is accepted, committed, or reported externally
from these static checks.

The static source-cubemap upload audit found a separate command-submission
owner inside `CubemapUploadStagingArena`: when a prepared RGBA16F source/PMREM
artifact was available, it wrote the staging buffer, finished a private encoder,
and called `queue.submit`; the caller then created and submitted the normal
direct or compiled scene encoder. This is a first-load or content-change path,
not a steady-state per-frame cost, but it split one logical frame dependency
into two driver submission boundaries. It also diverged from the current
renderer contract and Unreal's RDG direction, where upload/copy work belongs to
the frame command stream that consumes it. cmft/cmftStudio are offline producer
references here and do not define runtime submission ownership.

The repair gives the staging arena a caller-owned encoder and records its
`copy_buffer_to_texture` calls into the direct/compiled frame's serial prefix.
The existing padded row pitch, RGBA16F bytes, source/PMREM/IEM selection, and
fallback `queue.write_texture` behavior are unchanged. `CubemapUploadState`
now holds a pending key and advances the committed upload key only after that
frame's `queue.submit`; a failed frame discards pending state before the next
recording so it cannot claim that a dropped command buffer reached the GPU. The
compiled command-encoder set flushes its serial prefix before topology-ordered
parallel command buffers, so the copy remains ordered before any parallel scene
pass can sample the cubemap.

This is a submission-topology reduction, not a measured speedup. The next
Windows managed static-HDRI product capture must verify an unchanged skybox and
PBR reflection image, inspect the RenderDoc command sequence for the single
frame-owned copy path, and report the named adapter plus cold and warm frame
timing separately. No CPU interval, GPU-time, energy, power, or cross-engine
equivalence claim follows until that capture succeeds.

### Measurement-Boundary Re-check

The current shared GPU timestamp lifecycle begins after `write_scene_uniform`
in both the direct and compiled renderers. It therefore cannot attribute the
prepared static-cubemap `copy_buffer_to_texture` work without moving readback
admission and failure cleanup across both renderer preambles. Adding a
direct-only timestamp would create incomparable evidence and risks leaving a
compiled readback reservation active on existing early-return paths. Do not
make that local instrumentation change. The next managed run must first use
RenderDoc event durations for cold and warm static-HDRI frames. Only if that
evidence identifies the copy as material should the shared renderer own one
pre-graph timing boundary, with matching direct/compiled cleanup and source
regressions. This is a measurement-design decision, not a current performance
claim.

## 2026-08-24 Hydration-Cache Ownership Re-check

The follow-up audit tested a plausible structural concern before proposing a
cache rewrite: every compiled frame resolves the environment-IBL hydration
state, and a cache hit clones `SourceCubemapEnvironment` before restoring the
frame-local intensity and rotation. The data owners rule out the feared bulk
copy: `SourceCubemapMipChain` stores both source and PMREM texels as
`Arc<[[f32; 4]]>`, and `SourceCubemapUploadArtifact` stores every padded upload
row as `Arc<[u8]>`. The clone is therefore bounded metadata/vector work plus
Arc reference increments; it does not duplicate HDRI texels or prepared upload
bytes. The hydration cache keeps at most four entries and its stored payload
continues to preserve immutable artifact identity while per-frame presentation
parameters remain mutable.

No cache-owner refactor is authorized from this review. A future CPU profile
may count cache-hit clone wall time and allocation count alongside frame
extraction only if a named-adapter capture shows an actionable CPU frame cost;
that measurement must distinguish Arc metadata work from HDRI decoding,
artifact hydration, and the one-time GPU upload. No timing, allocation,
energy, power, or image-quality number is claimed here.

## 2026-08-24 Realtime Bootstrap Scope Boundary

The current realtime scheduler has already removed the historical duplicate
`CaptureCloud` operation. A default ticket now contains three two-face
`CaptureSky` slices, seven source-mip slices, ten PMREM slices, and one
terminal SH9 slice. The shader and graph plans must retain that single capture
ownership; reintroducing a second procedural-sky pass would add work without
adding cloud radiance.

Before the first terminal SH9 publication, the renderer deliberately keeps
the ordinary procedural-environment bindings. That fallback has no PMREM and
therefore cannot provide roughness convolution. After an environment has been
published, a changed sky bakes only into `ready_slot.other()` and continues to
sample the last published slot until the successor terminal SH9 submission
succeeds. Do not hide the first-publication readiness boundary with a
shader-side interpolation between the reflected direction and the normal:
that is the rejected pre-IBL approximation, not a prefiltered environment. A
product-quality bootstrap instead needs an owner-defined prefiltered fallback
generation through the environment generation/residency boundary; it cannot
be introduced as a local Shader06 binding cache or roughness formula change.

This is a scope decision, not an accepted startup-quality result. The next
managed capture must measure first-publication latency and show the fallback
transition on a named adapter before an environment-bootstrap implementation
is selected. No current CPU, GPU, energy, power, or image-quality claim is
made from this source review.

The retained 2026-08-23 P1-2 timing sidecars predate the separate
CaptureSky/source-mip creation counters. They record the PMREM/SH9 binding
cache fields and 21-operation topology, but contain no capture/source-mip
allocation attribution. Keep them only as historical topology and warm-slot
evidence; they cannot authorize a downsample-template optimization or quantify
its CPU benefit. The next profile must use the current sidecar schema and the
named-adapter no-submit profile together.

## 2026-08-24 Managed Windows CPU-Profile Gate

**Status: validation blocked by an existing Frameworks01 RHI/WGPU failure; no
performance result is claimed.** The current no-submit capture/source-mip
profile was invoked on Windows with `profiling`, a dedicated D-drive Cargo
target, and an explicit E-drive `ZIRCON_PROFILE_OUTPUT_ROOT`:

```powershell
$env:ZIRCON_PROFILE_OUTPUT_ROOT = 'E:\Git\ZirconEngine\docs\tests\runtime\shader\profiles\realtime-ibl-cpu-20260824'
cargo test -p zircon_runtime --features profiling --locked --target-dir D:\cargo-targets\zircon-engine\profile-realtime-ibl-20260824 profile_realtime_ibl_capture_and_source_mip_binding_encoding -- --ignored --nocapture --test-threads=1
```

The compilation stopped before the IBL test body in `zr_rhi_wgpu`, so the
configured output root contains no timeline, summary, hotspot report, PNG, or
RDC. The captured terminal evidence is retained only as a transient D-drive
build log. The eight compiler errors are in the existing backend owner:
missing `wgpu_device_features`, `WgpuUiSharedImageRegistry`,
`VertexAttributeDesc`, and `BindingResourceType` imports; an undefined
`tracker`; a stale `BindGroupLayoutEntryDesc.resource` field; and two
`TicketOrderedDiagnosticCompletions::default()` calls requiring an unintended
`DiagnosticBatchCompletion: Default` bound. This is the current concrete
reproduction of the already-open [Frameworks01 RHI/WGPU
failure](../../frameworks/01/failure-2026-08-02-rhi-wgpu-presenter-and-backend-contract-test-owner.md);
do not add a shader-local bypass or a duplicate failure record.

After that owner returns a verified compile repair, rerun the exact command
above and require one named-adapter console line, one exported frame, one
`cold_ticket` span, 255 `warm_ticket` spans, plus the separate graph-resource
profile before selecting a source-mip template or any uniform-ring design. GPU
duration, frame duration, energy, power, and screenshot/RenderDoc acceptance
remain pending.

## 2026-08-24 P1-2 Cache Scope and Measurement Freeze

The PMREM/SH9 binding cache is a normal-runtime P1-2 lifecycle design, not a
profiling-only switch: it retains immutable params buffers, compute pipelines,
and bind groups by physical work slot plus command slice. CPU capture only
attributes its command-plan, pipeline-ensure, and resource-creation windows;
it must not be described as making the cache behavior disappear outside a
capture.

The prior three-ticket sidecars establish only topology and warm-slot counters.
They do not establish current-source performance or output parity because the
current `zr_rhi_wgpu` compile gate prevents the recorder regression and product
fixture from reaching WGPU. Keep P1-2 and M4/M5/M6/M7/M8 `in_progress`, and do
not add a source-mip binding template, uniform ring, cache-key relaxation, or
other lifecycle optimization until the repaired named-adapter run produces the
defined cold/warm CPU spans, graph-resource profile, GPU timestamps, PNG, and
terminal-SH9 RenderDoc replay. This is a scope and evidence boundary, not a
claim about frame time, power, energy, or cache benefit.

`RealtimeIblGpuTimingReport` is limited to timestamp-domain data and structural
cache counters. CPU microsecond windows are exposed only through
`RealtimeIblCpuTimingReport` after accepted submission, so CPU attribution is
available even when an adapter lacks timestamp-query support; neither report
may serialize the other clock domain.

## 2026-08-24 CPU-Only Alternative Validation Attempt

To avoid making Shader06 progress depend solely on the WGPU product fixture,
Windows ran the CPU-only alternative below with an isolated D-drive target and
temporary directory:

```powershell
cargo check -p zircon_runtime --no-default-features --tests --locked --message-format short
```

It exited 101 before any IBL test body ran. The captured D-drive log contains
150 compiler diagnostic error lines: the first cluster is stale Scene/ECS API
imports (`SystemTickContext`, `SceneSystemTickPolicy`,
`RemovedComponentRetention`, and `ScheduleBuildReceipt`), followed by
unrelated task, animation, runtime, plugin, and test-path failures. Cargo
reports 32 library errors before later test-module diagnostics; the latter
also include unresolved source-cubemap test helpers, but do not identify an
IBL recipe, PMREM, or WGSL algorithm failure. With `graphics` disabled this
route does not compile `zr_rhi_wgpu`, so it is a second, independent workspace
test-compile blocker rather than evidence about the existing Frameworks01
WGPU failure. Do not create a Shader06-local feature bypass or a duplicate
failure handoff from this result.

The retained log is a transient validation artifact at
`D:\cargo-targets\zircon-engine\shader06-core-diagnosis-20260824`; no C-drive
output, profile report, PNG, GPU timestamp, CPU timing sidecar, or RenderDoc
capture was produced. The current-source CPU reference and recorder tests
remain unexecuted, and M4/M5/M6/M7/M8 remain `in_progress`.

## 2026-08-24 Realtime Binding-Lifecycle Audit

The current default ticket has 21 topology variants per physical slot. Its
compiled-graph cache is intentionally sized to 42, which covers one complete
generation for both A and B slots: generations one and two populate the two
slots, while a third generation returning to B can hit the same compiled graph
and PMREM/SH9 binding entries. Replacing the at-most-42-entry binding-cache
linear lookup with a hash map is not an approved optimization candidate until
the named-adapter CPU recording profile attributes meaningful time to it.

The remaining creation counters identify a separate candidate, not a current
change: each generation records three `CaptureSky` and seven source-mip
bindings that presently create their parameter buffer and bind group. Capture
parameters contain the changing procedural-sky revision; simply reusing one
uniform buffer and writing it for the next slice could overwrite bytes still
referenced by an in-flight command buffer. `record_graph_plan` currently owns a
device and encoder but not queue-submission retirement, so a valid future
template/ring design must be owned by the resource/submission lifecycle, index
storage by slot and in-flight revision, and prove that no submitted capture
observes a newer sky. First obtain the prescribed cold/255-warm CPU spans and
graph-resource timings. Until then this is a correctness constraint and
measurement plan, not a cache rewrite or a CPU/GPU/power claim.

## 2026-08-24 KHR Texture Transform Projection Ownership

The normal- and occlusion-texture import paths in both the runtime importer
and stable glTF plugin previously carried identical raw
`KHR_texture_transform` JSON projection code. The runtime asset importer now
owns that projection and exposes the transform plus resolved UV channel to
both consumers. This preserves the existing fallback behavior for absent or
malformed `texCoord`, scale, and offset fields; scale and offset are accepted
only as exactly two finite `f32` components, otherwise that field preserves
its identity default. It also applies the extension's rotation where those
two raw-JSON paths had omitted it. The base-color,
metallic-roughness, and emissive paths continue to use the glTF crate's typed
extension decoding; this change does not alter that boundary.

This is a material-import correctness and maintenance convergence, not a
runtime IBL optimization. It changes no GPU resource lifecycle, shader loop,
binding cache, or draw-time allocation behavior, so it contributes no CPU,
GPU, energy, power, or image-quality metric. M4/M5/M6/M7/M8 remain
`in_progress`; product PNG and RenderDoc acceptance stay gated on the existing
workspace compile repairs and the named-adapter run.

### Validation Attempt

Windows attempted the pure projection tests with an isolated D-drive Cargo
target and temporary directory:

```powershell
cargo test -p zircon_runtime --no-default-features --locked --lib gltf_texture_transform -- --nocapture
```

The test binary did not reach execution. Compilation produced 149 Rust error
headers in existing Scene/ECS, foundation, native-plugin-loader, and test-tree
code, including removed `RuntimeSceneSystemContext::tick` and
`SceneSystemMetadata::with_tick_policy` APIs. The log contains no diagnostics
from `gltf_texture_transform.rs`, either material-import consumer, or the
stable glTF plugin consumer, but that absence is not a passing Rust test. The
compiler process was stopped after this unrelated pre-test failure became
clear; the retained D-drive log is at
`D:\cargo-targets\zircon-engine\shader06-khr-transform-20260824\gltf_texture_transform-test.log`.
Scoped `rustfmt --check` and source contracts for the shared owner, both
consumers, rotation projection, fallback UV handling, and malformed-field
defaults pass. This is nonaccepting source evidence only.

## 2026-08-24 Normal-Map Convention Boundary

The Standard-PBR template and fallback mesh path now share
`zr_reconstruct_bc5_normal` with the explicit no-flip DX convention. This
matches glTF's required direct mapping of normal RGB to tangent-space XYZ:
the glTF importer marks normal images as linear and the default normal
descriptor selects the same convention. Both paths then apply
`normalTexture.scale` only to tangent-space XY before normalization, so the
fallback no longer owns an independently drifting decode formula. The change
adds no texture sample, binding, shader variant, uniform byte, or draw-time
allocation.

The review also found that `TextureNormalConvention` is currently a validated
texture-descriptor field, not a material-runtime input: texture resolution
loads it, but `ResolvedTextureReference` forwards only the resource identity,
dimension, and fallback state to `MaterialRuntime`. Although the fixed 256 B
Standard-PBR uniform has two reserved scalar positions at `data15.zw`, using
one for a per-material Y sign would add a shader operation to every normal-map
sample and make material state depend on texture metadata changes. That is not
the correct substitute for a general convention policy. A later generic-texture
initiative should normalize explicitly authored GL normal maps during texture
import/compression, with source/derived cache identity and reload semantics
owned by the texture pipeline; it must preserve the current glTF no-flip
contract and be profiled as an import-time cost. No Shader06 runtime
optimization is selected from this review, and M4/M5/M6/M7/M8 remain
`in_progress` pending the existing current-source Windows/RHI, screenshot,
timestamp, and RenderDoc gates.

## 2026-08-24 Standard-PBR Texture Coordinate Capacity Boundary

The material document and glTF projection can preserve an arbitrary nonnegative
`texCoord`, but the current Zircon mesh asset, GPU vertex record, Standard-PBR
template, fallback mesh shader, skinning, and velocity path carry only `uv0`
and `uv1`. The prior uniform packer collapsed every channel at or above one to
`uv1`; a valid glTF material selecting `TEXCOORD_2` therefore rendered with a
different coordinate set without a diagnostic. Unreal's glTF Interchange path
also treats texture-coordinate selection as explicit per-material state, which
confirms that this is a material/vertex-ABI capability boundary rather than a
texture-transform detail.

The MVP decision is to keep the fixed two-channel vertex ABI. Adding UV2+ would
add at least 8 bytes per vertex and require coordinated mesh serialization,
GPU layout, skinning, velocity, template, fallback, and pipeline changes. No
current profiler attributes a product bottleneck to that missing capacity, so
this record does not authorize an ABI expansion or claim a frame-time or power
benefit.

`StandardMaterialDescriptor` now exposes the two-channel limit and reports a
structured `UnsupportedTextureUvChannel` error only when a corresponding
Standard-PBR texture is present. Material readiness checks both the ordinary
descriptor and shader-driven standard-slot aliases, and the renderer treats the
error as blocking before a material can be used. As a defense for nonstandard
callers that bypass readiness, the CPU packer and both Standard-PBR/fallback
WGSL selectors accept only exact channel one; every other value resolves to
`uv0`, never the prior accidental `uv1`. The new regression covers a concrete
base-color texture using channel two and the preparation gate independently
asserts that the error blocks use.

This adds no vertex attribute, material-uniform byte, texture/sampler binding,
shader permutation, or draw-time work for valid materials. The five-slot
capacity scan occurs only during material validation/readiness, not per draw.
Scoped source contracts, `git diff --check`, and formatted owned Rust hunks
pass. The focused Rust regressions remain unexecuted because the current
Windows build stops before Shader06 test discovery in an unrelated RHI owner;
the exact attempt is recorded below. M4/M5/M6/M7/M8 remain `in_progress`; no
current PNG, GPU timing, power, or RenderDoc claim follows from this boundary
repair.

### 2026-08-24 Focused Rust Validation Attempt

Windows started the following cold, single-job validation from the dedicated
short D-drive target `D:\zct-s06-uv`:

```powershell
$env:CARGO_TARGET_DIR = 'D:\zct-s06-uv'
cargo test -p zircon_runtime --lib standard_pbr_readiness_rejects_texture_coordinates_outside_the_vertex_abi --locked --jobs 1 -- --nocapture
```

This target avoided the earlier transient `os error 3` dependency-output
directory failure and compiled the `zircon_runtime` crate dependency graph,
but no Shader06 test body ran. The command stopped with exactly two E0499
diagnostics in the currently untracked concurrent owner
`zircon_runtime/crates/zr_rhi/src/surface.rs:233-234`: the new
`SurfaceHandleRegistry::allocate` match returns simultaneous mutable borrows
of `state.next_*` and `state.active_*`. That file does not exist in `HEAD` and
is outside the Shader06/material ownership boundary. The log contains no
diagnostic for the material validation, uniform packer, WGSL selectors, or
either UV regression test.

The second regression, `importer_preserves_unsupported_gltf_texcoord_for_readiness_rejection`, is present but was not started because Rust cannot build the
test binary until the RHI owner resolves the E0499 errors. It uses the existing
UV0/UV1 glTF fixture with the material `texCoord` changed to two, asserts that
import preserves that authored value, and asserts the material readiness report
contains `UnsupportedTextureUvChannel { slot: "base_color", channel: 2,
supported_channel_count: 2 }`. After the owner repair, rerun both focused test
filters against the same D-drive target before any milestone advancement.

### Advanced Clearcoat Slot Boundary

The audit also found that the current `clearcoat_normal_texture` feature carries
only a texture reference. Its generated WGSL deliberately reuses the ordinary
normal UV, and no independent clearcoat transform or UV-channel field exists
in `StandardPbrMaterialFeatures`, `MaterialRuntime`, or the uniform ABI.
Neither the runtime nor stable importer advertises `KHR_materials_clearcoat` as
a supported required extension, so no currently supported glTF asset imports a
clearcoat normal slot through this path.

Do not add a local clearcoat UV field while the MVP vertex/material ABI is
fixed at two channels. A future advanced-PBR milestone must design the full
clearcoat texture-slot contract together: glTF extension admission, reference,
transform, UV selection/capacity validation, runtime payload, shader sampling,
and forward-path image acceptance. This boundary has no current CPU, GPU,
bandwidth, power, or image-quality metric and does not change M4/M5/M6/M7/M8
status.

## 2026-08-24 Realtime IBL Timing Scope Review

An independent review separated the current changes into two ownership classes.
The persistent PMREM/SH9 binding cache and terminal-SH9 command selection are
P1-2 resource-lifecycle changes: they retain command plans, pipelines, parameter
buffers, and bind groups across physical A/B slots. They are not profiling-only
instrumentation and must retain their own output-parity, cache-lifetime, and
current-source product acceptance. The CPU attribution fields merely split the
already-existing miss path into command-plan, pipeline-ensure, and WGPU
parameter/bind-group spans. No cache lookup, cache capacity, or SH9 planning
change is authorized by those fields until the named-adapter cold and repeat-warm
profiles exist.

The same review found the public `RealtimeIblGpuTimingReport` grows when CPU
attribution is exposed, which can break external struct-literal construction even
though no in-tree external literal was found. More importantly, public timing
drain is timestamp-readback backed. On an adapter without timestamp-query
support, CPU capture must be consumed through the distinct accepted-submission
path and must not be reported as a GPU timing result. Product-side evidence must
record adapter timestamp capability and the drain used.

No source change follows from this review. It records a scope gate: a later
profiling-only patch must not alter binding-cache lifetime or dispatch planning,
and a P1-2 lifecycle patch must not claim CPU, GPU, startup, or power improvement
without the prescribed current-source timing, PNG, and RenderDoc evidence.

## 2026-08-24 Procedural-Sky Evaluation Ownership Plan

The Runtime96 structural review identifies `ENV-P1-016`: the procedural
gradient-plus-sun evaluator must be a single shader-module owner for the visible
sky, realtime IBL capture, and environment fallback. The current Shader06
sources contain three equivalent mathematical implementations:

- `zr_environment_core.wgsl` evaluates the procedural fallback for PBR diffuse
  and reflection, then applies final environment intensity;
- `skybox_procedural.wgsl` evaluates the same gradient and sun for the visible
  sky, then applies the same final environment intensity;
- `realtime_ibl_capture.wgsl` evaluates the gradient and sun into source
  radiance for the capture cubemap, deliberately without final sampling
  intensity.

This is a semantic-drift and maintenance issue, not a measured hot-path
bottleneck. No timing, GPU instruction count, power, cache, allocation, bind
group, uniform, vertex ABI, or dispatch-shape improvement is claimed. The
shared function must receive an already-normalized direction, horizon/zenith/
ground radiance, CPU-normalized sun direction, sun color, and precomputed cosine
edges; it returns source radiance only. The PBR and visible-sky callers own the
final non-negative environment intensity multiplication, while realtime capture
does not. That preserves the current no-double-intensity contract and avoids
per-invocation vector normalization, sine, cosine, or IOR/BRDF work.

The integration must use a small raw WGSL helper included by Rust static source
assembly before each of the three pipeline families reaches Naga/WGPU. Template
`#include` resolution alone is insufficient because the skybox and capture
pipelines pass standalone raw WGSL to WGPU. The material-template registry,
deferred-lighting source, fallback-mesh source, skybox pipeline source, and
capture pipeline source therefore need the same helper byte sequence before the
consumer body. Tests must compile the assembled sources, assert that all three
call the one radiance helper, and retain the distinct visible-sky/fallback versus
capture intensity assertions.

After a fresh source fingerprint/review of the concurrently modified PBR
include graph, fallback source, environment core, and realtime capture
recorder, the shared helper is now composed ahead of each production consumer.
Focused source contracts cover the single radiance owner, all raw-source
assemblies, and the distinct intensity ownership. Static source guards and
owned Rust formatting pass, but Naga/WGPU parsing, current PNG, RenderDoc,
timestamp, and power evidence require a clean coordinator-managed Windows
lane. M4/M5/M6/M7/M8 remain `in_progress`; this structural repair makes no
milestone or performance-status claim.

### 2026-08-24 Managed Material-Readiness Validation Update

The initial managed attempts correctly rejected the unregistered
`D:\zct-s06-uv` target and a mismatched manually selected primary-pool target;
neither attempt started Cargo and neither is test evidence. A subsequent
`validate-matrix.ps1 -Ephemeral` run used the coordinator-managed F-drive lane
`F:\cargo-targets\zircon-engine\ephemeral\test\03ea5c1ab28b417b9b4aced8b31eb7e4`
for the exact `standard_pbr_readiness_rejects_texture_coordinates_outside_the_vertex_abi`
filter. Its `Cargo build` completed sufficiently for the script to launch the
filtered `Cargo test` stage. This current-source build therefore did not
reproduce the formerly observed external `zr_rhi::surface` E0499 errors.

The filtered test child then exited and the ephemeral lane was removed, but the
interactive terminal did not retain the script's final stdout, exit receipt, or
per-test summary. The individual Rust test must consequently remain
**unconfirmed**, not be reported as passed. A later managed rerun must retain
the stage transcript/receipt before a validation or milestone status advance.
The focused static material contracts and `rustfmt --check` remain passing;
current-source WGPU/PNG/RenderDoc/timestamp/power evidence remains absent.

### 2026-08-24 Procedural-Sky Integration Validation Blocked

The new integration-test request targeted
`runtime_environment_wgpu_cubemap_sampling_contract` with the
`runtime_environment_procedural_sky_uses_shared_source_radiance_owner` filter
through `validate-matrix.ps1 -Ephemeral`. The coordinator rejected it before
Cargo started because shared unmanaged artifacts remain below
`F:\cargo-targets\zircon-engine\ephemeral`. That shared Cargo root is outside
the Shader06 ownership boundary and was not removed. An empty, newly created
local transcript directory under `F:\ZirconBuilds\validation-logs` was removed
immediately after the rejection; no build output or user artifact remains
there.

The helper/consumer source contracts, complete direct-core assembly scan,
non-fallback `rustfmt --check`, and scoped `git diff --check` pass. The
fallback-source file contains concurrent pre-existing formatting drift outside
the inserted include lines, so it was not mechanically reformatted. Naga/WGPU
parse and validation of the newly assembled sources, current PNG, RenderDoc,
timestamp, and power evidence are still pending a clean coordinator-managed
Cargo lane. M4/M5/M6/M7/M8 remain `in_progress`.

### 2026-08-24 Procedural-Sky Uniform Contract Recheck

The post-integration source audit found no remaining direction or intensity
contract mismatch. `ProceduralSkyParams::resolved_sun` rejects non-finite or
near-zero sun directions, normalizes a valid direction once on CPU, clamps the
angular radius, and emits precomputed cosine edges. `SceneUniform::from_frame`
uses that same resolved value after the authored environment rotation, while
`capture_params_bytes` uses it without the final sampling intensity or rotation.
The existing unit contracts assert a non-unit authored sun becomes a normalized
scene/capture vector, and raw capture WGSL rejects `length`, `cos`, rotation,
and final-intensity work. The three shader consumers feed normalized directions:
the generic PBR API normalizes at its boundary, skybox world-ray construction
normalizes its result, and cubemap capture normalizes face directions.

Accordingly, no shader or uniform change is warranted. Adding a second
normalization or a uniform would add fragment/compute work or ABI pressure
without correcting a demonstrated defect. This is static/source evidence only;
Naga/WGPU validation, current image evidence, RenderDoc replay, and performance
measurements remain pending the managed Windows lane. M4/M5/M6/M7/M8 remain
`in_progress`.

## 2026-08-24 Standard-PBR Material-Input HDRI Fixture

**Status: implementation complete; product evidence pending.** The ignored
`export_runtime_shader_pbr_real_hdri_standard_inputs_png` fixture now writes a
separate immutable Shader06 archive candidate using the existing Metal009
base-color, OpenGL normal, and metallic-roughness maps. It applies the same
non-identity UV transform to those three material slots, sets
`normal_scale = 0.55`, and sets `ior = 2.0`. The construction-level regression
checks that all three transforms reach `StandardMaterialDescriptor`, the normal
scale reaches the descriptor, and the non-default IOR reaches advanced features
and selects the forward path. Its scalar metallic factor is explicitly `0.0`:
the roughness map remains sampled and transformed, while the metallic map cannot
mask the dielectric F0 change that the IOR screenshot is intended to expose.
The same descriptor contract is rerun after the exact project-TOML
persist/restore boundary used by the fixture, so serialized texture-slot
transforms and property overrides cannot silently collapse to defaults.

The fixture intentionally remains manual and shares the existing opt-in
per-frame RenderDoc request. It writes only
`runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_standard_inputs_20260824.png`
under `docs/tests/runtime/shader` via the existing create-new evidence writer;
the file does not exist yet, and this implementation step did not generate it.
No Cargo, Naga/WGPU, PNG, RenderDoc, GPU-timestamp, CPU, energy, or power
result is claimed. The new Rust regression and manual export must run through a
clean coordinator-managed Windows lane, then the generated PNG must be
inspected and the capture replayed before advancing any acceptance status.
M4/M5/M6/M7/M8 remain `in_progress`.

## 2026-08-24 AmbientCG Fixture Texture-Usage Correction Plan

The Standard-PBR and fallback paths deliberately keep glTF normal RGB as a
direct tangent-space mapping; this review therefore rejects a speculative
green-channel inversion for the `NormalGL` source. The actual fixture defect is
metadata-only: its normal and metallic-roughness maps were both written as
linear textures with the default `albedo` usage. That bypasses the normal
mip-filter/compression policy and makes the fixture unlike an imported PBR
material even though the shader sampling ABI is correct.

The corrective scope is restricted to the test asset writer. It will emit
`usage_hint = "normal"` with the established DX runtime convention for the
normal map, and `usage_hint = "data"` for packed metallic-roughness. This lets
the regular import path choose its normal-specific mip behavior and BC5 policy
without adding a material uniform, a texture sample, a shader branch, a PSO
variant, or draw-time allocation. It does not assert an import-time performance
gain: any reimport cost is one linear asset-processing pass and must be measured
separately before an optimization claim.

The fixture contract must check both serialized setting sets. A later managed
HDIR/WGPU run must prove the resulting importer descriptors and image remain
valid; its PNG, RenderDoc, timestamp, CPU, energy, and power results remain
pending. M4/M5/M6/M7/M8 remain `in_progress`.

### 2026-08-24 Implementation and Upload-Path Review

The fixture writer now emits the planned normal and data settings, while its
local regression verifies both the serialized TOML values and the applied
`TextureAssetDescriptor` defaults. The normal fixture resolves to linear,
DX-tangent-space metadata with offline mip generation, box filtering, and BC5
as the derived artifact target. The packed metallic-roughness fixture resolves
to linear data metadata with BC7 as its derived artifact target.

Before applying this change, the raw-RGBA upload implementation was reviewed:
`TexturePayload::Rgba8` selects `rgba8_upload_readiness` independently of the
descriptor compression target and produces an uncompressed upload plan. The
new regression asserts that both fixture assets still pass that path on an
uncompressed-only device. This is important because the test uses decoded PNG/
JPG inputs rather than precompressed containers; the metadata still informs
offline mip/compression production, but does not reinterpret raw bytes as BC5
or BC7.

`rustfmt --check` and `git diff --check` pass for the owned fixture source.
The static profile-harness contract also passed 17/17 Pester cases, including
the provenance, cold/warm separation, CPU/energy/GPU-timing evidence, and
optional RenderDoc replay gates. The added Rust regression has not run: the
coordinator-managed Windows Cargo lane remains unavailable due to the pre-Cargo
shared-artifact gate. No current PNG, RenderDoc capture, WGPU/Naga result,
timing, energy, or power metric is claimed, and M4/M5/M6/M7/M8 remain
`in_progress`.

### 2026-08-24 RenderDoc Toolchain Readiness

The supplied installation was inspected without launching a capture:
`D:\Tools\renderdoc\renderdoccmd.exe` reports x64 RenderDoc 1.44, build
`050034a0faa37d606ce1b8cf677dba4bc36984ea`, and its sibling
`renderdoc.dll` is present. This is an available local tool, not capture
evidence. The profile harness deliberately refuses an unpinned DLL: its
toolchain manifest must declare the absolute DLL path, SHA-256, and byte
length and the resolver recomputes all three before enabling
`--renderdoc-capture-once`.

At this initial inspection, no capture manifest, current `.rdc` file, replay
JSON, or screenshot existed. Bind the installed version into a
coordinator-approved toolchain manifest only together with the accepted current
viewer build and capture request, then retain the generated replay validation
beside the managed profile receipt. M4/M5/M6/M7/M8 remain `in_progress`.

### 2026-08-24 Pinned DX12 Capture Toolchain

The source-controlled machine-specific preparation manifest
`docs/tests/runtime/shader/zircon_shader_pbr_capture_toolchain_renderdoc144_dx12_20260824.json`
now pins `D:\Tools\renderdoc\renderdoc.dll` to SHA-256
`590e0b1bf885ed47c569d5e268d10577751fab5a3010dbc0e1d0262cdeed7cb0`
and 27,145,600 bytes. It declares the existing DX12 evidence policy explicitly:
`dx12` is the sole permitted WGPU backend and `vulkan` is excluded for this
particular capture. The profile toolchain resolver recomputed and accepted the
DLL and manifest fingerprints (manifest SHA-256
`a4a20efecea29cf88bb54cc7d4cec72ac7ff01590825c8664888313801dae9e8`),
and `renderdoccmd version` reported RenderDoc 1.44 build
`050034a0faa37d606ce1b8cf677dba4bc36984ea`.

This replaces the prior “no capture manifest” prerequisite only. It is a tool
identity/configuration artifact, not a viewer provenance, coordinator receipt,
RDC capture, replay result, screenshot, timing sample, energy/power sample, or
milestone acceptance. The managed current viewer build and a capture request
must still bind this manifest at runtime before M5 can advance; M4/M5/M6/M7/M8
remain `in_progress`.

## 2026-08-24 Decoded glTF Texture-Descriptor Parity Plan

Review found a concrete input-pipeline inconsistency, independent of normal-map
pixel orientation. Ordinary decoded image imports call
`TextureAsset::apply_import_settings`, so an undeclared role receives the
standard decoded-RGBA8 defaults: role-derived color space, offline mip policy,
role mip filter, compression target, and DX convention for normal metadata.
The glTF subasset path instead created `TextureAssetDescriptor::rgba8_srgb`,
then assigned only color space and `usage_hint`. A glTF normal texture therefore
ended as linear RGBA8 but retained `from_source`, Kaiser, `auto`, and no normal
convention; ordinary normal import ends as `generate_offline`, box, BC5, and
DX. Data textures similarly retained `from_source`/`auto` instead of the
ordinary decoded-data policy.

This is a semantic and artifact-quality inconsistency, not proof that current
raw RGBA8 upload is broken: raw payloads still upload uncompressed until an
offline artifact exists. The remedy is a single decoded-RGBA8 role factory on
`TextureAssetDescriptor`, used by glTF after role analysis and before sampler
translation. Its contract must equal `apply_import_settings` with only the
same `usage_hint` for every role. It must not transform source texels or flip a
green channel: the current Standard-PBR shader has a fixed DX decode contract,
while generic normal-convention conversion remains a separately versioned
texture-pipeline decision.

Unreal Interchange similarly carries glTF texture maps through explicit
material-role connections (`SetMap`) rather than inferring a normal-map pixel
rewrite from a filename. This supports role-driven descriptor convergence, not
an unmeasured shader change. The factory is import-time metadata construction
only: it adds no texture sample, material uniform, shader permutation, draw
allocation, CPU hot-loop, GPU work, energy, or power claim. Focused unit
coverage must compare the factory with the existing generic import-settings
path and verify glTF's normal/data role selection. M4/M5/M6/M7/M8 remain
`in_progress` pending managed current-source validation and product evidence.

### 2026-08-24 Implementation Result

`TextureAssetDescriptor::decoded_rgba8_for_import_usage(...)` now owns the
decoded single-mip role defaults. `gltf_decoded_texture_descriptor(...)` calls
it after the existing glTF role analysis, and only then applies the glTF sampler
translation. The removed glTF-local color-space helper was the last duplicate
policy point. The two new regressions are
`decoded_rgba8_usage_factory_matches_unconfigured_import_settings`, which
checks factory equality for albedo/normal/data/HDR/UI against the established
unconfigured settings parser, and
`gltf_normal_data_conflict_emits_role_qualified_variants`, which confirms
normal and data roles enter the same factory.

This is intentionally metadata-only. It neither changes raw source texels nor
asserts a flip/conversion for the AmbientCG `NormalGL` fixture, and raw RGBA8
remains on its existing uncompressed upload path until a derived artifact is
available. `rustfmt --check`, scoped `git diff --check`, the glTF source
contract, and the untracked-plan whitespace check pass. The two Rust tests,
Naga/WGPU validation, current HDRI PNG, RenderDoc replay, timestamp, CPU,
energy, and power measurements have not run because the clean
coordinator-managed Windows Cargo lane remains unavailable. M4/M5/M6/M7/M8
remain `in_progress`.

## 2026-08-24 glTF Normal/Data Texture-Variant Split Plan

The decoded-RGBA8 role factory exposes a remaining glTF importer defect:
`GltfTextureUsage` currently splits only conflicting sRGB and linear references.
When the same linear source image is used as both `normalTexture` and
metallic-roughness or occlusion, it emits one `data` subasset and every
material slot resolves that same URI. The normal material binding therefore
loses the normal-specific decoded-artifact policy (offline mip strategy, box
filter, BC5 target, and convention provenance). This is observable directly
from `usage_hint_for`: `normal && data` resolves to `data`.

The appropriate owner is glTF subasset projection, not Standard-PBR sampling.
Introduce a small role-qualified texture variant value carrying color space and
usage hint. Emit one variant per required derived policy: preserve the existing
single bare URI for an unambiguous texture, preserve `Srgb`/`Linear` suffixes
for color-space conflicts, and create `Normal` plus `Data` suffixes only when
one linear source is genuinely shared across those incompatible roles. Material
texture references must select the corresponding variant by slot role. The
URI becomes the source/derived artifact identity until the general texture
pipeline can deduplicate shared decoded source bytes below multiple derived
artifacts.

This follows the same source-role separation used by Unreal Interchange's
glTF material `SetMap` connections, while fitting Zircon's current
one-descriptor-per-`TextureAsset` contract. It must not flip pixels or alter
the established glTF no-flip shading math. Runtime cost remains unchanged:
the shader, bind layout, material uniform, draw key, and number of texture
samples do not change. Import-time CPU/memory may duplicate decoded RGBA8 only
for a source that is simultaneously consumed under incompatible normal/data
policies; measure that rare conflict case before claiming an import performance
benefit or cost bound. Unit regressions must cover variant enumeration, stable
labels, slot-role reference routing, and descriptors. M4/M5/M6/M7/M8 remain
`in_progress` pending managed Windows validation and product evidence.

### 2026-08-24 Implementation Result

`GltfTextureUsage::texture_variants` now projects each required derived
artifact role explicitly. A normal/data collision emits `TextureN/Normal` and
`TextureN/Data`, while an unambiguous texture keeps its existing bare label and
the prior sRGB/linear collision remains `Srgb`/`Linear`. The glTF texture
subasset loop makes the necessary decoded-RGBA8 clone only before its final
variant consumes the source buffer. `material::texture_reference` receives the
slot's color-space and import-role contract, so normal, metallic-roughness, and
occlusion bindings resolve exactly to the matching derived subasset.

Focused regressions now cover normal/data variant enumeration, both stable URI
labels, normal/data descriptor defaults, and material slot routing. `rustfmt
--check` passed for the importer, material routing, descriptor, and HDRI
fixture sources; scoped `git diff --check` passed with only the repository's
LF-to-CRLF advisory; and the source contract passed with eight production plus
regression `texture_reference` call sites. This is source-level evidence only:
the new Rust tests have not run while the shared coordinator-managed Cargo
artifact lane is unavailable. No WGPU/Naga result, current HDRI PNG, RenderDoc
replay, GPU timestamp, CPU, energy, power, or import-memory measurement is
claimed. M4/M5/M6/M7/M8 remain `in_progress`.

### 2026-08-24 Review Follow-up

Independent review found no P1 production defect in the role-variant design,
but correctly identified that the initial regressions tested texture projection
and material routing independently. The now-corrected plan naming refers to
`gltf_normal_data_conflict_emits_role_qualified_variants`. A new minimal
composition regression,
`gltf_normal_data_texture_variants_connect_imported_assets_to_material_slots`,
parses one glTF texture shared by `normalTexture` and
`metallicRoughnessTexture`, runs texture plus material subasset projection, and
asserts both derived entries/descriptors, both material slot locators, and both
material dependencies. A companion label regression locks the unchanged bare
URI plus the established `Srgb`/`Linear` conflict names. This closes the static
coverage gap without changing runtime code or introducing capture/test-only
behavior. To retain the importer as an orchestration boundary, these
role-variant regressions live in
`gltf_labeled_subassets/texture_variant_tests.rs`; the implementation file is
842 lines after the extraction rather than accumulating a second test
responsibility at the 1000-line boundary.

The composed Rust regression still requires the managed Cargo lane before it
can be reported as executed. Its static source contract, `rustfmt --check`, and
scoped `git diff --check` pass; no Naga/WGPU, PNG, RenderDoc, timing, energy,
power, or import-memory result is claimed. M4/M5/M6/M7/M8 remain
`in_progress`.

## 2026-08-24 Current Product-Evidence Recheck

**Status: implementation remains ahead of runtime acceptance.** The dedicated
Standard-PBR material-input export target is
`runtime_shader_pbr_real_hdri_lakes_ambientcg_metal009_texture_maps_standard_inputs_20260824.png`.
It is still absent from `docs/tests/runtime/shader`; no file was created by the
source-only fixture work. The most recent managed-recorder attempt also has no
terminal build or execution receipt: its coordinator `cargo.acquire` request
was accepted but reconciliation ended in `command_post_timeout`. This is a
coordination failure before a Cargo child can establish current-source
provenance, not a Shader06 product-test result.

Two pre-existing candidate images were inspected only as historical visual
context. The P1-2 8x8 matrix shows continuous metallic/roughness variation,
and the older Metal009 HDRI image retains the expected environment highlight
and normal-detail response. Their 2026-08-22/23 timestamps and absent current
build receipts mean they cannot validate the current glTF role-variant,
material-input, IOR, or realtime-IBL changes. Likewise, the associated
21-frame CPU text reports (`initial=780.610ms`, `update=871.090ms`,
`warm=905.681ms` averages) are non-comparable historical data rather than an
optimization result: the warm average is not an improvement and it does not
include the required named-adapter attribution, GPU duration, energy, or power
evidence.

The capture entrypoint already fail-closes on the source-controlled DX12
toolchain manifest: it resolves the manifest before creating evidence, pins the
RenderDoc DLL fingerprint, and records the selected backend in the profile
manifest. Do not add a second capture helper or speculate about a cache/uniform
lifecycle rewrite. Once the coordinator provides a terminal current-source
artifact receipt, run the managed Rust/WGPU/Naga contracts, export the missing
Standard-PBR PNG, collect the named-adapter cold/warm CPU and GPU reports, and
capture/replay the terminal-SH9 RenderDoc frame in the same provenance chain.
Until then M4/M5/M6/M7/M8 remain `in_progress` and no CPU, GPU, energy, power,
or cross-engine efficiency claim is made.

## 2026-08-24 Dispatch-Ownership Static Recheck

The open local handoff was rechecked against the current source without
reconstructing dispatch dimensions in another layer. The authoritative
`IBL_BAKE_IRRADIANCE_SH9_DISPATCH_GROUPS = [1, 1, 1]` remains defined in
`ibl_bake_graph_plan`; both `ibl_bake_shader_plan` and
`realtime_ibl_graph_plan` consume it. `RealtimeIblWgpuRecorder` deliberately
does not duplicate that constant: its `sh9_command` delegates to
`ibl_bake_irradiance_sh9_kernel_plan`, then materializes the canonical WGPU
command plan. This preserves the one-way graph/shader/WGPU ownership model.

The focused source-contract check passed all six assertions: graph owner,
offline shader-plan consumer, realtime-graph consumer, recorder canonical-SH9
delegation, graph-versus-encoded-command parity regression, and complete-ticket
budget regression. The latter still asserts exactly `4_124` workgroups over 21
batches. This verifies the previous `4x4x6` metadata drift is not present in
the current source; it does not execute the Rust tests, submit WGPU work, or
close `failure-2026-08-22-realtime-sh9-graph-dispatch-parity.md`. That record
and M4/M5/M6/M7/M8 remain `open`/`in_progress` until a current managed product
run supplies the required PNG, timing, and RenderDoc evidence.

## 2026-08-25 PBR Viewer Terminal-Outcome Infrastructure

The shader evidence consumer had a separate P0 process-contract defect: fatal
ApplicationHandler paths could call `event_loop.exit()` and allow `run_app()`
to return success. `zircon_shader_pbr_viewer` now carries an app-owned terminal
outcome through `main`, atomically writes
`zircon_shader_pbr_viewer_terminal_outcome.json` under its non-C: work root,
and returns stable nonzero codes for ten fatal categories (11--20). It records
three terminal states, the failure phase/category/message, viewer/HDRI source
chain, screenshot/GPU timing/RenderDoc artifact commit states, and cleanup
state. Startup failures before the app object also write the same schema.

This only corrects the binary outcome contract; it is not shader/PBR visual or
performance acceptance. A native-surface failure that follows the existing CPU
fallback remains recoverable, while a requested but unverified RenderDoc result
is reported as `not_committed`. The background scene loader now retains its
named-thread `JoinHandle` and cooperative cancellation token; every terminal
owner requests cancellation, and the post-loop owner performs a two-second
bounded join. The terminal record distinguishes completed/cancelled join from
timeout and join panic, and treats the latter two as nonzero `task_shutdown`
failures. These are static control-path changes only: no managed Cargo test,
WGPU/Naga submission, current-source screenshot, RenderDoc replay, CPU/GPU
timing, energy, or power measurement ran. M4/M5/M6/M7/M8 remain
`in_progress`.

The same viewer now labels its evidence host explicitly. `--host-mode`
resolves to either `offscreen-diagnostic` or `native-present`; screenshot and
GPU-timing evidence is permitted only in the former, while native viewport
surface errors are terminal rather than a hidden CPU-readback fallback. Ready
sidecars use schema v14 and record the standalone diagnostic composition ID,
mirror-sphere scene ID, CPU-readback capture target, and zero GPU scene-surface
presents. The managed profile and validator both require
`offscreen-diagnostic`. This is an evidence-capability correction, not a
product-host claim or visual/performance acceptance; M4/M5/M6/M7/M8 remain
`in_progress` pending a managed current-source run.

## 2026-08-30 Explicit Capture Artifact Runtime-Cache Writeback

The C4 identity prerequisites now reach the existing asynchronous writeback
owner. A capture request may carry a validated `IblBakeArtifactRequest` in
addition to its output URI; the capture source resolves the project cache store,
registers PMREM/SH9 sections from the filtered GPU target, and prepares those
copies through the bounded product-diagnostic router. The diagnostic frame and
capture command buffer share one submission ticket. Only after submission does
`IblBakeRuntimeGraphWritebackQueue::commit_submitted` retain the pending item;
the normal backend completion poll then assembles the sections and writes the
runtime cache. Duplicate requests and the existing four-entry limit still
short-circuit admission. Preparation, encoding, submission, completion
readback, and runtime-cache write failures abandon only the optional writeback
without invalidating the visible capture or publishing a partial artifact.

This closes the runtime-cache persistence subphase, not editor asset-derived
staging: `output_uri` remains carried as stable destination identity and is not
used as an unvalidated filesystem path. No current-source Cargo build, WGPU
run, screenshot, RenderDoc replay, timing, RSS, power, or energy result is
claimed; M4/M5/M6/M7/M8 remain `in_progress`.
