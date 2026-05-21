---
related_code:
  - dev/bevy/crates/bevy_image/src/lib.rs
  - dev/bevy/crates/bevy_image/src/image.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/core/framework/render/image/asset_usage.rs
  - zircon_runtime/src/core/framework/render/image/color_space.rs
  - zircon_runtime/src/core/framework/render/image/descriptor.rs
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/core/framework/render/image/fallback.rs
  - zircon_runtime/src/core/framework/render/image/sampler.rs
  - zircon_runtime/src/core/framework/render/image/usage.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/metadata.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_registry.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/core/framework/render/image/asset_usage.rs
  - zircon_runtime/src/core/framework/render/image/color_space.rs
  - zircon_runtime/src/core/framework/render/image/descriptor.rs
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/core/framework/render/image/fallback.rs
  - zircon_runtime/src/core/framework/render/image/sampler.rs
  - zircon_runtime/src/core/framework/render/image/usage.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/metadata.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
plan_sources:
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_texture_metadata_exposes_image_contract
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_texture_without_descriptor_uses_payload_metadata
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_texture_descriptor_overrides_payload_defaults
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Render Image Contracts

## Purpose

`zircon_runtime::core::framework::render::image` owns the neutral image and sampler vocabulary that texture assets project into before the concrete renderer prepares GPU textures. This mirrors Bevy's split where `bevy_image` owns `Image`, image loaders, sampler settings, texture formats, and render asset usage, while `bevy_render` later extracts and prepares those images for GPU use.

The Zircon module deliberately does not parse file formats, watch files, allocate WGPU textures, or resolve renderer fallback resources. Asset import remains under `zircon_runtime::asset`, and concrete GPU texture residency remains under `zircon_runtime::graphics`.

## Product Surface

`RenderImageDescriptor` is the cross-layer handoff for texture-like assets. It carries extent, depth or array layers, dimension, format, color space, sampler, usage flags, asset residency intent, mip count, array layer count, and fallback kind.

The extent fields preserve the Bevy/WGPU distinction between a 3D texture depth and 1D/2D array layers. `TextureAssetDescriptor` normalizes these fields before projection: 3D textures force `array_layer_count` to `1`, while 1D/2D array textures require `array_layer_count` and `depth_or_array_layers` to agree.

`RenderSamplerDescriptor` captures address modes and min/mag/mipmap filters. The default is clamp-to-edge linear sampling, matching the current default product texture behavior. Nearest sampling and repeat modes remain expressible without forcing an importer or renderer policy into this framework module.

`RenderImageUsage` records the intended renderer-side use: sampled, storage, render target, copy source, and copy destination. `RenderImageAssetUsage` records whether image data is expected to stay in the main world, render world, or both after preparation.

`RenderImageFallbackKind` gives the renderer and diagnostics a typed fallback class: missing image, opaque white, transparent black, or normal map. The framework contract only names the fallback; the actual fallback texture lives in the graphics resource system.

## Asset Projection

`TextureAsset::render_image_descriptor()` is the asset-facing projection. Container texture payloads can expose encoded format, mip count, and array layers even when the current GPU upload path cannot yet prepare that container directly. Descriptor overrides can request HDR formats, storage usage, custom asset usage, 3D dimensions, or different fallback classes.

The render image contract is intentionally richer than the current minimal upload path. This prevents the default renderer from treating every texture as a flat RGBA8 sampled image and gives later KTX2/DDS/HDR work a stable target without changing material, sprite, or UI contracts again.

## Current Limits

This module is not a full Bevy `ImagePlugin`. It does not register loaders, own dynamic texture atlases, generate mips, transcode compressed containers, or create bind groups. Those remain asset/importer and graphics-resource milestones.

The current concrete renderer can still fall back when a descriptor is valid but the payload is not upload-ready. That fallback must be reported by material, sprite, or texture resource diagnostics rather than hidden in the image descriptor itself.

## Test Coverage

`render_product_assets_texture_metadata_exposes_image_contract` proves explicit texture metadata projects width, height, array layers, format, color space, usage, asset usage, mip count, and fallback kind.

`render_product_assets_texture_without_descriptor_uses_payload_metadata` proves legacy/container payload metadata still projects into the image contract.

`render_product_assets_texture_descriptor_overrides_payload_defaults` proves descriptor overrides can request HDR color space, 3D texture semantics, storage usage, and normalized mip/layer defaults.
