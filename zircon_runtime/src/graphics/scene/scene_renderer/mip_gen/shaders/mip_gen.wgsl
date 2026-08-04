struct MipGenParams {
    source_extent: vec2<u32>,
    generated_mip_count: u32,
    is_srgb: u32,
    is_normal: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
};

@group(0) @binding(0) var<uniform> mipgen_params: MipGenParams;
@group(0) @binding(1) var source_mip: texture_2d_array<f32>;
@group(0) @binding(2) var target_mip_one: texture_storage_2d_array<rgba8unorm, write>;
@group(0) @binding(3) var target_mip_two: texture_storage_2d_array<rgba8unorm, write>;
@group(0) @binding(4) var target_mip_three: texture_storage_2d_array<rgba8unorm, write>;
@group(0) @binding(5) var target_mip_four: texture_storage_2d_array<rgba8unorm, write>;

var<workgroup> level_one: array<vec4<f32>, 64>;
var<workgroup> level_two: array<vec4<f32>, 16>;
var<workgroup> level_three: array<vec4<f32>, 4>;

fn srgb_to_linear(value: f32) -> f32 {
    if (value <= 0.04045) {
        return value / 12.92;
    }
    return pow((value + 0.055) / 1.055, 2.4);
}

fn linear_to_srgb(value: f32) -> f32 {
    if (value <= 0.0031308) {
        return value * 12.92;
    }
    return 1.055 * pow(value, 1.0 / 2.4) - 0.055;
}

fn decode_sample(sample: vec4<f32>) -> vec4<f32> {
    if (mipgen_params.is_srgb == 0u) {
        return sample;
    }
    return vec4<f32>(
        srgb_to_linear(sample.r),
        srgb_to_linear(sample.g),
        srgb_to_linear(sample.b),
        sample.a,
    );
}

fn encode_sample(sample: vec4<f32>) -> vec4<f32> {
    if (mipgen_params.is_srgb == 0u) {
        return sample;
    }
    return vec4<f32>(
        linear_to_srgb(sample.r),
        linear_to_srgb(sample.g),
        linear_to_srgb(sample.b),
        sample.a,
    );
}

fn reduce_normal(sample: vec4<f32>) -> vec4<f32> {
    let decoded = sample.xyz * 2.0 - vec3<f32>(1.0);
    let length_squared = dot(decoded, decoded);
    let normal = select(
        vec3<f32>(0.0, 0.0, 1.0),
        normalize(decoded),
        length_squared > 0.000001,
    );
    return vec4<f32>(normal * 0.5 + 0.5, sample.a);
}

fn reduce_four(
    first: vec4<f32>,
    second: vec4<f32>,
    third: vec4<f32>,
    fourth: vec4<f32>,
) -> vec4<f32> {
    let average = (first + second + third + fourth) * 0.25;
    if (mipgen_params.is_normal != 0u) {
        return reduce_normal(average);
    }
    return average;
}

fn source_texel(coord: vec2<u32>, layer: u32) -> vec4<f32> {
    let clamped_coord = min(coord, mipgen_params.source_extent - vec2<u32>(1u));
    return decode_sample(textureLoad(source_mip, vec2<i32>(clamped_coord), i32(layer), 0));
}

