---
related_code:
  - zircon_runtime/crates/zr_rhi/src/capabilities.rs
  - zircon_runtime/crates/zr_rhi/src/descriptors.rs
  - zircon_runtime/crates/zr_rhi/src/descriptors/pipeline.rs
  - zircon_runtime/crates/zr_rhi/src/device.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_device.rs
  - zircon_runtime/crates/zr_rhi/src/device/handles.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_pass.rs
  - zircon_runtime/crates/zr_rhi/src/texture_copy/aspect.rs
  - zircon_runtime/crates/zr_rhi/src/texture_copy/region.rs
  - zircon_runtime/crates/zr_rhi/src/texture_view.rs
  - zircon_runtime/crates/zr_rhi/src/device_profile.rs
  - zircon_runtime/crates/zr_rhi/src/diagnostic_readback.rs
  - zircon_runtime/crates/zr_rhi/src/lib.rs
  - zircon_runtime/crates/zr_rhi/src/memory.rs
  - zircon_runtime/crates/zr_rhi/src/submission.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/capabilities.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/bind_group_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/command_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/command_validation/render_state.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/construction.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/contract_caps.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/resources.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/state.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/surfaces.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/uploads.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/views.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device/capabilities.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/rhi_wgpu_lock_poison.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/pipeline_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/render_pass_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/resource_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/texture_copy.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/texture_view.rs
  - zircon_runtime/crates/zr_rhi/src/tests/boundary.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/resource_lifecycle.rs
implementation_files:
  - zircon_runtime/crates/zr_rhi/src/capabilities.rs
  - zircon_runtime/crates/zr_rhi/src/descriptors.rs
  - zircon_runtime/crates/zr_rhi/src/descriptors/pipeline.rs
  - zircon_runtime/crates/zr_rhi/src/device.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_device.rs
  - zircon_runtime/crates/zr_rhi/src/device/handles.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_pass.rs
  - zircon_runtime/crates/zr_rhi/src/texture_copy/aspect.rs
  - zircon_runtime/crates/zr_rhi/src/texture_copy/region.rs
  - zircon_runtime/crates/zr_rhi/src/device_profile.rs
  - zircon_runtime/crates/zr_rhi/src/diagnostic_readback.rs
  - zircon_runtime/crates/zr_rhi/src/lib.rs
  - zircon_runtime/crates/zr_rhi/src/memory.rs
  - zircon_runtime/crates/zr_rhi/src/submission.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/capabilities.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/bind_group_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/command_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/command_validation/render_state.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/construction.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/contract_caps.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/resources.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/state.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/surfaces.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/uploads.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device/views.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device/capabilities.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/rhi_wgpu_lock_poison.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/pipeline_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/render_pass_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/resource_validation.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/texture_copy.rs
  - zircon_runtime/crates/zr_rhi/src/tests/descriptors.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/command_list.rs
  - zircon_runtime/crates/zr_rhi/src/tests/boundary.rs
  - zircon_runtime/crates/zr_rhi/src/tests/diagnostic_readback.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/debug_markers.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/debug_status.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_command_list.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_clear_values.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_resolve.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_state.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_views.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/resource_lifecycle.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract/texture_views.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/pipeline.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/texture_copy.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/tests.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/capabilities.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/device_ownership.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/texture_views.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics/tests.rs
plan_sources:
  - user: 2026-06-02 implement ZirconEngine WGPU render main-chain closure plan
  - .codex/plans/ZirconEngine WGPU 渲染主链闭环计划.md
  - .codex/plans/Zircon SRPRHI 渲染管线补全计划.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/crates/zr_rhi/src/tests/descriptors.rs
  - zircon_runtime/crates/zr_rhi/src/tests/capabilities.rs
  - zircon_runtime/crates/zr_rhi/src/tests/boundary.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/capabilities.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/command_list.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/command_list/basic_commands.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/command_list/bind_groups.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/command_list/raster_draws.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/command_list/vertex_index_state.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/debug_markers.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/debug_status.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/resource_lifecycle.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_device_handles.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/rhi_wgpu_lock_poison.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_command_list.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_clear_values.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_resolve.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_state.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/render_pass_views.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/device_contract.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/pipeline.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/texture_copy.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/tests.rs
  - cargo +1.94.1 test -p zr_rhi --lib --locked --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zr_rhi_wgpu --lib --locked --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib runtime_15_rhi_device_handles_are_child_owner --locked --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib runtime_15_rhi_wgpu --locked --jobs 1 -- --test-threads=1
doc_type: module-detail
---

# RHI Descriptors

## Current Physical Ownership

Since the 2026-08-03 Frameworks01 hard cut, `zircon_runtime/crates/zr_rhi` is the only owner of
backend-neutral RHI declarations. Runtime always compiles this lightweight contract dependency
because the always-on `RenderFramework` trait names its native surface target;
`zircon_runtime/crates/zr_rhi_wgpu` remains graphics-optional, depends on `zr_rhi`, and owns the
deterministic WGPU contract backend, validation modules, GPU timer/readback and concrete UI
presenter. `zircon_runtime/src/rhi.rs` is a curated public facade, not a declaration owner or an
internal compatibility layer. The former `zircon_runtime/src/rhi/` and
`zircon_runtime/src/rhi_wgpu/` directories are deleted.

Backend-dependent command-list, render-pass, resource-lifecycle, pipeline and texture-copy tests
live under `zr_rhi_wgpu/src/tests/`; `zr_rhi/src/tests/` contains only neutral boundary,
capabilities and descriptor coverage. The implementation is statically GREEN and awaiting managed
Cargo acceptance, so this ownership statement does not claim Frameworks01 M2 completion.

## Purpose

The RHI descriptor layer is the neutral contract between render graph/SRP code and concrete WGPU resources. It names buffers, textures, samplers, shader modules, pipelines, swapchains, usages, and queue classes without exposing WGPU objects to app, editor, or framework consumers.

## Behavior Model

`BufferUsage` is a closed neutral bitmask for buffer role declarations. `BufferUsage::ALL` is the canonical mask used by descriptor validation to reject unknown bits while still allowing expected combinations such as uniform-upload buffers or vertex buffers with copy destinations. `BufferUsage::has_unknown_bits()` exposes the same closed-mask check used by descriptor tests and serde-boundary validation paths, keeping buffer usage diagnostics aligned with the `ColorWriteMask` helper in the pipeline descriptor module.

`TextureDesc` now carries enough shape information for the main render chain: HDR formats, depth/stencil formats, 2D arrays, cube textures, mip counts, MSAA sample counts, storage usage, copy usage, residency, and explicitly declared alternate view formats. `TextureFormat::bytes_per_pixel()` and `TextureDesc::checked_storage_size_bytes()` provide deterministic dense virtual sizing for the deterministic contract backend, production registry pressure accounting, and tests that need to verify allocation intent without allocating real GPU memory. `TextureDesc::max_full_mip_levels()` and `mip_levels_fit_shape()` expose neutral mip-chain capacity so graph, upload, and backend code can reject impossible mip declarations before a renderer tries to allocate or copy them.

