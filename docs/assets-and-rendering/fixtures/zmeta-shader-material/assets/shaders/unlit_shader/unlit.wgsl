struct VertexOut {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    var positions = array<vec2f, 3>(
        vec2f(-0.8, -0.8),
        vec2f(0.8, -0.8),
        vec2f(0.0, 0.8),
    );

    var out: VertexOut;
    out.clip_position = vec4f(positions[vertex_index], 0.0, 1.0);
    out.color = vec4f(1.0, 0.8, 0.2, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    return in.color;
}
