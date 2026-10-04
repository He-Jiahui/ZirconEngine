---
related_code:
  - zircon_runtime/src/graphics/material
  - zircon_runtime/src/graphics/pipeline
  - zircon_runtime/src/graphics/shader
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/backend
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompilerJobCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/ShaderPipelineCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/StreamingManagerTexture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/RenderAssetUpdate.cpp
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
tests:
  - 136 of 136 current Rust files reconciled and reviewed
  - 29960 physical lines and 284 inline tests
  - path plus physical-line-count plus per-file SHA-256 manifest fingerprint ea44d51aa946bfceabd0197e77b6532a1abb7ae4c0e1d84f6045858ec365860b
  - current managed Cargo, product WPR/xperf, GPU timestamps, RenderDoc and energy remain blocked by the non-runnable editor baseline
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Renderer/material/shader/streaming current architecture review (2026-08-15)

## Scope freeze and review method

This review freezes the current renderer resource vertical at **136/136 Rust files, 29,960 physical
lines and 284 inline tests**. The manifest fingerprint is
`ea44d51aa946bfceabd0197e77b6532a1abb7ae4c0e1d84f6045858ec365860b`; it is SHA-256 over sorted
`path|physical-lines|file-sha256` rows joined with LF.

| Current module | Files | Physical lines | Tests | Reconciliation |
|---|---:|---:|---:|---|
| `graphics/material` | 5 | 891 | 16 | prior 5/5 review reconciled; changed include resolver reread |
| `graphics/pipeline` | 49 | 8,778 | 66 | prior 46/46 review reconciled; current compile/cache/authoring deltas reread |
| `graphics/shader` | 32 | 10,824 | 129 | prior 23/23 review reconciled; new split tests and all current template/cache deltas reread |
| `scene/resources/resource_streamer` | 26 | 6,336 | 44 | prior 22-file review reconciled; new model dependency geometry path reread |
| `graphics/backend` | 24 | 3,131 | 29 | current backend follow-through reconciled with the 76-file RHI/WGPU review |

The July material, pipeline, shader and scene-resource reports remain the evidence for unchanged
files. This pass regenerated the current manifest, reread every modified or new Rust file, followed
all changed callers, and reconciled the unchanged files to those reports. Source-shape tests were
read as implementation locks, not treated as proof that the locked architecture is desirable.

This is static review completion only. The approved-root separator defect in
`tools/build-editor.ps1:130` still rejects valid D/E/F build roots before Cargo. Therefore no current
product executable exists for WPR/xperf, RenderDoc, GPU timestamps or energy capture. No timing or
power comparison to Unreal is claimed, and this module must remain out of `review.md`.

## Architecture verdict

The dominant problem is not a slow loop in isolation. Shader source assembly, material include
resolution, Naga/WGPU validation, runtime pipeline creation, disk persistence and render-resource
preparation are separate authorities with different keys, queues and lifetime rules. They repeatedly
materialize owned strings and descriptors, then either block the caller or publish through local
caches. Increasing worker counts or adding another cache would make the split more expensive and
less deterministic.

The required hard cut is one dependency chain:

`AssetCatalogGeneration -> ShaderSourceArtifactGeneration -> ShaderPermutationGeneration ->
PipelineSchemaGeneration -> RhiPipelineGeneration -> PreparedRenderAssetGeneration`.

Each generation must publish immutable, content-addressed artifacts and affected-only invalidation.
Frame/editor code may poll a typed ticket and use last-good state; it must not parse, compile, read or
write cache files, walk asset dependencies, create GPU objects, or wait for a private worker.

## P0/P1 current-source findings

### 1. Shader and PSO compilation have three incompatible schedulers and caches

`graphics/pipeline/async_compile.rs:66-261` creates a private OS thread and bounded channel for every
`PipelineAsyncCompiler`; `finish_pending` and `finish_pending_through` block, while `Drop` joins with
no deadline. The mesh pipeline cache constructs two instances, one for source validation and one for
base pipeline compilation. They bypass Runtime11's shared worker budget, priority, cancellation,
affinity and shutdown contracts.

