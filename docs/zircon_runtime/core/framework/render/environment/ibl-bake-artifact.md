---
related_code:
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_resolution.rs
  - zircon_runtime/src/core/framework/render/environment/rgba16f.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_resolution.rs
  - zircon_runtime/src/core/framework/render/environment/rgba16f.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs
  - zircon_runtime/src/asset/assets/texture/zcube.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
  - zircon_runtime/tests/runtime_texture_zcube_source_cubemap_contract.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
  - E:\cargo-targets\zircon-ibl-runtime-writeback-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_runtime_writeback --nocapture --test-threads=1 (passed 2/2)
  - E:\cargo-targets\zircon-ibl-live-pmrem-readback-seam-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::tests::runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams --exact --nocapture --test-threads=1 (passed 1/1)
  - E:\cargo-targets\zircon-ibl-live-iem-readback-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::tests::runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance --exact --nocapture --test-threads=1 (passed 1/1)
  - E:\cargo-targets\zircon-ibl-asset-derived-0708\debug\deps\runtime_environment_ibl_bake_asset_derived_contract-10ff0cfb48251da9.exe --nocapture --test-threads=1 (passed 3/3)
doc_type: module-detail
---

# IBL Bake Artifact Contract

## Purpose

`ibl_bake_artifact.rs` is the Plan 11 / Shader 06 §4.7 contract layer for persistent environment-lighting bake artifacts. It defines the CPU-visible DTOs used to decide whether an IBL PMREM/SH9/IEM artifact can be reused before runtime environment compute is scheduled.

The module is intentionally backend-free. It does not load files, create wgpu textures, or perform readback. Runtime GPU code, importers, and staged build tools can all consume the same request, descriptor, header, and selection rules without pulling wgpu into `core::framework::render`.

## Contract

The current artifact header records:

- the `IblBakeKey`,
- `IBL_BAKE_ALGORITHM_VERSION`,
- source cube face size and mip count,
- content bits for PMREM, SH9, and optional IEM.

`IblBakeArtifactContents` implements `BitOr` and `BitOrAssign` so request construction can express combined output families as `PMREM | SH9 | IEM` without leaking raw bit manipulation to callers. The bitmask remains a typed core-framework contract; validation still flows through descriptor content checks rather than ad hoc integer tests.

`IblBakeArtifactDescriptor::is_current_for(...)` rejects artifacts when the key, layout, content mask, or algorithm version does not match the request. This is the hard guard required by the plan so PMREM roughness mapping, FIS sampling, SH weighting, and IEM format changes cannot silently reuse old artifacts.

`IblBakeArtifactPayload` now defines the CPU-visible byte payload that follows a valid descriptor. The layout is deterministic: PMREM RGBA16F cubemap texels first, then SH9 as 9 vec4 f32 little-endian coefficients, then optional 32x32x6 IEM RGBA16F texels with alpha written as 1.0. RGBA16F packing is shared through `rgba16f.rs`, which is also used by the `.zcube` source-cubemap container so source mips and PMREM/IEM artifacts do not drift into different half-float layouts. `IblBakeArtifactDescriptor::expected_payload_size_bytes()` and the payload byte-range helpers expose the same offsets to importers, runtime cache code, and future file containers.

`IblBakeArtifactBlob` is the complete artifact byte container: fixed header first, then the descriptor-sized payload. `decode_current_for_request(...)` parses both sections and rejects stale key/layout/content/algorithm-version matches before runtime code treats the blob as reusable derived/cache data.

`asset::artifact::IblBakeArtifactAssetDerivedStore` owns the asset-derived filesystem entry for source 1. It writes and reads raw `.zribl` blobs under `render/ibl-derived/v{IBL_BAKE_ALGORITHM_VERSION}/...`, reports missing and rejected companion files without failing the fallback path, and exposes an asset-derived candidate only for current blobs. `write_source_cubemap_asset_derived_artifact(...)` is the current staged prebake helper for source-cubemap PMREM/SH9(+IEM) output until the TX-M3 `.zcube` importer is wired to the same store.

`asset::artifact::IblBakeArtifactCacheStore` owns the runtime filesystem entry for source 2. It writes and reads raw `.zribl` blobs under `.zircon/cache/render/ibl/v{IBL_BAKE_ALGORITHM_VERSION}/...`, reports missing and rejected cache files without failing the fallback path, and exposes a runtime-cache candidate only for current blobs.

