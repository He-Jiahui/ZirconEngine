---
date: 2026-08-15
related_plan: docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
doc_type: structural-performance-research
status: implementation_blocked_by_m0_baseline
coordination_owner: docs/plans/zircon_runtime/render/02
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/core/framework/render/frame_profile.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
tests:
  - deterministic serial/parallel command and cache-stat parity
  - GPUScene span lifetime and cache reuse/invalidation contracts
  - coordinator-managed Windows profile, PNG, RenderDoc, and power evidence
---

# Render02 Mesh Preparation Pipeline Ownership And Parallelism Research

## Status

This is a structural and measurement plan, not an implementation or a measured optimization.
M0.3 remains incomplete and the managed Cargo/GPU lane has not been released. No production source,
Cargo run, renderer run, RenderDoc capture, PNG, timing sample, or power sample was produced for
this record.

## Current-Source Findings

The existing parallel command builder is not a mutex-based fake parallel path. Its owner thread
sorts batches, resolves variant IDs, performs cache lookup, and builds immutable
`PreparedBatchPlan` records. Workers consume those records through the caller-provided `TaskPool`;
the owner thread then merges commands and cache stores in source order. Duplicate cache keys take
the serial path. This correctly preserves variant allocation and cache transaction ordering.

However, the parallel section begins only after `build_compiled_scene_draws` has completed and
after `assign_execution_owned_indirect_args` has patched residual draws. It calls
`TaskPool::install`, which synchronously waits for the worker result. The earlier serial path
still performs snapshot ordering, per-instance pending-draw expansion, model/skinning/morph
preparation, virtual-geometry planning, GPUScene registration/update staging, cache extraction,
and residual `MeshDraw` materialization. A multi-core speedup in the final command conversion is
therefore possible, but it is neither proof of prepare-pipeline overlap nor evidence that it is
the dominant CPU cost.

The static-command cache also cannot be treated as an immutable draw-data cache. Cache misses are
rebuilt after GPUScene synchronization because command construction embeds a
`GpuSceneInstance { first_instance_index, instance_count }` source. `GpuScene::register` preserves
that span while the same stable key stays registered, but `retain_registered_keys` unregisters
absent keys and the allocator reuses released spans after the frame boundary. A cache design that
moves before GPUScene ownership must either prove the relevant key remains registered or patch
the frame-local span after lookup. It must not assume a numeric span is a permanent object ID.

`RenderFrameProfile` currently reports whole-frame CPU submit time and per-pass recording time,
but has no typed CPU observation for mesh input collection, pending expansion, GPUScene staging,
cache extraction, plan preparation, worker wall time, or ordered merge. Existing mesh counters
can report command/cache totals, but cannot identify the CPU phase that limits a scene.

## Unreal Reference Decision

Unreal's `FParallelMeshDrawCommandPass` represents setup as an explicit task context and event;
the renderer can defer its synchronization until the draw pass requires sorted/merged commands.
Its important principle is an explicit ownership boundary between preparation data, task state,
and render-thread submission, not its allocator or RHI command-list implementation.

Zircon keeps its WGPU command-encoder and mutable GPUScene operations on their current owner. It
must not port Unreal's command-list threading or create an ad hoc render-only thread pool. The
relevant adoption is a Zircon `MeshPreparationTicket`-style boundary only after profiling proves
that waiting at the current `TaskPool::install` prevents useful overlap.

## Required Ownership Model

The future pipeline has three ordered owners:

1. `MeshPreparationCoordinator` remains the deterministic render-side owner of frame input order,
   resource/variant/cache transactions, GPUScene registration, WGPU resource creation and writes,
   and final cache-store commit.
2. A pure immutable plan contains source index, phase specifications, resolved variant IDs, cache
   lookup outcome, static revisions, visibility decision, and either a stable GPUScene key or a
   validated frame-local span. Workers may build command chunks only from this plan.
