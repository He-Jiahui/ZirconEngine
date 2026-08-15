---
date: 2026-08-14
related_plan: docs/plans/zircon_runtime/render/19-gpu-capability-optimizations.md
doc_type: structural-performance-research
status: implementation_in_progress
coordination_owner: docs/plans/zircon_runtime/render/19
related_code:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs
  - tools/validate_render_measurement_evidence.py
---

# GC-M2 Bindless Production Profiling Protocol

## Purpose

GC-M2 already has the capability gate, fixed-capacity fallback-filled texture slab,
CPU material payload registry, and the `GpuPrimitiveData::material_payload_slot`
ABI. It does not yet have a GPU payload storage binding in the bindless material
table or a production group-2 bindless pipeline layout. This protocol decides whether that remaining wiring is
profitable on the supported adapter before it changes the render hot path.

This is a measurement protocol, not an accepted milestone record. No performance
claim, status transition, or product evidence is implied until all artifacts below
are produced by a coordinator-managed Windows run.

## Current Status

Static architecture and telemetry-path review completed on 2026-08-14. The
protocol now requires RenderDoc-to-counter calibration and main-mesh/shadow
attribution before interpreting material-bind results. Execution remains pending
the MVP M0.3 baseline and explicit UI12 release of the managed Cargo/GPU lane; no
rendering, performance, power, or acceptance result has been claimed.

The offline sidecar validator and its focused regression suite are complete. It
checks the decision-critical JSON invariants and real artifact file structure
without starting the renderer; it does not produce an image, an RDC capture, or
a performance result, so it does not advance the evidence status above.

## Design Boundary Reviewed

The current source has three separate ownership layers:

1. `scene_renderer/material/bindless_slab.rs` owns a fixed-size group-2
   texture/sampler binding array and fills every vacant slot with the fallback
   texture. Its capacity is negotiated from WGPU limits.
2. `scene_renderer/material/bindless_material_payload_registry.rs` owns stable
   material-resource-to-payload rows and exports only dirty rows.
3. `scene/gpu_scene` owns per-primitive data and its group-3 storage layout.
   The primitive row reserves `material_payload_slot`, but group 3 currently has
   no material-payload storage buffer.

The current material-bind telemetry is an end-of-frame aggregate: it includes
both main-mesh and shadow replay. That aggregate is a useful integrity total,
but it cannot attribute a bindless result when one domain still uses the
standard fallback. The measurement implementation must therefore expose a
main-mesh/shadow split (or an equivalently explicit pass-domain tag) in the
sidecar before any result is used for a product decision. The aggregate remains
required and must equal the two attributed values.

The standard path instead creates a per-draw group-2 bind group in
`build_mesh_draws/create_mesh_draw.rs`; `MeshDrawCommandReplayer` binds it on
each material transition. Directly replacing that layout would invalidate the
fallback path and couple a global material table to every mesh variant. The
production design must therefore keep two explicit variants:

- standard variant: existing group-2 uniform-plus-texture layout and per-draw
  binding;
- bindless variant: a dedicated group-2 layout with texture/sampler arrays at
  bindings 0 and 1 plus the GPUScene-owned read-only material-payload buffer at
  binding 2, selected only by the existing capability gate.

The WGPU layout must remain group-compatible with the rest of GPUScene. A bindless
variant may bind one shared group-2 table per pass, but it must not change the
group-3 layout or bind-group identity used by skinning and visible-instance-remap
variants. GPUScene owns the buffer lifecycle and dirty-range writes, while the
material table borrows that buffer only when it creates its group-2 binding. This mirrors
Unreal's persistent GPUScene data ownership while retaining Bevy's fallback-filled,
fixed-capacity array discipline.

## Hypotheses To Falsify

The change is not assumed to be an optimization. It can win only when material
bind transitions dominate mesh-pass CPU encoding or when a dense material workload
reduces driver-side binding overhead. It can lose on small scenes, on texture-array
limited devices, or when the extra payload storage load increases fragment cost.

The accepted implementation must demonstrate all of the following on each eligible
adapter:

- image parity between standard and bindless variants;
- no regression outside the measured noise envelope on the small-scene control;
- a measured reduction in group-2 material binds or CPU mesh-pass recording time
  in the material-diverse stress scene; and
- no new fallback, validation, or RenderDoc resource-hazard event.

If any condition fails, the capability gate remains disabled for that adapter and
the standard variant remains the product default. The fallback path is a normal
outcome, not a test skip.

## Workloads

All workloads use the same camera, resolution, frame pacing, quality tier, and
scene extract for the standard and bindless pair. They run at 1280x720, followed
by a 320x240 correctness-only capture.

