---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_prefilter.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_irradiance_sh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_irradiance_cube.wgsl
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
plan_sources:
  - user: 2026-07-06 implement WGPU-to-render-pipeline design from docs/plans/zircon_runtime/render
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - cargo fmt --package zircon_runtime
  - cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-params-check-0706-final --message-format short --color never
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - cargo test -p zircon_runtime --lib ibl_bake_wgpu_command_plan --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-command-plan-0706 --message-format short --color never -- --nocapture --test-threads=1 (built test binary but wrapper timed out before writing an exit file; not counted as a pass)
  - E:\cargo-targets\zircon-ibl-wgpu-command-plan-0706\debug\deps\zircon_runtime-1ff53e05a9088131.exe ibl_bake_wgpu_command_plan --nocapture --test-threads=1 (passed 4/4; log docs/tests/runtime/render/plan11_ibl_wgpu_command_plan_direct_tests_20260706.out.log)
  - cargo test -p zircon_runtime --lib materialization_exposes_owned_cube_storage_texture_array_views --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-command-plan-0706 --message-format short --color never -- --nocapture --test-threads=1 (passed 1/1)
  - E:\cargo-targets\zircon-ibl-wgpu-binding-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_binding --nocapture --test-threads=1 (passed 2/2; log docs/tests/runtime/render/plan11_ibl_wgpu_binding_direct_tests_20260706.out.log)
  - cargo test -p zircon_runtime --lib ibl_bake_wgpu_dispatch --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-dispatch-0706 --message-format short --color never -- --nocapture --test-threads=1 (passed 4/4; 6984 filtered)
  - E:\cargo-targets\zircon-ibl-wgpu-dispatch-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_dispatch --nocapture --test-threads=1 (passed 4/4; log docs/tests/runtime/render/plan11_ibl_wgpu_dispatch_direct_tests_20260706.out.log)
doc_type: module-detail
---

# IBL Bake WGPU Command Plan

## Purpose

`ibl_bake_wgpu_command_plan.rs` is the renderer-local WGPU command/readback contract for Plan 11 / Shader 06 IBL baking. It sits between the shader/kernel plan and the real WGPU binding/executor path.

The module does not create GPU objects or submit work. It converts an `IblBakeArtifactRequest` into a deterministic list of command plans that the binding and executor layers can use to create bind groups, select pipelines, bind per-mip storage views, dispatch compute work, and copy outputs into artifact-aligned readback buffers.

## Related Files

- `ibl_bake_graph_plan.rs` owns RenderGraph resources and async-compute pass names.
- `ibl_bake_compute_executor.rs` owns opt-in executor ids and dispatch audit records.
- `ibl_bake_shader_plan.rs` owns WGSL kernel selection, shader locators, pipeline keys, and dispatch extents.
- `ibl_bake_wgpu_command_plan.rs` owns the WGPU binding ABI, output-view plan, and readback-copy plan.
- `ibl_bake_wgpu_binding.rs` consumes the ABI to create live params buffers, source samplers, bind group layouts, and bind groups.
- `ibl_bake_wgpu_pipeline_cache.rs` owns renderer-lifetime shader module, pipeline layout, and compute pipeline reuse for the command plan's `ComputePipelineCacheKey`.
- `ibl_bake_wgpu_dispatch.rs` consumes the command plan, bind group layout, and pipeline cache to encode dispatch commands, resolve focused graph-context resources, and record dispatch audit entries.
- `ibl_bake_artifact.rs` and `ibl_bake_artifact_readback.rs` own the artifact descriptor, payload section sizes, and readback section kinds.

## Behavior Model

`ibl_bake_wgpu_command_plan_for_request(...)` builds a current artifact descriptor from the request, asks `ibl_bake_shader_plan.rs` for kernel plans, and maps each kernel into an `IblBakeWgpuCommandPlan`.

Every command records:

- the compute kernel kind,
- the shader locator and pipeline cache key from the shader plan,
- the WGSL source used by the WGPU compute pipeline helper,
- the WGSL uniform parameter words and little-endian bytes,
- the output binding class,
- the output resource name and view or buffer shape,
- fixed dispatch groups,
- the readback copies needed to place output bytes into the final `.zribl` payload layout.

The bind-group layout is fixed to four entries:

