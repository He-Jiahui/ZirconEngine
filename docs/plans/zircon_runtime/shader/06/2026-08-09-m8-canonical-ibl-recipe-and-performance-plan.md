# Shader06 M8 Canonical IBL Recipe And Performance Plan

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Milestone: M8
Status: implementation_complete_pending_managed_validation
Depends on: M3, M4, M5
Owners: zircon_runtime core/framework render environment, asset importer, graphics scene renderer environment, zircon_app shader PBR viewer
Files: ["docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md", "docs/plans/zircon_runtime/shader/06/2026-08-09-m8-canonical-ibl-recipe-and-performance-plan.md", "docs/tests/runtime/shader/2026-08-13-startup-performance-architecture-review.md", "zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/app_tests.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/gpu_timing_evidence.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/work_paths.rs", "zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs", "zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs", "zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs", "zircon_runtime/src/asset/importer/environment_ibl.rs", "zircon_runtime/src/asset/importer/environment_ibl/import_settings.rs", "zircon_runtime/src/asset/importer/environment_ibl/source_identity.rs", "zircon_runtime/src/asset/importer/environment_ibl/source_staging/output.rs", "zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs", "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs", "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs", "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_resolution.rs", "zircon_runtime/src/core/framework/render/environment/ibl_bake_recipe.rs", "zircon_runtime/src/core/framework/render/environment/mod.rs", "zircon_runtime/src/core/framework/render/environment/skybox.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap/rebuild.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs", "zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs", "zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs", "zircon_runtime/src/graphics/mod.rs", "zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/environment_ibl_hydration_cache.rs", "zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/environment_ibl_compile_options.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/mod.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_readback.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_capture_wgpu.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_timestamps.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs", "zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs", "zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mod.rs", "zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs", "zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix_quantitative.rs", "tools/shader-pbr-profile-contract.ps1", "tools/write_zircon_shader_pbr_build_provenance.ps1", "tools/zircon_profile_shader_pbr_viewer.ps1", "tools/zircon_summarize_shader_pbr_profile.py", "tools/tests/zircon_profile_shader_pbr_viewer.Tests.ps1", "tools/tests/test_zircon_summarize_shader_pbr_profile.py", "tools/tests/test_zircon_validate_shader_pbr_viewer_evidence.py", "tools/tests/test_zircon_validate_shader_pbr_gpu_timing_evidence.py", "tools/zircon_validate_shader_pbr_gpu_timing_evidence.py", "tools/zircon_validate_shader_pbr_viewer_evidence.py"]

The GPU timing evidence writer also owns the `sha2` workspace dependency declared
by `zircon_app/Cargo.toml` and recorded in the root `Cargo.lock`.

Current implementation scope additionally includes the runtime reservation bridge:
`graphics/environment_ibl_bake_reservation.rs`, `graphics/mod.rs`, frame-context
construction and the three runtime submission paths, plus the scene-renderer
submission and compiled-scene render path. These files preserve cache-private
ownership while carrying one cache-miss reservation to the GPU writeback queue.

The staging data contract is isolated under
`zircon_runtime/src/asset/importer/environment_ibl/source_staging/`; the
`environment_ibl.rs` entry module retains source/derived artifact orchestration only.

## Decision

M8 first makes the persisted IBL recipe authoritative. It does not treat CPU
loop parallelism, WGSL source-volume reduction, or DX12 pipeline prewarm as a
substitute for agreeing on which algorithm is allowed to write a `.zribl`
artifact.

The MVP decision is:

- Imported source cubemaps keep the current CPU offline baker as the only
  canonical asset-derived `.zribl` producer until a GPU implementation passes
  the same declared recipe contract.
- GPU realtime IBL remains a frame-scheduled renderer feature. Its separate
  runtime-cache tier may retain a fallback result, but it cannot overwrite the
  asset-derived artifact or claim canonical CPU/GPU artifact equivalence.
- The future GPU offline baker is a capability behind the same recipe, not a
  second cache format selected by the call site. It may become the canonical
  producer only after CPU/GPU artifact parity and timing evidence are accepted.

This retains a headless/import-time MVP path, makes cache reuse deterministic,
and avoids forcing asset import to require a graphics device.

## Evidence

| Area | Current Zircon evidence | Reference evidence | Consequence |
| --- | --- | --- | --- |
| Persistent source IBL | `asset/importer/environment_ibl.rs` builds source mip, PMREM, SH9, and optional IEM on CPU, then stages `.zcube` and `.zribl`. | cmftStudio starts PMREM/IEM as an explicit background job and publishes it only after completion and GPU upload (`dev/cmftStudio/src/cmftstudio.cpp`). | The import/staging owner needs a declared bake result, not an implicit renderer side effect. |
| GPU IBL bake | `ibl_bake_graph_plan.rs` allocates transient PMREM/SH9/IEM outputs; `ibl_bake_runtime_writeback.rs` can read them back and write an artifact. | Unreal uses RDG resource lifetime, explicit source mip selection, and explicit readback only at the result boundary (`ReflectionEnvironmentCapture.cpp`). | GPU work and persistence are both present, but require a shared semantic contract. |
| Diffuse source resolution | CPU `source_cubemap_irradiance_mip_level` stops at the configured 32x32 source face; CPU IEM integrates texel solid angles at that mip. | Unreal `GetDiffuseConvolutionSourceMip` also reduces the source to no more than 32x32 before SH convolution; its reference IEM chooses a low mip in `ReflectionEnvironmentDiffuseIrradiance.cpp`. | High-resolution source texels are not required for diffuse irradiance and must not become an accidental cost multiplier. |
| GPU IEM semantics | M8 routes the framework-selected source mip through `IblIrradianceCubeParams` into `textureSampleLevel`; GPU IEM still uses a Hammersley estimator and existing parity only compares it against GPU SH9 at low frequency. | Unreal distinguishes production SH and reference IEM paths and records their selected source mips. | The source-resolution contract is shared, but CPU and GPU IEM are not interchangeable persistent outputs today. |
| DX12 startup | Historical phase accounting attributes the former 82--100 second delay to synchronous DX12 WGSL/PSO creation, not HDR decode, PMREM, cache restore, PNG, or readback. | Unreal routes reflection filtering through named RDG passes and pipeline-state infrastructure instead of hiding it in the source asset path. | PSO startup and cache-miss IBL must be measured and optimized independently. |

### 2026-08-10 Reference Re-audit

