fn zr_template_identity_mat4() -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
}

fn zr_template_skin_weight(joint_index: u32, weight: f32, joint_count: u32) -> f32 {
    if (weight <= 0.000001 || joint_index >= joint_count) {
        return 0.0;
    }
    return weight;
}

fn zr_template_skin_weight_sum(v: ZrVertexInput, joint_count: u32) -> f32 {
    return zr_template_skin_weight(v.joints.x, v.weights.x, joint_count)
        + zr_template_skin_weight(v.joints.y, v.weights.y, joint_count)
        + zr_template_skin_weight(v.joints.z, v.weights.z, joint_count)
        + zr_template_skin_weight(v.joints.w, v.weights.w, joint_count);
}

fn zr_skin_matrix(v: ZrVertexInput) -> mat4x4<f32> {
    let joint_count = zr_skinned_joint_count();
    let weight_sum = zr_template_skin_weight_sum(v, joint_count);
    if (weight_sum <= 0.000001) {
        return zr_template_identity_mat4();
    }
    let inverse_weight_sum = 1.0 / weight_sum;
    return zr_skinned_joint_matrix(v.joints.x) * zr_template_skin_weight(v.joints.x, v.weights.x, joint_count) * inverse_weight_sum
        + zr_skinned_joint_matrix(v.joints.y) * zr_template_skin_weight(v.joints.y, v.weights.y, joint_count) * inverse_weight_sum
        + zr_skinned_joint_matrix(v.joints.z) * zr_template_skin_weight(v.joints.z, v.weights.z, joint_count) * inverse_weight_sum
        + zr_skinned_joint_matrix(v.joints.w) * zr_template_skin_weight(v.joints.w, v.weights.w, joint_count) * inverse_weight_sum;
}

fn zr_prev_skin_matrix(v: ZrVertexInput) -> mat4x4<f32> {
    let joint_count = zr_previous_skinned_joint_count();
    let weight_sum = zr_template_skin_weight_sum(v, joint_count);
    if (weight_sum <= 0.000001) {
        return zr_template_identity_mat4();
    }
    let inverse_weight_sum = 1.0 / weight_sum;
    return zr_previous_skinned_joint_matrix(v.joints.x) * zr_template_skin_weight(v.joints.x, v.weights.x, joint_count) * inverse_weight_sum
        + zr_previous_skinned_joint_matrix(v.joints.y) * zr_template_skin_weight(v.joints.y, v.weights.y, joint_count) * inverse_weight_sum
        + zr_previous_skinned_joint_matrix(v.joints.z) * zr_template_skin_weight(v.joints.z, v.weights.z, joint_count) * inverse_weight_sum
        + zr_previous_skinned_joint_matrix(v.joints.w) * zr_template_skin_weight(v.joints.w, v.weights.w, joint_count) * inverse_weight_sum;
}

fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    _ = instance_index;
    return (zr_skin_matrix(v) * vec4<f32>(v.position, 1.0)).xyz;
}

fn fetch_prev_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    _ = instance_index;
    return (zr_prev_skin_matrix(v) * vec4<f32>(v.position, 1.0)).xyz;
}

fn fetch_normal(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    _ = instance_index;
    return zr_normalize_or_zero((zr_skin_matrix(v) * vec4<f32>(v.normal, 0.0)).xyz);
}

fn fetch_tangent(v: ZrVertexInput, instance_index: u32) -> vec4<f32> {
    _ = instance_index;
    return vec4<f32>(zr_normalize_or_zero((zr_skin_matrix(v) * vec4<f32>(v.tangent.xyz, 0.0)).xyz), v.tangent.w);
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
