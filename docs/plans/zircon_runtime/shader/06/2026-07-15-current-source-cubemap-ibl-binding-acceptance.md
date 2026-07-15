---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/reference_parity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_prefilter.wgsl
  - zircon_runtime/tests/runtime_texture_cube_resource_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs
  - zircon_runtime/tests/runtime_texture_external_cubemap_source_only_contract.rs
  - zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs
  - zircon_runtime/tests/runtime_environment_brdf_lut_contract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/irradiance_parity.rs
  - docs/plans/zircon_editor/editor/03/fixed-2026-07-15-external-source-cubemap-contract-api-drift.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/reference_parity.rs
  - zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs
  - docs/zircon_runtime/graphics/scene/scene_renderer/environment/ibl-bake-wgpu-dispatch.md
plan_sources:
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - dev/cmft/src/cmft/cubemapfilter.cpp
  - dev/cmft/src/cmft/cubemaputils.h
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShaders.usf
tests:
  - rustfmt --edition 2021 zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/reference_parity.rs
  - rustfmt --edition 2021 zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs
  - runtime_environment_external_cubemap_import_staging_contract-d6b4a7673c12dcb4.exe --nocapture --test-threads=1
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -RepoRoot E:/Git/ZirconEngine -Package zircon_runtime -SkipBuild -VerboseOutput
doc_type: plan-output-record
---

# Shader06 M2 current-source cubemap and IBL binding acceptance

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md

Milestone: M2

Status: completed

Files: ["docs/plans/zircon_runtime/shader/06/2026-07-15-current-source-cubemap-ibl-binding-acceptance.md", "docs/zircon_runtime/graphics/scene/scene_renderer/environment/ibl-bake-wgpu-dispatch.md", "zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/reference_parity.rs", "zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M2 | EC-M2 current-source cubemap resource, PMREM/SH9/BRDF/IEM, artifact/cache and source-only contracts | `completed` | 2026-07-15 | Current-source audit maps the plan list to existing owners: cube resource; PMREM CPU/GPU all-face/all-mip parity; SH9 CPU parity and constant band0; BRDF LUT corners; artifact PMREM+SH9(+IEM) byte roundtrip, stale rejection, first-miss/writeback/second-hit dispatch=0; source/display mips preserved when applying derived PMREM; external DDS/KTX mip chains remain source-only; PMREM artifact seam metrics; IEM/SH9 low-frequency parity. The missing explicit production-GPU constant-environment normalization gate is implemented as `render_env_prefilter_constant_env_is_identity`, reusing the production command plan, WGSL pipeline, bind group, dispatch, and RGBA16F readback path; it requires a real backend, rejects non-finite output, and checks every RGBA texel across six faces and all PMREM mips with a per-channel absolute-plus-relative tolerance. Direct execution of managed binaries passed the 104 focused M2 executions with zero failures. The external DDS/KTX/KTX2 staging contract was then hard-cut to the explicit `source_*` and `pmrem_*` accessors and passed 3/3 without restoring an alias, shim, fallback, or bypass. The cross-plan failure was returned to Editor03 as a fixed record. Independent review closed backend skip, non-finite, alpha, tolerance, helper-lifetime and tuple issues at Critical/Important/Minor = 0/0/0. Scoped rustfmt and diff-check pass. |

## Acceptance Mapping

| Plan gate | Current owner | Current status |
|---|---|---|
| Cube dimension, six-face payload and face order | `runtime_texture_cube_resource_contract.rs` | Passed 8/8. |
| GGX PMREM CPU/GPU parity | `render_env_prefilter_cpu_gpu_match_16` | Existing production GPU parity gate. |
| Constant environment PMREM identity | `render_env_prefilter_constant_env_is_identity` | Passed on the managed current-source lib-test binary; every RGBA texel, face and mip checked. |
| SH9 and IEM low-frequency parity | `render_env_sh9_matches_cpu_reference`, `render_env_iem_matches_sh9_low_frequency` | Existing production GPU gates. |
| Split-sum BRDF LUT corners and mirror grazing energy | `runtime_environment_brdf_lut_contract.rs` | Existing public contract. |
| Derived cache second launch dispatch=0 | `runtime_environment_ibl_bake_artifact_runtime_dispatch_writes_miss_readback_then_hits_cache` | Existing current descriptor/cache/writeback contract. |
| PMREM+SH9(+IEM) artifact roundtrip and stale rejection | `runtime_environment_ibl_bake_artifact_contract.rs` | Existing byte-exact format-v2 contract. |
| External prefiltered container remains source-only | `runtime_texture_external_cubemap_source_only_contract.rs` | Existing DDS/KTX1/KTX2 source-only contract. |
| External DDS/KTX/KTX2 import staging rebuilds Zircon PMREM | `runtime_environment_external_cubemap_import_staging_contract.rs` | Passed 3/3 with explicit source/PMREM accessors; no ambiguous compatibility surface. |
| PMREM seam preservation | `runtime_environment_ibl_bake_artifact_seam_contract.rs` | Existing artifact roundtrip seam metric contract. |

## Scope Delivered

- `reference_parity.rs` now drives the production WGPU PMREM command plan with a non-unit constant RGBA16F cubemap, requires a real backend, and validates finite RGBA identity across every face and destination mip with per-channel absolute-plus-relative tolerances.
- `runtime_environment_external_cubemap_import_staging_contract.rs` now expresses source-container layout and derived PMREM roles through the hard-cut `source_*` and `pmrem_*` API, preserving the assertion that imported source mips cannot masquerade as Zircon PMREM.
- `ibl-bake-wgpu-dispatch.md` records the production-GPU identity contract and the existing PMREM/SH9/IEM parity boundaries.
- The fixed Editor03 handoff records the cross-plan repair and return without transferring unrelated Editor03 work into Shader06.
- This child-plan record binds the exact four-file M2 code, documentation, focused contracts, review result, and package-level diagnostic. The returned `fixed-*` record remains an Editor03-owned output and is intentionally outside the Shader06 commit manifest.

## Fresh Testing Evidence

The exact production PMREM tests and complete focused M2 group passed 104/104. The current managed external cubemap integration binary passed 3/3. Frameworks03 prerequisite job `7d0c3e3ddfa148ce98e2350d8a3cc939` also exited 0, and the package-wide rerun progressed beyond both repaired targets before stopping in a later unrelated concurrent integration target. The package-wide command therefore remains red, but no current failure is attributed to the five-file Shader06 M2 manifest and none weakens its rendering thresholds.

## Review

Independent review verified backend enforcement, non-finite rejection, alpha coverage, per-channel normalized tolerance, tuple correctness, WGPU helper lifetime, and the external container source/PMREM role split. Final findings are Critical 0, Important 0, and Minor 0.
