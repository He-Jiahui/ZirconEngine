fn zr_decode_normal_xy(encoded_xy: vec2<f32>) -> vec2<f32> {
    return encoded_xy * 2.0 - vec2<f32>(1.0);
}

fn zr_reconstruct_bc5_normal(encoded_xy: vec2<f32>) -> vec3<f32> {
    let normal_xy = zr_decode_normal_xy(encoded_xy);
    let normal_z = sqrt(max(0.0, 1.0 - dot(normal_xy, normal_xy)));
    return normalize(vec3<f32>(normal_xy, normal_z));
}
