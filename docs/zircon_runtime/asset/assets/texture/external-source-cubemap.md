---
related_code:
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
implementation_files:
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
plan_sources:
  - user: 2026-07-08 external DDS/KTX real HDRI cubemap source-only classification before importer/staged bake wiring
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/tests/runtime_texture_external_cubemap_source_only_contract.rs
  - zircon_runtime/tests/runtime_texture_zcube_source_cubemap_contract.rs
doc_type: module-detail
---

# External Source Cubemap Containers

## Purpose

`external_source_cubemap.rs` classifies real external DDS, KTX1, and KTX2 cubemap containers that should enter the IBL bake/import path as source cubemaps. The module exists to keep imported HDRI cubemaps separate from regular material textures and from reusable PMREM artifacts.

The classifier is deliberately placed in the asset texture layer. Import plugins and staged build commands can consume the result later, but the source-only rule is enforced before generic compressed texture upload planning. A valid six-face DDS/KTX cubemap can therefore no longer slip through as a direct BC/ETC/ASTC material texture upload or as final reflection data.

## Related Files

`external_source_cubemap.rs` owns the header classification and metadata validation. `upload_support.rs` calls it before DDS/KTX upload planning and returns `EXTERNAL_SOURCE_CUBEMAP_UPLOAD_UNSUPPORTED_REASON` for valid source cubemap containers.

`.zcube` remains the Zircon-native source cubemap representation. External DDS/KTX containers are only recognized as source inputs at this stage; converting them into `.zcube`, deriving `.zribl`, and scheduling GPU/offline bake jobs are later importer/staged-build responsibilities.

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

The classifier reads header metadata only. It does not decode compressed blocks, transcode formats, resample faces, generate angular source mips, or bake PMREM. That is a separate importer/bake step so this module stays a low-level contract instead of becoming another partial renderer.

## Design and Rationale

The Plan 06/Plan 11 HDRI work requires a strict separation between source cubemap data, source display mip pyramids, specular PMREM chains, diffuse SH/IEM data, and final reusable `.zribl` artifacts. DDS and KTX can carry cubemap faces and mip chains, but their mere existence does not make the bytes a material texture upload target.

By classifying source cubemaps before upload readiness, the asset layer protects both sides of the pipeline:

- material textures still upload through existing DDS/KTX support when they are ordinary 2D or array textures,
- environment cubemaps are forced through the IBL bake/source conversion path,
- future importer code has a single public helper that recognizes the current D2+6 metadata shape and the future Cube+6 metadata shape.

## Edge Cases and Constraints

DDS classification recognizes legacy `DDSCAPS2_CUBEMAP` headers and DX10 `TEXTURECUBE` headers, while rejecting duplicated legacy-plus-DX10 cubemap declarations. Legacy DDS cubemaps must declare all six face flags. DDS cubemap arrays and KTX cubemap arrays are rejected because `.zcube` currently represents exactly one six-face source cubemap.

KTX1 uses `numberOfFaces == 6`; KTX2 uses `faceCount == 6`. A face count of `1` is treated as an ordinary texture and returns `Ok(None)`. Non-zero 3D depth is invalid for source cubemaps.

The complete mip-chain requirement is stricter than generic upload readiness because source cubemaps feed skybox minification, future filtered-importance source LOD, and diagnostics. Incomplete external mip chains must be repaired by importer/staged build code before entering the Zircon source-cubemap pipeline.

## Test Coverage

`runtime_texture_external_cubemap_source_only_contract.rs` covers:

- DDS cubemap with a complete source mip chain is classified as `Dds` and rejected by upload readiness with the source-only reason,
- KTX1 cubemap with a complete source mip chain is classified as `Ktx1` and rejected by upload readiness,
- KTX2 cubemap with a complete source mip chain is classified as `Ktx2` and rejected by upload readiness,
- ordinary 2D DDS remains BC upload-ready for material texture use,
- incomplete external source mip chains return a typed `IncompleteMipChain` error before bake/import acceptance.

`runtime_texture_zcube_source_cubemap_contract.rs` remains the native `.zcube` preservation guard for source mips after conversion.

## Plan Sources

This module implements the Plan 06/Plan 11 source-cubemap separation follow-up requested after the real HDRI cubemap visual review. It is the asset-side guard for the planned external DDS/KTX source-only importer path, not the final importer command or GPU/offline bake implementation.

## Open Issues

The next slice should connect the texture importer/staged build path so recognized DDS/KTX source cubemaps can be decoded or transformed into `.zcube`, then produce or reference derived `.zribl` artifacts for PMREM/SH9/IEM runtime use. This module intentionally does not decide the decode backend or bake scheduling strategy.