fn source_reduce(coord: vec2<u32>, layer: u32) -> vec4<f32> {
    let first_coord = coord * 2u;
    return reduce_four(
        source_texel(first_coord, layer),
        source_texel(first_coord + vec2<u32>(1u, 0u), layer),
        source_texel(first_coord + vec2<u32>(0u, 1u), layer),
        source_texel(first_coord + vec2<u32>(1u, 1u), layer),
    );
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    let layer = workgroup_id.z;
    let first_coord = workgroup_id.xy * vec2<u32>(8u, 8u) + local_id.xy;
    let first_index = local_id.y * 8u + local_id.x;
    if (all(first_coord < textureDimensions(target_mip_one))) {
        level_one[first_index] = source_reduce(first_coord, layer);
        textureStore(target_mip_one, vec2<i32>(first_coord), i32(layer), encode_sample(level_one[first_index]));
    } else {
        level_one[first_index] = vec4<f32>(0.0);
    }
    workgroupBarrier();

    if (mipgen_params.generated_mip_count >= 2u && all(local_id.xy < vec2<u32>(4u, 4u))) {
        let second_coord = workgroup_id.xy * vec2<u32>(4u, 4u) + local_id.xy;
        let second_index = local_id.y * 4u + local_id.x;
        let first_origin = local_id.xy * 2u;
        let level_one_extent = min(
            vec2<u32>(8u, 8u),
            textureDimensions(target_mip_one) - workgroup_id.xy * vec2<u32>(8u, 8u),
        );
        let first_max = level_one_extent - vec2<u32>(1u);
        let first_top_left = min(first_origin, first_max);
        let first_top_right = min(first_origin + vec2<u32>(1u, 0u), first_max);
        let first_bottom_left = min(first_origin + vec2<u32>(0u, 1u), first_max);
        let first_bottom_right = min(first_origin + vec2<u32>(1u, 1u), first_max);
        level_two[second_index] = reduce_four(
            level_one[first_top_left.y * 8u + first_top_left.x],
            level_one[first_top_right.y * 8u + first_top_right.x],
            level_one[first_bottom_left.y * 8u + first_bottom_left.x],
            level_one[first_bottom_right.y * 8u + first_bottom_right.x],
        );
        if (all(second_coord < textureDimensions(target_mip_two))) {
            textureStore(target_mip_two, vec2<i32>(second_coord), i32(layer), encode_sample(level_two[second_index]));
        }
    }
    workgroupBarrier();

    if (mipgen_params.generated_mip_count >= 3u && all(local_id.xy < vec2<u32>(2u, 2u))) {
        let third_coord = workgroup_id.xy * vec2<u32>(2u, 2u) + local_id.xy;
        let third_index = local_id.y * 2u + local_id.x;
        let second_origin = local_id.xy * 2u;
        let level_two_extent = min(
            vec2<u32>(4u, 4u),
            textureDimensions(target_mip_two) - workgroup_id.xy * vec2<u32>(4u, 4u),
        );
        let second_max = level_two_extent - vec2<u32>(1u);
        let second_top_left = min(second_origin, second_max);
        let second_top_right = min(second_origin + vec2<u32>(1u, 0u), second_max);
        let second_bottom_left = min(second_origin + vec2<u32>(0u, 1u), second_max);
        let second_bottom_right = min(second_origin + vec2<u32>(1u, 1u), second_max);
        level_three[third_index] = reduce_four(
            level_two[second_top_left.y * 4u + second_top_left.x],
            level_two[second_top_right.y * 4u + second_top_right.x],
            level_two[second_bottom_left.y * 4u + second_bottom_left.x],
            level_two[second_bottom_right.y * 4u + second_bottom_right.x],
        );
        if (all(third_coord < textureDimensions(target_mip_three))) {
            textureStore(target_mip_three, vec2<i32>(third_coord), i32(layer), encode_sample(level_three[third_index]));
        }
    }
    workgroupBarrier();

    if (mipgen_params.generated_mip_count >= 4u && local_id.x == 0u && local_id.y == 0u) {
        let fourth_coord = workgroup_id.xy;
        let level_three_extent = min(
            vec2<u32>(2u, 2u),
            textureDimensions(target_mip_three) - workgroup_id.xy * vec2<u32>(2u, 2u),
        );
        let third_max = level_three_extent - vec2<u32>(1u);
        let fourth = reduce_four(
            level_three[0],
            level_three[third_max.x],
            level_three[third_max.y * 2u],
            level_three[third_max.y * 2u + third_max.x],
        );
        if (all(fourth_coord < textureDimensions(target_mip_four))) {
            textureStore(target_mip_four, vec2<i32>(fourth_coord), i32(layer), encode_sample(fourth));
        }
    }
}
