---
title: Plugin Rendering Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/rendering
status: static_complete_product_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Shadows/ScreenSpaceShadows.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFog.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Materials/HLSLMaterialTranslator.cpp
---

# Plugin Rendering Current Source Performance Review

## 1. Coverage

The current Rust surface is **114/114 files**, **7,620 physical / 6,891 non-empty lines**, **274,461 bytes**, **62 test markers** and **3 ignored tests**. Its workspace-relative `path + LF + decoded text + LF` SHA-256 after the owned formatting repair is `a1db83f88527306a2fb9fb5718f10e393843cd481e652777efb354e542ccab3c`.

The baseline package was clean. The only production-source change from this review is rustfmt normalization in four Volumetric Fog files; it has no behavioral effect. Every root Runtime/Editor/Dist file and every Rust file under all 15 feature bundles was read. Generated manifest/Cargo surfaces, current registration paths, Runtime executor owners, shader-cache roots, ignored product tests and the relevant optimize owners were cross-checked.

| Area | Rust files | Physical lines | Test markers | Current execution truth |
|---|---:|---:|---:|---|
| Root Dist | 1 | 98 | 2 | Native entry publishes the umbrella manifest but no feature executor/provider bundle. |
| Root Editor | 3 | 56 | 0 | Capability/descriptor shell. |
| Root Runtime | 3 | 430 | 4 | Declares 15 features and four defaults; it does not register child feature behavior. |
| Baked Lighting | 6 | 146 | 1 | One registered no-op pass; enabled by default. |
| Contact Shadow | 7 | 1,419 | 8 | Own WGPU compute pass, but the algorithm is not a light-directed contact-shadow trace. |
| Decals | 6 | 199 | 1 | Descriptor plus one registered no-op pass. |
| Irradiance Volumes | 7 | 158 | 1 | Thin adapter to Runtime-owned executors. |
| Light Cookies | 7 | 159 | 1 | Thin adapter to Runtime-owned executors. |
| OIT | 7 | 364 | 5 | Thin adapter to Runtime-owned executors. |
| Planar Reflections | 7 | 251 | 3 | Thin adapter to Runtime-owned executor. |
| Post Process | 6 | 150 | 1 | One registered no-op pass; enabled by default. |
| Ray Tracing Policy | 6 | 189 | 2 | Capability report only; no executable backend/provider. |
| Reflection Probes | 13 | 1,138 | 9, 1 ignored | Descriptor has no pass; a synchronous capture helper exists outside registration. Enabled by default. |
| Shader Graph | 6 | 278 | 2 | Direct WGSL string concatenation plus one no-op pass. |
| SSAO | 6 | 227 | 1 | Real Runtime-integrated compute path. Enabled by default. |
| Subsurface Scattering | 7 | 296 | 6 | Thin adapter to Runtime-owned executors. |
| VFX Graph | 6 | 276 | 2 | Fixed workload descriptor plus two no-op executors. |
| Volumetric Fog | 10 | 1,786 | 13, 2 ignored | Three real Runtime-owned froxel executors exposed through a thin adapter. |

## 2. Structural performance findings

### P0: the stable product contract can report success without a rendering product

`plugin.toml` marks `runtime.plugin.rendering` as `stable/complete` and supports `native_dynamic`. The umbrella Runtime registers metadata and child manifests, while the Dist entry contains no child feature systems, executors, resources or provider lifecycle. Source-linked child crates can register behavior only when assembled separately; the declared native umbrella cannot recreate it. This makes source/dist behavior and cost depend on assembly accidents rather than one resolved product graph.

The four default features expose the defect in the MVP path. SSAO is executable, Reflection Probes registers no frame pass, and Baked Lighting/Post Process register executors that return `Ok(())` without work. Across the package there are **six no-op executor registrations in five functions**. A pass name, capability and success result can therefore be observed without pixels, artifacts or an execution receipt. Optimizing the small descriptor code would preserve a false product boundary.

### P0: feature ownership is split between plugins and Runtime