The prewarm path is the opposite extreme. `variant_cache/prewarm/worker.rs:80-200` executes one
serial variant loop. Its public budget explicitly rejects any `max_in_flight_variants != 1` in
`core/framework/render/shader/variant_prewarm/budget.rs:6-49`. Naga source validation is shared per
source, but WGPU module/pipeline validation and cache writes remain serial per variant.

The third owner, `pipeline_cache_gate.rs:14-124`, is a Vulkan-only WGPU driver cache. The Windows MVP
DX12 path is explicitly unsupported, startup may synchronously read and digest up to 64 MiB, and
`RuntimePipelineCache::drop` synchronously obtains and persists the driver blob. Its identity and
lifecycle are unrelated to the shader prewarm cache and the mesh compiler tickets.

### 2. The disk variant key is weaker than the source artifact identity

`ShaderVariantPrewarmSource` correctly hashes source label, full WGSL, include hashes, template
revision, Naga version and WGPU version into one source ID. However,
`variant_cache/disk.rs:23-34,260-270` builds the disk key only from the canonical variant string and
include hashes. Lookup at `:162-188` validates schema, hash and canonical string but accepts no
expected source ID, template revision, Naga version or WGPU version. Those versions are written only
as metadata at `:130-158`.

Consequently the cache API cannot prove that a hit belongs to the current compiler/backend source
artifact. Material revision or include changes may incidentally change some keys, but Naga/WGPU
version changes do not. This is both a stale-artifact correctness risk and evidence that cache
identity is owned at the wrong layer.

### 3. Shader template construction copies and reparses full source per permutation

`ShaderTemplateInclude::new` at `template/module_registry.rs:142-153` allocates token/source/owner,
hashes the full source, extracts include paths, strips directives and hashes the derived module.
Builtin include factories repeat this work for each assembly. Module resolution then clones tokens
and full include records into maps and resolved output.

`template/material_surface.rs:363-431` specializes one large WGSL string through eleven sequential
whole-string `replace` calls and a final format. Forward, deferred and TAA assembly each rebuild a
registry, concatenate full WGSL, rename entry points by string scanning, and carry owned source,
token, hash and line-segment vectors. This makes compile cost proportional to full source bytes for
each permutation even when most modules are unchanged.

The July source-table change removed per-variant WGSL ownership from the manifest, which is useful,
but it did not create a persistent parsed-module DAG or a shared compiler artifact generation.

### 4. Render-pipeline compilation is runtime graph authoring, not schema publication

`render_pipeline_asset/compile.rs:42-190` first materializes all feature descriptors for validation,
then clones enabled features and materializes option-filtered descriptors again. Validation calls
`pipeline_graph_resources` and discards the result; graph authoring rebuilds the resource view.
Feature names/descriptors, stages and diagnostics are repeatedly cloned.

`pass_authoring.rs:207-264` discovers unique producers and readers through nested pass/write/read
scans and BTreeSet adjacency, then the normal authoring loop adds `previous -> pass` for every pass.
The former is approximately `O(P^2 * A)` for P passes and A accesses; the latter destroys ready
width even when data dependencies are independent. PERF-MVP-633 already owns the RDG hard cut.

`CompiledGraphCacheKey` clones the full `RenderPipelineCompileOptions` and includes exact view/render
dimensions. A miss compiles synchronously before insertion into a 16-entry cache; eviction scans all
entries and clones the selected key. Exact extent in the topology key turns resize/dynamic resolution
into graph recompilation instead of resource rematerialization.

`compile_with_asset_context.rs` compiles the full graph first, then separately loads shader/material
assets and linearly scans entry-point/property/texture contracts to build owned diagnostics. This
diagnostic path has no generation gate and is mixed with production compile ownership.

### 5. Resource preparation polls the whole visible set instead of consuming dirty generations

`resource_streamer_ensure_scene_resources.rs:40-103` allocates per-frame HashMap/HashSet tables,
walks visible meshes/sprites/UI/resources, and synchronously calls `ensure_mesh`, `ensure_model`,
`ensure_material` and `ensure_texture` once per unique ID. A miss can load/clone/decode/hash, validate
material/shader dependencies, calculate bounds/SDF/deformation, and create buffers, textures,
samplers and bind groups on the render submission path.