`TextureDimension::D2Array` uses `TextureDesc::depth` as the array-layer count. `TextureDimension::Cube` also uses `depth` for face count or cube-array face count and must be square with a face count that is a multiple of six when the WGPU test device creates a texture. Ordinary `D1` textures must keep height/depth at one, ordinary `D2` textures must keep depth at one, and multisampling is only accepted for ordinary `D2` textures with a single mip level. These shape checks keep invalid skybox, reflection-probe, texture-array, and MSAA descriptors from entering RenderGraph resource lifetimes as plausible-looking resources.

`TextureResidency::SparseReserved` marks a texture as a sparse or virtual reservation instead of an ordinary dense backing allocation. The descriptor still preserves full virtual width, height, layer/depth count, mip count, format, and usage so RenderGraph, streaming, page table, and diagnostics code can reason about the intended resource shape. The current WGPU test backend reports `supports_sparse_texture = false`, so `create_texture(...)` rejects sparse reservations with a structured `InvalidTextureDescriptor` instead of silently allocating dense memory. If a backend later opts in, the WGPU test device stores zero committed bytes for the reservation; residency/page upload ownership must come from a later sparse texture provider rather than from `TextureDesc` itself.

  `RenderDevice::transient_allocator_stats()` is the neutral live resource pressure query for the RHI layer. It reports currently reserved backing bytes and allocation count for buffer and texture allocations without exposing concrete WGPU resources or allocator objects to RenderGraph/SRP code. The deterministic contract backend derives the count from its live tables. The production WGPU registry additionally counts logically destroyed buffer and texture objects retained in its retirement queue, and removes them only after every observed submission ticket is terminal and the completion pump reaps them. Descriptor-only objects such as samplers, bind group layouts, bind groups, shaders, and pipelines do not count as transient memory reservations. This gives later RenderGraph transient allocation tests a stable lower-layer metric before real WGPU heap pooling or aliasing is introduced.

`RenderDevice::memory_snapshot()` is the corresponding diagnostic breakdown. `GpuMemorySnapshot` keeps active and deferred-retired physical buffer/texture bytes separate from pending CPU upload payload bytes. `GpuMemoryBudget`, carried by `RenderDeviceProfile`, is the admission policy for these same RHI-owned classes: buffer and texture creation count active plus deferred-retired backing, while staged uploads count pending plus flushing CPU payload and have both byte and packet limits. Over-limit admission returns a typed `RhiError` before a native queue operation. Views, descriptor objects, and higher-level caches remain accountable to their own owners and must not duplicate these physical bytes.

`DiagnosticReadbackBudget`, also carried by `RenderDeviceProfile`, is independent from the resource-residency budget. It constrains per-request bytes, requests and bytes per diagnostic frame, all in-flight request bytes, completed receipt retention, dense diagnostic pass IDs, and timestamp/statistics query scopes. `DiagnosticReadbackTracker` is the one neutral lifecycle and quota owner for both copy readbacks and query resolves: it binds an admitted frame only to a matching `DeviceId`, `DeviceGeneration`, and `SubmissionTicket`; cancellation, map failure, device loss, shutdown, capability unavailability, and quota rejection remove or avoid native work before at most one terminal delivery. `DiagnosticQueryPlan` belongs to `zr_rhi`, uses compiler-owned dense pass IDs, and aggregates repeated physical scopes in linear time without pass-name strings. Each `PipelineStatisticsScope` consumes one native query-set index and resolves five consecutive `u64` counters; counter payload width must never be used to allocate query indices. `WgpuRenderDevice` owns submission-qualified buffer/color-texture copies plus timestamp/pipeline-statistics query frames. Its `RenderBackendCaps` reports timestamp support only when both WGPU timestamp feature bits were negotiated and pipeline-statistics support only when its required feature was negotiated. WGPU-specific code owns query sets, resolve/staging buffers, map callbacks, and a ticket-ordered result ring, but cannot submit or poll; its only completion collection occurs after `RenderDevice::poll_submissions()` has performed the device poll. Unsupported optional query features and bounded-admission rejection leave the rendering packet eligible for submission and produce `Unavailable` or `OverBudget` query delivery. This remains an implementation-specific diagnostic interface rather than a neutral synchronous `RenderDevice` readback capability. Diagnostic texture readback remains color-only; direct command-list copies separately permit `Depth32Float` texture-to-buffer work through `TextureCopyAspect::DepthOnly`, while `Depth24Plus`, depth writes, and depth-stencil planes remain rejected by the portable linear-copy contract. Legacy `GpuReadbackQueue`/timer owners plus product raw-path migration remain outside this source increment.

Current-source correction, 2026-08-24: the production diagnostic owner now supports submission-qualified buffer and color-texture copy requests, native timestamp/pipeline-statistics query resolve, checked padded row layout, a single shared diagnostic quota tracker, and ticket-ordered asynchronous delivery. The query command ABI requires a frame-qualified packet plan, exact-once scope consumption, one pipeline-statistics query index per scope, and five resolved counters per such index. The preceding historical M5 wording is not an acceptance receipt; managed Cargo and product capture validation remain pending.

`RenderDevice::write_texture(...)` is the corresponding CPU-to-GPU color-texture upload contract.
It carries one `TextureCopyRegion`, one source `bytes_per_row`, and source bytes, returning the
same copy-queue `SubmissionTicket` lifecycle as `write_buffer(...)`. One request addresses one
array layer or 3D slice; it must target a non-sparse, single-sampled `COPY_DST` color subresource
inside the selected mip. The source row pitch must cover and divide evenly by the format texel
size, and the source must contain every effective row. Data after the last effective pixel is not
retained as upload staging, so common terminal-row padding does not inflate the pending-upload
budget. `TextureCopyRegion` carries an explicit copy aspect, but depth/stencil uploads remain
fail-closed because WGPU forbids writes to the portable depth planes admitted by this MVP. Within
the neutral production-device path, `WgpuSubmissionService` is the only code that
calls WGPU queue write APIs; the production registry retains the texture through the ticket, and
the deterministic contract backend mirrors row-by-row writes for regression coverage. This is S1
infrastructure only: product resource streamers still own raw WGPU textures and have not migrated
to this interface.

`RenderDevice::write_buffer(...)` uses the same queue-upload model and therefore requires
`BufferUsage::COPY_DST` in both the deterministic mirror and production WGPU device. It is not a
mapped-write API: `STAGING_WRITE` remains the descriptor role for a future explicit map/unmap
contract, because WGPU permits that role only with `COPY_SRC`. This hard distinction avoids a
deterministic-only upload that the production device cannot create.

Current-source correction, 2026-08-27: `write_buffer_batch(...)` and
`write_texture_batch(...)` are now the durable upload contracts. One logical batch owns shared
immutable payload storage, validates all writes before mutation, and returns one copy-queue
`SubmissionTicket`; the single-write methods are compatibility-shaped adapters that construct a
one-item batch. This prevents one semantic mesh or texture upload from creating a ticket per
subresource while preserving ordered writes and submission-qualified resource lifetime in both
the deterministic and production WGPU backends.

