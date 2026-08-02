const ZR_PBR_EXTRAS_PI: f32 = 3.141592653589793;
const ZR_PBR_EXTRAS_EPSILON: f32 = 0.000001;

fn zr_pbr_fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    let grazing = pow(1.0 - clamp(cos_theta, 0.0, 1.0), 5.0);
    return f0 + (vec3<f32>(1.0) - f0) * grazing;
}

fn zr_pbr_smith_visibility(no_v: f32, no_l: f32, alpha: f32) -> f32 {
    let alpha_squared = alpha * alpha;
    let gv = no_l * sqrt(max(no_v * no_v * (1.0 - alpha_squared) + alpha_squared, 0.0));
    let gl = no_v * sqrt(max(no_l * no_l * (1.0 - alpha_squared) + alpha_squared, 0.0));
    return 0.5 / max(gv + gl, ZR_PBR_EXTRAS_EPSILON);
}

fn zr_pbr_isotropic_ggx(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    perceptual_roughness: f32,
    f0: vec3<f32>,
) -> vec3<f32> {
    let half_dir = zr_normalize_or_zero(view_dir + light_dir);
    let no_v = max(dot(normal, view_dir), ZR_PBR_EXTRAS_EPSILON);
    let no_l = max(dot(normal, light_dir), ZR_PBR_EXTRAS_EPSILON);
    let no_h = max(dot(normal, half_dir), 0.0);
    let vo_h = max(dot(view_dir, half_dir), 0.0);
    let alpha = max(perceptual_roughness * perceptual_roughness, 0.001);
    let alpha_squared = alpha * alpha;
    let denominator = no_h * no_h * (alpha_squared - 1.0) + 1.0;
    let distribution = alpha_squared / max(
        ZR_PBR_EXTRAS_PI * denominator * denominator,
        ZR_PBR_EXTRAS_EPSILON,
    );
    let visibility = zr_pbr_smith_visibility(no_v, no_l, alpha);
    return zr_pbr_fresnel_schlick(vo_h, f0) * distribution * visibility;
}
