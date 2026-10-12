fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    _ = instance_index;
    return v.position;
}

// velocity 调用者已将 previous_position 放入 v.position，并在 fetch 外应用上一帧 world/camera 矩阵。
// 此处转发对象空间位置，避免重复施加历史变换。
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

fn fetch_color(v: ZrVertexInput, instance_index: u32) -> vec4<f32> {
    _ = instance_index;
    return v.color;
}