`RenderDevice::append_submission_statuses(...)` appends exactly one status result per input ticket
in input order. Its default implementation preserves compatibility for future backends, while both
WGPU implementations override it so a bounded frame consumer observes K tracked submissions under
one submission-state lock instead of K locks. The method is observation-only: the product frame
owner remains responsible for one nonblocking `poll_submissions()` before downstream residency,
diagnostic, and retirement consumers run. A successful completion pump returns a
`SubmissionPollReceipt` containing the device id, device generation, and a strictly increasing
poll sequence. Production device-fault terminalization and ordinary native polling issue receipts
from the same sequence owner. The render-asset residency consumer requires a matching, advancing
receipt before it allocates observation work, queries status, or retires resources; a foreign or
replayed receipt produces a typed report failure with no manager mutation or RHI access. The
receipt is ordering evidence for a future single product frame owner, not an unforgeable capability
or proof that the legacy renderer no longer polls independently. The consumer currently uses a
device/generation/queue/sequence ordered frontier, explicit per-frame status and retirement
budgets, last-good publication, and fail-closed terminal errors. This is source-level infrastructure,
not product acceptance: the legacy `SceneRenderer` still owns its raw backend path, managed Cargo
validation and real GPU capture remain pending. Failed multi-resource destruction preserves
per-resource progress and requeues the artifact for a later bounded frame instead of losing its
remaining handles. `RenderAssetGpuResidencyLimits` hard-bounds ready-retirement artifact count;
upload binding, active release batches, and terminal publication reject before mutation when the
queue has no slot, while detached terminal uploads stay tracked until capacity returns. Old backlog
is retired before the frame's bounded status fan-out, and the maintenance report exposes queued
artifact bytes plus deferred terminal uploads. Those bytes are diagnostic ownership attribution,
not a second physical memory admission policy: RHI `GpuMemoryBudget` remains the sole buffer/texture
byte authority. Product profile wiring and shutdown integration remain required before the old
owner can be removed.

Current product bridge, 2026-08-30: `SceneRenderer` now owns one bounded scene-submission terminal
journal downstream of its normal frame-begin completion poll. Successful scene tickets are tracked
up to the device profile's unresolved-submission limit, and pending tickets are observed through one
`append_submission_statuses(...)` batch using reusable scratch storage. An empty journal does not
query submission state. Explicit blocking capture/readback may advance the same backend timeline
more than once inside its bounded wait, but every returned receipt is synchronously routed through
the same journal, IBL, typed-query, and timer consumers. The latest terminal result is published
separately from the current command-recording report as `RenderSceneSubmissionCompletionReport`,
retaining frame generation, ticket, poll receipt, exact terminal status, and typed
observation/tracking failure. Surface paths that already consumed the renderer poll receipt do not
consume it again. Submission error paths publish the latest report before returning, and the public
framework error preserves the typed scene-completion failure. The report also exposes pending count,
tracking capacity, and the last poll's observed/terminal counts for backlog profiling. The identity
fields describe the latest terminal or failure event and may lag the count fields, which describe
the latest accepted poll. This source increment does not make legacy scene commands neutral RHI
packets and has no managed Cargo, WGPU, capture, profile, or power acceptance yet.

Render-asset residency now also binds its completion frontier to one explicit device epoch. A
replacement device cannot continue through a fresh-looking poll receipt or submission ticket until
the owner calls `recover_device_epoch`. That cold-path transaction preflights every live resource
against current catalog/readiness data and reserves all replacement ticket ids before mutation;
then it preserves reference counts, terminalizes the old pending/active projection, abandons all
old-generation handle references and completion cursor state, and publishes one replacement
`QueuedIo` ticket per live resource in stable resource-id order. Failed preflight leaves ticket ids
and residency/GPU state unchanged. This is modeled after the release-all then reinitialize owner
ordering of Unreal's render resources without introducing a global mutable resource list. Abandoned
byte counts are diagnostics only: the failed generation's native registry is released by dropping
its `WgpuRenderDevice` owner, while `GpuMemoryBudget` remains the physical allocation authority.
Product device-owner replacement and injected device-loss validation are still pending.

The public device error contract is owned by `rhi/device/error.rs` and re-exported from the existing
`rhi::RhiError` facade. This keeps the neutral device command DTO root and the complete typed error
vocabulary as separate responsibilities; both files remain below the repository's 800-line review
budget, and no compatibility error type or forwarding conversion was added.

`SamplerDesc` now exposes the renderer-facing sampler states that later material, shadow, skybox, sprite, and postprocess paths need without leaking WGPU. It carries independent magnification, minification, and mipmap filters; U/V/W address modes; LOD min/max clamps; optional comparison sampling for shadow maps; and a bounded anisotropy clamp. `SamplerDesc::linear(...)` and `SamplerDesc::nearest(...)` remain the compact constructors for current callers, while `linear_mipmap_linear(...)`, `with_compare(...)`, `with_lod_clamp(...)`, and `with_anisotropy_clamp(...)` cover trilinear material sampling and shadow-map compare sampling. The WGPU test device rejects non-finite LOD clamps, reversed LOD ranges, and anisotropy outside `1..=16`.

`TextureViewDesc` separates a texture allocation from its shader-visible subresource view. It names a parent `TextureHandle`, optional view format, `TextureViewAspect`, view dimension, mip range, and array-layer range; omitted counts select the remaining range and an omitted format selects the parent format. `TextureViewHandle` is device/generation-qualified like every other RHI handle. The shared deterministic/production validation rejects impossible dimension overrides, undeclared format overrides, invalid color/depth/stencil aspect combinations, out-of-range subresources, non-singleton D2 views, and cube ranges not aligned to six faces. `TextureDesc::view_formats` declares the alternate formats when the backing texture is created; the base format is implicit and cannot be repeated. The portable MVP admits only `Rgba8Unorm`/`Rgba8UnormSrgb` and `Bgra8Unorm`/`Bgra8UnormSrgb` pairs, exactly matching WGPU's current sRGB-only reinterpretation rule. This supplies the runtime-mipgen shape of a linear `Rgba8Unorm` storage view plus an sRGB sampled view without a backend-specific typeless format API. `All` is required for color views and retained for combined depth-stencil attachments; `DepthOnly` exposes a depth sample type, while `StencilOnly` exposes `Uint`. A combined depth-stencil view is rejected as a sampled binding, so attachment and shader-resource intent cannot be confused. `TextureCopyRegion` independently carries `TextureCopyAspect`: color copies use `All`, while the portable MVP permits only `Depth32Float` texture-to-buffer copies through `DepthOnly`. Planar/YUV aspects, `Depth24Plus`, depth writes, and separate stencil-plane storage remain deferred rather than inheriting view semantics. Parent textures cannot be destroyed while a live view refers to them. This follows the UE texture-SRV model while keeping view identity and native WGPU objects private to the RHI registry; it is not an implicit global view cache.

