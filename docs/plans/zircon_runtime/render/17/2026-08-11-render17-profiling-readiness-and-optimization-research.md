---
date: 2026-08-11
related_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
doc_type: optimization-readiness-research
status: implementation_ready_for_managed_validation
coordination_owner: docs/plans/zircon_runtime/render/17
---

# Render17 Profiling Readiness And Optimization Research

## Status

This is an optimization-readiness report, not a performance result. The instrumentation repair is
complete at source level and its focused formatting, diff, and source guards pass. No Cargo, WGPU
product run, RenderDoc capture, GPU timestamp sample, power telemetry, PNG, or RDC was generated in
this session. Therefore it makes no frame-time, throughput, energy, or image-quality claim.

The next algorithm change must follow the measurement protocol in this record. Static source
inspection establishes which observations can be trusted and which instrumentation behavior must
be repaired first; it cannot establish a runtime bottleneck.

## Scope And References

- Plan authority: `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`, PF-M1 before
  CPU parallelization, budget tuning, or feature-quality changes.
- Primary reference principle: Unreal's GPU profiling keeps event ownership on the render path and
  accepts delayed query results rather than stalling frame submission. Zircon retains its own
  smaller WGPU/RDG design rather than copying Unreal's complete profiler or command-list system.
- Secondary reference principle: Bevy separates capability-gated GPU diagnostics from the render
  execution policy. A diagnostic mode must not silently measure a materially different renderer.
- Current Zircon owners inspected: `zr_rhi_wgpu::gpu_pass_timer`, product device negotiation,
  scene graph execution, `FrameProfiler`, `RenderFrameProfile`, and viewport capture attachment.

## Verified Current State

| Concern | Current source evidence | Consequence |
| --- | --- | --- |
| Profile contract | `core/framework/render/frame_profile.rs` contains frame, pass, subsystem, memory, upload, graph-cache, variant-miss, lint, and parallel-recording fields. | A single profile DTO already exists; a new parallel metrics format is not justified. |
| Delayed readback | `zr_rhi_wgpu/src/gpu_pass_timer.rs` resolves timestamps through `GpuReadbackQueue`; `FrameProfiler` retains four pending profile generations and merges late results. | Late GPU values can be attached to the source frame without stalling the CPU path. |
| Capture projection | `viewport_record/capture.rs` serializes a matching `RenderFrameProfile` into the captured frame and accepts a later timing backfill. | Product captures can carry profile metadata once a managed renderer run produces them. |
| Product graph instrumentation | `execute_graph_stage.rs` reserves a scope per live graph pass and writes begin/end timestamps around executor dispatch. | The graph path has a concrete timing attachment point. |
| Capability gate | `GPU_TIMESTAMP_REQUIRED_FEATURES` currently requires both `TIMESTAMP_QUERY` and `TIMESTAMP_QUERY_INSIDE_ENCODERS`; device negotiation requests the pair only when both are available. | The Render17 failure record currently makes this all-or-nothing pair authoritative, so any alternate pass-boundary design needs product evidence and an explicit successor decision. |
| Parallel-recording policy | `execute_graph_stage.rs` pre-reserves timestamp scopes and carries them into parallel encoder buckets; pipeline statistics remain serial. | Timestamp observation no longer selects a different graph-recording policy, while the statistics path retains its current mutable recorder owner. |

## Repaired Source Risks

These are source-level repairs, not measured bottlenecks.

1. The current `GpuPassTimer` uses `CommandEncoder::write_timestamp` and consequently requires
   `TIMESTAMP_QUERY_INSIDE_ENCODERS`. A pass-boundary descriptor design might extend coverage, but
   the active `failure-2026-07-29-gpu-timestamp-feature-set-const.md` requires the existing
   all-or-nothing feature pair. Static inspection alone cannot replace that accepted compatibility
   contract with a single-feature policy.
2. Query pairs are reserved deterministically before task fan-out. Each prepared pass owns a
   clonable timestamp scope, so timestamp-enabled frames remain eligible for the same parallel
   encoder partitioning as normal frames.
3. `GpuPassTimer::resolve_and_request` now records whether the bounded readback request was
   admitted. A rejected request becomes `deferred`; it is not reported as a zero-duration sample
   and does not create a retry ring.
4. Fixed timestamp-budget exhaustion becomes `capacity_exhausted`. Partial timings may still be
   attached to the source generation, but the frame remains explicitly non-comparable.

## Implemented Repair Direction

The completed source change is an observability correction, not a rendering-algorithm
optimization.

```text
adapter timestamp capability tier (current authoritative pair)
    -> graph timestamp scopes with an explicitly reported capability state
    -> serially reserve stable query pairs before optional parallel fan-out
    -> submit/resolve through the bounded readback ring
    -> profile status: measured | unavailable | deferred | capacity_exhausted
    -> late result merges with the matching capture generation
```

- Keep the active all-or-nothing feature pair until an approved successor has product evidence for
  descriptor-level render and compute timing across the supported adapter matrix. Do not silently
  weaken the existing failure contract.
- Preserve existing graph topology ordering. Timestamp scopes travel with their already prepared
  pass, so they do not establish a second scheduling owner.
- Do not change the timer ring size, add a new thread pool, alter radiance-cache dispatch, or tune
  shader workgroups in this repair. Those changes require data from the corrected profiler.
- Preserve the existing nonblocking admission and explicit `encode_copies` failure path. Add a
  profile-visible unavailable state only for a timer that was not admitted at frame start or that
  violates its request invariant; do not add retries, polling, or a second ring.

