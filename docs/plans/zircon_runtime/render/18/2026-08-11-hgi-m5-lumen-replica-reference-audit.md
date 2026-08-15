---
date: 2026-08-11
related_plan: docs/plans/zircon_runtime/render/18/2026-08-10-hgi-m5-sdf-architecture-and-performance-plan.md
doc_type: reference-implementation-audit
status: implementation_planning
reference_commit: 5896d707ca9936d22c1b4dc9bd9c7f8e4514912b
coordination_owner: docs/plans/zircon_runtime/render/18
---

# HGI M5 Lumen Replica Reference Audit

## Scope

This record audits the local compute-shader replica at
`dev/LumenInUE5.5.4WithComputeShader`, not Unreal Engine as an upstream API contract. It informs
M5 owner boundaries and validation design. It does not provide performance numbers, image-quality
evidence, WGPU validation, or a substitute for the required product PNG and RenderDoc capture.

## Observed Reference Flow

The reference keeps Radiance Cache state across frames through an indirection volume and separates
allocation, tracing, filtering, and final-atlas publication:

```text
previous indirection
  -> UpdateCacheForUsedProbes
  -> AllocateUsedProbes
  -> AllocateProbeTraces
  -> GenerateProbeTraceTiles / TraceFromProbes
  -> FilterProbeRadiance
  -> FixupBordersAndGenerateMips
  -> final radiance atlas
```

`AllocateUsedProbes.cpp` binds a free list, allocators, the current indirection volume, and
indirect-dispatch arguments. `AllocateProbeTraces.cpp` produces bounded trace data from that
indirection state. `FixupBordersAndGenerateMips.cpp` consumes the trace data and executes from an
indirect argument offset. The world trace path in `TraceVoxels.cpp` binds Global SDF page, coverage,
table, mip, and bounded page-object-grid inputs before sampling the Radiance Cache final atlas.

## Adopted Contracts

| Reference property | Zircon M5 contract |
| --- | --- |
| Frame-to-frame indirection ownership | Radiance Cache residency must retain stable slot generation and only replace a committed resident slot after the matching update succeeds. |
| Allocation before trace work | Mark/allocate/consume remain distinct WGPU dispatches; consume counters describe committed writes, not speculative marks. |
| Bounded indirect work | Per-frame page, probe, candidate, upload, and readback budgets are explicit. Overflow or unavailable work stays a typed fallback rather than becoming a valid empty sample. |
| Global SDF plus voxel fallback | A page is sampleable only after complete Ready Mesh SDF contributors produce a generation-matched page. Missing, deforming, invalid, or overflowed contributors stay on the voxel route. |
| Post-trace atlas finalization | Border and mip work must consume only validated committed cache payloads, preserving the shader-side slot-generation checks before the committed-write counter advances. |

## Non-Portable Details

The replica is a D3D12 teaching implementation with fixed descriptor slots, fixed initial dispatch
dimensions, global singleton resources, and direct resource-state management. Zircon must not copy
those choices into its WGPU render graph. Its product path keeps resource ownership in the compiled
graph, uses validated bind-group layouts, and records capabilities and typed fallbacks through the
existing frame-profile DTO.

The reference binds a large set of global tracing resources in one pass. Zircon retains the M5
three-axis capability graph instead: trace domain, intersection backend, and lighting source stay
independently selected so unavailable Global SDF work cannot suppress the voxel fallback or claim a
Global SDF hit.

## Implementation Consequences

1. Do not tune probe count, page dimensions, workgroup size, or atlas resolution from the replica's
   literals. Those require current-source CPU/GPU profiles and equal-output captures.
2. Preserve explicit frame generation on Global SDF build observations. A late readback may only
   commit the generation it was dispatched for; stale completion leaves the page dirty for a later
   build.
3. Keep allocation counters and build/readback diagnostics bounded by the shared WGPU frame ring.
   A frame rejected during readback admission is unavailable for that frame; staging capacity and
   encoding failures propagate through the existing render error path. Do not infer a capacity
   failure from an already-guarded request admission or turn it into a retry loop.
4. Require a matching product PNG under `docs/tests/runtime/render/`, frame profile, graph dump,
   and RenderDoc `.rdc` from `D:\Tools\renderdoc` before reporting image or performance parity.

## Current Status

The reference audit is complete. It validates the M5 architectural direction but closes no M5
acceptance gate. Dynamic WGPU execution, source-matched PNG/RDC output, timing, resource-byte
measurement, and power telemetry remain coordinator-owned evidence.
