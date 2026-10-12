// 把已完成显示映射与终端效果的局部颜色写入最终物理 viewport。
// 本 pass 不重新做曝光或 tonemap；片元输出的目标编码还受 Rust 所选 attachment format 约束。
@group(0) @binding(0) var tonemapped_tex: texture_2d<f32>;

// Rust 上传最终物理 viewport 的原点，并设置同一物理 render region；
// 源纹理从零开始，所以必须先移除目标原点再读取。
struct TerminalRegionParams {
    viewport_origin: vec4<u32>,
};

@group(0) @binding(1) var<uniform> params: TerminalRegionParams;

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

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let dimensions = textureDimensions(tonemapped_tex);
    let max_coord = vec2<i32>(dimensions - vec2<u32>(1u, 1u));
    let coord = clamp(
        vec2<i32>(position.xy) - vec2<i32>(params.viewport_origin.xy),
        vec2<i32>(0, 0),
        max_coord
    );
    return textureLoad(tonemapped_tex, coord, 0);
}