`BindGroupLayoutDesc` and `BindGroupDesc` are the neutral resource-binding contract that sits below shader asset pipeline-layout intent and above concrete WGPU bind groups. Layout entries name a binding index, resource type, and shader-stage visibility without carrying WGPU objects. Bind group entries bind allocated RHI buffers, texture views, and samplers to that layout. `BindingResourceType::SampledTexture` carries its sample type, view dimension, and multisample state; `BindingResourceType::Sampler` carries filtering, non-filtering, or comparison intent. `BindingResourceType::StorageTexture` carries an explicit `StorageTextureBindingDesc`: its write-only access, exact texture format, and non-cube view dimension are checked both when a layout is created and when a view binds to it. The MVP admits only `Rgba8Unorm` and `Rgba16Float`, matching the existing runtime mip-generation and HDR compute targets; storage views require `TextureUsage::STORAGE`, a single-sampled parent, and exactly the declared effective view format/dimension. Thus an sRGB reinterpretation remains valid for sampled binding but is rejected for an `Rgba8Unorm` storage layout; the same texture can expose its implicit linear view to the compute pass. Read/read-write storage and backend-specific feature negotiation remain deferred. The deterministic WGPU test backend rejects empty layouts, duplicate binding declarations, missing visibility, duplicate stage visibility, entry-count mismatches, duplicate bind group entries, missing layout bindings, resource-kind mismatches, unknown handles, and resources whose usage flags or typed view/sampler properties do not satisfy the layout declaration. Uniform bindings require `BufferUsage::UNIFORM`, storage buffer bindings require `BufferUsage::STORAGE`, sampled texture bindings require `TextureUsage::SAMPLED`, and sampler bindings require a compatible live sampler. The production WGPU owner encodes this typed sampled-texture/sampler/write-only-storage subset directly.

Bind group descriptors are metadata snapshots, not hidden strong references to buffer or texture backing storage. A bind group can remain queryable through `bind_group_desc(...)` after a referenced resource handle has been destroyed, which is useful for diagnostics and descriptor ownership tests. Command submission revalidates the descriptor against current live resource tables when `SetBindGroup` is recorded, so draw or dispatch work cannot accidentally use a stale buffer, texture, or sampler handle.

`PipelineLayoutDesc`, `ShaderModuleDesc`, and `PipelineDesc` now form the neutral shader/pipeline binding contract. A pipeline layout references zero or more allocated bind group layout handles; zero is valid for fullscreen, debug, or compute passes that use only push-free shader state, while duplicate or unknown bind group layout handles are rejected. Shader modules carry source, stage, and entry point; the test backend rejects empty shader source and empty entry point strings before a pipeline can reference them. Pipeline descriptors must reference a pipeline layout handle and stage-compatible shader module handles. Raster pipelines require a vertex shader, reject compute shaders, and must carry a `RasterPipelineStateDesc`. Color-output raster pipelines require a fragment shader; depth-only pipelines may omit it for shadow-map and depth-prepass style passes. Compute pipelines require a compute shader, reject vertex/fragment shaders, and cannot declare raster state. Ray-tracing pipelines remain a declared future `PipelineKind`, but the WGPU contract rejects them until a later backend capability and acceleration-structure layer exists.

`RasterPipelineStateDesc` is the neutral render-pipeline state block for raster consumers. It carries color target formats, optional blend state, color write masks, optional depth/stencil state, primitive topology, front-face winding, cull mode, MSAA sample count, and vertex input layout without embedding material semantics, shader asset names, or WGPU-specific objects. `ColorTargetDesc::blend = None` means the target writes without blending; `BlendStateDesc::replace()`, `alpha_blending()`, and `additive()` cover the canonical replace, transparent, and additive state families that later renderer caches can specialize without introducing their own ad hoc blend vocabulary.

`VertexInputLayoutDesc` models the pipeline-side vertex buffer declaration. Each `VertexBufferLayoutDesc` has an array stride, a `VertexStepMode` of per-vertex or per-instance, and one or more `VertexAttributeDesc` entries with shader location, byte offset, and `VertexFormat`. Empty vertex input remains valid for fullscreen, generated, or depth-only pipelines that do not read a vertex buffer. When buffers are declared, the WGPU test backend rejects empty buffer layouts, zero strides, duplicate shader locations across the whole vertex input, byte-range overflow, and attributes whose `offset + format.size_bytes()` exceeds the buffer stride.

The WGPU test backend rejects zero sample counts, pipelines with neither color nor depth/stencil targets, depth formats in color targets, empty or unknown color write masks, color formats in the depth/stencil slot, and stencil-enabled state on depth-only formats. Blend state is currently a neutral descriptor-only contract: enum shape prevents invalid factors or operations, and later real WGPU pipeline creation can map it directly to backend blend components.

`RenderBackendCaps` now carries debug instrumentation capability separately from resource and queue capability. `supports_debug_markers` and `supports_debug_groups` describe whether the backend accepts command-stream marker/group records, while `supports_graphics_debugger_capture` describes whether the backend exposes a graphics-debugger capture API such as WGPU's RenderDoc/Xcode hook. `RenderDebugInstrumentationStatus` is the neutral device-level status snapshot derived from those caps by default. It reports support and active capture state without viewport-specific pending-capture bookkeeping; the higher `RenderFramework` layer still owns viewport requests, frame numbers, and last submitted capture status.

`CommandListCommand` now includes neutral debug markers, debug groups, render-pass, explicit compute-pass, diagnostic pass scopes, and raster work submission. A command list can record `DebugMarker`, `PushDebugGroup`, `PopDebugGroup`, `BeginRenderPass`, `BeginRenderPassWithDiagnostics`, `EndRenderPass`, `BeginComputePass { label }`, `BeginComputePassWithDiagnostics`, `EndComputePass`, `SetViewport`, `SetScissorRect`, `SetPipeline { pipeline }`, `SetBindGroup { slot, bind_group }`, and `DispatchCompute { x, y, z }`. A diagnostic begin variant carries only a backend-neutral `DiagnosticPassQueryScope`; its immutable `RhiSubmissionPacket` must carry the matching frame-qualified `DiagnosticQueryPlan`, which validates every reserved timestamp/statistics range is used exactly once before native command encoding. Debug marker/group labels must be non-empty, debug groups must be balanced by command-list end, and render/compute-pass-local debug groups must be closed before their matching end command; this mirrors WGPU command encoder/pass debug grouping while keeping the public command stream backend-neutral. Render passes carry color attachment descriptors plus an optional depth/stencil attachment descriptor with neutral role-specific load operations and `RenderPassStoreOp` values. `RenderPassTextureViewDesc` identifies the texture subresource bound by an attachment with a texture handle, mip level, and array layer; color attachments, depth/stencil attachments, and color resolve targets all use this descriptor, while compatibility constructors still bind mip 0 and layer 0 by default. `RenderPassColorLoadOp::Clear(RenderClearColor)` carries the RGBA clear value for color targets, `RenderPassDepthLoadOp::Clear(f32)` carries the normalized depth clear value, and `RenderPassStencilLoadOp::Clear(u32)` carries the stencil clear value. A color attachment can optionally name a single-sampled resolve target view for MSAA resolves. Submit validation requires render passes on the graphics queue, rejects nested or unclosed passes, rejects empty attachment sets, rejects duplicate attachment or resolve subresources by texture+mip+layer, rejects out-of-range mip levels and array layers, requires every attachment and resolve target texture to have `TextureUsage::RENDER_ATTACHMENT`, derives render-pass extent from the selected mip, allows distinct array layers or cube faces of the same texture as distinct attachments, requires all pass attachment views to share the same extent and sample count, requires resolve sources to be multisampled, requires resolve target views to be single-sampled with the same format and selected-view extent as the color source, requires finite color clear values, requires depth clears inside `0.0..=1.0`, and keeps stencil load/store declarations paired and limited to stencil-capable depth formats.

