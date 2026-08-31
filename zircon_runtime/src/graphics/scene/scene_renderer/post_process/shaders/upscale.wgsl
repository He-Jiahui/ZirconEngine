@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

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
    let source_pixel = (input.clip_position.xy - vec2<f32>(0.5)) * (input_size / output_size)
        + vec2<f32>(0.5);
    let source_uv = source_pixel / source_allocation_size;
    return textureSampleLevel(source_tex, source_sampler, source_uv, 0.0);
}
