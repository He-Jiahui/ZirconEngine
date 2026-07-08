---
related_code:
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_writeback.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_blob.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs
implementation_files:
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_writeback.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact_readback.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs
  - E:\cargo-targets\zircon-ibl-runtime-writeback-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_runtime_writeback --nocapture --test-threads=1 (passed 2/2)
  - E:\cargo-targets\zircon-ibl-live-pmrem-readback-seam-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::tests::runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams --exact --nocapture --test-threads=1 (passed 1/1)
  - E:\cargo-targets\zircon-ibl-live-iem-readback-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::tests::runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance --exact --nocapture --test-threads=1 (passed 1/1)
doc_type: module-detail
---

# IBL Bake Artifact Runtime Writeback

## Purpose

`ibl_bake_artifact_runtime_writeback.rs` and `ibl_bake_artifact_runtime_dispatch.rs` are the asset-layer bridge for Plan 11 / Shader 06 §4.7 source 2: runtime `.zircon-cache` artifacts. The writeback module accepts already-acquired PMREM/SH9/IEM readback sections, validates they describe a current request, writes the encoded `.zribl` blob through `IblBakeArtifactCacheStore`, and reports exactly what happened. The dispatch module combines source 1 asset-derived blobs from `IblBakeArtifactAssetDerivedStore`, runtime-cache reads, and later readback writeback so a runtime-compute miss can become a runtime-cache hit on the next resolve.

The boundary is intentionally narrow. It does not issue wgpu commands, map buffers, choose bake work, or mutate `SourceCubemapEnvironment`. Those responsibilities stay in renderer/runtime bake scheduling and the render-core artifact-application bridge.

Renderer-local graph-output writeback now lives in `graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs`. That module is the first GPU-side consumer of this asset contract: it reads already-scheduled graph outputs before transient release, then calls `write_ibl_bake_artifact_runtime_dispatch_readback(...)`. The asset layer still owns stale descriptor rejection, payload assembly, and cache path generation.

## Contract

`write_ibl_bake_artifact_runtime_readback(...)` first checks `IblBakeArtifactDescriptor::is_current_for(request)`. Stale descriptors return `SkippedDescriptorNotCurrent` with no path and zero encoded/payload lengths, and section bytes are not validated because the artifact must not be written regardless of their shape.

For current descriptors, `IblBakeArtifactReadbackSections::into_payload()` validates required PMREM RGBA16F, SH9, and optional IEM RGBA16F byte lengths against the descriptor. The function then creates an `IblBakeArtifactBlob`, writes it through `IblBakeArtifactCacheStore::write_runtime_cache(...)`, and returns `Written` with the cache path, encoded length, and payload length.

`resolve_ibl_bake_artifact_runtime_dispatch(...)` reads `.zircon-cache`, merges the cache hit with caller-provided asset-derived blobs, and returns a report that exposes the selected source, resolved payload, and runtime compute dispatch count. `write_ibl_bake_artifact_runtime_dispatch_readback(...)` accepts a previous dispatch report and only writes readback bytes when that report required runtime compute. If the request already resolved from asset-derived or runtime-cache data, readback writeback is skipped and the runtime cache remains unchanged.

`ibl_bake_runtime_writeback.rs` relies on that skip behavior before touching graph resources. Cache/asset hits can therefore skip renderer readback even if no IBL transient resources exist for the frame, while compute misses read the graph sections and feed them into the same asset-layer writeback path.

Source 1 asset-derived files are documented in `docs/zircon_runtime/asset/ibl-bake-artifact-asset-derived.md`. They share the same `.zribl` blob format but live under `render/ibl-derived`, not `render/ibl`, and continue to win over runtime cache hits during dispatch resolution.

## Verification

`runtime_environment_ibl_bake_artifact_contract.rs` covers:

- current PMREM/SH9/IEM sections writing a cache blob and reading back as `IblBakeArtifactCacheRead::Hit`;
- runtime dispatch miss writing readback sections and resolving the next dispatch from runtime cache with zero compute dispatches;
- asset-derived dispatch hits skipping readback writeback and leaving runtime cache untouched;
- asset-derived companion store hits resolving before same-request runtime cache blobs with zero compute dispatches;
- stale descriptor writeback skipping without creating a runtime cache file;
- missing IEM section rejection;
- PMREM section length mismatch rejection.
- renderer-local SH9 graph output readback writing the runtime cache and resolving the next dispatch with zero compute dispatches.
- renderer-local live PMREM graph output readback writing the runtime cache, resolving the next dispatch with zero compute dispatches, decoding the PMREM payload, and preserving/reducing seam metrics on mid and rough mips.
- renderer-local live IEM graph output readback writing the runtime cache, resolving the next dispatch with zero compute dispatches, decoding the IEM payload, and preserving directional irradiance response against the CPU cosine-convolution reference after scale normalization.

Counted command:

```powershell
cargo test -p zircon_runtime --test runtime_environment_ibl_bake_artifact_contract --locked
```

Result: 21/21 passed. The direct Cargo output reported 0 failed, 0 ignored, 0 filtered, and repository-existing warnings only.

Renderer-local live PMREM graph writeback is covered by a focused lib test because it needs an offscreen WGPU graph dispatch. The Cargo wrapper timed out during cold compilation, then the generated `zircon_runtime-9b53a08da5f6e0e7.exe` test binary ran `runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams` successfully: 1/1 passed, 7310 filtered, test body 3.28s.

Renderer-local live IEM graph writeback is covered by the matching focused lib test. The Cargo wrapper timed out during cold compilation, then the generated `zircon_runtime-9b53a08da5f6e0e7.exe` test binary ran `runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance` successfully: 1/1 passed, 7312 filtered, test body 8.14s.

## Open Work

Final TX-M3 `.zcube` importer and staged build command wiring, external container source-only enforcement in the importer, RenderDoc/product capture, probe capture/blending, 4K/16K offline bake, and full CI remain open.
