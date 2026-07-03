fn shade_deferred_standard_pbr(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>) -> vec4<f32> {
    return shade_deferred_lit(position, coord, albedo, material, normal, ZR_SHADING_MODEL_STANDARD_PBR_ID);
}