The pre-optimization re-audit retains Unreal as the runtime reference. Its
`GetDiffuseConvolutionSourceMip` caps diffuse input at 32 faces, its runtime
SH path uses nine coefficients, and its reflection shader selects a filtered
cubemap mip from roughness before applying the split-sum preintegrated-GF
term. Zircon has the matching boundaries: the framework recipe selects the
32-face source mip, `zr_environment_core.wgsl` samples the PMREM at its
roughness mip and applies `f0 * A + saturate(50 * f0.g) * B`, and the source
cubemap regression binds both WGSL roughness constants to the canonical recipe.

cmft remains a useful offline comparison: it can reconstruct an irradiance
cubemap from fifth-order SH, while cmftStudio only schedules cmft's PMREM and
irradiance jobs in the background. Neither is a reason to replace the MVP
Unreal-aligned SH9 runtime contract or to promote the GPU Hammersley IEM
result. No sampling budget, cache identity, shader result, or startup policy
changed from this audit. The next optimization may begin only after the
five-run cold/warm profile attributes the current-machine cost to a named
phase rather than the historical DX12 PSO baseline.

The exact CPU IEM work at its selected 32x32 source mip is
`6 * 32^2` output texels multiplied by `6 * 32^2` source texels, or
37,748,736 source/output candidate iterations before source-texture access.
The direct convolution rejects non-positive `n dot l` candidates before a
source texel load, so actual reads are lower and direction-dependent. The
six-face executor can distribute output faces, but it cannot change that
algorithmic candidate bound. It remains an unaccepted leaf improvement pending
this milestone's contract and measurement stages.

## Structural Finding

Before M8, `IblBakeArtifactDescriptor` identified only a source key, layouts,
content bits, and one algorithm version. M8 records `AssetImporterCpu` or
`RendererGpuRuntime` provenance in the v3 descriptor header (120 bytes), then
hard-cuts `.zribl` to v4 by inserting a 32-byte BLAKE3 payload checksum between
that header and the payload. Reads reject equal-length corrupted payloads before
hydration; v3 artifacts are intentionally stale and rebuild. Asset-derived
selection accepts only CPU provenance; runtime-cache selection and writes accept
only GPU provenance. A source-cubemap key can therefore use both stores without
claiming that the two payloads are interchangeable. The current implementations
still differ for IEM in one observable way:

- CPU selects `source_cubemap_irradiance_mip_level` and integrates the
  discrete cubemap solid angle.
- GPU IEM samples the same selected source mip, but uses a Hammersley
  estimator instead of CPU discrete solid-angle integration.

The current GPU test tolerance is useful as a rendering quality smoke check,
but it is not artifact equivalence. Reusing the same persistent descriptor for
both results makes cache contents dependent on which path wrote first. This is
a correctness and reproducibility defect, not a micro-optimization issue.

A cache hit is not sufficient by itself: the chosen payload must replace the
source environment's PMREM/SH9/IEM content before graph compilation omits the
bake pass. The cold-process runtime-cache hit now performs that replacement,
and a four-entry framework-owned memo keyed by the complete request identity
reuses the hydrated Arc-backed source/PMREM/IEM data and prepared upload rows
on stable frames. Memo hits apply current intensity and rotation; misses are
not memoized because GPU writeback becomes available on a later frame. The
environment now retains the accepted artifact descriptor after successful
hydration. Replacing optional IEM content invalidates that provenance without
changing the source bake key. Artifact application publishes only after
successful decode, so corrupt or
layout-mismatched payloads do not erase the source skybox on their error path.

The persistent CPU asset-derived path now follows the same rule without first
rebuilding discarded data: it combines the staged `.zcube` source mips with
the validated `.zribl` PMREM/SH9/IEM sections directly. This removes an
otherwise redundant CPU PMREM bake on a process-cold asset-cache hit while
retaining the original source mips and their separately evaluated canonical
SH9 cache for skybox and future PMREM rebakes. Its startup effect remains
unmeasured until a current managed capture is available.

## Target Boundary

`zircon_runtime::core::framework::render::environment` owns the immutable
`IblBakeRecipe` contract. It defines the source-mip policy, PMREM sampling
policy, diffuse representation and integrator, output layout, numeric format,
and recipe identity used by the artifact descriptor. It is a framework data
contract, not a graphics implementation. The v3 descriptor's producer plus
recipe identity distinguishes CPU solid-angle from GPU Hammersley diffuse
integration; a future GPU canonical promotion must receive its own accepted
algorithm version.

`zircon_runtime::asset::importer` owns source decode, staging, and CPU fallback
execution. `zircon_runtime::graphics::scene::scene_renderer::environment` owns
GPU command recording, in-process pipeline caching, transient resources, GPU
timestamps, and readback. Neither consumer may invent a recipe or write a
persisted artifact with an undeclared one. `zircon_app` only requests the
viewer profile and reports phases; it does not choose an IBL algorithm.

The initial recipe uses the established source-mip policy and CPU result as the
canonical persisted behavior. A future GPU recipe must either produce the same
contract within declared per-section error bounds or receive a distinct recipe
identity and artifact invalidation. M8 advances the algorithm version to
`2026_08_09_0006` because the runtime GPU IEM now samples its declared source
mip rather than mip zero. The preceding final-storage allocation refactor
remains output-equivalent and was correctly left at `2026_08_02_0005`.

## Milestones

### M8.1 Recipe And Artifact Authority

- Completed: framework-owned diffuse source-mip policy is consumed by CPU and
  GPU IEM plans; v3 descriptors record producer provenance; runtime cache
  writes, reads, and selection reject CPU-provenance blobs; asset-derived
  writes and reads reject GPU-provenance blobs; cold runtime-cache hits
  rehydrate the source environment before suppressing graph bake; and the
  `2026_08_09_0006` algorithm version invalidates the prior GPU mip-zero cache.
- Implemented, managed verification pending: `.zribl` format v4 inserts a
  32-byte BLAKE3 payload checksum after its header and rejects corruption before
  payload hydration. The second review forward-fixed the runtime-cache
  writeback length contract to include the checksum bytes; scoped `rustfmt`
  and diff integrity pass. Managed Rust execution remains required before this
  integrity slice is accepted.
- Implemented, validation pending: a four-entry framework memo performs one
  artifact load/decode/rehydration, then uses Arc-backed clones for the next 60
  stable submissions with zero file access or upload-row re-encoding while
  preserving intensity and rotation. `SourceCubemapEnvironment` now retains
  the accepted descriptor after validated hydration and drops it if a manual
  IEM replacement changes uploaded content. Both direct CPU and general
  hydration recheck the descriptor against the concrete environment request;
  direct hydration additionally rejects the runtime GPU producer.
