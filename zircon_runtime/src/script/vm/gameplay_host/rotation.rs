//! World-space XYZ Euler writes reuse the existing validated TRS conversion path.
use crate::core::framework::script::{ScriptHostCallFrame, ScriptHostError, ScriptHostValue};
use crate::core::math::{EulerRot, Quat, Vec3};
use crate::script::runtime_context_for_frame;

use super::error::{GameplayHostError, GameplayHostResult};
use super::transform::update_world_transform;
use super::values::{expect_entity, expect_float};

pub(super) fn set_rotation_euler(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let entity = expect_entity(context, 0)?;
    let angles = Vec3::new(
        expect_float(context, 1)?,
        expect_float(context, 2)?,
        expect_float(context, 3)?,
    );
    if !angles.is_finite() {
        return Err(ScriptHostError::new(
            "Euler angles must be finite f32 radians",
        ));
    }
    let rotation = Quat::from_euler(EulerRot::XYZ, angles.x, angles.y, angles.z).normalize();
    let runtime = runtime_context_for_frame(context)?;
    let result = runtime
        .level
        .with_world_mut(|world| -> GameplayHostResult<bool> {
            let mut transform = world
                .world_transform(entity)
                .ok_or_else(|| GameplayHostError::missing_entity("set rotation for", entity))?;
            transform.rotation = rotation;
            update_world_transform(world, entity, transform)
        });
    result
        .map(ScriptHostValue::Bool)
        .map_err(ScriptHostError::from)
}
