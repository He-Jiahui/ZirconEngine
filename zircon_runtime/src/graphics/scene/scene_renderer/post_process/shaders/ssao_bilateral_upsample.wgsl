struct SceneUniform {
    view_proj: mat4x4<f32>,
    view_proj_unjittered: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    ambient_color: vec4<f32>,
    lightmapped_ambient_color: vec4<f32>,
    previous_view_proj_unjittered: mat4x4<f32>,
    motion_params: vec4<f32>,
    jitter_params: vec4<f32>,
    camera_world_position: vec4<f32>,
    camera_view_direction: vec4<f32>,
};

struct SsaoParams {
    extent_and_sample_counts: vec4<u32>,
    input_extent_and_resolution: vec4<u32>,
    world_radius_thickness_bias_falloff: vec4<f32>,
    intensity_and_limits: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var spatial_ao_tex: texture_2d<f32>;
@group(1) @binding(1) var depth_tex: texture_depth_2d;
@group(1) @binding(2) var normal_tex: texture_2d<f32>;
@group(1) @binding(3) var<uniform> params: SsaoParams;
@group(1) @binding(4) var full_ao_out: texture_storage_2d<rgba8unorm, write>;

const EPSILON: f32 = 0.000001;
const UPSAMPLE_NORMAL_REJECTION_POWER: f32 = 16.0;
const UPSAMPLE_DEPTH_SIGMA_SCALE: f32 = 0.5;

fn screen_uv_to_clip(uv: vec2<f32>, depth: f32) -> vec4<f32> {
    return vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
}

fn reconstruct_world_position(coord: vec2<i32>, depth: f32, input_extent: vec2<u32>) -> vec3<f32> {
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5)) / vec2<f32>(input_extent);
    let world = scene.inverse_view_proj * screen_uv_to_clip(uv, depth);
    if (abs(world.w) <= EPSILON) {
        return vec3<f32>(0.0);
    }
    return world.xyz / world.w;
}

fn decode_normal_or_zero(coord: vec2<i32>) -> vec3<f32> {
    let encoded = textureLoad(normal_tex, coord, 0).xyz * 2.0 - vec3<f32>(1.0);
    let encoded_length = length(encoded);
    if (encoded_length <= EPSILON) {
        return vec3<f32>(0.0);
    }
    return encoded / encoded_length;
}

fn work_to_input_coord(
    work_coord: vec2<i32>,
    input_extent: vec2<u32>,
    resolution_divisor: u32,
) -> vec2<i32> {
    let input_coord = vec2<u32>(work_coord) * resolution_divisor;
    return vec2<i32>(min(input_coord, input_extent - vec2<u32>(1u)));
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let input_extent = max(params.input_extent_and_resolution.xy, vec2<u32>(1u));
    if (invocation_id.x >= input_extent.x || invocation_id.y >= input_extent.y) {
        return;
    }

    let resolution_divisor = max(params.input_extent_and_resolution.z, 1u);
    let work_extent = max(params.extent_and_sample_counts.xy, vec2<u32>(1u));
    let input_coord = vec2<i32>(invocation_id.xy);
    let center_depth = textureLoad(depth_tex, input_coord, 0);
    if (center_depth >= 0.999999) {
        textureStore(full_ao_out, input_coord, vec4<f32>(1.0));
        return;
    }

    let center_position = reconstruct_world_position(input_coord, center_depth, input_extent);
    let center_normal = decode_normal_or_zero(input_coord);
    if (dot(center_normal, center_normal) <= EPSILON) {
        textureStore(full_ao_out, input_coord, vec4<f32>(1.0));
        return;
    }

    let depth_sigma = max(
        params.world_radius_thickness_bias_falloff.y * UPSAMPLE_DEPTH_SIGMA_SCALE,
        params.world_radius_thickness_bias_falloff.z * 2.0 + EPSILON,
    );
    let work_position = vec2<f32>(invocation_id.xy) / f32(resolution_divisor);
    let base_work_coord = vec2<i32>(floor(work_position));
    let work_fraction = fract(work_position);
    let maximum_work_coord = vec2<i32>(work_extent) - vec2<i32>(1);
    var weighted_visibility = 0.0;
    var total_weight = 0.0;

    for (var offset_y: i32 = 0; offset_y <= 1; offset_y += 1) {
        for (var offset_x: i32 = 0; offset_x <= 1; offset_x += 1) {
            let sample_work_coord = clamp(
                base_work_coord + vec2<i32>(offset_x, offset_y),
                vec2<i32>(0),
                maximum_work_coord,
            );
            let sample_input_coord = work_to_input_coord(
                sample_work_coord,
                input_extent,
                resolution_divisor,
            );
            let sample_depth = textureLoad(depth_tex, sample_input_coord, 0);
            if (sample_depth >= 0.999999) {
                continue;
            }
            let sample_position = reconstruct_world_position(
                sample_input_coord,
                sample_depth,
                input_extent,
            );
            let sample_normal = decode_normal_or_zero(sample_input_coord);
            if (dot(sample_normal, sample_normal) <= EPSILON) {
                continue;
            }

            let bilinear_x = select(1.0 - work_fraction.x, work_fraction.x, offset_x == 1);
            let bilinear_y = select(1.0 - work_fraction.y, work_fraction.y, offset_y == 1);
            let plane_distance = abs(dot(sample_position - center_position, center_normal));
            let depth_weight = exp2(-plane_distance / depth_sigma);
            let normal_weight = pow(
                max(dot(center_normal, sample_normal), 0.0),
                UPSAMPLE_NORMAL_REJECTION_POWER,
            );
            let weight = bilinear_x * bilinear_y * depth_weight * normal_weight;
            weighted_visibility += textureLoad(spatial_ao_tex, sample_work_coord, 0).r * weight;
            total_weight += weight;
        }
    }

    let fallback_work_coord = clamp(
        base_work_coord,
        vec2<i32>(0),
        maximum_work_coord,
    );
    var visibility = textureLoad(spatial_ao_tex, fallback_work_coord, 0).r;
    if (total_weight > EPSILON) {
        visibility = weighted_visibility / total_weight;
    }
    textureStore(
        full_ao_out,
        input_coord,
        vec4<f32>(visibility, visibility, visibility, 1.0),
    );
}