- Implemented, validation pending: frame compilation gives a current accepted
  CPU artifact descriptor precedence over runtime-cache resolution, so an
  importer-provided PMREM/SH9/IEM payload remains active and optional IEM is
  not replaced by a PMREM/SH9 runtime fallback. A cache miss now reserves its
  request in the framework-owned state before emitting one bake graph; later
  frames observe that pending reservation and omit duplicate IBL passes until
  the cross-frame writeback publishes the runtime-cache payload. The reservation
  is an owned frame-to-writeback handle: build, prepare, graph, device poll, map, or disk
  write failures drop it and make the request retryable, while a submitted
  readback retains it until completion. Without a project cache store, the
  source CPU environment remains active and frame compilation omits the
  otherwise unconsumed runtime bake graph on every frame.
- Implemented, validation pending: the framework recipe owns PMREM sample
  budgets, roughness mapping, diffuse source LOD, optional IEM face size,
  output format, and distinct CPU/GPU integrator identities. Artifact
  descriptors derive the corresponding identity, while renderer pipeline keys
  hash included WGSL source directly rather than trusting manually incremented
  hash constants.
- Remaining: managed tests must exercise descriptor/integrator mismatch,
  source-mip propagation, accepted-descriptor invalidation, and the
  source-derived WGSL cache key before any canonical GPU promotion.

### M8.2 Diffuse Correctness Before Throughput

- Completed: the GPU IEM parameter block now receives the framework-selected
  source mip. It remains a noncanonical runtime-cache fallback until artifact
  parity is accepted.
- Implement CPU/GPU IEM comparison against the CPU IEM output, including high
  frequency, constant environment, cube-edge, and source face sizes above 32;
  managed execution must still publish the resulting error values.
- Completed source-LOD coverage: the WGPU IEM regression uses a 128-face,
  eight-mip cubemap with deliberately different constant colors per mip and
  requires the output to match the canonical selected mip. CPU-IEM parity and
  error statistics remain required before canonical promotion.
- Completed statistics foundation: framework-owned
  `SourceCubemapIrradianceErrorStatistics` reports RGB absolute-error mean and
  maximum across every texel and separately across face-edge texels, plus the
  difference of CPU/GPU continuity deltas across real Cubemap face seams. The
  offscreen GPU IEM comparison consumes it and validates layout, sample counts,
  edge/seam counts, and finite values without inventing an unmeasured
  tolerance.
- Completed scenario coverage: the offscreen parity suite now compares CPU and
  GPU IEM for a constant environment and a high-frequency face-specific
  cube-edge signal at 64 faces, seven mips. Its CPU reference constructs only
  the source mip chain required by direct IEM, so the regression does not spend
  PMREM work merely to test diffuse parity. Each scenario emits total, edge,
  and cubemap-seam RGB mean/max statistics when the managed test invokes the
  Rust harness with `--nocapture`; preserve that output for the M8.2
  measurement record.
- Verified convention: CPU direct convolution and GPU hemisphere sampling both
  normalize the cosine-weighted result, and consumers multiply the resulting
  diffuse environment by albedo directly. This matches Unreal's division by
  accumulated weight and cmftStudio's direct irradiance-albedo product. Do not
  introduce an extra `pi` multiplier; it would over-brighten the current PBR
  convention.
- Record the resulting maximum, mean, and seam error by RGB channel from
  current-machine managed runs. A low-frequency SH-only comparison is
  insufficient for artifact authority.

### M8.3 Measured CPU Bake Work

- Retain direct final-storage writes and caller-owned executor reuse only after
  M8.1 proves they cannot alter the canonical result.
- Completed attribution surface: counters cover equirect projection, source
  mip construction, PMREM, SH9, IEM, staging write, and cache reuse. They
  report wall time, actual persisted output sizes, source/PMREM layouts, and
  aggregate submitted caller-executor work-item count plus its equirect
  projection, source-mip, PMREM, and IEM chunk phases; process CPU sampling
  remains the WPR responsibility in M8.4. Ready v12 carries those dispatch
  shapes but does not infer worker utilization or elapsed CPU time from them.
- Completed CPU import attribution: `EnvironmentIblSourceStagingReport` now
  carries wall-time ownership for source decode, combined cubemap build,
  equirect projection, source-mip construction, PMREM, SH9, IEM construction,
  and bundle write. `cubemap_build` remains the enclosing owner duration and
  the new sub-phases are diagnostic attribution rather than additional total
  terms. Reuse preserves the decode time that actually occurred before the
  cache decision. Container cubemap imports report decode/IEM/write but leave
  equirect-only sub-phases at zero. The PBR viewer carries those exact fields
  into its v12 Ready-frame sidecar, alongside actual `.zcube`/`.zribl` byte
  counts, aggregate caller-executor chunk work, its four submitted chunk
  phases, and direct IEM source/output candidate iterations. The retained
  v11/v12 field name contains `source_sample_visits`,
  but it is a layout-derived upper-bound work metric: convolution first
  evaluates `n dot l` and only positive candidates load a source texel. It is
  not an actual texture-read counter. The
  verifier requires every inherited IBL field for v12, requires non-empty persisted
  outputs, rejects cache reuse with submitted executor work, and requires
  submitted work for a written viewer
  HDRI. That last rule is scoped to the viewer's mandatory caller-owned
  parallel equirectangular path; serial container imports may correctly report
  zero and do not produce this viewer sidecar. It validates the
  four serial build subphases against their `cubemap_build` envelope, then
  validates the non-overlapping staging phases against `staging_elapsed`; it
  never double-counts those diagnostic subphases into the public total. The
  next DX12 screenshot can therefore identify CPU IBL ownership rather than
  only report a single restore number.
- Completed asset-cache hydration: a current staged `.zcube` plus CPU
  asset-derived `.zribl` now creates the prepared source environment directly
  from source mips and decoded artifact sections. It does not run the CPU
  PMREM baker only to discard that output on the following line, and transfers
  the decoded `.zcube` texel allocation into the environment rather than
  copying the full source pyramid again. It retains a separate source-derived
  SH9 cache so later PMREM reconfiguration cannot reuse artifact SH9 as source
  truth.
- Implemented, validation pending: importer source identity now feeds HDR or
  container bytes to BLAKE3 once, finalizes the unchanged source revision, then
  appends source face/mip layout and finalizes the unchanged source hash. The
  previous two full-source passes become one (50% fewer source bytes hashed),
  with legacy-equivalence tests covering empty, changed, and multi-layout
  sources. Source identity and the staging report contract now live in
  `environment_ibl/source_identity.rs` and `environment_ibl/source_staging/`;
  the entry orchestration module remains below 900 lines and below the warning
  threshold.