- binding 0: uniform params buffer,
- binding 1: sampled `texture_cube<f32>` source cubemap,
- binding 2: filtering sampler,
- binding 3: either write-only `texture_storage_2d_array<rgba16float>` or writable storage buffer.

This mirrors the WGSL ABI and avoids inventing a second resource contract inside the eventual executor.

## Uniform Params Layout

`IblBakeWgpuParamsPlan` stores the exact 32-bit words that must be uploaded to binding 0. It deliberately serializes numbers as little-endian bytes instead of relying on a Rust struct layout, because the WGSL structs include explicit padding words and the eventual executor should be able to create a uniform buffer directly from this plan.

The current layouts are:

- PMREM, 32 bytes: `face_size`, `mip_face_size`, `mip_level`, `mip_count`, `sample_count`, padding, `roughness` bits, padding.
- SH9, 16 bytes: `source_face_size`, `sample_face_size`, `source_lod` bits, padding.
- IEM, 16 bytes: `source_face_size`, `irradiance_face_size`, `sample_count`, padding.

`ibl_bake_wgpu_binding.rs` consumes the plan through `create_ibl_bake_wgpu_params_buffer(...)`. The same file also creates the source cubemap sampler with linear min/mag/mipmap filtering, allowing filtered source-mip sampling for FIS and cosine PMREM paths.

## WGPU Binding Bridge

`ibl_bake_wgpu_binding.rs` now consumes this command plan to create live WGPU bind group layouts, params buffers, source samplers, and bind groups. The command plan owns the binding numbers and output kind, while the binding bridge validates that the caller provides a matching storage texture or storage buffer before calling WGPU.

The binding bridge closes params/sampler/layout/bind-group creation as a reusable primitive. The sibling pipeline-cache bridge reuses shader modules, pipeline layouts, and compute pipelines from this plan's `pipeline_key`, while the dispatch bridge encodes compute passes against the live bind group and covers focused PMREM/SH9/IEM graph-context resource resolution/audit recording. Production scheduler integration, readback ownership, and cache artifact writeback remain separate.

## WGPU Dispatch Bridge

`ibl_bake_wgpu_dispatch.rs` is the live WGPU command encoder for this plan. Direct focused tests still create a shader module, pipeline layout, and compute pipeline using `IBL_BAKE_COMPUTE_ENTRY_POINT`; graph-context execution obtains that pipeline from `ibl_bake_wgpu_pipeline_cache.rs` before encoding bind group 0 and `command.dispatch_groups`. The helper rejects zero dispatch groups before WGPU pass creation.

The focused offscreen WGPU tests cover both output classes from this command plan: PMREM mip0 writes a `Rgba16Float` D2Array storage texture view, and SH9 writes a 144-byte storage buffer. The graph-context test additionally builds the IBL bake graph, materializes PMREM outputs, imports the source cubemap view, records a PMREM dispatch through `RenderPassExecutionContext`, and verifies the compute dispatch audit record. These tests still do not read output bytes or write runtime cache artifacts.

## PMREM Storage Texture Views

WGPU has no `texture_storage_cube`, and storage texture views cannot expose all mips for dynamic writes. PMREM therefore uses one command per mip and one storage view per command.

`ibl_bake_storage_texture_view_plan(mip_level)` fixes the view descriptor to:

- `format = Rgba16Float`,
- `dimension = D2Array`,
- `base_mip_level = mip_level`,
- `mip_level_count = 1`,
- `base_array_layer = 0`,
- `array_layer_count = 6`,
- `usage = STORAGE_BINDING`.

The later executor should call `to_wgpu_descriptor()` on this plan when creating the texture view. The module deliberately stores the plan instead of a live `wgpu::TextureView`, because this layer is still pure planning and can be tested without a device.

The graph execution resource table now provides the concrete bridge for the output side: `RenderGraphExecutionResources::owned_texture_view_with_descriptor(...)` can take this descriptor, resolve the owned transient Cube backing, validate format/usage/dimension/mip/layer ranges, and create the WGPU view. That closes the PMREM/IEM output storage-view resource gap. Params-buffer and bind-group construction helpers now exist in `ibl_bake_wgpu_binding.rs`, but they are still not wired into a production executor that allocates graph resources, records compute passes, submits the queue, or drains readback.

## Readback Layout

