const ZR_MORPH_DELTA_ROWS_PER_VERTEX_TARGET: u32 = 4u;
const ZR_MORPH_POSITION_ROW: u32 = 0u;
const ZR_MORPH_NORMAL_ROW: u32 = 1u;
const ZR_MORPH_TANGENT_ROW: u32 = 2u;
const ZR_MORPH_COLOR_ROW: u32 = 3u;
const ZR_MORPH_INVALID_ROW: u32 = 0xffffffffu;

fn zr_morph_payload_for_instance(instance_index: u32) -> vec4<u32> {
    let instance = zr_gpu_scene_instance(instance_index);
    return zr_gpu_scene_morph_payload(instance.morph_payload_slot);
}

fn zr_morph_payload_has_vertex(payload: vec4<u32>, vertex_index: u32) -> bool {
    return payload.z > 0u && payload.w > 0u && vertex_index < payload.z;
}

fn zr_morph_delta_row_index(
    payload: vec4<u32>,
    vertex_index: u32,
    target_index: u32,
    row_offset: u32,
) -> u32 {
    if (!zr_morph_payload_has_vertex(payload, vertex_index) || target_index >= payload.w) {
        return ZR_MORPH_INVALID_ROW;
    }
    let target_vertex = target_index * payload.z + vertex_index;
    return payload.x + target_vertex * ZR_MORPH_DELTA_ROWS_PER_VERTEX_TARGET + row_offset;
}

fn zr_morph_delta_row(
    payload: vec4<u32>,
    vertex_index: u32,
    target_index: u32,
    row_offset: u32,
) -> vec4<f32> {
    let row_index = zr_morph_delta_row_index(payload, vertex_index, target_index, row_offset);
    if (row_index == ZR_MORPH_INVALID_ROW) {
        return vec4<f32>(0.0);
    }
    return zr_gpu_scene_morph_delta_row(row_index);
}

fn zr_morph_weight(payload: vec4<u32>, target_index: u32) -> f32 {
    if (target_index >= payload.w) {
        return 0.0;
    }
    return zr_gpu_scene_morph_weight(payload.y + target_index);
}

fn zr_morph_previous_weight(payload: vec4<u32>, target_index: u32) -> f32 {
    if (target_index >= payload.w) {
        return 0.0;
    }
    return zr_gpu_scene_morph_weight(payload.y + payload.w + target_index);
}

fn zr_morph_vec3_delta_with_previous_mode(
    v: ZrVertexInput,
    instance_index: u32,
    row_offset: u32,
    use_previous_weights: bool,
) -> vec4<f32> {
    let payload = zr_morph_payload_for_instance(instance_index);
    var delta = vec3<f32>(0.0);
    var used = false;
    var target_index = 0u;
    loop {
        if (target_index >= payload.w) {
            break;
        }
        let row = zr_morph_delta_row(payload, v.vertex_index, target_index, row_offset);
        let weight = select(
            zr_morph_weight(payload, target_index),
            zr_morph_previous_weight(payload, target_index),
            use_previous_weights,
        );
        delta += row.xyz * weight;
        used = used || row.w > 0.5;
        target_index += 1u;
    }
    return vec4<f32>(delta, select(0.0, 1.0, used));
}

fn zr_morph_vec3_delta(v: ZrVertexInput, instance_index: u32, row_offset: u32) -> vec4<f32> {
    return zr_morph_vec3_delta_with_previous_mode(v, instance_index, row_offset, false);
}

fn zr_morph_position_delta(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    return zr_morph_vec3_delta(v, instance_index, ZR_MORPH_POSITION_ROW).xyz;
}

fn zr_morph_previous_position_delta(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    return zr_morph_vec3_delta_with_previous_mode(v, instance_index, ZR_MORPH_POSITION_ROW, true).xyz;
}

fn zr_morph_color_delta(v: ZrVertexInput, instance_index: u32) -> vec4<f32> {
    let payload = zr_morph_payload_for_instance(instance_index);
    var delta = vec4<f32>(0.0);
    var target_index = 0u;
    loop {
        if (target_index >= payload.w) {
            break;
        }
        let row = zr_morph_delta_row(payload, v.vertex_index, target_index, ZR_MORPH_COLOR_ROW);
        delta += row * zr_morph_weight(payload, target_index);
        target_index += 1u;
    }
    return delta;
}

fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    return v.position + zr_morph_position_delta(v, instance_index);
}

fn fetch_prev_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    return v.position + zr_morph_previous_position_delta(v, instance_index);
}

fn fetch_normal(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    let delta = zr_morph_vec3_delta(v, instance_index, ZR_MORPH_NORMAL_ROW);
    if (delta.w <= 0.5) {
        return v.normal;
    }
    return zr_normalize_or_zero(v.normal + delta.xyz);
}

fn fetch_tangent(v: ZrVertexInput, instance_index: u32) -> vec4<f32> {
    let delta = zr_morph_vec3_delta(v, instance_index, ZR_MORPH_TANGENT_ROW);
    if (delta.w <= 0.5) {
        return v.tangent;
    }
    return vec4<f32>(zr_normalize_or_zero(v.tangent.xyz + delta.xyz), v.tangent.w);
}

fn fetch_uv0(v: ZrVertexInput) -> vec2<f32> {
    return v.uv0;
}

fn fetch_uv1(v: ZrVertexInput) -> vec2<f32> {
    return v.uv1;
}

fn fetch_color(v: ZrVertexInput, instance_index: u32) -> vec4<f32> {
    return v.color + zr_morph_color_delta(v, instance_index);
}
