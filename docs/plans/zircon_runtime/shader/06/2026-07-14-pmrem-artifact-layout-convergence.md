# Shader06 PMREM Artifact Layout Convergence

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Milestone: M3
Status: completed
Files: ["zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs", "zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs", "zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs", "zircon_runtime/src/asset/importer/environment_ibl.rs", "zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs", "zircon_runtime/src/core/framework/render/environment/mod.rs", "zircon_runtime/src/core/framework/render/environment/skybox.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap_artifact.rs", "zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs", "zircon_runtime/tests/runtime_environment_ibl_source_import_staging_contract.rs", "docs/zircon_runtime/asset/ibl-source-cubemap-staging.md"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| EC-M3 / EC-M4 | PMREM artifact layout request identity | `completed` | 2026-07-14 | `IblBakeArtifactRequest` now owns source layout, independent PMREM face size/mip count, required contents, and bake key; asset-derived/runtime-cache identities include the complete request; descriptor reconstruction and `SourceCubemapEnvironment` preserve active PMREM layout; artifact application validates source layout then adopts descriptor PMREM layout; `environment_ibl_pmrem_face_size` stages the selected layout directly; the viewer no longer performs a post-load CPU rebake. Contract tests cover PMREM 128/512 identity separation, independent-layout artifact application, staged source128/PMREM64 restore/reuse, and explicit-upscale rejection; the lower resolution keeps the ordinary gate bounded while the ignored Poly Haven fixture owns PMREM256 product coverage. Reference evidence: `dev/cmft/include/cmft/cubemapfilter.h` exposes independent `_dstFaceSize`/`_mipCount`; `dev/cmft/src/cmft/cubemapfilter.cpp` clamps destination mips from `dstFaceSize`; `dev/cmftStudio/src/gui.cpp` exposes radiance mip configuration separately. Windows focused contracts, the current viewer build/startup, and the unchanged PMREM512 visual/RenderDoc product evidence are recorded below. |

## Scope Delivered

- Framework owner: `zircon_runtime/src/core/framework/render/environment/{ibl_bake_artifact.rs,source_cubemap.rs,source_cubemap_artifact.rs,skybox.rs}`.
- Asset owner: `zircon_runtime/src/asset/{importer/environment_ibl.rs,artifact/ibl_bake_artifact_cache.rs,artifact/ibl_bake_artifact_asset_derived.rs}`.
- Product consumer: `zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs`.
- Contract tests: `zircon_runtime/tests/runtime_environment_ibl_{bake_artifact,source_import_staging}_contract.rs`.
- Module documentation: `docs/zircon_runtime/asset/ibl-source-cubemap-staging.md`.

## Fresh Testing Evidence

- Windows `runtime_environment_ibl_bake_artifact_contract`: 23 passed, 0 failed in 0.18s. This covers PMREM 128/512 request/cache identity separation, descriptor currentness, and adopting an artifact PMREM layout independently from the source chain.
- Windows managed job `a60caf3ec926421cace5f114a1961447`: `runtime_environment_ibl_source_import_staging_contract` passed 6, failed 0, ignored 1 in 19.30s. The explicit-upscale fixture uses a 512x256 equirect source so the asserted resolved source face size is deterministically 128; source128/PMREM64 persistence and reuse also passed.
- Windows managed job `6ed8bfda86584d009e2ce262c46de04d`: `cargo build -p zircon_app --bin zircon_shader_pbr_viewer --locked --offline` exited 0. The delivered `E:\ZirconBuilds\shader-pbr-viewer-pmrem-artifact-layout-20260714\zircon_shader_pbr_viewer.exe` returns 0 for `--help`; a real source64/PMREM64 launch loaded the directly staged `.zcube`/`.zribl` pair and reached `Ready` after 68.99s.
- The rendering and PMREM filter shader did not change in this artifact-identity slice. Existing accepted product evidence remains `docs/tests/runtime/shader/runtime_shader_pbr_ibl_metallic_smoothness_matrix_angular0003_20260713.png`, `runtime_shader_pbr_real_hdri_lakes_pmrem512_angular0003_exact_multiview_contact_sheet_20260713.png`, and `zircon_shader_pbr_viewer_pmrem512_angular0003_dx12_renderdoc_20260714_capture.rdc`; the current executable startup proves the new direct artifact path is loadable but is not represented as a newly captured PMREM512 image.
- A broad `zircon_runtime` package attempt compiled production code but stopped in concurrently edited foreign test consumers. This slice claims only the focused contract and current viewer gates above, not a clean full-workspace result.

## Review

Independent review found one important test-fixture mismatch: the original 256x128 HDR fixture naturally resolved to a 64-face source while its assertion expected 128. The fixture is now 512x256 and the original rejection test passes. No critical or important findings remain in the PMREM artifact-layout diff; the coordinator review gate records that result separately.
