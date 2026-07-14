---
related_code:
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/mod.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs
implementation_files:
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/artifact/mod.rs
tests:
  - zircon_runtime/tests/runtime_texture_zcube_staged_ibl_bundle_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_source_import_staging_contract.rs
  - zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs
  - zircon_runtime/tests/runtime_texture_external_cubemap_source_only_contract.rs
plan_sources:
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
doc_type: module-detail
---

# IBL Source Cubemap Staging

## Purpose

`ibl_source_cubemap_staging.rs` owns the asset-layer handoff between source cubemap assets and reusable environment-lighting bake artifacts. It gives importer and staged build code one stable entry point for writing both sides of an IBL request:

- `.zcube` source cubemap bytes under `render/ibl-source/...`;
- `.zribl` PMREM/SH9/IEM bake bytes under `render/ibl-derived/...`.

This keeps the source skybox/FIS input and the final reflection artifact separate. A `.zcube` file remains a source mip pyramid and never becomes a PMREM cache blob by renaming or path placement.

## Related Files

The staging store reuses `texture_asset_from_source_cubemap_zcube(...)` from `zcube.rs` for source bytes and `IblBakeArtifactAssetDerivedStore::write_source_cubemap_asset_derived_artifact(...)` for the derived `.zribl` companion. The request identity hash is shared with `ibl_bake_artifact_cache.rs`, so source staging, asset-derived prebake, and runtime cache paths all agree on the same `IblBakeArtifactRequest` identity.

`artifact/mod.rs` exports the staging store and constants so build tools and importer code do not need to reach into private child modules.

## Behavior Model

`IblSourceCubemapStagingStore::new(library_root)` expects the same project library or staged output root used by the asset-derived store.

`source_cubemap_path(request)` returns:

```text
render/ibl-source/v{IBL_BAKE_ALGORITHM_VERSION}/{request_hash}/face_####_mips_##.zcube
```

`write_source_cubemap_zcube(...)` checks that the request face size and mip count match the provided `SourceCubemapMipChain`, writes the `.zcube` container bytes, and reports the path plus encoded and payload byte lengths.

`read_source_cubemap_zcube(...)` returns `Hit(cubemap)` or `Missing`. If a file exists but its header, mip layout, face count, payload length, or request metadata do not match `.zcube` rules, the read returns a decoding error instead of silently falling back.

`read_source_cubemap_environment(...)` is the render-consumption boundary for a completed pair. It requires both current files, reconstructs the source/display mip chain from `.zcube`, applies PMREM/SH9 and optional IEM from `.zribl`, restores source revision/hash from the bake request, and carries a nonzero derived artifact hash into the render upload key. Missing or rejected companions are typed errors rather than a hidden CPU rebake.

`write_source_cubemap_staged_bundle(...)` writes the source `.zcube` first, then writes the derived `.zribl` through `IblBakeArtifactAssetDerivedStore`. The two outputs deliberately live in different directories and use different extensions even though they are keyed by the same bake request.

`stage_environment_ibl_source(...)` now owns the source-format entry immediately above the store. It decodes image bytes as linear RGBA32F, recognizes HDR/EXR 2:1 sources automatically, derives the cubemap face size with the plan rule `clamp(next_pow2((height + 1) / 2), 64, 1024)`, and builds the source mip chain plus PMREM/SH9/IEM companion. `environment_ibl = false` disables automatic staging; `environment_ibl = true` explicitly requests staging for another image format and makes a non-2:1 layout an error; `environment_ibl_face_size` can request a smaller power-of-two face size without upscaling above the natural source resolution.

`environment_ibl_pmrem_face_size` independently selects the PMREM destination face size. An explicit value must be a power of two in `64..=1024` and cannot exceed the resolved source face size. When omitted, the runtime's 128-face PMREM policy remains the default. The importer projects the equirectangular source and bakes directly to the requested PMREM layout, avoiding a default bake followed by a second resize bake.

The source identity uses BLAKE3 bytes to populate both `IblBakeKey` source revision/hash fields. Before rebuilding, the stager reads the `.zcube` and `.zribl` through their current decoders. A current pair returns `Reused`; missing, rejected, or corrupt current-version artifacts are regenerated from the source. Project scanning invokes this entry after a successful texture import and after a ready artifact restore, so deleting a derived IBL file does not require touching the authoring HDR to recover it.

`IblBakeArtifactRequest` is the single layout identity for source face size/mips, PMREM face size/mips, required contents, and bake key. Asset-derived and runtime-cache hashes include all of those fields. Reconstructing a request from a `.zribl` descriptor restores the descriptor's PMREM layout instead of falling back to 128. Applying a current artifact checks the source layout against the source chain and then adopts the descriptor's independent PMREM layout and texels. `SourceCubemapEnvironment::ibl_bake_artifact_request(...)` likewise derives the PMREM layout from its active chain.

`stage_external_source_cubemap_texture(...)` applies the same dual-artifact lifecycle to directly decodable HDR cubemap containers. It accepts linear RGBA16F/RGBA32F DDS, KTX1, and uncompressed KTX2 payloads after strict six-face/full-mip classification. DDS bytes are decoded from cmft's face-major layout; KTX bytes are reordered from mip-major disk layout. The external mip pyramid is retained only as source/display data, while PMREM is regenerated through Zircon's cmft-aligned filter and diffuse SH9/IEM are rebuilt from the source.

## Design and Rationale

The store itself still starts at an already-built `SourceCubemapMipChain`; source-format policy remains in the sibling importer module. This preserves the artifact boundary while allowing the project importer to produce `.zcube` and `.zribl` automatically. HDR/EXR are implemented without changing `AssetImportContext` or the native/plugin ABI, and the ordinary imported `TextureAsset` remains available for editor inspection.

