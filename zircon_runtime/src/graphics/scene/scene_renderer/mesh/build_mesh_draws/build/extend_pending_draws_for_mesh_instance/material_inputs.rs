use crate::core::framework::render::{CastShadowsMode, RendererCommon};
use crate::core::math::Vec4;
use crate::graphics::scene::resources::{
    MaterialDisabledPasses, MaterialRuntime, PublishedMaterialDrawProxy,
    PublishedMaterialTextureBinding,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MaterialTextureBinding, MaterialTextureSet,
};

const ERROR_MATERIAL_BASE_COLOR: Vec4 = Vec4::new(1.0, 0.0, 1.0, 1.0);

pub(super) fn material_tinted(material: Option<&MaterialRuntime>, instance_tint: Vec4) -> Vec4 {
    resolve_material_tint(material.map(|material| material.base_color), instance_tint)
}

fn resolve_material_tint(material_tint: Option<Vec4>, instance_tint: Vec4) -> Vec4 {
    instance_tint * material_tint.unwrap_or(ERROR_MATERIAL_BASE_COLOR)
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

pub(super) fn material_half_resolution_transparency(material: Option<&MaterialRuntime>) -> bool {
    material.is_some_and(|material| material.alpha_blend && material.separate_translucency)
}

pub(super) fn material_disabled_passes(
    material: Option<&MaterialRuntime>,
) -> MaterialDisabledPasses {
    material
        .map(|material| material.disabled_passes)
        .unwrap_or_default()
}

pub(super) fn material_texture_set(material: PublishedMaterialDrawProxy<'_>) -> MaterialTextureSet {
    let textures = material.textures();
    MaterialTextureSet::new(
        material_texture_binding(textures.base_color),
        material_texture_binding(textures.normal),
        material_texture_binding(textures.metallic_roughness),
        material_texture_binding(textures.occlusion),
        material_texture_binding(textures.emissive),
        material_texture_binding(textures.clearcoat_normal),
    )
}

fn material_texture_binding(binding: PublishedMaterialTextureBinding) -> MaterialTextureBinding {
    match binding {
        PublishedMaterialTextureBinding::Texture(resource) => {
            MaterialTextureBinding::texture(resource)
        }
        PublishedMaterialTextureBinding::OutputTarget(resource) => {
            MaterialTextureBinding::output_target(resource)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{CastShadowsMode, RendererCommon};
    use crate::core::math::Vec4;

    use super::{resolve_material_tint, resolve_renderer_common_for_material};

    #[test]
    fn unpublished_material_uses_visible_error_proxy_tint() {
        assert_eq!(
            resolve_material_tint(None, Vec4::ONE),
            Vec4::new(1.0, 0.0, 1.0, 1.0)
        );
        assert_eq!(
            resolve_material_tint(
                Some(Vec4::new(0.25, 0.5, 0.75, 1.0)),
                Vec4::new(0.5, 0.5, 0.5, 1.0),
            ),
            Vec4::new(0.125, 0.25, 0.375, 1.0)
        );
    }

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
        let pending_material_source = include_str!("../pending_material_draw.rs");
        let mesh_draw_source = include_str!("../../../mesh_draw/mesh_draw.rs");
        let create_source = include_str!("../../create_mesh_draw.rs");
        let build_source = include_str!("../build.rs");
        let pending_body = struct_body(pending_source, "pub(super) struct PendingMeshDraw {");
        let mesh_draw_body = struct_body(mesh_draw_source, "pub(crate) struct MeshDraw {");

        assert!(pending_body.contains("common: Arc<RendererCommon>"));
        assert!(!pending_body.contains("cast_shadows: bool"));
        assert!(!pending_body.contains("receive_shadows: bool"));
        assert!(mesh_draw_body.contains("cast_shadows: CastShadowsMode"));
        assert!(!mesh_draw_body.contains("cast_shadows: bool"));
        assert!(!mesh_draw_body.contains("common: RendererCommon"));
        assert!(pending_body.contains("material: PendingMaterialDraw"));
        assert!(pending_material_source.contains("resource_id: ResourceId"));
        assert!(mesh_draw_body.contains("material_id: ResourceId"));
        assert!(create_source.contains("common: &RendererCommon"));
        assert!(create_source.contains("material_id: ResourceId"));
        assert!(build_source.contains("pending_draw.material.resource_id,"));
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