Compute command submission requires a bound `PipelineKind::Compute`, non-zero workgroup counts, and a graphics or compute queue. `BeginComputePass` provides a non-empty labelled scope for multiple dispatches; nested render/compute scopes, copies, and raster-only state are rejected until `EndComputePass`, while one-dispatch command lists may remain unscoped. The copy queue rejects compute dispatch and compute-pass entry explicitly, and compute dispatch cannot be recorded inside an active render pass. Bind group commands require a live bind group handle. When a pipeline is already active, binding a group also checks that the slot exists in the pipeline layout and that the bind group's layout matches that slot. Draw and dispatch commands validate the active pipeline layout again and require every declared bind group layout slot to have a compatible bound bind group. Bind groups may still be recorded before the pipeline, which lets command generators reuse the same state-ordering pattern as WGPU render passes while keeping final compatibility checks at work submission.

Raster command lists can set a `RenderViewportDesc`, set a `RenderScissorRect`, bind vertex buffer slices, bind an index buffer with `IndexFormat::Uint16` or `Uint32`, and issue `Draw` or `DrawIndexed`. Viewport and scissor commands are pass-local state: submit validation requires an active render pass, finite non-empty viewport dimensions, a depth range inside `0.0..=1.0`, and viewport/scissor rectangles that fit the active pass extent. Submit validation for drawing requires a graphics queue, an active render pass, a bound `PipelineKind::Raster`, matching render-pass color attachment count and formats, matching render-pass and raster-pipeline sample counts, matching depth/stencil attachment presence and format, non-zero draw and instance counts, live vertex/index buffers with `VERTEX` or `INDEX` usage, non-empty buffer binding ranges, required bind groups matching the active pipeline layout, required vertex buffer slots matching the pipeline vertex input layout, vertex/instance draw ranges that fit the bound buffer strides, aligned index buffer ranges, and indexed draw ranges that fit the bound index slice. Pipelines with empty vertex input still support generated fullscreen/debug-style vertex draws without binding a vertex buffer. Copy commands cannot execute inside an active render pass. The WGPU test backend treats compute and raster submission as contract-level execution paths: it validates handles, pipeline kind, queue usage, pass attachment compatibility, viewport/scissor bounds, bind group layout compatibility, and bound ranges, then advances the ticket lifecycle without pretending to run an algorithm-specific shader.

`TextureCopyRegion` is the neutral copy rectangle for buffer-to-texture and texture-to-buffer transfers. It carries mip level, x/y/z origin, width, and height instead of assuming a whole 2D base level. `zr_rhi_wgpu::texture_copy` converts that rectangle into a dense byte layout for the headless WGPU test backend, including D2 array layers, cube faces, and mipped levels. Sparse reservations and multisampled textures reject copy regions at validation time because later residency/upload systems must own sparse page commitment and MSAA resolve semantics explicitly.

## Design And Rationale

The descriptor set is intentionally smaller than WGPU's full format catalog, but it covers the render-main-chain requirements: scene HDR targets, depth prepass targets, reflection/cubemap slots, texture arrays, compute/storage-ready color targets, mipmapped sampled textures, comparison/trilinear/anisotropic sampler intent, bind group layout/resource binding intent, and sparse texture reservations. Compressed imported asset formats still flow through existing graphics asset upload code until the RHI facade needs a broader compressed-format vocabulary.

The shader and pipeline contract follows the same separation used by WGPU-facing examples and Bevy render code: shader modules, bind group layouts, pipeline layouts, color targets, blend state, primitive state, depth/stencil state, vertex buffer layouts, multisample state, and render/compute pipelines are distinct objects. Unity Graphics SRP also treats drawing settings, filtering settings, and render-state overrides as pass-side state rather than as import details. Zircon keeps raster state in neutral handles and descriptors so later renderer caches can specialize pipelines against the same lower contract without exposing WGPU.

Pipeline descriptors moved into `rhi/descriptors/pipeline.rs` while remaining re-exported through the public `crate::rhi` facade. This keeps the growing shader, blend, vertex input, and raster state responsibility out of the general resource descriptor file and preserves the repository rule that broad descriptor roots stay navigational instead of accumulating every render-state declaration.

The WGPU backend advertises graphics, compute, and copy command-list queue classes. Async compute scheduling remains a RenderGraph/SRP policy decision; the backend can still fall async graph lanes back to graphics while exposing compute pipeline and dispatch capability at the RHI boundary.

Texture copy tests now live in `rhi/tests/texture_copy.rs` rather than the general device contract module. This keeps the growing mip, array, cube, upload, and readback behavior isolated as its own RHI responsibility while the device contract module stays focused on handles, queues, resource descriptors, submission tickets, and boundary imports.

Pipeline and command-list tests now live in `rhi/tests/pipeline.rs` and `rhi/tests/command_list.rs`. This keeps shader/pipeline layout validation, raster state validation, compute dispatch validation, and queue command validation out of the general device contract module, which remains below the repository's large-file warning threshold. The remaining device contract tests import every public RHI handle through `crate::rhi`, so lib-test compilation catches facade export drift before upper UI or editor regressions are filtered.

`zr_rhi_wgpu::resource_validation` owns descriptor and usage validation for the headless WGPU test backend. Keeping those checks outside `device.rs` prevents the backend state and command execution file from becoming a catch-all as descriptor rules grow for sparse textures, cubemaps, array textures, samplers, and future residency/upload capabilities.

`zr_rhi_wgpu::bind_group_validation` owns bind group instance validation because that check needs both layout declarations and live resource handles. Splitting it out keeps `zr_rhi_wgpu::device` focused on allocation, state storage, command validation, and command execution while still letting the test backend enforce material, postprocess, and compute resource-binding contracts before renderer caches are migrated to the neutral RHI surface.

`zr_rhi_wgpu::pipeline_validation` owns shader module, pipeline layout, and pipeline descriptor validation. That module checks stage compatibility and layout/shader handle existence without mixing another responsibility into `zr_rhi_wgpu::device`, which remains the stateful allocator and command executor.

`zr_rhi_wgpu::command_validation` owns command-list submit validation and the headless copy execution path. The split keeps `zr_rhi_wgpu::device` focused on resource allocation, descriptor storage, command-list construction, and ticket lifecycle bookkeeping while command validation grows to cover debug marker/group balancing, compute dispatches, bind group command binding, raster draws, vertex/index buffer binding ranges, and texture-copy execution.

  `zr_rhi_wgpu::WgpuRenderDevice` is the generation-qualified production neutral owner. It allocates opaque device/generation-scoped resource handles and keeps native WGPU objects in a private registry. Its `WgpuSubmissionService` is the sole owner of native `Queue::submit`, completion callbacks, and queue writes: `RhiSubmissionPacket` is immutable and device-qualified, and `enqueue_submission_packet(...)` admits all of its command lists under one `SubmissionTicket`; the one-list convenience method follows the same path. The WGPU service retains the resulting command-buffer vector until `flush_submissions(...)`, preserving upload/packet order while coalescing consecutive uploads with following command buffers. `SubmissionLimits`, also carried by `RenderDeviceProfile`, bounds unresolved packets and caller-visible terminal status history. A compressed terminal sequence index preserves retirement safety after old receipt status is evicted. Non-blocking polling advances only callback-confirmed tickets to `Completed` and removes terminal dependencies from live resources. Every ticket carries device, generation, logical queue class, and sequence; the backend verifies it was issued before reporting status. `wait_for_submission(...)` is bounded and targets one ticket rather than waiting for the whole device. `cancel_submission(...)` terminalizes only `Accepted` work; already submitted native work returns a typed non-cancellable error instead of pretending that it stopped. The command-context pool recycles only ticket metadata after a completion or accepted-work cancellation because WGPU command encoders and buffers are one-shot. Logical destruction revokes a handle immediately while the registry retains native resources through every non-terminal use. Typed sampled texture views, sampler bind groups, and write-only `Rgba8Unorm`/`Rgba16Float` storage texture views now use the same registry and submission-qualified retirement path; read/read-write storage remains incomplete. The capability view admits buffer-to-buffer and color-texture copy, explicit compute scope/dispatch, raster draw/indexed draw, and debug markers/groups. Depth/stencil texture copy remains rejected until a neutral texture-aspect DTO exists; surface, timestamps/statistics, subgroup, binding-array, capture, ticket telemetry, and product raw-path migration remain incomplete. It does not expose `wgpu::Device`, `wgpu::Queue`, or native resources; synchronous `read_buffer` and `read_texture` deliberately return `ReadbackUnavailable` rather than executing a device-wide wait.

