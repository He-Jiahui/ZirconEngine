---
related_code:
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_resolution.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
  - zircon_runtime/tests/runtime_texture_zcube_staged_ibl_bundle_contract.rs
implementation_files:
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/artifact/mod.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
  - zircon_runtime/tests/runtime_texture_zcube_staged_ibl_bundle_contract.rs
  - E:\cargo-targets\zircon-ibl-asset-derived-0708\debug\deps\runtime_environment_ibl_bake_asset_derived_contract-10ff0cfb48251da9.exe --nocapture --test-threads=1
doc_type: module-detail
---

# IBL Bake Asset-Derived Artifacts

## Purpose

`ibl_bake_artifact_asset_derived.rs` owns Shader 06 / Render 11 source 1: asset-derived IBL bake blobs produced by importer or staged build work before runtime rendering. It keeps this companion artifact boundary in the asset layer, so render code can consume already-decoded blobs without owning file layout or staged build policy.

This store is separate from runtime cache source 2. Asset-derived blobs live below `library/render/ibl-derived/...`; runtime cache blobs continue to live below `library/render/ibl/...`. Both stores write the same raw `.zribl` header+payload format, so artifact priority can be decided by the shared render-core descriptor contract.

## Contract

`IblBakeArtifactAssetDerivedStore::new(library_root)` expects a project library root or staged build output root. `asset_derived_path(...)` writes paths under:

```text
render/ibl-derived/v{IBL_BAKE_ALGORITHM_VERSION}/{request_hash}/face_####_mips_##.zribl
```

The request hash is shared with `IblBakeArtifactCacheStore` and includes `IblBakeKey`, face size, and mip count. Content bits remain in the blob descriptor, so a PMREM+SH9 request can consume a PMREM+SH9+IEM blob, while a request requiring IEM rejects a PMREM+SH9-only blob.

`write_asset_derived_blob(...)` persists an already-built `IblBakeArtifactBlob`. `write_source_cubemap_asset_derived_artifact(...)` is the staged prebake helper for the current CPU bridge path: it creates a current descriptor from the request, encodes PMREM/SH9 and optional IEM data from a `SourceCubemapMipChain`, and writes the raw blob through the store.

`read_asset_derived_artifact(...)` returns `Hit(blob)`, `Missing`, or `Rejected(error)`. Missing and rejected files are non-fatal because source 2/runtime cache and source 3/runtime compute remain valid fallbacks. A hit can be converted to an `IblBakeArtifactBlobCandidate::asset_derived(...)`, preserving the source priority used by `resolve_ibl_bake_artifact_runtime_dispatch(...)`: asset-derived > runtime-cache > runtime-compute.

`ibl_source_cubemap_staging.rs` now provides the companion asset-layer bundle entry point. It writes the source `.zcube` under `render/ibl-source/...` and then delegates to this store for the matching `.zribl` under `render/ibl-derived/...`. This module still does not implement final TX-M3 `.hdr` / `.exr` decoding, external cmft DDS/KTX PMREM trust, GPU bake scheduling, or a CLI staged build command.

## Verification

`runtime_environment_ibl_bake_asset_derived_contract.rs` covers:

- staged source-cubemap prebake writing a current `.zribl` file under `render/ibl-derived`;
- reading the file back as source 1 and resolving runtime dispatch with `environment_compute_dispatch_count() == 0`;
- source 1 winning over a same-request runtime cache blob;
- stale algorithm descriptors being rejected without fatal read errors;
- PMREM seam metrics surviving asset-derived write/read/apply round-trip.

`runtime_texture_zcube_staged_ibl_bundle_contract.rs` verifies the integration point from staged `.zcube` source files into this asset-derived `.zribl` store, including zero-dispatch runtime resolution when the derived artifact is available.

Validation for the current slice:

```powershell
rustfmt --edition 2021 --check zircon_runtime\src\asset\artifact\ibl_bake_artifact_cache.rs zircon_runtime\src\asset\artifact\ibl_bake_artifact_asset_derived.rs zircon_runtime\src\asset\artifact\mod.rs zircon_runtime\tests\runtime_environment_ibl_bake_asset_derived_contract.rs
CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-asset-derived-0708 CARGO_INCREMENTAL=0 cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --message-format short --color never
CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-asset-derived-0708 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_environment_ibl_bake_asset_derived_contract --no-run --no-default-features --features core-min --locked --jobs 1 --message-format short --color never
E:\cargo-targets\zircon-ibl-asset-derived-0708\debug\deps\runtime_environment_ibl_bake_asset_derived_contract-10ff0cfb48251da9.exe --nocapture --test-threads=1
```

The direct test binary passed 3/3 in 1.52s. The first Cargo wrapper test attempts exited during cold compile before entering tests and are kept as non-passing diagnostics in `docs/tests/runtime/render/plan11_ibl_asset_derived_contract_20260708.*`.

## Open Work

The final importer/staged build integration still needs TX-M3 source decoding and a build-tool entry point that calls the staging store. External prefiltered DDS/KTX containers remain source-only until they carry Zircon `IblBakeKey`, current algorithm version, SH9/IEM payload, and `.zribl` header data.
