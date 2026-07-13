fn shade_deferred_subsurface(
    position: vec4<f32>,
    coord: vec2<i32>,
    albedo: vec4<f32>,
    material: vec4<f32>,
    normal: vec3<f32>,
) -> vec4<f32> {
    // The pass-disabled fallback must stay byte-equivalent to StandardPBR.
    return shade_deferred_lit(position, coord, albedo, material, normal, 2u);
}