## Current-Source Repair Contract

The re-review on 2026-08-11 confirms the owner boundary and the minimum repair before a product
measurement:

- `zr_rhi_wgpu::gpu_pass_timer` owns only timestamp-pair reservation, bounded query capacity,
  and the asynchronous readback outcome for one generation. Its frame observation reports
  `pending`, `deferred`, `capacity_exhausted`, or `no_passes`; it does not expose WGPU errors to
  framework consumers.
- `core::framework::render::RenderFrameProfile` receives a serializable neutral
  `RenderGpuTimingStatus`. The profile distinguishes `disabled`, adapter `unavailable`,
  `pending`, readback `deferred`, query `capacity_exhausted`, and `measured`, so an absent
  `gpu_frame_time_us` is no longer an ambiguous performance datum.
- The scene renderer is the only projection point: it records the requested capability tier,
  converts the RHI observation for the same frame generation, and leaves `FrameProfiler` to merge
  a delayed successful result. No core-framework type depends on WGPU.
- `GpuPassTimestampScope` owns cloned WGPU query-set handles plus pre-reserved indices. Therefore
  scopes can travel with an already prepared graph pass into the existing parallel command
  encoder buckets. Pipeline-statistics recording remains serial until it has its own equivalent
  pre-reservation contract. Timestamp observation must not itself change the graph topology,
  bucket partitioning, or command-buffer order.

Reference cross-check: Bevy's `crates/bevy_render/src/diagnostic/internal.rs` maintains a
current/submitted/finished frame lifecycle and publishes results only after asynchronous mapping;
Unreal's Lumen `Res/Shader/ScreenProbeGather/TraceVoxels.hlsl` keeps tracing bounded through
fixed page/object records instead of coupling diagnostics to an alternate rendering algorithm.
Zircon deliberately adopts the former's delayed-observation principle and the latter's
fixed-structure principle, while retaining the existing three-slot `GpuReadbackQueue` and its
current timestamp feature gate.

Focused regressions cover: capability-unavailable projection, readback-deferred projection,
capacity saturation with partial timings marked non-comparable, late-result transition to
`measured`, legacy profile JSON defaults, and source-level preservation of timestamp scopes in
the existing parallel recording path. A managed Windows product run remains required to prove
that the command-buffer ordering, image output, and timing samples are actually comparable.

## Measurement Protocol

The coordinator must run this protocol on a Windows product backend after the observation repair.
All generated artifacts remain outside `C:`.

1. Record adapter name, backend, driver version, enabled WGPU features, resolution, render scale,
   scene identifier, git/source fingerprint, and the exact capture command.
2. Use one fixed 1080p scene with deterministic camera motion. Run cold and warm conditions
   separately, with 60 warm-up frames discarded and at least 300 measured frames in each sample.
3. Export per-frame `RenderFrameProfile` data including GPU-status outcome, pass timings, CPU
   submit time, draw/dispatch/upload counts, parallel recording counters, memory fields, and
   fallback/degradation state. Report median and p95 only after preserving raw per-frame values.
4. Capture a product PNG under `docs/tests/runtime/render/` and a matching RenderDoc RDC using
   `D:\Tools\renderdoc`. The frame profile, graph dump, PNG, and RDC must share the same frame
   generation and source fingerprint.
5. Compare timestamp-disabled and timestamp-enabled runs with the same parallel-recording policy.
   A difference in pass order, command-buffer order, output pixels, fallback counters, or quality
   settings invalidates the comparison.
6. Power is optional evidence. If a platform-specific power source is available, record its sensor,
   sampling interval, and synchronization method. GPU timestamps, utilization, or frame time are
   not substitutes for power measurement; without this telemetry, no power comparison is reported.

## Decision Gates

| Gate | Required evidence | Permitted next action |
| --- | --- | --- |
| Instrumentation correctness | Timestamp capability tier, profile status branches, frame-generation match, and nonblocking readback source tests. | Request coordinator WGPU run. |
| Product observability | Current-source product PNG, matching profile, graph dump, GPU timing result, and RDC. | Rank actual pass costs. |
| Algorithm proposal | Cold/warm raw samples and equal-output comparison identify one dominant bounded cost. | Implement one scoped algorithm change. |
| Algorithm acceptance | Same-scene before/after profiles, matching pixels/fallback state, and a fresh RDC. | Record measured improvement only. |

Until the product-observability gate is complete, Global SDF, Radiance Cache, parallel command
recording, and quality ladders remain hypotheses. No static code path or historical capture is
accepted as a substitute for the requested runtime evidence.

## 2026-08-27 FrameProfiler Owner Follow-Up

Status: `runtime_17_15_frame_profiler_gpu_resolution_owner_split_static_passed_cargo_profile_deferred`.

The delayed-result resolution responsibility is now physically owned by
`frame_profiler/gpu_resolution.rs`: timer and pipeline-statistics merge, duplicate pass-occurrence
matching, subsystem GPU-time projection, and GPU budget warnings moved without changing their
implementation. The 796-line parent remains the current-frame assembly, bounded pending-ring, and
publication owner; the child is 153 lines. A normalized comparison passed for all seven moved items,
and the source/status guard, focused formatting, and diff checks pass.

This follow-up does not satisfy any measurement gate. It produced no Cargo, WGPU, RenderDoc, PNG,
timestamp, allocation, RSS, or power receipt, and it does not authorize tuning the ring, matching
algorithm, copy-on-write policy, budget model, command recording, or renderer quality.
