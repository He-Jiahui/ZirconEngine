use super::InspectorSnapshot;
use zircon_runtime_interface::math::{EulerRot, Quat};

impl InspectorSnapshot {
    // Engine local-transform XYZ Euler convention. Scene storage remains quaternion radians.
    pub(crate) fn rotation_degrees_from_quaternion(rotation: Quat) -> Option<[String; 3]> {
        let length_squared = rotation.length_squared();
        if !rotation.is_finite() || !length_squared.is_finite() || length_squared <= 0.0 {
            return None;
        }
        let (x, y, z) = rotation.normalize().to_euler(EulerRot::XYZ);
        Some([x, y, z].map(|radians| {
            let degrees = format!("{:.2}", radians.to_degrees());
            if degrees == "-0.00" {
                "0.00".to_string()
            } else {
                degrees
            }
        }))
    }
}

#[cfg(test)]
#[path = "tests/rotation.rs"]
mod tests;
