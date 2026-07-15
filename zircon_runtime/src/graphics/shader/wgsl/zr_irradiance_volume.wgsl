struct ZrIrradianceVolumeParams {
    world_to_volume: mat4x4<f32>,
    intensity_enabled: vec4<f32>,
    flags: vec4<u32>,
    normal_to_volume: mat3x3<f32>,
};

struct ZrIrradianceVolumeSample {
    irradiance: vec3<f32>,
    valid: u32,
};

@group(1) @binding(35) var zr_irradiance_volume_texture: texture_3d<f32>;
@group(1) @binding(36) var zr_irradiance_volume_sampler: sampler;
@group(1) @binding(37) var<uniform> zr_irradiance_volume_params: ZrIrradianceVolumeParams;

fn zr_irradiance_volume_sample(
    world_position: vec3<f32>,
    normal_ws: vec3<f32>,
    lightmapped: bool,
) -> ZrIrradianceVolumeSample {
    if (zr_irradiance_volume_params.intensity_enabled.y < 0.5) {
        return ZrIrradianceVolumeSample(vec3<f32>(0.0), 0u);
    }
    if (lightmapped && zr_irradiance_volume_params.flags.x == 0u) {
        return ZrIrradianceVolumeSample(vec3<f32>(0.0), 0u);
    }
    let local_position = (zr_irradiance_volume_params.world_to_volume
        * vec4<f32>(world_position, 1.0)).xyz;
    if (any(local_position < vec3<f32>(-0.5)) || any(local_position > vec3<f32>(0.5))) {
        return ZrIrradianceVolumeSample(vec3<f32>(0.0), 0u);
    }

    let atlas_dimensions_u = textureDimensions(zr_irradiance_volume_texture);
    let logical_dimensions_u = atlas_dimensions_u / vec3<u32>(1u, 2u, 3u);
    if (any(logical_dimensions_u == vec3<u32>(0u))) {
        return ZrIrradianceVolumeSample(vec3<f32>(0.0), 0u);
    }
    let atlas_dimensions = vec3<f32>(atlas_dimensions_u);
    let logical_dimensions = vec3<f32>(logical_dimensions_u);
    let sample_texel = clamp(
        (local_position + vec3<f32>(0.5)) * logical_dimensions,
        vec3<f32>(0.5),
        logical_dimensions - vec3<f32>(0.5),
    );
    let base_uvw = sample_texel / atlas_dimensions;
    let local_normal = zr_irradiance_volume_params.normal_to_volume * normal_ws;
    let normal_length = length(local_normal);
    let normal = select(vec3<f32>(0.0, 1.0, 0.0), local_normal / normal_length, normal_length > 0.000001);
    let negative_offset = select(vec3<f32>(0.0), vec3<f32>(0.5), normal < vec3<f32>(0.0));
    let sample_x = textureSampleLevel(
        zr_irradiance_volume_texture,
        zr_irradiance_volume_sampler,
        base_uvw + vec3<f32>(0.0, negative_offset.x, 0.0),
        0.0,
    ).rgb;
    let sample_y = textureSampleLevel(
        zr_irradiance_volume_texture,
        zr_irradiance_volume_sampler,
        base_uvw + vec3<f32>(0.0, negative_offset.y, 1.0 / 3.0),
        0.0,
    ).rgb;
    let sample_z = textureSampleLevel(
        zr_irradiance_volume_texture,
        zr_irradiance_volume_sampler,
        base_uvw + vec3<f32>(0.0, negative_offset.z, 2.0 / 3.0),
        0.0,
    ).rgb;
    let normal_squared = normal * normal;
    let irradiance = (sample_x * normal_squared.x
        + sample_y * normal_squared.y
        + sample_z * normal_squared.z)
        * zr_irradiance_volume_params.intensity_enabled.x;
    return ZrIrradianceVolumeSample(max(irradiance, vec3<f32>(0.0)), 1u);
}
