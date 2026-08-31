use crate::core::framework::render::{ShaderFeatureBits, ShaderPassType};

use super::module_registry::{
    ShaderTemplateInclude, environment_include, environment_only_pbr_include,
    environment_standard_pbr_include, irradiance_volume_include, light_cookie_include,
    light_grid_include, lightmap_include, pbr_extras_include, pbr_extras_include_for_features,
    shadow_disabled_include, shadow_include, volumetric_disabled_include, volumetric_include,
};

pub(crate) const MATERIAL_SHADER_TEMPLATE_REVISION: &str = "zr-material-template-v1";

const FORWARD_TEMPLATE_TOKEN: &str = "zr_template_forward.wgsl";
const ENVIRONMENT_ONLY_PBR_FORWARD_TEMPLATE_TOKEN: &str =
    "zr_template_forward_environment_only_pbr.wgsl";
const GBUFFER_TEMPLATE_TOKEN: &str = "zr_template_gbuffer.wgsl";
const DEPTH_TEMPLATE_TOKEN: &str = "zr_template_depth.wgsl";
const DEPTH_ALPHA_TEMPLATE_TOKEN: &str = "zr_template_depth_alpha.wgsl";
const SHADOW_TEMPLATE_TOKEN: &str = "zr_template_shadow.wgsl";
const SHADOW_ALPHA_TEMPLATE_TOKEN: &str = "zr_template_shadow_alpha.wgsl";
const VELOCITY_TEMPLATE_TOKEN: &str = "zr_template_velocity.wgsl";
const VELOCITY_ALPHA_TEMPLATE_TOKEN: &str = "zr_template_velocity_alpha.wgsl";
const TAA_REACTIVE_MASK_TEMPLATE_TOKEN: &str = "zr_template_taa_reactive_mask.wgsl";
const HIT_PROXY_TEMPLATE_TOKEN: &str = "zr_template_hit_proxy.wgsl";
const HIT_PROXY_ALPHA_TEMPLATE_TOKEN: &str = "zr_template_hit_proxy_alpha.wgsl";

const FORWARD_TEMPLATE: &str = include_str!("../wgsl/zr_template_forward.wgsl");
const ENVIRONMENT_ONLY_PBR_FORWARD_TEMPLATE: &str =
    include_str!("../wgsl/zr_template_forward_environment_only_pbr.wgsl");
const GBUFFER_TEMPLATE: &str = include_str!("../wgsl/zr_template_gbuffer.wgsl");
const DEPTH_TEMPLATE: &str = include_str!("../wgsl/zr_template_depth.wgsl");
const DEPTH_ALPHA_TEMPLATE: &str = include_str!("../wgsl/zr_template_depth_alpha.wgsl");
const SHADOW_TEMPLATE: &str = include_str!("../wgsl/zr_template_shadow.wgsl");
const SHADOW_ALPHA_TEMPLATE: &str = include_str!("../wgsl/zr_template_shadow_alpha.wgsl");
const VELOCITY_TEMPLATE: &str = include_str!("../wgsl/zr_template_velocity.wgsl");
const VELOCITY_ALPHA_TEMPLATE: &str = include_str!("../wgsl/zr_template_velocity_alpha.wgsl");
const TAA_REACTIVE_MASK_TEMPLATE: &str = include_str!("../wgsl/zr_template_taa_reactive_mask.wgsl");
const HIT_PROXY_TEMPLATE: &str = include_str!("../wgsl/zr_template_hit_proxy.wgsl");
const HIT_PROXY_ALPHA_TEMPLATE: &str = include_str!("../wgsl/zr_template_hit_proxy_alpha.wgsl");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderPassTemplate {
    pub(crate) include: ShaderTemplateInclude,
    pub(crate) support_includes: Vec<ShaderTemplateInclude>,
    pub(crate) requires_material_surface: bool,
    pub(crate) requires_shading_include: bool,
    pub(crate) uses_previous_position: bool,
}

pub(crate) fn pass_template_for(
    pass_type: ShaderPassType,
    features: ShaderFeatureBits,
) -> ShaderPassTemplate {
    pass_template_for_shading_model(pass_type, features, true)
}

