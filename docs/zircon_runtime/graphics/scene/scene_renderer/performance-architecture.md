---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/frame_command_encoder_set.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/parallel_encoder_set.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame/capture_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs
  - zircon_runtime/src/graphics/backend/render_backend/renderdoc_capture_file_path.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_pass_timer.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_materialization.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs
  - zircon_runtime/src/graphics/backend/render_backend/renderdoc_capture_file_path.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_pass_timer.rs
plan_sources:
  - user: 2026-08-09 investigate renderer architecture and profile before optimization
  - docs/plans/zircon_runtime/render/16-compute-neural.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/17/failure-2026-07-29-gpu-timestamp-feature-set-const.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs::tests::render_perf_gpu_timer_latency_within_three_frames
  - zircon_runtime/src/graphics/tests/render_perf_baseline.rs::render_perf_parallel_recording_product_path_preserves_topology_and_pixels
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_pass_timer.rs::tests::render_perf_gpu_timer_capability_gate
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs::tests::render_perf_parallel_recording_is_wired_to_product_stage_and_single_submit_owner
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame/capture_frame.rs::tests::async_viewport_poll_never_finishes_submission_or_waits_for_the_device
doc_type: workflow-detail
---

# Scene Renderer Performance Architecture

## Decision

This is the required pre-optimization architecture review for the WGPU scene renderer. It is not a milestone record, validation result, or performance claim. No optimization may be accepted from static source evidence alone: the managed Windows baseline defined below must identify the dominant CPU, GPU, memory, or synchronization cost first.

The ownership boundary is fixed:

- `render_graph` compiles resource lifetime, culling, dependencies, and transient allocation plans.
- `scene_renderer::graph_execution` materializes graph-owned resources and records pass work.
- `scene_renderer_core_render_compiled_scene::render` owns frame preparation, binding, execution, readback scheduling, and the single submission handoff.
- `render_framework` owns viewport lifetime, submission state, product publication, and explicit capture admission.
- `zr_rhi_wgpu` owns WGPU timestamp and readback lifecycle. Framework contracts carry only `RenderFrameProfile` data.

This keeps Plan 16's `GpuReadbackQueue` as the only ordinary-frame mapping owner. Product presentation must remain GPU resident; synchronous CPU RGBA is only an explicit screenshot, inspection, or pixel-test path.

`RenderPassId` is the graph identity; `pass_name` is a diagnostic label and is not globally unique. While the current timestamp path records graph passes serially, deferred timer and pipeline-statistics results consume one unmatched profile entry for each same-name result in emission order. A future timer that records parallel graph passes must carry an explicit pass identity rather than treating a name as a key.

## Current Architecture Review

`render_compiled_scene` already has the F16-required split into resource binding, graph-stage execution, and submission owners. Its remaining coordinator function creates frame-local preparation state, materializes live graph resources once, records the stage sequence, queues bounded readbacks, and passes finished buffers to one submit owner. It must not absorb per-pass behavior again.

The most material current risks are structural rather than shader micro-optimizations:

| Area | Static evidence | Consequence to measure |
| --- | --- | --- |
| Frame-state critical section | `submit_runtime_frame` retains the render-framework state guard through prepare, render, product publish, feedback, and stats. | Serializes viewport work and makes lock contention a first-class CPU metric. |
| Pass recording parallelism | `execute_graph_stage` only enters `ParallelEncoderSet` when timers, pipeline statistics, mutable owners, and non-parallel-safe executors are absent. Product frames with mesh lists or profiling often stay serial. `RenderFrameProfile` projects each graph record's `cpu_elapsed_micros` plus eligible and executed stage/bucket counts. | A topology bucket count alone is not proof of concurrent recording. Compare `parallel_recording_eligible_*` with `parallel_recording_executed_*` and the same capture's per-pass CPU encode time. |
| Transient realization | `transient_materialization` realizes compatible allocation slots per frame; `TransientResourcePool` reuses only exact descriptor keys across frames. | Separate allocation, reuse, retained bytes, and alias density before changing pool policy. |
| Product/capture separation | Direct presentation publishes `latest_viewport_texture`; asynchronous capture uses the shared readback queue. `capture_frame` still intentionally waits when an explicit caller requests a synchronous inspection fallback. | Normal present must show zero full-frame CPU copies; capture latency and dropped generations are separate metrics. |
| GPU timing mode | `GpuPassTimer` currently emits `CommandEncoder::write_timestamp` around generic graph executor work, so it correctly requires both `TIMESTAMP_QUERY` and `TIMESTAMP_QUERY_INSIDE_ENCODERS`. The current Plan 17 feature-set Failure owns that all-or-nothing contract. The original pass-boundary design would require wiring timestamp writes into each actual render or compute pass and formally superseding the Failure first. | Record the adapter's supported mode. Do not loosen the feature gate without changing the instrumentation boundary; otherwise GPU timings become invalid or unsupported. |

The review finding F3 remains applicable as a measurement target: track full-frame bytes and extract copies across the runtime-to-editor boundary. Runtime extraction now reports `extract.full_clones` and `extract.full_clone_bytes` for both cache population and reuse; cache hits must not be assumed to avoid deep copies. F16 is structurally addressed by the existing folder owners, so further changes must preserve that split rather than re-centralize the renderer.