The new model dependency fix preserves correctness but exposes the architectural cost. Stable
`ensure_model` calls `model_dependencies_are_current`; `model_geometry_resolution.rs:67-75` walks
every dependency and `:103-126` probes the registry by locator. A miss clones primitives, loads
external mesh assets, recomputes bounds/deformation/SDF and hashes a composite revision before GPU
creation. Stable cost is now `O(visible instances + unique resources + model dependency probes)`
rather than work proportional to changed assets.

The new morph geometry seed computes a hash over all weights when called. It currently has no product
caller beyond the collector accessor, so it is not reported as a measured frame bottleneck; before
activation it must be carried by the extract/shape generation rather than recomputed per query.

### 6. Plugin material include resolution scans and normalizes the shader catalog repeatedly

`material/shading_models/include_sources.rs:31-80` resolves forward/GBuffer/deferred tokens for each
plugin descriptor. Each token filters all ready shader records; each candidate normalizes token and
locator using trim, slash replacement, lowercase and suffix formatting. A hit then synchronously
loads the shader asset and copies runtime WGSL into the source set.

At D plugin descriptors, T include tokens and R ready shader records, construction is `O(D*T*R)`
record visits plus repeated string allocation and source cloning. The correct owner is a catalog-
generation `normalized-token -> ShaderSourceArtifactId` index, not another material-local cache.

### 7. Synchronous cache I/O and dead public DTOs extend the wrong ownership model

Shader disk lookup synchronously reads JSON and compressed WGSL, decodes zstd and allocates a String;
write synchronously compresses, pretty-serializes and atomically writes two files. Corrupt lookup
deletes files in the caller path. Runtime pipeline persistence performs I/O from `Drop`. None of
these operations has bytes/count/age/deadline/cancellation admission.

`graphics/shader/shader_assets.rs` declares and publicly re-exports `ShaderProgramAsset`,
`ShaderGraphAsset`, `MaterialGraphAsset` and another `ShaderVariantKey`; repository search finds no
consumer beyond the re-exports. These parallel authoring DTOs should be removed during the hard cut,
after the canonical asset/artifact types are selected. Keeping unused public models makes ownership
convergence harder and encourages a second shader system.

## Reference-engine evidence and transferable principles

### Unreal shader compilation and job cache

- `ShaderCompilerJobCache.cpp:38-100,2117-2152` enforces absolute/relative memory budgets and
  overflow reduction; `:273-336,1385-1469,1614-1646` keys jobs by compiler input hash, tracks one
  in-flight job and attaches duplicate waiters; `:1823-1824` can run per-shader DDC queries async.
- `ShaderCompiler.cpp:1274-1277,1438-1585` budgets worker counts, exposes outstanding jobs and
  separates job submission from async result processing. Worker processes are a deliberate compiler
  isolation domain, not a private thread created by every material or pipeline cache.
- `ShaderPipelineCache.cpp:30-112` records waiting/active/precompile memory/time counters and supports
  background/fast/precompile count and time budgets. `:434-497` owns the PSO precompile task and
  queue statistics. `:137-140` explicitly documents a disabled autosave path whose broad lock plus
  async work can deadlock low-core systems, supporting Zircon's ban on cache I/O from broad locks or
  destructors.

The transferable design is content-keyed single-flight work, explicit memory/time budgets, async
result publication and one cache authority. Zircon need not copy Unreal's process model or APIs.

### Unreal render-asset streaming

- `StreamingManagerTexture.cpp:392-447,786-806` applies explicit temporary-memory budgets while
  preparing async streaming work. `:1647-1763` limits texture requests per frame, and
  `:1975-2157` stages background calculation and result application rather than synchronously
  preparing every visible resource.
- `RenderAssetUpdate.cpp:198-252,278-376` advances updates through thread-aware task states and
  schedules game/render/async continuations. It attempts to avoid blocking game/render threads on
  low-priority async work.

The transferable design is event/state-driven updates with bounded calculation, staging and apply
phases. Zircon should not copy Unreal's texture-specific state machine into every resource type.

### Bevy secondary evidence

