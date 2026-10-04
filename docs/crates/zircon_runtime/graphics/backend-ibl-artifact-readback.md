---
related_code:
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/resources.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/staging.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/pending.rs
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_diagnostics.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device/diagnostics.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_region.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_buffer_bytes.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs
implementation_files:
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/resources.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/staging.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections/pending.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/mod.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
  - docs/tests/runtime/render/plan11_ibl_wgpu_artifact_sections_readback_helper_cargo_20260706.out.log
  - docs/tests/runtime/render/plan11_ibl_wgpu_artifact_sections_readback_helper_cargo_20260706.err.log
  - docs/tests/runtime/render/plan11_ibl_wgpu_artifact_sections_readback_helper_cargo_20260706.exit.txt
  - E:\cargo-targets\zircon-ibl-wgpu-readback-seam-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::backend::render_backend::read_ibl_bake_artifact_sections::tests::readback_sections_preserve_pmrem_seams_after_wgpu_texture_roundtrip --exact --nocapture --test-threads=1
doc_type: module-detail
---

# IBL Artifact WGPU Readback Sections

## Purpose

`read_ibl_bake_artifact_sections.rs` is the WGPU acquisition bridge between backend resources and the render-core artifact readback DTO. The product path accepts an `IblBakeArtifactDescriptor` plus the WGPU resources that contain PMREM, SH9, and optional IEM outputs, registers bounded section requests with the central product diagnostic scope, and returns a CPU-only pending aggregator. The synchronous section reader remains test-only.

## Contract

`IblBakeArtifactWgpuReadbackResources` names the three optional WGPU resource slots:

- PMREM `Rgba16Float` cubemap texture,
- SH9 compute-output buffer using `array<vec4<f32>, 9>` plus the exact source `offset` and `size` window,
- optional IEM `Rgba16Float` irradiance cubemap texture.

`request_ibl_bake_artifact_wgpu_readback(...)` first resolves every descriptor-required resource so a missing PMREM, SH9, or IEM input cannot leave a partially registered artifact. It then registers PMREM as six face-major/mip-major `Rgba16Float` regions, SH9 as one aligned buffer request using the supplied source window, and IEM as six single-mip `Rgba16Float` faces. The SH9 window is preserved into both staging `copy_buffer_to_buffer` and the product diagnostic buffer admission; it is never silently replaced with offset `0`. All requests join the active product diagnostic batch, its serial scene tail, and its scene submission ticket.

`IblBakeArtifactWgpuPendingReadback` holds no WGPU buffer, queue, map callback queue, or poll authority. Router callbacks validate every slot and expected byte length, preserve face-major/mip-major ordering, and record the first terminal error. Completion produces `IblBakeArtifactReadbackSections` only after every required slot terminates; any rejection, map failure, size mismatch, invalid slot, or duplicate delivery fails the entire artifact so partial cache bytes are never published. Cache writeback remains owned by `asset/artifact/ibl_bake_artifact_runtime_writeback.rs`.

The SH9 resource owner validates the descriptor byte length, checked `offset + size`, and physical buffer bounds before either synchronous staging or product diagnostic admission. The native `Rgba16Float` diagnostic source validates a single-sample D2 texture, exact format, `COPY_SRC`, mip/layer bounds, non-zero extent, and eight bytes per texel before admission. It only appends copy commands to the caller-owned diagnostic tail. Submission and completion remain owned by `WgpuRenderDevice`.

The 2026-07-08 seam guard writes a synthetic seam-stress PMREM payload into an actual `Rgba16Float` six-layer WGPU texture, writes the matching SH9 bytes into a WGPU buffer, then reads both resources through `read_ibl_bake_artifact_wgpu_sections(...)`. The assembled payload must match the original artifact bytes exactly before it is applied back to the source-cubemap model. This locks the backend texture readback byte order to the same face-major/mip-major artifact layout used by the CPU PMREM and runtime cache paths.

## Verification

Current 2026-08-30 source evidence: scoped Rust formatting parses the product bridge, range-carrying resource packet, staging copy path, and pending aggregator; source contracts confirm the pending owner contains no `wgpu::Buffer`, `map_async`, `device.poll`, or `queue.submit`, native diagnostic preparation owns no submit/poll, and the aggregator preserves face-major/mip-major PMREM ordering while failing the whole artifact on a terminal error. Cargo, real WGPU, PNG, RenderDoc, profiler, and power validation for this cutover have not run.

Focused command:

```powershell
cargo test -p zircon_runtime --lib read_ibl_bake_artifact_sections --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-sections-readback-0706 --message-format short --color never -- --nocapture --test-threads=1
```

Result: 3/3 passed. Logs are `docs/tests/runtime/render/plan11_ibl_wgpu_artifact_sections_readback_helper_cargo_20260706.{out,err}.log` and `.exit.txt`. The focused tests cover descriptor preservation, descriptor-content resource requirements, and missing-resource reporting before a copy is attempted.

Additional seam-focused validation:

```powershell
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-ibl-wgpu-readback-seam-0708'; $env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib readback_sections_preserve_pmrem_seams_after_wgpu_texture_roundtrip --no-default-features --features core-min --locked --jobs 1 --color never -- --nocapture --test-threads=1
E:\cargo-targets\zircon-ibl-wgpu-readback-seam-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::backend::render_backend::read_ibl_bake_artifact_sections::tests::readback_sections_preserve_pmrem_seams_after_wgpu_texture_roundtrip --exact --nocapture --test-threads=1
```

The Cargo wrapper built the final lib-test binary under the target above but exceeded the tool window twice during compile/link, so it is not counted as a passing Cargo-wrapper result. Direct execution of the final generated test binary passed 1/1 with 7308 filtered tests and a 1.81s test body. The test verifies exact PMREM/SH9 artifact byte preservation across WGPU texture/buffer readback, then reapplies the payload and checks mid/rough cube-edge seam statistics within `0.003`.

## Open Work

This module still assumes the PMREM texture, SH9 buffer, and optional IEM texture already exist. Product async scheduling and runtime readback-to-cache dispatch are now wired through the scene diagnostic owner, but current-source Cargo/WGPU validation, product second-launch `dispatch=0` evidence, offline/importer bake production, RenderDoc/product capture, screenshot/seam acceptance, performance profiling, and full CI remain open.
