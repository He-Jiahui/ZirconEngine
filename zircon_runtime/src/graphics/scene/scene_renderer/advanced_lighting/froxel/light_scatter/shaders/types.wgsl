struct ZrGpuLightData {
    position_range: vec4<f32>,
    color_intensity: vec4<f32>,
    direction_type: vec4<f32>,
    spot_angles_size: vec4<f32>,
    shadow_slot_layer: vec4<u32>,
    shadow_params: vec4<f32>,
    cookie_uv_rect: vec4<f32>,
    cookie_misc: vec4<u32>,
};

struct VolumetricLightScatterParams {
    grid_and_light_count: vec4<u32>,
    viewport_size: vec4<u32>,
    phase_and_ambient: vec4<f32>,
    view: ZrFroxelViewParams,
    temporal: VolumetricTemporalReprojection,
};

struct VolumetricTemporalReprojection {
    previous_clip_from_world: mat4x4<f32>,
    previous_camera_position: vec4<f32>,
    previous_camera_forward: vec4<f32>,
    previous_depth: vec4<f32>,
    jitter_and_history: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: VolumetricLightScatterParams;
@group(0) @binding(1) var froxel_media: texture_3d<f32>;
@group(0) @binding(2) var<storage, read> zr_light_data: array<ZrGpuLightData>;
@group(0) @binding(3) var froxel_scattering: texture_storage_3d<rgba16float, write>;
@group(0) @binding(4) var previous_froxel_scattering: texture_3d<f32>;

const ZR_GPU_LIGHT_TYPE_DIRECTIONAL: u32 = 0u;
const ZR_GPU_LIGHT_TYPE_POINT: u32 = 1u;
const ZR_GPU_LIGHT_TYPE_SPOT: u32 = 2u;
const ZR_GPU_LIGHT_TYPE_RECT: u32 = 3u;
const ZR_GPU_LIGHT_FLAG_CASTS_SHADOW: u32 = 1u;
const VOLUMETRIC_EPSILON: f32 = 0.000001;
const VOLUMETRIC_INV_FOUR_PI: f32 = 0.0795774715;

fn zr_gpu_light_type(light: ZrGpuLightData) -> u32 {
    return bitcast<u32>(light.direction_type.w);
}

fn zr_gpu_light_casts_shadow(light: ZrGpuLightData) -> bool {
    return (light.shadow_slot_layer.w & ZR_GPU_LIGHT_FLAG_CASTS_SHADOW) != 0u;
}

fn normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    let magnitude = length(value);
    return select(vec3<f32>(0.0), value / magnitude, magnitude > VOLUMETRIC_EPSILON);
}

fn henyey_greenstein(phase_g: f32, cos_theta: f32) -> f32 {
    let g = clamp(phase_g, -0.9, 0.9);
    let denominator = max(1.0 + g * g - 2.0 * g * clamp(cos_theta, -1.0, 1.0), 0.0001);
    return (1.0 - g * g) * VOLUMETRIC_INV_FOUR_PI / (denominator * sqrt(denominator));
}
