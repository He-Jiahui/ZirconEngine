use crate::core::framework::render::{CastShadowsMode, RenderImageUsage, RendererCommon};
use crate::core::math::Vec4;
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId};
use crate::graphics::scene::resources::{
    MaterialDisabledPasses, MaterialRuntime, ResourceStreamer,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MaterialTextureBinding, MaterialTextureSet,
};

pub(super) fn material_tinted(
    streamer: &ResourceStreamer,
    material: ResourceHandle<MaterialMarker>,
    instance_tint: Vec4,
) -> Vec4 {
    let material_tint = streamer
        .material(&material.id())
        .map(|material| material.base_color)
        .unwrap_or(Vec4::ONE);
    instance_tint * material_tint
}

pub(super) fn renderer_common_for_material(
    common: &RendererCommon,
    material: Option<&MaterialRuntime>,
) -> RendererCommon {
    resolve_renderer_common_for_material(
        common,
        material
            .map(|material| material.cast_shadows)
            .unwrap_or(true),
        material
            .map(|material| material.receive_shadows)
            .unwrap_or(true),
    )
}

fn resolve_renderer_common_for_material(
    common: &RendererCommon,
    material_casts_shadows: bool,
    material_receives_shadows: bool,
) -> RendererCommon {
    let mut resolved = common.clone();
    if !material_casts_shadows {
        if resolved.cast_shadows == CastShadowsMode::ShadowsOnly {
            resolved.enabled = false;
        }
        resolved.cast_shadows = CastShadowsMode::Off;
    }
    resolved.receive_shadows &= material_receives_shadows;
    resolved
}

pub(super) fn material_taa_reactive_mask_strength(material: Option<&MaterialRuntime>) -> f32 {
    material
        .map(|material| material.taa_reactive_mask_strength)
        .filter(|strength| strength.is_finite())
        .unwrap_or_default()
        .clamp(0.0, 1.0)
}

pub(super) fn material_disabled_passes(
    material: Option<&MaterialRuntime>,
) -> MaterialDisabledPasses {
    material
        .map(|material| material.disabled_passes)
        .unwrap_or_default()
}

pub(super) fn material_texture_set(
    streamer: &ResourceStreamer,
    material: Option<&MaterialRuntime>,
) -> MaterialTextureSet {
    MaterialTextureSet::new(
        material_texture_binding(
            streamer,
            material.and_then(|material| material.base_color_texture),
        ),
        material_normal_texture_binding(
            streamer,
            material.and_then(|material| material.normal_texture),
        ),
        material_texture_binding(
            streamer,
            material.and_then(|material| material.metallic_roughness_texture),
        ),
        material_texture_binding(
            streamer,
            material.and_then(|material| material.occlusion_texture),
        ),
        material_texture_binding(
            streamer,
            material.and_then(|material| material.emissive_texture),
        ),
        material_normal_texture_binding(
            streamer,
            material.and_then(|material| material.clearcoat_normal_texture),
        ),
    )
}

fn material_texture_binding(
    streamer: &ResourceStreamer,
    texture_id: Option<ResourceId>,
) -> MaterialTextureBinding {
    material_output_target_texture_binding(streamer, texture_id)
        .unwrap_or_else(|| MaterialTextureBinding::texture(streamer.texture(texture_id)))
}

fn material_normal_texture_binding(
    streamer: &ResourceStreamer,
    texture_id: Option<ResourceId>,
) -> MaterialTextureBinding {
    material_output_target_texture_binding(streamer, texture_id)
        .unwrap_or_else(|| MaterialTextureBinding::texture(streamer.normal_texture(texture_id)))
}