The deterministic product regression fixture has one serial and one profitable two-bucket case. It requires the serial report to remain empty and the parallel case to report one eligible/executed stage with two eligible/executed buckets, while preserving graph order and pixels. This confirms the counter's semantic route; it is not a WGPU timing or throughput result.

## Open Failure Routing

The following records are implementation ownership and evidence gates, not permission to substitute static claims for measurement:

| Failure family | Reported current-source posture | Baseline and optimization rule |
| --- | --- | --- |
| Render16 product/capture separation | Scene code publishes a GPU texture for direct presentation and keeps CPU `ViewportFrame` capture explicit; the original synchronous-readback failure remains open pending product evidence. | Do not profile a headless `ViewportFrame` exporter as normal presentation. Measure direct texture presentation and keep screenshot fallback bytes separate. |
| Render17 current-source capture | Consecutive `ZR_RENDERDOC_CAPTURE_FRAME_COUNT=2` triggering and a cold/warm visual exporter exist, but their current-source PNG and `.rdc` are absent. The temporal full-chain exporter creates its evidence directory and configures a process-unique RenderDoc file template before WGPU initialization, then manually arms cold and settled-warm captures because a contiguous two-frame sequence would otherwise capture the history-transition frame. It rejects capture-stop errors, requires exactly two template-matching `.rdc` files, and writes a paired JSON manifest containing the two captured frame profiles plus explicitly-pending replay metrics. | The first managed capture must use one process, one viewport, the cold frame, and its stable successor; run that exporter with both automatic variables (`ZR_RENDERDOC_CAPTURE_FRAME_COUNT` and legacy `ZR_RENDERDOC_CAPTURE_NEXT`) unset, or use `ZR_RENDERDOC_CAPTURE_FRAME_COUNT=0` to explicitly override the legacy switch. Replay draw/dispatch/copy and GPU event durations remain `unavailable` until both emitted RDC files are audited. |
| RHI product-owner cutover | The deterministic CPU contract device is reported as test-only; graphics backend WGPU device/queue is the product owner. | Exclude contract-device wall-clock and host mirrors from product measurements. |
| Scene-surface and profile projections | The scene-to-framework surface handoff and `RenderGraphPassProfileMetrics` root projection are reported repaired. | Managed compile is a prerequisite before interpreting a product trace, because a pre-test compiler stop yields no render evidence. |
| GPU timer capability | The all-or-nothing encoder timestamp feature set, one facade, generation ordering, and shared readback owner are reported implemented. | Retain the existing feature contract; on unsupported adapters use RenderDoc timings and record the missing capability. |
| Native UI presenter | Generation-owned compiled projection, compact instance data, spatial batching, persistent uploads, and presentation counters are reported implemented across the UI failures. | Treat UI as a separate workload in the matrix; verify painter-order pixels, draw/pass count, CPU p50/p95, and current WGPU capture before claiming a reduction. |
| Deferred lighting buffer lifetime | The full-lighting optional buffer owner and an environment-only regression are reported repaired. | Keep this as a compile/lifetime gate; it is not a measured renderer bottleneck without a successful current-source run. |
| Render-framework state accessor | Callers are reported migrated to `lock_state()` after the state-owner hard cut. | Measure lock duration and contention through the canonical accessor; do not widen internal state to collect data. |

These records converge on a strict baseline order: current-source compile first, then direct-present cold/stable frame evidence, then CPU/GPU/memory/power scaling. Their open status delays accepted closeout only; independent architecture review, instrumentation design, and forward repair continue.

## Reference Evidence

Unreal Engine is the primary reference. `RenderGraphBuilder.h` binds resource lifetime, barriers, culling, queue choice, profiling name, and async-compute eligibility to pass parameters and flags. `RenderGraphBuilder.cpp` preallocates compile containers and starts parallel compile work only beyond a resource-count threshold. `LumenSceneGPUDrivenUpdate.cpp` creates typed pass parameters and emits ordinary compute passes through `FComputeShaderUtils::AddPass`; readback submission remains outside the compute implementation.

Unity Graphics is the pipeline-structure cross-check. Its `RenderGraph` separates recording, resource execution, intra-frame aliasing, and profiling scopes. Bevy is the Rust lifecycle cross-check: `pipelined_rendering.rs` uses bounded one-element ownership transfer for sim/render overlap, while `gpu_readback.rs` gives readback buffers an explicit pool and asynchronous completion lifecycle.

`dev/LumenInUE5.5.4WithComputeShader` is useful only as an algorithm study reference. Its `RenderPass::PreExecute/Execute/PostExecute` performs manual barriers and descriptor work. Zircon must not copy that ownership model because the render graph already owns dependency, lifetime, and materialization decisions.

The converged design direction is therefore: retain graph-declared resources and immutable pass descriptors as the scheduling input; make preparation data independently recordable before attempting broader parallelism; keep WGPU mapping, product publication, and screenshot capture as separate lifecycle paths.

## Measurement Gate