- Implemented, validation pending: `.zcube` now has its own source-stage
  version and hashes only source identity plus source face/mip layout. A valid
  decoded source is reused when only `.zribl` is missing or stale, so HDR
  projection and source-mip construction are not repeated. Source and derived
  targets publish through one durable bundle transaction: a recipe-only rebuild
  retains byte-identical `.zcube` content for the same source identity, but
  may replace that target together with `.zribl` so recovery never exposes a
  cross-generation pair. The resulting write cost is unmeasured and must not
  be optimized before the current-machine profile attributes it. PMREM layout
  and optional IEM contents remain part of the independent derived artifact
  identity.
- Implemented, validation pending: the profile runner now writes both requested
  and active source/PMREM layouts plus every IBL staging subphase into each run
  report. Its summarizer rejects a sidecar whose requested or active layout does
  not match the profile manifest or whose aggregate chunk count does not equal
  its four phase counts, and emits phase medians including `pmrem_build` and
  PMREM chunk submissions. It also emits
  `ibl_post_stage_hydration_median_ns` from each run's
  `ibl_total_elapsed_ns - ibl_staging_elapsed_ns`, rejecting impossible negative
  intervals rather than subtracting independent medians. Omitted layout arguments are bound as `automatic`, so the
  actual viewer default and an explicit independent PMREM run can be compared
  without changing source resolution; the active layout must also agree across
  cold and warm samples. The summarizer replays each Ready PNG/sidecar through
  the current v12 validator in process, using the same JSON payload as its CLI
  without starting a fresh Python interpreter per run; saved validation payloads
  must exactly match that replay. Focused Python summary and validator contracts pass; no
  managed current-viewer measurement has been claimed.
- Corrected architecture audit (2026-08-15): the current 512-face CPU PMREM
  exposes six full-face tasks per mip. Mip zero has zero roughness and takes
  the direct source-sample path, so it does not execute the declared 32-sample
  GGX loop. The recipe-derived source-cubemap trilinear-call upper bound is
  18,417,408: 1,572,864 direct mip-zero calls plus 16,844,544 importance or
  cosine sample-loop calls. Mips zero and one account for 76.861% of that
  bound; the prior 67,176,192/93.656% record incorrectly charged mip zero with
  GGX samples. `docs/tests/runtime/shader/2026-08-13-startup-performance-architecture-review.md`
  records the corrected 512-versus-128 comparison, cmft/cmftStudio task-lifecycle
  distinction, and the required WPR/v12 evidence gate. This is not a measured
  bottleneck and does not authorize a resolution reduction or task-tiling
  change before the declared profile matrices complete.
- Implemented, coordinator integration pending (2026-08-15): the Viewer HDRI
  exposure sidecar now publishes through the runtime's durable sibling-staging
  `atomic_write` primitive. Its schema, cache key, fallback decode path, and
  canonical IBL timing boundary are unchanged; the focused source contract and
  `rustfmt --check` pass. This is intentionally separate from the shared IBL
  artifact writers.
- Implemented, coordinator integration and managed validation pending
  (2026-08-15): source `.zcube`, importer-derived `.zribl`, and renderer-runtime
  `.zribl` publish through durable storage without changing cache identity or
  producer boundaries. M8 additionally closes the source/importer pair as one
  recovery-backed bundle transaction: it pre-encodes both targets, commits them
  through one durable journal and owner lock, recovers an interruption before
  staging or after the first target commit, and makes readers retry until their
   source and derived snapshots are from the same settled generation. A
   standalone derived writer takes that same lock and rejects replacement when a
   paired source exists, so it cannot reintroduce a source/derived TOCTOU path.
   Environment and source-only readers also retry a source miss when a publisher
   creates the journal after their initial recovery check. Before reporting a
   miss, they acquire the same bundle owner lock used by publication, then
   re-read the source and journal while that lock is held. Focused interruption,
   interleaved-read, source-miss, final-barrier, and paired-write rejection
   contracts exist in source; scoped `rustfmt --check` and `git diff --check`
   pass without starting Cargo. Managed Rust/WGPU validation remains required
   before this is accepted as current product evidence. This coherence retry is
   deliberately not a cache-hit I/O optimization: a future lightweight
   source-fingerprint / manifest-header readiness path belongs to Optimize 09F1
   P0-9 and the 09D residency owner, and may replace full decode only after
   current-source profile evidence proves the cost while preserving this
   settled-generation guarantee.
- The viewer's canonical `ibl_total_elapsed` excludes its local exposure
  sidecar, but the broader `scene_startup_ibl_restore` wall-clock field still
  encloses that local persistence. Keep IBL attribution on the canonical field
  until the shared viewer timing schema can lease and separate the local phase.
- Evaluate an algorithmic diffuse replacement only from the M8.2 error data;
  do not replace direct IEM with SH reconstruction merely because it is faster.

### M8.4 Startup And GPU Attribution

- Keep DX12 Base/Deferred/SSS PSO creation separate from environment bake
  phases. Known exact Base prewarm belongs to renderer startup reporting, not
  to cache-hit IBL time.
- Verified instrumentation path: the renderer's existing `GpuPassTimer`
  reserves every unculled compiled render-graph pass before execution,
  including PMREM, SH9, and IEM bake passes. Its delayed WGPU timestamp result
  is merged by pass name into `RenderFrameProfile.passes`; do not add a
  competing IBL-only query/readback queue. The HDRI viewer takes the direct
  renderer path rather than a compiled graph, so it must use named direct
  encoder scopes on that same timer and shared readback queue. A GPU-runtime
  IBL bake measurement remains a separate managed compiled-graph scenario; a
  CPU-imported HDRI viewer run must not claim it measured GPU baking.
- Implemented, validation pending: runtime writeback encodes its copy command
  into the main frame submission, starts mapping only after submit, and keeps a
  four-entry cross-frame pending queue completed through nonblocking
  `device.poll(Poll)`. `submit_compiled_scene_frame` no longer performs a
  second submit or `wait_indefinitely`; the remaining one-time file write is
  performed when a later poll observes the mapped payload. The framework marks
  the initial cache miss pending before graph compilation, preventing that
  asynchronous file-write window from recompiling and resubmitting the same
  PMREM/SH9 graph every frame. A cache-miss reservation moves from frame
  compilation to the queued readback and is released on every failed path, so
  a failed build, submission, map, or cache write cannot suppress all future
  retries for that request.
- Capture cold and warm runs on the same machine with five repetitions each.
  Use WGPU timestamp queries for GPU IBL, engine phase counters for ownership,
  and Windows Performance Recorder CPU sampling for driver/WGSL/PSO stacks.
