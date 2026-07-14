use crate::core::framework::render::{
    RenderMaterialAlphaMode, ShaderFeatureBits, StandardMaterialDescriptor,
};

const STANDARD_MATERIAL_SURFACE_ENTRY_POINT: &str = "standard_material_surface";

const STANDARD_MATERIAL_SURFACE_SOURCE: &str = r#"
struct StandardMaterialPropertyUniform {
    data0: vec4<f32>,
    data1: vec4<f32>,
    data2: vec4<f32>,
    data3: vec4<f32>,
    data4: vec4<f32>,
    data5: vec4<f32>,
    data6: vec4<f32>,
    data7: vec4<f32>,
    data8: vec4<f32>,
    data9: vec4<f32>,
    data10: vec4<f32>,
    data11: vec4<f32>,
};

@group(2) @binding(0) var<uniform> standard_material_properties: StandardMaterialPropertyUniform;
@group(2) @binding(1) var standard_material_base_color_tex: texture_2d<f32>;
@group(2) @binding(2) var standard_material_base_color_sampler: sampler;
@group(2) @binding(3) var standard_material_normal_tex: texture_2d<f32>;
@group(2) @binding(4) var standard_material_normal_sampler: sampler;
@group(2) @binding(5) var standard_material_metallic_roughness_tex: texture_2d<f32>;
@group(2) @binding(6) var standard_material_metallic_roughness_sampler: sampler;
@group(2) @binding(7) var standard_material_occlusion_tex: texture_2d<f32>;
@group(2) @binding(8) var standard_material_occlusion_sampler: sampler;
@group(2) @binding(9) var standard_material_emissive_tex: texture_2d<f32>;
@group(2) @binding(10) var standard_material_emissive_sampler: sampler;
@group(2) @binding(11) var standard_material_clearcoat_normal_tex: texture_2d<f32>;
@group(2) @binding(12) var standard_material_clearcoat_normal_sampler: sampler;

const ZR_STANDARD_MATERIAL_SURFACE_MIN_ROUGHNESS: f32 = 0.001;

fn standard_material_select_uv(uv0: vec2<f32>, uv1: vec2<f32>, channel: f32) -> vec2<f32> {
    if (channel >= 0.5) {
        return uv1;
    }
    return uv0;
}

fn standard_material_transform_uv(uv: vec2<f32>, transform: vec4<f32>) -> vec2<f32> {
    return uv * transform.xy + transform.zw;
}

fn standard_material_transform_uv_channel(
    uv0: vec2<f32>,
    uv1: vec2<f32>,
    transform: vec4<f32>,
    channel: f32,
) -> vec2<f32> {
    return standard_material_transform_uv(
        standard_material_select_uv(uv0, uv1, channel),
        transform,
    );
}

fn standard_material_normalize_or_fallback(value: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    if (length(value) <= 0.00001) {
        return normalize(fallback);
    }
    return normalize(value);
}

struct StandardMaterialTangentFrame {
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
    normal: vec3<f32>,
};

fn standard_material_tangent_frame(
    input: ZrVertexOutput,
    normal_ws: vec3<f32>,
) -> StandardMaterialTangentFrame {
    let normal = standard_material_normalize_or_fallback(
        normal_ws,
        vec3<f32>(0.0, 0.0, 1.0),
    );
    var tangent = input.tangent_ws - normal * dot(input.tangent_ws, normal);
    if (length(tangent) <= 0.00001) {
        let helper_axis = select(
            vec3<f32>(0.0, 1.0, 0.0),
            vec3<f32>(1.0, 0.0, 0.0),
            abs(normal.x) < 0.9,
        );
        tangent = cross(helper_axis, normal);
    }
    tangent = normalize(tangent);
    let bitangent = standard_material_normalize_or_fallback(
        cross(normal, tangent) * input.tangent_handedness,
        vec3<f32>(0.0, 1.0, 0.0),
    );
    return StandardMaterialTangentFrame(tangent, bitangent, normal);
}

fn standard_material_tangent_normal(
    sampled_normal: vec3<f32>,
    frame: StandardMaterialTangentFrame,
) -> vec3<f32> {
    return standard_material_normalize_or_fallback(
        frame.tangent * sampled_normal.x
            + frame.bitangent * sampled_normal.y
            + frame.normal * sampled_normal.z,
        frame.normal,
    );
}