Irradiance Volumes, Light Cookies, OIT, Planar Reflections, Subsurface Scattering and Volumetric Fog forward directly to `zircon_runtime::graphics::*_render_pass_executor_registrations`. SSAO also consumes Runtime shader/graph machinery. The plugin owns product identity while Runtime owns the actual algorithm, resource lifetime and device behavior. The same split prevents coherent native loading, unload/device-loss handling, cache identity, editor diagnostics and per-feature cost attribution.

The required decision is binary for each feature: either it is a Runtime built-in and the plugin stops advertising optional ownership, or a typed `RenderFeatureProviderBundleV1` materializes the descriptor, executors, shaders/artifacts, resources, generation and lifecycle from every supported source/dist form. A second parallel renderer must not be introduced.

### P1: Contact Shadow is a fixed full-screen AO-like filter

The shader performs one invocation per output pixel and always reads center depth, normal, twelve fixed neighboring depth samples and one HZB sample. It never consumes a light ID/direction, light scissor, contact length, world/screen-space mode, surface thickness or temporal validity. The fixed stencil is therefore closer to local depth AO than contact shadows and can darken geometry unrelated to any light.

At 1920x1080 the shader schedules 2,073,600 invocations and at least 15 explicit texture loads per covered pixel, approximately **31.1 million loads** before cache effects; at 3840x2160 that rises to approximately **124.4 million**. These are static scale estimates, not measured GPU timings. The executor also holds a `Mutex<Option<Pipeline>>` through bind-group creation and command recording and allocates a bind group per frame. Fixing only the mutex would not repair the algorithm.

### P1: Reflection Probe capture is an unbudgeted synchronous six-render transaction

`capture_and_persist_reflection_probe` loops over six faces serially, clones the entire `RenderSceneSnapshot` per face, renders six independent HDR frames, accumulates all face texels, then synchronously writes `.zcube/.zribl`. The register path reads and decodes the just-written artifact again before insertion. Editor-triggered execution calls this chain synchronously and has no job handle, cancellation, progress, scene-revision key, frame budget or staged generation.

This is a main/editor-thread stall risk and an avoidable clone/write/read chain. It must become one capture/build job using a shared immutable scene snapshot, cube/multiview rendering where supported, time-sliced faces otherwise, asynchronous convolution/persistence, cancellation and atomic publication. Spawning six unconstrained threads would duplicate renderer/device pressure and is not an acceptable fix.

### P1: Shader Graph and VFX Graph do not implement their claimed algorithms

Shader Graph concatenates node snippets in input order. It has no stable node/edge identity, topology/cycle validation, typed pins, stage legality, binding/layout reflection, material domain, permutation key, compiler artifact or diagnostic source mapping. Its executor is no-op. VFX Graph checks only for shallow spawn/material presence, declares a fixed `[64, 1, 1]` simulation workload and `[1, 1, 1]` dispatch independent of `max_particles`, then registers no-op simulation and transparent passes.

These paths must converge on Runtime91's typed material/compiler pipeline and the canonical particle/VFX runtime, not grow a second string emitter or fixed-workload simulation inside this package.

### P1: Volumetric Fog has real work but no package-level budget contract

The plugin declares three async-compute passes, while all three executors live in Runtime's `advanced_lighting::froxel`. The current quality layouts use 48/64/96 depth slices and 3D RGBA16F resources; tests describe a 160x90x64 medium grid. Aliasing reduces three logical textures to two physical slots, but that medium pair still represents about **14.1 MiB** of raw texture storage and the 96-slice pair about **21.1 MiB**, excluding history, views and alignment. These are static resource estimates.

No current-source executable was available to prove queue overlap, occupancy, bandwidth, history stability or the 44,400 high-quality dispatch-group expectation in the ignored WGPU test. The package cannot claim async-compute benefit until Render Graph/RHI telemetry proves dependency-safe overlap on supported adapters.

### P1: editor capability shells do not close the authoring loop