- Store every generated ETL, timestamp report, PNG sidecar, and RenderDoc
  capture below `E:\Git\ZirconEngine\docs\tests\runtime\shader`; no task
  artifact is placed on `C:`. RenderDoc replay uses
  `D:\Tools\renderdoc\renderdoccmd.exe replay --loops 1`. The immutable
  replay snapshot is created beside the capture rather than through the system
  temporary directory, preserving this E: artifact boundary even while replay
  validation is running.
- Implemented, validation pending: `--work-dir` owns project staging, the
  default IBL cache, and the default RenderDoc capture template; an explicit
  `--ibl-cache-dir` or `--renderdoc-capture-path` still wins. The default is
  the workspace `docs/tests/runtime/shader` evidence root, with a D: fallback
  if the workspace itself is on C:, and every artifact option rejects C:
  explicitly. Managed viewer runs therefore remain within the required
  non-C artifact boundary without relying on the platform temporary directory.
- Implemented, validation pending: `--gpu-timing-report <path.txt>` enables
  timestamp resources only for a screenshot request, records the direct
  encoder's realtime-IBL work when present, then GPU-scene upload, scene,
  output-transfer, overlay, and conditionally UI scopes through the shared
  timer, and waits for the matching screenshot frame generation through at
  most eight nonblocking redraws. HDRI `EnvironmentOnlyPbrPreview` evidence
  must include the `direct_gpu_scene_upload`, `direct_scene_content`,
  `direct_output_transfer`, and `direct_overlays` diagnostic scopes. The
  upload scope remains present with zero GPU duration when no encoder upload is
  needed; realtime IBL and UI scopes are conditional. The report excludes CPU
  preparation, queue writes, readback copies, and presentation, so it is a
  named GPU-stage report rather than a fabricated direct-frame total. It
  identifies a measured, unavailable, or timed-out result rather than
  substituting CPU wall time for GPU duration.
- Implemented, validation pending: renderer startup now builds the environment
  BRDF LUT at Unreal's default `128 * 32 * 128 = 524,288` CPU integration
  iterations instead of `128 * 128 * 1024 = 16,777,216`. This is a 32x work
  reduction without changing Zircon's established visibility/integration
  model. A 16x16 sparse comparison of 128 samples against the same integrator
  at 4,096 samples measured mean absolute RG error `0.00238490` and maximum
  absolute error `0.01624132`; the Rust regression gates those values at
  `0.003` mean and `0.02` maximum. The quantitative PBR test now samples the
  actual rectangular runtime LUT rather than a square test-only surrogate.
- Completed startup-attribution infrastructure: Ready schema v12 reports
  renderer-lifetime shader/pipeline creation counts and CPU times plus
  successful async Base admission queue wait. The EnvironmentOnly Base PSO now
  uses a layout with no group 1 forward receiver, and the opaque pass only
  creates/binds that receiver when a generic Base variant is actually present.
  Unknown variants and a local-reflection-provider upgrade retain the generic
  ABI. `docs/tests/runtime/shader/2026-08-13-startup-performance-architecture-review.md`
  records the historical 82-second attribution, current source boundary, and
  executable five-cold/five-warm profile protocol. Its Python and PowerShell
  contract suites pass, but this is source-level infrastructure evidence: the
  current-source DX12/WPR/GPU/RenderDoc measurements remain pending.

### M8.5 Testing And Acceptance

- Run the declared managed Windows Cargo/WGSL, CPU/GPU artifact, and viewer
  checks only after the implementation set is complete.
- The CPU/GPU IEM parity command must run through the managed Cargo lane with
  Rust-harness `--nocapture`; the `coordinator-actions` template validates the
  coordinator itself and is not Zircon Rust/WGPU evidence.
- Run a second independent code review after all M8 implementation repairs.
- Completed implementation review: the independent structural pass found and
  forward-fixed runtime-cache payload selection, cold-process payload
  rehydration, and both cache-tier provenance write gates. It also confirmed
  the v3 header, selected-source-mip ABI, staging timing ownership, and the
  colored-mip WGPU regression coverage. This is not acceptance; M8.2--M8.4
  still require managed parity and current-machine evidence.
- Historical post-repair second review (`Critical 0 / Important 0`) confirmed
  CPU-artifact precedence, cold-process runtime-cache hydration,
  canonical source-SH9 retention across PMREM reconfiguration, and GPU-producer
  rejection on the direct asset path. It also confirmed request reservations
  release on build, submit, readback, disk-write, and device-poll failures, and
  that the no-project-cache path remains graph-free across consecutive frames.
  The subsequent staging-contract boundary review confirmed the same importer
  re-exports, importer-private constructors, and cache-reuse accounting after
  moving report declarations out of the orchestration module.
- The 2026-08-14 v4 checksum review found and forward-fixed one runtime-cache
  writeback length-contract expectation that omitted the checksum section.
  Scoped `rustfmt` and diff integrity passed after the repair. Repeat the
  managed Rust review before treating this static result as accepted evidence.
  This structural review is nonaccepting: managed Rust/WGPU parity, current DX12
  screenshot/RenderDoc evidence, timing, and energy measurements remain pending.
- Completed Ready-frame evidence integration and second review (`Critical 0 /
  Important 0`): schema v12 is emitted only after the successful Ready-frame
  CPU render and carries the current staging fields plus all twelve renderer
  PSO residency, creation, and async-admission queue-wait gauges. Default
  validation rejects pre-v12 sidecars; explicit legacy mode remains available
  for historical v2--v11 artifacts. This makes the next managed screenshot
  capable of separating IBL staging work from first-frame PSO work, but does
  not supply the required target-machine timing, image, WPR, or RenderDoc data.
- Completed direct GPU-stage evidence review and repair (`Critical 0 /
  Important 0`): the framework-neutral timestamp DTO is re-exported through
  `core`, `scene_renderer`, and `graphics`; a single-component relative report
  path does not try to create an empty directory; and every failure after early
  readback admission aborts/defer-resets that frame. Timestamp scopes now cover
  the direct encoder's realtime IBL, GPU-scene staging, scene, transfer,
  overlays, and optional UI work without claiming CPU preparation, queue-write,
  readback-copy, or presentation time. This structural review is
  nonaccepting: it still requires a managed DX12 measured report and the full
  M8.5 product-evidence set.
