---
related_code:
  - zircon_runtime/src/graphics/backend/render_backend/read_buffer_bytes.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/backend/render_backend/read_buffer_bytes.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/mod.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - zircon_runtime/src/graphics/backend/render_backend/read_buffer_bytes.rs
  - docs/tests/runtime/render/plan11_ibl_wgpu_buffer_readback_helper_cargo_20260706.out.log
  - docs/tests/runtime/render/plan11_ibl_wgpu_buffer_readback_helper_cargo_20260706.err.log
  - docs/tests/runtime/render/plan11_ibl_wgpu_buffer_readback_helper_cargo_20260706.exit.txt
doc_type: module-detail
---

# Backend Buffer Readback

## Purpose

`read_buffer_bytes.rs` is the narrow WGPU backend utility for copying a GPU buffer byte range into a map-readable staging buffer and returning tightly packed bytes. It exists so Plan 11 / Shader 06 IBL artifact acquisition can read the SH9 compute output without putting wgpu types into the render artifact contract.

## Contract

`read_buffer_bytes(...)` validates the requested copy length, records a `copy_buffer_to_buffer` operation with the shared RenderDoc readback marker, submits the copy, waits for map completion, and returns the mapped bytes.

`read_buffer_f32x4_array_bytes(...)` is the typed layout helper for compute outputs stored as an array of `vec4<f32>`. `read_buffer_sh9_f32x4_bytes(...)` fixes the IBL irradiance readback size to nine `vec4<f32>` values, or 144 bytes, matching `IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES`.

The module is intentionally backend-only. It does not decode SH coefficients, assemble artifact payload sections, write `.zircon-cache` files, or schedule the IBL compute passes that will produce the SH9 buffer.

## Verification

Focused command:

```powershell
cargo test -p zircon_runtime --lib read_buffer_bytes --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-buffer-readback-0706 --message-format short --color never -- --nocapture --test-threads=1
```

Result: 2/2 passed. Logs are `docs/tests/runtime/render/plan11_ibl_wgpu_buffer_readback_helper_cargo_20260706.{out,err}.log` and `.exit.txt`. The tests cover SH9 size parity with the artifact layout and zero/unaligned WGPU copy-size rejection.

## Open Work

Actual PMREM/SH9/IEM GPU compute production, asynchronous readback scheduling, runtime readback-to-cache dispatch integration, importer/staged artifact production, product second-launch dispatch=0 evidence, RenderDoc/product capture, and full CI remain open. Descriptor-driven assembly of WGPU PMREM/SH9/IEM resources into readback sections is documented in `docs/zircon_runtime/graphics/backend-ibl-artifact-readback.md`.
