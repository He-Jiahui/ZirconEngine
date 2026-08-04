@group(0) @binding(0) var source_depth_tex: texture_depth_2d;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct DepthDownsampleOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn clamped_depth_coord(coord: vec2<i32>, dimensions: vec2<u32>) -> vec2<i32> {
    let maximum = vec2<i32>(dimensions) - vec2<i32>(1, 1);
    return clamp(coord, vec2<i32>(0, 0), maximum);
}

@fragment
fn fs_depth_downsample(@builtin(position) position: vec4<f32>) -> DepthDownsampleOutput {
    let dimensions = textureDimensions(source_depth_tex);
    let base = vec2<i32>(position.xy) * 2;
    let depth00 = textureLoad(source_depth_tex, clamped_depth_coord(base, dimensions), 0);
    let depth10 = textureLoad(source_depth_tex, clamped_depth_coord(base + vec2<i32>(1, 0), dimensions), 0);
    let depth01 = textureLoad(source_depth_tex, clamped_depth_coord(base + vec2<i32>(0, 1), dimensions), 0);
    let depth11 = textureLoad(source_depth_tex, clamped_depth_coord(base + vec2<i32>(1, 1), dimensions), 0);
    var output: DepthDownsampleOutput;
    output.color = vec4<f32>(0.0);
    output.depth = min(min(depth00, depth10), min(depth01, depth11));
    return output;
}

@group(0) @binding(1) var half_color_tex: texture_2d<f32>;
@group(0) @binding(2) var half_depth_tex: texture_depth_2d;
@group(0) @binding(3) var full_depth_tex: texture_depth_2d;

struct HalfResolutionTransparencyParams {
    depth_sigma: f32,
    _pad0: vec3<f32>,
};

@group(0) @binding(4) var<uniform> half_resolution_transparency_params: HalfResolutionTransparencyParams;

fn depth_weight(full_depth: f32, half_depth: f32) -> f32 {
    return exp(-abs(full_depth - half_depth) * half_resolution_transparency_params.depth_sigma);
}

@fragment
fn fs_composite(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let full_dimensions = textureDimensions(full_depth_tex);
    let half_dimensions = textureDimensions(half_color_tex);
    let full_coord = clamped_depth_coord(vec2<i32>(position.xy), full_dimensions);
    let half_base = clamped_depth_coord(vec2<i32>(position.xy) / 2, half_dimensions);
    let full_depth = textureLoad(full_depth_tex, full_coord, 0);
    var weighted_premultiplied = vec3<f32>(0.0);
    var weighted_alpha = 0.0;
    var weight_sum = 0.0;

    for (var offset_y = 0; offset_y < 2; offset_y += 1) {
        for (var offset_x = 0; offset_x < 2; offset_x += 1) {
            let coord = clamped_depth_coord(
                half_base + vec2<i32>(offset_x, offset_y),
                half_dimensions,
            );
            let sample = textureLoad(half_color_tex, coord, 0);
            let half_depth = textureLoad(half_depth_tex, coord, 0);
            let weight = depth_weight(full_depth, half_depth) * step(0.0001, sample.a);
            weighted_premultiplied += sample.rgb * weight;
            weighted_alpha += sample.a * weight;
            weight_sum += weight;
        }
    }

    if (weight_sum <= 0.0001) {
        return vec4<f32>(0.0);
    }
    let premultiplied = weighted_premultiplied / weight_sum;
    let alpha = weighted_alpha / weight_sum;
    return vec4<f32>(premultiplied / max(alpha, 0.0001), alpha);
}
