// Bloom pass 从场景线性 HDR 提取亮部，写入独立的局部目标，再由后处理合成。
// 调用方负责在效果关闭时清黑；本 pass 不做显示映射，也不直接改写场景颜色。
// 与 execute_bloom 的 CPU 参数对应：xy 是有效局部尺寸，zw 是场景颜色源原点；
// 邻域限制在本视口内，避免分屏相机从相邻区域借入亮部。
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

// 输出已乘效果强度的亮部颜色；后续合成的权重须与 CPU 的效果调度约定一致。
@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = params.viewport.xy;
    let coord = min(vec2<u32>(position.xy), viewport_size - vec2<u32>(1u, 1u));
    let center = vec2<i32>(coord);
    let source_origin = vec2<i32>(params.viewport.zw);
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
            let sample_color = textureLoad(scene_color_tex, source_origin + sample_coord, 0).rgb;
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
