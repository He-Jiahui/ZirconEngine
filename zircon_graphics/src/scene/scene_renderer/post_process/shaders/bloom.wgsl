struct BloomParams {
    viewport: vec4<u32>,
    tuning: vec4<f32>,
};

@group(0) @binding(0) var scene_color_tex: texture_2d<f32>;
@group(0) @binding(1) var<uniform> params: BloomParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
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

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = params.viewport.xy;
    let coord = min(vec2<u32>(position.xy), viewport_size - vec2<u32>(1u, 1u));
    let center = vec2<i32>(coord);
    let stride = max(i32(round(params.tuning.z * 4.0)), 1);
    var bloom = vec3<f32>(0.0);
    var total_weight = 0.0;

    for (var y = -2; y <= 2; y = y + 1) {
        for (var x = -2; x <= 2; x = x + 1) {
            let sample_coord = clamp(
                center + vec2<i32>(x * stride, y * stride),
                vec2<i32>(0, 0),
                vec2<i32>(viewport_size) - vec2<i32>(1, 1)
            );
            let sample_color = textureLoad(scene_color_tex, sample_coord, 0).rgb;
            let bright = max(luminance(sample_color) - params.tuning.x, 0.0);
            let weight = 1.0 / (1.0 + f32(x * x + y * y));
            bloom += sample_color * bright * weight;
            total_weight += weight;
        }
    }

    if (total_weight > 0.0) {
        bloom = bloom / total_weight;
    }

    return vec4<f32>(bloom * params.tuning.y, 1.0);
}