`ibl_bake_artifact_readback.rs` owns the backend-free readback assembly contract. `IblBakeArtifactReadbackSections` accepts PMREM RGBA16F bytes, SH9 bytes, and optional IEM RGBA16F bytes as separate sections, validates each section against `IblBakeArtifactDescriptor` expected sizes, rejects missing/unexpected/wrong-length sections, and then delegates final payload validation to `IblBakeArtifactPayload::decode(...)`. The module deliberately contains no wgpu or filesystem code, so GPU readback, offline bake tools, and tests can all feed the same section DTO.

`asset::artifact::write_ibl_bake_artifact_runtime_readback(...)` owns the runtime cache writeback boundary once readback bytes are available. It skips stale descriptors before validating section bytes, assembles current readback sections into an `IblBakeArtifactBlob`, writes through `IblBakeArtifactCacheStore`, and returns an `IblBakeArtifactRuntimeWritebackReport` with status, descriptor, path, encoded length, and payload length. Actual GPU command submission and buffer acquisition remain outside this boundary.

`asset::artifact::resolve_ibl_bake_artifact_runtime_dispatch(...)` owns the asset-layer dispatch/cache integration contract. It reads the runtime cache for a request, combines any current runtime-cache blob with caller-provided asset-derived blobs, and returns an `IblBakeArtifactRuntimeDispatchReport` that preserves the same asset-derived > runtime-cache > runtime-compute priority while exposing the resolved payload and dispatch count. `write_ibl_bake_artifact_runtime_dispatch_readback(...)` only writes readback sections when that dispatch report actually required runtime compute; asset-derived or runtime-cache hits skip readback writeback so reusable sources cannot be overwritten by a stale or stray runtime readback.

`graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback` is the renderer-local consumer of that dispatch/cache contract. It rebuilds the artifact descriptor from the explicit request, skips graph readback when the dispatch report already resolved from asset/cache data, and reads already-scheduled graph outputs before transient release when runtime compute was required.

`ibl_bake_artifact_resolution.rs` owns the blob-backed resolution layer. `IblBakeArtifactBlobCandidate` wraps already-decoded asset-derived and runtime-cache blobs, while `resolve_ibl_bake_artifact_payload(...)` reuses the descriptor selection contract and returns `IblBakeArtifactResolvedPayload`. Current asset-derived blobs win over runtime-cache blobs, stale candidates are counted as rejected, and a full miss exposes no blob/payload while preserving runtime-compute dispatch counts. This keeps importer/cache file IO outside render core and keeps source-cubemap mutation in the artifact-application bridge.

`source_cubemap_artifact.rs` owns the bridge from a decoded payload back into the runtime source-cubemap model. `source_cubemap_mip_chain_with_bake_artifact(...)` preserves source/display mips, applies PMREM texels to the specular chain, and applies SH9 coefficients for diffuse irradiance. `source_cubemap_environment_with_bake_artifact(...)` applies the same decoded payload to a `SourceCubemapEnvironment` while preserving source revision/hash, intensity, and rotation, and attaches a decoded `SourceCubemapIrradianceCube` when the payload contains optional IEM data.

Artifact application also updates `SourceCubemapEnvironment::bake_artifact_hash`, which is part of `SourceCubemapUploadKey` but not part of `IblBakeKey`. This keeps cache selection source-based while ensuring `SceneEnvironmentCubemap::ensure_uploaded(...)` does not skip WGPU source/specular/IEM cube uploads when the source is unchanged but a different PMREM/SH9/IEM artifact payload has been applied.

`select_ibl_bake_artifact(...)` applies the three-source priority:

1. asset derived artifact,
2. runtime `.zircon/cache` artifact,
3. runtime compute fallback.

Valid derived/cache hits return `environment_compute_dispatch_count() == 0`. A miss returns `RuntimeCompute` and reports one dispatch unit per required output family, so PMREM+SH9+IEM currently reports `3`.

## Verification

