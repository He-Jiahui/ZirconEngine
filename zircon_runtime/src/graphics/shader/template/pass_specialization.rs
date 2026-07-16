use crate::core::framework::render::{ShaderFeatureBits, ShaderPassType};

use super::module_registry::{
    environment_include, irradiance_volume_include, light_cookie_include, light_grid_include,
    lightmap_include, pbr_extras_include, shadow_include, volumetric_disabled_include,
    volumetric_include, ShaderTemplateInclude,
};

pub(crate) const MATERIAL_SHADER_TEMPLATE_REVISION: &str = "zr-material-template-v1";

const FORWARD_TEMPLATE_TOKEN: &str = "zr_template_forward.wgsl";
const GBUFFER_TEMPLATE_TOKEN: &str = "zr_template_gbuffer.wgsl";
const DEPTH_TEMPLATE_TOKEN: &str = "zr_template_depth.wgsl";
const DEPTH_ALPHA_TEMPLATE_TOKEN: &str = "zr_template_depth_alpha.wgsl";
const SHADOW_TEMPLATE_TOKEN: &str = "zr_template_shadow.wgsl";
const SHADOW_ALPHA_TEMPLATE_TOKEN: &str = "zr_template_shadow_alpha.wgsl";
const VELOCITY_TEMPLATE_TOKEN: &str = "zr_template_velocity.wgsl";
const VELOCITY_ALPHA_TEMPLATE_TOKEN: &str = "zr_template_velocity_alpha.wgsl";
const TAA_REACTIVE_MASK_TEMPLATE_TOKEN: &str = "zr_template_taa_reactive_mask.wgsl";

const FORWARD_TEMPLATE: &str = include_str!("../wgsl/zr_template_forward.wgsl");
const GBUFFER_TEMPLATE: &str = include_str!("../wgsl/zr_template_gbuffer.wgsl");
const DEPTH_TEMPLATE: &str = include_str!("../wgsl/zr_template_depth.wgsl");
const DEPTH_ALPHA_TEMPLATE: &str = include_str!("../wgsl/zr_template_depth_alpha.wgsl");
const SHADOW_TEMPLATE: &str = include_str!("../wgsl/zr_template_shadow.wgsl");
const SHADOW_ALPHA_TEMPLATE: &str = include_str!("../wgsl/zr_template_shadow_alpha.wgsl");
const VELOCITY_TEMPLATE: &str = include_str!("../wgsl/zr_template_velocity.wgsl");
const VELOCITY_ALPHA_TEMPLATE: &str = include_str!("../wgsl/zr_template_velocity_alpha.wgsl");
const TAA_REACTIVE_MASK_TEMPLATE: &str = include_str!("../wgsl/zr_template_taa_reactive_mask.wgsl");

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
    let alpha_test = features.contains(ShaderFeatureBits::ALPHA_TEST);
    let volumetric = if features.contains(ShaderFeatureBits::VOLUMETRIC_FOG) {
        volumetric_include()
    } else {
        volumetric_disabled_include()
    };
    match pass_type {
        ShaderPassType::Forward => ShaderPassTemplate {
            include: ShaderTemplateInclude::new(FORWARD_TEMPLATE_TOKEN, FORWARD_TEMPLATE),
            support_includes: vec![
                environment_include(),
                light_cookie_include(),
                irradiance_volume_include(),
                lightmap_include(),
                light_grid_include(),
                shadow_include(),
                volumetric,
                pbr_extras_include(),
            ],
            requires_material_surface: true,
            requires_shading_include: true,
            uses_previous_position: false,
        },
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
    }
}