`zr_rhi_wgpu::DeterministicRhiContractDevice` is compiled only for RHI contract tests. Its host-memory resources make the explicit `Accepted -> Submitted -> Completed` ticket state machine deterministic, but it is not a product WGPU implementation. Current scene rendering still owns its real `wgpu::Device` and `wgpu::Queue` through `graphics::backend::RenderBackend` until the Runtime90 product cutover transfers that same ownership chain; `rhi/tests/device_contract/framework_boundary.rs` rejects the deterministic device name from product graphics sources so the CPU mirror cannot enter a frame path.

Runtime 15 M4 RHI device handle owner split is recorded as `runtime_15_rhi_device_handles_owner_split_static_passed_cargo_deferred`. `rhi/device.rs` remains the typed RHI DTO and error owner, `rhi/device/render_device.rs` owns the public `RenderDevice` trait, and `rhi/device/handles.rs` owns the neutral resource handle newtypes and the private allocator/validation machinery. Public code can inspect diagnostic identity plus device/generation, but cannot construct a handle or recover its allocator slot/generation from a public `new/raw` API. The parent re-exports the handle types and trait, so existing `rhi::device::*` and `rhi::*` type paths stay unchanged. The structure guard keeps handles from returning to the parent and keeps each owner below the production-file budget.

Runtime 15 M4 RHI WGPU command validation render-state owner split is recorded as `runtime_15_rhi_wgpu_command_validation_render_state_split_static_passed_cargo_lock_blocked`. `rhi_wgpu/command_validation.rs` stays the command-list validation/execution owner for `validate_recorded_commands(...)`, `execute_recorded_commands(...)`, debug group, render pass, queue, copy, draw, and dispatch traversal. `rhi_wgpu/command_validation/render_state.rs` now owns `RecordedRenderState`, `CommandResourceLookup`, bind group slot validation, binding range validation, vertex/index/strided draw-range helpers, and pipeline-layout lookup. The structure guard `runtime_15_rhi_wgpu_command_validation_state_is_child_owner` keeps those helpers from returning to the parent and keeps both owners under the 800-line production-file budget.

Runtime 15 M4 RHI WGPU device command-list owner split is recorded as `runtime_15_rhi_wgpu_device_command_list_owner_split_static_passed_cargo_deferred`. `rhi_wgpu/device.rs` stays the headless WGPU `RenderDevice` owner for backend state, resource storage, descriptor snapshots, submit validation handoff, ticket completion, transient allocator stats, and read/write paths, while `rhi_wgpu/device/command_list.rs` now owns `WgpuCommandList` and the neutral command recording implementation for debug marker/group, copy, render pass, bind group, viewport/scissor, vertex/index draw, and dispatch commands. The structure guard `runtime_15_rhi_wgpu_device_command_list_is_child_owner` keeps command-list recording from returning to the device state file and keeps both owners under the 800-line production-file budget.

Runtime 15 M3 RHI WGPU render device lock poison recovery is recorded as `runtime_15_rhi_wgpu_render_device_lock_poison_recovery_static_passed_cargo_deferred`. `rhi_wgpu/device.rs` keeps owning the headless WGPU `RenderDevice` contract, but its `WgpuRenderDeviceState` mutex is now opened only through `WgpuRenderDevice::lock_state()`, which recovers poisoned locks instead of panicking through direct `.lock().unwrap()`. The helper covers resource allocation/destruction, descriptor snapshots, bind group and pipeline validation lookups, command submission, ticket completion, transient allocator stats, staging buffer write/read, and texture readback without changing neutral RHI handles or descriptor semantics. The module-local `wgpu_render_device_state_accessors_recover_poisoned_lock` deliberately poisons the state lock and verifies transient stats, staging buffer creation, write, and read still recover. The structure guard `runtime_15_rhi_wgpu_render_device_lock_poison_recovery_guard_covers_device_state` locks the helper/test/status anchors through `docs/zircon_runtime/rhi/descriptors.md` and rejects direct production lock unwraps in `rhi_wgpu/device.rs`.

`zr_rhi_wgpu::render_pass_validation` owns active render-pass attachment validation and pipeline/pass compatibility checks. Keeping that state in its own module prevents `command_validation` from becoming another descriptor catch-all while letting WGPU-style render-pass rules grow toward concrete attachment views, resolves, load/store actions, and depth/stencil operations.

Render-pass load operations are type-specific instead of a single shared clear flag. This matches WGPU's value-carrying attachment operations, Bevy's clear-color path, and Unity Graphics SRP command-buffer clears while keeping invalid role/value pairings out of the neutral RHI API. Zircon validates color-value finiteness and depth normalization at submit time because renderer features, cameras, postprocess passes, and UI passes will all produce attachment clears through the same command-list contract.

Render-pass color resolve targets are explicit on the color attachment descriptor instead of being inferred from texture naming or postpass conventions. This follows WGPU's color attachment shape and Bevy's MSAA camera/writeback model while keeping anti-aliasing support in the lower RHI contract. Later SRP, camera, UI, and postprocess code can request MSAA color buffers and resolved outputs through RenderGraph resource declarations without inventing renderer-specific resolve side channels.

Render-pass attachment subresource views mirror the WGPU and Bevy `TextureViewDescriptor` pattern of selecting a base mip and array layer, but keep the public API neutral by naming only RHI texture handles and integer subresource coordinates. This is required before mip-generation, downsample chains, cube-face rendering, texture-array shadows, reflection probes, and postprocess history buffers can use one allocated texture across multiple render-pass views without leaking WGPU view objects into graph or renderer descriptors.

The public RHI facade exports neutral handles and descriptors only. The old `GpuBuffer = wgpu::Buffer` alias has been removed so framework DTOs cannot leak backend-owned WGPU objects across the `rhi_wgpu` and renderer backend boundary.

`rhi::tests::boundary` makes the M1 exposure rule executable. It scans production app, editor, framework, and runtime-interface Rust sources plus app/editor/interface manifests to reject direct raw `wgpu` imports or dependencies, while intentionally allowing current runtime graphics and `rhi_wgpu` implementation internals to keep owning backend details until later cutover slices migrate them. The same test module scans the neutral RHI capability, descriptor, and device source files for upper-rendering vocabulary such as mesh, material, light, scene, camera, UI, and sprite names. `rhi::ui_surface` is intentionally excluded from that second scan because it is the explicit UI presentation contract, not a neutral descriptor/device owner.