Root and feature Editor crates mostly expose descriptors/capabilities. They do not own live settings, preview generation, compile/capture/bake jobs, progress/cancellation, current-generation diagnostics or product-frame evidence. Reflection Probe has helper calls, but no registered editor product lifecycle. Enabling an editor capability must resolve to the same runtime provider generation and artifact identity that the viewport executes.

### P2: product tests write caches beside source

Contact Shadow and Volumetric Fog WGPU tests construct `ProjectAssetManager::default()` and the renderer's default shader cache resolves relative to its project root. The package tree currently contains six Contact Shadow cache files under `.zircon-cache` and eighteen Volumetric Fog cache files under `.zircon/cache`; two older Contact Shadow files are tracked (8,114 bytes total), while the remainder are ignored. This is reproducibility and source-hygiene failure, and cold/warm measurements become ambiguous.

The fix belongs to the shared test/project/DDC boundary: every product test must receive an explicit non-C isolated cache root, assert no source-tree writes and clean only its owned root. The two tracked artifacts should be removed only in the same change that prevents regeneration.

## 3. Unreal source constraints

Unreal is used here as an architectural constraint, not as a promise that copying constants produces equal performance.

- `ScreenSpaceShadows.cpp` builds contact shadows for a concrete light, derives a light scissor, passes light direction, contact length/mode, casting and non-casting intensity, fade and surface thickness, then schedules an RDG compute pass and upsample. Zircon's directionless fixed-neighbor filter does not satisfy the same feature definition.
- `ReflectionEnvironmentCapture.cpp` exposes runtime/editor face time slicing, camera-distance or frame-age priority, capture budgets, hysteresis, refresh handling and fade/cross-fade state. Zircon's one-call six-face clone/render/persist/read transaction lacks every corresponding scheduling boundary.
- `VolumetricFog.cpp` derives a bounded grid from pixel size and Z slices, carries temporal-reprojection/history validity and quality parameters, injects local lights and schedules graph passes. Zircon should preserve one Render Graph owner and make grid/history/memory/queue budgets explicit rather than treating `AsyncCompute` as evidence of overlap.
- `HLSLMaterialTranslator.cpp` traverses material expressions/functions/layers, compiles by material property, tracks typed values/conversions and associates errors with expressions. Zircon's source-order string concatenation is not an equivalent graph compiler and must converge on Runtime91's typed IR/artifact path.

## 4. Dependency-ordered optimization plan

### M0: make MVP product truth fail closed

Remove no-op passes from default assembly. Until Post Process, Baked Lighting and Reflection Probes have concrete behavior, mark them partial/unavailable and publish one structured reason rather than a successful pass. Add product tests proving that the default MVP resolves one deterministic feature set, executes no no-op pass, creates no shader/pipeline during a stable frame and writes no cache into source.

Make linked-source, generated and native-dist resolution produce the same feature IDs, provider generations, executors, resources and lifecycle, or reject unsupported packaging at admission. An umbrella manifest without child behavior is not a native rendering product.

### M1: establish one resolved rendering product graph

Introduce one immutable `ResolvedProductPluginGraph` output consumed by App, Runtime and Editor. For each selected feature, bind one typed provider bundle containing descriptor, pass executors, shader/artifact identities, required resources, capability/device requirements, quality/budget schema, generation and unload/device-loss hooks.

Move or expose the currently Runtime-owned executors through that contract. Do not duplicate them in plugins. Registration and hot update are transactional; a generation becomes visible only after all required executors/artifacts/resources preflight successfully.

### M2: repair the high-cost algorithms

Replace Contact Shadow with a per-light, light-directed, scissored screen-space trace using HZB stepping, quality-dependent resolution/sample budget, thickness/contact length and temporal filtering. Cache pipelines/bind layouts by device generation and reuse bind groups where identities allow.

Replace Reflection Probe synchronous capture with a cancellable job state machine: select/age/prioritize, acquire one immutable scene generation, render bounded faces per frame or cube/multiview, filter/convolve, persist to a staged artifact, then atomically publish. Eliminate six scene clones and the immediate write/read decode cycle.

