---
related_code:
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/mod.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs
implementation_files:
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/artifact/mod.rs
tests:
  - zircon_runtime/tests/runtime_texture_zcube_staged_ibl_bundle_contract.rs
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

`write_source_cubemap_staged_bundle(...)` writes the source `.zcube` first, then writes the derived `.zribl` through `IblBakeArtifactAssetDerivedStore`. The two outputs deliberately live in different directories and use different extensions even though they are keyed by the same bake request.

## Design and Rationale

The store does not decode `.hdr`, `.exr`, DDS, or KTX files. Those source-format decisions belong to importer/build-tool slices. This module starts at an already-built `SourceCubemapMipChain` so it can enforce the durable asset contract without taking ownership of image decoding policy.

The `.zcube` read path reconstructs a `TextureAsset::Container` with the request's face size, mip count, and six cube faces before passing the bytes through the normal `.zcube` decoder. That means staged files are validated by the same source-cubemap contract used by texture assets.

The derived `.zribl` side is not duplicated. The staging store delegates to the existing asset-derived store so PMREM/SH9/IEM payload layout, descriptor validation, algorithm-version checks, and runtime dispatch priority remain single-owner behavior.

## Edge Cases and Constraints

The writer rejects request/source layout mismatches before creating files. This avoids a common failure mode where the request hash path says one face size but the `.zcube` header carries another.

Missing staged `.zcube` files are non-fatal reads. Corrupt or mismatched staged files are explicit errors because falling through to runtime compute would hide a broken import artifact.

The bundle writer can leave a valid `.zcube` if the derived `.zribl` write fails afterward. This is acceptable for staged build tooling because source and derived artifacts are separately discoverable, and a subsequent build can overwrite both from the same request.

## Test Coverage

`runtime_texture_zcube_staged_ibl_bundle_contract.rs` verifies that one request writes distinct `.zcube` and `.zribl` paths, that `.zcube` round-trips source mip texels within RGBA16F tolerance, that the derived `.zribl` resolves as source 1 with zero runtime compute dispatches, and that raw `.zcube` bytes cannot decode as a reusable `.zribl` artifact.

Focused validation for this slice uses:

```powershell
CARGO_TARGET_DIR=E:\cargo-targets\zircon-zcube-staged-bundle-0708 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_texture_zcube_staged_ibl_bundle_contract --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1
```

2026-07-08 validation passed with the command above: 1/1 test passed, test body 0.07s, total build/test wall time about 7m17s in `E:\cargo-targets\zircon-zcube-staged-bundle-0708`. The output contained only existing workspace warnings. This slice does not generate screenshots.

## Plan Sources

This module advances Shader 06 EC-M3 and Render 11 environment-lighting source 1 work by closing the asset-layer handoff between TX-M3 `.zcube` source cubemap production and reusable `.zribl` prebake consumption.

## Open Work

Real source decoding and build entry points are still separate work: the importer or staged build command must decode `.hdr` / `.exr`, classify DDS/KTX source cubemaps, build `SourceCubemapMipChain`, and call this store. Production GPU/offline PMREM and SH9/IEM bake paths can then replace the current CPU bridge while preserving the same staging layout.
