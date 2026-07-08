---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
plan_sources:
  - user: 2026-07-06 implement WGPU-to-render-pipeline design from docs/plans/zircon_runtime/render
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - cargo test -p zircon_runtime --lib ibl_bake_wgpu_binding --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-binding-0706 --message-format short --color never -- --nocapture --test-threads=1 (built the test binary but exceeded the tool window; not counted as a pass)
  - E:\cargo-targets\zircon-ibl-wgpu-binding-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_binding --nocapture --test-threads=1 (passed 2/2; log docs/tests/runtime/render/plan11_ibl_wgpu_binding_direct_tests_20260706.out.log)
  - E:\cargo-targets\zircon-ibl-wgpu-dispatch-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_dispatch --nocapture --test-threads=1 (passed 4/4; log docs/tests/runtime/render/plan11_ibl_wgpu_dispatch_direct_tests_20260706.out.log)
doc_type: module-detail
---

# IBL Bake WGPU Binding

## Purpose

`ibl_bake_wgpu_binding.rs` is the renderer-local live WGPU binding bridge for Plan 11 / Shader 06 IBL baking. It consumes `IblBakeWgpuCommandPlan` data and creates the concrete objects needed immediately before compute command encoding:

- bind group layouts for storage-texture and storage-buffer outputs,
- a uniform params buffer initialized from the command plan's WGSL parameter words,
- the filtering source cubemap sampler,
- a four-entry WGPU bind group for params, source cubemap, sampler, and output.

This module does not compile compute pipelines, begin compute passes, dispatch workgroups, submit queues, allocate readback buffers, or write cache artifacts. Pipeline cache ownership lives in `ibl_bake_wgpu_pipeline_cache.rs`, while command encoding lives in `ibl_bake_wgpu_dispatch.rs`; production executor/readback wiring remains a later slice.

## Binding ABI

The binding bridge imports the binding numbers from `ibl_bake_wgpu_command_plan.rs`, so the command plan, WGSL ABI, and live bind group creation share the same contract:

- binding 0: uniform params buffer,
- binding 1: sampled source `texture_cube<f32>`,
- binding 2: filtering sampler,
- binding 3: output storage texture or output storage buffer.

Two bind group layouts are created up front because PMREM/IEM write `texture_storage_2d_array<rgba16float, write>`, while SH9 writes a writable storage buffer.

## Output Validation

`create_ibl_bake_wgpu_bind_group(...)` validates the command's output plan before it calls WGPU:

- storage texture commands must use the D2Array output kind,
- storage buffer commands must use the storage-buffer output kind,
- the caller-provided output resource kind must match the command plan.

This catches executor wiring mistakes before they become lower-level WGPU validation failures, and it prevents a PMREM/IEM command from accidentally binding an SH9 buffer or vice versa.

## Verification

Focused WGPU tests use `RenderBackend::new_offscreen()` to create real device objects. The direct lib-test run passed:

- `bind_groups_create_for_storage_texture_and_storage_buffer_outputs`,
- `bind_group_creation_rejects_output_kind_mismatches_before_wgpu_validation`.

The first Cargo wrapper timed out after building the test binary and is not counted. The direct binary run returned exit code 0 and wrote `docs/tests/runtime/render/plan11_ibl_wgpu_binding_direct_tests_20260706.out.log`.

## WGPU Dispatch Bridge

`ibl_bake_wgpu_dispatch.rs` consumes the bind groups created here and the shader source carried by the command plan. Focused offscreen WGPU tests now create PMREM and SH9 bind groups, create matching compute pipelines, encode both dispatches, submit the command buffer, and poll the device. Graph-context dispatch uses `ibl_bake_wgpu_pipeline_cache.rs` so PMREM/SH9/IEM passes resolve materialized source/output resources, reuse renderer-owned pipelines, and record compute dispatch audit entries.

## Open Issues

The helper closes live params/sampler/layout/bind-group creation as a reusable primitive. The sibling dispatch and pipeline-cache helpers close renderer-lifetime pipeline reuse, command-encoder dispatch, and focused PMREM/SH9/IEM graph-context resource resolution/audit recording. The production executor still needs full scheduler integration, readback buffers, async readback drain, runtime cache artifact writeback from GPU outputs, product second-launch dispatch=0 proof, RenderDoc/product evidence, starter SH9 reduction replacement, and strict roughness/SSIM/seam plus full CI gates.
