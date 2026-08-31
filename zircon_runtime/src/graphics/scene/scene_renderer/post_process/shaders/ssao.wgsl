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
    // xy = AO work extent, z = slice count, w = samples per slice side.
    extent_and_sample_counts: vec4<u32>,
    // xy = full-resolution input extent, z = resolution divisor, w = reserved.
    input_extent_and_resolution: vec4<u32>,
    // x = radius, y = thickness, z = depth bias, w = falloff start; all in world units.
    world_radius_thickness_bias_falloff: vec4<f32>,
    // x = intensity, y = maximum HZB mip, z = maximum projected radius, w = minimum visibility.
    intensity_and_limits: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var depth_tex: texture_depth_2d;
@group(1) @binding(1) var normal_tex: texture_2d<f32>;
@group(1) @binding(2) var<uniform> params: SsaoParams;
@group(1) @binding(3) var ao_out: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(4) var hzb_furthest_tex: texture_2d<f32>;

const PI: f32 = 3.14159265358979323846;
const HALF_PI: f32 = 1.57079632679489661923;
const EPSILON: f32 = 0.000001;
const MAX_SLICE_COUNT: u32 = 9u;
const MAX_SAMPLES_PER_SLICE_SIDE: u32 = 3u;

fn screen_uv_to_clip(uv: vec2<f32>, depth: f32) -> vec4<f32> {
    return vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
}

fn reconstruct_world_position(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let world = scene.inverse_view_proj * screen_uv_to_clip(uv, depth);
    if (abs(world.w) <= EPSILON) {
        return vec3<f32>(0.0);
    }
    return world.xyz / world.w;
}

fn normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    let value_length = length(value);
    if (value_length <= EPSILON) {
        return vec3<f32>(0.0);
    }
    return value / value_length;
}

fn normalize_or(value: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let normalized = normalize_or_zero(value);
    if (dot(normalized, normalized) <= EPSILON) {
        return fallback;
    }
    return normalized;
}

fn pixel_noise(pixel: vec2<u32>) -> vec2<f32> {
    let hash = pixel.x * 0x1f123bb5u ^ pixel.y * 0x5f356495u;
    return fract(
        vec2<f32>(f32(hash), f32(hash ^ 0x9e3779b9u))
            * vec2<f32>(0.0000001192092896, 0.0000001732050808)
    );
}

fn load_hzb_world_position(uv: vec2<f32>, requested_mip: u32) -> vec3<f32> {
    let mip_count = max(textureNumLevels(hzb_furthest_tex), 1u);
    let maximum_mip = min(u32(params.intensity_and_limits.y), mip_count - 1u);
    let mip = min(requested_mip, maximum_mip);
    let mip_extent = max(textureDimensions(hzb_furthest_tex, mip), vec2<u32>(1u));
    let safe_uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(0.99999994));
    let coord = min(vec2<u32>(safe_uv * vec2<f32>(mip_extent)), mip_extent - vec2<u32>(1u));
    let depth = textureLoad(hzb_furthest_tex, vec2<i32>(coord), mip).r;
    return reconstruct_world_position(safe_uv, depth);
}

fn update_sectors(
    minimum_horizon: f32,
    maximum_horizon: f32,
    sector_count: u32,
    bitmask: u32,
) -> u32 {
    if (maximum_horizon <= minimum_horizon || sector_count == 0u) {
        return bitmask;
    }
    let start = min(u32(minimum_horizon * f32(sector_count)), sector_count - 1u);
    let count = min(
        u32(ceil((maximum_horizon - minimum_horizon) * f32(sector_count))),
        sector_count - start,
    );
    if (count == 0u) {
        return bitmask;
    }
    let sector_mask = ((1u << count) - 1u) << start;
    return bitmask | sector_mask;
}

