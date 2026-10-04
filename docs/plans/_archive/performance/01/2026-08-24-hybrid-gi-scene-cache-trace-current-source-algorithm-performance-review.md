---
title: Hybrid GI Scene Cache Trace Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors
  - zircon_plugins/hybrid_gi/runtime/src/shaders
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/19-first-party-hybrid-gi-source-runtime-editor-dist-catalog-scene-representation-surface-cache-global-sdf-radiance-cache-probe-trace-denoise-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/98-runtime-hybrid-global-illumination-scene-representation-surface-cache-global-sdf-screen-probe-radiance-cache-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSceneData.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCacheFeedback.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeGather.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenRadianceCache.cpp
  - dev/godot/servers/rendering/renderer_rd/environment/gi.cpp
---

# Hybrid GI Scene Cache Trace Current-Source Algorithm Performance Review

## 1. Algorithm audit

This report covers the algorithmic core inside the 250-file fingerprint recorded by the product review. No individual loop is accepted as the root bottleneck until its producer, lifetime, scheduling and consumer are considered together.

### P0: scene representation is rebuilt as full snapshots

Each frame clones and sorts the complete mesh list and three light lists. Every mesh becomes one spherical card centered at translation with radius derived from maximum scale. Screen probes are the first `min(trace_budget, cards.len())` cards with sequential IDs; they are not placed from depth, normal, motion or view-space importance. Any light change dirties all resident surface pages.

This is neither an incremental render scene nor a screen-probe algorithm. Complexity and invalidation scale with authored scene size while quality remains unrelated to visible pixels.

### P0: the surface cache does not represent surfaces

The cache repeatedly clones eight vectors and materializes tree-map/tree-set snapshots. It assigns one card and one page per mesh; over-budget content survives only as IDs. Material capture samples one fixed UV `[0.5, 0.5]`, writes uniform color/depth tiles and does not rasterize the surface. The cache has no persistent albedo, normal, emissive, opacity and depth layers that can be independently invalidated and sampled.

Unreal's `LumenSceneData.h:1094-1157` owns sparse cards/page tables, unique update-index lists, async scatter uploads and persistent last-used buffers. `LumenScene.cpp:900-976,1058-1157,1422` maintains explicit pending add/update/remove sets and processes those deltas. Zircon should adopt those ownership properties rather than the exact API.

### P0: Global SDF performs CPU page discovery and transient batch upload

The current Global SDF owns four camera-snapped clipmaps, 64 cells per axis, eight-cell pages, eight pages per edge and a 128-page cap. Synchronization rebuilds and sorts all 512 logical pages per clipmap. Each dirty region scans all resident pages; each object visits four clipmaps and all influenced pages, retaining up to 32 candidates per page. More than 128 influenced pages for an object/clipmap falls back to whole-clipmap voxel work.

Dispatch preparation rebuilds hash maps and packing vectors and may upload up to four million voxel words. Each batch creates seven transient buffers plus a bind group. Completion reads data back to CPU before committing resident pages. The 2,048-slot trace table is rebuilt and FNV-hashed on each request even when its upload is later skipped. The shader performs nearest sampling for at most 16 steps and may return synthesized fallback color.

Godot's `gi.cpp:414-425,490-590` creates SDFGI resources once, persists probe/history/occlusion and per-cascade SDF/light buffers, and moves cascades from camera/probe-grid dirty regions. Unreal's Lumen scene similarly retains page allocation and consumes explicit deltas. Both references contradict rebuilding GPU authority through CPU maps/readbacks each frame.

### P0: voxel lighting is a fixed teaching grid

Voxelization is scene-centered rather than view-centered, limited to at most eight clipmaps, and evaluates a 4x4x4 grid. It iterates cards against overlapping cells for every clipmap, applies a manual light formula, stores RGB8 and later merges readback snapshots on CPU. This cannot be optimized into scalable GI through local allocation changes; the data representation and update authority must change.

### P0: completion shader has high-order scans

`update_completion_output.wgsl` scans all traces for every resident probe and calls the traced contribution twice. Each contribution may walk four ancestors; ancestor lookup scans resident probes and then traces again. Gather scans resident probes, and optional scene contribution scans descriptors. Pending probes repeat much of the same work.

The static work shape is approximately **O(P * (P + T * (1 + A) + A * P + D))**, where `P` is probes, `T` traces, `A <= 4` ancestors and `D` descriptors. Scene preparation can additionally scan all descriptors and repeat a descriptor scan through owner lookup, producing **O(D^2)** behavior per probe. Fixed small packet limits hide the scale problem rather than solve it.

### P0: radiance cache stores constant tiles, not directional radiance

The cache is capped at 32 entries with 4x4 RGBA8 tiles and a 2x2 interior. Trace writes the same radiance into all interior texels. Consume rejects an interpolation if any of eight corners is absent or mismatched. Update dispatches six stages over the same small update set. This has the pass count of a cache without directional sampling, meaningful atlas resolution or robust sparse interpolation.

