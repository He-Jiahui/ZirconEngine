---
related_code:
  - zircon_runtime/src/rhi/descriptors.rs
  - zircon_runtime/src/rhi/device.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/device.rs
implementation_files:
  - zircon_runtime/src/rhi/descriptors.rs
  - zircon_runtime/src/rhi/device.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/device.rs
  - zircon_runtime/src/rhi/tests/descriptors.rs
  - zircon_runtime/src/rhi/tests/device_contract.rs
plan_sources:
  - user: 2026-06-02 implement ZirconEngine WGPU render main-chain closure plan
  - .codex/plans/ZirconEngine WGPU 渲染主链闭环计划.md
  - .codex/plans/Zircon SRPRHI 渲染管线补全计划.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
tests:
  - zircon_runtime/src/rhi/tests/descriptors.rs
  - zircon_runtime/src/rhi/tests/descriptors.rs::texture_descriptors_mark_sparse_reservations_without_losing_virtual_size
  - zircon_runtime/src/rhi/tests/device_contract.rs
  - zircon_runtime/src/rhi/tests/device_contract.rs::command_list_records_compute_dispatch_and_submit_validates_pipeline
  - zircon_runtime/src/rhi/tests/device_contract.rs::wgpu_rhi_rejects_sparse_reserved_texture_without_backend_support
  - zircon_runtime/src/rhi_wgpu/tests.rs
  - cargo test -p zircon_runtime --lib --locked rhi --jobs 1 --message-format short --color never
doc_type: module-detail
---

# RHI Descriptors

## Purpose

The RHI descriptor layer is the neutral contract between render graph/SRP code and concrete WGPU resources. It names buffers, textures, samplers, shader modules, pipelines, swapchains, usages, and queue classes without exposing WGPU objects to app, editor, or framework consumers.

## Behavior Model

`TextureDesc` now carries enough shape information for the main render chain: HDR formats, depth/stencil formats, 2D arrays, cube textures, mip counts, MSAA sample counts, storage usage, copy usage, and residency. `TextureFormat::bytes_per_pixel()` and `TextureDesc::checked_storage_size_bytes()` provide deterministic dense virtual sizing for the headless WGPU device contract and for tests that need to verify allocation intent without allocating real GPU memory.

`TextureDimension::D2Array` uses `TextureDesc::depth` as the array-layer count. `TextureDimension::Cube` also uses `depth` for face count or cube-array face count and must be a multiple of six when the WGPU test device creates a texture. Multisampled textures may not declare mip levels greater than one.

`TextureResidency::SparseReserved` marks a texture as a sparse or virtual reservation instead of an ordinary dense backing allocation. The descriptor still preserves full virtual width, height, layer/depth count, mip count, format, and usage so RenderGraph, streaming, page table, and diagnostics code can reason about the intended resource shape. The current WGPU test backend reports `supports_sparse_texture = false`, so `create_texture(...)` rejects sparse reservations with a structured `InvalidTextureDescriptor` instead of silently allocating dense memory. If a backend later opts in, the WGPU test device stores zero committed bytes for the reservation; residency/page upload ownership must come from a later sparse texture provider rather than from `TextureDesc` itself.

`CommandListCommand` now includes neutral compute work submission. A command list can record `SetPipeline { pipeline }` and `DispatchCompute { x, y, z }`; submit validation requires a bound `PipelineKind::Compute`, non-zero workgroup counts, and a graphics or compute queue. The copy queue rejects compute dispatch explicitly. The WGPU test backend treats this as a contract-level execution path: it validates handles, pipeline kind, and queue usage, then completes the fence without pretending to run an algorithm-specific shader.

## Design And Rationale

The descriptor set is intentionally smaller than WGPU's full format catalog, but it covers the render-main-chain requirements: scene HDR targets, depth prepass targets, reflection/cubemap slots, texture arrays, compute/storage-ready color targets, mipmapped sampled textures, and sparse texture reservations. Compressed imported asset formats still flow through existing graphics asset upload code until the RHI facade needs a broader compressed-format vocabulary.

The WGPU backend advertises graphics, compute, and copy command-list queue classes. Async compute scheduling remains a RenderGraph/SRP policy decision; the backend can still fall async graph lanes back to graphics while exposing compute pipeline and dispatch capability at the RHI boundary.

The public RHI facade exports neutral handles and descriptors only. The old `GpuBuffer = wgpu::Buffer` alias has been removed so framework DTOs cannot leak backend-owned WGPU objects across the `rhi_wgpu` and renderer backend boundary.

## Test Coverage

Focused tests cover descriptor labels/usages, HDR array and cube descriptors, mip storage sizing, sparse reservation virtual sizing, WGPU sparse capability rejection, WGPU descriptor round-trips, invalid MSAA+mip descriptors, invalid cube face counts, WGPU queue capability reporting, and compute command-list dispatch validation.

Focused RHI validation on 2026-06-02 passed with 69 tests, 0 failures, using `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`.