- Completed local startup profile contract repair: capture provenance binds the
  local viewer bytes, critical-source manifest, and one queried terminal passed
  coordinator source-validation ticket. The profile and offline summarizer
  reject stale source hashes, nonterminal tickets, legacy receipt fields,
  mismatched Ready PNG/sidecar/timing inputs, energy CSV substitution, and
  output-root escape. This remains diagnostic-only rather than build proof:
  a copied or timestamp-adjusted executable can satisfy a locally authored
  byte record. Tooling01 now owns
  `failure-2026-08-14-managed-viewer-artifact-receipt.md`, which requires a
  managed artifact-level receipt binding the job, target-relative executable,
  SHA-256, byte length, command identity, and source manifest before M8 can
  profile it as current source. Current-source DX12 capture remains required
  before acceptance.
- Implemented, validation pending: the profile's critical-source manifest now
  includes the IBL artifact header/blob/resolution/cache identity, source-cubemap artifact and
  rebuild/PMREM/projection/mipmap/IEM modules, source/derived bundle staging,
  the paired asset-derived writer, BRDF LUT and skybox, and the deferred
  EnvironmentOnly, generic environment, and standard-PBR WGSL entry points. A
  receipt can therefore not bind an old `.zribl` payload format, bundle
  publication behavior, warm-cache hydration implementation, or PBR
  environment-reflection shader while claiming current viewer evidence. The
  focused PowerShell contract passes all 10 cases; this improves provenance
  coverage but supplies neither a managed artifact receipt nor target-machine
  timings.
- Implemented, validation pending: GPU timing reports use the independent
  `zircon_shader_pbr_viewer_gpu_timing_evidence_v1` contract. The validator
  binds each report to a supplied Ready PNG by filename plus SHA-256 digest and
  requires that PNG to exist; the existing screenshot validator remains the
  pixel/provenance owner. It accepts only a measured report with one positive
  frame generation, no terminal-only or unknown fields, and all four HDRI
  direct scopes (`direct_gpu_scene_upload`,
  `direct_scene_content`, `direct_output_transfer`, and `direct_overlays`);
  The writer rejects any report path that aliases the Ready PNG or its sidecar
  before it can overwrite that evidence. Zero upload time is valid when that
  mandatory diagnostic scope has no encoder
  work. A timed-out or
  unavailable report is retained as diagnostics but cannot satisfy M8.4.
- Accept only fresh current-source data: phase logs, WPR attribution, timestamp
  report, nonblank DX12 screenshot with sidecar, and successful RenderDoc
  replay. Historical captures remain comparison baselines.

## Known Measurements And Gaps

Historical data is retained solely to prevent misdiagnosis:

- The 2026-07-11 viewer samples reached 80.875 CPU seconds five seconds after
  post-load monitoring began and 84.031 CPU seconds at ten seconds.
- Later DX12 phase accounting recorded roughly 91.74 seconds in deferred scene
  resources and about 1.08 seconds in IBL cache restore before the current
  environment-only reductions.
- The old 8x8 procedural realtime IBL timestamp run recorded 4.990 ms for a
  full 18-dispatch update and 0.343 ms average slices. This is not comparable
  to a current 256-face HDRI import or DX12 first-frame PSO creation.

The 2026-08-10 structural performance review establishes these implementation
gates before another visual acceptance run:

| Owner | Current cost or defect | Required convergence |
| --- | --- | --- |
| Runtime hydration | A stable source-256 environment formerly churned at least 493 MiB/s at 60 submitted frames: 1,097,976 artifact bytes read, about 2,170,992 decoded bytes allocated, and 5,354,496 aligned upload bytes recreated per frame. | Implemented four-entry request memo and 60-hit Arc/prepared-upload regression; managed validation remains pending. |
| Runtime writeback | Cache miss formerly read about 1.10 MiB back through an extra submission and `wait_indefinitely` before frame return, and could recompile the same graph before persistence completed. | Implemented main-submit copy plus four-entry cross-frame `Poll` queue, request-keyed pending suppression, and RAII release across build/submit/device-poll/map/write failures; no project-cache path omits unused runtime bake graphs. Managed GPU validation remains pending. |
| Default diffuse | CPU direct IEM enumerates 37,748,736 source/output candidate iterations; only positive `n dot l` candidates read a source texel. The GPU estimator uses 393,216 samples but is not artifact-equivalent. | Implemented PMREM+SH9 default; CPU IEM now requires explicit `environment_ibl_irradiance_cube=true`, with parity still required for promotion. |
| PMREM | The framework `128x8` Normal baseline performs 1,214,208 value evaluations and up to roughly 9.7 million trilinear source-texel taps. The viewer currently overrides that layout to its resolved source size; see the viewer-default audit below. | Implemented central `IblBakeRecipe` sample budgets and roughness mapping; managed timing must profile default-layout policy and task scheduling before changing the algorithm. |
| BRDF LUT | Renderer setup formerly performed `128 * 128 * 1024 = 16,777,216` CPU integration iterations. Unreal `SystemTextures` uses `128 * 32 * 128 = 524,288`. | Implemented the reference work scale, a 32x reduction; same-integrator sparse error is mean `0.00238490`, max `0.01624132`; managed startup timing and image validation remain pending. |
| Source staging | `.zcube` identity formerly changed with PMREM/IEM recipe, duplicating source pyramids of about 1, 4, 16, or 64 MiB at face sizes 128, 256, 512, or 1024. | Implemented independent source-stage version/key, decoded-source reuse on derived-only miss, and `.zribl`-only writeback without rewriting `.zcube`; managed validation remains pending. |
| Source hashing | Importer scanned the full HDR/container bytes twice for revision and layout hash. | Implemented as one BLAKE3 source update with unchanged digests; focused managed test remains pending. |
| Viewer work root | Project staging, default IBL cache, and implicit RenderDoc template previously lacked one caller-owned root. | Implemented `--work-dir` ownership for all three; all managed viewer/capture runs still must pass an E: root. |

The BRDF comparison uses
`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SystemTextures.cpp`.
Unreal also builds its preintegrated GF texture on the CPU during system-texture
initialization; the useful reference is its rectangular domain and sample
budget, not an assumed compute or embedded-LUT path.