Unreal's `LumenRadianceCache.cpp:186-191` persists indirection/final/depth/occlusion atlases; `:309-423` uses GPU free-list and trace allocators; `:634-677` builds indirect args; `:788-874` generates uniform/adaptive trace tiles. `LumenScreenProbeGather.cpp:47,2380-2430,2643-2671` derives probe layout and adaptive work from the screen rather than selecting the first scene cards.

### P1: fixed packets and resolve hide missing reconstruction

The scene packet exposes at most 16 pages, four voxel clipmaps and 64 cells. Trace uses fixed 8x8 tiles and at most 16 tiles per probe; resolve maps a pixel to one tile and applies a 3x3 tile-space filter. Global SDF also caps tracing at 16 steps. These constants reduce execution cost by truncating information, not by preserving quality with bounded adaptive work.

## 2. Architecture target

The canonical owner should converge on one chain:

`HybridGiSceneCompiler -> HybridGiArtifactStore -> HybridGiRenderSceneService -> {SurfaceCacheResidencyService, GlobalSdfResidencyService} -> HybridGiTraceBackendRegistry -> RadianceCacheService -> HybridGiReconstructionService -> HybridGiBudgetController -> HybridGiDiagnosticsService`, with `HybridGiAuthoringService` consuming the same typed generations.

Each arrow carries immutable handles/deltas and explicit generation receipts. GPU-owned residency, page tables, atlases, trace queues and history remain on GPU; CPU readback is limited to bounded feedback, counters and requested capture products.

## 3. Dependency-ordered implementation plan

### M0: establish unique ownership and baseline counters

Hard-cutover duplicate core/plugin GI composition. Instrument per-generation add/update/remove counts, scene compile work, page updates, buffer/pipeline creation, upload/readback bytes, dispatch work, overflow/fallback and GPU timestamps. Capture correctness before changing representation.

### M1: implement an incremental render-scene generation

Compile stable mesh/material/light identities into persistent render-scene records. Publish add/update/remove deltas and independent transform/material/light generations. Replace full-vector clones/sorts and all-light invalidation with indexed dependency fan-out and dirty bounds.

### M2: replace spherical cards with cooked cards and a real surface cache

Generate coverage-oriented card captures during asset cook, store stable card/page metadata, rasterize real material/depth into layered persistent atlases and update sparse pages through bounded GPU feedback. Track last-used generations, byte residency and eviction without full snapshots.

### M3: derive screen probes from visible pixels

Place uniform and adaptive probes from depth/normal/motion and view resolution. Generate directional trace tiles, compact active work and indirect dispatch on GPU. Scene cards inform intersections/residency; they do not define screen-probe positions.

### M4: make Global SDF persistent and delta-driven

Persist clipmap page tables, allocator/free lists, object grids, scratch and voxel atlases by device generation. Scroll camera-centered cascades, clear only exposed regions, bin changed objects on GPU, compact work and update only dirty pages. Eliminate per-batch seven-buffer creation and full page result readback.

### M5: implement a directional radiance cache

Use persistent indirection/radiance/depth/visibility atlases, view-driven probe marking, GPU allocation and indirect trace scheduling. Store directional samples and robust validity/age; update only selected probes under a measured budget.

### M6: add temporal reconstruction and denoise

Use motion/depth/normal rejection, disocclusion handling, history confidence, spatial reuse and quality-aware upsampling. Replace the fixed tile lookup/3x3 filter with a reconstruction service that reports rejection and fallback counters.

### M7: add trace backends only after their owners exist

Software SDF/screen traces remain the baseline. Hardware ray tracing integration waits for Runtime28 to provide acceleration-structure lifetime, scheduling and capability contracts; it must not fork scene ownership.

### M8: complete authoring and scalability

Expose scene compiler state, cache residency, dirty reasons, probe/trace budgets, overflows, backend choice, timestamps and capture generations in the editor. Quality tiers must map to measurable budgets rather than three unrelated environment integers.

## 4. Quantified acceptance matrix

Use current-source builds on the same hardware across static, camera-motion, transform-heavy, material-change and light-change scenes at 1080p and 4K. Record raw samples and p50/p95/p99 for main/render/worker CPU and each GPU pass; also record scene deltas, pages updated, probes/traces, dispatches, allocations, upload/readback bytes, RSS/VRAM, scheduler waits, wakeups and package/GPU power.

Required invariants:

1. An unchanged warm frame performs zero scene recompile, full-list sort, fixed resource creation and complete-state readback.
2. A one-object transform updates work proportional to its dirty bounds/pages, not all scene objects/pages/lights.
3. A one-light edit invalidates only affected lighting dependencies.
4. Probe/trace work scales with visible resolution, adaptive occupancy and budgets; overflow is explicit and does not silently truncate correctness.
5. Captured GPU timestamps and RenderDoc pixels/resources agree with the declared pass graph and fallback mode.
6. Comparison with Unreal/Godot uses equivalent scenes, resolutions, quality and hardware. Source topology is evidence for design; it is not evidence for a latency or power number.

## 5. Current acceptance status

Static algorithm review is complete for the captured source fingerprint. Dynamic CPU/GPU/power qualification is pending because there is no launchable current-source executable; RenderDoc CLI is unavailable. The found architecture requires cross-owner hard cutover, so no local source tweak is represented as bottleneck removal.