pub(crate) fn pass_template_for_shading_model(
    pass_type: ShaderPassType,
    features: ShaderFeatureBits,
    uses_builtin_standard_pbr: bool,
) -> ShaderPassTemplate {
    if uses_builtin_standard_pbr
        && pass_type == ShaderPassType::Forward
        && features.contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR)
    {
        return ShaderPassTemplate {
            include: ShaderTemplateInclude::new(
                ENVIRONMENT_ONLY_PBR_FORWARD_TEMPLATE_TOKEN,
                ENVIRONMENT_ONLY_PBR_FORWARD_TEMPLATE,
            ),
            support_includes: vec![environment_only_pbr_include()],
            requires_material_surface: true,
            requires_shading_include: true,
            uses_previous_position: false,
        };
    }
    let alpha_test = features.contains(ShaderFeatureBits::ALPHA_TEST);
    match pass_type {
        ShaderPassType::Forward => {
            let shadow = if features.contains(ShaderFeatureBits::RECEIVE_SHADOWS) {
                shadow_include()
            } else {
                shadow_disabled_include()
            };
            let volumetric = if features.contains(ShaderFeatureBits::VOLUMETRIC_FOG) {
                volumetric_include()
            } else {
                volumetric_disabled_include()
            };
            let pbr_extras = if uses_builtin_standard_pbr {
                pbr_extras_include_for_features(features)
            } else {
                pbr_extras_include()
            };
            let environment = if uses_builtin_standard_pbr {
                environment_standard_pbr_include()
            } else {
                environment_include()
            };
            ShaderPassTemplate {
                include: ShaderTemplateInclude::new(FORWARD_TEMPLATE_TOKEN, FORWARD_TEMPLATE),
                support_includes: vec![
                    environment,
                    light_cookie_include(),
                    irradiance_volume_include(),
                    lightmap_include(),
                    light_grid_include(),
                    shadow,
                    volumetric,
                    pbr_extras,
                ],
                requires_material_surface: true,
                requires_shading_include: true,
                uses_previous_position: false,
            }
        }
        ShaderPassType::GBuffer => ShaderPassTemplate {
            include: ShaderTemplateInclude::new(GBUFFER_TEMPLATE_TOKEN, GBUFFER_TEMPLATE),
            support_includes: Vec::new(),
            requires_material_surface: true,
            requires_shading_include: false,
            uses_previous_position: false,
        },
        ShaderPassType::DepthPrepass => ShaderPassTemplate {
            include: if alpha_test {
                ShaderTemplateInclude::new(DEPTH_ALPHA_TEMPLATE_TOKEN, DEPTH_ALPHA_TEMPLATE)
            } else {
                ShaderTemplateInclude::new(DEPTH_TEMPLATE_TOKEN, DEPTH_TEMPLATE)
            },
            support_includes: Vec::new(),
            requires_material_surface: alpha_test,
            requires_shading_include: false,
            uses_previous_position: false,
        },
        ShaderPassType::Shadow => ShaderPassTemplate {
            include: if alpha_test {
                ShaderTemplateInclude::new(SHADOW_ALPHA_TEMPLATE_TOKEN, SHADOW_ALPHA_TEMPLATE)
            } else {
                ShaderTemplateInclude::new(SHADOW_TEMPLATE_TOKEN, SHADOW_TEMPLATE)
            },
            support_includes: Vec::new(),
            requires_material_surface: alpha_test,
            requires_shading_include: false,
            uses_previous_position: false,
        },
        ShaderPassType::Velocity => ShaderPassTemplate {
            include: if alpha_test {
                ShaderTemplateInclude::new(VELOCITY_ALPHA_TEMPLATE_TOKEN, VELOCITY_ALPHA_TEMPLATE)
            } else {
                ShaderTemplateInclude::new(VELOCITY_TEMPLATE_TOKEN, VELOCITY_TEMPLATE)
            },
            support_includes: Vec::new(),
            requires_material_surface: alpha_test,
            requires_shading_include: false,
            uses_previous_position: true,
        },
        ShaderPassType::TaaReactiveMask => ShaderPassTemplate {
            include: ShaderTemplateInclude::new(
                TAA_REACTIVE_MASK_TEMPLATE_TOKEN,
                TAA_REACTIVE_MASK_TEMPLATE,
            ),
            support_includes: Vec::new(),
            requires_material_surface: true,
            requires_shading_include: false,
            uses_previous_position: false,
        },
        ShaderPassType::HitProxy => ShaderPassTemplate {
            include: if alpha_test {
                ShaderTemplateInclude::new(HIT_PROXY_ALPHA_TEMPLATE_TOKEN, HIT_PROXY_ALPHA_TEMPLATE)
            } else {
                ShaderTemplateInclude::new(HIT_PROXY_TEMPLATE_TOKEN, HIT_PROXY_TEMPLATE)
            },
            support_includes: Vec::new(),
            requires_material_surface: alpha_test,
            requires_shading_include: false,
            uses_previous_position: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forward_shadow_include(features: ShaderFeatureBits) -> ShaderTemplateInclude {
        pass_template_for(ShaderPassType::Forward, features)
            .support_includes
            .into_iter()
            .find(|include| include.token == "zr_shadow.wgsl")
            .expect("forward shader template should provide the shadow API")
    }

    #[test]
    fn forward_without_receive_shadows_uses_binding_free_shadow_stub() {
        let include = forward_shadow_include(ShaderFeatureBits::default());

        assert!(
            include
                .source
                .contains("fn zr_gpu_light_shadow_visibility(")
        );
        assert!(include.source.contains("return 1.0;"));
        assert!(!include.source.contains("@group(1) @binding(8)"));
        assert!(!include.source.contains("fn zr_sample_shadow_slot("));
    }

    #[test]
    fn forward_with_receive_shadows_uses_full_shadow_module() {
        let include =
            forward_shadow_include(ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS));

        assert!(include.source.contains("@group(1) @binding(8)"));
        assert!(include.source.contains("fn zr_sample_shadow_slot("));
    }

    #[test]
    fn shadow_specialization_preserves_token_and_changes_content_hash() {
        let disabled = forward_shadow_include(ShaderFeatureBits::default());
        let enabled =
            forward_shadow_include(ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS));

        assert_eq!(disabled.token, enabled.token);
        assert_ne!(disabled.content_hash, enabled.content_hash);
    }

    #[test]
    fn custom_forward_models_keep_the_full_pbr_support_module_with_environment_only_features() {
        let pbr_extras = pass_template_for_shading_model(
            ShaderPassType::Forward,
            ShaderFeatureBits::new(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR),
            false,
        );
        assert_eq!(pbr_extras.include.token, FORWARD_TEMPLATE_TOKEN);
        let pbr_extras = pbr_extras
            .support_includes
            .into_iter()
            .find(|include| include.token == "zr_pbr_extras.wgsl")
            .expect("custom Forward template should retain the PBR support module");

        for required in [
            "@group(1) @binding(31) var zr_transmission_scene_color",
            "@group(1) @binding(38) var<uniform> zr_transmission_scene_color_params",
            "fn zr_aniso_ggx(",
            "fn zr_clearcoat_lobe(",
            "fn zr_pbr_screen_space_transmission(",
        ] {
            assert!(
                pbr_extras.source.contains(required),
                "custom Forward PBR support must retain `{required}`"
            );
        }
    }
}