Route Shader Graph through typed material IR, topology/type/stage validation, compiler/reflection diagnostics and deterministic artifact keys. Route VFX Graph through the particle owner with emitter/system state, alive-count-driven GPU workloads, bounded spawn/update/compact/sort/render passes and deterministic CPU fallback.

### M3: close budgets, scheduling and editor lifecycle

Every feature declares CPU submission, GPU time, transient/persistent VRAM, upload, worker queue and per-frame invalidation budgets. Render Graph owns culling, aliasing, barriers and queue choice; RHI owns device generation, fences and loss recovery. Async compute is enabled only by captured overlap evidence.

Editor authoring resolves the same provider generation as the viewport. Compile, bake and capture actions return job handles with progress, cancellation, diagnostics and current/stale generation. Inactive/realtime-disabled viewports perform no feature tick, capture or background rebuild.

Move shader/pipeline/capture artifacts to explicit DDC roots with complete keys and integrity checks. Tests use unique D/E/F roots and prove zero source-tree writes. Remove the two tracked Contact Shadow artifacts after prevention is in place.

### M4: current-source performance qualification

Build a current-source Windows executable through the managed validator, then profile fixed MVP scenes and focused feature scenes at fixed resolution/quality/hardware/driver/power mode. Record disabled/enabled and cold/warm CPU/GPU p50/p95/p99, main/render/worker samples, pass/draw/dispatch counts, queue overlap, allocations, RSS, VRAM, cache hits/misses, shader/pipeline builds, power and energy/frame.

Use WPR/ETW for CPU scheduling, waits, IO, allocations and power. Use RenderDoc only for current-source pixel, pass/resource, barrier, copy/draw/dispatch and GPU timing evidence. Compare with Unreal experience only after scene, quality, warm state and hardware are matched; source structure alone is not timing evidence.

## 5. Acceptance gates

1. One resolved product graph closes source/generated/native parity. Unsupported child behavior fails admission; no parsed contribution is silently ignored.
2. Default MVP contains zero no-op or descriptor-only feature reported as complete. A successful pass has a concrete execution receipt and observable product effect.
3. Stable frames create zero shader modules/pipelines, perform zero source-tree cache writes and show bounded allocations/locks. Disabled features perform zero pass, task, IO and GPU-resource work.
4. Contact Shadow consumes a specific light and scissor, passes multi-light/camera-cut/thin-geometry visual gates, and publishes measured cost by covered pixel/light/quality rather than a fixed full-screen stencil.
5. Reflection Probe capture is cancellable and budgeted, clones the scene zero times per face, avoids immediate artifact write/read, and publishes only a complete current scene/provider generation.
6. Volumetric Fog reports grid dimensions, physical aliased bytes, history validity, pass GPU time and actual queue overlap. Quality changes stay within declared CPU/GPU/VRAM budgets without stale history.
7. Shader/VFX graphs use typed validated IR and deterministic artifacts; workload scales with compiled graph/emitter state, not fixed placeholder dispatches.
8. Editor and Runtime execute the same provider/artifact generation and expose progress, cancellation, stale-state and failure diagnostics.
9. Current-source WPR and RenderDoc evidence passes visual correctness first, then publishes reproducible CPU/GPU/VRAM/power data. Only then can this module enter the protected review ledger.

## 6. Validation status

- Static per-Rust-file review: **114/114 complete**.
- `rustfmt --check --config skip_children=true`: **pass** for all 114 files after four mechanical formatting repairs.
- Root/native/default feature product truth: **failed statically** because native Dist does not materialize child behavior and default features include no-op/descriptor-only paths.
- Source-tree cache isolation: **failed statically**; 24 cache files are present, including 2 tracked Contact Shadow artifacts.
- Cargo/test execution: **pending** because the managed Windows validation session is not executable; no raw Cargo lane was substituted.
- WGPU ignored product tests: **not run**; their source assertions are not measurements for this review.
- Current-source executable, WPR/ETW, RenderDoc, visual, GPU, VRAM and energy qualification: **pending**.
- No structural rendering implementation was changed. The reviewed architecture requires owner-plan convergence before algorithm edits; the only source edit was behavior-neutral formatting.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