fn standard_material_sampled_normal(input: ZrVertexOutput, normal_uv: vec2<f32>) -> vec3<f32> {
    let geometric_normal = standard_material_normalize_or_fallback(
        input.normal_ws,
        vec3<f32>(0.0, 0.0, 1.0),
    );
    if (!ZR_FEATURE_HAS_NORMAL_TEXTURE) {
        return geometric_normal;
    }
    let frame = standard_material_tangent_frame(input, geometric_normal);
    let sampled_normal = textureSample(
        standard_material_normal_tex,
        standard_material_normal_sampler,
        normal_uv,
    ).xyz * 2.0 - vec3<f32>(1.0, 1.0, 1.0);
    return standard_material_tangent_normal(sampled_normal, frame);
}

fn standard_material_sampled_clearcoat_normal(
    input: ZrVertexOutput,
    normal_uv: vec2<f32>,
) -> vec3<f32> {
    let frame = standard_material_tangent_frame(input, input.normal_ws);
    let sampled_normal = textureSample(
        standard_material_clearcoat_normal_tex,
        standard_material_clearcoat_normal_sampler,
        normal_uv,
    ).xyz * 2.0 - vec3<f32>(1.0, 1.0, 1.0);
    return standard_material_tangent_normal(sampled_normal, frame);
}

fn standard_material_alpha_cutoff() -> f32 {
    let uniform_cutoff = clamp(standard_material_properties.data8.z, 0.0, 1.0);
    if (uniform_cutoff > 0.0) {
        return uniform_cutoff;
    }
    return ZR_STANDARD_MATERIAL_ALPHA_CUTOFF;
}

fn standard_material_decode_shading_model_id(encoded: f32) -> u32 {
    return u32(round(clamp(encoded, 0.0, 1.0) * 255.0));
}

fn standard_material_shading_model_id() -> u32 {
    return select(
        standard_material_decode_shading_model_id(standard_material_properties.data8.y),
        0u,
        standard_material_properties.data0.w >= 0.5,
    );
}

fn standard_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    let base_color_uv = standard_material_transform_uv_channel(
        input.uv0,
        input.uv1,
        standard_material_properties.data2,
        standard_material_properties.data7.x,
    );
    let normal_uv = standard_material_transform_uv_channel(
        input.uv0,
        input.uv1,
        standard_material_properties.data3,
        standard_material_properties.data7.y,
    );
    let metallic_roughness_uv = standard_material_transform_uv_channel(
        input.uv0,
        input.uv1,
        standard_material_properties.data4,
        standard_material_properties.data7.z,
    );
    let occlusion_uv = standard_material_transform_uv_channel(
        input.uv0,
        input.uv1,
        standard_material_properties.data5,
        standard_material_properties.data7.w,
    );
    let emissive_uv = standard_material_transform_uv_channel(
        input.uv0,
        input.uv1,
        standard_material_properties.data6,
        standard_material_properties.data1.w,
    );

    let sampled_base = textureSample(
        standard_material_base_color_tex,
        standard_material_base_color_sampler,
        base_color_uv,
    ) * input.tint * input.color;
    let metallic_roughness = textureSample(
        standard_material_metallic_roughness_tex,
        standard_material_metallic_roughness_sampler,
        metallic_roughness_uv,
    );
    let occlusion_sample = textureSample(
        standard_material_occlusion_tex,
        standard_material_occlusion_sampler,
        occlusion_uv,
    ).r;
    let emissive_sample = textureSample(
        standard_material_emissive_tex,
        standard_material_emissive_sampler,
        emissive_uv,
    ).rgb;

    var surface: ZrSurfaceOutput;
    surface.base_color = sampled_base;
    surface.normal_ws = standard_material_sampled_normal(input, normal_uv);
    let surface_frame = standard_material_tangent_frame(input, surface.normal_ws);
    surface.tangent_ws = surface_frame.tangent;
    surface.bitangent_ws = surface_frame.bitangent;
    surface.metallic = clamp(standard_material_properties.data0.x * metallic_roughness.b, 0.0, 1.0);
    surface.roughness = clamp(standard_material_properties.data0.y * metallic_roughness.g, ZR_STANDARD_MATERIAL_SURFACE_MIN_ROUGHNESS, 1.0);
    surface.occlusion = clamp(standard_material_properties.data0.z * occlusion_sample, 0.0, 1.0);
    surface.emissive = max(standard_material_properties.data1.rgb, vec3<f32>(0.0)) * emissive_sample;
    surface.alpha_cutoff = standard_material_alpha_cutoff();
    surface.unlit = standard_material_properties.data0.w;
    surface.shading_model_id = standard_material_shading_model_id();
    surface.clearcoat_normal_ws = standard_material_sampled_clearcoat_normal(input, normal_uv);
    surface.clearcoat = select(0.0, clamp(standard_material_properties.data9.x, 0.0, 1.0), ZR_FEATURE_PBR_CLEARCOAT);
    surface.clearcoat_roughness = select(0.5, clamp(standard_material_properties.data9.y, ZR_STANDARD_MATERIAL_SURFACE_MIN_ROUGHNESS, 1.0), ZR_FEATURE_PBR_CLEARCOAT);
    surface.anisotropy_strength = select(0.0, clamp(standard_material_properties.data9.z, 0.0, 1.0), ZR_FEATURE_PBR_ANISOTROPY);
    surface.anisotropy_rotation = select(0.0, standard_material_properties.data9.w, ZR_FEATURE_PBR_ANISOTROPY);
    surface.specular_transmission = select(0.0, clamp(standard_material_properties.data10.x, 0.0, 1.0), ZR_FEATURE_PBR_TRANSMISSION);
    surface.diffuse_transmission = select(0.0, clamp(standard_material_properties.data10.y, 0.0, 1.0), ZR_FEATURE_PBR_TRANSMISSION);
    surface.thickness = select(0.0, max(standard_material_properties.data10.z, 0.0), ZR_FEATURE_PBR_TRANSMISSION);
    surface.ior = select(1.5, max(standard_material_properties.data10.w, 1.0), ZR_FEATURE_PBR_TRANSMISSION);
    surface.attenuation_color = select(vec3<f32>(1.0), clamp(standard_material_properties.data11.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), ZR_FEATURE_PBR_TRANSMISSION);
    surface.attenuation_distance = select(1.0e30, max(standard_material_properties.data11.w, 0.000001), ZR_FEATURE_PBR_TRANSMISSION);
    surface.custom0 = vec4<f32>(
        standard_material_properties.data8.x,
        standard_material_properties.data8.y,
        standard_material_properties.data8.z,
        f32(standard_material_decode_shading_model_id(standard_material_properties.data8.w)),
    );
    return surface;
}
"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StandardMaterialSurfaceSource {
    pub(crate) source: String,
    pub(crate) entry_point: &'static str,
    pub(crate) features: ShaderFeatureBits,
}