## Test Coverage

Focused tests cover descriptor labels/usages, sampler mip/LOD/compare/anisotropy fields, bind group layout descriptors, pipeline layout and shader-stage pipeline descriptors, raster pipeline state descriptors, blend state descriptors, vertex input layout descriptors, HDR array and cube descriptors, mip storage sizing, mip-capacity reporting, sparse reservation virtual sizing, live transient allocator byte/allocation accounting, WGPU sparse capability rejection, WGPU descriptor round-trips, bind group layout and bind group round-trips, stale bind group resource rejection at command submit, pipeline layout and shader-bound pipeline round-trips, raster state round-trips for color+depth and depth-only pipelines, raster state validation, vertex input validation, bind group layout validation, bind group resource type and usage validation, shader module and pipeline descriptor validation, sampler descriptor validation, invalid D1/D2/Cube/MSAA/mip texture shapes, invalid cube face counts, WGPU queue capability reporting, debug marker/group recording and validation, debug instrumentation status reporting, direct-WGPU boundary source assertions for app/editor/framework/interface layers, neutral RHI upper-semantic source assertions for descriptor/device owners, compute command-list dispatch validation, compute and raster bind group command validation, bind group layout compatibility validation, render-pass begin/end validation, render attachment usage/format validation, render attachment extent/sample validation, render-pass attachment subresource view recording, mip/layer bounds validation, mip-derived extent validation, distinct array-layer attachment validation, render-pass color/depth/stencil clear-value recording and validation, MSAA color resolve target recording and validation, resolve target view-shape validation, raster pipeline attachment compatibility validation, raster pipeline sample-count compatibility validation, viewport/scissor pass-state validation, render-pass lifetime validation, raster command-list draw and indexed-draw validation, generated-vertex draw validation, vertex/index buffer usage validation, draw-range validation, base-level texture upload/readback, mip+array-layer copy, cube-face copy/readback, and copy-region range validation.

Focused RHI validation on 2026-06-02 passed with 69 tests, 0 failures, using `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`.

The 2026-06-04 texture-copy split passed scoped rustfmt, conflict-marker scanning, trailing-whitespace scanning, and path-scoped `git diff --check`. A later low-interference rerun used the already-built runtime lib-test binary at `D:\cargo-targets\zircon-asset-m6-root-0604-fresh\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe texture_copy --test-threads=1 --nocapture`; all 6 focused texture-copy tests passed, including mip+array-layer upload/readback and cube-face upload/readback. The Cargo wrapper form had previously timed out while unrelated runtime/editor/plugin Cargo lanes were compiling, so this evidence proves the current generated test binary behavior but is not a fresh workspace build claim.

The 2026-06-04 raster pipeline-state slice passed `rustfmt --edition 2021 --check` over the touched RHI files, `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` with existing warnings only, path-scoped `git diff --check`, conflict-marker scanning, and a source scan proving `descriptors.rs` still avoids the forbidden upper-layer type names. The Cargo test wrapper and no-run build timed out while compiling the Windows lib-test binary under active external Cargo load, but the generated binary was then run directly: `E:\cargo-targets\zircon-rhi-bind-group-0604\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe rhi::tests --test-threads=1 --nocapture` passed 41 tests, 0 failed, including the new pipeline and command-list modules.

The 2026-06-04 blend and vertex-input pipeline descriptor slice passed `rustfmt --edition 2021 --check` over the touched RHI files, path-scoped `git diff --check` with expected LF-to-CRLF warnings only, conflict-marker scanning, and a combined source scan proving `descriptors.rs` plus `descriptors/pipeline.rs` still avoid the forbidden upper-layer type names. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` passed with existing warnings only. The first focused `cargo test -p zircon_runtime --lib rhi::tests::pipeline ...` attempt failed because an unrelated active runtime-modules test split had a stale `RuntimeProfileId` import; after that other-session file changed, the same command passed 4 tests, 0 failed. The fresh generated binary `E:\cargo-targets\zircon-rhi-bind-group-0604\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe rhi::tests --test-threads=1 --nocapture` then passed 41 tests, 0 failed.

The 2026-06-04 BufferUsage helper unblock restored `BufferUsage::has_unknown_bits()` after a runtime lib-test compile reached `zircon_runtime/crates/zr_rhi/src/tests/descriptors.rs` and found the method missing while `BufferUsage::ALL` was already present. Static validation passed with `rustfmt --edition 2021 --check zircon_runtime/crates/zr_rhi/src/descriptors.rs` and path-scoped `git diff --check` over `zircon_runtime/crates/zr_rhi/src/descriptors.rs` plus this document. The refreshed runtime lib-test binary then passed `rhi::tests::descriptors::resource_descriptors_keep_stable_labels_and_usage` directly: 1 test, 0 failed, 2633 filtered out.

The 2026-06-04 raster command-list draw slice added neutral vertex/index buffer binding plus `Draw` and `DrawIndexed` commands, and split submit validation/execution into `zr_rhi_wgpu::command_validation` so `device.rs` stays focused. Validation passed with `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` using existing warning noise only. The first focused Cargo test wrapper timed out while the Windows lib-test binary was still compiling; after the same lane produced a fresh `E:\cargo-targets\zircon-rhi-bind-group-0604\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe` at 2026-06-04 14:19:43, direct execution with `rhi::tests::command_list --test-threads=1 --nocapture` passed 8 tests, 0 failed, 2631 filtered out. A later hot-cache wrapper run also passed: `cargo test -p zircon_runtime --lib rhi::tests::command_list --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` passed 8 tests, 0 failed, 2631 filtered out, with existing warnings only.

The 2026-06-04 command-list bind group binding slice added neutral `SetBindGroup { slot, bind_group }` recording and submit-time pipeline-layout compatibility checks for compute dispatches, direct draws, and indexed draws. Validation passed with scoped `rustfmt --edition 2021 --check`, conflict-marker scanning, forbidden upper-layer semantic scanning over the touched RHI command files, and path-scoped `git diff --check` with expected LF-to-CRLF warnings only. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` passed with existing warnings only. The focused command-list suite passed through the Cargo wrapper: `cargo test -p zircon_runtime --lib rhi::tests::command_list --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` ran 11 tests, 0 failed, 2631 filtered out, covering the new raster bind group, compute bind group, missing binding, invalid slot, unknown handle, and layout mismatch cases.

