use crate::hybrid_gi::types::{HybridGiPrepareCardCaptureRequest, HybridGiPrepareVoxelClipmap};
use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderMeshSnapshot, RenderPointLightSnapshot,
    RenderSpotLightSnapshot,
};
use zircon_runtime::core::math::{Vec3, Vec4};
use zircon_runtime::core::resource::ResourceId;

use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use super::material_capture_source::{HybridGiMaterialCaptureSeed, HybridGiMaterialCaptureSource};

fn card_capture_debug_rgba(request: &HybridGiPrepareCardCaptureRequest) -> [u8; 4] {
    [
        (32 + ((request.card_id * 17 + request.atlas_slot_id * 13) % 192)) as u8,
        (32 + ((request.page_id * 11 + request.capture_slot_id * 7) % 192)) as u8,
        (32 + ((request.card_id * 5 + request.page_id * 3 + request.capture_slot_id * 19) % 192))
            as u8,
        255,
    ]
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn clamp_positive(value: f32) -> f32 {
    value.max(0.0)
}

fn saturate_vec3(value: Vec3) -> Vec3 {
    Vec3::new(clamp01(value.x), clamp01(value.y), clamp01(value.z))
}

fn component_mul(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

fn lerp_vec3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}

fn component_div_by_one_plus(color: Vec3) -> Vec3 {
    Vec3::new(
        color.x / (1.0 + color.x),
        color.y / (1.0 + color.y),
        color.z / (1.0 + color.z),
    )
}

pub(super) fn rgba8_from_color_with_alpha(color: Vec3, alpha: u8) -> [u8; 4] {
    let mapped = component_div_by_one_plus(saturate_vec3(color.max(Vec3::ZERO)));
    [
        (mapped.x * 255.0).round().clamp(0.0, 255.0) as u8,
        (mapped.y * 255.0).round().clamp(0.0, 255.0) as u8,
        (mapped.z * 255.0).round().clamp(0.0, 255.0) as u8,
        alpha,
    ]
}

pub(super) fn rgba8_from_color(color: Vec3) -> [u8; 4] {
    rgba8_from_color_with_alpha(color, 255)
}

fn sample_material_texture_rgba(
    streamer: &impl HybridGiMaterialCaptureSource,
    texture_id: Option<ResourceId>,
    uv: [f32; 2],
) -> Vec4 {
    streamer
        .sample_texture_rgba(texture_id, uv)
        .unwrap_or(Vec4::ONE)
}

fn sample_material_texture_rgb(
    streamer: &impl HybridGiMaterialCaptureSource,
    texture_id: Option<ResourceId>,
    uv: [f32; 2],
) -> Vec3 {
    let sample = sample_material_texture_rgba(streamer, texture_id, uv);
    Vec3::new(sample.x, sample.y, sample.z)
}

fn sample_material_texture_value(
    streamer: &impl HybridGiMaterialCaptureSource,
    texture_id: Option<ResourceId>,
    uv: [f32; 2],
    channel: usize,
    default_value: f32,
) -> f32 {
    streamer
        .sample_texture_rgba(texture_id, uv)
        .map(|sample| match channel {
            0 => sample.x,
            1 => sample.y,
            2 => sample.z,
            3 => sample.w,
            _ => default_value,
        })
        .unwrap_or(default_value)
}

fn decode_normal_texture(sample: Vec3) -> Vec3 {
    let decoded = Vec3::new(
        sample.x * 2.0 - 1.0,
        sample.y * 2.0 - 1.0,
        sample.z * 2.0 - 1.0,
    )
    .normalize_or_zero();
    if decoded == Vec3::ZERO {
        Vec3::Z
    } else {
        decoded
    }
}

fn sample_material_normal(
    streamer: &impl HybridGiMaterialCaptureSource,
    texture_id: Option<ResourceId>,
    uv: [f32; 2],
    fallback_normal: Vec3,
) -> Vec3 {
    streamer
        .sample_texture_rgba(texture_id, uv)
        .map(|sample| decode_normal_texture(Vec3::new(sample.x, sample.y, sample.z)))
        .unwrap_or(fallback_normal)
}

fn sample_material_texture_uv(_mesh: &RenderMeshSnapshot, _sample_position: Vec3) -> [f32; 2] {
    // Scene-prepare capture currently only needs deterministic texture-backed surface truth.
    // A stable center sample is sufficient until card capture grows real UV-space capture.
    [0.5, 0.5]
}

fn surface_orientation(card_normal: Vec3, light_direction: Vec3, double_sided: bool) -> f32 {
    let direction = light_direction.normalize_or_zero();
    if direction == Vec3::ZERO {
        return 0.0;
    }

    let facing = card_normal.dot(direction);
    if double_sided {
        facing.abs().max(0.2)
    } else {
        facing.max(0.0)
    }
}

fn card_normal(mesh: &RenderMeshSnapshot) -> Vec3 {
    let normal = -mesh.transform.forward();
    if normal == Vec3::ZERO {
        Vec3::Z
    } else {
        normal
    }
}

fn directional_light_contribution(
    card_normal: Vec3,
    double_sided: bool,
    light: &RenderDirectionalLightSnapshot,
) -> Vec3 {
    let incoming = (-light.direction).normalize_or_zero();
    if incoming == Vec3::ZERO {
        return Vec3::ZERO;
    }

    let strength =
        clamp_positive(light.intensity) * surface_orientation(card_normal, incoming, double_sided);
    saturate_vec3(light.color) * strength
}

fn point_light_contribution(
    card_center: Vec3,
    card_normal: Vec3,
    double_sided: bool,
    light: &RenderPointLightSnapshot,
) -> Vec3 {
    if light.range <= 0.0 {
        return Vec3::ZERO;
    }

    let to_light = light.position - card_center;
    let distance = to_light.length();
    if distance >= light.range {
        return Vec3::ZERO;
    }

    let attenuation = (1.0 - (distance / light.range)).powi(2);
    let strength = clamp_positive(light.intensity)
        * attenuation
        * surface_orientation(card_normal, to_light.normalize_or_zero(), double_sided);
    saturate_vec3(light.color) * strength
}

fn spot_cone_weight(
    light_direction: Vec3,
    to_card: Vec3,
    inner_angle: f32,
    outer_angle: f32,
) -> f32 {
    let light_direction = light_direction.normalize_or_zero();
    let to_card = to_card.normalize_or_zero();
    if light_direction == Vec3::ZERO || to_card == Vec3::ZERO {
        return 0.0;
    }

    let inner_cos = clamp_positive(inner_angle).cos();
    let outer_cos = clamp_positive(outer_angle).cos();
    let (start, end) = if inner_cos >= outer_cos {
        (outer_cos, inner_cos)
    } else {
        (inner_cos, outer_cos)
    };
    let alignment = light_direction.dot(to_card);
    if alignment <= start {
        0.0
    } else if alignment >= end {
        1.0
    } else {
        (alignment - start) / (end - start).max(f32::EPSILON)
    }
}

fn spot_light_contribution(
    card_center: Vec3,
    card_normal: Vec3,
    double_sided: bool,
    light: &RenderSpotLightSnapshot,
) -> Vec3 {
    if light.range <= 0.0 {
        return Vec3::ZERO;
    }

    let to_light = light.position - card_center;
    let distance = to_light.length();
    if distance >= light.range {
        return Vec3::ZERO;
    }

    let attenuation = (1.0 - (distance / light.range)).powi(2);
    let cone_weight = spot_cone_weight(
        light.direction,
        card_center - light.position,
        light.inner_angle_radians,
        light.outer_angle_radians,
    );
    if cone_weight <= 0.0 {
        return Vec3::ZERO;
    }

    let strength = clamp_positive(light.intensity)
        * attenuation
        * cone_weight
        * surface_orientation(card_normal, to_light.normalize_or_zero(), double_sided);
    saturate_vec3(light.color) * strength
}

fn default_material_capture_seed() -> HybridGiMaterialCaptureSeed {
    HybridGiMaterialCaptureSeed {
        base_color: Vec4::ONE,
        emissive: Vec3::ZERO,
        metallic: 0.0,
        roughness: 1.0,
        double_sided: false,
        alpha_blend: false,
        alpha_cutoff: None,
        base_color_texture: None,
        normal_texture: None,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive_texture: None,
    }
}

fn material_capture_seed(
    streamer: &impl HybridGiMaterialCaptureSource,
    mesh: &RenderMeshSnapshot,
) -> HybridGiMaterialCaptureSeed {
    streamer
        .material_capture_seed(&mesh.material.id())
        .unwrap_or_else(default_material_capture_seed)
}

pub(super) fn mesh_capture_radiance(
    mesh: &RenderMeshSnapshot,
    sample_position: Vec3,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec3 {
    let material = material_capture_seed(streamer, mesh);
    let texture_uv = sample_material_texture_uv(mesh, sample_position);
    let tint = saturate_vec3(Vec3::new(mesh.tint.x, mesh.tint.y, mesh.tint.z));
    let base_color_texture =
        sample_material_texture_rgba(streamer, material.base_color_texture, texture_uv);
    let metallic_roughness_texture =
        sample_material_texture_rgb(streamer, material.metallic_roughness_texture, texture_uv);
    let occlusion = clamp01(sample_material_texture_value(
        streamer,
        material.occlusion_texture,
        texture_uv,
        0,
        1.0,
    ));
    let emissive_texture =
        sample_material_texture_rgb(streamer, material.emissive_texture, texture_uv);
    let material_albedo = component_mul(
        saturate_vec3(Vec3::new(
            material.base_color.x,
            material.base_color.y,
            material.base_color.z,
        )),
        Vec3::new(
            base_color_texture.x,
            base_color_texture.y,
            base_color_texture.z,
        ),
    );
    let surface_alpha = clamp01(material.base_color.w * base_color_texture.w);
    if material
        .alpha_cutoff
        .is_some_and(|cutoff| surface_alpha < clamp01(cutoff))
    {
        return Vec3::ZERO;
    }
    let alpha_scale = if material.alpha_blend {
        surface_alpha
    } else {
        1.0
    };
    let metallic = clamp01(material.metallic * metallic_roughness_texture.z);
    let roughness = clamp01(material.roughness * metallic_roughness_texture.y);
    let smoothness = 1.0 - roughness;
    let albedo = component_mul(material_albedo, tint);
    let emissive = component_mul(
        component_mul(saturate_vec3(material.emissive), emissive_texture),
        tint,
    );
    let diffuse_albedo = albedo * (1.0 - metallic);
    let specular_f0 = lerp_vec3(Vec3::splat(0.04), albedo, metallic);
    let ambient = diffuse_albedo * (0.03 + roughness * 0.05) * occlusion;
    let card_normal = sample_material_normal(
        streamer,
        material.normal_texture,
        texture_uv,
        card_normal(mesh),
    );
    let direct_light = inputs
        .directional_lights
        .iter()
        .fold(Vec3::ZERO, |acc, light| {
            acc + directional_light_contribution(card_normal, material.double_sided, light)
        })
        + inputs.point_lights.iter().fold(Vec3::ZERO, |acc, light| {
            acc + point_light_contribution(
                sample_position,
                card_normal,
                material.double_sided,
                light,
            )
        })
        + inputs.spot_lights.iter().fold(Vec3::ZERO, |acc, light| {
            acc + spot_light_contribution(
                sample_position,
                card_normal,
                material.double_sided,
                light,
            )
        });
    let diffuse_response = component_mul(diffuse_albedo, direct_light * (0.18 + roughness * 0.34));
    let specular_response = component_mul(
        specular_f0,
        direct_light * (0.08 + smoothness * (1.1 + metallic * 0.55)),
    );

    (ambient + diffuse_response + specular_response + emissive) * alpha_scale
}

pub(super) fn scene_card_capture_rgba(
    request: &HybridGiPrepareCardCaptureRequest,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
) -> [u8; 4] {
    let Some(mesh) = inputs
        .scene_meshes
        .iter()
        .find(|mesh| mesh.node_id as u32 == request.card_id)
    else {
        return card_capture_debug_rgba(request);
    };

    rgba8_from_color(mesh_capture_radiance(
        mesh,
        request.bounds_center,
        streamer,
        inputs,
    ))
}

pub(super) fn scene_voxel_clipmap_rgba(
    clipmap: &HybridGiPrepareVoxelClipmap,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
) -> [u8; 4] {
    if clipmap.half_extent <= 0.0 {
        return rgba8_from_color_with_alpha(Vec3::ZERO, 0);
    }

    let mut has_sample = false;
    let radiance = inputs
        .scene_meshes
        .iter()
        .filter_map(|mesh| {
            let sample_position = mesh.transform.translation;
            let local_position = sample_position - clipmap.center;
            if local_position.abs().max_element() > clipmap.half_extent {
                return None;
            }

            has_sample = true;
            Some(mesh_capture_radiance(
                mesh,
                sample_position,
                streamer,
                inputs,
            ))
        })
        .fold(Vec3::ZERO, |acc, sample| acc + sample);

    rgba8_from_color_with_alpha(radiance, if has_sample { 255 } else { 0 })
}