fn process_sample(
    delta_position: vec3<f32>,
    view_vector: vec3<f32>,
    sampling_direction: f32,
    normal_horizon: f32,
    thickness: f32,
    sector_count: u32,
    bitmask: u32,
) -> u32 {
    let distance_to_sample = length(delta_position);
    let radius = params.world_radius_thickness_bias_falloff.x;
    if (distance_to_sample <= EPSILON || distance_to_sample > radius) {
        return bitmask;
    }

    let falloff_start = params.world_radius_thickness_bias_falloff.w;
    var falloff = 1.0;
    let falloff_span = radius - falloff_start;
    if (falloff_span > EPSILON) {
        falloff = clamp((radius - distance_to_sample) / falloff_span, 0.0, 1.0);
    }
    if (falloff <= EPSILON) {
        return bitmask;
    }
    let delta_direction = delta_position / distance_to_sample;
    let back_face = delta_position - view_vector * thickness * falloff;
    let front_angle = acos(clamp(dot(delta_direction, view_vector), -1.0, 1.0)) / PI;
    let back_angle = acos(clamp(dot(normalize_or(back_face, delta_direction), view_vector), -1.0, 1.0)) / PI;
    var horizons = clamp(
        vec2<f32>(normal_horizon) - vec2<f32>(sampling_direction) * vec2<f32>(front_angle, back_angle),
        vec2<f32>(0.0),
        vec2<f32>(1.0),
    );
    if (sampling_direction >= 0.0) {
        horizons = horizons.yx;
    }
    return update_sectors(horizons.x, horizons.y, sector_count, bitmask);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let work_extent = params.extent_and_sample_counts.xy;
    if (invocation_id.x >= work_extent.x || invocation_id.y >= work_extent.y) {
        return;
    }

    let input_extent = max(params.input_extent_and_resolution.xy, vec2<u32>(1u));
    let resolution_divisor = max(params.input_extent_and_resolution.z, 1u);
    let work_coord = vec2<i32>(invocation_id.xy);
    let input_coord_u32 = min(
        invocation_id.xy * resolution_divisor,
        input_extent - vec2<u32>(1u),
    );
    let input_coord = vec2<i32>(input_coord_u32);
    let center_depth = textureLoad(depth_tex, input_coord, 0);
    if (center_depth >= 0.999999) {
        textureStore(ao_out, work_coord, vec4<f32>(1.0));
        return;
    }

    let uv = (vec2<f32>(input_coord_u32) + vec2<f32>(0.5)) / vec2<f32>(input_extent);
    let center_position_unbiased = reconstruct_world_position(uv, center_depth);
    let encoded_normal = textureLoad(normal_tex, input_coord, 0).xyz;
    let world_normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0));
    if (dot(world_normal, world_normal) <= EPSILON) {
        textureStore(ao_out, work_coord, vec4<f32>(1.0));
        return;
    }
    let center_position = center_position_unbiased
        + world_normal * params.world_radius_thickness_bias_falloff.z;

    let perspective_view = normalize_or(
        scene.camera_world_position.xyz - center_position,
        vec3<f32>(0.0, 0.0, 1.0),
    );
    let orthographic_view = normalize_or(scene.camera_view_direction.xyz, perspective_view);
    let view_vector = select(
        perspective_view,
        orthographic_view,
        scene.camera_view_direction.w >= 0.5,
    );

    let one_work_pixel_uv = vec2<f32>(f32(resolution_divisor)) / vec2<f32>(input_extent);
    let world_right = reconstruct_world_position(uv + vec2<f32>(one_work_pixel_uv.x, 0.0), center_depth)
        - center_position_unbiased;
    let world_down = reconstruct_world_position(uv + vec2<f32>(0.0, one_work_pixel_uv.y), center_depth)
        - center_position_unbiased;
    let meters_per_pixel = max(min(length(world_right), length(world_down)), EPSILON);
    let radius_pixels = clamp(
        params.world_radius_thickness_bias_falloff.x / meters_per_pixel,
        1.0,
        params.intensity_and_limits.z,
    );
    let screen_right = normalize_or(world_right, vec3<f32>(1.0, 0.0, 0.0));
    let screen_down = normalize_or(world_down, vec3<f32>(0.0, 1.0, 0.0));

    let slice_count = clamp(params.extent_and_sample_counts.z, 1u, MAX_SLICE_COUNT);
    let samples_per_side = clamp(
        params.extent_and_sample_counts.w,
        1u,
        MAX_SAMPLES_PER_SLICE_SIDE,
    );
    let sectors_per_slice = samples_per_side * 2u;
    let noise = pixel_noise(invocation_id.xy);
    var occluded_sector_count = 0u;

    for (var slice_index = 0u; slice_index < MAX_SLICE_COUNT; slice_index += 1u) {
        if (slice_index >= slice_count) {
            break;
        }
        let phi = PI * (f32(slice_index) + noise.x) / f32(slice_count);
        let omega = vec2<f32>(cos(phi), sin(phi));
        let slice_direction = normalize_or(
            screen_right * omega.x + screen_down * omega.y,
            screen_right,
        );
        let slice_axis = normalize_or(cross(slice_direction, view_vector), world_normal);
        let projected_normal = world_normal - slice_axis * dot(world_normal, slice_axis);
        let projected_normal_length = length(projected_normal);
        if (projected_normal_length <= EPSILON) {
            continue;
        }
        let orthographic_direction = slice_direction
            - view_vector * dot(slice_direction, view_vector);
        let normal_sign = sign(dot(orthographic_direction, projected_normal));
        let normal_cosine = clamp(
            dot(projected_normal, view_vector) / projected_normal_length,
            -1.0,
            1.0,
        );
        let normal_horizon = (HALF_PI - normal_sign * acos(normal_cosine)) / PI;
        var bitmask = 0u;

        for (var sample_index = 0u; sample_index < MAX_SAMPLES_PER_SLICE_SIDE; sample_index += 1u) {
            if (sample_index >= samples_per_side) {
                break;
            }
            let sequence = f32(slice_index + sample_index * samples_per_side) * 0.6180339887498948482;
            let sample_noise = fract(noise.y + sequence);
            var normalized_offset = (f32(sample_index) + sample_noise) / f32(samples_per_side);
            normalized_offset *= normalized_offset;
            let sample_offset_pixels = max(normalized_offset * radius_pixels, 1.0);
            let sample_offset_uv = omega * sample_offset_pixels * one_work_pixel_uv;
            let requested_mip = u32(max(floor(log2(sample_offset_pixels)) - 3.0, 0.0));
            let positive_position = load_hzb_world_position(uv + sample_offset_uv, requested_mip);
            let negative_position = load_hzb_world_position(uv - sample_offset_uv, requested_mip);

            bitmask = process_sample(
                positive_position - center_position,
                view_vector,
                -1.0,
                normal_horizon,
                params.world_radius_thickness_bias_falloff.y,
                sectors_per_slice,
                bitmask,
            );
            bitmask = process_sample(
                negative_position - center_position,
                view_vector,
                1.0,
                normal_horizon,
                params.world_radius_thickness_bias_falloff.y,
                sectors_per_slice,
                bitmask,
            );
        }
        occluded_sector_count += countOneBits(bitmask);
    }

    let total_sector_count = f32(slice_count * sectors_per_slice);
    let raw_visibility = 1.0 - f32(occluded_sector_count) / max(total_sector_count, 1.0);
    let minimum_visibility = params.intensity_and_limits.w;
    let powered_visibility = mix(1.0, raw_visibility, params.intensity_and_limits.x);
    let visibility = clamp(powered_visibility, minimum_visibility, 1.0);
    textureStore(ao_out, work_coord, vec4<f32>(visibility, visibility, visibility, 1.0));
}
