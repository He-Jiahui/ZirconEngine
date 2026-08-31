# Shader06 M6 Current-Source Attestation

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Milestone: M6
Status: in_progress
Files: ["docs/plans/zircon_runtime/shader/06/2026-08-03-m6-current-source-attestation.md", "docs/plans/zircon_runtime/shader/06/2026-08-25-pbr-ibl-preoptimization-architecture-audit.md", "zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs", "zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_environment_only_pbr.wgsl", "zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl", "zircon_runtime/src/graphics/shader/includes/zr_pbr_common.wgsl", "zircon_runtime/src/graphics/shader/includes/zr_pbr_extras_core.wgsl", "zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl", "zircon_runtime/src/graphics/shader/template/deferred_gbuffer.rs", "zircon_runtime/src/graphics/shader/template/module_registry.rs", "zircon_runtime/src/graphics/shader/template/module_registry/tests.rs", "zircon_runtime/src/graphics/shader/template/tests.rs", "zircon_runtime/src/graphics/shader/template/tests/environment/sampling.rs", "zircon_runtime/src/graphics/shader/template/tests/environment_only_pbr.rs", "zircon_runtime/src/graphics/shader/template/tests/environment_specular_occlusion.rs", "zircon_runtime/src/graphics/shader/template/tests/material_template_assembly.rs", "zircon_runtime/src/graphics/shader/template/tests/standard_material_surface_template.rs", "zircon_runtime/src/graphics/shader/template/tests/standard_pbr_specialization.rs", "zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_environment_core.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_environment_generic_api.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_environment_only_pbr.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_procedural_sky.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_shading_environment_only_pbr.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr_basic.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl", "zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl"]

## Scope Delivered

- Preserve the historical environment-only Forward source reduction while binding
  this M6 run to the exact current Standard-PBR Fresnel/transmission source
  closure listed above.
- Use one shared GGX/Fresnel owner across basic/advanced Forward, deferred, and
  fallback paths; use the same diffuse-energy contract for ambient, lightmap,
  environment-only, and environment IBL consumption.
- Keep advanced diffuse, reflected specular, environment specular, scene
  transmission, clearcoat, and emission separate until final layer composition.
  Specular transmission overrides diffuse transmission, retains reflection, and
  consumes base-color-tinted raw transmitted radiance below clearcoat.
- Add no material binding, uniform field, PSO identity, feature permutation, or
  ordinary opaque-path texture sample. Skip per-light diffuse BTDF work when the
  prepared effective diffuse-transmission weight is zero.

## Fresh Testing Evidence

- Pending coordinator-managed M6 Windows validation. Scoped `rustfmt --check`,
  `git diff --check`, old direct-light composition zero-match, and old advanced
  aggregate-transmission zero-match checks pass; these are static gates only.
- Current-source Cargo/Naga/WGPU, numeric endpoint tests, DX12 viewer, screenshot,
  quantitative-image, RenderDoc, GPU p50/p95/p99, RSS, and WPR/WPA energy evidence
  are not yet passed and must not be inferred from this record.
- Historical RenderDoc captures remain diagnostic only. Their sampled export
  frame attributes at least 70.08% of GPU time to copy/readback work, so it does
  not establish a steady-state shader bottleneck or a post-change result.

## Review

- The source implementation has received an in-session structural re-review, but
  the required distinct coordinator reviewer has not accepted this exact
  manifest. M6 remains `in_progress`; coordinator-recorded validation and
  independent review are required before service commit or WeCom notification.