The 2026-06-04 render-pass command-list slice added neutral `BeginRenderPass` and `EndRenderPass` commands plus WGPU submit validation for attachment usage, color/depth format compatibility, pass lifetime, queue class, and copy/compute exclusion inside an active pass. Existing raster command-list tests were updated to draw inside a matching color/depth pass so their bind group and vertex/index assertions still validate the intended contracts. Validation passed with scoped `rustfmt --edition 2021 --check`, conflict-marker scanning, forbidden upper-layer semantic scanning over the touched RHI command files, and path-scoped `git diff --check` with expected LF-to-CRLF warnings only. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` passed with existing warnings only. Focused tests passed through Cargo wrappers: `cargo test -p zircon_runtime --lib rhi::tests::render_pass_command_list --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` ran 5 tests, 0 failed, 2643 filtered after an orphaned timed-out compiler process was stopped and the warmed target completed; `cargo test -p zircon_runtime --lib rhi::tests::command_list --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` ran 11 tests, 0 failed, 2637 filtered.

The 2026-06-04 render-pass state slice added neutral `RenderViewportDesc`, `RenderScissorRect`, `SetViewport`, and `SetScissorRect` commands. WGPU submit validation now treats viewport/scissor as active render-pass state, checks viewport finiteness, size, depth range, and pass extent bounds, checks scissor size and bounds, and requires pass attachments to agree on extent plus sample count. Validation passed with `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` using existing warnings only. Focused `cargo test -p zircon_runtime --lib rhi::tests::render_pass_state --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` ran 5 tests, 0 failed, 2648 filtered.

The 2026-06-04 render-pass clear-value slice replaced the shared `RenderPassLoadOp` flag with typed color/depth/stencil load operations that carry actual clear values. Validation passed with `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` using existing warnings only. Focused `cargo test -p zircon_runtime --lib rhi::tests::render_pass_clear_values --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` ran 3 tests, 0 failed, 2654 filtered. Follow-up focused suites passed after the load-op hard cutover: `rhi::tests::render_pass_command_list` ran 5 tests, 0 failed, 2652 filtered; `rhi::tests::render_pass_state` ran 5 tests, 0 failed, 2652 filtered; and `rhi::tests::command_list` ran 11 tests, 0 failed, 2646 filtered. Existing warnings remain in unrelated runtime/UI/render modules.

The 2026-06-04 render-pass MSAA resolve-target slice added optional color attachment resolve targets and submit-time validation for multisampled sources, single-sampled resolve targets, matching format/extent, duplicate bindings, render-attachment usage, and raster pipeline sample-count compatibility. Validation passed with `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` using existing warnings only. Focused `cargo test -p zircon_runtime --lib rhi::tests::render_pass_resolve --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` ran 5 tests, 0 failed, 2657 filtered. Follow-up focused suites passed: `rhi::tests::render_pass_command_list` ran 5 tests, 0 failed, 2657 filtered; `rhi::tests::render_pass_state` ran 5 tests, 0 failed, 2657 filtered; `rhi::tests::command_list` ran 11 tests, 0 failed, 2651 filtered; and `rhi::tests::render_pass_clear_values` ran 3 tests, 0 failed, 2659 filtered. Existing warnings remain in unrelated runtime/UI/render modules.

The 2026-06-04 render-pass attachment subresource-view slice added neutral `RenderPassTextureViewDesc` plus color, depth/stencil, and resolve-target attachment view selection. Submit validation now checks mip and layer bounds, derives pass extent from the selected mip, treats duplicate bindings as texture+mip+layer conflicts, allows distinct array layers of one texture as separate attachments, and checks resolve-target view extent against the source pass extent. Validation passed with `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` using existing warnings only. The first focused view test run caught an unclosed matching-path test render pass; after closing that command-list pass, `cargo test -p zircon_runtime --lib rhi::tests::render_pass_views --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` ran 5 tests, 0 failed, 2663 filtered. Follow-up focused suites passed: `rhi::tests::render_pass_resolve` ran 5 tests, 0 failed, 2663 filtered; `rhi::tests::render_pass_command_list` ran 5 tests, 0 failed, 2663 filtered; `rhi::tests::render_pass_state` ran 5 tests, 0 failed, 2663 filtered; and `rhi::tests::command_list` ran 11 tests, 0 failed, 2657 filtered. Existing warnings remain in unrelated runtime/UI/render modules.

The 2026-06-04 scoped debug-group slice added neutral command-list `PushDebugGroup` and `PopDebugGroup` commands beside the existing insertion marker. Submit validation rejects empty labels, stray pop commands, groups left open at command-list end, render-pass-local groups left open at pass end, and scope mismatches where an encoder-level group is popped while a render pass is active. Validation passed with `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` using existing warnings only. The first focused `rhi::tests::debug_markers` wrapper timed out while compiling the Windows lib-test binary; after that process finished, the warmed wrapper passed 3 tests, 0 failed, 2669 filtered. Adjacent focused suites also passed on the warmed target: `rhi::tests::command_list` ran 11 tests, 0 failed, 2661 filtered, and `rhi::tests::render_pass_command_list` ran 5 tests, 0 failed, 2667 filtered. Final hygiene passed: scoped `rustfmt --edition 2021 --check`, path-scoped `git diff --check` with expected LF-to-CRLF warnings only, conflict-marker scanning, and an upper-layer semantic scan over touched RHI production files. During focused lib-test compilation, an unrelated scene test `unwrap_err()` assertion imposed a `Debug` bound on `QueryState`; the assertion was rewritten as an explicit match so the shared runtime test harness could compile without changing scene behavior.

The 2026-06-04 RHI debug instrumentation status slice added `supports_debug_markers`, `supports_debug_groups`, and `supports_graphics_debugger_capture` to `RenderBackendCaps`, plus neutral `RenderDebugInstrumentationStatus` and `RenderDevice::debug_instrumentation_status()`. WGPU capability mapping now reports marker/group/capture API support independently from surface support, while the headless test device reports no active capture by default. Validation passed with `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never` using existing warnings only. The first focused `rhi::tests::debug_status` wrapper timed out while compiling the Windows lib-test binary; after the same process finished, the warmed wrapper passed 2 tests, 0 failed, 2673 filtered. Follow-up focused suites passed on the warmed target: `rhi::tests::capabilities` ran 2 tests, 0 failed, 2673 filtered, and `zr_rhi_wgpu::tests::wgpu_caps_fall_back_to_graphics_and_copy_without_rt` ran 1 test, 0 failed, 2674 filtered. Existing warnings remain in unrelated runtime/UI/render modules.

The 2026-06-04 RHI boundary guardrail slice added `rhi::tests::boundary` to keep M1 exposure rules executable. The first focused `cargo test -p zircon_runtime --lib rhi::tests::boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` wrapper timed out while compiling the Windows lib-test binary; the same runtime compile process was still active, then exited cleanly. The warmed rerun passed 2 tests, 0 failed, 2675 filtered, covering direct-WGPU source/dependency rejection for app/editor/framework/interface layers and neutral RHI upper-semantic source rejection for descriptor/device owners. Existing warnings remain in unrelated runtime/UI/render modules.

The 2026-06-04 RHI resource lifecycle stats slice added `RenderDevice::transient_allocator_stats()` and the WGPU headless implementation backed by live buffer and texture storage. The first focused `cargo test -p zircon_runtime --lib rhi::tests::resource_lifecycle --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rhi-bind-group-0604 --message-format short --color never -- --test-threads=1 --nocapture` wrapper timed out while compiling the Windows lib-test binary; process inspection showed the same runtime compile still active, and it produced a fresh binary. The warmed rerun passed 2 tests, 0 failed, 2677 filtered, covering live byte/allocation accounting and resource destruction updating stats without implicitly deleting descriptor-only bind group metadata. Follow-up submit-time liveness validation now revalidates bind group descriptors against live resources in `zr_rhi_wgpu::command_validation`; the first wrapper again timed out during compile and then produced a fresh binary, and the warmed rerun passed 3 tests, 0 failed, 2677 filtered, including stale bind group resource rejection. Existing warnings remain in unrelated runtime/UI/render modules.