`bevy_render/render_resource/pipeline_cache.rs:45-76,213-237,669-745,838-846` keeps queued/creating/
ready/error states behind stable `CachedPipelineId`, tracks waiting pipelines, polls async tasks and
uses the shared async compute pool where supported. It is useful secondary evidence that frame code
should poll stable IDs rather than block on local compiler workers.

## Required hard-cut target architecture

### A. `ShaderArtifactGeneration`

Runtime04/Render08 must publish one immutable generation containing:

1. interned module/token IDs and a normalized plugin-token index;
2. one parsed source artifact per content ID: original bytes, stripped source, include edges,
   diagnostic line map and hash inputs;
3. one material/template IR whose specialization patches typed slots rather than repeatedly replacing
   full strings;
4. a canonical permutation key covering source artifact ID, definitions, pass, geometry, shading
   model, quality, layout/schema, compiler/Naga version, WGPU/backend version and platform/device
   compatibility identity;
5. typed success/error artifacts and affected reverse-dependency ranges.

Stable generation work for scan/normalize/hash/parse/assembly must be zero. A source change reparses
once and invalidates only its reverse closure.

### B. shared compile and artifact service

Runtime11 must execute preprocess, Naga validation, backend module/PSO creation, compression/I/O and
publish as a dependency graph with keyed single-flight admission. The service needs global and
domain budgets for jobs, source bytes, resident bytes, I/O bytes, age, priority and cancellation.
Main/render/editor threads only submit or poll tickets; last-good artifacts remain active until an
atomic generation publish.

Disk cache lookup and persistence are explicit jobs. `Drop` performs no I/O or unbounded join. Cache
keys contain the canonical artifact identity; metadata is diagnostic, not the only version guard.
DX12/Vulkan/other backend support is selected by RHI capability and measured independently.

### C. `PipelineSchemaGeneration` and RHI PSO publication

Plan02 M3/Render01 must compile dense pass/resource/executor slots once per schema generation. Frame
instances carry dynamic extents, roots, versioned writes and per-view data without rebuilding schema.
The `previous -> pass` chain, pass-name dispatch and String resource identity are deleted.

Render08 lowers shader permutation artifacts into one `RhiPipelineGeneration`; GPU creation runs on
the RHI affinity executor and publishes a typed PSO ticket. There is no second mesh-local compiler or
direct WGPU pipeline owner. Failed/late PSOs use an explicit last-good/fallback/error policy without
waiting in submission.

### D. `PreparedRenderAssetGeneration`

Runtime04 publishes one asset-event/revision snapshot and dependency DAG. Dirty resource IDs schedule
CPU load/decode/bounds/SDF/material/shader work once through shared jobs. Render02/08/13 consume ready
artifacts; RHI alone creates GPU resources and applies uploads under byte/object/time budgets.

The render frame consumes an immutable `PreparedSceneResourceSet`. Stable frames perform zero asset
manager loads, locator scans, dependency polls, source validation, GPU creation and uploads. A 1%
change performs work near the dirty dependency closure. Missing/failed work retains last-good or a
device-generation shared fallback.

### E. plugin and editor integration

Plugins01 contributes shader module bytes/metadata through the catalog generation and stable VM ABI;
no Rust shader object crosses a dynamic library boundary. Reload revokes affected artifact IDs,
quiesces tickets and publishes one new generation.

Editor09 consumes the same parsed source/DAG/diagnostic artifacts for IDE environment and preview.
Typing/reload storms coalesce by source generation; stale preview jobs cancel; the UI thread never
runs Naga, WGPU creation, full tree scans or cache I/O.

## Deletion gates

The replacing milestones are incomplete until they delete, in the same hard cut:

- product `PipelineAsyncCompiler` private threads, blocking finish APIs and unbounded join;
- serial prewarm execution contract and direct `ShaderVariantCacheDisk` filesystem ownership;
- `RuntimePipelineCache` destructor persistence and cache identity outside RHI generation;
- per-assembly builtin source parse/hash/clone and full-string replacement specialization;
- per-frame resource `ensure_*` polling and direct asset-manager/GPU creation from submission;
- material-local ready-shader catalog scans and normalized String matching;
- unconsumed parallel shader DTOs and source-shape tests that require the retired implementation.

