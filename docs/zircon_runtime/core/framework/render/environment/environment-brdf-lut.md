---
related_code:
  - zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs
  - zircon_runtime/tests/runtime_environment_brdf_lut_contract.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs
plan_sources:
  - user: 2026-07-07 mirror sphere grazing should look physically realistic and not over-emphasize the right side
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/tests/runtime_environment_brdf_lut_contract.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_orthographic_reflection_20260707.png
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_mirror_sphere_perspective_reflection_20260707.png
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_ortho_cpu_ref_compare_20260707.png
  - docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_persp_cpu_ref_compare_20260707.png
doc_type: module-detail
---

# Environment BRDF LUT

`environment_brdf_lut.rs` builds the split-sum environment BRDF lookup table used by standard PBR indirect specular lighting. The table stores scale/bias terms for the shader-side expression `F0 * A + F90 * B`; `zr_environment.wgsl` samples it from scene group binding 3 and multiplies the resulting coefficient by the PMREM reflection sample.

For a perfect metallic mirror (`F0 = 1`, `F90 = 1`, `roughness = 0`), the coefficient collapses to `A + B`. That value must not exceed `1.0`, otherwise the LUT creates non-physical grazing amplification before the reflected environment color is even considered. The current builder normalizes each generated texel when `scale + bias > 1.0`, preserving the scale/bias ratio while keeping the perfect-mirror response energy-conserving.

`zr_environment.wgsl` also clamps the sampled BRDF coefficient to `[0, 1]`. That shader-side guard protects runtime rendering from stale LUT textures, quantization, or future approximation paths that would otherwise reintroduce edge amplification. This clamp does not change HDR environment radiance itself; it only prevents the BRDF multiplier from boosting a perfect mirror above the sampled PMREM/source environment.

The 2026-07-07 grazing slice refreshed the perfect-mirror real HDRI screenshots after this energy guard. The refreshed perspective mirror image has SHA256 `35F0711D0990EC1825DC6C5D734ED6F3BC5ADD51C953280A3928DA5D0E1CCE61`, and the orthographic image has SHA256 `A610CDA7C5F095E3EDDA5D5F97F5A9FD296B888AFF6027A96B6063827F925FF1`. CPU source-reference comparison artifacts were written beside them under `docs/tests/runtime/shader`, with SSIM `0.9596` for perspective and `0.9573` for orthographic.

Validation evidence for this slice:

- Direct lib-test binary: `environment_brdf_lut_conserves_smooth_perfect_mirror_grazing_energy` passed 1/1 from `E:\cargo-targets\zircon-cmft-skybox-0707\debug\deps\zircon_runtime-1ff53e05a9088131.exe`.
- Mirror export: `cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export export_runtime_shader_pbr_real_hdri_mirror_reflection_png --no-default-features --features core-min --locked --jobs 1 -- --ignored --exact --nocapture --test-threads=1` passed 1/1 and rewrote the two mirror PNGs under `docs/tests/runtime/shader`.
- CPU reference check: the regenerated compare PNGs show the runtime sphere against direct HDRI reflection reference; measured left/right edge luma stays balanced enough for the real asymmetric HDRI, while structure remains above the SSIM floor.
