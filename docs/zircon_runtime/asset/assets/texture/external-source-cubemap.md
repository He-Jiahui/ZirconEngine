---
related_code:
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_plugins/texture_importer/runtime/src/container/dds.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - tools/tests/test_runtime_asset_schema_naming.py
implementation_files:
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_plugins/texture_importer/runtime/src/container/dds.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
plan_sources:
  - user: 2026-07-10 prioritize runtime structure convention and code-review findings
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - user: 2026-07-08 external DDS/KTX real HDRI cubemap source-only classification before importer/staged bake wiring
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - tools/tests/test_runtime_asset_schema_naming.py
  - zircon_runtime/tests/runtime_texture_external_cubemap_source_only_contract.rs
  - zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs
  - zircon_runtime/tests/runtime_texture_zcube_source_cubemap_contract.rs
  - zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs
doc_type: module-detail
---

# External Source Cubemap Containers

## Purpose

`external_source_cubemap.rs` classifies real external DDS, KTX1, and KTX2 cubemap containers that should enter the IBL bake/import path as source cubemaps. The module exists to keep imported HDRI cubemaps separate from regular material textures and from reusable PMREM artifacts.

The classifier is deliberately placed in the asset texture layer. Import plugins and staged build commands can consume the result later, but the source-only rule is enforced before generic compressed texture upload planning. A valid six-face DDS/KTX cubemap can therefore no longer slip through as a direct BC/ETC/ASTC material texture upload or as final reflection data.

## Related Files

`external_source_cubemap.rs` owns the header classification and metadata validation. `upload_support.rs` calls it before DDS/KTX upload planning and returns `EXTERNAL_SOURCE_CUBEMAP_UPLOAD_UNSUPPORTED_REASON` for valid source cubemap containers.

`.zcube` remains the Zircon-native source cubemap representation. `external_source_cubemap/decode.rs` converts supported linear-float DDS/KTX payloads into that representation, while `environment_ibl.rs` stages the `.zcube` source and its separately rebuilt `.zribl` PMREM/SH9/IEM companion. Project scanning runs the same staging step after a fresh import and after restoring a ready cached texture artifact.

## Behavior Model

The public entry point is `external_source_cubemap_container_info(&TextureAsset)`. It returns:

- `Ok(Some(info))` for a valid external source cubemap container,
- `Ok(None)` for non-container textures, unsupported format families, or ordinary 2D DDS/KTX material textures,
- a typed `ExternalSourceCubemapContainerError` when the container claims to be a cubemap but violates the source-cubemap contract.

The accepted source contract is intentionally narrow:

- format family must be `dds/`, `ktx/`, or `ktx2/`,
- the header must declare exactly six cubemap faces and no cubemap arrays,
- face width and height must be square and power-of-two,
- the mip chain must be complete down to 1x1 using `source_cubemap_mip_count(face_size)`,
- payload metadata and render descriptor metadata must agree with the container header,
- render dimension may be current importer-style `D2` with six layers or future explicit `Cube` with six layers.
- pixel storage must be directly decodable linear HDR: DDS D3DFMT 113/116 or DX10 DXGI 10/2, KTX1 `GL_RGBA16F`/`GL_RGBA32F`, or uncompressed KTX2 Vulkan 97/109.

The classifier reads header metadata only. The sibling decoder consumes complete float source mip chains using cmft's canonical memory rules: DDS is face-major with all mips for one face contiguous; KTX1/KTX2 is mip-major on disk and is reordered to face-major. Those imported mips remain source/display data. Zircon always regenerates specular PMREM plus SH9/IEM instead of trusting an external mip chain as prefiltered reflection data.

`stage_external_source_cubemap_texture(...)` hashes the imported container bytes and normalized source URI into the same versioned IBL request identity used by HDR/EXR staging. A current `.zcube`/`.zribl` pair is reused; a missing or corrupt companion is rebuilt.

## Design and Rationale

The Plan 06/Plan 11 HDRI work requires a strict separation between source cubemap data, source display mip pyramids, specular PMREM chains, diffuse SH/IEM data, and final reusable `.zribl` artifacts. DDS and KTX can carry cubemap faces and mip chains, but their mere existence does not make the bytes a material texture upload target.

