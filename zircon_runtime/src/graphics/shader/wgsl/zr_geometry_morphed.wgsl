@group(3) @binding(7) var<storage, read> zr_morph_deltas: array<vec4<f32>>;
@group(3) @binding(8) var<storage, read> zr_morph_weights: array<f32>;

fn zr_morph_position_delta(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    let delta_index = instance_index + v.joints.x;
    return zr_morph_deltas[delta_index].xyz * zr_morph_weights[delta_index];
}

fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    return v.position + zr_morph_position_delta(v, instance_index);
}

fn fetch_prev_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    return fetch_position(v, instance_index);
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
