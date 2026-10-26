use zircon_runtime_interface::math::{EulerRot, Quat};

/// The inspector and its write path share the engine's canonical local XYZ decomposition.
/// At gimbal lock this selects glam's canonical solution; winding is not stored in a quaternion.
pub(crate) fn local_xyz_degrees(rotation: Quat) -> Option<[f32; 3]> {
    let length_squared = rotation.length_squared();
    if !rotation.is_finite() || !length_squared.is_finite() || length_squared <= 0.0 {
        return None;
    }
    let (x, y, z) = rotation.normalize().to_euler(EulerRot::XYZ);
    let degrees = [x, y, z].map(f32::to_degrees);
    degrees
        .iter()
        .all(|value| value.is_finite())
        .then_some(degrees)
}

pub(crate) fn with_local_xyz_degrees(rotation: Quat, updates: [Option<f32>; 3]) -> Option<Quat> {
    let mut degrees = local_xyz_degrees(rotation)?;
    let mut changed = false;
    for (current, update) in degrees.iter_mut().zip(updates) {
        let Some(update) = update else { continue };
        if !update.is_finite() {
            return None;
        }
        // Focus/blur submits the two-decimal display. Never round the stored quaternion or
        // an untouched axis just because that display (possibly plus whole turns) was submitted.
        let displayed = format!("{current:.2}").parse::<f32>().ok()?;
        // A signed remainder retains tiny negative angles that adding 360 would round away.
        let update = update % 360.0;
        if (update - displayed) % 360.0 == 0.0 || (update - *current) % 360.0 == 0.0 {
            continue;
        }
        *current = update;
        changed = true;
    }
    if !changed {
        return Some(rotation);
    }
    let [x, y, z] = degrees.map(f32::to_radians);
    let result = Quat::from_euler(EulerRot::XYZ, x, y, z).normalize();
    result.is_finite().then_some(result)
}

#[cfg(test)]
#[path = "../../tests/editing/euler_rotation.rs"]
mod tests;