| Workload | Scene shape | Primary question |
|---|---|---|
| `control_shared_material` | 256 visible mesh instances sharing one material and fallback-safe textures | Does the new storage/layout regress the ordinary low-transition path? |
| `stress_unique_materials` | 4,096 visible instances, at least 512 material resources and a mix of shared and distinct texture references | Does one pass-level group-2 table reduce material-state work at realistic slot pressure? |
| `stress_culled_materials` | Same material set with stable HZB-visible and HZB-culled partitions | Does GPUScene payload preparation preserve culling/indirect-count behavior? |

The stress workloads must not exceed the negotiated binding-array capacity. A run
with fewer than two usable array slots records `gate_ineligible` and produces only
standard-path baseline evidence.

## Measurement Method

1. Use the existing `RenderSubmissionConfig::with_gpu_timing()` path. Record 30
   warm-up frames, then 120 settled frames for each variant; no shader or graph
   compilation is allowed in the measured window.
2. Persist `RenderFrameProfile`, resolved GPU timing observations, graph cache
   counters, draw-call statistics, and material-bind transition counters in a
   JSON sidecar for every run. GPU timestamp observations are asynchronous; a
   frame with pending or unavailable timing is retained but excluded from latency
   aggregates and counted separately. The material-transition fields are
   `last_mesh_replay_material_bind_group_set_count` and
   `last_mesh_replay_material_bind_group_skip_count`; they classify only group-2
   mesh-material binds and include the shadow replay aggregate. Before a
   decision run, the sidecar also records their main-mesh and shadow-replay
   contributions. The aggregate must equal that split; until it does, it is an
   observability prerequisite rather than performance evidence.
3. Replay one cold and one settled warm frame for each variant through
    `D:\\Tools\\renderdoc\\renderdoccmd.exe`. The RDC inspection records mesh-pass
    draw/API event counts, group-2 binding changes, the bindless payload-buffer
    identity at group-2 binding 2, and the unchanged group-3 resource identity.
    A capture error is a failed experiment, not missing metadata.
4. Calibrate the material-bind telemetry on every captured settled frame. The
   JSON sidecar's `last_mesh_replay_material_bind_group_set_count` must exactly
   match the manually exported RenderDoc count of group-2 descriptor/table-bind
   events corresponding to WGPU `set_bind_group(2, ...)` calls issued by the
   mesh and shadow replay paths for that same frame. RenderDoc exposes the
   backend-native Vulkan/D3D12 event rather than the WGPU method name. The
   matching scope deliberately excludes non-mesh group-2 users. Any mismatch
   makes the experiment invalid: the raw counters and event list are retained,
   but neither may be used to justify a hot-path change.
5. Collect board power and GPU-utilization samples during the same settled window
    using the vendor telemetry available on the test machine. Record sampling
    interval, adapter identity, driver version, and whether the power source is
    AC. If no telemetry source is available, record `power_unavailable` rather
    than inferring power from frame time.
6. Export a standard and bindless PNG for each workload at 320x240. Compare
   decoded RGBA pixels. Exact equality is required for opaque/control scenes;
   transparent workloads may use the established renderer tolerance only after
   the test records the reason and maximum error.

The measurement runner is coordinator-managed because it owns the sole Cargo/GPU
lane. The current session must not start Cargo until UI12 explicitly releases the
lane.

## Environment Preflight

The protocol was preflighted on 2026-08-14 without launching the renderer:

- GPU: NVIDIA GeForce RTX 3060 Laptop GPU, driver `591.86`;
- RenderDoc CLI: `D:\\Tools\\renderdoc\\renderdoccmd.exe`, with capture and replay
  commands available;
- power telemetry: `nvidia-smi.exe` is available;
- observed idle snapshot: `25.24 W`, `1%` GPU utilization.

The idle snapshot is only proof that telemetry can be queried. It is not a
baseline and must not be compared with GC-M2 results. The managed run records
the actual adapter selected by WGPU so an integrated/virtual display adapter
cannot be mistaken for the measured NVIDIA device.

## Decision Rule

For each metric, calculate median, p95, and median absolute deviation (MAD) over
the settled valid observations. The standard-vs-bindless difference is meaningful
only when it exceeds `max(2%, 2 * MAD / median)` for that workload; otherwise it
is reported as noise-bound. This makes the non-regression threshold relative to
measured run noise instead of an ungrounded fixed target.

The bindless variant can advance to product-default consideration only when:

- `control_shared_material` is not worse outside its noise envelope for GPU frame
  time, CPU mesh-pass encoding time, or board power;
- `stress_unique_materials` shows a meaningful improvement in either CPU mesh-pass
   encoding time or **main-mesh** group-2 bind transitions without a meaningful
   GPU-time/power regression; the shadow-inclusive aggregate is reported as a
   guardrail but cannot alone establish this condition;
