---
date: 2026-08-15
related_plan: docs/plans/zircon_runtime/render/04-visibility-culling.md
doc_type: structural-performance-research
status: implementation_blocked_by_m0_baseline
coordination_owner: docs/plans/zircon_runtime/render/04
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/bind_group_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/phase_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingLoadBalancer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingLoadBalancer.h
tests:
  - source and unit contracts for shared arena ranges and graph backing identity
  - coordinator-managed Windows WGPU product run, PNG, RenderDoc, and power/timing evidence
---

# Render04 HZB Indirect Resource Ownership And Load-Balancing Research

## Status

This is an architecture and measurement plan, not an implementation or a performance result.
Static source inspection on 2026-08-15 found a graph-resource ownership mismatch before any
load-balancing decision could be credible. M0.3 is not green and the managed Cargo/GPU lane has
not been released, so no production source was changed and no Cargo, renderer, RenderDoc, PNG,
GPU-time, or power measurement was run for this record.

## Current-Source Finding

The Render04 graph declares one logical external buffer for each HZB indirect-resource family.
`bind_execution_owned_graph_resources.rs` resolves every such logical resource from the first
non-empty HZB execution and names its physical backing `:phase0`. In contrast,
`HzbOcclusionCuller::execute` iterates all eligible HZB phase executions (currently opaque,
alpha-mask, advanced-PBR opaque, and velocity) and directly creates a per-execution compute
binding. Each execution owns a separate argument buffer and separate compaction resources.

Consequently, the graph declaration represents phase zero while later HZB dispatches consume
different physical buffers outside that graph resource identity. This is an ownership and
observability defect, regardless of whether it changes pixels on a particular workload. It must
be corrected before interpreting a graph capture or optimizing the culling shader.

The current shader launches one invocation per indirect argument with a workgroup size of 64.
Each invocation scans that argument's instance span while compacting its visible instances. This
is efficient for short and similarly sized spans, but a long span can dominate the completion time
of its wave. Existing readback statistics report only tested/culled argument and instance totals;
they cannot distinguish balanced inputs from a long-tail span distribution.

## Reference Decision

Unreal's `InstanceCullingLoadBalancer` uses 64-thread groups and packs item spans into batches,
splitting large spans so work is distributed over lanes. Zircon should adopt that principle only
after its own input distribution and HZB GPU scope prove it is needed. A direct port would import
Unreal command-list and buffer conventions that do not belong to Zircon's WGPU/RDG boundary.

The first correction is therefore resource ownership, not shader scheduling. The second,
conditional correction is a Zircon-specific packed-span compaction path.

## Proposed Ownership Boundary

`mesh_pass` should own a frame-scoped, grow-only `MeshIndirectFrameArena` (final type name is
subject to the implementation review). It provides one physical backing per indirect resource
family and phase-local ranges for:

- source indirect arguments;
- compaction metadata;
- visible-instance indices;
- draw counts; and
- compacted indirect arguments.

All mesh phases may reserve ranges from the same arena, including phases not currently HZB
eligible. A `MeshIndirectDrawExecution` remains the phase-facing owner of its range and plan; it
does not regain ownership of a second set of backing buffers. The arena owns allocation growth,
resource revision, and frame lifetime. Phase execution owns offsets, active byte sizes, and clear
ranges.

The six HZB graph external resources must bind to those arena backings, with no `phase0` fallback
name. The HZB compute bind group must bind the execution's ranges, and its cache identity must
include the arena resource revision plus every bound range and the sampled HZB identity. Storage
binding alignment is obtained from the negotiated device limits rather than a literal alignment
constant. Output clears cover only the phase ranges, not unused arena capacity.

This preserves the existing per-phase replay behavior while making graph declarations, RenderDoc
resources, and actual dispatch inputs describe the same physical objects.

## Required Static Contracts

The ownership change is acceptable only when focused tests prove all of the following:

1. phase range allocation is non-overlapping, bounded by its backing capacity, and remains stable
   when an unrelated phase changes;
2. grow/revision invalidates cached bindings, while unchanged backing-plus-range reuses them;
3. every declared HZB external resource maps to the shared arena backing, never an arbitrary first
   execution;
4. clears and binding ranges are phase-local, including zero-argument phases; and
5. mesh replay, visible remap, indirect-count fallback, and HZB-disabled paths retain their
   existing draw-source selection.

No phase-specific graph-name fanout, hidden alias, second HZB resource registry, or phase-zero
compatibility fallback is permitted. Those options retain the ownership ambiguity instead of
removing it.

## Scheduling Decision Gate

Before changing the one-argument-per-invocation shader, add diagnostic-only span telemetry from
the already prepared CPU compaction plan. Per HZB phase, record argument count, total instances,
maximum span, and histogram buckets `1`, `2-4`, `5-16`, `17-64`, and `65+`. This instrumentation
must not add per-instance atomics to the normal GPU shader.

After the shared-arena correction has product evidence, compare the span histogram with a
timestamp scope covering each HZB cull phase. A packed-span prototype is justified only if the
measured long tail coincides with HZB GPU time or frame-time degradation relative to the balanced
control. Totals alone, a large scene name, or a reference-engine implementation are insufficient.

When justified, pack cull items into 64-lane batches. Large source spans are partitioned into
fixed-size items; each lane tests one item; a bounded scan/prefix phase compacts output indices;
and the final draw-count/argument update remains deterministic. The packed metadata is a distinct
ABI from the current one-argument metadata and must be versioned rather than overloaded. The
normal short-span path remains available until paired evidence establishes a safe default.

## Product Measurement Protocol

The coordinator-managed Windows run starts only after M0.3 is green and UI12 releases the
Cargo/GPU lane. For each implementation stage, use the same source fingerprint, adapter, driver,
resolution, camera, quality configuration, and scene extract for baseline and comparison runs.

1. Run 30 warm-up frames and retain 120 settled samples. Record raw per-frame CPU submit/mesh
   preparation time, HZB phase GPU timestamps, candidate counts, span histogram, arena bytes,
   allocation/revision counts, and fallback state.
2. Capture one cold and one warm frame with `D:\\Tools\\renderdoc\\renderdoccmd.exe`. Verify that
   the six logical HZB graph resources and every active cull dispatch resolve to the shared arena
   backing plus their documented ranges.
3. Write matched product PNGs, JSON profile sidecars, graph dumps, and RDC captures under
   `docs/tests/runtime/render/` using the prefix `plan04_hzb_indirect_`. PNG equality is required
   for the deterministic opaque control scene. A screenshot without matching sidecar and RDC is
   not acceptance evidence.
4. Collect GPU utilization and board power through the available vendor telemetry. Record adapter,
   driver, AC state, sampling interval, and unavailable telemetry explicitly; do not infer power
   from frame time.
5. Compare median, p95, and MAD for valid samples. A scheduling change advances only if it removes
   a measured HZB bottleneck without an image, fallback, CPU, GPU, or power regression outside the
   measured noise envelope.

## Ordered Implementation Gates

1. Wait for the current-source M0.3 compile baseline and managed-lane release.
2. Implement and statically review the shared-arena ownership correction first.
3. Run the managed compile, product PNG, RenderDoc, and telemetry protocol for that correction.
4. Add CPU-side span telemetry and establish whether the current scheduling is actually skewed.
5. Only then implement, measure, and compare the packed-span prototype.

This record deliberately makes no claim about a frame-time reduction, energy reduction, or
algorithmic optimality. Those conclusions require the artifacts and paired measurements above.