All measurements use coordinator-managed Windows validation. Do not launch Cargo, WGPU applications, or RenderDoc manually. RenderDoc is invoked from `D:\Tools\renderdoc`, and every image or `.rdc` artifact is written under `docs/tests/runtime/render/`, never under `C:`.

`render_product_post_process_full_chain::visual_export` is retained as a real-WGPU visual and capture-contract test, but its headless path intentionally resolves a `ViewportFrame` through explicit CPU capture. It is not a normal-present performance driver and must not be used to rank the product render path; the baseline driver must exercise direct GPU-resident presentation.

The direct-present driver is the existing `zircon_shader_pbr_viewer` native-surface path: its `render_to_viewport_surface` call renders the final texture through the WGPU surface blit and never constructs a `ViewportFrame`. A coordinator-managed direct capture uses the viewer's one-shot RenderDoc CLI flags with `D:\Tools\renderdoc\renderdoc.dll`, a process-unique template rooted in `docs/tests/runtime/render/`, and `--exit-after-capture`; it must not also request `--screenshot`, because that deliberately selects the CPU-readback evidence path. The full-chain exporter supplies the paired effect PNGs and JSON profile manifest, while the viewer capture supplies a real product surface `.rdc`; both are required before any direct-present performance statement.

The baseline matrix is:

| Case | Warm-up and sample | Required evidence | Primary decision signal |
| --- | --- | --- | --- |
| 1080p mid, one viewport | Warm 300 frames; sample frames 180-300 | `RenderFrameProfile`, WPR CPU trace, one RenderDoc capture, PNG, available power telemetry | CPU submit p50/p95, GPU pass total, lock wait, full-frame copy bytes, average/peak watts when exposed |
| 4K mid, one viewport | Same cadence | Profile, PNG, RenderDoc capture, available power telemetry | Resolution scaling of GPU time, transient peak, staging bytes, average/peak watts when exposed |
| 1080p mid, two viewports | Same cadence | Profile, WPR trace, PNG, available power telemetry | State-lock contention, encoder overlap, product texture lifetime, average/peak watts when exposed |
| Cold then warm pipeline | First frame and frame 300 | Profile plus graph/pipeline cache counters | Compile cost, variant misses, allocation churn |

For each case, preserve the adapter/backend/capability report, frame generation, pass names, CPU submit time, pass GPU microseconds, `profile_latency_frames`, transient created/reused/evicted counts and bytes, readback in-flight count, capture drops, command-buffer count, frame-age data, and available board-power or system-energy readings. The cold/warm manifest records the capture-frame profile separately from `resolved_gpu_frame_profile`, whose generation may lag because timestamps resolve asynchronously. If Windows or the installed driver exposes no trustworthy power provider, record that absence explicitly; do not infer watts from utilization. The existing 1080p-mid reference budget is 14,000 microseconds, but it is a target budget rather than current measured performance.

Before recording the matrix, record the actual adapter timestamp mode. The present generic graph scope may emit timings only on adapters with both required features. An adapter with base pass-descriptor timestamps but no encoder timestamps is a future instrumentation-design case, not grounds to relax the current gate. If the required mode is unavailable, use RenderDoc event timing and record the capability fact; never fill GPU values with zero.

The scaling check compares ratios, not unrelated-engine FPS: 4K has four times the 1080p pixel count, so fragment-dominated GPU work and transient bytes should be interpreted against that fourfold input change; two independent 1080p viewports should expose near-twofold per-view work while preserving shared shader and persistent-resource cache behavior. Unreal, Unity, and Bevy guide the expected ownership and scheduling shape, but do not supply a cross-engine numeric target because content, backends, and hardware differ.

## Optimization Order After Evidence

1. Correct measurement integrity and expose the missing deterministic counters. Preserve the existing all-or-nothing direct encoder mode required by the current Plan 17 Failure. A future pass-descriptor design requires timing sinks in every actual WGPU pass and a formal Failure supersession. This is instrumentation, not a throughput claim.
2. Remove the bottleneck with the largest measured share: product readback/copy path, framework lock duration, serial recording eligibility, graph materialization churn, or GPU pass cost.
3. Re-run the same matrix and compare deltas for CPU p50/p95, GPU total and per-pass time, bytes moved, allocation/reuse, resident memory, frame age, viewport scaling, and available power readings.
4. Keep only changes that preserve graph ordering, texture-generation lifetime, bounded readback semantics, and the Plan 16/17 contracts. Revert no integrated snapshot for an ordinary validation failure; repair forward through the owning plan.

The minimum acceptance evidence after an optimization is a managed PNG, `.rdc`, and paired profile manifest in `docs/tests/runtime/render/`, a profile comparison with absolute and percentage deltas, and a second independent review. Only then may the coordinator create the milestone commit and its quantified WeCom notification.

## Current Gaps

No managed current-source baseline, PNG, `.rdc`, or paired profile manifest is attached to this document. Consequently, no present bottleneck ranking, FPS result, power reading, or scalability claim is asserted here. The open Plan 16 readback Failure remains an integrated-forward-fix candidate pending managed compile and product evidence; it is not an accepted closeout.
