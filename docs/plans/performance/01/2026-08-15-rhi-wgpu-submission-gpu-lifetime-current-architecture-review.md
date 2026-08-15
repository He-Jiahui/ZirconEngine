---
related_code:
  - zircon_runtime/crates/zr_rhi/src
  - zircon_runtime/crates/zr_rhi_wgpu/src
  - zircon_runtime/src/graphics/backend/render_backend
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/scene_renderer/core
  - zircon_runtime/src/dynamic_api/session
  - zircon_app/src/entry/entry_runner/editor.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/optimize/zircon_runtime/index.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Private/RHICommandList.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Private/RHIResources.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderResource.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Allocation.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/D3D12RHI/Private/D3D12Submission.cpp
  - dev/bevy/crates/bevy_render/src/renderer/mod.rs
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
tests:
  - 76 of 76 current zr_rhi and zr_rhi_wgpu Rust files reconciled and reviewed
  - 23731 physical lines and 249 inline tests
  - path plus per-file SHA-256 manifest fingerprint 336dfe9df6fca33f03ef6f45ce11eb721ec3b8989f829781bbb6bf4ff7192a53
  - current managed Cargo, WPR/xperf, GPU timestamps, RenderDoc and energy remain blocked by the non-runnable product baseline
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# RHI/WGPU submission and GPU lifetime current architecture review (2026-08-15)

## Scope and source freeze

This review covers every current Rust file in `zr_rhi` and `zr_rhi_wgpu`, then follows the real
editor product path through device creation, frame recording, queue submission, presentation,
readback and diagnostics. The source freeze is **76/76 files, 23,731 physical lines and 249 tests**.
Its path plus per-file SHA-256 manifest fingerprint is
`336dfe9df6fca33f03ef6f45ce11eb721ec3b8989f829781bbb6bf4ff7192a53`.

| Module group | Files | Lines | Tests | Review method |
|---|---:|---:|---:|---|
| `zr_rhi` contract/core | 11 | 4,082 | 20 | current full source and public owner surface read |
| `zr_rhi` tests | 4 | 715 | 15 | current behavior/source-shape coverage read |
| `zr_rhi_wgpu` contract/core/UI | 32 | 12,353 | 124 | prior full reports reconciled; all current changed/new files reread |
| `zr_rhi_wgpu/gpu_readback_queue` | 5 | 1,248 | 16 | current queue, ring, ticket and tests read |
| `zr_rhi_wgpu` tests | 24 | 5,333 | 74 | current contract/lifecycle/command/UI coverage read |

The five earlier RHI reports remain valid for unchanged files. This pass reconciled their manifests
against the current 76-file tree and reread all 18 modified plus four new files, including the GPU
timer/readback changes and `ui_surface/{presentation,shared_image_registry,image_cache/resource}`.
This is static review completion, not acceptance. Current Cargo has not executed these 249 tests and
there is no valid current-source editor product for WPR, xperf, GPU timestamps, RenderDoc or energy
capture. The module must remain pending; `review.md` must not be advanced.

## Corrections and useful work to retain

- `zr_rhi::RenderDevice` provides a backend-neutral descriptor and command contract with explicit
  create/destroy/submit/readback operations and deterministic validation.
- The current shared `GpuReadbackQueue` is bounded to three staging slots, packs requests at the
  required 256-byte alignment, grows by powers of two, shrinks only after 240 low-utilization frames
  and does not itself call `wait_indefinitely`.
- GPU timing is opt-in and bounded. The pass timer caps one frame at 64 timed passes; recent status
  values distinguish pending, deferred, capacity exhaustion and empty frames instead of silently
  presenting missing samples as zero.
- The current UI presenter acquires its surface before doing CPU preparation and encodes one UI
  frame into one encoder/submit. Retained geometry and a shared image registry remove some
  presenter-local duplicate texture ownership.
- The compiled-scene main submission batches graph command buffers and optional IBL writeback into
  one call. That useful batching behavior should become the default RHI batch contract.

These repairs do not resolve the product ownership split below.

## P0 architecture findings

