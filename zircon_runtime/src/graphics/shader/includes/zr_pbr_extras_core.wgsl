const ZR_PBR_EXTRAS_PI: f32 = 3.141592653589793;
const ZR_PBR_EXTRAS_EPSILON: f32 = 0.000001;

fn zr_pbr_normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    return zr_pbr_common_normalize_or_zero(value);
}

fn zr_pbr_smith_joint_visibility_approx(no_v: f32, no_l: f32, alpha: f32) -> f32 {
    let visibility_v = no_l * (no_v * (1.0 - alpha) + alpha);
    let visibility_l = no_v * (no_l * (1.0 - alpha) + alpha);
    return 0.5 / max(visibility_v + visibility_l, ZR_PBR_EXTRAS_EPSILON);
}

fn zr_pbr_isotropic_ggx(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    perceptual_roughness: f32,
    f0: vec3<f32>,
) -> vec3<f32> {
    let half_dir = zr_pbr_normalize_or_zero(view_dir + light_dir);
    let no_v = clamp(dot(normal, view_dir), ZR_PBR_EXTRAS_EPSILON, 1.0);
    let no_l = clamp(dot(normal, light_dir), ZR_PBR_EXTRAS_EPSILON, 1.0);
    let no_h = clamp(dot(normal, half_dir), 0.0, 1.0);
    let vo_h = clamp(dot(view_dir, half_dir), 0.0, 1.0);
    let alpha = max(perceptual_roughness * perceptual_roughness, 0.001);
    let alpha_squared = alpha * alpha;
    let denominator = no_h * no_h * (alpha_squared - 1.0) + 1.0;
    let distribution = alpha_squared / (ZR_PBR_EXTRAS_PI * denominator * denominator);
    let visibility = zr_pbr_smith_joint_visibility_approx(no_v, no_l, alpha);
    let fresnel = zr_pbr_fresnel_schlick(vo_h, f0);
    return fresnel * distribution * visibility;
}
