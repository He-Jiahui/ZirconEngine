use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::math::{EulerRot, Quat, Transform};

use super::{InspectorEditError, InspectorTransformField};

pub(super) fn rotation_axis(field: &str) -> Option<usize> {
    match field {
        "transform.rotation_degrees.x" => Some(0),
        "transform.rotation_degrees.y" => Some(1),
        "transform.rotation_degrees.z" => Some(2),
        _ => None,
    }
}

/// Numeric fields use the same XYZ Euler convention as the displayed inspector.
/// Only authored axes change; scene storage and undo retain a normalized quaternion.
pub(super) fn rotation_update<'a>(
    scene: &Scene,
    node: NodeId,
    changes: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<Option<Transform>, InspectorEditError> {
    let invalid = || InspectorEditError::InvalidTransformFields {
        field: InspectorTransformField::Rotation,
    };
    let mut axes = [None; 3];
    for (field, value) in changes {
        if let Some(axis) = rotation_axis(field) {
            let degrees = value.trim().parse::<f32>().map_err(|_| invalid())?;
            if !degrees.is_finite() {
                return Err(invalid());
            }
            axes[axis] = Some(degrees.to_radians());
        }
    }
    if axes.iter().all(Option::is_none) {
        return Ok(None);
    }
    let mut transform = scene.local_transform(node).ok_or_else(invalid)?;
    let current = transform.rotation;
    if !current.is_finite() || current.length_squared() <= 0.0 {
        return Err(invalid());
    }
    let (x, y, z) = current.normalize().to_euler(EulerRot::XYZ);
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        axes[0].unwrap_or(x),
        axes[1].unwrap_or(y),
        axes[2].unwrap_or(z),
    )
    .normalize();
    transform.rotation = rotation;
    Ok(Some(transform))
}