The 2026-08-10 IEM structure audit establishes the next CPU optimization gate:
the direct cosine IEM enumerates every selected source/output texel pair for a
32x32 output. It evaluates `n dot l` first and loads the source texel only for
a positive hemisphere candidate. The selected source mip is capped at 32x32,
so the deterministic worst current candidate shape is
`6 * 32^2 * 6 * 32^2 = 37,748,736` iterations per bake; actual source reads
are lower and direction-dependent. This is intentional canonical-quality work,
not the historical DX12 PSO bottleneck. cmft's `imageIrradianceFilterSh` builds its IEM through spherical
harmonic reconstruction, an `O(source samples + output texels * coefficients)`
approximation rather than Zircon's direct convolution. Zircon already carries
SH9, but it must not substitute that approximation for the persisted direct
IEM until M8.2 records CPU/GPU IEM RGB and seam error. Ready-frame evidence
schema v12 retains the historical `source_sample_visits` key for the direct IEM
candidate-iteration count alongside its wall time; it is zero when IEM was not
built or the staged artifact was reused. A future actual-read profiler must be
kept separate so it does not add atomics to the canonical bake loop. First
collect that v12 throughput on the target machine; only then evaluate an
output-equivalent source-sample table or an explicitly versioned, measured SH
approximation.

### 2026-08-13 Direct IEM Task-Granularity Decision

The next non-semantic CPU IEM optimization is explicitly limited to task
granularity. `source_irradiance_cubemap.rs` formerly gave the caller-owned
executor one task for each complete output face. That capped the fixed 32x32
IEM at six tasks even though the 37,748,736 direct candidate iterations
dominate the optional IEM phase. The framework contract already permits an
arbitrary mutable slice of independent output work; it does not require
framework code to construct a global pool.

- Keep the exact source-mip selection, cubemap direction order, solid-angle
  weighting, per-output accumulation order, final storage layout, artifact
  recipe identity, and serial path unchanged.
- Represent one output task as a face plus a contiguous four-row range. The
  fixed 32x32 output then submits 48 independent tasks through one
  caller-owned executor call, rather than six full-face tasks. Each texel still
  performs the same direct source scan, so its result must be bit-identical to
  the serial path; this is scheduling, not a filter change.
- Add a framework regression that compares the complete parallel and serial
  cubes and verifies the one dispatch / 48 work-item shape. The existing
  importer counter will then expose the additional executor work in Ready v12
  without changing the distinct `source_sample_visits` candidate-iteration
  metric.
- `dev/cmft/src/cmft/cubemapfilter.cpp` is the algorithm contrast: its
  `imageIrradianceFilterSh` reconstructs irradiance from SH and is therefore
  asymptotically cheaper but approximate. It remains an M8.2 error-gated,
  versioned alternative, not this optimization. Its radiance path separately
  schedules `(mip, face)` work. `dev/cmftStudio/src/backgroundjobs.cpp` runs
  both filters off the UI thread. Unreal's ShaderPipelineCache similarly uses
  bounded precompile batches rather than forcing all work into one startup
  batch. These references support independent work scheduling, not replacing
  Zircon's CPU-importer ownership or canonical direct IEM.
- Implemented: the caller-owned parallel path now submits the planned 48
  output-row tiles, while the serial and parallel results are compared as full
  cubes. Rustfmt parsing and scoped diff integrity passed after the coordinator
  applied the source patch. Managed target-machine CPU/WPR measurement remains
  pending; do not infer a wall-time speedup from task count alone, especially
  on low-core hosts.

### 2026-08-14 Structural Performance Review

- Viewer preflight reads source bytes and parses the HDR dimensions, then the
  importer derives the exact BLAKE3-backed source identity before selecting a
  persisted bundle. A current bundle does not invoke `decode_viewer_hdri`:
  `.zcube` and `.zribl` rehydrate the environment and the Viewer reads its
  small exposure sidecar. Header dimensions alone cannot establish cache
  freshness because source bytes are part of the artifact identity.
- If that exposure sidecar is missing after a complete bundle has already been
  validated and hydrated, the Viewer now decodes the HDR only to reconstruct
  exposure, preserves the first restored environment, and writes the sidecar
  on a best-effort basis. It does not run the `Reused` staging probe or hydrate
  the same bundle a second time. This is a lifecycle repair, not a PMREM or
  source-identity change.
- cmft splits PMREM by mip and face, while cmftStudio runs its filter as a
  background job and publishes a completed result before recreating GPU
  buffers. Zircon retains its headless, atomic asset-derived producer instead
  of importing a UI-thread model into the asset pipeline.
- A future warm-path memory optimization must split exact streaming source
  identity from the decode context, so a cache hit can discard source bytes
  after hashing rather than retain them in `AssetImportContext`. Filesystem
  metadata or a header-only shortcut may not replace the exact source identity,
  PMREM/IEM recipe semantics, or corrupt-artifact recovery. It remains
  measurement-gated by the declared five-cold/five-warm profile.

### 2026-08-14 PMREM Dispatch-Shape Audit

- Zircon currently invokes the caller executor once for each PMREM mip and
  submits its six complete face outputs in that call. A normal `128x8` PMREM
  therefore has 48 submitted face work items, but eight ordered executor
  batches. `dev/cmft/src/cmft/cubemapfilter.cpp` instead builds one shared
  `mipCount * 6` task list. This is a scheduling difference, not evidence that
  Zircon's GGX result or historical 82-second startup needs a new filter.
- The former Ready staging output recorded only aggregate
  `parallel_executor_work_items`. It now also records the submitted chunks for
  equirect projection, source-mip construction, PMREM, and optional IEM. The
  aggregate is derived from those four fields and the profile summarizer
  rejects a sidecar whose total does not equal their sum. These are dispatch
  shapes, not worker-utilization, overlap, or wall-time measurements; they
  therefore cannot assign a cost to mip barriers or worker idleness without
  `pmrem_build`, HDR decode, WPR worker utilization, and the five cold/warm
  distributions.
- Implemented and statically verified: the phase counters are carried from
  `source_cubemap/{projection,rebuild}.rs` through
  `environment_ibl/source_staging/{mod,output}.rs` into the Viewer Ready
  sidecar and the strict profile summarizer. The aggregate remains derived
  from the four phases; Python contract tests pass 17 cases and the PowerShell
  profiler Pester suite passes 10 cases. Rust `rustfmt --check` and scoped
  `git diff --check` also pass. This is observability infrastructure only, so
  it does not close the required managed Viewer capture or authorize a default
  layout change.
- Second independent review completed: it found the former parallel-chain
  dispatch-shape test still expected projection plus PMREM only after source
  mip construction became caller-executor work. The test now asserts
  `2 * mip_count` for projection, `mip_count - 1` source-mip calls, and
  `mip_count` PMREM calls. No formula, artifact, or scheduling policy issue
  remained after that forward fix.
- Only if PMREM is material and the capture shows spare workers behind the
  six-face batches may M8 split construction into a shared `(mip, face)` queue.
  A further row-tile shape is permitted only for measured high-core imbalance.
  Both changes must preserve source mip selection, Hammersley order per output
  texel, GGX/PDF LOD, per-texel accumulation order, artifact identity, and
  final face-major layout exactly; the serial/parallel full-cube regression is
  the prerequisite. No PMREM formula, sample budget, or cache version changes
  are authorized by this audit alone.

