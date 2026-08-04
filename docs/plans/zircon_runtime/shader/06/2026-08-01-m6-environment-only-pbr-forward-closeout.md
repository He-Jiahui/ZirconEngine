# Shader06 M6 Environment-Only PBR Forward Closeout

Historical Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Historical Milestone: M6
Historical Status: in_progress
Historical implementation files: ["docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md", "docs/plans/zircon_runtime/shader/06/2026-07-29-m5-direct-present-performance.md", "docs/plans/zircon_runtime/shader/06/2026-08-01-m6-environment-only-pbr-forward-closeout.md", "zircon_runtime/src/core/framework/render/shader/variant_key.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs", "zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs", "zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs", "zircon_runtime/src/graphics/shader/template/assemble.rs", "zircon_runtime/src/graphics/shader/template/module_registry.rs", "zircon_runtime/src/graphics/shader/template/pass_specialization.rs", "zircon_runtime/src/graphics/shader/template/tests/environment.rs", "zircon_runtime/src/graphics/shader/wgsl/zr_shading_environment_only_pbr.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_template_forward_environment_only_pbr.wgsl"]

The current M6 validation manifest is deliberately owned by
`2026-08-03-m6-current-source-attestation.md`; this historical implementation
record remains descriptive evidence and must not bind a new coordinator run.

## Reason

M5 owns an immutable earlier current-source manifest. Its managed validation attempt reached terminal `failed` during workspace materialization with `validation_copy_attribution_stale` after the working tree acquired the environment-only performance delta. Rebinding M5 to a larger path set is rejected by design, while rolling back the integrated/current implementation would violate the same-plan forward-fix policy.

M6 is an append-only sibling that depends on accepted M1-M4. It first integrates every post-snapshot owner, including the two new WGSL sources required by Rust `include_str!` calls. M5 can then rerun its original immutable manifest against that forward baseline. Goal closeout still requires accepted succeeded attempts for both M5 and M6; M6 does not hide or skip the historical M5 failure.

## Implementation

- Apply `ShaderFeatureBits::ENVIRONMENT_ONLY_PBR` only to compatible fallback Standard-PBR opaque Base variants used by the environment-only viewer prewarm.
- Keep GBuffer, custom shader, shadow, alpha, unlit, advanced-PBR, volumetric, and non-Base variants on the generic source path.
- Assemble an environment-only Forward template that retains material evaluation, alpha handling, ambient, environment IBL, and emissive, while excluding unreachable direct-light grid/cookie, lightmap/irradiance-volume, shadow, volumetric, and advanced-PBR modules.
- Preserve the canonical Standard-PBR include token while giving the reduced body a distinct content hash, so cache identity and include manifests cannot alias the generic shader.
- Reuse the existing asynchronous mesh-pipeline completion path; do not introduce a second cache or a synchronous compatibility path.
- Reuse `SceneUniform.camera_world_position.w` for the global material texture mip bias, with a zero default and finite `0..4` clamp, so render-budget degradation needs neither a new uniform allocation nor an invalid GPU sampling bias.

## Performance Evidence

The prior 82--100 second load was dominated by synchronous DX12 WGSL/PSO construction, not IBL cache restore. Historical deferred construction was about 91.74 seconds while IBL restore was about 1.08 seconds; the later environment-only path reduced scene construction to about 8.28 seconds and Ready to about 19.21 seconds. These historical measurements explain the bottleneck but are not current-source acceptance evidence.

For the current reduced source, a conservative comparable inventory is 72,564 bytes for the generic Forward lower bound and 39,435 bytes for the environment-only assembly, a 45.7% reduction. This proves source-volume convergence only. It must not be reported as DX12 PSO time or GPU duration until a fresh current-source viewer capture measures the real path.

## Review

The required independent second review reports `Critical 0 / Important 0 / Minor 0 / Ready`. It checked feature-profile boundaries, pipeline cache identity, WGSL resource ABI, Standard-PBR environment math equivalence, assembled-source regression coverage, and compatibility with asynchronous pipeline completion.

## Acceptance