3. A single merge point consumes completed chunks in canonical source/phase/sort-key order,
   patches any explicitly frame-local instance spans, commits cache stores, and publishes one
   `MeshPassCommandBuffers` artifact for indirect-plan construction and graph execution.

Workers never hold `&mut MeshPipelineVariantResolver`, `&mut CachedMeshDrawCommands`,
`&mut GpuScene`, `wgpu::Device`, `wgpu::Queue`, or `wgpu::CommandEncoder`. No per-batch mutex,
second cache/variant registry, detached retry loop, or unordered result map is permitted.

For cached commands, choose one explicit contract during implementation review:

- retain a GPUScene registration lease for every cache-resident command and invalidate the command
  before its span can be recycled; or
- cache a stable source key and patch `first_instance_index`/`instance_count` after the current
  frame's GPUScene registration.

The first option trades residency for simpler replay; the second permits reclamation but makes the
patch point part of the command ABI. The current source proves neither is a safe implicit
assumption across an absent/reappearing instance.

## Measurement Before Algorithm Change

First add a narrow, typed mesh-preparation observation to the existing frame-profile contract.
It records disabled/serial/parallel/fallback mode, TaskPool parallelism, input mesh count,
pending-draw count, prebuilt cache-hit count, residual count, and CPU elapsed time for:

- ordered input and pending expansion;
- virtual-geometry/morph preparation;
- GPUScene registration/staging and upload submission;
- cache extraction and owner-side variant/cache planning;
- worker chunk wall time; and
- ordered merge/indirect-plan preparation.

Fallback reason is typed (`single_worker`, `small_batch`, `duplicate_cache_key`, or
`parallel_disabled`), not inferred from a zero duration. Instrumentation is diagnostic-gated and
must not add a timer, allocation, or lock to ordinary product frames when disabled.

Only a profile that identifies one of these phases as material can authorize the associated change:
parallel CPU input expansion, deferred ticket synchronization, cache-ABI patching, or batch-key
algorithm refinement. A command-count decrease or a successful rayon test alone is not a
bottleneck measurement.

## Product Protocol

After M0.3 is green and UI12 releases the managed lane, run a deterministic static-heavy control,
a material-diverse control, and a skinned/morphed dynamic control at identical source fingerprint,
adapter, driver, resolution, camera, and quality settings.

1. Discard 30 warm-up frames and retain 120 settled frames per mode. Preserve raw mesh-preparation
   observations, CPU submit time, GPU pass timing, cache counters, command counts, and fallback
   mode before calculating median, p95, and MAD.
2. Capture cold and warm frames through `D:\\Tools\\renderdoc\\renderdoccmd.exe`. Confirm pass
   order, command/draw counts, GPUScene buffer identity, and indirect source match between the
   compared modes.
3. Store matched PNGs, JSON profiles, graph dumps, and RDC files under
   `docs/tests/runtime/render/` with prefix `plan02_mesh_preparation_`. The opaque deterministic
   control requires exact decoded RGBA equality; every image must have its matching profile and
   RDC.
4. Record GPU utilization and board power with adapter, driver, AC state, and sampling interval.
   Report unavailable telemetry explicitly and never use frame time as a power proxy.

The deferred-ticket path is accepted only when it reduces a measured CPU critical-path phase
outside its noise envelope, preserves serial/parallel command and cache-stat parity, preserves
images/fallback behavior, and does not cause a GPU-time or power regression outside the measured
noise envelope.

## Ordered Gates

1. Wait for current-source M0.3 and managed-lane release.
2. Add the typed observation and source contracts without changing scheduling.
3. Establish which mesh-preparation phase is material on the three workload classes.
4. Implement only the ownership change associated with that phase, beginning with GPUScene cache
   span lifetime if cache reuse is the limiting path.
5. Re-run the paired Windows profile, PNG, RenderDoc, and telemetry comparison before claiming
   improvement.

No frame-time, throughput, energy, or algorithmic-optimality claim is made by this record.