- the image comparison and RenderDoc audit pass for every eligible workload; and
- the same adapter/driver/configuration is written with the measurements.

Otherwise, retain runtime capability gating and record the observed break-even
material diversity. No claim of algorithmic optimality is valid until this data
exists; this protocol measures the actual break-even rather than assuming one.

## Required Evidence Paths

The coordinator writes all experimental artifacts outside `C:`:

- `docs/tests/runtime/render/plan19_gcm2_bindless_<workload>_<variant>_*.png`
- `docs/tests/runtime/render/plan19_gcm2_bindless_<workload>_<variant>_*.json`
- `docs/tests/runtime/render/plan19_gcm2_bindless_<workload>_<variant>_*.rdc`
- `docs/tests/runtime/render/plan19_gcm2_bindless_summary_*.json`

The summary must identify the source revision, session/validation ticket, adapter,
driver, WGPU feature/limit gate, slot capacity, all exclusion counts, PNG paths,
RDC paths, and the decision-rule result. A screenshot without its matching
sidecar/RDC is insufficient evidence.

## JSON Sidecar Contract

Each workload/variant run writes one JSON sidecar. Its schema is intentionally
small but complete enough to reproduce a comparison without relying on a log
parser:

```text
source: revision, source_fingerprint, session_id, validation_ticket
adapter: name, backend, driver, requested_features, limits, bindless_gate, slot_capacity
workload: name, variant, resolution, quality_profile, camera_fingerprint, warmup_frames, sampled_frames
observations: valid_frame_count, excluded_pending_timing_count, excluded_unavailable_timing_count,
              cpu_mesh_encode_ns{median,p95,mad}, gpu_frame_ns{median,p95,mad},
              board_power_w{median,p95,mad}|power_unavailable
material_binds: aggregate_set_count, aggregate_skip_count,
                main_mesh{set_count,skip_count}, shadow{set_count,skip_count}
calibration: captured_frame, renderdoc_group2_event_count, counter_set_count, matched
artifacts: png_path, png_pixel_comparison, rdc_cold_path, rdc_warm_path, graph_dump_path
decision: noise_threshold, control_result, stress_result, accepted_for_default, rationale
```

`aggregate_set_count` and `aggregate_skip_count` must equal the saturating sum
of their `main_mesh` and `shadow` fields. `matched` is false when the captured
RenderDoc event count does not match the scoped counter; such a run is retained
for diagnosis but excluded from a default-gate decision. A power field may be
`power_unavailable` only with its telemetry probe result and sampling interval.

The executable contract is `tools/validate_render_measurement_evidence.py`. It
requires the top-level `schema` value
`zircon_render_measurement_evidence_v1` and rejects unrecognized fields. JSON
types are fixed as follows:

```text
workload.resolution: {width: positive integer, height: positive integer}
workload.warmup_frames, workload.sampled_frames: exactly 30 and 120
observations.power_telemetry: {probe: available|unavailable,
                               sampling_interval_ms: positive integer,
                               ac_power: boolean}
statistics: {median: non-negative number, p95: non-negative number,
             mad: non-negative number}
artifacts.png_pixel_comparison: {passed: boolean,
                                 max_channel_error: non-negative integer,
                                 reason: non-empty string}
```

All artifact paths are relative to `docs/tests/runtime/render`; running the
validator with `--require-artifacts` verifies that the PNG, cold/warm RDC, and
graph dump are non-empty files below that root, and rejects a non-PNG payload.
The validator cannot establish renderer provenance on its own. The source
revision/fingerprint, coordinator ticket, capture-time RenderDoc review, and
actual image inspection remain mandatory evidence.

`accepted_for_default=true` is valid only for an eligible bindless run with
measured board power, a matching RenderDoc calibration, a passing pixel
comparison, `control_result=not_worse`, `stress_result=improved`, and a
nonzero main-mesh bind count. This deliberately prevents an aggregate-only
shadow result or an unavailable power probe from changing the product default.

## Implementation Order After Evidence

Only after the baseline run is captured and the material-diverse workload confirms
the stated bottleneck:

1. Add a GPUScene-owned material-payload storage buffer with dirty-range upload
   and a fallback row at index zero; expose it only as binding 2 of the dedicated
   bindless group-2 table and keep its ABI in a folder-backed GPUScene leaf module.
2. Add the dedicated bindless group-2 layout and corresponding mesh pipeline-layout
   family. Keep the standard group-2 and all group-3 layouts/pipeline keys intact.
3. Feed registry dirty rows and primitive payload slots during mesh preparation;
   bind the shared table only on bindless commands.
4. Add product parity coverage, then repeat this protocol before changing the
   capability gate default.

Each step preserves the capability-gated fallback and follows the repository's
module-boundary rule: no new graphics facade re-export and no mixed material/GPUScene
implementation file.