Behavior, artifact identity, diagnostics, fallback and failure tests replace source-shape assertions.

## Dependency-ordered implementation plan

| Order | Owner/milestone | Required output | Entry dependency |
|---|---|---|---|
| 0 | Performance M0/tool owner | current-source editor bundle on E: and product fingerprint | approved-root separator fix |
| 1 | Plan02 M1 / Runtime11 | shared affinity/task executor, keyed single-flight, budgets, cancellation, deadline shutdown | TaskGraph hard cut |
| 2 | Runtime04 / Plugins01 | asset catalog generation, shader token index, dependency DAG, dirty event ranges | module/plugin catalog generation |
| 3 | Render08 | parsed shader artifacts, canonical permutation identity, async compile tickets, explicit disk artifact service | orders 1-2 |
| 4 | Plan02 M3 / Render01 / RHI | dense pipeline schema, frame graph instance, one RHI PSO generation and publish | RDG and RHI packet hard cuts |
| 5 | Render02/13 | prepared render-asset generation and bounded upload apply | orders 1-4 |
| 6 | Editor09 | IDE/preview consumption of shared artifacts with coalescing/cancellation | order 3 |
| 7 | Render17 / Performance | current Cargo plus WPR/xperf/GPU/RenderDoc/energy acceptance | runnable current product |

No production source change was made in this review. The affected source files are foreign-dirty and
owned by active render/framework sessions, and local fixes would preserve the wrong authorities.

## Complexity and quantitative acceptance

| Dimension | Required matrix | Hard acceptance |
|---|---|---|
| shader source graph | 1/100/10k modules, 4 KiB/1 MiB source, depth 1/100/1k, stable/1% change | stable scan/hash/parse/assembly=0; changed parse=1/source; work near changed reverse closure; cycle/diagnostic mapping preserved |
| permutations | 1/1k/100k variants, shared-source ratios 0/50/99%, workers 1/2/8/64 | one canonical source payload; duplicate compile=0; queues/bytes/age/RSS bounded; main/render/UI compile wait=0 |
| pipeline schema | 1/32/256/1k passes, 1/8/64 accesses, 1/2/8 views, stable/resize/reload | schema finalize near `O(P+A+E)`; stable descriptor/String rebuild=0; resize topology rebuild=0; independent ready width >1 |
| material/plugin | 1/100/1k descriptors, 1/100/10k shader records, reload 0/1/100% | stable catalog scan/normalize/load/source clone=0; index build=1/catalog generation; reload near affected entries |
| render assets | instances 1/1k/100k, unique assets 1/10/1k/10k, 4 KiB/4 MiB/256 MiB, stable/1% change | stable loads/decode/hash/dependency probes/GPU create/upload=0; changed work near dirty DAG; apply budget never exceeded |
| cache/lifecycle | cold/warm/corrupt/version/driver/device loss; 1 MiB/64 MiB/1 GiB artifacts | stale hit=0; caller/Drop I/O=0; atomic publish; bounded eviction; cancellation/shutdown terminate by deadline |

Dynamic runs must share one current-source product fingerprint and run ID:

- WPR/xperf: main/render/RHI/compiler workers, ReadyThread/waits/context switches, queue depth/age,
  file I/O, allocations, RSS and idle wakeups;
- CPU counters/scopes: source scans/hash bytes/Naga parses, descriptor materializations, cache hit/miss,
  asset probes/load/decode and GPU object/upload counts;
- GPU timestamps: warmup plus p50/p95/p99 for submission and affected passes;
- RenderDoc CLI: PSO creation/reuse, pass/copy/submit sequence, resource create/upload and present;
- energy: F0 startup, F2 300-frame stable and 1% reload, F4 30-second idle plus preview storm, each at
  least three runs.

Only after those gates pass may the project compare wall time, frame time and power to mature-engine
experience. Source complexity alone is not a substitute for measured parity.

## Status and routing

Static review is complete for this frozen 136-file manifest. Dynamic acceptance is blocked by the
non-runnable product baseline, so `pending.md` must retain the module and `review.md` must remain
unchanged. Protected-plan owner merges are specified in
`2026-08-15-renderer-material-shader-streaming-protected-plan-routing.md`.
