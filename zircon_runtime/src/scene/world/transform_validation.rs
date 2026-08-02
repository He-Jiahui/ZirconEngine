use crate::core::math::{Real, Transform};
use crate::scene::EntityId;

use super::{SceneError, SceneResult, World};

pub(super) fn validate_transform_for_write(
    entity: EntityId,
    transform: Transform,
) -> SceneResult<()> {
    validate_finite_transform_components(entity, "translation", transform.translation.to_array())?;

    let rotation = transform.rotation.to_array();
    validate_finite_transform_components(entity, "rotation", rotation)?;
    let rotation_length_squared = rotation
        .iter()
        .map(|component| component * component)
        .sum::<Real>();
    if rotation_length_squared <= Real::EPSILON {
        return Err(SceneError::ZeroLengthQuaternion {
            property_path: transform_property_path(entity, "rotation"),
        });
    }

    let scale = transform.scale.to_array();
    validate_finite_transform_components(entity, "scale", scale)?;
    for (axis, component) in ["x", "y", "z"].into_iter().zip(scale) {
        if component == 0.0 {
            return Err(SceneError::ZeroScaleTransform { entity, axis });
        }
    }

    Ok(())
}

pub(super) fn validate_persisted_transforms(world: &World) -> SceneResult<()> {
    for (&entity, local_transform) in &world.local_transforms {
        validate_transform_for_write(entity, local_transform.transform)?;
    }
    Ok(())
}

fn validate_finite_transform_components(
    entity: EntityId,
    field: &'static str,
    components: impl IntoIterator<Item = Real>,
) -> SceneResult<()> {
    if components
        .into_iter()
        .any(|component| !component.is_finite())
    {
        return Err(SceneError::NonFinitePropertyValue {
            property_path: transform_property_path(entity, field),
            expected: match field {
                "translation" => "translation",
                "rotation" => "quaternion",
                "scale" => "scale",
                _ => unreachable!("transform validation only accepts known fields"),
            },
        });
    }
    Ok(())
}

fn transform_property_path(entity: EntityId, field: &str) -> String {
    format!("entities[{entity}].transform.{field}")
}