The `.zcube` read path reconstructs a `TextureAsset::Container` with the request's face size, mip count, and six cube faces before passing the bytes through the normal `.zcube` decoder. That means staged files are validated by the same source-cubemap contract used by texture assets.

The derived `.zribl` side is not duplicated. The staging store delegates to the existing asset-derived store so PMREM/SH9/IEM payload layout, descriptor validation, algorithm-version checks, and runtime dispatch priority remain single-owner behavior.

This follows cmft's separation between source face size and radiance-filter `dstFaceSize`/`mipCount`. Zircon keeps its Unreal-style GGX importance-sampled PMREM algorithm, while using the cmft/cmftStudio configuration model to keep source resolution, PMREM result resolution, and mip count explicit rather than conflating them.

## Edge Cases and Constraints

The writer rejects request/source layout mismatches before creating files. This avoids a common failure mode where the request hash path says one face size but the `.zcube` header carries another.

Missing staged `.zcube` files are non-fatal reads. Corrupt or mismatched staged files are explicit errors because falling through to runtime compute would hide a broken import artifact.

Automatic mode skips HDR/EXR images that are not exactly 2:1, allowing HDR lookup textures to stay ordinary textures. Explicit mode rejects that shape instead of silently producing a wrongly projected cube. Float samples are bilinearly filtered with horizontal wrap, vertical clamp, finite-value sanitization, non-negative radiance, and the RGBA16F maximum of 65504; alpha is fixed to one for environment assets. External containers also sanitize non-finite/negative radiance and reject compressed or supercompressed payloads until a real transcode path exists.

The bundle writer can leave a valid `.zcube` if the derived `.zribl` write fails afterward. This is acceptable for staged build tooling because source and derived artifacts are separately discoverable, and a subsequent build can overwrite both from the same request.

## Test Coverage

`runtime_texture_zcube_staged_ibl_bundle_contract.rs` verifies that one request writes distinct `.zcube` and `.zribl` paths, that `.zcube` round-trips source mip texels within RGBA16F tolerance, that the derived `.zribl` resolves as source 1 with zero runtime compute dispatches, and that raw `.zcube` bytes cannot decode as a reusable `.zribl` artifact.

`runtime_environment_ibl_source_import_staging_contract.rs` verifies that HDR values above one survive RGBA32F decode and RGBA16F staging, a 128x64 equirect source produces a 64-face `.zcube` plus PMREM/SH9/IEM `.zribl`, a second invocation reuses the current pair, automatic mode skips a non-2:1 HDR, and explicit mode rejects the same invalid environment layout. The test executable compiled successfully in `E:\cargo-targets\zircon-ibl-source-import-0710` and passed 4/4 in 7.00s when run directly. A later Cargo rerun was blocked before these tests by an unrelated concurrent `TextShapeRequest.features` initializer error.

The 2026-07-14 PMREM-layout extension adds contracts for staging a 128-face source with an independent 64-face/7-mip PMREM, restoring it from the `.zribl`, reuse under the complete request identity, rejection of an explicit PMREM upscale, cache-path separation between PMREM 128 and 512, and applying an artifact whose PMREM result layout differs from the source chain's initial PMREM layout. The lightweight 64-face contract keeps the ordinary test gate bounded; the ignored Poly Haven product fixture exercises a 256-face PMREM, and PMREM512 is covered by the viewer's visual/RenderDoc evidence. Windows validation passed 23/23 artifact contracts and 6/6 non-ignored source-import contracts, with the Poly Haven staging fixture remaining explicitly ignored for product-only generation. The current viewer build also loaded a directly staged source64/PMREM64 artifact and reached its ready state.

`runtime_environment_external_cubemap_import_staging_contract.rs` passed 3/3 for RGBA16F DDS/KTX1/KTX2 conversion, face/mip ordering, regenerated PMREM, artifact reuse, and dual staging. `runtime_texture_external_cubemap_source_only_contract.rs` passed 5/5 for strict float source classification and ordinary 2D material-texture separation. The texture importer numeric cmft DDS parser test passed 1/1.

The staged-bundle contract now also restores a complete `SourceCubemapEnvironment` from the two files and compares quantized source plus PMREM texels against the written input. The ignored real-product fixture stages `polyhaven_lakes_2k.hdr` at face size 256 into `docs/tests/runtime/shader/polyhaven_lakes_2k_staged_ibl_20260710`, reloads it through the same API, and writes `polyhaven_lakes_2k_staged_ibl_20260710_report.txt`. The generated `.zcube` is 4,194,320 bytes and the `.zribl` is 4,243,692 bytes.

Focused validation for this slice uses:

```powershell
CARGO_TARGET_DIR=E:\cargo-targets\zircon-zcube-staged-bundle-0708 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_texture_zcube_staged_ibl_bundle_contract --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1
```

2026-07-08 validation passed with the command above: 1/1 test passed, test body 0.07s, total build/test wall time about 7m17s in `E:\cargo-targets\zircon-zcube-staged-bundle-0708`. The output contained only existing workspace warnings. This slice does not generate screenshots.

## Plan Sources

This module advances Shader 06 EC-M3 and Render 11 environment-lighting source 1 work by closing the asset-layer handoff between TX-M3 `.zcube` source cubemap production and reusable `.zribl` prebake consumption.

## Open Work

Compressed DDS/KTX and Basis/KTX2 supercompressed payloads still need importer-side transcoding into linear float texels. Six-file and cross-layout authoring inputs are also still open TX-M3 work, together with general cubemap/array asset and binding semantics. High-resolution offline 4K/16K PMREM production and editor-authored bake settings remain follow-up work; the request and artifact identity now preserve those future output layouts without a viewer-only override.
