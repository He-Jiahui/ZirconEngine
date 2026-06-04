use zircon_runtime::core::math::{Real, Vec3};

pub(super) fn ray_sphere_quadratic_distances(
    origin: Vec3,
    direction: Vec3,
    center: Vec3,
    radius: Real,
) -> Option<[f64; 2]> {
    let offset_x = f64::from(origin.x) - f64::from(center.x);
    let offset_y = f64::from(origin.y) - f64::from(center.y);
    let offset_z = f64::from(origin.z) - f64::from(center.z);
    let direction_x = f64::from(direction.x);
    let direction_y = f64::from(direction.y);
    let direction_z = f64::from(direction.z);
    let a = direction_x * direction_x + direction_y * direction_y + direction_z * direction_z;
    let b = 2.0 * (offset_x * direction_x + offset_y * direction_y + offset_z * direction_z);
    let c =
        offset_x * offset_x + offset_y * offset_y + offset_z * offset_z - f64::from(radius).powi(2);
    ray_quadratic_distances(a, b, c)
}

pub(super) fn ray_quadratic_distances(a: f64, b: f64, c: f64) -> Option<[f64; 2]> {
    if !a.is_finite() || a <= f64::EPSILON || !b.is_finite() || !c.is_finite() {
        return None;
    }
    let discriminant = b * b - 4.0 * a * c;
    if !discriminant.is_finite() || discriminant < 0.0 {
        return None;
    }
    let sqrt_discriminant = discriminant.sqrt();
    Some([
        (-b - sqrt_discriminant) / (2.0 * a),
        (-b + sqrt_discriminant) / (2.0 * a),
    ])
}

pub(super) fn ray_distance_to_real(distance: f64, max_distance: Real) -> Option<Real> {
    if !(0.0..=f64::from(max_distance)).contains(&distance) {
        return None;
    }
    let distance = distance as Real;
    distance.is_finite().then_some(distance)
}
