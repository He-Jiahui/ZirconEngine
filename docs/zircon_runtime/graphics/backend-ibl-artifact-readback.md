---
related_code:
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_region.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_buffer_bytes.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs
implementation_files:
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
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

`read_ibl_bake_artifact_sections.rs` is the WGPU acquisition bridge between backend resources and the render-core artifact readback DTO. It accepts an `IblBakeArtifactDescriptor` plus the WGPU resources that contain PMREM, SH9, and optional IEM outputs, then returns `IblBakeArtifactReadbackSections`.

## Contract

`IblBakeArtifactWgpuReadbackResources` names the three optional WGPU resource slots:

- PMREM `Rgba16Float` cubemap texture,
- SH9 compute-output buffer using `array<vec4<f32>, 9>`,
- optional IEM `Rgba16Float` irradiance cubemap texture.

`read_ibl_bake_artifact_wgpu_sections(...)` reads only the resources required by the descriptor contents. PMREM uses `read_texture_rgba16float_cube_mip_chain(...)` with the descriptor face size and mip count. SH9 uses `read_buffer_sh9_f32x4_bytes(...)`, which returns the 144-byte layout required by `IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES`. IEM uses the fixed `SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE` and a single mip.

The bridge does not validate section lengths itself. It returns `IblBakeArtifactReadbackSections`, leaving descriptor-sized section validation and payload assembly to `ibl_bake_artifact_readback.rs`, and cache writeback to `asset/artifact/ibl_bake_artifact_runtime_writeback.rs`.

The 2026-07-08 seam guard writes a synthetic seam-stress PMREM payload into an actual `Rgba16Float` six-layer WGPU texture, writes the matching SH9 bytes into a WGPU buffer, then reads both resources through `read_ibl_bake_artifact_wgpu_sections(...)`. The assembled payload must match the original artifact bytes exactly before it is applied back to the source-cubemap model. This locks the backend texture readback byte order to the same face-major/mip-major artifact layout used by the CPU PMREM and runtime cache paths.

## Verification

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

This module still assumes the PMREM texture, SH9 buffer, and optional IEM texture already exist. Backend PMREM texture readback layout and seam preservation are now guarded, but live compute-produced PMREM readback, offline bake production, async scheduling, queueing readback after compute completion, runtime readback-to-cache dispatch integration, importer/staged artifact production, product second-launch dispatch=0 evidence, RenderDoc/product capture, and full CI remain open.