By classifying and converting source cubemaps before upload readiness, the asset layer protects both sides of the pipeline:

- material textures still upload through existing DDS/KTX support when they are ordinary 2D or array textures,
- environment cubemaps are forced through the IBL bake/source conversion path,
- importer code has one public classifier/decoder boundary for the current D2+6 metadata shape and the future Cube+6 metadata shape.

## Edge Cases and Constraints

DDS classification recognizes the original-header `DDSCAPS2_CUBEMAP` flags and DX10 `TEXTURECUBE` flags, while rejecting duplicate caps2-plus-DX10 cubemap declarations. A caps2 cubemap must declare all six face flags. The implementation and diagnostics use these protocol field names instead of the ambiguous `legacy` label. DDS cubemap arrays and KTX cubemap arrays are rejected because `.zcube` currently represents exactly one six-face source cubemap.

cmft writes the pre-DX10 half/float formats as numeric FourCC values rather than four printable ASCII bytes. The texture importer therefore recognizes D3DFMT 113 and 116 before ASCII FourCC validation, emits stable `dds/D3DFMT-*` format names, and reports unsupported numeric values as `pre-DX10 D3D format` rather than using an ambiguous migration label.

KTX1 uses `numberOfFaces == 6`; KTX2 uses `faceCount == 6`. A face count of `1` is treated as an ordinary texture and returns `Ok(None)`. Non-zero 3D depth is invalid for source cubemaps.

The complete mip-chain requirement is stricter than generic upload readiness because source cubemaps feed skybox minification, filtered-importance source LOD, and diagnostics. Compressed BC/ETC/ASTC, Basis payloads, and KTX2 supercompression are typed unsupported source pixel formats until an importer-side transcoder can produce linear float texels; they are not accepted as a completed IBL conversion path.

## Test Coverage

`runtime_texture_external_cubemap_source_only_contract.rs` covers:

- DDS cubemap with a complete source mip chain is classified as `Dds` and rejected by upload readiness with the source-only reason,
- KTX1 cubemap with a complete source mip chain is classified as `Ktx1` and rejected by upload readiness,
- KTX2 cubemap with a complete source mip chain is classified as `Ktx2` and rejected by upload readiness,
- ordinary 2D DDS remains BC upload-ready for material texture use,
- incomplete external source mip chains return a typed `IncompleteMipChain` error before bake/import acceptance.

`runtime_environment_external_cubemap_import_staging_contract.rs` covers cmft-style RGBA16F DDS face-major input, KTX1/KTX2 mip-major reorder, all face/mip source identities, `.zcube` plus `.zribl` staging, current-artifact reuse, and the requirement that regenerated PMREM differs from unfiltered external source mips.

The texture importer unit test `dds_container_importer_accepts_cmft_numeric_rgba16f_fourcc` verifies the numeric D3DFMT 113 header used by cmft. Focused validation passed 3/3 external staging tests, 5/5 source-only classifier tests, and 1/1 plugin parser test on 2026-07-10.

`runtime_texture_zcube_source_cubemap_contract.rs` remains the native `.zcube` preservation guard for source mips after conversion.

`test_runtime_asset_schema_naming.py` locks the DDS caps2 protocol vocabulary and rejects a return of `legacy_cubemap` or legacy-labelled diagnostics.

## Plan Sources

This module implements the Plan 06/Plan 11 source-cubemap separation follow-up requested after the real HDRI cubemap visual review. It closes the direct linear-float DDS/KTX conversion and staged dual-artifact path while preserving the rule that external source mips are never treated as final reflection PMREM.

## Open Issues

Compressed DDS/KTX and Basis/KTX2 supercompressed payloads still need an importer-side transcode backend. TX-M3 now has general `CubemapAsset`/`Texture2DArrayAsset` assembly, six-file/cross/equirectangular manifests, D2Array/Cube GPU views, and material dimension fallback diagnostics. Cubemap arrays and production GPU/offline bake remain open; they can replace the current CPU artifact builder without changing the source-container decode contract or request identity.
