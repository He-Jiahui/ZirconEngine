---
related_code:
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/descriptor/settings.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/assets/texture/upload_support/compressed.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs
  - zircon_runtime/tests/runtime_texture_cube_resource_contract.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/descriptor/settings.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/assets/texture/upload_support/compressed.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs
plan_sources:
  - user: 2026-07-05 cubemap skybox/reflection mosaic correction and cmft/Unreal mip design request
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/tests/runtime_texture_cube_resource_contract.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/upload_support/tests.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs
doc_type: module-detail
---

# Cube Texture Resource Contract

## Purpose

This document records the EC-M1c resource-contract layer for real cubemap rendering. It does not implement the final skybox shader or GGX PMREM generation. It gives the runtime a valid path to describe, validate, upload, and view cube textures so later `equirect_to_cube.wgsl`, source-skybox sampling, and PMREM passes can stop using the sampled equirectangular storage-buffer bridge.

## Related Files

`RenderImageDimension::Cube` is defined in `core/framework/render/image/dimension.rs`. Texture import settings parse `dimension = "cube"` and the `cubemap` alias in `asset/assets/texture/descriptor/settings.rs`. Descriptor normalization and validation live in `asset/assets/texture/descriptor.rs`.

Upload readiness is split by payload: uncompressed RGBA8 checks live in `asset/assets/texture/upload_support.rs`, and compressed container shape checks live in `asset/assets/texture/upload_support/compressed.rs`. Concrete WGPU texture and view creation live in `graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs`.

## Behavior Model

`RenderImageDimension::Cube` means a WGPU 2D texture with a layer count that is a non-zero multiple of six. A single cubemap uses six layers in cmft/wgpu face order. A cube array uses `6 * cube_count` layers.

Descriptor normalization follows these rules:

- `dimension = "cube"` defaults `array_layer_count` and `depth_or_array_layers` to six when no explicit layer count is provided.
- `dimension = "cubemap"` is accepted as an import-settings alias for `Cube`.
- Explicit cube layer metadata must agree between `array_layer_count` and `depth_or_array_layers`.
- The layer count must be a non-zero multiple of six, otherwise `TextureDescriptorError::CubeLayerCount` is returned.

RGBA8 upload readiness additionally requires square faces and a complete payload for every face/layer/mip. Compressed container readiness now accepts square cube shapes with complete `6N` layers, but the existing compressed upload path still rejects `mip_count > 1`; generated PMREM mip chains remain later engine output, not a promise that imported compressed cubemaps with mips are fully resident today.

## GPU View Mapping

WGPU has no separate `TextureDimension::Cube`; cube textures are created as `TextureDimension::D2` with six or more array layers. The renderer maps the dimension as follows:

- `D1` -> `TextureDimension::D1`, `TextureViewDimension::D1`
- `D2` -> `TextureDimension::D2`, material texture view remains `TextureViewDimension::D2` over layer 0
- `D3` -> `TextureDimension::D3`, `TextureViewDimension::D3`
- `Cube` -> `TextureDimension::D2`, `TextureViewDimension::Cube` for six layers or `CubeArray` for more than six

Keeping ordinary material `D2` textures as one-layer `D2` views is intentional. It avoids changing the existing material texture ABI while adding the cube view path needed for environment resources.

## Design And Rationale

The earlier sampled HDRI path improved resolution but still kept environment data in a custom buffer. That made skybox sampling and roughness mip selection structurally different from the final plan. This contract moves the resource layer to native cube texture semantics without yet changing the render graph or shader sampling path.

The split follows the engine structure rules: neutral image vocabulary stays in `core/framework/render`, import/metadata validation stays in `asset`, and WGPU-specific texture/view creation stays in `graphics`. No compatibility facade or parallel old cubemap type is introduced.

## Edge Cases And Constraints

Cube layer counts are normalized defensively, but renderer code should still provide exact layer counts. A malformed descriptor with `Cube` and fewer than six layers is treated as at least six for view construction only after descriptor normalization should already have made the shape valid.

Post-process LUT resource code maps `Cube` to the underlying WGPU `D2` dimension only to keep the exhaustive dimension contract compile-clean. Cube LUTs are not a post-process LUT feature in this slice.

This layer does not generate blurred mipmaps. EC-M1d now provides a CPU source-cubemap bridge with cmftStudio-style radiance mips for HDRI screenshot validation, while the production cmft/Unreal-style GGX filtered mip chain remains shader 06 EC-M2.

## Test Coverage

The public integration test `runtime_texture_cube_resource_contract.rs` verifies:

- `dimension = "cube"` normalizes to six faces,
- the `cubemap` alias rejects a five-layer shape,
- a square six-face RGBA8 payload is upload-ready.

Private module tests also cover the GPU view mapping and the existing D2 material view contract, but the full Windows lib-test harness is too heavy for every implementation slice. The focused validation for this slice was:

```powershell
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never
cargo test -p zircon_runtime --test runtime_texture_cube_resource_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1
```

Both commands passed. A broader `cargo check --tests` is still blocked by an unrelated existing integration-test field mismatch in `m1_runtime_editor_boundary_contract.rs`, so it is not counted as acceptance for this slice.

## Open Issues

EC-M1d has connected source cubemap allocation/population from HDR pixels and skybox/PBR shader sampling from `texture_cube` for the manual HDRI validation path. Remaining follow-up is the hard deletion of sampled-equirect environment buffers, then EC-M2 GGX filtered importance sampling, SH9, BRDF LUT, RGBA16F PMREM storage, and the final quantitative 8x8 HDRI PBR acceptance tests.
