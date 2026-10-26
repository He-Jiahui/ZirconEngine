use super::InspectorSnapshot;
use crate::core::editing::euler_rotation::local_xyz_degrees;
use zircon_runtime_interface::math::Quat;

impl InspectorSnapshot {
    // Engine local-transform XYZ Euler convention. Scene storage remains quaternion radians.
    pub(crate) fn rotation_degrees_from_quaternion(rotation: Quat) -> Option<[String; 3]> {
        Some(local_xyz_degrees(rotation)?.map(|value| {
            let degrees = format!("{value:.2}");
            if degrees == "-0.00" {
                "0.00".to_string()
            } else {
                degrees
            }
        }))
    }
}

#[cfg(test)]
#[path = "../tests/rotation.rs"]
mod tests;
