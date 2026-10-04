// 主/次空间放大共用这个 shader，分别使用自己的参数 buffer，避免同帧两个阶段互相覆盖尺寸。
// 逻辑输入/输出尺寸决定缩放；纹理实际尺寸只用于采样坐标归一化。
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

// 由对应 ViewFamily phase 的有效 viewport 尺寸生成，不能用带对齐尾部的 allocation 尺寸代替。
struct UpscaleParams {
    input_output_size: vec4<u32>,
};

@group(0) @binding(2) var<uniform> params: UpscaleParams;

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
    let position = positions[vertex_index];
    output.clip_position = vec4<f32>(position, 0.0, 1.0);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let input_size = vec2<f32>(params.input_output_size.xy);
    let output_size = vec2<f32>(params.input_output_size.zw);
    let source_allocation_size = vec2<f32>(textureDimensions(source_tex));
    // Map destination pixel centers to source pixel centers. The allocation can contain aligned
    // tail texels, so only the logical sizes participate in the scale and the allocation size is
    // used for the final normalized coordinate.
    // BUG: [CR-POST-SHADER-0005] 放大时若源 allocation 含尾部，末端双线性采样会混入无效 texel；
    // 证据：逻辑输入 2、allocation 8、输出 4 的最后片元映到 source_pixel=2，跨越有效末端中心 1.5。
    // Rust 只将 sampler 限制到 allocation 边缘，尚未限制到逻辑边缘。
    let source_pixel = (input.clip_position.xy - vec2<f32>(0.5)) * (input_size / output_size)
        + vec2<f32>(0.5);
    let source_uv = source_pixel / source_allocation_size;
    return textureSampleLevel(source_tex, source_sampler, source_uv, 0.0);
}
