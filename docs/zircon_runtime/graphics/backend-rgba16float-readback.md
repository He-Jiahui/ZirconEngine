---
related_code:
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_region.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
implementation_files:
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_region.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_region.rs
doc_type: module-detail
---

# RGBA16F Texture Readback

## Purpose

`read_texture_rgba16float_region.rs` is the backend utility for reading `Rgba16Float` texture bytes from a specific mip level and array-layer region. It exists to support Plan 11 / Shader 06 IBL artifact readback without making the environment artifact contract depend on wgpu.

## Contract

`read_texture_rgba16float_region(...)` copies a selected texture region into a padded readback buffer, waits for mapping, strips row padding, and returns tightly packed RGBA16F bytes in layer-major row order.

`read_texture_rgba16float_cube_mip_chain(...)` iterates cube faces first and mip levels second, returning bytes in the same face-major all-mip order used by `SourceCubemapMipChain` and `IblBakeArtifactPayload` PMREM sections. It is a synchronous acquisition helper; future runtime scheduling can batch or pipeline the copies without changing the byte layout.

Environment source/specular/IEM cube textures now include `COPY_SRC` usage so PMREM/IEM data produced or uploaded into those textures can be copied to readback buffers. This only enables acquisition; it does not schedule bake work, issue IBL compute passes, or write cache artifacts by itself.

## Verification

Focused command:

```powershell
cargo test -p zircon_runtime --lib read_texture_rgba16float_region --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-readback-0706 --message-format short --color never -- --nocapture --test-threads=1
```

Result: 2/2 passed. Logs are `docs/tests/runtime/render/plan11_ibl_wgpu_rgba16float_region_readback_helper_cargo_20260706.{out,err}.log` and `.exit.txt`.

## Open Work

Actual PMREM/SH9/IEM GPU compute production, asynchronous scheduling, runtime readback-to-cache dispatch integration, importer/staged artifact production, product second-launch dispatch=0 evidence, RenderDoc/product capture, and full CI remain open. SH9 buffer byte acquisition is tracked in `docs/zircon_runtime/graphics/backend-buffer-readback.md`, and descriptor-driven WGPU artifact section acquisition is tracked in `docs/zircon_runtime/graphics/backend-ibl-artifact-readback.md`.