pub(crate) fn standard_material_surface_source(
    descriptor: &StandardMaterialDescriptor,
) -> StandardMaterialSurfaceSource {
    standard_material_surface_source_for_features(
        standard_material_shader_features(descriptor),
        standard_material_alpha_cutoff(descriptor),
    )
}

pub(crate) fn standard_material_surface_source_for_features(
    features: ShaderFeatureBits,
    alpha_cutoff: f32,
) -> StandardMaterialSurfaceSource {
    StandardMaterialSurfaceSource {
        source: format!(
            "const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = {};\n{}",
            format_wgsl_f32(clamp_alpha_cutoff(alpha_cutoff)),
            STANDARD_MATERIAL_SURFACE_SOURCE
        ),
        entry_point: STANDARD_MATERIAL_SURFACE_ENTRY_POINT,
        features,
    }
}

fn standard_material_shader_features(descriptor: &StandardMaterialDescriptor) -> ShaderFeatureBits {
    let mut bits = 0;
    if matches!(descriptor.alpha_mode, RenderMaterialAlphaMode::Mask { .. }) {
        bits |= ShaderFeatureBits::ALPHA_TEST;
    }
    if descriptor.receive_shadows {
        bits |= ShaderFeatureBits::RECEIVE_SHADOWS;
    }
    if descriptor.double_sided {
        bits |= ShaderFeatureBits::DOUBLE_SIDED;
    }
    if descriptor.normal_texture.is_some() {
        bits |= ShaderFeatureBits::HAS_NORMAL_TEXTURE;
    }
    if descriptor.advanced_features.uses_clearcoat() {
        bits |= ShaderFeatureBits::PBR_CLEARCOAT;
    }
    if descriptor.advanced_features.uses_anisotropy() {
        bits |= ShaderFeatureBits::PBR_ANISOTROPY;
    }
    if descriptor.advanced_features.uses_transmission() {
        bits |= ShaderFeatureBits::PBR_TRANSMISSION;
    }
    ShaderFeatureBits::new(bits)
}

fn standard_material_alpha_cutoff(descriptor: &StandardMaterialDescriptor) -> f32 {
    match descriptor.alpha_mode {
        RenderMaterialAlphaMode::Mask { cutoff } => clamp_alpha_cutoff(cutoff),
        RenderMaterialAlphaMode::Opaque | RenderMaterialAlphaMode::Blend => 0.0,
    }
}

fn clamp_alpha_cutoff(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn format_wgsl_f32(value: f32) -> String {
    format!("{value:.8}")
}
