fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    _ = instance_index;
    return v.position;
}

fn fetch_prev_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    _ = instance_index;
    return v.position;
}

fn fetch_normal(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    _ = instance_index;
    return v.normal;
}

fn fetch_tangent(v: ZrVertexInput, instance_index: u32) -> vec4<f32> {
    _ = instance_index;
    return v.tangent;
}

fn fetch_uv0(v: ZrVertexInput) -> vec2<f32> {
    return v.uv0;
}

fn fetch_uv1(v: ZrVertexInput) -> vec2<f32> {
    return v.uv1;
}

fn fetch_color(v: ZrVertexInput) -> vec4<f32> {
    return v.color;
}