fn material_output_target_texture_binding(
    streamer: &ResourceStreamer,
    texture_id: Option<ResourceId>,
) -> Option<MaterialTextureBinding> {
    let output_target = streamer.output_target_texture_resource(&texture_id?)?;
    output_target
        .descriptor()
        .usage
        .contains(&RenderImageUsage::Sampled)
        .then(|| MaterialTextureBinding::output_target(output_target))
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{CastShadowsMode, RendererCommon};

    use super::resolve_renderer_common_for_material;

    #[test]
    fn render_material_shadow_gate_forces_cast_mode_off_when_material_disables_casting() {
        for mode in [
            CastShadowsMode::On,
            CastShadowsMode::TwoSided,
            CastShadowsMode::ShadowsOnly,
        ] {
            let common = renderer_common(mode, true);

            let resolved = resolve_renderer_common_for_material(&common, false, true);

            assert_eq!(resolved.cast_shadows, CastShadowsMode::Off);
        }
    }

    #[test]
    fn render_material_shadow_gate_disables_shadows_only_renderer_without_a_shadow_pass() {
        let common = renderer_common(CastShadowsMode::ShadowsOnly, true);

        let resolved = resolve_renderer_common_for_material(&common, false, true);

        assert_eq!(resolved.cast_shadows, CastShadowsMode::Off);
        assert!(!resolved.enabled);
    }

    #[test]
    fn render_material_shadow_gate_preserves_non_boolean_cast_modes() {
        for mode in [CastShadowsMode::TwoSided, CastShadowsMode::ShadowsOnly] {
            let common = renderer_common(mode, true);

            let resolved = resolve_renderer_common_for_material(&common, true, true);

            assert_eq!(resolved.cast_shadows, mode);
        }
    }

    #[test]
    fn render_material_shadow_gate_conjoins_receive_shadow_inputs() {
        assert!(
            resolve_renderer_common_for_material(
                &renderer_common(CastShadowsMode::On, true),
                true,
                true
            )
            .receive_shadows
        );
        assert!(
            !resolve_renderer_common_for_material(
                &renderer_common(CastShadowsMode::On, false),
                true,
                true,
            )
            .receive_shadows
        );
        assert!(
            !resolve_renderer_common_for_material(
                &renderer_common(CastShadowsMode::On, true),
                true,
                false,
            )
            .receive_shadows
        );
    }

    #[test]
    fn render_missing_material_preserves_renderer_common_fallback_semantics() {
        let common = renderer_common(CastShadowsMode::ShadowsOnly, false);

        let resolved = super::renderer_common_for_material(&common, None);

        assert_eq!(resolved, common);
    }

    #[test]
    fn render_mesh_draw_chain_owns_renderer_common_instead_of_shadow_bools() {
        let pending_source = include_str!("../pending_mesh_draw.rs");
        let mesh_draw_source = include_str!("../../../mesh_draw/mesh_draw.rs");
        let create_source = include_str!("../../create_mesh_draw.rs");
        let pending_body = struct_body(pending_source, "pub(super) struct PendingMeshDraw {");
        let mesh_draw_body = struct_body(mesh_draw_source, "pub(crate) struct MeshDraw {");

        assert!(pending_body.contains("common: Arc<RendererCommon>"));
        assert!(!pending_body.contains("cast_shadows: bool"));
        assert!(!pending_body.contains("receive_shadows: bool"));
        assert!(mesh_draw_body.contains("cast_shadows: CastShadowsMode"));
        assert!(!mesh_draw_body.contains("cast_shadows: bool"));
        assert!(!mesh_draw_body.contains("common: RendererCommon"));
        assert!(create_source.contains("common: &RendererCommon"));
        assert!(!create_source.contains("cast_shadows: bool"));
        assert!(create_source.contains("common.cast_shadows"));
        assert!(!create_source.contains("common.clone()"));
    }

    fn renderer_common(cast_shadows: CastShadowsMode, receive_shadows: bool) -> RendererCommon {
        RendererCommon {
            cast_shadows,
            receive_shadows,
            ..RendererCommon::default()
        }
    }

    fn struct_body<'a>(source: &'a str, declaration: &str) -> &'a str {
        source
            .split_once(declaration)
            .unwrap_or_else(|| panic!("missing source declaration: {declaration}"))
            .1
            .split_once("\n}")
            .map(|(body, _)| body)
            .expect("source declaration should have a closing brace")
    }
}
