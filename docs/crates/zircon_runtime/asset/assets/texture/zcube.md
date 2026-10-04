---
related_code:
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/rgba16f.rs
implementation_files:
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
tests:
  - zircon_runtime/tests/runtime_texture_external_cubemap_source_only_contract.rs
  - zircon_runtime/tests/runtime_texture_zcube_staged_ibl_bundle_contract.rs
  - zircon_runtime/tests/runtime_texture_zcube_source_cubemap_contract.rs
plan_sources:
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
doc_type: module-detail
---

# Zcube Source Cubemap Texture

## Purpose

`zcube.rs` defines the asset-layer container for Zircon source cubemaps. It is the TX-M3 source side of the environment IBL chain: `.hdr` / `.exr` import or staged build code can write a `.zcube` texture asset that contains the source cubemap mip pyramid in face-major RGBA16F form.

The important boundary is that `.zcube` is not a PMREM artifact. It stores source mips for skybox minification, future FIS source LOD, and diagnostics. Reusable reflection bake output still belongs in `.zribl`, which carries `IblBakeKey`, algorithm version, PMREM, SH9, and optional IEM sections.

## Related Files

The source mip data comes from `SourceCubemapMipChain::source_texels()` in `source_cubemap.rs`. Shared RGBA16F packing lives in `environment/rgba16f.rs`, so `.zcube` and `.zribl` agree on little-endian half-float texel layout without duplicating conversion code.

`texture_asset_from_source_cubemap_zcube(...)` wraps those source mips in a `TextureAsset::Container` payload. The payload format is `zircon/zcube-source-cubemap-rgba16f-v1`; the render descriptor describes the decoded target shape as linear `rgba16float`, cube dimension, six layers, and the source mip count.

`external_source_cubemap.rs` is the companion guard for DDS/KTX source containers before they are converted into `.zcube`. It recognizes valid six-face DDS/KTX source cubemaps, requires a complete source mip chain, and blocks generic upload readiness with a source-only reason. That keeps external source containers on the same IBL bake path as native `.zcube` instead of allowing them to masquerade as PMREM or material texture payloads.

`ibl_source_cubemap_staging.rs` is the asset artifact-side writer for staged `.zcube` files. It writes source cubemap bytes under `render/ibl-source/...` and can write the matching `.zribl` PMREM/SH9 companion under `render/ibl-derived/...` for the same `IblBakeArtifactRequest`.

## Behavior Model

The `.zcube` byte stream starts with a fixed 32-byte header:

- magic `ZRZCUBE1`,
- format version `1`,
- face size,
- mip count,
- face count `6`,
- texel format `RGBA16F`,
- contents bit `source mips`.

The payload follows immediately after the header. Texels are stored face-major, with each face containing mip 0 through the final 1x1 mip. Each texel is RGBA16F little-endian. Decoder validation rejects truncated headers, wrong magic, unsupported versions, non-six-face payloads, non-RGBA16F texel formats, non-source contents, mismatched texture metadata, and wrong payload lengths.

## Design and Rationale

The payload format remains a custom source-container marker rather than `rgba16float` because the bytes include a Zircon header and because direct material texture upload would bypass the IBL bake boundary. `TextureAssetDescriptor` still records the decoded GPU shape, which allows staged import and future bake tooling to understand the intended cube texture layout.

`TextureAsset::upload_readiness(...)` explicitly returns unsupported for `.zcube` with a source-only reason. This prevents source mips from being silently used as final PMREM reflection data or uploaded through the existing RGBA8-only uncompressed texture path.

The same source-only rule now applies to classified external DDS/KTX cubemaps. The asset layer accepts the current importer metadata shape (`D2` with six layers) and the future explicit cube metadata shape (`Cube` with six layers), but it does not decode or transcode the external bytes. Importer/staged build code must still turn those source containers into `.zcube` or a derived bake input before runtime reflection data is produced.

## Test Coverage

`runtime_texture_zcube_source_cubemap_contract.rs` verifies that a `.zcube` texture:

- preserves `SourceCubemapMipChain::source_texels()` after RGBA16F round-trip,
- keeps descriptor metadata as linear RGBA16F cube with six layers,
- stays different from the PMREM chain on rough mips,
- is rejected by generic texture upload readiness,
- cannot be decoded as a reusable `.zribl` PMREM/SH9 artifact.

`runtime_texture_external_cubemap_source_only_contract.rs` verifies that DDS, KTX1, and KTX2 cubemaps with complete source mip chains are classified as source-only external cubemaps and rejected by upload readiness, while ordinary 2D DDS remains upload-ready for material texture use.

`runtime_texture_zcube_staged_ibl_bundle_contract.rs` verifies that staged `.zcube` source bytes remain separate from derived `.zribl` PMREM/SH9 bytes, round-trip source mip texels, and do not trigger runtime compute when the matching asset-derived artifact is present.

The focused validation run for this slice is recorded in the Shader 06 and Render 11 status tables.

## Open Work

The source container contract, external DDS/KTX source-only guard, and asset-layer staged `.zcube`/`.zribl` bundle writer are now in place. Real importer/staged command wiring still needs to decode `.hdr` / `.exr` or external source cubemap containers into `SourceCubemapMipChain` and call the staging store. Production GPU/offline bake can then replace the current CPU bridge while preserving the same `.zcube` source handoff.
