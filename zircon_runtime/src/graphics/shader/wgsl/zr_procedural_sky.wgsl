fn zr_procedural_sky_radiance(
    normalized_direction: vec3<f32>,
    horizon_color: vec3<f32>,
    zenith_color: vec3<f32>,
    ground_color: vec3<f32>,
    sun_direction: vec4<f32>,
    sun_color: vec3<f32>,
    sun_params: vec4<f32>,
) -> vec3<f32> {
    let sky_t = clamp(normalized_direction.y * 0.5 + 0.5, 0.0, 1.0);
    let ground_t = clamp(normalized_direction.y + 1.0, 0.0, 1.0);
    let sky = mix(horizon_color, zenith_color, sky_t);
    let ground = mix(ground_color, horizon_color, ground_t);
    var color = select(ground, sky, normalized_direction.y >= 0.0);
    if (sun_direction.w >= 0.5 && sun_params.x > 0.0) {
        let sun_mask = smoothstep(
            sun_params.y,
            sun_params.z,
            dot(normalized_direction, sun_direction.xyz),
        );
        color += sun_color * sun_params.x * sun_mask;
    }
    return color;
}