### 2026-08-14 Viewer Default-PMREM Architecture Audit

- The M8 `128x8` Normal work estimate is a valid independent-PMREM baseline,
  but it is not the current viewer default. The checked-in
  `polyhaven_lakes_2k.hdr` header is `2048x1024`; viewer preflight maps that
  height to source face size `512`, and `resolved_pmrem_face_size(None,
  source_face_size)` currently returns that same `512`. The import settings
  therefore request a `512x10` PMREM unless the user explicitly passes
  `--pmrem-face-size`.
- Under the current immutable recipe, `512x10` has an 18,417,408
  source-cubemap trilinear-call bound, of which 16,844,544 occur in
  importance/cosine sample-loop iterations. This is 15.17 times the 1,214,208
  `128x8` call bound and can issue up to 147,339,264 trilinear source-texel
  taps. These are static bounds, not a claim that they explain the historical
  82-second run or any current-machine wall time.
- Before a default-policy change, capture five isolated cold runs and five
  warm runs of the same HDRI with the current default and an explicit
  `--pmrem-face-size 128`, retaining source face size, scene, camera, driver,
  WPR, Ready timing sidecar, GPU timing report, and RenderDoc capture policy.
  Compare the resulting screenshots and record `pmrem_build`, CPU worker
  utilization, cache bytes, and wall-time distributions. Only a visual match
  plus a material measured reduction authorizes making independent `128x8`
  PMREM the viewer default; the explicit source-sized override remains a
  quality option and cache identity remains layout-specific.
- The current profiling contract now exposes PMREM chunk submissions separately
  from projection, source-mip, and IEM submissions. This closes the previous
  observability gap but does not prove PMREM is material, that Rayon has spare
  workers, or that a flat `(mip, face)` queue is faster. The required run
  matrix remains the decision gate for any scheduling or default-policy change.
- A flat `(mip, face)` queue is a secondary candidate. The current face-major
  artifact layout can provide 48 non-overlapping output slices without changing
  samples or final bytes, but the existing executor invokes Rayon once per mip
  and uses at most six face tasks per batch. Implement it only after the
  measurement above shows PMREM material and spare compute workers; row tiles
  require separate evidence because they increase scheduling overhead and may
  be memory-bandwidth-bound.
- Reference-engine routing confirms the priority. cmft's offline CPU path
  builds one `mipCount * 6` task list, while cmftStudio runs that work through a
  background job instead of its UI path. Unreal's
  `ReflectionEnvironmentCapture.cpp` filters through named RDG mip/face passes
  and exposes runtime capture timeslice and budget controls. Zircon's viewer
  instead blocks its first Ready frame on a CPU import. For the MVP, preserve
  the explicit high-resolution option but first decouple the default result
  layout and rely on the existing persistent cache; progressive/background or
  GPU bake is a later lifecycle design, not a disguised PMREM math change.

### 2026-08-14 Viewer Warm-Cache Implementation

- Implemented, not accepted: the importer derives its canonical request from HDRI
  header dimensions and source bytes before pixel decoding, then restores only
  a semantically applicable current `.zcube` plus asset-derived `.zribl`
  environment. An `ApplyAssetDerived` restore failure deletes only the invalid
  derived artifact and forwards to the established source-only rebuild path.
- The viewer retains cold-path exposure exactly through a versioned sidecar
  named beside the source `.zcube`. When a current `.zcube` and `.zribl` have
  already restored the environment, a missing or malformed sidecar decodes
  only to reconstruct exposure and preserves that first hydrated environment;
  it does not stage or hydrate the bundle again. Pixel decode is skipped only
  when both the restored environment and exposure sidecar validate.
- The viewer reports header probe plus actual source-pixel decode as
  `hdr_decode`, and subtracts the latter from `ibl_restore`, so cold-path HDR
  decoding cannot be attributed to both phases. The follow-up timing review
  found and forward-fixed one P2 accounting inconsistency: the cold path now
  starts artifact hydration after the Viewer-local exposure-sidecar write and
  reports `ibl_total_elapsed` as `staging_elapsed + artifact_hydration_elapsed`.
  Restored bundles use their restore duration directly, so Viewer-local
  sidecar I/O is excluded in both paths.
- Earlier focused static checks passed for the landed viewer and runtime
  integration files. The P2 timing-boundary source contract, `rustfmt --check`,
  and scoped diff integrity check pass; managed Rust/WGPU validation remains
  required before renewing any source-wide claim. The direct
  recovery regression creates a real cache directory, removes a bad `.zribl`,
  and preserves its `.zcube` for the derived-only rebuild.
- M8 implementation is complete pending managed validation: managed Rust/WGPU
  compilation, target-machine timings, screenshot verification, and RenderDoc
  replay remain pending.

### 2026-08-16 Static Closure Record

The completed implementation received an independent second review across the
durable source/derived publisher, source-miss publication barrier, viewer
timing phase boundary, provenance contract, and multiview cold-start guard.
Those reviews reported no Critical or Important issue. The focused
`rustfmt --check` and scoped diff-integrity checks previously recorded for the
owned implementation remain the static evidence for this milestone; no Cargo,
WGPU product run, screenshot, RenderDoc replay, timing matrix, or power
measurement was executed in this record. This status deliberately records
implementation closure without promoting it to visual or performance
acceptance.

The coordinator refused the managed compile request before it created a Cargo
job because unrelated D/E/F artifacts were not registered. This is not a Cargo
pass or failure and does not alter the static verification above.

WPR (`C:\Windows\System32\wpr.exe`), WPA, and RenderDoc are installed, but no
current-source viewer executable exists in the approved `E:` output roots. A
new profile must wait for the current managed build rather than profiling a
historical executable and treating it as acceptance. Energy or watts data is
also absent; CPU sample time and GPU timestamps do not establish power parity
with Unreal or Unity. M8 will report power only if the capture platform exposes
an energy counter with an explicit unit and sampling interval.

## References Reviewed

- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentDiffuseIrradiance.cpp`
- `dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/SkyManager.cs`
- `dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/ReflectionProbeTextureCache.cs`
- `dev/cmft/include/cmft/cubemapfilter.h`
- `dev/cmftStudio/src/cmftstudio.cpp`

The deliberate Zircon divergence is to preserve a CPU canonical offline path
for the MVP instead of requiring a graphics device during source asset import.
This is compatible with the repository's runtime/framework boundary and keeps
GPU bake as a renderer capability rather than an importer dependency.