### 1. The public RHI and the product GPU backend are different systems

`zr_rhi/src/device.rs:629` defines `RenderDevice`, but the only implementation is
`DeterministicRhiContractDevice` in `zr_rhi_wgpu/src/device.rs:29-550`. It is a CPU test double:
resources are descriptor/data entries in `Arc<Mutex<...>>` maps, submit validates and executes the
command list on CPU, completion is immediate, and readback clones CPU bytes. Repository consumers
of this implementation are contract tests and validation helpers.

The product `RenderBackend` instead owns `wgpu::Instance`, `Adapter`, `Device` and `Queue` directly,
requests the WGPU device directly, and exposes direct device/queue access to renderer, resource,
surface, UI and readback code. Therefore the engine has two incompatible truths:

1. a neutral RHI whose lifetime/fence semantics are exercised only by deterministic tests; and
2. a production WGPU graph with no common RHI lifetime, submission or device-generation authority.

Optimizing the deterministic implementation or adding another adapter would not improve the MVP
product and would prolong the split. The hard cut must either provide a real product-used
`WgpuRhiDevice` behind the neutral contract or delete/replace the dead public contract in the same
milestone. The deterministic device may remain only as an explicitly test-only implementation.

### 2. The default editor uses the synchronous submission path

`RuntimeProfile::pipelined_render` returns true only for `RuntimePipelined`. The editor is created
with the `editor` profile, and session construction writes the pipelined configuration only when the
profile requests it. `RenderSubmissionConfig` otherwise defaults to synchronous execution.

This means the MVP editor does not normally exercise the optional render submission worker. Its
caller records and executes the submission inline while holding the framework operation authority.
The problem is architectural rather than a missing Boolean default: switching the flag would select
a second execution model instead of fixing one product submission contract.

### 3. The optional submission worker is a one-slot handoff, not a render/RHI pipeline

`graphics/runtime/render_framework/pipelined/queue.rs` creates one private OS thread named
`zircon-render-submit` and a one-slot bounded channel. The producer waits for frame N-1 completion,
sends N, then waits for a worker-start acknowledgement. The worker holds `core.lock_operation()`
over the full submission. At most one frame can overlap and both backpressure and completion are
expressed through bespoke waits outside the shared TaskGraph.

Parallel graph recording does not compensate for this. `execute_graph_stage.rs:223-228` searches
the full pass list by pass name for each stage entry. Parallel eligibility is globally disabled when
pipeline statistics or mutable mesh/UI/IBL owners are present. The current product render graph also
authors a total pass order, so typical ready width and profitable bucket count are one.

### 4. One presented scene frame has a static minimum of two GPU submissions

`submit_compiled_scene_frame.rs:35-88` submits the graph command buffers once. The rendered image is
offscreen; `render_frame_with_pipeline.rs:315-330` then calls `ViewportSurface::present_texture`.
`viewport_surface.rs:231-278` creates a new present bind group and encoder and calls
`queue.submit` again for the blit. Thus the normal presented scene path has a source-proven minimum
of **two GPU submissions per frame** before optional work:

1. scene/graph submission;
2. surface blit/present submission.

Output-target writeback can create a third independent encoder/submit in
`resource_streamer_execute_output_target_writeback.rs:43-69`. Texture upload, scene-clear,
environment and UI authorities contain further direct queue calls. This is a source-path count, not
a measured driver submission count; RenderDoc/GPU capture is still required.

### 5. Product readbacks still contain indefinite caller stalls

The bounded shared queue is not the only readback path. `read_buffer_bytes.rs:18-64` allocates a
staging buffer and encoder, submits, calls `device.poll(wait_indefinitely())`, receives the map
result and copies it into a new `Vec`. RGBA/16f helpers repeat the same shape. The production IBL
artifact path also submits and waits indefinitely in
`read_ibl_bake_artifact_sections.rs:310-322`.

These helpers bypass shared admission, in-flight budgeting and the frame batch. Any main/render
caller pays GPU completion latency and allocation/copy latency synchronously. They must be replaced
by a typed asynchronous readback ticket whose copy is encoded into the owning submission batch.