The artifact payload is face-major and sectioned as PMREM RGBA16F all mips, SH9 bytes, then optional IEM RGBA16F bytes. A whole-mip D2Array copy would not directly match that layout, so the command plan emits explicit copy plans:

- PMREM: one texture copy per face for each mip, with `artifact_byte_offset` derived from `source_cubemap_face_mip_offset(...) * 8`.
- SH9: one 144-byte buffer copy from the SH9 storage buffer.
- IEM: one texture copy per face for the 32x32 irradiance cubemap.

Texture copies include both unpadded and padded row-byte counts. `padded_byte_len` follows `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`; `unpadded_byte_len` is the actual payload byte count. The eventual readback owner must strip row padding before passing sections into `IblBakeArtifactReadbackSections`.

## Design And Rationale

The command plan intentionally reuses `ibl_bake_compute_kernel_plans_for_request(...)` instead of recalculating shader selection or dispatch sizes. That keeps pipeline labels, pipeline cache keys, and dispatch groups in one owner.

The module also keeps PMREM, SH9, and IEM output planning in one file because those commands share the same compute ABI but differ in output binding kind. Splitting before the executor exists would create more surface without reducing current complexity.

## Edge Cases And Constraints

- Requests that omit PMREM or IEM do not emit readback copies for those outputs.
- SH9-only requests place the SH9 section at artifact byte offset 0.
- PMREM requests with multiple mips emit one command per mip and six readback copies per mip.
- The plan assumes RGBA16F payload texels are 8 bytes and SH9 output is the documented 9 `vec4<f32>` layout.
- The plan does not handle the final GPU-side 1x1 six-face average, optimized SH9 reduction, or queue/drain ordering.

## Test Coverage

The direct focused lib-test run on 2026-07-06 passed 4/4 before the uniform-params follow-up:

- `bind_group_layout_entries_match_compute_shader_abi`,
- `command_plan_uses_per_mip_d2_array_storage_views`,
- `readback_plan_uses_face_major_artifact_offsets`,
- `command_plan_omits_unrequested_outputs`.

The uniform-params follow-up adds `command_plan_serializes_wgsl_uniform_params_in_layout_order`, covering PMREM mip0/mip7, SH9, IEM, and little-endian serialization. Current local evidence is a fresh `cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1` pass under `E:\cargo-targets\zircon-ibl-wgpu-params-check-0706-final` after fixing the WGPU sampler `mipmap_filter` type. A focused `cargo test -p zircon_runtime --lib command_plan_serializes_wgsl_uniform_params_in_layout_order ...` did not reach this assertion in the current dirty workspace: the full `zircon_runtime` lib-test tree is large and intermittently failed before the focused test with unrelated `runtime_absorption/structure_convention` generated-test diagnostics or process exit `-1`. That focused test is therefore not counted as green yet.

The first Cargo wrapper built the test binary but exceeded the UI tool window before writing an exit file; it is recorded as not counted. The generated lib-test program was then executed directly from `E:\cargo-targets\zircon-ibl-wgpu-command-plan-0706` and returned exit code 0.

The follow-up output-view resource bridge was verified by `materialization_exposes_owned_cube_storage_texture_array_views`, which passed 1/1 through an offscreen WGPU backend. That test materializes an RGBA16F Cube transient, creates the D2Array storage view required by the PMREM command plan, and locks invalid mip, array-layer, and usage errors.

The live binding bridge was verified by the direct `ibl_bake_wgpu_binding` lib-test run, which passed 2/2. It creates real WGPU bind groups for the PMREM storage-texture path and the SH9 storage-buffer path, and checks output-kind mismatches before WGPU validation.

The live dispatch bridge was verified by focused `ibl_bake_wgpu_dispatch` tests, which passed 4/4. The tests create PMREM and SH9 compute pipelines from the command plan WGSL sources, encode both dispatches into one command buffer, submit the queue, poll the device, reject a zero dispatch group before opening a WGPU pass, and verify the focused PMREM graph-context dispatch/audit path.

## Open Issues

The next implementation slices still need production executor wiring for source cubemap/output resource lookup, params/readback buffer ownership in graph execution, compute pipeline cache lookup/reuse, queue scheduling, asynchronous readback draining, cache writeback from GPU outputs, product second-launch dispatch=0 evidence, RenderDoc/product capture, strict roughness/SSIM/seam validation, optimized SH9 reduction, and full CI.
