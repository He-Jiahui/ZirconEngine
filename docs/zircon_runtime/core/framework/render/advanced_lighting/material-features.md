---
related_code:
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_usage.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/screen_space_transmission.rs
  - zircon_runtime/src/asset/assets/material/material_asset/advanced_features.rs
  - zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl
implementation_files:
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_usage.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/mod.rs
  - zircon_runtime/src/asset/assets/material/material_asset/advanced_features.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
plan_sources:
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
tests:
  - render_advanced_material_features_default_has_no_feature_work
  - render_advanced_material_features_enable_only_authored_lobes
  - render_advanced_material_features_normalize_invalid_values
  - render_advanced_material_default_variant_key_is_unchanged
  - render_advanced_material_variant_key_tracks_authored_lobes
  - render_advanced_lighting_material_usage_keeps_child_features_when_parent_is_missing
  - render_shader_template_projects_advanced_pbr_features
  - render_transmission_queue_value_is_2900_in_transparent_band
  - render_screen_space_transmission_settings_normalize_step_budget
doc_type: module-detail
---

# Advanced PBR Material Features

`StandardPbrMaterialFeatures` is the renderer-neutral contract for clearcoat,
anisotropy, and transmission. It contains no WGPU resources and remains owned by
`core::framework::render::advanced_lighting`.

## Defaults And Activation

The default contract preserves the legacy Standard PBR variant: all lobe
strengths are zero, no clearcoat normal texture is referenced, and no
screen-color copy is requested. Clearcoat, anisotropy, and either transmission
lobe require the forward material path. Only specular transmission requires the
screen-color copy used for refraction.

The no-absorption default uses finite `f32::MAX` instead of infinity so the DTO
round-trips through TOML and other finite-number serializers while retaining an
effectively unbounded attenuation distance.

## Normalization

`normalized()` clamps authored strengths and colors to `[0, 1]`, rejects
non-finite values, prevents negative thickness, and keeps the index of
refraction at or above `1.0`. Asset projection must normalize once before
constructing shader variants or GPU material data.

## Integration Boundary

The asset projection maps active lobes to `PBR_CLEARCOAT`,
`PBR_ANISOTROPY`, and `PBR_TRANSMISSION`. Default values keep the previous
variant key unchanged, so materials that do not author advanced lobes retain
the legacy pipeline-cache identity.

The normalized values are packed into the 192-byte material uniform and are
consumed by `zr_pbr_extras.wgsl`. Clearcoat and anisotropy use the late forward
opaque path. Transmission is assigned render queue `2900`, ahead of ordinary
transparent queue `3000`; queue offsets do not move it out of that fixed band.

Effective-material projection starts from the loadable child material. A
missing, unresolved, cyclic, over-depth, or shader-incompatible parent stops
inheritance at that boundary instead of discarding the child. Values inherited
from every successfully loaded ancestor remain merged, the child-owned
advanced PBR values remain authoritative, and the projected material clears
its parent reference before feature usage is recorded.

View-local material-usage extraction only counts meshes visible to the
selected camera layer set. Specular transmission is the sole trigger for the
`transmission.scene_copy` graph resource and pass. The copy owns a transient
single-sample `Rgba16Float` destination while reading the imported physical
scene-color texture. `ScreenSpaceTransmissionSettings` exposes a clamped
`0..=4` step budget: zero keeps one environment-only transmission draw, while
positive values partition the depth-sorted command stream and request one copy
per non-empty step. Product/render-capture evidence remains an AF-M1
testing-stage requirement; see the transmission module document for the WGPU
execution contract.