`runtime_environment_ibl_bake_artifact_contract.rs` covers header round-trip, stale algorithm invalidation, derived-before-cache priority, stale-candidate rejection, runtime-compute fallback dispatch counts, PMREM/SH9/IEM payload round-trip, payload length/IEM mismatch rejection, full blob round-trip, truncated blob rejection, stale blob rejection, runtime cache write/read, runtime-cache candidate conversion, missing cache fallback, stale cache rejection, truncated cache rejection, runtime readback sections writing a cache blob, stale-descriptor writeback skip without file creation, missing/wrong-length readback section rejection, blob-backed resolved payload priority, stale-derived-to-cache fallback, resolved-payload miss behavior, runtime dispatch miss-to-readback-to-cache-hit behavior, asset-derived hit readback skipping, payload application to a source cubemap chain, source mip preservation, optional IEM attachment to a source-cubemap environment, preservation of environment metadata, artifact upload-key invalidation without changing the source bake key, and layout/content rejection while applying payloads.

`runtime_environment_ibl_bake_asset_derived_contract.rs` covers the source 1 companion store: staged source-cubemap prebake writes a current `.zribl` below `render/ibl-derived`, readback resolves before a same-request runtime cache blob with zero compute dispatches, stale algorithm blobs are rejected as non-fatal, and PMREM seam metrics survive asset-derived write/read/apply.

`runtime_texture_zcube_source_cubemap_contract.rs` covers the source-only side of the asset chain. It verifies that `.zcube` preserves `SourceCubemapMipChain::source_texels()` through RGBA16F round-trip, remains distinct from PMREM rough mips, is rejected by generic texture upload readiness, and cannot be decoded as a reusable `.zribl` PMREM/SH9 artifact.

Counted command:

```powershell
cargo test -p zircon_runtime --test runtime_environment_ibl_bake_artifact_contract --locked
```

Result: 21/21 passed, 0 failed, 0 ignored, 0 filtered. The dispatch/cache integration test body finished in 0.03s after the warmed Windows test build; stderr contained repository-existing warnings only. This verification does not generate screenshots or write image artifacts.

The 2026-07-06 PMREM cosine-tail slice typechecked the new `IblBakeArtifactContents` bitmask operators as part of the focused lib-test build for `source_cubemap` (`15/15` passed for the filtered source-cubemap run). No new artifact-contract functional run is claimed for this bitmask-only edit.

The 2026-07-08 live PMREM graph-output slice verifies the renderer-local artifact bridge with actual WGPU PMREM compute output. `runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams` dispatches PMREM graph passes, writes the produced graph output into the runtime cache through the artifact readback/writeback contract, resolves the next request from cache with zero compute dispatches, decodes the PMREM bytes, and checks mid/rough seam metrics.

The 2026-07-08 live IEM graph-output slice verifies the same artifact bridge with actual WGPU irradiance-cube compute output. `runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance` dispatches the IEM graph pass, writes the produced `environment.ibl.irradiance_cube` texture into runtime cache through the readback/writeback contract, resolves the next request from cache with zero compute dispatches, decodes the IEM bytes, and checks normalized directional response against the CPU cosine-convolution reference.

The 2026-07-08 asset-derived store slice verifies source 1 without renderer involvement. The direct test binary `runtime_environment_ibl_bake_asset_derived_contract-10ff0cfb48251da9.exe --nocapture --test-threads=1` passed 3/3, covering asset-derived path layout, source 1 priority over source 2, stale blob rejection, and PMREM seam preservation.

## Open Work

This contract closes the reusable artifact decision surface, the in-memory PMREM/SH9/IEM payload byte layout, the complete header+payload artifact blob byte container, the source 1 asset-derived companion file store, the source 2 runtime cache file entry, the runtime readback-section to cache-blob writeback boundary, the asset-layer runtime dispatch/cache integration contract, the renderer-local graph-output runtime cache writeback bridge for already-scheduled SH9 plus live PMREM and IEM outputs, the blob-backed resolved-payload priority layer, the source-cubemap/environment artifact-application bridge, WGPU upload-key invalidation for artifact-applied PMREM/SH9/IEM content, and the asset-level `.zcube` source-only container contract. Final TX-M3 `.zcube` importer/staged command wiring, external container source-only enforcement in importer tests, RenderDoc capture, 4K/16K offline bake, and full CI remain Plan 11 / Shader 06 follow-up gates.