### 6. Polling and diagnostics have multiple owners and repeated per-pass work

The three-slot readback queue polls the device and scans its slots. GPU pass timing and pipeline
statistics each call completion polling, so enabling both can duplicate device polls and scans in a
frame. Pass names are owned `String` values; timing clones one per recorded pass, and pipeline
statistics aggregate duplicate names with a linear search. Enabling statistics also disables the
entire parallel recording path rather than only the affected scope.

There is no single diagnostics generation connecting dense compiled pass slots, timestamp/statistic
query ranges, submission tickets and one per-frame poll. Consequently instrumentation changes
scheduling behavior and adds string/allocation work to the path it is meant to observe.

### 7. GPU resource retirement and device loss have no product-wide authority

Production WGPU resources mainly rely on Rust `Drop`. There is no generation-tagged registry,
last-use submission ticket, deferred-destruction queue, memory budget owner or proof that a resource
cannot be destroyed before its last GPU use. Surface `Lost`/`Outdated` is locally reconfigured, but
there is no product-wide device generation, uncaptured-error owner, admission stop, in-flight ticket
failure or rebuild/terminal policy.

The new shared UI image registry is useful deduplication but remains another direct device/queue
owner. Its mutex covers hash-map lookup plus texture creation and `queue.write_texture`; admission
computes total bytes and sorts candidates only after exceeding 256 entries or 64 MiB. It should
become a client of the single GPU resource registry, not a second lifetime domain.

## Reference-engine evidence

### Unreal RHI command translation and submission

- `RHICommandList.cpp:775-848` separates parallel command-list dispatch, translation and submit
  tasks. `:970-1058` closes translation chains and starts submit work from dependencies rather than
  waiting for a private worker-start acknowledgement.
- `RHICommandList.cpp:1306-1345` coalesces finalized platform command lists and waits on translation
  completion before one submission stage.
- `RHICommandList.h:4387-4394` makes submit/delete operations explicit. `:4482-4499` orders async
  command lists by prerequisites, while an immediate flush may wait for the RHI thread without
  implying a GPU completion wait. `:5141-5297` carries completion events and dedicated task pipes.

The transferable principle is one dependency-aware translation/submission authority with explicit
completion, not a one-slot private thread or many direct queue owners.

### Unreal resource retirement and GPU failure handling

- `RHIResources.cpp:12-102` queues pending resource deletes and dispatches lifetime release to the
  RHI thread.
- `RenderResource.cpp:175-230,290-377` centralizes Init/Release/Update RHI and batches release work
  before enqueueing it to the render command pipe.
- `D3D12Allocation.cpp:555-581,622-649` records the next-frame fence on retired blocks and reclaims
  them only after fence completion.
- `D3D12Submission.cpp:1094-1179` checks device removal, records pending interrupt/fence completion,
  uses completion events and detects timeouts/hangs.

This evidence supports explicit retirement and device-failure state. It does **not** prove that
Unreal transparently recreates every lost device; Zircon must choose and test either a terminal
policy or a new device generation instead of claiming unsupported parity.

### Bevy single-encoder readback integration

`bevy_render/src/renderer/mod.rs:79-108` runs the graph, appends screenshot/readback copy commands to
the same encoder, submits once, and then presents. `gpu_readback.rs:367-415` maps asynchronously
after that submission. This is directly relevant to Zircon's extra output/readback submits and
indefinite polling, while remaining simpler than Unreal's multithreaded RHI task graph.

## Required hard-cut architecture

The implementation order is dependency-sensitive; no compatibility adapter may keep the old direct
queue paths alive after their replacement.

1. **`RhiDeviceGeneration`** owns instance/adapter/device/queue/capabilities/surfaces and the sole
   `Active -> Lost -> Retiring` state. It is the only product device-poll and queue-submit authority.
2. **`RhiCommandPacket`** is the immutable output of the compiled render graph: dense pass/resource
   slots, command buffers, upload/readback ranges, diagnostics scopes and surface presents. It uses
   no pass-name lookup or per-pass `String` allocation.