- Managed Windows Rust/WGSL tests pass for compatibility filtering, exact environment-only prewarm/cache reuse, include-hash isolation, and assembled-source validity.
- The assembled environment-only Forward source excludes the unreachable module/function set and is at least 25% smaller than the comparable generic source.
- The global material mip bias preserves the existing scene-uniform layout, defaults to zero, and sanitizes non-finite or out-of-range CPU input before GPU upload.
- A fresh current-source DX12 viewer run records scene construction, Ready, Base prewarm, first present, and readback/present-path timing without substituting historical logs.
- A fresh current-source screenshot and metadata are retained under `docs/tests/runtime/shader`, and the image satisfies the plan's PBR/IBL quantitative gates.
- `D:\Tools\renderdoc\renderdoccmd.exe replay --loops 1` accepts the matching current-source capture.
- M6 is accepted and coordinator-integrated before M5 is retried against the new baseline; both milestones must be accepted before Goal closeout.

## Current Status

Implementation, static source guards, formatting, diff integrity, source-volume measurement, and the independent second review are complete. The latest immutable Windows validation copy (`job 3b27c26867f2457e898972f9297c1e91`, `run 808f1cf31210424189ba4542dcb70953`) terminated before executing any Shader06 test because `cargo --locked` found that clean `main@322a03acfec7c8527cec593a4165af3ae31437b5` declares `meshopt` in both workspace/runtime manifests but does not contain its package or runtime dependency in `Cargo.lock`. The canonical forward-fix owner remains `docs/plans/zircon_runtime/runtime/04/failure-2026-07-17-woc-gltf-meshopt-webp-import.md` (`status: open`); M6 does not modify the foreign manifests/lockfile, use an unlocked bypass, or duplicate that Failure lifecycle.

The 2026-08-01 follow-up review rechecked the 16-path manifest (16 unique, none missing), scoped Rust formatting, diff integrity, new-file whitespace/conflict markers, the environment-only compatibility boundaries, synchronous MVP prewarm behavior, and the conservative source-volume threshold. It found no new production-code change justified before fresh DX12 timing: removing the remaining local-probe/planar source would require splitting or duplicating the canonical 24 KB environment implementation and would widen PBR/cache risk without measured startup evidence. Managed Rust/WGSL validation, current-source DX12 timing/screenshot, RenderDoc replay, coordinator commit, and the subsequent M5 retry remain outstanding, so this record stays `in_progress`.

The subsequent resource audit found one bounded startup allocation outside the shader-source inventory: the zero-direct-light environment-only viewer constructed the default 4096x4096 shadow atlas even though its Base material neither receives shadows nor records shadow passes. `EnvironmentOnlyPbrPreview` now selects a 1x1/one-slot `ShadowAtlasResources` placeholder, retaining valid shared shadow bindings and disabled GPU globals while avoiding approximately 64 MiB of Depth32 allocation. FullScene and StandardPbrPreview retain the default atlas. This is resource-allocation convergence, not a DX12 timing claim. The same audit confirmed that WGPU 29.0.3 enables persistent driver `RuntimePipelineCache` seed data only for Vulkan; DX12 intentionally reports `UnsupportedBackend`, while the viewer's Base prewarm `cache_hit` field is only an in-process mesh-cache result. This explains why historical DX12 restart runs did not gain a driver-PSO seed and must not be attributed to IBL. Scoped source parsing, `rustfmt --emit stdout`, `git diff --check`, and an independent review (Critical 0 / Important 0 / Minor 0) pass; managed product evidence remains required before changing the milestone state.

The 2026-08-03 forward repair addresses bounded asynchronous Base-PSO admission for the one-shot viewer gate. A full compiler budget now remains recoverable pending work and short-circuits before WGSL source assembly; it is retried by the Viewer on the existing bounded recheck cadence, while worker and shader-compilation errors remain terminal and screenshot/RenderDoc requests retain their shared 45-second deadline. The deterministic 64-slot regression holds a second worker barrier before asserting re-admission, then uses bounded five-second observer waits before synchronizing the target. Independent review found `Critical 0 / Important 0`; its test-timeout Minor was fixed. Scoped Rust parsing and diff integrity pass, but no Cargo, current-source DX12 screenshot, or RenderDoc replay was run in this repair. The canonical Shader03 failure handoff remains `open`, and M6 remains `in_progress` until its managed product gates complete.
