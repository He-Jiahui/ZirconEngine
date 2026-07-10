---
related_code:
  - zircon_runtime/src/asset/assets/texture/cube_asset.rs
  - zircon_runtime/src/asset/assets/texture/array_asset.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_plugins/texture_importer/runtime/src/cubemap.rs
  - zircon_plugins/texture_importer/runtime/src/array.rs
  - zircon_plugins/texture_importer/runtime/src/manifest_source.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/plugin.toml
implementation_files:
  - zircon_runtime/src/asset/assets/texture/cube_asset.rs
  - zircon_runtime/src/asset/assets/texture/array_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_plugins/texture_importer/runtime/src/cubemap.rs
  - zircon_plugins/texture_importer/runtime/src/array.rs
  - zircon_plugins/texture_importer/runtime/src/manifest_source.rs
plan_sources:
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - dev/cmft/src/cmft/image.cpp
tests:
  - zircon_runtime/tests/runtime_texture_cube_resource_contract.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs
  - zircon_plugins/texture_importer/runtime/src/tests/cubemap.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/texture_slot_diagnostics.rs
doc_type: module-detail
---

# Cubemap And Texture Array Authoring

## Purpose

This module implements Plan 13 TX-M3 authoring and assembly without introducing a second runtime texture resource type. `CubemapAsset` and `Texture2DArrayAsset` describe source manifests and validate resolved layers. Successful assembly produces the existing canonical `TextureAsset`, so artifact caching, project resource registration, upload readiness, residency, and GPU allocation keep one authority.

## Asset Contracts

`CubemapAsset` stores a `TextureAssetDescriptor`, a `CubemapSourceLayout`, and source references. Layout source counts are strict:

- `six_files` requires six sources in `+X, -X, +Y, -Y, +Z, -Z` order.
- `horizontal_cross`, `vertical_cross`, and `equirectangular` require one source.

Resolved faces must be square single-layer RGBA8 textures with equal dimensions, format, color space, mip count, and payload length. Mips are repacked from per-face mip chains to the GPU upload order: mip first, then all six faces. The assembled descriptor is `RenderImageDimension::Cube` with six layers.

`Texture2DArrayAsset` accepts references to separate layer images or one `SlicedFromImage` source. Resolved layers must be single-layer RGBA8 2D textures with equal dimensions, format, color space, mip count, and payload length. Mips are likewise repacked to mip-first/layer-second order. The assembled descriptor remains `D2` with `array_layer_count > 1`; GPU view creation maps that shape to `TextureViewDimension::D2Array` over every layer.

## Manifest Import

The texture importer registers `.zcube` and `.zarray` source manifests. Source paths are project-relative, may include child directories, and reject absolute paths, URI paths, and parent traversal. This keeps multi-source reads bounded to the manifest's project subtree.

A six-file cubemap manifest is:

```toml
layout = "six_files"
sources = ["posx.png", "negx.png", "posy.png", "negy.png", "posz.png", "negz.png"]
```

Cross manifests use one image:

```toml
layout = "horizontal_cross"
sources = ["environment_cross.png"]
```

The horizontal and vertical tile locations follow `dev/cmft/src/cmft/image.cpp::imageCubemapFromCross`. A vertical cross rotates the `-Z` tile by 180 degrees, matching cmft's `FLIP_X | FLIP_Y` operation. This is required to preserve front/back and top/bottom orientation across skybox and reflection sampling.

Equirectangular manifests require a 2:1 image. Projection uses the shared `cubemap_texel_direction` and `equirect_uv_from_direction` contract, then bilinearly samples with longitude wrapping and latitude clamping. `cubemap_face_size` may override the default `height / 2` face size through import settings.

Texture arrays support either separate images:

```toml
sources = ["layer0.png", "layer1.png", "layer2.png"]
```

or one vertical strip:

```toml
source = "layers.png"
row_count = 4
```

Exactly one of `row_count` and `row_height` is accepted for strip slicing.

## HDR And IBL Boundary

`.zcube` manifests assemble ordinary RGBA8 GPU cubemaps. They do not treat source mips as roughness-prefiltered reflection data. Real HDR environment lighting continues to use the dedicated `.hdr/.exr` import route, which preserves linear float data and stages a source `.zcube` plus independently generated `.zribl` PMREM/SH9/IEM companion. This separation prevents a display mip chain or author-provided container mip chain from being mistaken for GGX PMREM.

## Material Slot Dimensions

Shader slot kinds `texture_2d`, `texture_2d_array`, `texture_cube`, and `texture_3d` map to `RenderMaterialTextureDimension`. Resource resolution derives the actual dimension from `RenderImageDescriptor`, including the distinction between one-layer D2 and multi-layer D2Array.

Mismatch is non-blocking: the slot receives `RenderMaterialTextureSlotFallbackReason::DimensionMismatch`, the readiness report receives `TextureDimensionMismatch`, and `RenderMaterialTextureSlotState` records expected and actual dimensions. The material remains renderable through its normal fallback policy.

## Validation Status

The 2026-07-10 milestone testing stage passed the following focused checks:

- Runtime cubemap/array assembly contract: 8/8 tests passed, covering face/layer order, mismatched dimensions and formats, Cube descriptor validation, and complete six-face upload readiness.
- Texture importer manifest contract: 5/5 tests passed, covering six-file order, horizontal cross order, vertical `-Z` rotation, equirectangular six-face output, separate array layers, and vertical-strip slicing.
- Runtime private tests: D2Array GPU view selection passed 1/1; 2D-to-Cube material dimension fallback passed 1/1.
- Scoped Rust formatting, runtime asset-schema naming, and the repository plugin structure audit passed. The existing workspace warning set remains outside this slice.