3. **`RhiSubmissionBatch` / `RhiSubmissionTicket`** provides monotonic submission identity,
   in-flight bounds, typed split reasons, backpressure and completion. Normal scene, surface blit,
   output writeback and readback copies are coalesced before one physical submit per queue/frame.
4. **RHI affinity executor** runs on the shared Runtime11 TaskGraph. The private
   `zircon-render-submit` thread and synchronous-vs-pipelined semantic fork are deleted in the same
   milestone.
5. **`GpuResourceRegistry`** provides generation-tagged handles, last-use ticket, deferred
   destruction, transient/persistent memory budgets and cached surface bindings. Retirement work is
   proportional to the completed frontier plus retired resources, not all live resources.
6. **`GpuReadbackTicket`** admits through a byte/count/age-bounded staging ring. Copies join the
   frame batch; map completion never waits on main/render/RHI, and callbacks are forwarded outside
   the RHI owner.
7. **`GpuDiagnosticsGeneration`** owns timestamp/statistic slot allocation and one poll/collection
   pass per frame. It publishes pooled/dense result rows and must not disable unrelated parallel
   recording.
8. **Device failure policy** stops admission, fails all in-flight tickets, collects artifacts and
   then either terminates explicitly or creates a new `RhiDeviceGeneration`. Old-generation handles
   can never resolve against the new device.

At the end of this milestone, product code above the physical backend contains zero direct
`wgpu::Device`, `wgpu::Queue`, `queue.submit`, `device.poll` and GPU-resource creation authorities.
The deterministic RHI device is clearly test-only; there is no dual public contract.

## Complexity and performance acceptance

| Dimension | Required samples | Hard acceptance |
|---|---|---|
| graph/submission scale | 1/32/256/1k passes; 1/2/8 queues or surfaces | packet build near `O(commands + resource uses)`; no pass-name scans; normal presented viewport <=1 physical graphics submit per queue/frame |
| frames in flight | 1/2/3 | bounded tickets; main/editor wait on GPU/readback/worker-start = 0; no unbounded queue age |
| resource lifetime | 0/1/64/1k created/retired resources | stable create/destroy = 0; destroyed-before-completion = 0; deferred queue bounded and drains after completed frontier |
| readback | 0/1/64/1k requests and 1 B to 64 MiB | bounded admission/bytes/age/drop; product `wait_indefinitely` = 0; copy encoded into owning batch |
| diagnostics | off/timestamps/statistics/both | device poll owners = 1/frame; pass-name allocation = 0; unrelated parallel recording remains eligible |
| lifecycle | stable/resize/reload/surface loss/device loss | one generation transition; stale handles rejected; all in-flight tickets receive typed terminal; no UAF/deadlock |

Dynamic acceptance must use one current-source product fingerprint and run ID:

- WPR/xperf: main/render/RHI ReadyThread, waits, context switches, queue depth/age, CPU samples,
  allocations, RSS and idle wakeups;
- GPU timestamps: per-pass and submission-boundary p50/p95/p99 after warmup;
- RenderDoc CLI: submit count, passes, copies, barriers, resource create/destroy and present sequence;
- energy trace: F2 300-frame workload and F4 30-second idle/authoring sequence, at least three runs;
- functional gates: resize, multiple views, screenshots/readback, hot reload, surface loss and the
  chosen device-loss terminal/recreation policy.

No timing, power or "close to Unreal" claim is made here. The current product bundle is blocked
before Cargo by the approved-root separator defect recorded in
`failure-2026-08-15-build-editor-approved-root-separator.md`; stale binaries and source-model counts
are not acceptable substitutes.

## Implementation decision for this review

No production source was edited. A local extra-submit removal, poll deduplication or image-registry
lock reduction would preserve the wrong device/queue owner and conflict with currently leased UI and
render files. The smallest correct change is the cross-module hard cut above, after Plan02 M3's RDG
packet and Runtime11's shared affinity executor exist. The exact owner routing and protected-plan
write blocker are recorded in `2026-08-15-rhi-wgpu-protected-plan-routing.md`.
