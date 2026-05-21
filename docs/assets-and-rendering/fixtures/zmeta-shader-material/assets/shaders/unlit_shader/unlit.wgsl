struct VertexOut {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
};

@group(1) @binding(0) var<uniform> base_color: vec4f;
@group(1) @binding(1) var base_color_texture: texture_2d<f32>;
@group(1) @binding(2) var base_color_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    var positions = array<vec2f, 3>(
        vec2f(-0.8, -0.8),
        vec2f(0.8, -0.8),
        vec2f(0.0, 0.8),
    );
    var uvs = array<vec2f, 3>(
        vec2f(0.0, 0.0),
        vec2f(1.0, 0.0),
        vec2f(0.5, 1.0),
    );

    var out: VertexOut;
    out.clip_position = vec4f(positions[vertex_index], 0.0, 1.0);
    out.uv = uvs[vertex_index];
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    let sampled_base_color = textureSample(base_color_texture, base_color_sampler, in.uv);
    return sampled_base_color * base_color;
}
